-- Drop triggers
DROP TRIGGER IF EXISTS trigger_update_playback_progress_calculated_fields ON playback_progress;
DROP TRIGGER IF EXISTS trigger_update_playback_progress_timestamp ON playback_progress;

-- Drop functions
DROP FUNCTION IF EXISTS update_playback_progress_calculated_fields();
DROP FUNCTION IF EXISTS update_playback_progress_timestamp();

-- Drop table
DROP TABLE IF EXISTS playback_progress;
