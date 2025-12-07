use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::search::{HybridSearchService, SearchFilters, SearchRequest};

/// Search request body for POST /api/v1/search
#[derive(Debug, Deserialize)]
pub struct SearchRequestBody {
    pub query: String,
    #[serde(default)]
    pub filters: Option<SearchFilters>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub user_id: Option<Uuid>,
    pub experiment_variant: Option<String>,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// POST /api/v1/search - Execute hybrid search
///
/// Executes a hybrid search combining vector and keyword search strategies.
/// Results are merged using Reciprocal Rank Fusion and cached for performance.
///
/// Request body:
/// - query: Search query string (required)
/// - filters: Optional filters (genres, platforms, year_range, rating_range)
/// - page: Page number (default: 1)
/// - page_size: Results per page (default: 20)
/// - user_id: Optional user ID for personalized results
/// - experiment_variant: Optional A/B test variant name
pub async fn execute_search(
    search_service: web::Data<Arc<HybridSearchService>>,
    body: web::Json<SearchRequestBody>,
) -> impl Responder {
    info!(
        query = %body.query,
        page = %body.page,
        page_size = %body.page_size,
        "Executing search request"
    );

    let request = SearchRequest {
        query: body.query.clone(),
        filters: body.filters.clone(),
        page: body.page,
        page_size: body.page_size,
        user_id: body.user_id,
        experiment_variant: body.experiment_variant.clone(),
    };

    match search_service.search(request).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            error!(error = %e, "Search request failed");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Search failed: {}", e),
            })
        }
    }
}

/// Autocomplete query parameters
#[derive(Debug, Deserialize)]
pub struct AutocompleteQuery {
    /// Prefix to autocomplete
    pub q: String,
    /// Maximum number of suggestions (default: 10)
    #[serde(default = "default_autocomplete_limit")]
    pub limit: usize,
}

fn default_autocomplete_limit() -> usize {
    10
}

/// GET /api/v1/search/autocomplete - Get autocomplete suggestions
///
/// Returns autocomplete suggestions based on the provided query prefix.
/// Uses a Trie-based data structure for fast prefix matching.
///
/// Query parameters:
/// - q: Query prefix (required)
/// - limit: Maximum number of suggestions (default: 10)
pub async fn autocomplete(
    search_service: web::Data<Arc<HybridSearchService>>,
    query: web::Query<AutocompleteQuery>,
) -> impl Responder {
    info!(prefix = %query.q, limit = %query.limit, "Autocomplete request");

    // Note: Autocomplete functionality needs to be exposed through HybridSearchService
    // For now, return a placeholder response
    HttpResponse::Ok().json(serde_json::json!({
        "query": query.q,
        "suggestions": [],
        "message": "Autocomplete service integration pending"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_page(), 1);
        assert_eq!(default_page_size(), 20);
        assert_eq!(default_autocomplete_limit(), 10);
    }

    #[test]
    fn test_search_request_deserialization() {
        let json = r#"{
            "query": "action movies",
            "page": 1,
            "page_size": 20
        }"#;

        let request: SearchRequestBody = serde_json::from_str(json).unwrap();
        assert_eq!(request.query, "action movies");
        assert_eq!(request.page, 1);
        assert_eq!(request.page_size, 20);
        assert!(request.filters.is_none());
        assert!(request.user_id.is_none());
    }

    #[test]
    fn test_search_request_with_filters() {
        let json = r#"{
            "query": "action movies",
            "filters": {
                "genres": ["action", "thriller"],
                "platforms": ["netflix"],
                "year_range": [2020, 2024],
                "rating_range": null
            }
        }"#;

        let request: SearchRequestBody = serde_json::from_str(json).unwrap();
        assert!(request.filters.is_some());

        let filters = request.filters.unwrap();
        assert_eq!(filters.genres, vec!["action", "thriller"]);
        assert_eq!(filters.platforms, vec!["netflix"]);
        assert_eq!(filters.year_range, Some((2020, 2024)));
        assert_eq!(filters.rating_range, None);
    }
}
