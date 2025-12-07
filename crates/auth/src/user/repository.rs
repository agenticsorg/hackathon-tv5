use crate::error::{AuthError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            email_verified: user.email_verified,
            created_at: user.created_at,
        }
    }
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    async fn update_email_verified(&self, id: Uuid, verified: bool) -> Result<()>;
    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        display_name: &str,
    ) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, display_name)
            VALUES ($1, $2, $3)
            RETURNING id, email, password_hash, display_name, email_verified, created_at, updated_at, deleted_at
            "#
        )
        .bind(email)
        .bind(password_hash)
        .bind(display_name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AuthError::Internal("Email already exists".to_string());
                }
            }
            AuthError::Database(e.to_string())
        })?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, display_name, email_verified, created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, display_name, email_verified, created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn update_email_verified(&self, id: Uuid, verified: bool) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET email_verified = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(verified)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_response_conversion() {
        let user = User {
            id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hash123".to_string(),
            display_name: "Test User".to_string(),
            email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let response: UserResponse = user.clone().into();
        assert_eq!(response.id, user.id);
        assert_eq!(response.email, user.email);
        assert_eq!(response.display_name, user.display_name);
        assert_eq!(response.email_verified, user.email_verified);
    }

    #[test]
    fn test_create_user_request_serialization() {
        let request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "Test1234".to_string(),
            display_name: "Test User".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("test@example.com"));
        assert!(json.contains("Test User"));
    }
}
