import rateLimit from 'express-rate-limit';

/**
 * General API rate limiter
 * 100 requests per minute per IP
 */
export const rateLimiter = rateLimit({
  windowMs: 60 * 1000, // 1 minute
  max: 100,
  standardHeaders: true,
  legacyHeaders: false,
  message: {
    success: false,
    data: null,
    error: {
      code: 'RATE_LIMIT_EXCEEDED',
      message: 'Too many requests. Please try again later.',
      details: {
        limit: 100,
        window: '1 minute',
      },
    },
    timestamp: new Date().toISOString(),
  },
});

/**
 * Emotion detection rate limiter
 * 30 requests per minute (more expensive)
 */
export const emotionRateLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 30,
  standardHeaders: true,
  legacyHeaders: false,
  skipSuccessfulRequests: false,
  message: {
    success: false,
    data: null,
    error: {
      code: 'EMOTION_RATE_LIMIT',
      message: 'Emotion detection rate limit exceeded.',
      details: {
        limit: 30,
        window: '1 minute',
      },
    },
    timestamp: new Date().toISOString(),
  },
});

/**
 * Recommendation rate limiter
 * 60 requests per minute
 */
export const recommendRateLimiter = rateLimit({
  windowMs: 60 * 1000,
  max: 60,
  standardHeaders: true,
  legacyHeaders: false,
});
