# Media Gateway Core - Quick Reference

## Import Statements

```rust
use media_gateway_core::{
    // Core types
    ContentType, Platform, AvailabilityType, Genre, Region,
    VideoQuality, AudioQuality, MaturityRating,

    // Models
    CanonicalContent, UserProfile, SearchQuery, SearchResult,

    // Error handling
    MediaGatewayError, Result,
};
```

## Quick Examples

### Content Operations

```rust
// Create new content
let mut content = CanonicalContent::new(
    ContentType::Movie,
    "Inception".to_string(),
    2010,
);

// Add genres
content.genres = vec![Genre::SciFi, Genre::Action, Genre::Thriller];

// Check availability
if content.is_available_on(Platform::Netflix) {
    println!("Available on Netflix!");
}

// Check region
if content.is_available_in_region("US") {
    println!("Available in US!");
}

// Serialize
let json = serde_json::to_string(&content)?;
```

### User Operations

```rust
// Create user
let mut user = UserProfile::new(
    "user@example.com".to_string(),
    "Jane Smith".to_string(),
    "US".to_string(),
);

// Manage watchlist
let content_id = Uuid::new_v4();
user.add_to_watchlist(content_id);

if user.is_in_watchlist(content_id) {
    println!("In watchlist!");
}

user.remove_from_watchlist(content_id);

// Check subscription
if user.has_active_subscription(Platform::Netflix) {
    println!("Has Netflix!");
}
```

### Search Operations

```rust
// Basic search
let query = SearchQuery::new("star wars".to_string());

// Personalized search
let user_id = Uuid::new_v4();
let mut query = SearchQuery::personalized(
    "comedy movies".to_string(),
    user_id,
);

// Add filters
query.filters.genres = vec![Genre::Comedy];
query.filters.min_release_year = Some(2020);
query.filters.platforms = vec![Platform::Netflix, Platform::Hulu];
query.filters.free_only = true;

// Configure query
query.limit = 50;
query.sort_order = SortOrder::HighestRated;
query.strategy = SearchStrategy::Hybrid;
```

### Validation

```rust
use media_gateway_core::validation::*;

// Validate various fields
validate_imdb_id("tt0111161")?;
validate_email("user@example.com")?;
validate_language_code("en")?;
validate_country_code("US")?;
validate_url("https://example.com")?;
validate_release_year(2024)?;
validate_runtime(120)?;
validate_rating(8.5)?;
```

### Error Handling

```rust
use media_gateway_core::MediaGatewayError;

// Create errors
let err = MediaGatewayError::not_found("12345");
let err = MediaGatewayError::validation("Invalid input");
let err = MediaGatewayError::rate_limit(100, "minute".to_string(), Some(60));

// Check error type
if err.is_retryable() {
    if let Some(seconds) = err.retry_after_seconds() {
        println!("Retry after {} seconds", seconds);
    }
}

if err.is_client_error() {
    println!("Client error (4xx)");
}

if err.is_server_error() {
    println!("Server error (5xx)");
}
```

## Core Enums Reference

### ContentType
- `Movie` - Feature-length movie
- `Series` - Multi-episode series
- `Episode` - Individual episode
- `Short` - Short-form content
- `Documentary` - Documentary
- `Special` - Special content

### Platform
- `Netflix`, `PrimeVideo`, `DisneyPlus`, `Hulu`
- `AppleTVPlus`, `HBOMax`, `Peacock`, `ParamountPlus`
- `YouTube`, `Crave`, `BBCiPlayer`

### AvailabilityType
- `Subscription` - Included with subscription
- `Rental` - Temporary rental
- `Purchase` - Permanent purchase
- `Free` - Free content

### Genre
- `Action`, `Adventure`, `Animation`, `Comedy`, `Crime`
- `Documentary`, `Drama`, `Family`, `Fantasy`, `Horror`
- `Mystery`, `Romance`, `SciFi`, `Thriller`, `Western`
- `Musical`, `War`, `Biography`, `History`, `Sport`
- `GameShow`, `RealityTV`, `TalkShow`, `News`

### VideoQuality
- `SD` - Standard Definition (480p)
- `HD` - High Definition (720p/1080p)
- `UHD` - Ultra HD (4K)
- `HDR` - High Dynamic Range

### SearchStrategy
- `Vector` - Vector similarity search
- `Graph` - Graph-based search
- `Keyword` - Traditional keyword search
- `Hybrid` - Combination of strategies

### SortOrder
- `Relevance`, `Newest`, `Oldest`
- `HighestRated`, `LowestRated`, `MostPopular`
- `Alphabetical`, `ReverseAlphabetical`

## Key Struct Fields

### CanonicalContent (30+ fields)
```rust
canonical_id: Uuid
content_type: ContentType
title: String
original_title: Option<String>
description: Option<String>
release_year: i32
runtime_minutes: Option<i32>
genres: Vec<Genre>
maturity_rating: Option<MaturityRating>
external_ids: ExternalIds
platform_availability: Vec<PlatformAvailability>
series_metadata: Option<SeriesMetadata>
images: ContentImages
credits: Credits
average_rating: Option<f32>
popularity_score: Option<f32>
data_quality_score: f32
```

### UserProfile (20+ fields)
```rust
user_id: Uuid
email: String
display_name: String
home_region: Region
preferences: UserPreferences
privacy: PrivacySettings
subscriptions: Vec<PlatformSubscription>
devices: Vec<Device>
watch_history: Vec<WatchHistoryEntry>
watchlist: Vec<WatchlistEntry>
ratings: Vec<UserRating>
is_active: bool
is_verified: bool
```

### SearchQuery
```rust
query_id: Uuid
user_id: Option<Uuid>
query: String
filters: SearchFilters
strategy: SearchStrategy
sort_order: SortOrder
limit: usize
offset: usize
include_facets: bool
use_personalization: bool
```

### SearchResult
```rust
query_id: Uuid
items: Vec<SearchResultItem>
total_count: usize
offset: usize
limit: usize
has_more: bool
facets: Vec<Facet>
aggregations: Option<SearchAggregations>
execution_time_ms: i64
strategy_used: SearchStrategy
```

## Common Patterns

### Builder Pattern for Content
```rust
let content = CanonicalContent::new(ContentType::Movie, "Title".to_string(), 2024);
// Then set optional fields
content.description = Some("Description".to_string());
content.runtime_minutes = Some(120);
content.genres = vec![Genre::Action];
```

### Handling Results
```rust
fn process_content(id: &str) -> Result<CanonicalContent> {
    let content = fetch_content(id)?;
    validate_content(&content)?;
    Ok(content)
}

match process_content("123") {
    Ok(content) => println!("Success: {}", content.title),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Iterating Collections
```rust
// Filter content by platform
for availability in &content.platform_availability {
    if availability.platform == Platform::Netflix {
        println!("Found on Netflix!");
    }
}

// Check user subscriptions
for sub in &user.subscriptions {
    if sub.is_active {
        println!("{:?} subscription active", sub.platform);
    }
}
```

## File Locations

All source files in: `/workspaces/media-gateway/crates/core/src/`

- `lib.rs` - Module exports
- `types/mod.rs` - Core type definitions
- `models/content.rs` - Content models
- `models/user.rs` - User models
- `models/search.rs` - Search models
- `error.rs` - Error types
- `validation.rs` - Validation utilities
- `tests.rs` - Integration tests

## Dependencies

```toml
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
validator = { workspace = true }
tokio = { workspace = true }
regex = "1.10"
once_cell = "1.19"
```

## Testing

Run tests with:
```bash
cargo test --package media-gateway-core
```

## Documentation

Generate docs with:
```bash
cargo doc --package media-gateway-core --open
```
