//! HTTP handlers for ingestion API endpoints

use actix_web::{web, HttpResponse, Result};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::repository::{ContentRepository, PostgresContentRepository};

/// Query parameters for expiring content endpoint
#[derive(Debug, Deserialize)]
pub struct ExpiringContentQuery {
    /// Number of days to look ahead (default: 7)
    pub days: Option<i64>,
    /// Platform filter (optional)
    pub platform: Option<String>,
    /// Region filter (optional)
    pub region: Option<String>,
    /// Limit number of results (default: 100, max: 1000)
    pub limit: Option<i64>,
}

/// Response item for expiring content
#[derive(Debug, Serialize)]
pub struct ExpiringContentItem {
    pub content_id: Uuid,
    pub title: String,
    pub platform: String,
    pub region: String,
    pub expires_at: String,
    pub days_until_expiration: i64,
}

/// Response for expiring content endpoint
#[derive(Debug, Serialize)]
pub struct ExpiringContentResponse {
    pub total: usize,
    pub window_days: i64,
    pub items: Vec<ExpiringContentItem>,
}

/// GET /api/v1/content/expiring
///
/// Get list of content expiring within the specified window
pub async fn get_expiring_content(
    pool: web::Data<PgPool>,
    query: web::Query<ExpiringContentQuery>,
) -> Result<HttpResponse> {
    let days = query.days.unwrap_or(7).max(1).min(90); // Clamp between 1 and 90 days
    let limit = query.limit.unwrap_or(100).max(1).min(1000); // Clamp between 1 and 1000

    let repository = PostgresContentRepository::new(pool.get_ref().clone());
    let duration = Duration::days(days);

    // Find expiring content
    let expiring = repository
        .find_expiring_within(duration)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))?;

    // Apply filters
    let mut filtered = expiring;

    if let Some(platform) = &query.platform {
        filtered.retain(|c| c.platform.eq_ignore_ascii_case(platform));
    }

    if let Some(region) = &query.region {
        filtered.retain(|c| c.region.eq_ignore_ascii_case(region));
    }

    // Limit results
    filtered.truncate(limit as usize);

    // Convert to response format
    let now = Utc::now();
    let items: Vec<ExpiringContentItem> = filtered
        .iter()
        .map(|c| ExpiringContentItem {
            content_id: c.content_id,
            title: c.title.clone(),
            platform: c.platform.clone(),
            region: c.region.clone(),
            expires_at: c.expires_at.to_rfc3339(),
            days_until_expiration: (c.expires_at - now).num_days(),
        })
        .collect();

    let response = ExpiringContentResponse {
        total: items.len(),
        window_days: days,
        items,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expiring_content_query_defaults() {
        let query = ExpiringContentQuery {
            days: None,
            platform: None,
            region: None,
            limit: None,
        };

        assert!(query.days.is_none());
        assert!(query.platform.is_none());
    }

    #[test]
    fn test_expiring_content_item_serialization() {
        let item = ExpiringContentItem {
            content_id: Uuid::new_v4(),
            title: "Test Movie".to_string(),
            platform: "netflix".to_string(),
            region: "US".to_string(),
            expires_at: Utc::now().to_rfc3339(),
            days_until_expiration: 7,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("Test Movie"));
        assert!(json.contains("netflix"));
    }
}
