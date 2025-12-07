use crate::error::{AuthError, Result};
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

const KEY_LENGTH: usize = 32;
const PREFIX: &str = "mg_live_";
const PREFIX_LENGTH: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyWithSecret {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub rate_limit_per_minute: Option<i32>,
    pub expires_in_days: Option<i32>,
}

pub struct ApiKeyManager {
    db_pool: PgPool,
}

impl ApiKeyManager {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub fn generate_key() -> String {
        let random_part: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(KEY_LENGTH)
            .map(char::from)
            .collect();

        format!("{}{}", PREFIX, random_part)
    }

    pub fn hash_key(key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn extract_prefix(key: &str) -> Result<String> {
        if !key.starts_with(PREFIX) {
            return Err(AuthError::InvalidToken(
                "Invalid API key format".to_string(),
            ));
        }

        if key.len() < PREFIX.len() + PREFIX_LENGTH {
            return Err(AuthError::InvalidToken("API key too short".to_string()));
        }

        Ok(key[..PREFIX.len() + PREFIX_LENGTH].to_string())
    }

    pub fn validate_scopes(scopes: &[String]) -> Result<()> {
        let valid_scopes = [
            "read:content",
            "read:recommendations",
            "write:watchlist",
            "write:progress",
            "admin:full",
        ];

        for scope in scopes {
            if !valid_scopes.contains(&scope.as_str()) {
                return Err(AuthError::InvalidScope(scope.clone()));
            }
        }

        Ok(())
    }

    pub async fn create_api_key(
        &self,
        user_id: Uuid,
        request: CreateApiKeyRequest,
    ) -> Result<ApiKeyWithSecret> {
        Self::validate_scopes(&request.scopes)?;

        let key = Self::generate_key();
        let key_hash = Self::hash_key(&key);
        let key_prefix = Self::extract_prefix(&key)?;

        let expires_at = request
            .expires_in_days
            .map(|days| Utc::now() + Duration::days(days as i64));

        let rate_limit = request.rate_limit_per_minute.unwrap_or(60);

        let api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (user_id, name, key_prefix, key_hash, scopes, rate_limit_per_minute, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, user_id, name, key_prefix, key_hash, scopes, rate_limit_per_minute,
                      expires_at, last_used_at, created_at, revoked_at
            "#,
        )
        .bind(user_id)
        .bind(&request.name)
        .bind(&key_prefix)
        .bind(&key_hash)
        .bind(&request.scopes)
        .bind(rate_limit)
        .bind(expires_at)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(ApiKeyWithSecret {
            id: api_key.id,
            name: api_key.name,
            key,
            scopes: api_key.scopes,
            rate_limit_per_minute: api_key.rate_limit_per_minute,
            expires_at: api_key.expires_at,
            created_at: api_key.created_at,
        })
    }

    pub async fn list_user_keys(&self, user_id: Uuid) -> Result<Vec<ApiKey>> {
        let keys = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, key_prefix, key_hash, scopes, rate_limit_per_minute,
                   expires_at, last_used_at, created_at, revoked_at
            FROM api_keys
            WHERE user_id = $1 AND revoked_at IS NULL
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.db_pool)
        .await?;

        Ok(keys)
    }

    pub async fn revoke_key(&self, user_id: Uuid, key_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE api_keys
            SET revoked_at = NOW()
            WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(key_id)
        .bind(user_id)
        .execute(&self.db_pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AuthError::InvalidToken(
                "API key not found or already revoked".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn verify_key(&self, key: &str) -> Result<ApiKey> {
        let key_hash = Self::hash_key(key);
        let key_prefix = Self::extract_prefix(key)?;

        let api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT id, user_id, name, key_prefix, key_hash, scopes, rate_limit_per_minute,
                   expires_at, last_used_at, created_at, revoked_at
            FROM api_keys
            WHERE key_prefix = $1 AND key_hash = $2 AND revoked_at IS NULL
            "#,
        )
        .bind(&key_prefix)
        .bind(&key_hash)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or_else(|| AuthError::InvalidToken("Invalid API key".to_string()))?;

        if let Some(expires_at) = api_key.expires_at {
            if Utc::now() > expires_at {
                return Err(AuthError::TokenExpired);
            }
        }

        Ok(api_key)
    }

    pub async fn update_last_used(&self, key_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE api_keys
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(key_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = ApiKeyManager::generate_key();
        assert!(key.starts_with(PREFIX));
        assert_eq!(key.len(), PREFIX.len() + KEY_LENGTH);
    }

    #[test]
    fn test_hash_key() {
        let key = "mg_live_test123456789012345678901234";
        let hash1 = ApiKeyManager::hash_key(key);
        let hash2 = ApiKeyManager::hash_key(key);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_extract_prefix() {
        let key = "mg_live_x7k9m2p4q8r1";
        let prefix = ApiKeyManager::extract_prefix(key).unwrap();
        assert_eq!(prefix, "mg_live_x7k9m2p4q8r1");
    }

    #[test]
    fn test_extract_prefix_invalid() {
        let result = ApiKeyManager::extract_prefix("invalid_key");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_scopes_valid() {
        let scopes = vec!["read:content".to_string(), "write:watchlist".to_string()];
        assert!(ApiKeyManager::validate_scopes(&scopes).is_ok());
    }

    #[test]
    fn test_validate_scopes_invalid() {
        let scopes = vec!["invalid:scope".to_string()];
        assert!(ApiKeyManager::validate_scopes(&scopes).is_err());
    }

    #[tokio::test]
    async fn test_create_api_key() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                key_prefix VARCHAR(12) NOT NULL,
                key_hash VARCHAR(64) NOT NULL,
                scopes TEXT[] NOT NULL DEFAULT '{}',
                rate_limit_per_minute INTEGER DEFAULT 60,
                expires_at TIMESTAMP WITH TIME ZONE,
                last_used_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                revoked_at TIMESTAMP WITH TIME ZONE,
                UNIQUE(key_prefix)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let manager = ApiKeyManager::new(pool.clone());
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Test Key".to_string(),
            scopes: vec!["read:content".to_string()],
            rate_limit_per_minute: Some(100),
            expires_in_days: None,
        };

        let result = manager.create_api_key(user_id, request).await;
        assert!(result.is_ok());

        let api_key = result.unwrap();
        assert_eq!(api_key.name, "Test Key");
        assert!(api_key.key.starts_with(PREFIX));
        assert_eq!(api_key.rate_limit_per_minute, 100);

        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(api_key.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_verify_key() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                key_prefix VARCHAR(12) NOT NULL,
                key_hash VARCHAR(64) NOT NULL,
                scopes TEXT[] NOT NULL DEFAULT '{}',
                rate_limit_per_minute INTEGER DEFAULT 60,
                expires_at TIMESTAMP WITH TIME ZONE,
                last_used_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                revoked_at TIMESTAMP WITH TIME ZONE,
                UNIQUE(key_prefix)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let manager = ApiKeyManager::new(pool.clone());
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Verify Test".to_string(),
            scopes: vec!["read:content".to_string()],
            rate_limit_per_minute: None,
            expires_in_days: None,
        };

        let created = manager.create_api_key(user_id, request).await.unwrap();
        let verified = manager.verify_key(&created.key).await.unwrap();

        assert_eq!(verified.id, created.id);
        assert_eq!(verified.user_id, user_id);
        assert_eq!(verified.scopes, vec!["read:content".to_string()]);

        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(created.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_list_user_keys() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                key_prefix VARCHAR(12) NOT NULL,
                key_hash VARCHAR(64) NOT NULL,
                scopes TEXT[] NOT NULL DEFAULT '{}',
                rate_limit_per_minute INTEGER DEFAULT 60,
                expires_at TIMESTAMP WITH TIME ZONE,
                last_used_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                revoked_at TIMESTAMP WITH TIME ZONE,
                UNIQUE(key_prefix)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let manager = ApiKeyManager::new(pool.clone());
        let user_id = Uuid::new_v4();

        let request1 = CreateApiKeyRequest {
            name: "Key 1".to_string(),
            scopes: vec!["read:content".to_string()],
            rate_limit_per_minute: None,
            expires_in_days: None,
        };

        let request2 = CreateApiKeyRequest {
            name: "Key 2".to_string(),
            scopes: vec!["write:watchlist".to_string()],
            rate_limit_per_minute: None,
            expires_in_days: None,
        };

        let key1 = manager.create_api_key(user_id, request1).await.unwrap();
        let key2 = manager.create_api_key(user_id, request2).await.unwrap();

        let keys = manager.list_user_keys(user_id).await.unwrap();
        assert_eq!(keys.len(), 2);

        sqlx::query("DELETE FROM api_keys WHERE id = ANY($1)")
            .bind(&[key1.id, key2.id])
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_revoke_key() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                key_prefix VARCHAR(12) NOT NULL,
                key_hash VARCHAR(64) NOT NULL,
                scopes TEXT[] NOT NULL DEFAULT '{}',
                rate_limit_per_minute INTEGER DEFAULT 60,
                expires_at TIMESTAMP WITH TIME ZONE,
                last_used_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                revoked_at TIMESTAMP WITH TIME ZONE,
                UNIQUE(key_prefix)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let manager = ApiKeyManager::new(pool.clone());
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Revoke Test".to_string(),
            scopes: vec!["read:content".to_string()],
            rate_limit_per_minute: None,
            expires_in_days: None,
        };

        let created = manager.create_api_key(user_id, request).await.unwrap();

        let revoke_result = manager.revoke_key(user_id, created.id).await;
        assert!(revoke_result.is_ok());

        let verify_result = manager.verify_key(&created.key).await;
        assert!(verify_result.is_err());

        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(created.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_expired_key() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url).await.unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_keys (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id UUID NOT NULL,
                name VARCHAR(255) NOT NULL,
                key_prefix VARCHAR(12) NOT NULL,
                key_hash VARCHAR(64) NOT NULL,
                scopes TEXT[] NOT NULL DEFAULT '{}',
                rate_limit_per_minute INTEGER DEFAULT 60,
                expires_at TIMESTAMP WITH TIME ZONE,
                last_used_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                revoked_at TIMESTAMP WITH TIME ZONE,
                UNIQUE(key_prefix)
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let manager = ApiKeyManager::new(pool.clone());
        let user_id = Uuid::new_v4();

        let request = CreateApiKeyRequest {
            name: "Expired Key".to_string(),
            scopes: vec!["read:content".to_string()],
            rate_limit_per_minute: None,
            expires_in_days: Some(-1),
        };

        let created = manager.create_api_key(user_id, request).await.unwrap();
        let verify_result = manager.verify_key(&created.key).await;

        assert!(verify_result.is_err());
        if let Err(AuthError::TokenExpired) = verify_result {
            assert!(true);
        } else {
            panic!("Expected TokenExpired error");
        }

        sqlx::query("DELETE FROM api_keys WHERE id = $1")
            .bind(created.id)
            .execute(&pool)
            .await
            .unwrap();
    }
}
