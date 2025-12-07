# Search Analytics and Query Insights

## Overview

The Search Analytics system provides comprehensive tracking and analysis of search behavior, enabling data-driven optimization and content gap identification.

## Features

### 1. Search Event Tracking
- Query logging with SHA-256 hashing for deduplication
- User ID anonymization for privacy compliance
- Filter tracking (genres, platforms, year/rating ranges)
- Result count and latency measurement

### 2. Click-Through Tracking
- Position-aware click logging
- Event-based attribution
- CTR calculation per query and overall

### 3. Latency Analytics
- Percentile calculations (P50, P95, P99)
- Average latency tracking
- Time-series optimization for fast queries

### 4. Popular Searches
- Hourly, daily, and weekly aggregations
- Query frequency tracking
- Average results and latency per query
- CTR metrics per query

### 5. Content Gap Analysis
- Zero-result query identification
- Frequency tracking for missing content
- Priority ranking for content acquisition

## Database Schema

### search_events
```sql
CREATE TABLE search_events (
    id UUID PRIMARY KEY,
    query_hash VARCHAR(64) NOT NULL,      -- SHA-256 hash for deduplication
    query_text VARCHAR(500) NOT NULL,     -- Original query
    user_id_hash VARCHAR(64),             -- Anonymized user ID
    result_count INTEGER NOT NULL,        -- Number of results returned
    latency_ms INTEGER NOT NULL,          -- Query execution time
    filters_applied JSONB,                -- Applied filters
    created_at TIMESTAMP WITH TIME ZONE
);
```

### search_clicks
```sql
CREATE TABLE search_clicks (
    id UUID PRIMARY KEY,
    search_event_id UUID REFERENCES search_events(id),
    content_id UUID NOT NULL,             -- Clicked content
    position INTEGER NOT NULL,            -- Result position (0-indexed)
    clicked_at TIMESTAMP WITH TIME ZONE
);
```

### popular_searches
```sql
CREATE TABLE popular_searches (
    id UUID PRIMARY KEY,
    query_text VARCHAR(500) NOT NULL,
    period_type VARCHAR(10) NOT NULL,     -- 'hourly', 'daily', 'weekly'
    period_start TIMESTAMP WITH TIME ZONE,
    search_count INTEGER,
    avg_results FLOAT,
    avg_latency_ms FLOAT,
    ctr FLOAT,
    UNIQUE(query_text, period_type, period_start)
);
```

## API Usage

### Log Search Event

```rust
use discovery::analytics::SearchAnalytics;
use std::collections::HashMap;

let analytics = SearchAnalytics::new(pool);

let mut filters = HashMap::new();
filters.insert("genre".to_string(), serde_json::json!("action"));

let event_id = analytics
    .query_log()
    .log_search(
        "action movies",
        Some("user123"),
        42,     // result_count
        156,    // latency_ms
        filters,
    )
    .await?;
```

### Log Click

```rust
let click_id = analytics
    .query_log()
    .log_click(event_id, content_id, 0)
    .await?;
```

### Get Latency Statistics

```rust
use chrono::{Utc, Duration};

let since = Utc::now() - Duration::hours(24);
let stats = analytics.calculate_latency_stats(since).await?;

println!("P50: {}ms", stats.p50);
println!("P95: {}ms", stats.p95);
println!("P99: {}ms", stats.p99);
```

### Get Top Queries

```rust
let top_queries = analytics.get_top_queries(since, 10).await?;

for query in top_queries {
    println!(
        "{}: {} searches, {:.2}% CTR",
        query.query,
        query.count,
        query.ctr * 100.0
    );
}
```

### Get Zero-Result Queries

```rust
let zero_results = analytics
    .get_zero_result_queries(since, 10)
    .await?;

for query in zero_results {
    println!("{}: {} searches", query.query, query.count);
}
```

### Aggregate Popular Searches

```rust
use discovery::analytics::PeriodType;

let period_start = Utc::now()
    .date_naive()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .and_utc();

analytics
    .aggregate_popular_searches(PeriodType::Hourly, period_start)
    .await?;

analytics
    .aggregate_popular_searches(PeriodType::Daily, period_start)
    .await?;
```

### Get Dashboard

```rust
let dashboard = analytics.get_dashboard("24h", 10).await?;

println!("Total Searches: {}", dashboard.total_searches);
println!("Unique Queries: {}", dashboard.unique_queries);
println!("Zero-Result Rate: {:.2}%", dashboard.zero_result_rate * 100.0);
println!("Average CTR: {:.2}%", dashboard.avg_ctr * 100.0);
```

## HTTP API Endpoint

### GET /api/v1/admin/search/analytics

Query parameters:
- `period` (optional): Time period - "1h", "24h", "7d", "30d" (default: "24h")
- `limit` (optional): Number of top/zero-result queries (default: 10)

Response:
```json
{
  "period": "24h",
  "total_searches": 15420,
  "unique_queries": 8750,
  "avg_latency_ms": 156.3,
  "p95_latency_ms": 342,
  "zero_result_rate": 0.08,
  "avg_ctr": 0.32,
  "top_queries": [
    {
      "query": "action movies",
      "count": 523,
      "ctr": 0.45,
      "avg_results": 42.3,
      "avg_latency_ms": 145.2
    }
  ],
  "zero_result_queries": [
    {
      "query": "xyz nonexistent",
      "count": 12
    }
  ]
}
```

## Axum Router Setup

```rust
use discovery::analytics::SearchAnalytics;
use discovery::server::analytics_router;

let analytics = SearchAnalytics::new(pool);
let router = analytics_router(Arc::new(analytics));

// Mount on main application
app.nest("/", router);
```

## Privacy & Security

### User ID Anonymization
All user IDs are hashed using SHA-256 before storage:

```rust
use discovery::analytics::QueryLog;

let anonymized = QueryLog::anonymize_user_id("user123");
// Returns: 64-character hex string
```

### Query Hashing
Queries are hashed for efficient deduplication:

```rust
let hash = QueryLog::hash_query("action movies");
// Returns: 64-character SHA-256 hash
```

## Performance Optimization

### Time-Series Indexes
```sql
CREATE INDEX idx_search_events_time ON search_events(created_at DESC);
CREATE INDEX idx_search_events_query_hash ON search_events(query_hash);
CREATE INDEX idx_search_events_result_count ON search_events(result_count)
    WHERE result_count = 0;
```

### Aggregation Tables
Pre-computed aggregations reduce query load:
- Hourly aggregations for real-time dashboards
- Daily aggregations for trend analysis
- Weekly aggregations for reporting

### Non-Blocking Logging
Search event logging is spawned as a background task to avoid blocking search responses:

```rust
tokio::spawn(async move {
    let _ = analytics
        .query_log()
        .log_search(query, user_id, result_count, latency_ms, filters)
        .await;
});
```

## Monitoring & Alerting

### Key Metrics to Monitor
1. **Zero-Result Rate**: Sudden increases indicate content gaps
2. **P95 Latency**: Performance degradation indicator
3. **CTR**: User engagement and relevance metric
4. **Query Volume**: Traffic patterns and trends

### Recommended Thresholds
- Zero-result rate > 15%: Content gap alert
- P95 latency > 500ms: Performance alert
- CTR < 20%: Relevance issue
- Query volume drop > 30%: Availability alert

## Integration with Search Service

Analytics is automatically integrated into `HybridSearchService`:

```rust
let search_service = HybridSearchService::new(
    config,
    intent_parser,
    vector_search,
    keyword_search,
    db_pool,
    cache,
);

// Analytics are logged automatically on each search
let response = search_service.search(request).await?;

// Access analytics service
if let Some(analytics) = search_service.analytics() {
    let dashboard = analytics.get_dashboard("24h", 10).await?;
}
```

## Testing

### Unit Tests
All analytics components include comprehensive unit tests:
```bash
cargo test --package discovery --lib analytics
```

### Integration Tests
Integration tests use real PostgreSQL database:
```bash
# Set DATABASE_URL
export DATABASE_URL=postgres://postgres:postgres@localhost/media_gateway_test

# Run integration tests
cargo test --package discovery --test search_analytics_integration_test -- --ignored
```

### Example Usage
See `examples/analytics_usage.rs` for complete usage examples:
```bash
cargo run --package discovery --example analytics_usage
```

## Best Practices

1. **Regular Aggregation**: Run aggregation jobs hourly/daily via cron
2. **Data Retention**: Archive old events after 90 days
3. **Index Maintenance**: Regular VACUUM and ANALYZE on search_events
4. **Privacy Compliance**: Never log personally identifiable information
5. **Dashboard Caching**: Cache dashboard results for 5-15 minutes
6. **Alert Integration**: Connect metrics to monitoring systems

## Future Enhancements

- [ ] Real-time analytics streaming
- [ ] ML-based query suggestions
- [ ] A/B test variant tracking
- [ ] Conversion funnel analysis
- [ ] Geographic search patterns
- [ ] Seasonal trend detection
- [ ] Automated content gap reports
