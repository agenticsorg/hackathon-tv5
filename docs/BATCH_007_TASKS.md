# BATCH_007: SONA (AI/ML Recommendation Engine) - Implementation Tasks

**Generated**: 2025-12-06
**Crate**: `/workspaces/media-gateway/crates/sona/`
**Status**: Partial Implementation - Multiple Critical Gaps
**Priority**: HIGH - Core AI/ML recommendation functionality incomplete

---

## Executive Summary

The SONA crate implements a Self-Optimizing Neural Architecture for personalized content recommendations. Analysis reveals **significant implementation gaps** with many features simulated or incomplete. While the architecture is well-designed with proper types and algorithms, actual AI/ML functionality requires substantial completion work.


### Implementation Status Overview

| Component | Status | Completeness | Priority |
|-----------|--------|--------------|----------|
| **ONNX Inference** | ✅ Implemented | 85% | LOW |
| **LoRA Adapters** | ✅ Implemented | 90% | LOW |
| **Matrix Factorization (ALS)** | ✅ Implemented | 95% | LOW |
| **Collaborative Filtering** | ✅ Implemented | 85% | MEDIUM |
| **Graph Recommendations** | ✅ Implemented | 90% | LOW |
| **Context-Aware Filtering** | ✅ Implemented | 90% | LOW |
| **A/B Testing** | ✅ Implemented | 95% | LOW |
| **Content-Based Filtering** | ⚠️ Partial | 70% | HIGH |
| **Cold Start Handling** | ❌ Simulated | 30% | HIGH |
| **Recommendation Engine** | ❌ Simulated | 40% | CRITICAL |
| **HTTP Server/API** | ❌ Incomplete | 25% | CRITICAL |
| **Database Integration** | ❌ Missing | 10% | CRITICAL |
| **ExperimentRepository** | ❌ Not Implemented | 0% | HIGH |
| **Real Embedding Storage** | ❌ Not Implemented | 0% | HIGH |

---

## Critical Issues Found

### 1. Simulated/Placeholder Implementations (CRITICAL)

Multiple core functions return empty or simulated data:

#### recommendation.rs (Lines 153-227)
- `get_collaborative_candidates()` - Returns empty Vec
- `get_content_based_candidates()` - Returns empty Vec  
- `get_context_aware_candidates()` - Returns empty Vec
- `get_watched_content_ids()` - Returns empty Vec

#### cold_start.rs (Lines 63-134)
- `get_watch_count()` - Always returns 0
- `get_genre_recommendations()` - Returns random UUIDs
- `get_demographic_recommendations()` - Returns simulated data
- `get_trending_recommendations()` - Returns simulated data

#### server.rs (Lines 44-47, 166-169, 301, 395)
- Embedding function returns `vec![0.0; 512]` (zero vector)
- Component scores simulated
- Preference vector storage simulated

#### inference.rs (Lines 294-309)
- Tokenization uses character modulo arithmetic, not real tokenizer
- Placeholder implementation incompatible with real ONNX models

### 2. Missing Types/Modules (CRITICAL)

**ExperimentRepository Not Implemented**

server.rs Line 20 references `ExperimentRepository` which doesn't exist in the crate.

Error: Server will not compile without this type.

Used in lines 181-200:
- `experiment_repo.list_running_experiments()`
- `experiment_repo.assign_variant()`  
- `experiment_repo.record_exposure()`

### 3. Incomplete HTTP Server (HIGH)

#### main.rs (Lines 1-35)
- Only has `/health` endpoint
- No recommendation, profile, LoRA training, or A/B testing endpoints
- server.rs has HTTP handler code but not integrated into main.rs

Missing Endpoints:
- POST /api/v1/recommendations
- POST /api/v1/profile/update
- POST /api/v1/lora/train  
- GET /api/v1/profile/{user_id}
- POST /api/v1/experiments/assign
- POST /api/v1/events/track
- GET /api/v1/similar/{content_id}

### 4. Database Schema Gaps (HIGH)

No migrations found for critical SONA tables:

**Missing Tables:**
1. `user_lora_adapters` - LoRA adapter persistence
2. `user_profiles` - User preference vectors
3. `viewing_events` - Event tracking  
4. `content_embeddings` - Content vector storage
5. `user_embeddings` - User vector storage
6. `recommendation_cache` - Caching layer

**Partially Implemented:**
- A/B testing tables documented in ab_testing.rs comments (lines 7-45) but migration status unknown

**Integration Issues:**
- server.rs queries non-existent `viewing_events` table (lines 26-65)

### 5. Integration Gaps (MEDIUM)

**Real Tokenizer Required:**
- Current placeholder won't work with real ONNX models
- Need HuggingFace tokenizers crate integration
- Need vocabulary mapping, attention masks, special tokens

**Real ONNX Model Required:**
- Code expects `SONA_MODEL_PATH` env variable
- No default model provided
- No model download/initialization logic
- Tests skip model-dependent tests

**Qdrant Integration:**
- Collections created but no initial data seeding
- No incremental update strategy beyond batch retrain
- Assumes embeddings already exist (no generation pipeline)

---

## Detailed Task Breakdown

## TASK 007.1: Implement ExperimentRepository (CRITICAL)

**Priority**: P0 - Blocking compilation  
**Effort**: 2-3 hours  
**Dependencies**: None

### Objective
Implement the missing ExperimentRepository type that bridges ABTestingService with HTTP server.

### Implementation

Create `/workspaces/media-gateway/crates/sona/src/experiment_repository.rs`:

```rust
//! Experiment Repository - Bridge between ABTestingService and HTTP server

use crate::ab_testing::{ABTestingService, Experiment, Variant};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ExperimentRepository {
    ab_service: ABTestingService,
}

impl ExperimentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            ab_service: ABTestingService::new(pool),
        }
    }

    pub async fn list_running_experiments(&self) -> Result<Vec<Experiment>> {
        self.ab_service.get_running_experiments().await
    }

    pub async fn assign_variant(&self, experiment_id: Uuid, user_id: Uuid) -> Result<Variant> {
        self.ab_service.assign_variant(experiment_id, user_id).await
    }

    pub async fn record_exposure(
        &self,
        experiment_id: Uuid,
        variant_id: Uuid,
        user_id: Uuid,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        self.ab_service.record_exposure(experiment_id, variant_id, user_id).await
    }

    pub async fn record_conversion(
        &self,
        experiment_id: Uuid,
        variant_id: Uuid,
        user_id: Uuid,
        metric_name: &str,
        value: f64,
    ) -> Result<()> {
        self.ab_service.record_conversion(experiment_id, variant_id, user_id, metric_name, value).await
    }
}
```

Add export to lib.rs:
```rust
pub mod experiment_repository;
pub use experiment_repository::ExperimentRepository;
```

### Acceptance Criteria
- [ ] ExperimentRepository compiles without errors
- [ ] Exported from lib.rs
- [ ] server.rs compiles successfully
- [ ] All methods have proper error handling

---

## TASK 007.2: Implement Real Content-Based Filtering (HIGH)

**Priority**: P1 - Core recommendation functionality  
**Effort**: 4-6 hours  
**Dependencies**: TASK 007.5 (content embeddings)

### Current Issue
recommendation.rs Lines 162-168 returns empty Vec instead of real Qdrant search.

### Implementation

Replace simulated function with real Qdrant vector search:

```rust
async fn get_content_based_candidates(
    profile: &UserProfile,
    qdrant_client: &QdrantClient,
    limit: usize,
) -> Result<Vec<ScoredContent>> {
    let search_result = qdrant_client
        .search_points(&SearchPoints {
            collection_name: "content_embeddings".to_string(),
            vector: profile.preference_vector.clone(),
            limit: limit as u64,
            with_payload: Some(true.into()),
            score_threshold: Some(0.7),
            ..Default::default()
        })
        .await
        .context("Failed to search content embeddings")?;

    let mut candidates = Vec::new();
    for scored_point in search_result.result {
        if let Some(payload) = scored_point.payload.get("content_id") {
            if let Some(content_id_str) = payload.as_str() {
                if let Ok(content_id) = Uuid::parse_str(content_id_str) {
                    candidates.push(ScoredContent {
                        content_id,
                        score: scored_point.score,
                        source: RecommendationType::ContentBased,
                        based_on: vec!["user_preference_vector".to_string()],
                    });
                }
            }
        }
    }
    Ok(candidates)
}
```

Update GenerateRecommendations::execute signature to add `qdrant_client: Option<&QdrantClient>`.

### Acceptance Criteria
- [ ] Returns real Qdrant search results
- [ ] Respects similarity threshold (0.7)
- [ ] Proper error handling
- [ ] Unit tests with mock Qdrant
- [ ] Integration tests with real Qdrant

---

## TASK 007.3: Implement Cold Start Handling (HIGH)

**Priority**: P1 - Critical for new users  
**Effort**: 6-8 hours  
**Dependencies**: TASK 007.6 (database schema)

### Objective
Replace all simulated cold start functions with real database queries.

### Sub-Tasks

#### 007.3.1: Real Watch Count Query (cold_start.rs Lines 63-67)

```rust
async fn get_watch_count(user_id: Uuid, pool: &PgPool) -> Result<usize> {
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM watch_progress WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(result.0 as usize)
}
```

#### 007.3.2: Genre Recommendations (Lines 69-90)

```rust
async fn get_genre_recommendations(
    genres: &[String],
    limit: usize,
    pool: &PgPool,
) -> Result<Vec<Recommendation>> {
    let query = r#"
        SELECT DISTINCT c.id, c.popularity_score
        FROM content c
        INNER JOIN content_genres cg ON c.id = cg.content_id
        WHERE LOWER(cg.genre) = ANY($1)
        ORDER BY c.popularity_score DESC
        LIMIT $2
    "#;

    let genre_array: Vec<String> = genres.iter().map(|g| g.to_lowercase()).collect();
    let rows = sqlx::query(query)
        .bind(&genre_array)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

    let mut recommendations = Vec::new();
    for row in rows {
        let content_id: Uuid = row.try_get("id")?;
        let popularity: f64 = row.try_get("popularity_score")?;

        recommendations.push(Recommendation {
            content_id,
            confidence_score: popularity as f32,
            recommendation_type: RecommendationType::ContentBased,
            based_on: vec![format!("Selected genres: {}", genres.join(", "))],
            explanation: format!("Popular in {}", genres.join(", ")),
            generated_at: Utc::now(),
            ttl_seconds: 3600,
            experiment_variant: None,
        });
    }
    Ok(recommendations)
}
```

#### 007.3.3: Trending Recommendations (Lines 116-134)

```rust
async fn get_trending_recommendations(
    limit: usize,
    pool: &PgPool,
) -> Result<Vec<Recommendation>> {
    let query = r#"
        SELECT c.id, COUNT(DISTINCT wp.user_id) as viewer_count,
               c.popularity_score
        FROM content c
        INNER JOIN watch_progress wp ON c.id = wp.content_id
        WHERE wp.last_watched >= NOW() - INTERVAL '7 days'
        GROUP BY c.id, c.popularity_score
        ORDER BY viewer_count DESC, c.popularity_score DESC
        LIMIT $1
    "#;

    let rows = sqlx::query(query)
        .bind(limit as i64)
        .fetch_all(pool)
        .await?;

    let mut recommendations = Vec::new();
    for row in rows {
        let content_id: Uuid = row.try_get("id")?;
        let viewer_count: i64 = row.try_get("viewer_count")?;
        let trending_score = (viewer_count as f64).log10() * 0.5 + 0.5;

        recommendations.push(Recommendation {
            content_id,
            confidence_score: trending_score as f32,
            recommendation_type: RecommendationType::ContentBased,
            based_on: vec!["trending_now".to_string()],
            explanation: format!("{} viewers this week", viewer_count),
            generated_at: Utc::now(),
            ttl_seconds: 1800,
            experiment_variant: None,
        });
    }
    Ok(recommendations)
}
```

All functions need `pool: &PgPool` parameter added.

### Acceptance Criteria
- [ ] All cold start functions query real database
- [ ] SQL injection prevention (parameterized queries)
- [ ] Handles empty result sets gracefully
- [ ] Performance <200ms
- [ ] Unit tests with test database

---

## TASK 007.4: Complete HTTP Server Implementation (CRITICAL)

**Priority**: P0 - Blocking deployment  
**Effort**: 8-10 hours  
**Dependencies**: TASK 007.1, 007.6

### Objective
Integrate existing HTTP handlers into main.rs and implement missing endpoints.

### Implementation

Update main.rs to include full server configuration:

```rust
use actix_web::{web, App, HttpServer, middleware};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

mod server;
use server::{AppState, health, get_recommendations};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .json()
        .init();

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());

    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let qdrant_client = qdrant_client::client::QdrantClient::from_url(&qdrant_url)
        .build()
        .expect("Failed to create Qdrant client");

    let sona_engine = media_gateway_sona::SonaEngine::new(
        media_gateway_sona::SonaConfig::default()
    );
    let lora_storage = Arc::new(media_gateway_sona::LoRAStorage::new(db_pool.clone()));
    let experiment_repo = Arc::new(media_gateway_sona::ExperimentRepository::new(db_pool.clone()));
    let cf_engine = Arc::new(
        media_gateway_sona::CollaborativeFilteringEngine::new(
            db_pool.clone(),
            qdrant_client.clone()
        )
    );

    cf_engine.initialize_collections().await.ok();

    let app_state = web::Data::new(AppState {
        engine: Arc::new(sona_engine),
        lora_storage,
        experiment_repo,
        db_pool: db_pool.clone(),
        qdrant_client: Arc::new(qdrant_client),
        cf_engine,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api/v1")
                    .route("/recommendations", web::post().to(get_recommendations))
                    .route("/profile/{user_id}", web::get().to(server::get_profile))
                    .route("/events/track", web::post().to(server::track_event))
            )
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}
```

Update AppState in server.rs:

```rust
pub struct AppState {
    pub engine: Arc<SonaEngine>,
    pub lora_storage: Arc<LoRAStorage>,
    pub experiment_repo: Arc<ExperimentRepository>,
    pub db_pool: PgPool,
    pub qdrant_client: Arc<QdrantClient>,
    pub cf_engine: Arc<CollaborativeFilteringEngine>,
}
```

Implement missing endpoints in server.rs:

```rust
pub async fn get_profile(
    user_id: web::Path<Uuid>,
    state: web::Data<AppState>,
) -> impl Responder {
    match state.load_user_profile(*user_id).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[derive(Debug, Deserialize)]
pub struct TrackEventRequest {
    user_id: Uuid,
    content_id: Uuid,
    event_type: String,
    completion_rate: Option<f32>,
    rating: Option<u8>,
}

pub async fn track_event(
    req: web::Json<TrackEventRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let result = sqlx::query(
        "INSERT INTO viewing_events (user_id, content_id, event_type, completion_rate, rating, timestamp) VALUES ($1, $2, $3, $4, $5, NOW())"
    )
    .bind(req.user_id)
    .bind(req.content_id)
    .bind(&req.event_type)
    .bind(req.completion_rate)
    .bind(req.rating.map(|r| r as i16))
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()}))
    }
}
```

### Acceptance Criteria
- [ ] All endpoints return proper HTTP status codes
- [ ] JSON serialization works correctly
- [ ] Error responses include messages
- [ ] Logging captures requests
- [ ] Request validation implemented

---

## TASK 007.5: Implement Embedding Storage (HIGH)

**Priority**: P1 - Required for vector search  
**Effort**: 6-8 hours  
**Dependencies**: TASK 007.6

### Objective
Create pipeline for generating, storing, and retrieving content embeddings.

### Implementation

Create embedding_service.rs:

```rust
use anyhow::{Context, Result};
use qdrant_client::prelude::*;
use sqlx::PgPool;
use uuid::Uuid;
use crate::inference::ONNXInference;

const CONTENT_EMBEDDINGS_COLLECTION: &str = "content_embeddings";

pub struct EmbeddingService {
    pool: PgPool,
    qdrant: QdrantClient,
    inference: Arc<ONNXInference>,
}

impl EmbeddingService {
    pub fn new(pool: PgPool, qdrant: QdrantClient, inference: Arc<ONNXInference>) -> Self {
        Self { pool, qdrant, inference }
    }

    pub async fn initialize_collection(&self) -> Result<()> {
        self.qdrant
            .create_collection(&CreateCollection {
                collection_name: CONTENT_EMBEDDINGS_COLLECTION.to_string(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: 512,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await
            .ok();
        Ok(())
    }

    pub async fn generate_and_store_embedding(&self, content_id: Uuid) -> Result<Vec<f32>> {
        let content = sqlx::query!(
            "SELECT title, description, genres FROM content WHERE id = $1",
            content_id
        )
        .fetch_one(&self.pool)
        .await?;

        let text = format!(
            "{} {} {}",
            content.title.unwrap_or_default(),
            content.description.unwrap_or_default(),
            content.genres.join(" ")
        );

        let embedding = self.inference.generate_embedding(&text).await?;

        let point = PointStruct::new(
            content_id.as_u128() as u64,
            embedding.clone(),
            [("content_id", content_id.to_string())].into(),
        );

        self.qdrant
            .upsert_points_blocking(CONTENT_EMBEDDINGS_COLLECTION, None, vec![point], None)
            .await?;

        sqlx::query!("UPDATE content SET embedding = $1 WHERE id = $2", &embedding, content_id)
            .execute(&self.pool)
            .await?;

        Ok(embedding)
    }

    pub async fn get_embedding(&self, content_id: Uuid) -> Result<Vec<f32>> {
        let result = sqlx::query_scalar!(
            "SELECT embedding FROM content WHERE id = $1",
            content_id
        )
        .fetch_one(&self.pool)
        .await?;

        if let Some(embedding) = result {
            if !embedding.is_empty() {
                return Ok(embedding);
            }
        }

        self.generate_and_store_embedding(content_id).await
    }
}
```

Replace placeholder embedding functions in server.rs with real service calls.

### Acceptance Criteria
- [ ] Embeddings generated using real ONNX model
- [ ] Stored in Qdrant and PostgreSQL
- [ ] On-demand generation for missing embeddings
- [ ] Performance <50ms per embedding
- [ ] Proper error handling

---

## TASK 007.6: Create Database Schema Migrations (CRITICAL)

**Priority**: P0 - Blocking all database operations  
**Effort**: 4-6 hours  
**Dependencies**: None

### Objective
Create SQL migrations for all missing SONA tables.

### Migrations Required

#### 010_sona_lora_adapters.sql

```sql
CREATE TABLE IF NOT EXISTS user_lora_adapters (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    adapter_name VARCHAR(255) NOT NULL DEFAULT 'default',
    version INTEGER NOT NULL DEFAULT 1,
    weights BYTEA NOT NULL,
    size_bytes BIGINT NOT NULL,
    training_iterations INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, adapter_name, version)
);

CREATE INDEX idx_user_lora_user_id ON user_lora_adapters(user_id);
```

#### 011_sona_user_profiles.sql

```sql
CREATE TABLE IF NOT EXISTS user_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    preference_vector FLOAT[] NOT NULL,
    genre_affinities JSONB NOT NULL DEFAULT '{}',
    temporal_patterns JSONB NOT NULL DEFAULT '{}',
    interaction_count INTEGER NOT NULL DEFAULT 0,
    last_update_time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT preference_vector_length CHECK (array_length(preference_vector, 1) = 512)
);
```

#### 012_sona_viewing_events.sql

```sql
CREATE TABLE IF NOT EXISTS viewing_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content_id UUID NOT NULL REFERENCES content(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,
    completion_rate FLOAT,
    rating SMALLINT,
    is_rewatch BOOLEAN NOT NULL DEFAULT FALSE,
    dismissed BOOLEAN NOT NULL DEFAULT FALSE,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT rating_range CHECK (rating IS NULL OR (rating >= 1 AND rating <= 5))
);

CREATE INDEX idx_viewing_events_user_id ON viewing_events(user_id, timestamp DESC);
```

#### 013_sona_content_embeddings.sql

```sql
ALTER TABLE content ADD COLUMN IF NOT EXISTS embedding FLOAT[];
ALTER TABLE content ADD CONSTRAINT content_embedding_length
    CHECK (embedding IS NULL OR array_length(embedding, 1) = 512);
```

#### 014_sona_ab_testing.sql

```sql
CREATE TABLE IF NOT EXISTS experiments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    traffic_allocation FLOAT NOT NULL DEFAULT 1.0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS experiment_variants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experiment_id UUID NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    weight FLOAT NOT NULL DEFAULT 0.5,
    config JSONB NOT NULL DEFAULT '{}',
    UNIQUE(experiment_id, name)
);

CREATE TABLE IF NOT EXISTS experiment_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experiment_id UUID NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    variant_id UUID NOT NULL REFERENCES experiment_variants(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(experiment_id, user_id)
);

CREATE TABLE IF NOT EXISTS experiment_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    experiment_id UUID NOT NULL REFERENCES experiments(id) ON DELETE CASCADE,
    variant_id UUID NOT NULL REFERENCES experiment_variants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    metric_name VARCHAR(255) NOT NULL,
    metric_value FLOAT NOT NULL,
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
```

### Acceptance Criteria
- [ ] All tables created successfully
- [ ] Foreign key constraints enforced
- [ ] Indexes created for performance
- [ ] Check constraints validate data

---

## Summary & Priority Order

### Critical Path (Must Complete First)

1. **TASK 007.1**: ExperimentRepository (2-3h) - Blocking compilation
2. **TASK 007.6**: Database Schema (4-6h) - Blocking all DB operations  
3. **TASK 007.4**: HTTP Server (8-10h) - Blocking deployment

### High Priority (Core Functionality)

4. **TASK 007.2**: Content-Based Filtering (4-6h)
5. **TASK 007.3**: Cold Start Handling (6-8h)
6. **TASK 007.5**: Embedding Storage (6-8h)

### Total Estimated Effort

**Critical Path**: 14-19 hours
**High Priority**: 16-22 hours  
**Total**: 30-41 hours (4-5 business days)

---

## Risk Assessment

### High Risk
- **ONNX Model Availability**: No default model provided
- **Qdrant Performance**: Vector search at scale may require tuning
- **LoRA Training**: Computational overhead could impact latency

### Medium Risk
- **Database Performance**: Complex joins for graph recommendations
- **Cache Invalidation**: Stale recommendations if TTL too long

### Low Risk
- **API Design**: Well-structured endpoints already defined
- **Type Safety**: Strong Rust type system prevents errors

---

## Dependencies & Prerequisites

### External Services
- PostgreSQL 15+ (required)
- Qdrant 1.8+ (required)
- ONNX Runtime (required)

### Models Required
- Content embedding model (ONNX format, 512-dim output)
- Tokenizer vocabulary file (HuggingFace format)

---

## Success Metrics

### Performance
- Recommendation latency: <200ms (P95)
- Embedding generation: <50ms per item
- LoRA training: <5s per user

### Quality
- Recommendation diversity: >0.3 (MMR threshold)
- Cold start conversion: >5% within first session

### Reliability
- Service uptime: >99.9%
- Error rate: <0.1%
- Database query success: >99.99%

---

**End of BATCH_007 Tasks**

Generated by Code Quality Analyzer  
Date: 2025-12-06  
Repository: media-gateway  
Crate: sona
