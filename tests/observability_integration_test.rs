//! Integration tests for the observability module
//!
//! Tests the structured logging and tracing initialization functionality.

use media_gateway_core::observability::{
    api_span, db_span, init_logging, request_span, with_correlation_id, LogConfig, LogFormat,
};

#[test]
fn test_init_logging_json_format() {
    // Test JSON format initialization
    let config = LogConfig::production("test-service-json".to_string());

    // Note: We can't actually initialize the subscriber multiple times in tests
    // because tracing_subscriber::registry().try_init() can only be called once
    // per process. This test verifies the configuration is correct.

    assert_eq!(config.format, LogFormat::Json);
    assert_eq!(config.level, "info");
    assert_eq!(config.service_name, "test-service-json");
}

#[test]
fn test_init_logging_pretty_format() {
    // Test Pretty format initialization
    let config = LogConfig::development("test-service-pretty".to_string());

    assert_eq!(config.format, LogFormat::Pretty);
    assert_eq!(config.level, "debug");
    assert_eq!(config.service_name, "test-service-pretty");
}

#[test]
fn test_with_correlation_id_execution() {
    // Test that with_correlation_id executes the function correctly
    let result = with_correlation_id("test-correlation-id-123", || {
        // Simulate some work
        let sum = 10 + 32;
        sum
    });

    assert_eq!(result, 42);
}

#[test]
fn test_with_correlation_id_string_return() {
    // Test with_correlation_id with string return type
    let result = with_correlation_id("req-xyz-789", || {
        String::from("test-result")
    });

    assert_eq!(result, "test-result");
}

#[test]
fn test_request_span_creation() {
    // Test that request span can be created
    let span = request_span("req-12345", "POST", "/api/v1/content");

    // Verify span is not disabled
    assert!(!span.is_disabled());

    // Enter the span and verify context
    let _enter = span.enter();
    // The span is now active
}

#[test]
fn test_db_span_creation() {
    // Test database span creation
    let span = db_span("INSERT", "content");

    assert!(!span.is_disabled());

    let _enter = span.enter();
}

#[test]
fn test_api_span_creation() {
    // Test external API span creation
    let span = api_span("tmdb", "/3/movie/550");

    assert!(!span.is_disabled());

    let _enter = span.enter();
}

#[test]
fn test_nested_spans() {
    // Test nested span creation and entering
    let request = request_span("req-nested-001", "GET", "/api/content/123");
    let _request_enter = request.enter();

    let db = db_span("SELECT", "content");
    let _db_enter = db.enter();

    // Both spans are now active in nested fashion
    // This simulates a request that makes a database call
}

#[test]
fn test_log_config_custom() {
    // Test custom log configuration
    let config = LogConfig::new(
        LogFormat::Json,
        "debug,sqlx=warn,actix_web=info".to_string(),
        "custom-service".to_string(),
    );

    assert_eq!(config.format, LogFormat::Json);
    assert_eq!(config.level, "debug,sqlx=warn,actix_web=info");
    assert_eq!(config.service_name, "custom-service");
}

#[test]
fn test_log_format_parse() {
    use std::str::FromStr;

    assert_eq!(LogFormat::from_str("json").unwrap(), LogFormat::Json);
    assert_eq!(LogFormat::from_str("JSON").unwrap(), LogFormat::Json);
    assert_eq!(LogFormat::from_str("pretty").unwrap(), LogFormat::Pretty);
    assert_eq!(LogFormat::from_str("PRETTY").unwrap(), LogFormat::Pretty);

    assert!(LogFormat::from_str("invalid").is_err());
    assert!(LogFormat::from_str("yaml").is_err());
}

#[test]
fn test_multiple_correlation_contexts() {
    // Test multiple sequential correlation contexts
    let result1 = with_correlation_id("ctx-001", || {
        "result1"
    });

    let result2 = with_correlation_id("ctx-002", || {
        "result2"
    });

    assert_eq!(result1, "result1");
    assert_eq!(result2, "result2");
}

#[test]
fn test_span_with_work() {
    // Test span creation with actual work inside
    let span = request_span("req-work-001", "POST", "/api/search");
    let _enter = span.enter();

    // Simulate some work
    let query = "test search query";
    let results = vec![1, 2, 3, 4, 5];

    assert_eq!(query.len(), 17);
    assert_eq!(results.len(), 5);
}
