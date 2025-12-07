-- Drop index
DROP INDEX IF EXISTS idx_users_parental_controls_enabled;

-- Drop column
ALTER TABLE users DROP COLUMN IF EXISTS parental_controls;
