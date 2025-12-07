use chrono::Utc;
/// Example: Using the Search Analytics system
///
/// This example demonstrates how to:
/// 1. Initialize the SearchAnalytics service
/// 2. Log search events and clicks
/// 3. Query analytics data
/// 4. Generate dashboard reports
use discovery::analytics::{PeriodType, SearchAnalytics};
use sqlx::PgPool;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/media_gateway".to_string());

    let pool = PgPool::connect(&database_url).await?;

    // Create analytics service
    let analytics = SearchAnalytics::new(pool.clone());

    println!("=== Search Analytics Example ===\n");

    // Example 1: Log a search event
    println!("1. Logging search events...");
    let mut filters = HashMap::new();
    filters.insert("genre".to_string(), serde_json::json!("action"));

    let event_id = analytics
        .query_log()
        .log_search(
            "action movies",
            Some("user123"),
            42,  // result_count
            156, // latency_ms
            filters,
        )
        .await?;

    println!("   Logged search event: {}", event_id);

    // Example 2: Log a click on search result
    println!("\n2. Logging click event...");
    let content_id = uuid::Uuid::new_v4();
    let click_id = analytics
        .query_log()
        .log_click(event_id, content_id, 0) // position 0
        .await?;

    println!("   Logged click: {}", click_id);

    // Example 3: Calculate latency statistics
    println!("\n3. Calculating latency statistics...");
    let since = Utc::now() - chrono::Duration::hours(24);
    let latency_stats = analytics.calculate_latency_stats(since).await?;

    println!("   P50 latency: {}ms", latency_stats.p50);
    println!("   P95 latency: {}ms", latency_stats.p95);
    println!("   P99 latency: {}ms", latency_stats.p99);
    println!("   Average latency: {:.2}ms", latency_stats.avg);

    // Example 4: Get top queries
    println!("\n4. Fetching top queries...");
    let top_queries = analytics.get_top_queries(since, 10).await?;

    println!("   Top {} queries:", top_queries.len());
    for (idx, query) in top_queries.iter().enumerate() {
        println!(
            "   {}. '{}' - {} searches, {:.2}% CTR",
            idx + 1,
            query.query,
            query.count,
            query.ctr * 100.0
        );
    }

    // Example 5: Get zero-result queries
    println!("\n5. Fetching zero-result queries...");
    let zero_results = analytics.get_zero_result_queries(since, 10).await?;

    println!("   Zero-result queries ({}):", zero_results.len());
    for query in &zero_results {
        println!("   - '{}': {} searches", query.query, query.count);
    }

    // Example 6: Calculate click-through rate
    println!("\n6. Calculating overall CTR...");
    let ctr = analytics.calculate_ctr(since).await?;
    println!("   Overall CTR: {:.2}%", ctr * 100.0);

    // Example 7: Aggregate popular searches
    println!("\n7. Aggregating popular searches...");
    let period_start = Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    analytics
        .aggregate_popular_searches(PeriodType::Hourly, period_start)
        .await?;

    println!(
        "   Aggregated searches for hourly period starting at {}",
        period_start
    );

    analytics
        .aggregate_popular_searches(PeriodType::Daily, period_start)
        .await?;

    println!("   Aggregated searches for daily period");

    // Example 8: Get complete dashboard
    println!("\n8. Generating analytics dashboard...");
    let dashboard = analytics.get_dashboard("24h", 10).await?;

    println!("\n=== Dashboard for {} ===", dashboard.period);
    println!("Total Searches: {}", dashboard.total_searches);
    println!("Unique Queries: {}", dashboard.unique_queries);
    println!("Average Latency: {:.2}ms", dashboard.avg_latency_ms);
    println!("P95 Latency: {}ms", dashboard.p95_latency_ms);
    println!(
        "Zero-Result Rate: {:.2}%",
        dashboard.zero_result_rate * 100.0
    );
    println!("Average CTR: {:.2}%", dashboard.avg_ctr * 100.0);

    println!("\nTop Queries:");
    for (idx, query) in dashboard.top_queries.iter().enumerate() {
        println!(
            "  {}. '{}' - {} searches, {:.2}% CTR, {:.0} avg results, {:.0}ms avg latency",
            idx + 1,
            query.query,
            query.count,
            query.ctr * 100.0,
            query.avg_results,
            query.avg_latency_ms
        );
    }

    println!("\nContent Gaps (Zero Results):");
    for query in &dashboard.zero_result_queries {
        println!("  - '{}': {} searches", query.query, query.count);
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
