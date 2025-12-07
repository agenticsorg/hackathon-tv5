-- Drop triggers
DROP TRIGGER IF EXISTS trigger_playback_sessions_updated_at ON playback_sessions;
DROP TRIGGER IF EXISTS trigger_content_updated_at ON content;

-- Drop functions
DROP FUNCTION IF EXISTS update_playback_sessions_updated_at();
DROP FUNCTION IF EXISTS update_content_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_playback_sessions_user_content;
DROP INDEX IF EXISTS idx_playback_sessions_content;
DROP INDEX IF EXISTS idx_playback_sessions_user;
DROP INDEX IF EXISTS idx_search_history_query;
DROP INDEX IF EXISTS idx_search_history_user;
DROP INDEX IF EXISTS idx_content_created;
DROP INDEX IF EXISTS idx_content_views;
DROP INDEX IF EXISTS idx_content_type;
DROP INDEX IF EXISTS idx_content_title;

-- Drop tables
DROP TABLE IF EXISTS playback_sessions;
DROP TABLE IF EXISTS search_history;
DROP TABLE IF EXISTS content;
