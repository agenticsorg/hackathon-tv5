use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a search event with anonymized user context
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchEvent {
    pub id: Uuid,
    pub query_hash: String,
    pub query_text: String,
    pub user_id_hash: Option<String>,
    pub result_count: i32,
    pub latency_ms: i32,
    #[sqlx(json)]
    pub filters_applied: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Represents a click on a search result
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchClick {
    pub id: Uuid,
    pub search_event_id: Uuid,
    pub content_id: Uuid,
    pub position: i32,
    pub clicked_at: DateTime<Utc>,
}

/// Query logger with privacy-preserving anonymization
#[derive(Clone)]
pub struct QueryLog {
    pool: PgPool,
}

impl QueryLog {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Hash a query string for deduplication while preserving privacy
    pub fn hash_query(query: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Anonymize user ID using SHA-256
    pub fn anonymize_user_id(user_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Log a search event
    pub async fn log_search(
        &self,
        query: &str,
        user_id: Option<&str>,
        result_count: i32,
        latency_ms: i32,
        filters: HashMap<String, serde_json::Value>,
    ) -> Result<Uuid, sqlx::Error> {
        let query_hash = Self::hash_query(query);
        let user_id_hash = user_id.map(Self::anonymize_user_id);
        let filters_json = serde_json::to_value(filters).unwrap_or(serde_json::json!({}));

        let result: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO search_events (query_hash, query_text, user_id_hash, result_count, latency_ms, filters_applied)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#
        )
        .bind(query_hash)
        .bind(query)
        .bind(user_id_hash)
        .bind(result_count)
        .bind(latency_ms)
        .bind(filters_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Log a click on a search result
    pub async fn log_click(
        &self,
        search_event_id: Uuid,
        content_id: Uuid,
        position: i32,
    ) -> Result<Uuid, sqlx::Error> {
        let result: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO search_clicks (search_event_id, content_id, position)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(search_event_id)
        .bind(content_id)
        .bind(position)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.0)
    }

    /// Get recent search events
    pub async fn get_recent_events(&self, limit: i64) -> Result<Vec<SearchEvent>, sqlx::Error> {
        sqlx::query_as::<_, SearchEvent>(
            r#"
            SELECT id, query_hash, query_text, user_id_hash, result_count, latency_ms,
                   filters_applied, created_at
            FROM search_events
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    /// Get clicks for a search event
    pub async fn get_event_clicks(
        &self,
        search_event_id: Uuid,
    ) -> Result<Vec<SearchClick>, sqlx::Error> {
        sqlx::query_as::<_, SearchClick>(
            r#"
            SELECT id, search_event_id, content_id, position, clicked_at
            FROM search_clicks
            WHERE search_event_id = $1
            ORDER BY clicked_at ASC
            "#,
        )
        .bind(search_event_id)
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_query() {
        let query = "action movies";
        let hash1 = QueryLog::hash_query(query);
        let hash2 = QueryLog::hash_query(query);

        assert_eq!(hash1, hash2, "Same query should produce same hash");
        assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 characters");

        let different_query = "comedy movies";
        let hash3 = QueryLog::hash_query(different_query);
        assert_ne!(
            hash1, hash3,
            "Different queries should produce different hashes"
        );
    }

    #[test]
    fn test_anonymize_user_id() {
        let user_id = "user123";
        let hash1 = QueryLog::anonymize_user_id(user_id);
        let hash2 = QueryLog::anonymize_user_id(user_id);

        assert_eq!(hash1, hash2, "Same user ID should produce same hash");
        assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 characters");
        assert_ne!(hash1, user_id, "Hash should not match original user ID");

        let different_user = "user456";
        let hash3 = QueryLog::anonymize_user_id(different_user);
        assert_ne!(
            hash1, hash3,
            "Different user IDs should produce different hashes"
        );
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_log_search() {
        let pool = setup_test_db().await;
        let query_log = QueryLog::new(pool.clone());

        let mut filters = HashMap::new();
        filters.insert("genre".to_string(), serde_json::json!("action"));

        let event_id = query_log
            .log_search("action movies", Some("user123"), 42, 150, filters)
            .await
            .expect("Failed to log search");

        assert!(!event_id.is_nil(), "Event ID should not be nil");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_log_click() {
        let pool = setup_test_db().await;
        let query_log = QueryLog::new(pool.clone());

        let event_id = query_log
            .log_search("action movies", Some("user123"), 42, 150, HashMap::new())
            .await
            .expect("Failed to log search");

        let content_id = Uuid::new_v4();
        let click_id = query_log
            .log_click(event_id, content_id, 1)
            .await
            .expect("Failed to log click");

        assert!(!click_id.is_nil(), "Click ID should not be nil");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_get_recent_events() {
        let pool = setup_test_db().await;
        let query_log = QueryLog::new(pool.clone());

        // Log multiple events
        for i in 0..5 {
            query_log
                .log_search(
                    &format!("query {}", i),
                    Some("user123"),
                    10 + i,
                    100 + i * 10,
                    HashMap::new(),
                )
                .await
                .expect("Failed to log search");
        }

        let events = query_log
            .get_recent_events(3)
            .await
            .expect("Failed to get recent events");

        assert_eq!(events.len(), 3, "Should return 3 most recent events");
        assert!(
            events[0].created_at >= events[1].created_at,
            "Events should be ordered by time descending"
        );

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_get_event_clicks() {
        let pool = setup_test_db().await;
        let query_log = QueryLog::new(pool.clone());

        let event_id = query_log
            .log_search("action movies", Some("user123"), 42, 150, HashMap::new())
            .await
            .expect("Failed to log search");

        // Log multiple clicks
        for i in 0..3 {
            let content_id = Uuid::new_v4();
            query_log
                .log_click(event_id, content_id, i)
                .await
                .expect("Failed to log click");
        }

        let clicks = query_log
            .get_event_clicks(event_id)
            .await
            .expect("Failed to get clicks");

        assert_eq!(clicks.len(), 3, "Should return all clicks for the event");
        assert_eq!(clicks[0].position, 0, "First click should be at position 0");

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

        // Run migrations
        sqlx::query(include_str!(
            "../../migrations/20251206_search_analytics.sql"
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
