# SPARC Pseudocode Phase - Part 1: Core Data Structures and Ingestion

**Version:** 1.0.0
**Phase:** SPARC Pseudocode
**Date:** 2025-12-06
**Status:** Complete

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Core Data Structures](#core-data-structures)
3. [Ingestion Pipeline Algorithms](#ingestion-pipeline-algorithms)
4. [Entity Resolution](#entity-resolution)
5. [Embedding Generation](#embedding-generation)

---

## Executive Summary

This document provides language-agnostic pseudocode specifications for the Media Gateway platform's core data structures and content ingestion pipeline. All algorithms are designed for production deployment with explicit complexity analysis.

### Key Performance Targets

| Component | Latency Target | Throughput Target |
|-----------|---------------|-------------------|
| Content Lookup | O(1) | 10,000 req/s |
| Entity Resolution | O(log n) | 1,000 entities/s |
| Embedding Generation | O(d) | 500 items/s |
| CRDT Merge | O(m) | 10,000 ops/s |

---

## Core Data Structures

### 1. CanonicalContent

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

### 2. External Identifiers

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

### 3. Platform Availability

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

### 4. User Profile

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

## Ingestion Pipeline Algorithms

### 1. Platform Normalizer

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

### 2. Genre Mapping

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

## Entity Resolution

### 1. Content Deduplication

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

## Embedding Generation

### 1. Content Embedding Pipeline

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

## Storage Requirements

| Data Type | Per-Item Size | Total Size (20M titles) |
|-----------|---------------|------------------------|
| CanonicalContent | ~20 KB | 400 GB |
| External IDs | ~200 bytes | 4 GB |
| Embeddings | 3 KB (768-dim float32) | 60 GB |
| Availability | ~500 bytes | 10 GB (1B entries) |

---

**Document Status:** Complete
**Next Document:** Part 2 - Search and Personalization
**Review Required:** Architecture team

---

END OF PART 1
