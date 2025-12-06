/**
 * API Response Middleware
 *
 * Standardized response formatting for all API endpoints.
 */

/**
 * Standard API response wrapper
 */
export interface ApiResponseData<T = unknown> {
  success: boolean;
  data: T;
  timestamp: string;
  meta?: {
    version?: string;
    requestId?: string;
  };
}

/**
 * Create a standardized API response
 */
export function apiResponse<T>(data: T, meta?: { version?: string; requestId?: string }): ApiResponseData<T> {
  return {
    success: true,
    data,
    timestamp: new Date().toISOString(),
    meta: {
      version: '1.0.0',
      ...meta,
    },
  };
}

/**
 * Create an error response
 */
export function apiErrorResponse(
  message: string,
  code: string,
  details?: unknown
): { success: false; error: { message: string; code: string; details?: unknown }; timestamp: string } {
  return {
    success: false,
    error: {
      message,
      code,
      details,
    },
    timestamp: new Date().toISOString(),
  };
}
