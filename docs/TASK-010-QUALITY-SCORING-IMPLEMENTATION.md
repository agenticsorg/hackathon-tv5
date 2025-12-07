# TASK-010: Content Quality Scoring System Implementation

## Overview
Implemented a comprehensive content quality scoring system for the Media Gateway platform to identify and track content metadata completeness and quality.

## Implementation Summary

### 1. Quality Scoring Engine (Ingestion Crate)

#### Files Modified/Created:
- `/workspaces/media-gateway/crates/ingestion/src/quality/scorer.rs` - Enhanced with `score_content()` method
- `/workspaces/media-gateway/crates/ingestion/src/quality/canonical_adapter.rs` - Existing adapter functions used
- `/workspaces/media-gateway/crates/ingestion/src/quality/mod.rs` - Existing batch functions used

#### Key Features:
- **QualityScorer struct** with configurable `QualityWeights`
- `score_content()` method returning 0.0-1.0 float score
- `batch_score_content()` for bulk operations
- Quality score decay based on freshness

#### Scoring Dimensions:
```rust
pub struct QualityWeights {
    pub has_description: f32,      // 0.15 (metadata completeness)
    pub has_poster: f32,            // 0.15 (image quality)
    pub has_backdrop: f32,          // 0.10 (image quality)
    pub has_release_year: f32,      // 0.05 (metadata completeness)
    pub has_runtime: f32,           // 0.05 (metadata completeness)
    pub has_genres: f32,            // 0.10 (metadata completeness)
    pub has_imdb_rating: f32,       // 0.15 (external ratings)
    pub has_external_ids: f32,      // 0.10 (external ratings)
    pub freshness_weight: f32,      // 0.15 (freshness factor)
}
```

### 2. Database Layer (Repository)

#### Files Modified:
- `/workspaces/media-gateway/crates/ingestion/src/repository.rs`
  - Added `find_low_quality_content()` method to `ContentRepository` trait
  - Implemented `find_low_quality_content()` in `PostgresContentRepository`
  - Added `LowQualityContentItem` struct

#### Database Query:
```sql
SELECT
    c.id, c.title, c.content_type, c.overview,
    COALESCE(c.quality_score, 0.0) as quality_score,
    pi.platform, ...
FROM content c
LEFT JOIN platform_ids pi ON pi.content_id = c.id
WHERE COALESCE(c.quality_score, 0.0) < $threshold
ORDER BY quality_score ASC
LIMIT $limit
```

### 3. Admin Quality API (Discovery Crate)

#### Files Created:
- `/workspaces/media-gateway/crates/discovery/src/server/handlers/quality.rs`

#### API Endpoint:
**GET** `/api/v1/admin/content/quality-report`

Query Parameters:
- `threshold` (f32, default: 0.6) - Quality score threshold
- `limit` (i64, default: 100) - Maximum items to return

Response:
```json
{
  "total_low_quality": 42,
  "threshold": 0.6,
  "low_quality_items": [
    {
      "id": "uuid",
      "title": "Movie Title",
      "quality_score": 0.35,
      "missing_fields": ["description", "poster", "runtime"],
      "platform": "netflix",
      "content_type": "movie"
    }
  ]
}
```

#### Missing Fields Detection:
- description (overview)
- poster (medium or large)
- runtime
- high_res_poster
- backdrop
- imdb_rating
- imdb_id
- external_ratings (TMDB/Rotten Tomatoes)
- release_year
- genres

### 4. Integration Points

#### Files Modified:
- `/workspaces/media-gateway/crates/ingestion/src/pipeline.rs`
  - Already computes quality scores in `enrich_metadata()` method
  - Calls `compute_quality_score()` for each content item
  - Updates quality_score column via `repository.update_quality_score()`

- `/workspaces/media-gateway/crates/ingestion/src/lib.rs`
  - Exported `LowQualityContentItem` for use in discovery crate

- `/workspaces/media-gateway/crates/discovery/src/server/handlers/mod.rs`
  - Exported `get_quality_report` handler

- `/workspaces/media-gateway/crates/discovery/src/server/mod.rs`
  - Added `quality_router()` function for admin routes

- `/workspaces/media-gateway/crates/discovery/Cargo.toml`
  - Added dependency: `media-gateway-ingestion = { path = "../ingestion" }`

### 5. Search Ranking Integration (Documentation)

#### File Modified:
- `/workspaces/media-gateway/crates/discovery/src/search/mod.rs`
  - Added comprehensive documentation on `reciprocal_rank_fusion()` function
  - Documented quality boost integration approach

#### Integration Steps (Future Enhancement):
1. Add `quality_score` field to `ContentSummary` struct
2. Apply quality boost in RRF algorithm:
   ```rust
   let quality_boost = result.content.quality_score * quality_weight;
   let final_score = rrf_score * (1.0 + quality_boost);
   ```
3. Configure `quality_weight` in `DiscoveryConfig` (recommended: 0.1-0.3)

### 6. Database Migration

#### File Verified:
- `/workspaces/media-gateway/migrations/013_add_quality_score.sql`

```sql
-- Add quality score column to content table
ALTER TABLE content ADD COLUMN quality_score REAL NOT NULL DEFAULT 0.0;

-- Create index for quality score queries
CREATE INDEX idx_content_quality_score ON content(quality_score);

-- Update existing content with initial quality scores
UPDATE content SET quality_score = 0.5 WHERE quality_score = 0.0;
```

### 7. Comprehensive Test Suite

#### File Created:
- `/workspaces/media-gateway/crates/ingestion/tests/quality_scoring_test.rs`

#### Test Coverage:
- Complete content scoring (high score)
- Minimal content scoring (low score)
- Partial data scoring (medium score)
- Custom weights functionality
- Batch scoring operations
- Quality report generation
- Score distribution buckets
- Missing fields summary
- Freshness decay over time
- Image quality scoring
- External ratings scoring
- Metadata completeness dimensions
- Score clamping (0.0-1.0 bounds)
- Low quality thresholds

**Total: 18 comprehensive unit tests**

## Usage Examples

### 1. Score Individual Content
```rust
use media_gateway_ingestion::quality::QualityScorer;

let scorer = QualityScorer::default();
let score = scorer.score_content(&canonical_content);
println!("Quality score: {}", score);
```

### 2. Batch Score Content
```rust
use media_gateway_ingestion::quality::batch_score_content;

let content_items = vec![
    (content1, last_updated1),
    (content2, last_updated2),
];

let scores = batch_score_content(&scorer, content_items).await;
```

### 3. Generate Quality Report
```rust
use media_gateway_ingestion::quality::generate_quality_report;

let content_items = vec![
    (content, quality_score),
    // ... more items
];

let report = generate_quality_report(content_items, 0.6);
println!("Average score: {}", report.average_score);
```

### 4. Query Low Quality Content (API)
```bash
# Get low quality content below 0.6 threshold
curl "http://localhost:8080/api/v1/admin/content/quality-report?threshold=0.6&limit=50"

# Get severely low quality content
curl "http://localhost:8080/api/v1/admin/content/quality-report?threshold=0.3&limit=20"
```

### 5. Repository Integration
```rust
use media_gateway_ingestion::repository::ContentRepository;

let low_quality = repository
    .find_low_quality_content(0.6, 100)
    .await?;

for item in low_quality {
    println!("Low quality: {} (score: {})", item.title, item.quality_score);
}
```

## Quality Scoring Formula

### Base Score Calculation
```
base_score =
    (has_description ? 0.15 : 0) +
    (has_poster ? 0.15 : 0) +
    (has_backdrop ? 0.10 : 0) +
    (has_release_year ? 0.05 : 0) +
    (has_runtime ? 0.05 : 0) +
    (has_genres ? 0.10 : 0) +
    (has_imdb_rating ? 0.15 : 0) +
    (has_external_ids ? 0.10 : 0)
```

### Freshness Decay
```
freshness_factor = max(0.5, 1.0 - days_since_update / 365)
final_score = base_score * (1.0 - freshness_weight) + freshness_factor * freshness_weight
```

### Clamping
```
final_score = clamp(final_score, 0.0, 1.0)
```

## Architecture Decisions

### 1. Configurable Weights
- Allows customization per platform or content type
- Different weights for movies vs TV shows
- Adjustable importance of different quality dimensions

### 2. Freshness Decay
- Recent content gets quality boost
- Older content (>1 year) gets capped at 50% freshness
- Encourages metadata refresh for stale content

### 3. Batch Operations
- Efficient bulk scoring via `batch_score_content()`
- Optimized database queries with limits
- Paginated quality reports for large datasets

### 4. Missing Fields Analysis
- Identifies specific metadata gaps
- Prioritizes enrichment efforts
- Aggregates common missing fields across content

### 5. Integration with Ingestion Pipeline
- Quality scores computed during metadata enrichment
- Stored in database for fast retrieval
- Indexed for efficient querying

## Performance Considerations

### 1. Database Query Optimization
- Uses `COALESCE(c.quality_score, 0.0)` for null safety
- Index on `quality_score` column for fast filtering
- LIMIT clause prevents large result sets

### 2. Batch Processing
- Processes content in configurable batches
- Async/await for non-blocking operations
- Parallel scoring for multiple items

### 3. Caching Opportunities (Future)
- Quality reports can be cached (15-30 min TTL)
- Scores change infrequently (only on metadata updates)
- Redis cache for frequently accessed reports

## Future Enhancements

### 1. ML-Based Quality Prediction
- Train models on user engagement metrics
- Predict quality based on content patterns
- Automatic weight optimization

### 2. Platform-Specific Weights
- Different scoring profiles per platform
- Netflix vs YouTube vs Disney+ quality standards
- Configurable via admin API

### 3. Quality Alerts
- Webhook notifications for low-quality content
- Automated enrichment workflows
- Integration with metadata providers

### 4. Quality Trends
- Track quality score changes over time
- Identify degrading content
- Quality improvement metrics

### 5. Search Ranking Integration
- Boost high-quality content in search results
- Configurable quality weight factor
- A/B testing for optimal boost values

## Testing

### Running Tests
```bash
# Run all quality scoring tests
cargo test --package media-gateway-ingestion quality_scoring

# Run specific test
cargo test --package media-gateway-ingestion test_score_content_complete

# Run with output
cargo test --package media-gateway-ingestion quality_scoring -- --nocapture
```

### Test Database Setup
Tests use in-memory data structures and don't require a database connection.

## Monitoring

### Metrics to Track
- Average quality score across all content
- Distribution of scores (0-0.2, 0.2-0.4, etc.)
- Number of items below quality threshold
- Most common missing fields
- Quality score trends over time

### Logging
- Quality report requests logged at INFO level
- Low quality content counts logged
- Missing fields summary logged

## API Documentation

### GET /api/v1/admin/content/quality-report

**Authentication:** Admin role required (future implementation)

**Query Parameters:**
| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| threshold | f32  | 0.6     | Quality score threshold (0.0-1.0) |
| limit     | i64  | 100     | Maximum items to return |

**Response 200 OK:**
```json
{
  "total_low_quality": 42,
  "threshold": 0.6,
  "low_quality_items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "Example Movie",
      "quality_score": 0.35,
      "missing_fields": ["description", "poster", "runtime"],
      "platform": "netflix",
      "content_type": "movie"
    }
  ]
}
```

**Response 500 Internal Server Error:**
```json
{
  "error": "Failed to fetch low-quality content: <error details>"
}
```

## Files Summary

### Created:
1. `/workspaces/media-gateway/crates/discovery/src/server/handlers/quality.rs` (268 lines)
2. `/workspaces/media-gateway/crates/ingestion/tests/quality_scoring_test.rs` (326 lines)
3. `/workspaces/media-gateway/docs/TASK-010-QUALITY-SCORING-IMPLEMENTATION.md` (this file)

### Modified:
1. `/workspaces/media-gateway/crates/ingestion/src/quality/scorer.rs` (+13 lines)
2. `/workspaces/media-gateway/crates/ingestion/src/repository.rs` (+154 lines)
3. `/workspaces/media-gateway/crates/ingestion/src/lib.rs` (+1 line)
4. `/workspaces/media-gateway/crates/ingestion/src/pipeline.rs` (+1 line import)
5. `/workspaces/media-gateway/crates/discovery/src/server/handlers/mod.rs` (+2 lines)
6. `/workspaces/media-gateway/crates/discovery/src/server/mod.rs` (+6 lines)
7. `/workspaces/media-gateway/crates/discovery/src/search/mod.rs` (+14 lines documentation)
8. `/workspaces/media-gateway/crates/discovery/Cargo.toml` (+1 line dependency)

### Verified:
1. `/workspaces/media-gateway/migrations/013_add_quality_score.sql` (exists and correct)

## Completion Status

✅ All requirements implemented:
1. ✅ QualityScorer struct with configurable scoring rules
2. ✅ score_content() method returning 0.0-1.0 float
3. ✅ batch_score_content() for bulk operations
4. ✅ Scoring dimensions: metadata completeness, image quality, freshness, external ratings
5. ✅ Quality score decay with configurable rate
6. ✅ Admin endpoint: GET /api/v1/admin/content/quality-report
7. ✅ Search ranking integration documentation
8. ✅ Migration 013 verified
9. ✅ Comprehensive unit tests (18 tests)
10. ✅ Integration with ingestion pipeline

## Implementation Date
2025-12-06

## Author
Claude Opus 4.5 (Coder Agent)
