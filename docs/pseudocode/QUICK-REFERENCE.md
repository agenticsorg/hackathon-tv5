# CLI Pseudocode - Quick Reference Card

## Commands (8 Total)

```
┌──────────────┬──────────────┬─────────────────────────────────────┐
│   Command    │  Complexity  │           Description               │
├──────────────┼──────────────┼─────────────────────────────────────┤
│ search       │ O(n log n)   │ Search media with filters           │
│ recommend    │ O(n log n)   │ Personalized recommendations        │
│ watchlist    │ O(1)/O(n)    │ Manage watchlist (sync is O(n))     │
│ devices      │ O(n log n)   │ Manage streaming devices            │
│ cast         │ O(p×c)       │ Cast media to device                │
│ auth         │ O(t)         │ OAuth 2.0 authentication            │
│ mcp          │ O(1)         │ Start MCP server (STDIO/SSE)        │
│ config       │ O(1)         │ Configuration management            │
└──────────────┴──────────────┴─────────────────────────────────────┘
```

## Key Algorithms

### Search
```
Input: query, filters
1. Normalize query
2. Check cache (5-min TTL)
3. Build filter spec
4. Execute API call
5. Cache results
6. Interactive browse OR format output
```

### Recommend
```
Input: context, mood, preferences
1. Build user profile
2. Get user history
3. Call recommendation engine
4. Score each item (weighted 0-100)
   - Base: 75, Mood: +20, Context: +15
   - Duration: +10, Platform: +5, History: +25
5. Sort by score
6. Format with rationale
```

### Watchlist Sync
```
1. Get local items
2. Get cloud items
3. Calculate diff (3-way merge)
   - toAdd (local → cloud)
   - toRemove (cloud → local)
   - toUpdate (conflicts)
4. Resolve conflicts
   - Server wins for titles
   - Client wins for notes
   - Latest timestamp wins
5. Apply changes
```

### Deep Link
```
Input: media, platform, device
1. Get platform base URL
2. Get platform-specific media ID
3. Switch on platform:
   - Netflix: /watch/{id}?t={time}
   - Hulu: /watch/{id}
   - Disney+: /video/{id}
   - Prime: /gp/video/detail/{id}
   - YouTube: /watch?v={id}&t={time}s
   - Spotify: /track/{id}
4. Add quality/subtitle params
5. Return { url, protocol, fallback }
```

### OAuth Login
```
1. Request device code from auth server
2. Display verification URL + user code
3. Show spinner "Waiting..."
4. Poll every 5 seconds (max 60 attempts)
5. Check status:
   - "authorized" → save tokens → success
   - "pending" → continue polling
   - "denied"/"expired" → error
6. Get user info
7. Save config
```

## Data Structures

### MediaItem
```
{
  id: string
  title: string
  type: "movie" | "tv" | "music"
  year: integer (1888-current+2)
  rating: float (0-10)
  genres: string[]
  platforms: string[]
  duration: integer (minutes)
  description: string
  posterUrl: string
  cast: Person[]
  directors: Person[]
  metadata: Map<string, any>
}

Size: ~1-2 KB
Indices: id, (type,year), platform, genre
```

### SearchOptions
```
{
  type: MediaType[]
  genre: string[]
  platform: string[]
  year: Range<integer>
  rating: Range<float>
  limit: integer (1-100, default 20)
  offset: integer (≥0, default 0)
  sortBy: SortField (default "relevance")
  sortOrder: "asc" | "desc" (default "desc")
  interactive: boolean (default false)
  format: "table" | "json" | "pretty"
}
```

### LRUCache
```
Fields:
  capacity: 1000 (max entries)
  cache: Map<key, CacheEntry>
  accessOrder: DoublyLinkedList<key>

Operations:
  get(key): O(1)
    - Check expiry
    - Move to front
    - Touch (update hits)

  set(key, value, ttl): O(1)
    - Evict LRU if full
    - Add to front
    - Set expiry

  evict(key): O(1)
    - Remove from map
    - Remove from list
```

### WatchlistDiff
```
{
  toAdd: WatchlistItem[]        // Local → Cloud
  toRemove: string[]            // Cloud → Local
  toUpdate: WatchlistItem[]     // Conflicts
  conflicts: ConflictRecord[]
}

Algorithm: O(n+m)
  1. Create maps: localMap, cloudMap
  2. Find items only in local → toAdd
  3. Find items only in cloud → toRemove
  4. Find items in both → check conflicts
  5. Resolve with strategy (server/client/latest/merge)
```

## Exit Codes

```
0 - Success
1 - General error
2 - Invalid arguments
3 - Authentication required
4 - Network error
5 - Permission denied
6 - Resource not found
7 - Timeout
8 - User cancelled
```

## Design Patterns

```
┌────────────────┬──────────────────────────────────────┐
│    Pattern     │            Usage                     │
├────────────────┼──────────────────────────────────────┤
│ Command        │ Each CLI command implements execute()│
│ Strategy       │ Output formatters (table/json)       │
│ Observer       │ Progress updates, events             │
│ Builder        │ SearchOptions, DeepLink construction │
│ Factory        │ Platform-specific deep links         │
└────────────────┴──────────────────────────────────────┘
```

## Performance

### Cache Strategy
- **TTL**: 5 minutes
- **Hit Rate**: ~80% expected
- **Eviction**: LRU (Least Recently Used)
- **Max Size**: 1000 entries (~1 GB)

### Optimizations
1. Pagination (constant memory)
2. Lazy loading (commands on-demand)
3. Connection pooling (HTTP reuse)
4. Parallel requests (independent ops)
5. Early termination (search limits)

### Memory Usage
```
Component          Typical    Maximum
─────────────────  ─────────  ─────────
CLI footprint      50 MB      200 MB
Search cache       100 MB     1 GB
Watchlist          1 MB       10 MB
Device list        100 KB     1 MB
MCP connections    10 MB/conn Variable
```

## Security

### Token Storage
- Location: OS keychain (encrypted)
- Format: JWT (access + refresh)
- Expiry: Auto-check with 5-min buffer
- Refresh: Automatic before expiry

### Input Validation
```
All inputs sanitized:
  - Email: regex validation
  - URLs: protocol whitelist
  - IDs: alphanumeric only
  - File paths: no traversal
  - SQL: parameterized queries
```

### Rate Limiting
```
MCP Server: 100 requests/minute
API calls: Exponential backoff
Auth attempts: 3 failures → lockout
```

## Error Recovery

```
Error Type    → Recovery Strategy
─────────────────────────────────────────────
Network       → Retry 3x with backoff (1s, 2s, 4s)
Auth          → Clear tokens → prompt login
Timeout       → Return partial results if any
Parse         → Show error + example
Not Found     → Suggest alternatives
Permission    → Check auth → prompt if needed
```

## Testing Strategy

### Unit Tests
```
✓ Each algorithm independently
✓ Mock external dependencies
✓ Edge cases (empty, max, invalid)
✓ Error paths
Target: 90%+ coverage
```

### Integration Tests
```
✓ Command execution flows
✓ API interactions
✓ Auth flows
✓ Cache behavior
```

### E2E Tests
```
✓ Real user scenarios
✓ Interactive mode
✓ Multi-command workflows
✓ Performance benchmarks
```

## Implementation Phases

```
Phase 1 (Week 1-2): Foundation
  ├─ CLI framework
  ├─ Argument parser
  ├─ Config manager
  └─ Basic commands

Phase 2 (Week 3-4): Core Features
  ├─ Auth (OAuth)
  ├─ API client
  ├─ Search
  └─ Watchlist

Phase 3 (Week 5-6): Advanced
  ├─ Recommend
  ├─ Devices
  ├─ Cast
  └─ Interactive modes

Phase 4 (Week 7-8): MCP
  ├─ STDIO transport
  ├─ SSE transport
  ├─ Tool handlers
  └─ Rate limiting

Phase 5 (Week 9-10): Polish
  ├─ Error handling
  ├─ Performance tuning
  ├─ Security audit
  └─ User testing
```

## File Structure

```
src/
├── cli.ts                    # Entry point
├── commands/
│   ├── search.ts
│   ├── recommend.ts
│   ├── watchlist.ts
│   ├── devices.ts
│   ├── cast.ts
│   ├── auth.ts
│   ├── mcp.ts
│   └── config.ts
├── lib/
│   ├── parser.ts            # Argument parsing
│   ├── validator.ts         # Input validation
│   ├── formatter.ts         # Output formatting
│   └── cache.ts             # LRU cache
├── api/
│   ├── client.ts
│   ├── auth.ts
│   ├── media.ts
│   └── devices.ts
├── mcp/
│   ├── server.ts
│   ├── stdio.ts
│   ├── sse.ts
│   └── handlers.ts
└── types.ts
```

## Complexity Cheat Sheet

```
Operation                Time        Space
──────────────────────────────────────────────
Argument parse           O(n)        O(n)
Command lookup           O(1)        O(c)
Search (cached)          O(1)        O(r)
Search (API)             O(n log n)  O(r)
Recommend                O(n log n)  O(n)
Watchlist add            O(1)        O(1)
Watchlist sync           O(n+m)      O(n+m)
Device list              O(d log d)  O(d)
Cast                     O(p×c)      O(1)
Auth poll                O(t)        O(1)
MCP handle               O(1)        O(1)
Table format             O(r×c)      O(r×c)
Cache get/set            O(1)        O(k)

Variables:
  n = args/results, c = commands, r = cached results
  d = devices, p = platforms, c = capabilities
  t = attempts, k = cache size
```

## Documentation Map

```
INDEX.md
  ├─ Navigation guide
  ├─ Quick reference
  └─ Roadmap

README.md
  ├─ Executive summary
  ├─ Component overview
  └─ Guidelines

cli-implementation.md
  ├─ All 8 commands
  ├─ 14 algorithms
  └─ Subroutines

algorithm-flows.md
  ├─ 10 flowcharts
  ├─ Decision trees
  └─ State machines

data-structures.md
  ├─ 25+ structures
  ├─ Memory estimates
  └─ Index strategies
```

---

**Quick Reference v1.0** | SPARC Pseudocode Phase | 2025-12-06
