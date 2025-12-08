# API Specification - Media Discovery Platform

**Version:** 1.0
**Date:** 2025-12-05
**Base URL:** `https://api.mediadiscovery.com`

---

## Table of Contents

1. [Authentication](#authentication)
2. [Search API](#search-api)
3. [Content API](#content-api)
4. [User API](#user-api)
5. [Recommendation API](#recommendation-api)
6. [Platform API](#platform-api)
7. [GraphQL Schema](#graphql-schema)
8. [Error Codes](#error-codes)
9. [Rate Limiting](#rate-limiting)
10. [Webhooks](#webhooks)

---

## Authentication

### Overview

The API uses JWT (JSON Web Tokens) with RS256 signing for authentication.

### Authentication Flow

```
1. User Login
   POST /api/v1/auth/login
   → Returns: { accessToken, refreshToken }

2. Use Access Token
   GET /api/v1/search
   Headers: Authorization: Bearer <accessToken>

3. Refresh Token (when access token expires)
   POST /api/v1/auth/refresh
   Body: { refreshToken }
   → Returns: { accessToken, refreshToken }
```

### Endpoints

#### Login

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securePassword123"
}
```

**Response (200 OK):**
```json
{
  "accessToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 900,
  "tokenType": "Bearer",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "roles": ["user"]
  }
}
```

#### Register

```http
POST /api/v1/auth/register
Content-Type: application/json

{
  "email": "newuser@example.com",
  "password": "securePassword123",
  "name": "Jane Doe"
}
```

**Response (201 Created):**
```json
{
  "accessToken": "...",
  "refreshToken": "...",
  "user": {
    "id": "...",
    "email": "newuser@example.com",
    "name": "Jane Doe",
    "roles": ["user"]
  }
}
```

#### Refresh Token

```http
POST /api/v1/auth/refresh
Content-Type: application/json

{
  "refreshToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Response (200 OK):**
```json
{
  "accessToken": "...",
  "refreshToken": "...",
  "expiresIn": 900
}
```

#### Logout

```http
POST /api/v1/auth/logout
Authorization: Bearer <accessToken>
Content-Type: application/json

{
  "refreshToken": "..."
}
```

**Response (204 No Content)**

---

## Search API

### Natural Language Search

```http
POST /api/v1/search
Authorization: Bearer <accessToken>
Content-Type: application/json

{
  "query": "Find me a rom-com streaming tonight",
  "limit": 20,
  "offset": 0,
  "filters": {
    "type": ["movie"],
    "platforms": ["netflix", "hulu"],
    "minRating": 7.0
  }
}
```

**Response (200 OK):**
```json
{
  "query": "Find me a rom-com streaming tonight",
  "totalResults": 245,
  "results": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "title": "The Proposal",
      "originalTitle": "The Proposal",
      "type": "movie",
      "description": "A pushy boss forces her young assistant to marry her...",
      "releaseYear": 2009,
      "runtime": 108,
      "contentRating": "PG-13",
      "genres": ["Comedy", "Romance"],
      "languages": ["English"],
      "countries": ["USA"],
      "rating": {
        "average": 8.2,
        "count": 15420,
        "distribution": {
          "1": 120,
          "2": 350,
          "3": 1100,
          "4": 4200,
          "5": 9650
        }
      },
      "cast": [
        {
          "id": "...",
          "name": "Sandra Bullock",
          "role": "actor",
          "character": "Margaret Tate",
          "profileImage": "https://cdn.mediadiscovery.com/people/sandra-bullock.jpg"
        },
        {
          "id": "...",
          "name": "Ryan Reynolds",
          "role": "actor",
          "character": "Andrew Paxton",
          "profileImage": "https://cdn.mediadiscovery.com/people/ryan-reynolds.jpg"
        }
      ],
      "crew": [
        {
          "id": "...",
          "name": "Anne Fletcher",
          "role": "director"
        }
      ],
      "availability": [
        {
          "platform": {
            "id": "...",
            "name": "Netflix",
            "slug": "netflix",
            "logo": "https://cdn.mediadiscovery.com/platforms/netflix.png"
          },
          "available": true,
          "url": "https://www.netflix.com/title/70111470",
          "price": {
            "tier": "subscription",
            "amount": null,
            "currency": "USD"
          },
          "region": "US",
          "lastVerified": "2025-12-05T10:30:00Z"
        },
        {
          "platform": {
            "id": "...",
            "name": "Hulu",
            "slug": "hulu",
            "logo": "https://cdn.mediadiscovery.com/platforms/hulu.png"
          },
          "available": true,
          "url": "https://www.hulu.com/watch/...",
          "price": {
            "tier": "subscription",
            "amount": null,
            "currency": "USD"
          },
          "region": "US",
          "lastVerified": "2025-12-05T09:15:00Z"
        }
      ],
      "media": {
        "posterUrl": "https://cdn.mediadiscovery.com/posters/the-proposal.jpg",
        "backdropUrl": "https://cdn.mediadiscovery.com/backdrops/the-proposal.jpg",
        "trailerUrl": "https://www.youtube.com/watch?v=RFL6KxB7Ywk"
      },
      "metadata": {
        "imdbId": "tt1041829",
        "tmdbId": 19995
      },
      "relevanceScore": 0.95,
      "userContext": {
        "inWatchlist": false,
        "watched": false,
        "userRating": null
      }
    }
    // ... more results
  ],
  "facets": {
    "genres": {
      "Comedy": 180,
      "Romance": 245,
      "Drama": 85
    },
    "platforms": {
      "netflix": 120,
      "hulu": 95,
      "disney-plus": 30
    },
    "releaseYears": {
      "2023": 15,
      "2022": 25,
      "2021": 30,
      "2020": 20,
      "2019": 25,
      "older": 130
    }
  },
  "metadata": {
    "searchTimeMs": 185,
    "cacheHit": false,
    "sources": ["vector", "sql", "elasticsearch"]
  },
  "pagination": {
    "limit": 20,
    "offset": 0,
    "totalResults": 245,
    "hasMore": true,
    "nextOffset": 20
  }
}
```

### Search Suggestions (Autocomplete)

```http
GET /api/v1/search/suggestions?q=romantic&limit=10
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "query": "romantic",
  "suggestions": [
    {
      "text": "romantic comedies",
      "type": "genre",
      "score": 0.95
    },
    {
      "text": "romantic dramas",
      "type": "genre",
      "score": 0.88
    },
    {
      "text": "Romantic Movie",
      "type": "title",
      "contentId": "...",
      "score": 0.82
    },
    {
      "text": "romantic movies on netflix",
      "type": "query",
      "score": 0.78
    }
  ]
}
```

### Semantic Search (Advanced)

```http
POST /api/v1/search/semantic
Authorization: Bearer <accessToken>
Content-Type: application/json

{
  "query": "Movies about artificial intelligence and ethics",
  "searchType": "semantic",
  "limit": 20,
  "filters": {
    "minRating": 7.5,
    "releaseYear": { "gte": 2010 }
  }
}
```

---

## Content API

### Get Content by ID

```http
GET /api/v1/content/{contentId}
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Inception",
  "type": "movie",
  "description": "A thief who steals corporate secrets...",
  // ... full content details (same structure as search results)
  "similar": [
    {
      "id": "...",
      "title": "The Matrix",
      "posterUrl": "...",
      "rating": 8.7,
      "similarityScore": 0.89
    }
    // ... more similar content
  ]
}
```

### Get Content Availability

```http
GET /api/v1/content/{contentId}/availability
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "contentId": "550e8400-e29b-41d4-a716-446655440000",
  "availability": [
    {
      "platform": {
        "id": "...",
        "name": "Netflix",
        "slug": "netflix"
      },
      "available": true,
      "url": "https://www.netflix.com/title/70111470",
      "price": {
        "tier": "subscription"
      },
      "lastVerified": "2025-12-05T10:30:00Z"
    }
  ],
  "verifiedAt": "2025-12-05T10:30:00Z"
}
```

### Get Similar Content

```http
GET /api/v1/content/{contentId}/similar?limit=10
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "contentId": "550e8400-e29b-41d4-a716-446655440000",
  "similar": [
    {
      "id": "...",
      "title": "The Matrix",
      "type": "movie",
      "genres": ["Action", "Sci-Fi"],
      "rating": 8.7,
      "posterUrl": "...",
      "similarityScore": 0.89,
      "reason": "Similar themes and visual style"
    }
    // ... more results
  ]
}
```

### Trending Content

```http
GET /api/v1/trending?limit=20&timeframe=week
Authorization: Bearer <accessToken>
```

**Query Parameters:**
- `limit` (default: 20, max: 100)
- `timeframe` (day, week, month)
- `type` (movie, series, documentary)

**Response (200 OK):**
```json
{
  "timeframe": "week",
  "trending": [
    {
      "rank": 1,
      "content": {
        // ... full content object
      },
      "metrics": {
        "uniqueViewers": 125000,
        "growthRate": 0.45,
        "averageRating": 8.5
      }
    }
    // ... more results
  ]
}
```

---

## User API

### Get Current User Profile

```http
GET /api/v1/users/me
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "name": "John Doe",
  "roles": ["user"],
  "createdAt": "2024-01-15T08:30:00Z",
  "emailVerified": true,
  "status": "active"
}
```

### Update User Profile

```http
PATCH /api/v1/users/me
Authorization: Bearer <accessToken>
Content-Type: application/json

{
  "name": "John Smith",
  "preferences": {
    "favoriteGenres": ["thriller", "sci-fi", "drama"],
    "preferredLanguages": ["en", "es"],
    "contentRatingLimit": "R",
    "preferredPlatforms": ["netflix", "hbo"],
    "notificationSettings": {
      "email": true,
      "push": true,
      "newContent": true,
      "recommendations": false
    }
  }
}
```

**Response (200 OK):**
```json
{
  "id": "...",
  "name": "John Smith",
  "preferences": {
    // ... updated preferences
  },
  "updatedAt": "2025-12-05T10:30:00Z"
}
```

### Get User Viewing History

```http
GET /api/v1/users/me/history?limit=50&offset=0
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "history": [
    {
      "id": "...",
      "content": {
        "id": "...",
        "title": "Inception",
        "type": "movie",
        "posterUrl": "..."
      },
      "watchedAt": "2025-12-04T20:15:00Z",
      "duration": 7200,
      "completed": true,
      "platform": {
        "id": "...",
        "name": "Netflix",
        "slug": "netflix"
      }
    }
    // ... more history entries
  ],
  "pagination": {
    "limit": 50,
    "offset": 0,
    "total": 245,
    "hasMore": true
  }
}
```

### Get User Watchlist

```http
GET /api/v1/users/me/watchlist?limit=50&offset=0
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "watchlist": [
    {
      "content": {
        // ... full content object
      },
      "addedAt": "2025-12-01T14:30:00Z",
      "priority": 1,
      "availableOn": ["netflix", "hulu"]
    }
    // ... more watchlist items
  ],
  "pagination": {
    "limit": 50,
    "offset": 0,
    "total": 42,
    "hasMore": false
  }
}
```

### Add to Watchlist

```http
POST /api/v1/users/me/watchlist
Authorization: Bearer <accessToken>
Content-Type: application/json

{
  "contentId": "550e8400-e29b-41d4-a716-446655440000",
  "priority": 1
}
```

**Response (201 Created):**
```json
{
  "contentId": "550e8400-e29b-41d4-a716-446655440000",
  "addedAt": "2025-12-05T10:30:00Z",
  "priority": 1
}
```

### Rate Content

```http
POST /api/v1/users/me/ratings
Authorization: Bearer <accessToken>
Content-Type: application/json

{
  "contentId": "550e8400-e29b-41d4-a716-446655440000",
  "rating": 4.5,
  "review": "Great movie! Loved the concept and execution."
}
```

**Response (201 Created):**
```json
{
  "contentId": "550e8400-e29b-41d4-a716-446655440000",
  "rating": 4.5,
  "review": "Great movie! Loved the concept and execution.",
  "createdAt": "2025-12-05T10:30:00Z"
}
```

---

## Recommendation API

### Get Personalized Recommendations

```http
GET /api/v1/recommendations?limit=20
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "userId": "550e8400-e29b-41d4-a716-446655440000",
  "recommendations": [
    {
      "content": {
        // ... full content object
      },
      "score": 0.92,
      "reason": "Because you watched The Matrix",
      "strategy": "collaborative",
      "confidence": 0.88
    },
    {
      "content": {
        // ... full content object
      },
      "score": 0.89,
      "reason": "Trending in Sci-Fi",
      "strategy": "trending",
      "confidence": 0.92
    },
    {
      "content": {
        // ... full content object
      },
      "score": 0.85,
      "reason": "Hidden gem based on your taste",
      "strategy": "serendipity",
      "confidence": 0.75
    }
    // ... more recommendations
  ],
  "generatedAt": "2025-12-05T10:30:00Z",
  "expiresAt": "2025-12-05T11:30:00Z",
  "metadata": {
    "strategies": {
      "collaborative": 8,
      "contentBased": 6,
      "trending": 4,
      "serendipity": 2
    }
  }
}
```

### Get Recommendations by Category

```http
GET /api/v1/recommendations/category/{category}?limit=20
Authorization: Bearer <accessToken>
```

**Categories:**
- `because-you-watched` - Similar to watched content
- `trending` - Currently trending
- `new-releases` - Recently released
- `highly-rated` - Top-rated content
- `hidden-gems` - Lesser-known quality content

**Response:** Same structure as personalized recommendations

---

## Platform API

### List Platforms

```http
GET /api/v1/platforms
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "platforms": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Netflix",
      "slug": "netflix",
      "logo": "https://cdn.mediadiscovery.com/platforms/netflix.png",
      "websiteUrl": "https://www.netflix.com",
      "active": true,
      "contentCount": 5247,
      "regions": ["US", "UK", "CA", "AU"]
    }
    // ... more platforms
  ]
}
```

### List Genres

```http
GET /api/v1/genres
Authorization: Bearer <accessToken>
```

**Response (200 OK):**
```json
{
  "genres": [
    {
      "name": "Action",
      "slug": "action",
      "contentCount": 8542
    },
    {
      "name": "Comedy",
      "slug": "comedy",
      "contentCount": 12458
    }
    // ... more genres
  ]
}
```

---

## GraphQL Schema

### GraphQL Endpoint

```
POST /api/v1/graphql
Authorization: Bearer <accessToken>
Content-Type: application/json
```

### Schema

```graphql
type Query {
  # Search
  search(input: SearchInput!): SearchResult!

  # Content
  content(id: ID!): Content
  contentByIds(ids: [ID!]!): [Content!]!
  trending(limit: Int = 20, timeframe: Timeframe = WEEK): [TrendingContent!]!

  # Recommendations
  recommendations(limit: Int = 20): [Recommendation!]!
  recommendationsByCategory(category: String!, limit: Int = 20): [Recommendation!]!

  # User
  me: User!
  myWatchlist(limit: Int = 50, offset: Int = 0): WatchlistConnection!
  myHistory(limit: Int = 50, offset: Int = 0): HistoryConnection!

  # Platforms
  platforms: [Platform!]!
  platformAvailability(contentId: ID!): [PlatformAvailability!]!

  # Genres
  genres: [Genre!]!
}

type Mutation {
  # Auth
  login(email: String!, password: String!): AuthPayload!
  register(input: RegisterInput!): AuthPayload!
  refreshToken(refreshToken: String!): AuthPayload!
  logout(refreshToken: String!): Boolean!

  # User
  updateProfile(input: ProfileInput!): User!
  updatePreferences(input: PreferencesInput!): UserPreferences!

  # Interactions
  addToWatchlist(contentId: ID!, priority: Int): WatchlistItem!
  removeFromWatchlist(contentId: ID!): Boolean!
  rateContent(contentId: ID!, rating: Float!, review: String): Rating!
  recordView(contentId: ID!, duration: Int, completed: Boolean): Boolean!
}

type Subscription {
  # Real-time availability updates
  availabilityChanged(contentId: ID!): PlatformAvailability!

  # New content alerts
  newContentForUser: Content!

  # Recommendation updates
  recommendationsUpdated: [Recommendation!]!
}

# Types
type Content {
  id: ID!
  title: String!
  originalTitle: String
  type: ContentType!
  description: String
  releaseYear: Int
  runtime: Int
  contentRating: String
  genres: [String!]!
  languages: [String!]!
  countries: [String!]!
  rating: Rating!
  cast: [Person!]!
  crew: [Person!]!
  availability: [PlatformAvailability!]!
  media: MediaUrls!
  metadata: ContentMetadata!
  similar(limit: Int = 10): [Content!]!
  userContext: UserContentContext
}

type Person {
  id: ID!
  name: String!
  role: String!
  character: String
  profileImage: String
}

type PlatformAvailability {
  platform: Platform!
  available: Boolean!
  url: String
  price: Price
  region: String!
  lastVerified: DateTime!
}

type Platform {
  id: ID!
  name: String!
  slug: String!
  logo: String
  websiteUrl: String
  active: Boolean!
  contentCount: Int!
}

type SearchResult {
  query: String!
  totalResults: Int!
  results: [Content!]!
  facets: Facets!
  metadata: SearchMetadata!
  pagination: Pagination!
}

type Recommendation {
  content: Content!
  score: Float!
  reason: String!
  strategy: String!
  confidence: Float!
}

type User {
  id: ID!
  email: String!
  name: String
  roles: [String!]!
  preferences: UserPreferences
  createdAt: DateTime!
  emailVerified: Boolean!
  status: String!
}

type UserPreferences {
  favoriteGenres: [String!]!
  preferredLanguages: [String!]!
  contentRatingLimit: String
  preferredPlatforms: [String!]!
  notificationSettings: NotificationSettings!
}

# Inputs
input SearchInput {
  query: String!
  limit: Int = 20
  offset: Int = 0
  filters: SearchFilters
}

input SearchFilters {
  type: [ContentType!]
  genres: [String!]
  platforms: [String!]
  minRating: Float
  maxRating: Float
  releaseYear: RangeInput
  runtime: RangeInput
  languages: [String!]
  regions: [String!]
}

input RangeInput {
  gte: Int
  lte: Int
}

input RegisterInput {
  email: String!
  password: String!
  name: String
}

input ProfileInput {
  name: String
  preferences: PreferencesInput
}

input PreferencesInput {
  favoriteGenres: [String!]
  preferredLanguages: [String!]
  contentRatingLimit: String
  preferredPlatforms: [String!]
  notificationSettings: NotificationSettingsInput
}

# Enums
enum ContentType {
  MOVIE
  SERIES
  DOCUMENTARY
  SHORT
}

enum Timeframe {
  DAY
  WEEK
  MONTH
}

# Scalars
scalar DateTime
scalar JSON
```

### Example Query

```graphql
query SearchMovies {
  search(input: {
    query: "sci-fi movies with time travel"
    limit: 10
    filters: {
      type: [MOVIE]
      minRating: 7.0
      releaseYear: { gte: 2010 }
    }
  }) {
    query
    totalResults
    results {
      id
      title
      releaseYear
      rating {
        average
        count
      }
      genres
      availability {
        platform {
          name
          slug
        }
        available
        url
      }
      media {
        posterUrl
      }
    }
    pagination {
      hasMore
      nextOffset
    }
  }
}
```

### Example Mutation

```graphql
mutation AddToWatchlist {
  addToWatchlist(
    contentId: "550e8400-e29b-41d4-a716-446655440000"
    priority: 1
  ) {
    contentId
    addedAt
    priority
  }
}
```

### Example Subscription

```graphql
subscription WatchAvailability {
  availabilityChanged(contentId: "550e8400-e29b-41d4-a716-446655440000") {
    platform {
      name
    }
    available
    url
    lastVerified
  }
}
```

---

## Error Codes

### HTTP Status Codes

| Code | Meaning | Usage |
|------|---------|-------|
| 200 | OK | Successful GET, PATCH, PUT |
| 201 | Created | Successful POST |
| 204 | No Content | Successful DELETE |
| 400 | Bad Request | Invalid request body/parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Authenticated but insufficient permissions |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource conflict (duplicate) |
| 422 | Unprocessable Entity | Validation error |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 502 | Bad Gateway | Upstream service error |
| 503 | Service Unavailable | Service temporarily unavailable |

### Error Response Format

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": [
      {
        "field": "email",
        "message": "Email is required"
      },
      {
        "field": "password",
        "message": "Password must be at least 8 characters"
      }
    ],
    "requestId": "req_1234567890",
    "timestamp": "2025-12-05T10:30:00Z"
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `AUTHENTICATION_FAILED` | Invalid credentials |
| `TOKEN_EXPIRED` | Access token expired |
| `TOKEN_INVALID` | Invalid token format |
| `UNAUTHORIZED` | Authentication required |
| `FORBIDDEN` | Insufficient permissions |
| `NOT_FOUND` | Resource not found |
| `VALIDATION_ERROR` | Request validation failed |
| `RATE_LIMIT_EXCEEDED` | Too many requests |
| `CONFLICT` | Resource already exists |
| `INTERNAL_ERROR` | Server error |
| `SERVICE_UNAVAILABLE` | Service temporarily down |
| `UPSTREAM_ERROR` | External service error |

---

## Rate Limiting

### Rate Limit Tiers

| User Type | Requests/Minute | Requests/Hour | Requests/Day |
|-----------|----------------|---------------|--------------|
| Anonymous | 10 | 100 | 500 |
| User | 100 | 1,000 | 10,000 |
| Premium | 1,000 | 10,000 | 100,000 |
| Enterprise | Custom | Custom | Custom |

### Rate Limit Headers

Every API response includes rate limit headers:

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 87
X-RateLimit-Reset: 1733401200
Retry-After: 45
```

### Rate Limit Exceeded Response

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
Retry-After: 45

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded. Please try again in 45 seconds.",
    "limit": 100,
    "remaining": 0,
    "resetAt": "2025-12-05T10:31:00Z",
    "retryAfter": 45
  }
}
```

---

## Webhooks

### Webhook Events

Register webhook endpoints to receive real-time notifications.

#### Available Events

| Event | Description |
|-------|-------------|
| `content.added` | New content added to platform |
| `content.updated` | Content metadata updated |
| `content.removed` | Content removed from platform |
| `availability.changed` | Content availability changed |
| `recommendation.ready` | New recommendations generated |
| `user.watchlist.updated` | User's watchlist updated |

### Webhook Payload

```json
{
  "event": "availability.changed",
  "timestamp": "2025-12-05T10:30:00Z",
  "data": {
    "contentId": "550e8400-e29b-41d4-a716-446655440000",
    "platform": "netflix",
    "available": true,
    "url": "https://www.netflix.com/title/70111470",
    "previousState": {
      "available": false
    }
  },
  "signature": "sha256=abc123..."
}
```

### Webhook Signature Verification

```typescript
import crypto from 'crypto';

function verifyWebhookSignature(
  payload: string,
  signature: string,
  secret: string
): boolean {
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');

  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(`sha256=${expectedSignature}`)
  );
}
```

### Webhook Registration

```http
POST /api/v1/webhooks
Authorization: Bearer <accessToken>
Content-Type: application/json

{
  "url": "https://your-app.com/webhooks/mediadiscovery",
  "events": ["availability.changed", "content.added"],
  "secret": "your-webhook-secret"
}
```

---

## API Versioning

### Version Strategy

- **URL-based versioning**: `/api/v1/`, `/api/v2/`
- **Current version**: v1
- **Deprecation policy**: 12 months notice
- **Sunset header**: Deprecated endpoints include `Sunset` header

### Deprecated Endpoint Response

```http
HTTP/1.1 200 OK
Deprecation: true
Sunset: Wed, 05 Dec 2026 10:00:00 GMT
Link: </api/v2/search>; rel="successor-version"

{
  "warning": "This endpoint is deprecated and will be removed on 2026-12-05. Please migrate to /api/v2/search",
  "data": { ... }
}
```

---

This API specification provides a complete reference for integrating with the Media Discovery Platform.
