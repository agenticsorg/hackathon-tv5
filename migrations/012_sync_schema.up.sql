-- Sync Service Database Schema
-- Media Gateway - Cross-device synchronization with CRDT persistence
--
-- Run this migration to enable sync service persistence
-- Dependencies: User management (assumes users table exists)

-- ============================================================================
-- User Watchlists Table (OR-Set CRDT)
-- ============================================================================
-- Stores watchlist items with OR-Set semantics (add-wins bias)

CREATE TABLE IF NOT EXISTS user_watchlists (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    content_id VARCHAR(255) NOT NULL,
    unique_tag VARCHAR(255) NOT NULL,
    timestamp_physical BIGINT NOT NULL,
    timestamp_logical INTEGER NOT NULL,
    device_id VARCHAR(255) NOT NULL,
    is_removed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_tag UNIQUE(user_id, unique_tag)
);

CREATE INDEX idx_watchlists_user ON user_watchlists(user_id);
CREATE INDEX idx_watchlists_content ON user_watchlists(content_id);
CREATE INDEX idx_watchlists_removed ON user_watchlists(is_removed);
CREATE INDEX idx_watchlists_device ON user_watchlists(device_id);

COMMENT ON TABLE user_watchlists IS 'User watchlists with OR-Set CRDT semantics';
COMMENT ON COLUMN user_watchlists.unique_tag IS 'Unique tag for OR-Set addition (UUID)';
COMMENT ON COLUMN user_watchlists.timestamp_physical IS 'HLC physical time (milliseconds)';
COMMENT ON COLUMN user_watchlists.timestamp_logical IS 'HLC logical counter';
COMMENT ON COLUMN user_watchlists.is_removed IS 'Marks tag as removed in OR-Set';

-- ============================================================================
-- User Progress Table (LWW-Register CRDT)
-- ============================================================================
-- Tracks watch progress with last-writer-wins conflict resolution

CREATE TABLE IF NOT EXISTS user_progress (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    content_id VARCHAR(255) NOT NULL,
    position_seconds INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    state VARCHAR(50) NOT NULL DEFAULT 'stopped',
    timestamp_physical BIGINT NOT NULL,
    timestamp_logical INTEGER NOT NULL,
    device_id VARCHAR(255) NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_content UNIQUE(user_id, content_id),
    CONSTRAINT valid_position CHECK (position_seconds >= 0),
    CONSTRAINT valid_duration CHECK (duration_seconds >= 0),
    CONSTRAINT valid_state CHECK (state IN ('playing', 'paused', 'stopped'))
);

CREATE INDEX idx_progress_user ON user_progress(user_id);
CREATE INDEX idx_progress_content ON user_progress(content_id);
CREATE INDEX idx_progress_state ON user_progress(state);
CREATE INDEX idx_progress_updated ON user_progress(updated_at);

COMMENT ON TABLE user_progress IS 'User playback progress with LWW-Register CRDT';
COMMENT ON COLUMN user_progress.timestamp_physical IS 'HLC physical time for LWW resolution';
COMMENT ON COLUMN user_progress.timestamp_logical IS 'HLC logical counter for tie-breaking';
COMMENT ON COLUMN user_progress.state IS 'Playback state: playing, paused, or stopped';

-- ============================================================================
-- User Devices Table
-- ============================================================================
-- Stores registered devices with capabilities and presence tracking

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

    CONSTRAINT unique_user_device UNIQUE(user_id, device_id),
    CONSTRAINT valid_device_type CHECK (device_type IN ('TV', 'Phone', 'Tablet', 'Web', 'Desktop'))
);

CREATE INDEX idx_devices_user ON user_devices(user_id);
CREATE INDEX idx_devices_online ON user_devices(is_online);
CREATE INDEX idx_devices_last_seen ON user_devices(last_seen);
CREATE INDEX idx_devices_type ON user_devices(device_type);

COMMENT ON TABLE user_devices IS 'Registered user devices with capabilities';
COMMENT ON COLUMN user_devices.capabilities IS 'Device capabilities JSON (resolution, HDR, codecs)';
COMMENT ON COLUMN user_devices.last_seen IS 'Last heartbeat timestamp';
COMMENT ON COLUMN user_devices.is_online IS 'Current online status (updated by heartbeat)';

-- ============================================================================
-- Helper Views
-- ============================================================================

-- View: Effective watchlist (OR-Set result)
CREATE OR REPLACE VIEW effective_watchlists AS
SELECT
    user_id,
    content_id,
    MIN(timestamp_physical) as first_added_physical,
    MAX(timestamp_physical) as last_updated_physical,
    COUNT(*) as addition_count
FROM user_watchlists
WHERE is_removed = false
GROUP BY user_id, content_id;

COMMENT ON VIEW effective_watchlists IS 'Effective watchlist items (OR-Set: additions - removals)';

-- View: In-progress content
CREATE OR REPLACE VIEW in_progress_content AS
SELECT
    user_id,
    content_id,
    position_seconds,
    duration_seconds,
    (position_seconds::FLOAT / NULLIF(duration_seconds, 0)::FLOAT) as completion_percent,
    state,
    device_id,
    updated_at
FROM user_progress
WHERE
    position_seconds > 0
    AND (position_seconds::FLOAT / NULLIF(duration_seconds, 0)::FLOAT) < 0.9
ORDER BY updated_at DESC;

COMMENT ON VIEW in_progress_content IS 'Content in progress (<90% complete, position > 0)';

-- View: Completed content
CREATE OR REPLACE VIEW completed_content AS
SELECT
    user_id,
    content_id,
    position_seconds,
    duration_seconds,
    (position_seconds::FLOAT / NULLIF(duration_seconds, 0)::FLOAT) as completion_percent,
    device_id,
    updated_at
FROM user_progress
WHERE (position_seconds::FLOAT / NULLIF(duration_seconds, 0)::FLOAT) >= 0.9
ORDER BY updated_at DESC;

COMMENT ON VIEW completed_content IS 'Completed content (>=90% watched)';

-- View: Online devices summary
CREATE OR REPLACE VIEW online_devices_summary AS
SELECT
    user_id,
    COUNT(*) as total_devices,
    COUNT(*) FILTER (WHERE is_online = true) as online_count,
    COUNT(*) FILTER (WHERE device_type = 'TV') as tv_count,
    COUNT(*) FILTER (WHERE device_type = 'Phone') as phone_count,
    COUNT(*) FILTER (WHERE device_type = 'Tablet') as tablet_count,
    MAX(last_seen) as most_recent_activity
FROM user_devices
GROUP BY user_id;

COMMENT ON VIEW online_devices_summary IS 'Per-user device statistics';

-- ============================================================================
-- Functions
-- ============================================================================

-- Function: Clean up stale device sessions (offline if no heartbeat in 5 minutes)
CREATE OR REPLACE FUNCTION mark_stale_devices_offline()
RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    UPDATE user_devices
    SET is_online = false
    WHERE is_online = true
      AND last_seen < NOW() - INTERVAL '5 minutes';

    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION mark_stale_devices_offline IS 'Mark devices offline if no heartbeat in 5 minutes';

-- Function: Clean up old removed watchlist entries (garbage collection)
CREATE OR REPLACE FUNCTION cleanup_removed_watchlist_entries(days_old INTEGER DEFAULT 30)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM user_watchlists
    WHERE is_removed = true
      AND created_at < NOW() - (days_old || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_removed_watchlist_entries IS 'Delete old removed watchlist entries (default: 30 days)';
