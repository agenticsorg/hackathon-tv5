use anyhow::Result;
use media_gateway_tests::{fixtures, TestClient, TestContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct CreateSessionRequest {
    content_id: String,
    user_id: String,
}

#[derive(Debug, Deserialize)]
struct SessionResponse {
    id: String,
    user_id: String,
    content_id: String,
    position_seconds: i32,
    duration_seconds: Option<i32>,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Serialize)]
struct UpdatePositionRequest {
    position_seconds: i32,
}

#[derive(Debug, Deserialize)]
struct PositionUpdateResponse {
    id: String,
    position_seconds: i32,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
struct ResumeResponse {
    id: String,
    content_id: String,
    position_seconds: i32,
    duration_seconds: Option<i32>,
}

#[tokio::test]
async fn test_playback_flow_create_update_resume() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Setup test data
    let user = fixtures::create_test_user(&ctx).await?;
    let content = fixtures::create_test_content(&ctx).await?;
    let auth_token = "test-auth-token";

    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    // Step 1: Create playback session
    let create_req = CreateSessionRequest {
        content_id: content.id.to_string(),
        user_id: user.id.to_string(),
    };

    let create_response = client
        .post("/api/v1/playback/sessions", &create_req)
        .await?;
    assert_eq!(create_response.status(), 201);

    let session_data: SessionResponse = create_response.json().await?;
    assert_eq!(session_data.content_id, content.id.to_string());
    assert_eq!(session_data.user_id, user.id.to_string());
    assert_eq!(session_data.position_seconds, 0);

    let session_id = session_data.id.clone();

    // Step 2: Update playback position
    let update_req = UpdatePositionRequest {
        position_seconds: 300, // 5 minutes
    };

    let update_response = client
        .patch(
            &format!("/api/v1/playback/sessions/{}/position", session_id),
            &update_req,
        )
        .await?;
    assert_eq!(update_response.status(), 200);

    let update_data: PositionUpdateResponse = update_response.json().await?;
    assert_eq!(update_data.position_seconds, 300);

    // Step 3: Update position again (simulate watching more)
    let update_req2 = UpdatePositionRequest {
        position_seconds: 600, // 10 minutes
    };

    let update_response2 = client
        .patch(
            &format!("/api/v1/playback/sessions/{}/position", session_id),
            &update_req2,
        )
        .await?;
    assert_eq!(update_response2.status(), 200);

    let update_data2: PositionUpdateResponse = update_response2.json().await?;
    assert_eq!(update_data2.position_seconds, 600);

    // Step 4: Resume playback (get latest session for content)
    let resume_response = client
        .get(&format!(
            "/api/v1/playback/sessions/resume?content_id={}",
            content.id
        ))
        .await?;
    assert_eq!(resume_response.status(), 200);

    let resume_data: ResumeResponse = resume_response.json().await?;
    assert_eq!(resume_data.content_id, content.id.to_string());
    assert_eq!(resume_data.position_seconds, 600);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_create_multiple_sessions_same_content() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let user = fixtures::create_test_user(&ctx).await?;
    let content = fixtures::create_test_content(&ctx).await?;
    let auth_token = "test-auth-token";

    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    // Create first session
    let create_req = CreateSessionRequest {
        content_id: content.id.to_string(),
        user_id: user.id.to_string(),
    };

    let response1 = client
        .post("/api/v1/playback/sessions", &create_req)
        .await?;
    assert_eq!(response1.status(), 201);
    let session1: SessionResponse = response1.json().await?;

    // Update position
    client
        .patch(
            &format!("/api/v1/playback/sessions/{}/position", session1.id),
            &UpdatePositionRequest {
                position_seconds: 100,
            },
        )
        .await?;

    // Create second session (new playback instance)
    let response2 = client
        .post("/api/v1/playback/sessions", &create_req)
        .await?;
    assert_eq!(response2.status(), 201);
    let session2: SessionResponse = response2.json().await?;

    // Sessions should have different IDs
    assert_ne!(session1.id, session2.id);

    // Resume should return the most recent session
    let resume_response = client
        .get(&format!(
            "/api/v1/playback/sessions/resume?content_id={}",
            content.id
        ))
        .await?;
    let resume_data: ResumeResponse = resume_response.json().await?;
    assert_eq!(resume_data.id, session2.id);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_update_position_invalid_session() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let auth_token = "test-auth-token";
    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    let invalid_session_id = uuid::Uuid::new_v4();
    let update_req = UpdatePositionRequest {
        position_seconds: 100,
    };

    let response = client
        .patch(
            &format!("/api/v1/playback/sessions/{}/position", invalid_session_id),
            &update_req,
        )
        .await?;
    assert_eq!(response.status(), 404);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_update_position_negative_value() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let user = fixtures::create_test_user(&ctx).await?;
    let content = fixtures::create_test_content(&ctx).await?;
    let auth_token = "test-auth-token";

    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    // Create session
    let create_req = CreateSessionRequest {
        content_id: content.id.to_string(),
        user_id: user.id.to_string(),
    };
    let create_response = client
        .post("/api/v1/playback/sessions", &create_req)
        .await?;
    let session_data: SessionResponse = create_response.json().await?;

    // Try to update with negative position
    let update_req = UpdatePositionRequest {
        position_seconds: -10,
    };

    let response = client
        .patch(
            &format!("/api/v1/playback/sessions/{}/position", session_data.id),
            &update_req,
        )
        .await?;
    assert_eq!(response.status(), 400); // Bad Request

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_resume_no_existing_session() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let content = fixtures::create_test_content(&ctx).await?;
    let auth_token = "test-auth-token";

    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    let response = client
        .get(&format!(
            "/api/v1/playback/sessions/resume?content_id={}",
            content.id
        ))
        .await?;
    assert_eq!(response.status(), 404);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_get_user_sessions() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let user = fixtures::create_test_user(&ctx).await?;
    let content1 = fixtures::create_test_content(&ctx).await?;
    let content2 = fixtures::create_test_content(&ctx).await?;
    let auth_token = "test-auth-token";

    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    // Create sessions for different content
    client
        .post(
            "/api/v1/playback/sessions",
            &CreateSessionRequest {
                content_id: content1.id.to_string(),
                user_id: user.id.to_string(),
            },
        )
        .await?;

    client
        .post(
            "/api/v1/playback/sessions",
            &CreateSessionRequest {
                content_id: content2.id.to_string(),
                user_id: user.id.to_string(),
            },
        )
        .await?;

    // Get all sessions for user
    let response = client
        .get(&format!("/api/v1/playback/sessions?user_id={}", user.id))
        .await?;
    assert_eq!(response.status(), 200);

    let sessions: Vec<SessionResponse> = response.json().await?;
    assert_eq!(sessions.len(), 2);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_session() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let user = fixtures::create_test_user(&ctx).await?;
    let content = fixtures::create_test_content(&ctx).await?;
    let auth_token = "test-auth-token";

    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    // Create session
    let create_req = CreateSessionRequest {
        content_id: content.id.to_string(),
        user_id: user.id.to_string(),
    };
    let create_response = client
        .post("/api/v1/playback/sessions", &create_req)
        .await?;
    let session_data: SessionResponse = create_response.json().await?;

    // Delete session
    let delete_response = client
        .delete(&format!("/api/v1/playback/sessions/{}", session_data.id))
        .await?;
    assert_eq!(delete_response.status(), 204);

    // Verify session is gone
    let get_response = client
        .get(&format!("/api/v1/playback/sessions/{}", session_data.id))
        .await?;
    assert_eq!(get_response.status(), 404);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_unauthorized_access() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let user = fixtures::create_test_user(&ctx).await?;
    let content = fixtures::create_test_content(&ctx).await?;

    // Client without auth token
    let client = TestClient::new(&ctx.playback_url);

    let create_req = CreateSessionRequest {
        content_id: content.id.to_string(),
        user_id: user.id.to_string(),
    };

    let response = client
        .post("/api/v1/playback/sessions", &create_req)
        .await?;
    assert_eq!(response.status(), 401); // Unauthorized

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_complete_session_when_position_reaches_end() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let user = fixtures::create_test_user(&ctx).await?;
    let content = fixtures::create_test_content(&ctx).await?;
    let auth_token = "test-auth-token";

    let client = TestClient::new(&ctx.playback_url).with_auth(auth_token);

    // Create session with duration
    let create_req = CreateSessionRequest {
        content_id: content.id.to_string(),
        user_id: user.id.to_string(),
    };
    let create_response = client
        .post("/api/v1/playback/sessions", &create_req)
        .await?;
    let session_data: SessionResponse = create_response.json().await?;

    // Update position to end of content (assuming 3600s duration)
    let update_req = UpdatePositionRequest {
        position_seconds: 3595, // Near end (within 5 seconds)
    };

    let update_response = client
        .patch(
            &format!("/api/v1/playback/sessions/{}/position", session_data.id),
            &update_req,
        )
        .await?;
    assert_eq!(update_response.status(), 200);

    // Session should be marked as completed
    let session_response = client
        .get(&format!("/api/v1/playback/sessions/{}", session_data.id))
        .await?;
    let session: serde_json::Value = session_response.json().await?;
    assert_eq!(session["is_completed"], true);

    ctx.teardown().await?;
    Ok(())
}
