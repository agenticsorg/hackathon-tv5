# Observability Module - Quick Reference Card

## Import

```rust
use media_gateway_core::observability::{
    init_logging, with_correlation_id, request_span, db_span, api_span,
    LogConfig, LogFormat, ObservabilityError,
};
```

## Initialize Logging

```rust
// Production (JSON)
let config = LogConfig::production("media-gateway-api".to_string());
init_logging(&config)?;

// Development (Pretty)
let config = LogConfig::development("media-gateway-dev".to_string());
init_logging(&config)?;

// Custom
let config = LogConfig::new(
    LogFormat::Json,
    "info,media_gateway=debug,sqlx=warn".to_string(),
    "my-service".to_string()
);
init_logging(&config)?;
```

## Correlation IDs

```rust
with_correlation_id("req-12345", || {
    tracing::info!("Processing request");
    // All logs include correlation_id
});
```

## Spans

```rust
// HTTP Request
let span = request_span("req-id", "POST", "/api/content");
let _enter = span.enter();

// Database
let span = db_span("SELECT", "users");
let _enter = span.enter();

// External API
let span = api_span("tmdb", "/3/movie/550");
let _enter = span.enter();
```

## Logging Levels

```rust
tracing::error!("Critical error");
tracing::warn!("Warning message");
tracing::info!("Informational");
tracing::debug!("Debug details");
tracing::trace!("Verbose trace");
```

## Structured Fields

```rust
tracing::info!(
    user_id = %user.id,
    action = "login",
    duration_ms = %elapsed.as_millis(),
    "User action completed"
);
```

## Public API

| Function | Purpose |
|----------|---------|
| `init_logging(&LogConfig)` | Initialize tracing subscriber |
| `with_correlation_id(id, fn)` | Execute function with correlation ID |
| `current_correlation_id()` | Get current correlation ID (if any) |
| `request_span(id, method, path)` | Create HTTP request span |
| `db_span(operation, table)` | Create database operation span |
| `api_span(service, endpoint)` | Create external API call span |

## Configuration

| Field | Type | Description |
|-------|------|-------------|
| `format` | `LogFormat` | `Json` or `Pretty` |
| `level` | `String` | Log level filter (e.g., "info") |
| `service_name` | `String` | Service identifier |

## Environment Variables

```bash
# Set log level
export RUST_LOG=info

# Module-specific levels
export RUST_LOG=info,media_gateway=debug,sqlx=warn,actix_web=info
```

## Complete Example

```rust
use media_gateway_core::observability::{
    init_logging, with_correlation_id, request_span, db_span,
    LogConfig, LogFormat,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let config = LogConfig::production("api".to_string());
    init_logging(&config)?;

    // Process request with correlation
    with_correlation_id("req-001", || {
        let span = request_span("req-001", "GET", "/api/users");
        let _enter = span.enter();

        tracing::info!("Processing request");

        // Database operation
        {
            let db = db_span("SELECT", "users");
            let _db_enter = db.enter();
            // ... query database ...
        }

        tracing::info!("Request completed");
    });

    Ok(())
}
```

## Files

- **Implementation**: `crates/core/src/observability.rs` (512 lines)
- **Integration Tests**: `tests/observability_integration_test.rs` (155 lines)
- **Documentation**: `docs/observability_usage_examples.md` (504 lines)
- **Summary**: `docs/BATCH_002_TASK-007_SUMMARY.md`

## Dependencies

```toml
[dependencies]
tracing = { workspace = true }
tracing-subscriber = { workspace = true }  # Features: env-filter, json, fmt
```

---

**Status**: ✅ Production-ready | **Tests**: 21 comprehensive tests | **Documentation**: Complete
