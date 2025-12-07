# Rate Limiting Algorithm (Token Bucket) - Pseudocode Specification

## Overview
Token bucket rate limiting provides flexible, efficient throttling with burst capacity. This implementation supports per-user, per-IP, and per-endpoint rate limits with sliding window tracking.

---

## Data Structures

```
STRUCTURE TokenBucket:
    key: String (rate_limit:{type}:{identifier}:{endpoint})
    capacity: Integer (maximum tokens, e.g., 100)
    tokens: Float (current token count)
    refill_rate: Float (tokens per second, e.g., 10)
    last_refill: Timestamp
    created_at: Timestamp

STRUCTURE RateLimitConfig:
    endpoint: String (e.g., "/api/videos", "/oauth/token")
    limit_type: Enum["per_user", "per_ip", "per_client", "global"]
    capacity: Integer
    refill_rate: Float (tokens per second)
    window: Integer (seconds)
    burst_allowance: Float (multiplier for burst capacity, e.g., 1.5)

STRUCTURE RateLimitResult:
    allowed: Boolean
    remaining: Integer (tokens remaining)
    reset_at: Timestamp (when bucket refills to capacity)
    retry_after: Integer (seconds to wait if blocked)

STRUCTURE SlidingWindowCounter:
    key: String
    current_window: Integer (Unix timestamp / window_size)
    current_count: Integer
    previous_count: Integer
    window_size: Integer (seconds)
```

---

## Algorithm 1: Token Bucket Rate Limiting

```
ALGORITHM: CheckRateLimit
INPUT: identifier (string), limit_type (string), endpoint (string)
OUTPUT: rate_limit_result (RateLimitResult)

BEGIN
    // Step 1: Get rate limit configuration
    config ← GetRateLimitConfig(endpoint, limit_type)

    IF config is null THEN
        // No rate limit configured - allow
        RETURN RateLimitResult{
            allowed: true,
            remaining: -1,
            reset_at: null,
            retry_after: 0
        }
    END IF

    // Step 2: Build bucket key
    bucket_key ← "rate_limit:" + limit_type + ":" + identifier + ":" + endpoint

    // Step 3: Get or create bucket
    bucket ← Cache.get(bucket_key)

    IF bucket is null THEN
        // Create new bucket
        bucket ← TokenBucket{
            key: bucket_key,
            capacity: config.capacity,
            tokens: config.capacity, // Start full
            refill_rate: config.refill_rate,
            last_refill: GetCurrentTimestamp(),
            created_at: GetCurrentTimestamp()
        }
    ELSE
        // Step 4: Refill tokens based on elapsed time
        current_time ← GetCurrentTimestamp()
        elapsed ← current_time - bucket.last_refill

        // Calculate tokens to add
        tokens_to_add ← elapsed * bucket.refill_rate

        // Update bucket
        bucket.tokens ← MIN(bucket.tokens + tokens_to_add, bucket.capacity)
        bucket.last_refill ← current_time
    END IF

    // Step 5: Check if request is allowed (needs at least 1 token)
    IF bucket.tokens >= 1 THEN
        // Consume token
        bucket.tokens ← bucket.tokens - 1

        // Step 6: Save updated bucket
        ttl ← CalculateBucketTTL(bucket)
        Cache.setWithTTL(bucket_key, bucket, ttl)

        // Step 7: Calculate reset time
        tokens_until_full ← bucket.capacity - bucket.tokens
        seconds_until_full ← tokens_until_full / bucket.refill_rate
        reset_at ← GetCurrentTimestamp() + seconds_until_full

        RETURN RateLimitResult{
            allowed: true,
            remaining: FLOOR(bucket.tokens),
            reset_at: reset_at,
            retry_after: 0
        }
    ELSE
        // Step 8: Request blocked
        // Calculate wait time for next token
        tokens_needed ← 1 - bucket.tokens
        retry_after ← CEIL(tokens_needed / bucket.refill_rate)

        // Save bucket state
        ttl ← CalculateBucketTTL(bucket)
        Cache.setWithTTL(bucket_key, bucket, ttl)

        // Step 9: Audit log rate limit violation
        AuditLog.record(
            event="rate_limit_exceeded",
            identifier=identifier,
            limit_type=limit_type,
            endpoint=endpoint,
            retry_after=retry_after,
            severity="warning"
        )

        RETURN RateLimitResult{
            allowed: false,
            remaining: 0,
            reset_at: GetCurrentTimestamp() + retry_after,
            retry_after: retry_after
        }
    END IF
END

SUBROUTINE: CalculateBucketTTL
INPUT: bucket (TokenBucket)
OUTPUT: ttl (integer, seconds)

BEGIN
    // Calculate time until bucket is full
    tokens_until_full ← bucket.capacity - bucket.tokens
    seconds_until_full ← tokens_until_full / bucket.refill_rate

    // Add buffer to prevent premature deletion
    ttl ← CEIL(seconds_until_full) + 60

    RETURN ttl
END
```

**Time Complexity**: O(1) with Redis cache
**Space Complexity**: O(1) per bucket

---

## Algorithm 2: Sliding Window Rate Limiting

```
ALGORITHM: CheckSlidingWindowLimit
INPUT: identifier (string), limit_type (string), endpoint (string)
OUTPUT: rate_limit_result (RateLimitResult)

CONSTANTS:
    WINDOW_SIZE = 60 seconds (1 minute)
    MAX_REQUESTS = 100

BEGIN
    // Step 1: Get configuration
    config ← GetRateLimitConfig(endpoint, limit_type)

    // Step 2: Calculate current and previous windows
    current_timestamp ← GetCurrentTimestamp()
    current_window ← FLOOR(current_timestamp / config.window)
    previous_window ← current_window - 1

    // Step 3: Build keys
    current_key ← "rate_limit_window:" + limit_type + ":" + identifier + ":" + endpoint + ":" + current_window
    previous_key ← "rate_limit_window:" + limit_type + ":" + identifier + ":" + endpoint + ":" + previous_window

    // Step 4: Get counts from both windows
    current_count ← Cache.get(current_key) OR 0
    previous_count ← Cache.get(previous_key) OR 0

    // Step 5: Calculate weighted count (sliding window)
    window_progress ← (current_timestamp % config.window) / config.window
    estimated_count ← (previous_count * (1 - window_progress)) + current_count

    // Step 6: Check if limit exceeded
    IF estimated_count >= config.capacity THEN
        // Calculate retry_after
        time_in_current_window ← current_timestamp % config.window
        retry_after ← config.window - time_in_current_window

        AuditLog.record(
            event="sliding_window_limit_exceeded",
            identifier=identifier,
            limit_type=limit_type,
            endpoint=endpoint,
            estimated_count=estimated_count,
            capacity=config.capacity,
            retry_after=retry_after,
            severity="warning"
        )

        RETURN RateLimitResult{
            allowed: false,
            remaining: 0,
            reset_at: current_timestamp + retry_after,
            retry_after: retry_after
        }
    END IF

    // Step 7: Increment current window counter
    Cache.increment(current_key)
    Cache.setTTL(current_key, config.window * 2) // Keep for 2 windows

    // Step 8: Calculate remaining requests
    remaining ← config.capacity - CEIL(estimated_count) - 1

    RETURN RateLimitResult{
        allowed: true,
        remaining: remaining,
        reset_at: current_timestamp + (config.window - (current_timestamp % config.window)),
        retry_after: 0
    }
END
```

**Time Complexity**: O(1)
**Space Complexity**: O(1) per window

---

## Algorithm 3: Adaptive Rate Limiting

```
ALGORITHM: CheckAdaptiveRateLimit
INPUT: identifier (string), limit_type (string), endpoint (string), user_trust_score (float)
OUTPUT: rate_limit_result (RateLimitResult)

BEGIN
    // Step 1: Get base configuration
    base_config ← GetRateLimitConfig(endpoint, limit_type)

    // Step 2: Adjust limits based on trust score
    // Trust score: 0.0 (untrusted) to 1.0 (highly trusted)
    // High trust = higher limits

    adjusted_capacity ← base_config.capacity * (1 + user_trust_score)
    adjusted_refill_rate ← base_config.refill_rate * (1 + (user_trust_score * 0.5))

    // Step 3: Create adjusted config
    adjusted_config ← RateLimitConfig{
        endpoint: base_config.endpoint,
        limit_type: base_config.limit_type,
        capacity: FLOOR(adjusted_capacity),
        refill_rate: adjusted_refill_rate,
        window: base_config.window,
        burst_allowance: base_config.burst_allowance
    }

    // Step 4: Apply token bucket with adjusted limits
    RETURN CheckRateLimitWithConfig(identifier, adjusted_config)
END

SUBROUTINE: CalculateUserTrustScore
INPUT: user_id (string)
OUTPUT: trust_score (float, 0.0 to 1.0)

BEGIN
    // Factors:
    // - Account age (older = more trusted)
    // - Email verified
    // - Previous rate limit violations
    // - Successful API calls vs errors

    user ← GetUserByID(user_id)
    score ← 0.0

    // Account age (max 0.3)
    account_age_days ← (GetCurrentTimestamp() - user.created_at) / 86400
    age_score ← MIN(account_age_days / 365, 0.3) // Cap at 1 year

    // Email verified (0.2)
    email_score ← user.email_verified ? 0.2 : 0.0

    // Rate limit violation history (max -0.2)
    violations ← CountRateLimitViolations(user_id, last_30_days=true)
    violation_penalty ← MIN(violations * 0.05, 0.2)

    // API success rate (max 0.5)
    success_rate ← GetAPISuccessRate(user_id, last_7_days=true)
    success_score ← success_rate * 0.5

    // Total score
    score ← age_score + email_score + success_score - violation_penalty
    score ← CLAMP(score, 0.0, 1.0)

    RETURN score
END
```

**Time Complexity**: O(1) with cached user metrics
**Space Complexity**: O(1)

---

## Algorithm 4: Distributed Rate Limiting (Redis Lua Script)

```
ALGORITHM: DistributedTokenBucket
INPUT: bucket_key (string), capacity (integer), refill_rate (float)
OUTPUT: allowed (boolean), remaining (integer)

// This algorithm runs as an atomic Lua script in Redis
LUA_SCRIPT:
```lua
-- KEYS[1]: bucket_key
-- ARGV[1]: capacity
-- ARGV[2]: refill_rate (tokens per second)
-- ARGV[3]: current_timestamp

local bucket_key = KEYS[1]
local capacity = tonumber(ARGV[1])
local refill_rate = tonumber(ARGV[2])
local current_time = tonumber(ARGV[3])

-- Get bucket data
local bucket = redis.call('HMGET', bucket_key, 'tokens', 'last_refill')
local tokens = tonumber(bucket[1])
local last_refill = tonumber(bucket[2])

-- Initialize if bucket doesn't exist
if not tokens then
    tokens = capacity
    last_refill = current_time
end

-- Refill tokens
local elapsed = current_time - last_refill
local tokens_to_add = elapsed * refill_rate
tokens = math.min(tokens + tokens_to_add, capacity)

-- Check if request allowed
if tokens >= 1 then
    -- Consume token
    tokens = tokens - 1

    -- Update bucket
    redis.call('HMSET', bucket_key,
        'tokens', tokens,
        'last_refill', current_time)

    -- Set TTL
    local ttl = math.ceil((capacity - tokens) / refill_rate) + 60
    redis.call('EXPIRE', bucket_key, ttl)

    return {1, math.floor(tokens)} -- allowed, remaining
else
    -- Request blocked
    redis.call('HMSET', bucket_key,
        'tokens', tokens,
        'last_refill', current_time)

    return {0, 0} -- blocked, remaining
end
```
```

**Time Complexity**: O(1) atomic operation
**Space Complexity**: O(1)

---

## Algorithm 5: Burst Detection and Mitigation

```
ALGORITHM: DetectAndMitigateBurst
INPUT: identifier (string), endpoint (string)
OUTPUT: mitigation_action (string)

CONSTANTS:
    BURST_WINDOW = 10 seconds
    BURST_THRESHOLD = 50 requests
    PENALTY_DURATION = 300 seconds (5 minutes)

BEGIN
    // Step 1: Track requests in short window
    burst_key ← "burst:" + identifier + ":" + endpoint
    current_window ← FLOOR(GetCurrentTimestamp() / BURST_WINDOW)
    window_key ← burst_key + ":" + current_window

    // Step 2: Increment counter
    request_count ← Cache.increment(window_key)
    Cache.setTTL(window_key, BURST_WINDOW * 2)

    // Step 3: Check if burst threshold exceeded
    IF request_count > BURST_THRESHOLD THEN
        // Step 4: Apply penalty
        penalty_key ← "penalty:" + identifier + ":" + endpoint

        existing_penalty ← Cache.get(penalty_key)

        IF existing_penalty is null THEN
            // First offense - warning
            Cache.setWithTTL(penalty_key, "warning", PENALTY_DURATION)

            AuditLog.record(
                event="burst_detected_warning",
                identifier=identifier,
                endpoint=endpoint,
                request_count=request_count,
                severity="warning"
            )

            RETURN "warning_issued"
        ELSE IF existing_penalty == "warning" THEN
            // Second offense - temporary block
            Cache.setWithTTL(penalty_key, "blocked", PENALTY_DURATION * 2)

            AuditLog.record(
                event="burst_detected_blocked",
                identifier=identifier,
                endpoint=endpoint,
                request_count=request_count,
                severity="error"
            )

            RETURN "blocked_temporarily"
        ELSE
            // Already blocked
            RETURN "already_blocked"
        END IF
    END IF

    RETURN "normal"
END
```

**Time Complexity**: O(1)
**Space Complexity**: O(w) where w = number of windows tracked

---

## Rate Limit Configurations

```
// OAuth Token Endpoint (strict)
RATE_LIMIT_CONFIG: /oauth/token
    limit_type: per_client
    capacity: 5
    refill_rate: 0.083 (5 requests per minute)
    window: 60
    burst_allowance: 1.0 (no burst)

// API Endpoints (moderate)
RATE_LIMIT_CONFIG: /api/*
    limit_type: per_user
    capacity: 100
    refill_rate: 10 (600 requests per minute)
    window: 60
    burst_allowance: 1.5

// Device Authorization (lenient)
RATE_LIMIT_CONFIG: /oauth/device
    limit_type: per_ip
    capacity: 10
    refill_rate: 0.033 (2 requests per minute)
    window: 60
    burst_allowance: 1.0

// Public Content (very lenient)
RATE_LIMIT_CONFIG: /public/*
    limit_type: per_ip
    capacity: 1000
    refill_rate: 100 (6000 requests per minute)
    window: 60
    burst_allowance: 2.0
```

---

## HTTP Response Headers

```
ALGORITHM: AddRateLimitHeaders
INPUT: http_response, rate_limit_result
OUTPUT: modified_response

BEGIN
    // Standard rate limit headers (draft RFC)
    http_response.setHeader("X-RateLimit-Limit", rate_limit_result.capacity)
    http_response.setHeader("X-RateLimit-Remaining", rate_limit_result.remaining)
    http_response.setHeader("X-RateLimit-Reset", rate_limit_result.reset_at)

    IF NOT rate_limit_result.allowed THEN
        http_response.setStatus(429) // Too Many Requests
        http_response.setHeader("Retry-After", rate_limit_result.retry_after)

        http_response.setBody({
            error: "rate_limit_exceeded",
            error_description: "Too many requests. Please retry after " + rate_limit_result.retry_after + " seconds.",
            retry_after: rate_limit_result.retry_after
        })
    END IF

    RETURN http_response
END
```

---

## Security Best Practices

### 1. Multiple Layers
Apply rate limits at multiple levels:
- **Per IP**: Prevent DDoS attacks
- **Per User**: Prevent abuse from authenticated users
- **Per Client**: Prevent misbehaving OAuth clients
- **Per Endpoint**: Protect expensive operations

### 2. Token Bucket vs Sliding Window
- **Token Bucket**: Better for APIs with burst traffic (file uploads)
- **Sliding Window**: Better for strict enforcement (authentication endpoints)

### 3. Graceful Degradation
- Return 429 status with `Retry-After` header
- Include helpful error messages
- Don't penalize users for minor violations

### 4. Monitoring
Track metrics:
- Rate limit hit rate (should be < 1%)
- Burst detection frequency
- Per-endpoint limit effectiveness
- User trust score distribution

### 5. Whitelisting
- Whitelist internal services
- Whitelist trusted partners with API keys
- Use higher limits for paid tiers

---

## Complexity Analysis

### Time Complexity
- **CheckRateLimit**: O(1) with Redis
- **CheckSlidingWindowLimit**: O(1)
- **CheckAdaptiveRateLimit**: O(1) with cached user data
- **DistributedTokenBucket**: O(1) atomic operation

### Space Complexity
- **Per Bucket**: O(1) - Fixed size (tokens + last_refill)
- **Sliding Window**: O(w) where w = number of windows (2 per identifier)
- **Burst Detection**: O(w) per identifier

### Redis Commands
```redis
# Token bucket
HMGET rate_limit:user:123:/api/videos tokens last_refill
HMSET rate_limit:user:123:/api/videos tokens 99 last_refill 1701234567
EXPIRE rate_limit:user:123:/api/videos 120

# Sliding window
INCR rate_limit_window:user:123:/api/videos:28370395
EXPIRE rate_limit_window:user:123:/api/videos:28370395 120
```

---

## Audit Events

- `rate_limit_exceeded` - Request blocked
- `burst_detected_warning` - Burst traffic warning
- `burst_detected_blocked` - Temporarily blocked for burst
- `rate_limit_config_updated` - Configuration changed
- `rate_limit_whitelist_added` - Identifier whitelisted

---

**Algorithm Designed By**: Security Algorithm Design Agent
**SPARC Phase**: Pseudocode
**Rate Limiting Strategy**: Token Bucket with Sliding Window hybrid
**Last Updated**: 2025-12-06
