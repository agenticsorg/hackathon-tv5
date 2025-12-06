import dotenv from 'dotenv';
import { createApp } from './api';

// Load environment variables
dotenv.config();

const PORT = parseInt(process.env.PORT || '3000', 10);
const HOST = process.env.HOST || '0.0.0.0';

/**
 * Start the EmotiStream API server
 */
async function start() {
  try {
    const app = createApp();

    const server = app.listen(PORT, HOST, () => {
      console.log('\n🎬 EmotiStream API Server');
      console.log('═'.repeat(50));
      console.log(`🚀 Server running at http://${HOST}:${PORT}`);
      console.log(`📊 Health check: http://${HOST}:${PORT}/health`);
      console.log(`🎯 API base: http://${HOST}:${PORT}/api/v1`);
      console.log('═'.repeat(50));
      console.log('\n📍 Available endpoints:');
      console.log('  POST /api/v1/emotion/analyze       - Analyze emotional state');
      console.log('  GET  /api/v1/emotion/history/:id   - Get emotion history');
      console.log('  POST /api/v1/recommend             - Get recommendations');
      console.log('  GET  /api/v1/recommend/history/:id - Get recommendation history');
      console.log('  POST /api/v1/feedback              - Submit feedback');
      console.log('  GET  /api/v1/feedback/progress/:id - Get learning progress');
      console.log('  GET  /api/v1/feedback/experiences/:id - Get experiences');
      console.log('\n✨ Press Ctrl+C to stop\n');
    });

    // Graceful shutdown
    const shutdown = async (signal: string) => {
      console.log(`\n\n📡 Received ${signal}. Starting graceful shutdown...`);

      server.close(() => {
        console.log('✅ HTTP server closed');
        console.log('👋 Goodbye!\n');
        process.exit(0);
      });

      // Force shutdown after 10 seconds
      setTimeout(() => {
        console.error('⚠️  Forced shutdown after timeout');
        process.exit(1);
      }, 10000);
    };

    process.on('SIGTERM', () => shutdown('SIGTERM'));
    process.on('SIGINT', () => shutdown('SIGINT'));

  } catch (error) {
    console.error('❌ Failed to start server:', error);
    process.exit(1);
  }
}

// Start server
start();
