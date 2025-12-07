# SPARC Phase 2: Master Pseudocode Document

**Version:** 1.0.0
**Phase:** SPARC Pseudocode (Complete)
**Date:** 2025-12-06
**Status:** Complete
**Total Sections:** 16 Major Components

---

## Document Overview

This master document consolidates all pseudocode specifications for the Media Gateway platform. It provides language-agnostic algorithm definitions for all core components, from data structures and ingestion to authentication and error handling.

### Document Structure

| Section | Description | Source |
|---------|-------------|--------|
| 1-5 | Core Data Structures & Ingestion | Part 1 |
| 6-9 | Search & SONA Personalization | Part 2 |
| 10-14 | Real-time Sync & MCP Server | Part 3 |
| 15-16 | Authentication & CLI | Part 4 |

### Key Performance Targets

| Component | Latency Target | Throughput Target |
|-----------|---------------|-------------------|
| Content Lookup | O(1) | 10,000 req/s |
| Entity Resolution | O(log n) | 1,000 entities/s |
| Embedding Generation | O(d) | 500 items/s |
| CRDT Merge | O(m) | 10,000 ops/s |
| Search Query | <500ms | <400ms |
| Personalization Score | <5ms | <3ms |
| Recommendation Generation | <200ms | <150ms |
| HLC Operations | <1ms | O(1) |
| MCP Request | <150ms | O(1) core |

---

# PART 1: Core Data Structures and Ingestion

---

## 1. Core Data Structures

### 1.1 CanonicalContent

**Purpose:** Unified representation of media content across all streaming platforms.

**Design Decision:** Single source of truth for content metadata, normalized from multiple platform-specific formats.

```pseudocode
TYPE CanonicalContent

  FIELDS:
    // Primary identification
    id: UUID                          // Internal unique identifier
    content_type: ContentType         // ENUM(movie, series, episode, short)

    // Core metadata
    title: string                     // Primary title (user's preferred language)
    original_title: string            // Original language title
    overview: string                  // Plot summary/description
    tagline: string NULLABLE          // Marketing tagline

    // Temporal metadata
    release_date: Date                // Initial release date
    premiere_date: Date NULLABLE      // Streaming premiere (if different)
    last_updated: Timestamp           // Metadata last modified

    // External identifiers (cross-referencing)
    external_ids: ExternalIds         // EIDR, IMDb, TMDb, etc.

    // Classification
    genres: List<Genre>               // Comedy, Drama, Sci-Fi, etc.
    themes: List<string>              // Revenge, redemption, coming-of-age
    moods: List<string>               // Dark, uplifting, intense

    // People
    credits: Credits                  // Cast and crew

    // Media assets
    images: ContentImages             // Posters, backdrops, stills

    // Content ratings (regional)
    ratings: Map<Region, ContentRating>  // US: PG-13, UK: 12A, etc.

    // Runtime
    runtime_minutes: integer NULLABLE    // Total runtime (null for series)

    // Metrics
    popularity_score: float           // Trending score (0.0-1.0)
    average_rating: float             // User ratings average (0.0-10.0)
    vote_count: integer               // Number of ratings

    // Platform availability
    availability: List<PlatformAvailability>  // Where to watch

    // Series-specific fields (null for movies)
    series_metadata: SeriesMetadata NULLABLE

  INVARIANTS:
    - id MUST be unique across all content
    - content_type determines which optional fields are populated
    - external_ids MUST contain at least one identifier
    - release_date MUST be <= current_date for released content
    - popularity_score MUST be in range [0.0, 1.0]
    - average_rating MUST be in range [0.0, 10.0]

END TYPE


TYPE ContentType ENUM
  VALUES:
    MOVIE           // Feature film
    SERIES          // TV series (container for seasons)
    EPISODE         // Individual episode
    SHORT           // Short film (<45 minutes)
    DOCUMENTARY     // Documentary content
    SPECIAL         // Special episode or one-off
END ENUM
```

**Complexity Analysis:**
- Storage: O(1) per content item
- Retrieval by ID: O(1) with hash index
- Search by genre: O(n) without index, O(log n) with B-tree on genres

### 1.2 External Identifiers

```pseudocode
TYPE ExternalIds

  FIELDS:
    eidr_id: string NULLABLE          // Entertainment Identifier Registry
                                       // Format: "10.5240/XXXX-XXXX-XXXX-XXXX-XXXX-X"

    imdb_id: string NULLABLE          // IMDb identifier
                                       // Format: "tt0123456" (7+ digits)

    tmdb_id: integer NULLABLE         // The Movie Database ID
                                       // Format: positive integer

    tvdb_id: integer NULLABLE         // TheTVDB identifier
                                       // Format: positive integer

    gracenote_tms_id: string NULLABLE // Gracenote TMS ID
                                       // Format: "MV001234560000" or "SH012345670000"

    platform_ids: Map<Platform, string>  // Platform-specific IDs
                                          // Netflix: "80123456"
                                          // Prime: "amzn1.dv.gti.12345"
                                          // Disney+: "series-uuid"

  METHODS:

    FUNCTION has_identifier() -> boolean
      // At least one identifier must be present
      BEGIN
        RETURN eidr_id IS NOT NULL OR
               imdb_id IS NOT NULL OR
               tmdb_id IS NOT NULL OR
               tvdb_id IS NOT NULL OR
               gracenote_tms_id IS NOT NULL OR
               NOT platform_ids.is_empty()
      END

    FUNCTION get_canonical_id() -> string
      // Prefer EIDR as most authoritative, fallback to others
      BEGIN
        IF eidr_id IS NOT NULL THEN
          RETURN eidr_id
        ELSE IF imdb_id IS NOT NULL THEN
          RETURN imdb_id
        ELSE IF tmdb_id IS NOT NULL THEN
          RETURN "tmdb:" + string(tmdb_id)
        ELSE
          RETURN "unknown"
        END IF
      END

  INVARIANTS:
    - At least one identifier MUST be non-null
    - imdb_id MUST match pattern "tt[0-9]{7,}"
    - eidr_id MUST match pattern "10.5240/[A-F0-9-]+"

END TYPE


TYPE Platform ENUM
  VALUES:
    NETFLIX
    PRIME_VIDEO
    DISNEY_PLUS
    HULU
    APPLE_TV_PLUS
    HBO_MAX
    PEACOCK
    PARAMOUNT_PLUS
    YOUTUBE
    CRAVE
    BBC_IPLAYER
END ENUM
```

### 1.3 Platform Availability

```pseudocode
TYPE PlatformAvailability

  FIELDS:
    platform: Platform                // Streaming platform
    region: Region                    // ISO 3166-1 alpha-2 country code
    availability_type: AvailabilityType  // subscription, rental, purchase, free

    // Pricing (null for subscription/free)
    price: Money NULLABLE             // Rental or purchase price

    // Deep linking
    deep_link: URL                    // Platform-specific deep link
    web_fallback: URL                 // HTTPS fallback URL

    // Temporal availability
    available_from: Timestamp         // When content became available
    expires_at: Timestamp NULLABLE    // When content leaves platform

    // Quality metadata
    video_quality: Set<VideoQuality>  // SD, HD, UHD, HDR
    audio_tracks: Set<AudioTrack>     // Available audio languages
    subtitle_tracks: Set<SubtitleTrack>  // Available subtitles

  METHODS:

    FUNCTION is_currently_available() -> boolean
      BEGIN
        current_time <- GET_CURRENT_TIMESTAMP()
        IF current_time < available_from THEN
          RETURN false
        END IF
        IF expires_at IS NOT NULL AND current_time > expires_at THEN
          RETURN false
        END IF
        RETURN true
      END

    FUNCTION days_until_expiry() -> integer NULLABLE
      BEGIN
        IF expires_at IS NULL THEN
          RETURN NULL  // Never expires
        END IF
        current_time <- GET_CURRENT_TIMESTAMP()
        IF current_time > expires_at THEN
          RETURN 0  // Already expired
        END IF
        delta <- expires_at - current_time
        RETURN CEILING(delta.total_seconds() / 86400)
      END

  INVARIANTS:
    - available_from MUST be <= expires_at (if expires_at is not null)
    - price MUST be null for SUBSCRIPTION and FREE types
    - price MUST be non-null for RENTAL and PURCHASE types

END TYPE


TYPE AvailabilityType ENUM
  VALUES:
    SUBSCRIPTION    // Included with platform subscription
    RENTAL          // Time-limited rental (24-48 hours)
    PURCHASE        // Permanent ownership
    FREE            // Free with ads (AVOD)
END ENUM
```

### 1.4 User Profile

```pseudocode
TYPE UserProfile

  FIELDS:
    user_id: UUID                     // Internal user identifier
    external_auth_id: string          // OAuth provider ID

    // Account metadata
    created_at: Timestamp             // Account creation date
    last_active: Timestamp            // Last interaction timestamp

    // User preferences
    preferences: UserPreferences      // Genre preferences, platform subscriptions

    // Privacy settings
    privacy_settings: PrivacySettings // GDPR/CCPA consent flags

    // Devices
    devices: List<Device>             // Registered devices for sync

  METHODS:

    FUNCTION is_active_user() -> boolean
      // User is considered active if last activity within 30 days
      BEGIN
        current_time <- GET_CURRENT_TIMESTAMP()
        delta <- current_time - last_active
        RETURN delta.total_days() <= 30
      END

    FUNCTION update_last_active() -> void
      BEGIN
        last_active <- GET_CURRENT_TIMESTAMP()
      END

  INVARIANTS:
    - created_at MUST be <= last_active
    - user_id MUST be globally unique

END TYPE


TYPE UserPreferences

  FIELDS:
    favorite_genres: Set<Genre>       // User-selected favorite genres
    disliked_genres: Set<Genre>       // Genres to avoid
    preferred_languages: List<LanguageCode>  // Ordered by preference
    subscribed_platforms: Set<Platform>  // Active subscriptions
    max_content_rating: ContentRating NULLABLE  // Age filter
    preferred_video_quality: VideoQuality  // Preferred streaming quality
    autoplay_next: boolean               // Auto-play next episode

  METHODS:

    FUNCTION add_favorite_genre(genre: Genre) -> void
      BEGIN
        favorite_genres.add(genre)
        disliked_genres.remove(genre)  // Can't be both
      END

    FUNCTION has_platform_subscription(platform: Platform) -> boolean
      BEGIN
        RETURN subscribed_platforms.contains(platform)
      END

  INVARIANTS:
    - favorite_genres and disliked_genres MUST be disjoint sets

END TYPE
```

---

## 2. Ingestion Pipeline Algorithms

### 2.1 Platform Normalizer

```pseudocode
ALGORITHM: NormalizeToCanonical
INPUT: platformData (RawPlatformContent), platform (Platform)
OUTPUT: CanonicalContent

CONSTANTS:
  DEFAULT_POPULARITY = 0.5
  MAX_GENRES = 5
  MAX_CAST = 20

BEGIN
  canonical <- NEW CanonicalContent()

  // Generate internal ID
  canonical.id <- GenerateUUIDv5(
    namespace: CONTENT_NAMESPACE,
    name: platform.name + ":" + platformData.platform_id
  )

  // Map content type
  canonical.content_type <- MapContentType(platformData.type, platform)

  // Normalize title
  canonical.title <- NormalizeTitle(platformData.title)
  canonical.original_title <- platformData.original_title ?? canonical.title

  // Normalize description
  canonical.overview <- SanitizeHTML(platformData.description)
  canonical.overview <- TruncateWithEllipsis(canonical.overview, 1000)

  // Parse release date
  canonical.release_date <- ParseDateFlexible(platformData.release_date)

  // Map external IDs
  canonical.external_ids <- MapExternalIds(platformData, platform)

  // Normalize genres
  raw_genres <- platformData.genres ?? []
  canonical.genres <- MapGenres(raw_genres, platform)
  canonical.genres <- canonical.genres.slice(0, MAX_GENRES)

  // Extract themes and moods (NLP-based)
  IF platformData.description IS NOT NULL THEN
    themes_moods <- ExtractThemesMoods(platformData.description)
    canonical.themes <- themes_moods.themes
    canonical.moods <- themes_moods.moods
  END IF

  // Normalize credits
  canonical.credits <- NormalizeCredits(platformData.cast, platformData.crew, MAX_CAST)

  // Map images
  canonical.images <- MapImages(platformData.images, platform)

  // Map ratings
  canonical.ratings <- MapContentRatings(platformData.ratings, platform)

  // Set runtime
  IF platformData.runtime IS NOT NULL THEN
    canonical.runtime_minutes <- ConvertToMinutes(platformData.runtime)
  END IF

  // Calculate popularity score
  IF platformData.popularity IS NOT NULL THEN
    canonical.popularity_score <- NormalizePopularity(platformData.popularity, platform)
  ELSE
    canonical.popularity_score <- DEFAULT_POPULARITY
  END IF

  // Map user ratings
  canonical.average_rating <- platformData.rating ?? 0.0
  canonical.vote_count <- platformData.vote_count ?? 0

  // Create availability entry
  availability <- NEW PlatformAvailability()
  availability.platform <- platform
  availability.region <- platformData.region
  availability.availability_type <- MapAvailabilityType(platformData)
  availability.deep_link <- GenerateDeepLink(platformData, platform)
  availability.available_from <- GET_CURRENT_TIMESTAMP()
  canonical.availability <- [availability]

  // Series-specific handling
  IF canonical.content_type = ContentType.SERIES THEN
    canonical.series_metadata <- ExtractSeriesMetadata(platformData)
  END IF

  canonical.last_updated <- GET_CURRENT_TIMESTAMP()

  RETURN canonical
END
```

**Complexity:** O(g + c + d) where g=genres, c=credits, d=description length

### 2.2 Genre Mapping

```pseudocode
ALGORITHM: MapGenres
INPUT: rawGenres (List<string>), platform (Platform)
OUTPUT: List<Genre>

CONSTANTS:
  GENRE_MAPPINGS = {
    // Netflix mappings
    "netflix:action-adventure": Genre.ACTION,
    "netflix:comedies": Genre.COMEDY,
    "netflix:dramas": Genre.DRAMA,
    "netflix:sci-fi-fantasy": Genre.SCIENCE_FICTION,
    "netflix:thrillers": Genre.THRILLER,
    "netflix:horror-movies": Genre.HORROR,
    "netflix:documentaries": Genre.DOCUMENTARY,
    "netflix:romantic-movies": Genre.ROMANCE,

    // Prime Video mappings
    "prime:action_and_adventure": Genre.ACTION,
    "prime:comedy": Genre.COMEDY,
    "prime:drama": Genre.DRAMA,
    "prime:science_fiction": Genre.SCIENCE_FICTION,

    // Generic mappings (fallback)
    "action": Genre.ACTION,
    "adventure": Genre.ADVENTURE,
    "animation": Genre.ANIMATION,
    "comedy": Genre.COMEDY,
    "crime": Genre.CRIME,
    "documentary": Genre.DOCUMENTARY,
    "drama": Genre.DRAMA,
    "family": Genre.FAMILY,
    "fantasy": Genre.FANTASY,
    "horror": Genre.HORROR,
    "mystery": Genre.MYSTERY,
    "romance": Genre.ROMANCE,
    "sci-fi": Genre.SCIENCE_FICTION,
    "science fiction": Genre.SCIENCE_FICTION,
    "thriller": Genre.THRILLER
  }

BEGIN
  mappedGenres <- NEW Set<Genre>()

  FOR EACH rawGenre IN rawGenres DO
    // Normalize genre string
    normalized <- rawGenre.toLowerCase().trim()

    // Try platform-specific mapping first
    platformKey <- platform.name.toLowerCase() + ":" + normalized
    IF GENRE_MAPPINGS.has(platformKey) THEN
      mappedGenres.add(GENRE_MAPPINGS.get(platformKey))
      CONTINUE
    END IF

    // Try generic mapping
    IF GENRE_MAPPINGS.has(normalized) THEN
      mappedGenres.add(GENRE_MAPPINGS.get(normalized))
      CONTINUE
    END IF

    // Fuzzy match as last resort
    bestMatch <- FuzzyMatchGenre(normalized, GENRE_MAPPINGS.keys())
    IF bestMatch.confidence > 0.8 THEN
      mappedGenres.add(GENRE_MAPPINGS.get(bestMatch.key))
    END IF
  END FOR

  RETURN mappedGenres.toList()
END
```

**Complexity:** O(g * m) where g=raw genres, m=mapping table size

---

## 3. Entity Resolution

### 3.1 Content Deduplication

```pseudocode
ALGORITHM: ResolveContentEntity
INPUT: newContent (CanonicalContent), existingCatalog (ContentIndex)
OUTPUT: ResolvedContent (existing or new)

CONSTANTS:
  TITLE_SIMILARITY_THRESHOLD = 0.85
  YEAR_TOLERANCE = 1
  CONFIDENCE_MERGE_THRESHOLD = 0.9

BEGIN
  // Step 1: Try exact ID match (fastest path)
  FOR EACH (idType, idValue) IN newContent.external_ids DO
    IF idValue IS NOT NULL THEN
      existing <- existingCatalog.findByExternalId(idType, idValue)
      IF existing IS NOT NULL THEN
        // Merge availability into existing content
        MergeAvailability(existing, newContent)
        RETURN existing
      END IF
    END IF
  END FOR

  // Step 2: Candidate retrieval via blocking
  candidates <- existingCatalog.findCandidates(
    title: newContent.title,
    year: newContent.release_date.year,
    type: newContent.content_type
  )

  // Step 3: Pairwise comparison
  bestMatch <- NULL
  bestScore <- 0.0

  FOR EACH candidate IN candidates DO
    score <- CalculateMatchScore(newContent, candidate)
    IF score > bestScore THEN
      bestScore <- score
      bestMatch <- candidate
    END IF
  END FOR

  // Step 4: Decision
  IF bestMatch IS NOT NULL AND bestScore >= CONFIDENCE_MERGE_THRESHOLD THEN
    MergeContent(bestMatch, newContent)
    RETURN bestMatch
  ELSE
    // Insert as new content
    existingCatalog.insert(newContent)
    RETURN newContent
  END IF
END


ALGORITHM: CalculateMatchScore
INPUT: content1 (CanonicalContent), content2 (CanonicalContent)
OUTPUT: float (0.0-1.0)

WEIGHTS:
  TITLE_WEIGHT = 0.35
  YEAR_WEIGHT = 0.15
  RUNTIME_WEIGHT = 0.10
  GENRE_WEIGHT = 0.15
  CREDITS_WEIGHT = 0.25

BEGIN
  totalScore <- 0.0

  // Title similarity (Levenshtein + Jaro-Winkler average)
  titleSim <- (
    LevenshteinSimilarity(content1.title, content2.title) +
    JaroWinklerSimilarity(content1.title, content2.title)
  ) / 2
  totalScore <- totalScore + (titleSim * TITLE_WEIGHT)

  // Year match
  yearDiff <- ABS(content1.release_date.year - content2.release_date.year)
  yearSim <- MAX(0, 1.0 - (yearDiff / 5))
  totalScore <- totalScore + (yearSim * YEAR_WEIGHT)

  // Runtime match (if available)
  IF content1.runtime_minutes IS NOT NULL AND content2.runtime_minutes IS NOT NULL THEN
    runtimeDiff <- ABS(content1.runtime_minutes - content2.runtime_minutes)
    runtimeSim <- MAX(0, 1.0 - (runtimeDiff / 30))
    totalScore <- totalScore + (runtimeSim * RUNTIME_WEIGHT)
  END IF

  // Genre overlap (Jaccard similarity)
  genreSim <- JaccardSimilarity(content1.genres, content2.genres)
  totalScore <- totalScore + (genreSim * GENRE_WEIGHT)

  // Credits overlap (top 5 cast)
  cast1 <- content1.credits.cast.slice(0, 5).map(c => c.name)
  cast2 <- content2.credits.cast.slice(0, 5).map(c => c.name)
  creditsSim <- JaccardSimilarity(cast1, cast2)
  totalScore <- totalScore + (creditsSim * CREDITS_WEIGHT)

  RETURN totalScore
END
```

**Complexity:**
- ID lookup: O(1)
- Candidate retrieval: O(log n) with index
- Pairwise comparison: O(k) where k=candidates (typically <10)

---

## 4. Embedding Generation

### 4.1 Content Embedding Pipeline

```pseudocode
ALGORITHM: GenerateContentEmbedding
INPUT: content (CanonicalContent)
OUTPUT: embedding (float[768])

CONSTANTS:
  EMBEDDING_DIM = 768
  TEXT_WEIGHT = 0.4
  METADATA_WEIGHT = 0.3
  GRAPH_WEIGHT = 0.3

BEGIN
  // Step 1: Generate text embedding from title + overview
  textInput <- content.title + " " + content.overview
  textEmbedding <- TextEmbeddingModel.encode(textInput)
  textEmbedding <- L2Normalize(textEmbedding)

  // Step 2: Generate metadata embedding
  metadataFeatures <- []

  // Genre one-hot encoding (18 genres -> 18 dimensions)
  genreVector <- GenreToOneHot(content.genres)
  metadataFeatures.append(genreVector)

  // Year encoding (normalized 1900-2030)
  yearNorm <- (content.release_date.year - 1900) / 130.0
  metadataFeatures.append([yearNorm])

  // Popularity and rating (normalized 0-1)
  metadataFeatures.append([content.popularity_score])
  metadataFeatures.append([content.average_rating / 10.0])

  // Content type one-hot (6 types)
  typeVector <- ContentTypeToOneHot(content.content_type)
  metadataFeatures.append(typeVector)

  // Concatenate and project to embedding space
  metadataConcat <- Concatenate(metadataFeatures)
  metadataEmbedding <- MetadataProjectionLayer.project(metadataConcat)
  metadataEmbedding <- L2Normalize(metadataEmbedding)

  // Step 3: Generate graph embedding from relationships
  graphEmbedding <- GraphEmbeddingModel.embed(
    node_id: content.id,
    node_type: "content",
    include_neighbors: true,
    max_hops: 2
  )
  graphEmbedding <- L2Normalize(graphEmbedding)

  // Step 4: Combine embeddings with weighted sum
  finalEmbedding <- (
    textEmbedding * TEXT_WEIGHT +
    metadataEmbedding * METADATA_WEIGHT +
    graphEmbedding * GRAPH_WEIGHT
  )

  // Final normalization
  finalEmbedding <- L2Normalize(finalEmbedding)

  RETURN finalEmbedding
END


ALGORITHM: BatchGenerateEmbeddings
INPUT: contentBatch (List<CanonicalContent>)
OUTPUT: embeddings (Map<ContentId, float[768]>)

CONSTANTS:
  BATCH_SIZE = 32
  MAX_PARALLEL_BATCHES = 4

BEGIN
  embeddings <- NEW Map<ContentId, float[768]>()

  // Split into batches
  batches <- SplitIntoBatches(contentBatch, BATCH_SIZE)

  // Process batches in parallel
  FOR EACH batchGroup IN GroupBatches(batches, MAX_PARALLEL_BATCHES) DO
    results <- PARALLEL_FOR_EACH batch IN batchGroup DO
      batchResults <- []
      FOR EACH content IN batch DO
        embedding <- GenerateContentEmbedding(content)
        batchResults.append((content.id, embedding))
      END FOR
      RETURN batchResults
    END PARALLEL

    // Collect results
    FOR EACH result IN Flatten(results) DO
      embeddings.set(result.id, result.embedding)
    END FOR
  END FOR

  RETURN embeddings
END
```

**Complexity:**
- Single embedding: O(d) where d=embedding dimension (768)
- Batch generation: O(n*d/p) where n=items, p=parallel threads

---

## 5. Storage Requirements (Part 1)

| Data Type | Per-Item Size | Total Size (20M titles) |
|-----------|---------------|------------------------|
| CanonicalContent | ~20 KB | 400 GB |
| External IDs | ~200 bytes | 4 GB |
| Embeddings | 3 KB (768-dim float32) | 60 GB |
| Availability | ~500 bytes | 10 GB (1B entries) |

---

# PART 2: Search and SONA Personalization

---

## 6. Search and Discovery Engine

### 6.1 Search Query Processing

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

### 6.2 Search Result Structure

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

## 7. Intent Parsing

### 7.1 Natural Language Understanding

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
```

**Complexity:** O(p * q) where p=pattern count, q=query length

---

## 8. Hybrid Search Algorithm

### 8.1 Main Search Orchestrator

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

### 8.2 Vector Search Implementation

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

## 9. SONA Personalization Engine

### 9.1 User Profile Embedding

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
    profile <- GetUserProfile(userId)
    IF profile is null THEN
        profile <- InitializeNewProfile(userId)
    END IF

    // Filter and weight viewing events
    weightedEvents <- []
    currentTime <- GetCurrentTime()

    FOR EACH event IN viewingHistory DO
        // Skip low-engagement content
        IF event.completionRate < MIN_WATCH_THRESHOLD THEN
            CONTINUE
        END IF

        // Calculate temporal decay weight
        daysSince <- (currentTime - event.timestamp).days
        decayWeight <- DECAY_RATE ^ (daysSince / 30)

        // Calculate engagement weight
        engagementWeight <- CalculateEngagementWeight(event)

        // Combined weight
        totalWeight <- decayWeight * engagementWeight

        // Get content embedding
        contentEmbedding <- GetContentEmbedding(event.contentId)

        weightedEvents.append({
            embedding: contentEmbedding,
            weight: totalWeight,
            timestamp: event.timestamp
        })
    END FOR

    // Aggregate embeddings with weighted average
    IF weightedEvents.length > 0 THEN
        totalWeight <- SUM(weightedEvents.map(e => e.weight))
        aggregatedVector <- ZEROS(EMBEDDING_DIM)

        FOR EACH event IN weightedEvents DO
            normalizedWeight <- event.weight / totalWeight
            aggregatedVector <- aggregatedVector + (event.embedding * normalizedWeight)
        END FOR

        // L2 normalize
        profile.preferenceVector <- L2Normalize(aggregatedVector)
    END IF

    profile.lastUpdateTime <- currentTime
    RETURN profile.preferenceVector
END
```

### 9.2 Two-Tier LoRA Adaptation

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
    profile <- GetUserProfile(userId)

    // Check if enough new data for training
    IF recentEvents.length < MIN_TRAINING_EVENTS THEN
        RETURN
    END IF

    adapter <- profile.loraAdapter
    IF adapter IS NULL THEN
        adapter <- InitializeLoRAAdapter(userId)
        profile.loraAdapter <- adapter
    END IF

    // Prepare training data
    trainingPairs <- []
    FOR EACH event IN recentEvents DO
        contentEmbedding <- GetContentEmbedding(event.contentId)
        engagementLabel <- CalculateEngagementWeight(event)
        trainingPairs.append((contentEmbedding, engagementLabel))
    END FOR

    // LoRA training loop (few-shot adaptation)
    FOR iteration FROM 1 TO 5 DO
        totalLoss <- 0

        FOR EACH (embedding, label) IN trainingPairs DO
            // Forward pass through LoRA
            // h = W*x + (B*A)*x * scaling_factor
            loraOutput <- ComputeLoRAForward(adapter, embedding)

            // Predicted engagement
            predicted <- Sigmoid(DotProduct(loraOutput, profile.preferenceVector))

            // Binary cross-entropy loss
            loss <- -label * Log(predicted) - (1 - label) * Log(1 - predicted)
            totalLoss <- totalLoss + loss

            // Backward pass (gradient descent on user layer only)
            gradient <- ComputeLoRAGradient(adapter, embedding, predicted - label)
            adapter.userLayerWeights <- adapter.userLayerWeights - LEARNING_RATE * gradient
        END FOR

        avgLoss <- totalLoss / trainingPairs.length
    END FOR

    adapter.lastTrainedTime <- GetCurrentTime()
    adapter.trainingIterations <- adapter.trainingIterations + 1
END


ALGORITHM: ComputeLoRAForward
INPUT: adapter (UserLoRAAdapter), inputVector (float[INPUT_DIM])
OUTPUT: outputVector (float[OUTPUT_DIM])

BEGIN
    // LoRA: output = B * A * input * scaling_factor
    // A: [rank, input_dim], B: [output_dim, rank]

    // Low-rank projection
    intermediate <- MatMul(adapter.baseLayerWeights, inputVector)  // [rank]

    // User-specific adaptation
    output <- MatMul(adapter.userLayerWeights, intermediate)  // [output_dim]

    // Scale by alpha/rank
    scaledOutput <- output * adapter.scalingFactor

    RETURN scaledOutput
END
```

**Memory per User:** ~10KB (LoRA adapter with rank=8)

### 9.3 Hybrid Recommendation Engine

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
    profile <- GetUserProfile(userId)

    // Step 1: Generate candidate pool from multiple sources
    PARALLEL DO
        // Collaborative filtering (users with similar taste)
        collaborativeCandidates <- CollaborativeFilter(userId, limit: 100)

        // Content-based (similar to watched content)
        contentCandidates <- ContentBasedFilter(profile, limit: 100)

        // Graph-based (connected through actors, directors, genres)
        graphCandidates <- GraphBasedFilter(profile, limit: 100)

        // Context-aware (time, device, mood)
        contextCandidates <- ContextAwareFilter(profile, context, limit: 50)
    END PARALLEL

    // Step 2: Merge and deduplicate candidates
    allCandidates <- MergeCandidates([
        (collaborativeCandidates, COLLABORATIVE_WEIGHT),
        (contentCandidates, CONTENT_WEIGHT),
        (graphCandidates, GRAPH_WEIGHT),
        (contextCandidates, CONTEXT_WEIGHT)
    ])

    // Step 3: Filter already watched content
    watchedIds <- GetWatchedContentIds(userId)
    filteredCandidates <- allCandidates.filter(c => NOT watchedIds.contains(c.id))

    // Step 4: Apply LoRA personalization
    IF profile.loraAdapter IS NOT NULL THEN
        FOR EACH candidate IN filteredCandidates DO
            contentEmbedding <- GetContentEmbedding(candidate.id)
            loraScore <- ComputeLoRAScore(profile.loraAdapter, contentEmbedding, profile.preferenceVector)
            candidate.score <- candidate.score * (1 + loraScore * 0.3)
        END FOR
    END IF

    // Step 5: Apply diversity filter (MMR - Maximal Marginal Relevance)
    diverseResults <- ApplyDiversityFilter(
        candidates: filteredCandidates,
        threshold: DIVERSITY_THRESHOLD,
        limit: MAX_RECOMMENDATIONS
    )

    // Step 6: Generate explanations
    recommendations <- []
    FOR EACH result IN diverseResults DO
        explanation <- GenerateExplanation(result, profile)
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
    sortedCandidates <- SORT_BY(candidates, c => c.score, DESCENDING)

    selected <- []
    remaining <- sortedCandidates.copy()

    WHILE selected.length < limit AND remaining.length > 0 DO
        bestScore <- -INFINITY
        bestCandidate <- NULL
        bestIndex <- -1

        FOR EACH (index, candidate) IN ENUMERATE(remaining) DO
            // MMR score = lambda * relevance - (1-lambda) * max_similarity_to_selected
            relevance <- candidate.score

            maxSimilarity <- 0
            FOR EACH s IN selected DO
                sim <- CosineSimilarity(
                    GetContentEmbedding(candidate.id),
                    GetContentEmbedding(s.id)
                )
                maxSimilarity <- MAX(maxSimilarity, sim)
            END FOR

            mmrScore <- LAMBDA * relevance - (1 - LAMBDA) * maxSimilarity

            IF mmrScore > bestScore THEN
                bestScore <- mmrScore
                bestCandidate <- candidate
                bestIndex <- index
            END IF
        END FOR

        selected.append(bestCandidate)
        remaining.removeAt(bestIndex)
    END WHILE

    RETURN selected
END
```

### 9.4 Cold Start Handling

```pseudocode
ALGORITHM: HandleColdStartUser
INPUT: userId (string), signupContext (SignupContext)
OUTPUT: List<Recommendation>

BEGIN
    // Step 1: Check if truly new user
    profile <- GetUserProfile(userId)
    watchCount <- GetWatchCount(userId)

    IF watchCount > 5 THEN
        // Not a cold start, use normal recommendations
        RETURN GenerateRecommendations(userId, GetDefaultContext())
    END IF

    // Step 2: Use signup preferences if available
    IF signupContext.selectedGenres IS NOT NULL THEN
        genreRecommendations <- GetTopContentByGenres(
            genres: signupContext.selectedGenres,
            limit: 20
        )
        RETURN FormatAsRecommendations(genreRecommendations, "Based on your selected genres")
    END IF

    // Step 3: Use demographic-based recommendations
    IF signupContext.ageRange IS NOT NULL THEN
        demographicRecs <- GetDemographicRecommendations(
            ageRange: signupContext.ageRange,
            region: signupContext.region
        )
        RETURN FormatAsRecommendations(demographicRecs, "Popular in your area")
    END IF

    // Step 4: Fall back to trending content
    trendingContent <- GetTrendingContent(limit: 20)
    RETURN FormatAsRecommendations(trendingContent, "Trending now")
END


ALGORITHM: ProgressivePersonalization
INPUT: userId (string), newEvent (ViewingEvent)
OUTPUT: void

// Progressive personalization as user builds history
BEGIN
    profile <- GetUserProfile(userId)
    watchCount <- profile.interactionCount + 1
    profile.interactionCount <- watchCount

    // Update genre affinities
    contentGenres <- GetContentGenres(newEvent.contentId)
    engagement <- CalculateEngagementWeight(newEvent)

    FOR EACH genre IN contentGenres DO
        currentAffinity <- profile.genreAffinities.get(genre, 0.5)
        // Exponential moving average
        newAffinity <- currentAffinity * 0.9 + engagement * 0.1
        profile.genreAffinities.set(genre, newAffinity)
    END FOR

    // Update preference vector periodically
    IF watchCount MOD 5 = 0 THEN
        recentHistory <- GetRecentWatchHistory(userId, limit: 50)
        BuildUserPreferenceVector(userId, recentHistory)
    END IF

    // Train LoRA adapter after sufficient history
    IF watchCount >= 10 AND (watchCount MOD 10 = 0) THEN
        recentEvents <- GetRecentWatchHistory(userId, limit: 20)
        UpdateUserLoRA(userId, recentEvents)
    END IF

    SaveUserProfile(profile)
END
```

---

# PART 3: Real-time Sync and MCP Server

---

## 10. Real-time Synchronization (CRDTs)

### 10.1 Hybrid Logical Clock (HLC)

```pseudocode
STRUCTURE HybridLogicalClock:
    physicalTime: uint48          // Milliseconds since epoch
    logicalCounter: uint16        // Logical counter for causality
    nodeId: UUID                  // Device/node identifier

CONSTANTS:
    MAX_CLOCK_DRIFT = 60000       // 60 seconds max drift tolerance


ALGORITHM: HLC_Increment
INPUT: currentHLC (HybridLogicalClock), wallClock (timestamp)
OUTPUT: newHLC (HybridLogicalClock)

BEGIN
    newHLC <- NEW HybridLogicalClock()
    newHLC.nodeId <- currentHLC.nodeId

    IF wallClock > currentHLC.physicalTime THEN
        // Wall clock advanced - reset logical counter
        newHLC.physicalTime <- wallClock
        newHLC.logicalCounter <- 0
    ELSE
        // Wall clock same or behind - increment logical
        newHLC.physicalTime <- currentHLC.physicalTime
        newHLC.logicalCounter <- currentHLC.logicalCounter + 1

        // Check for counter overflow
        IF newHLC.logicalCounter > 65535 THEN
            // Wait for wall clock to advance
            WAIT_UNTIL(GetWallClock() > currentHLC.physicalTime)
            newHLC.physicalTime <- GetWallClock()
            newHLC.logicalCounter <- 0
        END IF
    END IF

    RETURN newHLC
END


ALGORITHM: HLC_Receive
INPUT: localHLC (HybridLogicalClock), remoteHLC (HybridLogicalClock)
OUTPUT: mergedHLC (HybridLogicalClock)

BEGIN
    wallClock <- GetWallClock()
    mergedHLC <- NEW HybridLogicalClock()
    mergedHLC.nodeId <- localHLC.nodeId

    // Detect clock drift
    IF ABS(remoteHLC.physicalTime - wallClock) > MAX_CLOCK_DRIFT THEN
        LOG_WARNING("Large clock drift detected: " + (remoteHLC.physicalTime - wallClock))
    END IF

    maxPhysical <- MAX(localHLC.physicalTime, remoteHLC.physicalTime, wallClock)

    IF maxPhysical = localHLC.physicalTime AND maxPhysical = remoteHLC.physicalTime THEN
        // Both have same physical time - take max logical + 1
        mergedHLC.physicalTime <- maxPhysical
        mergedHLC.logicalCounter <- MAX(localHLC.logicalCounter, remoteHLC.logicalCounter) + 1
    ELSE IF maxPhysical = localHLC.physicalTime THEN
        mergedHLC.physicalTime <- maxPhysical
        mergedHLC.logicalCounter <- localHLC.logicalCounter + 1
    ELSE IF maxPhysical = remoteHLC.physicalTime THEN
        mergedHLC.physicalTime <- maxPhysical
        mergedHLC.logicalCounter <- remoteHLC.logicalCounter + 1
    ELSE
        // Wall clock is largest
        mergedHLC.physicalTime <- maxPhysical
        mergedHLC.logicalCounter <- 0
    END IF

    RETURN mergedHLC
END


ALGORITHM: HLC_Compare
INPUT: a (HybridLogicalClock), b (HybridLogicalClock)
OUTPUT: integer (-1, 0, or 1)

BEGIN
    IF a.physicalTime < b.physicalTime THEN
        RETURN -1
    ELSE IF a.physicalTime > b.physicalTime THEN
        RETURN 1
    ELSE IF a.logicalCounter < b.logicalCounter THEN
        RETURN -1
    ELSE IF a.logicalCounter > b.logicalCounter THEN
        RETURN 1
    ELSE IF a.nodeId < b.nodeId THEN
        RETURN -1
    ELSE IF a.nodeId > b.nodeId THEN
        RETURN 1
    ELSE
        RETURN 0
    END IF
END
```

**Complexity:** All operations O(1)

### 10.2 LWW-Register (Last-Writer-Wins)

```pseudocode
STRUCTURE LWWRegister<T>:
    value: T
    timestamp: HybridLogicalClock
    deviceId: UUID


ALGORITHM: LWW_Set
INPUT: register (LWWRegister<T>), newValue (T), hlc (HybridLogicalClock), deviceId (UUID)
OUTPUT: boolean (true if value was updated)

BEGIN
    IF HLC_Compare(hlc, register.timestamp) > 0 THEN
        // New timestamp is greater - update value
        register.value <- newValue
        register.timestamp <- hlc
        register.deviceId <- deviceId
        RETURN true
    ELSE IF HLC_Compare(hlc, register.timestamp) = 0 AND deviceId > register.deviceId THEN
        // Same timestamp - tie-break by device ID
        register.value <- newValue
        register.timestamp <- hlc
        register.deviceId <- deviceId
        RETURN true
    ELSE
        // Old timestamp - ignore
        RETURN false
    END IF
END


ALGORITHM: LWW_Merge
INPUT: local (LWWRegister<T>), remote (LWWRegister<T>)
OUTPUT: merged (LWWRegister<T>)

BEGIN
    comparison <- HLC_Compare(local.timestamp, remote.timestamp)

    IF comparison > 0 THEN
        RETURN local
    ELSE IF comparison < 0 THEN
        RETURN remote
    ELSE
        // Same timestamp - tie-break by device ID
        IF local.deviceId > remote.deviceId THEN
            RETURN local
        ELSE
            RETURN remote
        END IF
    END IF
END
```

### 10.3 OR-Set (Observed-Remove Set)

```pseudocode
STRUCTURE ORSet<T>:
    added: Map<T, Set<UniqueTag>>     // Element -> set of add tags
    removed: Set<UniqueTag>            // Set of removed tags

STRUCTURE UniqueTag:
    elementId: T
    deviceId: UUID
    timestamp: timestamp
    randomId: UUID


ALGORITHM: ORSet_Add
INPUT: set (ORSet<T>), element (T), deviceId (UUID)
OUTPUT: void

BEGIN
    tag <- NEW UniqueTag()
    tag.elementId <- element
    tag.deviceId <- deviceId
    tag.timestamp <- GetCurrentTime()
    tag.randomId <- GenerateUUID()

    IF NOT set.added.has(element) THEN
        set.added.set(element, NEW Set<UniqueTag>())
    END IF

    set.added.get(element).add(tag)
END


ALGORITHM: ORSet_Remove
INPUT: set (ORSet<T>), element (T)
OUTPUT: void

BEGIN
    IF set.added.has(element) THEN
        tags <- set.added.get(element)

        FOR EACH tag IN tags DO
            set.removed.add(tag)
        END FOR
    END IF
END


ALGORITHM: ORSet_Contains
INPUT: set (ORSet<T>), element (T)
OUTPUT: boolean

BEGIN
    IF NOT set.added.has(element) THEN
        RETURN false
    END IF

    tags <- set.added.get(element)

    FOR EACH tag IN tags DO
        IF NOT set.removed.contains(tag) THEN
            RETURN true  // At least one active tag
        END IF
    END FOR

    RETURN false  // All tags removed
END


ALGORITHM: ORSet_Merge
INPUT: local (ORSet<T>), remote (ORSet<T>)
OUTPUT: merged (ORSet<T>)

BEGIN
    merged <- NEW ORSet<T>()

    // Union of added elements
    FOR EACH (element, localTags) IN local.added DO
        IF remote.added.has(element) THEN
            remoteTags <- remote.added.get(element)
            merged.added.set(element, localTags.union(remoteTags))
        ELSE
            merged.added.set(element, localTags.copy())
        END IF
    END FOR

    FOR EACH (element, remoteTags) IN remote.added DO
        IF NOT merged.added.has(element) THEN
            merged.added.set(element, remoteTags.copy())
        END IF
    END FOR

    // Union of removed tags
    merged.removed <- local.removed.union(remote.removed)

    RETURN merged
END
```

---

## 11. PubNub Integration

### 11.1 Channel Management

```pseudocode
CONSTANTS:
    CHANNELS:
        USER_SYNC = "user.{userId}.sync"
        USER_DEVICES = "user.{userId}.devices"
        USER_NOTIFICATIONS = "user.{userId}.notifications"
        GLOBAL_TRENDING = "global.trending"

    HEARTBEAT_INTERVAL = 30  // seconds
    PRESENCE_TIMEOUT = 60    // seconds


STRUCTURE PubNubClient:
    publishKey: string
    subscribeKey: string
    userId: string
    deviceId: string
    subscribedChannels: Set<string>
    messageHandlers: Map<string, Function>


ALGORITHM: InitializePubNub
INPUT: userId (string), deviceId (string)
OUTPUT: PubNubClient

BEGIN
    client <- NEW PubNubClient()
    client.publishKey <- GetConfigValue("PUBNUB_PUBLISH_KEY")
    client.subscribeKey <- GetConfigValue("PUBNUB_SUBSCRIBE_KEY")
    client.userId <- userId
    client.deviceId <- deviceId
    client.subscribedChannels <- NEW Set<string>()
    client.messageHandlers <- NEW Map<string, Function>()

    // Subscribe to user channels
    userSyncChannel <- CHANNELS.USER_SYNC.replace("{userId}", userId)
    userDevicesChannel <- CHANNELS.USER_DEVICES.replace("{userId}", userId)

    SubscribeToChannels(client, [userSyncChannel, userDevicesChannel])

    // Start presence heartbeat
    StartHeartbeat(client)

    RETURN client
END


ALGORITHM: HandleMessage
INPUT: client (PubNubClient), message (PubNubMessage)
OUTPUT: void

BEGIN
    // Ignore own messages
    IF message.senderId = client.deviceId THEN
        RETURN
    END IF

    // Route to appropriate handler
    MATCH message.type
        CASE "WATCH_PROGRESS":
            HandleWatchProgressSync(client.userId, message.payload)

        CASE "WATCHLIST_ADD":
            HandleWatchlistAddSync(client.userId, message.payload)

        CASE "WATCHLIST_REMOVE":
            HandleWatchlistRemoveSync(client.userId, message.payload)

        CASE "PREFERENCE_UPDATE":
            HandlePreferenceSync(client.userId, message.payload)

        CASE "REMOTE_COMMAND":
            HandleRemoteCommand(client.deviceId, message.payload)

        DEFAULT:
            LOG_WARNING("Unknown message type: " + message.type)
    END MATCH
END
```

### 11.2 Remote Control Protocol

```pseudocode
STRUCTURE RemoteCommand:
    commandId: string
    sourceDevice: string
    targetDevice: string
    action: CommandAction
    payload: object
    timestamp: timestamp
    expiresAt: timestamp

ENUM CommandAction:
    PLAY
    PAUSE
    SEEK
    STOP
    NAVIGATE
    VOLUME


ALGORITHM: SendRemoteCommand
INPUT: sourceDevice (string), targetDevice (string), action (CommandAction), payload (object)
OUTPUT: Promise<CommandResult>

CONSTANTS:
    COMMAND_TIMEOUT = 5000  // 5 seconds
    ACK_TIMEOUT = 2000      // 2 seconds

BEGIN
    commandId <- GenerateUUID()

    command <- NEW RemoteCommand()
    command.commandId <- commandId
    command.sourceDevice <- sourceDevice
    command.targetDevice <- targetDevice
    command.action <- action
    command.payload <- payload
    command.timestamp <- GetCurrentTime()
    command.expiresAt <- command.timestamp + COMMAND_TIMEOUT

    // Create acknowledgment promise
    ackPromise <- CreateAckPromise(commandId, ACK_TIMEOUT)

    // Publish command to target device channel
    targetChannel <- "device." + targetDevice + ".commands"

    TRY
        AWAIT PublishMessage(GetPubNubClient(), targetChannel, {
            type: "REMOTE_COMMAND",
            payload: command
        })

        // Wait for acknowledgment
        ack <- AWAIT ackPromise

        IF ack.status = "RECEIVED" THEN
            // Wait for completion
            completionPromise <- CreateCompletionPromise(commandId, COMMAND_TIMEOUT)
            completion <- AWAIT completionPromise

            RETURN CommandResult(
                success: completion.status = "COMPLETED",
                commandId: commandId,
                error: completion.error
            )
        ELSE
            RETURN CommandResult(success: false, error: "Device did not acknowledge")
        END IF

    CATCH TimeoutError
        RETURN CommandResult(success: false, error: "Command timed out")
    END TRY
END
```

---

## 12. MCP Server Core

### 12.1 Server Architecture

```pseudocode
STRUCTURE MCPServer:
    tools: Map<string, MCPTool>
    resources: Map<string, MCPResource>
    transport: Transport
    requestHandlers: Map<string, Function>
    config: ServerConfig


ALGORITHM: InitializeMCPServer
INPUT: config (ServerConfig)
OUTPUT: MCPServer

BEGIN
    server <- NEW MCPServer()
    server.config <- config
    server.tools <- NEW Map<string, MCPTool>()
    server.resources <- NEW Map<string, MCPResource>()

    // Register standard request handlers
    server.requestHandlers <- {
        "initialize": HandleInitialize,
        "tools/list": HandleToolsList,
        "tools/call": HandleToolCall,
        "resources/list": HandleResourcesList,
        "resources/read": HandleResourceRead
    }

    // Register tools
    RegisterTool(server, SemanticSearchTool)
    RegisterTool(server, GetContentDetailsTool)
    RegisterTool(server, GetRecommendationsTool)
    RegisterTool(server, ListDevicesTool)
    RegisterTool(server, InitiatePlaybackTool)
    RegisterTool(server, ControlPlaybackTool)

    // Initialize transport
    IF config.transport = "stdio" THEN
        server.transport <- NEW StdioTransport()
    ELSE IF config.transport = "sse" THEN
        server.transport <- NEW SSETransport(config.port)
    END IF

    RETURN server
END


ALGORITHM: HandleRequest
INPUT: server (MCPServer), request (MCPRequest)
OUTPUT: MCPResponse

BEGIN
    // Validate JSON-RPC format
    IF request.jsonrpc != "2.0" THEN
        RETURN ErrorResponse(request.id, -32600, "Invalid Request")
    END IF

    // Find handler
    IF NOT server.requestHandlers.has(request.method) THEN
        RETURN ErrorResponse(request.id, -32601, "Method not found")
    END IF

    handler <- server.requestHandlers.get(request.method)

    TRY
        result <- AWAIT handler(server, request.params)
        RETURN SuccessResponse(request.id, result)

    CATCH ValidationError AS e
        RETURN ErrorResponse(request.id, -32602, e.message)

    CATCH error AS e
        LOG_ERROR("Handler error: " + e.message)
        RETURN ErrorResponse(request.id, -32603, "Internal error")
    END TRY
END
```

---

## 13. MCP Tool Implementations

### 13.1 Semantic Search Tool

```pseudocode
TOOL: semantic_search

DEFINITION:
    name: "semantic_search"
    description: "Search for movies and TV shows using natural language"
    inputSchema: {
        type: "object",
        properties: {
            query: { type: "string", description: "Natural language search query" },
            filters: {
                type: "object",
                properties: {
                    genres: { type: "array", items: { type: "string" } },
                    year_range: { type: "object", properties: { min: integer, max: integer } },
                    platforms: { type: "array", items: { type: "string" } },
                    rating_min: { type: "number" }
                }
            },
            page: { type: "integer", default: 1 },
            page_size: { type: "integer", default: 20 }
        },
        required: ["query"]
    }


ALGORITHM: ExecuteSemanticSearch
INPUT: params (object)
OUTPUT: SearchResults

BEGIN
    // Validate input
    IF params.query IS NULL OR params.query.trim().length = 0 THEN
        THROW ValidationError("Query is required")
    END IF

    // Build search query
    query <- NEW SearchQuery()
    query.query_text <- params.query
    query.page <- params.page OR 1
    query.page_size <- MIN(params.page_size OR 20, 100)
    query.strategy <- SearchStrategy.HYBRID

    // Apply filters
    IF params.filters IS NOT NULL THEN
        query.filters <- NEW SearchFilters()

        IF params.filters.genres IS NOT NULL THEN
            query.filters.genres <- MapGenreStrings(params.filters.genres)
        END IF

        IF params.filters.year_range IS NOT NULL THEN
            query.filters.year_range <- YearRange(
                min_year: params.filters.year_range.min,
                max_year: params.filters.year_range.max
            )
        END IF

        IF params.filters.platforms IS NOT NULL THEN
            query.filters.platforms <- MapPlatformStrings(params.filters.platforms)
        END IF

        IF params.filters.rating_min IS NOT NULL THEN
            query.filters.rating_range <- RatingRange(
                min_rating: params.filters.rating_min,
                max_rating: 10.0
            )
        END IF
    END IF

    // Execute search
    results <- ExecuteHybridSearch(query)

    // Format response
    RETURN {
        results: results.results.map(r => FormatContentResult(r)),
        total_count: results.total_count,
        page: results.page,
        page_size: results.page_size,
        has_more: results.page * results.page_size < results.total_count
    }
END
```

### 13.2 Playback Control Tools

```pseudocode
TOOL: initiate_playback

DEFINITION:
    name: "initiate_playback"
    description: "Start playing content on a specific device"
    inputSchema: {
        type: "object",
        properties: {
            content_id: { type: "string", description: "Content ID to play" },
            device_id: { type: "string", description: "Target device ID" },
            platform: { type: "string", description: "Streaming platform" },
            start_position: { type: "integer", description: "Start position in seconds" }
        },
        required: ["content_id", "device_id"]
    }


ALGORITHM: ExecuteInitiatePlayback
INPUT: params (object), context (RequestContext)
OUTPUT: PlaybackResult

BEGIN
    // Validate device exists and is online
    device <- GetDevice(params.device_id)
    IF device IS NULL THEN
        THROW ValidationError("Device not found: " + params.device_id)
    END IF

    IF device.state = PresenceState.OFFLINE THEN
        THROW ValidationError("Device is offline")
    END IF

    // Get content details
    content <- GetContent(params.content_id)
    IF content IS NULL THEN
        THROW ValidationError("Content not found: " + params.content_id)
    END IF

    // Determine platform
    IF params.platform IS NOT NULL THEN
        platform <- ParsePlatform(params.platform)
    ELSE
        // Auto-select based on user subscriptions
        userPrefs <- GetUserPreferences(context.userId)
        platform <- SelectBestPlatform(content.availability, userPrefs.subscribed_platforms)
    END IF

    // Get deep link
    availability <- content.availability.find(a => a.platform = platform)
    IF availability IS NULL THEN
        THROW ValidationError("Content not available on " + platform.toString())
    END IF

    deepLink <- GenerateDeepLink(availability, params.start_position)

    // Send playback command to device
    result <- AWAIT SendRemoteCommand(
        context.deviceId,
        params.device_id,
        CommandAction.PLAY,
        {
            contentId: params.content_id,
            deepLink: deepLink,
            position: params.start_position OR 0
        }
    )

    RETURN {
        success: result.success,
        device_id: params.device_id,
        content_id: params.content_id,
        platform: platform.toString(),
        deep_link: deepLink,
        error: result.error
    }
END
```

---

## 14. ARW Protocol

### 14.1 Manifest Generation

```pseudocode
STRUCTURE ARWManifest:
    protocol_version: string
    base_url: string
    capabilities: List<Capability>
    tools: List<ToolDefinition>
    authentication: AuthConfig
    rate_limits: RateLimitConfig


ALGORITHM: GenerateARWManifest
INPUT: server (MCPServer), baseUrl (string)
OUTPUT: ARWManifest

BEGIN
    manifest <- NEW ARWManifest()
    manifest.protocol_version <- "1.0"
    manifest.base_url <- baseUrl

    // Define capabilities
    manifest.capabilities <- [
        Capability("search", "Semantic content search"),
        Capability("recommendations", "Personalized recommendations"),
        Capability("playback", "Remote playback control"),
        Capability("devices", "Device management"),
        Capability("sync", "Cross-device synchronization")
    ]

    // Export tool definitions
    manifest.tools <- []
    FOR EACH (name, tool) IN server.tools DO
        manifest.tools.append(ToolDefinition(
            name: tool.name,
            description: tool.description,
            input_schema: tool.inputSchema,
            required_scopes: GetToolScopes(tool)
        ))
    END FOR

    // Authentication configuration
    manifest.authentication <- AuthConfig(
        type: "oauth2",
        authorization_url: baseUrl + "/oauth/authorize",
        token_url: baseUrl + "/oauth/token",
        scopes: [
            "read:content",
            "read:recommendations",
            "write:watchlist",
            "control:playback",
            "read:devices"
        ]
    )

    // Rate limits
    manifest.rate_limits <- RateLimitConfig(
        requests_per_minute: 60,
        burst_limit: 10,
        tier_overrides: {
            "premium": { requests_per_minute: 300, burst_limit: 50 }
        }
    )

    RETURN manifest
END
```

---

# PART 4: Authentication, CLI, and Error Handling

---

## 15. Authentication and Authorization

### 15.1 OAuth 2.0 + PKCE Flow

```pseudocode
STRUCTURE OAuthState:
    state: string
    codeVerifier: string
    redirectUri: string
    scopes: List<string>
    createdAt: timestamp
    expiresAt: timestamp

CONSTANTS:
    CODE_VERIFIER_LENGTH = 64
    STATE_LENGTH = 32
    AUTH_CODE_EXPIRY = 600        // 10 minutes
    ACCESS_TOKEN_EXPIRY = 3600    // 1 hour
    REFRESH_TOKEN_EXPIRY = 2592000  // 30 days


ALGORITHM: InitiateOAuthFlow
INPUT: clientId (string), redirectUri (string), scopes (List<string>)
OUTPUT: AuthorizationURL

BEGIN
    // Generate PKCE code verifier (high-entropy random string)
    codeVerifier <- GenerateSecureRandomString(CODE_VERIFIER_LENGTH)

    // Generate code challenge (SHA-256 hash of verifier, base64url encoded)
    codeChallenge <- Base64URLEncode(SHA256(codeVerifier))

    // Generate state parameter (CSRF protection)
    state <- GenerateSecureRandomString(STATE_LENGTH)

    // Store state for verification
    oauthState <- NEW OAuthState()
    oauthState.state <- state
    oauthState.codeVerifier <- codeVerifier
    oauthState.redirectUri <- redirectUri
    oauthState.scopes <- scopes
    oauthState.createdAt <- GetCurrentTime()
    oauthState.expiresAt <- oauthState.createdAt + AUTH_CODE_EXPIRY

    StoreOAuthState(state, oauthState)

    // Build authorization URL
    authUrl <- BuildURL(GetAuthorizationEndpoint(), {
        response_type: "code",
        client_id: clientId,
        redirect_uri: redirectUri,
        scope: scopes.join(" "),
        state: state,
        code_challenge: codeChallenge,
        code_challenge_method: "S256"
    })

    RETURN authUrl
END


ALGORITHM: ExchangeCodeForTokens
INPUT: authCode (string), state (string), codeVerifier (string)
OUTPUT: TokenResponse

BEGIN
    // Verify state matches stored state
    storedState <- GetOAuthState(state)

    IF storedState IS NULL THEN
        THROW AuthError("Invalid state parameter")
    END IF

    IF GetCurrentTime() > storedState.expiresAt THEN
        DeleteOAuthState(state)
        THROW AuthError("Authorization code expired")
    END IF

    IF codeVerifier != storedState.codeVerifier THEN
        THROW AuthError("Code verifier mismatch")
    END IF

    // Exchange code for tokens
    response <- AWAIT HTTPPost(GetTokenEndpoint(), {
        grant_type: "authorization_code",
        code: authCode,
        redirect_uri: storedState.redirectUri,
        code_verifier: codeVerifier,
        client_id: GetClientId()
    })

    IF response.status != 200 THEN
        THROW AuthError("Token exchange failed: " + response.body.error)
    END IF

    // Parse and store tokens
    tokens <- ParseTokenResponse(response.body)

    // Clean up state
    DeleteOAuthState(state)

    RETURN tokens
END
```

### 15.2 Device Authorization Grant (RFC 8628)

```pseudocode
STRUCTURE DeviceCode:
    deviceCode: string
    userCode: string
    verificationUri: string
    expiresIn: integer
    interval: integer

CONSTANTS:
    USER_CODE_LENGTH = 8
    DEVICE_CODE_LENGTH = 40
    DEVICE_CODE_EXPIRY = 900      // 15 minutes
    POLL_INTERVAL = 5             // 5 seconds


ALGORITHM: RequestDeviceCode
INPUT: clientId (string), scopes (List<string>)
OUTPUT: DeviceCode

BEGIN
    response <- AWAIT HTTPPost(GetDeviceAuthorizationEndpoint(), {
        client_id: clientId,
        scope: scopes.join(" ")
    })

    IF response.status != 200 THEN
        THROW AuthError("Device authorization request failed")
    END IF

    deviceCode <- NEW DeviceCode()
    deviceCode.deviceCode <- response.body.device_code
    deviceCode.userCode <- response.body.user_code
    deviceCode.verificationUri <- response.body.verification_uri
    deviceCode.expiresIn <- response.body.expires_in
    deviceCode.interval <- response.body.interval OR POLL_INTERVAL

    RETURN deviceCode
END


ALGORITHM: PollForToken
INPUT: deviceCode (DeviceCode)
OUTPUT: TokenResponse

CONSTANTS:
    MAX_POLL_ATTEMPTS = 180  // 15 minutes at 5 second intervals

BEGIN
    attempts <- 0
    pollInterval <- deviceCode.interval

    LOOP
        attempts <- attempts + 1

        IF attempts > MAX_POLL_ATTEMPTS THEN
            THROW AuthError("Device authorization timed out")
        END IF

        // Wait before polling
        AWAIT Sleep(pollInterval * 1000)

        response <- AWAIT HTTPPost(GetTokenEndpoint(), {
            grant_type: "urn:ietf:params:oauth:grant-type:device_code",
            device_code: deviceCode.deviceCode,
            client_id: GetClientId()
        })

        IF response.status = 200 THEN
            // Authorization successful
            RETURN ParseTokenResponse(response.body)
        END IF

        error <- response.body.error

        MATCH error
            CASE "authorization_pending":
                // User hasn't authorized yet, continue polling
                CONTINUE

            CASE "slow_down":
                // Server asking to slow down
                pollInterval <- pollInterval + 5
                CONTINUE

            CASE "expired_token":
                THROW AuthError("Device code expired")

            CASE "access_denied":
                THROW AuthError("User denied authorization")

            DEFAULT:
                THROW AuthError("Unexpected error: " + error)
        END MATCH
    END LOOP
END
```

### 15.3 JWT Token Management

```pseudocode
STRUCTURE JWTClaims:
    sub: string           // Subject (user ID)
    aud: string           // Audience
    iss: string           // Issuer
    exp: integer          // Expiration time
    iat: integer          // Issued at
    jti: string           // JWT ID (for revocation)
    scope: string         // Granted scopes


ALGORITHM: GenerateJWT
INPUT: userId (string), scopes (List<string>)
OUTPUT: string (JWT token)

BEGIN
    now <- GetCurrentTimeUnix()

    header <- {
        alg: "RS256",
        typ: "JWT",
        kid: GetCurrentKeyId()
    }

    claims <- NEW JWTClaims()
    claims.sub <- userId
    claims.aud <- GetAudience()
    claims.iss <- GetIssuer()
    claims.exp <- now + ACCESS_TOKEN_EXPIRY
    claims.iat <- now
    claims.jti <- GenerateUUID()
    claims.scope <- scopes.join(" ")

    // Encode header and claims
    encodedHeader <- Base64URLEncode(JSON.stringify(header))
    encodedClaims <- Base64URLEncode(JSON.stringify(claims))

    // Sign with private key
    message <- encodedHeader + "." + encodedClaims
    signature <- RS256Sign(message, GetPrivateKey())
    encodedSignature <- Base64URLEncode(signature)

    jwt <- message + "." + encodedSignature

    RETURN jwt
END


ALGORITHM: ValidateJWT
INPUT: token (string)
OUTPUT: JWTClaims

BEGIN
    // Split token
    parts <- token.split(".")
    IF parts.length != 3 THEN
        THROW AuthError("Invalid token format")
    END IF

    encodedHeader <- parts[0]
    encodedClaims <- parts[1]
    encodedSignature <- parts[2]

    // Decode header
    header <- JSON.parse(Base64URLDecode(encodedHeader))

    // Get public key for verification
    IF header.kid IS NULL THEN
        THROW AuthError("Missing key ID in token header")
    END IF

    publicKey <- GetPublicKey(header.kid)
    IF publicKey IS NULL THEN
        THROW AuthError("Unknown signing key")
    END IF

    // Verify signature
    message <- encodedHeader + "." + encodedClaims
    signature <- Base64URLDecode(encodedSignature)

    IF NOT RS256Verify(message, signature, publicKey) THEN
        THROW AuthError("Invalid token signature")
    END IF

    // Decode and validate claims
    claims <- JSON.parse(Base64URLDecode(encodedClaims))

    now <- GetCurrentTimeUnix()

    // Check expiration (with 60 second clock skew tolerance)
    IF claims.exp < now - 60 THEN
        THROW AuthError("Token expired")
    END IF

    // Check not-before (iat)
    IF claims.iat > now + 60 THEN
        THROW AuthError("Token not yet valid")
    END IF

    // Check issuer
    IF claims.iss != GetIssuer() THEN
        THROW AuthError("Invalid issuer")
    END IF

    // Check audience
    IF claims.aud != GetAudience() THEN
        THROW AuthError("Invalid audience")
    END IF

    // Check revocation
    IF IsTokenRevoked(claims.jti) THEN
        THROW AuthError("Token has been revoked")
    END IF

    RETURN claims
END
```

### 15.4 Refresh Token Rotation

```pseudocode
STRUCTURE RefreshTokenRecord:
    tokenId: string
    userId: string
    familyId: string          // For detecting reuse
    previousTokenId: string   // Chain for rotation
    createdAt: timestamp
    expiresAt: timestamp
    usedAt: timestamp NULLABLE
    revokedAt: timestamp NULLABLE


ALGORITHM: RotateRefreshToken
INPUT: oldToken (string)
OUTPUT: TokenResponse

BEGIN
    // Look up old token
    oldRecord <- GetRefreshToken(oldToken)

    IF oldRecord IS NULL THEN
        THROW AuthError("Invalid refresh token")
    END IF

    // Check if already used (potential token theft)
    IF oldRecord.usedAt IS NOT NULL THEN
        // Revoke entire token family
        RevokeTokenFamily(oldRecord.familyId)
        THROW AuthError("Refresh token reuse detected - all tokens revoked")
    END IF

    // Check expiration
    IF GetCurrentTime() > oldRecord.expiresAt THEN
        THROW AuthError("Refresh token expired")
    END IF

    // Check if revoked
    IF oldRecord.revokedAt IS NOT NULL THEN
        THROW AuthError("Refresh token has been revoked")
    END IF

    // Mark old token as used
    oldRecord.usedAt <- GetCurrentTime()
    UpdateRefreshToken(oldRecord)

    // Issue new refresh token in same family
    newTokenId <- GenerateSecureRandomString(64)

    newRecord <- NEW RefreshTokenRecord()
    newRecord.tokenId <- newTokenId
    newRecord.userId <- oldRecord.userId
    newRecord.familyId <- oldRecord.familyId
    newRecord.previousTokenId <- oldRecord.tokenId
    newRecord.createdAt <- GetCurrentTime()
    newRecord.expiresAt <- newRecord.createdAt + REFRESH_TOKEN_EXPIRY

    StoreRefreshToken(newRecord)

    // Generate new access token
    user <- GetUser(oldRecord.userId)
    accessToken <- GenerateJWT(user.id, user.scopes)

    RETURN TokenResponse(
        accessToken: accessToken,
        refreshToken: newTokenId,
        expiresIn: ACCESS_TOKEN_EXPIRY
    )
END
```

### 15.5 Rate Limiting

```pseudocode
STRUCTURE RateLimitBucket:
    key: string
    tokens: float
    lastRefill: timestamp
    capacity: integer
    refillRate: float         // Tokens per second


ALGORITHM: TokenBucketRateLimit
INPUT: key (string), cost (integer)
OUTPUT: RateLimitResult

CONSTANTS:
    DEFAULT_CAPACITY = 60
    DEFAULT_REFILL_RATE = 1.0   // 1 token per second

BEGIN
    bucket <- GetOrCreateBucket(key)

    // Refill tokens based on elapsed time
    now <- GetCurrentTime()
    elapsed <- (now - bucket.lastRefill).seconds
    refillAmount <- elapsed * bucket.refillRate

    bucket.tokens <- MIN(bucket.capacity, bucket.tokens + refillAmount)
    bucket.lastRefill <- now

    // Check if request can be allowed
    IF bucket.tokens >= cost THEN
        bucket.tokens <- bucket.tokens - cost
        SaveBucket(bucket)

        RETURN RateLimitResult(
            allowed: true,
            remaining: FLOOR(bucket.tokens),
            resetAt: now + ((bucket.capacity - bucket.tokens) / bucket.refillRate)
        )
    ELSE
        // Rate limited
        retryAfter <- (cost - bucket.tokens) / bucket.refillRate
        SaveBucket(bucket)

        RETURN RateLimitResult(
            allowed: false,
            remaining: 0,
            resetAt: now + retryAfter,
            retryAfter: CEIL(retryAfter)
        )
    END IF
END
```

---

## 16. CLI Command Implementation and Error Handling

### 16.1 Command Parser Framework

```pseudocode
STRUCTURE CLIApp:
    commands: Map<string, Command>
    config: AppConfig
    logger: Logger
    version: string

CONSTANTS:
    EXIT_SUCCESS = 0
    EXIT_ERROR = 1
    EXIT_INVALID_ARGS = 2
    EXIT_AUTH_REQUIRED = 3
    EXIT_NETWORK_ERROR = 4


ALGORITHM: CLIRun
INPUT: args (List<string>)
OUTPUT: exitCode (integer)

BEGIN
    TRY
        // Parse arguments
        parsed <- ParseArguments(args)

        // Handle global flags
        IF parsed.flags.version THEN
            PRINT "media-gateway v" + GetVersion()
            RETURN EXIT_SUCCESS
        END IF

        IF parsed.flags.help OR parsed.command IS NULL THEN
            ShowHelp()
            RETURN EXIT_SUCCESS
        END IF

        // Validate command exists
        IF NOT commands.has(parsed.command) THEN
            PRINT_ERROR "Unknown command: " + parsed.command
            PRINT "Run 'media-gateway --help' for available commands"
            RETURN EXIT_INVALID_ARGS
        END IF

        command <- commands.get(parsed.command)

        // Check authentication requirement
        IF command.requiresAuth AND NOT IsAuthenticated() THEN
            PRINT_ERROR "Authentication required"
            PRINT "Run 'media-gateway auth login' to authenticate"
            RETURN EXIT_AUTH_REQUIRED
        END IF

        // Execute command
        IF IsInteractiveMode() AND command.hasInteractiveMode THEN
            result <- command.runInteractive(parsed.options)
        ELSE
            result <- command.runBatch(parsed.options)
        END IF

        // Output result
        OutputResult(result, parsed.options.format)

        RETURN result.success ? EXIT_SUCCESS : EXIT_ERROR

    CATCH NetworkError AS e
        PRINT_ERROR "Network error: " + e.message
        RETURN EXIT_NETWORK_ERROR

    CATCH AuthError AS e
        PRINT_ERROR "Authentication error: " + e.message
        RETURN EXIT_AUTH_REQUIRED

    CATCH error AS e
        PRINT_ERROR "Error: " + e.message
        IF config.debug THEN
            PRINT_ERROR e.stackTrace
        END IF
        RETURN EXIT_ERROR
    END TRY
END
```

### 16.2 Search Command

```pseudocode
COMMAND: search

DEFINITION:
    name: "search"
    description: "Search for movies and TV shows"
    requiresAuth: false
    aliases: ["s", "find"]
    options: [
        Option("query", alias: "q", required: true, description: "Search query"),
        Option("type", alias: "t", values: ["movie", "series", "all"], default: "all"),
        Option("genre", alias: "g", description: "Filter by genre"),
        Option("year", description: "Filter by year or range (e.g., 2020 or 2015-2020)"),
        Option("platform", alias: "p", description: "Filter by platform"),
        Option("interactive", alias: "i", type: boolean, description: "Interactive mode"),
        Option("format", alias: "f", values: ["table", "json", "minimal"], default: "table")
    ]


ALGORITHM: ExecuteSearchCommand
INPUT: options (object)
OUTPUT: CommandResult

BEGIN
    // Build search query
    query <- options.query

    filters <- NEW SearchFilters()

    IF options.type != "all" THEN
        filters.content_types <- [ParseContentType(options.type)]
    END IF

    IF options.genre IS NOT NULL THEN
        filters.genres <- [ParseGenre(options.genre)]
    END IF

    IF options.year IS NOT NULL THEN
        filters.year_range <- ParseYearRange(options.year)
    END IF

    IF options.platform IS NOT NULL THEN
        filters.platforms <- [ParsePlatform(options.platform)]
    END IF

    // Execute search
    ShowSpinner("Searching...")

    results <- AWAIT ExecuteSearch(query, filters)

    HideSpinner()

    IF results.total_count = 0 THEN
        PRINT "No results found for: " + query
        RETURN CommandResult(success: true, data: [])
    END IF

    // Interactive mode
    IF options.interactive THEN
        RETURN RunInteractiveSearchBrowser(results)
    END IF

    // Display results
    DisplaySearchResults(results, options.format)

    RETURN CommandResult(success: true, data: results)
END
```

### 16.3 Error Classification and Retry Strategy

```pseudocode
HIERARCHY ErrorTypes:

  NetworkError
    - ConnectionTimeout      // retry: exponential backoff
    - DNSFailure             // retry: failover to backup
    - SSLError               // no retry, alert

  AuthError
    - TokenExpired           // auto-refresh, then retry
    - TokenInvalid           // re-authenticate
    - InsufficientScope      // show upgrade prompt

  APIError
    - RateLimited (429)      // wait, retry with backoff
    - ServiceUnavailable (503)  // circuit breaker
    - ValidationError (400)  // user feedback
    - NotFound (404)         // graceful degradation

  DataError
    - SyncConflict           // CRDT merge, auto-resolve
    - StaleData              // refresh cache
    - CorruptedData          // restore from backup

  PlatformError
    - YouTubeQuotaExceeded   // fallback to cache
    - StreamingUnavailable   // suggest alternatives
    - RegionRestricted       // notify user


ALGORITHM: ExecuteWithRetry
INPUT: operation (Function), config (RetryConfig)
OUTPUT: Result<T, Error>

CONSTANTS:
    DEFAULT_MAX_RETRIES = 3
    DEFAULT_BASE_DELAY = 1000
    DEFAULT_MAX_DELAY = 30000
    DEFAULT_BACKOFF = 2.0
    DEFAULT_JITTER = 0.3

BEGIN
    config <- config OR GetDefaultRetryConfig()
    attempt <- 0

    LOOP
        attempt <- attempt + 1

        TRY
            result <- AWAIT operation()
            RETURN Success(result)

        CATCH error
            // Check if error is retryable
            IF NOT IsRetryableError(error) THEN
                RETURN Failure(error)
            END IF

            IF attempt >= config.maxRetries THEN
                LOG_ERROR("Max retries exceeded", {
                    attempts: attempt,
                    error: error.message
                })
                RETURN Failure(error)
            END IF

            // Calculate delay with exponential backoff and jitter
            baseDelay <- config.baseDelay * (config.backoffMultiplier ^ (attempt - 1))
            jitter <- baseDelay * config.jitterFactor * Random(-1, 1)
            delay <- MIN(baseDelay + jitter, config.maxDelay)

            LOG_WARNING("Retrying operation", {
                attempt: attempt,
                maxRetries: config.maxRetries,
                delay: delay,
                error: error.message
            })

            AWAIT Sleep(delay)
        END TRY
    END LOOP
END
```

### 16.4 Circuit Breaker Pattern

```pseudocode
ENUM CircuitState:
    CLOSED      // Normal operation
    OPEN        // Failing, reject requests
    HALF_OPEN   // Testing if service recovered

STRUCTURE CircuitBreaker:
    state: CircuitState
    failureCount: integer
    successCount: integer
    lastFailureTime: timestamp
    failureThreshold: integer
    successThreshold: integer
    resetTimeout: integer         // seconds

CONSTANTS:
    DEFAULT_FAILURE_THRESHOLD = 5
    DEFAULT_SUCCESS_THRESHOLD = 2
    DEFAULT_RESET_TIMEOUT = 60


ALGORITHM: CircuitBreakerExecute
INPUT: breaker (CircuitBreaker), operation (Function)
OUTPUT: Result<T, Error>

BEGIN
    // Check if circuit is open
    IF breaker.state = CircuitState.OPEN THEN
        // Check if reset timeout has elapsed
        IF GetCurrentTime() - breaker.lastFailureTime > breaker.resetTimeout THEN
            breaker.state <- CircuitState.HALF_OPEN
            breaker.successCount <- 0
            LOG_INFO("Circuit breaker entering half-open state")
        ELSE
            THROW CircuitOpenError("Circuit breaker is open")
        END IF
    END IF

    TRY
        result <- AWAIT operation()

        // Record success
        RecordSuccess(breaker)

        RETURN Success(result)

    CATCH error
        // Record failure
        RecordFailure(breaker, error)

        THROW error
    END TRY
END
```

### 16.5 Graceful Degradation

```pseudocode
ENUM DegradationLevel:
    FULL        // All features available
    REDUCED     // Some features disabled
    MINIMAL     // Core features only
    OFFLINE     // Offline mode


ALGORITHM: SearchWithFallbacks
INPUT: query (SearchQuery)
OUTPUT: SearchResults

BEGIN
    primary <- () => ExecuteHybridSearch(query)

    fallbacks <- [
        Fallback("cached_results", () => GetCachedSearch(query)),
        Fallback("keyword_only", () => ExecuteKeywordSearch(query)),
        Fallback("trending", () => GetTrendingContent())
    ]

    result <- AWAIT ExecuteWithFallback(primary, fallbacks)

    IF result.isSuccess THEN
        RETURN result.value
    ELSE
        // Return empty results as last resort
        RETURN SearchResults(results: [], total_count: 0)
    END IF
END
```

---

# Appendix

## A. Complexity Summary

### Algorithm Complexity Table

| Algorithm | Time Complexity | Space Complexity | Notes |
|-----------|----------------|------------------|-------|
| **Core Data Structures** |
| Content Lookup | O(1) | O(1) | Hash index |
| Genre Mapping | O(g * m) | O(g) | g=genres, m=mappings |
| Entity Resolution | O(log n + k) | O(k) | k=candidates |
| Embedding Generation | O(d) | O(d) | d=768 dimensions |
| **Search & Personalization** |
| Intent Parsing | O(p * q) | O(1) | p=patterns, q=query |
| Vector Search | O(log n) | O(k) | HNSW index |
| Hybrid Search | O(m log m) | O(m) | m=candidates |
| MMR Diversity | O(k * s) | O(s) | k=candidates, s=selected |
| LoRA Forward | O(r * d) | O(r) | r=rank |
| **Real-time Sync** |
| HLC Operations | O(1) | O(1) | Constant |
| LWW Merge | O(1) | O(1) | Constant |
| OR-Set Merge | O(n * k) | O(n * k) | n=elements, k=tags |
| **Authentication** |
| OAuth PKCE Flow | O(1) | O(1) | Per-request |
| JWT Validation | O(1) | O(1) | Cached keys |
| Token Rotation | O(1) | O(f) | f=family size |
| Rate Limiting | O(1) | O(k) | k=unique keys |
| **Error Handling** |
| Retry with Backoff | O(r) | O(1) | r=retry count |
| Circuit Breaker | O(1) | O(1) | Amortized |

## B. Storage Requirements

| Component | Per-Unit Size | Total (Scale) |
|-----------|--------------|---------------|
| CanonicalContent | ~20 KB | 400 GB (20M titles) |
| External IDs | ~200 bytes | 4 GB |
| Embeddings | 3 KB | 60 GB |
| Availability | ~500 bytes | 10 GB (1B entries) |
| User Preferences | ~10 KB | 100 GB (10M users) |
| LoRA Adapter | ~10 KB | 100 GB (10M users) |
| OAuth State | 200 bytes | 200 MB (active) |
| Rate Limit Buckets | 100 bytes | 100 MB |

## C. Error Codes

| Code | Name | Description | User Message |
|------|------|-------------|--------------|
| E1001 | AUTH_REQUIRED | Not authenticated | Please log in to continue |
| E1002 | TOKEN_EXPIRED | Access token expired | Your session expired. Refreshing... |
| E1003 | INVALID_TOKEN | Token validation failed | Please log in again |
| E2001 | RATE_LIMITED | Rate limit exceeded | Too many requests. Try again in {retry_after}s |
| E2002 | SERVICE_DOWN | Service unavailable | Service temporarily unavailable |
| E3001 | NOT_FOUND | Content not found | Content not found |
| E3002 | DEVICE_OFFLINE | Device not online | Device is offline |
| E4001 | SYNC_CONFLICT | Data conflict | Syncing your data... |
| E4002 | NETWORK_ERROR | Network failure | Check your internet connection |

---

## Document Metadata

| Property | Value |
|----------|-------|
| **Total Algorithms** | 45+ |
| **Total Data Structures** | 30+ |
| **Total Enumerations** | 15+ |
| **Source Documents** | 4 parts |
| **Combined Pages** | ~200 |
| **Phase Status** | Complete |
| **Next Phase** | Architecture |
| **Review Required** | Architecture team, ML team, Security team, DevOps team |

---

**END OF PHASE 2 MASTER PSEUDOCODE DOCUMENT**
