//! Authentication Handlers (REQ-009)
//!
//! Login issues a signed JWT in an httpOnly cookie. httpOnly rather than a
//! token in localStorage because the frontend is served same-origin through
//! nginx, so a cookie needs no CORS work — and injected JavaScript can't read
//! it.

use axum::{extract::State, Extension, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use std::sync::Arc;
use tracing::{info, warn};

use crate::api::middleware::{issue_token, AuthUser, SESSION_COOKIE, SESSION_HOURS};
use crate::api::router::AppState;
use crate::db::repositories::user_repo::{verify_password, UserRepo};
use crate::error::{AppError, AppResult};
use crate::models::user::{
    ChangePasswordRequest, LoginRequest, UserResponse, DEFAULT_ADMIN_PASSWORD,
};

/// The same message for an unknown username and a wrong password. Different
/// wording would let anyone enumerate which accounts exist.
const BAD_CREDENTIALS: &str = "Invalid username or password";

/// Minimum length for a new password. Deliberately not a complexity ruleset —
/// length is the part that actually helps.
const MIN_PASSWORD_LEN: usize = 8;

fn session_cookie(token: String, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, token))
        .http_only(true)
        .same_site(SameSite::Strict)
        .path("/")
        .max_age(time::Duration::hours(SESSION_HOURS))
        // Driven by COOKIE_SECURE rather than hardcoded: production is served
        // over HTTPS and must set it, while local development is plain HTTP
        // where a Secure cookie would never be sent and login would silently
        // fail.
        .secure(secure)
        .build()
}

fn to_response(user: &crate::models::user::User) -> UserResponse {
    UserResponse {
        id: user.id,
        username: user.username.clone(),
        role: user.role.clone(),
        using_default_password: verify_password(DEFAULT_ADMIN_PASSWORD, &user.password_hash),
    }
}

/// POST /api/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<UserResponse>)> {
    let user = UserRepo::find_by_username(&state.db, body.username.trim()).await?;

    let user = match user {
        Some(u) if verify_password(&body.password, &u.password_hash) => u,
        _ => {
            warn!("Failed login attempt for username '{}'", body.username.trim());
            return Err(AppError::Unauthorized(BAD_CREDENTIALS.to_string()));
        }
    };

    let token = issue_token(&user, &state.config.jwt_secret)?;
    info!("User '{}' signed in", user.username);

    Ok((
        jar.add(session_cookie(token, state.config.cookie_secure)),
        Json(to_response(&user)),
    ))
}

/// POST /api/auth/logout
///
/// Clears the cookie. The token itself stays valid until it expires — there is
/// no revocation list — so this ends the session on this browser only.
pub async fn logout(jar: CookieJar) -> AppResult<(CookieJar, Json<serde_json::Value>)> {
    let mut removal = Cookie::from(SESSION_COOKIE);
    removal.set_path("/");
    Ok((
        jar.remove(removal),
        Json(serde_json::json!({ "status": "signed_out" })),
    ))
}

/// GET /api/auth/me — lets the SPA restore its session after a reload, since
/// an httpOnly cookie is invisible to JavaScript.
pub async fn me(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthUser>,
) -> AppResult<Json<UserResponse>> {
    let user = UserRepo::find_by_id(&state.db, auth.id)
        .await?
        // A valid token for a deleted account: treat as signed out, not 404.
        .ok_or_else(|| AppError::Unauthorized("Account no longer exists".to_string()))?;
    Ok(Json(to_response(&user)))
}

/// POST /api/auth/change-password
///
/// Requires the current password even though the caller is already
/// authenticated, so an unattended session can't be used to lock the owner out.
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    Extension(auth): Extension<AuthUser>,
    Json(body): Json<ChangePasswordRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = UserRepo::find_by_id(&state.db, auth.id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Account no longer exists".to_string()))?;

    if !verify_password(&body.current_password, &user.password_hash) {
        return Err(AppError::Unauthorized(
            "Current password is incorrect".to_string(),
        ));
    }

    if body.new_password.len() < MIN_PASSWORD_LEN {
        return Err(AppError::Validation(format!(
            "New password must be at least {} characters",
            MIN_PASSWORD_LEN
        )));
    }

    if body.new_password == body.current_password {
        return Err(AppError::Validation(
            "New password must be different from the current one".to_string(),
        ));
    }

    UserRepo::update_password(&state.db, user.id, &body.new_password).await?;
    info!("User '{}' changed their password", user.username);

    let updated = UserRepo::find_by_id(&state.db, user.id)
        .await?
        .ok_or_else(|| AppError::Internal("User vanished mid-update".to_string()))?;
    Ok(Json(to_response(&updated)))
}
