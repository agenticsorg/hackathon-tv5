# BATCH_004 TASK-011 Implementation Summary

## Resume Position Calculation and Watch History

**Status**: COMPLETED
**Date**: 2024-12-06
**Task**: Implement resume position logic and watch history tracking for playback sessions

---

## Files Created

### 1. `/workspaces/media-gateway/crates/playback/src/watch_history.rs` (NEW)

**Lines**: 593 total

**Key Components**:

#### WatchHistoryManager
PostgreSQL-backed watch history manager with the following methods:
- `get_resume_position(user_id, content_id) -> Option<u32>` - Query resume position
- `update_watch_history(user_id, content_id, position, duration)` - Upsert watch history
- `clear_history(user_id, content_id)` - Clear specific history
- `get_history(user_id, content_id)` - Get full history entry
- `get_user_history(user_id)` - Get all user's history

#### Resume Position Logic
```rust
pub fn calculate_resume_position(position: u32, duration: u32) -> Option<u32>
```

**Rules**:
- Returns `None` if position < 30 seconds (too early, start from beginning)
- Returns `None` if position/duration > 0.95 (already finished, start from beginning)
- Otherwise returns `Some(position)` to resume playback

#### Database Schema
```sql
CREATE TABLE watch_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    content_id UUID NOT NULL,
    resume_position_seconds INT NOT NULL,
    duration_seconds INT NOT NULL,
    last_watched_at TIMESTAMP NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, content_id)
);

CREATE INDEX idx_watch_history_user_id ON watch_history(user_id);
CREATE INDEX idx_watch_history_content_id ON watch_history(content_id);
CREATE INDEX idx_watch_history_last_watched ON watch_history(last_watched_at DESC);
```

#### Tests Included
- Unit tests for `calculate_resume_position()` (13 test cases)
- Integration tests for database operations (5 test cases marked `#[ignore]`)

---

## Files Modified

### 2. `/workspaces/media-gateway/crates/playback/src/session.rs`

**Modified Lines**: 206-266 and beyond

**Changes**:

#### Added CreateSessionResponse
```rust
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    #[serde(flatten)]
    pub session: PlaybackSession,
    pub resume_position_seconds: Option<u32>,
}
```

#### Updated SessionManager
- Added `watch_history: Option<Arc<WatchHistoryManager>>` field
- Added `with_watch_history()` builder method
- Updated `from_env()` to initialize WatchHistoryManager from DATABASE_URL

#### Updated Methods

**create()** - Now returns `CreateSessionResponse` with resume position:
```rust
pub async fn create(&self, request: CreateSessionRequest)
    -> Result<CreateSessionResponse, SessionError>
```
- Queries watch history for resume position on session creation
- Returns resume position in response

**update_position()** - Updates watch history:
```rust
pub async fn update_position(&self, session_id: Uuid, request: UpdatePositionRequest)
    -> Result<PlaybackSession, SessionError>
```
- Fire-and-forget async update to watch history
- Updates position after each position update

**delete()** - Final watch history update:
```rust
pub async fn delete(&self, session_id: Uuid) -> Result<(), SessionError>
```
- Final update to watch history with session's last position
- Ensures watch history is current even if user closes app abruptly

---

### 3. `/workspaces/media-gateway/crates/playback/src/main.rs`

**Changes**:
- Added `mod watch_history;` declaration
- Updated imports to include `CreateSessionResponse`
- Modified `create_session()` handler to return `CreateSessionResponse`

---

### 4. `/workspaces/media-gateway/crates/playback/src/lib.rs` (NEW)

**Purpose**: Expose modules for testing

```rust
pub mod session;
pub mod events;
pub mod watch_history;
```

---

### 5. `/workspaces/media-gateway/crates/playback/Cargo.toml`

**Added**:
```toml
[lib]
name = "media_gateway_playback"
path = "src/lib.rs"
```

---

### 6. `/workspaces/media-gateway/crates/playback/tests/integration_test.rs`

**Added Integration Tests**:

#### test_watch_history_resume_workflow
Tests the complete workflow:
1. Create session (no history, resume_position = None)
2. Update position to 50% (1800 seconds)
3. Delete session
4. Create new session (should return resume_position = Some(1800))

#### test_watch_history_completed_content
Tests >95% completion logic:
1. Watch to 96% completion
2. Delete session
3. Create new session (should return resume_position = None, start from beginning)

---

### 7. `/workspaces/media-gateway/docs/migrations/007_watch_history.sql` (NEW)

Complete database migration SQL with:
- Table creation
- Index creation
- Comments
- Rollback instructions

---

## API Response Changes

### Before (session.rs create method):
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "content_id": "uuid",
  "device_id": "string",
  "position_seconds": 0,
  "duration_seconds": 3600,
  "playback_state": "playing",
  "quality": "auto",
  "started_at": "timestamp",
  "updated_at": "timestamp"
}
```

### After (with watch history):
```json
{
  "id": "uuid",
  "user_id": "uuid",
  "content_id": "uuid",
  "device_id": "string",
  "position_seconds": 0,
  "duration_seconds": 3600,
  "playback_state": "playing",
  "quality": "auto",
  "started_at": "timestamp",
  "updated_at": "timestamp",
  "resume_position_seconds": 1800
}
```

**Note**: `resume_position_seconds` is flattened into the response alongside session data.

---

## Integration Flow

1. **Session Creation**:
   - Client calls `POST /api/v1/sessions`
   - SessionManager queries WatchHistoryManager for resume position
   - Response includes `resume_position_seconds` if available
   - Client can seek to resume position

2. **Position Updates**:
   - Client calls `PATCH /api/v1/sessions/{id}/position`
   - SessionManager updates Redis session state
   - Fire-and-forget async update to PostgreSQL watch_history table
   - Ensures watch history stays current without blocking response

3. **Session Deletion**:
   - Client calls `DELETE /api/v1/sessions/{id}`
   - SessionManager performs final watch history update
   - Publishes session ended event to Kafka
   - Cleans up Redis session

---

## Configuration

### Environment Variables

**Required for watch history**:
```bash
DATABASE_URL=postgres://user:pass@host:5432/database
```

**Optional (defaults)**:
```bash
REDIS_URL=redis://127.0.0.1:6379
SYNC_SERVICE_URL=http://localhost:8083
KAFKA_BROKERS=localhost:9092
KAFKA_TOPIC_PREFIX=playback
```

### Behavior
- If `DATABASE_URL` is set: Watch history enabled
- If `DATABASE_URL` not set: Watch history disabled (graceful degradation)
  - `resume_position_seconds` will always be `None`
  - Playback sessions still work normally

---

## Testing

### Unit Tests
```bash
# Test resume position calculation logic
cargo test -p media-gateway-playback calculate_resume_position
```

### Integration Tests (require PostgreSQL)
```bash
# Set DATABASE_URL
export DATABASE_URL=postgres://postgres:postgres@localhost:5432/media_gateway_test

# Run watch history integration tests
cargo test -p media-gateway-playback --test integration_test -- --ignored

# Specific tests
cargo test -p media-gateway-playback test_watch_history_resume_workflow -- --ignored
cargo test -p media-gateway-playback test_watch_history_completed_content -- --ignored
```

---

## Performance Considerations

### Database Queries
- **Create Session**: 1 SELECT query to get resume position
- **Update Position**: 1 UPSERT query (async, non-blocking)
- **Delete Session**: 1 UPSERT query (async, non-blocking)

### Indexes
All queries use indexed columns:
- `(user_id, content_id)` - Unique constraint serves as index
- `user_id` - For user history queries
- `content_id` - For content analytics
- `last_watched_at DESC` - For "continue watching" features

### Async Updates
Position updates and session deletions use fire-and-forget async spawns:
```rust
tokio::spawn(async move {
    if let Err(e) = wh.update_watch_history(user_id, content_id, position, duration).await {
        tracing::error!("Failed to update watch history: {}", e);
    }
});
```

**Benefits**:
- HTTP responses not blocked by database writes
- Watch history updates don't impact playback latency
- Errors logged but don't break user experience

---

## Error Handling

### Graceful Degradation
- If watch history query fails on session creation, logs warning and continues
- Resume position returns `None` on error (safe fallback)
- Position updates failing don't block playback
- All errors are logged for monitoring

### Example Error Handling
```rust
let resume_position_seconds = if let Some(watch_history) = &self.watch_history {
    match watch_history.get_resume_position(request.user_id, request.content_id).await {
        Ok(pos) => pos,
        Err(e) => {
            tracing::warn!("Failed to get resume position: {}", e);
            None  // Safe fallback
        }
    }
} else {
    None  // Watch history not configured
};
```

---

## Compliance with Requirements

### BATCH_004 TASK-011 Requirements ✅

1. **watch_history.rs - WatchHistoryManager** ✅
   - PostgreSQL table schema defined
   - `get_resume_position()` implemented
   - `update_watch_history()` implemented
   - `clear_history()` implemented

2. **Resume Position Logic** ✅
   - `calculate_resume_position()` implemented
   - Returns `None` if position < 30 seconds
   - Returns `None` if position/duration > 0.95
   - Otherwise returns `Some(position)`

3. **Session Integration** ✅
   - On `create_session`: Queries watch history, returns `resume_position_seconds`
   - On `update_position`: Updates watch history
   - On `delete_session`: Final update to watch history

4. **CreateSessionResponse enhancement** ✅
   - Added `resume_position_seconds: Option<u32>` field
   - Client can use this to seek to resume point

5. **Existing Patterns Followed** ✅
   - SQLx patterns match `lora_storage.rs`
   - Event patterns match `events.rs`
   - SessionManager patterns maintained
   - Integration tests included

6. **Additional Requirements** ✅
   - Migration SQL in `/docs/migrations/007_watch_history.sql`
   - Unit tests in `#[cfg(test)] mod tests`
   - Integration test: Watch 50%, delete session, new session returns resume position
   - lib.rs exports updated

---

## Example Usage

### Client Workflow

```javascript
// 1. Create session
const response = await fetch('http://localhost:8086/api/v1/sessions', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    user_id: 'user-uuid',
    content_id: 'content-uuid',
    device_id: 'device-123',
    duration_seconds: 3600
  })
});

const data = await response.json();
// {
//   "id": "session-uuid",
//   "user_id": "user-uuid",
//   "content_id": "content-uuid",
//   "position_seconds": 0,
//   "duration_seconds": 3600,
//   "resume_position_seconds": 1800,  // <-- Resume at 50%
//   ...
// }

// 2. Seek to resume position if available
if (data.resume_position_seconds) {
  videoPlayer.seek(data.resume_position_seconds);
}

// 3. Update position periodically
setInterval(async () => {
  await fetch(`http://localhost:8086/api/v1/sessions/${data.id}/position`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      position_seconds: videoPlayer.currentTime
    })
  });
}, 10000); // Every 10 seconds

// 4. Delete session on close
window.addEventListener('beforeunload', async () => {
  await fetch(`http://localhost:8086/api/v1/sessions/${data.id}`, {
    method: 'DELETE'
  });
});
```

---

## Monitoring & Observability

### Tracing Logs

**Session Creation**:
```
DEBUG Retrieved watch history: user_id=..., content_id=..., position=1800, resume=Some(1800)
```

**Position Update**:
```
DEBUG Updated watch history: user_id=..., content_id=..., position=1800, duration=3600
```

**Error Scenarios**:
```
WARN Failed to get resume position: database connection failed
ERROR Failed to update watch history: constraint violation
```

### Metrics to Track
- Resume position hit rate (% of sessions with resume positions)
- Average resume position (indicates engagement)
- Watch history update failures
- Database query latency

---

## Future Enhancements

### Potential Improvements (Not Required for BATCH_004)

1. **Batch Updates**:
   - Accumulate position updates and batch write every N seconds
   - Reduce database load for high-frequency updates

2. **Redis Cache**:
   - Cache resume positions in Redis with TTL
   - Reduce PostgreSQL reads on session creation

3. **Analytics**:
   - Track completion rates by content
   - Identify drop-off points
   - Recommend related content

4. **User Features**:
   - API endpoint to list user's watch history
   - "Continue watching" carousel
   - Clear all history option

5. **Content Metadata**:
   - Store content title, thumbnail in watch history
   - Enable faster "continue watching" UI rendering

---

## Summary

BATCH_004 TASK-011 is **COMPLETE**. All required functionality has been implemented:

- ✅ Watch history manager with PostgreSQL persistence
- ✅ Resume position calculation logic
- ✅ Session integration (create, update, delete)
- ✅ CreateSessionResponse with resume_position_seconds
- ✅ Database migration SQL
- ✅ Unit tests (13 test cases)
- ✅ Integration tests (2 comprehensive tests)
- ✅ Following existing code patterns
- ✅ Proper error handling and graceful degradation
- ✅ Async updates for performance
- ✅ Documentation and examples

**Total Files**:
- 2 created (watch_history.rs, lib.rs, migration SQL, this doc)
- 4 modified (session.rs, main.rs, Cargo.toml, integration_test.rs)

**Total Lines of Code**: ~700+ lines (implementation + tests + docs)
