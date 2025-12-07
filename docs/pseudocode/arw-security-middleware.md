# ARW Manifest & Security Middleware - Pseudocode Design

## 1. ARW Manifest Generation

### 1.1 Dynamic Manifest Algorithm

```
ALGORITHM: GenerateARWManifest
INPUT:
    request (HttpRequest) - Incoming request for manifest
OUTPUT:
    manifest (object) - ARW manifest JSON

DATA STRUCTURES:
    ToolRegistry: Map of available MCP tools
        Type: In-memory registry
        Key: tool_name
        Fields: description, schema, required_scopes, enabled

    ManifestCache: Cache for generated manifests
        Type: LRU Cache
        Size: 100 entries
        TTL: 1 hour

CONSTANTS:
    MANIFEST_VERSION = "1.0"
    BASE_URL = process.env.ARW_BASE_URL OR "https://api.example.com"

BEGIN
    // Step 1: Get host from request
    host ← request.headers["host"] OR BASE_URL
    protocol ← request.protocol OR "https"
    baseUrl ← protocol + "://" + host

    // Step 2: Check cache
    cacheKey ← "arw:manifest:" + baseUrl
    cachedManifest ← ManifestCache.get(cacheKey)

    IF cachedManifest is not null THEN
        LOG("Cache hit for ARW manifest")
        RETURN cachedManifest
    END IF

    // Step 3: Get enabled tools
    enabledTools ← GetEnabledTools()

    // Step 4: Build capabilities object
    capabilities ← BuildCapabilities(enabledTools)

    // Step 5: Generate OAuth scopes
    scopes ← GenerateOAuthScopes(enabledTools)

    // Step 6: Build endpoints object
    endpoints ← {
        mcp: baseUrl + "/mcp",
        oauth: {
            authorize: baseUrl + "/oauth/authorize",
            token: baseUrl + "/oauth/token",
            revoke: baseUrl + "/oauth/revoke"
        },
        health: baseUrl + "/health"
    }

    // Step 7: Assemble manifest
    manifest ← {
        version: MANIFEST_VERSION,
        name: "Media Gateway MCP Server",
        description: "Unified media search, recommendations, and playback control",
        provider: {
            name: "Media Gateway",
            url: "https://example.com",
            support_email: "support@example.com"
        },
        endpoints: endpoints,
        capabilities: capabilities,
        authentication: {
            type: "oauth2",
            flows: {
                authorization_code: {
                    authorization_url: endpoints.oauth.authorize,
                    token_url: endpoints.oauth.token,
                    scopes: scopes
                }
            }
        },
        metadata: {
            generated_at: GetCurrentTime(),
            transport: ["stdio", "sse"],
            protocol_version: "2024-11-05"
        }
    }

    // Step 8: Cache manifest
    ManifestCache.set(cacheKey, manifest)

    // Step 9: Return manifest
    RETURN manifest
END


SUBROUTINE: GetEnabledTools
INPUT: none
OUTPUT: tools (array)

BEGIN
    // Get all registered tools
    allTools ← ToolRegistry.getAllTools()

    // Filter enabled tools
    enabledTools ← []

    FOR EACH tool IN allTools DO
        IF tool.enabled AND tool.visible_in_manifest THEN
            enabledTools.append(tool)
        END IF
    END FOR

    RETURN enabledTools
END


SUBROUTINE: BuildCapabilities
INPUT: tools (array)
OUTPUT: capabilities (object)

BEGIN
    capabilities ← {
        content: {
            search: false,
            details: false,
            recommendations: false
        },
        devices: {
            list: false,
            status: false
        },
        playback: {
            initiate: false,
            control: false
        }
    }

    // Map tools to capability categories
    FOR EACH tool IN tools DO
        name ← tool.name

        IF name EQUALS "semantic_search" THEN
            capabilities.content.search ← true

        ELSE IF name EQUALS "get_content_details" THEN
            capabilities.content.details ← true

        ELSE IF name EQUALS "get_recommendations" THEN
            capabilities.content.recommendations ← true

        ELSE IF name EQUALS "list_devices" THEN
            capabilities.devices.list ← true

        ELSE IF name EQUALS "get_device_status" THEN
            capabilities.devices.status ← true

        ELSE IF name EQUALS "initiate_playback" THEN
            capabilities.playback.initiate ← true

        ELSE IF name EQUALS "control_playback" THEN
            capabilities.playback.control ← true
        END IF
    END FOR

    RETURN capabilities
END


SUBROUTINE: GenerateOAuthScopes
INPUT: tools (array)
OUTPUT: scopes (object)

BEGIN
    scopes ← {}

    // Collect unique scopes from all tools
    scopeSet ← new Set()

    FOR EACH tool IN tools DO
        IF tool.required_scopes is not null THEN
            FOR EACH scope IN tool.required_scopes DO
                scopeSet.add(scope)
            END FOR
        END IF
    END FOR

    // Convert to scope object with descriptions
    FOR EACH scope IN scopeSet DO
        scopes[scope] ← GetScopeDescription(scope)
    END FOR

    RETURN scopes
END


SUBROUTINE: GetScopeDescription
INPUT: scope (string)
OUTPUT: description (string)

BEGIN
    scopeDescriptions ← {
        "content:read": "Access to search and view content information",
        "content:recommendations": "Access to personalized content recommendations",
        "devices:read": "View your registered devices",
        "devices:write": "Manage your devices",
        "playback:control": "Control media playback on your devices",
        "profile:read": "Access your viewing profile and preferences"
    }

    RETURN scopeDescriptions[scope] OR "Access to " + scope
END
```

## 2. Rate Limiting Middleware

### 2.1 Token Bucket Rate Limiter

```
ALGORITHM: RateLimiter
DATA STRUCTURES:
    UserBuckets: Map of rate limit buckets per user
        Type: Redis Hash
        Key: "ratelimit:" + userId
        Fields: tokens, last_refill, requests_count

    RateLimitConfig: Configuration per tier
        Type: In-memory config
        Fields: max_tokens, refill_rate, window_size

CONSTANTS:
    DEFAULT_MAX_TOKENS = 100
    DEFAULT_REFILL_RATE = 10  // tokens per second
    BURST_MULTIPLIER = 1.5

CLASS RateLimiter
    PROPERTIES:
        config: RateLimitConfig
        redis: RedisClient

    CONSTRUCTOR(config: RateLimitConfig)
        this.config ← config OR GetDefaultConfig()
        this.redis ← ConnectToRedis()
    END CONSTRUCTOR

    FUNCTION CheckLimit(clientId: string, cost: integer = 1): boolean
        // Step 1: Get user tier
        tier ← GetUserTier(clientId)
        limits ← config[tier]

        // Step 2: Get or create bucket
        bucket ← GetBucket(clientId)

        IF bucket is null THEN
            bucket ← CreateBucket(clientId, limits)
        END IF

        // Step 3: Refill tokens based on time elapsed
        currentTime ← GetCurrentTime()
        elapsed ← (currentTime - bucket.last_refill) / 1000  // Convert to seconds

        tokensToAdd ← elapsed * limits.refill_rate
        bucket.tokens ← MIN(bucket.tokens + tokensToAdd, limits.max_tokens)
        bucket.last_refill ← currentTime

        // Step 4: Check if enough tokens available
        IF bucket.tokens >= cost THEN
            // Deduct tokens
            bucket.tokens ← bucket.tokens - cost
            bucket.requests_count ← bucket.requests_count + 1

            // Update bucket in Redis
            UpdateBucket(clientId, bucket)

            RETURN true
        ELSE
            // Not enough tokens - rate limited
            LOG_WARN("Rate limit exceeded for client: " + clientId)

            // Update metrics
            RecordRateLimitHit(clientId, tier)

            RETURN false
        END IF
    END FUNCTION

    FUNCTION GetBucket(clientId: string): Bucket or null
        key ← "ratelimit:" + clientId
        data ← redis.hGetAll(key)

        IF data is empty THEN
            RETURN null
        END IF

        RETURN {
            tokens: parseFloat(data.tokens),
            last_refill: parseInt(data.last_refill),
            requests_count: parseInt(data.requests_count)
        }
    END FUNCTION

    FUNCTION CreateBucket(clientId: string, limits: Limits): Bucket
        bucket ← {
            tokens: limits.max_tokens,
            last_refill: GetCurrentTime(),
            requests_count: 0
        }

        UpdateBucket(clientId, bucket)

        RETURN bucket
    END FUNCTION

    FUNCTION UpdateBucket(clientId: string, bucket: Bucket)
        key ← "ratelimit:" + clientId

        redis.hSet(key, {
            tokens: bucket.tokens.toString(),
            last_refill: bucket.last_refill.toString(),
            requests_count: bucket.requests_count.toString()
        })

        // Set expiration (reset after 24 hours of inactivity)
        redis.expire(key, 86400)
    END FUNCTION

    FUNCTION GetRateLimitInfo(clientId: string): Object
        bucket ← GetBucket(clientId)

        IF bucket is null THEN
            tier ← GetUserTier(clientId)
            limits ← config[tier]

            RETURN {
                remaining: limits.max_tokens,
                limit: limits.max_tokens,
                reset_in: 0
            }
        END IF

        tier ← GetUserTier(clientId)
        limits ← config[tier]

        RETURN {
            remaining: Math.floor(bucket.tokens),
            limit: limits.max_tokens,
            reset_in: CalculateResetTime(bucket, limits)
        }
    END FUNCTION

    FUNCTION CalculateResetTime(bucket: Bucket, limits: Limits): integer
        IF bucket.tokens >= limits.max_tokens THEN
            RETURN 0
        END IF

        tokensNeeded ← limits.max_tokens - bucket.tokens
        secondsNeeded ← tokensNeeded / limits.refill_rate

        RETURN Math.ceil(secondsNeeded)
    END FUNCTION
END CLASS


SUBROUTINE: GetUserTier
INPUT: clientId (string)
OUTPUT: tier (string)

BEGIN
    // Check cache first
    cachedTier ← Cache.get("user:tier:" + clientId)

    IF cachedTier is not null THEN
        RETURN cachedTier
    END IF

    // Query database
    query ← "SELECT tier FROM users WHERE user_id = ?"
    result ← Database.execute(query, [clientId])

    IF result.rows.length EQUALS 0 THEN
        RETURN "free"  // Default tier
    END IF

    tier ← result.rows[0].tier OR "free"

    // Cache for 5 minutes
    Cache.set("user:tier:" + clientId, tier, ttl: 300)

    RETURN tier
END


FUNCTION GetDefaultConfig(): RateLimitConfig
    RETURN {
        free: {
            max_tokens: 100,
            refill_rate: 10,      // 10 requests per second
            window_size: 60       // 1 minute window
        },
        basic: {
            max_tokens: 500,
            refill_rate: 50,
            window_size: 60
        },
        premium: {
            max_tokens: 2000,
            refill_rate: 200,
            window_size: 60
        },
        enterprise: {
            max_tokens: 10000,
            refill_rate: 1000,
            window_size: 60
        }
    }
END
```

## 3. Authentication Middleware

### 3.1 OAuth Token Validation

```
ALGORITHM: Authenticator
DATA STRUCTURES:
    TokenCache: Cache for validated tokens
        Type: Redis cache
        TTL: Token expiration time

    TokenBlacklist: Revoked tokens
        Type: Redis set
        TTL: Original token expiration

CLASS Authenticator
    PROPERTIES:
        jwtSecret: string
        tokenCache: RedisClient
        allowedScopes: Map<method, array>

    CONSTRUCTOR(config: AuthConfig)
        this.jwtSecret ← config.jwtSecret OR process.env.JWT_SECRET
        this.tokenCache ← ConnectToRedis()
        this.allowedScopes ← LoadScopeConfiguration()
    END CONSTRUCTOR

    FUNCTION Authenticate(token: string, method: string): AuthResult
        // Step 1: Validate token format
        IF token is null OR token is empty THEN
            RETURN {
                valid: false,
                error: "No token provided",
                context: null
            }
        END IF

        // Step 2: Extract token from Bearer scheme
        IF token.startsWith("Bearer ") THEN
            token ← token.substring(7)
        END IF

        // Step 3: Check blacklist (revoked tokens)
        IF IsTokenBlacklisted(token) THEN
            RETURN {
                valid: false,
                error: "Token has been revoked",
                context: null
            }
        END IF

        // Step 4: Check cache
        cacheKey ← "auth:token:" + HashToken(token)
        cachedAuth ← tokenCache.get(cacheKey)

        IF cachedAuth is not null THEN
            authContext ← JSON_PARSE(cachedAuth)

            // Verify method permissions
            IF NOT HasMethodPermission(authContext, method) THEN
                RETURN {
                    valid: false,
                    error: "Insufficient permissions for method: " + method,
                    context: null
                }
            END IF

            RETURN {
                valid: true,
                error: null,
                context: authContext
            }
        END IF

        // Step 5: Validate JWT
        TRY:
            payload ← VerifyJWT(token, jwtSecret)

        CATCH JWTExpiredError:
            RETURN {
                valid: false,
                error: "Token expired",
                context: null
            }

        CATCH JWTInvalidError:
            RETURN {
                valid: false,
                error: "Invalid token",
                context: null
            }
        END TRY

        // Step 6: Build auth context
        authContext ← {
            userId: payload.sub,
            scopes: payload.scope.split(" "),
            tier: payload.tier OR "free",
            region: payload.region OR "US",
            expiresAt: payload.exp
        }

        // Step 7: Verify method permissions
        IF NOT HasMethodPermission(authContext, method) THEN
            RETURN {
                valid: false,
                error: "Insufficient permissions for method: " + method,
                context: null
            }
        END IF

        // Step 8: Cache auth context
        ttl ← payload.exp - GetCurrentTime()
        tokenCache.set(cacheKey, JSON_STRINGIFY(authContext), ttl)

        // Step 9: Return successful auth
        RETURN {
            valid: true,
            error: null,
            context: authContext
        }
    END FUNCTION

    FUNCTION HasMethodPermission(authContext: Object, method: string): boolean
        // Get required scopes for method
        requiredScopes ← allowedScopes.get(method)

        IF requiredScopes is null OR requiredScopes.length EQUALS 0 THEN
            // Method doesn't require specific scopes
            RETURN true
        END IF

        // Check if user has at least one required scope
        FOR EACH required IN requiredScopes DO
            IF required IN authContext.scopes THEN
                RETURN true
            END IF
        END FOR

        RETURN false
    END FUNCTION

    FUNCTION RevokeToken(token: string)
        // Add to blacklist
        tokenHash ← HashToken(token)
        key ← "auth:blacklist:" + tokenHash

        // Verify token to get expiration
        TRY:
            payload ← VerifyJWT(token, jwtSecret, ignoreExpiration: true)
            ttl ← payload.exp - GetCurrentTime()

            IF ttl > 0 THEN
                tokenCache.set(key, "1", ttl)
            END IF

        CATCH:
            // Invalid token, nothing to revoke
        END TRY

        // Remove from cache
        cacheKey ← "auth:token:" + tokenHash
        tokenCache.delete(cacheKey)
    END FUNCTION

    FUNCTION IsTokenBlacklisted(token: string): boolean
        tokenHash ← HashToken(token)
        key ← "auth:blacklist:" + tokenHash

        RETURN tokenCache.exists(key)
    END FUNCTION

    FUNCTION LoadScopeConfiguration(): Map
        // Map MCP methods to required OAuth scopes
        RETURN new Map([
            ["tools/call:semantic_search", ["content:read"]],
            ["tools/call:get_content_details", ["content:read"]],
            ["tools/call:get_recommendations", ["content:recommendations", "profile:read"]],
            ["tools/call:list_devices", ["devices:read"]],
            ["tools/call:get_device_status", ["devices:read"]],
            ["tools/call:initiate_playback", ["playback:control", "devices:write"]],
            ["tools/call:control_playback", ["playback:control"]]
        ])
    END FUNCTION

    FUNCTION Cleanup()
        // Close Redis connection
        tokenCache.quit()
    END FUNCTION
END CLASS


SUBROUTINE: VerifyJWT
INPUT: token (string), secret (string), options (object)
OUTPUT: payload (object)

BEGIN
    // Use standard JWT library (e.g., jsonwebtoken)
    // This is a simplified pseudocode representation

    // Decode header and payload
    parts ← token.split(".")

    IF parts.length NOT EQUALS 3 THEN
        THROW JWTInvalidError("Invalid token format")
    END IF

    header ← Base64Decode(parts[0])
    payload ← Base64Decode(parts[1])
    signature ← parts[2]

    // Verify signature
    expectedSignature ← HMAC_SHA256(parts[0] + "." + parts[1], secret)

    IF signature NOT EQUALS expectedSignature THEN
        THROW JWTInvalidError("Invalid signature")
    END IF

    // Parse payload
    claims ← JSON_PARSE(payload)

    // Check expiration (unless ignored)
    IF NOT options.ignoreExpiration THEN
        IF claims.exp < GetCurrentTime() THEN
            THROW JWTExpiredError("Token expired")
        END IF
    END IF

    RETURN claims
END


SUBROUTINE: HashToken
INPUT: token (string)
OUTPUT: hash (string)

BEGIN
    // Use SHA-256 for consistent hashing
    RETURN SHA256(token).substring(0, 32)
END
```

## 4. Complexity Analysis

### ARW Manifest Generation

**Time Complexity:**
- GetEnabledTools: O(t) where t = total tools
- BuildCapabilities: O(t)
- GenerateOAuthScopes: O(t * s) where s = scopes per tool
- **Total: O(t * s)** - typically < 10ms

**Space Complexity:**
- Manifest object: O(t) - proportional to tools
- Cache storage: O(c) where c = cached manifests
- **Typical: < 10KB per manifest**

### Rate Limiting

**Time Complexity:**
- CheckLimit: O(1) - Redis hash operations
- GetBucket: O(1)
- UpdateBucket: O(1)
- **Total: O(1)** - typically < 1ms

**Space Complexity:**
- Per-user bucket: O(1)
- Total buckets: O(u) where u = active users
- **Typical: < 100 bytes per bucket**

### Authentication

**Time Complexity:**
- Authenticate: O(1) - cached lookups
- VerifyJWT: O(1) - cryptographic operations
- HasMethodPermission: O(s) where s = scopes
- **Total: O(s)** - typically < 5ms

**Space Complexity:**
- Token cache: O(u * t) where u = users, t = tokens
- Blacklist: O(r) where r = revoked tokens
- **Typical: < 500 bytes per cached token**

### Optimization Strategies

1. **Manifest Caching**
   - Cache for 1 hour
   - Invalidate on tool configuration changes
   - Use CDN for manifest endpoint

2. **Rate Limiting**
   - Use Redis for distributed rate limiting
   - Implement sliding window for smoother limiting
   - Batch Redis operations where possible

3. **Authentication**
   - Cache validated tokens with TTL
   - Use connection pooling for Redis
   - Implement token refresh to reduce validation

4. **Security**
   - Rotate JWT secrets periodically
   - Implement IP-based rate limiting as fallback
   - Monitor for unusual authentication patterns
