# MCP Server Utilities

## Retry Utility

Robust HTTP request handling with timeout, exponential backoff, and fallback strategies.

### Installation

```typescript
import { fetchWithRetry } from './retry.js';
```

### Basic Usage

```typescript
// Simple GET request with defaults
const response = await fetchWithRetry('https://api.example.com/data');
const data = await response.json();
```

### Advanced Usage

```typescript
// POST request with custom retry configuration
const response = await fetchWithRetry(
  'https://api.example.com/users',
  {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ name: 'John' }),
  },
  {
    timeout: 200,      // 200ms timeout
    maxRetries: 3,     // 3 retry attempts
    baseDelay: 100,    // 100ms base delay
  }
);
```

### Configuration

```typescript
interface RetryConfig {
  timeout: number;      // Request timeout in milliseconds (default: 100)
  maxRetries: number;   // Maximum retry attempts (default: 2)
  baseDelay: number;    // Base delay for exponential backoff in ms (default: 50)
}
```

### Default Configuration

```typescript
{
  timeout: 100,       // 100ms
  maxRetries: 2,      // 2 retries (3 total attempts)
  baseDelay: 50,      // 50ms base delay
}
```

### Retry Delays

With default `baseDelay: 50ms`:

| Attempt | Delay Range |
|---------|-------------|
| 1st     | 0ms (immediate) |
| 2nd     | 50-60ms |
| 3rd     | 100-110ms |
| 4th     | 200-210ms |

Formula: `baseDelay * 2^attempt + random(0-10ms)`

### Error Handling

```typescript
import { fetchWithRetry, type RetryError } from './retry.js';

try {
  const response = await fetchWithRetry(url);
} catch (error) {
  if (error instanceof Error && 'attempts' in error) {
    const retryError = error as RetryError;
    console.error(`Failed after ${retryError.attempts} attempts`);
    console.error(`Retry after ${retryError.retryAfter}s`);
    console.error(`Cause:`, retryError.cause);
  }
}
```

### Error Types

#### TimeoutError

Thrown when a single request exceeds the timeout duration.

```typescript
{
  name: 'TimeoutError',
  message: 'Request timeout',
  attempts: 1,
  retryAfter: 5  // Suggest retry after 5 seconds
}
```

#### MaxRetriesError

Thrown when all retry attempts are exhausted.

```typescript
{
  name: 'MaxRetriesError',
  message: 'Request failed after N attempts: ...',
  attempts: 3,
  retryAfter: 30,  // Suggest retry after 30 seconds
  cause: Error     // Original error that caused failure
}
```

### Cache Management

The retry utility includes built-in caching for GET requests.

#### Automatic Caching

```typescript
// GET requests are automatically cached (5 minute TTL)
const response = await fetchWithRetry('https://api.example.com/data');
```

#### Manual Cache Operations

```typescript
import { setCacheEntry, getCacheEntry, isCacheFresh, clearCache } from './retry.js';

// Set cache entry
setCacheEntry('my-key', { data: 'value' }, 300); // 300 second TTL

// Get cached data (returns null if not found)
const data = getCacheEntry('my-key');

// Check if cache is fresh
if (isCacheFresh('my-key')) {
  console.log('Cache is fresh');
}

// Clear all cache
clearCache();
```

### Fallback Behavior

When all retries fail, the utility attempts fallback strategies:

1. **Check local cache** for stale data
2. If found, return cached data with `X-Cache-Status: STALE` header
3. Otherwise, throw `MaxRetriesError`

```typescript
const response = await fetchWithRetry(url);

// Check if response is from stale cache
if (response.headers.get('X-Cache-Status') === 'STALE') {
  console.warn('Using stale cached data');
}
```

### Structured Logging

All retry attempts are logged with structured JSON:

```json
{
  "url": "https://api.example.com/data",
  "attempt": 2,
  "maxRetries": 3,
  "delayMs": 105,
  "error": "Network error",
  "timestamp": "2025-12-06T17:04:46.174Z"
}
```

Logs are written to `console.warn` with `[Retry]` prefix.

### AbortController Support

The utility respects existing AbortController signals:

```typescript
const controller = new AbortController();

// External abort will be respected
const promise = fetchWithRetry(url, { signal: controller.signal });

// Abort the request
controller.abort();
```

### Best Practices

#### 1. Use Appropriate Timeouts

```typescript
// Fast endpoint - short timeout
await fetchWithRetry(url, {}, { timeout: 50 });

// Slow endpoint - longer timeout
await fetchWithRetry(url, {}, { timeout: 500 });
```

#### 2. Adjust Retries Based on Criticality

```typescript
// Critical operation - more retries
await fetchWithRetry(url, {}, { maxRetries: 5 });

// Non-critical - fewer retries
await fetchWithRetry(url, {}, { maxRetries: 1 });
```

#### 3. Handle Stale Cache Gracefully

```typescript
const response = await fetchWithRetry(url);
const isStale = response.headers.get('X-Cache-Status') === 'STALE';

if (isStale) {
  // Show warning to user
  console.warn('Showing cached data, may be outdated');
}
```

#### 4. Respect Retry Hints

```typescript
try {
  await fetchWithRetry(url);
} catch (error) {
  const retryError = error as RetryError;

  // Schedule retry based on hint
  setTimeout(() => {
    // Retry request
  }, retryError.retryAfter * 1000);
}
```

### Performance Considerations

#### Worst-Case Latency

With default config (timeout: 100ms, maxRetries: 2, baseDelay: 50ms):

```
Attempt 1: 0-100ms (timeout)
Delay 1:   50-60ms
Attempt 2: 0-100ms (timeout)
Delay 2:   100-110ms
Attempt 3: 0-100ms (timeout)
Total:     ~250-470ms worst case
```

#### Cache Benefits

- **Reduced latency**: Cached responses return immediately
- **Reduced load**: Fewer requests to backend services
- **Better UX**: Stale data better than no data during outages

#### Jitter Benefits

Random jitter (0-10ms) prevents thundering herd:
- Multiple concurrent retries won't align
- Spreads load over time
- Reduces backend spike risk

### Testing

```typescript
import { fetchWithRetry, clearCache } from './retry.js';
import { describe, it, beforeEach, afterEach, vi } from 'vitest';

describe('my feature', () => {
  beforeEach(() => {
    clearCache();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('should handle retry', async () => {
    // Mock fetch to fail twice, succeed on third
    global.fetch = vi.fn()
      .mockRejectedValueOnce(new Error('Network error'))
      .mockRejectedValueOnce(new Error('Network error'))
      .mockResolvedValueOnce(new Response('OK'));

    const promise = fetchWithRetry(url);

    // Fast-forward timers
    await vi.runAllTimersAsync();

    const response = await promise;
    expect(response.status).toBe(200);
  });
});
```

### Examples

#### Example 1: Discovery Service Search

```typescript
const response = await fetchWithRetry(
  `${config.services.discovery}/api/v1/search/semantic`,
  {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ query: 'action movies' }),
  },
  {
    timeout: 100,
    maxRetries: 2,
    baseDelay: 50,
  }
);

if (!response.ok) {
  throw new Error(`Discovery service returned ${response.status}`);
}

const data = await response.json();
```

#### Example 2: User Service with Authentication

```typescript
const response = await fetchWithRetry(
  `${config.services.user}/api/v1/user/devices`,
  {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      'X-User-ID': userId,
    },
  },
  {
    timeout: 100,
    maxRetries: 2,
    baseDelay: 50,
  }
);

if (!response.ok) {
  throw new Error(`User service returned ${response.status}`);
}

const { devices } = await response.json();
```

#### Example 3: Conditional Retry

```typescript
async function fetchWithConditionalRetry(url: string) {
  try {
    return await fetchWithRetry(url);
  } catch (error) {
    const retryError = error as RetryError;

    // Only retry on specific errors
    if (retryError.cause?.message.includes('ECONNREFUSED')) {
      console.log('Service unavailable, retrying later');
      throw error;
    }

    // Don't retry on auth errors
    if (retryError.cause?.message.includes('401')) {
      throw new Error('Authentication required');
    }

    throw error;
  }
}
```

### API Reference

#### fetchWithRetry()

```typescript
async function fetchWithRetry(
  url: string,
  options?: RequestInit,
  config?: Partial<RetryConfig>
): Promise<Response>
```

Main function for making HTTP requests with retry logic.

#### setCacheEntry()

```typescript
function setCacheEntry<T>(
  key: string,
  data: T,
  ttlSeconds?: number
): void
```

Manually set a cache entry (default TTL: 300 seconds).

#### getCacheEntry()

```typescript
function getCacheEntry<T>(key: string): T | null
```

Get cached data (returns even if stale).

#### isCacheFresh()

```typescript
function isCacheFresh(key: string): boolean
```

Check if cache entry is still fresh.

#### clearCache()

```typescript
function clearCache(): void
```

Clear all cached entries.

#### getCacheSize()

```typescript
function getCacheSize(): number
```

Get number of entries in cache.

---

## Support

For issues or questions, please refer to:
- Implementation: `/workspaces/media-gateway/apps/mcp-server/src/utils/retry.ts`
- Tests: `/workspaces/media-gateway/apps/mcp-server/src/utils/retry.test.ts`
- Documentation: `/workspaces/media-gateway/docs/BATCH_003_TASK_003_IMPLEMENTATION.md`
