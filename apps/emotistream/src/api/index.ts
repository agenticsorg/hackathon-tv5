import express, { Express } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import { errorHandler } from './middleware/error-handler.js';
import { requestLogger } from './middleware/logger.js';
import { rateLimiter } from './middleware/rate-limiter.js';
import emotionRoutes from './routes/emotion.js';
import recommendRoutes from './routes/recommend.js';
import feedbackRoutes from './routes/feedback.js';
import progressRoutes from './routes/progress.js';
import { createAuthRouter } from './routes/auth.js';
import { getServices } from '../services/index.js';

/**
 * Create and configure Express application
 */
export function createApp(): Express {
  const app = express();

  // Security middleware
  app.use(helmet());

  // CORS configuration - allow Cloud Run frontend and local development
  const defaultOrigins = [
    'http://localhost:3000',
    'http://localhost:5173',
    'https://emotistream-web-727540765498.europe-west1.run.app',
    'https://emotistream-web-530247135934.us-central1.run.app',
  ];
  const allowedOrigins = process.env.ALLOWED_ORIGINS?.split(',') || defaultOrigins;

  app.use(cors({
    origin: allowedOrigins,
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

  // Get services for auth routes
  const services = getServices();
  const authRouter = createAuthRouter(
    services.jwtService,
    services.passwordService,
    services.userStore
  );

  // API routes
  app.use('/api/v1/auth', authRouter);
  app.use('/api/v1/emotion', emotionRoutes);
  app.use('/api/v1/recommend', recommendRoutes);
  app.use('/api/v1/feedback', feedbackRoutes);
  app.use('/api/v1/progress', progressRoutes);

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
