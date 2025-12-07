# EmotiStream Logging System

## Overview

EmotiStream uses an environment-aware structured logging system that automatically adjusts log verbosity based on the deployment environment. This ensures detailed logs during development while minimizing overhead and information exposure in production.

## Quick Start

```typescript
import { logger, createLogger } from './utils/logger';

// Use default logger
logger.info('Application started', { port: 3000 });
logger.error('Operation failed', error, { userId: '123' });

// Create module-specific logger
const engineLogger = createLogger('RecommendationEngine');
engineLogger.debug('Processing recommendations', { count: 10 });
```

## Environment-Based Behavior

### Development (Default)
```bash
NODE_ENV=development
LOG_LEVEL=info  # Shows: info, warn, error
```

Logs are formatted with pretty-printing for readability:
```
[2025-12-07T10:30:45.123Z] [INFO] [RecommendationEngine] Generated recommendations
  Data: {
    "count": 10,
    "userId": "user123"
  }
```

### Production
```bash
NODE_ENV=production
LOG_LEVEL=warn  # Shows only: warn, error (default)
```

Logs are formatted as JSON for machine parsing:
```json
{"timestamp":"2025-12-07T10:30:45.123Z","level":"WARN","message":"High memory usage","context":"System","data":{"usage":500000000}}
```

### Debug Mode
```bash
LOG_LEVEL=debug  # Shows all: debug, info, warn, error
```

## Log Levels

| Level | Priority | Usage | Development | Production |
|-------|----------|-------|-------------|------------|
| **debug** | 0 | Detailed diagnostics, variable dumps, state tracking | ✅ | ❌ |
| **info** | 1 | Application lifecycle, user actions, general information | ✅ | ❌ |
| **warn** | 2 | Warnings, deprecated features, approaching limits | ✅ | ✅ |
| **error** | 3 | Errors, exceptions, failures | ✅ | ✅ |

## API Reference

### Default Logger

```typescript
import { logger } from './utils/logger';

// Debug level - filtered in production
logger.debug(message: string, data?: unknown): void

// Info level - filtered in production
logger.info(message: string, data?: unknown): void

// Warning level - always logged
logger.warn(message: string, data?: unknown): void

// Error level - always logged
logger.error(message: string, error?: Error | unknown, data?: unknown): void
```

### Module-Specific Logger

```typescript
import { createLogger } from './utils/logger';

const moduleLogger = createLogger('ModuleName');

moduleLogger.info('Module initialized');
moduleLogger.debug('Processing', { items: 5 });
```

### Child Logger (Nested Context)

```typescript
const parentLogger = createLogger('API');
const authLogger = parentLogger.child('Auth');
const dbLogger = parentLogger.child('Database');

authLogger.info('User authenticated');  // [API:Auth] User authenticated
dbLogger.debug('Query executed');       // [API:Database] Query executed
```

## Usage Examples

### Application Startup

```typescript
import { logger } from './utils/logger';

logger.info('Server starting', {
  port: CONFIG.api.port,
  environment: process.env.NODE_ENV,
});

logger.info('Database connected', {
  host: db.host,
  database: db.database,
});
```

### API Request Logging

```typescript
import { createLogger } from './utils/logger';

const apiLogger = createLogger('API');

app.use((req, res, next) => {
  apiLogger.info('Request received', {
    method: req.method,
    path: req.path,
    ip: req.ip,
    userId: req.user?.id,
  });
  next();
});
```

### Error Handling

```typescript
import { createLogger } from './utils/logger';

const dbLogger = createLogger('Database');

try {
  const result = await db.query(sql, params);
  dbLogger.debug('Query executed', {
    query: sql,
    rowCount: result.rows.length,
    duration: result.duration,
  });
  return result;
} catch (error) {
  dbLogger.error('Query failed', error, {
    query: sql,
    params,
  });
  throw error;
}
```

### Performance Monitoring

```typescript
import { createLogger } from './utils/logger';

const perfLogger = createLogger('Performance');

const start = Date.now();
const recommendations = await generateRecommendations(userId);
const duration = Date.now() - start;

perfLogger.debug('Recommendations generated', {
  userId,
  count: recommendations.length,
  duration,
});

if (duration > 1000) {
  perfLogger.warn('Slow recommendation generation', {
    userId,
    duration,
    threshold: 1000,
  });
}
```

### State Debugging

```typescript
import { createLogger } from './utils/logger';

const rlLogger = createLogger('RL');

rlLogger.debug('State transition', {
  from: {
    valence: currentState.valence,
    arousal: currentState.arousal,
    stress: currentState.stress,
  },
  to: {
    valence: nextState.valence,
    arousal: nextState.arousal,
    stress: nextState.stress,
  },
  action: selectedContent.contentId,
  reward,
});
```

## Configuration

### Environment Variables

```bash
# .env file
NODE_ENV=production        # Environment: development | production | test
LOG_LEVEL=warn            # Level: debug | info | warn | error
```

### Runtime Override

```bash
# Temporarily change log level without modifying .env
LOG_LEVEL=debug npm start

# Or export for session
export LOG_LEVEL=debug
npm start
```

### Programmatic Access

```typescript
import { CONFIG } from './utils/config';

console.log('Current log level:', CONFIG.logging.level);
console.log('Pretty printing:', CONFIG.logging.pretty);
```

## Migration from console.log

### Find console.log Usage

```bash
# Analyze current console usage
./scripts/check-logging.sh

# Find all console.log calls
grep -rn "console\." src/
```

### Migration Patterns

```typescript
// ❌ Before: String concatenation
console.log('User ' + userId + ' logged in at ' + timestamp);

// ✅ After: Structured logging
logger.info('User logged in', { userId, timestamp });

// ❌ Before: Direct error logging
console.error('Failed to save:', error);

// ✅ After: Error with context
logger.error('Save operation failed', error, { userId, documentId });

// ❌ Before: Debug print
console.log('[DEBUG]', state);

// ✅ After: Debug logging
logger.debug('State update', { state });
```

### Migration Checklist

- [ ] Replace `console.log()` with `logger.info()` or `logger.debug()`
- [ ] Replace `console.error()` with `logger.error()`
- [ ] Replace `console.warn()` with `logger.warn()`
- [ ] Convert string concatenation to structured data objects
- [ ] Remove sensitive information (passwords, tokens, etc.)
- [ ] Add relevant context (userId, requestId, etc.)
- [ ] Create module-specific loggers for better organization
- [ ] Test with different log levels
- [ ] Update tests to use logger

## Testing

### Unit Tests

```typescript
import { logger } from './utils/logger';

describe('MyService', () => {
  let infoSpy: jest.SpyInstance;
  let errorSpy: jest.SpyInstance;

  beforeEach(() => {
    infoSpy = jest.spyOn(logger, 'info').mockImplementation();
    errorSpy = jest.spyOn(logger, 'error').mockImplementation();
  });

  afterEach(() => {
    infoSpy.mockRestore();
    errorSpy.mockRestore();
  });

  it('should log successful operation', () => {
    service.process();

    expect(infoSpy).toHaveBeenCalledWith(
      'Processing completed',
      expect.objectContaining({ status: 'success' })
    );
  });

  it('should log errors', () => {
    const error = new Error('Test error');
    service.processError(error);

    expect(errorSpy).toHaveBeenCalledWith(
      'Processing failed',
      error,
      expect.any(Object)
    );
  });
});
```

### Integration Tests

```typescript
// For integration tests, you may want to see actual logs
// Just don't mock the logger

describe('API Integration', () => {
  it('should handle requests', async () => {
    const response = await request(app)
      .post('/api/recommendations')
      .send({ userId: 'test' });

    expect(response.status).toBe(200);
    // Logger will output to console for debugging
  });
});
```

## Best Practices

### ✅ Do

1. **Use structured data**
   ```typescript
   logger.info('User action', { userId, action, timestamp });
   ```

2. **Create module-specific loggers**
   ```typescript
   const logger = createLogger('MyModule');
   ```

3. **Log errors with Error objects**
   ```typescript
   logger.error('Operation failed', error, { context });
   ```

4. **Include relevant context**
   ```typescript
   logger.info('Request processed', { requestId, userId, duration });
   ```

5. **Use appropriate log levels**
   - `debug` for detailed diagnostics
   - `info` for general information
   - `warn` for warnings
   - `error` for errors

### ❌ Don't

1. **Don't concatenate strings**
   ```typescript
   // Bad
   logger.info('User ' + userId + ' logged in');

   // Good
   logger.info('User logged in', { userId });
   ```

2. **Don't log sensitive data**
   ```typescript
   // Bad - NEVER do this
   logger.info('User credentials', { password, token });

   // Good
   logger.info('User authenticated', { userId });
   ```

3. **Don't use console.log**
   ```typescript
   // Bad
   console.log('Something happened');

   // Good
   logger.info('Event occurred', { eventType: 'something' });
   ```

4. **Don't log in tight loops**
   ```typescript
   // Bad
   items.forEach(item => logger.debug('Processing', item));

   // Good - log summary
   logger.debug('Processing items', { count: items.length });
   ```

## Performance Considerations

### Lazy Evaluation

The logger only evaluates data if the log level is active:

```typescript
// This is efficient - expensive function only called if debug is active
logger.debug('State dump', {
  state: expensiveStateSerializer(), // Only called if level >= DEBUG
});
```

### Production Overhead

With `LOG_LEVEL=warn` in production:
- `debug` calls: **~0 overhead** (filtered immediately)
- `info` calls: **~0 overhead** (filtered immediately)
- `warn` calls: **minimal overhead** (logged)
- `error` calls: **minimal overhead** (logged)

Expected reduction in production: **~90% fewer logs**

## Monitoring and Alerting

### JSON Format for Log Aggregation

Production logs are JSON-formatted for easy parsing:

```json
{
  "timestamp": "2025-12-07T10:30:45.123Z",
  "level": "ERROR",
  "message": "Database connection failed",
  "context": "Database",
  "error": {
    "message": "Connection timeout",
    "code": "ETIMEDOUT",
    "stack": "Error: Connection timeout\n    at ..."
  },
  "data": {
    "host": "localhost",
    "port": 5432,
    "attemptCount": 3
  }
}
```

### Integration with Log Services

This format works seamlessly with:
- AWS CloudWatch
- Google Cloud Logging
- DataDog
- Splunk
- Elasticsearch/Kibana
- Grafana Loki

### Example CloudWatch Query

```
fields @timestamp, level, message, context, data.userId
| filter level = "ERROR"
| sort @timestamp desc
| limit 100
```

## Troubleshooting

### Logs not appearing

1. Check log level:
   ```bash
   echo $LOG_LEVEL
   # Should be: debug, info, warn, or error
   ```

2. Check environment:
   ```bash
   echo $NODE_ENV
   # In production, only warn/error are shown by default
   ```

3. Temporarily increase verbosity:
   ```bash
   LOG_LEVEL=debug npm start
   ```

### Too many logs in production

1. Set appropriate level:
   ```bash
   # .env
   NODE_ENV=production
   LOG_LEVEL=warn
   ```

2. Review debug/info usage:
   ```bash
   ./scripts/check-logging.sh
   ```

### Logs missing context

Add structured data:
```typescript
// Before
logger.info('Processing complete');

// After
logger.info('Processing complete', {
  userId,
  requestId,
  duration,
  itemsProcessed: items.length,
});
```

## Resources

- **Implementation**: `/workspaces/hackathon-tv5/apps/emotistream/src/utils/logger.ts`
- **Configuration**: `/workspaces/hackathon-tv5/apps/emotistream/src/utils/config.ts`
- **Tests**: `/workspaces/hackathon-tv5/apps/emotistream/tests/utils/logger.test.ts`
- **Migration Guide**: `/workspaces/hackathon-tv5/apps/emotistream/docs/logging-migration-guide.md`
- **Quick Reference**: `/workspaces/hackathon-tv5/apps/emotistream/docs/logger-quick-reference.md`
- **Demo**: `/workspaces/hackathon-tv5/apps/emotistream/examples/logger-demo.ts`
- **Analysis Script**: `/workspaces/hackathon-tv5/apps/emotistream/scripts/check-logging.sh`

## Support

For issues or questions about the logging system:
1. Review this documentation
2. Check the migration guide
3. Run the demo: `LOG_LEVEL=debug ts-node examples/logger-demo.ts`
4. Analyze your usage: `./scripts/check-logging.sh`
