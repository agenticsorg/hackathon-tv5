# Media Gateway Integration Tests

This directory contains the integration test framework for the Media Gateway project.

## Overview

The integration test framework provides:

- **TestContext**: Manages test database and Redis connections
- **TestClient**: HTTP client wrapper with authentication support
- **Fixtures**: Helper functions to create test data (users, content, sessions)
- **Test Suites**: Comprehensive integration tests for auth, search, and playback flows

## Architecture

```
tests/
├── src/
│   ├── lib.rs              # Public exports
│   ├── context.rs          # TestContext - database/redis setup
│   ├── client.rs           # TestClient - HTTP client wrapper
│   ├── fixtures.rs         # Test data creation helpers
│   ├── auth_tests.rs       # Auth flow integration tests
│   ├── search_tests.rs     # Search flow integration tests
│   └── playback_tests.rs   # Playback flow integration tests
├── .env.test               # Test environment configuration
└── README.md               # This file
```

## Prerequisites

### Database Setup

1. **PostgreSQL** (version 14+)
   - Create test database: `createdb media_gateway_test`
   - Or use Docker: `docker run -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:14`

2. **Redis** (version 6+)
   - Use default instance on port 6379
   - Or use Docker: `docker run -p 6379:6379 redis:6`

### Environment Variables

Copy `.env.test` to your shell environment:

```bash
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/media_gateway_test
export REDIS_URL=redis://localhost:6379/1
```

Or use `direnv` or `dotenv` to load automatically.

## Running Tests

### All Integration Tests

```bash
# Run all integration tests
cargo test --package media-gateway-tests

# Run with output
cargo test --package media-gateway-tests -- --nocapture

# Run specific test file
cargo test --package media-gateway-tests --test auth_tests

# Run specific test
cargo test --package media-gateway-tests test_auth_flow_register_verify_login_refresh
```

### Individual Test Suites

```bash
# Auth tests
cargo test --package media-gateway-tests --test auth_tests

# Search tests
cargo test --package media-gateway-tests --test search_tests

# Playback tests
cargo test --package media-gateway-tests --test playback_tests
```

### Parallel Execution

The test framework supports parallel test execution with isolated databases:

```bash
# Run tests in parallel (default)
cargo test --package media-gateway-tests -- --test-threads=4

# Run tests serially (for debugging)
cargo test --package media-gateway-tests -- --test-threads=1
```

## Test Framework Usage

### TestContext

`TestContext` manages database and Redis connections for tests:

```rust
use media_gateway_tests::TestContext;

#[tokio::test]
async fn my_test() -> Result<()> {
    // Setup
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Your test logic here
    // ...

    // Cleanup
    ctx.teardown().await?;
    Ok(())
}
```

**Features:**
- Automatic database pool creation
- Redis connection management
- Migration execution
- Cleanup helpers (TRUNCATE tables, FLUSHDB)

### TestClient

`TestClient` provides an HTTP client wrapper with authentication:

```rust
use media_gateway_tests::TestClient;

let client = TestClient::new("http://localhost:8081");

// Make unauthenticated request
let response = client.get("/api/v1/health").await?;

// Make authenticated request
let authed_client = client.with_auth("jwt-token-here");
let response = authed_client.get("/api/v1/profile").await?;

// Post JSON
let response = client.post_json("/api/v1/login", &login_request).await?;
```

**Features:**
- Bearer token authentication
- JSON serialization/deserialization
- Response status assertions
- Timeout handling (10s default)

### Fixtures

Fixtures help create test data:

```rust
use media_gateway_tests::fixtures;

// Create test user
let user = fixtures::create_test_user(&ctx).await?;

// Create unverified user
let unverified = fixtures::create_unverified_user(&ctx).await?;

// Create content
let content = fixtures::create_test_content(&ctx).await?;
let video = fixtures::create_test_content_with_type(&ctx, "video/mp4", "My Video").await?;

// Create playback session
let session = fixtures::create_test_session(&ctx, &user, &content).await?;
let session = fixtures::create_test_session_with_position(&ctx, &user, &content, 300).await?;

// Cleanup
fixtures::cleanup_user(&ctx.db_pool, user.id).await?;
fixtures::cleanup_content(&ctx.db_pool, content.id).await?;
fixtures::cleanup_session(&ctx.db_pool, session.id).await?;
```

## Test Coverage

### Auth Tests (`auth_tests.rs`)

- ✅ Complete auth flow: Register → Verify → Login → Refresh
- ✅ Duplicate email registration
- ✅ Invalid credentials
- ✅ Unverified user login
- ✅ Invalid refresh token
- ✅ Protected endpoints without auth
- ✅ Expired token handling
- ✅ Logout invalidation

### Search Tests (`search_tests.rs`)

- ✅ Search → Get details → Track view flow
- ✅ Content type filtering
- ✅ Pagination
- ✅ Empty results
- ✅ Content not found
- ✅ Search history tracking
- ✅ Invalid pagination parameters
- ✅ Popular content ranking

### Playback Tests (`playback_tests.rs`)

- ✅ Create → Update position → Resume flow
- ✅ Multiple sessions for same content
- ✅ Invalid session ID
- ✅ Negative position values
- ✅ Resume with no existing session
- ✅ Get user sessions
- ✅ Delete session
- ✅ Unauthorized access
- ✅ Session completion on position near end

## Database Schema

The test framework expects these tables:

```sql
users (id, email, password_hash, display_name, email_verified, created_at, updated_at)
content (id, title, content_type, url, metadata, views, created_at, updated_at)
playback_sessions (id, user_id, content_id, position_seconds, duration_seconds, is_completed, created_at, updated_at)
search_history (id, user_id, query, content_type, results_count, created_at)
```

Migrations are automatically applied via `ctx.run_migrations()`.

## Best Practices

### 1. Always Cleanup

```rust
#[tokio::test]
async fn my_test() -> Result<()> {
    let ctx = TestContext::new().await?;
    ctx.run_migrations().await?;

    // Test logic

    ctx.teardown().await?; // Always cleanup!
    Ok(())
}
```

### 2. Use Real Database Connections

```rust
// ✅ Good - uses real database
let user = fixtures::create_test_user(&ctx).await?;

// ❌ Bad - mocks don't test real integration
let mock_user = MockUser::new();
```

### 3. Test Complete Flows

```rust
// ✅ Good - tests entire user journey
#[tokio::test]
async fn test_complete_user_journey() {
    // Register → Verify → Login → Use API → Logout
}

// ❌ Bad - only tests single endpoint
#[tokio::test]
async fn test_just_register() {
    // Only register
}
```

### 4. Assert on Real Data

```rust
// ✅ Good - verifies actual database state
let user = create_test_user(&ctx).await?;
let db_user = sqlx::query!("SELECT * FROM users WHERE id = $1", user.id)
    .fetch_one(&ctx.db_pool)
    .await?;
assert_eq!(db_user.email, user.email);

// ❌ Bad - only checks response
assert_eq!(response.status(), 200);
```

### 5. Isolate Tests

Each test should:
- Create its own test data
- Not depend on other tests
- Clean up after itself
- Work in parallel with other tests

## Troubleshooting

### "Failed to connect to test database"

```bash
# Check PostgreSQL is running
pg_isready

# Create test database
createdb media_gateway_test

# Verify DATABASE_URL
echo $DATABASE_URL
```

### "Failed to connect to Redis"

```bash
# Check Redis is running
redis-cli ping

# Start Redis
redis-server

# Verify REDIS_URL
echo $REDIS_URL
```

### "Migration failed"

```bash
# Drop and recreate test database
dropdb media_gateway_test
createdb media_gateway_test

# Verify migration files
ls -la ../migrations/
```

### "Tests are flaky"

```bash
# Run tests serially to debug
cargo test --package media-gateway-tests -- --test-threads=1

# Enable full logging
RUST_LOG=debug cargo test --package media-gateway-tests -- --nocapture
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      redis:
        image: redis:6
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Create test database
        run: |
          createdb -h localhost -U postgres media_gateway_test
        env:
          PGPASSWORD: postgres

      - name: Run integration tests
        run: cargo test --package media-gateway-tests
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/media_gateway_test
          REDIS_URL: redis://localhost:6379/1
```

## Contributing

When adding new integration tests:

1. Follow the existing test structure
2. Use `TestContext` for setup/teardown
3. Use fixtures for test data creation
4. Test complete user flows, not just endpoints
5. Assert on real database state
6. Clean up after tests
7. Add documentation for new test patterns

## License

MIT License - see LICENSE file for details
