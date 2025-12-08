import dotenv from 'dotenv';
import { createApp } from './api/index.js';
import { getServices } from './services/index.js';
import { createLogger } from './utils/logger.js';

// Load environment variables
dotenv.config();

const logger = createLogger('Server');

const PORT = parseInt(process.env.PORT || '3000', 10);
const HOST = process.env.HOST || '0.0.0.0';

/**
 * Start the EmotiStream API server
 */
async function start() {
  try {
    // Initialize services (loads TMDB content or mock data)
    logger.info('EmotiStream API Server Starting...');
    const services = getServices();
    await services.initialize();

    const contentSource = services.isUsingTMDB() ? 'TMDB (real movies/TV)' : 'Mock data';

    const app = createApp();

    const server = app.listen(PORT, HOST, () => {
      logger.info('EmotiStream API Server Ready', {
        host: HOST,
        port: PORT,
        healthCheck: `http://${HOST}:${PORT}/health`,
        apiBase: `http://${HOST}:${PORT}/api/v1`,
        contentSource,
      });
      logger.info('Available endpoints: POST /api/v1/emotion/analyze, POST /api/v1/recommend, POST /api/v1/feedback, GET /api/v1/progress/:userId');
    });

    // Graceful shutdown
    const shutdown = async (signal: string) => {
      logger.info(`Received ${signal}. Starting graceful shutdown...`);

      server.close(() => {
        logger.info('HTTP server closed. Goodbye!');
        process.exit(0);
      });

      // Force shutdown after 10 seconds
      setTimeout(() => {
        logger.error('Forced shutdown after timeout');
        process.exit(1);
      }, 10000);
    };

    process.on('SIGTERM', () => shutdown('SIGTERM'));
    process.on('SIGINT', () => shutdown('SIGINT'));

  } catch (error) {
    logger.error('Failed to start server', error);
    process.exit(1);
  }
}

// Start server
start();
