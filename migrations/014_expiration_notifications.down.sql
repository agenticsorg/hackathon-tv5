-- Drop indexes
DROP INDEX IF EXISTS idx_expiration_notifications_expires;
DROP INDEX IF EXISTS idx_expiration_notifications_notified;
DROP INDEX IF EXISTS idx_expiration_notifications_content;

-- Drop table
DROP TABLE IF EXISTS expiration_notifications;
