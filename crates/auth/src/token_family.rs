//! Token Family Management for Refresh Token Rotation
//!
//! Implements refresh token rotation with family tracking to detect token theft.
//! When a refresh token is reused (indicating potential theft), all tokens
//! in the family are revoked.

use anyhow::{Context, Result};
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use std::collections::HashSet;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// Token family TTL (7 days - matches refresh token lifetime)
const FAMILY_TTL_SECS: u64 = 604800;

/// Token family information
#[derive(Debug, Clone)]
pub struct TokenFamily {
    pub family_id: Uuid,
    pub user_id: Uuid,
    pub active_jtis: HashSet<String>,
}

/// Token family manager for Redis-backed family tracking
pub struct TokenFamilyManager {
    client: Client,
}

impl TokenFamilyManager {
    /// Create new token family manager
    pub fn new(redis_url: &str) -> Result<Self> {
        let client =
            Client::open(redis_url).context("Failed to create Redis client for token family")?;
        Ok(Self { client })
    }

    /// Create from environment
    pub fn from_env() -> Result<Self> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self::new(&redis_url)
    }

    async fn get_conn(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis connection")
    }

    /// Create a new token family for a user
    #[instrument(skip(self))]
    pub async fn create_family(&self, user_id: Uuid) -> Result<Uuid> {
        let family_id = Uuid::new_v4();
        let mut conn = self.get_conn().await?;

        // Store family metadata
        let meta_key = format!("token_family:{}:meta", family_id);
        conn.hset::<_, _, _, ()>(&meta_key, "user_id", user_id.to_string())
            .await?;
        conn.expire::<_, ()>(&meta_key, FAMILY_TTL_SECS as i64)
            .await?;

        // Initialize empty JTI set
        let jtis_key = format!("token_family:{}:jtis", family_id);
        conn.expire::<_, ()>(&jtis_key, FAMILY_TTL_SECS as i64)
            .await?;

        info!(family_id = %family_id, user_id = %user_id, "Created token family");
        Ok(family_id)
    }

    /// Add a token (JTI) to a family
    #[instrument(skip(self))]
    pub async fn add_token_to_family(&self, family_id: Uuid, jti: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let jtis_key = format!("token_family:{}:jtis", family_id);

        conn.sadd::<_, _, ()>(&jtis_key, jti).await?;
        conn.expire::<_, ()>(&jtis_key, FAMILY_TTL_SECS as i64)
            .await?;

        debug!(family_id = %family_id, jti = %jti, "Added token to family");
        Ok(())
    }

    /// Check if a token (JTI) is in a family
    #[instrument(skip(self))]
    pub async fn is_token_in_family(&self, family_id: Uuid, jti: &str) -> Result<bool> {
        let mut conn = self.get_conn().await?;
        let jtis_key = format!("token_family:{}:jtis", family_id);

        let is_member: bool = conn.sismember(&jtis_key, jti).await?;
        debug!(family_id = %family_id, jti = %jti, is_member = %is_member, "Checked token membership");
        Ok(is_member)
    }

    /// Remove a token (JTI) from a family (used after successful refresh)
    #[instrument(skip(self))]
    pub async fn remove_token_from_family(&self, family_id: Uuid, jti: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let jtis_key = format!("token_family:{}:jtis", family_id);

        conn.srem::<_, _, ()>(&jtis_key, jti).await?;
        debug!(family_id = %family_id, jti = %jti, "Removed token from family");
        Ok(())
    }

    /// Revoke entire token family (called when token reuse detected)
    #[instrument(skip(self))]
    pub async fn revoke_family(&self, family_id: Uuid) -> Result<()> {
        let mut conn = self.get_conn().await?;

        let meta_key = format!("token_family:{}:meta", family_id);
        let jtis_key = format!("token_family:{}:jtis", family_id);

        // Get all JTIs before deletion for logging
        let jtis: Vec<String> = conn.smembers(&jtis_key).await.unwrap_or_default();

        // Delete both keys
        conn.del::<_, ()>(&meta_key).await?;
        conn.del::<_, ()>(&jtis_key).await?;

        warn!(
            family_id = %family_id,
            revoked_tokens = %jtis.len(),
            "Revoked entire token family"
        );
        Ok(())
    }

    /// Get family metadata (user_id)
    #[instrument(skip(self))]
    pub async fn get_family_user(&self, family_id: Uuid) -> Result<Option<Uuid>> {
        let mut conn = self.get_conn().await?;
        let meta_key = format!("token_family:{}:meta", family_id);

        let user_id_str: Option<String> = conn.hget(&meta_key, "user_id").await?;

        match user_id_str {
            Some(s) => Ok(Some(Uuid::parse_str(&s)?)),
            None => Ok(None),
        }
    }

    /// Validate refresh token and check for reuse
    /// Returns Ok(true) if token is valid, Ok(false) if already used (reuse detected)
    #[instrument(skip(self))]
    pub async fn validate_refresh_token(&self, family_id: Uuid, jti: &str) -> Result<bool> {
        let is_valid = self.is_token_in_family(family_id, jti).await?;

        if !is_valid {
            // Token not in family - either already used (reuse attack) or invalid
            // Get user_id for security logging
            if let Some(user_id) = self.get_family_user(family_id).await? {
                error!(
                    family_id = %family_id,
                    attempted_jti = %jti,
                    user_id = %user_id,
                    "TOKEN REUSE DETECTED - Revoking entire family"
                );

                // Revoke the entire family
                self.revoke_family(family_id).await?;
            }
            return Ok(false);
        }

        Ok(true)
    }

    /// Rotate refresh token: remove old JTI, add new JTI
    #[instrument(skip(self))]
    pub async fn rotate_token(&self, family_id: Uuid, old_jti: &str, new_jti: &str) -> Result<()> {
        self.remove_token_from_family(family_id, old_jti).await?;
        self.add_token_to_family(family_id, new_jti).await?;

        debug!(family_id = %family_id, old_jti = %old_jti, new_jti = %new_jti, "Rotated refresh token");
        Ok(())
    }

    /// Check if family exists
    pub async fn family_exists(&self, family_id: Uuid) -> Result<bool> {
        let mut conn = self.get_conn().await?;
        let meta_key = format!("token_family:{}:meta", family_id);
        let exists: bool = conn.exists(&meta_key).await?;
        Ok(exists)
    }

    /// Revoke all refresh tokens for a user by revoking all families
    /// Returns count of revoked families/tokens
    #[instrument(skip(self))]
    pub async fn revoke_all_user_tokens(&self, user_id: &Uuid) -> Result<u32> {
        let mut conn = self.get_conn().await?;

        // Scan for all token families belonging to this user
        let pattern = format!("token_family:*:meta");
        let mut revoked_count = 0u32;

        // Use SCAN to find all family metadata keys
        let mut cursor = 0;
        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .context("Failed to scan for token families")?;

            for meta_key in keys {
                // Extract family_id from key "token_family:{family_id}:meta"
                if let Some(family_id_str) = meta_key
                    .strip_prefix("token_family:")
                    .and_then(|s| s.strip_suffix(":meta"))
                {
                    if let Ok(family_id) = Uuid::parse_str(family_id_str) {
                        // Check if this family belongs to the user
                        if let Some(family_user_id) = self.get_family_user(family_id).await? {
                            if family_user_id == *user_id {
                                self.revoke_family(family_id).await?;
                                revoked_count += 1;
                            }
                        }
                    }
                }
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        info!(
            user_id = %user_id,
            revoked_families = %revoked_count,
            "Revoked all refresh token families for user"
        );

        Ok(revoked_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_family_lifecycle() {
        // Skip if Redis not available
        let manager = match TokenFamilyManager::new("redis://localhost:6379") {
            Ok(m) => m,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let user_id = Uuid::new_v4();
        let family_id = manager.create_family(user_id).await.unwrap();

        // Add initial token
        let jti1 = Uuid::new_v4().to_string();
        manager.add_token_to_family(family_id, &jti1).await.unwrap();
        assert!(manager.is_token_in_family(family_id, &jti1).await.unwrap());

        // Rotate to new token
        let jti2 = Uuid::new_v4().to_string();
        manager.rotate_token(family_id, &jti1, &jti2).await.unwrap();

        // Old token should be removed
        assert!(!manager.is_token_in_family(family_id, &jti1).await.unwrap());
        // New token should be present
        assert!(manager.is_token_in_family(family_id, &jti2).await.unwrap());

        // Cleanup
        manager.revoke_family(family_id).await.unwrap();
    }

    #[tokio::test]
    async fn test_token_reuse_detection() {
        let manager = match TokenFamilyManager::new("redis://localhost:6379") {
            Ok(m) => m,
            Err(_) => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let user_id = Uuid::new_v4();
        let family_id = manager.create_family(user_id).await.unwrap();

        let jti1 = Uuid::new_v4().to_string();
        manager.add_token_to_family(family_id, &jti1).await.unwrap();

        // First validation should succeed
        assert!(manager
            .validate_refresh_token(family_id, &jti1)
            .await
            .unwrap());

        // Remove token (simulating rotation)
        manager
            .remove_token_from_family(family_id, &jti1)
            .await
            .unwrap();

        // Reuse attempt should fail and revoke family
        assert!(!manager
            .validate_refresh_token(family_id, &jti1)
            .await
            .unwrap());

        // Family should be revoked
        assert!(!manager.family_exists(family_id).await.unwrap());
    }
}
