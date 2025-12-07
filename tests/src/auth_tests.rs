use anyhow::Result;
use media_gateway_tests::{TestClient, TestContext};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    user_id: String,
    email: String,
    verification_required: bool,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

#[derive(Debug, Serialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Debug, Deserialize)]
struct RefreshResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Serialize)]
struct VerifyRequest {
    token: String,
}

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    success: bool,
    message: String,
}

#[tokio::test]
async fn test_auth_flow_register_verify_login_refresh() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.auth_url);
    let email = format!("test-{}@example.com", uuid::Uuid::new_v4());
    let password = "SecurePassword123!";

    // Step 1: Register
    let register_req = RegisterRequest {
        email: email.clone(),
        password: password.to_string(),
    };

    let register_response = client.post("/api/v1/auth/register", &register_req).await?;
    assert_eq!(register_response.status(), 201);

    let register_data: RegisterResponse = register_response.json().await?;
    assert_eq!(register_data.email, email);
    assert!(register_data.verification_required);

    // Step 2: Verify email (simulate token from email)
    let verify_token = "simulated-verification-token";
    let verify_req = VerifyRequest {
        token: verify_token.to_string(),
    };

    let verify_response = client.post("/api/v1/auth/verify", &verify_req).await?;
    assert_eq!(verify_response.status(), 200);

    let verify_data: VerifyResponse = verify_response.json().await?;
    assert!(verify_data.success);

    // Step 3: Login
    let login_req = LoginRequest {
        email: email.clone(),
        password: password.to_string(),
    };

    let login_response = client.post("/api/v1/auth/login", &login_req).await?;
    assert_eq!(login_response.status(), 200);

    let login_data: LoginResponse = login_response.json().await?;
    assert!(!login_data.access_token.is_empty());
    assert!(!login_data.refresh_token.is_empty());
    assert!(login_data.expires_in > 0);

    // Step 4: Refresh token
    let refresh_req = RefreshRequest {
        refresh_token: login_data.refresh_token.clone(),
    };

    let refresh_response = client.post("/api/v1/auth/refresh", &refresh_req).await?;
    assert_eq!(refresh_response.status(), 200);

    let refresh_data: RefreshResponse = refresh_response.json().await?;
    assert!(!refresh_data.access_token.is_empty());
    assert!(refresh_data.expires_in > 0);

    // Verify new access token works
    let authed_client = TestClient::new(&ctx.auth_url).with_auth(&refresh_data.access_token);
    let me_response = authed_client.get("/api/v1/auth/me").await?;
    assert_eq!(me_response.status(), 200);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_register_duplicate_email_fails() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.auth_url);
    let email = format!("duplicate-{}@example.com", uuid::Uuid::new_v4());
    let password = "SecurePassword123!";

    // First registration succeeds
    let register_req = RegisterRequest {
        email: email.clone(),
        password: password.to_string(),
    };

    let first_response = client.post("/api/v1/auth/register", &register_req).await?;
    assert_eq!(first_response.status(), 201);

    // Second registration with same email fails
    let second_response = client.post("/api/v1/auth/register", &register_req).await?;
    assert_eq!(second_response.status(), 409); // Conflict

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_login_with_invalid_credentials_fails() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.auth_url);
    let email = format!("invalid-{}@example.com", uuid::Uuid::new_v4());

    // Register user
    let register_req = RegisterRequest {
        email: email.clone(),
        password: "CorrectPassword123!".to_string(),
    };
    client.post("/api/v1/auth/register", &register_req).await?;

    // Verify user
    let verify_req = VerifyRequest {
        token: "simulated-verification-token".to_string(),
    };
    client.post("/api/v1/auth/verify", &verify_req).await?;

    // Login with wrong password
    let login_req = LoginRequest {
        email: email.clone(),
        password: "WrongPassword123!".to_string(),
    };

    let login_response = client.post("/api/v1/auth/login", &login_req).await?;
    assert_eq!(login_response.status(), 401); // Unauthorized

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_login_unverified_user_fails() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.auth_url);
    let email = format!("unverified-{}@example.com", uuid::Uuid::new_v4());
    let password = "SecurePassword123!";

    // Register but don't verify
    let register_req = RegisterRequest {
        email: email.clone(),
        password: password.to_string(),
    };
    client.post("/api/v1/auth/register", &register_req).await?;

    // Try to login without verification
    let login_req = LoginRequest {
        email: email.clone(),
        password: password.to_string(),
    };

    let login_response = client.post("/api/v1/auth/login", &login_req).await?;
    assert_eq!(login_response.status(), 403); // Forbidden - verification required

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_refresh_with_invalid_token_fails() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.auth_url);

    let refresh_req = RefreshRequest {
        refresh_token: "invalid-refresh-token".to_string(),
    };

    let refresh_response = client.post("/api/v1/auth/refresh", &refresh_req).await?;
    assert_eq!(refresh_response.status(), 401); // Unauthorized

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_protected_endpoint_without_auth_fails() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.auth_url);
    let me_response = client.get("/api/v1/auth/me").await?;
    assert_eq!(me_response.status(), 401); // Unauthorized

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_protected_endpoint_with_expired_token_fails() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Create an expired token (this would be generated by auth service in real scenario)
    let expired_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyLCJleHAiOjE1MTYyMzkwMjJ9.4Adcj0vt2wTY7n3bvKw5n1TZL0l4lZJ2lXKJ5Qs1Y2Y";

    let client = TestClient::new(&ctx.auth_url).with_auth(expired_token);
    let me_response = client.get("/api/v1/auth/me").await?;
    assert_eq!(me_response.status(), 401); // Unauthorized

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_logout_invalidates_token() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.auth_url);
    let email = format!("logout-{}@example.com", uuid::Uuid::new_v4());
    let password = "SecurePassword123!";

    // Register and verify
    let register_req = RegisterRequest {
        email: email.clone(),
        password: password.to_string(),
    };
    client.post("/api/v1/auth/register", &register_req).await?;

    let verify_req = VerifyRequest {
        token: "simulated-verification-token".to_string(),
    };
    client.post("/api/v1/auth/verify", &verify_req).await?;

    // Login
    let login_req = LoginRequest {
        email: email.clone(),
        password: password.to_string(),
    };
    let login_response = client.post("/api/v1/auth/login", &login_req).await?;
    let login_data: LoginResponse = login_response.json().await?;

    // Verify token works
    let authed_client = TestClient::new(&ctx.auth_url).with_auth(&login_data.access_token);
    let me_response = authed_client.get("/api/v1/auth/me").await?;
    assert_eq!(me_response.status(), 200);

    // Logout
    let logout_response = authed_client
        .post("/api/v1/auth/logout", &json!({}))
        .await?;
    assert_eq!(logout_response.status(), 200);

    // Verify token no longer works
    let me_response_after = authed_client.get("/api/v1/auth/me").await?;
    assert_eq!(me_response_after.status(), 401);

    ctx.teardown().await?;
    Ok(())
}
