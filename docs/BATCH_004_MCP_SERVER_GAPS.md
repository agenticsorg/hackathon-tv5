# BATCH_004: MCP Server Implementation Gaps Analysis

**Analysis Date**: 2025-12-06
**Component**: MCP Server (`apps/mcp-server/`)
**Analyzer**: Coder Agent

## Context

- **BATCH_001**: Core platform foundation (completed)
- **BATCH_002**: Service integrations (completed)
- **BATCH_003**: MCP tool timeouts/retries via `fetchWithRetry` (completed)
- **BATCH_004**: NEW gaps in MCP protocol compliance and server features

## Identified Gaps (High Priority)

---

### GAP 1: Missing MCP Protocol 2024-11-05 Methods

**File**: `/workspaces/media-gateway/apps/mcp-server/src/server.ts`
**Lines**: 14-43 (handleMCPRequest switch statement)

**What's Missing**:
The MCP protocol 2024-11-05 defines several standard methods that are NOT implemented:

1. **`ping`** - Connection health check (CRITICAL for long-lived connections)
2. **`notifications/initialized`** - Client initialization signal
3. **`notifications/cancelled`** - Request cancellation support
4. **`logging/setLevel`** - Dynamic log level control
5. **`completion/complete`** - Argument auto-completion

**Current Implementation**:
```typescript
export async function handleMCPRequest(method: string, params: any): Promise<any> {
  switch (method) {
    case 'initialize':
      return handleInitialize();
    case 'tools/list':
      return handleToolsList();
    case 'tools/call':
      return handleToolCall(params);
    case 'resources/list':
      return handleResourcesList();
    case 'resources/read':
      return handleResourceRead(params);
    case 'prompts/list':
      return handlePromptsList();
    case 'prompts/get':
      return handlePromptGet(params);
    default:
      throw {
        code: MCPErrorCode.METHOD_NOT_FOUND,
        message: `Method not found: ${method}`,
      };
  }
}
```

**Importance**: HIGH
- `ping` is CRITICAL for SSE transport connection health monitoring
- `notifications/cancelled` enables request cancellation (prevents wasted compute)
- `logging/setLevel` enables runtime debugging without restarts
- Missing these violates MCP 2024-11-05 specification

**Acceptance Criteria**:
- [ ] Implement `ping` method returning `{}`
- [ ] Add `notifications/initialized` handler
- [ ] Add `notifications/cancelled` handler with request abort support
- [ ] Add `logging/setLevel` method updating config.logging.level
- [ ] Add `completion/complete` for tool/resource argument suggestions
- [ ] Update integration tests to verify all protocol methods
- [ ] Document new methods in ARW manifest

---

### GAP 2: Tool Response Format Inconsistency

**File**: `/workspaces/media-gateway/apps/mcp-server/src/server.ts`
**Lines**: 96-105 (handleToolCall response)

**What's Missing**:
MCP protocol specifies that tool responses should support multiple content types and proper metadata. Current implementation ONLY returns JSON as text, missing:

1. **Content type variation** - No support for images, embedded resources
2. **Metadata fields** - Missing `isError` flag for graceful error presentation
3. **Resource references** - Cannot reference MCP resources in responses
4. **Streaming responses** - No support for progressive tool output

**Current Implementation**:
```typescript
async function handleToolCall(params: {
  name: string;
  arguments: any;
  userContext?: UserContext;
}): Promise<any> {
  // ... executor lookup ...

  try {
    const result = await executor(args, userContext);
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),  // ONLY text/json
        },
      ],
    };
  } catch (error) {
    // Error thrown, not returned as content with isError: true
    throw {
      code: MCPErrorCode.TOOL_EXECUTION_ERROR,
      message: error instanceof Error ? error.message : 'Tool execution failed',
      data: error,
    };
  }
}
```

**Problems**:
- ALL responses are `type: 'text'` with JSON stringified
- No support for `type: 'image'` (thumbnail URLs from content)
- No support for `type: 'resource'` (linking to `media://trending`)
- Errors are thrown, not gracefully returned with `isError: true`
- No progress updates for long-running operations

**Importance**: HIGH
- AI agents need rich responses (thumbnails, structured data)
- Current format wastes tokens (JSON stringified in text)
- Error handling breaks agent conversation flow
- Cannot leverage existing MCP resources in tool responses

**Acceptance Criteria**:
- [ ] Add content type detection based on tool result
- [ ] Support `type: 'image'` for thumbnail URLs
- [ ] Support `type: 'resource'` for MCP resource references
- [ ] Add `isError: true` flag for graceful error responses
- [ ] Implement response metadata (processingTime, confidence, etc.)
- [ ] Add streaming support via SSE for long operations
- [ ] Update tool schemas to declare supported content types
- [ ] Add response format validation tests

---

### GAP 3: Missing Conversation Context Management

**File**: `/workspaces/media-gateway/apps/mcp-server/src/` (NEW FILE NEEDED)
**Lines**: N/A - Feature doesn't exist

**What's Missing**:
MCP servers should maintain conversation context across multiple tool calls to enable:

1. **Tool chaining** - "Show me recommendations for that movie" (no "that" context)
2. **Session state** - User's current search filters, selected content
3. **Multi-turn interactions** - Follow-up questions without repeating context
4. **Personalization** - Remember user's viewing history within conversation

**Current Implementation**:
NONE - Each tool call is completely stateless. No session tracking, no context preservation.

**Example Problem Scenarios**:

**Scenario 1 - Tool Chaining**:
```
User: "Find me sci-fi thrillers from 2020"
Agent: [Uses semantic_search, gets results]
User: "Check availability for the first one"
Agent: ❌ FAILS - no context of what "first one" refers to
```

**Scenario 2 - Multi-turn Search**:
```
User: "Show me action movies"
Agent: [Returns list]
User: "Filter to only ones on Netflix"
Agent: ❌ Must re-specify entire query
```

**Scenario 3 - Personalization**:
```
User: "I liked Inception, recommend similar"
Agent: [Gets recommendations]
User: "Not these, try more cerebral"
Agent: ❌ No memory of previous recommendations to exclude
```

**Missing Components**:
- Session ID generation and tracking
- Conversation history storage (last N tool calls + results)
- Context extraction from natural language ("that", "it", "the first one")
- Reference resolution (contentId from previous results)
- Session cleanup/expiration

**Importance**: HIGH
- Critical for natural conversation flow
- Reduces token usage (don't repeat full context)
- Enables sophisticated multi-step workflows
- Required for production AI agent experience

**Acceptance Criteria**:
- [ ] Create `/workspaces/media-gateway/apps/mcp-server/src/context/session.ts`
- [ ] Implement SessionManager with in-memory store (Map<sessionId, Context>)
- [ ] Add session ID to tool call parameters
- [ ] Store last 5 tool calls + results per session
- [ ] Add context resolution utilities (resolveContentReference, etc.)
- [ ] Implement session expiration (30min TTL)
- [ ] Add `context/get` method to retrieve session state
- [ ] Add `context/clear` method for explicit reset
- [ ] Update tool executors to accept/update session context
- [ ] Add session management tests
- [ ] Document context patterns in prompts

**Implementation Hint**:
```typescript
// apps/mcp-server/src/context/session.ts
interface SessionContext {
  id: string;
  createdAt: Date;
  lastAccessedAt: Date;
  history: ToolCallHistory[];
  state: {
    lastSearchResults?: SearchResult[];
    lastRecommendations?: RecommendationResult[];
    selectedContentId?: string;
    filters?: Record<string, any>;
  };
}

interface ToolCallHistory {
  toolName: string;
  params: any;
  result: any;
  timestamp: Date;
}

export class SessionManager {
  private sessions = new Map<string, SessionContext>();

  getOrCreate(sessionId?: string): SessionContext;
  update(sessionId: string, update: Partial<SessionContext>): void;
  cleanup(): void; // Remove expired sessions
  resolveReference(sessionId: string, ref: string): any; // "the first one" -> contentId
}
```

---

### GAP 4: SSE Transport Missing Server-Sent Notifications

**File**: `/workspaces/media-gateway/apps/mcp-server/src/transports/sse.ts`
**Lines**: 70-88 (SSE events endpoint), 218-231 (sendEvent/broadcastEvent)

**What's Missing**:
The SSE transport has `sendEvent` and `broadcastEvent` methods but they are NEVER USED. MCP protocol requires server-initiated notifications for:

1. **Resource updates** - Notify when trending content changes
2. **Tool discovery** - Notify when new tools become available
3. **Progress notifications** - Long-running tool execution updates
4. **Log messages** - Server logs for debugging

**Current Implementation**:
```typescript
// SSE endpoint established
this.app.get('/mcp/events', (req: Request, res: Response) => {
  const clientId = req.query.clientId as string || `client-${Date.now()}`;

  res.setHeader('Content-Type', 'text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');

  this.clients.set(clientId, res);

  // Send initial connection event
  res.write(`event: connected\n`);
  res.write(`data: ${JSON.stringify({ clientId, timestamp: new Date().toISOString() })}\n\n`);

  // ❌ NO OTHER EVENTS EVER SENT
});

// Methods exist but unused:
public sendEvent(clientId: string, event: string, data: any): void {
  const client = this.clients.get(clientId);
  if (client) {
    client.write(`event: ${event}\n`);
    client.write(`data: ${JSON.stringify(data)}\n\n`);
  }
}

public broadcastEvent(event: string, data: any): void {
  this.clients.forEach((client) => {
    client.write(`event: ${event}\n`);
    client.write(`data: ${JSON.stringify(data)}\n\n`);
  });
}
```

**Problems**:
- SSE connection established but only sends initial "connected" event
- No notifications for `resources/list` changes
- No notifications for `tools/list` changes
- No progress updates for long-running tool calls
- No server logs sent to client for debugging

**MCP Protocol Requirements**:
According to MCP spec, servers MUST send notifications for:
- `notifications/resources/list_changed` - When resources change
- `notifications/tools/list_changed` - When tools change
- `notifications/progress` - For tool execution progress
- `notifications/message` - For log messages

**Importance**: MEDIUM-HIGH
- Required for reactive MCP clients
- Enables real-time updates without polling
- Critical for long-running operations (SONA training, large searches)
- Currently advertising `listChanged: false` - should be true with notifications

**Acceptance Criteria**:
- [ ] Implement periodic resource update checker (trending content changes)
- [ ] Broadcast `notifications/resources/list_changed` when resources update
- [ ] Add progress tracking to tool executors
- [ ] Send `notifications/progress` during long operations (>1s)
- [ ] Implement log level filtering and `notifications/message` for logs
- [ ] Add heartbeat/ping events (every 30s) to detect connection issues
- [ ] Update `handleInitialize` to return `listChanged: true`
- [ ] Add SSE notification integration tests
- [ ] Document SSE events in API docs

**Implementation Hint**:
```typescript
// In SSE transport
private startResourceMonitor(): void {
  setInterval(async () => {
    const changed = await this.checkResourceChanges();
    if (changed) {
      this.broadcastEvent('notification', {
        method: 'notifications/resources/list_changed',
      });
    }
  }, 60000); // Check every minute
}

// In tool executors (semantic_search.ts, etc.)
async function executeSemanticSearch(input, context, progressCallback?) {
  progressCallback?.({ status: 'embedding_query', progress: 0.2 });
  const embedding = await embedQuery(input.query);

  progressCallback?.({ status: 'searching', progress: 0.5 });
  const results = await search(embedding);

  progressCallback?.({ status: 'complete', progress: 1.0 });
  return results;
}
```

---

## Summary

**Total Gaps Identified**: 4
**Priority Breakdown**:
- HIGH: 3 gaps (ping/protocol methods, tool responses, conversation context)
- MEDIUM-HIGH: 1 gap (SSE notifications)

**Estimated Implementation Effort**:
- GAP 1 (Protocol methods): 4-6 hours
- GAP 2 (Tool responses): 6-8 hours
- GAP 3 (Context management): 8-12 hours
- GAP 4 (SSE notifications): 4-6 hours

**Total**: 22-32 hours (3-4 days)

**Dependencies**:
- GAP 3 (Context) should be implemented before GAP 2 (Responses) for maximum benefit
- GAP 4 (SSE) can be implemented in parallel
- GAP 1 (Protocol) is independent and can be done first

**Impact on System**:
- **User Experience**: MAJOR - Natural conversations, rich media responses
- **Protocol Compliance**: CRITICAL - Missing standard MCP methods
- **Performance**: MODERATE - Better error handling, streaming for long operations
- **Maintainability**: HIGH - Proper separation of concerns, testability

---

## Recommendations

1. **Immediate** (This week):
   - GAP 1: Add missing protocol methods (enables proper tooling)
   - GAP 3: Implement basic session management (fixes broken UX)

2. **Short-term** (Next sprint):
   - GAP 2: Rich tool response formats
   - GAP 4: SSE notification system

3. **Testing Priority**:
   - Integration tests for all MCP protocol methods
   - Session management edge cases (expiration, concurrency)
   - SSE connection lifecycle (connect, reconnect, cleanup)

4. **Documentation**:
   - Update ARW manifest with new capabilities
   - Add MCP protocol compliance matrix
   - Document conversation patterns for AI agents

---

**Generated**: 2025-12-06
**Agent**: Coder (MCP Server Analysis)
**Next Steps**: Review with team, prioritize for BATCH_004 implementation
