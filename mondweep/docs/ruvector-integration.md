# RuVector Engine Integration Guide

## Overview

This document describes the integration of the **RuVector Engine** (GPU-less Serverless Media Recommendation Engine) into the UMMID (Unified Media Metadata Integration & Distribution) platform for the Agentics Foundation TV5 Hackathon.

## Why RuVector for UMMID?

The RuVector Engine provides critical capabilities that align perfectly with UMMID's metadata optimization goals:

### 1. **Semantic Metadata Search**
- **Hypergraph-based relationships** model complex media connections (cast, genre, director, themes)
- **Vector similarity search** enables natural language queries for content discovery
- Solves the "30-minute decision problem" through intelligent recommendations

### 2. **Metadata Enrichment**
- **CPU-optimized embeddings** (FastRP, Node2Vec) generate semantic representations of media
- **AI-driven tagging** through vector similarity without GPU requirements
- **Contextual relationships** between metadata fields (e.g., "films with similar mood and cast")

### 3. **Content Discovery & Recommendations**
- **Multi-seed recommendations** combine user preferences with metadata attributes
- **Trending analysis** based on interaction patterns
- **Similar content discovery** for cross-platform metadata mapping

### 4. **Scalability for UMMID**
- **CloudRun-native** architecture matches UMMID's serverless design
- **Stateless operation** enables horizontal scaling
- **Distributed learning** supports the PRD's 500M concurrent stream target

## Architecture Integration

```
┌─────────────────────────────────────────────────────────────────┐
│                    UMMID Platform                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Metadata    │  │  Validation  │  │ Distribution │          │
│  │  Ingestion   │  │   Engine     │  │   Engine     │          │
│  └──────┬───────┘  └──────────────┘  └──────────────┘          │
│         │                                                        │
│         ▼                                                        │
│  ┌─────────────────────────────────────────────────┐           │
│  │         RuVector Integration Layer              │           │
│  │  ┌──────────────┐  ┌──────────────┐            │           │
│  │  │  Embedding   │  │  Similarity  │            │           │
│  │  │  Generator   │  │    Search    │            │           │
│  │  └──────────────┘  └──────────────┘            │           │
│  └─────────────────────────────────────────────────┘           │
│         │                                                        │
│         ▼                                                        │
├─────────────────────────────────────────────────────────────────┤
│                    RuVector Engine                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Hypergraph  │  │  Embeddings  │  │   Vector     │          │
│  │    Model     │  │  (FastRP)    │  │    Store     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Setup

### Repository Structure

The RuVector Engine is integrated as a **git submodule** from the `vibe-cast` repository:

```
hackathon-tv5/
├── mondweep/
│   ├── vibe-cast/                    # Git submodule
│   │   └── ruvector-engine/          # Full RuVector implementation
│   ├── ruvector-engine -> vibe-cast/ruvector-engine  # Symlink for convenience
│   └── docs/
│       ├── ruvector-integration.md   # This file
│       └── setup-ruvector-reference.md
```

### Installation

1. **Clone with submodules** (for new clones):
   ```bash
   git clone --recursive https://github.com/mondweep/hackathon-tv5.git
   ```

2. **Initialize submodules** (if already cloned):
   ```bash
   git submodule update --init --recursive
   ```

3. **Install RuVector dependencies**:
   ```bash
   cd mondweep/ruvector-engine
   npm install
   ```

4. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your GCP project settings
   ```

### Updating RuVector

To pull the latest changes from the vibe-cast repository:

```bash
cd mondweep/vibe-cast
git pull origin claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn
cd ../..
git add mondweep/vibe-cast
git commit -m "Update RuVector submodule"
```

## Use Cases for UMMID

### 1. Semantic Metadata Search

**Problem**: Finding content based on natural language descriptions  
**Solution**: Use RuVector's embedding-based similarity search

```javascript
// Example: Find movies similar to a description
const query = "dark psychological thriller with twist ending";
const results = await ruvectorEngine.semanticSearch(query, {
  filters: { type: 'movie', rating: 'R' },
  limit: 10
});
```

**UMMID Integration Point**: Enrichment Engine (FR-ENR-02)

### 2. Metadata Gap Analysis

**Problem**: Identifying missing or incomplete metadata fields  
**Solution**: Use RuVector to find similar content with complete metadata

```javascript
// Example: Find similar content to fill metadata gaps
const incompleteItem = {
  title: "New Indie Film",
  genres: ["Drama"],
  // Missing: cast, keywords, mood tags
};

const similar = await ruvectorEngine.findSimilar(incompleteItem.id, {
  limit: 5,
  includeMetadata: true
});

// Extract common metadata from similar items
const suggestedMetadata = extractCommonFields(similar);
```

**UMMID Integration Point**: Enrichment Workflows (FR-ENR-01, FR-ENR-02)

### 3. Content Relationship Mapping

**Problem**: Understanding complex relationships between content  
**Solution**: Leverage RuVector's hypergraph model

```javascript
// Example: Map relationships for a series
const relationships = await ruvectorEngine.getHypergraph({
  nodeId: 'series-breaking-bad',
  depth: 2,
  relationshipTypes: ['cast', 'genre', 'similar_themes']
});
```

**UMMID Integration Point**: Golden Record Data Model (FR-DM-01)

### 4. Platform-Specific Recommendations

**Problem**: Different platforms require different content recommendations  
**Solution**: Multi-seed recommendations with platform constraints

```javascript
// Example: Generate recommendations for a FAST platform
const recommendations = await ruvectorEngine.multiSeedRecommendations({
  seeds: {
    genres: ['Action', 'Thriller'],
    mood: ['Intense', 'Fast-paced'],
    targetAudience: 'TV-14'
  },
  filters: {
    hasAdBreakMarkers: true,
    duration: { min: 1800, max: 5400 } // 30-90 minutes
  },
  limit: 20
});
```

**UMMID Integration Point**: Distribution Engine (FR-DST-02)

### 5. Trending Content Analysis

**Problem**: Understanding what content is gaining traction  
**Solution**: RuVector's interaction-based trending analysis

```javascript
// Example: Get trending content for metadata prioritization
const trending = await ruvectorEngine.getTrending({
  timeWindow: '7d',
  category: 'streaming',
  minInteractions: 100
});

// Prioritize metadata updates for trending content
await ummid.prioritizeMetadataEnrichment(trending.map(t => t.id));
```

**UMMID Integration Point**: Analytics (FR-ANA-01)

## API Integration Examples

### Initialize RuVector with UMMID Catalog

```javascript
const RuVectorEngine = require('./mondweep/ruvector-engine/src/services/recommendation-engine');

// Initialize with UMMID metadata
const engine = new RuVectorEngine({
  embeddingDimensions: 128,
  vectorStoreConfig: {
    metric: 'cosine',
    indexType: 'ivf'
  }
});

// Load catalog from UMMID
const catalog = await ummid.getCatalog();
for (const item of catalog) {
  await engine.addMedia({
    id: item.eidr,
    title: item.title,
    genres: item.genres,
    actors: item.cast,
    directors: item.directors,
    year: item.releaseYear,
    metadata: {
      synopsis: item.synopsis,
      mood: item.aiGeneratedMood,
      themes: item.themes
    }
  });
}

// Generate embeddings
await engine.generateEmbeddings();
```

### Real-time Metadata Enrichment

```javascript
// When new content is ingested into UMMID
ummid.on('content:ingested', async (content) => {
  // Add to RuVector
  const mediaId = await ruvectorEngine.addMedia({
    title: content.title,
    genres: content.genres,
    // ... other metadata
  });

  // Find similar content
  const similar = await ruvectorEngine.findSimilar(mediaId, { limit: 10 });

  // Extract enrichment suggestions
  const enrichment = {
    suggestedKeywords: extractKeywords(similar),
    suggestedMood: extractMood(similar),
    similarTitles: similar.map(s => s.title)
  };

  // Update UMMID Golden Record
  await ummid.enrichMetadata(content.id, enrichment);
});
```

### Platform-Specific Validation

```javascript
// Validate metadata completeness before distribution
async function validateForPlatform(contentId, platform) {
  const content = await ummid.getContent(contentId);
  const platformRequirements = ummid.getPlatformRequirements(platform);

  // Use RuVector to find similar content successfully delivered to platform
  const successfulDeliveries = await ruvectorEngine.findSimilar(contentId, {
    filters: {
      deliveredTo: platform,
      deliveryStatus: 'success'
    },
    limit: 5
  });

  // Compare metadata completeness
  const missingFields = compareMetadata(content, successfulDeliveries, platformRequirements);

  return {
    isValid: missingFields.length === 0,
    missingFields,
    suggestions: generateSuggestions(successfulDeliveries)
  };
}
```

## Deployment

### Local Development

```bash
# Terminal 1: Run RuVector Engine
cd mondweep/ruvector-engine
npm start

# Terminal 2: Run UMMID Platform
cd apps/ummid-platform
npm run dev
```

### Google Cloud Run Deployment

```bash
# Deploy RuVector as a microservice
cd mondweep/ruvector-engine
gcloud run deploy ruvector-engine \
  --source . \
  --region us-central1 \
  --platform managed \
  --allow-unauthenticated \
  --memory 2Gi \
  --cpu 2 \
  --max-instances 100

# Get the service URL
export RUVECTOR_URL=$(gcloud run services describe ruvector-engine \
  --region us-central1 \
  --format 'value(status.url)')

# Configure UMMID to use RuVector
echo "RUVECTOR_ENGINE_URL=$RUVECTOR_URL" >> apps/ummid-platform/.env
```

### Docker Compose (Full Stack)

```yaml
# docker-compose.yml
version: '3.8'
services:
  ruvector:
    build: ./mondweep/ruvector-engine
    ports:
      - "8080:8080"
    environment:
      - GCP_PROJECT_ID=${GCP_PROJECT_ID}
      - VECTOR_DIMENSIONS=128
    
  ummid-platform:
    build: ./apps/ummid-platform
    ports:
      - "3000:3000"
    environment:
      - RUVECTOR_ENGINE_URL=http://ruvector:8080
    depends_on:
      - ruvector
```

## Performance Considerations

### Embedding Generation

- **Initial catalog load**: ~1-2 seconds per 1000 items (CPU-optimized FastRP)
- **Incremental updates**: ~10-50ms per item
- **Batch processing**: Recommended for bulk imports (>10,000 items)

### Vector Search

- **Query latency**: <100ms for catalogs up to 100K items
- **IVF indexing**: Reduces search time by ~10x for large catalogs
- **Filtering**: Query-time filters maintain stateless operation

### Scaling Strategy

1. **Small catalogs (<10K items)**: Single CloudRun instance
2. **Medium catalogs (10K-100K)**: Auto-scaling with 2-10 instances
3. **Large catalogs (>100K)**: Distributed learning + pgvector storage

## Monitoring & Observability

### Key Metrics

```javascript
// Get RuVector statistics
const stats = await ruvectorEngine.getStats();
console.log({
  totalNodes: stats.hypergraph.nodeCount,
  totalEdges: stats.hypergraph.edgeCount,
  embeddingDimensions: stats.embeddings.dimensions,
  vectorStoreSize: stats.vectorStore.size,
  avgQueryTime: stats.performance.avgQueryTimeMs
});
```

### Integration with UMMID Analytics

- Track metadata enrichment success rates
- Monitor recommendation quality (CTR, engagement)
- Measure time-to-market improvements from automated enrichment

## Troubleshooting

### Common Issues

1. **Submodule not initialized**
   ```bash
   git submodule update --init --recursive
   ```

2. **Symlink broken**
   ```bash
   cd mondweep
   ln -sf vibe-cast/ruvector-engine ruvector-engine
   ```

3. **Port conflicts**
   ```bash
   # Change RuVector port
   PORT=8081 npm start
   ```

4. **Memory issues with large catalogs**
   ```bash
   # Increase Node.js memory
   NODE_OPTIONS="--max-old-space-size=4096" npm start
   ```

## Future Enhancements

### Phase 1 (Current Hackathon)
- ✅ Basic integration with UMMID metadata ingestion
- ✅ Semantic search for content discovery
- ✅ Metadata gap analysis

### Phase 2 (Post-Hackathon)
- [ ] Real-time embedding updates via Pub/Sub
- [ ] Multi-language embedding support
- [ ] Integration with Gracenote/TiVo enrichment

### Phase 3 (Production)
- [ ] Federated learning across multiple UMMID instances
- [ ] A/B testing framework for recommendation quality
- [ ] Advanced hypergraph analytics for content strategy

## References

- **RuVector Engine**: [GitHub - vibe-cast/ruvector-engine](https://github.com/mondweep/vibe-cast/tree/claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn/ruvector-engine)
- **UMMID PRD**: [Metadata Optimization Platform PRD](./Metadata%20Optimization%20Platform%20PRD.md)
- **Agentics Hackathon**: [agentics.org/hackathon](https://agentics.org/hackathon)

## License

Both UMMID and RuVector Engine are licensed under Apache-2.0.

---

**Last Updated**: 2025-12-05  
**Integration Status**: ✅ Active Development  
**Maintainer**: @mondweep
