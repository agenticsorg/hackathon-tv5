# Media Gateway Data Structures Pseudocode
## SPARC Pseudocode Phase - Core Type Definitions

**Document Version:** 1.0.0
**Date:** 2025-12-06
**Status:** SPARC Pseudocode Phase
**Author:** Data Structure Design Agent

---

## Table of Contents

1. [Content Entity Types](#1-content-entity-types)
2. [User Domain Types](#2-user-domain-types)
3. [Search and Recommendation Types](#3-search-and-recommendation-types)
4. [Synchronization Types (CRDT)](#4-synchronization-types-crdt)
5. [MCP Protocol Types](#5-mcp-protocol-types)
6. [Complexity Analysis](#6-complexity-analysis)

---

## 1. Content Entity Types

### 1.1 CanonicalContent

**Purpose:** Unified representation of media content across all streaming platforms

**Design Decision:** Single source of truth for content metadata, normalized from multiple platform-specific formats

```
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


TYPE SeriesMetadata

  FIELDS:
    total_seasons: integer            // Total number of seasons
    total_episodes: integer           // Total episodes across all seasons
    status: SeriesStatus              // ENUM(ongoing, completed, cancelled)
    season_data: List<SeasonSummary>  // Summary per season

  INVARIANTS:
    - total_seasons >= 1
    - total_episodes >= total_seasons (at least 1 ep/season)

END TYPE


TYPE SeasonSummary

  FIELDS:
    season_number: integer            // 1-indexed season number
    episode_count: integer            // Episodes in this season
    air_date: Date NULLABLE           // Season premiere date
    overview: string NULLABLE         // Season description

END TYPE


TYPE SeriesStatus ENUM
  VALUES:
    ONGOING         // Currently airing new episodes
    COMPLETED       // Finished, no more episodes planned
    CANCELLED       // Discontinued before planned end
    HIATUS          // Temporarily on break
END ENUM
```

**Complexity Notes:**
- Storage: O(1) per content item
- Retrieval by ID: O(1) with hash index
- Search by genre: O(n) without index, O(log n) with B-tree on genres

---

### 1.2 ExternalIds

**Purpose:** Cross-reference content across multiple identifier systems

**Design Decision:** Support multiple ID schemes for maximum compatibility with aggregator APIs

```
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
    // Extensible for new platforms
END ENUM
```

**Complexity Notes:**
- has_identifier(): O(1)
- get_canonical_id(): O(1)
- Lookup by external ID: O(1) with hash index on each ID field

---

### 1.3 PlatformAvailability

**Purpose:** Track where and how content is available for viewing

**Design Decision:** Support multiple availability types (subscription, rental, purchase) with regional variations

```
TYPE PlatformAvailability

  FIELDS:
    platform: Platform                // Streaming platform
    region: Region                    // ISO 3166-1 alpha-2 country code
    availability_type: AvailabilityType  // subscription, rental, purchase, free

    // Pricing (null for subscription/free)
    price: Money NULLABLE             // Rental or purchase price

    // Deep linking
    deep_link: URL                    // Platform-specific deep link
                                       // e.g., "netflix://title/80123456"
    web_fallback: URL                 // HTTPS fallback URL

    // Temporal availability
    available_from: Timestamp         // When content became available
    expires_at: Timestamp NULLABLE    // When content leaves platform (null = indefinite)

    // Quality metadata
    video_quality: Set<VideoQuality>  // SD, HD, UHD, HDR
    audio_tracks: Set<AudioTrack>     // Available audio languages
    subtitle_tracks: Set<SubtitleTrack>  // Available subtitles

  METHODS:

    FUNCTION is_currently_available() -> boolean
      // Check if content is available right now
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
      // Days remaining before content expires
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

    FUNCTION get_display_price() -> string
      // Format price for user display
      BEGIN
        IF availability_type = AvailabilityType.SUBSCRIPTION THEN
          RETURN "Included with subscription"
        ELSE IF availability_type = AvailabilityType.FREE THEN
          RETURN "Free"
        ELSE IF price IS NOT NULL THEN
          RETURN price.format_with_currency()  // e.g., "$3.99"
        ELSE
          RETURN "Price unavailable"
        END IF
      END

  INVARIANTS:
    - available_from MUST be <= expires_at (if expires_at is not null)
    - price MUST be null for SUBSCRIPTION and FREE types
    - price MUST be non-null for RENTAL and PURCHASE types
    - deep_link MUST be valid platform-specific URL scheme

END TYPE


TYPE AvailabilityType ENUM
  VALUES:
    SUBSCRIPTION    // Included with platform subscription
    RENTAL          // Time-limited rental (24-48 hours)
    PURCHASE        // Permanent ownership
    FREE            // Free with ads (AVOD)
END ENUM


TYPE VideoQuality ENUM
  VALUES:
    SD              // Standard definition (480p)
    HD              // High definition (720p/1080p)
    UHD             // Ultra HD (4K)
    HDR             // High dynamic range
    DOLBY_VISION    // Dolby Vision HDR
END ENUM


TYPE AudioTrack

  FIELDS:
    language: LanguageCode            // ISO 639-1 language code
    format: AudioFormat               // Stereo, 5.1, Atmos
    is_original: boolean              // Original language track

END TYPE


TYPE AudioFormat ENUM
  VALUES:
    STEREO
    SURROUND_5_1
    SURROUND_7_1
    DOLBY_ATMOS
    DTS_X
END ENUM


TYPE SubtitleTrack

  FIELDS:
    language: LanguageCode            // ISO 639-1 language code
    is_sdh: boolean                   // Subtitles for deaf/hard-of-hearing
    is_forced: boolean                // Forced narrative subtitles

END TYPE


TYPE Region STRING
  // ISO 3166-1 alpha-2 country codes
  // Examples: "US", "CA", "UK", "DE", "FR", "JP"

  INVARIANTS:
    - Length MUST be exactly 2 characters
    - Characters MUST be uppercase A-Z

END TYPE


TYPE Money

  FIELDS:
    amount: Decimal                   // Price amount (2 decimal places)
    currency: CurrencyCode            // ISO 4217 currency code

  METHODS:

    FUNCTION format_with_currency() -> string
      BEGIN
        symbol <- GET_CURRENCY_SYMBOL(currency)
        RETURN symbol + amount.to_string_with_precision(2)
      END

END TYPE


TYPE CurrencyCode STRING
  // ISO 4217 currency codes
  // Examples: "USD", "CAD", "GBP", "EUR", "JPY"

END TYPE
```

**Complexity Notes:**
- is_currently_available(): O(1)
- days_until_expiry(): O(1)
- Filter content by region: O(n) without index, O(log n) with composite index on (platform, region)

---

### 1.4 ContentRating

**Purpose:** Age-appropriate content classification by region

**Design Decision:** Support multiple regional rating systems (MPAA, BBFC, FSK, etc.)

```
TYPE ContentRating

  FIELDS:
    system: RatingSystem              // Rating system (MPAA, BBFC, etc.)
    rating: string                    // Rating value (e.g., "PG-13", "15")
    descriptors: List<string>         // Content descriptors
                                       // e.g., ["Violence", "Language", "Drug Use"]
    certification: string NULLABLE    // Official certification ID

  METHODS:

    FUNCTION get_minimum_age() -> integer NULLABLE
      // Map rating to minimum recommended age
      BEGIN
        MATCH (system, rating)
          CASE (RatingSystem.MPAA, "G"):        RETURN 0
          CASE (RatingSystem.MPAA, "PG"):       RETURN 0
          CASE (RatingSystem.MPAA, "PG-13"):    RETURN 13
          CASE (RatingSystem.MPAA, "R"):        RETURN 17
          CASE (RatingSystem.MPAA, "NC-17"):    RETURN 18

          CASE (RatingSystem.BBFC, "U"):        RETURN 0
          CASE (RatingSystem.BBFC, "PG"):       RETURN 8
          CASE (RatingSystem.BBFC, "12A"):      RETURN 12
          CASE (RatingSystem.BBFC, "15"):       RETURN 15
          CASE (RatingSystem.BBFC, "18"):       RETURN 18

          CASE (RatingSystem.FSK, "0"):         RETURN 0
          CASE (RatingSystem.FSK, "6"):         RETURN 6
          CASE (RatingSystem.FSK, "12"):        RETURN 12
          CASE (RatingSystem.FSK, "16"):        RETURN 16
          CASE (RatingSystem.FSK, "18"):        RETURN 18

          DEFAULT:                               RETURN NULL
        END MATCH
      END

END TYPE


TYPE RatingSystem ENUM
  VALUES:
    MPAA           // Motion Picture Association (USA)
    TV_PARENTAL    // TV Parental Guidelines (USA)
    BBFC           // British Board of Film Classification (UK)
    FSK            // Freiwillige Selbstkontrolle (Germany)
    CHVRS          // Canadian Home Video Rating System
    ACB            // Australian Classification Board
    EIRIN          // Eiga Rinri Kanri Iinkai (Japan)
END ENUM
```

---

### 1.5 Credits

**Purpose:** Cast and crew information for content

**Design Decision:** Separate cast and crew roles with positional ordering for display

```
TYPE Credits

  FIELDS:
    cast: List<CastMember>            // Actors/voice actors (ordered by billing)
    crew: List<CrewMember>            // Directors, writers, producers

  METHODS:

    FUNCTION get_primary_cast(limit: integer) -> List<CastMember>
      // Get top-billed cast members
      BEGIN
        RETURN cast.slice(0, limit)
      END

    FUNCTION get_directors() -> List<CrewMember>
      // Filter crew by director role
      BEGIN
        directors <- EMPTY_LIST
        FOR EACH member IN crew DO
          IF member.job = "Director" THEN
            directors.append(member)
          END IF
        END FOR
        RETURN directors
      END

END TYPE


TYPE CastMember

  FIELDS:
    person_id: UUID                   // Internal person identifier
    name: string                      // Actor name
    character: string                 // Character name
    order: integer                    // Billing order (0 = top billing)
    profile_image: URL NULLABLE       // Actor headshot

  INVARIANTS:
    - order MUST be >= 0
    - order MUST be unique within a Credits.cast list

END TYPE


TYPE CrewMember

  FIELDS:
    person_id: UUID                   // Internal person identifier
    name: string                      // Crew member name
    job: string                       // Job title (Director, Writer, Producer, etc.)
    department: string                // Department (Directing, Writing, Production)
    profile_image: URL NULLABLE       // Profile photo

END TYPE
```

**Complexity Notes:**
- get_primary_cast(k): O(k) where k is the limit
- get_directors(): O(n) where n is crew size (typically <100)

---

### 1.6 ContentImages

**Purpose:** Media assets (posters, backdrops, stills) for content

**Design Decision:** Multiple aspect ratios and sizes for responsive display

```
TYPE ContentImages

  FIELDS:
    poster_portrait: ImageSet         // Vertical poster (2:3 aspect ratio)
    backdrop_landscape: ImageSet      // Horizontal backdrop (16:9 aspect ratio)
    logo: ImageSet NULLABLE           // Transparent logo overlay
    stills: List<ImageSet>            // Scene screenshots

END TYPE


TYPE ImageSet

  FIELDS:
    base_url: URL                     // CDN base URL
    sizes: Map<ImageSize, string>     // Size -> relative path
                                       // e.g., SMALL -> "/w300/abc123.jpg"
    aspect_ratio: float               // Width / Height (e.g., 0.667 for 2:3)

  METHODS:

    FUNCTION get_url(size: ImageSize) -> URL
      // Construct full image URL for requested size
      BEGIN
        IF sizes.contains(size) THEN
          relative_path <- sizes.get(size)
          RETURN base_url + relative_path
        ELSE
          // Fallback to largest available
          largest <- sizes.get_largest_key()
          relative_path <- sizes.get(largest)
          RETURN base_url + relative_path
        END IF
      END

END TYPE


TYPE ImageSize ENUM
  VALUES:
    THUMBNAIL      // 100-200px width
    SMALL          // 300-400px width
    MEDIUM         // 500-700px width
    LARGE          // 1000-1500px width
    ORIGINAL       // Original resolution
END ENUM
```

---

## 2. User Domain Types

### 2.1 UserProfile

**Purpose:** User account and preference data

**Design Decision:** Privacy-first design with on-device processing for sensitive data

```
TYPE UserProfile

  FIELDS:
    // Identity
    user_id: UUID                     // Internal user identifier
    external_auth_id: string          // OAuth provider ID (Google, Apple, etc.)

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


TYPE PrivacySettings

  FIELDS:
    // GDPR/CCPA compliance flags
    analytics_consent: boolean        // Allow usage analytics
    personalization_consent: boolean  // Allow personalized recommendations
    cross_device_sync: boolean        // Enable device synchronization
    history_retention: boolean        // Store watch history server-side

    // Data export/deletion
    data_export_requested: Timestamp NULLABLE   // User requested data export
    data_deletion_requested: Timestamp NULLABLE // User requested account deletion

  METHODS:

    FUNCTION can_collect_analytics() -> boolean
      BEGIN
        RETURN analytics_consent = true
      END

    FUNCTION can_personalize() -> boolean
      BEGIN
        RETURN personalization_consent = true
      END

END TYPE
```

---

### 2.2 UserPreferences

**Purpose:** User content preferences and platform subscriptions

**Design Decision:** Explicit preferences + implicit learning from behavior

```
TYPE UserPreferences

  FIELDS:
    // Explicit preferences
    favorite_genres: Set<Genre>       // User-selected favorite genres
    disliked_genres: Set<Genre>       // Genres to avoid

    preferred_languages: List<LanguageCode>  // Ordered by preference

    // Platform subscriptions (declared by user)
    subscribed_platforms: Set<Platform>  // Active subscriptions

    // Content filters
    max_content_rating: ContentRating NULLABLE  // Age-appropriate content filter

    // Viewing preferences
    preferred_video_quality: VideoQuality  // Preferred streaming quality
    autoplay_next: boolean               // Auto-play next episode

  METHODS:

    FUNCTION add_favorite_genre(genre: Genre) -> void
      BEGIN
        favorite_genres.add(genre)
        disliked_genres.remove(genre)  // Can't be both favorite and disliked
      END

    FUNCTION add_disliked_genre(genre: Genre) -> void
      BEGIN
        disliked_genres.add(genre)
        favorite_genres.remove(genre)  // Can't be both favorite and disliked
      END

    FUNCTION has_platform_subscription(platform: Platform) -> boolean
      BEGIN
        RETURN subscribed_platforms.contains(platform)
      END

  INVARIANTS:
    - favorite_genres and disliked_genres MUST be disjoint sets
    - preferred_languages MUST NOT contain duplicates

END TYPE


TYPE Genre ENUM
  VALUES:
    ACTION
    ADVENTURE
    ANIMATION
    COMEDY
    CRIME
    DOCUMENTARY
    DRAMA
    FAMILY
    FANTASY
    HISTORY
    HORROR
    MUSIC
    MYSTERY
    ROMANCE
    SCIENCE_FICTION
    THRILLER
    WAR
    WESTERN
END ENUM
```

---

### 2.3 WatchHistory (CRDT-based)

**Purpose:** User viewing history with conflict-free synchronization

**Design Decision:** Last-Write-Wins Register per content item, on-device storage by default

```
TYPE WatchHistory

  FIELDS:
    entries: Map<ContentId, WatchEntry>  // ContentId -> Watch metadata
    hlc: HybridLogicalClock              // For causality tracking

  METHODS:

    FUNCTION add_watch_entry(
      content_id: ContentId,
      progress_seconds: integer,
      device_id: DeviceId
    ) -> void
      // Add or update watch entry with LWW semantics
      BEGIN
        current_hlc <- hlc.now()

        new_entry <- WatchEntry {
          content_id: content_id,
          progress_seconds: progress_seconds,
          total_seconds: GET_CONTENT_RUNTIME(content_id),
          last_watched: current_hlc.physical_time,
          device_id: device_id,
          hlc_timestamp: current_hlc
        }

        IF entries.contains(content_id) THEN
          existing <- entries.get(content_id)

          // Last-Write-Wins: Compare HLC timestamps
          IF current_hlc > existing.hlc_timestamp THEN
            entries.set(content_id, new_entry)
          END IF
        ELSE
          entries.set(content_id, new_entry)
        END IF
      END

    FUNCTION get_watch_progress(content_id: ContentId) -> float
      // Get watch progress percentage (0.0-1.0)
      BEGIN
        IF NOT entries.contains(content_id) THEN
          RETURN 0.0
        END IF

        entry <- entries.get(content_id)
        IF entry.total_seconds = 0 THEN
          RETURN 0.0
        END IF

        progress_pct <- entry.progress_seconds / entry.total_seconds
        RETURN MIN(progress_pct, 1.0)
      END

    FUNCTION merge(remote_history: WatchHistory) -> void
      // Merge remote watch history (CRDT merge operation)
      BEGIN
        FOR EACH (content_id, remote_entry) IN remote_history.entries DO
          IF entries.contains(content_id) THEN
            local_entry <- entries.get(content_id)

            // LWW: Keep entry with larger HLC
            IF remote_entry.hlc_timestamp > local_entry.hlc_timestamp THEN
              entries.set(content_id, remote_entry)
            END IF
          ELSE
            // No local entry, add remote entry
            entries.set(content_id, remote_entry)
          END IF
        END FOR

        // Update HLC to account for remote clock
        hlc.update(remote_history.hlc)
      END

  INVARIANTS:
    - progress_seconds MUST be <= total_seconds
    - hlc MUST advance monotonically

END TYPE


TYPE WatchEntry

  FIELDS:
    content_id: ContentId             // Reference to content
    progress_seconds: integer         // Playback position in seconds
    total_seconds: integer            // Total content duration
    last_watched: Timestamp           // Last viewing timestamp
    device_id: DeviceId               // Device used for watching
    hlc_timestamp: HybridLogicalClock // For conflict resolution

END TYPE
```

**Complexity Notes:**
- add_watch_entry(): O(1) average case (hash map)
- merge(): O(m) where m is number of entries in remote history
- get_watch_progress(): O(1)

---

### 2.4 Watchlist (CRDT OR-Set)

**Purpose:** User's "to watch" list with add-wins conflict resolution

**Design Decision:** Observed-Remove Set (OR-Set) CRDT for conflict-free multi-device sync

```
TYPE Watchlist

  FIELDS:
    added: Map<ContentId, Set<UniqueTag>>     // Content -> set of add tags
    removed: Set<UniqueTag>                    // Set of removed tags

  METHODS:

    FUNCTION add(content_id: ContentId, device_id: DeviceId) -> void
      // Add content to watchlist with unique tag
      BEGIN
        tag <- UniqueTag {
          content_id: content_id,
          device_id: device_id,
          timestamp: GET_CURRENT_TIMESTAMP(),
          random_id: GENERATE_UUID()
        }

        IF added.contains(content_id) THEN
          existing_tags <- added.get(content_id)
          existing_tags.add(tag)
        ELSE
          added.set(content_id, Set { tag })
        END IF
      END

    FUNCTION remove(content_id: ContentId) -> void
      // Remove content from watchlist (mark all tags as removed)
      BEGIN
        IF added.contains(content_id) THEN
          tags <- added.get(content_id)

          FOR EACH tag IN tags DO
            removed.add(tag)
          END FOR
        END IF
      END

    FUNCTION contains(content_id: ContentId) -> boolean
      // Check if content is in watchlist (not all tags removed)
      BEGIN
        IF NOT added.contains(content_id) THEN
          RETURN false
        END IF

        tags <- added.get(content_id)

        FOR EACH tag IN tags DO
          IF NOT removed.contains(tag) THEN
            RETURN true  // At least one add tag not removed
          END IF
        END FOR

        RETURN false  // All tags removed
      END

    FUNCTION get_all_items() -> List<ContentId>
      // Get all watchlist items (with at least one active tag)
      BEGIN
        items <- EMPTY_LIST

        FOR EACH (content_id, tags) IN added DO
          has_active_tag <- false

          FOR EACH tag IN tags DO
            IF NOT removed.contains(tag) THEN
              has_active_tag <- true
              BREAK
            END IF
          END FOR

          IF has_active_tag THEN
            items.append(content_id)
          END IF
        END FOR

        RETURN items
      END

    FUNCTION merge(remote_watchlist: Watchlist) -> void
      // Merge remote watchlist (CRDT merge operation)
      BEGIN
        // Union of added sets
        FOR EACH (content_id, remote_tags) IN remote_watchlist.added DO
          IF added.contains(content_id) THEN
            local_tags <- added.get(content_id)
            merged_tags <- local_tags.union(remote_tags)
            added.set(content_id, merged_tags)
          ELSE
            added.set(content_id, remote_tags)
          END IF
        END FOR

        // Union of removed sets
        removed <- removed.union(remote_watchlist.removed)
      END

  INVARIANTS:
    - UniqueTag MUST be globally unique (device_id + timestamp + random_id)
    - removed set only grows (tombstones)

END TYPE


TYPE UniqueTag

  FIELDS:
    content_id: ContentId             // Content being added
    device_id: DeviceId               // Device that added content
    timestamp: Timestamp              // When content was added
    random_id: UUID                   // Randomness for uniqueness

  METHODS:

    FUNCTION equals(other: UniqueTag) -> boolean
      BEGIN
        RETURN device_id = other.device_id AND
               timestamp = other.timestamp AND
               random_id = other.random_id
      END

    FUNCTION hash() -> integer
      BEGIN
        RETURN HASH(device_id, timestamp, random_id)
      END

END TYPE
```

**Complexity Notes:**
- add(): O(1) average case
- remove(): O(k) where k is number of tags for content (typically 1-3)
- contains(): O(k) where k is number of tags
- get_all_items(): O(n*k) where n is total content items, k is avg tags per item
- merge(): O(m*k) where m is remote entries, k is avg tags

**CRDT Properties:**
- **Commutativity**: add/remove operations can be applied in any order
- **Idempotence**: Applying same operation multiple times has same effect as once
- **Add-wins**: If concurrent add/remove, add wins (removed tags don't affect new adds)

---

### 2.5 Device

**Purpose:** Track user devices for synchronization

**Design Decision:** Each device has unique ID for CRDT causality tracking

```
TYPE Device

  FIELDS:
    device_id: UUID                   // Unique device identifier
    device_name: string               // User-friendly name
    device_type: DeviceType           // Phone, tablet, TV, etc.

    // Device capabilities
    max_video_quality: VideoQuality   // Maximum supported quality
    supported_codecs: Set<VideoCodec> // H.264, H.265, VP9, AV1

    // Registration metadata
    registered_at: Timestamp          // First registration time
    last_sync: Timestamp              // Last successful sync

    // Online status
    is_online: boolean                // Currently connected

  METHODS:

    FUNCTION update_last_sync() -> void
      BEGIN
        last_sync <- GET_CURRENT_TIMESTAMP()
      END

    FUNCTION mark_online() -> void
      BEGIN
        is_online <- true
      END

    FUNCTION mark_offline() -> void
      BEGIN
        is_online <- false
      END

END TYPE


TYPE DeviceType ENUM
  VALUES:
    SMARTPHONE
    TABLET
    SMART_TV
    STREAMING_STICK
    DESKTOP
    LAPTOP
    GAME_CONSOLE
END ENUM


TYPE VideoCodec ENUM
  VALUES:
    H264
    H265_HEVC
    VP9
    AV1
END ENUM
```

---

## 3. Search and Recommendation Types

### 3.1 SearchQuery

**Purpose:** User search request with natural language and structured filters

**Design Decision:** Hybrid search supporting both natural language and explicit filters

```
TYPE SearchQuery

  FIELDS:
    // Natural language query
    query_text: string                // User input (e.g., "movies like The Matrix")

    // Structured filters (all optional)
    filters: SearchFilters NULLABLE   // Genre, year, rating filters

    // Pagination
    page: integer DEFAULT 1           // Page number (1-indexed)
    page_size: integer DEFAULT 20     // Results per page

    // Search strategy hints
    strategy: SearchStrategy DEFAULT HYBRID  // Vector, graph, keyword, hybrid

    // User context (for personalization)
    user_id: UUID NULLABLE            // Requesting user (null for anonymous)
    region: Region DEFAULT "US"       // User region for availability

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
    content_types: Set<ContentType> NULLABLE  // movie, series, etc.

    year_range: YearRange NULLABLE    // Release year range
    rating_range: RatingRange NULLABLE  // User rating range (0.0-10.0)

    platforms: Set<Platform> NULLABLE // Filter by available platforms
    availability_type: AvailabilityType NULLABLE  // subscription, rental, etc.

    max_runtime_minutes: integer NULLABLE  // Maximum runtime

END TYPE


TYPE YearRange

  FIELDS:
    min_year: integer NULLABLE        // Minimum release year
    max_year: integer NULLABLE        // Maximum release year

  INVARIANTS:
    - If both specified: min_year <= max_year

END TYPE


TYPE RatingRange

  FIELDS:
    min_rating: float NULLABLE        // Minimum rating (0.0-10.0)
    max_rating: float NULLABLE        // Maximum rating (0.0-10.0)

  INVARIANTS:
    - If both specified: min_rating <= max_rating
    - min_rating MUST be in range [0.0, 10.0]
    - max_rating MUST be in range [0.0, 10.0]

END TYPE


TYPE SearchStrategy ENUM
  VALUES:
    VECTOR          // Semantic vector search only
    GRAPH           // Graph traversal only
    KEYWORD         // Keyword matching only
    HYBRID          // Combine all strategies with RRF
END ENUM
```

---

### 3.2 ParsedIntent

**Purpose:** Natural language understanding of search query

**Design Decision:** Extract semantic meaning from user queries (mood, themes, references)

```
TYPE ParsedIntent

  FIELDS:
    // Extracted entities
    mentioned_titles: List<string>    // Referenced content titles
    mentioned_people: List<string>    // Actor/director names

    // Semantic understanding
    mood: List<string>                // "dark", "uplifting", "intense"
    themes: List<string>              // "revenge", "coming-of-age"

    // Temporal constraints
    time_period: TimePeriod NULLABLE  // "80s movies", "recent releases"

    // Intent type
    intent_type: IntentType           // search, recommendation, question

  METHODS:

    FUNCTION has_content_reference() -> boolean
      BEGIN
        RETURN NOT mentioned_titles.is_empty()
      END

    FUNCTION has_person_reference() -> boolean
      BEGIN
        RETURN NOT mentioned_people.is_empty()
      END

END TYPE


TYPE TimePeriod

  FIELDS:
    era: string NULLABLE              // "80s", "90s", "2000s"
    relative: string NULLABLE         // "recent", "classic", "new"
    exact_year: integer NULLABLE      // Specific year

END TYPE


TYPE IntentType ENUM
  VALUES:
    SEARCH              // Direct search for content
    RECOMMENDATION      // "suggest something like X"
    QUESTION            // "what should I watch"
    TRIVIA              // "who directed X"
END ENUM
```

---

### 3.3 SearchResult

**Purpose:** Ranked search results with explainability

**Design Decision:** Include relevance scores and match reasons for transparency

```
TYPE SearchResult

  FIELDS:
    content: CanonicalContent         // Matched content

    // Ranking
    relevance_score: float            // Overall relevance (0.0-1.0)

    // Scoring breakdown
    vector_similarity: float          // Semantic similarity score
    graph_score: float                // Graph-based score
    keyword_score: float              // Keyword match score
    popularity_boost: float           // Trending boost

    // Explainability
    match_reasons: List<string>       // Why this content matched
                                       // e.g., ["Genre: Sci-Fi", "Similar to The Matrix"]

    // User-specific
    user_affinity: float NULLABLE     // Personalization score (null for anonymous)

  METHODS:

    FUNCTION get_primary_reason() -> string
      // Get most important match reason
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

**Complexity Notes:**
- Ranking N results: O(N log N) with heap-based top-K
- RRF fusion of K strategies: O(N * K) where N is result set size

---

### 3.4 Recommendation

**Purpose:** Personalized content recommendation with explanation

**Design Decision:** Transparent recommendations with clear reasoning

```
TYPE Recommendation

  FIELDS:
    content: CanonicalContent         // Recommended content

    // Confidence
    confidence_score: float           // Recommendation confidence (0.0-1.0)

    // Reasoning
    recommendation_type: RecommendationType  // Similar, genre-based, trending
    based_on: List<ContentId>         // Content this is based on

    explanation: string               // Human-readable explanation
                                       // e.g., "Because you watched The Matrix"

    // Metadata
    generated_at: Timestamp           // When recommendation was generated
    ttl_seconds: integer              // Recommendation validity duration

  METHODS:

    FUNCTION is_valid() -> boolean
      // Check if recommendation is still valid (not expired)
      BEGIN
        current_time <- GET_CURRENT_TIMESTAMP()
        age_seconds <- (current_time - generated_at).total_seconds()
        RETURN age_seconds < ttl_seconds
      END

END TYPE


TYPE RecommendationType ENUM
  VALUES:
    SIMILAR_CONTENT     // Based on similar content
    GENRE_MATCH         // Matches user genre preferences
    COLLABORATIVE       // Other users with similar taste
    TRENDING            // Popular/trending content
    CONTINUE_WATCHING   // Resume in-progress content
    NEW_RELEASE         // Newly available content
END ENUM
```

---

### 3.5 PersonalizationContext

**Purpose:** User context for SONA personalization engine

**Design Decision:** Privacy-preserving context with differential privacy

```
TYPE PersonalizationContext

  FIELDS:
    user_id: UUID                     // User identifier

    // Behavioral signals (on-device)
    recent_watches: List<ContentId>   // Last 20 watched items
    genre_distribution: Map<Genre, float>  // Genre preference weights

    // Temporal patterns
    preferred_watch_time: List<HourOfDay>  // When user typically watches
    binge_tendency: float             // Likelihood to watch multiple episodes

    // SONA LoRA adapter reference
    lora_adapter_id: string NULLABLE  // Per-user LoRA adapter ID

  METHODS:

    FUNCTION get_dominant_genres(top_k: integer) -> List<Genre>
      // Get user's top K favorite genres by weight
      BEGIN
        sorted_genres <- genre_distribution.sort_by_value_descending()
        RETURN sorted_genres.slice(0, top_k).keys()
      END

END TYPE


TYPE HourOfDay INTEGER
  // Hour in range [0, 23]

  INVARIANTS:
    - Value MUST be in range [0, 23]

END TYPE
```

---

## 4. Synchronization Types (CRDT)

### 4.1 HybridLogicalClock

**Purpose:** Causality tracking for distributed synchronization

**Design Decision:** HLC provides total ordering of events across devices

```
TYPE HybridLogicalClock

  FIELDS:
    physical_time: Timestamp          // Physical wall clock time
    logical_counter: integer          // Logical counter for causality
    node_id: UUID                     // Node/device identifier

  METHODS:

    FUNCTION now() -> HybridLogicalClock
      // Generate new HLC timestamp
      BEGIN
        current_physical <- GET_CURRENT_TIMESTAMP()

        IF current_physical > physical_time THEN
          // Physical time advanced
          RETURN HybridLogicalClock {
            physical_time: current_physical,
            logical_counter: 0,
            node_id: node_id
          }
        ELSE
          // Physical time same or in past (clock drift)
          RETURN HybridLogicalClock {
            physical_time: physical_time,
            logical_counter: logical_counter + 1,
            node_id: node_id
          }
        END IF
      END

    FUNCTION update(remote_hlc: HybridLogicalClock) -> void
      // Update HLC based on received remote timestamp
      BEGIN
        current_physical <- GET_CURRENT_TIMESTAMP()

        max_physical <- MAX(physical_time, remote_hlc.physical_time, current_physical)

        IF max_physical = physical_time AND max_physical = remote_hlc.physical_time THEN
          // Same physical time
          logical_counter <- MAX(logical_counter, remote_hlc.logical_counter) + 1
        ELSE IF max_physical = physical_time THEN
          logical_counter <- logical_counter + 1
        ELSE IF max_physical = remote_hlc.physical_time THEN
          logical_counter <- remote_hlc.logical_counter + 1
        ELSE
          logical_counter <- 0
        END IF

        physical_time <- max_physical
      END

    FUNCTION compare(other: HybridLogicalClock) -> Ordering
      // Compare two HLC timestamps for total ordering
      BEGIN
        IF physical_time < other.physical_time THEN
          RETURN LESS
        ELSE IF physical_time > other.physical_time THEN
          RETURN GREATER
        ELSE IF logical_counter < other.logical_counter THEN
          RETURN LESS
        ELSE IF logical_counter > other.logical_counter THEN
          RETURN GREATER
        ELSE IF node_id < other.node_id THEN
          RETURN LESS
        ELSE IF node_id > other.node_id THEN
          RETURN GREATER
        ELSE
          RETURN EQUAL
        END IF
      END

  INVARIANTS:
    - physical_time MUST advance monotonically (or logical_counter increments)
    - logical_counter MUST reset to 0 when physical_time advances

END TYPE


TYPE Ordering ENUM
  VALUES:
    LESS
    EQUAL
    GREATER
END ENUM
```

**Complexity Notes:**
- now(): O(1)
- update(): O(1)
- compare(): O(1)

**HLC Properties:**
- **Monotonicity**: Timestamps always advance
- **Causality**: If event A happened-before event B, then HLC(A) < HLC(B)
- **Total ordering**: Any two timestamps can be compared

---

### 4.2 LWWRegister<T>

**Purpose:** Last-Write-Wins CRDT for single-value synchronization

**Design Decision:** Simple conflict resolution - last writer wins based on HLC

```
GENERIC TYPE LWWRegister<T>

  FIELDS:
    value: T                          // Current value
    timestamp: HybridLogicalClock     // When value was written
    device_id: UUID                   // Device that wrote value

  METHODS:

    FUNCTION set(new_value: T, hlc: HybridLogicalClock, device: UUID) -> void
      // Update value with new write
      BEGIN
        IF hlc > timestamp THEN
          value <- new_value
          timestamp <- hlc
          device_id <- device
        ELSE IF hlc = timestamp AND device > device_id THEN
          // Tie-break by device ID for deterministic ordering
          value <- new_value
          timestamp <- hlc
          device_id <- device
        END IF
        // Else: ignore write (older timestamp)
      END

    FUNCTION get() -> T
      BEGIN
        RETURN value
      END

    FUNCTION merge(remote: LWWRegister<T>) -> void
      // Merge remote register (CRDT merge operation)
      BEGIN
        IF remote.timestamp > timestamp THEN
          value <- remote.value
          timestamp <- remote.timestamp
          device_id <- remote.device_id
        ELSE IF remote.timestamp = timestamp AND remote.device_id > device_id THEN
          value <- remote.value
          timestamp <- remote.timestamp
          device_id <- remote.device_id
        END IF
      END

END TYPE
```

**Example Usage:**
```
// User's preferred video quality setting (synced across devices)
TYPE UserVideoQualitySetting = LWWRegister<VideoQuality>

preferred_quality: UserVideoQualitySetting <- LWWRegister {
  value: VideoQuality.HD,
  timestamp: HLC.now(),
  device_id: current_device_id
}

// Update from mobile device
preferred_quality.set(VideoQuality.SD, mobile_hlc, mobile_device_id)

// Later merge from TV device (if TV timestamp is newer, TV setting wins)
preferred_quality.merge(tv_quality_setting)
```

---

### 4.3 ORSet<T> (Already covered in Watchlist)

See section 2.4 for complete OR-Set implementation.

---

### 4.4 SyncMessage

**Purpose:** Real-time synchronization messages over PubNub

**Design Decision:** Structured messages for different sync operations

```
TYPE SyncMessage

  FIELDS:
    message_id: UUID                  // Unique message identifier
    message_type: SyncMessageType     // Type of sync operation

    sender_device_id: UUID            // Device that sent message
    timestamp: HybridLogicalClock     // Message timestamp

    payload: SyncPayload              // Type-specific payload

  METHODS:

    FUNCTION serialize() -> bytes
      // Serialize to binary format for network transmission
      BEGIN
        RETURN MSGPACK_ENCODE(this)
      END

    STATIC FUNCTION deserialize(data: bytes) -> SyncMessage
      BEGIN
        RETURN MSGPACK_DECODE(data)
      END

END TYPE


TYPE SyncMessageType ENUM
  VALUES:
    WATCHLIST_ADD       // Add item to watchlist
    WATCHLIST_REMOVE    // Remove item from watchlist
    WATCH_PROGRESS      // Update watch progress
    PREFERENCE_UPDATE   // Update user preferences
    DEVICE_PRESENCE     // Device online/offline status
END ENUM


TYPE SyncPayload UNION
  VARIANTS:
    watchlist_add: WatchlistAddPayload
    watchlist_remove: WatchlistRemovePayload
    watch_progress: WatchProgressPayload
    preference_update: PreferenceUpdatePayload
    device_presence: DevicePresencePayload

END UNION


TYPE WatchlistAddPayload

  FIELDS:
    content_id: ContentId
    unique_tag: UniqueTag             // OR-Set tag

END TYPE


TYPE WatchlistRemovePayload

  FIELDS:
    content_id: ContentId
    removed_tags: Set<UniqueTag>      // Tags to mark as removed

END TYPE


TYPE WatchProgressPayload

  FIELDS:
    content_id: ContentId
    progress_seconds: integer
    total_seconds: integer

END TYPE


TYPE PreferenceUpdatePayload

  FIELDS:
    preference_key: string            // e.g., "preferred_video_quality"
    preference_value: string          // Serialized value
    hlc: HybridLogicalClock           // For LWW resolution

END TYPE


TYPE DevicePresencePayload

  FIELDS:
    device_id: UUID
    is_online: boolean
    last_active: Timestamp

END TYPE
```

**Complexity Notes:**
- serialize(): O(n) where n is payload size
- deserialize(): O(n) where n is message size

---

## 5. MCP Protocol Types

### 5.1 MCPRequest

**Purpose:** JSON-RPC 2.0 request for Model Context Protocol

**Design Decision:** Standard JSON-RPC for AI agent communication

```
TYPE MCPRequest

  FIELDS:
    jsonrpc: string DEFAULT "2.0"     // JSON-RPC version
    method: string                    // Method name (e.g., "search", "recommend")
    params: Map<string, JSONValue>    // Method parameters
    id: RequestId                     // Request identifier (string or integer)

  METHODS:

    FUNCTION to_json() -> string
      BEGIN
        RETURN JSON_ENCODE({
          "jsonrpc": jsonrpc,
          "method": method,
          "params": params,
          "id": id
        })
      END

    STATIC FUNCTION from_json(json_str: string) -> Result<MCPRequest, ParseError>
      BEGIN
        obj <- JSON_DECODE(json_str)

        IF obj.jsonrpc != "2.0" THEN
          RETURN ERROR("Invalid JSON-RPC version")
        END IF

        RETURN OK(MCPRequest {
          jsonrpc: obj.jsonrpc,
          method: obj.method,
          params: obj.params,
          id: obj.id
        })
      END

  INVARIANTS:
    - jsonrpc MUST be "2.0"
    - method MUST NOT be empty string

END TYPE


TYPE RequestId UNION
  VARIANTS:
    string_id: string
    integer_id: integer
    null_id: null                     // For notifications (no response expected)

END UNION
```

---

### 5.2 MCPResponse

**Purpose:** JSON-RPC 2.0 response from MCP server

**Design Decision:** Either result or error, never both

```
TYPE MCPResponse

  FIELDS:
    jsonrpc: string DEFAULT "2.0"     // JSON-RPC version
    id: RequestId                     // Matching request ID

    // Exactly one of these must be present
    result: JSONValue NULLABLE        // Success result
    error: MCPError NULLABLE          // Error object

  METHODS:

    FUNCTION is_success() -> boolean
      BEGIN
        RETURN result IS NOT NULL AND error IS NULL
      END

    FUNCTION is_error() -> boolean
      BEGIN
        RETURN error IS NOT NULL AND result IS NULL
      END

    FUNCTION to_json() -> string
      BEGIN
        IF is_success() THEN
          RETURN JSON_ENCODE({
            "jsonrpc": jsonrpc,
            "id": id,
            "result": result
          })
        ELSE
          RETURN JSON_ENCODE({
            "jsonrpc": jsonrpc,
            "id": id,
            "error": error
          })
        END IF
      END

  INVARIANTS:
    - Exactly one of result or error MUST be non-null

END TYPE


TYPE MCPError

  FIELDS:
    code: integer                     // Error code (JSON-RPC standard codes)
    message: string                   // Human-readable error message
    data: JSONValue NULLABLE          // Additional error data

END TYPE


// Standard JSON-RPC error codes
CONSTANTS:
  MCP_ERROR_PARSE = -32700            // Parse error
  MCP_ERROR_INVALID_REQUEST = -32600  // Invalid request
  MCP_ERROR_METHOD_NOT_FOUND = -32601 // Method not found
  MCP_ERROR_INVALID_PARAMS = -32602   // Invalid parameters
  MCP_ERROR_INTERNAL = -32603         // Internal error

  // Application-specific error codes (range -32000 to -32099)
  MCP_ERROR_CONTENT_NOT_FOUND = -32001    // Content not found
  MCP_ERROR_PLATFORM_UNAVAILABLE = -32002 // Platform API unavailable
  MCP_ERROR_QUOTA_EXCEEDED = -32003       // API quota exceeded
END CONSTANTS
```

---

### 5.3 MCPTool

**Purpose:** Tool definition for MCP server capabilities

**Design Decision:** Self-describing tools with JSON Schema for parameters

```
TYPE MCPTool

  FIELDS:
    name: string                      // Tool name (e.g., "semantic_search")
    description: string               // Human-readable description

    // JSON Schema for input parameters
    input_schema: JSONSchema          // Defines expected parameters

    // Handler function reference
    handler: ToolHandler              // Function to execute tool

  METHODS:

    FUNCTION validate_params(params: Map<string, JSONValue>) -> Result<void, ValidationError>
      // Validate parameters against input schema
      BEGIN
        errors <- JSON_SCHEMA_VALIDATE(params, input_schema)

        IF NOT errors.is_empty() THEN
          RETURN ERROR(errors.join(", "))
        ELSE
          RETURN OK(void)
        END IF
      END

    ASYNC FUNCTION execute(params: Map<string, JSONValue>) -> Result<JSONValue, ToolError>
      // Execute tool with validated parameters
      BEGIN
        validation <- validate_params(params)

        IF validation.is_error() THEN
          RETURN ERROR(ToolError.InvalidParams(validation.error()))
        END IF

        TRY
          result <- AWAIT handler(params)
          RETURN OK(result)
        CATCH e AS Error
          RETURN ERROR(ToolError.ExecutionFailed(e.message))
        END TRY
      END

END TYPE


TYPE ToolHandler FUNCTION
  SIGNATURE: ASYNC (params: Map<string, JSONValue>) -> JSONValue

END TYPE


TYPE ToolError UNION
  VARIANTS:
    InvalidParams: string             // Parameter validation failed
    ExecutionFailed: string           // Tool execution failed
    Timeout: Duration                 // Tool execution timed out

END UNION
```

**Example Tool Definition:**
```
semantic_search_tool: MCPTool <- {
  name: "semantic_search",
  description: "Search for movies and TV shows using natural language",

  input_schema: {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "Natural language search query",
        "minLength": 1,
        "maxLength": 500
      },
      "filters": {
        "type": "object",
        "properties": {
          "genres": {
            "type": "array",
            "items": { "type": "string" }
          },
          "year_range": {
            "type": "object",
            "properties": {
              "min_year": { "type": "integer" },
              "max_year": { "type": "integer" }
            }
          }
        }
      },
      "page": {
        "type": "integer",
        "minimum": 1,
        "default": 1
      }
    },
    "required": ["query"]
  },

  handler: ASYNC FUNCTION(params)
    BEGIN
      query <- SearchQuery {
        query_text: params["query"],
        filters: parse_filters(params["filters"]),
        page: params.get("page", 1)
      }

      results <- AWAIT search_engine.search(query)

      RETURN {
        "results": results.map(r => r.to_json()),
        "total_results": results.total_count,
        "page": query.page
      }
    END
}
```

---

### 5.4 MCPResource

**Purpose:** Static or dynamic resources exposed by MCP server

**Design Decision:** Resources can be static files or dynamically generated

```
TYPE MCPResource

  FIELDS:
    uri: string                       // Resource URI (e.g., "platform://netflix/catalog")
    name: string                      // Human-readable name
    description: string               // Resource description

    mime_type: string                 // Content MIME type

    content_provider: ResourceProvider  // Function to get resource content

  METHODS:

    ASYNC FUNCTION get_content() -> Result<bytes, ResourceError>
      // Fetch resource content
      BEGIN
        TRY
          content <- AWAIT content_provider()
          RETURN OK(content)
        CATCH e AS Error
          RETURN ERROR(ResourceError.FetchFailed(e.message))
        END TRY
      END

END TYPE


TYPE ResourceProvider FUNCTION
  SIGNATURE: ASYNC () -> bytes

END TYPE


TYPE ResourceError UNION
  VARIANTS:
    NotFound: string                  // Resource not found
    FetchFailed: string               // Failed to fetch content
    AccessDenied: string              // Permission denied

END UNION
```

**Example Resource:**
```
netflix_catalog_resource: MCPResource <- {
  uri: "platform://netflix/catalog",
  name: "Netflix Catalog",
  description: "Current Netflix content catalog for user's region",
  mime_type: "application/json",

  content_provider: ASYNC FUNCTION()
    BEGIN
      user_region <- get_user_region()
      catalog <- AWAIT netflix_connector.get_catalog(user_region)

      RETURN JSON_ENCODE(catalog)
    END
}
```

---

## 6. Complexity Analysis

### 6.1 Data Structure Operations

| Operation | Data Structure | Time Complexity | Space Complexity | Notes |
|-----------|---------------|----------------|------------------|-------|
| **Content lookup by ID** | CanonicalContent | O(1) | O(1) | Hash index on UUID |
| **Search by genre** | CanonicalContent | O(log n) | O(1) | B-tree index on genres |
| **Cross-reference ID** | ExternalIds | O(1) | O(1) | Hash index per ID type |
| **Check availability** | PlatformAvailability | O(k) | O(1) | k = platforms per content (~5-10) |
| **Add to watchlist** | Watchlist (OR-Set) | O(1) avg | O(tags) | tags typically 1-3 per item |
| **Merge watchlists** | Watchlist (OR-Set) | O(m*k) | O(m) | m = remote items, k = avg tags |
| **Update watch progress** | WatchHistory (LWW) | O(1) avg | O(1) | Hash map lookup + update |
| **Merge watch history** | WatchHistory (LWW) | O(m) | O(m) | m = remote entries |
| **HLC comparison** | HybridLogicalClock | O(1) | O(1) | Simple field comparison |
| **Vector search** | Ruvector | O(log n) | O(d) | HNSW index, d = embedding dim |
| **Graph traversal** | Ruvector | O(k*log n) | O(k) | k = traversal depth |
| **Recommendation generation** | Hybrid engine | O(n log n) | O(n) | Sorting N candidates |

### 6.2 Storage Requirements

| Data Type | Per-Item Size | Total Size (1M users) | Notes |
|-----------|--------------|---------------------|-------|
| **CanonicalContent** | ~20 KB | 400 GB (20M titles) | With all metadata |
| **UserProfile** | ~5 KB | 5 GB | Minimal user data |
| **WatchHistory** | ~100 bytes/entry | 10 GB (avg 100 items/user) | On-device primarily |
| **Watchlist** | ~200 bytes/entry | 4 GB (avg 20 items/user) | OR-Set with tags |
| **Embeddings** | 3 KB (768-dim) | 60 GB (20M titles) | Vector representations |
| **LoRA adapters** | 10 KB/user | 10 GB | Per-user personalization |

### 6.3 Network Bandwidth

| Operation | Payload Size | Frequency | Bandwidth/User |
|-----------|-------------|-----------|---------------|
| **Search request** | 1-2 KB | 5-10/session | 50-200 bytes/min |
| **Search results** | 20-50 KB | 5-10/session | 1-5 KB/min |
| **Recommendation** | 10-20 KB | 1/day | Negligible |
| **Watch progress sync** | 200 bytes | 1/minute (active viewing) | 3.3 bytes/sec |
| **Watchlist sync** | 500 bytes-2 KB | 1-5/session | 8-160 bytes/min |
| **MCP request/response** | 1-10 KB | Variable | Variable |

---

## Appendix: Type System Conventions

### Naming Conventions
- **Types**: PascalCase (e.g., `CanonicalContent`, `SearchQuery`)
- **Fields**: snake_case (e.g., `release_date`, `content_type`)
- **Methods**: snake_case (e.g., `is_valid()`, `get_all_items()`)
- **Constants**: UPPER_SNAKE_CASE (e.g., `MCP_ERROR_PARSE`)

### Nullability
- **NULLABLE**: Field may be null/undefined
- **DEFAULT value**: Field has default if not specified
- **Required fields**: No annotation means required

### Collections
- **List<T>**: Ordered collection with duplicates allowed
- **Set<T>**: Unordered collection, no duplicates
- **Map<K, V>**: Key-value mapping

### Time Types
- **Timestamp**: Absolute point in time (milliseconds since Unix epoch)
- **Date**: Calendar date without time
- **Duration**: Time span (seconds, minutes, etc.)

---

**Document Status:** Complete
**Next Phase:** Architecture (component design, service interfaces)
**Review Required:** Architecture team, security team

---

END OF PSEUDOCODE SPECIFICATION
