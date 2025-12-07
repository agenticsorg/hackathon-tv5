-- Migration: Playback Progress and Continue Watching
-- Description: Stores user playback progress for cross-device sync and continue watching feature
-- Date: 2025-12-06

-- Create playback_progress table
CREATE TABLE IF NOT EXISTS playback_progress (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    content_id UUID NOT NULL,
    platform_id VARCHAR(50) NOT NULL,
    progress_seconds INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER NOT NULL,
    progress_percentage FLOAT NOT NULL DEFAULT 0.0,
    last_position_ms BIGINT NOT NULL DEFAULT 0,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    device_id UUID,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_user_content_platform UNIQUE(user_id, content_id, platform_id),
    CONSTRAINT valid_progress CHECK (progress_seconds >= 0),
    CONSTRAINT valid_duration CHECK (duration_seconds > 0),
    CONSTRAINT valid_percentage CHECK (progress_percentage >= 0.0 AND progress_percentage <= 100.0)
);

-- Index for fetching user's continue watching list (incomplete items first, ordered by last watched)
CREATE INDEX idx_progress_user_incomplete ON playback_progress(user_id, updated_at DESC)
    WHERE NOT is_completed;

-- Index for fetching all user progress (including completed)
CREATE INDEX idx_progress_user_all ON playback_progress(user_id, updated_at DESC);

-- Index for cleanup of stale progress (>30 days)
CREATE INDEX idx_progress_stale ON playback_progress(updated_at)
    WHERE updated_at < NOW() - INTERVAL '30 days';

-- Index for content-specific queries
CREATE INDEX idx_progress_content ON playback_progress(content_id);

-- Index for device-specific queries
CREATE INDEX idx_progress_device ON playback_progress(device_id)
    WHERE device_id IS NOT NULL;

-- Function to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_playback_progress_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update timestamp on row update
CREATE TRIGGER trigger_update_playback_progress_timestamp
    BEFORE UPDATE ON playback_progress
    FOR EACH ROW
    EXECUTE FUNCTION update_playback_progress_timestamp();

-- Function to automatically calculate progress percentage and completion status
CREATE OR REPLACE FUNCTION update_playback_progress_calculated_fields()
RETURNS TRIGGER AS $$
BEGIN
    -- Calculate progress percentage
    IF NEW.duration_seconds > 0 THEN
        NEW.progress_percentage = (NEW.progress_seconds::FLOAT / NEW.duration_seconds::FLOAT) * 100.0;
    ELSE
        NEW.progress_percentage = 0.0;
    END IF;

    -- Calculate last_position_ms from progress_seconds
    NEW.last_position_ms = NEW.progress_seconds::BIGINT * 1000;

    -- Auto-mark as completed if progress >= 95%
    IF NEW.progress_percentage >= 95.0 THEN
        NEW.is_completed = TRUE;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to calculate fields before insert or update
CREATE TRIGGER trigger_update_playback_progress_calculated_fields
    BEFORE INSERT OR UPDATE ON playback_progress
    FOR EACH ROW
    EXECUTE FUNCTION update_playback_progress_calculated_fields();
