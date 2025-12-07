use anyhow::Result;
use media_gateway_tests::{fixtures, TestClient, TestContext};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct SearchRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<ContentResult>,
    total: i64,
    limit: i32,
    offset: i32,
}

#[derive(Debug, Deserialize)]
struct ContentResult {
    id: String,
    title: String,
    content_type: String,
    url: String,
    metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ContentDetail {
    id: String,
    title: String,
    content_type: String,
    url: String,
    metadata: serde_json::Value,
    views: i64,
}

#[derive(Debug, Serialize)]
struct TrackViewRequest {
    content_id: String,
}

#[tokio::test]
async fn test_search_flow_search_get_details_track_view() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Create test content
    let content1 =
        fixtures::create_test_content_with_type(&ctx, "video/mp4", "Action Movie 2024").await?;
    let content2 =
        fixtures::create_test_content_with_type(&ctx, "video/mp4", "Action Series Episode 1")
            .await?;
    let content3 =
        fixtures::create_test_content_with_type(&ctx, "audio/mp3", "Action Podcast").await?;

    // Create test user for authenticated search
    let user = fixtures::create_test_user(&ctx).await?;
    let auth_token = "test-auth-token"; // In real scenario, get from auth service

    let client = TestClient::new(&ctx.discovery_url).with_auth(auth_token);

    // Step 1: Search for "Action" content
    let search_req = SearchRequest {
        query: "Action".to_string(),
        content_type: None,
        limit: Some(10),
        offset: Some(0),
    };

    let search_response = client
        .get(&format!(
            "/api/v1/search?query={}&limit={}&offset={}",
            search_req.query,
            search_req.limit.unwrap(),
            search_req.offset.unwrap()
        ))
        .await?;
    assert_eq!(search_response.status(), 200);

    let search_data: SearchResponse = search_response.json().await?;
    assert_eq!(search_data.total, 3);
    assert_eq!(search_data.results.len(), 3);

    // Step 2: Get details for first result
    let first_result = &search_data.results[0];
    let details_response = client
        .get(&format!("/api/v1/content/{}", first_result.id))
        .await?;
    assert_eq!(details_response.status(), 200);

    let details_data: ContentDetail = details_response.json().await?;
    assert_eq!(details_data.id, first_result.id);
    assert_eq!(details_data.title, first_result.title);

    // Step 3: Track view
    let track_req = TrackViewRequest {
        content_id: first_result.id.clone(),
    };

    let track_response = client.post("/api/v1/analytics/view", &track_req).await?;
    assert_eq!(track_response.status(), 200);

    // Verify view count increased
    let updated_details = client
        .get(&format!("/api/v1/content/{}", first_result.id))
        .await?;
    let updated_data: ContentDetail = updated_details.json().await?;
    assert_eq!(updated_data.views, details_data.views + 1);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_with_content_type_filter() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Create mixed content types
    fixtures::create_test_content_with_type(&ctx, "video/mp4", "Video Content").await?;
    fixtures::create_test_content_with_type(&ctx, "audio/mp3", "Audio Content").await?;
    fixtures::create_test_content_with_type(&ctx, "video/mp4", "Another Video").await?;

    let client = TestClient::new(&ctx.discovery_url);

    // Search for videos only
    let search_response = client
        .get("/api/v1/search?query=Content&content_type=video/mp4")
        .await?;
    assert_eq!(search_response.status(), 200);

    let search_data: SearchResponse = search_response.json().await?;
    assert_eq!(search_data.total, 2);
    for result in &search_data.results {
        assert_eq!(result.content_type, "video/mp4");
    }

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_pagination() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Create 5 items
    for i in 1..=5 {
        fixtures::create_test_content_with_type(&ctx, "video/mp4", &format!("Test Video {}", i))
            .await?;
    }

    let client = TestClient::new(&ctx.discovery_url);

    // Get first page (2 items)
    let page1_response = client
        .get("/api/v1/search?query=Test&limit=2&offset=0")
        .await?;
    let page1_data: SearchResponse = page1_response.json().await?;
    assert_eq!(page1_data.results.len(), 2);
    assert_eq!(page1_data.total, 5);
    assert_eq!(page1_data.limit, 2);
    assert_eq!(page1_data.offset, 0);

    // Get second page (2 items)
    let page2_response = client
        .get("/api/v1/search?query=Test&limit=2&offset=2")
        .await?;
    let page2_data: SearchResponse = page2_response.json().await?;
    assert_eq!(page2_data.results.len(), 2);
    assert_eq!(page2_data.total, 5);
    assert_eq!(page2_data.offset, 2);

    // Get third page (1 item)
    let page3_response = client
        .get("/api/v1/search?query=Test&limit=2&offset=4")
        .await?;
    let page3_data: SearchResponse = page3_response.json().await?;
    assert_eq!(page3_data.results.len(), 1);
    assert_eq!(page3_data.total, 5);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_no_results() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    fixtures::create_test_content_with_type(&ctx, "video/mp4", "Action Movie").await?;

    let client = TestClient::new(&ctx.discovery_url);

    let search_response = client.get("/api/v1/search?query=NonExistent").await?;
    assert_eq!(search_response.status(), 200);

    let search_data: SearchResponse = search_response.json().await?;
    assert_eq!(search_data.total, 0);
    assert_eq!(search_data.results.len(), 0);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_get_content_details_not_found() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.discovery_url);
    let non_existent_id = Uuid::new_v4();

    let details_response = client
        .get(&format!("/api/v1/content/{}", non_existent_id))
        .await?;
    assert_eq!(details_response.status(), 404);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_history_tracking() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let user = fixtures::create_test_user(&ctx).await?;
    let auth_token = "test-auth-token";
    let client = TestClient::new(&ctx.discovery_url).with_auth(auth_token);

    // Perform multiple searches
    client.get("/api/v1/search?query=Action").await?;
    client.get("/api/v1/search?query=Comedy").await?;
    client.get("/api/v1/search?query=Drama").await?;

    // Get search history
    let history_response = client.get("/api/v1/search/history").await?;
    assert_eq!(history_response.status(), 200);

    let history: Vec<serde_json::Value> = history_response.json().await?;
    assert_eq!(history.len(), 3);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_search_with_invalid_limit() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    let client = TestClient::new(&ctx.discovery_url);

    // Limit too high
    let response = client.get("/api/v1/search?query=Test&limit=1000").await?;
    assert_eq!(response.status(), 400); // Bad Request

    // Negative limit
    let response = client.get("/api/v1/search?query=Test&limit=-1").await?;
    assert_eq!(response.status(), 400);

    ctx.teardown().await?;
    Ok(())
}

#[tokio::test]
async fn test_popular_content_endpoint() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Create content and track views
    let content1 =
        fixtures::create_test_content_with_type(&ctx, "video/mp4", "Popular Video").await?;
    let content2 =
        fixtures::create_test_content_with_type(&ctx, "video/mp4", "Less Popular").await?;

    let client = TestClient::new(&ctx.discovery_url);

    // Track views for content1 (3 times)
    for _ in 0..3 {
        client
            .post(
                "/api/v1/analytics/view",
                &TrackViewRequest {
                    content_id: content1.id.to_string(),
                },
            )
            .await?;
    }

    // Track views for content2 (1 time)
    client
        .post(
            "/api/v1/analytics/view",
            &TrackViewRequest {
                content_id: content2.id.to_string(),
            },
        )
        .await?;

    // Get popular content
    let popular_response = client.get("/api/v1/content/popular?limit=10").await?;
    assert_eq!(popular_response.status(), 200);

    let popular_data: Vec<ContentDetail> = popular_response.json().await?;
    assert_eq!(popular_data.len(), 2);

    // Most popular should be first
    assert_eq!(popular_data[0].id, content1.id.to_string());
    assert!(popular_data[0].views >= popular_data[1].views);

    ctx.teardown().await?;
    Ok(())
}
