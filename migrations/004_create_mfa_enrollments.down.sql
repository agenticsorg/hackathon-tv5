-- Drop indexes
DROP INDEX IF EXISTS idx_mfa_enrollments_verified;
DROP INDEX IF EXISTS idx_mfa_enrollments_user_id;

-- Drop table
DROP TABLE IF EXISTS mfa_enrollments;
