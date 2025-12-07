/**
 * Request Timeout and Retry Utility
 * Provides robust HTTP request handling with timeout, exponential backoff, and fallback strategies
 */

export interface RetryConfig {
  /** Request timeout in milliseconds */
  timeout: number;
  /** Maximum number of retry attempts */
  maxRetries: number;
  /** Base delay for exponential backoff in milliseconds */
  baseDelay: number;
}

export interface RetryError extends Error {
  /** Original error that caused the retry failure */
  cause?: Error;
  /** Number of retry attempts made */
  attempts?: number;
  /** Suggested time to retry after (in seconds) */
  retryAfter?: number;
}

export interface CacheEntry<T> {
  data: T;
  timestamp: number;
  expiresAt: number;
}

// Default configuration
const DEFAULT_CONFIG: RetryConfig = {
  timeout: 100, // 100ms
  maxRetries: 2,
  baseDelay: 50, // 50ms
};

// Simple in-memory cache for stale data fallback
const cache = new Map<string, CacheEntry<any>>();

/**
 * Set cache entry with TTL
 */
export function setCacheEntry<T>(key: string, data: T, ttlSeconds: number = 300): void {
  const now = Date.now();
  cache.set(key, {
    data,
    timestamp: now,
    expiresAt: now + ttlSeconds * 1000,
  });
}

/**
 * Get cache entry (returns even if stale)
 */
export function getCacheEntry<T>(key: string): T | null {
  const entry = cache.get(key);
  if (!entry) return null;
  return entry.data;
}

/**
 * Check if cache entry is fresh
 */
export function isCacheFresh(key: string): boolean {
  const entry = cache.get(key);
  if (!entry) return false;
  return Date.now() < entry.expiresAt;
}

/**
 * Calculate delay with exponential backoff and jitter
 */
function calculateDelay(baseDelay: number, attempt: number): number {
  // Exponential backoff: baseDelay * 2^attempt
  const exponentialDelay = baseDelay * Math.pow(2, attempt);

  // Add random jitter (0-10ms) to prevent thundering herd
  const jitter = Math.random() * 10;

  return exponentialDelay + jitter;
}

/**
 * Create a timeout error with retry hint
 */
function createTimeoutError(attempt: number, maxRetries: number): RetryError {
  const error = new Error('Request timeout') as RetryError;
  error.name = 'TimeoutError';
  error.attempts = attempt + 1;
  error.retryAfter = 5; // Suggest retry after 5 seconds
  return error;
}

/**
 * Create a max retries error
 */
function createMaxRetriesError(lastError: Error, attempts: number): RetryError {
  const error = new Error(
    `Request failed after ${attempts} attempts: ${lastError.message}`
  ) as RetryError;
  error.name = 'MaxRetriesError';
  error.cause = lastError;
  error.attempts = attempts;
  error.retryAfter = 30; // Suggest retry after 30 seconds for exhausted retries
  return error;
}

/**
 * Log retry attempt with structured logging
 */
function logRetryAttempt(
  url: string,
  attempt: number,
  maxRetries: number,
  delay: number,
  error?: Error
): void {
  const logData = {
    url,
    attempt: attempt + 1,
    maxRetries: maxRetries + 1,
    delayMs: Math.round(delay),
    error: error?.message,
    timestamp: new Date().toISOString(),
  };

  console.warn('[Retry]', JSON.stringify(logData));
}

/**
 * Fetch with timeout, retry, and fallback support
 *
 * @param url - The URL to fetch
 * @param options - Standard fetch options
 * @param config - Retry configuration (optional)
 * @returns Promise resolving to Response
 *
 * @example
 * ```typescript
 * const response = await fetchWithRetry(
 *   'http://api.example.com/data',
 *   { method: 'GET' },
 *   { timeout: 200, maxRetries: 3 }
 * );
 * ```
 */
export async function fetchWithRetry(
  url: string,
  options: RequestInit = {},
  config?: Partial<RetryConfig>
): Promise<Response> {
  const mergedConfig: RetryConfig = { ...DEFAULT_CONFIG, ...config };
  const { timeout, maxRetries, baseDelay } = mergedConfig;

  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      // Create AbortController for timeout
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), timeout);

      try {
        // Merge abort signal with existing signal if provided
        const signal = options.signal
          ? createCombinedSignal(controller.signal, options.signal)
          : controller.signal;

        const response = await fetch(url, {
          ...options,
          signal,
        });

        clearTimeout(timeoutId);

        // Success - cache the response if GET request
        if (options.method === 'GET' || !options.method) {
          // Clone response to cache it
          const clonedResponse = response.clone();
          clonedResponse.json().then(data => {
            setCacheEntry(url, data);
          }).catch(() => {
            // Ignore cache errors
          });
        }

        return response;
      } catch (error) {
        clearTimeout(timeoutId);

        // Check if it's an abort error (timeout)
        if (error instanceof Error && error.name === 'AbortError') {
          lastError = createTimeoutError(attempt, maxRetries);
        } else {
          lastError = error instanceof Error ? error : new Error('Unknown error');
        }

        // Don't retry if it's the last attempt
        if (attempt === maxRetries) {
          throw lastError;
        }

        // Calculate delay with exponential backoff
        const delay = calculateDelay(baseDelay, attempt);

        // Log retry attempt
        logRetryAttempt(url, attempt, maxRetries, delay, lastError);

        // Wait before retrying
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    } catch (error) {
      // This catch handles errors from the retry logic itself
      lastError = error instanceof Error ? error : new Error('Unknown error');

      if (attempt === maxRetries) {
        break; // Exit retry loop
      }
    }
  }

  // All retries exhausted - try fallback strategies

  // Strategy 1: Check for cached stale data
  const cachedData = getCacheEntry(url);
  if (cachedData) {
    console.warn('[Retry] Returning stale cached data as fallback', {
      url,
      cacheAge: Date.now() - (cache.get(url)?.timestamp || 0),
    });

    // Create a mock Response with cached data
    return new Response(JSON.stringify(cachedData), {
      status: 200,
      statusText: 'OK (Cached)',
      headers: {
        'Content-Type': 'application/json',
        'X-Cache-Status': 'STALE',
      },
    });
  }

  // Strategy 2: Return graceful error with retry hint
  throw createMaxRetriesError(
    lastError || new Error('Unknown error'),
    maxRetries + 1
  );
}

/**
 * Create a combined abort signal from multiple signals
 */
function createCombinedSignal(signal1: AbortSignal, signal2: AbortSignal): AbortSignal {
  const controller = new AbortController();

  const abortHandler = () => controller.abort();

  if (signal1.aborted || signal2.aborted) {
    controller.abort();
  } else {
    signal1.addEventListener('abort', abortHandler);
    signal2.addEventListener('abort', abortHandler);
  }

  return controller.signal;
}

/**
 * Helper to clear cache (useful for testing)
 */
export function clearCache(): void {
  cache.clear();
}

/**
 * Get cache size (useful for monitoring)
 */
export function getCacheSize(): number {
  return cache.size;
}
