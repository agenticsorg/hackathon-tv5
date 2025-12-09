# Nexus-UMMID Metadata API Documentation Summary

## Overview

Comprehensive OpenAPI 3.0 specification created for the Nexus-UMMID Metadata API, an enterprise-grade metadata integration and distribution platform serving 400M+ users.

**Location**: `/home/user/hackathon-tv5/apps/metadata-api/docs/openapi.yaml`

---

## API Information

- **Title**: Nexus-UMMID Metadata API
- **Version**: 1.0.0
- **Format**: OpenAPI 3.0.0
- **Authentication**: API Key (via `x-api-key` header)

### Servers

1. **Production**: `https://api.nexus-ummid.io/v1`
2. **Staging**: `https://staging-api.nexus-ummid.io/v1`
3. **Local Development**: `http://localhost:3000/v1`

---

## Documented Endpoints

### 1. Health Check
- **GET** `/health`
- **Purpose**: Service health and status monitoring
- **Security**: Public (no authentication required)
- **Response**: System status, uptime, dependency health

### 2. Metadata Management

#### List Metadata
- **GET** `/api/v1/metadata`
- **Purpose**: Retrieve paginated list of media metadata
- **Parameters**:
  - `contentType` (movie, series, episode, etc.)
  - `genre`, `territory`, `platform`
  - `limit` (1-100, default: 20)
  - `offset` (pagination)
- **Response**: Array of metadata records with pagination info

#### Create Metadata
- **POST** `/api/v1/metadata`
- **Purpose**: Ingest new media metadata record
- **Features**:
  - Automatic vectorization of text fields
  - EIDR resolution and linking
  - Validation against MovieLabs MEC schema
- **Response**: Created metadata record (201)

#### Get Metadata by ID
- **GET** `/api/v1/metadata/{id}`
- **Purpose**: Retrieve detailed metadata for specific asset
- **Parameters**:
  - `id` (EIDR ID or internal UUID)
  - `includeRights` (boolean)
  - `includeTechnical` (boolean)
- **Response**: Complete metadata record with provenance

#### Update Metadata
- **PUT** `/api/v1/metadata/{id}`
- **Purpose**: Update existing metadata record
- **Features**: Supports partial updates
- **Response**: Updated metadata record (200)

#### Delete Metadata
- **DELETE** `/api/v1/metadata/{id}`
- **Purpose**: Soft delete metadata record
- **Note**: Record retained for audit purposes
- **Response**: No content (204)

### 3. Semantic Search

#### Search Metadata
- **POST** `/api/v1/metadata/search`
- **Purpose**: Vector-powered semantic search
- **Features**:
  - Natural language queries
  - Vector similarity matching
  - Advanced filtering (year ranges, genres, territories)
- **Example Query**: "movies about artificial intelligence and consciousness"
- **Response**: Ranked results with relevance scores

### 4. Content Recommendations

#### Generate Recommendations
- **POST** `/api/v1/recommendations`
- **Purpose**: AI-powered personalized recommendations
- **Algorithm**: Hypergraph cognitive architecture
- **Features**:
  - User preference matching
  - Contextual awareness (territory, platform, time)
  - Collaborative and content-based filtering
  - Explainable AI reasoning
- **Response**: Ranked recommendations with match factors

#### Get User Recommendations
- **GET** `/api/v1/recommendations/{userId}`
- **Purpose**: Retrieve cached recommendations
- **Parameters**:
  - `userId` (required)
  - `limit` (1-50, default: 20)
- **Response**: Cached recommendation list

### 5. Rights Management

#### Check Availability
- **GET** `/api/v1/rights/availability`
- **Purpose**: Query content availability windows
- **Features**:
  - Bitemporal modeling support
  - Historical and future-dated queries
  - Territory and platform-specific rights
- **Parameters**:
  - `contentId`, `territory` (required)
  - `platform`, `asOfDate` (optional)
- **Response**: Availability status, windows, rights, restrictions

---

## Data Schemas

### Core Schemas

#### 1. MediaMetadata
**Required**: `id`, `title`, `contentType`

**Properties**:
- **Identifiers**: `id`, `eidrId`
- **Descriptive**: `title`, `originalTitle`, `synopsis`, `logline`
- **Classification**: `contentType`, `genres`, `releaseYear`
- **People**: `director`, `cast[]`
- **Geographic**: `territories[]`
- **Ratings**: `ratings[]` (MPAA, BBFC, etc.)
- **Technical**: `technicalSpecs` (resolution, colorimetry, audio)
- **AI-Enhanced**: `tags[]` (mood, theme, tone)
- **Provenance**: `source`, `confidence`, `lastVerified`
- **Timestamps**: `createdAt`, `updatedAt`

#### 2. ContentRecommendation
**Required**: `contentId`, `title`, `score`

**Properties**:
- **Identification**: `contentId`, `title`
- **Scoring**: `score` (0-1 confidence)
- **Explanation**: `reasoning` (human-readable)
- **Match Factors**:
  - `genreMatch`
  - `moodMatch`
  - `similarityToHistory`
  - `trendingScore`
- **Metadata**: Embedded content metadata

#### 3. UserPreference
**Properties**:
- **Content Preferences**: `genres[]`, `mood[]`, `excludeGenres[]`
- **Quality Settings**: `minRating`
- **Language**: `preferredLanguages[]` (ISO 639-1)
- **Behavior**: `excludeViewed` (boolean)

### Supporting Schemas

- **CastMember**: `name`, `role`, `characterName`, `order`
- **ContentRating**: `territory`, `system`, `rating`, `descriptors[]`
- **TechnicalSpecifications**: `resolution`, `aspectRatio`, `colorimetry`, `audioFormats[]`
- **DataProvenance**: `source`, `confidence`, `lastVerified`, `verifiedBy`
- **AvailabilityResponse**: `window`, `rights[]`, `qualityTiers[]`, `restrictions`

---

## Error Responses

### Standard HTTP Error Codes

#### 400 Bad Request
- **Cause**: Invalid input parameters
- **Example**: Invalid content type, malformed request body
- **Response**: Error code, message, field details

#### 401 Unauthorized
- **Cause**: Missing or invalid API key
- **Response**: Authentication error details

#### 404 Not Found
- **Cause**: Resource does not exist
- **Example**: Metadata record with specified ID not found
- **Response**: Error with resource identifier

#### 409 Conflict
- **Cause**: Duplicate record
- **Example**: Record with same EIDR ID already exists
- **Response**: Conflict details

#### 500 Internal Server Error
- **Cause**: Unexpected server error
- **Response**: Generic error with request ID for support

### Error Response Schema
```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error message",
  "details": {},
  "requestId": "unique-request-id",
  "timestamp": "2025-12-06T00:00:00Z"
}
```

---

## Key Features Documented

### 1. Hypergraph Architecture
- N-dimensional rights modeling via hyperedges
- Connects Asset + Territory + Platform + Window + Quality simultaneously
- Eliminates join table complexity

### 2. Semantic Search
- Vector-powered natural language queries
- HNSW (Hierarchical Navigable Small World) indexing
- Automatic text vectorization using media-tuned LLMs

### 3. EIDR Integration
- Canonical identifier resolution
- Automatic entity linking and reconciliation
- Support for multiple ID systems (EIDR, TMS, House IDs)

### 4. Bitemporal Modeling
- Valid Time (real-world rights validity)
- Transaction Time (data entry timestamp)
- Historical and future-dated queries
- Time-travel debugging capability

### 5. AI-Powered Enrichment
- Automated metadata generation
- Mood, tone, and theme extraction
- Confidence scoring and provenance tracking

### 6. Multi-Platform Compliance
- Netflix IMF (Interoperable Master Format) support
- Apple UMC (Universal Media Catalog) feeds
- Dolby Vision dynamic metadata
- BCP-47 language tag enforcement

### 7. Real-Time Recommendations
- Hypergraph cognitive architecture
- Collaborative + content-based filtering
- Explainable AI with match factor breakdown
- Context-aware (territory, platform, time of day)

---

## Authentication

### API Key Authentication
- **Header**: `x-api-key`
- **Type**: API Key
- **Required**: All endpoints except `/health`
- **Format**: String token

**Example**:
```bash
curl -H "x-api-key: your-api-key-here" \
  https://api.nexus-ummid.io/v1/api/v1/metadata
```

---

## Example Use Cases

### 1. Content Discovery
```bash
POST /api/v1/metadata/search
{
  "query": "mind-bending thrillers about dreams",
  "filters": {
    "releaseYear": { "min": 2000, "max": 2024 }
  }
}
```

### 2. Personalized Recommendations
```bash
POST /api/v1/recommendations
{
  "userId": "user-12345",
  "preferences": {
    "genres": ["sci-fi", "thriller"],
    "mood": ["intense", "thought-provoking"]
  },
  "context": {
    "territory": "US",
    "platform": "streaming"
  }
}
```

### 3. Rights Verification
```bash
GET /api/v1/rights/availability
  ?contentId=eidr:10.5240/ABCD-1234
  &territory=US
  &platform=netflix
  &asOfDate=2025-06-01T00:00:00Z
```

### 4. Metadata Ingestion
```bash
POST /api/v1/metadata
{
  "title": "Blade Runner 2049",
  "contentType": "movie",
  "releaseYear": 2017,
  "genres": ["sci-fi", "thriller"],
  "technicalSpecs": {
    "resolution": "UHD",
    "colorimetry": "Dolby Vision"
  }
}
```

---

## Scale & Performance

### Design Targets
- **Graph Size**: 100M+ nodes, 1B+ hyperedges
- **User Scale**: 400M+ users
- **API Latency**: <20ms (p99) for read operations
- **Search Speed**: <500ms for complex queries
- **Ingest Throughput**: 10,000 records/minute

### Architecture Optimizations
- Namespace sharding (Asset/User/Usage data)
- HNSW vector indexing (150x faster search)
- Change Data Capture (CDC) for real-time updates
- Kafka-based delta publishing
- Distributed graph storage

---

## Integration Standards

### Industry Compliance
- **MovieLabs MEC** (Media Entertainment Core)
- **EMA Avails** (Entertainment Merchants Association)
- **EIDR** (Entertainment Identifier Registry)
- **SMPTE ST 2067** (IMF Standard)
- **ISO 3166-1** (Country codes)
- **ISO 639-1** (Language codes)
- **BCP-47** (Language tags)

---

## Next Steps

### For Developers
1. Review the OpenAPI specification: `/apps/metadata-api/docs/openapi.yaml`
2. Import into API client tools (Postman, Insomnia, Swagger UI)
3. Generate client SDKs using OpenAPI Generator
4. Contact support for API key provisioning

### For Implementers
1. Test endpoints with provided examples
2. Validate request/response schemas
3. Implement error handling for all error codes
4. Configure authentication headers
5. Monitor API health endpoint

### For Stakeholders
1. Review API capabilities and coverage
2. Validate alignment with business requirements
3. Plan integration workflows
4. Define SLA requirements

---

## Documentation Quality

✅ **OpenAPI 3.0 Compliant**
✅ **Complete request/response examples**
✅ **All schemas documented with types and constraints**
✅ **Authentication and security schemes defined**
✅ **Error responses with detailed examples**
✅ **Industry-standard data formats**
✅ **Comprehensive descriptions and use cases**
✅ **Scale and performance targets specified**

---

**Generated**: 2025-12-06
**API Version**: 1.0.0
**Specification**: OpenAPI 3.0.0
**Documentation Format**: YAML
**Location**: `/home/user/hackathon-tv5/apps/metadata-api/docs/openapi.yaml`
