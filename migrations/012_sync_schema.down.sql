-- Drop functions
DROP FUNCTION IF EXISTS cleanup_removed_watchlist_entries(INTEGER);
DROP FUNCTION IF EXISTS mark_stale_devices_offline();

-- Drop views
DROP VIEW IF EXISTS online_devices_summary;
DROP VIEW IF EXISTS completed_content;
DROP VIEW IF EXISTS in_progress_content;
DROP VIEW IF EXISTS effective_watchlists;

-- Drop indexes
DROP INDEX IF EXISTS idx_devices_type;
DROP INDEX IF EXISTS idx_devices_last_seen;
DROP INDEX IF EXISTS idx_devices_online;
DROP INDEX IF EXISTS idx_devices_user;
DROP INDEX IF EXISTS idx_progress_updated;
DROP INDEX IF EXISTS idx_progress_state;
DROP INDEX IF EXISTS idx_progress_content;
DROP INDEX IF EXISTS idx_progress_user;
DROP INDEX IF EXISTS idx_watchlists_device;
DROP INDEX IF EXISTS idx_watchlists_removed;
DROP INDEX IF EXISTS idx_watchlists_content;
DROP INDEX IF EXISTS idx_watchlists_user;

-- Drop tables
DROP TABLE IF EXISTS user_devices;
DROP TABLE IF EXISTS user_progress;
DROP TABLE IF EXISTS user_watchlists;
