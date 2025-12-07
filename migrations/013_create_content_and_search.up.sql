-- Migration: Content and Search History Tables
-- Description: Create content catalog and search history tracking
-- Date: 2025-12-06

-- Create content table
CREATE TABLE IF NOT EXISTS content (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    url TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    views BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Create indexes on content
CREATE INDEX idx_content_title ON content USING GIN(to_tsvector('english', title));
CREATE INDEX idx_content_type ON content(content_type) WHERE deleted_at IS NULL;
CREATE INDEX idx_content_views ON content(views DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_content_created ON content(created_at DESC) WHERE deleted_at IS NULL;

-- Create search_history table
CREATE TABLE IF NOT EXISTS search_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    query TEXT NOT NULL,
    content_type VARCHAR(100),
    results_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes on search_history
CREATE INDEX idx_search_history_user ON search_history(user_id, created_at DESC);
CREATE INDEX idx_search_history_query ON search_history(query);

-- Create playback_sessions table (alias for playback_progress for test compatibility)
CREATE TABLE IF NOT EXISTS playback_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    content_id UUID NOT NULL,
    position_seconds INTEGER NOT NULL DEFAULT 0,
    duration_seconds INTEGER,
    is_completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes on playback_sessions
CREATE INDEX idx_playback_sessions_user ON playback_sessions(user_id, updated_at DESC);
CREATE INDEX idx_playback_sessions_content ON playback_sessions(content_id, updated_at DESC);
CREATE INDEX idx_playback_sessions_user_content ON playback_sessions(user_id, content_id, updated_at DESC);

-- Function to update content updated_at timestamp
CREATE OR REPLACE FUNCTION update_content_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_content_updated_at
    BEFORE UPDATE ON content
    FOR EACH ROW
    EXECUTE FUNCTION update_content_updated_at();

-- Function to update playback_sessions updated_at timestamp
CREATE OR REPLACE FUNCTION update_playback_sessions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_playback_sessions_updated_at
    BEFORE UPDATE ON playback_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_playback_sessions_updated_at();
