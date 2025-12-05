import { Request, Response, NextFunction } from 'express';
import { createChildLogger } from '../../utils/logger.js';

const logger = createChildLogger('error-handler');

export function errorHandler(
  error: Error,
  req: Request,
  res: Response,
  next: NextFunction
): void {
  logger.error(
    {
      error: error.message,
      stack: error.stack,
      method: req.method,
      url: req.url
    },
    'Unhandled error'
  );

  res.status(500).json({
    success: false,
    error: {
      code: 'INTERNAL_ERROR',
      message: 'An unexpected error occurred'
    }
  });
}
