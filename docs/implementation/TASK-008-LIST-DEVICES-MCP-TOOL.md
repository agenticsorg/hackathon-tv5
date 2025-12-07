# TASK-008: Implement MCP Server list_devices Tool

## Overview
Implementation of the `list_devices` MCP tool for listing user devices with their capabilities and status.

## Implementation Details

### Files Modified

#### 1. `/workspaces/media-gateway/crates/mcp-server/src/tools.rs`
- Added `ListDevicesTool` struct with database pool
- Added `DeviceInfo` struct for database query results
- Implemented `ToolExecutor` trait for device listing
- Query fetches devices ordered by `last_seen DESC`

**Key Features:**
- Queries `user_devices` table from migration 016_sync_schema.sql
- Returns device information including:
  - `device_id`: Unique device identifier
  - `device_type`: Type (TV, Phone, Tablet, Web, Desktop)
  - `platform`: Platform name (Tizen, iOS, Android, etc.)
  - `capabilities`: JSON object with device capabilities
  - `last_seen`: Last heartbeat timestamp
  - `is_online`: Current online status
  - `device_name`: Optional friendly name

#### 2. `/workspaces/media-gateway/crates/mcp-server/src/handlers.rs`
- Registered `list_devices` in `handle_tools_list()`
- Added tool executor case in `handle_tools_call()`

### Tool Definition

**Name:** `list_devices`

**Description:** List all registered devices for a user with their capabilities and status

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "user_id": {
      "type": "string",
      "description": "User UUID",
      "format": "uuid"
    }
  },
  "required": ["user_id"]
}
```

**Output Format:**
```json
{
  "user_id": "uuid",
  "device_count": 3,
  "devices": [
    {
      "device_id": "string",
      "device_type": "TV",
      "platform": "Tizen",
      "capabilities": {
        "max_resolution": "UHD_4K",
        "hdr_support": ["HDR10", "DolbyVision"],
        "audio_codecs": ["AAC", "DolbyAtmos"],
        "remote_controllable": true,
        "can_cast": false,
        "screen_size": 65.0
      },
      "last_seen": "2025-12-06T23:00:00Z",
      "is_online": true,
      "device_name": "Living Room TV"
    }
  ]
}
```

### Tests Added

#### 3. `/workspaces/media-gateway/crates/mcp-server/tests/integration_test.rs`

**Test Cases:**
1. `test_tools_list` - Updated to verify `list_devices` is in tool list
2. `test_list_devices_tool` - Tests listing devices with fixture data
3. `test_list_devices_tool_empty` - Tests empty result for user with no devices
4. `test_list_devices_tool_invalid_uuid` - Tests validation error for invalid UUID

#### 4. `/workspaces/media-gateway/crates/mcp-server/tests/fixtures/test_devices.sql`

**Test Fixture:**
- Created 3 test devices for user `123e4567-e89b-12d3-a456-426614174000`
- Devices: TV, Phone, Tablet with realistic capabilities
- Mixed online/offline status for testing

## Database Schema

Uses existing `user_devices` table from migration `016_sync_schema.sql`:

```sql
CREATE TABLE IF NOT EXISTS user_devices (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    device_id VARCHAR(255) NOT NULL,
    device_type VARCHAR(50) NOT NULL,
    platform VARCHAR(50) NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '{}',
    app_version VARCHAR(50) NOT NULL,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_online BOOLEAN NOT NULL DEFAULT false,
    device_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_user_device UNIQUE(user_id, device_id)
);
```

## Example Usage

### JSON-RPC Request
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_devices",
    "arguments": {
      "user_id": "123e4567-e89b-12d3-a456-426614174000"
    }
  }
}
```

### JSON-RPC Response
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"user_id\": \"123e4567-e89b-12d3-a456-426614174000\",\n  \"device_count\": 3,\n  \"devices\": [...]\n}"
      }
    ],
    "is_error": false
  }
}
```

## Build Verification

```bash
# Check compilation
cargo check -p media-gateway-mcp

# Build package
cargo build -p media-gateway-mcp

# Run tests (requires database)
cargo test -p media-gateway-mcp
```

## Integration Points

- **MCP Protocol**: Follows JSON-RPC 2.0 specification
- **Database**: Uses sqlx with PostgreSQL
- **Error Handling**: Proper validation and database error handling
- **Logging**: Instrumented with tracing for observability
- **Type Safety**: Strongly typed with Rust structs and sqlx::FromRow

## Related Files

- `/workspaces/media-gateway/migrations/016_sync_schema.sql` - Database schema
- `/workspaces/media-gateway/crates/mcp-server/src/protocol.rs` - MCP protocol types
- `/workspaces/media-gateway/plans/batches/BATCH_010_TASKS.md` - Task specification

## Status

✅ **COMPLETED**
- Implementation complete
- Tests added with fixtures
- Code compiles successfully
- Follows existing patterns in codebase
- Ready for integration testing
