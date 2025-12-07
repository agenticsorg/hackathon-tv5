# TVDB Recommender - Hackathon Alignment

## The Problem We're Solving

> **"Every night, millions spend up to 45 minutes deciding what to watch"**

The TVDB Self-Learning Recommendation System directly addresses the **Entertainment Discovery** track by:

1. **Learning user preferences** - No more generic recommendations
2. **Cross-platform discovery** - Find content regardless of where it streams
3. **Natural language search** - "Show me something like Breaking Bad but funnier"
4. **Instant recommendations** - Sub-millisecond responses, not 45 minutes of browsing

---

## Alignment with Hackathon Tracks

### Primary: Entertainment Discovery Track
| Requirement | Our Solution |
|-------------|--------------|
| Solve the 45-minute problem | Self-learning patterns reduce decision time to seconds |
| Cross-platform discovery | TVDB has streaming availability data for all platforms |
| Personalization | Vector embeddings + ReasoningBank for true personalization |
| Natural language | Semantic search understands "sci-fi like Stranger Things" |

### Secondary: Multi-Agent Systems
| Capability | Implementation |
|------------|----------------|
| Agent coordination | Agentic-flow workflow orchestration |
| Memory persistence | AgentDB ReasoningBank + PostgreSQL |
| Learning agents | Pattern learning with 80%+ success rates |

### Tertiary: Agentic Workflows
| Feature | Technology |
|---------|------------|
| Autonomous workflows | RecommendationWorkflow class |
| Self-improvement | ReflexionMemory for self-critique |
| Task orchestration | Multi-step recommendation pipeline |

---

## Key Improvements Needed

### 1. ARW Integration (Agent-Ready Web)
Add ARW manifest for agent discovery:

```json
{
  "version": "0.1",
  "profile": "ARW-1",
  "site": {
    "name": "TVDB Smart Recommendations",
    "description": "AI-powered TV and movie discovery that learns your preferences"
  },
  "actions": [
    {
      "id": "semantic_search",
      "endpoint": "/api/search",
      "method": "POST",
      "description": "Natural language content search"
    },
    {
      "id": "get_recommendations",
      "endpoint": "/api/recommendations",
      "method": "POST",
      "description": "Personalized recommendations based on user profile"
    },
    {
      "id": "find_similar",
      "endpoint": "/api/similar",
      "method": "GET",
      "description": "Find content similar to a given title"
    },
    {
      "id": "check_availability",
      "endpoint": "/api/availability",
      "method": "GET",
      "description": "Check streaming platform availability"
    }
  ],
  "machineViews": [
    {
      "pattern": "/content/*",
      "endpoint": "/api/machine-view/content"
    }
  ]
}
```

### 2. Streaming Platform Availability
TVDB provides streaming availability data. Add:

```typescript
interface StreamingAvailability {
  contentId: string;
  platforms: {
    name: string;        // Netflix, Hulu, Disney+, etc.
    country: string;
    type: 'subscription' | 'rent' | 'buy';
    price?: number;
    deepLink: string;    // Direct link to watch
  }[];
}
```

### 3. Natural Language Query Processing
Enhance search to understand:
- "Something fun for date night" → Comedy + Romance, 90+ rating
- "Show my kids can watch" → Family-friendly, Animation
- "Like Breaking Bad but shorter" → Drama + Crime, limited series
- "What's trending this week" → Recent, high popularity

### 4. MCP Server Integration
Add MCP tools for Claude/Gemini integration:

```typescript
const mcpTools = [
  {
    name: 'discover_content',
    description: 'Find TV shows or movies based on natural language query',
    inputSchema: {
      type: 'object',
      properties: {
        query: { type: 'string', description: 'Natural language search query' },
        userId: { type: 'string', description: 'User ID for personalization' }
      }
    }
  },
  {
    name: 'get_personalized_recommendations',
    description: 'Get AI-curated recommendations based on watch history',
    inputSchema: {
      type: 'object',
      properties: {
        userId: { type: 'string' },
        mood: { type: 'string', enum: ['relaxing', 'exciting', 'thoughtful', 'funny'] },
        timeAvailable: { type: 'number', description: 'Minutes available to watch' }
      }
    }
  },
  {
    name: 'find_where_to_watch',
    description: 'Find which streaming platforms have a title',
    inputSchema: {
      type: 'object',
      properties: {
        title: { type: 'string' },
        country: { type: 'string', default: 'US' }
      }
    }
  }
];
```

### 5. Integration with Media Discovery App
Connect to the existing `apps/media-discovery` app:

```typescript
// In apps/media-discovery/src/app/api/recommendations/route.ts
import { TVDBRecommender } from '@/lib/tvdb-recommender';

export async function POST(request: Request) {
  const { userId, query, mood } = await request.json();

  const recommender = new TVDBRecommender();
  await recommender.initialize();

  const recommendations = await recommender.getRecommendations({
    userId,
    context: {
      query,
      mood,
      timeOfDay: getCurrentTimeOfDay()
    }
  });

  return Response.json({
    recommendations: recommendations.recommendations.map(r => ({
      id: r.contentId,
      title: r.metadata.title,
      year: r.metadata.year,
      genres: r.metadata.genres,
      rating: r.metadata.rating,
      reason: r.reason.explanation,
      streamingOn: r.streamingPlatforms // Where to watch
    })),
    pattern: recommendations.patternUsed,
    generatedIn: `${Date.now() - startTime}ms`
  });
}
```

---

## Success Metrics

| Metric | Current | Target | Impact |
|--------|---------|--------|--------|
| Decision time | 45 min | <1 min | 97% reduction |
| Recommendation latency | 0.95ms | <100ms | ✅ Exceeds |
| Personalization accuracy | 80% | 85% | Learning improves |
| Cross-platform coverage | 1 (TVDB) | 5+ | Multi-source |

---

## Implementation Priority

1. **High Priority**
   - [ ] Add streaming availability from TVDB
   - [ ] Create MCP tools for agent integration
   - [ ] Add ARW manifest

2. **Medium Priority**
   - [ ] Natural language query processing
   - [ ] Integration with media-discovery app
   - [ ] User preference onboarding flow

3. **Nice to Have**
   - [ ] Watch party coordination
   - [ ] Social recommendations ("friends are watching")
   - [ ] Mood-based discovery

---

## Hackathon Demo Flow

1. **User says**: "I have 2 hours and want something exciting"
2. **Agent discovers** ARW manifest, finds `get_recommendations` action
3. **System queries** with context: `{ mood: 'exciting', timeAvailable: 120 }`
4. **ReasoningBank** selects best pattern (action_based, 85% success)
5. **Vector search** finds matching content in 0.95ms
6. **Response includes** title, streaming platforms, deep links
7. **User clicks** → Opens directly in streaming app
8. **Feedback loop** records success, improves future recommendations

**Total time: ~2 seconds vs 45 minutes**
