# Media Gateway Discovery Service

Hybrid search engine implementing vector similarity, BM25 keyword search, and reciprocal rank fusion.

## Features

- **Natural Language Intent Parsing**: GPT-4o-mini powered query understanding
- **Hybrid Search**: Combines vector (HNSW), keyword (BM25), and graph-based search
- **Reciprocal Rank Fusion**: Merges results from multiple strategies
- **Performance**: Target p95 latency < 400ms
- **Filtering**: Genre, platform, year, and rating filters

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Discovery Service                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐   ┌──────────────┐   ┌─────────────┐ │
│  │  Intent      │   │   Vector     │   │  Keyword    │ │
│  │  Parser      │   │   Search     │   │  Search     │ │
│  │ (GPT-4o-mini)│   │   (Qdrant)   │   │  (Tantivy)  │ │
│  └──────┬───────┘   └──────┬───────┘   └──────┬──────┘ │
│         │                  │                   │        │
│         └──────────────────┼───────────────────┘        │
│                            │                            │
│                   ┌────────▼────────┐                   │
│                   │  RRF Fusion     │                   │
│                   │  (Hybrid Merge) │                   │
│                   └────────┬────────┘                   │
│                            │                            │
│                   ┌────────▼────────┐                   │
│                   │  Search Results │                   │
│                   └─────────────────┘                   │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## API Endpoints

### Health Check
```bash
GET /api/v1/health
```

### Hybrid Search
```bash
POST /api/v1/search
Content-Type: application/json

{
  "query": "movies like The Matrix",
  "filters": {
    "genres": ["action", "sci-fi"],
    "year_range": {"min": 1990, "max": 2020},
    "platforms": ["netflix", "prime_video"]
  },
  "page": 1,
  "page_size": 20
}
```

### Semantic Search (Vector-only)
```bash
POST /api/v1/search/semantic
Content-Type: application/json

{
  "query": "dark psychological thriller",
  "limit": 10
}
```

### Keyword Search (BM25-only)
```bash
POST /api/v1/search/keyword
Content-Type: application/json

{
  "query": "inception",
  "limit": 10
}
```

### Content Lookup
```bash
GET /api/v1/content/{id}
```

## Configuration

Create `config/discovery.toml`:

```toml
[server]
host = "0.0.0.0"
port = 8081
workers = 4

[search]
max_candidates = 1000
page_size = 20
max_page_size = 100
strategy_timeout_ms = 300
total_timeout_ms = 450
rrf_k = 60.0

[search.weights]
vector = 0.35
graph = 0.30
keyword = 0.20
popularity = 0.15

[vector]
qdrant_url = "http://localhost:6333"
collection_name = "media_embeddings"
dimension = 768
ef_search = 64
top_k = 50
similarity_threshold = 0.7

[keyword]
index_path = "./data/tantivy"
top_k = 50
min_score = 0.5

[database]
url = "postgresql://localhost/media_gateway"
max_connections = 10
connect_timeout_sec = 10
query_timeout_sec = 5

[cache]
redis_url = "redis://localhost:6379"
search_ttl_sec = 1800
embedding_ttl_sec = 3600
intent_ttl_sec = 600

[embedding]
model = "text-embedding-3-small"
api_url = "https://api.openai.com/v1/embeddings"
api_key = "sk-..."
timeout_ms = 5000
```

## Environment Variables

```bash
DISCOVERY_SERVER__PORT=8081
DISCOVERY_DATABASE__URL=postgresql://localhost/media_gateway
DISCOVERY_VECTOR__QDRANT_URL=http://localhost:6333
DISCOVERY_EMBEDDING__API_KEY=sk-...
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Search Latency (p50) | <150ms |
| Search Latency (p95) | <400ms |
| Search Latency (p99) | <800ms |
| Throughput | 2,000 RPS |
| Cache Hit Rate | >40% |

## Search Algorithms

### Reciprocal Rank Fusion (RRF)

Combines results from multiple search strategies:

```
score(d) = Σ(weight_s / (k + rank_s(d)))
```

Where:
- `d` = document
- `s` = search strategy
- `k` = RRF constant (60)
- `weight_s` = strategy weight (vector: 0.35, keyword: 0.20)

### Intent Parsing

Extracts structured information from queries:
- Mood keywords (dark, uplifting, intense)
- Theme keywords (heist, romance, sci-fi)
- References ("like The Matrix")
- Filters (platform, genre, year)

### Filter Strategy

Dynamically chooses pre-filtering vs post-filtering based on selectivity:
- Selectivity < 10%: Pre-filter (faster)
- Selectivity > 10%: Post-filter (more accurate)

## Running the Service

```bash
# Development
cargo run --bin discovery

# Production
cargo build --release --bin discovery
./target/release/discovery

# With custom config
DISCOVERY_SERVER__PORT=8082 cargo run --bin discovery
```

## Testing

```bash
# Run tests
cargo test

# Run specific test
cargo test test_reciprocal_rank_fusion

# With logs
RUST_LOG=debug cargo test
```

## Dependencies

- **Actix-web**: HTTP server framework
- **Qdrant**: Vector database for HNSW search
- **Tantivy**: Full-text search engine (BM25)
- **PostgreSQL**: Primary database
- **Redis**: Caching layer
- **OpenAI API**: Embedding generation

## License

MIT
