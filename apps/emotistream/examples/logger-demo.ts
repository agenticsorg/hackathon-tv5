/**
 * Logger Demonstration
 *
 * This file demonstrates the environment-based logging behavior.
 * Run with different LOG_LEVEL values to see filtering in action.
 *
 * Usage:
 *   LOG_LEVEL=debug ts-node examples/logger-demo.ts
 *   LOG_LEVEL=info ts-node examples/logger-demo.ts
 *   LOG_LEVEL=warn ts-node examples/logger-demo.ts
 *   LOG_LEVEL=error ts-node examples/logger-demo.ts
 */

import { logger, createLogger, LogLevel } from '../src/utils/logger';
import { CONFIG } from '../src/utils/config';

console.log('===============================================');
console.log('EmotiStream Logger Demonstration');
console.log('===============================================');
console.log(`Current Environment: ${process.env.NODE_ENV || 'development'}`);
console.log(`Current Log Level:   ${CONFIG.logging.level}`);
console.log(`Pretty Printing:     ${CONFIG.logging.pretty}`);
console.log('===============================================\n');

// 1. Basic logging with default logger
console.log('1. Basic Logging (default logger):');
console.log('-----------------------------------');
logger.debug('This is a DEBUG message - detailed diagnostic info');
logger.info('This is an INFO message - general information');
logger.warn('This is a WARN message - potential issues');
logger.error('This is an ERROR message - something went wrong');
console.log('');

// 2. Logging with data
console.log('2. Structured Logging (with data):');
console.log('-----------------------------------');
logger.info('User action', {
  userId: 'user123',
  action: 'login',
  timestamp: new Date().toISOString(),
});

logger.debug('State processing', {
  stateHash: 'abc123',
  valence: 0.7,
  arousal: 0.5,
  candidateCount: 10,
});
console.log('');

// 3. Module-specific logger
console.log('3. Module-Specific Logger:');
console.log('-----------------------------------');
const engineLogger = createLogger('RecommendationEngine');
engineLogger.info('Engine initialized');
engineLogger.debug('Processing recommendations', { count: 5 });
engineLogger.warn('High exploration rate', { epsilon: 0.5 });
console.log('');

// 4. Child logger with nested context
console.log('4. Child Logger (nested context):');
console.log('-----------------------------------');
const parentLogger = createLogger('API');
const authLogger = parentLogger.child('Auth');
const dbLogger = parentLogger.child('Database');

authLogger.info('User authenticated', { userId: 'user123' });
dbLogger.debug('Query executed', { table: 'users', duration: 45 });
console.log('');

// 5. Error logging
console.log('5. Error Logging:');
console.log('-----------------------------------');
try {
  throw new Error('Example database connection error');
} catch (error) {
  logger.error('Database operation failed', error, {
    operation: 'connect',
    host: 'localhost',
    port: 5432,
  });
}
console.log('');

// 6. Performance logging
console.log('6. Performance Logging:');
console.log('-----------------------------------');
const start = Date.now();
// Simulate some work
let sum = 0;
for (let i = 0; i < 1000000; i++) {
  sum += i;
}
const duration = Date.now() - start;

logger.debug('Operation completed', {
  operation: 'calculation',
  duration,
  result: sum,
});
console.log('');

// 7. Log level behavior
console.log('7. Log Level Filtering Demonstration:');
console.log('-----------------------------------');
console.log(`Current level: ${CONFIG.logging.level}\n`);

const levels = ['debug', 'info', 'warn', 'error'] as const;
const currentLevelNum = LogLevel[CONFIG.logging.level.toUpperCase() as keyof typeof LogLevel];

levels.forEach((level) => {
  const levelNum = LogLevel[level.toUpperCase() as keyof typeof LogLevel];
  const willLog = levelNum >= currentLevelNum;
  console.log(`${level.toUpperCase().padEnd(6)} - ${willLog ? '✅ Will log' : '❌ Filtered'}`);
});
console.log('');

// 8. Best practices example
console.log('8. Best Practices Example:');
console.log('-----------------------------------');
const apiLogger = createLogger('API');

// Good: structured data
apiLogger.info('Request processed', {
  method: 'POST',
  path: '/api/recommendations',
  duration: 123,
  statusCode: 200,
});

// Good: error with context
try {
  throw new Error('Validation failed');
} catch (error) {
  apiLogger.error('Request validation failed', error, {
    method: 'POST',
    path: '/api/recommendations',
    body: { userId: 'user123' },
  });
}

// Good: debug with expensive data (only processed if debug is active)
apiLogger.debug('Detailed state', {
  // This would only be computed if debug level is active
  expensiveData: { computed: true },
});
console.log('');

console.log('===============================================');
console.log('Summary:');
console.log('===============================================');
console.log('✅ Logger supports debug, info, warn, error levels');
console.log('✅ Production defaults to warn (only warnings and errors)');
console.log('✅ Development defaults to info');
console.log('✅ Structured logging with context and data');
console.log('✅ Module-specific and child loggers');
console.log('✅ Zero overhead for filtered logs');
console.log('');
console.log('Try different log levels:');
console.log('  LOG_LEVEL=debug ts-node examples/logger-demo.ts');
console.log('  LOG_LEVEL=info ts-node examples/logger-demo.ts');
console.log('  LOG_LEVEL=warn ts-node examples/logger-demo.ts');
console.log('  LOG_LEVEL=error ts-node examples/logger-demo.ts');
console.log('===============================================');
