# Nexus-UMMID Firestore Database Schema

## Overview

This directory contains the Firestore database schema for the Nexus-UMMID Entertainment Discovery Platform, designed to scale to **400M+ users** with hypergraph query capabilities.

## Files

- **`firestore.ts`** - Firestore client initialization, connection pooling, batch operations
- **`models.ts`** - TypeScript type definitions for all Firestore documents
- **`schema.ts`** - Collection definitions, indexes, and query helpers
- **`firestore.indexes.json`** - Firestore composite index definitions (deploy to Firebase)

## Collections

### Core Collections

| Collection | Purpose | Scale Considerations |
|------------|---------|---------------------|
| `content` | Media metadata (movies, series, episodes) | Partitioned by platform for horizontal scaling |
| `users` | User profiles and preferences | Sharded by user_id hash |
| `user_preferences` | Simplified user data for quick lookups | Denormalized for read performance |
| `recommendations` | Content recommendations | TTL-based cleanup for expired recommendations |
| `recommendation_sets` | Pre-computed recommendation batches | Cached sets regenerated nightly |
| `platforms` | Streaming platform metadata | Small collection, fully cached |

### Advanced Collections

| Collection | Purpose | Scale Considerations |
|------------|---------|---------------------|
| `hyperedges` | N-dimensional relationships (rights, territories) | Indexed by edge_type and validity period |
| `content_metrics` | Performance analytics | Updated via Pub/Sub events |
| `eidr_cache` | EIDR resolution cache | TTL-based expiration |
| `api_requests` | Rate limiting and monitoring | Time-series data, archived to BigQuery |

## Key Features

### 🎯 Hypergraph Support

The schema supports **hypergraph relationships** where a single edge connects multiple nodes:

```typescript
// Example: Distribution rights hyperedge
{
  edge_type: 'distribution_right',
  nodes: [
    { collection: 'content', document_id: 'inception', role: 'asset' },
    { collection: 'platforms', document_id: 'netflix', role: 'platform' },
    { collection: 'territories', document_id: 'france', role: 'territory' }
  ],
  valid_from: '2025-01-01',
  valid_to: '2025-06-30',
  properties: {
    license_type: 'exclusive',
    quality: ['UHD', '4K', 'HDR10']
  }
}
```

### ⏱️ Bitemporal Modeling

Support for time-travel queries with two time dimensions:
- **Valid Time**: When the data was valid in the real world
- **Transaction Time**: When the data was recorded in the database

```typescript
interface MediaMetadata {
  valid_from: Timestamp;  // When this version became valid
  valid_to?: Timestamp;   // When this version expired
  version: number;        // Version number for ordering
  created_at: Timestamp;  // Transaction time
}
```

### 🔍 Semantic Search Integration

Vector embeddings are stored in **Cloud SQL with pgvector**, referenced by `embedding_id`:

```typescript
interface MediaMetadata {
  embedding_id?: string;  // Reference to vector in Cloud SQL
  keywords?: string[];    // For hybrid search
}
```

### 📊 Performance Optimizations

1. **Denormalization**: User preferences duplicated for fast access
2. **Pre-computed Sets**: Recommendation batches generated nightly
3. **Composite Indexes**: Optimized for common query patterns
4. **Batch Operations**: Built-in helpers for bulk writes (500 ops/batch)
5. **Connection Pooling**: Singleton pattern prevents pool exhaustion

## Usage

### Initialize Firestore Client

```typescript
import { getFirestore } from './db/firestore';

const db = getFirestore();
```

### Query Content by Genre

```typescript
import { queryContentByGenre } from './db/schema';

const thrillerMovies = await queryContentByGenre('thriller', 20);
```

### Batch Insert Content

```typescript
import { batchUpsertContent } from './db/schema';

await batchUpsertContent([
  {
    id: 'inception',
    title: 'Inception',
    genres: ['sci-fi', 'thriller'],
    release_year: 2010,
    // ... other fields
  },
  // ... more content
]);
```

### Query Hypergraph Edges

```typescript
import { queryHyperedgesByType } from './db/schema';

const rights = await queryHyperedgesByType('distribution_right');
```

## Indexes

### Deploy Indexes to Firebase

```bash
# From project root
firebase deploy --only firestore:indexes --project agentics-foundation25lon-1899
```

### Required Composite Indexes

The schema requires 10+ composite indexes for optimal performance. See `firestore.indexes.json` for full definitions.

**Critical indexes:**
- `content_by_genre_and_year` - Genre browsing with popularity
- `content_by_platform_and_rating` - Platform-specific queries
- `recommendations_by_user_and_score` - Personalized recommendations
- `hyperedge_by_type_and_validity` - Rights collision detection

## Scalability

### Horizontal Scaling Strategies

1. **Collection Sharding**: Partition large collections by hash key
   ```typescript
   // Example: Shard users by ID
   const shard = hash(userId) % 10;
   const collection = `users_shard_${shard}`;
   ```

2. **Time-based Partitioning**: Archive old data to BigQuery
   ```typescript
   // Example: Archive API requests older than 30 days
   const cutoff = new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);
   ```

3. **Denormalization**: Duplicate frequently accessed data
   ```typescript
   // UserPreference contains denormalized watch history (last 100 items)
   // Full history stored in separate collection or BigQuery
   ```

### Query Optimization

1. **Use Composite Indexes**: Always query indexed fields together
2. **Limit Result Sets**: Use pagination, avoid scanning large datasets
3. **Cache Frequently Accessed Data**: Use Memorystore for hot data
4. **Async Batch Operations**: Process in background with Pub/Sub

## Environment Variables

```bash
# Required
GCP_PROJECT_ID=agentics-foundation25lon-1899
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json

# Optional
FIRESTORE_EMULATOR_HOST=localhost:8080  # For local development
```

## Testing

```typescript
// Use Firestore emulator for testing
process.env.FIRESTORE_EMULATOR_HOST = 'localhost:8080';

import { getFirestore } from './db/firestore';
const db = getFirestore();

// Run tests...
```

## Monitoring

### Health Check

```typescript
import { checkFirestoreHealth } from './db/firestore';

const isHealthy = await checkFirestoreHealth();
```

### Performance Metrics

Monitor these metrics in Cloud Monitoring:
- `firestore.googleapis.com/document/read_count`
- `firestore.googleapis.com/document/write_count`
- `firestore.googleapis.com/api/request_latencies`

## Migration Guide

### Initial Setup

```typescript
import { initializeDatabase } from './db/schema';

await initializeDatabase();
```

### Schema Validation

```typescript
import { validateDatabaseSchema } from './db/schema';

const { valid, errors } = await validateDatabaseSchema();
if (!valid) {
  console.error('Schema validation failed:', errors);
}
```

## References

- [Firestore Best Practices](https://firebase.google.com/docs/firestore/best-practices)
- [Composite Indexes](https://firebase.google.com/docs/firestore/query-data/indexing)
- [Scaling Firestore](https://firebase.google.com/docs/firestore/solutions/scale)
- [Bitemporal Modeling](https://en.wikipedia.org/wiki/Temporal_database)

## License

Part of Nexus-UMMID platform (c) 2025
