//! Federation Worker - Pattern aggregation across shards
//!
//! This worker periodically aggregates high-quality viewing patterns across
//! all shards, detects trends, and distributes consolidated patterns back.
//!
//! Runs every hour to:
//! 1. Collect top patterns from all shards
//! 2. Perform quality-weighted federated averaging
//! 3. Detect trending content
//! 4. Distribute results back to shards

mod config;

use config::FederationConfig;
use metrics_exporter_prometheus::PrometheusBuilder;
use omega_constellation::{register_metrics, ViewingPattern};
use sqlx::PgPool;
use std::collections::HashMap;
use tokio::signal;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,federation_worker=debug"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .json()
        .init();

    info!("Starting Exogenesis Omega Federation Worker");

    // Load configuration
    let config = FederationConfig::from_env()?;
    info!(
        "Configuration loaded: {} shards, interval={}s",
        config.shard_postgres_urls.len(),
        config.federation_interval_secs
    );

    // Register omega-constellation metrics
    register_metrics();

    // Start Prometheus metrics exporter
    let metrics_addr: std::net::SocketAddr = std::env::var("METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9091".to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid metrics address: {}", e))?;
    info!("Starting Prometheus metrics server on {}", metrics_addr);
    PrometheusBuilder::new()
        .with_http_listener(metrics_addr)
        .install()?;

    metrics::counter!("federation_worker_starts_total",
        "region" => config.region.clone()
    ).increment(1);

    // Connect to all shards
    let mut shard_pools = Vec::new();
    for (idx, url) in config.shard_postgres_urls.iter().enumerate() {
        info!("Connecting to shard {} at {}", idx, mask_url(url));
        let pool = PgPool::connect(url).await?;
        shard_pools.push(pool);
    }
    info!("Connected to {} shards", shard_pools.len());

    // Create federation interval ticker
    let mut ticker = interval(config.interval());

    info!("Federation worker running! First round in {}s", config.federation_interval_secs);

    // Run federation rounds periodically
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                info!("Starting federation round");
                let start = std::time::Instant::now();

                match run_federation_round(&config, &shard_pools).await {
                    Ok(result) => {
                        let duration = start.elapsed();
                        info!(
                            "Federation round completed: {} patterns aggregated, {} trends detected ({:.2}s)",
                            result.patterns_aggregated,
                            result.trends_detected,
                            duration.as_secs_f64()
                        );

                        metrics::counter!("federation_rounds_total", "status" => "success").increment(1);
                        metrics::histogram!("federation_round_duration_seconds").record(duration.as_secs_f64());
                        metrics::gauge!("federation_patterns_aggregated").set(result.patterns_aggregated as f64);
                        metrics::gauge!("federation_trends_detected").set(result.trends_detected as f64);
                    }
                    Err(e) => {
                        error!("Federation round failed: {}", e);
                        metrics::counter!("federation_rounds_total", "status" => "error").increment(1);
                    }
                }
            }
            _ = shutdown_signal() => {
                info!("Shutdown signal received, stopping federation worker");
                break;
            }
        }
    }

    info!("Federation worker stopped");
    Ok(())
}

/// Federation round result
#[derive(Debug)]
struct FederationResult {
    patterns_aggregated: usize,
    trends_detected: usize,
}

/// Run a single federation round
async fn run_federation_round(
    config: &FederationConfig,
    shard_pools: &[PgPool],
) -> anyhow::Result<FederationResult> {
    // 1. Collect top patterns from each shard
    info!("Collecting patterns from {} shards", shard_pools.len());
    let mut all_patterns: Vec<ShardPattern> = Vec::new();

    for (shard_id, pool) in shard_pools.iter().enumerate() {
        match collect_shard_patterns(pool, config.patterns_per_shard, config.min_quality_threshold).await {
            Ok(patterns) => {
                info!("Collected {} patterns from shard {}", patterns.len(), shard_id);
                all_patterns.extend(patterns.into_iter().map(|p| ShardPattern {
                    shard_id: shard_id as u32,
                    pattern: p,
                }));
            }
            Err(e) => {
                warn!("Failed to collect patterns from shard {}: {}", shard_id, e);
            }
        }
    }

    if all_patterns.is_empty() {
        warn!("No patterns collected from any shard");
        return Ok(FederationResult {
            patterns_aggregated: 0,
            trends_detected: 0,
        });
    }

    // 2. Perform federated averaging
    info!("Aggregating {} patterns", all_patterns.len());
    let aggregated = federated_average(&all_patterns);

    // 3. Detect trends
    info!("Detecting trends");
    let trends = detect_trends(&all_patterns, config.trend_decay_rate);

    // 4. Distribute aggregated patterns back to shards
    info!("Distributing {} aggregated patterns to shards", aggregated.len());
    for (shard_id, pool) in shard_pools.iter().enumerate() {
        if let Err(e) = distribute_to_shard(pool, &aggregated, &trends).await {
            warn!("Failed to distribute to shard {}: {}", shard_id, e);
        }
    }

    Ok(FederationResult {
        patterns_aggregated: aggregated.len(),
        trends_detected: trends.len(),
    })
}

/// Pattern from a specific shard
#[derive(Debug, Clone)]
struct ShardPattern {
    shard_id: u32,
    pattern: Pattern,
}

/// Pattern representation
#[derive(Debug, Clone)]
struct Pattern {
    id: uuid::Uuid,
    embedding: Vec<f32>,
    success_rate: f32,
    sample_count: i32,
    context_genre: String,
}

/// Aggregated pattern
#[derive(Debug, Clone)]
struct AggregatedPattern {
    embedding: Vec<f32>,
    quality_score: f32,
    source_count: usize,
    genre: String,
}

/// Trend signal
#[derive(Debug, Clone)]
struct Trend {
    content_id: String,
    trending_score: f32,
}

/// Collect high-quality patterns from a shard
async fn collect_shard_patterns(
    pool: &PgPool,
    limit: i64,
    min_quality: f32,
) -> anyhow::Result<Vec<Pattern>> {
    let patterns: Vec<Pattern> = sqlx::query_as(
        r#"
        SELECT
            id,
            embedding::REAL[] as embedding,
            success_rate,
            sample_count,
            context->>'genre_hints'->0 as context_genre
        FROM patterns
        WHERE success_rate >= $1
          AND sample_count >= 5
        ORDER BY success_rate DESC, sample_count DESC
        LIMIT $2
        "#,
    )
    .bind(min_quality)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to collect patterns: {}", e))?;

    Ok(patterns)
}

/// Perform quality-weighted federated averaging
fn federated_average(patterns: &[ShardPattern]) -> Vec<AggregatedPattern> {
    // Group patterns by genre
    let mut genre_groups: HashMap<String, Vec<&Pattern>> = HashMap::new();

    for shard_pattern in patterns {
        let genre = shard_pattern.pattern.context_genre.clone();
        genre_groups.entry(genre).or_default().push(&shard_pattern.pattern);
    }

    // Aggregate within each genre
    let mut aggregated = Vec::new();

    for (genre, group) in genre_groups {
        if group.len() < 3 {
            // Require patterns from multiple sources
            continue;
        }

        // Calculate weighted average embedding
        let total_weight: f32 = group
            .iter()
            .map(|p| p.success_rate * p.sample_count as f32)
            .sum();

        if total_weight == 0.0 {
            continue;
        }

        let dim = group[0].embedding.len();
        let avg_embedding: Vec<f32> = (0..dim)
            .map(|i| {
                group
                    .iter()
                    .map(|p| p.embedding[i] * p.success_rate * p.sample_count as f32)
                    .sum::<f32>()
                    / total_weight
            })
            .collect();

        aggregated.push(AggregatedPattern {
            embedding: avg_embedding,
            quality_score: total_weight / group.len() as f32,
            source_count: group.len(),
            genre,
        });
    }

    // Sort by quality score
    aggregated.sort_by(|a, b| b.quality_score.partial_cmp(&a.quality_score).unwrap());

    aggregated
}

/// Detect trending content
fn detect_trends(patterns: &[ShardPattern], decay_rate: f32) -> Vec<Trend> {
    let mut genre_scores: HashMap<String, f32> = HashMap::new();

    for shard_pattern in patterns {
        let genre = &shard_pattern.pattern.context_genre;
        let score = shard_pattern.pattern.success_rate * (shard_pattern.pattern.sample_count as f32).sqrt();

        *genre_scores.entry(genre.clone()).or_default() += score;
    }

    let mut trends: Vec<Trend> = genre_scores
        .into_iter()
        .map(|(content_id, score)| Trend {
            content_id,
            trending_score: score * decay_rate,
        })
        .collect();

    trends.sort_by(|a, b| b.trending_score.partial_cmp(&a.trending_score).unwrap());

    trends.into_iter().take(50).collect()
}

/// Distribute aggregated patterns to a shard
async fn distribute_to_shard(
    pool: &PgPool,
    aggregated: &[AggregatedPattern],
    trends: &[Trend],
) -> anyhow::Result<()> {
    // Store aggregated patterns in a global patterns table
    for pattern in aggregated {
        sqlx::query(
            r#"
            INSERT INTO global_patterns (embedding, quality_score, source_count, genre, updated_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (genre) DO UPDATE SET
                embedding = EXCLUDED.embedding,
                quality_score = EXCLUDED.quality_score,
                source_count = EXCLUDED.source_count,
                updated_at = NOW()
            "#,
        )
        .bind(&pattern.embedding)
        .bind(pattern.quality_score)
        .bind(pattern.source_count as i32)
        .bind(&pattern.genre)
        .execute(pool)
        .await
        .ok(); // Ignore errors for individual patterns
    }

    // Store trends
    for trend in trends {
        sqlx::query(
            r#"
            INSERT INTO trending_content (content_id, trending_score, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (content_id) DO UPDATE SET
                trending_score = EXCLUDED.trending_score,
                updated_at = NOW()
            "#,
        )
        .bind(&trend.content_id)
        .bind(trend.trending_score)
        .execute(pool)
        .await
        .ok(); // Ignore errors for individual trends
    }

    Ok(())
}

/// Wait for shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Mask sensitive parts of database URL for logging
fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        if let Some(proto_end) = url.find("://") {
            let proto = &url[..proto_end + 3];
            let host = &url[at_pos + 1..];
            return format!("{}***@{}", proto, host);
        }
    }
    "***".to_string()
}

// SQLx query macros require these trait bounds
impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Pattern {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(Pattern {
            id: row.try_get("id")?,
            embedding: row.try_get("embedding")?,
            success_rate: row.try_get("success_rate")?,
            sample_count: row.try_get("sample_count")?,
            context_genre: row.try_get::<Option<String>, _>("context_genre")?.unwrap_or_else(|| "unknown".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federated_average() {
        let patterns = vec![
            ShardPattern {
                shard_id: 0,
                pattern: Pattern {
                    id: uuid::Uuid::new_v4(),
                    embedding: vec![1.0, 0.0],
                    success_rate: 0.9,
                    sample_count: 10,
                    context_genre: "action".to_string(),
                },
            },
            ShardPattern {
                shard_id: 1,
                pattern: Pattern {
                    id: uuid::Uuid::new_v4(),
                    embedding: vec![0.8, 0.2],
                    success_rate: 0.85,
                    sample_count: 8,
                    context_genre: "action".to_string(),
                },
            },
            ShardPattern {
                shard_id: 2,
                pattern: Pattern {
                    id: uuid::Uuid::new_v4(),
                    embedding: vec![0.9, 0.1],
                    success_rate: 0.88,
                    sample_count: 9,
                    context_genre: "action".to_string(),
                },
            },
        ];

        let aggregated = federated_average(&patterns);

        assert!(!aggregated.is_empty());
        assert_eq!(aggregated[0].source_count, 3);
        assert_eq!(aggregated[0].genre, "action");
    }

    #[test]
    fn test_mask_url() {
        let url = "postgresql://user:password@localhost:5432/db";
        let masked = mask_url(url);
        assert!(!masked.contains("password"));
        assert!(masked.contains("localhost"));
    }
}
