# Redis Key Patterns - Media Gateway

## Overview

Redis serves as the primary caching and session management layer for the Media Gateway platform. This document defines all key patterns, data structures, and TTL policies.

## Key Naming Convention

```
{namespace}:{entity}:{identifier}[:{subkey}]
```

## 1. Session Management

### Active Sessions
```
Pattern: session:active:{session_id}
Type: Hash
TTL: 7 days (604800s)
Fields:
  - user_id: UUID
  - device_id: UUID
  - created_at: ISO8601 timestamp
  - last_active: ISO8601 timestamp
  - ip_address: string
  - user_agent: string
```

### Session Index by User
```
Pattern: session:user:{user_id}
Type: Set
TTL: 30 days
Members: session_id strings
```

### Session Blacklist (Logout/Revoke)
```
Pattern: session:blacklist:{session_id}
Type: String
TTL: 7 days
Value: revoked_at timestamp
```

## 2. Authentication & Authorization

### JWT Token Blacklist
```
Pattern: auth:blacklist:{token_jti}
Type: String
TTL: Token expiration time
Value: "revoked"
```

### OAuth State
```
Pattern: auth:oauth:{state}
Type: Hash
TTL: 10 minutes (600s)
Fields:
  - provider: string
  - redirect_uri: string
  - created_at: timestamp
```

### Refresh Token Metadata
```
Pattern: auth:refresh:{token_hash}
Type: Hash
TTL: 30 days (2592000s)
Fields:
  - user_id: UUID
  - device_id: UUID
  - issued_at: timestamp
  - last_used: timestamp
```

## 3. Rate Limiting

### API Rate Limits
```
Pattern: ratelimit:api:{user_id|ip}:{endpoint}
Type: String (counter)
TTL: 60s (1 minute window)
Value: request count
```

### Search Rate Limits
```
Pattern: ratelimit:search:{user_id}
Type: String
TTL: 3600s (1 hour window)
Value: search count
```

### Recommendation Rate Limits
```
Pattern: ratelimit:recommend:{user_id}
Type: String
TTL: 300s (5 minute window)
Value: recommendation request count
```

## 4. Content Caching

### Content Metadata
```
Pattern: content:meta:{content_id}
Type: Hash
TTL: 24 hours (86400s)
Fields:
  - title: string
  - content_type: string
  - release_date: ISO8601
  - popularity_score: float
  - average_rating: float
  - genres: JSON array
  - platforms: JSON array
```

### Content Availability by Platform/Region
```
Pattern: content:avail:{content_id}:{platform}:{region}
Type: Hash
TTL: 6 hours (21600s)
Fields:
  - availability_type: string
  - price_cents: integer
  - deep_link: string
  - video_qualities: JSON array
  - expires_at: ISO8601
```

### Search Results Cache
```
Pattern: cache:search:{query_hash}
Type: List
TTL: 1 hour (3600s)
Members: JSON-encoded content objects
```

### Popular Content
```
Pattern: cache:popular:{region}:{content_type}
Type: Sorted Set
TTL: 30 minutes (1800s)
Members: content_id
Scores: popularity_score
```

## 5. User Data Caching

### User Profile
```
Pattern: user:profile:{user_id}
Type: Hash
TTL: 1 hour (3600s)
Fields:
  - email: string
  - display_name: string
  - avatar_url: string
  - is_premium: boolean
  - subscription_tier: string
```

### User Preferences
```
Pattern: user:prefs:{user_id}
Type: Hash
TTL: 24 hours (86400s)
Fields:
  - favorite_genres: JSON array
  - preferred_languages: JSON array
  - subscribed_platforms: JSON array
  - max_content_rating: string
```

### User Watchlist (Cache)
```
Pattern: user:watchlist:{user_id}
Type: Sorted Set
TTL: 5 minutes (300s)
Members: content_id
Scores: added_at timestamp
```

### Watch Progress Cache
```
Pattern: user:progress:{user_id}:{content_id}
Type: Hash
TTL: 30 minutes (1800s)
Fields:
  - position_seconds: integer
  - duration_seconds: integer
  - completion_rate: float
  - last_watched: ISO8601
  - device_id: UUID
```

## 6. Recommendation Engine

### User Recommendation Cache
```
Pattern: recommend:user:{user_id}
Type: List
TTL: 15 minutes (900s)
Members: JSON-encoded content recommendations
```

### Similar Content
```
Pattern: recommend:similar:{content_id}
Type: Sorted Set
TTL: 1 hour (3600s)
Members: content_id
Scores: similarity score
```

### Trending Content
```
Pattern: recommend:trending:{region}:{timeframe}
Type: Sorted Set
TTL: 10 minutes (600s)
Members: content_id
Scores: trending score
```

## 7. Platform Data Sync

### Last Sync Timestamp
```
Pattern: sync:platform:{platform}:last
Type: String
TTL: None (persistent)
Value: ISO8601 timestamp
```

### Sync Lock
```
Pattern: sync:lock:{platform}
Type: String
TTL: 5 minutes (300s)
Value: sync_job_id
```

### Sync Progress
```
Pattern: sync:progress:{sync_job_id}
Type: Hash
TTL: 1 hour (3600s)
Fields:
  - total_items: integer
  - processed_items: integer
  - failed_items: integer
  - started_at: ISO8601
  - updated_at: ISO8601
```

## 8. Queue Management

### Job Queue (Using Lists)
```
Pattern: queue:job:{job_type}
Type: List
TTL: None
Members: JSON-encoded job payloads
```

### Dead Letter Queue
```
Pattern: queue:dlq:{job_type}
Type: List
TTL: 7 days
Members: JSON-encoded failed jobs
```

### Job Status
```
Pattern: job:status:{job_id}
Type: Hash
TTL: 24 hours (86400s)
Fields:
  - status: string (pending|processing|completed|failed)
  - created_at: ISO8601
  - started_at: ISO8601
  - completed_at: ISO8601
  - error: string
```

## 9. Feature Flags

### Global Feature Flags
```
Pattern: feature:flag:{flag_name}
Type: String
TTL: None
Value: JSON config
```

### User-specific Feature Flags
```
Pattern: feature:user:{user_id}:{flag_name}
Type: String
TTL: 1 hour
Value: "enabled" | "disabled"
```

## 10. Analytics & Metrics

### Real-time Counters
```
Pattern: metrics:counter:{metric_name}:{window}
Type: String
TTL: Based on window (1m, 5m, 1h, 1d)
Value: count
```

### User Activity Tracking
```
Pattern: analytics:activity:{user_id}:{date}
Type: Hash
TTL: 90 days
Fields:
  - searches: integer
  - content_views: integer
  - watchlist_adds: integer
  - recommendations_requested: integer
```

## Cache Invalidation Strategies

### Invalidation Patterns

1. **Time-based**: Automatic TTL expiration
2. **Event-based**: Manual deletion on data updates
3. **Pattern-based**: Use `SCAN` + `DEL` for bulk invalidation

### Invalidation Triggers

- User profile update: `DEL user:profile:{user_id}`
- Content update: `DEL content:meta:{content_id}`, `DEL content:avail:{content_id}:*`
- Watchlist change: `DEL user:watchlist:{user_id}`
- Platform sync: `DEL cache:popular:*`, `DEL recommend:trending:*`

## Redis Configuration Recommendations

```yaml
maxmemory-policy: allkeys-lru
maxmemory: 4gb
save: ""  # Disable RDB persistence (cache only)
appendonly: no  # Disable AOF (cache only)
tcp-keepalive: 300
timeout: 0
```

## Monitoring Keys

```bash
# Key count by pattern
redis-cli --scan --pattern 'session:*' | wc -l

# Memory usage by pattern
redis-cli --bigkeys

# TTL distribution
redis-cli --scan --pattern '*' | xargs -I{} redis-cli TTL {}
```

## Best Practices

1. **Always set TTL**: Every key must have an appropriate TTL
2. **Use namespaces**: Consistent prefix patterns for organization
3. **Hash for structured data**: Use hashes for multi-field entities
4. **Sorted sets for rankings**: Use for leaderboards, trending, etc.
5. **Pipeline operations**: Batch Redis commands when possible
6. **Monitor memory**: Alert on >80% memory usage
7. **Key naming consistency**: Follow the namespace:entity:id pattern
8. **Avoid large collections**: Limit set/list sizes to <10k items

## Migration Notes

When updating key patterns:
1. Implement dual-write during migration
2. Use versioned key patterns: `{namespace}:v2:{entity}:{id}`
3. Monitor old pattern usage before cleanup
4. Set short TTLs on deprecated patterns for auto-cleanup
