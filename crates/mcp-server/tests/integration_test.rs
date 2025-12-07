//! Integration tests for MCP Server
//!
//! These tests verify the JSON-RPC protocol implementation and tool execution.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use media_gateway_mcp::{handlers, McpServerState};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper function to create test app
async fn create_test_app(db_pool: PgPool) -> Router {
    let state = Arc::new(McpServerState::new(db_pool));
    Router::new()
        .route("/", axum::routing::post(handlers::handle_jsonrpc))
        .route("/health", axum::routing::get(handlers::health_check))
        .with_state(state)
}

/// Helper function to send JSON-RPC request
async fn send_jsonrpc_request(
    app: Router,
    method: &str,
    params: Option<serde_json::Value>,
) -> (StatusCode, serde_json::Value) {
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    (status, json)
}

#[sqlx::test]
async fn test_health_check(pool: PgPool) {
    let app = create_test_app(pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "healthy");
}

#[sqlx::test]
async fn test_initialize(pool: PgPool) {
    let app = create_test_app(pool).await;

    let params = json!({
        "protocol_version": "1.0",
        "capabilities": {},
        "client_info": {
            "name": "test-client",
            "version": "1.0.0"
        }
    });

    let (status, response) = send_jsonrpc_request(app, "initialize", Some(params)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["protocol_version"], "1.0");
    assert!(response["result"]["capabilities"].is_object());
    assert!(response["result"]["server_info"].is_object());
}

#[sqlx::test]
async fn test_tools_list(pool: PgPool) {
    let app = create_test_app(pool).await;

    let (status, response) = send_jsonrpc_request(app, "tools/list", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 5);

    // Verify expected tools are present
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(tool_names.contains(&"semantic_search"));
    assert!(tool_names.contains(&"get_recommendations"));
    assert!(tool_names.contains(&"check_availability"));
    assert!(tool_names.contains(&"get_content_details"));
    assert!(tool_names.contains(&"sync_watchlist"));
    assert!(tool_names.contains(&"list_devices"));
}

#[sqlx::test]
async fn test_resources_list(pool: PgPool) {
    let app = create_test_app(pool).await;

    let (status, response) = send_jsonrpc_request(app, "resources/list", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"]["resources"].is_array());

    let resources = response["result"]["resources"].as_array().unwrap();
    assert!(resources.len() >= 3);

    // Verify expected resources are present
    let resource_uris: Vec<&str> = resources.iter().filter_map(|r| r["uri"].as_str()).collect();

    assert!(resource_uris.contains(&"content://catalog"));
    assert!(resource_uris
        .iter()
        .any(|&uri| uri.starts_with("user://preferences/")));
    assert!(resource_uris
        .iter()
        .any(|&uri| uri.starts_with("content://item/")));
}

#[sqlx::test]
async fn test_prompts_list(pool: PgPool) {
    let app = create_test_app(pool).await;

    let (status, response) = send_jsonrpc_request(app, "prompts/list", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"]["prompts"].is_array());

    let prompts = response["result"]["prompts"].as_array().unwrap();
    assert!(prompts.len() >= 3);

    // Verify expected prompts are present
    let prompt_names: Vec<&str> = prompts.iter().filter_map(|p| p["name"].as_str()).collect();

    assert!(prompt_names.contains(&"discover_content"));
    assert!(prompt_names.contains(&"find_similar"));
    assert!(prompt_names.contains(&"watchlist_suggestions"));
}

#[sqlx::test]
async fn test_method_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;

    let (status, response) = send_jsonrpc_request(app, "invalid_method", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601);
}

#[sqlx::test]
async fn test_invalid_params(pool: PgPool) {
    let app = create_test_app(pool).await;

    // Initialize without required params
    let (status, response) = send_jsonrpc_request(app, "initialize", None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32602);
}

#[sqlx::test(fixtures("test_content"))]
async fn test_semantic_search_tool(pool: PgPool) {
    let app = create_test_app(pool).await;

    let params = json!({
        "name": "semantic_search",
        "arguments": {
            "query": "action movies",
            "limit": 5
        }
    });

    let (status, response) = send_jsonrpc_request(app, "tools/call", Some(params)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert!(response["result"]["content"].is_array());
}

#[sqlx::test]
async fn test_prompts_get(pool: PgPool) {
    let app = create_test_app(pool).await;

    let params = json!({
        "name": "discover_content",
        "arguments": {
            "genre": "sci-fi",
            "mood": "exciting"
        }
    });

    let (status, response) = send_jsonrpc_request(app, "prompts/get", Some(params)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert!(response["result"]["text"].is_string());

    let text = response["result"]["text"].as_str().unwrap();
    assert!(text.contains("sci-fi"));
    assert!(text.contains("exciting"));
}

#[sqlx::test(fixtures("test_devices"))]
async fn test_list_devices_tool(pool: PgPool) {
    let app = create_test_app(pool).await;

    // Get the test user ID from the fixture
    let user_id = "123e4567-e89b-12d3-a456-426614174000"; // Test user ID from fixture

    let params = json!({
        "name": "list_devices",
        "arguments": {
            "user_id": user_id
        }
    });

    let (status, response) = send_jsonrpc_request(app, "tools/call", Some(params)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert!(response["result"]["content"].is_array());
    assert_eq!(response["result"]["is_error"], false);

    // Parse the text content to verify device information
    let content = response["result"]["content"].as_array().unwrap();
    assert_eq!(content.len(), 1);

    let text_content = &content[0];
    assert_eq!(text_content["type"], "text");

    let text = text_content["text"].as_str().unwrap();
    let device_data: serde_json::Value = serde_json::from_str(text).unwrap();

    assert_eq!(device_data["user_id"], user_id);
    assert!(device_data["device_count"].is_number());
    assert!(device_data["devices"].is_array());

    let devices = device_data["devices"].as_array().unwrap();
    if !devices.is_empty() {
        let first_device = &devices[0];
        assert!(first_device["device_id"].is_string());
        assert!(first_device["device_type"].is_string());
        assert!(first_device["platform"].is_string());
        assert!(first_device["capabilities"].is_object());
        assert!(first_device["last_seen"].is_string());
        assert!(first_device["is_online"].is_boolean());
    }
}

#[sqlx::test]
async fn test_list_devices_tool_empty(pool: PgPool) {
    let app = create_test_app(pool).await;

    // Use a random UUID that shouldn't have any devices
    let user_id = Uuid::new_v4().to_string();

    let params = json!({
        "name": "list_devices",
        "arguments": {
            "user_id": user_id
        }
    });

    let (status, response) = send_jsonrpc_request(app, "tools/call", Some(params)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert!(response["result"]["content"].is_array());

    // Parse the response
    let content = response["result"]["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();
    let device_data: serde_json::Value = serde_json::from_str(text).unwrap();

    assert_eq!(device_data["device_count"], 0);
    assert_eq!(device_data["devices"].as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn test_list_devices_tool_invalid_uuid(pool: PgPool) {
    let app = create_test_app(pool).await;

    let params = json!({
        "name": "list_devices",
        "arguments": {
            "user_id": "invalid-uuid"
        }
    });

    let (status, response) = send_jsonrpc_request(app, "tools/call", Some(params)).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32602); // Invalid params
}
