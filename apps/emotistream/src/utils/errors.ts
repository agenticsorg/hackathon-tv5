/**
 * EmotiStream Custom Error Classes
 *
 * Provides type-safe error handling across the application.
 */

/**
 * Base application error
 */
export class EmotiStreamError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly statusCode: number = 500,
    public readonly details?: unknown
  ) {
    super(message);
    this.name = 'EmotiStreamError';
    Error.captureStackTrace(this, this.constructor);
  }

  toJSON() {
    return {
      error: this.code,
      message: this.message,
      details: this.details,
      timestamp: Date.now(),
    };
  }
}

/**
 * Validation error (400)
 */
export class ValidationError extends EmotiStreamError {
  constructor(message: string, details?: unknown) {
    super(message, 'VALIDATION_ERROR', 400, details);
    this.name = 'ValidationError';
  }
}

/**
 * Not found error (404)
 */
export class NotFoundError extends EmotiStreamError {
  constructor(resource: string, identifier?: string) {
    const message = identifier
      ? `${resource} with identifier '${identifier}' not found`
      : `${resource} not found`;
    super(message, 'NOT_FOUND', 404);
    this.name = 'NotFoundError';
  }
}

/**
 * Configuration error (500)
 */
export class ConfigurationError extends EmotiStreamError {
  constructor(message: string, details?: unknown) {
    super(message, 'CONFIGURATION_ERROR', 500, details);
    this.name = 'ConfigurationError';
  }
}

/**
 * Gemini API error (502)
 */
export class GeminiAPIError extends EmotiStreamError {
  constructor(message: string, details?: unknown) {
    super(message, 'GEMINI_API_ERROR', 502, details);
    this.name = 'GeminiAPIError';
  }
}

/**
 * Database error (500)
 */
export class DatabaseError extends EmotiStreamError {
  constructor(message: string, details?: unknown) {
    super(message, 'DATABASE_ERROR', 500, details);
    this.name = 'DatabaseError';
  }
}

/**
 * Emotion detection error (422)
 */
export class EmotionDetectionError extends EmotiStreamError {
  constructor(message: string, details?: unknown) {
    super(message, 'EMOTION_DETECTION_ERROR', 422, details);
    this.name = 'EmotionDetectionError';
  }
}

/**
 * Content profiling error (422)
 */
export class ContentProfilingError extends EmotiStreamError {
  constructor(message: string, details?: unknown) {
    super(message, 'CONTENT_PROFILING_ERROR', 422, details);
    this.name = 'ContentProfilingError';
  }
}

/**
 * Policy error (500)
 */
export class PolicyError extends EmotiStreamError {
  constructor(message: string, details?: unknown) {
    super(message, 'POLICY_ERROR', 500, details);
    this.name = 'PolicyError';
  }
}

/**
 * Rate limit error (429)
 */
export class RateLimitError extends EmotiStreamError {
  constructor(retryAfter: number) {
    super(
      `Rate limit exceeded. Retry after ${retryAfter} seconds.`,
      'RATE_LIMIT_EXCEEDED',
      429,
      { retryAfter }
    );
    this.name = 'RateLimitError';
  }
}

/**
 * Type guard for EmotiStreamError
 */
export const isEmotiStreamError = (error: unknown): error is EmotiStreamError => {
  return error instanceof EmotiStreamError;
};

/**
 * Error handler utility
 */
export const handleError = (error: unknown): EmotiStreamError => {
  if (isEmotiStreamError(error)) {
    return error;
  }

  if (error instanceof Error) {
    return new EmotiStreamError(
      error.message,
      'UNKNOWN_ERROR',
      500,
      { originalError: error.name }
    );
  }

  return new EmotiStreamError(
    'An unknown error occurred',
    'UNKNOWN_ERROR',
    500,
    { originalError: String(error) }
  );
};

/**
 * Async error wrapper
 */
export const asyncHandler = <T extends unknown[], R>(
  fn: (...args: T) => Promise<R>
) => {
  return async (...args: T): Promise<R> => {
    try {
      return await fn(...args);
    } catch (error) {
      throw handleError(error);
    }
  };
};
