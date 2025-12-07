# Search and Discovery Engine - Pseudocode Design

## System Overview

The Search and Discovery Engine provides intelligent media search with natural language understanding, multi-strategy querying, vector similarity, graph traversal, and personalized ranking.

**Latency Budget**: <500ms total (P95)
**Target Accuracy**: >85% intent understanding
**Scalability**: 1000+ concurrent queries

---

## 1. Natural Language Intent Parser

### 1.1 Main Algorithm

```
ALGORITHM: ParseNaturalLanguageIntent
INPUT: query (string), userId (UUID), cacheEnabled (boolean)
OUTPUT: ParsedIntent {mood, themes, references, fallbackQuery, confidence}

CONSTANTS:
    CACHE_TTL = 600 seconds (10 minutes)
    CACHE_NAMESPACE = "intent:parse:"
    GPT_MODEL = "gpt-4o-mini"
    GPT_TIMEOUT = 2000 ms
    CONFIDENCE_THRESHOLD = 0.7

BEGIN
    // Phase 1: Cache lookup (Target: <5ms)
    IF cacheEnabled THEN
        cacheKey ← CACHE_NAMESPACE + HASH(query)
        cachedIntent ← Cache.get(cacheKey)

        IF cachedIntent IS NOT NULL THEN
            RETURN cachedIntent
        END IF
    END IF

    // Phase 2: GPT-4o-mini parsing (Target: <1500ms)
    prompt ← BuildIntentPrompt(query)

    TRY
        response ← GPTClient.complete({
            model: GPT_MODEL,
            messages: [
                {role: "system", content: INTENT_PARSER_SYSTEM_PROMPT},
                {role: "user", content: prompt}
            ],
            timeout: GPT_TIMEOUT,
            temperature: 0.3,
            response_format: {type: "json_object"}
        })

        parsedIntent ← ParseGPTResponse(response)

        // Validate structured output
        IF NOT ValidateIntentStructure(parsedIntent) THEN
            THROW ValidationError("Invalid GPT response structure")
        END IF

    CATCH GPTTimeoutError, ValidationError, GPTError AS e
        // Fallback to basic parsing
        Log.warn("GPT parsing failed, using fallback", {error: e.message})
        parsedIntent ← FallbackBasicParse(query)
    END TRY

    // Phase 3: Cache storage
    IF cacheEnabled AND parsedIntent.confidence >= CONFIDENCE_THRESHOLD THEN
        Cache.set(cacheKey, parsedIntent, CACHE_TTL)
    END IF

    RETURN parsedIntent
END

SUBROUTINE: BuildIntentPrompt
INPUT: query (string)
OUTPUT: prompt (string)

BEGIN
    prompt ← """
    Analyze this media search query and extract structured information:

    Query: "{query}"

    Extract:
    1. Mood/Vibes: emotional tone (e.g., "dark", "uplifting", "tense")
    2. Themes: main subjects (e.g., "heist", "romance", "sci-fi")
    3. References: "similar to X" or "like Y" mentions
    4. Filters: platform, genre, year constraints
    5. Confidence: 0.0-1.0 score for extraction quality

    Return JSON:
    {
      "mood": ["mood1", "mood2"],
      "themes": ["theme1", "theme2"],
      "references": ["title1", "title2"],
      "filters": {
        "genre": ["genre1"],
        "platform": ["platform1"],
        "yearRange": {"min": 2020, "max": 2024}
      },
      "fallbackQuery": "simplified query string",
      "confidence": 0.85
    }
    """

    RETURN prompt
END

SUBROUTINE: ParseGPTResponse
INPUT: response (GPTResponse)
OUTPUT: ParsedIntent

BEGIN
    json ← JSON.parse(response.choices[0].message.content)

    intent ← {
        mood: json.mood OR [],
        themes: json.themes OR [],
        references: json.references OR [],
        filters: {
            genre: json.filters?.genre OR [],
            platform: json.filters?.platform OR [],
            yearRange: json.filters?.yearRange OR NULL
        },
        fallbackQuery: json.fallbackQuery OR "",
        confidence: json.confidence OR 0.0
    }

    RETURN intent
END

SUBROUTINE: FallbackBasicParse
INPUT: query (string)
OUTPUT: ParsedIntent

BEGIN
    // Extract keywords using simple NLP
    tokens ← Tokenize(query.toLowerCase())

    // Genre detection
    genres ← ExtractGenres(tokens, GENRE_DICTIONARY)

    // Platform detection
    platforms ← ExtractPlatforms(tokens, PLATFORM_DICTIONARY)

    // Reference detection (simple "like X" pattern)
    references ← ExtractReferences(query, PATTERN: /like\s+([^,\.]+)/i)

    intent ← {
        mood: [],
        themes: [],
        references: references,
        filters: {
            genre: genres,
            platform: platforms,
            yearRange: NULL
        },
        fallbackQuery: query,
        confidence: 0.5
    }

    RETURN intent
END

SUBROUTINE: ValidateIntentStructure
INPUT: intent (object)
OUTPUT: isValid (boolean)

BEGIN
    IF intent IS NULL THEN
        RETURN false
    END IF

    IF NOT intent.hasProperty("confidence") THEN
        RETURN false
    END IF

    IF intent.confidence < 0 OR intent.confidence > 1 THEN
        RETURN false
    END IF

    requiredFields ← ["mood", "themes", "references", "fallbackQuery"]
    FOR EACH field IN requiredFields DO
        IF NOT intent.hasProperty(field) THEN
            RETURN false
        END IF
    END FOR

    RETURN true
END
```

### 1.2 Complexity Analysis

**Time Complexity:**
- Cache hit: O(1)
- Cache miss with GPT: O(1) + O(GPT) where O(GPT) ≈ 1000-1500ms
- Fallback parsing: O(n) where n = query length
- Total (cached): O(1) ≈ 5ms
- Total (uncached): O(GPT) ≈ 1500ms

**Space Complexity:**
- Intent object: O(1)
- Cache storage: O(m) where m = unique queries
- Total: O(m)

**Optimization Notes:**
- Cache hit rate target: >70%
- Consider bloom filter for cache existence checks
- Batch multiple queries for cache warming
- Monitor GPT API latency and adjust timeout

---

## 2. Multi-Strategy Search Orchestrator

### 2.1 Main Orchestration Algorithm

```
ALGORITHM: HybridSearch
INPUT: query (string), filters (object), userContext (object)
OUTPUT: rankedResults (array of Media objects)

CONSTANTS:
    STRATEGY_TIMEOUT = 300 ms per strategy
    MIN_RESULTS = 20
    MAX_RESULTS = 100
    TOTAL_TIMEOUT = 450 ms

BEGIN
    // Phase 1: Parse intent (Target: <100ms cached, <1500ms uncached)
    intent ← ParseNaturalLanguageIntent(query, userContext.userId, true)

    // Phase 2: Parallel strategy execution (Target: <300ms)
    strategies ← [
        {name: "title_match", weight: 1.0, timeout: STRATEGY_TIMEOUT},
        {name: "reference_based", weight: 0.9, timeout: STRATEGY_TIMEOUT},
        {name: "genre_filter", weight: 0.8, timeout: STRATEGY_TIMEOUT},
        {name: "vector_similarity", weight: 1.2, timeout: STRATEGY_TIMEOUT},
        {name: "graph_discovery", weight: 0.7, timeout: STRATEGY_TIMEOUT}
    ]

    // Execute strategies concurrently
    strategyPromises ← []
    FOR EACH strategy IN strategies DO
        promise ← ExecuteStrategyAsync(strategy, query, intent, filters, userContext)
        strategyPromises.append(promise)
    END FOR

    // Wait for all with timeout
    strategyResults ← AwaitAllWithTimeout(strategyPromises, TOTAL_TIMEOUT)

    // Phase 3: Merge and deduplicate (Target: <30ms)
    allResults ← []
    FOR EACH result IN strategyResults DO
        IF result.success THEN
            FOR EACH item IN result.items DO
                allResults.append({
                    media: item.media,
                    score: item.score * result.strategy.weight,
                    source: result.strategy.name
                })
            END FOR
        ELSE
            Log.warn("Strategy failed", {
                strategy: result.strategy.name,
                error: result.error
            })
        END IF
    END FOR

    deduplicatedResults ← DeduplicateByMediaId(allResults)

    // Phase 4: Re-rank with multi-factor scoring (Target: <50ms)
    rankedResults ← RankResults(
        deduplicatedResults,
        intent,
        userContext,
        filters
    )

    // Phase 5: Apply availability filtering (Target: <20ms)
    finalResults ← FilterByAvailability(
        rankedResults,
        userContext.platforms,
        userContext.region,
        filters
    )

    // Ensure minimum results or fall back to relaxed search
    IF LENGTH(finalResults) < MIN_RESULTS THEN
        relaxedResults ← RelaxedSearch(query, filters, userContext)
        finalResults ← MERGE(finalResults, relaxedResults)
    END IF

    RETURN finalResults.slice(0, MAX_RESULTS)
END

SUBROUTINE: ExecuteStrategyAsync
INPUT: strategy (object), query (string), intent (ParsedIntent),
       filters (object), userContext (object)
OUTPUT: Promise<StrategyResult>

BEGIN
    RETURN ASYNC FUNCTION:
        TRY
            SWITCH strategy.name DO
                CASE "title_match":
                    results ← TitleMatchSearch(query, intent, filters)

                CASE "reference_based":
                    results ← ReferenceBasedSearch(intent.references, filters)

                CASE "genre_filter":
                    results ← GenreFilterSearch(intent.filters.genre, filters)

                CASE "vector_similarity":
                    results ← VectorSimilaritySearch(query, intent, filters)

                CASE "graph_discovery":
                    results ← GraphDiscoverySearch(intent, userContext, filters)

                DEFAULT:
                    THROW Error("Unknown strategy: " + strategy.name)
            END SWITCH

            RETURN {
                success: true,
                strategy: strategy,
                items: results,
                latency: MEASURE_TIME()
            }

        CATCH error AS e
            RETURN {
                success: false,
                strategy: strategy,
                error: e.message,
                latency: MEASURE_TIME()
            }
        END TRY
    END ASYNC
END

SUBROUTINE: DeduplicateByMediaId
INPUT: results (array of SearchResult)
OUTPUT: deduplicated (array of SearchResult)

BEGIN
    seen ← MAP<mediaId, SearchResult>()

    FOR EACH result IN results DO
        mediaId ← result.media.id

        IF NOT seen.has(mediaId) THEN
            seen.set(mediaId, result)
        ELSE
            // Merge scores from different strategies
            existing ← seen.get(mediaId)
            existing.score ← existing.score + result.score
            existing.sources ← existing.sources UNION [result.source]
        END IF
    END FOR

    deduplicated ← seen.values().toArray()
    RETURN deduplicated
END

SUBROUTINE: RelaxedSearch
INPUT: query (string), filters (object), userContext (object)
OUTPUT: results (array of Media)

BEGIN
    // Relax filters progressively
    relaxedFilters ← CLONE(filters)

    // Remove year constraints
    DELETE relaxedFilters.yearRange

    // Expand genre to related genres
    IF relaxedFilters.genre IS NOT EMPTY THEN
        relatedGenres ← GetRelatedGenres(relaxedFilters.genre)
        relaxedFilters.genre ← relaxedFilters.genre UNION relatedGenres
    END IF

    // Retry with relaxed filters
    results ← VectorSimilaritySearch(query, NULL, relaxedFilters)

    RETURN results
END
```

### 2.2 Individual Search Strategies

```
SUBROUTINE: TitleMatchSearch
INPUT: query (string), intent (ParsedIntent), filters (object)
OUTPUT: results (array of SearchResult)

CONSTANTS:
    EXACT_MATCH_BOOST = 2.0
    FUZZY_THRESHOLD = 0.7
    MAX_RESULTS = 50

BEGIN
    results ← []
    queryLower ← query.toLowerCase().trim()

    // Strategy 1: Exact title match
    exactMatches ← Database.query("""
        SELECT m.*, 1.0 as base_score
        FROM media m
        WHERE LOWER(m.title) = $1
        LIMIT 10
    """, [queryLower])

    FOR EACH match IN exactMatches DO
        results.append({
            media: match,
            score: EXACT_MATCH_BOOST,
            matchType: "exact"
        })
    END FOR

    // Strategy 2: Prefix match
    prefixMatches ← Database.query("""
        SELECT m.*, 0.9 as base_score
        FROM media m
        WHERE LOWER(m.title) LIKE $1 || '%'
        AND LOWER(m.title) != $1
        LIMIT 20
    """, [queryLower])

    FOR EACH match IN prefixMatches DO
        results.append({
            media: match,
            score: 0.9,
            matchType: "prefix"
        })
    END FOR

    // Strategy 3: Fuzzy match (trigram similarity)
    IF LENGTH(results) < MAX_RESULTS THEN
        fuzzyMatches ← Database.query("""
            SELECT m.*,
                   SIMILARITY(LOWER(m.title), $1) as similarity_score
            FROM media m
            WHERE SIMILARITY(LOWER(m.title), $1) > $2
            ORDER BY similarity_score DESC
            LIMIT 30
        """, [queryLower, FUZZY_THRESHOLD])

        FOR EACH match IN fuzzyMatches DO
            results.append({
                media: match,
                score: match.similarity_score,
                matchType: "fuzzy"
            })
        END FOR
    END IF

    // Apply filters
    results ← ApplyBasicFilters(results, filters)

    RETURN results
END

SUBROUTINE: ReferenceBasedSearch
INPUT: references (array of string), filters (object)
OUTPUT: results (array of SearchResult)

CONSTANTS:
    MAX_SIMILAR_PER_REF = 10
    SIMILARITY_THRESHOLD = 0.75

BEGIN
    IF LENGTH(references) = 0 THEN
        RETURN []
    END IF

    results ← []

    FOR EACH refTitle IN references DO
        // Find the reference media
        refMedia ← Database.query("""
            SELECT id, title
            FROM media
            WHERE LOWER(title) ILIKE '%' || $1 || '%'
            LIMIT 1
        """, [refTitle.toLowerCase()])

        IF refMedia IS NULL THEN
            CONTINUE
        END IF

        // Get similar media via graph relationships
        similarMedia ← Database.query("""
            SELECT m.*, r.similarity_score
            FROM media m
            JOIN relationships r ON r.target_id = m.id
            WHERE r.source_id = $1
            AND r.type = 'SIMILAR_TO'
            AND r.similarity_score >= $2
            ORDER BY r.similarity_score DESC
            LIMIT $3
        """, [refMedia.id, SIMILARITY_THRESHOLD, MAX_SIMILAR_PER_REF])

        FOR EACH media IN similarMedia DO
            results.append({
                media: media,
                score: media.similarity_score,
                referenceTitle: refTitle
            })
        END FOR
    END FOR

    // Apply filters
    results ← ApplyBasicFilters(results, filters)

    RETURN results
END

SUBROUTINE: GenreFilterSearch
INPUT: genres (array of string), filters (object)
OUTPUT: results (array of SearchResult)

CONSTANTS:
    MAX_RESULTS = 100
    POPULARITY_WEIGHT = 0.3
    RECENCY_WEIGHT = 0.2

BEGIN
    IF LENGTH(genres) = 0 THEN
        RETURN []
    END IF

    // Build genre filter query
    results ← Database.query("""
        SELECT m.*,
               m.popularity_score,
               m.release_year,
               EXTRACT(EPOCH FROM (NOW() - m.created_at)) / 86400 as days_old
        FROM media m
        WHERE m.genres && $1::text[]
        ORDER BY
            m.popularity_score DESC,
            m.release_year DESC
        LIMIT $2
    """, [genres, MAX_RESULTS])

    // Score based on popularity and recency
    scoredResults ← []
    currentYear ← CURRENT_YEAR()

    FOR EACH media IN results DO
        popularityScore ← media.popularity_score / 100.0

        yearDiff ← currentYear - media.release_year
        recencyScore ← 1.0 / (1.0 + yearDiff * 0.1)

        finalScore ← (popularityScore * POPULARITY_WEIGHT) +
                     (recencyScore * RECENCY_WEIGHT) + 0.5

        scoredResults.append({
            media: media,
            score: finalScore
        })
    END FOR

    // Apply additional filters
    scoredResults ← ApplyBasicFilters(scoredResults, filters)

    RETURN scoredResults
END

SUBROUTINE: ApplyBasicFilters
INPUT: results (array of SearchResult), filters (object)
OUTPUT: filtered (array of SearchResult)

BEGIN
    filtered ← []

    FOR EACH result IN results DO
        media ← result.media
        shouldInclude ← true

        // Platform filter
        IF filters.platform IS NOT EMPTY THEN
            IF NOT media.platforms INTERSECTS filters.platform THEN
                shouldInclude ← false
            END IF
        END IF

        // Year range filter
        IF filters.yearRange IS NOT NULL THEN
            IF media.release_year < filters.yearRange.min OR
               media.release_year > filters.yearRange.max THEN
                shouldInclude ← false
            END IF
        END IF

        // Genre filter
        IF filters.genre IS NOT EMPTY THEN
            IF NOT media.genres INTERSECTS filters.genre THEN
                shouldInclude ← false
            END IF
        END IF

        IF shouldInclude THEN
            filtered.append(result)
        END IF
    END FOR

    RETURN filtered
END
```

### 2.3 Complexity Analysis

**Time Complexity (Parallel Execution):**
- Intent parsing: O(1) cached or O(GPT)
- Strategy execution: O(max(S₁, S₂, S₃, S₄, S₅)) with timeout
  - Title match: O(log n) with index
  - Reference-based: O(k * log n) where k = reference count
  - Genre filter: O(log n) with index
  - Vector search: O(log n) with HNSW
  - Graph discovery: O(d * e) where d = depth, e = edges
- Deduplication: O(r) where r = total results
- Ranking: O(r log r)
- Availability filter: O(r)
- **Total: O(max(strategies)) + O(r log r) ≈ 300ms + 50ms = 350ms**

**Space Complexity:**
- Strategy results: O(s * r) where s = strategies
- Deduplicated: O(r)
- Total: O(r)

---

## 3. Vector Similarity Search (Ruvector Integration)

### 3.1 HNSW Query Algorithm

```
ALGORITHM: VectorSimilaritySearch
INPUT: query (string), intent (ParsedIntent), filters (object)
OUTPUT: results (array of SearchResult)

CONSTANTS:
    EMBEDDING_MODEL = "text-embedding-3-small"
    EMBEDDING_DIMENSION = 1536
    HNSW_EF_SEARCH = 64
    TOP_K = 50
    SIMILARITY_THRESHOLD = 0.7
    FILTER_SELECTIVITY_THRESHOLD = 0.1

BEGIN
    // Phase 1: Generate query embedding (Target: <50ms)
    queryVector ← GenerateEmbedding(query, EMBEDDING_MODEL)

    // Phase 2: Determine filter strategy
    filterStrategy ← DetermineFilterStrategy(filters)

    // Phase 3: Execute HNSW search with filters
    IF filterStrategy = "pre_filter" THEN
        results ← HNSWSearchWithPreFilter(queryVector, filters)
    ELSE IF filterStrategy = "post_filter" THEN
        results ← HNSWSearchWithPostFilter(queryVector, filters)
    ELSE
        results ← HNSWSearchNoFilter(queryVector)
    END IF

    // Phase 4: Score normalization (Target: <10ms)
    normalizedResults ← NormalizeScores(results)

    // Phase 5: Deduplication (Target: <5ms)
    finalResults ← DeduplicateResults(normalizedResults)

    RETURN finalResults
END

SUBROUTINE: GenerateEmbedding
INPUT: text (string), model (string)
OUTPUT: vector (array of float)

BEGIN
    // Cache check
    cacheKey ← "embedding:" + HASH(text)
    cached ← Cache.get(cacheKey)

    IF cached IS NOT NULL THEN
        RETURN cached
    END IF

    // Call embedding API
    response ← EmbeddingAPI.generate({
        model: model,
        input: text,
        encoding_format: "float"
    })

    vector ← response.data[0].embedding

    // Cache for 1 hour
    Cache.set(cacheKey, vector, 3600)

    RETURN vector
END

SUBROUTINE: DetermineFilterStrategy
INPUT: filters (object)
OUTPUT: strategy (string: "pre_filter" | "post_filter" | "none")

BEGIN
    IF filters IS EMPTY THEN
        RETURN "none"
    END IF

    // Estimate filter selectivity
    totalMedia ← GetTotalMediaCount()

    estimatedMatches ← EstimateFilterMatches(filters)
    selectivity ← estimatedMatches / totalMedia

    // Pre-filter if highly selective (< 10% of data)
    IF selectivity < FILTER_SELECTIVITY_THRESHOLD THEN
        RETURN "pre_filter"
    ELSE
        RETURN "post_filter"
    END IF
END

SUBROUTINE: HNSWSearchWithPreFilter
INPUT: queryVector (array of float), filters (object)
OUTPUT: results (array of SearchResult)

BEGIN
    // Build filter clause
    filterClause ← BuildSQLFilterClause(filters)

    // Query Ruvector with pre-filtering
    results ← Database.query("""
        SELECT
            m.id,
            m.title,
            m.description,
            m.genres,
            m.platforms,
            1 - (v.embedding <=> $1::vector) as similarity
        FROM media m
        JOIN media_vectors v ON v.media_id = m.id
        WHERE {filterClause}
        ORDER BY v.embedding <=> $1::vector
        LIMIT $2
    """, [queryVector, TOP_K])

    // Filter by similarity threshold
    filtered ← []
    FOR EACH result IN results DO
        IF result.similarity >= SIMILARITY_THRESHOLD THEN
            filtered.append({
                media: result,
                score: result.similarity,
                searchType: "vector_pre_filter"
            })
        END IF
    END FOR

    RETURN filtered
END

SUBROUTINE: HNSWSearchWithPostFilter
INPUT: queryVector (array of float), filters (object)
OUTPUT: results (array of SearchResult)

CONSTANTS:
    OVERQUERY_FACTOR = 3

BEGIN
    // Fetch more results than needed
    overqueryK ← TOP_K * OVERQUERY_FACTOR

    // Query without filters
    results ← Database.query("""
        SELECT
            m.id,
            m.title,
            m.description,
            m.genres,
            m.platforms,
            m.release_year,
            1 - (v.embedding <=> $1::vector) as similarity
        FROM media m
        JOIN media_vectors v ON v.media_id = m.id
        WHERE 1 - (v.embedding <=> $1::vector) >= $2
        ORDER BY v.embedding <=> $1::vector
        LIMIT $3
    """, [queryVector, SIMILARITY_THRESHOLD, overqueryK])

    // Apply filters in-memory
    filtered ← []
    FOR EACH result IN results DO
        IF MatchesFilters(result, filters) THEN
            filtered.append({
                media: result,
                score: result.similarity,
                searchType: "vector_post_filter"
            })

            IF LENGTH(filtered) >= TOP_K THEN
                BREAK
            END IF
        END IF
    END FOR

    RETURN filtered
END

SUBROUTINE: HNSWSearchNoFilter
INPUT: queryVector (array of float)
OUTPUT: results (array of SearchResult)

BEGIN
    results ← Database.query("""
        SELECT
            m.id,
            m.title,
            m.description,
            m.genres,
            m.platforms,
            1 - (v.embedding <=> $1::vector) as similarity
        FROM media m
        JOIN media_vectors v ON v.media_id = m.id
        ORDER BY v.embedding <=> $1::vector
        LIMIT $2
    """, [queryVector, TOP_K])

    scoredResults ← []
    FOR EACH result IN results DO
        scoredResults.append({
            media: result,
            score: result.similarity,
            searchType: "vector_no_filter"
        })
    END FOR

    RETURN scoredResults
END

SUBROUTINE: NormalizeScores
INPUT: results (array of SearchResult)
OUTPUT: normalized (array of SearchResult)

BEGIN
    IF LENGTH(results) = 0 THEN
        RETURN []
    END IF

    // Find min and max scores
    minScore ← MIN(results.map(r => r.score))
    maxScore ← MAX(results.map(r => r.score))

    // Avoid division by zero
    IF maxScore = minScore THEN
        FOR EACH result IN results DO
            result.normalizedScore ← 1.0
        END FOR
        RETURN results
    END IF

    // Min-Max normalization to [0, 1]
    FOR EACH result IN results DO
        result.normalizedScore ← (result.score - minScore) / (maxScore - minScore)
    END FOR

    RETURN results
END

SUBROUTINE: MatchesFilters
INPUT: media (object), filters (object)
OUTPUT: matches (boolean)

BEGIN
    // Platform filter
    IF filters.platform IS NOT EMPTY THEN
        IF NOT media.platforms INTERSECTS filters.platform THEN
            RETURN false
        END IF
    END IF

    // Genre filter
    IF filters.genre IS NOT EMPTY THEN
        IF NOT media.genres INTERSECTS filters.genre THEN
            RETURN false
        END IF
    END IF

    // Year range filter
    IF filters.yearRange IS NOT NULL THEN
        IF media.release_year < filters.yearRange.min OR
           media.release_year > filters.yearRange.max THEN
            RETURN false
        END IF
    END IF

    RETURN true
END
```

### 3.2 Complexity Analysis

**Time Complexity:**
- Embedding generation: O(1) cached or O(API) ≈ 50ms
- HNSW search: O(log n) with ef_search parameter
- Pre-filter: O(f + log n) where f = filter evaluation
- Post-filter: O(k * f) where k = overquery results
- Score normalization: O(r)
- **Total: O(log n) + O(r) ≈ 100-150ms**

**Space Complexity:**
- Query vector: O(d) where d = embedding dimension
- Results: O(k)
- Total: O(k + d)

**Index Optimization:**
- HNSW parameters: M=16, ef_construction=64, ef_search=64
- Expected recall: >95%
- Index size: O(n * d * M)

---

## 4. Graph-Based Discovery

### 4.1 Relationship Traversal Algorithm

```
ALGORITHM: GraphDiscoverySearch
INPUT: intent (ParsedIntent), userContext (object), filters (object)
OUTPUT: results (array of SearchResult)

CONSTANTS:
    MAX_DEPTH = 3
    MAX_TRAVERSALS = 100
    RELATIONSHIP_TYPES = ["SIMILAR_TO", "SAME_FRANCHISE", "SAME_DIRECTOR",
                          "SHARED_CAST", "CO_WATCHED"]
    EDGE_SCORE_WEIGHTS = {
        "SIMILAR_TO": 1.0,
        "SAME_FRANCHISE": 0.9,
        "SAME_DIRECTOR": 0.7,
        "SHARED_CAST": 0.6,
        "CO_WATCHED": 0.8
    }

BEGIN
    // Phase 1: Identify seed nodes
    seedNodes ← IdentifySeedNodes(intent, userContext, filters)

    IF LENGTH(seedNodes) = 0 THEN
        RETURN []
    END IF

    // Phase 2: Multi-path graph traversal
    discoveredMedia ← MAP<mediaId, DiscoveryScore>()
    visited ← SET()

    FOR EACH seed IN seedNodes DO
        traversalResults ← BreadthFirstTraversal(
            seed,
            MAX_DEPTH,
            MAX_TRAVERSALS,
            visited,
            filters
        )

        // Merge results with score decay
        FOR EACH result IN traversalResults DO
            IF discoveredMedia.has(result.mediaId) THEN
                // Accumulate scores from multiple paths
                existing ← discoveredMedia.get(result.mediaId)
                existing.score ← existing.score + (result.score * 0.5)
                existing.paths ← existing.paths + 1
            ELSE
                discoveredMedia.set(result.mediaId, {
                    media: result.media,
                    score: result.score,
                    depth: result.depth,
                    relationshipType: result.relationshipType,
                    paths: 1
                })
            END IF
        END FOR
    END FOR

    // Phase 3: Boost multi-path discoveries
    finalResults ← []
    FOR EACH [mediaId, discovery] IN discoveredMedia DO
        // Multiple paths increase confidence
        pathBoost ← MIN(discovery.paths * 0.1, 0.5)
        finalScore ← discovery.score + pathBoost

        finalResults.append({
            media: discovery.media,
            score: finalScore,
            searchType: "graph_discovery",
            metadata: {
                depth: discovery.depth,
                paths: discovery.paths,
                relationshipType: discovery.relationshipType
            }
        })
    END FOR

    // Sort by score
    finalResults.sortByDescending(score)

    RETURN finalResults
END

SUBROUTINE: IdentifySeedNodes
INPUT: intent (ParsedIntent), userContext (object), filters (object)
OUTPUT: seedNodes (array of Media)

BEGIN
    seeds ← []

    // Strategy 1: User's recent watches
    IF userContext.recentWatches IS NOT EMPTY THEN
        recentSeeds ← userContext.recentWatches.slice(0, 5)
        seeds ← seeds CONCAT recentSeeds
    END IF

    // Strategy 2: User's favorites
    IF userContext.favorites IS NOT EMPTY THEN
        favoriteSeeds ← userContext.favorites.slice(0, 3)
        seeds ← seeds CONCAT favoriteSeeds
    END IF

    // Strategy 3: Reference titles from intent
    IF LENGTH(intent.references) > 0 THEN
        FOR EACH refTitle IN intent.references DO
            refMedia ← FindMediaByTitle(refTitle)
            IF refMedia IS NOT NULL THEN
                seeds.append(refMedia)
            END IF
        END FOR
    END IF

    // Strategy 4: Popular in genre (fallback)
    IF LENGTH(seeds) = 0 AND LENGTH(intent.filters.genre) > 0 THEN
        popularInGenre ← Database.query("""
            SELECT m.*
            FROM media m
            WHERE m.genres && $1::text[]
            ORDER BY m.popularity_score DESC
            LIMIT 3
        """, [intent.filters.genre])

        seeds ← seeds CONCAT popularInGenre
    END IF

    RETURN seeds
END

SUBROUTINE: BreadthFirstTraversal
INPUT: startNode (Media), maxDepth (int), maxTraversals (int),
       visited (SET), filters (object)
OUTPUT: discoveries (array of DiscoveryResult)

BEGIN
    queue ← QUEUE()
    discoveries ← []
    traversalCount ← 0

    // Initialize with start node
    queue.enqueue({
        mediaId: startNode.id,
        depth: 0,
        score: 1.0,
        path: [startNode.id]
    })

    visited.add(startNode.id)

    WHILE NOT queue.isEmpty() AND traversalCount < maxTraversals DO
        current ← queue.dequeue()
        traversalCount ← traversalCount + 1

        // Stop at max depth
        IF current.depth >= maxDepth THEN
            CONTINUE
        END IF

        // Get outgoing relationships
        relationships ← Database.query("""
            SELECT
                r.target_id,
                r.type,
                r.weight,
                m.*
            FROM relationships r
            JOIN media m ON m.id = r.target_id
            WHERE r.source_id = $1
            AND r.type = ANY($2::text[])
            ORDER BY r.weight DESC
            LIMIT 10
        """, [current.mediaId, RELATIONSHIP_TYPES])

        FOR EACH rel IN relationships DO
            IF visited.has(rel.target_id) THEN
                CONTINUE
            END IF

            // Apply filters
            IF NOT MatchesFilters(rel, filters) THEN
                CONTINUE
            END IF

            visited.add(rel.target_id)

            // Calculate score with depth decay
            depthDecay ← POW(0.7, current.depth + 1)
            edgeWeight ← EDGE_SCORE_WEIGHTS[rel.type]
            newScore ← current.score * depthDecay * edgeWeight * rel.weight

            // Add to discoveries
            discoveries.append({
                mediaId: rel.target_id,
                media: rel,
                score: newScore,
                depth: current.depth + 1,
                relationshipType: rel.type,
                path: current.path CONCAT [rel.target_id]
            })

            // Continue traversal
            queue.enqueue({
                mediaId: rel.target_id,
                depth: current.depth + 1,
                score: newScore,
                path: current.path CONCAT [rel.target_id]
            })
        END FOR
    END WHILE

    RETURN discoveries
END

SUBROUTINE: FindSharedCastDiscoveries
INPUT: actorIds (array of UUID), filters (object)
OUTPUT: results (array of SearchResult)

BEGIN
    // Find media with overlapping cast
    results ← Database.query("""
        SELECT
            m.*,
            COUNT(DISTINCT c.actor_id) as shared_count,
            ARRAY_AGG(DISTINCT a.name) as shared_actors
        FROM media m
        JOIN cast_members c ON c.media_id = m.id
        JOIN actors a ON a.id = c.actor_id
        WHERE c.actor_id = ANY($1::uuid[])
        GROUP BY m.id
        HAVING COUNT(DISTINCT c.actor_id) >= 2
        ORDER BY shared_count DESC
        LIMIT 20
    """, [actorIds])

    scoredResults ← []
    FOR EACH result IN results DO
        // Score based on shared cast percentage
        score ← result.shared_count / LENGTH(actorIds)

        scoredResults.append({
            media: result,
            score: score,
            searchType: "shared_cast",
            metadata: {
                sharedActors: result.shared_actors,
                sharedCount: result.shared_count
            }
        })
    END FOR

    RETURN scoredResults
END
```

### 4.2 Complexity Analysis

**Time Complexity:**
- Seed identification: O(s) where s = seed count
- BFS traversal: O(v + e) where v = visited nodes, e = edges
  - Limited by MAX_TRAVERSALS and MAX_DEPTH
- Total: O(s * (v + e)) with bounded v, e
- **Practical: O(100) ≈ 50-100ms**

**Space Complexity:**
- Queue: O(w) where w = max width
- Visited set: O(v)
- Discoveries: O(d) where d = discovered nodes
- Total: O(v + d)

**Graph Optimization:**
- Index on (source_id, type) for fast relationship lookup
- Denormalize high-degree nodes
- Cache popular traversal paths
- Use bi-directional BFS for deep searches

---

## 5. Result Ranking Algorithm

### 5.1 Multi-Factor Scoring

```
ALGORITHM: RankResults
INPUT: results (array of SearchResult), intent (ParsedIntent),
       userContext (object), filters (object)
OUTPUT: rankedResults (array of SearchResult)

CONSTANTS:
    // Score component weights
    WEIGHT_BASE_MATCH = 1.0
    WEIGHT_THEME_MATCH = 0.5
    WEIGHT_PREFERENCE = 0.8
    WEIGHT_POPULARITY = 0.3
    WEIGHT_FRESHNESS = 0.2
    WEIGHT_PLATFORM_MATCH = 0.4

    // Normalization constants
    MAX_POPULARITY = 100.0
    FRESHNESS_DECAY_DAYS = 365

BEGIN
    // Phase 1: Load user preferences from SONA
    userPreferences ← LoadUserPreferences(userContext.userId)

    // Phase 2: Calculate multi-factor scores
    scoredResults ← []

    FOR EACH result IN results DO
        // Component 1: Base match score (from search strategy)
        baseScore ← result.score * WEIGHT_BASE_MATCH

        // Component 2: Theme matching
        themeScore ← CalculateThemeMatch(result.media, intent) * WEIGHT_THEME_MATCH

        // Component 3: Preference alignment (SONA)
        preferenceScore ← CalculatePreferenceAlignment(
            result.media,
            userPreferences
        ) * WEIGHT_PREFERENCE

        // Component 4: Popularity multiplier
        popularityScore ← (result.media.popularity_score / MAX_POPULARITY) *
                          WEIGHT_POPULARITY

        // Component 5: Freshness boost
        freshnessScore ← CalculateFreshnessBoost(result.media) * WEIGHT_FRESHNESS

        // Component 6: Platform availability boost
        platformScore ← CalculatePlatformMatch(
            result.media,
            userContext.platforms
        ) * WEIGHT_PLATFORM_MATCH

        // Aggregate final score
        finalScore ← baseScore + themeScore + preferenceScore +
                     popularityScore + freshnessScore + platformScore

        scoredResults.append({
            media: result.media,
            finalScore: finalScore,
            scoreBreakdown: {
                base: baseScore,
                theme: themeScore,
                preference: preferenceScore,
                popularity: popularityScore,
                freshness: freshnessScore,
                platform: platformScore
            },
            originalSource: result.searchType
        })
    END FOR

    // Phase 3: Sort by final score
    scoredResults.sortByDescending(finalScore)

    // Phase 4: Diversity enforcement (avoid genre clustering)
    diversifiedResults ← EnforceDiversity(scoredResults)

    RETURN diversifiedResults
END

SUBROUTINE: LoadUserPreferences
INPUT: userId (UUID)
OUTPUT: preferences (UserPreferences)

BEGIN
    // Query SONA neural network for user preferences
    preferences ← Database.query("""
        SELECT
            preferred_genres,
            preferred_actors,
            preferred_directors,
            mood_preferences,
            average_rating,
            watch_history_vector
        FROM user_preferences
        WHERE user_id = $1
    """, [userId])

    IF preferences IS NULL THEN
        // Return default neutral preferences
        RETURN {
            preferred_genres: [],
            preferred_actors: [],
            preferred_directors: [],
            mood_preferences: {},
            average_rating: 3.5,
            watch_history_vector: NULL
        }
    END IF

    RETURN preferences
END

SUBROUTINE: CalculateThemeMatch
INPUT: media (Media), intent (ParsedIntent)
OUTPUT: score (float)

BEGIN
    score ← 0.0
    matchCount ← 0

    // Match mood tags
    IF LENGTH(intent.mood) > 0 THEN
        FOR EACH mood IN intent.mood DO
            IF media.tags CONTAINS mood THEN
                score ← score + 0.3
                matchCount ← matchCount + 1
            END IF
        END FOR
    END IF

    // Match theme tags
    IF LENGTH(intent.themes) > 0 THEN
        FOR EACH theme IN intent.themes DO
            IF media.tags CONTAINS theme OR media.description CONTAINS theme THEN
                score ← score + 0.4
                matchCount ← matchCount + 1
            END IF
        END FOR
    END IF

    // Normalize by number of intent themes
    totalIntentItems ← LENGTH(intent.mood) + LENGTH(intent.themes)
    IF totalIntentItems > 0 THEN
        score ← score / totalIntentItems
    END IF

    RETURN MIN(score, 1.0)
END

SUBROUTINE: CalculatePreferenceAlignment
INPUT: media (Media), preferences (UserPreferences)
OUTPUT: score (float)

BEGIN
    score ← 0.0
    components ← 0

    // Genre preference alignment
    IF LENGTH(preferences.preferred_genres) > 0 THEN
        genreOverlap ← INTERSECTION(media.genres, preferences.preferred_genres)
        genreScore ← LENGTH(genreOverlap) / LENGTH(preferences.preferred_genres)
        score ← score + genreScore
        components ← components + 1
    END IF

    // Actor preference alignment
    IF LENGTH(preferences.preferred_actors) > 0 THEN
        actorOverlap ← INTERSECTION(media.cast, preferences.preferred_actors)
        actorScore ← LENGTH(actorOverlap) / MIN(3, LENGTH(preferences.preferred_actors))
        score ← score + actorScore
        components ← components + 1
    END IF

    // Director preference alignment
    IF LENGTH(preferences.preferred_directors) > 0 THEN
        IF media.director IN preferences.preferred_directors THEN
            score ← score + 1.0
        END IF
        components ← components + 1
    END IF

    // Vector similarity with watch history
    IF preferences.watch_history_vector IS NOT NULL AND
       media.embedding IS NOT NULL THEN
        vectorSimilarity ← CosineSimilarity(
            media.embedding,
            preferences.watch_history_vector
        )
        score ← score + vectorSimilarity
        components ← components + 1
    END IF

    // Normalize
    IF components > 0 THEN
        score ← score / components
    END IF

    RETURN score
END

SUBROUTINE: CalculateFreshnessBoost
INPUT: media (Media)
OUTPUT: score (float)

BEGIN
    currentDate ← CURRENT_DATE()

    // Calculate days since release or added to platform
    IF media.platform_added_date IS NOT NULL THEN
        daysSinceAdded ← DAYS_BETWEEN(media.platform_added_date, currentDate)
    ELSE
        // Fallback to release year
        releaseDate ← DATE(media.release_year, 1, 1)
        daysSinceAdded ← DAYS_BETWEEN(releaseDate, currentDate)
    END IF

    // Exponential decay
    freshnessScore ← EXP(-daysSinceAdded / FRESHNESS_DECAY_DAYS)

    RETURN freshnessScore
END

SUBROUTINE: CalculatePlatformMatch
INPUT: media (Media), userPlatforms (array of string)
OUTPUT: score (float)

BEGIN
    IF LENGTH(userPlatforms) = 0 THEN
        RETURN 0.0
    END IF

    // Check if media is available on user's platforms
    availableOnUserPlatforms ← INTERSECTION(media.platforms, userPlatforms)

    IF LENGTH(availableOnUserPlatforms) = 0 THEN
        RETURN 0.0
    END IF

    // Bonus for being on multiple user platforms
    platformCount ← LENGTH(availableOnUserPlatforms)
    score ← MIN(platformCount / 2.0, 1.0)

    RETURN score
END

SUBROUTINE: EnforceDiversity
INPUT: results (array of ScoredResult)
OUTPUT: diversified (array of ScoredResult)

CONSTANTS:
    WINDOW_SIZE = 5
    MAX_SAME_GENRE_IN_WINDOW = 3

BEGIN
    diversified ← []
    genreWindow ← []

    FOR EACH result IN results DO
        primaryGenre ← result.media.genres[0]

        // Check genre concentration in recent window
        genreCount ← COUNT(genreWindow, primaryGenre)

        IF genreCount < MAX_SAME_GENRE_IN_WINDOW THEN
            // Add to results
            diversified.append(result)
            genreWindow.append(primaryGenre)

            // Maintain window size
            IF LENGTH(genreWindow) > WINDOW_SIZE THEN
                genreWindow ← genreWindow.slice(1)
            END IF
        ELSE
            // Skip for now, re-insert later
            // (Could implement a deferred queue here)
            CONTINUE
        END IF
    END FOR

    RETURN diversified
END

SUBROUTINE: CosineSimilarity
INPUT: vector1 (array of float), vector2 (array of float)
OUTPUT: similarity (float)

BEGIN
    IF LENGTH(vector1) != LENGTH(vector2) THEN
        THROW Error("Vector dimensions must match")
    END IF

    dotProduct ← 0.0
    magnitude1 ← 0.0
    magnitude2 ← 0.0

    FOR i FROM 0 TO LENGTH(vector1) - 1 DO
        dotProduct ← dotProduct + (vector1[i] * vector2[i])
        magnitude1 ← magnitude1 + (vector1[i] * vector1[i])
        magnitude2 ← magnitude2 + (vector2[i] * vector2[i])
    END FOR

    magnitude1 ← SQRT(magnitude1)
    magnitude2 ← SQRT(magnitude2)

    IF magnitude1 = 0 OR magnitude2 = 0 THEN
        RETURN 0.0
    END IF

    similarity ← dotProduct / (magnitude1 * magnitude2)

    RETURN similarity
END
```

### 5.2 Complexity Analysis

**Time Complexity:**
- Load preferences: O(1) with user_id index
- Theme matching: O(r * (m + t)) where m = moods, t = themes
- Preference alignment: O(r * (g + a + d)) where g, a, d = preference sizes
- Vector similarity: O(r * d) where d = embedding dimension
- Sorting: O(r log r)
- Diversity enforcement: O(r)
- **Total: O(r log r) + O(r * d) ≈ 30-50ms for r=100**

**Space Complexity:**
- Preferences: O(1)
- Scored results: O(r)
- Diversity window: O(w) where w = window size
- Total: O(r)

---

## 6. Availability Filtering

### 6.1 Regional and Platform Filtering

```
ALGORITHM: FilterByAvailability
INPUT: results (array of ScoredResult), userPlatforms (array of string),
       region (string), filters (object)
OUTPUT: filtered (array of ScoredResult)

CONSTANTS:
    AVAILABILITY_CACHE_TTL = 3600 // 1 hour
    PRICE_TIER_ORDER = ["free", "subscription", "rent", "buy"]

BEGIN
    filtered ← []

    FOR EACH result IN results DO
        media ← result.media

        // Get availability data
        availability ← GetMediaAvailability(media.id, region)

        IF availability IS NULL OR LENGTH(availability) = 0 THEN
            // No availability data, skip
            CONTINUE
        END IF

        // Filter by user platforms
        platformMatches ← FilterByPlatforms(availability, userPlatforms)

        IF LENGTH(platformMatches) = 0 THEN
            CONTINUE
        END IF

        // Apply price tier filter
        IF filters.priceTier IS NOT NULL THEN
            platformMatches ← FilterByPriceTier(platformMatches, filters.priceTier)

            IF LENGTH(platformMatches) = 0 THEN
                CONTINUE
            END IF
        END IF

        // Determine best availability option
        bestOption ← SelectBestAvailability(platformMatches)

        // Enhance result with availability info
        result.availability ← {
            platform: bestOption.platform,
            type: bestOption.type,
            price: bestOption.price,
            url: bestOption.url,
            allOptions: platformMatches
        }

        // Boost score for better availability
        availabilityBoost ← CalculateAvailabilityBoost(bestOption)
        result.finalScore ← result.finalScore + availabilityBoost

        filtered.append(result)
    END FOR

    // Re-sort after availability boost
    filtered.sortByDescending(finalScore)

    RETURN filtered
END

SUBROUTINE: GetMediaAvailability
INPUT: mediaId (UUID), region (string)
OUTPUT: availability (array of AvailabilityOption)

BEGIN
    // Check cache first
    cacheKey ← "availability:" + mediaId + ":" + region
    cached ← Cache.get(cacheKey)

    IF cached IS NOT NULL THEN
        RETURN cached
    END IF

    // Query availability database
    availability ← Database.query("""
        SELECT
            a.platform,
            a.availability_type,
            a.price,
            a.currency,
            a.url,
            a.quality,
            a.last_verified
        FROM media_availability a
        WHERE a.media_id = $1
        AND a.region = $2
        AND a.is_active = true
        ORDER BY
            CASE a.availability_type
                WHEN 'free' THEN 1
                WHEN 'subscription' THEN 2
                WHEN 'rent' THEN 3
                WHEN 'buy' THEN 4
            END
    """, [mediaId, region])

    // Cache for 1 hour
    Cache.set(cacheKey, availability, AVAILABILITY_CACHE_TTL)

    RETURN availability
END

SUBROUTINE: FilterByPlatforms
INPUT: availability (array of AvailabilityOption), userPlatforms (array of string)
OUTPUT: matches (array of AvailabilityOption)

BEGIN
    IF LENGTH(userPlatforms) = 0 THEN
        // No platform filter, return all
        RETURN availability
    END IF

    matches ← []

    FOR EACH option IN availability DO
        IF option.platform IN userPlatforms THEN
            matches.append(option)
        END IF
    END FOR

    RETURN matches
END

SUBROUTINE: FilterByPriceTier
INPUT: availability (array of AvailabilityOption), maxTier (string)
OUTPUT: filtered (array of AvailabilityOption)

BEGIN
    maxTierIndex ← INDEX_OF(PRICE_TIER_ORDER, maxTier)

    IF maxTierIndex = -1 THEN
        // Invalid tier, return all
        RETURN availability
    END IF

    filtered ← []

    FOR EACH option IN availability DO
        tierIndex ← INDEX_OF(PRICE_TIER_ORDER, option.availability_type)

        IF tierIndex <= maxTierIndex THEN
            filtered.append(option)
        END IF
    END FOR

    RETURN filtered
END

SUBROUTINE: SelectBestAvailability
INPUT: options (array of AvailabilityOption)
OUTPUT: best (AvailabilityOption)

BEGIN
    IF LENGTH(options) = 0 THEN
        RETURN NULL
    END IF

    // Priority: free > subscription > rent > buy
    FOR EACH tier IN PRICE_TIER_ORDER DO
        FOR EACH option IN options DO
            IF option.availability_type = tier THEN
                RETURN option
            END IF
        END FOR
    END FOR

    // Fallback to first option
    RETURN options[0]
END

SUBROUTINE: CalculateAvailabilityBoost
INPUT: option (AvailabilityOption)
OUTPUT: boost (float)

BEGIN
    // Boost based on availability type
    SWITCH option.availability_type DO
        CASE "free":
            RETURN 0.3
        CASE "subscription":
            RETURN 0.2
        CASE "rent":
            RETURN 0.1
        CASE "buy":
            RETURN 0.05
        DEFAULT:
            RETURN 0.0
    END SWITCH
END
```

### 6.2 Complexity Analysis

**Time Complexity:**
- Per result availability lookup: O(1) cached or O(log n) query
- Platform filtering: O(a * p) where a = availability count, p = platform count
- Price tier filtering: O(a)
- Best option selection: O(a)
- Total: O(r * a) where r = results
- **Practical: O(100 * 5) ≈ 10-20ms**

**Space Complexity:**
- Availability data: O(r * a)
- Filtered results: O(r)
- Total: O(r * a)

---

## 7. Data Structures

### 7.1 Core Data Structures

```
DATA STRUCTURE: ParsedIntent
    mood: array of string
    themes: array of string
    references: array of string
    filters: FilterObject {
        genre: array of string
        platform: array of string
        yearRange: {min: integer, max: integer} or NULL
    }
    fallbackQuery: string
    confidence: float [0.0, 1.0]

DATA STRUCTURE: SearchResult
    media: Media object
    score: float
    searchType: string (strategy identifier)
    metadata: object (strategy-specific data)

DATA STRUCTURE: ScoredResult
    media: Media object
    finalScore: float
    scoreBreakdown: {
        base: float
        theme: float
        preference: float
        popularity: float
        freshness: float
        platform: float
    }
    availability: AvailabilityInfo or NULL
    originalSource: string

DATA STRUCTURE: AvailabilityInfo
    platform: string
    type: string ("free" | "subscription" | "rent" | "buy")
    price: float or NULL
    url: string
    allOptions: array of AvailabilityOption

DATA STRUCTURE: UserPreferences
    preferred_genres: array of string
    preferred_actors: array of UUID
    preferred_directors: array of UUID
    mood_preferences: MAP<mood, float>
    average_rating: float
    watch_history_vector: array of float (embedding)
```

### 7.2 Index Structures

```
INDEXES:

1. Media Title Index (B-Tree + GIN Trigram)
   - Purpose: Fast title matching (exact, prefix, fuzzy)
   - Type: Composite
   - Query: O(log n)

   CREATE INDEX idx_media_title_lower ON media (LOWER(title));
   CREATE INDEX idx_media_title_trigram ON media USING GIN (title gin_trgm_ops);

2. Genre Array Index (GIN)
   - Purpose: Fast genre filtering
   - Type: Generalized Inverted Index
   - Query: O(log n)

   CREATE INDEX idx_media_genres ON media USING GIN (genres);

3. Platform Array Index (GIN)
   - Purpose: Fast platform filtering
   - Type: Generalized Inverted Index
   - Query: O(log n)

   CREATE INDEX idx_media_platforms ON media USING GIN (platforms);

4. Vector Similarity Index (HNSW)
   - Purpose: Fast vector similarity search
   - Type: Hierarchical Navigable Small World
   - Query: O(log n)
   - Parameters: M=16, ef_construction=64, ef_search=64

   CREATE INDEX idx_media_vectors_hnsw ON media_vectors
   USING hnsw (embedding vector_cosine_ops)
   WITH (m = 16, ef_construction = 64);

5. Relationship Graph Index (B-Tree)
   - Purpose: Fast graph traversal
   - Type: Composite B-Tree
   - Query: O(log e) where e = edge count

   CREATE INDEX idx_relationships_source_type
   ON relationships (source_id, type, weight DESC);

6. Availability Lookup Index (B-Tree)
   - Purpose: Fast availability queries
   - Type: Composite B-Tree
   - Query: O(log n)

   CREATE INDEX idx_availability_media_region
   ON media_availability (media_id, region, is_active);

7. Cache Index (Hash)
   - Purpose: O(1) cache lookups
   - Type: Hash Map (Redis)
   - Query: O(1)
   - TTL: Per-key expiration
```

---

## 8. Latency Budget Breakdown

### 8.1 Target Latencies (P95)

```
LATENCY BUDGET (Total: <500ms)

Phase 1: Intent Parsing
  - Cache hit: 5ms
  - Cache miss (GPT): 100ms (amortized with cache)
  - Fallback parsing: 20ms
  - Budgeted: 50ms average

Phase 2: Multi-Strategy Search (Parallel)
  - Title match: 50ms
  - Reference-based: 80ms
  - Genre filter: 40ms
  - Vector similarity: 150ms
  - Graph discovery: 100ms
  - Parallel execution: MAX(150ms) = 150ms
  - Budgeted: 150ms

Phase 3: Deduplication
  - Hash-based dedup: 10ms
  - Budgeted: 10ms

Phase 4: Ranking
  - Load preferences: 20ms
  - Multi-factor scoring: 40ms
  - Sorting: 10ms
  - Diversity: 10ms
  - Budgeted: 80ms

Phase 5: Availability Filtering
  - Availability lookup (cached): 20ms
  - Platform filtering: 5ms
  - Re-sorting: 5ms
  - Budgeted: 30ms

Phase 6: Serialization
  - JSON encoding: 20ms
  - Budgeted: 20ms

TOTAL BUDGETED: 340ms
BUFFER: 160ms (for network, locks, etc.)
P95 TARGET: <500ms
```

### 8.2 Optimization Strategies

```
OPTIMIZATION STRATEGIES:

1. Caching Layers
   - Intent parsing cache (10-minute TTL): >70% hit rate
   - Embedding cache (1-hour TTL): >80% hit rate
   - Availability cache (1-hour TTL): >90% hit rate
   - User preference cache (5-minute TTL): >60% hit rate

2. Database Optimizations
   - Connection pooling: 20 connections per instance
   - Prepared statements: All queries pre-compiled
   - Query timeout: 200ms per query
   - Index-only scans: Where possible

3. Parallel Execution
   - Strategy execution: 5 strategies in parallel
   - Availability lookup: Batch 100 media IDs
   - Embedding generation: Batch 10 queries

4. Algorithm Optimization
   - Early termination: Stop strategies at timeout
   - Result limits: Cap at 50 per strategy
   - Filter selectivity: Pre-filter vs post-filter decision
   - Graph traversal: Bounded depth and width

5. Infrastructure
   - Read replicas: 3 replicas for search queries
   - CDN caching: Static availability data
   - Redis cluster: 3-node cluster for cache
   - Load balancing: Round-robin across 5 instances

6. Monitoring and Alerts
   - P50, P95, P99 latency tracking
   - Cache hit rate monitoring
   - Strategy timeout rate tracking
   - Alert on P95 > 600ms
```

---

## 9. Performance Analysis Summary

### 9.1 Overall Complexity

```
TOTAL SYSTEM COMPLEXITY:

Time Complexity:
  - Best case (cache hit): O(log n) ≈ 50ms
  - Average case: O(max(strategies)) + O(r log r) ≈ 340ms
  - Worst case (no cache): O(GPT) + O(max(strategies)) ≈ 1650ms

Space Complexity:
  - Per query: O(r + d) where r = results, d = embedding dim
  - Cache: O(m) where m = unique queries/embeddings
  - Total: O(r + d + m)

Scalability:
  - Concurrent queries: 1000+ with horizontal scaling
  - Database size: O(log n) query time with proper indexes
  - Cache size: O(m) linear with query diversity
```

### 9.2 Trade-offs and Decisions

```
DESIGN TRADE-OFFS:

1. Accuracy vs Latency
   - Decision: Parallel strategies with timeout
   - Trade-off: May miss some results but meet latency budget
   - Mitigation: Relaxed search for low result counts

2. Freshness vs Performance
   - Decision: Cache with 10-minute TTL for intent parsing
   - Trade-off: Slightly stale intent for repeat queries
   - Mitigation: Short TTL, cache invalidation on major changes

3. Personalization vs Privacy
   - Decision: Server-side preference storage (SONA)
   - Trade-off: Requires user data storage
   - Mitigation: Encrypted storage, opt-in feature

4. Diversity vs Relevance
   - Decision: Sliding window diversity enforcement
   - Trade-off: May lower overall relevance score
   - Mitigation: Only apply after top-k ranking

5. Pre-filter vs Post-filter (Vector Search)
   - Decision: Dynamic based on selectivity
   - Trade-off: Pre-filter faster but less flexible
   - Mitigation: Estimate selectivity and choose strategy
```

---

## End of Document

**Document**: search-discovery-engine.md
**Path**: /workspaces/media-gateway/docs/sparc/pseudocode/search-discovery-engine.md
**Phase**: SPARC Pseudocode
**Date**: 2025-12-06
**Status**: Complete
