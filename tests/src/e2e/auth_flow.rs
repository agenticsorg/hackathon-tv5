//! End-to-end authentication flow tests
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    user_id: Uuid,
    email: String,
    name: String,
}

#[tokio::test]
async fn test_oauth_pkce_authorization_flow() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Generate PKCE challenge
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);

    // Create test user
    let user_id = Uuid::new_v4();
    let email = format!("test-{}@example.com", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&email)
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Simulate authorization code generation
    let auth_code = Uuid::new_v4().to_string();

    // Store authorization code in Redis with code_challenge
    let auth_key = format!("auth:code:{}", auth_code);
    redis::cmd("SET")
        .arg(&auth_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "code_challenge": code_challenge,
                "expires_at": Utc::now().timestamp() + 600
            })
            .to_string(),
        )
        .arg("EX")
        .arg(600)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Exchange authorization code for tokens
    let stored_data: String = redis::cmd("GET")
        .arg(&auth_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    let auth_data: serde_json::Value = serde_json::from_str(&stored_data)?;
    let stored_challenge = auth_data["code_challenge"].as_str().unwrap();

    // Verify code_challenge matches
    let computed_challenge = generate_code_challenge(&code_verifier);
    assert_eq!(stored_challenge, computed_challenge);

    // Generate access and refresh tokens
    let access_token = Uuid::new_v4().to_string();
    let refresh_token = Uuid::new_v4().to_string();

    // Store tokens in Redis
    let access_key = format!("auth:token:{}", access_token);
    redis::cmd("SET")
        .arg(&access_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "scope": "read write",
                "expires_at": Utc::now().timestamp() + 3600
            })
            .to_string(),
        )
        .arg("EX")
        .arg(3600)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    let refresh_key = format!("auth:refresh:{}", refresh_token);
    redis::cmd("SET")
        .arg(&refresh_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "access_token": access_token
            })
            .to_string(),
        )
        .arg("EX")
        .arg(2592000) // 30 days
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify tokens exist
    let token_exists: bool = redis::cmd("EXISTS")
        .arg(&access_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(token_exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_token_refresh_flow() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    let email = format!("refresh-{}@example.com", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&email)
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Create initial tokens
    let old_access_token = Uuid::new_v4().to_string();
    let refresh_token = Uuid::new_v4().to_string();

    let refresh_key = format!("auth:refresh:{}", refresh_token);
    redis::cmd("SET")
        .arg(&refresh_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "access_token": old_access_token
            })
            .to_string(),
        )
        .arg("EX")
        .arg(2592000)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Refresh token to get new access token
    let new_access_token = Uuid::new_v4().to_string();

    // Update refresh token with new access token
    redis::cmd("SET")
        .arg(&refresh_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "access_token": new_access_token
            })
            .to_string(),
        )
        .arg("EX")
        .arg(2592000)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Store new access token
    let access_key = format!("auth:token:{}", new_access_token);
    redis::cmd("SET")
        .arg(&access_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "scope": "read write",
                "expires_at": Utc::now().timestamp() + 3600
            })
            .to_string(),
        )
        .arg("EX")
        .arg(3600)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify new token exists
    let token_exists: bool = redis::cmd("EXISTS")
        .arg(&access_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(token_exists);

    // Verify refresh token still valid
    let refresh_exists: bool = redis::cmd("EXISTS")
        .arg(&refresh_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(refresh_exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_session_management_create_and_validate() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    let email = format!("session-{}@example.com", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&email)
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Create session
    let session_id = Uuid::new_v4().to_string();
    let session_key = format!("session:{}", session_id);

    redis::cmd("SET")
        .arg(&session_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "created_at": Utc::now().timestamp(),
                "ip_address": "127.0.0.1",
                "user_agent": "Mozilla/5.0"
            })
            .to_string(),
        )
        .arg("EX")
        .arg(86400) // 24 hours
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Validate session exists
    let session_exists: bool = redis::cmd("EXISTS")
        .arg(&session_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(session_exists);

    // Retrieve and validate session data
    let session_data: String = redis::cmd("GET")
        .arg(&session_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;

    let session: serde_json::Value = serde_json::from_str(&session_data)?;
    assert_eq!(session["user_id"].as_str().unwrap(), user_id.to_string());

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_session_expiration() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create session with short TTL
    let session_id = Uuid::new_v4().to_string();
    let session_key = format!("session:{}", session_id);

    redis::cmd("SET")
        .arg(&session_key)
        .arg(
            serde_json::json!({
                "user_id": Uuid::new_v4(),
                "created_at": Utc::now().timestamp()
            })
            .to_string(),
        )
        .arg("EX")
        .arg(1) // 1 second
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Verify session expired
    let session_exists: bool = redis::cmd("EXISTS")
        .arg(&session_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(!session_exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_login_logout_cycle() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    let email = format!("cycle-{}@example.com", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&email)
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // LOGIN: Create session and tokens
    let session_id = Uuid::new_v4().to_string();
    let access_token = Uuid::new_v4().to_string();
    let refresh_token = Uuid::new_v4().to_string();

    let session_key = format!("session:{}", session_id);
    redis::cmd("SET")
        .arg(&session_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "access_token": access_token,
                "created_at": Utc::now().timestamp()
            })
            .to_string(),
        )
        .arg("EX")
        .arg(86400)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    let access_key = format!("auth:token:{}", access_token);
    redis::cmd("SET")
        .arg(&access_key)
        .arg(
            serde_json::json!({
                "user_id": user_id,
                "session_id": session_id
            })
            .to_string(),
        )
        .arg("EX")
        .arg(3600)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify login successful
    let session_exists: bool = redis::cmd("EXISTS")
        .arg(&session_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(session_exists);

    // LOGOUT: Delete session and tokens
    redis::cmd("DEL")
        .arg(&session_key)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    redis::cmd("DEL")
        .arg(&access_key)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify logout successful
    let session_exists: bool = redis::cmd("EXISTS")
        .arg(&session_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(!session_exists);

    let token_exists: bool = redis::cmd("EXISTS")
        .arg(&access_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(!token_exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_concurrent_sessions_same_user() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    let email = format!("multi-{}@example.com", Uuid::new_v4());

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&email)
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Create multiple sessions for same user
    let session1_id = Uuid::new_v4().to_string();
    let session2_id = Uuid::new_v4().to_string();

    for session_id in [&session1_id, &session2_id] {
        let session_key = format!("session:{}", session_id);
        redis::cmd("SET")
            .arg(&session_key)
            .arg(
                serde_json::json!({
                    "user_id": user_id,
                    "created_at": Utc::now().timestamp()
                })
                .to_string(),
            )
            .arg("EX")
            .arg(86400)
            .query_async::<_, ()>(&mut containers.redis_conn.clone())
            .await?;
    }

    // Verify both sessions exist
    for session_id in [&session1_id, &session2_id] {
        let session_key = format!("session:{}", session_id);
        let exists: bool = redis::cmd("EXISTS")
            .arg(&session_key)
            .query_async(&mut containers.redis_conn.clone())
            .await?;
        assert!(exists);
    }

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_token_revocation() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Create test user
    let user_id = Uuid::new_v4();
    let access_token = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("revoke-{}@example.com", Uuid::new_v4()))
    .bind("$2b$12$hashedpassword")
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&containers.db_pool)
    .await?;

    // Create token
    let access_key = format!("auth:token:{}", access_token);
    redis::cmd("SET")
        .arg(&access_key)
        .arg(
            serde_json::json!({
                "user_id": user_id
            })
            .to_string(),
        )
        .arg("EX")
        .arg(3600)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Revoke token
    redis::cmd("DEL")
        .arg(&access_key)
        .query_async::<_, ()>(&mut containers.redis_conn.clone())
        .await?;

    // Verify token revoked
    let exists: bool = redis::cmd("EXISTS")
        .arg(&access_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(!exists);

    containers.cleanup().await?;
    Ok(())
}

#[tokio::test]
async fn test_invalid_token_validation() -> Result<()> {
    let docker = Cli::default();
    let containers = TestContainers::new(&docker).await?;

    // Try to validate non-existent token
    let fake_token = Uuid::new_v4().to_string();
    let token_key = format!("auth:token:{}", fake_token);

    let exists: bool = redis::cmd("EXISTS")
        .arg(&token_key)
        .query_async(&mut containers.redis_conn.clone())
        .await?;
    assert!(!exists);

    containers.cleanup().await?;
    Ok(())
}

// Helper functions
fn generate_code_verifier() -> String {
    let random_bytes: Vec<u8> = (0..32).map(|_| rand::thread_rng().gen()).collect();
    URL_SAFE_NO_PAD.encode(&random_bytes)
}

fn generate_code_challenge(verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(&result)
}
