//! User Repository (REQ-009)
//!
//! Login accounts and the Argon2id hashing that guards them. Hashing lives
//! here rather than in the handler so there is exactly one place a plaintext
//! password can reach the database, and it can't reach it unhashed.

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::user::{Role, User};

pub struct UserRepo;

impl UserRepo {
    /// Case-insensitive, matching the unique index from migration 026.
    pub async fn find_by_username(pool: &PgPool, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, role, created_at, updated_at
            FROM users
            WHERE LOWER(username) = LOWER($1)
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, password_hash, role, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    pub async fn count(pool: &PgPool) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await?;
        Ok(count)
    }

    pub async fn create(
        pool: &PgPool,
        username: &str,
        password: &str,
        role: Role,
    ) -> AppResult<User> {
        let hash = hash_password(password)?;
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, password_hash, role)
            VALUES ($1, $2, $3)
            RETURNING id, username, password_hash, role, created_at, updated_at
            "#,
        )
        .bind(username)
        .bind(hash)
        .bind(role.as_str())
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub async fn update_password(pool: &PgPool, id: Uuid, new_password: &str) -> AppResult<()> {
        let hash = hash_password(new_password)?;
        sqlx::query("UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(hash)
            .execute(pool)
            .await?;
        Ok(())
    }
}

/// Argon2id with a fresh random salt per call. The salt travels inside the
/// returned PHC string, so nothing else needs storing.
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(format!("Password hashing failed: {}", e)))
}

/// A malformed stored hash verifies as `false` rather than erroring: a corrupt
/// row should deny the login, not hand the caller a 500 that distinguishes it
/// from a wrong password.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    match PasswordHash::new(stored_hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_round_trips() {
        let hash = hash_password("admin1234!").unwrap();
        assert!(verify_password("admin1234!", &hash));
    }

    #[test]
    fn wrong_password_does_not_verify() {
        let hash = hash_password("admin1234!").unwrap();
        assert!(!verify_password("admin1234", &hash));
        assert!(!verify_password("", &hash));
    }

    #[test]
    fn same_password_hashes_differently_each_time() {
        // Distinct salts — two accounts sharing a password must not share a
        // hash, or the table leaks which users picked the same one.
        let a = hash_password("admin1234!").unwrap();
        let b = hash_password("admin1234!").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn corrupt_hash_denies_rather_than_panics() {
        assert!(!verify_password("admin1234!", "not-a-phc-string"));
        assert!(!verify_password("admin1234!", ""));
    }

    #[test]
    fn unknown_role_reads_as_viewer() {
        assert_eq!(Role::from_str_or_viewer("admin"), Role::Admin);
        assert_eq!(Role::from_str_or_viewer("viewer"), Role::Viewer);
        assert_eq!(Role::from_str_or_viewer("superuser"), Role::Viewer);
    }
}
