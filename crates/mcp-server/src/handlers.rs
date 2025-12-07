//! JSON-RPC request handlers
//!
//! This module implements the JSON-RPC 2.0 protocol handlers for MCP methods.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

use crate::{
    protocol::{
        InitializeParams, InitializeResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, Prompt,
        PromptArgument, PromptListResult, PromptParams, PromptsCapability, RequestId,
        ResourceListResult, ResourceParams, ResourcesCapability, ServerCapabilities, ServerInfo,
        ToolListResult, ToolParams, ToolsCapability, MCP_VERSION,
    },
    resources::ResourceManager,
    tools::ToolExecutor,
    McpServerState,
};

/// Handle JSON-RPC request
#[instrument(skip(state, request))]
pub async fn handle_jsonrpc(
    State(state): State<Arc<McpServerState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    debug!(method = %request.method, id = ?request.id, "Processing JSON-RPC request");

    let response = match request.method.as_str() {
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
    };

    Json(response).into_response()
}

/// Handle initialize request
async fn handle_initialize(id: RequestId, params: Option<serde_json::Value>) -> JsonRpcResponse {
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
async fn handle_tools_list(id: RequestId) -> JsonRpcResponse {
    debug!("Listing tools");

    let tools = vec![
        crate::tools::SemanticSearchTool::definition(),
        crate::tools::GetRecommendationsTool::definition(),
        crate::tools::CheckAvailabilityTool::definition(),
        crate::tools::GetContentDetailsTool::definition(),
        crate::tools::SyncWatchlistTool::definition(),
        crate::tools::ListDevicesTool::definition(),
    ];

    let result = ToolListResult { tools };

    JsonRpcResponse::success(id, json!(result))
}

/// Handle tools/call request
async fn handle_tools_call(
    state: Arc<McpServerState>,
    id: RequestId,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
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
        "list_devices" => Box::new(crate::tools::ListDevicesTool::new(state.db_pool.clone())),
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
async fn handle_resources_list(id: RequestId) -> JsonRpcResponse {
    debug!("Listing resources");

    let resources = ResourceManager::list_resources();
    let result = ResourceListResult { resources };

    JsonRpcResponse::success(id, json!(result))
}

/// Handle resources/read request
async fn handle_resources_read(
    state: Arc<McpServerState>,
    id: RequestId,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
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
async fn handle_prompts_list(id: RequestId) -> JsonRpcResponse {
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
async fn handle_prompts_get(id: RequestId, params: Option<serde_json::Value>) -> JsonRpcResponse {
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
            format!("Generate personalized watchlist suggestions based on user viewing history and preferences.")
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

/// Health check handler
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "healthy" })))
}
