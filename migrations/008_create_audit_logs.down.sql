-- Drop function
DROP FUNCTION IF EXISTS cleanup_old_audit_logs(INTEGER);

-- Drop indexes
DROP INDEX IF EXISTS idx_audit_logs_user_timestamp;
DROP INDEX IF EXISTS idx_audit_logs_resource_type;
DROP INDEX IF EXISTS idx_audit_logs_action;
DROP INDEX IF EXISTS idx_audit_logs_user_id;
DROP INDEX IF EXISTS idx_audit_logs_timestamp;

-- Drop table
DROP TABLE IF EXISTS audit_logs;
