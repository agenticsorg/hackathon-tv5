/**
 * Rate Limiting Middleware
 *
 * Simple in-memory rate limiting for API endpoints.
 * For production, use Redis for distributed rate limiting.
 */

// Rate limit configuration per endpoint
export interface RateLimitConfig {
  windowMs: number;     // Time window in milliseconds
  maxRequests: number;  // Maximum requests per window
  keyPrefix?: string;   // Prefix for rate limit key
}

// Default rate limit configurations (matching ARW manifest)
export const RATE_LIMITS: Record<string, RateLimitConfig> = {
  'search': {
    windowMs: 60 * 1000,    // 1 minute
    maxRequests: 100,       // 100 requests/min
    keyPrefix: 'search',
  },
  'recommendations': {
    windowMs: 60 * 1000,
    maxRequests: 200,
    keyPrefix: 'rec',
  },
  'discover': {
    windowMs: 60 * 1000,
    maxRequests: 1000,
    keyPrefix: 'discover',
  },
  'default': {
    windowMs: 60 * 1000,
    maxRequests: 60,
    keyPrefix: 'default',
  },
};

// Rate limit entry
interface RateLimitEntry {
  count: number;
  resetTime: number;
}

// In-memory store for rate limiting (replace with Redis for production)
const rateLimitStore = new Map<string, RateLimitEntry>();

// Cleanup old entries periodically
const CLEANUP_INTERVAL_MS = 60 * 1000; // 1 minute
let lastCleanup = Date.now();

function cleanupExpiredEntries() {
  const now = Date.now();
  if (now - lastCleanup < CLEANUP_INTERVAL_MS) return;

  for (const [key, entry] of rateLimitStore.entries()) {
    if (now >= entry.resetTime) {
      rateLimitStore.delete(key);
    }
  }
  lastCleanup = now;
}

/**
 * Get client identifier from request
 * Uses IP address or API key
 */
export function getClientIdentifier(request: Request): string {
  // Try to get API key first
  const apiKey = request.headers.get('x-api-key');
  if (apiKey) {
    return `api:${apiKey}`;
  }

  // Fall back to IP address
  const forwarded = request.headers.get('x-forwarded-for');
  const realIp = request.headers.get('x-real-ip');
  const ip = forwarded?.split(',')[0].trim() || realIp || 'unknown';

  return `ip:${ip}`;
}

/**
 * Check rate limit for a client
 * Returns { allowed, remaining, resetTime, retryAfter }
 */
export function checkRateLimit(
  clientId: string,
  endpoint: string = 'default'
): {
  allowed: boolean;
  remaining: number;
  resetTime: number;
  retryAfter: number | null;
} {
  // Cleanup old entries periodically
  cleanupExpiredEntries();

  const config = RATE_LIMITS[endpoint] || RATE_LIMITS.default;
  const key = `${config.keyPrefix}:${clientId}`;
  const now = Date.now();

  let entry = rateLimitStore.get(key);

  // Create new entry if doesn't exist or expired
  if (!entry || now >= entry.resetTime) {
    entry = {
      count: 0,
      resetTime: now + config.windowMs,
    };
  }

  // Check if over limit
  if (entry.count >= config.maxRequests) {
    const retryAfter = Math.ceil((entry.resetTime - now) / 1000);
    return {
      allowed: false,
      remaining: 0,
      resetTime: entry.resetTime,
      retryAfter,
    };
  }

  // Increment count
  entry.count++;
  rateLimitStore.set(key, entry);

  return {
    allowed: true,
    remaining: config.maxRequests - entry.count,
    resetTime: entry.resetTime,
    retryAfter: null,
  };
}

/**
 * Apply rate limiting to API response
 * Returns 429 if rate limited, null if allowed
 */
export function applyRateLimit(
  request: Request,
  endpoint: string = 'default'
): Response | null {
  const clientId = getClientIdentifier(request);
  const result = checkRateLimit(clientId, endpoint);

  if (!result.allowed) {
    return new Response(
      JSON.stringify({
        success: false,
        error: 'Too many requests',
        retryAfter: result.retryAfter,
      }),
      {
        status: 429,
        headers: {
          'Content-Type': 'application/json',
          'X-RateLimit-Limit': String(RATE_LIMITS[endpoint]?.maxRequests || 60),
          'X-RateLimit-Remaining': '0',
          'X-RateLimit-Reset': String(Math.ceil(result.resetTime / 1000)),
          'Retry-After': String(result.retryAfter),
        },
      }
    );
  }

  return null;
}

/**
 * Get rate limit headers for successful responses
 */
export function getRateLimitHeaders(
  request: Request,
  endpoint: string = 'default'
): Record<string, string> {
  const clientId = getClientIdentifier(request);
  const key = `${RATE_LIMITS[endpoint]?.keyPrefix || 'default'}:${clientId}`;
  const entry = rateLimitStore.get(key);
  const config = RATE_LIMITS[endpoint] || RATE_LIMITS.default;

  if (entry) {
    return {
      'X-RateLimit-Limit': String(config.maxRequests),
      'X-RateLimit-Remaining': String(Math.max(0, config.maxRequests - entry.count)),
      'X-RateLimit-Reset': String(Math.ceil(entry.resetTime / 1000)),
    };
  }

  return {
    'X-RateLimit-Limit': String(config.maxRequests),
    'X-RateLimit-Remaining': String(config.maxRequests),
  };
}

/**
 * Reset rate limit for a client (for testing or admin use)
 */
export function resetRateLimit(clientId: string, endpoint: string = 'default'): void {
  const config = RATE_LIMITS[endpoint] || RATE_LIMITS.default;
  const key = `${config.keyPrefix}:${clientId}`;
  rateLimitStore.delete(key);
}

/**
 * Get current rate limit stats (for monitoring)
 */
export function getRateLimitStats(): {
  totalClients: number;
  entriesByEndpoint: Record<string, number>;
} {
  const entriesByEndpoint: Record<string, number> = {};

  for (const key of rateLimitStore.keys()) {
    const prefix = key.split(':')[0];
    entriesByEndpoint[prefix] = (entriesByEndpoint[prefix] || 0) + 1;
  }

  return {
    totalClients: rateLimitStore.size,
    entriesByEndpoint,
  };
}
