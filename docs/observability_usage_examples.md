# Observability Module Usage Examples

## Overview

The observability module provides structured logging and distributed tracing capabilities using the `tracing` and `tracing-subscriber` crates. It supports both JSON format for production environments and pretty-printed format for development.

## Table of Contents

- [Initialization](#initialization)
- [Correlation IDs](#correlation-ids)
- [Span Helpers](#span-helpers)
- [Configuration](#configuration)
- [Best Practices](#best-practices)

## Initialization

### Production Configuration (JSON)

```rust
use media_gateway_core::observability::{init_logging, LogConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with production settings (JSON format, info level)
    let config = LogConfig::production("media-gateway-api".to_string());
    init_logging(&config)?;

    tracing::info!("Service started");
    Ok(())
}
```

Output (JSON):
```json
{
  "timestamp": "2025-12-06T16:00:00.000Z",
  "level": "INFO",
  "message": "Service started",
  "target": "my_service",
  "service_name": "media-gateway-api"
}
```

### Development Configuration (Pretty)

```rust
use media_gateway_core::observability::{init_logging, LogConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with development settings (Pretty format, debug level)
    let config = LogConfig::development("media-gateway-dev".to_string());
    init_logging(&config)?;

    tracing::debug!("Debug message visible in development");
    Ok(())
}
```

Output (Pretty):
```
2025-12-06T16:00:00.000Z DEBUG my_service: Debug message visible in development
```

### Custom Configuration

```rust
use media_gateway_core::observability::{init_logging, LogConfig, LogFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Custom configuration with module-specific log levels
    let config = LogConfig::new(
        LogFormat::Json,
        "info,media_gateway=debug,sqlx=warn,actix_web=info".to_string(),
        "media-gateway-custom".to_string(),
    );
    init_logging(&config)?;

    Ok(())
}
```

### Environment-Based Configuration

```rust
use media_gateway_core::observability::{init_logging, LogConfig};
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use RUST_LOG environment variable if set
    let mut config = LogConfig::default();

    if let Ok(rust_log) = env::var("RUST_LOG") {
        config.level = rust_log;
    }

    init_logging(&config)?;
    Ok(())
}
```

## Correlation IDs

### Basic Usage

```rust
use media_gateway_core::observability::with_correlation_id;

fn handle_request(request_id: &str) {
    with_correlation_id(request_id, || {
        tracing::info!("Processing request");

        // All logs within this closure will include correlation_id
        process_data();

        tracing::info!("Request completed");
    });
}

fn process_data() {
    tracing::debug!("Processing data");
    // This log will also include the correlation_id from the parent context
}
```

### HTTP Request Handler Example

```rust
use media_gateway_core::observability::with_correlation_id;
use uuid::Uuid;

async fn api_handler(request: HttpRequest) -> Result<HttpResponse, Error> {
    // Generate or extract correlation ID
    let correlation_id = request
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    with_correlation_id(&correlation_id, || {
        tracing::info!("Handling API request");

        // All nested operations will include this correlation_id
        let result = process_request(request);

        tracing::info!("API request completed");
        result
    })
}
```

### Async Context

```rust
use media_gateway_core::observability::with_correlation_id;

async fn async_operation(correlation_id: String) {
    with_correlation_id(&correlation_id, || {
        async move {
            tracing::info!("Starting async operation");

            // Async work here
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            tracing::info!("Async operation completed");
        }
    })
    .await;
}
```

## Span Helpers

### Request Span

```rust
use media_gateway_core::observability::request_span;

async fn handle_http_request(method: &str, path: &str, correlation_id: &str) {
    let span = request_span(correlation_id, method, path);
    let _enter = span.enter();

    tracing::info!("Processing HTTP request");

    // All logs within this scope include request metadata
    let response = process_request().await;

    tracing::info!(status = %response.status(), "Request completed");
}
```

### Database Span

```rust
use media_gateway_core::observability::db_span;

async fn query_user(user_id: &str) -> Result<User, Error> {
    let span = db_span("SELECT", "users");
    let _enter = span.enter();

    tracing::debug!(user_id = %user_id, "Querying user");

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

    tracing::debug!("User found");
    Ok(user)
}

async fn insert_content(content: &Content) -> Result<(), Error> {
    let span = db_span("INSERT", "content");
    let _enter = span.enter();

    tracing::info!(content_id = %content.id, "Inserting content");

    sqlx::query(
        "INSERT INTO content (id, title, description) VALUES ($1, $2, $3)"
    )
    .bind(&content.id)
    .bind(&content.title)
    .bind(&content.description)
    .execute(&pool)
    .await?;

    Ok(())
}
```

### External API Span

```rust
use media_gateway_core::observability::api_span;

async fn fetch_movie_data(movie_id: i64) -> Result<MovieData, Error> {
    let span = api_span("tmdb", &format!("/3/movie/{}", movie_id));
    let _enter = span.enter();

    tracing::info!(movie_id = %movie_id, "Fetching movie data from TMDB");

    let response = reqwest::get(&url).await?;

    tracing::info!(
        status = %response.status(),
        "TMDB API response received"
    );

    let movie_data = response.json().await?;
    Ok(movie_data)
}
```

### Nested Spans

```rust
use media_gateway_core::observability::{request_span, db_span, api_span};

async fn complex_operation(request_id: &str) {
    let request = request_span(request_id, "POST", "/api/content/sync");
    let _request_enter = request.enter();

    tracing::info!("Starting complex operation");

    // Database operation
    {
        let db = db_span("SELECT", "content");
        let _db_enter = db.enter();

        tracing::debug!("Fetching existing content");
        let existing = fetch_content().await;
    }

    // External API call
    {
        let api = api_span("tmdb", "/3/movie/popular");
        let _api_enter = api.enter();

        tracing::info!("Fetching popular movies");
        let movies = fetch_popular_movies().await;
    }

    tracing::info!("Complex operation completed");
}
```

## Configuration

### Log Levels

Set log levels using the `RUST_LOG` environment variable or in code:

```bash
# Environment variable
export RUST_LOG=debug
export RUST_LOG=info,media_gateway=debug,sqlx=warn
```

```rust
// In code
let config = LogConfig::new(
    LogFormat::Json,
    "info,media_gateway=debug,sqlx=warn".to_string(),
    "my-service".to_string(),
);
```

### Format Selection

```rust
use media_gateway_core::observability::{LogConfig, LogFormat};

// JSON format (production)
let json_config = LogConfig::new(
    LogFormat::Json,
    "info".to_string(),
    "prod-service".to_string(),
);

// Pretty format (development)
let pretty_config = LogConfig::new(
    LogFormat::Pretty,
    "debug".to_string(),
    "dev-service".to_string(),
);
```

### Module-Specific Levels

```rust
let config = LogConfig::new(
    LogFormat::Json,
    // Format: "default_level,module1=level1,module2=level2"
    "info,media_gateway_api=debug,sqlx=warn,actix_web=info,reqwest=warn".to_string(),
    "media-gateway".to_string(),
);
```

## Best Practices

### 1. Initialize Early

Initialize logging as early as possible in your application:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging first
    let config = LogConfig::production("media-gateway-api".to_string());
    init_logging(&config)?;

    tracing::info!("Application starting");

    // Rest of application initialization
    run_server().await?;

    Ok(())
}
```

### 2. Use Correlation IDs for Request Tracing

Always use correlation IDs for HTTP requests:

```rust
async fn api_middleware(req: HttpRequest) -> Result<HttpResponse, Error> {
    let correlation_id = extract_or_generate_correlation_id(&req);

    with_correlation_id(&correlation_id, || {
        async move {
            // Handle request
            handle_request(req).await
        }
    })
    .await
}
```

### 3. Add Contextual Information

Use structured fields for better log filtering:

```rust
tracing::info!(
    user_id = %user.id,
    action = "login",
    ip_address = %req.peer_addr(),
    "User login successful"
);
```

### 4. Use Appropriate Log Levels

- `error!` - Errors that require immediate attention
- `warn!` - Warning conditions that should be investigated
- `info!` - Significant events in normal operation
- `debug!` - Detailed information for debugging
- `trace!` - Very detailed information (rarely used)

```rust
tracing::error!(error = ?err, "Database connection failed");
tracing::warn!(retries = %retry_count, "Retrying failed operation");
tracing::info!(user_id = %user.id, "User registered");
tracing::debug!(query = %sql, "Executing query");
```

### 5. Span Best Practices

```rust
// Create spans for significant operations
let span = request_span(correlation_id, method, path);

// Use _enter to ensure span is entered
let _enter = span.enter();

// Add dynamic fields after creation
span.record("user_id", &user.id);
span.record("duration_ms", elapsed.as_millis());
```

### 6. Error Logging

```rust
match dangerous_operation().await {
    Ok(result) => {
        tracing::info!("Operation successful");
        Ok(result)
    }
    Err(e) => {
        tracing::error!(
            error = ?e,
            operation = "dangerous_operation",
            "Operation failed"
        );
        Err(e)
    }
}
```

### 7. Performance Considerations

```rust
// Don't do expensive operations in log statements
// Bad:
tracing::debug!("Data: {}", expensive_serialization(&data));

// Good:
if tracing::level_enabled!(tracing::Level::DEBUG) {
    tracing::debug!("Data: {}", expensive_serialization(&data));
}

// Or use lazy evaluation
tracing::debug!(data = ?data, "Processing data");
```

## Integration with OpenTelemetry

For production deployments, consider integrating with OpenTelemetry for distributed tracing:

```rust
// This is an advanced setup - observability module provides the foundation
use opentelemetry::global;
use tracing_subscriber::layer::SubscriberExt;

// Future enhancement: OpenTelemetry integration
// The current observability module provides the tracing foundation
// that can be extended with OpenTelemetry layers
```

## Troubleshooting

### Logs Not Appearing

1. Check that `init_logging()` was called before any logging
2. Verify `RUST_LOG` environment variable is set correctly
3. Check log level configuration

### Performance Issues

1. Reduce log level in production (use `info` or `warn`)
2. Use JSON format (more efficient than Pretty)
3. Avoid expensive operations in log statements

### Correlation IDs Not Showing

1. Ensure you're using `with_correlation_id` wrapper
2. Verify spans are properly entered
3. Check JSON output format for correlation_id field

## Summary

The observability module provides:

- ✅ Production-ready structured logging (JSON/Pretty formats)
- ✅ Correlation ID support for request tracing
- ✅ Span helpers for HTTP, database, and API operations
- ✅ Configurable log levels per module
- ✅ Integration with tracing ecosystem
- ✅ Foundation for OpenTelemetry integration

For more information, see:
- [tracing documentation](https://docs.rs/tracing)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber)
