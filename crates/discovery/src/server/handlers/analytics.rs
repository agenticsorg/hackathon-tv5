use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

use crate::analytics::SearchAnalytics;

/// Query parameters for analytics endpoint
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    /// Time period: "1h", "24h", "7d", "30d"
    #[serde(default = "default_period")]
    pub period: String,

    /// Limit for top queries
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_period() -> String {
    "24h".to_string()
}

fn default_limit() -> i64 {
    10
}

/// Error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// GET /api/v1/analytics - Get search analytics dashboard data
///
/// Returns comprehensive analytics including:
/// - Total searches and unique queries
/// - Latency statistics (avg, p95)
/// - Zero-result rate
/// - Click-through rate
/// - Top performing queries
/// - Zero-result queries for content gap analysis
///
/// Query parameters:
/// - period: Time period ("1h", "24h", "7d", "30d", default: "24h")
/// - limit: Limit for top queries (default: 10)
pub async fn get_analytics(
    analytics: web::Data<Arc<SearchAnalytics>>,
    params: web::Query<AnalyticsQuery>,
) -> impl Responder {
    info!(
        period = %params.period,
        limit = %params.limit,
        "Fetching analytics dashboard"
    );

    match analytics.get_dashboard(&params.period, params.limit).await {
        Ok(dashboard) => HttpResponse::Ok().json(dashboard),
        Err(e) => {
            error!(error = %e, "Failed to get analytics");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get analytics: {}", e),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_period() {
        assert_eq!(default_period(), "24h");
    }

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 10);
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_get_analytics() {
        use sqlx::PgPool;
        use std::collections::HashMap;

        let pool = setup_test_db().await;
        let analytics = Arc::new(SearchAnalytics::new(pool.clone()));

        // Create test data
        for i in 0..20 {
            let event_id = analytics
                .query_log()
                .log_search(
                    &format!("query {}", i % 5),
                    Some("user123"),
                    if i % 7 == 0 { 0 } else { 10 + i },
                    100 + i * 5,
                    HashMap::new(),
                )
                .await
                .expect("Failed to log search");

            if i % 3 == 0 {
                let content_id = uuid::Uuid::new_v4();
                analytics
                    .query_log()
                    .log_click(event_id, content_id, 0)
                    .await
                    .expect("Failed to log click");
            }
        }

        let params = AnalyticsQuery {
            period: "24h".to_string(),
            limit: 10,
        };

        let result = get_analytics(State(analytics.clone()), Query(params)).await;

        assert!(result.is_ok(), "Should successfully get analytics");
        let Json(dashboard) = result.unwrap();

        assert_eq!(dashboard.period, "24h");
        assert_eq!(dashboard.total_searches, 20);
        assert!(dashboard.unique_queries > 0);
        assert!(dashboard.avg_latency_ms > 0.0);
        assert!(dashboard.p95_latency_ms > 0);
        assert!(!dashboard.top_queries.is_empty());

        cleanup_test_db(&pool).await;
    }

    // Test helpers
    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
        });

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        sqlx::query(include_str!(
            "../../../migrations/20251206_search_analytics.sql"
        ))
        .execute(&pool)
        .await
        .expect("Failed to run migrations");

        pool
    }

    async fn cleanup_test_db(pool: &PgPool) {
        sqlx::query("TRUNCATE search_events, search_clicks, popular_searches CASCADE")
            .execute(pool)
            .await
            .expect("Failed to cleanup test database");
    }
}
