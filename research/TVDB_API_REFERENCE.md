<!--
Source: https://thetvdb.github.io/v4-api/ and https://thetvdb.com/api-information
Fetched: 2025-12-07T00:00:00.000Z
Topic: TheTVDB API v4 Documentation - Complete Reference Guide
-->

# TheTVDB API v4 - Complete Reference Guide

## Table of Contents
1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Authentication](#authentication)
4. [Core Endpoints](#core-endpoints)
5. [Data Models](#data-models)
6. [Rate Limits & Quotas](#rate-limits--quotas)
7. [Best Practices](#best-practices)
8. [Code Examples](#code-examples)
9. [Error Handling](#error-handling)
10. [Resources](#resources)

---

## Overview

TheTVDB is a community-driven TV and movie database founded in 2006, providing comprehensive metadata for hundreds of thousands of titles worldwide. The v4 API offers programmatic access to series, episodes, movies, people, artwork, and related metadata.

**Base URL:** `https://api4.thetvdb.com/v4`
**API Version:** 4.7.10
**Protocol:** HTTPS (TLSv1.2 minimum required)
**Response Format:** JSON

### Key Features
- JWT-based authentication with 1-month token validity
- Comprehensive series, episode, and movie data
- Multi-language support with translations
- Artwork and media assets
- People and character information
- Award and company data
- Advanced search and filtering capabilities
- Update tracking for incremental syncing

---

## Getting Started

### 1. Account Setup

1. **Create an account** at [TheTVDB.com](https://thetvdb.com)
2. **Obtain an API key** via the [API Key Dashboard](https://www.thetvdb.com/dashboard/account/apikey)
3. **Select appropriate subscription tier** based on your usage:
   - **Free**: Companies with <$50k annual revenue (requires attribution)
   - **$1,000/year**: $50k-$250k annual revenue
   - **$10,000/year**: $250k-$1M annual revenue
   - **Custom pricing**: $1M+ revenue or special arrangements
   - **User subscription**: $12/year for personal projects

### 2. Attribution Requirements

Unless explicitly approved otherwise, you **must display attribution** with a direct link to TheTVDB.com when showing API metadata to end users. Command-line tools and development libraries may show attribution in readme or about pages instead.

### 3. Security Requirements

- All API calls must use **HTTPS** with TLSv1.2 or higher
- Store API keys securely (environment variables, secure vaults)
- Never expose API keys in client-side code or public repositories
- Rotate tokens periodically for enhanced security

---

## Authentication

TheTVDB v4 API uses **JWT (JSON Web Token)** bearer authentication.

### Authentication Flow

#### 1. Login Request

**Endpoint:** `POST /login`

**Request Body:**
```json
{
  "apikey": "your-api-key-here",
  "pin": "optional-subscriber-pin"
}
```

**Parameters:**
- `apikey` (required): Your API key from the dashboard
- `pin` (optional): Subscriber PIN for user-supported accounts. Omit entirely if not using a subscriber account.

**Response (200 OK):**
```json
{
  "status": "success",
  "data": {
    "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

**Error Response (401 Unauthorized):**
```json
{
  "status": "failure",
  "message": "Unauthorized"
}
```

#### 2. Using the Token

Include the JWT token in the `Authorization` header for all subsequent requests:

```
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Token Validity:** 1 month from issuance

**Token Refresh:** Request a new token before expiration by calling `/login` again

---

## Core Endpoints

### Search & Discovery

#### Multi-Entity Search
**Endpoint:** `GET /search`

**Query Parameters:**
- `query` (string): Search query
- `type` (string): Filter by entity type (series, movie, people, company)
- `year` (string): Filter by year
- `company` (string): Filter by company
- `country` (string): Filter by country
- `director` (string): Filter by director
- `language` (string): Filter by language
- `primaryType` (string): Filter by primary type
- `network` (string): Filter by network
- `remote_id` (string): Filter by remote ID (IMDB, etc.)
- `offset` (integer): Pagination offset
- `limit` (integer): Results per page (default: 20)

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "objectID": "series-12345",
      "id": "12345",
      "type": "series",
      "name": "Breaking Bad",
      "year": "2008",
      "image_url": "https://...",
      "thumbnail": "https://...",
      "status": "Ended",
      "overview": "...",
      "primary_language": "eng",
      "country": "usa"
    }
  ],
  "links": {
    "prev": null,
    "self": "...",
    "next": "...",
    "total_items": 150,
    "page_size": 20
  }
}
```

#### Search by Remote ID
**Endpoint:** `GET /search/remoteid/{remoteId}`

Find entities by external IDs (IMDB, TMDB, EIDR, etc.)

**Example:** `GET /search/remoteid/tt0903747` (Breaking Bad IMDB ID)

---

### Series Endpoints

#### Get All Series
**Endpoint:** `GET /series`

**Query Parameters:**
- `page` (integer): Page number for pagination

**Response:** Paginated list of series base records

#### Get Series by ID (Base)
**Endpoint:** `GET /series/{id}`

**Parameters:**
- `id` (integer): Series ID

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 81189,
    "name": "Breaking Bad",
    "slug": "breaking-bad",
    "image": "https://artworks.thetvdb.com/banners/posters/81189-1.jpg",
    "firstAired": "2008-01-20",
    "lastAired": "2013-09-29",
    "nextAired": "",
    "score": 5678.45,
    "status": {
      "id": 2,
      "name": "Ended",
      "recordType": "series",
      "keepUpdated": false
    },
    "originalCountry": "usa",
    "originalLanguage": "eng",
    "defaultSeasonType": 1,
    "isOrderRandomized": false,
    "averageRuntime": 47,
    "country": "usa",
    "year": "2008",
    "aliases": [
      {
        "language": "eng",
        "name": "BB"
      }
    ],
    "nameTranslations": ["eng", "spa", "fra", "deu"],
    "overviewTranslations": ["eng", "spa", "fra"]
  }
}
```

#### Get Series by ID (Extended)
**Endpoint:** `GET /series/{id}/extended`

**Query Parameters:**
- `meta` (string): Include additional metadata (translations, episodes)
- `short` (boolean): Return shortened response

**Extended Response Includes:**
- Full artwork array with all images
- Characters with actor information
- Content ratings by country
- Genres
- Companies (production, network, studio, distributor)
- Remote IDs (IMDB, TMDB, etc.)
- Translations
- Seasons
- Tags
- Lists
- Inspiration sources

#### Get Series Episodes by Season Type
**Endpoint:** `GET /series/{id}/episodes/{season-type}`

**Parameters:**
- `id` (integer): Series ID
- `season-type` (string): Season type slug (default, dvd, absolute, alternate, regional, altdvd, etc.)

**Query Parameters:**
- `page` (integer): Page number
- `season` (integer): Filter by season number
- `episodeNumber` (integer): Filter by episode number
- `airDate` (string): Filter by air date

**Season Types:**
- `default` / `official` - Aired order (most common)
- `dvd` - DVD order
- `absolute` - Absolute numbering (common for anime)
- `alternate` - Alternative order
- `regional` - Regional variations
- `altdvd` - Alternative DVD order

#### Get Series Artwork
**Endpoint:** `GET /series/{id}/artworks`

**Query Parameters:**
- `lang` (string): Language code (e.g., "eng", "spa")
- `type` (integer): Artwork type ID (from `/artwork/types`)

**Artwork Types:**
1. Banner
2. Poster
3. Background/Fanart
4. Icon
5. Season Poster
6. Season Banner
7. Series Background
8. And more...

#### Filter Series
**Endpoint:** `GET /series/filter`

**Query Parameters:**
- `country` (string): Three-letter country code
- `lang` (string): Language code
- `company` (integer): Company ID
- `contentRating` (integer): Content rating ID
- `genre` (integer): Genre ID (1-36)
- `sort` (string): Sort field (score, firstAired, lastAired, name)
- `sortType` (string): Sort direction (asc, desc)
- `status` (integer): Status ID (1=Continuing, 2=Ended, 3=Upcoming)
- `year` (integer): Release year

**Example:** `GET /series/filter?country=usa&genre=10&status=2&sort=score&sortType=desc`

#### Get Series Translations
**Endpoint:** `GET /series/{id}/translations/{language}`

**Parameters:**
- `id` (integer): Series ID
- `language` (string): Language code (eng, spa, fra, etc.)

**Response:**
```json
{
  "status": "success",
  "data": {
    "name": "Breaking Bad",
    "overview": "Un profesor de química de...",
    "language": "spa",
    "aliases": ["Breaking Bad: La Serie"]
  }
}
```

---

### Episode Endpoints

#### Get All Episodes
**Endpoint:** `GET /episodes`

**Query Parameters:**
- `page` (integer): Page number

#### Get Episode by ID (Base)
**Endpoint:** `GET /episodes/{id}`

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 349232,
    "seriesId": 81189,
    "name": "Pilot",
    "aired": "2008-01-20",
    "runtime": 58,
    "nameTranslations": ["eng", "spa", "fra"],
    "overview": "When an unassuming high school chemistry teacher...",
    "overviewTranslations": ["eng", "spa"],
    "image": "https://artworks.thetvdb.com/banners/episodes/81189/349232.jpg",
    "imageType": 12,
    "isMovie": 0,
    "seasons": null,
    "number": 1,
    "seasonNumber": 1,
    "lastUpdated": "2023-08-15 12:30:45",
    "finaleType": null,
    "year": "2008"
  }
}
```

#### Get Episode by ID (Extended)
**Endpoint:** `GET /episodes/{id}/extended`

**Query Parameters:**
- `meta` (string): Additional metadata to include

**Extended Response Includes:**
- Awards
- Characters with episode-specific roles
- Content ratings
- Companies
- Networks
- Nominations
- Production codes
- Remote IDs
- Taglines
- Trailers
- Translations

#### Get Episode Translations
**Endpoint:** `GET /episodes/{id}/translations/{language}`

---

### Movie Endpoints

#### Get All Movies
**Endpoint:** `GET /movies`

**Query Parameters:**
- `page` (integer): Page number

#### Get Movie by ID (Base)
**Endpoint:** `GET /movies/{id}`

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 12345,
    "name": "The Matrix",
    "slug": "the-matrix",
    "image": "https://...",
    "nameTranslations": ["eng", "spa", "fra"],
    "overviewTranslations": ["eng", "spa"],
    "score": 8234.56,
    "runtime": 136,
    "status": {
      "id": 1,
      "name": "Released"
    },
    "lastUpdated": "2023-09-01 10:15:30",
    "year": "1999"
  }
}
```

#### Get Movie by ID (Extended)
**Endpoint:** `GET /movies/{id}/extended`

**Extended Response Includes:**
- Aliases
- Artworks
- Audio languages
- Awards
- Box office data (worldwide and US)
- Budget
- Characters and cast
- Companies (production, distribution, special effects, etc.)
- Content ratings
- First release date
- Genres
- Inspiration sources
- Lists
- Production countries
- Release dates by country
- Remote IDs
- Runtime
- Studios
- Subtitle languages
- Taglines
- Trailers
- Translations

#### Filter Movies
**Endpoint:** `GET /movies/filter`

**Query Parameters:**
- `country` (string): Three-letter country code
- `lang` (string): Language code
- `company` (integer): Company ID
- `contentRating` (integer): Content rating ID
- `genre` (integer): Genre ID
- `sort` (string): Sort field (score, firstReleased, name)
- `sortType` (string): Sort direction (asc, desc)
- `status` (integer): Status ID
- `year` (integer): Release year

#### Get Movie Translations
**Endpoint:** `GET /movies/{id}/translations/{language}`

#### Get Movie Statuses
**Endpoint:** `GET /movies/statuses`

Returns list of all possible movie statuses.

---

### People Endpoints

#### Get All People
**Endpoint:** `GET /people`

**Query Parameters:**
- `page` (integer): Page number

#### Get Person by ID (Base)
**Endpoint:** `GET /people/{id}`

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 12345,
    "name": "Bryan Cranston",
    "image": "https://...",
    "score": 9876.54,
    "nameTranslations": ["eng"],
    "overviewTranslations": ["eng", "spa"]
  }
}
```

#### Get Person by ID (Extended)
**Endpoint:** `GET /people/{id}/extended`

**Extended Response Includes:**
- Aliases
- Awards
- Biographies in multiple languages
- Birth date and place
- Characters portrayed
- Death date (if applicable)
- Gender
- Known for (notable works)
- Nationalities
- People type (actor, director, writer, etc.)
- Race
- Remote IDs
- Tagline
- Translations

#### Get Person Translations
**Endpoint:** `GET /people/{id}/translations/{language}`

---

### Character Endpoints

#### Get Character by ID
**Endpoint:** `GET /characters/{id}`

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 67890,
    "name": "Walter White",
    "peopleId": 12345,
    "seriesId": 81189,
    "series": {
      "id": 81189,
      "name": "Breaking Bad"
    },
    "movie": null,
    "movieId": null,
    "episodeId": null,
    "type": 3,
    "image": "https://...",
    "sort": 1,
    "isFeatured": true,
    "url": "https://...",
    "nameTranslations": ["eng", "spa"],
    "overviewTranslations": ["eng"],
    "aliases": ["Heisenberg"],
    "peopleType": "Actor",
    "personName": "Bryan Cranston",
    "tagOptions": [],
    "personImgURL": "https://..."
  }
}
```

---

### Artwork Endpoints

#### Get Artwork by ID (Base)
**Endpoint:** `GET /artwork/{id}`

#### Get Artwork by ID (Extended)
**Endpoint:** `GET /artwork/{id}/extended`

**Extended Response Includes:**
- Episode, movie, series, season, network, or people ID
- Height and width
- Image URL and thumbnail
- Language
- Score
- Status
- Tag options
- Type
- Whether it includes text overlay
- Update timestamp

#### Get Artwork Types
**Endpoint:** `GET /artwork/types`

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "id": 1,
      "name": "Banner",
      "recordType": "series",
      "slug": "banner",
      "imageFormat": "png",
      "width": 758,
      "height": 140,
      "thumbWidth": 300,
      "thumbHeight": 55
    },
    {
      "id": 2,
      "name": "Poster",
      "recordType": "series",
      "slug": "poster",
      "imageFormat": "jpg",
      "width": 680,
      "height": 1000,
      "thumbWidth": 204,
      "thumbHeight": 300
    }
    // ... more types
  ]
}
```

#### Get Artwork Statuses
**Endpoint:** `GET /artwork/statuses`

---

### Award Endpoints

#### Get All Awards
**Endpoint:** `GET /awards`

#### Get Award by ID
**Endpoint:** `GET /awards/{id}`

#### Get Award Extended
**Endpoint:** `GET /awards/{id}/extended`

#### Get Award Category by ID
**Endpoint:** `GET /awards/categories/{id}`

**Extended Response Includes:**
- Award name and ID
- Category name
- Nominees with details
- Winners
- Associated series/movies

---

### Company Endpoints

#### Get All Companies
**Endpoint:** `GET /companies`

#### Get Company by ID
**Endpoint:** `GET /companies/{id}`

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 1,
    "name": "AMC",
    "slug": "amc",
    "nameTranslations": ["eng"],
    "overviewTranslations": [],
    "aliases": [],
    "country": "usa",
    "primaryCompanyType": 1,
    "activeDate": "1984-01-01",
    "inactiveDate": null,
    "companyType": {
      "companyTypeId": 1,
      "companyTypeName": "Network"
    },
    "parentCompany": {
      "id": null,
      "name": null,
      "relation": null
    },
    "tagOptions": []
  }
}
```

#### Get Company Types
**Endpoint:** `GET /companies/types`

**Company Types:**
1. Network
2. Production Company
3. Studio
4. Distributor
5. Special Effects
6. And more...

---

### List Endpoints

#### Get All Lists
**Endpoint:** `GET /lists`

#### Get List by ID
**Endpoint:** `GET /lists/{id}`

#### Get List by Slug
**Endpoint:** `GET /lists/slug/{slug}`

#### Get List Extended
**Endpoint:** `GET /lists/{id}/extended`

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 123,
    "name": "Top Sci-Fi Series",
    "overview": "The best science fiction series of all time",
    "url": "https://...",
    "isOfficial": true,
    "nameTranslations": ["eng"],
    "overviewTranslations": ["eng"],
    "aliases": [],
    "score": 456.78,
    "entities": [
      {
        "seriesId": 81189,
        "name": "Breaking Bad",
        "image": "https://...",
        "order": 1
      }
    ]
  }
}
```

#### Get List Translations
**Endpoint:** `GET /lists/{id}/translations/{language}`

---

### Season Endpoints

#### Get All Seasons
**Endpoint:** `GET /seasons`

#### Get Season by ID
**Endpoint:** `GET /seasons/{id}`

#### Get Season Extended
**Endpoint:** `GET /seasons/{id}/extended`

#### Get Season Types
**Endpoint:** `GET /seasons/types`

**Season Types:**
1. Aired Order (default)
2. DVD Order
3. Absolute Order
4. Alternate Order
5. Regional Order
6. Alternate DVD Order

---

### Reference Data Endpoints

#### Get All Genres
**Endpoint:** `GET /genres`

**Response:**
```json
{
  "status": "success",
  "data": [
    {"id": 1, "name": "Action", "slug": "action"},
    {"id": 2, "name": "Adventure", "slug": "adventure"},
    {"id": 3, "name": "Animation", "slug": "animation"},
    {"id": 4, "name": "Comedy", "slug": "comedy"},
    {"id": 5, "name": "Crime", "slug": "crime"},
    {"id": 6, "name": "Documentary", "slug": "documentary"},
    {"id": 7, "name": "Drama", "slug": "drama"},
    {"id": 8, "name": "Family", "slug": "family"},
    {"id": 9, "name": "Fantasy", "slug": "fantasy"},
    {"id": 10, "name": "Horror", "slug": "horror"}
    // ... more genres
  ]
}
```

#### Get Genre by ID
**Endpoint:** `GET /genres/{id}`

#### Get All Languages
**Endpoint:** `GET /languages`

**Response:**
```json
{
  "status": "success",
  "data": [
    {"id": "eng", "name": "English", "nativeName": "English"},
    {"id": "spa", "name": "Spanish", "nativeName": "Español"},
    {"id": "fra", "name": "French", "nativeName": "Français"}
    // ... more languages
  ]
}
```

#### Get All Countries
**Endpoint:** `GET /countries`

#### Get All Content Ratings
**Endpoint:** `GET /content/ratings`

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "id": 1,
      "name": "TV-MA",
      "country": "usa",
      "description": "Mature Audience Only",
      "contentType": "series",
      "order": 7,
      "fullName": "TV Mature Audience"
    }
    // ... more ratings
  ]
}
```

#### Get All Genders
**Endpoint:** `GET /genders`

#### Get All Inspiration Types
**Endpoint:** `GET /inspiration/types`

#### Get All Source Types
**Endpoint:** `GET /sources/types`

#### Get All Entity Types
**Endpoint:** `GET /entities`

---

### Update Tracking Endpoints

#### Get Updates
**Endpoint:** `GET /updates`

**Query Parameters:**
- `since` (integer): Unix timestamp to get updates since
- `type` (string): Entity type (series, movies, episodes, people, artworks, lists, seasons, companies, awards)
- `action` (string): Update action (update, delete)
- `page` (integer): Page number

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "recordId": 81189,
      "recordType": "series",
      "methodInt": 2,
      "method": "update",
      "extraInfo": null,
      "userId": 12345,
      "timeStamp": 1699876543,
      "mergeToId": null,
      "mergeToType": null
    },
    {
      "recordId": 67890,
      "recordType": "series",
      "methodInt": 3,
      "method": "delete",
      "extraInfo": null,
      "userId": 12345,
      "timeStamp": 1699876500,
      "mergeToId": 81189,
      "mergeToType": "series"
    }
  ],
  "links": {
    "prev": null,
    "self": "...",
    "next": "...",
    "total_items": 1543,
    "page_size": 100
  }
}
```

**Update Methods:**
- `create` (1): New record created
- `update` (2): Record updated
- `delete` (3): Record deleted (check mergeToId for merge target)

**Usage Pattern:**
1. Store last sync timestamp
2. Call `/updates?since={timestamp}&type=series`
3. Process updates (add, update, delete/merge)
4. Update sync timestamp
5. Repeat periodically

**Merge Handling:**
When a record is deleted with a `mergeToId`, move data from the deleted record to the merge target.

---

### User Endpoints

#### Get Current User
**Endpoint:** `GET /user`

**Response:**
```json
{
  "status": "success",
  "data": {
    "id": 12345,
    "name": "username",
    "language": "eng",
    "type": "subscriber"
  }
}
```

#### Get User by ID
**Endpoint:** `GET /user/{id}`

#### Get User Favorites
**Endpoint:** `GET /user/favorites`

**Response:**
```json
{
  "status": "success",
  "data": {
    "series": [81189, 75760, 79349],
    "movies": [12345, 67890],
    "episodes": [349232],
    "people": [12345],
    "lists": [123]
  }
}
```

#### Add User Favorites
**Endpoint:** `POST /user/favorites`

**Request Body:**
```json
{
  "series": 81189
}
```

Or for other entity types:
```json
{
  "movie": 12345
}
```

---

## Data Models

### SeriesBaseRecord

```typescript
interface SeriesBaseRecord {
  id: number;
  name: string;
  slug: string;
  image: string;
  nameTranslations: string[];        // Language codes
  overviewTranslations: string[];    // Language codes
  aliases: Alias[];
  firstAired: string;                // ISO date
  lastAired: string;                 // ISO date
  nextAired: string;                 // ISO date
  score: number;                     // Popularity score
  status: Status;
  originalCountry: string;           // 3-letter code
  originalLanguage: string;          // 3-4 character code
  defaultSeasonType: number;         // Season type ID
  isOrderRandomized: boolean;
  lastUpdated: string;               // ISO timestamp
  averageRuntime: number | null;     // Minutes
  episodes: EpisodeBaseRecord[];
  overview: string;
  year: string;
  country: string;
}

interface Alias {
  language: string;                  // 3-4 character code
  name: string;                      // Max 100 chars
}

interface Status {
  id: number;                        // 1=Continuing, 2=Ended, 3=Upcoming
  name: string;
  recordType: string;
  keepUpdated: boolean;
}
```

### SeriesExtendedRecord

Extends `SeriesBaseRecord` with:

```typescript
interface SeriesExtendedRecord extends SeriesBaseRecord {
  abbreviation: string;
  airsDays: SeriesAirsDays;
  airsTime: string;
  artworks: ArtworkExtendedRecord[];
  characters: Character[];
  contentRatings: ContentRating[];
  firstRelease: Release;
  genres: GenreBaseRecord[];
  companies: Companies;
  originalNetwork: Company;
  latestNetwork: Company;
  lists: ListBaseRecord[];
  remoteIds: RemoteID[];
  seasons: SeasonBaseRecord[];
  tags: Tag[];
  trailers: Trailer[];
  translations: Translation[];
  nextEpisode: EpisodeBaseRecord | null;
  lastEpisode: EpisodeBaseRecord | null;
}
```

### EpisodeBaseRecord

```typescript
interface EpisodeBaseRecord {
  id: number;
  seriesId: number;
  name: string;
  aired: string;                     // ISO date
  runtime: number | null;            // Minutes
  nameTranslations: string[];
  overview: string;
  overviewTranslations: string[];
  image: string;
  imageType: number | null;
  isMovie: number;                   // 0 or 1
  seasons: SeasonBaseRecord[];
  number: number;                    // Episode number
  seasonNumber: number;
  lastUpdated: string;
  finaleType: string | null;         // "season", "midseason", "series"
  year: string;
  absoluteNumber: number;
  airsAfterSeason: number;
  airsBeforeEpisode: number;
  airsBeforeSeason: number;
  linkedMovie: number;
  seasonName: string;
}
```

### EpisodeExtendedRecord

Extends `EpisodeBaseRecord` with:

```typescript
interface EpisodeExtendedRecord extends EpisodeBaseRecord {
  awards: AwardBaseRecord[];
  characters: Character[];
  companies: Company[];
  contentRatings: ContentRating[];
  networks: Company[];
  nominations: Nomination[];
  productionCode: string;
  remoteIds: RemoteID[];
  tagOptions: TagOption[];
  trailers: Trailer[];
  translations: Translation[];
}
```

### MovieBaseRecord

```typescript
interface MovieBaseRecord {
  id: number;
  name: string;
  slug: string;
  image: string;
  nameTranslations: string[];
  overviewTranslations: string[];
  aliases: Alias[];
  score: number;
  runtime: number | null;            // Minutes
  status: Status;
  lastUpdated: string;
  year: string;
}
```

### MovieExtendedRecord

Extends `MovieBaseRecord` with:

```typescript
interface MovieExtendedRecord extends MovieBaseRecord {
  artworks: ArtworkBaseRecord[];
  audioLanguages: string[];
  awards: AwardBaseRecord[];
  boxOffice: string;
  boxOfficeUS: string;
  budget: string;
  characters: Character[];
  companies: Companies;
  contentRatings: ContentRating[];
  first_release: Release;
  genres: GenreBaseRecord[];
  inspirations: Inspiration[];
  lists: ListBaseRecord[];
  originalCountry: string;
  originalLanguage: string;
  production_countries: ProductionCountry[];
  releases: Release[];
  remoteIds: RemoteID[];
  runtime: number;
  studios: Studio[];
  subtitleLanguages: string[];
  tagOptions: TagOption[];
  trailers: Trailer[];
  translations: Translation[];
}
```

### ArtworkBaseRecord

```typescript
interface ArtworkBaseRecord {
  id: number;
  image: string;
  thumbnail: string;
  language: string;
  type: number;                      // Artwork type ID
  score: number;
  width: number;
  height: number;
  includesText: boolean;
}
```

### ArtworkExtendedRecord

Extends `ArtworkBaseRecord` with:

```typescript
interface ArtworkExtendedRecord extends ArtworkBaseRecord {
  episodeId: number;
  movieId: number;
  networkId: number;
  peopleId: number;
  seasonId: number;
  seriesId: number;
  seriesPeopleId: number;
  status: ArtworkStatus;
  tagOptions: TagOption[];
  thumbnailWidth: number;
  thumbnailHeight: number;
  updatedAt: number;
}
```

### Character

```typescript
interface Character {
  id: number;
  name: string;
  peopleId: number;
  seriesId: number | null;
  series: SeriesBaseRecord | null;
  movie: MovieBaseRecord | null;
  movieId: number | null;
  episodeId: number | null;
  type: number;
  image: string;
  sort: number;
  isFeatured: boolean;
  url: string;
  nameTranslations: string[];
  overviewTranslations: string[];
  aliases: Alias[];
  peopleType: string;
  personName: string;
  tagOptions: TagOption[];
  personImgURL: string;
}
```

### RemoteID

```typescript
interface RemoteID {
  id: string;
  type: number;                      // Remote ID type
  sourceName: string;                // "IMDB", "TheMovieDB.com", etc.
}
```

### Translation

```typescript
interface Translation {
  name: string;
  overview: string;
  language: string;
  isPrimary: boolean;
  aliases: string[];
  tagline: string;
}
```

---

## Rate Limits & Quotas

### Official Rate Limits

TheTVDB API v4 **does not explicitly publish hard rate limits** in their documentation. However, best practices suggest:

1. **Respect the service**: Don't make excessive concurrent requests
2. **Implement retry logic**: Handle 429 (Too Many Requests) responses with exponential backoff
3. **Cache aggressively**: Use the caching strategies outlined below
4. **Monitor usage**: Track your API calls and implement client-side throttling

### Recommended Client-Side Limits

Based on community best practices:

- **Max concurrent requests**: 5-10 simultaneous connections
- **Request interval**: 100-200ms between requests
- **Burst limit**: No more than 50 requests in 10 seconds
- **Daily monitoring**: Track total requests per day

### HTTP Status Codes for Rate Limiting

- `429 Too Many Requests`: Slow down your requests
- `503 Service Unavailable`: Temporary server issue, retry with backoff

### Subscription Considerations

- **Free tier**: May have stricter limits
- **Paid tiers**: Generally more lenient limits
- **User subscriptions**: $12/year for personal projects

---

## Best Practices

### 1. Data Retrieval Strategies

TheTVDB recommends three main approaches:

#### Strategy A: Full Database Copy with Update Monitoring

**Best for**: Large-scale applications, offline access needs

**Implementation:**
1. Download complete dataset for your entity types
2. Store locally in your database
3. Monitor `/updates` endpoint for changes (polling every 15-60 minutes)
4. Apply incremental updates (create, update, delete/merge)

**Advantages:**
- Fast local queries
- No repeated API calls for same data
- Full control over data structure

**Code Pattern:**
```javascript
// Initial sync
await downloadAllSeries();
await downloadAllEpisodes();

// Store last sync timestamp
let lastSync = Date.now();

// Periodic update check (every 30 minutes)
setInterval(async () => {
  const updates = await fetch(`/updates?since=${Math.floor(lastSync/1000)}&type=series`);

  for (const update of updates.data) {
    if (update.method === 'create' || update.method === 'update') {
      await syncRecord(update.recordId, update.recordType);
    } else if (update.method === 'delete') {
      if (update.mergeToId) {
        await mergeRecord(update.recordId, update.mergeToId, update.recordType);
      } else {
        await deleteRecord(update.recordId, update.recordType);
      }
    }
  }

  lastSync = Date.now();
}, 30 * 60 * 1000);
```

#### Strategy B: Caching Proxy

**Best for**: Medium-scale applications, reducing API calls

**Implementation:**
1. Implement intermediary cache (Redis, Memcached, Squid, Varnish)
2. Configure different TTLs per endpoint type:
   - Reference data (genres, languages): 7+ days
   - Series/movie base records: 24 hours
   - Episode data: 6-12 hours
   - Updates: 15-30 minutes
3. Cache-Control headers guide refresh timing

**Cache TTL Recommendations:**
```javascript
const CACHE_TTLS = {
  genres: 7 * 24 * 60 * 60,           // 7 days
  languages: 7 * 24 * 60 * 60,        // 7 days
  artworkTypes: 7 * 24 * 60 * 60,     // 7 days
  series: 24 * 60 * 60,               // 24 hours
  movies: 24 * 60 * 60,               // 24 hours
  episodes: 6 * 60 * 60,              // 6 hours
  people: 24 * 60 * 60,               // 24 hours
  updates: 15 * 60,                   // 15 minutes
  search: 60 * 60,                    // 1 hour
};
```

#### Strategy C: Direct API Access

**Best for**: Small-scale applications, prototypes, user-facing features

**Implementation:**
- Allow end-users to use their own API subscriptions
- Proxy requests through your backend with user's token
- Implement basic caching for common queries

### 2. Understanding the Score Field

The `score` field appears across almost all entities (series, movies, episodes, people, etc.).

**Important Notes:**
- **Relative, not absolute**: Score indicates relative popularity within entity type
- **Not comparable across types**: Series scores can't be compared to movie scores
- **Use for sorting only**: Perfect for "most popular" queries
- **Dynamic**: Scores change over time based on community engagement

**Usage:**
```javascript
// ✅ Good: Sort series by popularity
GET /series/filter?sort=score&sortType=desc

// ❌ Bad: Compare series score to movie score
if (series.score > movie.score) // Meaningless comparison
```

### 3. Artwork Best Practices

**Image Selection:**
- Base records include **best popular artwork** for each type
- Extended records include **all available artwork**
- **Rarely need full artwork sets** unless building an image gallery
- Filter by language for localized artwork
- Check `includesText` to avoid text overlays

**Episode Images:**
- May be 4:3 or 16:9 aspect ratio
- Depends on original broadcast format
- Check dimensions before assuming aspect ratio

**Artwork Types to Know:**
- **Type 2**: Poster (680x1000)
- **Type 3**: Background/Fanart (1920x1080)
- **Type 1**: Banner (758x140)
- **Type 14**: Series Banner
- **Type 7**: Season Poster

### 4. Season Type Handling

Series support **multiple season orderings**:

**Common Types:**
- `default` / `official`: Aired order (most common)
- `dvd`: DVD release order
- `absolute`: Continuous numbering (anime)
- `alternate`: Alternative storyline order
- `regional`: Regional broadcast variations

**Implementation:**
```javascript
// Get default season order
const episodes = await fetch(`/series/${seriesId}/episodes/default`);

// Check if series has alternative orderings
const seasonTypes = await fetch(`/seasons/types`);
const seriesSeasons = await fetch(`/series/${seriesId}/extended`);

// If DVD order exists, fetch it
if (seriesSeasons.data.seasons.some(s => s.type.id === 2)) {
  const dvdEpisodes = await fetch(`/series/${seriesId}/episodes/dvd`);
}
```

### 5. Search as Primary Discovery

**Search endpoint is the main discovery mechanism**

**Best Practices:**
- Use `/search` for user queries
- Filter by type for targeted results
- Use remote IDs when available (IMDB, TMDB)
- Implement autocomplete with debouncing
- Cache popular search results

**Search Tips:**
```javascript
// General search
GET /search?query=breaking%20bad

// Filtered by type
GET /search?query=matrix&type=movie

// By IMDB ID (most reliable)
GET /search/remoteid/tt0903747

// With filters
GET /search?query=sci-fi&year=2015&country=usa
```

### 6. Translation Handling

**Multiple ways to get translations:**
1. Check `nameTranslations` / `overviewTranslations` arrays for available languages
2. Fetch specific translation via `/translations/{language}` endpoint
3. Extended records include all translations in single call

**Implementation:**
```javascript
// Check available translations
const series = await fetch(`/series/${id}`);
const availableLangs = series.data.nameTranslations; // ["eng", "spa", "fra"]

// Fetch Spanish translation
if (availableLangs.includes('spa')) {
  const spanish = await fetch(`/series/${id}/translations/spa`);
  console.log(spanish.data.name); // "Breaking Bad" (or localized)
  console.log(spanish.data.overview); // Spanish overview
}
```

### 7. Error Handling & Retries

**Implement robust error handling:**

```javascript
async function fetchWithRetry(url, options = {}, maxRetries = 3) {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
          ...options.headers
        }
      });

      if (response.status === 401) {
        // Token expired, refresh it
        await refreshToken();
        continue;
      }

      if (response.status === 429) {
        // Rate limited, exponential backoff
        const delay = Math.pow(2, i) * 1000;
        await new Promise(resolve => setTimeout(resolve, delay));
        continue;
      }

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();

    } catch (error) {
      if (i === maxRetries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
    }
  }
}
```

### 8. Token Management

**JWT tokens are valid for 1 month:**

```javascript
class TVDBClient {
  constructor(apiKey, pin = null) {
    this.apiKey = apiKey;
    this.pin = pin;
    this.token = null;
    this.tokenExpiry = null;
  }

  async ensureToken() {
    if (this.token && this.tokenExpiry > Date.now()) {
      return this.token;
    }

    const response = await fetch('https://api4.thetvdb.com/v4/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        apikey: this.apiKey,
        ...(this.pin && { pin: this.pin })
      })
    });

    const data = await response.json();
    this.token = data.data.token;
    this.tokenExpiry = Date.now() + (30 * 24 * 60 * 60 * 1000); // 30 days

    return this.token;
  }

  async request(endpoint, options = {}) {
    await this.ensureToken();
    return fetchWithRetry(`https://api4.thetvdb.com/v4${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${this.token}`,
        ...options.headers
      }
    });
  }
}
```

### 9. Pagination Handling

**Most list endpoints support pagination:**

```javascript
async function fetchAllPages(endpoint) {
  const results = [];
  let page = 0;
  let hasMore = true;

  while (hasMore) {
    const response = await client.request(`${endpoint}?page=${page}`);
    results.push(...response.data);

    hasMore = response.links && response.links.next;
    page++;

    // Optional: Add delay between pages
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  return results;
}
```

### 10. Reference Data Caching

**Cache reference data long-term:**

```javascript
class ReferenceDataCache {
  constructor(client) {
    this.client = client;
    this.cache = new Map();
  }

  async getGenres() {
    if (!this.cache.has('genres')) {
      const genres = await this.client.request('/genres');
      this.cache.set('genres', {
        data: genres.data,
        expiry: Date.now() + (7 * 24 * 60 * 60 * 1000) // 7 days
      });
    }

    const cached = this.cache.get('genres');
    if (cached.expiry < Date.now()) {
      this.cache.delete('genres');
      return this.getGenres();
    }

    return cached.data;
  }

  async getLanguages() { /* Similar pattern */ }
  async getArtworkTypes() { /* Similar pattern */ }
  async getCountries() { /* Similar pattern */ }
}
```

---

## Code Examples

### Complete Node.js Client

```javascript
const https = require('https');

class TVDBClient {
  constructor(apiKey, pin = null) {
    this.apiKey = apiKey;
    this.pin = pin;
    this.token = null;
    this.tokenExpiry = null;
    this.baseURL = 'https://api4.thetvdb.com/v4';
  }

  // Authentication
  async login() {
    const body = JSON.stringify({
      apikey: this.apiKey,
      ...(this.pin && { pin: this.pin })
    });

    const response = await this.request('/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(body)
      },
      body
    });

    this.token = response.data.token;
    this.tokenExpiry = Date.now() + (30 * 24 * 60 * 60 * 1000); // 30 days

    return this.token;
  }

  async ensureToken() {
    if (!this.token || this.tokenExpiry <= Date.now()) {
      await this.login();
    }
    return this.token;
  }

  // Generic request method
  async request(path, options = {}) {
    return new Promise(async (resolve, reject) => {
      const url = new URL(path, this.baseURL);

      const headers = {
        ...options.headers
      };

      // Add auth token for protected endpoints
      if (path !== '/login') {
        await this.ensureToken();
        headers['Authorization'] = `Bearer ${this.token}`;
      }

      const reqOptions = {
        method: options.method || 'GET',
        headers
      };

      const req = https.request(url, reqOptions, (res) => {
        let data = '';

        res.on('data', (chunk) => {
          data += chunk;
        });

        res.on('end', () => {
          try {
            const parsed = JSON.parse(data);

            if (res.statusCode >= 200 && res.statusCode < 300) {
              resolve(parsed);
            } else {
              reject(new Error(`HTTP ${res.statusCode}: ${parsed.message || data}`));
            }
          } catch (error) {
            reject(new Error(`Failed to parse response: ${error.message}`));
          }
        });
      });

      req.on('error', reject);

      if (options.body) {
        req.write(options.body);
      }

      req.end();
    });
  }

  // Series methods
  async searchSeries(query) {
    return this.request(`/search?query=${encodeURIComponent(query)}&type=series`);
  }

  async getSeries(id, extended = false) {
    const path = extended ? `/series/${id}/extended` : `/series/${id}`;
    return this.request(path);
  }

  async getSeriesEpisodes(id, seasonType = 'default', page = 0) {
    return this.request(`/series/${id}/episodes/${seasonType}?page=${page}`);
  }

  async getSeriesArtwork(id, lang = null, type = null) {
    let path = `/series/${id}/artworks`;
    const params = [];
    if (lang) params.push(`lang=${lang}`);
    if (type) params.push(`type=${type}`);
    if (params.length) path += `?${params.join('&')}`;
    return this.request(path);
  }

  // Episode methods
  async getEpisode(id, extended = false) {
    const path = extended ? `/episodes/${id}/extended` : `/episodes/${id}`;
    return this.request(path);
  }

  // Movie methods
  async searchMovies(query) {
    return this.request(`/search?query=${encodeURIComponent(query)}&type=movie`);
  }

  async getMovie(id, extended = false) {
    const path = extended ? `/movies/${id}/extended` : `/movies/${id}`;
    return this.request(path);
  }

  // People methods
  async searchPeople(query) {
    return this.request(`/search?query=${encodeURIComponent(query)}&type=people`);
  }

  async getPerson(id, extended = false) {
    const path = extended ? `/people/${id}/extended` : `/people/${id}`;
    return this.request(path);
  }

  // Reference data methods
  async getGenres() {
    return this.request('/genres');
  }

  async getLanguages() {
    return this.request('/languages');
  }

  async getArtworkTypes() {
    return this.request('/artwork/types');
  }

  // Update tracking
  async getUpdates(since, type = null, page = 0) {
    let path = `/updates?since=${since}&page=${page}`;
    if (type) path += `&type=${type}`;
    return this.request(path);
  }

  // User methods
  async getCurrentUser() {
    return this.request('/user');
  }

  async getFavorites() {
    return this.request('/user/favorites');
  }

  async addFavorite(type, id) {
    return this.request('/user/favorites', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ [type]: id })
    });
  }
}

// Usage example
async function main() {
  const client = new TVDBClient('your-api-key-here');

  try {
    // Search for a series
    const searchResults = await client.searchSeries('Breaking Bad');
    console.log('Search results:', searchResults);

    // Get series details
    const seriesId = searchResults.data[0].id;
    const series = await client.getSeries(seriesId, true);
    console.log('Series details:', series);

    // Get episodes
    const episodes = await client.getSeriesEpisodes(seriesId);
    console.log('Episodes:', episodes);

    // Get artwork
    const artwork = await client.getSeriesArtwork(seriesId, 'eng');
    console.log('Artwork:', artwork);

  } catch (error) {
    console.error('Error:', error.message);
  }
}

module.exports = TVDBClient;
```

### Python Example

```python
import requests
from datetime import datetime, timedelta
from typing import Optional, Dict, Any

class TVDBClient:
    def __init__(self, api_key: str, pin: Optional[str] = None):
        self.api_key = api_key
        self.pin = pin
        self.token = None
        self.token_expiry = None
        self.base_url = 'https://api4.thetvdb.com/v4'
        self.session = requests.Session()

    def login(self) -> str:
        """Authenticate and get JWT token"""
        body = {'apikey': self.api_key}
        if self.pin:
            body['pin'] = self.pin

        response = self.session.post(f'{self.base_url}/login', json=body)
        response.raise_for_status()

        data = response.json()
        self.token = data['data']['token']
        self.token_expiry = datetime.now() + timedelta(days=30)

        # Set auth header for future requests
        self.session.headers.update({
            'Authorization': f'Bearer {self.token}'
        })

        return self.token

    def ensure_token(self):
        """Ensure we have a valid token"""
        if not self.token or datetime.now() >= self.token_expiry:
            self.login()

    def request(self, path: str, method: str = 'GET', **kwargs) -> Dict[Any, Any]:
        """Make authenticated API request"""
        if path != '/login':
            self.ensure_token()

        url = f'{self.base_url}{path}'
        response = self.session.request(method, url, **kwargs)
        response.raise_for_status()

        return response.json()

    # Series methods
    def search_series(self, query: str) -> Dict:
        return self.request(f'/search?query={query}&type=series')

    def get_series(self, series_id: int, extended: bool = False) -> Dict:
        path = f'/series/{series_id}/extended' if extended else f'/series/{series_id}'
        return self.request(path)

    def get_series_episodes(self, series_id: int, season_type: str = 'default', page: int = 0) -> Dict:
        return self.request(f'/series/{series_id}/episodes/{season_type}?page={page}')

    # Episode methods
    def get_episode(self, episode_id: int, extended: bool = False) -> Dict:
        path = f'/episodes/{episode_id}/extended' if extended else f'/episodes/{episode_id}'
        return self.request(path)

    # Movie methods
    def search_movies(self, query: str) -> Dict:
        return self.request(f'/search?query={query}&type=movie')

    def get_movie(self, movie_id: int, extended: bool = False) -> Dict:
        path = f'/movies/{movie_id}/extended' if extended else f'/movies/{movie_id}'
        return self.request(path)

    # People methods
    def search_people(self, query: str) -> Dict:
        return self.request(f'/search?query={query}&type=people')

    def get_person(self, person_id: int, extended: bool = False) -> Dict:
        path = f'/people/{person_id}/extended' if extended else f'/people/{person_id}'
        return self.request(path)

    # Reference data
    def get_genres(self) -> Dict:
        return self.request('/genres')

    def get_languages(self) -> Dict:
        return self.request('/languages')

    # Updates
    def get_updates(self, since: int, entity_type: Optional[str] = None, page: int = 0) -> Dict:
        path = f'/updates?since={since}&page={page}'
        if entity_type:
            path += f'&type={entity_type}'
        return self.request(path)

# Usage
if __name__ == '__main__':
    client = TVDBClient('your-api-key-here')

    # Search for series
    results = client.search_series('Breaking Bad')
    print(f"Found {len(results['data'])} results")

    # Get first series details
    if results['data']:
        series_id = results['data'][0]['id']
        series = client.get_series(series_id, extended=True)
        print(f"Series: {series['data']['name']}")
        print(f"Status: {series['data']['status']['name']}")
        print(f"Genres: {[g['name'] for g in series['data'].get('genres', [])]}")
```

### cURL Examples

```bash
# Login
curl -X POST https://api4.thetvdb.com/v4/login \
  -H "Content-Type: application/json" \
  -d '{"apikey": "your-api-key"}'

# Get token from response and export
export TVDB_TOKEN="your-jwt-token-here"

# Search for series
curl https://api4.thetvdb.com/v4/search?query=breaking%20bad \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Get series details
curl https://api4.thetvdb.com/v4/series/81189 \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Get series extended
curl https://api4.thetvdb.com/v4/series/81189/extended \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Get episodes
curl https://api4.thetvdb.com/v4/series/81189/episodes/default \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Get episode details
curl https://api4.thetvdb.com/v4/episodes/349232 \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Search movies
curl https://api4.thetvdb.com/v4/search?query=matrix&type=movie \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Get movie
curl https://api4.thetvdb.com/v4/movies/12345/extended \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Get genres
curl https://api4.thetvdb.com/v4/genres \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Get updates since timestamp
curl "https://api4.thetvdb.com/v4/updates?since=1699876543&type=series" \
  -H "Authorization: Bearer $TVDB_TOKEN"

# Add favorite
curl -X POST https://api4.thetvdb.com/v4/user/favorites \
  -H "Authorization: Bearer $TVDB_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"series": 81189}'
```

---

## Error Handling

### HTTP Status Codes

| Code | Meaning | Action |
|------|---------|--------|
| 200 | Success | Process response data |
| 400 | Bad Request | Check request parameters and format |
| 401 | Unauthorized | Refresh or obtain new token |
| 404 | Not Found | Entity doesn't exist or has been deleted |
| 429 | Too Many Requests | Implement exponential backoff |
| 500 | Server Error | Retry with backoff, report if persistent |
| 503 | Service Unavailable | Temporary issue, retry later |

### Error Response Format

```json
{
  "status": "failure",
  "message": "Unauthorized",
  "error": "Invalid token"
}
```

### Common Errors & Solutions

#### 401 Unauthorized
**Causes:**
- Expired token (after 1 month)
- Invalid API key
- Missing Authorization header
- Invalid PIN for subscriber accounts

**Solution:**
```javascript
if (response.status === 401) {
  // Clear token and login again
  this.token = null;
  await this.login();
  // Retry original request
}
```

#### 404 Not Found
**Causes:**
- Entity has been deleted
- Entity has been merged to another ID
- Invalid ID

**Solution:**
```javascript
// Check updates endpoint for merge information
const updates = await client.getUpdates(lastWeek, 'series');
const deletion = updates.data.find(u => u.recordId === seriesId && u.method === 'delete');

if (deletion && deletion.mergeToId) {
  // Entity was merged, use new ID
  const newSeries = await client.getSeries(deletion.mergeToId);
}
```

#### 429 Too Many Requests
**Cause:** Exceeded rate limits

**Solution:**
```javascript
async function fetchWithBackoff(url, options, maxRetries = 5) {
  for (let i = 0; i < maxRetries; i++) {
    const response = await fetch(url, options);

    if (response.status === 429) {
      const delay = Math.pow(2, i) * 1000; // Exponential backoff
      await new Promise(resolve => setTimeout(resolve, delay));
      continue;
    }

    return response;
  }

  throw new Error('Max retries exceeded');
}
```

---

## Resources

### Official Links
- **API Documentation**: https://thetvdb.github.io/v4-api/
- **API Information**: https://thetvdb.com/api-information
- **API Key Dashboard**: https://www.thetvdb.com/dashboard/account/apikey
- **Support Portal**: https://support.thetvdb.com/
- **GitHub Repository**: https://github.com/thetvdb/v4-api
- **Issue Tracker**: https://github.com/thetvdb/v4-api/issues

### Testing Tools
- **Postman Collection**: https://www.getpostman.com/collections/7a9397ce69ff246f74d0
- **Swagger UI**: https://thetvdb.github.io/v4-api/ (interactive API explorer)

### Official Libraries
- **Python**: https://github.com/thetvdb/tvdb-v4-python
- **Community Libraries**: Check GitHub for language-specific implementations

### Community Resources
- **TheTVDB Forums**: Community support and discussions
- **Discord/Slack**: Check support portal for community channels

### Legal & Terms
- **Terms of Service**: https://thetvdb.com/tos
- **Attribution Requirements**: Must link to TheTVDB.com
- **DMCA Policy**: https://thetvdb.com/dmca
- **Privacy Policy**: https://thetvdb.com/privacy-policy

### Additional Documentation
- **FAQ**: https://support.thetvdb.com/kb/faq.php
- **Air Times Explanation**: https://support.thetvdb.com/kb/faq.php?id=29
- **API Migration Guide**: For v3 to v4 migration (if applicable)

---

## Changelog & Version History

**Current Version**: 4.7.10

### Key Changes in v4
- JWT-based authentication (replaced API key header)
- Extended records with richer data
- Improved search capabilities
- Update tracking for incremental sync
- Multi-language translation support
- Enhanced artwork management
- User favorites functionality

### Breaking Changes from v3
- Authentication method changed to JWT
- Endpoint structure reorganized
- Response format standardized
- Some legacy endpoints removed
- Image URLs now use CDN

---

## Quick Reference: Common Workflows

### Workflow 1: Search and Display Series
```javascript
// 1. Search for series
const results = await client.searchSeries('Breaking Bad');

// 2. Get first result ID
const seriesId = results.data[0].id;

// 3. Get full series details
const series = await client.getSeries(seriesId, true);

// 4. Display information
console.log(`Name: ${series.data.name}`);
console.log(`Status: ${series.data.status.name}`);
console.log(`Rating: ${series.data.contentRatings[0].name}`);
console.log(`Genres: ${series.data.genres.map(g => g.name).join(', ')}`);
console.log(`Poster: ${series.data.image}`);
```

### Workflow 2: Get All Episodes for a Series
```javascript
const seriesId = 81189;
const allEpisodes = [];
let page = 0;
let hasMore = true;

while (hasMore) {
  const response = await client.getSeriesEpisodes(seriesId, 'default', page);
  allEpisodes.push(...response.data.episodes);
  hasMore = response.links && response.links.next;
  page++;
}

console.log(`Total episodes: ${allEpisodes.length}`);
```

### Workflow 3: Incremental Sync
```javascript
// Store in database
let lastSyncTimestamp = getLastSyncFromDB();

// Get updates since last sync
const updates = await client.getUpdates(lastSyncTimestamp, 'series');

for (const update of updates.data) {
  switch (update.method) {
    case 'create':
    case 'update':
      const series = await client.getSeries(update.recordId);
      await saveToDB(series.data);
      break;

    case 'delete':
      if (update.mergeToId) {
        await mergeInDB(update.recordId, update.mergeToId);
      } else {
        await deleteFromDB(update.recordId);
      }
      break;
  }
}

// Update sync timestamp
saveLastSyncToDB(Math.floor(Date.now() / 1000));
```

### Workflow 4: Multi-Language Content
```javascript
const seriesId = 81189;

// Get available languages
const series = await client.getSeries(seriesId);
const availableLanguages = series.data.nameTranslations;

// Fetch translations
const translations = {};
for (const lang of availableLanguages) {
  const translation = await client.request(`/series/${seriesId}/translations/${lang}`);
  translations[lang] = translation.data;
}

// Display in user's language
const userLang = 'spa';
if (translations[userLang]) {
  console.log(translations[userLang].name);
  console.log(translations[userLang].overview);
}
```

---

## Summary

TheTVDB API v4 provides comprehensive access to TV and movie metadata with:

1. **JWT Authentication**: 1-month tokens via `/login`
2. **Rich Endpoints**: Series, episodes, movies, people, artwork, and more
3. **Advanced Search**: Multi-entity search with filters
4. **Update Tracking**: Incremental sync via `/updates`
5. **Multi-Language**: Translation support for global audiences
6. **Flexible Data Access**: Base and extended records
7. **Reference Data**: Genres, languages, countries, ratings

**Key Recommendations:**
- Implement caching for better performance
- Use update tracking for large datasets
- Handle authentication token expiry
- Respect rate limits with retry logic
- Cache reference data long-term
- Use search as primary discovery method

**Getting Started:**
1. Sign up at TheTVDB.com
2. Get API key from dashboard
3. Authenticate via `/login`
4. Start querying endpoints
5. Implement caching and error handling

For support, consult the official documentation, community forums, or GitHub issues.
