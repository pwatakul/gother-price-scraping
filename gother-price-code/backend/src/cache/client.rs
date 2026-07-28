//! Redis Client
//!
//! Creates and manages Redis connection.

use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::Client;
use serde::{de::DeserializeOwned, Serialize};

/// Create a new Redis connection manager
pub async fn create_client(redis_url: &str) -> Result<ConnectionManager> {
    let client = Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;
    Ok(manager)
}

/// Cache operations helper
pub struct CacheOps;

impl CacheOps {
    /// Get a value from cache
    pub async fn get<T: DeserializeOwned>(
        conn: &mut ConnectionManager,
        key: &str,
    ) -> Result<Option<T>> {
        let result: Option<String> = redis::cmd("GET").arg(key).query_async(conn).await?;

        match result {
            Some(data) => {
                let value: T = serde_json::from_str(&data)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a value in cache with expiration
    pub async fn set<T: Serialize>(
        conn: &mut ConnectionManager,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<()> {
        let data = serde_json::to_string(value)?;
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl_seconds)
            .arg(data)
            .query_async::<_, ()>(conn)
            .await?;
        Ok(())
    }

    /// Delete a key from cache
    pub async fn delete(conn: &mut ConnectionManager, key: &str) -> Result<()> {
        redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(conn)
            .await?;
        Ok(())
    }

    /// Check if key exists
    pub async fn exists(conn: &mut ConnectionManager, key: &str) -> Result<bool> {
        let result: i32 = redis::cmd("EXISTS").arg(key).query_async(conn).await?;
        Ok(result > 0)
    }

    /// Increment a counter (for rate limiting)
    pub async fn incr(conn: &mut ConnectionManager, key: &str, ttl_seconds: u64) -> Result<i64> {
        let count: i64 = redis::cmd("INCR").arg(key).query_async(conn).await?;

        // Set expiry if this is the first increment
        if count == 1 {
            redis::cmd("EXPIRE")
                .arg(key)
                .arg(ttl_seconds)
                .query_async::<_, ()>(conn)
                .await?;
        }

        Ok(count)
    }

    /// Get TTL of a key
    pub async fn ttl(conn: &mut ConnectionManager, key: &str) -> Result<i64> {
        let ttl: i64 = redis::cmd("TTL").arg(key).query_async(conn).await?;
        Ok(ttl)
    }
}
