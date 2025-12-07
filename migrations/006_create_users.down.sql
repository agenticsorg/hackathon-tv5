-- Drop trigger
DROP TRIGGER IF EXISTS trigger_users_updated_at ON users;

-- Drop function
DROP FUNCTION IF EXISTS update_users_updated_at();

-- Drop indexes
DROP INDEX IF EXISTS idx_users_active;
DROP INDEX IF EXISTS idx_users_email;

-- Drop table
DROP TABLE IF EXISTS users;
