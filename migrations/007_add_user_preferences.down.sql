-- Drop audit_log indexes
DROP INDEX IF EXISTS idx_audit_log_action;
DROP INDEX IF EXISTS idx_audit_log_created_at;
DROP INDEX IF EXISTS idx_audit_log_user_id;

-- Drop oauth_providers indexes
DROP INDEX IF EXISTS idx_oauth_providers_provider;
DROP INDEX IF EXISTS idx_oauth_providers_user_id;

-- Drop tables
DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS oauth_providers;

-- Drop users table columns and indexes
DROP INDEX IF EXISTS idx_users_preferences;
ALTER TABLE users DROP COLUMN IF EXISTS preferences;
ALTER TABLE users DROP COLUMN IF EXISTS avatar_url;
