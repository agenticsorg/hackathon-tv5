-- Migration: Expiration Notifications Tracking
-- Description: Create table to track content expiration notifications
-- Date: 2025-12-06

-- Create expiration_notifications table to track sent notifications
CREATE TABLE IF NOT EXISTS expiration_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID NOT NULL,
    platform VARCHAR(100) NOT NULL,
    region VARCHAR(10) NOT NULL,
    notification_window VARCHAR(10) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    notified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(content_id, platform, region, notification_window, expires_at)
);

-- Create indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_expiration_notifications_content
ON expiration_notifications(content_id, notification_window);

CREATE INDEX IF NOT EXISTS idx_expiration_notifications_notified
ON expiration_notifications(notified_at DESC);

CREATE INDEX IF NOT EXISTS idx_expiration_notifications_expires
ON expiration_notifications(expires_at);

-- Add comments for documentation
COMMENT ON TABLE expiration_notifications IS 'Tracks expiration notifications sent for content to prevent duplicates';
COMMENT ON COLUMN expiration_notifications.content_id IS 'Content that was notified about';
COMMENT ON COLUMN expiration_notifications.platform IS 'Platform where content is expiring';
COMMENT ON COLUMN expiration_notifications.region IS 'Region where content is expiring';
COMMENT ON COLUMN expiration_notifications.notification_window IS 'Window identifier (e.g., 7d, 3d, 1d)';
COMMENT ON COLUMN expiration_notifications.expires_at IS 'When the content expires';
COMMENT ON COLUMN expiration_notifications.notified_at IS 'When the notification was sent';
