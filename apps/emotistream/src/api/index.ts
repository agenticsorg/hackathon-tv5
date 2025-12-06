import express, { Express } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import { errorHandler } from './middleware/error-handler';
import { requestLogger } from './middleware/logger';
import { rateLimiter } from './middleware/rate-limiter';
import emotionRoutes from './routes/emotion';
import recommendRoutes from './routes/recommend';
import feedbackRoutes from './routes/feedback';

/**
 * Create and configure Express application
 */
export function createApp(): Express {
  const app = express();

  // Security middleware
  app.use(helmet());
  app.use(cors({
    origin: process.env.ALLOWED_ORIGINS?.split(',') || ['http://localhost:3000', 'http://localhost:5173'],
    credentials: true,
  }));

  // Body parsing middleware
  app.use(express.json({ limit: '10mb' }));
  app.use(express.urlencoded({ extended: true, limit: '10mb' }));

  // Compression
  app.use(compression());

  // Request logging
  app.use(requestLogger);

  // Rate limiting (applied to API routes)
  app.use('/api', rateLimiter);

  // Health check (before routes for fast response)
  app.get('/health', (req, res) => {
    res.json({
      status: 'ok',
      version: '1.0.0',
      timestamp: new Date().toISOString(),
    });
  });

  // API routes
  app.use('/api/v1/emotion', emotionRoutes);
  app.use('/api/v1/recommend', recommendRoutes);
  app.use('/api/v1/feedback', feedbackRoutes);

  // 404 handler
  app.use((req, res) => {
    res.status(404).json({
      success: false,
      error: {
        code: 'NOT_FOUND',
        message: `Route not found: ${req.method} ${req.path}`,
      },
      timestamp: new Date().toISOString(),
    });
  });

  // Global error handler (must be last)
  app.use(errorHandler);

  return app;
}

export default createApp();
