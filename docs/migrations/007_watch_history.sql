-- Migration: Watch History Table
-- Description: Tracks user watch progress for resume functionality
-- Date: 2024-12-06
-- Task: BATCH_004 TASK-011

-- Create watch_history table
CREATE TABLE IF NOT EXISTS watch_history (
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

-- Create indexes for efficient queries
CREATE INDEX idx_watch_history_user_id ON watch_history(user_id);
CREATE INDEX idx_watch_history_content_id ON watch_history(content_id);
CREATE INDEX idx_watch_history_last_watched ON watch_history(last_watched_at DESC);

-- Add comment
COMMENT ON TABLE watch_history IS 'Stores user watch progress for resume functionality across sessions';
COMMENT ON COLUMN watch_history.resume_position_seconds IS 'Last playback position in seconds';
COMMENT ON COLUMN watch_history.duration_seconds IS 'Total content duration in seconds';
COMMENT ON COLUMN watch_history.last_watched_at IS 'Timestamp of last watch session';

-- Rollback
-- DROP INDEX IF EXISTS idx_watch_history_last_watched;
-- DROP INDEX IF EXISTS idx_watch_history_content_id;
-- DROP INDEX IF EXISTS idx_watch_history_user_id;
-- DROP TABLE IF EXISTS watch_history;
