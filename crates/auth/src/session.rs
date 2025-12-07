use crate::error::{AuthError, Result};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const SESSION_TTL: u64 = 7 * 24 * 3600; // 7 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub device_id: Option<String>,
    pub refresh_token_jti: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

impl Session {
    pub fn new(user_id: String, refresh_token_jti: String, device_id: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            device_id,
            refresh_token_jti,
            created_at: now,
            last_accessed: now,
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
    }
}

pub struct SessionManager {
    redis_client: Client,
}

impl SessionManager {
    pub fn new(redis_url: &str) -> Result<Self> {
        let redis_client = Client::open(redis_url)
            .map_err(|e| AuthError::Redis(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self { redis_client })
    }

    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.redis_client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to get Redis connection: {}", e)))
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        user_id: String,
        refresh_token_jti: String,
        device_id: Option<String>,
    ) -> Result<Session> {
        let session = Session::new(user_id, refresh_token_jti, device_id);

        let mut conn = self.get_connection().await?;

        let session_key = format!("session:{}", session.session_id);
        let session_json = serde_json::to_string(&session)
            .map_err(|e| AuthError::Internal(format!("Failed to serialize session: {}", e)))?;

        conn.set_ex::<_, _, ()>(&session_key, session_json, SESSION_TTL)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to store session: {}", e)))?;

        // Index by user_id for easy lookup
        let user_sessions_key = format!("user:{}:sessions", session.user_id);
        conn.sadd::<_, _, ()>(&user_sessions_key, &session.session_id)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to index session: {}", e)))?;

        conn.expire::<_, ()>(&user_sessions_key, SESSION_TTL as i64)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to set expiry: {}", e)))?;

        Ok(session)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let mut conn = self.get_connection().await?;

        let session_key = format!("session:{}", session_id);
        let session_json: Option<String> = conn
            .get(&session_key)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to get session: {}", e)))?;

        match session_json {
            Some(json) => {
                let mut session: Session = serde_json::from_str(&json).map_err(|e| {
                    AuthError::Internal(format!("Failed to deserialize session: {}", e))
                })?;

                session.touch();

                // Update last accessed time
                let updated_json = serde_json::to_string(&session).map_err(|e| {
                    AuthError::Internal(format!("Failed to serialize session: {}", e))
                })?;

                conn.set_ex::<_, _, ()>(&session_key, updated_json, SESSION_TTL)
                    .await
                    .map_err(|e| AuthError::Redis(format!("Failed to update session: {}", e)))?;

                Ok(session)
            }
            None => Err(AuthError::SessionNotFound),
        }
    }

    /// Revoke a specific session
    pub async fn revoke_session(&self, session_id: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // Get session to find user_id
        let session = self.get_session(session_id).await?;

        // Delete session
        let session_key = format!("session:{}", session_id);
        conn.del::<_, ()>(&session_key)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to delete session: {}", e)))?;

        // Remove from user's session set
        let user_sessions_key = format!("user:{}:sessions", session.user_id);
        conn.srem::<_, _, ()>(&user_sessions_key, session_id)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to remove session from index: {}", e)))?;

        Ok(())
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all_user_sessions(&self, user_id: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let user_sessions_key = format!("user:{}:sessions", user_id);

        // Get all session IDs for the user
        let session_ids: Vec<String> = conn
            .smembers(&user_sessions_key)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to get user sessions: {}", e)))?;

        // Delete all sessions
        for session_id in &session_ids {
            let session_key = format!("session:{}", session_id);
            conn.del::<_, ()>(&session_key)
                .await
                .map_err(|e| AuthError::Redis(format!("Failed to delete session: {}", e)))?;
        }

        // Delete user sessions index
        conn.del::<_, ()>(&user_sessions_key).await.map_err(|e| {
            AuthError::Redis(format!("Failed to delete user sessions index: {}", e))
        })?;

        tracing::info!(
            "Revoked {} sessions for user {}",
            session_ids.len(),
            user_id
        );

        Ok(())
    }

    /// Invalidate all user sessions except optionally one
    /// Returns count of invalidated sessions
    pub async fn invalidate_all_user_sessions(
        &self,
        user_id: &Uuid,
        except_session_id: Option<&str>,
    ) -> Result<u32> {
        let mut conn = self.get_connection().await?;

        let user_sessions_key = format!("user:{}:sessions", user_id);

        // Get all session IDs for the user
        let session_ids: Vec<String> = conn
            .smembers(&user_sessions_key)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to get user sessions: {}", e)))?;

        let mut invalidated_count = 0u32;

        // Delete all sessions except the one to keep
        for session_id in &session_ids {
            if let Some(keep_id) = except_session_id {
                if session_id == keep_id {
                    continue;
                }
            }

            let session_key = format!("session:{}", session_id);
            conn.del::<_, ()>(&session_key)
                .await
                .map_err(|e| AuthError::Redis(format!("Failed to delete session: {}", e)))?;

            // Remove from user's session set
            conn.srem::<_, _, ()>(&user_sessions_key, session_id)
                .await
                .map_err(|e| {
                    AuthError::Redis(format!("Failed to remove session from index: {}", e))
                })?;

            invalidated_count += 1;
        }

        tracing::info!(
            "Invalidated {} sessions for user {} (kept: {:?})",
            invalidated_count,
            user_id,
            except_session_id
        );

        Ok(invalidated_count)
    }

    /// Check if a refresh token JTI is revoked
    pub async fn is_token_revoked(&self, jti: &str) -> Result<bool> {
        let mut conn = self.get_connection().await?;

        let revoked_key = format!("revoked:{}", jti);
        let exists: bool = conn
            .exists(&revoked_key)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to check revocation: {}", e)))?;

        Ok(exists)
    }

    /// Mark a token as revoked
    pub async fn revoke_token(&self, jti: &str, ttl: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let revoked_key = format!("revoked:{}", jti);
        conn.set_ex::<_, _, ()>(&revoked_key, "1", ttl)
            .await
            .map_err(|e| AuthError::Redis(format!("Failed to revoke token: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_session_creation() {
        let manager = SessionManager::new("redis://127.0.0.1/").unwrap();

        let session = manager
            .create_session(
                "user123".to_string(),
                "jti123".to_string(),
                Some("device123".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(session.user_id, "user123");
        assert_eq!(session.refresh_token_jti, "jti123");
    }

    #[tokio::test]
    #[ignore] // Requires Redis
    async fn test_session_retrieval() {
        let manager = SessionManager::new("redis://127.0.0.1/").unwrap();

        let session = manager
            .create_session("user123".to_string(), "jti123".to_string(), None)
            .await
            .unwrap();

        let retrieved = manager.get_session(&session.session_id).await.unwrap();
        assert_eq!(retrieved.user_id, session.user_id);
    }
}
