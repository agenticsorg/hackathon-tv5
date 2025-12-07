# Discovery Service API Endpoints

## Base URL
`/api/v1`

## Public Endpoints

### Health Check
```
GET /health
```
**Response**:
```json
{
  "status": "healthy",
  "service": "discovery-service",
  "version": "0.1.0"
}
```

---

### Main Search
```
POST /search
```
**Request Body**:
```json
{
  "query": "action movies",
  "filters": {
    "genres": ["action", "thriller"],
    "platforms": ["netflix", "hbo"],
    "year_range": [2020, 2024],
    "rating_range": [7.0, 10.0]
  },
  "page": 1,
  "page_size": 20,
  "user_id": "uuid-string",  // optional
  "experiment_variant": "high_boost"  // optional
}
```

**Response**:
```json
{
  "results": [
    {
      "content": {
        "id": "uuid",
        "title": "Movie Title",
        "overview": "Description",
        "release_year": 2023,
        "genres": ["action"],
        "platforms": ["netflix"],
        "popularity_score": 0.85
      },
      "relevance_score": 0.92,
      "match_reasons": ["title_match", "genre_match"],
      "vector_similarity": 0.88,
      "keyword_score": 0.75
    }
  ],
  "total_count": 150,
  "page": 1,
  "page_size": 20,
  "query_parsed": {
    "original_query": "action movies",
    "normalized_query": "action movie",
    "intent": "search",
    "filters": {}
  },
  "search_time_ms": 45,
  "facets": {
    "genres": [
      {"value": "action", "count": 50},
      {"value": "thriller", "count": 30}
    ],
    "platforms": [
      {"value": "netflix", "count": 40}
    ]
  }
}
```

---

### Autocomplete
```
GET /search/autocomplete?q=marvel&limit=10
```
**Query Parameters**:
- `q` (required): Query prefix
- `limit` (optional): Max suggestions (default: 10)

**Response**:
```json
{
  "query": "marvel",
  "suggestions": [
    {
      "text": "Marvel Cinematic Universe",
      "type": "keyword",
      "popularity": 0.95,
      "metadata": {
        "category": "franchise"
      }
    }
  ],
  "cached": true
}
```

---

### Analytics Dashboard
```
GET /analytics?period=24h&limit=10
```
**Query Parameters**:
- `period` (optional): Time period ("1h", "24h", "7d", "30d", default: "24h")
- `limit` (optional): Top queries limit (default: 10)

**Response**:
```json
{
  "period": "24h",
  "total_searches": 1523,
  "unique_queries": 847,
  "avg_latency_ms": 52.3,
  "p95_latency_ms": 120,
  "zero_result_rate": 0.05,
  "click_through_rate": 0.68,
  "top_queries": [
    {
      "query": "avengers",
      "search_count": 234,
      "avg_results": 45,
      "avg_latency_ms": 48
    }
  ],
  "zero_result_queries": [
    {
      "query": "obscure movie title",
      "search_count": 12
    }
  ]
}
```

---

### Quality Report
```
GET /quality/report?threshold=0.6&limit=100
```
**Query Parameters**:
- `threshold` (optional): Quality score threshold 0.0-1.0 (default: 0.6)
- `limit` (optional): Max items to return (default: 100)

**Response**:
```json
{
  "total_low_quality": 45,
  "threshold": 0.6,
  "low_quality_items": [
    {
      "id": "uuid",
      "title": "Movie Title",
      "quality_score": 0.42,
      "missing_fields": [
        "description",
        "poster",
        "imdb_rating"
      ],
      "platform": "netflix",
      "content_type": "movie"
    }
  ]
}
```

---

## Admin Endpoints
**Authentication**: All admin endpoints require JWT token in `Authorization: Bearer <token>` header
**Authorization**: User must have `admin` role

### Get Default Ranking Config
```
GET /admin/search/ranking
Authorization: Bearer <jwt-token>
```
**Response**:
```json
{
  "version": 5,
  "vector_weight": 0.6,
  "keyword_weight": 0.3,
  "quality_weight": 0.05,
  "freshness_weight": 0.05,
  "created_by": "uuid",
  "created_at": "2024-01-15T10:30:00Z",
  "description": "Balanced ranking configuration"
}
```

---

### Update Default Ranking Config
```
PUT /admin/search/ranking
Authorization: Bearer <jwt-token>
Content-Type: application/json
```
**Request Body**:
```json
{
  "vector_weight": 0.6,
  "keyword_weight": 0.3,
  "quality_weight": 0.05,
  "freshness_weight": 0.05,
  "description": "New balanced config"
}
```

**Validation**:
- All weights must be between 0.0 and 1.0
- Sum of weights must equal 1.0

**Response**: Same as GET config (new version created)

---

### List Ranking Variants
```
GET /admin/search/ranking/variants
Authorization: Bearer <jwt-token>
```
**Response**:
```json
[
  {
    "name": "quality_boost",
    "config": {
      "version": 3,
      "vector_weight": 0.5,
      "keyword_weight": 0.25,
      "quality_weight": 0.2,
      "freshness_weight": 0.05
    },
    "is_active": true,
    "traffic_percentage": 10
  }
]
```

---

### Get Ranking Variant
```
GET /admin/search/ranking/variants/{name}
Authorization: Bearer <jwt-token>
```
**Response**: Same as list item

**Errors**:
- `404`: Variant not found

---

### Create/Update Ranking Variant
```
PUT /admin/search/ranking/variants/{name}
Authorization: Bearer <jwt-token>
Content-Type: application/json
```
**Request Body**:
```json
{
  "vector_weight": 0.5,
  "keyword_weight": 0.25,
  "quality_weight": 0.2,
  "freshness_weight": 0.05,
  "description": "Boost quality content",
  "is_active": true,
  "traffic_percentage": 10
}
```

**Validation**:
- `traffic_percentage`: 0-100 if provided
- Weights validation same as default config

**Response**: Created/updated variant

---

### Delete Ranking Variant
```
DELETE /admin/search/ranking/variants/{name}
Authorization: Bearer <jwt-token>
```
**Response**:
```json
{
  "message": "Ranking variant 'quality_boost' deleted successfully"
}
```

**Errors**:
- `404`: Variant not found

---

### Get Config History
```
GET /admin/search/ranking/history/{version}
Authorization: Bearer <jwt-token>
```
**Response**: Historical config for specified version

**Errors**:
- `404`: Version not found

---

## Error Responses

All endpoints return errors in this format:
```json
{
  "error": "Error message description"
}
```

### HTTP Status Codes
- `200`: Success
- `400`: Bad request (validation error)
- `401`: Unauthorized (missing/invalid JWT)
- `403`: Forbidden (insufficient permissions)
- `404`: Not found
- `500`: Internal server error

---

## Rate Limiting
*Not yet implemented*

Future implementation will use:
- Public endpoints: 100 requests/minute per IP
- Admin endpoints: 1000 requests/minute per user

---

## CORS
*Configuration required*

Allowed origins should be configured in production environment.

---

## Versioning
API version is included in the base path: `/api/v1/`

Breaking changes will increment the version number.
