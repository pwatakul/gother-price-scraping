//! User Model (REQ-009)
//!
//! Login accounts. `role` is stored and returned to the client but is not yet
//! enforced anywhere — the column exists so restricting endpoints later is a
//! change in one middleware rather than a migration plus a backfill.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The password every fresh install starts with. Kept as a constant rather
/// than a literal so the seeding path and the "you are still on the default"
/// check can never drift apart.
pub const DEFAULT_ADMIN_USERNAME: &str = "admin";
pub const DEFAULT_ADMIN_PASSWORD: &str = "admin1234!";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Viewer,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Viewer => "viewer",
        }
    }

    /// Unknown values fall back to the least-privileged role. The DB has a
    /// CHECK constraint so this shouldn't happen, but a token minted before a
    /// future role was renamed shouldn't be read as `admin`.
    pub fn from_str_or_viewer(s: &str) -> Self {
        match s {
            "admin" => Role::Admin,
            _ => Role::Viewer,
        }
    }
}

/// A row of `users`, password hash included — never serialize this directly.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// What the API returns about the signed-in user.
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    /// True while this account still uses `DEFAULT_ADMIN_PASSWORD`. Surfaced
    /// so the UI can nag; otherwise the seeded credential quietly becomes
    /// permanent.
    pub using_default_password: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}
