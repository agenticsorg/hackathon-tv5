# Content Search Tools - Pseudocode Design

## 1. Semantic Search Tool

### 1.1 Main Search Algorithm

```
ALGORITHM: SemanticSearch
INPUT:
    query (string) - Natural language search query
    filters (object) - Optional filters
    limit (integer) - Max results (default: 20)
    offset (integer) - Pagination offset (default: 0)
OUTPUT:
    results (array) - Ranked search results with availability

DATA STRUCTURES:
    SearchIndex: Inverted index for keyword search
        Type: Map<token, List<documentId>>
        Operations: O(1) lookup, O(k) retrieval where k = matching docs

    VectorStore: Embedding-based semantic search
        Type: Vector database (e.g., FAISS, Pinecone)
        Operations: O(log n) nearest neighbor search

    ContentCache: LRU cache for frequent searches
        Type: LRU Cache
        Size: 10,000 entries
        TTL: 5 minutes

BEGIN
    // Step 1: Parse and normalize query
    normalizedQuery ← NormalizeQuery(query)
    queryTokens ← Tokenize(normalizedQuery)
    queryIntent ← DetectIntent(normalizedQuery)

    // Step 2: Check cache
    cacheKey ← GenerateCacheKey(normalizedQuery, filters, limit, offset)
    cachedResult ← ContentCache.get(cacheKey)

    IF cachedResult is not null THEN
        LOG("Cache hit for query: " + query)
        RETURN cachedResult
    END IF

    // Step 3: Hybrid search (keyword + semantic)
    keywordCandidates ← KeywordSearch(queryTokens, filters)
    semanticCandidates ← SemanticSearch(normalizedQuery, filters)

    // Step 4: Merge and deduplicate candidates
    allCandidates ← MergeCandidates(keywordCandidates, semanticCandidates)

    // Step 5: Apply filters
    filteredCandidates ← ApplyFilters(allCandidates, filters)

    // Step 6: Score and rank
    scoredResults ← []
    FOR EACH candidate IN filteredCandidates DO
        score ← CalculateRelevanceScore(candidate, query, queryIntent)
        scoredResults.append({
            content: candidate,
            score: score
        })
    END FOR

    // Step 7: Sort by score (descending)
    scoredResults.sortByDescending(score)

    // Step 8: Enrich with availability data
    enrichedResults ← EnrichWithAvailability(scoredResults, authContext)

    // Step 9: Apply pagination
    paginatedResults ← enrichedResults.slice(offset, offset + limit)

    // Step 10: Format results
    formattedResults ← FormatSearchResults(paginatedResults)

    // Step 11: Cache results
    ContentCache.set(cacheKey, formattedResults)

    RETURN formattedResults
END


SUBROUTINE: NormalizeQuery
INPUT: query (string)
OUTPUT: normalized (string)

BEGIN
    // Convert to lowercase
    normalized ← query.toLowerCase()

    // Remove special characters (keep spaces and alphanumeric)
    normalized ← normalized.replace(/[^a-z0-9\s]/g, " ")

    // Collapse multiple spaces
    normalized ← normalized.replace(/\s+/g, " ")

    // Trim whitespace
    normalized ← normalized.trim()

    RETURN normalized
END


SUBROUTINE: DetectIntent
INPUT: query (string)
OUTPUT: intent (object)

BEGIN
    intent ← {
        type: "general",
        entities: [],
        mood: null,
        genre: null,
        platform: null
    }

    // Genre detection
    genrePatterns ← [
        {pattern: /\b(action|thriller|adventure)\b/i, genre: "action"},
        {pattern: /\b(comedy|funny|hilarious)\b/i, genre: "comedy"},
        {pattern: /\b(drama|serious)\b/i, genre: "drama"},
        {pattern: /\b(sci-?fi|science fiction|space)\b/i, genre: "sci-fi"},
        {pattern: /\b(horror|scary|creepy)\b/i, genre: "horror"},
        {pattern: /\b(romance|romantic|love)\b/i, genre: "romance"}
    ]

    FOR EACH pattern IN genrePatterns DO
        IF query.match(pattern.pattern) THEN
            intent.genre ← pattern.genre
            BREAK
        END IF
    END FOR

    // Mood detection
    moodPatterns ← [
        {pattern: /\b(relax|chill|calm)\b/i, mood: "relaxing"},
        {pattern: /\b(exciting|thrilling|intense)\b/i, mood: "exciting"},
        {pattern: /\b(feel-?good|uplifting|happy)\b/i, mood: "uplifting"},
        {pattern: /\b(dark|gritty|serious)\b/i, mood: "dark"}
    ]

    FOR EACH pattern IN moodPatterns DO
        IF query.match(pattern.pattern) THEN
            intent.mood ← pattern.mood
            BREAK
        END IF
    END FOR

    // Platform detection
    platformPatterns ← [
        {pattern: /\b(netflix)\b/i, platform: "netflix"},
        {pattern: /\b(disney|disney\+|disneyplus)\b/i, platform: "disney_plus"},
        {pattern: /\b(hulu)\b/i, platform: "hulu"},
        {pattern: /\b(prime|amazon)\b/i, platform: "amazon_prime"},
        {pattern: /\b(hbo|max)\b/i, platform: "hbo_max"}
    ]

    FOR EACH pattern IN platformPatterns DO
        IF query.match(pattern.pattern) THEN
            intent.platform ← pattern.platform
            BREAK
        END IF
    END FOR

    // Entity extraction (titles, actors, directors)
    entities ← ExtractEntities(query)
    intent.entities ← entities

    RETURN intent
END


SUBROUTINE: KeywordSearch
INPUT: tokens (array), filters (object)
OUTPUT: candidates (array)

BEGIN
    candidates ← SET()

    // Look up each token in inverted index
    FOR EACH token IN tokens DO
        matches ← SearchIndex.get(token)

        IF matches is not null THEN
            candidates ← candidates UNION matches
        END IF
    END FOR

    // Convert to array and add keyword match count
    results ← []
    FOR EACH contentId IN candidates DO
        content ← GetContentById(contentId)
        matchCount ← CountTokenMatches(content, tokens)

        results.append({
            content: content,
            keywordMatches: matchCount
        })
    END FOR

    RETURN results
END


SUBROUTINE: SemanticSearch
INPUT: query (string), filters (object)
OUTPUT: candidates (array)

BEGIN
    // Generate query embedding
    queryEmbedding ← GenerateEmbedding(query)

    // Search vector store for similar content
    // Using approximate nearest neighbor search
    vectorResults ← VectorStore.search(
        queryEmbedding,
        topK: 100,
        filters: filters
    )

    // Convert to candidate format
    candidates ← []
    FOR EACH result IN vectorResults DO
        candidates.append({
            content: result.document,
            semanticSimilarity: result.score
        })
    END FOR

    RETURN candidates
END


SUBROUTINE: MergeCandidates
INPUT: keywordCandidates (array), semanticCandidates (array)
OUTPUT: merged (array)

BEGIN
    // Use map to deduplicate by content ID
    candidateMap ← new Map()

    // Add keyword candidates
    FOR EACH candidate IN keywordCandidates DO
        contentId ← candidate.content.entity_id

        candidateMap.set(contentId, {
            content: candidate.content,
            keywordMatches: candidate.keywordMatches,
            semanticSimilarity: 0
        })
    END FOR

    // Merge semantic candidates
    FOR EACH candidate IN semanticCandidates DO
        contentId ← candidate.content.entity_id

        IF candidateMap.has(contentId) THEN
            // Update existing entry
            existing ← candidateMap.get(contentId)
            existing.semanticSimilarity ← candidate.semanticSimilarity
        ELSE
            // Add new entry
            candidateMap.set(contentId, {
                content: candidate.content,
                keywordMatches: 0,
                semanticSimilarity: candidate.semanticSimilarity
            })
        END IF
    END FOR

    // Convert map to array
    merged ← Array.from(candidateMap.values())

    RETURN merged
END


SUBROUTINE: ApplyFilters
INPUT: candidates (array), filters (object)
OUTPUT: filtered (array)

BEGIN
    filtered ← []

    FOR EACH candidate IN candidates DO
        content ← candidate.content
        passesFilters ← true

        // Type filter (movie, tv_show, etc.)
        IF filters.type is defined THEN
            IF content.content_type NOT EQUALS filters.type THEN
                passesFilters ← false
            END IF
        END IF

        // Genre filter
        IF filters.genre is defined AND passesFilters THEN
            IF filters.genre NOT IN content.genres THEN
                passesFilters ← false
            END IF
        END IF

        // Year range filter
        IF filters.year_min is defined AND passesFilters THEN
            IF content.release_year < filters.year_min THEN
                passesFilters ← false
            END IF
        END IF

        IF filters.year_max is defined AND passesFilters THEN
            IF content.release_year > filters.year_max THEN
                passesFilters ← false
            END IF
        END IF

        // Platform filter
        IF filters.platform is defined AND passesFilters THEN
            hasPlatform ← false
            FOR EACH availability IN content.availability DO
                IF availability.platform EQUALS filters.platform THEN
                    hasplatform ← true
                    BREAK
                END IF
            END FOR

            IF NOT hasplatform THEN
                passesFilters ← false
            END IF
        END IF

        // Rating filter (e.g., "PG", "PG-13", "R")
        IF filters.rating is defined AND passesFilters THEN
            IF content.rating NOT EQUALS filters.rating THEN
                passesFilters ← false
            END IF
        END IF

        // Minimum IMDb rating
        IF filters.min_rating is defined AND passesFilters THEN
            IF content.imdb_rating < filters.min_rating THEN
                passesFilters ← false
            END IF
        END IF

        // Add to filtered results if passed all filters
        IF passesFilters THEN
            filtered.append(candidate)
        END IF
    END FOR

    RETURN filtered
END


SUBROUTINE: CalculateRelevanceScore
INPUT: candidate (object), query (string), intent (object)
OUTPUT: score (float)

BEGIN
    score ← 0.0
    content ← candidate.content

    // Keyword match score (weight: 0.3)
    keywordScore ← candidate.keywordMatches * 2
    score ← score + (keywordScore * 0.3)

    // Semantic similarity score (weight: 0.4)
    semanticScore ← candidate.semanticSimilarity * 10
    score ← score + (semanticScore * 0.4)

    // Title exact match bonus
    IF query.toLowerCase() IN content.title.toLowerCase() THEN
        score ← score + 5
    END IF

    // Genre match bonus
    IF intent.genre is not null AND intent.genre IN content.genres THEN
        score ← score + 3
    END IF

    // Popularity boost (weight: 0.2)
    popularityScore ← NormalizePopularity(content.popularity)
    score ← score + (popularityScore * 0.2)

    // Rating boost (weight: 0.1)
    ratingScore ← content.imdb_rating OR 0
    score ← score + (ratingScore * 0.1)

    // Recency boost for recent content
    IF content.release_year >= (CurrentYear - 2) THEN
        score ← score + 1
    END IF

    // Platform availability boost
    IF intent.platform is not null THEN
        FOR EACH availability IN content.availability DO
            IF availability.platform EQUALS intent.platform THEN
                score ← score + 2
                BREAK
            END IF
        END FOR
    END IF

    RETURN score
END


SUBROUTINE: EnrichWithAvailability
INPUT: results (array), authContext (object)
OUTPUT: enriched (array)

BEGIN
    // Get user's region (default to US)
    region ← authContext.region OR "US"

    enriched ← []

    FOR EACH result IN results DO
        content ← result.content

        // Get platform availability for user's region
        availability ← GetAvailabilityByRegion(content.entity_id, region)

        // Add availability to content
        content.availability ← availability
        content.available_platforms ← ExtractPlatformNames(availability)

        enriched.append(result)
    END FOR

    RETURN enriched
END


SUBROUTINE: FormatSearchResults
INPUT: results (array)
OUTPUT: formatted (object)

BEGIN
    items ← []

    FOR EACH result IN results DO
        content ← result.content

        items.append({
            entity_id: content.entity_id,
            title: content.title,
            type: content.content_type,
            release_year: content.release_year,
            genres: content.genres,
            rating: content.rating,
            imdb_rating: content.imdb_rating,
            description: content.description,
            poster_url: content.poster_url,
            availability: content.availability,
            relevance_score: result.score
        })
    END FOR

    RETURN {
        results: items,
        total: items.length,
        query: originalQuery
    }
END
```

## 2. Complexity Analysis

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| NormalizeQuery | O(n) | n = query length |
| Tokenize | O(n) | n = query length |
| DetectIntent | O(n * p) | p = number of patterns |
| KeywordSearch | O(k * t) | k = tokens, t = avg matches per token |
| SemanticSearch | O(log m) | m = total documents (ANN search) |
| MergeCandidates | O(c) | c = total candidates |
| ApplyFilters | O(c * f) | f = number of filters |
| CalculateRelevanceScore | O(1) | Fixed calculations |
| EnrichWithAvailability | O(r) | r = result count |
| **Total** | **O(n + k*t + log m + c*f + r)** | Dominated by keyword search |

### Space Complexity

| Component | Complexity | Notes |
|-----------|-----------|-------|
| Query tokens | O(k) | k = token count |
| Keyword candidates | O(t) | t = matching documents |
| Semantic candidates | O(s) | s = top K results (typically 100) |
| Merged candidates | O(c) | c = unique candidates |
| Scored results | O(c) | Same as candidates |
| **Total** | **O(k + t + s + c)** | Typically c < 1000 |

### Optimization Strategies

1. **Index Optimization**
   - Use inverted index with posting lists
   - Implement skip pointers for efficient intersection
   - Partition index by content type for faster filtering

2. **Vector Search Optimization**
   - Use approximate nearest neighbor (ANN) algorithms (HNSW, IVF)
   - Implement product quantization for memory efficiency
   - Pre-filter vectors by metadata before ANN search

3. **Caching Strategy**
   - Cache popular queries with TTL
   - Cache embeddings for common queries
   - Cache availability data with regional partitioning

4. **Query Processing**
   - Parallel execution of keyword and semantic search
   - Early termination for low-scoring candidates
   - Batch availability lookups

5. **Database Queries**
   - Use prepared statements
   - Implement connection pooling
   - Add compound indexes on (type, genre, year, rating)

### Performance Targets

- Query latency: < 200ms (p95)
- Cache hit rate: > 60%
- Throughput: 1000+ queries/second
- Index size: < 2GB for 100K documents
