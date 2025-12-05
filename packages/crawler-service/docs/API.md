# Crawler Service API Documentation

## Base URL

```
http://localhost:3000/api/v1
```

## Authentication

Currently, the API does not require authentication. In production, implement API key authentication.

## Endpoints

### 1. Scrape Single Page

Scrape a single URL and extract content.

**Endpoint:** `POST /scrape`

**Request Body:**
```json
{
  "url": "https://example.com",
  "extractionMode": "machine-view",  // Optional: "html" | "text" | "markdown" | "machine-view"
  "userAgent": "Custom Agent/1.0",   // Optional
  "timeout": 30000,                   // Optional: milliseconds
  "waitFor": "selector",              // Optional: CSS selector or milliseconds
  "screenshot": false,                // Optional: capture screenshot
  "headers": {                        // Optional: custom headers
    "Authorization": "Bearer token"
  },
  "cookies": [                        // Optional: custom cookies
    {
      "name": "session",
      "value": "abc123",
      "domain": "example.com"
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "url": "https://example.com",
    "title": "Example Domain",
    "content": "...",
    "links": ["https://example.com/page1", "..."],
    "metadata": {
      "title": "Example",
      "description": "Example description",
      "author": "John Doe",
      "keywords": ["example", "test"]
    },
    "timestamp": "2024-01-01T00:00:00.000Z",
    "statusCode": 200,
    "contentType": "text/html",
    "machineView": {
      // Only included if extractionMode is "machine-view"
      "title": "Example Domain",
      "description": "Example description",
      "mainContent": "...",
      "sections": [...],
      "links": [...],
      "metadata": {...}
    }
  }
}
```

### 2. Start Crawl Job

Start an asynchronous crawl job.

**Endpoint:** `POST /crawl`

**Request Body:**
```json
{
  "url": "https://example.com",
  "maxDepth": 3,                      // Optional: 1-10, default 3
  "maxPages": 100,                    // Optional: 1-1000, default 100
  "respectRobotsTxt": true,           // Optional: default false
  "followSitemap": true,              // Optional: default false
  "extractionMode": "machine-view",   // Optional
  "userAgent": "Custom Agent/1.0",    // Optional
  "timeout": 30000                    // Optional
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "jobId": "crawl-1234567890-abc123",
    "status": "pending",
    "message": "Crawl job created successfully"
  }
}
```

### 3. Get Crawl Job Status

Check the status of a crawl job.

**Endpoint:** `GET /crawl/:id`

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "crawl-1234567890-abc123",
    "url": "https://example.com",
    "options": {...},
    "status": "completed",  // "pending" | "processing" | "completed" | "failed"
    "progress": 100,
    "results": [...],       // Array of CrawlResult
    "error": null,
    "createdAt": "2024-01-01T00:00:00.000Z",
    "updatedAt": "2024-01-01T00:01:00.000Z",
    "completedAt": "2024-01-01T00:01:00.000Z"
  }
}
```

### 4. Generate Site Map

Generate a site map for a domain.

**Endpoint:** `POST /map`

**Request Body:**
```json
{
  "url": "https://example.com",
  "maxDepth": 3,              // Optional: 1-5, default 3
  "respectRobotsTxt": true    // Optional: default false
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "url": "https://example.com",
    "pages": [
      {
        "url": "https://example.com",
        "title": "Home",
        "depth": 0
      },
      {
        "url": "https://example.com/about",
        "title": "About Us",
        "depth": 1
      }
    ],
    "totalPages": 25
  }
}
```

### 5. Batch Scrape

Scrape multiple URLs in batch.

**Endpoint:** `POST /batch`

**Request Body:**
```json
{
  "urls": [
    "https://example.com/page1",
    "https://example.com/page2",
    "https://example.com/page3"
  ],
  "options": {
    "extractionMode": "machine-view",
    "timeout": 30000
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "jobId": "batch-1234567890-abc123",
    "status": "pending",
    "urlCount": 3,
    "message": "Batch job created successfully"
  }
}
```

### 6. Discover ARW Resources

Discover ARW resources (llms.txt, robots.txt, sitemap.xml).

**Endpoint:** `POST /discover`

**Request Body:**
```json
{
  "url": "https://example.com"
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "llmsTxt": {
      "url": "https://example.com/llms.txt",
      "content": "...",
      "sections": [
        {
          "title": "About",
          "content": "...",
          "links": ["https://example.com/about"]
        }
      ],
      "parsedAt": "2024-01-01T00:00:00.000Z"
    },
    "robotsTxt": {
      "url": "https://example.com/robots.txt",
      "content": "...",
      "rules": [
        {
          "userAgent": "*",
          "allow": ["/"],
          "disallow": ["/admin"]
        }
      ],
      "sitemaps": ["https://example.com/sitemap.xml"],
      "crawlDelay": 1000,
      "parsedAt": "2024-01-01T00:00:00.000Z"
    },
    "sitemap": {
      "url": "https://example.com/sitemap.xml",
      "urls": [
        {
          "loc": "https://example.com",
          "lastmod": "2024-01-01",
          "changefreq": "daily",
          "priority": 1.0
        }
      ],
      "parsedAt": "2024-01-01T00:00:00.000Z"
    }
  }
}
```

### 7. Get Service Statistics

Get crawler service statistics.

**Endpoint:** `GET /stats`

**Response:**
```json
{
  "success": true,
  "data": {
    "queue": {
      "crawl": {
        "waiting": 5,
        "active": 2,
        "completed": 100,
        "failed": 3
      },
      "batch": {
        "waiting": 1,
        "active": 0,
        "completed": 10,
        "failed": 0
      }
    },
    "cache": {
      "hits": 1500,
      "misses": 200,
      "keys": 150,
      "ksize": 150,
      "vsize": 1024000
    }
  }
}
```

### 8. Clear Cache

Clear the crawler cache.

**Endpoint:** `DELETE /cache`

**Response:**
```json
{
  "success": true,
  "message": "Cache cleared successfully"
}
```

### 9. Health Check

Check service health.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "ok",
  "timestamp": "2024-01-01T00:00:00.000Z"
}
```

## Error Responses

All errors follow this format:

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {}  // Optional additional details
  }
}
```

### Error Codes

- `VALIDATION_ERROR` (400) - Invalid request data
- `NOT_FOUND` (404) - Resource not found
- `RATE_LIMIT_ERROR` (429) - Rate limit exceeded
- `NETWORK_ERROR` (503) - Network/fetch error
- `INTERNAL_ERROR` (500) - Unexpected server error
- `JOB_NOT_FOUND` (404) - Crawl job not found

## Rate Limiting

- **Window:** 60 seconds (configurable)
- **Max Requests:** 100 per window (configurable)

When rate limited, you'll receive a 429 response with:
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests, please try again later"
  }
}
```

## Best Practices

1. **Use Batch Operations:** For multiple URLs, use `/batch` instead of individual `/scrape` calls
2. **Check robots.txt:** Always set `respectRobotsTxt: true` for public crawling
3. **Set Reasonable Limits:** Don't set `maxDepth` or `maxPages` too high
4. **Cache Results:** The service caches results automatically, but avoid unnecessary requests
5. **Monitor Job Status:** Poll `/crawl/:id` to check job progress
6. **Handle Rate Limits:** Implement exponential backoff when rate limited

## Examples

### Example 1: Simple Scrape

```bash
curl -X POST http://localhost:3000/api/v1/scrape \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

### Example 2: Machine View Extraction

```bash
curl -X POST http://localhost:3000/api/v1/scrape \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "extractionMode": "machine-view"
  }'
```

### Example 3: Start Crawl with Sitemap

```bash
curl -X POST http://localhost:3000/api/v1/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://example.com",
    "maxDepth": 2,
    "maxPages": 50,
    "respectRobotsTxt": true,
    "followSitemap": true
  }'
```

### Example 4: Check Job Status

```bash
curl http://localhost:3000/api/v1/crawl/crawl-1234567890-abc123
```

### Example 5: Discover ARW Resources

```bash
curl -X POST http://localhost:3000/api/v1/discover \
  -H "Content-Type: application/json" \
  -d '{"url": "https://example.com"}'
```

## TypeScript Types

```typescript
import {
  CrawlOptions,
  ScrapeOptions,
  CrawlResult,
  MachineView,
  ARWDiscoveryResult
} from '@agent-ready-web/crawler-service';

// Use types in your application
const options: CrawlOptions = {
  url: 'https://example.com',
  maxDepth: 3,
  respectRobotsTxt: true
};
```

## Support

For issues and questions:
- GitHub Issues: https://github.com/agent-ready-web/agent-ready-web/issues
- Documentation: https://agent-ready-web.dev
