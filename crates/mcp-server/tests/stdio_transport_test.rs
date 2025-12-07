//! STDIO transport integration tests
//!
//! Tests for the STDIO transport layer of the MCP server.

use media_gateway_mcp::protocol::*;
use serde_json::json;

/// Test JSON-RPC request parsing
#[test]
fn test_jsonrpc_request_parsing() {
    let json = r#"{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocol_version":"1.0","capabilities":{},"client_info":{"name":"test","version":"1.0"}}}"#;
    let request: Result<JsonRpcRequest, _> = serde_json::from_str(json);
    assert!(request.is_ok());

    let request = request.unwrap();
    assert_eq!(request.method, "initialize");
    assert_eq!(request.jsonrpc, "2.0");
}

/// Test JSON-RPC response serialization
#[test]
fn test_jsonrpc_response_serialization() {
    let response = JsonRpcResponse::success(
        RequestId::String("test-123".to_string()),
        json!({"result": "success"}),
    );

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
    assert!(serialized.contains("\"id\":\"test-123\""));
    assert!(serialized.contains("\"result\""));
}

/// Test error response creation
#[test]
fn test_error_response_creation() {
    let error = JsonRpcError::method_not_found("unknown_method");
    let response = JsonRpcResponse::error(RequestId::Number(42), error);

    assert_eq!(response.jsonrpc, "2.0");
    assert_eq!(response.id, RequestId::Number(42));
    assert!(response.error.is_some());
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32601);
    assert!(error.message.contains("unknown_method"));
}

/// Test parse error handling
#[test]
fn test_parse_error_handling() {
    let error = JsonRpcError::parse_error("Invalid JSON");
    assert_eq!(error.code, -32700);
    assert_eq!(error.message, "Invalid JSON");
}

/// Test invalid params error
#[test]
fn test_invalid_params_error() {
    let error = JsonRpcError::invalid_params("Missing required field");
    assert_eq!(error.code, -32602);
    assert_eq!(error.message, "Missing required field");
}

/// Test internal error
#[test]
fn test_internal_error() {
    let error = JsonRpcError::internal_error("Database connection failed");
    assert_eq!(error.code, -32603);
    assert_eq!(error.message, "Database connection failed");
}

/// Test RequestId variants serialization
#[test]
fn test_request_id_variants() {
    // String ID
    let id = RequestId::String("abc-123".to_string());
    let serialized = serde_json::to_string(&id).unwrap();
    assert_eq!(serialized, r#""abc-123""#);

    // Number ID
    let id = RequestId::Number(999);
    let serialized = serde_json::to_string(&id).unwrap();
    assert_eq!(serialized, "999");

    // Null ID
    let id = RequestId::Null;
    let serialized = serde_json::to_string(&id).unwrap();
    assert_eq!(serialized, "null");
}

/// Test initialize request structure
#[test]
fn test_initialize_request_structure() {
    let init_params = InitializeParams {
        protocol_version: "1.0".to_string(),
        capabilities: ClientCapabilities { experimental: None },
        client_info: ClientInfo {
            name: "Test Client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let serialized = serde_json::to_string(&init_params).unwrap();
    assert!(serialized.contains("protocol_version"));
    assert!(serialized.contains("capabilities"));
    assert!(serialized.contains("client_info"));
}

/// Test initialize result structure
#[test]
fn test_initialize_result_structure() {
    let result = InitializeResult {
        protocol_version: MCP_VERSION.to_string(),
        capabilities: ServerCapabilities {
            tools: ToolsCapability {
                list_changed: false,
            },
            resources: ResourcesCapability {
                subscribe: false,
                list_changed: false,
            },
            prompts: PromptsCapability {
                list_changed: false,
            },
        },
        server_info: ServerInfo {
            name: "Media Gateway MCP Server".to_string(),
            version: "0.1.0".to_string(),
        },
    };

    let serialized = serde_json::to_string(&result).unwrap();
    assert!(serialized.contains("protocol_version"));
    assert!(serialized.contains("capabilities"));
    assert!(serialized.contains("server_info"));
}

/// Test tool parameters parsing
#[test]
fn test_tool_parameters_parsing() {
    let json = r#"{"name":"semantic_search","arguments":{"query":"test query","limit":10}}"#;
    let params: Result<ToolParams, _> = serde_json::from_str(json);
    assert!(params.is_ok());

    let params = params.unwrap();
    assert_eq!(params.name, "semantic_search");
    assert!(params.arguments.is_some());

    let args = params.arguments.unwrap();
    assert_eq!(args.get("query").unwrap().as_str().unwrap(), "test query");
    assert_eq!(args.get("limit").unwrap().as_i64().unwrap(), 10);
}

/// Test resource parameters parsing
#[test]
fn test_resource_parameters_parsing() {
    let json = r#"{"uri":"content://123"}"#;
    let params: Result<ResourceParams, _> = serde_json::from_str(json);
    assert!(params.is_ok());

    let params = params.unwrap();
    assert_eq!(params.uri, "content://123");
}

/// Test prompt parameters parsing
#[test]
fn test_prompt_parameters_parsing() {
    let json = r#"{"name":"discover_content","arguments":{"genre":"action","mood":"exciting"}}"#;
    let params: Result<PromptParams, _> = serde_json::from_str(json);
    assert!(params.is_ok());

    let params = params.unwrap();
    assert_eq!(params.name, "discover_content");
    assert!(params.arguments.is_some());

    let args = params.arguments.unwrap();
    assert_eq!(args.get("genre").unwrap().as_str().unwrap(), "action");
    assert_eq!(args.get("mood").unwrap().as_str().unwrap(), "exciting");
}

/// Test complete request-response cycle serialization
#[test]
fn test_request_response_cycle() {
    // Create a request
    let request = JsonRpcRequest {
        jsonrpc: JSONRPC_VERSION.to_string(),
        id: RequestId::String("req-1".to_string()),
        method: "tools/list".to_string(),
        params: None,
    };

    // Serialize request
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains("tools/list"));

    // Parse it back
    let parsed_request: JsonRpcRequest = serde_json::from_str(&request_json).unwrap();
    assert_eq!(parsed_request.method, request.method);
    assert_eq!(parsed_request.id, request.id);

    // Create a response
    let response = JsonRpcResponse::success(parsed_request.id.clone(), json!({"tools": []}));

    // Serialize response
    let response_json = serde_json::to_string(&response).unwrap();
    assert!(response_json.contains("\"tools\":[]"));

    // Parse it back
    let parsed_response: JsonRpcResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(parsed_response.id, request.id);
    assert!(parsed_response.result.is_some());
    assert!(parsed_response.error.is_none());
}
