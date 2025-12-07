# MCP Protocol 2024-11-05 - Quick Reference

## New Methods Summary

| Method | Type | Request ID | Returns | Purpose |
|--------|------|-----------|---------|---------|
| `ping` | Request | Required | `{}` | Health check |
| `notifications/initialized` | Notification | N/A | void | Init complete |
| `notifications/cancelled` | Notification | N/A | void | Cancel request |
| `logging/setLevel` | Request | Required | `{previousLevel, currentLevel}` | Change log level |
| `completion/complete` | Request | Required | `{completion}` | Autocomplete |

## Quick Examples

### Ping
```json
// Request
{ "jsonrpc": "2.0", "method": "ping", "id": 1 }

// Response
{ "jsonrpc": "2.0", "result": {}, "id": 1 }
```

### Set Log Level
```json
// Request
{
  "jsonrpc": "2.0",
  "method": "logging/setLevel",
  "params": { "level": "debug" },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "previousLevel": "info",
    "currentLevel": "debug"
  },
  "id": 2
}
```

### Cancel Request
```json
// Notification (no response)
{
  "jsonrpc": "2.0",
  "method": "notifications/cancelled",
  "params": {
    "requestId": "req-123",
    "reason": "Timeout"
  }
}
```

### Autocomplete
```json
// Request
{
  "jsonrpc": "2.0",
  "method": "completion/complete",
  "params": {
    "ref": { "type": "ref/prompt", "name": "search" },
    "argument": { "name": "genre", "value": "act" }
  },
  "id": 3
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "completion": {
      "values": ["action"],
      "hasMore": false,
      "total": 1
    }
  },
  "id": 3
}
```

## Log Levels

From most to least verbose:
1. **debug** - All messages
2. **info** - Informational and above (default)
3. **warn** - Warnings and errors only
4. **error** - Errors only

## Completion Contexts

| Argument | Completions |
|----------|-------------|
| `genre` | action, comedy, drama, sci-fi, horror, etc. |
| `platform` | netflix, hulu, disney-plus, amazon-prime, etc. |
| `region` | US, UK, CA, AU, DE, FR, JP |
| `contentType` | movie, tv |

## Files

- **Implementation**: `src/server.ts`
- **Types**: `src/types/index.ts`
- **Tests**: `src/tests/server.test.ts`
- **Docs**: `docs/MCP_PROTOCOL_2024-11-05.md`

## Testing

```bash
# Run tests
npm test

# Type check
npm run typecheck

# Build
npm run build
```

## Helper Functions

```typescript
// Get current log level
const level = getCurrentLogLevel(); // 'debug' | 'info' | 'warn' | 'error'

// Get active request count
const count = getActiveRequestsCount(); // number
```
