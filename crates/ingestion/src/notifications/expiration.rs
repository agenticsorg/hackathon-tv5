//! Content Expiration Notification System
//!
//! This module provides scheduled jobs to detect expiring content and emit
//! notifications through Kafka events. It tracks notification history to prevent
//! duplicate notifications for the same expiration window.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::events::{EventError, EventProducer, KafkaEventProducer};
use crate::repository::{ContentRepository, ExpiringContent, PostgresContentRepository};

/// Configuration for expiration notification windows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpirationNotificationConfig {
    /// Notification windows in days (e.g., [7, 3, 1] for 7 days, 3 days, 1 day before expiry)
    pub notification_windows: Vec<i64>,

    /// Whether to enable email notifications (future feature)
    pub enable_email: bool,

    /// Whether to enable Kafka event notifications
    pub enable_kafka: bool,

    /// How often to run the expiration check job (in seconds)
    pub check_interval_seconds: u64,
}

impl Default for ExpirationNotificationConfig {
    fn default() -> Self {
        Self {
            notification_windows: vec![7, 3, 1],
            enable_email: false,
            enable_kafka: true,
            check_interval_seconds: 3600, // Run every hour
        }
    }
}

/// Notification window for expiring content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationWindow {
    /// 7 days before expiration
    SevenDays,
    /// 3 days before expiration
    ThreeDays,
    /// 1 day before expiration
    OneDay,
    /// Custom number of days
    Custom(i64),
}

impl NotificationWindow {
    /// Get the duration for this notification window
    pub fn duration(&self) -> Duration {
        let days = match self {
            NotificationWindow::SevenDays => 7,
            NotificationWindow::ThreeDays => 3,
            NotificationWindow::OneDay => 1,
            NotificationWindow::Custom(d) => *d,
        };
        Duration::days(days)
    }

    /// Get the window identifier for database storage
    pub fn identifier(&self) -> String {
        match self {
            NotificationWindow::SevenDays => "7d".to_string(),
            NotificationWindow::ThreeDays => "3d".to_string(),
            NotificationWindow::OneDay => "1d".to_string(),
            NotificationWindow::Custom(d) => format!("{}d", d),
        }
    }

    /// Create from days value
    pub fn from_days(days: i64) -> Self {
        match days {
            7 => NotificationWindow::SevenDays,
            3 => NotificationWindow::ThreeDays,
            1 => NotificationWindow::OneDay,
            d => NotificationWindow::Custom(d),
        }
    }
}

/// Status of a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStatus {
    pub content_id: Uuid,
    pub window: NotificationWindow,
    pub notified_at: DateTime<Utc>,
    pub platform: String,
    pub region: String,
    pub expires_at: DateTime<Utc>,
}

/// Kafka event for content expiring notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentExpiringEvent {
    /// Type of event
    pub event_type: String,

    /// Content identifier
    pub content_id: Uuid,

    /// Content title
    pub title: String,

    /// Platform where content is expiring
    pub platform: String,

    /// Region where content is expiring
    pub region: String,

    /// When the content expires
    pub expires_at: DateTime<Utc>,

    /// Days until expiration
    pub days_until_expiration: i64,

    /// Notification window that triggered this event
    pub notification_window: String,

    /// Timestamp of the notification
    pub timestamp: DateTime<Utc>,

    /// Correlation ID for tracking
    pub correlation_id: Uuid,
}

impl ContentExpiringEvent {
    /// Create a new content expiring event
    pub fn new(content: &ExpiringContent, window: NotificationWindow) -> Self {
        let now = Utc::now();
        let days_until = (content.expires_at - now).num_days();

        Self {
            event_type: "content.expiring".to_string(),
            content_id: content.content_id,
            title: content.title.clone(),
            platform: content.platform.clone(),
            region: content.region.clone(),
            expires_at: content.expires_at,
            days_until_expiration: days_until,
            notification_window: window.identifier(),
            timestamp: now,
            correlation_id: Uuid::new_v4(),
        }
    }
}

/// Expiration notification job
pub struct ExpirationNotificationJob {
    pool: PgPool,
    repository: Arc<PostgresContentRepository>,
    producer: Option<Arc<KafkaEventProducer>>,
    config: ExpirationNotificationConfig,
}

impl ExpirationNotificationJob {
    /// Create a new expiration notification job
    pub fn new(pool: PgPool, config: ExpirationNotificationConfig) -> Result<Self, EventError> {
        let repository = Arc::new(PostgresContentRepository::new(pool.clone()));

        let producer = if config.enable_kafka {
            Some(Arc::new(KafkaEventProducer::from_env()?))
        } else {
            None
        };

        Ok(Self {
            pool,
            repository,
            producer,
            config,
        })
    }

    /// Create with custom producer (for testing)
    pub fn with_producer(
        pool: PgPool,
        producer: Arc<KafkaEventProducer>,
        config: ExpirationNotificationConfig,
    ) -> Self {
        let repository = Arc::new(PostgresContentRepository::new(pool.clone()));

        Self {
            pool,
            repository,
            producer: Some(producer),
            config,
        }
    }

    /// Initialize the notification tracking table if it doesn't exist
    pub async fn initialize_tracking_table(&self) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
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
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_expiration_notifications_content
            ON expiration_notifications(content_id, notification_window)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_expiration_notifications_notified
            ON expiration_notifications(notified_at DESC)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check for expiring content and send notifications
    pub async fn check_and_notify(&self) -> Result<usize, anyhow::Error> {
        info!("Running expiration notification check");

        let mut total_notifications = 0;

        for &days in &self.config.notification_windows {
            let window = NotificationWindow::from_days(days);
            let count = self.check_window(window).await?;
            total_notifications += count;
        }

        info!(
            total_notifications = total_notifications,
            "Completed expiration notification check"
        );

        Ok(total_notifications)
    }

    /// Check a specific notification window
    async fn check_window(&self, window: NotificationWindow) -> Result<usize, anyhow::Error> {
        let duration = window.duration();

        debug!(
            window = ?window,
            days = duration.num_days(),
            "Checking notification window"
        );

        // Find content expiring within this window
        let expiring_content = self
            .repository
            .find_expiring_within(duration)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find expiring content: {}", e))?;

        debug!(
            count = expiring_content.len(),
            window = ?window,
            "Found expiring content"
        );

        let mut notification_count = 0;

        for content in expiring_content {
            // Check if already notified for this window
            let already_notified = self.is_already_notified(&content, window).await?;

            if already_notified {
                debug!(
                    content_id = %content.content_id,
                    title = %content.title,
                    window = ?window,
                    "Already notified, skipping"
                );
                continue;
            }

            // Send notification
            match self.send_notification(&content, window).await {
                Ok(_) => {
                    // Mark as notified
                    self.mark_as_notified(&content, window).await?;
                    notification_count += 1;

                    info!(
                        content_id = %content.content_id,
                        title = %content.title,
                        platform = %content.platform,
                        region = %content.region,
                        expires_at = %content.expires_at,
                        window = ?window,
                        "Sent expiration notification"
                    );
                }
                Err(e) => {
                    error!(
                        content_id = %content.content_id,
                        title = %content.title,
                        error = %e,
                        "Failed to send notification"
                    );
                }
            }
        }

        Ok(notification_count)
    }

    /// Check if content has already been notified for this window
    async fn is_already_notified(
        &self,
        content: &ExpiringContent,
        window: NotificationWindow,
    ) -> Result<bool, anyhow::Error> {
        let exists = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM expiration_notifications
                WHERE content_id = $1
                  AND platform = $2
                  AND region = $3
                  AND notification_window = $4
                  AND expires_at = $5
            )
            "#,
        )
        .bind(content.content_id)
        .bind(&content.platform)
        .bind(&content.region)
        .bind(window.identifier())
        .bind(content.expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    /// Mark content as notified for this window
    async fn mark_as_notified(
        &self,
        content: &ExpiringContent,
        window: NotificationWindow,
    ) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            INSERT INTO expiration_notifications (
                content_id, platform, region, notification_window, expires_at
            )
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (content_id, platform, region, notification_window, expires_at)
            DO NOTHING
            "#,
        )
        .bind(content.content_id)
        .bind(&content.platform)
        .bind(&content.region)
        .bind(window.identifier())
        .bind(content.expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Send notification for expiring content
    async fn send_notification(
        &self,
        content: &ExpiringContent,
        window: NotificationWindow,
    ) -> Result<(), anyhow::Error> {
        // Create Kafka event
        if self.config.enable_kafka {
            if let Some(producer) = &self.producer {
                let event = ContentExpiringEvent::new(content, window);
                let event_json = serde_json::to_string(&event)?;

                // Publish to Kafka (using mock for now)
                // In production, this would use actual Kafka producer
                debug!(
                    event = %event_json,
                    "Publishing content expiring event"
                );

                // For now, log the event
                // Future: producer.publish_event(event).await?;
            }
        }

        // Future: Send email notifications
        if self.config.enable_email {
            warn!("Email notifications not yet implemented");
        }

        Ok(())
    }

    /// Get notification history for content
    pub async fn get_notification_history(
        &self,
        content_id: Uuid,
    ) -> Result<Vec<NotificationStatus>, anyhow::Error> {
        let results =
            sqlx::query_as::<_, (Uuid, String, String, String, DateTime<Utc>, DateTime<Utc>)>(
                r#"
            SELECT
                content_id,
                platform,
                region,
                notification_window,
                expires_at,
                notified_at
            FROM expiration_notifications
            WHERE content_id = $1
            ORDER BY notified_at DESC
            "#,
            )
            .bind(content_id)
            .fetch_all(&self.pool)
            .await?;

        let history = results
            .into_iter()
            .map(
                |(content_id, platform, region, window_str, expires_at, notified_at)| {
                    let window = if window_str == "7d" {
                        NotificationWindow::SevenDays
                    } else if window_str == "3d" {
                        NotificationWindow::ThreeDays
                    } else if window_str == "1d" {
                        NotificationWindow::OneDay
                    } else {
                        // Parse custom format like "14d"
                        let days = window_str.trim_end_matches('d').parse().unwrap_or(1);
                        NotificationWindow::Custom(days)
                    };

                    NotificationStatus {
                        content_id,
                        window,
                        notified_at,
                        platform,
                        region,
                        expires_at,
                    }
                },
            )
            .collect();

        Ok(history)
    }

    /// Cleanup old notification records (older than 90 days)
    pub async fn cleanup_old_notifications(&self) -> Result<u64, anyhow::Error> {
        let cutoff = Utc::now() - Duration::days(90);

        let result = sqlx::query("DELETE FROM expiration_notifications WHERE notified_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await?;

        info!(
            deleted_count = result.rows_affected(),
            cutoff_date = %cutoff,
            "Cleaned up old notification records"
        );

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_window_duration() {
        assert_eq!(NotificationWindow::SevenDays.duration().num_days(), 7);
        assert_eq!(NotificationWindow::ThreeDays.duration().num_days(), 3);
        assert_eq!(NotificationWindow::OneDay.duration().num_days(), 1);
        assert_eq!(NotificationWindow::Custom(14).duration().num_days(), 14);
    }

    #[test]
    fn test_notification_window_identifier() {
        assert_eq!(NotificationWindow::SevenDays.identifier(), "7d");
        assert_eq!(NotificationWindow::ThreeDays.identifier(), "3d");
        assert_eq!(NotificationWindow::OneDay.identifier(), "1d");
        assert_eq!(NotificationWindow::Custom(14).identifier(), "14d");
    }

    #[test]
    fn test_notification_window_from_days() {
        assert_eq!(
            NotificationWindow::from_days(7),
            NotificationWindow::SevenDays
        );
        assert_eq!(
            NotificationWindow::from_days(3),
            NotificationWindow::ThreeDays
        );
        assert_eq!(NotificationWindow::from_days(1), NotificationWindow::OneDay);
        assert_eq!(
            NotificationWindow::from_days(14),
            NotificationWindow::Custom(14)
        );
    }

    #[test]
    fn test_expiration_notification_config_default() {
        let config = ExpirationNotificationConfig::default();
        assert_eq!(config.notification_windows, vec![7, 3, 1]);
        assert!(!config.enable_email);
        assert!(config.enable_kafka);
        assert_eq!(config.check_interval_seconds, 3600);
    }
}
