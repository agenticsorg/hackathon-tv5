# Media Gateway CLI - Data Structures Specification

## SPARC Pseudocode Phase - Data Structure Design

**Document Version**: 1.0.0
**Last Updated**: 2025-12-06

---

## 1. CORE DATA STRUCTURES

### 1.1 ParsedCommand

```
STRUCTURE ParsedCommand:
    FIELDS:
        command: string                    // Main command name
        options: Map<string, any>          // Key-value options
        flags: Set<string>                 // Boolean flags
        positionalArgs: string[]           // Unnamed arguments

    OPERATIONS:
        hasOption(key: string): boolean    // O(1)
        getOption(key: string): any        // O(1)
        hasFlag(flag: string): boolean     // O(1)
        getArg(index: integer): string     // O(1)

    INVARIANTS:
        - command IS NOT null
        - options keys are lowercase
        - flags are deduplicated

    EXAMPLE:
        Input: search "matrix" --type movie --year 1999 -i
        Result:
            command = "search"
            options = { type: "movie", year: 1999 }
            flags = { "interactive" }
            positionalArgs = [ "matrix" ]
```

**Memory Layout**: ~200 bytes + dynamic size based on arguments
**Access Pattern**: Read-heavy, write-once during parsing

---

### 1.2 MediaItem

```
STRUCTURE MediaItem:
    FIELDS:
        id: string                         // Unique identifier
        title: string                      // Display title
        type: MediaType                    // Enum: movie, tv, music
        year: integer                      // Release year
        rating: float                      // 0.0 - 10.0
        genres: string[]                   // Genre tags
        platforms: string[]                // Available platforms
        duration: integer                  // Minutes
        description: string                // Synopsis
        posterUrl: string                  // Image URL
        cast: Person[]                     // Cast members
        directors: Person[]                // Directors
        metadata: Map<string, any>         // Additional data

    OPERATIONS:
        isAvailableOn(platform: string): boolean  // O(p) where p = platforms
        hasGenre(genre: string): boolean          // O(g) where g = genres
        getDisplayTitle(): string                 // O(1)
        toJSON(): object                          // O(1)

    INVARIANTS:
        - id is unique
        - rating is in range [0, 10]
        - year is in range [1888, current_year + 2]
        - type is valid MediaType
        - platforms is not empty

    INDICES:
        Primary: id
        Secondary: (type, year), (platform), (genre)
```

**Memory Size**: ~1-2 KB per item
**Optimization**: Lazy load metadata and detailed fields

---

### 1.3 SearchOptions

```
STRUCTURE SearchOptions:
    FIELDS:
        type: MediaType[]                  // Filter by type
        genre: string[]                    // Filter by genre
        platform: string[]                 // Filter by platform
        year: Range<integer>               // Year range filter
        rating: Range<float>               // Rating filter
        limit: integer                     // Max results (default: 20)
        offset: integer                    // Pagination offset
        sortBy: SortField                  // Sort field
        sortOrder: SortOrder               // asc or desc
        interactive: boolean               // Interactive mode
        format: OutputFormat               // Output format

    DEFAULTS:
        limit = 20
        offset = 0
        sortBy = "relevance"
        sortOrder = "desc"
        interactive = false
        format = "table"

    VALIDATION:
        - limit: 1 <= limit <= 100
        - offset: offset >= 0
        - year.min <= year.max
        - rating.min <= rating.max

    BUILDER PATTERN:
        SearchOptions.builder()
            .withType("movie")
            .withGenre("action", "sci-fi")
            .withYearRange(2000, 2024)
            .build()
```

---

### 1.4 SearchResult

```
STRUCTURE SearchResult:
    FIELDS:
        items: MediaItem[]                 // Result items
        total: integer                     // Total matches
        hasMore: boolean                   // More results available
        metadata: ResultMetadata           // Query metadata

    NESTED STRUCTURE ResultMetadata:
        query: string                      // Original query
        filters: FilterSpec                // Applied filters
        executionTime: integer             // Milliseconds
        source: string                     // "cache" or "api"
        timestamp: timestamp               // Result timestamp

    OPERATIONS:
        isEmpty(): boolean                 // O(1)
        getPage(page: integer, size: integer): MediaItem[]  // O(size)
        getTotalPages(pageSize: integer): integer  // O(1)

    INVARIANTS:
        - items.length <= total
        - hasMore = (items.length < total)
```

**Cache Key Generation**:
```
FUNCTION GenerateCacheKey(query: string, options: SearchOptions): string
    components = [
        Normalize(query),
        SortArray(options.type).join(","),
        SortArray(options.genre).join(","),
        SortArray(options.platform).join(","),
        options.year.toString(),
        options.rating.toString(),
        options.sortBy,
        options.sortOrder
    ]
    RETURN Hash(components.join("|"))
```

---

## 2. RECOMMENDATION STRUCTURES

### 2.1 UserProfile

```
STRUCTURE UserProfile:
    FIELDS:
        context: string                    // "date night", "solo", etc.
        mood: string                       // "action", "relaxing", etc.
        duration: integer                  // Preferred duration (minutes)
        excludeGenres: string[]            // Genres to exclude
        platform: string[]                 // Available platforms
        watchedIds: string[]               // Previously watched

    CONTEXTUAL MAPPING:
        context → mood mapping:
            "date night"    → "romantic", "entertaining"
            "family time"   → "family-friendly", "fun"
            "solo"          → "intellectual", "intense"
            "party"         → "entertaining", "upbeat"

    MOOD ATTRIBUTES:
        Each mood maps to:
            - Genre weights: Map<genre, weight>
            - Tone preferences: "light", "dark", "balanced"
            - Pacing: "slow", "medium", "fast"
```

---

### 2.2 RecommendationItem

```
STRUCTURE RecommendationItem:
    FIELDS:
        media: MediaItem                   // The recommended media
        score: float                       // Final score (0-100)
        baseScore: float                   // Engine base score
        rationale: string                  // Explanation text
        matchFactors: MatchFactor[]        // Score breakdown

    NESTED STRUCTURE MatchFactor:
        factor: string                     // Factor name
        weight: float                      // Weight in final score
        value: float                       // Factor value
        contribution: float                // Weighted contribution

    EXAMPLE:
        {
            media: { title: "Inception", ... },
            score: 87.5,
            baseScore: 75.0,
            rationale: "Great match for action-loving sci-fi fans",
            matchFactors: [
                { factor: "mood", weight: 0.2, value: 90, contribution: 18 },
                { factor: "context", weight: 0.15, value: 85, contribution: 12.75 },
                { factor: "duration", weight: 0.1, value: 95, contribution: 9.5 },
                { factor: "platform", weight: 0.05, value: 100, contribution: 5 },
                { factor: "history", weight: 0.25, value: 82, contribution: 20.5 }
            ]
        }

    OPERATIONS:
        getTopFactors(n: integer): MatchFactor[]  // O(k log k)
        formatExplanation(): string               // O(1)
```

---

## 3. WATCHLIST STRUCTURES

### 3.1 WatchlistItem

```
STRUCTURE WatchlistItem:
    FIELDS:
        mediaId: string                    // Reference to MediaItem
        addedAt: timestamp                 // When added
        notes: string                      // User notes
        priority: Priority                 // Enum: low, medium, high
        watched: boolean                   // Watched status
        watchedAt: timestamp | null        // When watched
        rating: float | null               // User rating (0-10)

    OPERATIONS:
        markWatched(rating: float): void   // O(1)
        updateNotes(notes: string): void   // O(1)
        setPriority(p: Priority): void     // O(1)

    INDICES:
        Primary: mediaId
        Secondary: addedAt, priority, watched
```

---

### 3.2 WatchlistDiff

```
STRUCTURE WatchlistDiff:
    FIELDS:
        toAdd: WatchlistItem[]             // Items to add to cloud
        toRemove: string[]                 // MediaIds to remove locally
        toUpdate: WatchlistItem[]          // Items with conflicts
        conflicts: ConflictRecord[]        // Detailed conflicts

    NESTED STRUCTURE ConflictRecord:
        mediaId: string
        localVersion: WatchlistItem
        cloudVersion: WatchlistItem
        resolution: ResolutionStrategy
        resolvedVersion: WatchlistItem

    RESOLUTION STRATEGIES:
        - SERVER_WINS: Use cloud version
        - CLIENT_WINS: Use local version
        - LATEST_TIMESTAMP: Use most recent
        - MERGE: Combine both (for notes)

    ALGORITHM: CalculateWatchlistDiff
        INPUT: local (WatchlistItem[]), cloud (WatchlistItem[])
        OUTPUT: WatchlistDiff

        BEGIN
            localMap ← Map(local, item => item.mediaId)
            cloudMap ← Map(cloud, item => item.mediaId)

            toAdd ← []
            toRemove ← []
            toUpdate ← []
            conflicts ← []

            // Items in local but not in cloud → add to cloud
            FOR EACH item IN local DO
                IF NOT cloudMap.has(item.mediaId) THEN
                    toAdd.append(item)
                END IF
            END FOR

            // Items in cloud but not in local → remove from local
            FOR EACH item IN cloud DO
                IF NOT localMap.has(item.mediaId) THEN
                    toRemove.append(item.mediaId)
                END IF
            END FOR

            // Items in both → check for conflicts
            FOR EACH mediaId IN localMap.keys() DO
                IF cloudMap.has(mediaId) THEN
                    localItem ← localMap.get(mediaId)
                    cloudItem ← cloudMap.get(mediaId)

                    IF HasConflict(localItem, cloudItem) THEN
                        conflict ← ResolveConflict(localItem, cloudItem)
                        conflicts.append(conflict)
                        toUpdate.append(conflict.resolvedVersion)
                    END IF
                END IF
            END FOR

            RETURN {
                toAdd: toAdd,
                toRemove: toRemove,
                toUpdate: toUpdate,
                conflicts: conflicts
            }
        END

        COMPLEXITY:
            Time: O(n + m) where n = local size, m = cloud size
            Space: O(n + m) for maps
```

---

## 4. DEVICE STRUCTURES

### 4.1 Device

```
STRUCTURE Device:
    FIELDS:
        id: string                         // Unique device ID
        name: string                       // User-friendly name
        type: DeviceType                   // Enum
        platform: string                   // "roku", "fire_tv", etc.
        lastSeen: timestamp                // Last activity
        ipAddress: string                  // IP address
        capabilities: string[]             // Supported platforms
        metadata: DeviceMetadata           // Additional info

    ENUM DeviceType:
        TV = "tv"
        MOBILE = "mobile"
        BROWSER = "browser"
        STREAMING_DEVICE = "streaming_device"

    NESTED STRUCTURE DeviceMetadata:
        model: string
        manufacturer: string
        osVersion: string
        appVersion: string
        resolution: string                 // "1920x1080", "3840x2160"
        audioCapabilities: string[]        // "stereo", "5.1", "dolby_atmos"

    COMPUTED FIELDS:
        isOnline(): boolean
            RETURN (CurrentTime() - lastSeen) < 5_MINUTES

        supportsHD(): boolean
            RETURN metadata.resolution IN ["1920x1080", "3840x2160"]

        supports4K(): boolean
            RETURN metadata.resolution = "3840x2160"

    VALIDATION:
        - name: 2-50 characters
        - ipAddress: valid IPv4 or IPv6
        - capabilities: non-empty array
```

---

### 4.2 DeepLink

```
STRUCTURE DeepLink:
    FIELDS:
        url: string                        // Full URL
        platform: string                   // Platform identifier
        protocol: string                   // URL scheme
        fallbackUrl: string                // Web fallback
        params: Map<string, string>        // Query parameters

    PLATFORM PROTOCOLS:
        netflix:      "nflx://"
        hulu:         "hulu://"
        disney_plus:  "disneyplus://"
        amazon_prime: "aiv://"
        youtube:      "vnd.youtube://"
        spotify:      "spotify://"

    OPERATIONS:
        toNativeUrl(): string              // Protocol-based URL
        toWebUrl(): string                 // HTTPS URL
        addParam(key: string, value: string): void
        build(): string

    BUILDER:
        DeepLink.forPlatform("netflix")
            .withMediaId("80057281")
            .withStartTime(120)
            .withQuality("4k")
            .build()

        Result: "nflx://www.netflix.com/watch/80057281?t=120&quality=4k"
```

---

## 5. AUTHENTICATION STRUCTURES

### 5.1 AuthTokens

```
STRUCTURE AuthTokens:
    FIELDS:
        accessToken: string                // JWT access token
        refreshToken: string               // JWT refresh token
        tokenType: string                  // "Bearer"
        expiresAt: timestamp               // Expiration time
        scope: string[]                    // Granted scopes

    OPERATIONS:
        isExpired(): boolean
            RETURN CurrentTime() >= expiresAt

        needsRefresh(): boolean
            RETURN CurrentTime() >= (expiresAt - 5_MINUTES)

        getAuthHeader(): string
            RETURN tokenType + " " + accessToken

    SECURITY:
        - Stored in OS keychain
        - Never logged or displayed
        - Automatically cleared on logout
```

---

### 5.2 User

```
STRUCTURE User:
    FIELDS:
        id: string                         // Unique user ID
        email: string                      // Email address
        name: string                       // Display name
        avatar: string                     // Avatar URL
        preferences: UserPreferences       // User settings
        subscription: SubscriptionInfo     // Account info

    NESTED STRUCTURE UserPreferences:
        defaultPlatforms: string[]
        favoriteGenres: string[]
        language: string
        region: string
        notifications: boolean
        autoSync: boolean

    NESTED STRUCTURE SubscriptionInfo:
        tier: "free" | "premium" | "enterprise"
        expiresAt: timestamp | null
        features: string[]
```

---

## 6. MCP STRUCTURES

### 6.1 MCPRequest

```
STRUCTURE MCPRequest:
    FIELDS:
        jsonrpc: "2.0"                     // Protocol version
        id: string | integer               // Request ID
        method: string                     // Method name
        params: object | null              // Method parameters

    VALIDATION:
        - jsonrpc MUST be "2.0"
        - id MUST be unique per session
        - method MUST be non-empty string
        - params MUST be object or null

    EXAMPLE:
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "search",
            "params": {
                "query": "inception",
                "options": {
                    "type": ["movie"],
                    "limit": 10
                }
            }
        }
```

---

### 6.2 MCPResponse

```
STRUCTURE MCPResponse:
    FIELDS:
        jsonrpc: "2.0"                     // Protocol version
        id: string | integer               // Matches request ID
        result: object | null              // Success result
        error: MCPError | null             // Error object

    NESTED STRUCTURE MCPError:
        code: integer                      // Error code
        message: string                    // Error message
        data: any | null                   // Additional data

    ERROR CODES:
        -32700: Parse error
        -32600: Invalid request
        -32601: Method not found
        -32602: Invalid params
        -32603: Internal error

    INVARIANT:
        - Exactly one of result or error MUST be present
        - Never both or neither
```

---

## 7. CACHING STRUCTURES

### 7.1 CacheEntry

```
STRUCTURE CacheEntry<T>:
    FIELDS:
        key: string                        // Cache key
        value: T                           // Cached data
        createdAt: timestamp               // Creation time
        expiresAt: timestamp               // Expiration time
        hits: integer                      // Access count
        lastAccessAt: timestamp            // Last access

    OPERATIONS:
        isExpired(): boolean
            RETURN CurrentTime() >= expiresAt

        isValid(): boolean
            RETURN NOT isExpired()

        touch(): void
            lastAccessAt ← CurrentTime()
            hits ← hits + 1

    TTL CALCULATION:
        ttl = expiresAt - createdAt
```

---

### 7.2 LRUCache

```
STRUCTURE LRUCache<K, V>:
    FIELDS:
        capacity: integer                  // Max entries
        cache: Map<K, CacheEntry<V>>       // Storage
        accessOrder: DoublyLinkedList<K>   // LRU ordering

    OPERATIONS:
        get(key: K): V | null
            TIME: O(1)
            BEGIN
                IF NOT cache.has(key) THEN
                    RETURN null
                END IF

                entry ← cache.get(key)

                IF entry.isExpired() THEN
                    evict(key)
                    RETURN null
                END IF

                // Move to front (most recently used)
                accessOrder.moveToFront(key)
                entry.touch()

                RETURN entry.value
            END

        set(key: K, value: V, ttl: integer): void
            TIME: O(1)
            BEGIN
                // Evict if at capacity
                IF cache.size >= capacity AND NOT cache.has(key) THEN
                    lruKey ← accessOrder.back()
                    evict(lruKey)
                END IF

                entry ← {
                    key: key,
                    value: value,
                    createdAt: CurrentTime(),
                    expiresAt: CurrentTime() + ttl,
                    hits: 0,
                    lastAccessAt: CurrentTime()
                }

                cache.set(key, entry)
                accessOrder.addFront(key)
            END

        evict(key: K): void
            TIME: O(1)
            BEGIN
                cache.delete(key)
                accessOrder.remove(key)
            END

        clear(): void
            TIME: O(1)
            BEGIN
                cache.clear()
                accessOrder.clear()
            END

    INVARIANTS:
        - cache.size <= capacity
        - accessOrder.size = cache.size
        - Front of accessOrder is most recently used
```

**Memory Overhead**: ~100 bytes per entry + data size

---

## 8. OUTPUT FORMATTING STRUCTURES

### 8.1 TableFormat

```
STRUCTURE TableFormat:
    FIELDS:
        columns: Column[]                  // Column definitions
        style: TableStyle                  // Border style
        maxRows: integer                   // Row limit
        showHeaders: boolean               // Header row

    NESTED STRUCTURE Column:
        name: string                       // Column name
        field: string                      // Data field
        width: integer                     // Character width
        align: "left" | "right" | "center"
        formatter: (value: any) => string

    NESTED STRUCTURE TableStyle:
        topLeft: string
        topRight: string
        bottomLeft: string
        bottomRight: string
        horizontal: string
        vertical: string
        cross: string

    PREDEFINED STYLES:
        ASCII:
            { topLeft: "+", horizontal: "-", vertical: "|", ... }

        ROUNDED:
            { topLeft: "╭", horizontal: "─", vertical: "│", ... }

        MINIMAL:
            { topLeft: "", horizontal: "─", vertical: " ", ... }
```

---

### 8.2 ProgressIndicator

```
STRUCTURE Spinner:
    FIELDS:
        frames: string[]                   // Animation frames
        interval: integer                  // Ms between frames
        message: string                    // Display message
        currentFrame: integer              // Current frame index
        isRunning: boolean                 // Active state
        timerId: Timer                     // Interval timer

    DEFAULT FRAMES:
        ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]

    OPERATIONS:
        start(): void
        stop(): void
        updateMessage(msg: string): void

STRUCTURE ProgressBar:
    FIELDS:
        current: integer                   // Current value
        total: integer                     // Total value
        width: integer                     // Character width
        showPercentage: boolean
        showCount: boolean

    OPERATIONS:
        update(current: integer): void
        render(): string
        complete(): void
```

---

## 9. CONFIGURATION STRUCTURES

### 9.1 AppConfig

```
STRUCTURE AppConfig:
    FIELDS:
        version: string                    // Config version
        user: User | null                  // Current user
        apiEndpoint: string                // API base URL
        logLevel: LogLevel                 // Logging level
        colorMode: ColorMode               // Color output
        autoSync: boolean                  // Auto watchlist sync
        cacheEnabled: boolean              // Enable caching
        cacheTTL: integer                  // Cache TTL (seconds)
        maxCacheSize: integer              // Max cache entries
        defaultPageSize: integer           // Pagination size
        timeout: integer                   // Request timeout (ms)

    DEFAULTS:
        apiEndpoint = "https://api.media-gateway.com"
        logLevel = "info"
        colorMode = "auto"
        autoSync = true
        cacheEnabled = true
        cacheTTL = 300  // 5 minutes
        maxCacheSize = 1000
        defaultPageSize = 20
        timeout = 30000  // 30 seconds

    PERSISTENCE:
        - Stored in ~/.media-gateway/config.json
        - Encrypted sensitive fields
        - Validated on load
```

---

## 10. FILTER STRUCTURES

### 10.1 FilterSpec

```
STRUCTURE FilterSpec:
    FIELDS:
        filters: Filter[]                  // List of filters

    NESTED STRUCTURE Filter:
        field: string                      // Field name
        operator: Operator                 // Comparison operator
        value: any                         // Filter value

    ENUM Operator:
        EQ = "="                           // Equal
        NE = "!="                          // Not equal
        GT = ">"                           // Greater than
        GTE = ">="                         // Greater than or equal
        LT = "<"                           // Less than
        LTE = "<="                         // Less than or equal
        IN = "IN"                          // In array
        NOT_IN = "NOT IN"                  // Not in array
        CONTAINS = "CONTAINS"              // Contains substring
        STARTS_WITH = "STARTS_WITH"        // Starts with
        ENDS_WITH = "ENDS_WITH"            // Ends with

    OPERATIONS:
        add(filter: Filter): void          // O(1)
        remove(field: string): void        // O(n)
        has(field: string): boolean        // O(n)
        toQueryString(): string            // O(n)

    EXAMPLE:
        filters = [
            { field: "type", operator: "IN", value: ["movie", "tv"] },
            { field: "year", operator: ">=", value: 2000 },
            { field: "year", operator: "<=", value: 2024 },
            { field: "rating", operator: ">=", value: 7.0 },
            { field: "genre", operator: "CONTAINS", value: "action" }
        ]
```

---

## MEMORY USAGE ESTIMATES

| Structure | Typical Size | Max Size | Notes |
|-----------|-------------|----------|-------|
| ParsedCommand | 200 B - 2 KB | 10 KB | Depends on args |
| MediaItem | 1-2 KB | 5 KB | With full metadata |
| SearchResult | 20-200 KB | 2 MB | 20-100 items |
| WatchlistItem | 500 B | 2 KB | With notes |
| Device | 500 B | 1 KB | Full metadata |
| CacheEntry | 100 B + data | Variable | Overhead only |
| LRUCache | 100 MB | 1 GB | 1000 entries |
| User | 500 B | 2 KB | With preferences |

---

## INDEX STRATEGIES

### Primary Indices (Unique)
- MediaItem: `id`
- Device: `id`
- User: `id`
- WatchlistItem: `mediaId`

### Secondary Indices (Non-unique)
- MediaItem: `(type, year)`, `platform`, `genre`
- WatchlistItem: `addedAt`, `priority`, `watched`
- Device: `lastSeen`, `type`
- CacheEntry: `expiresAt` (for cleanup)

### Composite Indices
- Search: `(query_hash, filters_hash)`
- Recommendations: `(userId, context, mood)`

---

## CONCLUSION

These data structures provide:

1. **Efficiency**: Optimized for common access patterns
2. **Scalability**: Bounded memory usage with limits
3. **Flexibility**: Extensible through metadata fields
4. **Type Safety**: Clear contracts and validations
5. **Maintainability**: Well-documented invariants

Ready for implementation in any strongly-typed language (TypeScript, Rust, Go, etc.).
