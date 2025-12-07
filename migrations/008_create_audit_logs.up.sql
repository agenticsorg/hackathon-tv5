-- Migration: Create audit_logs table
-- Description: Audit logging system for tracking user actions and system events

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id UUID,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    details JSONB NOT NULL DEFAULT '{}',
    ip_address INET,
    user_agent TEXT
);

-- Indexes for query performance
CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
CREATE INDEX IF NOT EXISTS idx_audit_logs_resource_type ON audit_logs(resource_type);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_audit_logs_user_timestamp ON audit_logs(user_id, timestamp DESC);

-- Function to automatically clean up old audit logs (retention policy)
-- Default retention: 90 days (configurable)
-- Usage: SELECT cleanup_old_audit_logs();        -- Uses default 90 days
--        SELECT cleanup_old_audit_logs(30);      -- Custom 30 days retention
-- Note: This function should be called periodically by a background job
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs(retention_days INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM audit_logs
    WHERE timestamp < NOW() - (retention_days || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Comment on table and columns
COMMENT ON TABLE audit_logs IS 'Audit trail for all user actions and system events';
COMMENT ON COLUMN audit_logs.id IS 'Unique identifier for the audit log entry';
COMMENT ON COLUMN audit_logs.timestamp IS 'When the action occurred';
COMMENT ON COLUMN audit_logs.user_id IS 'ID of the user who performed the action (NULL for system actions)';
COMMENT ON COLUMN audit_logs.action IS 'Type of action performed (e.g., AUTH_LOGIN, USER_CREATED)';
COMMENT ON COLUMN audit_logs.resource_type IS 'Type of resource affected (e.g., user, content, api_key)';
COMMENT ON COLUMN audit_logs.resource_id IS 'ID of the specific resource affected';
COMMENT ON COLUMN audit_logs.details IS 'Additional context and metadata about the action';
COMMENT ON COLUMN audit_logs.ip_address IS 'IP address of the client that initiated the action';
COMMENT ON COLUMN audit_logs.user_agent IS 'User agent string of the client';
