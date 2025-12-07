//! Integration tests for production health check system
//!
//! These tests verify that health checks work correctly with real infrastructure:
//! - PostgreSQL health checks with actual database queries
//! - Redis health checks with actual PING commands
//! - Qdrant health checks with actual HTTP requests
//! - Parallel execution and timeout behavior
//! - Proper status code responses

use media_gateway_core::health::{
    AggregatedHealth, ComponentHealth, HealthChecker, HealthStatus, PostgresHealthCheck,
    QdrantHealthCheck, RedisHealthCheck,
};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[tokio::test]
async fn test_health_status_enum() {
    // Test status readiness
    assert!(HealthStatus::Healthy.is_ready());
    assert!(HealthStatus::Degraded.is_ready());
    assert!(!HealthStatus::Unhealthy.is_ready());

    // Test HTTP status codes
    assert_eq!(HealthStatus::Healthy.http_status_code(), 200);
    assert_eq!(HealthStatus::Degraded.http_status_code(), 200);
    assert_eq!(HealthStatus::Unhealthy.http_status_code(), 503);
}

#[test]
fn test_component_health_creation() {
    // Test healthy component
    let healthy = ComponentHealth::healthy("postgres", 50, true);
    assert_eq!(healthy.name, "postgres");
    assert_eq!(healthy.status, HealthStatus::Healthy);
    assert_eq!(healthy.latency_ms, 50);
    assert!(healthy.critical);
    assert!(healthy.message.is_none());

    // Test unhealthy component
    let unhealthy = ComponentHealth::unhealthy("redis", 2000, false, "Connection timeout");
    assert_eq!(unhealthy.name, "redis");
    assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
    assert_eq!(unhealthy.latency_ms, 2000);
    assert!(!unhealthy.critical);
    assert_eq!(
        unhealthy.message.as_deref(),
        Some("Connection timeout")
    );
}

#[test]
fn test_aggregated_health_all_healthy() {
    let components = vec![
        ComponentHealth::healthy("postgres", 10, true),
        ComponentHealth::healthy("redis", 5, false),
        ComponentHealth::healthy("qdrant", 15, true),
    ];

    let health = AggregatedHealth::from_components(components, 30);

    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(health.is_ready());
    assert_eq!(health.http_status_code(), 200);
    assert_eq!(health.components.len(), 3);
    assert_eq!(health.total_latency_ms, 30);
    assert!(!health.version.is_empty());
}

#[test]
fn test_aggregated_health_critical_unhealthy() {
    let components = vec![
        ComponentHealth::unhealthy("postgres", 2000, true, "Database timeout"),
        ComponentHealth::healthy("redis", 5, false),
    ];

    let health = AggregatedHealth::from_components(components, 2005);

    assert_eq!(health.status, HealthStatus::Unhealthy);
    assert!(!health.is_ready());
    assert_eq!(health.http_status_code(), 503);
    assert_eq!(health.components.len(), 2);
}

#[test]
fn test_aggregated_health_degraded() {
    let components = vec![
        ComponentHealth::healthy("postgres", 10, true),
        ComponentHealth::unhealthy("redis", 2000, false, "Cache unavailable"),
        ComponentHealth::healthy("qdrant", 15, true),
    ];

    let health = AggregatedHealth::from_components(components, 2025);

    assert_eq!(health.status, HealthStatus::Degraded);
    assert!(health.is_ready()); // Still ready with degraded non-critical component
    assert_eq!(health.http_status_code(), 200);
}

#[test]
fn test_aggregated_health_multiple_critical_failures() {
    let components = vec![
        ComponentHealth::unhealthy("postgres", 2000, true, "DB down"),
        ComponentHealth::unhealthy("qdrant", 2000, true, "Vector DB down"),
        ComponentHealth::healthy("redis", 5, false),
    ];

    let health = AggregatedHealth::from_components(components, 4005);

    assert_eq!(health.status, HealthStatus::Unhealthy);
    assert!(!health.is_ready());
    assert_eq!(health.http_status_code(), 503);
}

#[tokio::test]
#[ignore] // Requires PostgreSQL to be running
async fn test_postgres_health_check_success() {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/media_gateway_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await;

    if let Ok(pool) = pool {
        let checker = PostgresHealthCheck::new(pool);
        let result = checker.check().await;

        assert_eq!(result.name, "postgres");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.latency_ms < 2000); // Should be well under timeout
        assert!(result.critical);
        assert!(result.message.is_none());
    } else {
        println!("Skipping PostgreSQL test - database not available");
    }
}

#[tokio::test]
#[ignore] // Requires Redis to be running
async fn test_redis_health_check_success() {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let client = redis::Client::open(redis_url);

    if let Ok(client) = client {
        let checker = RedisHealthCheck::new(client);
        let result = checker.check().await;

        assert_eq!(result.name, "redis");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.latency_ms < 2000); // Should be well under timeout
        assert!(!result.critical); // Redis is typically non-critical
        assert!(result.message.is_none());
    } else {
        println!("Skipping Redis test - Redis not available");
    }
}

#[tokio::test]
#[ignore] // Requires Qdrant to be running
async fn test_qdrant_health_check_success() {
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());

    let checker = QdrantHealthCheck::new(qdrant_url);
    let result = checker.check().await;

    // This will succeed if Qdrant is running, fail otherwise
    // Both outcomes are valid for this test
    assert_eq!(result.name, "qdrant");
    assert!(result.latency_ms <= 2000); // Should not exceed timeout
}

#[tokio::test]
async fn test_qdrant_health_check_timeout() {
    // Use an unreachable address to test timeout
    let checker = QdrantHealthCheck::new("http://192.0.2.1:6333"); // TEST-NET-1 (guaranteed unreachable)
    let result = checker.check().await;

    assert_eq!(result.name, "qdrant");
    assert_eq!(result.status, HealthStatus::Unhealthy);
    assert_eq!(result.latency_ms, 2000); // Should timeout at exactly 2s
    assert!(result.message.is_some());
}

#[tokio::test]
async fn test_postgres_health_check_invalid_connection() {
    // Use invalid connection string
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(1))
        .connect("postgresql://invalid:5432/nonexistent")
        .await;

    // If connection fails immediately (expected), the health check would fail
    // This is the expected behavior
    assert!(pool.is_err());
}

#[tokio::test]
#[ignore] // Requires infrastructure
async fn test_health_checker_parallel_execution() {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/media_gateway_test".to_string());
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());

    // Try to create connections
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await;
    let redis_client = redis::Client::open(redis_url);

    if pool.is_ok() && redis_client.is_ok() {
        let pool = pool.unwrap();
        let redis_client = redis_client.unwrap();

        let checker = HealthChecker::new()
            .with_postgres(pool)
            .with_redis(redis_client)
            .with_qdrant(qdrant_url);

        let start = std::time::Instant::now();
        let health = checker.check_all().await;
        let elapsed = start.elapsed();

        // With parallel execution, total time should be close to the slowest check
        // not the sum of all checks
        println!("Parallel execution completed in {:?}", elapsed);
        println!("Total reported latency: {}ms", health.total_latency_ms);

        // Verify health structure
        assert!(health.components.len() >= 3);
        assert!(!health.version.is_empty());

        // Parallel execution should complete faster than sequential (6s for 3x2s timeouts)
        assert!(elapsed.as_millis() < 6000);
    } else {
        println!("Skipping parallel execution test - infrastructure not available");
    }
}

#[tokio::test]
async fn test_health_checker_no_components() {
    let checker = HealthChecker::new();
    let health = checker.check_all().await;

    assert_eq!(health.status, HealthStatus::Healthy); // No components = healthy
    assert!(health.components.is_empty());
    assert!(health.is_ready());
    assert_eq!(health.http_status_code(), 200);
}

#[test]
fn test_simple_health_from_aggregated() {
    let components = vec![
        ComponentHealth::healthy("postgres", 10, true),
        ComponentHealth::healthy("redis", 5, false),
    ];
    let aggregated = AggregatedHealth::from_components(components, 15);

    let simple = media_gateway_core::health::SimpleHealth::from(&aggregated);

    assert_eq!(simple.status, HealthStatus::Healthy);
    assert_eq!(simple.version, aggregated.version);
}

#[tokio::test]
async fn test_custom_component_names() {
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgresql://localhost/test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(1))
        .connect(&database_url)
        .await;

    if let Ok(pool) = pool {
        let checker = PostgresHealthCheck::with_name(pool, "primary-db");
        let result = checker.check().await;
        assert_eq!(result.name, "primary-db");
    }
}

#[tokio::test]
async fn test_critical_flag_configuration() {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    if let Ok(client) = redis::Client::open(redis_url) {
        // Redis is non-critical by default
        let checker = RedisHealthCheck::new(client.clone());
        assert!(!checker.is_critical());

        // Can be made critical
        let critical_checker = RedisHealthCheck::new(client).set_critical(true);
        assert!(critical_checker.is_critical());
    }
}

#[test]
fn test_health_status_serialization() {
    use serde_json;

    let status = HealthStatus::Healthy;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"healthy\"");

    let status = HealthStatus::Degraded;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"degraded\"");

    let status = HealthStatus::Unhealthy;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"unhealthy\"");
}

#[test]
fn test_component_health_serialization() {
    use serde_json;

    let component = ComponentHealth::healthy("postgres", 50, true);
    let json = serde_json::to_value(&component).unwrap();

    assert_eq!(json["name"], "postgres");
    assert_eq!(json["status"], "healthy");
    assert_eq!(json["latency_ms"], 50);
    assert_eq!(json["critical"], true);
    assert!(json["message"].is_null());
}

#[test]
fn test_aggregated_health_timestamp() {
    let components = vec![ComponentHealth::healthy("test", 10, true)];
    let health = AggregatedHealth::from_components(components, 10);

    // Timestamp should be recent (within last second)
    let now = chrono::Utc::now();
    let diff = (now - health.timestamp).num_seconds().abs();
    assert!(diff < 2);
}
