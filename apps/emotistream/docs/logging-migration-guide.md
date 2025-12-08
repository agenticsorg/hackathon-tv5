# Logging Migration Guide

This guide explains how to migrate from direct `console.log()` calls to the environment-aware logger.

## Overview

The EmotiStream logger provides:
- **Environment-based log levels**: Production defaults to `warn`, development to `info`
- **Structured logging**: Consistent format with timestamps, context, and data
- **Performance**: Logs are filtered at runtime, reducing overhead in production
- **Type safety**: TypeScript support for all log methods

## Log Levels

| Level | Priority | When to Use | Production? |
|-------|----------|-------------|-------------|
| `debug` | 0 | Detailed diagnostic information | ❌ No |
| `info` | 1 | General informational messages | ⚠️ Dev only |
| `warn` | 2 | Warning messages, potential issues | ✅ Yes |
| `error` | 3 | Error messages, exceptions | ✅ Yes |

## Environment Configuration

**Development (default):**
```bash
NODE_ENV=development
LOG_LEVEL=info  # Shows: info, warn, error
```

**Production (recommended):**
```bash
NODE_ENV=production
LOG_LEVEL=warn  # Shows only: warn, error
```

**Debugging:**
```bash
LOG_LEVEL=debug  # Shows all logs
```

## Migration Examples

### Basic console.log → logger.info

**Before:**
```typescript
console.log('User logged in:', userId);
```

**After:**
```typescript
import { logger } from './utils/logger';

logger.info('User logged in', { userId });
```

### console.error → logger.error

**Before:**
```typescript
try {
  await riskyOperation();
} catch (error) {
  console.error('Operation failed:', error);
}
```

**After:**
```typescript
import { logger } from './utils/logger';

try {
  await riskyOperation();
} catch (error) {
  logger.error('Operation failed', error);
}
```

### Debug logging → logger.debug

**Before:**
```typescript
console.log('[DEBUG] Processing state:', state);
```

**After:**
```typescript
import { logger } from './utils/logger';

logger.debug('Processing state', { state });
```

### Module-specific logger

**Before:**
```typescript
console.log('[RecommendationEngine] Generated recommendations:', recs.length);
```

**After:**
```typescript
import { createLogger } from './utils/logger';

const logger = createLogger('RecommendationEngine');

logger.info('Generated recommendations', { count: recs.length });
```

### Nested contexts

**Before:**
```typescript
console.log('[RecommendationEngine:Ranker] Ranking candidates');
```

**After:**
```typescript
import { createLogger } from './utils/logger';

const engineLogger = createLogger('RecommendationEngine');
const rankerLogger = engineLogger.child('Ranker');

rankerLogger.debug('Ranking candidates');
```

## Common Patterns

### API Request Logging

```typescript
import { createLogger } from './utils/logger';

const logger = createLogger('API');

export const logRequest = (req: Request) => {
  logger.info('Incoming request', {
    method: req.method,
    path: req.path,
    userId: req.user?.id,
  });
};

export const logError = (req: Request, error: Error) => {
  logger.error('Request failed', error, {
    method: req.method,
    path: req.path,
    userId: req.user?.id,
  });
};
```

### Performance Logging

```typescript
import { createLogger } from './utils/logger';

const logger = createLogger('Performance');

const start = Date.now();
await expensiveOperation();
const duration = Date.now() - start;

logger.debug('Operation completed', { operation: 'expensiveOperation', duration });
```

### State Transitions

```typescript
import { createLogger } from './utils/logger';

const logger = createLogger('StateMachine');

logger.debug('State transition', {
  from: currentState,
  to: nextState,
  trigger: event,
});
```

### Database Operations

```typescript
import { createLogger } from './utils/logger';

const logger = createLogger('Database');

logger.debug('Executing query', { query: sql, params });

try {
  const result = await db.query(sql, params);
  logger.debug('Query completed', { rowCount: result.rows.length });
  return result;
} catch (error) {
  logger.error('Query failed', error, { query: sql });
  throw error;
}
```

## When to Use Each Level

### debug
- Detailed diagnostic information
- Variable values during development
- Step-by-step execution flow
- Performance metrics

```typescript
logger.debug('Computing recommendations', {
  stateHash: hash,
  candidateCount: candidates.length,
  explorationMode: epsilon > 0.2,
});
```

### info
- Application lifecycle events (startup, shutdown)
- Successful operations
- Configuration loaded
- User actions

```typescript
logger.info('Server started', { port: CONFIG.api.port });
logger.info('User session created', { userId, sessionId });
```

### warn
- Deprecated feature usage
- Approaching resource limits
- Recoverable errors
- Configuration issues

```typescript
logger.warn('High memory usage', { usage: process.memoryUsage().heapUsed });
logger.warn('Rate limit approaching', { requests: currentRate, limit: maxRate });
```

### error
- Unhandled exceptions
- Failed operations
- Data integrity issues
- External service failures

```typescript
logger.error('Failed to process feedback', error, {
  userId,
  contentId,
  attemptCount,
});
```

## Best Practices

### ✅ Do

- Use structured data objects instead of string concatenation
- Create module-specific loggers for better context
- Log errors with the original Error object
- Use appropriate log levels
- Include relevant context (userId, contentId, etc.)

```typescript
// Good
logger.info('Recommendation generated', {
  userId,
  contentId,
  score: recommendation.score,
});

logger.error('Database query failed', error, {
  query: 'SELECT * FROM users',
  userId,
});
```

### ❌ Don't

- Don't concatenate strings (use data objects)
- Don't log sensitive information (passwords, tokens, etc.)
- Don't use `console.log` directly
- Don't log the same event at multiple levels
- Don't log in tight loops without sampling

```typescript
// Bad
console.log('User ' + userId + ' logged in'); // Use logger.info with data object
logger.info('Password: ' + password); // NEVER log sensitive data
logger.debug('Processing item'); // In a loop - too verbose
logger.error('Error occurred: ' + error.message); // Use error object
```

## Production Considerations

### Log Volume

In production, only `warn` and `error` logs are emitted by default:

```typescript
// These are filtered out in production (LOG_LEVEL=warn):
logger.debug('...'); // ❌ Not logged
logger.info('...');  // ❌ Not logged

// These are always logged:
logger.warn('...');  // ✅ Logged
logger.error('...'); // ✅ Logged
```

### Performance

The logger checks log levels before formatting, so filtered logs have minimal overhead:

```typescript
// This is fast even with expensive data computation:
logger.debug('State details', {
  // This function only runs if debug level is active
  state: expensiveStateSerializer(),
});
```

### Log Aggregation

The logger outputs structured logs that work well with log aggregation services:

**Pretty format (development):**
```
[2025-12-07T10:30:45.123Z] [INFO] [RecommendationEngine] Generated recommendations
  Data: { "count": 10, "userId": "user123" }
```

**JSON format (production):**
```json
{"timestamp":"2025-12-07T10:30:45.123Z","level":"INFO","message":"Generated recommendations","context":"RecommendationEngine","data":{"count":10,"userId":"user123"}}
```

## Automated Migration

To help migrate existing code, you can use this regex pattern to find console.log statements:

```bash
# Find all console.log statements
grep -rn "console\.\(log\|debug\|info\|warn\|error\)" src/

# Count by type
grep -roh "console\.[a-z]*" src/ | sort | uniq -c
```

## Testing

When writing tests, mock the logger to verify logging behavior:

```typescript
import { logger } from './utils/logger';

jest.spyOn(logger, 'info').mockImplementation();
jest.spyOn(logger, 'error').mockImplementation();

// Your test code
someFunction();

expect(logger.info).toHaveBeenCalledWith('Expected message', expect.any(Object));
```

## Summary

1. **Replace all `console.log` with `logger.debug` or `logger.info`**
2. **Replace `console.error` with `logger.error`**
3. **Replace `console.warn` with `logger.warn`**
4. **Use structured data objects for context**
5. **Create module-specific loggers for better organization**
6. **Set `LOG_LEVEL=warn` in production**

For questions or issues, refer to the logger implementation at `src/utils/logger.ts`.
