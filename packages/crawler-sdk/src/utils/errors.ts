/**
 * Custom error classes for the SDK
 */

export class CrawlerError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public details?: any
  ) {
    super(message);
    this.name = 'CrawlerError';
    Error.captureStackTrace(this, this.constructor);
  }
}

export class AuthenticationError extends CrawlerError {
  constructor(message = 'Authentication failed') {
    super(message, 401);
    this.name = 'AuthenticationError';
  }
}

export class RateLimitError extends CrawlerError {
  constructor(message = 'Rate limit exceeded') {
    super(message, 429);
    this.name = 'RateLimitError';
  }
}

export class ValidationError extends CrawlerError {
  constructor(message: string, details?: any) {
    super(message, 400, details);
    this.name = 'ValidationError';
  }
}

export class NetworkError extends CrawlerError {
  constructor(message: string, public originalError?: Error) {
    super(message, 0);
    this.name = 'NetworkError';
  }
}

export class TimeoutError extends CrawlerError {
  constructor(message = 'Request timed out') {
    super(message, 408);
    this.name = 'TimeoutError';
  }
}

export class ServerError extends CrawlerError {
  constructor(message = 'Internal server error') {
    super(message, 500);
    this.name = 'ServerError';
  }
}

/**
 * Maps HTTP status codes to appropriate error classes
 */
export function createErrorFromResponse(
  statusCode: number,
  message: string,
  details?: any
): CrawlerError {
  switch (statusCode) {
    case 401:
    case 403:
      return new AuthenticationError(message);
    case 429:
      return new RateLimitError(message);
    case 400:
      return new ValidationError(message, details);
    case 408:
      return new TimeoutError(message);
    case 500:
    case 502:
    case 503:
    case 504:
      return new ServerError(message);
    default:
      return new CrawlerError(message, statusCode, details);
  }
}
