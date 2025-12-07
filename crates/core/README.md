# media-gateway-core

Core data structures and types for the Media Gateway platform.

## Overview

This crate provides the fundamental building blocks for content management, user profiles, search functionality, and error handling across the Media Gateway ecosystem.

## Features

- **Type-safe data structures** for content, users, and search
- **Comprehensive validation** using the validator crate
- **Serialization support** via serde (JSON and other formats)
- **Error handling** with detailed error types using thiserror
- **Well-documented** with examples and extensive inline documentation
- **Fully tested** with unit and integration tests

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
media-gateway-core = "0.1.0"
```

## Quick Start

```rust
use media_gateway_core::{
    CanonicalContent, ContentType, Platform, UserProfile,
};

// Create content
let content = CanonicalContent::new(
    ContentType::Movie,
    "Inception".to_string(),
    2010,
);

// Check availability
if content.is_available_on(Platform::Netflix) {
    println!("Available on Netflix!");
}

// Create user
let user = UserProfile::new(
    "user@example.com".to_string(),
    "John Doe".to_string(),
    "US".to_string(),
);

// Manage watchlist
let content_id = content.canonical_id;
user.add_to_watchlist(content_id);
```

## Modules

### `types`

Core type definitions:
- `ContentType` - Movie, Series, Episode, etc.
- `Platform` - Netflix, PrimeVideo, DisneyPlus, etc.
- `Genre` - Action, Comedy, Drama, etc.
- `AvailabilityType` - Subscription, Rental, Purchase, Free
- `VideoQuality`, `AudioQuality`, `MaturityRating`

### `models`

Domain models:

#### `content`
- `CanonicalContent` - Primary content record
- `ExternalIds` - Cross-platform identifiers
- `PlatformAvailability` - Platform-specific availability
- `SeriesMetadata` - Series/episode information
- `ContentImages` - Image assets
- `Credits` - Cast and crew

#### `user`
- `UserProfile` - User account and preferences
- `UserPreferences` - Playback and content preferences
- `Device` - Registered devices
- `PrivacySettings` - Privacy controls
- `PlatformSubscription` - Platform subscriptions
- `WatchHistoryEntry`, `WatchlistEntry`, `UserRating`

#### `search`
- `SearchQuery` - Search request parameters
- `SearchFilters` - Filter options
- `SearchResult` - Search response
- `SearchStrategy` - Vector, Graph, Keyword, Hybrid
- `AutocompleteRequest`, `AutocompleteResponse`

### `error`

Error types covering all platform operations:
- `ValidationError` - Field validation failures
- `NotFoundError` - Resource not found
- `AuthenticationError` - Auth failures
- `RateLimitError` - Rate limiting
- `DatabaseError` - Database operations
- `ExternalAPIError` - Third-party API errors
- And 14+ more specific error types

### `validation`

Validation utilities:
- `validate_imdb_id()` - IMDb ID format
- `validate_email()` - Email addresses
- `validate_language_code()` - ISO 639-1 codes
- `validate_country_code()` - ISO 3166-1 alpha-2 codes
- `validate_url()` - URLs
- `validate_rating()` - Rating ranges
- And more...

## Examples

### Content Management

```rust
use media_gateway_core::{
    CanonicalContent, ContentType, Genre, Platform,
    PlatformAvailability, AvailabilityType, VideoQuality,
};

let mut content = CanonicalContent::new(
    ContentType::Movie,
    "The Matrix".to_string(),
    1999,
);

content.genres = vec![Genre::SciFi, Genre::Action];
content.runtime_minutes = Some(136);
content.description = Some("A computer hacker learns...".to_string());

// Add platform availability
content.platform_availability.push(PlatformAvailability {
    platform: Platform::Netflix,
    availability_type: AvailabilityType::Subscription,
    price_cents: None,
    video_qualities: vec![VideoQuality::UHD, VideoQuality::HD],
    // ... other fields
});

// Check availability
assert!(content.is_available_on(Platform::Netflix));
assert!(content.is_available_in_region("US"));
```

### User Management

```rust
use media_gateway_core::{UserProfile, Platform, Genre};

let mut user = UserProfile::new(
    "user@example.com".to_string(),
    "Jane Doe".to_string(),
    "US".to_string(),
);

// Set preferences
user.preferences.favorite_genres = vec![Genre::SciFi, Genre::Action];
user.preferences.preferred_platforms = vec![Platform::Netflix];

// Manage watchlist
let content_id = uuid::Uuid::new_v4();
user.add_to_watchlist(content_id);

if user.is_in_watchlist(content_id) {
    println!("Content is in watchlist");
}

// Check subscriptions
if user.has_active_subscription(Platform::Netflix) {
    println!("User has Netflix subscription");
}
```

### Search

```rust
use media_gateway_core::{
    SearchQuery, SearchFilters, SearchStrategy,
    SortOrder, Genre, Platform,
};

let mut query = SearchQuery::new("science fiction".to_string());

// Configure search
query.strategy = SearchStrategy::Hybrid;
query.sort_order = SortOrder::HighestRated;
query.limit = 50;

// Add filters
query.filters.genres = vec![Genre::SciFi];
query.filters.platforms = vec![Platform::Netflix, Platform::PrimeVideo];
query.filters.min_rating = Some(7.0);
query.filters.min_release_year = Some(2020);

// For personalized search
let user_id = uuid::Uuid::new_v4();
query = SearchQuery::personalized("action movies".to_string(), user_id);
```

### Error Handling

```rust
use media_gateway_core::{MediaGatewayError, Result};

fn fetch_content(id: &str) -> Result<CanonicalContent> {
    if id.is_empty() {
        return Err(MediaGatewayError::validation("ID cannot be empty"));
    }

    // ... fetch logic

    Err(MediaGatewayError::not_found(id))
}

match fetch_content("123") {
    Ok(content) => println!("Found: {}", content.title),
    Err(e) => {
        if e.is_retryable() {
            println!("Retry after {} seconds", e.retry_after_seconds().unwrap_or(60));
        } else {
            eprintln!("Error: {}", e);
        }
    }
}
```

### Validation

```rust
use media_gateway_core::validation::*;

// Validate various fields
validate_imdb_id("tt0111161")?;
validate_email("user@example.com")?;
validate_language_code("en")?;
validate_country_code("US")?;
validate_release_year(2024)?;
validate_rating(8.5)?;
```

### Serialization

```rust
use media_gateway_core::CanonicalContent;

let content = CanonicalContent::new(/* ... */);

// Serialize to JSON
let json = serde_json::to_string(&content)?;
println!("{}", json);

// Deserialize from JSON
let content: CanonicalContent = serde_json::from_str(&json)?;
```

## Testing

Run tests:

```bash
cargo test
```

Run with coverage:

```bash
cargo tarpaulin --out Html
```

## Documentation

Generate documentation:

```bash
cargo doc --open
```

## SPARC Compliance

This crate implements all data structures specified in the SPARC (Specification, Pseudocode, Architecture, Refinement, Completion) methodology documents for the Media Gateway platform.

**Complexity Targets:**
- Content lookup: O(1) via UUID-based indexing
- Storage per content: ~20KB

## License

See LICENSE file in repository root.

## Contributing

See CONTRIBUTING.md in repository root.

## Architecture

This crate is designed to be:
- **Portable** - No platform-specific dependencies
- **Efficient** - Zero-cost abstractions, minimal allocations
- **Safe** - No unsafe code, comprehensive validation
- **Thread-safe** - All types implement Send + Sync
- **Async-ready** - Compatible with tokio runtime

## Performance Considerations

- UUID-based indexing for O(1) lookups
- Efficient serialization with serde
- Lazy regex compilation with once_cell
- Minimal heap allocations where possible
- Clone is cheap for small types (Copy trait used)

## Version History

### 0.1.0 (2025-12-06)
- Initial implementation
- Complete SPARC specification compliance
- 40+ structs and enums
- Comprehensive validation
- Full test coverage
