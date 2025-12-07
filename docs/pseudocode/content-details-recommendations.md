# Content Details & Recommendations - Pseudocode Design

## 1. Get Content Details Tool

### 1.1 Main Algorithm

```
ALGORITHM: GetContentDetails
INPUT:
    entity_id (string) - Unique content identifier
    include_credits (boolean) - Include cast/crew (default: true)
    include_similar (boolean) - Include similar content (default: true)
    region (string) - User region for availability (default: "US")
OUTPUT:
    content (object) - Complete content details

DATA STRUCTURES:
    ContentCache: LRU cache for content details
        Type: LRU Cache
        Size: 5,000 entries
        TTL: 15 minutes

    CreditsCache: LRU cache for cast/crew data
        Type: LRU Cache
        Size: 10,000 entries
        TTL: 1 hour

BEGIN
    // Step 1: Validate input
    IF entity_id is null OR entity_id is empty THEN
        THROW ValidationError("entity_id is required")
    END IF

    // Step 2: Check cache
    cacheKey ← GenerateCacheKey(entity_id, include_credits, include_similar, region)
    cachedContent ← ContentCache.get(cacheKey)

    IF cachedContent is not null THEN
        LOG("Cache hit for content: " + entity_id)
        RETURN cachedContent
    END IF

    // Step 3: Fetch base content data
    content ← FetchContentById(entity_id)

    IF content is null THEN
        THROW NotFoundError("Content not found: " + entity_id)
    END IF

    // Step 4: Enrich with additional data (parallel fetches)
    enrichmentTasks ← []

    // Task 1: Availability data (always included)
    enrichmentTasks.append(
        ASYNC FetchAvailability(entity_id, region)
    )

    // Task 2: Credits (if requested)
    IF include_credits THEN
        enrichmentTasks.append(
            ASYNC FetchCredits(entity_id)
        )
    END IF

    // Task 3: Similar content (if requested)
    IF include_similar THEN
        enrichmentTasks.append(
            ASYNC FindSimilarContent(entity_id, limit: 10)
        )
    END IF

    // Task 4: User-specific data (ratings, watch history)
    enrichmentTasks.append(
        ASYNC FetchUserContentData(entity_id, authContext.userId)
    )

    // Wait for all tasks to complete
    [availability, credits, similar, userData] ← AWAIT_ALL(enrichmentTasks)

    // Step 5: Assemble complete content object
    enrichedContent ← {
        entity_id: content.entity_id,
        title: content.title,
        type: content.content_type,
        release_year: content.release_year,
        runtime: content.runtime,
        genres: content.genres,
        rating: content.rating,
        imdb_rating: content.imdb_rating,
        description: content.description,
        tagline: content.tagline,
        poster_url: content.poster_url,
        backdrop_url: content.backdrop_url,
        trailer_url: content.trailer_url,
        availability: availability,
        credits: credits,
        similar_content: similar,
        user_data: userData,
        metadata: {
            last_updated: content.updated_at,
            data_source: content.data_source
        }
    }

    // Step 6: Cache result
    ContentCache.set(cacheKey, enrichedContent)

    // Step 7: Track analytics
    TrackContentView(entity_id, authContext.userId)

    RETURN enrichedContent
END


SUBROUTINE: FetchContentById
INPUT: entity_id (string)
OUTPUT: content (object) or null

BEGIN
    // Query database with optimized index
    query ← "SELECT * FROM content WHERE entity_id = ?"
    result ← Database.execute(query, [entity_id])

    IF result.rows.length EQUALS 0 THEN
        RETURN null
    END IF

    RETURN result.rows[0]
END


SUBROUTINE: FetchAvailability
INPUT: entity_id (string), region (string)
OUTPUT: availability (array)

BEGIN
    // Query availability with region filter
    query ← `
        SELECT
            platform_id,
            platform_name,
            availability_type,
            price,
            quality,
            deep_link_url,
            expires_at
        FROM content_availability
        WHERE entity_id = ?
          AND region = ?
          AND (expires_at IS NULL OR expires_at > NOW())
        ORDER BY availability_type, price
    `

    results ← Database.execute(query, [entity_id, region])

    // Group by platform
    platformMap ← new Map()

    FOR EACH row IN results.rows DO
        platformId ← row.platform_id

        IF NOT platformMap.has(platformId) THEN
            platformMap.set(platformId, {
                platform_id: platformId,
                platform_name: row.platform_name,
                options: []
            })
        END IF

        platform ← platformMap.get(platformId)
        platform.options.append({
            type: row.availability_type,
            price: row.price,
            quality: row.quality,
            link: row.deep_link_url,
            expires_at: row.expires_at
        })
    END FOR

    RETURN Array.from(platformMap.values())
END


SUBROUTINE: FetchCredits
INPUT: entity_id (string)
OUTPUT: credits (object)

BEGIN
    // Check cache first
    cachedCredits ← CreditsCache.get(entity_id)
    IF cachedCredits is not null THEN
        RETURN cachedCredits
    END IF

    // Query cast and crew
    query ← `
        SELECT
            person_id,
            name,
            role_type,
            character_name,
            job_title,
            profile_image_url,
            order_index
        FROM content_credits
        WHERE entity_id = ?
        ORDER BY order_index
    `

    results ← Database.execute(query, [entity_id])

    // Separate cast and crew
    cast ← []
    crew ← []

    FOR EACH row IN results.rows DO
        person ← {
            person_id: row.person_id,
            name: row.name,
            profile_image: row.profile_image_url
        }

        IF row.role_type EQUALS "cast" THEN
            person.character ← row.character_name
            cast.append(person)
        ELSE
            person.job ← row.job_title
            crew.append(person)
        END IF
    END FOR

    credits ← {
        cast: cast,
        crew: crew
    }

    // Cache for 1 hour
    CreditsCache.set(entity_id, credits)

    RETURN credits
END


SUBROUTINE: FindSimilarContent
INPUT: entity_id (string), limit (integer)
OUTPUT: similar (array)

BEGIN
    // Get content for similarity comparison
    baseContent ← FetchContentById(entity_id)

    IF baseContent is null THEN
        RETURN []
    END IF

    // Method 1: Content-based similarity
    // Find content with matching genres and similar metadata
    query ← `
        SELECT
            c.entity_id,
            c.title,
            c.content_type,
            c.release_year,
            c.genres,
            c.imdb_rating,
            c.poster_url,
            CalculateSimilarity(c.genres, ?, c.release_year, ?) AS similarity_score
        FROM content c
        WHERE c.entity_id != ?
          AND c.content_type = ?
          AND c.genres && ?
        ORDER BY similarity_score DESC
        LIMIT ?
    `

    results ← Database.execute(query, [
        baseContent.genres,
        baseContent.release_year,
        entity_id,
        baseContent.content_type,
        baseContent.genres,
        limit
    ])

    similar ← []
    FOR EACH row IN results.rows DO
        similar.append({
            entity_id: row.entity_id,
            title: row.title,
            type: row.content_type,
            release_year: row.release_year,
            genres: row.genres,
            rating: row.imdb_rating,
            poster_url: row.poster_url,
            similarity_score: row.similarity_score
        })
    END FOR

    RETURN similar
END


SUBROUTINE: FetchUserContentData
INPUT: entity_id (string), user_id (string)
OUTPUT: userData (object)

BEGIN
    IF user_id is null THEN
        RETURN {
            watched: false,
            in_watchlist: false,
            user_rating: null
        }
    END IF

    // Query user interactions with this content
    query ← `
        SELECT
            watched,
            watch_progress,
            in_watchlist,
            user_rating,
            last_watched_at
        FROM user_content_interactions
        WHERE user_id = ? AND entity_id = ?
    `

    result ← Database.execute(query, [user_id, entity_id])

    IF result.rows.length EQUALS 0 THEN
        RETURN {
            watched: false,
            watch_progress: 0,
            in_watchlist: false,
            user_rating: null,
            last_watched_at: null
        }
    END IF

    row ← result.rows[0]

    RETURN {
        watched: row.watched,
        watch_progress: row.watch_progress,
        in_watchlist: row.in_watchlist,
        user_rating: row.user_rating,
        last_watched_at: row.last_watched_at
    }
END
```

## 2. Get Recommendations Tool

### 2.1 SONA Personalization Algorithm

```
ALGORITHM: GetRecommendations
INPUT:
    context (string) - Context/mood (e.g., "family movie night")
    preferences (object) - User preferences
    limit (integer) - Max results (default: 20)
OUTPUT:
    recommendations (array) - Personalized content recommendations

DATA STRUCTURES:
    UserProfile: User viewing history and preferences
        Type: Document store
        Fields: genres, platforms, watch_history, ratings

    RecommendationCache: Cache for user-specific recommendations
        Type: Cache with user-based partitioning
        TTL: 1 hour

CONSTANTS:
    AGE_RATINGS_ORDER = ["G", "PG", "PG-13", "R", "NC-17"]
    DEFAULT_RECOMMENDATION_POOL = 100

BEGIN
    // Step 1: Parse context and extract intent
    intent ← ParseRecommendationContext(context)

    // Step 2: Load user profile
    userProfile ← LoadUserProfile(authContext.userId)

    // Step 3: Merge preferences (context overrides profile)
    mergedPreferences ← MergePreferences(userProfile, preferences, intent)

    // Step 4: Apply age-appropriate filtering
    maxRating ← DetermineMaxRating(mergedPreferences.age_restriction)

    // Step 5: Build candidate pool
    candidates ← BuildCandidatePool(
        userProfile,
        mergedPreferences,
        maxRating,
        poolSize: DEFAULT_RECOMMENDATION_POOL
    )

    // Step 6: Score candidates using SONA algorithm
    scoredCandidates ← []
    FOR EACH candidate IN candidates DO
        score ← CalculateSONAScore(
            candidate,
            userProfile,
            mergedPreferences,
            intent
        )

        scoredCandidates.append({
            content: candidate,
            score: score
        })
    END FOR

    // Step 7: Sort by score (descending)
    scoredCandidates.sortByDescending(score)

    // Step 8: Apply diversity filter
    diverseRecommendations ← ApplyDiversityFilter(
        scoredCandidates,
        limit: limit
    )

    // Step 9: Enrich with availability
    enrichedRecommendations ← EnrichWithAvailability(
        diverseRecommendations,
        authContext
    )

    // Step 10: Filter by platform availability (if specified)
    IF mergedPreferences.platforms is not empty THEN
        enrichedRecommendations ← FilterByPlatforms(
            enrichedRecommendations,
            mergedPreferences.platforms
        )
    END IF

    // Step 11: Format results
    formattedResults ← FormatRecommendations(
        enrichedRecommendations,
        context
    )

    RETURN formattedResults
END


SUBROUTINE: ParseRecommendationContext
INPUT: context (string)
OUTPUT: intent (object)

BEGIN
    intent ← {
        occasion: null,
        mood: null,
        audience: null,
        genre_hints: [],
        time_of_day: null
    }

    // Occasion detection
    occasionPatterns ← [
        {pattern: /family (movie|night|time)/i, occasion: "family"},
        {pattern: /date night/i, occasion: "date"},
        {pattern: /kids|children/i, occasion: "kids"},
        {pattern: /party|friends/i, occasion: "social"},
        {pattern: /alone|myself/i, occasion: "solo"}
    ]

    FOR EACH pattern IN occasionPatterns DO
        IF context.match(pattern.pattern) THEN
            intent.occasion ← pattern.occasion
            BREAK
        END IF
    END FOR

    // Mood detection
    moodPatterns ← [
        {pattern: /relax|chill|unwind/i, mood: "relaxing"},
        {pattern: /exciting|thrilling|intense/i, mood: "exciting"},
        {pattern: /laugh|funny|comedy/i, mood: "lighthearted"},
        {pattern: /cry|emotional|touching/i, mood: "emotional"},
        {pattern: /think|thoughtful|cerebral/i, mood: "thoughtful"}
    ]

    FOR EACH pattern IN moodPatterns DO
        IF context.match(pattern.pattern) THEN
            intent.mood ← pattern.mood
            BREAK
        END IF
    END FOR

    // Genre hints from context
    genreKeywords ← ExtractGenreKeywords(context)
    intent.genre_hints ← genreKeywords

    RETURN intent
END


SUBROUTINE: LoadUserProfile
INPUT: user_id (string)
OUTPUT: profile (object)

BEGIN
    IF user_id is null THEN
        RETURN GetDefaultProfile()
    END IF

    // Fetch user profile with aggregated viewing data
    query ← `
        SELECT
            preferred_genres,
            preferred_platforms,
            avg_rating,
            total_watches,
            recent_genres,
            recent_platforms
        FROM user_profiles
        WHERE user_id = ?
    `

    result ← Database.execute(query, [user_id])

    IF result.rows.length EQUALS 0 THEN
        RETURN GetDefaultProfile()
    END IF

    profile ← result.rows[0]

    // Fetch recent watch history
    historyQuery ← `
        SELECT
            c.entity_id,
            c.genres,
            c.content_type,
            u.user_rating,
            u.last_watched_at
        FROM user_content_interactions u
        JOIN content c ON c.entity_id = u.entity_id
        WHERE u.user_id = ?
          AND u.watched = true
        ORDER BY u.last_watched_at DESC
        LIMIT 50
    `

    history ← Database.execute(historyQuery, [user_id])

    profile.watch_history ← history.rows

    RETURN profile
END


SUBROUTINE: MergePreferences
INPUT: profile (object), preferences (object), intent (object)
OUTPUT: merged (object)

BEGIN
    merged ← {
        genres: [],
        platforms: [],
        age_restriction: null,
        exclude_watched: true,
        min_rating: null
    }

    // Genre preferences (priority: intent > preferences > profile)
    IF intent.genre_hints.length > 0 THEN
        merged.genres ← intent.genre_hints
    ELSE IF preferences.genres is defined THEN
        merged.genres ← preferences.genres
    ELSE IF profile.preferred_genres is defined THEN
        merged.genres ← profile.preferred_genres
    END IF

    // Platform preferences
    IF preferences.platforms is defined THEN
        merged.platforms ← preferences.platforms
    ELSE IF profile.preferred_platforms is defined THEN
        merged.platforms ← profile.preferred_platforms
    END IF

    // Age restriction (most restrictive wins)
    IF preferences.age_restriction is defined THEN
        merged.age_restriction ← preferences.age_restriction
    ELSE IF intent.occasion EQUALS "family" OR intent.occasion EQUALS "kids" THEN
        merged.age_restriction ← "PG"
    END IF

    // Minimum rating filter
    IF preferences.min_rating is defined THEN
        merged.min_rating ← preferences.min_rating
    ELSE
        merged.min_rating ← 6.0  // Default minimum IMDb rating
    END IF

    // Exclude watched content
    IF preferences.exclude_watched is defined THEN
        merged.exclude_watched ← preferences.exclude_watched
    END IF

    RETURN merged
END


SUBROUTINE: CalculateSONAScore
INPUT: content (object), profile (object), preferences (object), intent (object)
OUTPUT: score (float)

BEGIN
    score ← 0.0

    // S - Semantic Similarity (weight: 0.25)
    genreMatch ← CalculateGenreMatch(content.genres, preferences.genres)
    moodMatch ← CalculateMoodMatch(content, intent.mood)
    semanticScore ← (genreMatch * 0.7) + (moodMatch * 0.3)
    score ← score + (semanticScore * 0.25)

    // O - Occasion Appropriateness (weight: 0.20)
    occasionScore ← CalculateOccasionFit(content, intent.occasion)
    score ← score + (occasionScore * 0.20)

    // N - Neural Personalization (weight: 0.35)
    // Based on user's watch history and ratings
    historyScore ← CalculateHistoryAffinity(content, profile.watch_history)
    ratingScore ← PredictUserRating(content, profile)
    neuralScore ← (historyScore * 0.4) + (ratingScore * 0.6)
    score ← score + (neuralScore * 0.35)

    // A - Availability & Accessibility (weight: 0.20)
    platformScore ← CalculatePlatformScore(content, preferences.platforms)
    qualityScore ← NormalizeQuality(content.max_quality)
    availabilityScore ← (platformScore * 0.7) + (qualityScore * 0.3)
    score ← score + (availabilityScore * 0.20)

    // Boost for highly-rated content
    IF content.imdb_rating >= 8.0 THEN
        score ← score * 1.1
    END IF

    // Penalty for very old content (unless it's a classic)
    ageYears ← CurrentYear - content.release_year
    IF ageYears > 10 AND content.imdb_rating < 7.5 THEN
        score ← score * 0.9
    END IF

    // Diversity penalty (avoid recommending same genre repeatedly)
    diversityPenalty ← CalculateDiversityPenalty(content, profile.recent_genres)
    score ← score * (1 - diversityPenalty)

    RETURN score
END


SUBROUTINE: ApplyDiversityFilter
INPUT: scored (array), limit (integer)
OUTPUT: diverse (array)

BEGIN
    diverse ← []
    genreCounts ← new Map()
    typeCounts ← new Map()

    FOR EACH item IN scored DO
        IF diverse.length >= limit THEN
            BREAK
        END IF

        content ← item.content

        // Track genre distribution
        primaryGenre ← content.genres[0]
        genreCount ← genreCounts.get(primaryGenre) OR 0

        // Track type distribution (movie vs TV)
        typeCount ← typeCounts.get(content.content_type) OR 0

        // Enforce diversity rules:
        // - No more than 40% of same genre
        // - No more than 70% of same type
        genreLimit ← limit * 0.4
        typeLimit ← limit * 0.7

        IF genreCount < genreLimit AND typeCount < typeLimit THEN
            diverse.append(item)
            genreCounts.set(primaryGenre, genreCount + 1)
            typeCounts.set(content.content_type, typeCount + 1)
        END IF
    END FOR

    // If we didn't get enough diverse results, fill remaining with top-scored
    IF diverse.length < limit THEN
        FOR EACH item IN scored DO
            IF diverse.length >= limit THEN
                BREAK
            END IF

            IF NOT diverse.includes(item) THEN
                diverse.append(item)
            END IF
        END FOR
    END IF

    RETURN diverse
END
```

## 3. Complexity Analysis

### Get Content Details

**Time Complexity:**
- FetchContentById: O(log n) - indexed lookup
- FetchAvailability: O(a) - a = availability records
- FetchCredits: O(c) - c = credits count
- FindSimilarContent: O(log n + k) - k = result limit
- **Total: O(log n + a + c + k)** - typically < 50ms

**Space Complexity:**
- Content object: O(1)
- Availability array: O(a)
- Credits: O(c)
- Similar content: O(k)
- **Total: O(a + c + k)** - typically < 100KB

### Get Recommendations

**Time Complexity:**
- LoadUserProfile: O(log n + h) - h = history size
- BuildCandidatePool: O(p) - p = pool size (100)
- CalculateSONAScore (per candidate): O(g + m) - g = genres, m = history
- Total scoring: O(p * (g + m))
- Sorting: O(p log p)
- **Total: O(p * (g + m) + p log p)** - typically < 200ms

**Space Complexity:**
- User profile: O(h)
- Candidate pool: O(p)
- Scored results: O(p)
- **Total: O(h + p)** - typically < 500KB

### Optimization Strategies

1. **Caching**
   - Cache content details for 15 minutes
   - Cache user profiles for 5 minutes
   - Cache recommendations for 1 hour with user partitioning

2. **Database Optimization**
   - Compound index on (entity_id, region) for availability
   - Index on (user_id, entity_id) for interactions
   - Denormalize frequently accessed fields

3. **Parallel Processing**
   - Fetch availability, credits, and similar content in parallel
   - Batch database queries where possible

4. **Algorithm Optimization**
   - Limit candidate pool to reasonable size (100-200)
   - Early termination for low-scoring candidates
   - Pre-compute user affinities during off-peak hours
