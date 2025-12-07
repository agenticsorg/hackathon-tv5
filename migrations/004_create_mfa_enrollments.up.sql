-- Create mfa_enrollments table for storing MFA secrets and backup codes
CREATE TABLE IF NOT EXISTS mfa_enrollments (
    id UUID PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL UNIQUE,
    encrypted_secret BYTEA NOT NULL,
    backup_codes_hash TEXT[] NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ
);

-- Index for fast user lookup
CREATE INDEX IF NOT EXISTS idx_mfa_enrollments_user_id ON mfa_enrollments(user_id);

-- Index for verified enrollments
CREATE INDEX IF NOT EXISTS idx_mfa_enrollments_verified ON mfa_enrollments(is_verified) WHERE is_verified = TRUE;
