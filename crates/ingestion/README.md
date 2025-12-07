# Media Gateway Ingestion Pipeline

The ingestion pipeline is responsible for fetching, normalizing, and enriching content metadata from multiple streaming platforms.

## Overview

This crate implements the complete data ingestion pipeline as specified in the SPARC architecture documents, including:

- **Platform Normalizers**: Convert platform-specific data to canonical format
- **Entity Resolution**: Match content across platforms using EIDR, external IDs, and fuzzy matching
- **Genre Mapping**: Map platform-specific genres to canonical taxonomy
- **Embedding Generation**: Create 768-dimension content embeddings for similarity search
- **Deep Link Generation**: Generate platform-specific deep links for mobile/web/TV
- **Rate Limiting**: Multi-key rotation and quota management
- **Aggregator Clients**: Unified clients for metadata APIs

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Ingestion Pipeline                        │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  Catalog     │  │ Availability │  │  Expiring    │      │
│  │  Refresh     │  │    Sync      │  │   Content    │      │
│  │  (6 hours)   │  │  (1 hour)    │  │  (15 min)    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                    ┌───────▼────────┐                        │
│                    │  Normalizers   │                        │
│                    │  - Netflix     │                        │
│                    │  - Prime Video │                        │
│                    │  - Disney+     │                        │
│                    │  - YouTube     │                        │
│                    │  - Generic     │                        │
│                    └───────┬────────┘                        │
│                            │                                 │
│         ┌──────────────────┼──────────────────┐              │
│         │                  │                  │              │
│   ┌─────▼─────┐   ┌────────▼────────┐  ┌─────▼──────┐      │
│   │  Entity   │   │     Genre       │  │ Embedding  │      │
│   │ Resolution│   │    Mapping      │  │ Generation │      │
│   └───────────┘   └─────────────────┘  └────────────┘      │
│         │                  │                  │              │
│         └──────────────────┴──────────────────┘              │
│                            │                                 │
│                    ┌───────▼────────┐                        │
│                    │    Database    │                        │
│                    └────────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. Platform Normalizers (`src/normalizer/`)

Each platform has a dedicated normalizer implementing the `PlatformNormalizer` trait:

- **Netflix** (`netflix.rs`): Via Streaming Availability API
  - Deep link: `netflix://title/{id}`
  - Genre mapping: `action-adventure` → `Action`

- **Prime Video** (`prime_video.rs`): Via Streaming Availability API
  - Deep link: `aiv://detail?asin={id}`
  - Supports both subscription and purchase/rental

- **Disney+** (`disney_plus.rs`): Via Streaming Availability API
  - Deep link: `disneyplus://content/{id}`
  - Superhero genre maps to Action + Fantasy

- **YouTube** (`youtube.rs`): Direct YouTube Data API v3
  - OAuth 2.0 authentication
  - Deep link: `youtube://watch?v={id}`
  - Multi-key rotation (5 keys, 10,000 units/day each)

- **Generic** (`generic.rs`): Fallback for any platform
  - Uses Streaming Availability API
  - Web-only deep links

### 2. Entity Resolution (`src/entity_resolution.rs`)

Implements the `ResolveContentEntity` algorithm with multiple matching strategies:

1. **EIDR Exact Match** (100% confidence)
   - Direct EIDR identifier matching
   - O(1) lookup with index

2. **External ID Match** (99% confidence)
   - IMDb ID matching
   - TMDb ID matching
   - O(1) lookup with indices

3. **Fuzzy Title + Year** (90-98% confidence)
   - Normalized Levenshtein distance
   - Threshold: 0.85
   - 1-year tolerance on release year

4. **Embedding Similarity** (85-95% confidence)
   - Cosine similarity on 768-dim embeddings
   - Threshold: 0.92
   - O(n) search (optimized with indices in production)

**Performance**: O(log n) with proper indexing

### 3. Genre Mapping (`src/genre_mapping.rs`)

Maps platform-specific genres to canonical taxonomy:

**Canonical Genres**:
- Action, Adventure, Animation, Comedy, Crime
- Documentary, Drama, Family, Fantasy, History
- Horror, Music, Mystery, Romance, Science Fiction
- Thriller, War, Western

**Platform Mappings**:
- Netflix: `action-adventure` → `Action`, `sci-fi` → `Science Fiction`
- Disney+: `superhero` → `Action` + `Fantasy`
- YouTube: `film` → `Drama`, `education` → `Documentary`

**Fuzzy Fallback**: Uses Levenshtein distance (threshold: 0.8)

### 4. Embedding Generation (`src/embedding.rs`)

Implements the `GenerateContentEmbedding` algorithm:

**Components** (768 dimensions):
1. **Text Embedding** (weight: 0.4)
   - Title + overview
   - Feature hashing (production: sentence-transformers)

2. **Metadata Embedding** (weight: 0.3)
   - Genres (50%)
   - Release year (25%)
   - User rating (25%)

3. **Graph Embedding** (weight: 0.3)
   - Platform relationships
   - Content type encoding
   - (Production: GNN-based)

**Output**: L2-normalized 768-dim vector

**Complexity**: O(d) where d=768

### 5. Deep Link Generation (`src/deep_link.rs`)

Generates platform-specific deep links:

**Supported Platforms**:
- Netflix: `netflix://title/{id}`
- Prime Video: `aiv://aiv/view?gti={id}`
- Disney+: `disneyplus://content/{id}`
- YouTube: `vnd.youtube://watch?v={id}`
- Hulu: `hulu://watch/{id}`
- HBO Max: `hbomax://content/{id}`
- Apple TV+: `videos://watch/{id}`
- Paramount+: `paramountplus://content/{id}`
- Peacock: `peacock://watch/{id}`

**Output**: Mobile URL, Web URL, TV URL

### 6. Rate Limiting (`src/rate_limit.rs`)

Multi-key rotation and quota management:

**Configurations**:
- **Streaming Availability**: 100 req/min
- **Watchmode**: 1000 req/day
- **YouTube**: 100 searches/day per key (5-key rotation = 500/day)
- **TMDb**: 40 req/10s

**Features**:
- Automatic key rotation
- Jitter to prevent thundering herd
- Per-platform quota tracking

### 7. Aggregator Clients (`src/aggregator/`)

Unified clients for metadata APIs:

- **Streaming Availability** (`streaming_availability.rs`)
  - 1-hour cache TTL
  - Search, details, changes endpoints

- **Watchmode** (`watchmode.rs`)
  - 24-hour cache TTL (due to daily limit)
  - Search, title details, sources

- **TMDb** (`tmdb.rs`)
  - 7-day cache TTL
  - Movie/TV search, details, external IDs

### 8. Pipeline Orchestration (`src/pipeline.rs`)

Scheduled ingestion tasks:

**Schedules**:
- **Catalog Refresh**: Every 6 hours
  - Full content metadata fetch
  - Entity resolution
  - Genre mapping
  - Embedding generation

- **Availability Sync**: Every 1 hour
  - Pricing updates
  - Subscription status

- **Expiring Content**: Every 15 minutes
  - Content leaving platforms soon
  - Update expiration dates

- **Metadata Enrichment**: Every 24 hours
  - Regenerate embeddings
  - Update quality scores

**Performance Target**: 500 items/s batch processing

## API Rate Limits

| API | Limit | Window | Keys |
|-----|-------|--------|------|
| Streaming Availability | 100 | 1 minute | 1 |
| Watchmode | 1000 | 24 hours | 1 |
| YouTube Data API | 10,000 units | 24 hours | 5 (rotation) |
| TMDb | 40 | 10 seconds | 1 |

## Data Flow

```
Platform API
    │
    ▼
[Fetch Catalog Delta] ─────► Rate Limiter
    │                              │
    ▼                              │
[Normalize to Canonical] <─────────┘
    │
    ├──► [Entity Resolution]
    │         │
    │         ├─ EIDR Index (O(1))
    │         ├─ IMDb Index (O(1))
    │         ├─ Fuzzy Match (O(n))
    │         └─ Embedding Similarity (O(n))
    │
    ├──► [Genre Mapping]
    │         │
    │         ├─ Platform Mappings
    │         └─ Fuzzy Fallback
    │
    └──► [Embedding Generation]
              │
              ├─ Text (0.4)
              ├─ Metadata (0.3)
              └─ Graph (0.3)
                    │
                    ▼
              L2 Normalize
                    │
                    ▼
              [Database Persist]
```

## Performance Characteristics

### Entity Resolution
- **EIDR/External ID**: O(1) with hash index
- **Fuzzy Matching**: O(n) worst case, optimized with filtering
- **Embedding**: O(n × d) where d=768

### Embedding Generation
- **Complexity**: O(d) where d=768
- **Throughput**: ~1000 embeddings/second (simplified implementation)

### Batch Processing
- **Target**: 500 items/second
- **Batch Size**: 100 items
- **Parallelization**: Per-platform, per-region

## Configuration

Environment variables:

```bash
# Streaming Availability API
STREAMING_AVAILABILITY_API_KEY=your_key_here

# Watchmode API (fallback)
WATCHMODE_API_KEY=your_key_here

# YouTube Data API (5 keys for rotation)
YOUTUBE_API_KEY_1=key1
YOUTUBE_API_KEY_2=key2
YOUTUBE_API_KEY_3=key3
YOUTUBE_API_KEY_4=key4
YOUTUBE_API_KEY_5=key5

# TMDb API
TMDB_API_KEY=your_key_here

# Scheduling (optional, defaults provided)
CATALOG_REFRESH_HOURS=6
AVAILABILITY_SYNC_HOURS=1
EXPIRING_CONTENT_MINUTES=15
METADATA_ENRICHMENT_HOURS=24
```

## Usage

```rust
use media_gateway_ingestion::{
    IngestionPipeline,
    IngestionSchedule,
    normalizer::{NetflixNormalizer, PrimeVideoNormalizer},
    EntityResolver,
    GenreMapper,
    EmbeddingGenerator,
    RateLimitManager,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let entity_resolver = EntityResolver::new();
    let genre_mapper = GenreMapper::new();
    let embedding_generator = EmbeddingGenerator::new();
    let rate_limiter = RateLimitManager::new();

    // Configure rate limits
    rate_limiter.init_defaults().await;

    // Create normalizers
    let normalizers: Vec<Arc<dyn PlatformNormalizer>> = vec![
        Arc::new(NetflixNormalizer::new(env::var("STREAMING_AVAILABILITY_API_KEY")?)),
        Arc::new(PrimeVideoNormalizer::new(env::var("STREAMING_AVAILABILITY_API_KEY")?)),
        // ... add more platforms
    ];

    // Create pipeline
    let pipeline = IngestionPipeline::new(
        normalizers,
        entity_resolver,
        genre_mapper,
        embedding_generator,
        rate_limiter,
        IngestionSchedule::default(),
        vec!["us".to_string(), "uk".to_string()], // regions
    );

    // Start ingestion
    pipeline.start().await?;

    Ok(())
}
```

## Testing

Run tests:

```bash
cargo test -p media-gateway-ingestion
```

Run with specific tests:

```bash
cargo test -p media-gateway-ingestion entity_resolution
cargo test -p media-gateway-ingestion genre_mapping
cargo test -p media-gateway-ingestion embedding
```

## File Structure

```
crates/ingestion/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs                          # Module exports
    ├── pipeline.rs                     # Main orchestration
    ├── normalizer/
    │   ├── mod.rs                      # Trait and common types
    │   ├── netflix.rs                  # Netflix normalizer
    │   ├── prime_video.rs              # Prime Video normalizer
    │   ├── disney_plus.rs              # Disney+ normalizer
    │   ├── youtube.rs                  # YouTube normalizer
    │   └── generic.rs                  # Generic fallback
    ├── entity_resolution.rs            # Entity matching
    ├── genre_mapping.rs                # Genre taxonomy
    ├── embedding.rs                    # Content embeddings
    ├── deep_link.rs                    # Deep link generation
    ├── rate_limit.rs                   # Rate limiting
    └── aggregator/
        ├── mod.rs                      # Common types
        ├── streaming_availability.rs   # Primary API
        ├── watchmode.rs                # Fallback API
        └── tmdb.rs                     # Metadata enrichment
```

## Future Enhancements

1. **Machine Learning**
   - Replace feature hashing with sentence-transformers
   - Train graph neural network for relationships
   - Learn genre embeddings from data

2. **Performance**
   - Implement vector database for embedding search
   - Add database connection pooling
   - Optimize batch sizes based on throughput

3. **Additional Platforms**
   - Max (HBO Max rebrand)
   - Crunchyroll
   - Funimation
   - Regional platforms

4. **Advanced Features**
   - Content recommendation based on embeddings
   - Anomaly detection in metadata
   - Quality scoring for entity matches

## License

See workspace LICENSE file.
