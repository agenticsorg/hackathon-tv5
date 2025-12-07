# SONA Crate Implementation Gap Analysis

**Analysis Date**: 2025-12-06
**Crate Path**: `/workspaces/media-gateway/crates/sona/`
**Total Lines of Code**: 4,149 lines
**Analysis Scope**: Recommendation strategies, ML model integration, training/inference pipelines

---

## Executive Summary

The SONA crate has **strong foundational architecture** with well-designed algorithms for user profiling, LoRA adaptation, and hybrid recommendations. However, **critical ML integration gaps** exist that prevent production deployment. The implementation is approximately **60% complete** - core algorithms are implemented but lack real data integration and production-ready inference pipelines.

### Overall Assessment

✅ **Implemented (60%)**:
- User preference vector building with temporal decay
- LoRA adapter mathematics (forward pass, gradient descent)
- LoRA persistence layer with <2ms retrieval
- Collaborative filtering (database-backed)
- Content-based filtering (Qdrant vector search)
- Context-aware filtering (time/device/mood)
- Diversity filtering (MMR algorithm)
- Cold start handling
- A/B testing framework

❌ **Missing (40%)**:
- Graph-based recommendation engine
- ONNX Runtime integration for inference
- Embedding generation pipeline
- LoRA training orchestration
- Real-time model serving
- Batch inference workflows

---

## 1. Missing Recommendation Strategies

### 1.1 Graph-Based Recommendations ❌ CRITICAL GAP

**Current Status**: Stub implementation only

**Location**: `/workspaces/media-gateway/crates/sona/src/recommendation.rs:144-151`

```rust
async fn get_graph_based_candidates(
    _profile: &UserProfile,
    limit: usize,
) -> Result<Vec<ScoredContent>> {
    // Simulated graph-based filtering
    // In real implementation: traverse content graph
    Ok(Vec::new())  // ❌ Returns empty vector
}
```

**Impact**:
- Graph-based recommendations are weighted at **30%** in hybrid algorithm
- Currently returning no candidates, reducing recommendation quality
- Missing content relationship traversal (e.g., "People who watched X also watched Y")

**Required Implementation**:

1. **Content Graph Database Integration**
   - Neo4j or PostgreSQL recursive CTEs
   - Node types: Content, Genre, Actor, Director, Franchise
   - Edge types: SIMILAR_TO, SEQUEL_OF, SAME_GENRE, SAME_CAST

2. **Graph Traversal Algorithms**
   - Personalized PageRank for user-specific graph importance
   - Random walk with restart for exploration
   - Community detection for genre clustering

3. **Scoring Function**
   - Path distance weighting
   - Node importance (popularity, recency)
   - User affinity to discovered nodes

**Recommended Approach**:
```rust
pub struct GraphBasedEngine {
    pool: PgPool,
    max_depth: usize,
}

impl GraphBasedEngine {
    async fn traverse_content_graph(
        &self,
        seed_content_ids: &[Uuid],
        user_preferences: &UserProfile,
        max_depth: usize,
    ) -> Result<Vec<ScoredContent>> {
        // Recursive CTE for graph traversal
        let query = r#"
            WITH RECURSIVE content_graph AS (
                SELECT content_id, related_content_id, relationship_type, 1 as depth
                FROM content_relationships
                WHERE content_id = ANY($1)

                UNION ALL

                SELECT cr.content_id, cr.related_content_id, cr.relationship_type, cg.depth + 1
                FROM content_relationships cr
                JOIN content_graph cg ON cr.content_id = cg.related_content_id
                WHERE cg.depth < $2
            )
            SELECT related_content_id, COUNT(*) as path_count, AVG(depth) as avg_distance
            FROM content_graph
            GROUP BY related_content_id
            ORDER BY path_count DESC, avg_distance ASC
            LIMIT $3
        "#;

        // Execute and score results
    }
}
```

**Files to Create**:
- `/workspaces/media-gateway/crates/sona/src/graph_based.rs` (~300 lines)
- Migration: `migrations/20250106_create_content_relationships.sql`

---

### 1.2 Real-Time Contextual Boosting ⚠️ PARTIAL

**Current Status**: Basic context filtering exists but lacks real-time signals

**What's Implemented**:
- Time-of-day filtering ✅
- Device type filtering ✅
- Mood-based filtering ✅

**What's Missing**:
- Real-time trending boost
- Seasonal event detection (holidays, sports events)
- User session context (binge-watching mode detection)
- Weather-based recommendations
- Social context (viewing party mode)

**Gap Impact**: Medium - Basic context filtering works but misses personalization opportunities

---

## 2. Incomplete ML Model Integration

### 2.1 ONNX Runtime Integration ❌ CRITICAL GAP

**Current Status**: `ort` dependency declared in `Cargo.toml` but **NEVER USED**

**Evidence**:
```bash
$ grep -r "ort::\|SessionBuilder\|InferenceSession" crates/sona/src/
# No matches found ❌
```

**Dependency Declared**:
```toml
# crates/sona/Cargo.toml:35
ort = { version = "2.0.0-rc.10", features = ["download-binaries"] }
```

**Impact**:
- Cannot load pre-trained embedding models
- Cannot perform real inference on content/user data
- All embedding calls return dummy vectors: `Ok(vec![0.0; 512])`

**Required Implementation**:

1. **Embedding Model Service**

```rust
// File: crates/sona/src/inference/embedding_service.rs

use ort::{Session, SessionBuilder, Value};
use ndarray::Array2;

pub struct EmbeddingService {
    session: Session,
    input_dim: usize,
    output_dim: usize,
}

impl EmbeddingService {
    pub fn new(model_path: &str) -> Result<Self> {
        let session = SessionBuilder::new()?
            .with_optimization_level(ort::GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        Ok(Self {
            session,
            input_dim: 768,  // From model metadata
            output_dim: 512,
        })
    }

    pub async fn encode_content(&self, content_id: Uuid) -> Result<Vec<f32>> {
        // 1. Fetch content metadata from database
        // 2. Prepare input features (title, genres, cast, etc.)
        // 3. Run ONNX inference
        // 4. Return embedding vector
    }

    pub async fn encode_text(&self, text: &str) -> Result<Vec<f32>> {
        // For query embeddings, title embeddings
    }
}
```

2. **Model Loading on Startup**

```rust
// File: crates/sona/src/main.rs

#[actix_web::main]
async fn main() -> Result<()> {
    // Load embedding model
    let embedding_model_path = std::env::var("EMBEDDING_MODEL_PATH")
        .unwrap_or("models/content_encoder.onnx".to_string());

    let embedding_service = Arc::new(
        EmbeddingService::new(&embedding_model_path)?
    );

    // Wire into HTTP handlers
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(embedding_service.clone()))
            .route("/recommendations", web::post().to(get_recommendations))
    })
}
```

3. **Replace Stub Embedding Calls**

**Current Stub** (in 3 locations):
```rust
let get_embedding = |content_id: Uuid| -> Result<Vec<f32>> {
    // In production, this would query the embedding database
    Ok(vec![0.0; 512])  // ❌ STUB
};
```

**Should Be**:
```rust
let get_embedding = {
    let embedding_service = embedding_service.clone();
    move |content_id: Uuid| -> Result<Vec<f32>> {
        // Try cache first
        if let Some(cached) = embedding_cache.get(&content_id) {
            return Ok(cached);
        }

        // Run inference or fetch from database
        let embedding = embedding_service.encode_content(content_id).await?;
        embedding_cache.insert(content_id, embedding.clone());
        Ok(embedding)
    }
};
```

**Files to Create**:
- `/workspaces/media-gateway/crates/sona/src/inference/mod.rs` (~50 lines)
- `/workspaces/media-gateway/crates/sona/src/inference/embedding_service.rs` (~400 lines)
- `/workspaces/media-gateway/crates/sona/src/inference/model_cache.rs` (~200 lines)

**Models Needed**:
- `models/content_encoder.onnx` - Content → 512-dim embedding
- `models/text_encoder.onnx` - Text → 512-dim embedding (for queries)
- `models/user_encoder.onnx` - User history → 512-dim embedding (optional)

---

### 2.2 Embedding Generation Pipeline ❌ CRITICAL GAP

**Current Status**: No batch embedding generation for content catalog

**What's Missing**:

1. **Offline Embedding Generation**
   - Batch process for all content in catalog
   - Incremental updates for new content
   - Store embeddings in PostgreSQL or Qdrant

2. **Feature Engineering Pipeline**
   ```rust
   pub struct ContentFeatureExtractor {
       pub fn extract_features(&self, content: &ContentMetadata) -> Result<ContentFeatures> {
           // Combine structured + unstructured features
           // - Title text
           // - Description/synopsis
           // - Genre tags
           // - Cast/crew
           // - Release year
           // - Runtime
           // - User ratings
       }
   }
   ```

3. **Embedding Storage Schema**
   ```sql
   CREATE TABLE content_embeddings (
       content_id UUID PRIMARY KEY,
       embedding VECTOR(512),  -- pgvector extension
       model_version VARCHAR(50),
       generated_at TIMESTAMP,
       INDEX USING ivfflat (embedding vector_cosine_ops)
   );
   ```

**Impact**: Without pre-computed embeddings, content-based filtering cannot work at scale.

---

## 3. Missing Training/Inference Pipelines

### 3.1 LoRA Training Orchestration ❌ CRITICAL GAP

**Current Status**: Training algorithm implemented but **no execution layer**

**What Exists**:
- `UpdateUserLoRA::execute()` implements gradient descent ✅
- Binary cross-entropy loss calculation ✅
- User layer gradient updates ✅

**What's Missing**:

1. **Training Job Queue**
   - No background job system for async training
   - No batch processing for multiple users
   - No priority queue for active users

2. **Training Triggers**
   ```rust
   // Current: Manual trigger via HTTP endpoint
   // Missing: Automatic triggers

   pub struct LoRATrainingScheduler {
       pub async fn schedule_user_training(&self, user_id: Uuid) -> Result<()> {
           // Check if user has enough new events (>10)
           // Enqueue training job
           // Return immediately (async processing)
       }

       pub async fn batch_train_active_users(&self) -> Result<()> {
           // Find users with pending updates
           // Train in parallel batches
       }
   }
   ```

3. **Training Monitoring**
   - No metrics collection for training convergence
   - No model versioning strategy
   - No A/B testing of different LoRA configurations

**Recommended Implementation**:

```rust
// File: crates/sona/src/training/scheduler.rs

use tokio::task::JoinSet;

pub struct LoRATrainingScheduler {
    pool: PgPool,
    lora_storage: Arc<LoRAStorage>,
    max_concurrent_jobs: usize,
}

impl LoRATrainingScheduler {
    pub async fn run_training_loop(&self) -> Result<()> {
        loop {
            // Find users needing training
            let pending_users = self.find_pending_users(100).await?;

            // Train in parallel batches
            let mut join_set = JoinSet::new();
            for user_id in pending_users {
                join_set.spawn(self.train_user_lora(user_id));
            }

            // Wait for batch completion
            while let Some(result) = join_set.join_next().await {
                match result {
                    Ok(Ok(())) => tracing::info!("LoRA training completed"),
                    Ok(Err(e)) => tracing::error!("Training failed: {}", e),
                    Err(e) => tracing::error!("Join error: {}", e),
                }
            }

            tokio::time::sleep(Duration::from_secs(300)).await; // Every 5 min
        }
    }

    async fn train_user_lora(&self, user_id: Uuid) -> Result<()> {
        // Load profile
        // Load recent events
        // Load or create LoRA adapter
        // Call UpdateUserLoRA::execute()
        // Save updated adapter
        // Record metrics
    }
}
```

**Files to Create**:
- `/workspaces/media-gateway/crates/sona/src/training/mod.rs` (~100 lines)
- `/workspaces/media-gateway/crates/sona/src/training/scheduler.rs` (~500 lines)
- `/workspaces/media-gateway/crates/sona/src/training/metrics.rs` (~200 lines)

---

### 3.2 Inference Optimization ⚠️ PARTIAL

**Current Status**: Basic inference works but not optimized for production

**What's Missing**:

1. **Batch Inference**
   - Currently processes one user at a time
   - Should batch multiple users for GPU efficiency
   - Should cache frequently accessed embeddings

2. **Model Quantization**
   - LoRA models stored as full precision (f32)
   - Should support f16 or int8 quantization for 2-4x memory savings

3. **Inference Caching**
   - No TTL-based recommendation caching
   - No precomputation for popular content

**Recommended TTL Strategy**:
```rust
pub struct RecommendationCache {
    cache: Arc<DashMap<(Uuid, ContextFingerprint), CachedRecommendations>>,

    pub fn get_or_compute(
        &self,
        user_id: Uuid,
        context: &RecommendationContext,
        ttl: Duration,
        compute_fn: impl FnOnce() -> Result<Vec<Recommendation>>,
    ) -> Result<Vec<Recommendation>> {
        let key = (user_id, context.fingerprint());

        if let Some(cached) = self.cache.get(&key) {
            if cached.is_fresh(ttl) {
                return Ok(cached.recommendations.clone());
            }
        }

        let recommendations = compute_fn()?;
        self.cache.insert(key, CachedRecommendations::new(recommendations.clone()));
        Ok(recommendations)
    }
}
```

---

## 4. Additional Gaps and Concerns

### 4.1 Data Integration Issues

**Simulated Data Calls** (found 15 instances):

1. `/workspaces/media-gateway/crates/sona/src/server.rs:165`
   ```rust
   let get_embedding = |content_id: Uuid| -> Result<Vec<f32>> {
       Ok(vec![0.0; 512])  // ❌ Simulated
   };
   ```

2. `/workspaces/media-gateway/crates/sona/src/recommendation.rs:130-160`
   - `get_collaborative_candidates()` - Returns empty Vec
   - `get_content_based_candidates()` - Returns empty Vec
   - `get_graph_based_candidates()` - Returns empty Vec
   - `get_context_aware_candidates()` - Returns empty Vec
   - `get_watched_content_ids()` - Returns empty Vec

3. `/workspaces/media-gateway/crates/sona/src/cold_start.rs:64-147`
   - `get_watch_count()` - Returns 0
   - `get_genre_recommendations()` - Returns dummy data
   - `get_demographic_recommendations()` - Returns dummy data
   - `get_trending_recommendations()` - Returns dummy data

**Impact**: Integration tests will pass but system is non-functional with real data.

---

### 4.2 Missing Production Features

1. **Rate Limiting**
   - No per-user rate limits on expensive operations
   - LoRA training could be triggered in tight loops

2. **Circuit Breakers**
   - No fallback when embedding service is down
   - Should degrade gracefully to popularity-based recommendations

3. **Observability**
   - No recommendation quality metrics (precision@k, recall@k)
   - No latency tracking per recommendation source
   - No A/B test integration for recommendation strategies

4. **Scalability**
   - No Redis caching layer for hot users
   - No CDN integration for static recommendations
   - No multi-region deployment strategy

---

## 5. Comparison with BATCH_001-004

### Already Covered in Previous Batches

✅ **BATCH_002**: Structured logging (observability module) - Available for SONA
✅ **BATCH_003**: HTTP endpoint wiring - Completed for SONA server
✅ **BATCH_004**: Query processing - Not applicable to SONA (Discovery service)

### Not Covered - New Work Required

❌ Graph-based recommendations (Section 1.1)
❌ ONNX Runtime integration (Section 2.1)
❌ Embedding pipeline (Section 2.2)
❌ LoRA training orchestration (Section 3.1)
❌ Inference optimization (Section 3.2)

---

## 6. Prioritized Implementation Plan

### Phase 1: Critical Path (Week 1-2)

**Priority 1: ONNX Runtime Integration**
- **Effort**: 3-4 days
- **Files**: `crates/sona/src/inference/embedding_service.rs` (~400 lines)
- **Dependencies**: Pre-trained ONNX model
- **Blockers**: Need content_encoder.onnx model file

**Priority 2: Replace Simulated Data Calls**
- **Effort**: 2-3 days
- **Files**: Update `recommendation.rs`, `cold_start.rs`, `server.rs`
- **Impact**: Makes system functional with real data

**Priority 3: Graph-Based Recommendations**
- **Effort**: 4-5 days
- **Files**: `crates/sona/src/graph_based.rs` (~300 lines)
- **Migration**: Create `content_relationships` table
- **Impact**: Restores 30% of recommendation quality

### Phase 2: Production Readiness (Week 3-4)

**Priority 4: LoRA Training Orchestration**
- **Effort**: 3-4 days
- **Files**: `crates/sona/src/training/scheduler.rs` (~500 lines)
- **Background**: Tokio task for continuous training loop

**Priority 5: Embedding Generation Pipeline**
- **Effort**: 3-4 days
- **Files**: Batch processing script + database schema
- **Storage**: PostgreSQL with pgvector extension

**Priority 6: Inference Optimization**
- **Effort**: 2-3 days
- **Features**: Batch inference, caching, quantization

### Phase 3: Advanced Features (Week 5-6)

**Priority 7: Real-Time Contextual Boosting**
- **Effort**: 2-3 days
- **Features**: Trending, seasonal events, session context

**Priority 8: Observability & Monitoring**
- **Effort**: 2 days
- **Metrics**: Precision@k, latency tracking, A/B test integration

**Priority 9: Scalability Enhancements**
- **Effort**: 3-4 days
- **Features**: Redis caching, CDN integration, multi-region

---

## 7. Technical Debt Assessment

### Code Quality: B+ (Good)

**Strengths**:
- ✅ Well-structured modules with clear separation of concerns
- ✅ Comprehensive test coverage for core algorithms
- ✅ Proper error handling with `anyhow::Result`
- ✅ Good documentation with algorithm references

**Weaknesses**:
- ⚠️ Excessive use of simulated/stub functions (15+ instances)
- ⚠️ No integration tests with real database
- ⚠️ Dependency declared but unused (`ort` crate)

### Architecture: A- (Excellent)

**Strengths**:
- ✅ Follows SPARC pseudocode specifications precisely
- ✅ Clean trait-based design for extensibility
- ✅ Efficient LoRA storage with <2ms retrieval
- ✅ Hybrid recommendation blending multiple strategies

**Weaknesses**:
- ⚠️ Missing service layer for ONNX inference
- ⚠️ No clear boundary between online/offline processing

### Production Readiness: D (Poor)

**Blockers for Production**:
1. ❌ Cannot generate real embeddings (no ONNX integration)
2. ❌ Returns empty results for most recommendation types
3. ❌ No training orchestration (LoRA models won't improve)
4. ❌ No monitoring or alerting
5. ❌ No fallback mechanisms for service degradation

---

## 8. Recommendations

### Immediate Actions (This Sprint)

1. **Integrate ONNX Runtime** - Highest priority, unblocks everything else
2. **Remove all simulated/stub implementations** - Replace with real database queries
3. **Implement graph-based recommendations** - Restores 30% of recommendation quality

### Short-Term (Next Sprint)

4. **Build LoRA training scheduler** - Enable continuous model improvement
5. **Create embedding generation pipeline** - Support content-based filtering at scale
6. **Add recommendation caching** - Reduce latency and database load

### Long-Term (Next Quarter)

7. **Implement A/B testing for recommendation strategies** - Data-driven optimization
8. **Add multi-armed bandit for cold start** - Improve new user experience
9. **Build real-time feature store** - Support contextual boosting
10. **Deploy to production with monitoring** - Gradual rollout with metrics

---

## 9. Conclusion

The SONA crate demonstrates **strong algorithmic foundations** but requires **significant ML infrastructure work** before production deployment. The core recommendation logic is well-designed and follows best practices, but the lack of ONNX integration and prevalence of simulated data calls means the system is currently a **sophisticated prototype** rather than a production-ready service.

**Estimated completion**: 4-6 weeks of focused development (1-2 engineers)

**Risk Level**: Medium-High
- Technical implementation is straightforward (well-defined tasks)
- Dependency on obtaining/training ONNX models (potential blocker)
- Integration complexity with existing services (moderate)

**Recommendation**: Proceed with Phase 1 implementation immediately. The architecture is sound and the gaps are well-defined, making this a low-risk, high-value engineering effort.

---

**Report Generated**: 2025-12-06
**Analyzer**: Code Quality Analysis System
**Next Review**: After Phase 1 completion
