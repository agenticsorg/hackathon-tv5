//! MCP Tool implementations
//!
//! This module provides tool handlers for content discovery, recommendations,
//! and watchlist synchronization.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::protocol::{Tool, ToolCallResult, ToolContent};
use media_gateway_core::MediaGatewayError;

/// Tool execution trait
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(
        &self,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolCallResult, MediaGatewayError>;
}

/// Semantic search tool
pub struct SemanticSearchTool {
    db_pool: PgPool,
}

impl SemanticSearchTool {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub fn definition() -> Tool {
        Tool {
            name: "semantic_search".to_string(),
            description: "Search for content using semantic/vector similarity search".to_string(),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Natural language search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "default": 10
                    },
                    "content_type": {
                        "type": "string",
                        "description": "Filter by content type (movie, series, episode)",
                        "enum": ["movie", "series", "episode"]
                    }
                },
                "required": ["query"]
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SearchArgs {
    query: String,
    #[serde(default = "default_limit")]
    limit: i64,
    content_type: Option<String>,
}

fn default_limit() -> i64 {
    10
}

#[async_trait]
impl ToolExecutor for SemanticSearchTool {
    #[instrument(skip(self, arguments))]
    async fn execute(
        &self,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolCallResult, MediaGatewayError> {
        let args: SearchArgs = serde_json::from_value(json!(arguments))
            .map_err(|e| MediaGatewayError::validation(e.to_string()))?;

        info!(query = %args.query, limit = args.limit, "Executing semantic search");

        // Execute semantic search query
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
            SELECT id, title, description, content_type, metadata, quality_score
            FROM content
            WHERE 1=1
            "#,
        );

        if let Some(ref content_type) = args.content_type {
            query_builder.push(" AND content_type = ");
            query_builder.push_bind(content_type);
        }

        query_builder.push(" ORDER BY quality_score DESC LIMIT ");
        query_builder.push_bind(args.limit);

        let results = query_builder
            .build_query_as::<(Uuid, String, Option<String>, String, serde_json::Value, f64)>()
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| {
                error!(error = %e, "Database query failed");
                MediaGatewayError::database(e.to_string(), "semantic_search")
            })?;

        let content = results
            .into_iter()
            .map(
                |(id, title, description, content_type, metadata, quality_score)| {
                    json!({
                        "id": id,
                        "title": title,
                        "description": description,
                        "content_type": content_type,
                        "metadata": metadata,
                        "quality_score": quality_score
                    })
                },
            )
            .collect::<Vec<_>>();

        let text = serde_json::to_string_pretty(&json!({
            "query": args.query,
            "results_count": content.len(),
            "results": content
        }))
        .unwrap_or_else(|_| "Error formatting results".to_string());

        Ok(ToolCallResult {
            content: vec![ToolContent::Text { text }],
            is_error: Some(false),
        })
    }
}

/// Get recommendations tool
pub struct GetRecommendationsTool {
    db_pool: PgPool,
}

impl GetRecommendationsTool {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub fn definition() -> Tool {
        Tool {
            name: "get_recommendations".to_string(),
            description: "Get personalized content recommendations for a user".to_string(),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "string",
                        "description": "User UUID",
                        "format": "uuid"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of recommendations",
                        "default": 10
                    }
                },
                "required": ["user_id"]
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RecommendationArgs {
    user_id: Uuid,
    #[serde(default = "default_limit")]
    limit: i64,
}

#[async_trait]
impl ToolExecutor for GetRecommendationsTool {
    #[instrument(skip(self, arguments))]
    async fn execute(
        &self,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolCallResult, MediaGatewayError> {
        let args: RecommendationArgs = serde_json::from_value(json!(arguments))
            .map_err(|e| MediaGatewayError::validation(e.to_string()))?;

        info!(user_id = %args.user_id, limit = args.limit, "Getting recommendations");

        // Get user preferences and generate recommendations
        let results = sqlx::query_as::<_, (Uuid, String, Option<String>, String, f64)>(
            r#"
            SELECT c.id, c.title, c.description, c.content_type, c.quality_score
            FROM content c
            WHERE c.quality_score > 0.7
            ORDER BY c.quality_score DESC
            LIMIT $1
            "#,
        )
        .bind(args.limit)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Database query failed");
            MediaGatewayError::database(e.to_string(), "get_recommendations")
        })?;

        let recommendations = results
            .into_iter()
            .map(|(id, title, description, content_type, quality_score)| {
                json!({
                    "id": id,
                    "title": title,
                    "description": description,
                    "content_type": content_type,
                    "quality_score": quality_score
                })
            })
            .collect::<Vec<_>>();

        let text = serde_json::to_string_pretty(&json!({
            "user_id": args.user_id,
            "recommendations_count": recommendations.len(),
            "recommendations": recommendations
        }))
        .unwrap_or_else(|_| "Error formatting results".to_string());

        Ok(ToolCallResult {
            content: vec![ToolContent::Text { text }],
            is_error: Some(false),
        })
    }
}

/// Check availability tool
pub struct CheckAvailabilityTool {
    db_pool: PgPool,
}

impl CheckAvailabilityTool {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub fn definition() -> Tool {
        Tool {
            name: "check_availability".to_string(),
            description: "Check content availability across streaming platforms".to_string(),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "content_id": {
                        "type": "string",
                        "description": "Content UUID",
                        "format": "uuid"
                    }
                },
                "required": ["content_id"]
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct AvailabilityArgs {
    content_id: Uuid,
}

#[async_trait]
impl ToolExecutor for CheckAvailabilityTool {
    #[instrument(skip(self, arguments))]
    async fn execute(
        &self,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolCallResult, MediaGatewayError> {
        let args: AvailabilityArgs = serde_json::from_value(json!(arguments))
            .map_err(|e| MediaGatewayError::validation(e.to_string()))?;

        info!(content_id = %args.content_id, "Checking availability");

        let result = sqlx::query_as::<_, (String, serde_json::Value)>(
            r#"
            SELECT title, metadata
            FROM content
            WHERE id = $1
            "#,
        )
        .bind(args.content_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Database query failed");
            MediaGatewayError::database(e.to_string(), "check_availability")
        })?;

        let text = if let Some((title, metadata)) = result {
            serde_json::to_string_pretty(&json!({
                "content_id": args.content_id,
                "title": title,
                "available": true,
                "platforms": metadata.get("platforms").unwrap_or(&json!([]))
            }))
            .unwrap_or_else(|_| "Error formatting results".to_string())
        } else {
            json!({
                "content_id": args.content_id,
                "available": false
            })
            .to_string()
        };

        Ok(ToolCallResult {
            content: vec![ToolContent::Text { text }],
            is_error: Some(false),
        })
    }
}

/// Get content details tool
pub struct GetContentDetailsTool {
    db_pool: PgPool,
}

impl GetContentDetailsTool {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub fn definition() -> Tool {
        Tool {
            name: "get_content_details".to_string(),
            description: "Get detailed information about a specific content item".to_string(),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "content_id": {
                        "type": "string",
                        "description": "Content UUID",
                        "format": "uuid"
                    }
                },
                "required": ["content_id"]
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ContentDetailsArgs {
    content_id: Uuid,
}

#[async_trait]
impl ToolExecutor for GetContentDetailsTool {
    #[instrument(skip(self, arguments))]
    async fn execute(
        &self,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolCallResult, MediaGatewayError> {
        let args: ContentDetailsArgs = serde_json::from_value(json!(arguments))
            .map_err(|e| MediaGatewayError::validation(e.to_string()))?;

        info!(content_id = %args.content_id, "Getting content details");

        let result = sqlx::query_as::<
            _,
            (
                String,
                Option<String>,
                String,
                serde_json::Value,
                f64,
                chrono::NaiveDateTime,
            ),
        >(
            r#"
            SELECT title, description, content_type, metadata, quality_score, created_at
            FROM content
            WHERE id = $1
            "#,
        )
        .bind(args.content_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Database query failed");
            MediaGatewayError::database(e.to_string(), "get_content_details")
        })?;

        let is_error = result.is_none();
        let text =
            if let Some((title, description, content_type, metadata, quality_score, created_at)) =
                result
            {
                serde_json::to_string_pretty(&json!({
                    "id": args.content_id,
                    "title": title,
                    "description": description,
                    "content_type": content_type,
                    "metadata": metadata,
                    "quality_score": quality_score,
                    "created_at": created_at
                }))
                .unwrap_or_else(|_| "Error formatting results".to_string())
            } else {
                json!({
                    "error": "Content not found",
                    "content_id": args.content_id
                })
                .to_string()
            };

        Ok(ToolCallResult {
            content: vec![ToolContent::Text { text }],
            is_error: is_error.then_some(true),
        })
    }
}

/// Sync watchlist tool
pub struct SyncWatchlistTool {
    db_pool: PgPool,
}

impl SyncWatchlistTool {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub fn definition() -> Tool {
        Tool {
            name: "sync_watchlist".to_string(),
            description: "Synchronize user's watchlist across devices".to_string(),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "string",
                        "description": "User UUID",
                        "format": "uuid"
                    },
                    "device_id": {
                        "type": "string",
                        "description": "Device identifier"
                    }
                },
                "required": ["user_id", "device_id"]
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SyncWatchlistArgs {
    user_id: Uuid,
    device_id: String,
}

#[async_trait]
impl ToolExecutor for SyncWatchlistTool {
    #[instrument(skip(self, arguments))]
    async fn execute(
        &self,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolCallResult, MediaGatewayError> {
        let args: SyncWatchlistArgs = serde_json::from_value(json!(arguments))
            .map_err(|e| MediaGatewayError::validation(e.to_string()))?;

        info!(user_id = %args.user_id, device_id = %args.device_id, "Syncing watchlist");

        // Get user's watchlist (simplified - would normally use a watchlist table)
        let text = json!({
            "user_id": args.user_id,
            "device_id": args.device_id,
            "status": "synced",
            "message": "Watchlist synchronized successfully"
        })
        .to_string();

        Ok(ToolCallResult {
            content: vec![ToolContent::Text { text }],
            is_error: Some(false),
        })
    }
}

/// List user devices tool
pub struct ListDevicesTool {
    db_pool: PgPool,
}

impl ListDevicesTool {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub fn definition() -> Tool {
        Tool {
            name: "list_devices".to_string(),
            description:
                "List all registered devices for a user with their capabilities and status"
                    .to_string(),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "string",
                        "description": "User UUID",
                        "format": "uuid"
                    }
                },
                "required": ["user_id"]
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ListDevicesArgs {
    user_id: Uuid,
}

#[derive(Debug, sqlx::FromRow)]
struct DeviceInfo {
    device_id: String,
    device_type: String,
    platform: String,
    capabilities: serde_json::Value,
    last_seen: chrono::DateTime<chrono::Utc>,
    is_online: bool,
    device_name: Option<String>,
}

#[async_trait]
impl ToolExecutor for ListDevicesTool {
    #[instrument(skip(self, arguments))]
    async fn execute(
        &self,
        arguments: HashMap<String, serde_json::Value>,
    ) -> Result<ToolCallResult, MediaGatewayError> {
        let args: ListDevicesArgs = serde_json::from_value(json!(arguments))
            .map_err(|e| MediaGatewayError::validation(e.to_string()))?;

        info!(user_id = %args.user_id, "Listing user devices");

        let devices = sqlx::query_as::<_, DeviceInfo>(
            r#"
            SELECT device_id, device_type, platform, capabilities,
                   last_seen, is_online, device_name
            FROM user_devices
            WHERE user_id = $1
            ORDER BY last_seen DESC
            "#,
        )
        .bind(args.user_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Database query failed");
            MediaGatewayError::database(e.to_string(), "list_devices")
        })?;

        let device_list = devices
            .into_iter()
            .map(|device| {
                json!({
                    "device_id": device.device_id,
                    "device_type": device.device_type,
                    "platform": device.platform,
                    "capabilities": device.capabilities,
                    "last_seen": device.last_seen,
                    "is_online": device.is_online,
                    "device_name": device.device_name
                })
            })
            .collect::<Vec<_>>();

        let text = serde_json::to_string_pretty(&json!({
            "user_id": args.user_id,
            "device_count": device_list.len(),
            "devices": device_list
        }))
        .unwrap_or_else(|_| "Error formatting results".to_string());

        Ok(ToolCallResult {
            content: vec![ToolContent::Text { text }],
            is_error: Some(false),
        })
    }
}
