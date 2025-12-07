# SONA Personalization Engine

**Self-Optimizing Neural Architecture** for Media Gateway personalization.

## Overview

SONA implements the personalization layer specified in SPARC documents, providing:

- **User Profile Embeddings**: 512-dimensional preference vectors with temporal decay
- **Two-Tier LoRA Adaptation**: Per-user personalization with ~10KB memory footprint
- **Hybrid Recommendations**: Collaborative + Content-based + Graph-based + Context-aware filtering
- **Diversity Filtering**: Maximal Marginal Relevance (MMR) algorithm
- **Cold Start Handling**: Progressive personalization for new users

## Architecture

### Modules

- **profile.rs**: User preference vector construction (BuildUserPreferenceVector algorithm)
- **lora.rs**: Two-tier LoRA adaptation (UpdateUserLoRA, ComputeLoRAForward algorithms)
- **recommendation.rs**: Hybrid recommendation engine (GenerateRecommendations algorithm)
- **collaborative.rs**: User-based collaborative filtering
- **content_based.rs**: Content similarity filtering
- **context.rs**: Temporal and device-aware filtering
- **diversity.rs**: MMR diversity filter (ApplyDiversityFilter algorithm)
- **cold_start.rs**: New user handling (HandleColdStartUser algorithm)
- **server.rs**: Actix-web HTTP server on port 8082

## API Endpoints

### Health Check
```
GET /health
```

### Get Recommendations
```
POST /api/v1/recommendations
{
  "user_id": "uuid",
  "context": {
    "device_type": "mobile",
    "time_of_day": "evening",
    "mood": "relaxing"
  },
  "limit": 20
}
```

### Get Similar Content
```
POST /api/v1/recommendations/similar
{
  "content_id": "uuid",
  "limit": 10
}
```

### Get Personalization Score
```
POST /api/v1/personalization/score
{
  "user_id": "uuid",
  "content_id": "uuid"
}
```

### Update User Profile
```
POST /api/v1/profile/update
{
  "user_id": "uuid",
  "viewing_events": [
    {
      "content_id": "uuid",
      "timestamp": "2025-12-06T12:00:00Z",
      "completion_rate": 0.9,
      "rating": 5,
      "is_rewatch": false,
      "dismissed": false
    }
  ]
}
```

### Trigger LoRA Training
```
POST /api/v1/lora/train
{
  "user_id": "uuid",
  "force": false
}
```

## Performance Targets (from SPARC)

| Metric | Target | Implementation |
|--------|--------|----------------|
| Personalization latency (p50) | <2ms | LoRA forward pass |
| Personalization latency (p95) | <5ms | With cache |
| LoRA load time | <10ms | In-memory storage |
| Throughput | 1,500 RPS | Async Actix-web |
| Model accuracy | >80% CTR | Training loop |

## Algorithm Details

### BuildUserPreferenceVector
- **Input**: User viewing history
- **Output**: 512-dim preference vector
- **Complexity**: O(n) where n = viewing events
- **Key Features**:
  - Temporal decay (0.95^(days/30))
  - Engagement weighting (completion, rating, rewatch)
  - L2 normalization

### UpdateUserLoRA
- **Input**: Recent viewing events (minimum 10)
- **Output**: Updated LoRA adapter
- **Complexity**: O(k * r * d) where k=iterations, r=rank, d=dimensions
- **Key Features**:
  - Binary cross-entropy loss
  - 5 training iterations
  - User layer updates only (base layer frozen)

### GenerateRecommendations
- **Input**: User ID, context
- **Output**: Top-N recommendations
- **Complexity**: O(m log m) where m = candidate count
- **Weights**:
  - Collaborative: 0.35
  - Content-based: 0.25
  - Graph-based: 0.30
  - Context: 0.10

### ApplyDiversityFilter
- **Input**: Scored candidates
- **Output**: Diverse recommendations
- **Complexity**: O(n^2 * d) where n=limit, d=embedding_dim
- **MMR Formula**: score = λ * relevance - (1-λ) * max_similarity
- **Lambda**: 0.7 (balance relevance vs. diversity)

## Memory Footprint

Per user:
- Preference vector: 512 * 4 bytes = 2KB
- LoRA adapter (rank=8):
  - Base layer: 8 * 512 * 4 = 16KB (shared)
  - User layer: 768 * 8 * 4 = 24KB
  - **Total per user**: ~10KB

For 100K users: ~1GB memory

## Building

```bash
cd /workspaces/media-gateway/crates/sona
cargo build --release
```

## Running

```bash
cargo run --bin sona-server
```

Server starts on `http://0.0.0.0:8082`

## Testing

```bash
cargo test
```

## Integration with Media Gateway

SONA is called by:
- **Search Service**: User affinity boosting
- **MCP Service**: Recommendation tool
- **API Gateway**: `/v1/recommendations` routing

Dependencies:
- **media-gateway-core**: Shared types
- **PostgreSQL**: User profiles, viewing history
- **Valkey**: Caching (recommendations, profiles)

## SPARC Compliance

This implementation follows:
- **SPARC Pseudocode Part 2**: All algorithms implemented
- **SPARC Architecture Part 2**: Service definition (Section 4)
- **Performance targets**: From Architecture document

## License

Part of Media Gateway platform.
