# Logger Quick Reference

## Import

```typescript
import { logger, createLogger } from './utils/logger';
```

## Basic Usage

```typescript
// Default logger
logger.debug('Debug message', { data: value });
logger.info('Info message', { userId: '123' });
logger.warn('Warning message', { metric: high });
logger.error('Error message', error, { context: data });

// Module-specific logger
const moduleLogger = createLogger('MyModule');
moduleLogger.info('Module initialized');

// Child logger (nested context)
const childLogger = moduleLogger.child('SubModule');
childLogger.debug('Sub-module processing');
```

## Environment Configuration

| Environment | Default Level | Logs Shown |
|-------------|---------------|------------|
| Development | `info` | info, warn, error |
| Production | `warn` | warn, error only |
| Debug | `debug` | all logs |

**Override:** Set `LOG_LEVEL=debug|info|warn|error` in `.env`

## Log Levels

| Level | Use For | Production? |
|-------|---------|-------------|
| `debug` | Detailed diagnostics, state dumps | ❌ |
| `info` | General information, lifecycle events | ❌ |
| `warn` | Warnings, potential issues | ✅ |
| `error` | Errors, exceptions | ✅ |

## Migration Cheat Sheet

```typescript
// Before → After
console.log(...)        → logger.info(...)
console.debug(...)      → logger.debug(...)
console.info(...)       → logger.info(...)
console.warn(...)       → logger.warn(...)
console.error(...)      → logger.error(...)

// String concatenation → Structured data
console.log('User ' + id + ' logged in')
  → logger.info('User logged in', { userId: id })

// Error logging
console.error('Failed:', err)
  → logger.error('Operation failed', err, { context })
```

## Common Patterns

```typescript
// API Request
logger.info('Request received', {
  method: req.method,
  path: req.path,
  userId: req.user?.id,
});

// Database Query
logger.debug('Query executed', {
  query: sql,
  duration: ms,
  rowCount: rows.length,
});

// Performance
const start = Date.now();
await operation();
logger.debug('Operation completed', {
  operation: 'name',
  duration: Date.now() - start,
});

// Error Handling
try {
  await riskyOperation();
} catch (error) {
  logger.error('Operation failed', error, {
    operation: 'name',
    userId,
  });
  throw error;
}
```

## Best Practices

✅ **Do:**
- Use structured data objects
- Create module-specific loggers
- Log errors with original Error object
- Include relevant context (userId, requestId, etc.)

❌ **Don't:**
- Don't concatenate strings
- Don't log sensitive data (passwords, tokens)
- Don't use console.log directly
- Don't log in tight loops

## Testing

```typescript
import { logger } from './utils/logger';

const infoSpy = jest.spyOn(logger, 'info').mockImplementation();

myFunction();

expect(infoSpy).toHaveBeenCalledWith(
  'Expected message',
  expect.objectContaining({ userId: '123' })
);

infoSpy.mockRestore();
```

## Scripts

```bash
# Analyze console.log usage
./scripts/check-logging.sh

# Run logger demo
LOG_LEVEL=debug ts-node examples/logger-demo.ts
```

## Files

- Implementation: `src/utils/logger.ts`
- Configuration: `src/utils/config.ts`
- Tests: `tests/utils/logger.test.ts`
- Guide: `docs/logging-migration-guide.md`
