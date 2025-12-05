import { Request, Response, NextFunction } from 'express';
import { createChildLogger } from '../../utils/logger.js';

const logger = createChildLogger('http');

export function requestLogger(req: Request, res: Response, next: NextFunction): void {
  const start = Date.now();

  res.on('finish', () => {
    const duration = Date.now() - start;

    logger.info(
      {
        method: req.method,
        url: req.url,
        statusCode: res.statusCode,
        duration,
        userAgent: req.get('user-agent')
      },
      'HTTP request'
    );
  });

  next();
}
