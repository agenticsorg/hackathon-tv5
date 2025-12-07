use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use super::query_log::QueryLog;

/// Period type for aggregation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PeriodType {
    Hourly,
    Daily,
    Weekly,
}

impl PeriodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeriodType::Hourly => "hourly",
            PeriodType::Daily => "daily",
            PeriodType::Weekly => "weekly",
        }
    }

    pub fn duration(&self) -> Duration {
        match self {
            PeriodType::Hourly => Duration::hours(1),
            PeriodType::Daily => Duration::days(1),
            PeriodType::Weekly => Duration::weeks(1),
        }
    }
}

/// Popular search query with statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopularQuery {
    pub query: String,
    pub count: i64,
    pub ctr: f64,
    pub avg_results: f64,
    pub avg_latency_ms: f64,
}

/// Zero-result query for content gap analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroResultQuery {
    pub query: String,
    pub count: i64,
}

/// Latency statistics with percentiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub p50: i32,
    pub p95: i32,
    pub p99: i32,
    pub avg: f64,
}

/// Analytics dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsDashboard {
    pub period: String,
    pub total_searches: i64,
    pub unique_queries: i64,
    pub avg_latency_ms: f64,
    pub p95_latency_ms: i32,
    pub zero_result_rate: f64,
    pub avg_ctr: f64,
    pub top_queries: Vec<PopularQuery>,
    pub zero_result_queries: Vec<ZeroResultQuery>,
}

/// Popular search record in database
#[derive(Debug, Clone, FromRow)]
struct PopularSearch {
    id: Uuid,
    query_text: String,
    period_type: String,
    period_start: DateTime<Utc>,
    search_count: i32,
    avg_results: Option<f64>,
    avg_latency_ms: Option<f64>,
    ctr: Option<f64>,
}

/// Search analytics service
#[derive(Clone)]
pub struct SearchAnalytics {
    pool: PgPool,
    query_log: QueryLog,
}

impl SearchAnalytics {
    pub fn new(pool: PgPool) -> Self {
        let query_log = QueryLog::new(pool.clone());
        Self { pool, query_log }
    }

    /// Get query log instance
    pub fn query_log(&self) -> &QueryLog {
        &self.query_log
    }

    /// Calculate latency percentiles for a time period
    pub async fn calculate_latency_stats(
        &self,
        since: DateTime<Utc>,
    ) -> Result<LatencyStats, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT latency_ms
            FROM search_events
            WHERE created_at >= $1
            ORDER BY latency_ms ASC
            "#,
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        let latencies: Vec<i32> = rows.iter().map(|row| row.get("latency_ms")).collect();

        if latencies.is_empty() {
            return Ok(LatencyStats {
                p50: 0,
                p95: 0,
                p99: 0,
                avg: 0.0,
            });
        }

        let len = latencies.len();
        let p50_idx = (len as f64 * 0.50) as usize;
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;

        let avg: f64 = latencies.iter().map(|&x| x as f64).sum::<f64>() / len as f64;

        Ok(LatencyStats {
            p50: latencies[p50_idx],
            p95: latencies[p95_idx.min(len - 1)],
            p99: latencies[p99_idx.min(len - 1)],
            avg,
        })
    }

    /// Get top popular queries for a period
    pub async fn get_top_queries(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<PopularQuery>, sqlx::Error> {
        let results = sqlx::query(
            r#"
            SELECT
                query_text,
                COUNT(*) as search_count,
                AVG(result_count) as avg_results,
                AVG(latency_ms) as avg_latency_ms,
                COALESCE(
                    COUNT(DISTINCT sc.id)::FLOAT / NULLIF(COUNT(DISTINCT se.id), 0),
                    0
                ) as ctr
            FROM search_events se
            LEFT JOIN search_clicks sc ON se.id = sc.search_event_id
            WHERE se.created_at >= $1
            GROUP BY query_text
            ORDER BY search_count DESC
            LIMIT $2
            "#,
        )
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| PopularQuery {
                query: r.get("query_text"),
                count: r.get::<Option<i64>, _>("search_count").unwrap_or(0),
                ctr: r.get::<Option<f64>, _>("ctr").unwrap_or(0.0),
                avg_results: r.get::<Option<f64>, _>("avg_results").unwrap_or(0.0),
                avg_latency_ms: r.get::<Option<f64>, _>("avg_latency_ms").unwrap_or(0.0),
            })
            .collect())
    }

    /// Get zero-result queries for content gap analysis
    pub async fn get_zero_result_queries(
        &self,
        since: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<ZeroResultQuery>, sqlx::Error> {
        let results = sqlx::query(
            r#"
            SELECT query_text, COUNT(*) as search_count
            FROM search_events
            WHERE created_at >= $1 AND result_count = 0
            GROUP BY query_text
            ORDER BY search_count DESC
            LIMIT $2
            "#,
        )
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| ZeroResultQuery {
                query: r.get("query_text"),
                count: r.get::<Option<i64>, _>("search_count").unwrap_or(0),
            })
            .collect())
    }

    /// Calculate overall click-through rate
    pub async fn calculate_ctr(&self, since: DateTime<Utc>) -> Result<f64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT
                COUNT(DISTINCT se.id) as total_searches,
                COUNT(DISTINCT sc.id) as total_clicks
            FROM search_events se
            LEFT JOIN search_clicks sc ON se.id = sc.search_event_id
            WHERE se.created_at >= $1
            "#,
        )
        .bind(since)
        .fetch_one(&self.pool)
        .await?;

        let total_searches = result.get::<Option<i64>, _>("total_searches").unwrap_or(0) as f64;
        let total_clicks = result.get::<Option<i64>, _>("total_clicks").unwrap_or(0) as f64;

        if total_searches == 0.0 {
            Ok(0.0)
        } else {
            Ok(total_clicks / total_searches)
        }
    }

    /// Aggregate popular searches into summary table
    pub async fn aggregate_popular_searches(
        &self,
        period_type: PeriodType,
        period_start: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let period_end = period_start + period_type.duration();

        sqlx::query(
            r#"
            INSERT INTO popular_searches (query_text, period_type, period_start, search_count, avg_results, avg_latency_ms, ctr)
            SELECT
                query_text,
                $1 as period_type,
                $2 as period_start,
                COUNT(*) as search_count,
                AVG(result_count) as avg_results,
                AVG(latency_ms) as avg_latency_ms,
                COALESCE(
                    COUNT(DISTINCT sc.id)::FLOAT / NULLIF(COUNT(DISTINCT se.id), 0),
                    0
                ) as ctr
            FROM search_events se
            LEFT JOIN search_clicks sc ON se.id = sc.search_event_id
            WHERE se.created_at >= $2 AND se.created_at < $3
            GROUP BY query_text
            ON CONFLICT (query_text, period_type, period_start)
            DO UPDATE SET
                search_count = EXCLUDED.search_count,
                avg_results = EXCLUDED.avg_results,
                avg_latency_ms = EXCLUDED.avg_latency_ms,
                ctr = EXCLUDED.ctr
            "#
        )
        .bind(period_type.as_str())
        .bind(period_start)
        .bind(period_end)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get analytics dashboard for a time period
    pub async fn get_dashboard(
        &self,
        period: &str,
        top_limit: i64,
    ) -> Result<AnalyticsDashboard, sqlx::Error> {
        let since = match period {
            "1h" => Utc::now() - Duration::hours(1),
            "24h" => Utc::now() - Duration::hours(24),
            "7d" => Utc::now() - Duration::days(7),
            "30d" => Utc::now() - Duration::days(30),
            _ => Utc::now() - Duration::hours(24), // default to 24h
        };

        // Get total searches and unique queries
        let stats = sqlx::query(
            r#"
            SELECT
                COUNT(*) as total_searches,
                COUNT(DISTINCT query_hash) as unique_queries,
                COUNT(*) FILTER (WHERE result_count = 0) as zero_results
            FROM search_events
            WHERE created_at >= $1
            "#,
        )
        .bind(since)
        .fetch_one(&self.pool)
        .await?;

        let total_searches = stats.get::<Option<i64>, _>("total_searches").unwrap_or(0);
        let unique_queries = stats.get::<Option<i64>, _>("unique_queries").unwrap_or(0);
        let zero_results = stats.get::<Option<i64>, _>("zero_results").unwrap_or(0);

        let zero_result_rate = if total_searches > 0 {
            zero_results as f64 / total_searches as f64
        } else {
            0.0
        };

        // Calculate latency stats
        let latency_stats = self.calculate_latency_stats(since).await?;

        // Calculate CTR
        let avg_ctr = self.calculate_ctr(since).await?;

        // Get top queries
        let top_queries = self.get_top_queries(since, top_limit).await?;

        // Get zero-result queries
        let zero_result_queries = self.get_zero_result_queries(since, top_limit).await?;

        Ok(AnalyticsDashboard {
            period: period.to_string(),
            total_searches,
            unique_queries,
            avg_latency_ms: latency_stats.avg,
            p95_latency_ms: latency_stats.p95,
            zero_result_rate,
            avg_ctr,
            top_queries,
            zero_result_queries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_period_type_as_str() {
        assert_eq!(PeriodType::Hourly.as_str(), "hourly");
        assert_eq!(PeriodType::Daily.as_str(), "daily");
        assert_eq!(PeriodType::Weekly.as_str(), "weekly");
    }

    #[test]
    fn test_period_type_duration() {
        assert_eq!(PeriodType::Hourly.duration(), Duration::hours(1));
        assert_eq!(PeriodType::Daily.duration(), Duration::days(1));
        assert_eq!(PeriodType::Weekly.duration(), Duration::weeks(1));
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_calculate_latency_stats() {
        let pool = setup_test_db().await;
        let analytics = SearchAnalytics::new(pool.clone());

        // Log events with known latencies
        let latencies = vec![100, 150, 200, 250, 300, 350, 400, 450, 500];
        for latency in &latencies {
            analytics
                .query_log()
                .log_search("test query", Some("user123"), 10, *latency, HashMap::new())
                .await
                .expect("Failed to log search");
        }

        let stats = analytics
            .calculate_latency_stats(Utc::now() - Duration::hours(1))
            .await
            .expect("Failed to calculate latency stats");

        assert_eq!(stats.p50, 300, "P50 should be median");
        assert!(stats.p95 >= 450, "P95 should be near high end");
        assert!(
            stats.avg > 200.0 && stats.avg < 400.0,
            "Average should be in middle range"
        );

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_get_top_queries() {
        let pool = setup_test_db().await;
        let analytics = SearchAnalytics::new(pool.clone());

        // Log multiple searches with different queries
        for _ in 0..5 {
            analytics
                .query_log()
                .log_search("popular query", Some("user123"), 10, 100, HashMap::new())
                .await
                .expect("Failed to log search");
        }

        for _ in 0..3 {
            analytics
                .query_log()
                .log_search("less popular", Some("user456"), 8, 120, HashMap::new())
                .await
                .expect("Failed to log search");
        }

        let top_queries = analytics
            .get_top_queries(Utc::now() - Duration::hours(1), 10)
            .await
            .expect("Failed to get top queries");

        assert!(!top_queries.is_empty(), "Should have top queries");
        assert_eq!(
            top_queries[0].query, "popular query",
            "Most popular should be first"
        );
        assert_eq!(top_queries[0].count, 5, "Should have correct count");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_get_zero_result_queries() {
        let pool = setup_test_db().await;
        let analytics = SearchAnalytics::new(pool.clone());

        // Log searches with zero results
        for _ in 0..3 {
            analytics
                .query_log()
                .log_search(
                    "nonexistent content",
                    Some("user123"),
                    0,
                    100,
                    HashMap::new(),
                )
                .await
                .expect("Failed to log search");
        }

        // Log normal searches
        analytics
            .query_log()
            .log_search("normal query", Some("user456"), 10, 100, HashMap::new())
            .await
            .expect("Failed to log search");

        let zero_results = analytics
            .get_zero_result_queries(Utc::now() - Duration::hours(1), 10)
            .await
            .expect("Failed to get zero result queries");

        assert!(!zero_results.is_empty(), "Should have zero result queries");
        assert_eq!(zero_results[0].query, "nonexistent content");
        assert_eq!(zero_results[0].count, 3);

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_calculate_ctr() {
        let pool = setup_test_db().await;
        let analytics = SearchAnalytics::new(pool.clone());

        // Log searches and clicks
        for i in 0..5 {
            let event_id = analytics
                .query_log()
                .log_search("test query", Some("user123"), 10, 100, HashMap::new())
                .await
                .expect("Failed to log search");

            // Add clicks for first 3 searches
            if i < 3 {
                let content_id = Uuid::new_v4();
                analytics
                    .query_log()
                    .log_click(event_id, content_id, 0)
                    .await
                    .expect("Failed to log click");
            }
        }

        let ctr = analytics
            .calculate_ctr(Utc::now() - Duration::hours(1))
            .await
            .expect("Failed to calculate CTR");

        assert!(ctr > 0.5 && ctr <= 0.6, "CTR should be around 0.6 (3/5)");

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_aggregate_popular_searches() {
        let pool = setup_test_db().await;
        let analytics = SearchAnalytics::new(pool.clone());

        let period_start = Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        // Log some searches
        for _ in 0..5 {
            analytics
                .query_log()
                .log_search("test query", Some("user123"), 10, 100, HashMap::new())
                .await
                .expect("Failed to log search");
        }

        analytics
            .aggregate_popular_searches(PeriodType::Hourly, period_start)
            .await
            .expect("Failed to aggregate");

        let aggregated = sqlx::query(
            r#"
            SELECT search_count, avg_results, avg_latency_ms
            FROM popular_searches
            WHERE query_text = 'test query' AND period_type = 'hourly'
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch aggregated data");

        assert_eq!(
            aggregated.get::<i32, _>("search_count"),
            5,
            "Should aggregate search count"
        );
        assert!(
            aggregated.get::<Option<f64>, _>("avg_results").is_some(),
            "Should have average results"
        );

        cleanup_test_db(&pool).await;
    }

    #[tokio::test]
    #[ignore] // Integration test - requires database
    async fn test_get_dashboard() {
        let pool = setup_test_db().await;
        let analytics = SearchAnalytics::new(pool.clone());

        // Create test data
        for i in 0..10 {
            let event_id = analytics
                .query_log()
                .log_search(
                    &format!("query {}", i % 3),
                    Some("user123"),
                    if i == 5 { 0 } else { 10 },
                    100 + i * 10,
                    HashMap::new(),
                )
                .await
                .expect("Failed to log search");

            if i % 2 == 0 {
                let content_id = Uuid::new_v4();
                analytics
                    .query_log()
                    .log_click(event_id, content_id, 0)
                    .await
                    .expect("Failed to log click");
            }
        }

        let dashboard = analytics
            .get_dashboard("24h", 5)
            .await
            .expect("Failed to get dashboard");

        assert_eq!(dashboard.period, "24h");
        assert_eq!(dashboard.total_searches, 10);
        assert!(dashboard.unique_queries > 0);
        assert!(dashboard.avg_latency_ms > 0.0);
        assert!(dashboard.p95_latency_ms > 0);
        assert!(dashboard.zero_result_rate > 0.0);
        assert!(dashboard.avg_ctr > 0.0);
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
