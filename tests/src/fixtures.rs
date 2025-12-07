use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::TestContext;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TestContent {
    pub id: Uuid,
    pub title: String,
    pub content_type: String,
    pub url: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TestSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub position_seconds: i32,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create_test_user(ctx: &TestContext) -> Result<TestUser> {
    let id = Uuid::new_v4();
    let email = format!("test-{}@example.com", id);
    let display_name = format!("Test User {}", id);
    let password_hash = "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5lW8V0z0QhXyK"; // "password123"

    let user = sqlx::query_as::<_, TestUser>(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, email_verified)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, email, password_hash, display_name, email_verified, created_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(true)
    .fetch_one(&ctx.db_pool)
    .await
    .context("Failed to create test user")?;

    Ok(user)
}

pub async fn create_unverified_user(ctx: &TestContext) -> Result<TestUser> {
    let id = Uuid::new_v4();
    let email = format!("unverified-{}@example.com", id);
    let display_name = format!("Unverified User {}", id);
    let password_hash = "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5lW8V0z0QhXyK";

    let user = sqlx::query_as::<_, TestUser>(
        r#"
        INSERT INTO users (id, email, password_hash, display_name, email_verified)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, email, password_hash, display_name, email_verified, created_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(false)
    .fetch_one(&ctx.db_pool)
    .await
    .context("Failed to create unverified test user")?;

    Ok(user)
}

pub async fn create_test_content(ctx: &TestContext) -> Result<TestContent> {
    let id = Uuid::new_v4();
    let title = format!("Test Content {}", id);

    let metadata = serde_json::json!({
        "duration": 3600,
        "resolution": "1920x1080",
        "codec": "h264"
    });

    let content = sqlx::query_as::<_, TestContent>(
        r#"
        INSERT INTO content (id, title, content_type, url, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, title, content_type, url, metadata, created_at
        "#,
    )
    .bind(id)
    .bind(title)
    .bind("video/mp4")
    .bind(format!("https://example.com/video/{}.mp4", id))
    .bind(metadata)
    .fetch_one(&ctx.db_pool)
    .await
    .context("Failed to create test content")?;

    Ok(content)
}

pub async fn create_test_content_with_type(
    ctx: &TestContext,
    content_type: &str,
    title: &str,
) -> Result<TestContent> {
    let id = Uuid::new_v4();

    let metadata = serde_json::json!({
        "duration": 3600,
        "type": content_type
    });

    let content = sqlx::query_as::<_, TestContent>(
        r#"
        INSERT INTO content (id, title, content_type, url, metadata)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, title, content_type, url, metadata, created_at
        "#,
    )
    .bind(id)
    .bind(title)
    .bind(content_type)
    .bind(format!("https://example.com/media/{}.mp4", id))
    .bind(metadata)
    .fetch_one(&ctx.db_pool)
    .await
    .context("Failed to create test content with type")?;

    Ok(content)
}

pub async fn create_test_session(
    ctx: &TestContext,
    user: &TestUser,
    content: &TestContent,
) -> Result<TestSession> {
    let id = Uuid::new_v4();

    let session = sqlx::query_as::<_, TestSession>(
        r#"
        INSERT INTO playback_sessions (id, user_id, content_id, position_seconds, duration_seconds)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, content_id, position_seconds, duration_seconds, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(user.id)
    .bind(content.id)
    .bind(0)
    .bind(Some(3600))
    .fetch_one(&ctx.db_pool)
    .await
    .context("Failed to create test session")?;

    Ok(session)
}

pub async fn create_test_session_with_position(
    ctx: &TestContext,
    user: &TestUser,
    content: &TestContent,
    position_seconds: i32,
) -> Result<TestSession> {
    let id = Uuid::new_v4();

    let session = sqlx::query_as::<_, TestSession>(
        r#"
        INSERT INTO playback_sessions (id, user_id, content_id, position_seconds, duration_seconds)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, content_id, position_seconds, duration_seconds, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(user.id)
    .bind(content.id)
    .bind(position_seconds)
    .bind(Some(3600))
    .fetch_one(&ctx.db_pool)
    .await
    .context("Failed to create test session with position")?;

    Ok(session)
}

pub async fn cleanup_user(pool: &PgPool, user_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .context("Failed to cleanup user")?;
    Ok(())
}

pub async fn cleanup_content(pool: &PgPool, content_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(pool)
        .await
        .context("Failed to cleanup content")?;
    Ok(())
}

pub async fn cleanup_session(pool: &PgPool, session_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM playback_sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await
        .context("Failed to cleanup session")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestContext;

    #[tokio::test]
    async fn test_create_test_user() {
        let ctx = TestContext::new()
            .await
            .expect("Failed to create test context");
        ctx.run_migrations()
            .await
            .expect("Failed to run migrations");

        let user = create_test_user(&ctx)
            .await
            .expect("Failed to create test user");
        assert!(user.email_verified);
        assert!(user.email.contains("@example.com"));
        assert!(!user.display_name.is_empty());

        ctx.teardown().await.expect("Failed to teardown");
    }

    #[tokio::test]
    async fn test_create_test_content() {
        let ctx = TestContext::new()
            .await
            .expect("Failed to create test context");
        ctx.run_migrations()
            .await
            .expect("Failed to run migrations");

        let content = create_test_content(&ctx)
            .await
            .expect("Failed to create test content");
        assert_eq!(content.content_type, "video/mp4");
        assert!(content.url.starts_with("https://"));

        ctx.teardown().await.expect("Failed to teardown");
    }

    #[tokio::test]
    async fn test_create_test_session() {
        let ctx = TestContext::new()
            .await
            .expect("Failed to create test context");
        ctx.run_migrations()
            .await
            .expect("Failed to run migrations");

        let user = create_test_user(&ctx)
            .await
            .expect("Failed to create test user");
        let content = create_test_content(&ctx)
            .await
            .expect("Failed to create test content");
        let session = create_test_session(&ctx, &user, &content)
            .await
            .expect("Failed to create test session");

        assert_eq!(session.user_id, user.id);
        assert_eq!(session.content_id, content.id);
        assert_eq!(session.position_seconds, 0);

        ctx.teardown().await.expect("Failed to teardown");
    }
}
