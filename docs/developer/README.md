# Developer Guide

Technical documentation for extending and integrating the Samsung TV system.

## Table of Contents

1. [Architecture](#architecture)
2. [Module Reference](#module-reference)
3. [API Reference](#api-reference)
4. [Learning System](#learning-system)
5. [Extending](#extending)
6. [Testing](#testing)

**Additional Docs:**
- [Vercel Setup](VERCEL_SETUP.md) - Deployment guide
- [GitHub Workflows](WORKFLOWS.md) - CI/CD configuration

---

## Architecture

```
src/
├── lib/                    # Core TV functionality
│   ├── types.ts           # Zod schemas, interfaces
│   ├── tv-client.ts       # Samsung WebSocket client
│   └── discovery.ts       # SSDP network discovery
│
├── learning/              # On-device ML
│   ├── types.ts           # Learning schemas
│   ├── embeddings.ts      # WASM vector operations
│   ├── preference-learning.ts  # Q-Learning system
│   ├── persistence.ts     # Model storage
│   └── smart-tv-client.ts # Learning-enhanced client
│
├── content/               # Content discovery
│   ├── tmdb-client.ts     # TMDb API wrapper
│   └── discovery-tools.ts # MCP tools
│
├── mcp/                   # MCP server
│   ├── server.ts          # Request handler
│   └── learning-tools.ts  # Learning MCP tools
│
└── utils/                 # Utilities
    ├── config.ts          # Device storage
    └── helpers.ts         # Common helpers
```

---

## Module Reference

### TV Client (`lib/tv-client.ts`)

```typescript
import { createTVClient, SamsungTVClient } from './lib/tv-client.js';

// Create client from saved device
const client = createTVClient(device);

// Connect (returns token on first connection)
const { success, token } = await client.connect();

// Send commands
await client.sendKey('KEY_POWER');
await client.setVolume('up', 5);
await client.navigate('enter');
await client.launchApp('111299001912');
await client.launchStreamingApp('NETFLIX');
await client.goHome();

// Get state
const { state } = await client.getState();
const { apps } = await client.getApps();
```

### Discovery (`lib/discovery.ts`)

```typescript
import { discoverTVs, checkTVOnline, getTVInfo } from './lib/discovery.js';

// Find TVs on network
const devices = await discoverTVs({ timeout: 5000 });

// Check if specific TV is online
const online = await checkTVOnline('192.168.1.100');

// Get TV info from API
const info = await getTVInfo('192.168.1.100');
```

### Learning System (`learning/preference-learning.ts`)

```typescript
import { PreferenceLearningSystem } from './learning/preference-learning.js';

const learner = new PreferenceLearningSystem({
  learningRate: 0.1,
  discountFactor: 0.95,
  explorationRate: 0.3,
  minExploration: 0.05,
  memorySize: 1000,
});

// Add content
learner.addContent({
  id: 'movie-1',
  title: 'Inception',
  type: 'movie',
  genres: ['action', 'science_fiction'],
  rating: 8.8,
  duration: 148,
  actors: ['Leonardo DiCaprio'],
  directors: ['Christopher Nolan'],
  keywords: ['dreams', 'heist'],
});

// Record session
learner.recordSession({
  id: 'session-1',
  contentId: 'movie-1',
  contentMetadata: content,
  startTime: new Date().toISOString(),
  watchDuration: 148,
  completionRate: 1.0,
  userRating: 5,
  implicit: { paused: 2, rewound: 1, fastForwarded: 0, volumeChanges: 3 },
}, 'recommend_similar');

// Get recommendations
const recs = learner.getRecommendations(5);

// Export/import model
const model = learner.exportModel();
learner.importModel(model);
```

### Embeddings (`learning/embeddings.ts`)

```typescript
import {
  generateContentEmbedding,
  cosineSimilarity,
  batchSimilarity,
  ContentEmbeddingCache,
} from './learning/embeddings.js';

// Generate embedding (64-dim Float32Array)
const embedding = generateContentEmbedding(content);

// Compare similarity
const similarity = cosineSimilarity(embedding1, embedding2);

// Find top-k similar
const similar = batchSimilarity(query, vectors, 10);

// Use cache
const cache = new ContentEmbeddingCache(1000);
const emb = cache.getOrCompute(content);
```

### TMDb Client (`content/tmdb-client.ts`)

```typescript
import { createTMDbClient } from './content/tmdb-client.js';

const client = createTMDbClient(process.env.TMDB_API_KEY);

// Search
const movies = await client.searchMovies('inception');
const shows = await client.searchTVShows('breaking bad');

// Browse
const trending = await client.getTrending('all', 'week');
const popular = await client.getPopularMovies();
const topRated = await client.getTopRatedTVShows();

// Discover
const discovered = await client.discoverMovies({
  genres: [28, 878],  // action, sci-fi
  minRating: 7,
  minYear: 2020,
});

// Convert to ContentMetadata
const content = await client.movieToContentMetadata(movie, true);
```

---

## API Reference

### Types

```typescript
// Device
interface SamsungTVDevice {
  id: string;
  name: string;
  ip: string;
  port: number;  // Usually 8002
  mac?: string;  // For Wake-on-LAN
  token?: string;  // Auth token
  model?: string;
  isOnline: boolean;
}

// Content
interface ContentMetadata {
  id: string;
  title: string;
  type: 'movie' | 'tv_show' | 'documentary' | 'sports' | 'news' | 'music' | 'kids' | 'gaming';
  genres: Genre[];
  duration?: number;
  releaseYear?: number;
  rating?: number;
  popularity?: number;
  description?: string;
  posterUrl?: string;
  backdropUrl?: string;
  actors: string[];
  directors: string[];
  keywords: string[];
  appId?: string;
  appName?: string;
}

// Session
interface ViewingSession {
  id: string;
  contentId: string;
  contentMetadata: ContentMetadata;
  startTime: string;
  endTime?: string;
  watchDuration: number;
  completionRate: number;
  userRating?: number;
  implicit: {
    paused: number;
    rewound: number;
    fastForwarded: number;
    volumeChanges: number;
  };
}

// Recommendation
interface Recommendation {
  contentId: string;
  title: string;
  type: ContentType;
  genres: Genre[];
  score: number;
  reason: string;
  action: LearningAction;
  confidence: number;
  appId?: string;
}
```

### Learning Actions

```typescript
type LearningAction =
  | 'recommend_similar'
  | 'recommend_popular'
  | 'recommend_trending'
  | 'recommend_genre'
  | 'recommend_new_release'
  | 'recommend_continue_watching'
  | 'recommend_based_on_time'
  | 'explore_new_genre'
  | 'explore_new_type';
```

### Genres

```typescript
type Genre =
  | 'action' | 'adventure' | 'animation' | 'comedy' | 'crime'
  | 'documentary' | 'drama' | 'family' | 'fantasy' | 'history'
  | 'horror' | 'music' | 'mystery' | 'romance' | 'science_fiction'
  | 'thriller' | 'war' | 'western' | 'reality' | 'sports' | 'news';
```

---

## Learning System

### Q-Learning Algorithm

The system uses Q-Learning with epsilon-greedy exploration:

```
Q(s,a) = Q(s,a) + α * (r + γ * max Q(s',a') - Q(s,a))

Where:
- α = learning rate (0.1)
- γ = discount factor (0.95)
- r = reward from viewing session
- s = state (time, recent genres, completion rate)
- a = action (recommendation strategy)
```

### State Representation

States are encoded as:
- Time of day (morning/afternoon/evening/night)
- Day of week (weekday/weekend)
- Recent genres watched (top 3)
- Recent content types (top 2)
- Average completion rate

### Reward Calculation

```typescript
reward = completionRate * 0.5
       + (userRating / 5) * 0.3
       + (watchDuration / expectedDuration) * 0.1
       + engagementSignals * 0.1
```

### Embedding Generation

64-dimension content embeddings:
- Genres: 10 dims (weighted average of genre vectors)
- Content type: 8 dims (one-hot)
- Popularity: 1 dim (normalized)
- Rating: 1 dim (normalized)
- Recency: 1 dim (year decay)
- Duration: 5 dims (bucket encoding)
- Keywords: 38 dims (hash features)

---

## Extending

### Adding MCP Tools

1. Define tool in appropriate file:

```typescript
// src/mcp/my-tools.ts
export const MY_TOOLS = [{
  name: 'my_custom_tool',
  description: 'Does something useful',
  inputSchema: {
    type: 'object',
    properties: {
      param1: { type: 'string', description: 'First param' },
    },
    required: ['param1'],
  },
}];

export async function handleMyToolCall(
  toolName: string,
  args: Record<string, unknown>
): Promise<MCPToolResult> {
  // Implementation
}
```

2. Register in server.ts:

```typescript
import { MY_TOOLS, handleMyToolCall } from './my-tools.js';

export const MCP_TOOLS = [...TV_TOOLS, ...LEARNING_TOOLS, ...DISCOVERY_TOOLS, ...MY_TOOLS];

// In handleToolCall:
if (toolName.startsWith('my_')) {
  return handleMyToolCall(toolName, args);
}
```

### Adding Content Sources

Implement the conversion to `ContentMetadata`:

```typescript
async function mySourceToContentMetadata(item: MySourceItem): Promise<ContentMetadata> {
  return {
    id: `mysource-${item.id}`,
    title: item.name,
    type: mapType(item.category),
    genres: mapGenres(item.genres),
    // ... other fields
  };
}
```

---

## Testing

### Run Tests

```bash
npm test                 # Run all tests
npm run test:watch      # Watch mode
npm run test:coverage   # Coverage report
```

### Test Structure

```
tests/
├── helpers.test.ts      # Utility tests
├── types.test.ts        # Schema validation
├── learning.test.ts     # Q-Learning tests
└── content-discovery.test.ts  # TMDb tests
```

### Writing Tests

```typescript
import { describe, it, expect, beforeEach, vi } from 'vitest';

describe('MyFeature', () => {
  beforeEach(() => {
    // Setup
  });

  it('should do something', () => {
    expect(result).toBe(expected);
  });

  it('should handle errors', async () => {
    await expect(fn()).rejects.toThrow('error');
  });
});
```

### Mocking

```typescript
// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

mockFetch.mockResolvedValueOnce({
  ok: true,
  json: () => Promise.resolve({ data: 'test' }),
});
```

---

## Build & Deploy

```bash
# Development
npm run build           # Compile TypeScript
npm run build:watch    # Watch mode

# Production
npm run build
npm start              # Start MCP server
```

### MCP Server Modes

```bash
# STDIO (default, for Claude Desktop)
npm start

# SSE (for web clients)
npm start -- --transport sse --port 3000
```
