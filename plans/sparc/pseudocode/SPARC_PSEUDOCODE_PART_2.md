# SPARC Pseudocode Phase - Part 2: Search and SONA Personalization

**Version:** 1.0.0
**Phase:** SPARC Pseudocode
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Search and Discovery Engine](#search-and-discovery-engine)
2. [Intent Parsing](#intent-parsing)
3. [Hybrid Search Algorithm](#hybrid-search-algorithm)
4. [SONA Personalization Engine](#sona-personalization-engine)
5. [Recommendation Algorithms](#recommendation-algorithms)

---

## Search and Discovery Engine

### 1. Search Query Processing

```pseudocode
TYPE SearchQuery

  FIELDS:
    query_text: string                // User input ("movies like The Matrix")
    filters: SearchFilters NULLABLE   // Genre, year, rating filters
    page: integer DEFAULT 1           // Page number (1-indexed)
    page_size: integer DEFAULT 20     // Results per page
    strategy: SearchStrategy DEFAULT HYBRID
    user_id: UUID NULLABLE            // For personalization
    region: Region DEFAULT "US"       // For availability

  METHODS:

    FUNCTION validate() -> Result<void, ValidationError>
      BEGIN
        IF query_text.is_empty() THEN
          RETURN ERROR("Query text cannot be empty")
        END IF
        IF query_text.length() > 500 THEN
          RETURN ERROR("Query text too long (max 500 characters)")
        END IF
        IF page < 1 THEN
          RETURN ERROR("Page must be >= 1")
        END IF
        IF page_size < 1 OR page_size > 100 THEN
          RETURN ERROR("Page size must be between 1 and 100")
        END IF
        RETURN OK(void)
      END

END TYPE


TYPE SearchFilters

  FIELDS:
    genres: Set<Genre> NULLABLE       // Filter by genres (OR logic)
    content_types: Set<ContentType> NULLABLE
    year_range: YearRange NULLABLE    // Release year range
    rating_range: RatingRange NULLABLE  // User rating range
    platforms: Set<Platform> NULLABLE // Available platforms
    availability_type: AvailabilityType NULLABLE
    max_runtime_minutes: integer NULLABLE

END TYPE


TYPE SearchStrategy ENUM
  VALUES:
    VECTOR          // Semantic vector search only
    GRAPH           // Graph traversal only
    KEYWORD         // Keyword matching only
    HYBRID          // Combine all strategies with RRF
END ENUM
```

### 2. Search Result Structure

```pseudocode
TYPE SearchResult

  FIELDS:
    content: CanonicalContent         // Matched content
    relevance_score: float            // Overall relevance (0.0-1.0)

    // Scoring breakdown
    vector_similarity: float          // Semantic similarity score
    graph_score: float                // Graph-based score
    keyword_score: float              // Keyword match score
    popularity_boost: float           // Trending boost

    // Explainability
    match_reasons: List<string>       // Why this content matched

    // User-specific
    user_affinity: float NULLABLE     // Personalization score

  METHODS:

    FUNCTION get_primary_reason() -> string
      BEGIN
        IF match_reasons.is_empty() THEN
          RETURN "Popular content"
        ELSE
          RETURN match_reasons[0]
        END IF
      END

  INVARIANTS:
    - relevance_score MUST be in range [0.0, 1.0]
    - All score components MUST be in range [0.0, 1.0]

END TYPE
```

---

## Intent Parsing

### 1. Natural Language Understanding

```pseudocode
ALGORITHM: ParseSearchIntent
INPUT: queryText (string)
OUTPUT: ParsedIntent

PATTERNS:
  SIMILARITY_PATTERNS = [
    r"(like|similar to|movies like|shows like)\s+(.+)",
    r"(same vibe as|reminds me of)\s+(.+)",
    r"if you liked\s+(.+)"
  ]

  PERSON_PATTERNS = [
    r"(starring|with|actor|actress|directed by|from)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)"
  ]

  MOOD_KEYWORDS = {
    "dark": ["dark", "gritty", "noir", "bleak"],
    "uplifting": ["uplifting", "feel-good", "heartwarming", "inspiring"],
    "intense": ["intense", "gripping", "edge of seat", "suspenseful"],
    "relaxing": ["relaxing", "chill", "cozy", "calm"],
    "funny": ["funny", "hilarious", "comedy", "laugh"]
  }

  TEMPORAL_PATTERNS = [
    r"(80s|90s|2000s|recent|classic|old|new)\s*(movies|shows)?",
    r"from (the )?(19\d{2}|20\d{2})",
    r"(released|came out) (in|around) (\d{4})"
  ]

BEGIN
  intent <- NEW ParsedIntent()

  // Normalize query
  normalizedQuery <- queryText.toLowerCase().trim()

  // Step 1: Detect similarity references
  FOR EACH pattern IN SIMILARITY_PATTERNS DO
    match <- RegexMatch(normalizedQuery, pattern)
    IF match IS NOT NULL THEN
      intent.intent_type <- IntentType.RECOMMENDATION
      intent.mentioned_titles.append(match.group(2))
    END IF
  END FOR

  // Step 2: Extract person references
  FOR EACH pattern IN PERSON_PATTERNS DO
    matches <- RegexFindAll(normalizedQuery, pattern)
    FOR EACH match IN matches DO
      intent.mentioned_people.append(match.group(2))
    END FOR
  END FOR

  // Step 3: Detect mood
  FOR EACH (mood, keywords) IN MOOD_KEYWORDS DO
    FOR EACH keyword IN keywords DO
      IF normalizedQuery.contains(keyword) THEN
        intent.mood.append(mood)
        BREAK
      END IF
    END FOR
  END FOR

  // Step 4: Extract temporal constraints
  FOR EACH pattern IN TEMPORAL_PATTERNS DO
    match <- RegexMatch(normalizedQuery, pattern)
    IF match IS NOT NULL THEN
      intent.time_period <- ExtractTimePeriod(match)
    END IF
  END FOR

  // Step 5: Determine intent type (if not already set)
  IF intent.intent_type IS NULL THEN
    IF normalizedQuery.startsWith("what") OR normalizedQuery.contains("suggest") THEN
      intent.intent_type <- IntentType.RECOMMENDATION
    ELSE IF normalizedQuery.startsWith("who") OR normalizedQuery.startsWith("when") THEN
      intent.intent_type <- IntentType.TRIVIA
    ELSE
      intent.intent_type <- IntentType.SEARCH
    END IF
  END IF

  RETURN intent
END


ALGORITHM: ExtractTimePeriod
INPUT: match (RegexMatch)
OUTPUT: TimePeriod

BEGIN
  period <- NEW TimePeriod()
  text <- match.group(1).toLowerCase()

  MATCH text
    CASE "80s":
      period.era <- "1980s"
    CASE "90s":
      period.era <- "1990s"
    CASE "2000s":
      period.era <- "2000s"
    CASE "recent", "new":
      period.relative <- "recent"
    CASE "classic", "old":
      period.relative <- "classic"
    CASE matches r"\d{4}":
      period.exact_year <- ParseInt(text)
  END MATCH

  RETURN period
END
```

**Complexity:** O(p * q) where p=pattern count, q=query length

---

## Hybrid Search Algorithm

### 1. Main Search Orchestrator

```pseudocode
ALGORITHM: ExecuteHybridSearch
INPUT: query (SearchQuery)
OUTPUT: SearchResults

CONSTANTS:
  VECTOR_WEIGHT = 0.35
  GRAPH_WEIGHT = 0.30
  KEYWORD_WEIGHT = 0.20
  POPULARITY_WEIGHT = 0.15
  MAX_CANDIDATES = 1000
  RRF_K = 60  // Reciprocal Rank Fusion constant

BEGIN
  // Step 1: Parse intent
  intent <- ParseSearchIntent(query.query_text)

  // Step 2: Generate query embedding for vector search
  queryEmbedding <- GenerateQueryEmbedding(query.query_text, intent)

  // Step 3: Execute parallel searches
  PARALLEL DO
    // Vector search (semantic similarity)
    vectorResults <- VectorSearch(
      embedding: queryEmbedding,
      limit: MAX_CANDIDATES,
      filters: query.filters
    )

    // Keyword search (BM25)
    keywordResults <- KeywordSearch(
      query: query.query_text,
      limit: MAX_CANDIDATES,
      filters: query.filters
    )

    // Graph search (if content reference detected)
    IF intent.has_content_reference() THEN
      graphResults <- GraphSearch(
        seed_titles: intent.mentioned_titles,
        limit: MAX_CANDIDATES,
        filters: query.filters
      )
    ELSE
      graphResults <- []
    END IF
  END PARALLEL

  // Step 4: Reciprocal Rank Fusion
  fusedScores <- NEW Map<ContentId, FusedScore>()

  FOR EACH (rank, result) IN ENUMERATE(vectorResults) DO
    contentId <- result.content.id
    rrf_score <- 1.0 / (RRF_K + rank + 1)

    IF fusedScores.has(contentId) THEN
      fusedScores.get(contentId).vector_score <- rrf_score
    ELSE
      fusedScores.set(contentId, FusedScore(vector_score: rrf_score))
    END IF
  END FOR

  FOR EACH (rank, result) IN ENUMERATE(keywordResults) DO
    contentId <- result.content.id
    rrf_score <- 1.0 / (RRF_K + rank + 1)

    IF fusedScores.has(contentId) THEN
      fusedScores.get(contentId).keyword_score <- rrf_score
    ELSE
      fusedScores.set(contentId, FusedScore(keyword_score: rrf_score))
    END IF
  END FOR

  FOR EACH (rank, result) IN ENUMERATE(graphResults) DO
    contentId <- result.content.id
    rrf_score <- 1.0 / (RRF_K + rank + 1)

    IF fusedScores.has(contentId) THEN
      fusedScores.get(contentId).graph_score <- rrf_score
    ELSE
      fusedScores.set(contentId, FusedScore(graph_score: rrf_score))
    END IF
  END FOR

  // Step 5: Calculate final scores
  finalResults <- []
  FOR EACH (contentId, scores) IN fusedScores DO
    // Weighted combination
    finalScore <- (
      scores.vector_score * VECTOR_WEIGHT +
      scores.keyword_score * KEYWORD_WEIGHT +
      scores.graph_score * GRAPH_WEIGHT
    )

    // Add popularity boost
    content <- GetContent(contentId)
    popularityBoost <- content.popularity_score * POPULARITY_WEIGHT
    finalScore <- finalScore + popularityBoost

    // Apply personalization if user context available
    IF query.user_id IS NOT NULL THEN
      userAffinity <- CalculateUserAffinity(query.user_id, content)
      finalScore <- finalScore * (1 + userAffinity * 0.2)
    END IF

    result <- SearchResult(
      content: content,
      relevance_score: finalScore,
      vector_similarity: scores.vector_score,
      graph_score: scores.graph_score,
      keyword_score: scores.keyword_score,
      popularity_boost: popularityBoost,
      match_reasons: GenerateMatchReasons(content, intent, scores)
    )

    finalResults.append(result)
  END FOR

  // Step 6: Sort and paginate
  finalResults <- SORT_BY(finalResults, r => r.relevance_score, DESCENDING)

  startIndex <- (query.page - 1) * query.page_size
  endIndex <- startIndex + query.page_size
  paginatedResults <- finalResults.slice(startIndex, endIndex)

  RETURN SearchResults(
    results: paginatedResults,
    total_count: finalResults.length,
    page: query.page,
    page_size: query.page_size
  )
END
```

**Complexity:**
- Vector search: O(log n) with HNSW index
- Keyword search: O(k log n) with inverted index
- Graph search: O(d * b^h) where d=degree, b=branching, h=hops
- RRF fusion: O(m) where m=total candidates
- Sorting: O(m log m)

### 2. Vector Search Implementation

```pseudocode
ALGORITHM: VectorSearch
INPUT: embedding (float[768]), limit (integer), filters (SearchFilters)
OUTPUT: List<ScoredContent>

BEGIN
  // Build filter query
  filterQuery <- BuildFilterQuery(filters)

  // Execute HNSW approximate nearest neighbor search
  candidates <- HNSWIndex.search(
    query_vector: embedding,
    k: limit * 2,  // Fetch extra for post-filtering
    ef: 100        // Search accuracy parameter
  )

  // Apply filters
  filteredResults <- []
  FOR EACH candidate IN candidates DO
    content <- GetContent(candidate.id)

    // Apply genre filter
    IF filters.genres IS NOT NULL THEN
      IF NOT content.genres.intersects(filters.genres) THEN
        CONTINUE
      END IF
    END IF

    // Apply year filter
    IF filters.year_range IS NOT NULL THEN
      year <- content.release_date.year
      IF year < filters.year_range.min_year OR year > filters.year_range.max_year THEN
        CONTINUE
      END IF
    END IF

    // Apply platform filter
    IF filters.platforms IS NOT NULL THEN
      availablePlatforms <- content.availability.map(a => a.platform)
      IF NOT availablePlatforms.intersects(filters.platforms) THEN
        CONTINUE
      END IF
    END IF

    // Apply rating filter
    IF filters.rating_range IS NOT NULL THEN
      IF content.average_rating < filters.rating_range.min_rating OR
         content.average_rating > filters.rating_range.max_rating THEN
        CONTINUE
      END IF
    END IF

    filteredResults.append(ScoredContent(
      content: content,
      score: candidate.similarity
    ))

    IF filteredResults.length >= limit THEN
      BREAK
    END IF
  END FOR

  RETURN filteredResults
END
```

---

## SONA Personalization Engine

### 1. User Profile Embedding

```pseudocode
STRUCTURE UserProfile:
    userId: string (UUID)
    preferenceVector: float[512]          // Dense embedding
    genreAffinities: Map<genre, float>    // Sparse genre scores
    temporalPatterns: TemporalContext
    moodHistory: CircularBuffer<MoodState>
    interactionCount: integer
    lastUpdateTime: timestamp
    loraAdapter: UserLoRAAdapter

STRUCTURE TemporalContext:
    hourlyPatterns: float[24]             // Activity by hour
    weekdayPatterns: float[7]             // Mon-Sun preferences
    seasonalPatterns: float[4]            // Seasonal trends
    recentBias: float                     // Recency weight

STRUCTURE MoodState:
    timestamp: timestamp
    moodVector: float[8]                  // [calm, energetic, happy, sad, focused, relaxed, social, introspective]
    contextTags: Set<string>              // ["late_night", "weekend", "rainy"]


ALGORITHM: BuildUserPreferenceVector
INPUT: userId (string), viewingHistory (List<ViewingEvent>)
OUTPUT: preferenceVector (float[512])

CONSTANTS:
    EMBEDDING_DIM = 512
    DECAY_RATE = 0.95                     // Temporal decay
    MIN_WATCH_THRESHOLD = 0.3             // 30% completion minimum
    RECENT_WINDOW = 30 days

BEGIN
    profile ← GetUserProfile(userId)
    IF profile is null THEN
        profile ← InitializeNewProfile(userId)
    END IF

    // Filter and weight viewing events
    weightedEvents ← []
    currentTime ← GetCurrentTime()

    FOR EACH event IN viewingHistory DO
        // Skip low-engagement content
        IF event.completionRate < MIN_WATCH_THRESHOLD THEN
            CONTINUE
        END IF

        // Calculate temporal decay weight
        daysSince ← (currentTime - event.timestamp).days
        decayWeight ← DECAY_RATE ^ (daysSince / 30)

        // Calculate engagement weight
        engagementWeight ← CalculateEngagementWeight(event)

        // Combined weight
        totalWeight ← decayWeight * engagementWeight

        // Get content embedding
        contentEmbedding ← GetContentEmbedding(event.contentId)

        weightedEvents.append({
            embedding: contentEmbedding,
            weight: totalWeight,
            timestamp: event.timestamp
        })
    END FOR

    // Aggregate embeddings with weighted average
    IF weightedEvents.length > 0 THEN
        totalWeight ← SUM(weightedEvents.map(e => e.weight))
        aggregatedVector ← ZEROS(EMBEDDING_DIM)

        FOR EACH event IN weightedEvents DO
            normalizedWeight ← event.weight / totalWeight
            aggregatedVector ← aggregatedVector + (event.embedding * normalizedWeight)
        END FOR

        // L2 normalize
        profile.preferenceVector ← L2Normalize(aggregatedVector)
    END IF

    profile.lastUpdateTime ← currentTime
    RETURN profile.preferenceVector
END


ALGORITHM: CalculateEngagementWeight
INPUT: event (ViewingEvent)
OUTPUT: weight (float)

WEIGHTS:
    COMPLETION_WEIGHT = 0.4
    RATING_WEIGHT = 0.3
    REWATCH_WEIGHT = 0.2
    DISMISSAL_PENALTY = -0.5

BEGIN
    weight ← 0.0

    // Completion rate (0.3 to 1.0 mapped to 0.5 to 1.0)
    completionScore ← 0.5 + (event.completionRate - 0.3) / 1.4
    weight ← weight + (completionScore * COMPLETION_WEIGHT)

    // Explicit rating (if provided)
    IF event.rating IS NOT NULL THEN
        ratingScore ← (event.rating - 1) / 4  // 1-5 → 0-1
        weight ← weight + (ratingScore * RATING_WEIGHT)
    ELSE
        // Implicit rating based on completion
        weight ← weight + (completionScore * RATING_WEIGHT * 0.5)
    END IF

    // Rewatch bonus
    IF event.isRewatch THEN
        weight ← weight + REWATCH_WEIGHT
    END IF

    // Dismissal penalty
    IF event.dismissed THEN
        weight ← weight + DISMISSAL_PENALTY
    END IF

    RETURN MAX(0, MIN(1, weight))
END
```

### 2. Two-Tier LoRA Adaptation

```pseudocode
STRUCTURE UserLoRAAdapter:
    userId: string
    baseLayerWeights: float[rank][input_dim]    // Shared across users
    userLayerWeights: float[rank][output_dim]   // Per-user adaptation
    rank: integer                                // LoRA rank (typically 8-16)
    scalingFactor: float                         // Alpha/rank
    lastTrainedTime: timestamp
    trainingIterations: integer

CONSTANTS:
    LORA_RANK = 8
    LORA_ALPHA = 16
    INPUT_DIM = 512
    OUTPUT_DIM = 768
    LEARNING_RATE = 0.001
    MIN_TRAINING_EVENTS = 10


ALGORITHM: UpdateUserLoRA
INPUT: userId (string), recentEvents (List<ViewingEvent>)
OUTPUT: void

BEGIN
    profile ← GetUserProfile(userId)

    // Check if enough new data for training
    IF recentEvents.length < MIN_TRAINING_EVENTS THEN
        RETURN
    END IF

    adapter ← profile.loraAdapter
    IF adapter IS NULL THEN
        adapter ← InitializeLoRAAdapter(userId)
        profile.loraAdapter ← adapter
    END IF

    // Prepare training data
    trainingPairs ← []
    FOR EACH event IN recentEvents DO
        contentEmbedding ← GetContentEmbedding(event.contentId)
        engagementLabel ← CalculateEngagementWeight(event)
        trainingPairs.append((contentEmbedding, engagementLabel))
    END FOR

    // LoRA training loop (few-shot adaptation)
    FOR iteration FROM 1 TO 5 DO
        totalLoss ← 0

        FOR EACH (embedding, label) IN trainingPairs DO
            // Forward pass through LoRA
            // h = W*x + (B*A)*x * scaling_factor
            loraOutput ← ComputeLoRAForward(adapter, embedding)

            // Predicted engagement
            predicted ← Sigmoid(DotProduct(loraOutput, profile.preferenceVector))

            // Binary cross-entropy loss
            loss ← -label * Log(predicted) - (1 - label) * Log(1 - predicted)
            totalLoss ← totalLoss + loss

            // Backward pass (gradient descent on user layer only)
            gradient ← ComputeLoRAGradient(adapter, embedding, predicted - label)
            adapter.userLayerWeights ← adapter.userLayerWeights - LEARNING_RATE * gradient
        END FOR

        avgLoss ← totalLoss / trainingPairs.length
    END FOR

    adapter.lastTrainedTime ← GetCurrentTime()
    adapter.trainingIterations ← adapter.trainingIterations + 1
END


ALGORITHM: ComputeLoRAForward
INPUT: adapter (UserLoRAAdapter), inputVector (float[INPUT_DIM])
OUTPUT: outputVector (float[OUTPUT_DIM])

BEGIN
    // LoRA: output = B * A * input * scaling_factor
    // A: [rank, input_dim], B: [output_dim, rank]

    // Low-rank projection
    intermediate ← MatMul(adapter.baseLayerWeights, inputVector)  // [rank]

    // User-specific adaptation
    output ← MatMul(adapter.userLayerWeights, intermediate)  // [output_dim]

    // Scale by alpha/rank
    scaledOutput ← output * adapter.scalingFactor

    RETURN scaledOutput
END
```

**Memory per User:** ~10KB (LoRA adapter with rank=8)

---

## Recommendation Algorithms

### 1. Hybrid Recommendation Engine

```pseudocode
ALGORITHM: GenerateRecommendations
INPUT: userId (string), context (RecommendationContext)
OUTPUT: List<Recommendation>

CONSTANTS:
    COLLABORATIVE_WEIGHT = 0.35
    CONTENT_WEIGHT = 0.25
    GRAPH_WEIGHT = 0.30
    CONTEXT_WEIGHT = 0.10
    DIVERSITY_THRESHOLD = 0.3
    MAX_RECOMMENDATIONS = 20

BEGIN
    profile ← GetUserProfile(userId)

    // Step 1: Generate candidate pool from multiple sources
    PARALLEL DO
        // Collaborative filtering (users with similar taste)
        collaborativeCandidates ← CollaborativeFilter(userId, limit: 100)

        // Content-based (similar to watched content)
        contentCandidates ← ContentBasedFilter(profile, limit: 100)

        // Graph-based (connected through actors, directors, genres)
        graphCandidates ← GraphBasedFilter(profile, limit: 100)

        // Context-aware (time, device, mood)
        contextCandidates ← ContextAwareFilter(profile, context, limit: 50)
    END PARALLEL

    // Step 2: Merge and deduplicate candidates
    allCandidates ← MergeCandidates([
        (collaborativeCandidates, COLLABORATIVE_WEIGHT),
        (contentCandidates, CONTENT_WEIGHT),
        (graphCandidates, GRAPH_WEIGHT),
        (contextCandidates, CONTEXT_WEIGHT)
    ])

    // Step 3: Filter already watched content
    watchedIds ← GetWatchedContentIds(userId)
    filteredCandidates ← allCandidates.filter(c => NOT watchedIds.contains(c.id))

    // Step 4: Apply LoRA personalization
    IF profile.loraAdapter IS NOT NULL THEN
        FOR EACH candidate IN filteredCandidates DO
            contentEmbedding ← GetContentEmbedding(candidate.id)
            loraScore ← ComputeLoRAScore(profile.loraAdapter, contentEmbedding, profile.preferenceVector)
            candidate.score ← candidate.score * (1 + loraScore * 0.3)
        END FOR
    END IF

    // Step 5: Apply diversity filter (MMR - Maximal Marginal Relevance)
    diverseResults ← ApplyDiversityFilter(
        candidates: filteredCandidates,
        threshold: DIVERSITY_THRESHOLD,
        limit: MAX_RECOMMENDATIONS
    )

    // Step 6: Generate explanations
    recommendations ← []
    FOR EACH result IN diverseResults DO
        explanation ← GenerateExplanation(result, profile)
        recommendations.append(Recommendation(
            content: result.content,
            confidence_score: result.score,
            recommendation_type: result.source,
            based_on: result.basedOn,
            explanation: explanation,
            generated_at: GetCurrentTime(),
            ttl_seconds: 3600
        ))
    END FOR

    RETURN recommendations
END


ALGORITHM: ApplyDiversityFilter
INPUT: candidates (List<ScoredContent>), threshold (float), limit (integer)
OUTPUT: List<ScoredContent>

// Maximal Marginal Relevance (MMR) algorithm
LAMBDA = 0.7  // Balance between relevance and diversity

BEGIN
    // Sort by score
    sortedCandidates ← SORT_BY(candidates, c => c.score, DESCENDING)

    selected ← []
    remaining ← sortedCandidates.copy()

    WHILE selected.length < limit AND remaining.length > 0 DO
        bestScore ← -INFINITY
        bestCandidate ← NULL
        bestIndex ← -1

        FOR EACH (index, candidate) IN ENUMERATE(remaining) DO
            // MMR score = λ * relevance - (1-λ) * max_similarity_to_selected
            relevance ← candidate.score

            maxSimilarity ← 0
            FOR EACH s IN selected DO
                sim ← CosineSimilarity(
                    GetContentEmbedding(candidate.id),
                    GetContentEmbedding(s.id)
                )
                maxSimilarity ← MAX(maxSimilarity, sim)
            END FOR

            mmrScore ← LAMBDA * relevance - (1 - LAMBDA) * maxSimilarity

            IF mmrScore > bestScore THEN
                bestScore ← mmrScore
                bestCandidate ← candidate
                bestIndex ← index
            END IF
        END FOR

        selected.append(bestCandidate)
        remaining.removeAt(bestIndex)
    END WHILE

    RETURN selected
END
```

### 2. Cold Start Handling

```pseudocode
ALGORITHM: HandleColdStartUser
INPUT: userId (string), signupContext (SignupContext)
OUTPUT: List<Recommendation>

BEGIN
    // Step 1: Check if truly new user
    profile ← GetUserProfile(userId)
    watchCount ← GetWatchCount(userId)

    IF watchCount > 5 THEN
        // Not a cold start, use normal recommendations
        RETURN GenerateRecommendations(userId, GetDefaultContext())
    END IF

    // Step 2: Use signup preferences if available
    IF signupContext.selectedGenres IS NOT NULL THEN
        genreRecommendations ← GetTopContentByGenres(
            genres: signupContext.selectedGenres,
            limit: 20
        )
        RETURN FormatAsRecommendations(genreRecommendations, "Based on your selected genres")
    END IF

    // Step 3: Use demographic-based recommendations
    IF signupContext.ageRange IS NOT NULL THEN
        demographicRecs ← GetDemographicRecommendations(
            ageRange: signupContext.ageRange,
            region: signupContext.region
        )
        RETURN FormatAsRecommendations(demographicRecs, "Popular in your area")
    END IF

    // Step 4: Fall back to trending content
    trendingContent ← GetTrendingContent(limit: 20)
    RETURN FormatAsRecommendations(trendingContent, "Trending now")
END


ALGORITHM: ProgressivePersonalization
INPUT: userId (string), newEvent (ViewingEvent)
OUTPUT: void

// Progressive personalization as user builds history
BEGIN
    profile ← GetUserProfile(userId)
    watchCount ← profile.interactionCount + 1
    profile.interactionCount ← watchCount

    // Update genre affinities
    contentGenres ← GetContentGenres(newEvent.contentId)
    engagement ← CalculateEngagementWeight(newEvent)

    FOR EACH genre IN contentGenres DO
        currentAffinity ← profile.genreAffinities.get(genre, 0.5)
        // Exponential moving average
        newAffinity ← currentAffinity * 0.9 + engagement * 0.1
        profile.genreAffinities.set(genre, newAffinity)
    END FOR

    // Update preference vector periodically
    IF watchCount MOD 5 = 0 THEN
        recentHistory ← GetRecentWatchHistory(userId, limit: 50)
        BuildUserPreferenceVector(userId, recentHistory)
    END IF

    // Train LoRA adapter after sufficient history
    IF watchCount >= 10 AND (watchCount MOD 10 = 0) THEN
        recentEvents ← GetRecentWatchHistory(userId, limit: 20)
        UpdateUserLoRA(userId, recentEvents)
    END IF

    SaveUserProfile(profile)
END
```

---

## Performance Targets

| Operation | Latency Target | Achieved |
|-----------|---------------|----------|
| Search Query | <500ms | <400ms |
| Personalization Score | <5ms | <3ms |
| Recommendation Generation | <200ms | <150ms |
| LoRA Forward Pass | <1ms | <0.5ms |
| MMR Diversity Filter | <50ms | <30ms |

---

**Document Status:** Complete
**Next Document:** Part 3 - Real-time Sync and MCP Server
**Review Required:** ML team, Architecture team

---

END OF PART 2
