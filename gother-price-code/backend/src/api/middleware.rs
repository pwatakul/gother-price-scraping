//! API Middleware
//!
//! Session authentication (REQ-009). Everything under `/api` passes through
//! `require_auth` except `/api/health` and `/api/auth/login`, which are built
//! on a separate un-layered router in `router.rs` — so a route added later is
//! protected by default rather than by remembering to protect it.

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::AppState;
use crate::error::{AppError, AppResult};
use crate::models::user::{Role, User};

/// Cookie carrying the session token.
pub const SESSION_COOKIE: &str = "gother_session";

/// How long a session lasts. There is no server-side revocation list, so this
/// is also the worst-case lifetime of a stolen cookie — short enough to matter,
/// long enough to cover a working day without a re-login.
pub const SESSION_HOURS: i64 = 12;

/// JWT payload. `sub` is the user id, so a renamed account keeps its session
/// and a deleted-then-recreated one does not.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

/// The authenticated caller, inserted as a request extension by `require_auth`
/// and readable by any handler that wants it.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
    pub role: Role,
}

pub fn issue_token(user: &User, secret: &str) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user.id.to_string(),
        username: user.username.clone(),
        role: user.role.clone(),
        iat: now.timestamp(),
        exp: (now + Duration::hours(SESSION_HOURS)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Could not issue session token: {}", e)))
}

/// Verifies signature *and* expiry (`jsonwebtoken` checks `exp` by default).
pub fn verify_token(token: &str, secret: &str) -> Option<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .ok()
    .map(|data| data.claims)
}

/// Rejects anything without a valid session cookie with a 401.
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or_else(|| AppError::Unauthorized("Not signed in".to_string()))?;

    let claims = verify_token(&token, &state.config.jwt_secret)
        .ok_or_else(|| AppError::Unauthorized("Session expired or invalid".to_string()))?;

    let id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Malformed session".to_string()))?;

    request.extensions_mut().insert(AuthUser {
        id,
        username: claims.username,
        role: Role::from_str_or_viewer(&claims.role),
    });

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_user() -> User {
        User {
            id: Uuid::new_v4(),
            username: "admin".to_string(),
            password_hash: String::new(),
            role: "admin".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    const SECRET: &str = "a-test-secret-long-enough-for-hs256";

    #[test]
    fn token_round_trips() {
        let user = sample_user();
        let token = issue_token(&user, SECRET).unwrap();
        let claims = verify_token(&token, SECRET).expect("should verify");
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.username, "admin");
        assert_eq!(claims.role, "admin");
    }

    #[test]
    fn token_signed_with_another_secret_is_rejected() {
        // The whole point of requiring JWT_SECRET: a token minted elsewhere
        // must not open a session here.
        let token = issue_token(&sample_user(), SECRET).unwrap();
        assert!(verify_token(&token, "a-different-secret-of-adequate-length").is_none());
    }

    #[test]
    fn tampered_token_is_rejected() {
        let token = issue_token(&sample_user(), SECRET).unwrap();
        let mut chars: Vec<char> = token.chars().collect();
        // Flip a character in the payload segment.
        let idx = token.find('.').unwrap() + 3;
        chars[idx] = if chars[idx] == 'a' { 'b' } else { 'a' };
        let tampered: String = chars.into_iter().collect();
        assert!(verify_token(&tampered, SECRET).is_none());
    }

    #[test]
    fn garbage_is_rejected() {
        assert!(verify_token("", SECRET).is_none());
        assert!(verify_token("not.a.token", SECRET).is_none());
    }
}
