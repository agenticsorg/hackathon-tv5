//! Redis-backed storage for OAuth authentication state

use crate::error::{AuthError, Result};
use crate::oauth::{
    device::DeviceCode,
    pkce::{AuthorizationCode, PkceChallenge},
};
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};

/// TTL constants for different token types
const PKCE_TTL_SECS: u64 = 600; // 10 minutes
const AUTH_CODE_TTL_SECS: u64 = 300; // 5 minutes
const DEVICE_CODE_TTL_SECS: u64 = 900; // 15 minutes

/// Redis storage manager for auth state
#[derive(Clone)]
pub struct AuthStorage {
    client: Client,
}

impl AuthStorage {
    /// Create new Redis storage connection
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| AuthError::Internal(format!("Redis connection failed: {}", e)))?;
        Ok(Self { client })
    }

    /// Create from REDIS_URL environment variable
    pub fn from_env() -> Result<Self> {
        let url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        Self::new(&url)
    }

    /// Get async connection
    async fn get_conn(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::Internal(format!("Redis connection error: {}", e)))
    }

    // ========== PKCE Sessions ==========

    /// Store PKCE session with 10 minute TTL
    pub async fn store_pkce(&self, state: &str, pkce: &PkceChallenge) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("pkce:{}", state);
        let value = serde_json::to_string(pkce)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        conn.set_ex::<_, _, ()>(&key, value, PKCE_TTL_SECS)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    /// Get PKCE session
    pub async fn get_pkce(&self, state: &str) -> Result<Option<PkceChallenge>> {
        let mut conn = self.get_conn().await?;
        let key = format!("pkce:{}", state);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let pkce: PkceChallenge = serde_json::from_str(&v)
                    .map_err(|e| AuthError::Internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(pkce))
            }
            None => Ok(None),
        }
    }

    /// Delete PKCE session
    pub async fn delete_pkce(&self, state: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("pkce:{}", state);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    // ========== Authorization Codes ==========

    /// Store authorization code with 5 minute TTL
    pub async fn store_auth_code(&self, code: &str, auth_code: &AuthorizationCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);
        let value = serde_json::to_string(auth_code)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        conn.set_ex::<_, _, ()>(&key, value, AUTH_CODE_TTL_SECS)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    /// Get authorization code
    pub async fn get_auth_code(&self, code: &str) -> Result<Option<AuthorizationCode>> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let auth_code: AuthorizationCode = serde_json::from_str(&v)
                    .map_err(|e| AuthError::Internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(auth_code))
            }
            None => Ok(None),
        }
    }

    /// Update authorization code (for marking as used)
    pub async fn update_auth_code(&self, code: &str, auth_code: &AuthorizationCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);

        // Get remaining TTL
        let ttl: i64 = conn
            .ttl::<_, i64>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis TTL error: {}", e)))?;

        if ttl > 0 {
            let value = serde_json::to_string(auth_code)
                .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;
            conn.set_ex::<_, _, ()>(&key, value, ttl as u64)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;
        }

        Ok(())
    }

    /// Delete authorization code
    pub async fn delete_auth_code(&self, code: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("authcode:{}", code);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    // ========== Device Codes ==========

    /// Store device code with 15 minute TTL
    pub async fn store_device_code(&self, device_code: &str, device: &DeviceCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("devicecode:{}", device_code);
        let user_code_key = format!("devicecode:user:{}", device.user_code);
        let value = serde_json::to_string(device)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        // Store by device_code
        conn.set_ex::<_, _, ()>(&key, value.clone(), DEVICE_CODE_TTL_SECS)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        // Store mapping from user_code to device_code for approval lookup
        conn.set_ex::<_, _, ()>(&user_code_key, device_code, DEVICE_CODE_TTL_SECS)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    /// Get device code
    pub async fn get_device_code(&self, device_code: &str) -> Result<Option<DeviceCode>> {
        let mut conn = self.get_conn().await?;
        let key = format!("devicecode:{}", device_code);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let device: DeviceCode = serde_json::from_str(&v)
                    .map_err(|e| AuthError::Internal(format!("Deserialization error: {}", e)))?;
                Ok(Some(device))
            }
            None => Ok(None),
        }
    }

    /// Update device code (for setting user_id after approval)
    pub async fn update_device_code(&self, device_code: &str, device: &DeviceCode) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("devicecode:{}", device_code);

        let ttl: i64 = conn
            .ttl::<_, i64>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis TTL error: {}", e)))?;

        if ttl > 0 {
            let value = serde_json::to_string(device)
                .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;
            conn.set_ex::<_, _, ()>(&key, value, ttl as u64)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;
        }

        Ok(())
    }

    /// Get device code by user_code
    pub async fn get_device_code_by_user_code(
        &self,
        user_code: &str,
    ) -> Result<Option<DeviceCode>> {
        let mut conn = self.get_conn().await?;
        let user_code_key = format!("devicecode:user:{}", user_code);

        // Get device_code from user_code mapping
        let device_code: Option<String> = conn
            .get(&user_code_key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match device_code {
            Some(dc) => self.get_device_code(&dc).await,
            None => Ok(None),
        }
    }

    /// Delete device code
    pub async fn delete_device_code(&self, device_code: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;

        // Get the device to find user_code for cleanup
        if let Some(device) = self.get_device_code(device_code).await? {
            let user_code_key = format!("devicecode:user:{}", device.user_code);
            let _: () = conn
                .del::<_, ()>(&user_code_key)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        }

        let key = format!("devicecode:{}", device_code);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    /// Check Redis health
    pub async fn is_healthy(&self) -> bool {
        match self.get_conn().await {
            Ok(mut conn) => redis::cmd("PING")
                .query_async::<_, String>(&mut conn)
                .await
                .is_ok(),
            Err(_) => false,
        }
    }

    // ========== MFA Rate Limiting ==========

    /// Check and increment MFA verification attempts
    /// Returns remaining attempts (0 means rate limited)
    pub async fn check_mfa_rate_limit(&self, user_id: &str) -> Result<u32> {
        let mut conn = self.get_conn().await?;
        let key = format!("mfa:attempts:{}", user_id);
        let max_attempts = 5;
        let window_secs = 60;

        // Get current count
        let count: Option<u32> = conn
            .get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        let current = count.unwrap_or(0);

        if current >= max_attempts {
            return Ok(0);
        }

        // Increment counter
        let new_count: u32 = conn
            .incr::<_, _, u32>(&key, 1)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis INCR error: {}", e)))?;

        // Set expiry on first attempt
        if new_count == 1 {
            conn.expire::<_, ()>(&key, window_secs)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis EXPIRE error: {}", e)))?;
        }

        Ok(max_attempts.saturating_sub(new_count))
    }

    /// Reset MFA rate limit for user
    pub async fn reset_mfa_rate_limit(&self, user_id: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("mfa:attempts:{}", user_id);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    // ========== Password Reset ==========

    /// Store password reset token with 1 hour TTL
    pub async fn store_password_reset_token(
        &self,
        token: &str,
        reset_token: &crate::password_reset::PasswordResetToken,
    ) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("password_reset:{}", token);
        let value = serde_json::to_string(reset_token)
            .map_err(|e| AuthError::Internal(format!("Serialization error: {}", e)))?;

        conn.set_ex::<_, _, ()>(&key, value, 3600)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis SET error: {}", e)))?;

        Ok(())
    }

    /// Get password reset token
    pub async fn get_password_reset_token(
        &self,
        token: &str,
    ) -> Result<Option<crate::password_reset::PasswordResetToken>> {
        let mut conn = self.get_conn().await?;
        let key = format!("password_reset:{}", token);

        let value: Option<String> = conn
            .get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        match value {
            Some(v) => {
                let reset_token: crate::password_reset::PasswordResetToken =
                    serde_json::from_str(&v).map_err(|e| {
                        AuthError::Internal(format!("Deserialization error: {}", e))
                    })?;
                Ok(Some(reset_token))
            }
            None => Ok(None),
        }
    }

    /// Delete password reset token (single-use)
    pub async fn delete_password_reset_token(&self, token: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let key = format!("password_reset:{}", token);
        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        Ok(())
    }

    /// Check password reset rate limit (3 per hour per email)
    pub async fn check_password_reset_rate_limit(&self, email: &str) -> Result<u32> {
        let mut conn = self.get_conn().await?;
        let key = format!("password_reset_attempts:{}", email);
        let max_attempts = 3;
        let window_secs = 3600; // 1 hour

        let count: Option<u32> = conn
            .get(&key)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis GET error: {}", e)))?;

        let current = count.unwrap_or(0);

        if current >= max_attempts {
            return Ok(0);
        }

        let new_count: u32 = conn
            .incr::<_, _, u32>(&key, 1)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis INCR error: {}", e)))?;

        if new_count == 1 {
            conn.expire::<_, ()>(&key, window_secs)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis EXPIRE error: {}", e)))?;
        }

        Ok(max_attempts.saturating_sub(new_count))
    }

    /// Delete all user sessions by user_id pattern
    pub async fn delete_user_sessions(&self, user_id: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;
        let pattern = format!("session:user:{}:*", user_id);

        // Get all matching keys
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| AuthError::Internal(format!("Redis KEYS error: {}", e)))?;

        if !keys.is_empty() {
            conn.del::<_, ()>(keys)
                .await
                .map_err(|e| AuthError::Internal(format!("Redis DEL error: {}", e)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_constants() {
        assert_eq!(PKCE_TTL_SECS, 600);
        assert_eq!(AUTH_CODE_TTL_SECS, 300);
        assert_eq!(DEVICE_CODE_TTL_SECS, 900);
    }
}
