//! STDIO transport implementation for MCP server
//!
//! This transport layer enables the MCP server to communicate via standard input/output,
//! which is required for Claude Desktop integration and other MCP clients.

use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::McpServerState;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};

/// Run MCP server with STDIO transport
///
/// This function starts the MCP server using standard input/output for communication.
/// It reads JSON-RPC requests from stdin (one per line) and writes responses to stdout.
///
/// # Arguments
///
/// * `state` - Shared server state containing database pool and resource manager
///
/// # Protocol
///
/// - Input: JSON-RPC 2.0 requests, one per line
/// - Output: JSON-RPC 2.0 responses, one per line
/// - Empty lines are ignored
///
/// # Errors
///
/// Returns error if:
/// - I/O operations fail
/// - JSON parsing fails for requests
/// - Response serialization fails
pub async fn run_stdio_server(state: Arc<McpServerState>) -> Result<()> {
    info!("Starting MCP server with STDIO transport");

    let stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        debug!(request = %line, "Received STDIO request");

        // Parse JSON-RPC request
        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                // Handle the request using existing handlers
                let response = handle_request(state.clone(), request).await;

                // Serialize and write response
                match serde_json::to_string(&response) {
                    Ok(output) => {
                        if let Err(e) = stdout.write_all(output.as_bytes()).await {
                            error!(error = %e, "Failed to write response");
                            break;
                        }
                        if let Err(e) = stdout.write_all(b"\n").await {
                            error!(error = %e, "Failed to write newline");
                            break;
                        }
                        if let Err(e) = stdout.flush().await {
                            error!(error = %e, "Failed to flush output");
                            break;
                        }
                        debug!(response = %output, "Sent STDIO response");
                    }
                    Err(e) => {
                        error!(error = %e, "Failed to serialize response");
                        // Try to send an error response
                        let error_response = JsonRpcResponse::error(
                            response.id,
                            JsonRpcError::internal_error(format!("Serialization error: {}", e)),
                        );
                        if let Ok(output) = serde_json::to_string(&error_response) {
                            let _ = stdout.write_all(output.as_bytes()).await;
                            let _ = stdout.write_all(b"\n").await;
                            let _ = stdout.flush().await;
                        }
                    }
                }
            }
            Err(e) => {
                warn!(error = %e, line = %line, "Parse error");
                // Send parse error response
                let error_response = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    },
                    "id": null
                });
                if let Ok(output) = serde_json::to_string(&error_response) {
                    let _ = stdout.write_all(output.as_bytes()).await;
                    let _ = stdout.write_all(b"\n").await;
                    let _ = stdout.flush().await;
                }
            }
        }
    }

    info!("STDIO transport shutting down");
    Ok(())
}

/// Handle a JSON-RPC request
///
/// This is a wrapper around the existing HTTP handlers that adapts them
/// for STDIO transport. It dispatches requests to the appropriate handler
/// based on the method name.
async fn handle_request(state: Arc<McpServerState>, request: JsonRpcRequest) -> JsonRpcResponse {
    debug!(method = %request.method, id = ?request.id, "Processing request");

    match request.method.as_str() {
        "initialize" => handle_initialize(request.id, request.params).await,
        "tools/list" => handle_tools_list(request.id).await,
        "tools/call" => handle_tools_call(state, request.id, request.params).await,
        "resources/list" => handle_resources_list(request.id).await,
        "resources/read" => handle_resources_read(state, request.id, request.params).await,
        "prompts/list" => handle_prompts_list(request.id).await,
        "prompts/get" => handle_prompts_get(request.id, request.params).await,
        _ => {
            warn!(method = %request.method, "Unknown method");
            JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(request.method))
        }
    }
}

/// Handle initialize request (inline implementation)
async fn handle_initialize(
    id: crate::protocol::RequestId,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    use crate::protocol::{
        InitializeParams, InitializeResult, PromptsCapability, ResourcesCapability,
        ServerCapabilities, ServerInfo, ToolsCapability, MCP_VERSION,
    };

    info!("Initializing MCP server");

    let _params: InitializeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                error!(error = %e, "Invalid initialize params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)),
                );
            }
        },
        None => {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_params("Missing parameters"));
        }
    };

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
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    JsonRpcResponse::success(id, json!(result))
}

/// Handle tools/list request
async fn handle_tools_list(id: crate::protocol::RequestId) -> JsonRpcResponse {
    use crate::protocol::ToolListResult;

    debug!("Listing tools");

    let tools = vec![
        crate::tools::SemanticSearchTool::definition(),
        crate::tools::GetRecommendationsTool::definition(),
        crate::tools::CheckAvailabilityTool::definition(),
        crate::tools::GetContentDetailsTool::definition(),
        crate::tools::SyncWatchlistTool::definition(),
    ];

    let result = ToolListResult { tools };

    JsonRpcResponse::success(id, json!(result))
}

/// Handle tools/call request
async fn handle_tools_call(
    state: Arc<McpServerState>,
    id: crate::protocol::RequestId,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    use crate::protocol::ToolParams;
    use crate::tools::ToolExecutor;

    let tool_params: ToolParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                error!(error = %e, "Invalid tool params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)),
                );
            }
        },
        None => {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_params("Missing parameters"));
        }
    };

    info!(tool = %tool_params.name, "Calling tool");

    let executor: Box<dyn ToolExecutor> = match tool_params.name.as_str() {
        "semantic_search" => Box::new(crate::tools::SemanticSearchTool::new(state.db_pool.clone())),
        "get_recommendations" => Box::new(crate::tools::GetRecommendationsTool::new(
            state.db_pool.clone(),
        )),
        "check_availability" => Box::new(crate::tools::CheckAvailabilityTool::new(
            state.db_pool.clone(),
        )),
        "get_content_details" => Box::new(crate::tools::GetContentDetailsTool::new(
            state.db_pool.clone(),
        )),
        "sync_watchlist" => Box::new(crate::tools::SyncWatchlistTool::new(state.db_pool.clone())),
        _ => {
            return JsonRpcResponse::error(
                id,
                JsonRpcError::method_not_found(format!("Unknown tool: {}", tool_params.name)),
            );
        }
    };

    let arguments = tool_params.arguments.unwrap_or_default();

    match executor.execute(arguments).await {
        Ok(result) => JsonRpcResponse::success(id, json!(result)),
        Err(e) => {
            error!(error = %e, tool = %tool_params.name, "Tool execution failed");
            JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string()))
        }
    }
}

/// Handle resources/list request
async fn handle_resources_list(id: crate::protocol::RequestId) -> JsonRpcResponse {
    use crate::protocol::ResourceListResult;
    use crate::resources::ResourceManager;

    debug!("Listing resources");

    let resources = ResourceManager::list_resources();
    let result = ResourceListResult { resources };

    JsonRpcResponse::success(id, json!(result))
}

/// Handle resources/read request
async fn handle_resources_read(
    state: Arc<McpServerState>,
    id: crate::protocol::RequestId,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    use crate::protocol::ResourceParams;

    let resource_params: ResourceParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                error!(error = %e, "Invalid resource params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)),
                );
            }
        },
        None => {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_params("Missing parameters"));
        }
    };

    info!(uri = %resource_params.uri, "Reading resource");

    match state
        .resource_manager
        .read_resource(&resource_params.uri)
        .await
    {
        Ok(content) => JsonRpcResponse::success(id, json!(content)),
        Err(e) => {
            error!(error = %e, uri = %resource_params.uri, "Resource read failed");
            JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string()))
        }
    }
}

/// Handle prompts/list request
async fn handle_prompts_list(id: crate::protocol::RequestId) -> JsonRpcResponse {
    use crate::protocol::{Prompt, PromptArgument, PromptListResult};

    debug!("Listing prompts");

    let prompts = vec![
        Prompt {
            name: "discover_content".to_string(),
            description: "Discover new content based on preferences".to_string(),
            arguments: Some(vec![
                PromptArgument {
                    name: "genre".to_string(),
                    description: "Preferred genre".to_string(),
                    required: false,
                },
                PromptArgument {
                    name: "mood".to_string(),
                    description: "Current mood or viewing preference".to_string(),
                    required: false,
                },
            ]),
        },
        Prompt {
            name: "find_similar".to_string(),
            description: "Find content similar to a given title".to_string(),
            arguments: Some(vec![PromptArgument {
                name: "reference_title".to_string(),
                description: "Title to find similar content for".to_string(),
                required: true,
            }]),
        },
        Prompt {
            name: "watchlist_suggestions".to_string(),
            description: "Get suggestions to add to watchlist".to_string(),
            arguments: Some(vec![PromptArgument {
                name: "user_id".to_string(),
                description: "User UUID".to_string(),
                required: true,
            }]),
        },
    ];

    let result = PromptListResult { prompts };

    JsonRpcResponse::success(id, json!(result))
}

/// Handle prompts/get request
async fn handle_prompts_get(
    id: crate::protocol::RequestId,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    use crate::protocol::PromptParams;

    let prompt_params: PromptParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(params) => params,
            Err(e) => {
                error!(error = %e, "Invalid prompt params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params(format!("Invalid parameters: {}", e)),
                );
            }
        },
        None => {
            return JsonRpcResponse::error(id, JsonRpcError::invalid_params("Missing parameters"));
        }
    };

    info!(prompt = %prompt_params.name, "Getting prompt");

    let prompt_text = match prompt_params.name.as_str() {
        "discover_content" => {
            let genre = prompt_params
                .arguments
                .as_ref()
                .and_then(|a| a.get("genre"))
                .and_then(|v| v.as_str())
                .unwrap_or("any");
            let mood = prompt_params
                .arguments
                .as_ref()
                .and_then(|a| a.get("mood"))
                .and_then(|v| v.as_str())
                .unwrap_or("general");

            format!(
                "Discover {} content that matches a {} mood. Use semantic search to find relevant titles.",
                genre, mood
            )
        }
        "find_similar" => {
            let reference = prompt_params
                .arguments
                .as_ref()
                .and_then(|a| a.get("reference_title"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            format!(
                "Find content similar to '{}'. Analyze themes, genre, and style.",
                reference
            )
        }
        "watchlist_suggestions" => {
            "Generate personalized watchlist suggestions based on user viewing history and preferences.".to_string()
        }
        _ => {
            return JsonRpcResponse::error(
                id,
                JsonRpcError::method_not_found(format!("Unknown prompt: {}", prompt_params.name)),
            );
        }
    };

    JsonRpcResponse::success(id, json!({ "text": prompt_text }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::JSONRPC_VERSION;

    #[test]
    fn test_request_id_serialization() {
        use crate::protocol::RequestId;

        let id = RequestId::String("test-123".to_string());
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, r#""test-123""#);

        let id = RequestId::Number(42);
        let serialized = serde_json::to_string(&id).unwrap();
        assert_eq!(serialized, "42");
    }

    #[test]
    fn test_parse_jsonrpc_request() {
        let json = r#"{"jsonrpc":"2.0","id":"1","method":"initialize","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "initialize");
        assert_eq!(request.jsonrpc, JSONRPC_VERSION);
    }
}
