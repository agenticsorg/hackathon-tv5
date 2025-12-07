//! MCP Resource implementations
//!
//! This module provides resource handlers for accessing content and user preferences.

use serde_json::json;
use sqlx::PgPool;
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::protocol::{Resource, ResourceContent};
use media_gateway_core::MediaGatewayError;

/// Resource manager
pub struct ResourceManager {
    db_pool: PgPool,
}

impl ResourceManager {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    /// List available resources
    pub fn list_resources() -> Vec<Resource> {
        vec![
            Resource {
                uri: "content://catalog".to_string(),
                name: "Content Catalog".to_string(),
                description: "Complete content catalog with metadata".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "user://preferences/{user_id}".to_string(),
                name: "User Preferences".to_string(),
                description: "User preferences and settings".to_string(),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "content://item/{content_id}".to_string(),
                name: "Content Item".to_string(),
                description: "Detailed content item information".to_string(),
                mime_type: Some("application/json".to_string()),
            },
        ]
    }

    /// Read a resource by URI
    #[instrument(skip(self))]
    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContent, MediaGatewayError> {
        info!(uri = %uri, "Reading resource");

        if uri == "content://catalog" {
            self.read_content_catalog().await
        } else if uri.starts_with("user://preferences/") {
            let user_id = uri
                .trim_start_matches("user://preferences/")
                .parse::<Uuid>()
                .map_err(|e| MediaGatewayError::validation(format!("Invalid user_id: {}", e)))?;
            self.read_user_preferences(user_id).await
        } else if uri.starts_with("content://item/") {
            let content_id = uri
                .trim_start_matches("content://item/")
                .parse::<Uuid>()
                .map_err(|e| MediaGatewayError::validation(format!("Invalid content_id: {}", e)))?;
            self.read_content_item(content_id).await
        } else {
            Err(MediaGatewayError::not_found(format!(
                "Resource not found: {}",
                uri
            )))
        }
    }

    /// Read content catalog
    async fn read_content_catalog(&self) -> Result<ResourceContent, MediaGatewayError> {
        let results = sqlx::query_as::<_, (Uuid, String, Option<String>, String)>(
            r#"
            SELECT id, title, description, content_type
            FROM content
            ORDER BY quality_score DESC
            LIMIT 100
            "#,
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to fetch content catalog");
            MediaGatewayError::database(e.to_string(), "read_content_catalog")
        })?;

        let catalog = results
            .into_iter()
            .map(|(id, title, description, content_type)| {
                json!({
                    "id": id,
                    "title": title,
                    "description": description,
                    "content_type": content_type
                })
            })
            .collect::<Vec<_>>();

        let text = serde_json::to_string_pretty(&json!({
            "total": catalog.len(),
            "items": catalog
        }))?;

        Ok(ResourceContent {
            uri: "content://catalog".to_string(),
            mime_type: Some("application/json".to_string()),
            text: Some(text),
            blob: None,
        })
    }

    /// Read user preferences
    async fn read_user_preferences(
        &self,
        user_id: Uuid,
    ) -> Result<ResourceContent, MediaGatewayError> {
        let result = sqlx::query_as::<_, (serde_json::Value,)>(
            r#"
            SELECT preferences
            FROM user_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, user_id = %user_id, "Failed to fetch user preferences");
            MediaGatewayError::database(e.to_string(), "read_user_preferences")
        })?;

        let preferences = if let Some((prefs,)) = result {
            prefs
        } else {
            json!({
                "user_id": user_id,
                "preferences": {}
            })
        };

        let text = serde_json::to_string_pretty(&preferences)?;

        Ok(ResourceContent {
            uri: format!("user://preferences/{}", user_id),
            mime_type: Some("application/json".to_string()),
            text: Some(text),
            blob: None,
        })
    }

    /// Read content item
    async fn read_content_item(
        &self,
        content_id: Uuid,
    ) -> Result<ResourceContent, MediaGatewayError> {
        let result = sqlx::query_as::<_, (String, Option<String>, String, serde_json::Value, f64)>(
            r#"
            SELECT title, description, content_type, metadata, quality_score
            FROM content
            WHERE id = $1
            "#,
        )
        .bind(content_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            error!(error = %e, content_id = %content_id, "Failed to fetch content item");
            MediaGatewayError::database(e.to_string(), "read_content_item")
        })?;

        if let Some((title, description, content_type, metadata, quality_score)) = result {
            let item = json!({
                "id": content_id,
                "title": title,
                "description": description,
                "content_type": content_type,
                "metadata": metadata,
                "quality_score": quality_score
            });

            let text = serde_json::to_string_pretty(&item)?;

            Ok(ResourceContent {
                uri: format!("content://item/{}", content_id),
                mime_type: Some("application/json".to_string()),
                text: Some(text),
                blob: None,
            })
        } else {
            Err(MediaGatewayError::not_found(format!(
                "Content not found: {}",
                content_id
            )))
        }
    }
}
