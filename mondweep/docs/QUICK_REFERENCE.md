# RuVector Quick Reference

## 🚀 Quick Commands

### Setup
```bash
# Initialize submodule
git submodule update --init --recursive

# Install dependencies
cd mondweep/ruvector-engine && npm install

# Configure environment
cp .env.example .env
```

### Run
```bash
# Start server (port 8080)
npm start

# Run demo
npm run demo

# Run tests
npm test
```

### Deploy
```bash
# Deploy to Cloud Run
npm run gcloud:deploy

# Build Docker image
npm run docker:build

# Run in Docker
npm run docker:run
```

## 📡 API Quick Reference

### Health Check
```bash
curl http://localhost:8080/api/v1/health
```

### Initialize Engine
```bash
curl -X POST http://localhost:8080/api/v1/initialize \
  -H "Content-Type: application/json" \
  -d '{"config": {"embeddingDimensions": 128}}'
```

### Add Media
```bash
curl -X POST http://localhost:8080/api/v1/media \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Inception",
    "genres": ["Sci-Fi", "Thriller"],
    "actors": ["Leonardo DiCaprio"],
    "directors": ["Christopher Nolan"],
    "year": 2010
  }'
```

### Get Recommendations
```bash
# User-based
curl http://localhost:8080/api/v1/recommendations/user-alice?limit=10

# Similar items
curl http://localhost:8080/api/v1/similar/movie-inception?limit=10

# Trending
curl http://localhost:8080/api/v1/trending?limit=20
```

### Record Interaction
```bash
curl -X POST http://localhost:8080/api/v1/interactions \
  -H "Content-Type: application/json" \
  -d '{
    "userId": "user-123",
    "mediaId": "movie-inception",
    "interactionType": "watch",
    "rating": 5
  }'
```

### Get Statistics
```bash
curl http://localhost:8080/api/v1/stats
```

## 🔧 Integration Patterns

### Initialize with UMMID Catalog
```javascript
const RuVectorEngine = require('./mondweep/ruvector-engine/src/services/recommendation-engine');

const engine = new RuVectorEngine({
  embeddingDimensions: 128,
  vectorStoreConfig: { metric: 'cosine', indexType: 'ivf' }
});

// Load catalog
const catalog = await ummid.getCatalog();
for (const item of catalog) {
  await engine.addMedia({
    id: item.eidr,
    title: item.title,
    genres: item.genres,
    actors: item.cast,
    directors: item.directors,
    year: item.releaseYear
  });
}

await engine.generateEmbeddings();
```

### Semantic Search
```javascript
const results = await engine.semanticSearch(
  "dark psychological thriller with twist ending",
  { 
    filters: { type: 'movie', rating: 'R' },
    limit: 10 
  }
);
```

### Metadata Enrichment
```javascript
const similar = await engine.findSimilar(itemId, { limit: 5 });
const enrichment = {
  suggestedKeywords: extractKeywords(similar),
  suggestedMood: extractMood(similar),
  similarTitles: similar.map(s => s.title)
};
await ummid.enrichMetadata(itemId, enrichment);
```

### Platform Validation
```javascript
const successfulDeliveries = await engine.findSimilar(contentId, {
  filters: {
    deliveredTo: 'netflix',
    deliveryStatus: 'success'
  },
  limit: 5
});

const missingFields = compareMetadata(
  content, 
  successfulDeliveries, 
  platformRequirements
);
```

## 🎯 Common Use Cases

| Use Case | Endpoint | Method |
|----------|----------|--------|
| Content Discovery | `/api/v1/recommendations/:userId` | GET |
| Similar Content | `/api/v1/similar/:itemId` | GET |
| Trending Analysis | `/api/v1/trending` | GET |
| Multi-Seed Recs | `/api/v1/recommendations/multi-seed` | POST |
| Add Content | `/api/v1/media` | POST |
| Track Interaction | `/api/v1/interactions` | POST |
| Fine-Tune Model | `/api/v1/fine-tune` | POST |

## 🔍 Troubleshooting

### Submodule Issues
```bash
# Submodule not initialized
git submodule update --init --recursive

# Submodule out of sync
cd mondweep/vibe-cast
git pull origin claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn
cd ../..
git add mondweep/vibe-cast
git commit -m "Update RuVector submodule"
```

### Symlink Issues
```bash
# Recreate symlink
cd mondweep
rm ruvector-engine
ln -s vibe-cast/ruvector-engine ruvector-engine
```

### Port Conflicts
```bash
# Use different port
PORT=8081 npm start
```

### Memory Issues
```bash
# Increase Node.js memory
NODE_OPTIONS="--max-old-space-size=4096" npm start
```

## 📊 Performance Benchmarks

| Operation | Time (avg) | Notes |
|-----------|------------|-------|
| Add Media | 10-50ms | Single item |
| Generate Embeddings | 1-2s / 1K items | CPU-optimized FastRP |
| Vector Search | <100ms | Up to 100K items |
| Recommendations | 50-150ms | Includes filtering |

## 🌐 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Server port | 8080 |
| `GCP_PROJECT_ID` | Google Cloud project | - |
| `GCP_REGION` | Deployment region | us-central1 |
| `VECTOR_DIMENSIONS` | Embedding size | 128 |
| `MAX_INSTANCES` | CloudRun max scale | 100 |

## 📚 Documentation Links

- **[Integration Guide](./ruvector-integration.md)** - Comprehensive integration documentation
- **[Setup Reference](./setup-ruvector-reference.md)** - Multiple setup approaches
- **[UMMID PRD](./Metadata%20Optimization%20Platform%20PRD.md)** - Product requirements
- **[RuVector README](../ruvector-engine/README.md)** - Engine documentation

## 🔗 Quick Links

- **Source**: [vibe-cast/ruvector-engine](https://github.com/mondweep/vibe-cast/tree/claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn/ruvector-engine)
- **Hackathon**: [agentics.org/hackathon](https://agentics.org/hackathon)
- **GCP Console**: [console.cloud.google.com](https://console.cloud.google.com)

---

**Last Updated**: 2025-12-05  
**Version**: 1.0.0  
**Status**: ✅ Active Development
