//! Watch History Management
//!
//! Tracks user watch progress across sessions for resume functionality.
//! Implements resume position logic with PostgreSQL persistence.
//!
//! ## Database Schema
//!
//! ```sql
//! CREATE TABLE watch_history (
//!     id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
//!     user_id UUID NOT NULL,
//!     content_id UUID NOT NULL,
//!     resume_position_seconds INT NOT NULL,
//!     duration_seconds INT NOT NULL,
//!     last_watched_at TIMESTAMP NOT NULL DEFAULT NOW(),
//!     created_at TIMESTAMP NOT NULL DEFAULT NOW(),
//!     updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
//!     UNIQUE(user_id, content_id)
//! );
//!
//! CREATE INDEX idx_watch_history_user_id ON watch_history(user_id);
//! CREATE INDEX idx_watch_history_content_id ON watch_history(content_id);
//! CREATE INDEX idx_watch_history_last_watched ON watch_history(last_watched_at DESC);
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Watch history entry
#[derive(Debug, Clone)]
pub struct WatchHistoryEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content_id: Uuid,
    pub resume_position_seconds: u32,
    pub duration_seconds: u32,
    pub last_watched_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Watch history manager with PostgreSQL storage
pub struct WatchHistoryManager {
    pool: PgPool,
}

impl WatchHistoryManager {
    /// Create new watch history manager
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get resume position for a user's content
    ///
    /// Returns the position where the user should resume playback, or None if:
    /// - No watch history exists
    /// - Position is less than 30 seconds (start from beginning)
    /// - Position/duration > 0.95 (content already finished)
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `content_id` - The content's UUID
    ///
    /// # Returns
    /// * `Some(position)` - Resume position in seconds
    /// * `None` - Start from beginning
    pub async fn get_resume_position(
        &self,
        user_id: Uuid,
        content_id: Uuid,
    ) -> Result<Option<u32>> {
        let row = sqlx::query(
            r#"
            SELECT resume_position_seconds, duration_seconds
            FROM watch_history
            WHERE user_id = $1 AND content_id = $2
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query watch history")?;

        match row {
            Some(r) => {
                let position: i32 = r.try_get("resume_position_seconds")?;
                let duration: i32 = r.try_get("duration_seconds")?;

                let resume_pos = calculate_resume_position(position as u32, duration as u32);

                tracing::debug!(
                    "Retrieved watch history: user_id={}, content_id={}, position={}, resume={:?}",
                    user_id,
                    content_id,
                    position,
                    resume_pos
                );

                Ok(resume_pos)
            }
            None => {
                tracing::debug!(
                    "No watch history found: user_id={}, content_id={}",
                    user_id,
                    content_id
                );
                Ok(None)
            }
        }
    }

    /// Update watch history with current playback position
    ///
    /// Uses UPSERT (INSERT ... ON CONFLICT) to create or update history.
    /// Updates last_watched_at timestamp and resume position.
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `content_id` - The content's UUID
    /// * `position` - Current playback position in seconds
    /// * `duration` - Total content duration in seconds
    pub async fn update_watch_history(
        &self,
        user_id: Uuid,
        content_id: Uuid,
        position: u32,
        duration: u32,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO watch_history (
                user_id,
                content_id,
                resume_position_seconds,
                duration_seconds,
                last_watched_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, NOW(), NOW(), NOW())
            ON CONFLICT (user_id, content_id)
            DO UPDATE SET
                resume_position_seconds = EXCLUDED.resume_position_seconds,
                duration_seconds = EXCLUDED.duration_seconds,
                last_watched_at = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .bind(position as i32)
        .bind(duration as i32)
        .execute(&self.pool)
        .await
        .context("Failed to update watch history")?;

        tracing::debug!(
            "Updated watch history: user_id={}, content_id={}, position={}, duration={}",
            user_id,
            content_id,
            position,
            duration
        );

        Ok(())
    }

    /// Clear watch history for specific content
    ///
    /// Useful when user wants to restart content from beginning.
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `content_id` - The content's UUID
    ///
    /// # Returns
    /// `true` if history was deleted, `false` if none existed
    pub async fn clear_history(&self, user_id: Uuid, content_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM watch_history
            WHERE user_id = $1 AND content_id = $2
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .execute(&self.pool)
        .await
        .context("Failed to clear watch history")?;

        let deleted = result.rows_affected() > 0;

        tracing::info!(
            "Cleared watch history: user_id={}, content_id={}, deleted={}",
            user_id,
            content_id,
            deleted
        );

        Ok(deleted)
    }

    /// Get watch history entry by user and content
    pub async fn get_history(
        &self,
        user_id: Uuid,
        content_id: Uuid,
    ) -> Result<Option<WatchHistoryEntry>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                user_id,
                content_id,
                resume_position_seconds,
                duration_seconds,
                last_watched_at,
                created_at,
                updated_at
            FROM watch_history
            WHERE user_id = $1 AND content_id = $2
            "#,
        )
        .bind(user_id)
        .bind(content_id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to query watch history entry")?;

        match row {
            Some(r) => Ok(Some(WatchHistoryEntry {
                id: r.try_get("id")?,
                user_id: r.try_get("user_id")?,
                content_id: r.try_get("content_id")?,
                resume_position_seconds: r.try_get::<i32, _>("resume_position_seconds")? as u32,
                duration_seconds: r.try_get::<i32, _>("duration_seconds")? as u32,
                last_watched_at: r.try_get("last_watched_at")?,
                created_at: r.try_get("created_at")?,
                updated_at: r.try_get("updated_at")?,
            })),
            None => Ok(None),
        }
    }

    /// Get all watch history for a user
    ///
    /// Returns entries sorted by last_watched_at (newest first)
    pub async fn get_user_history(&self, user_id: Uuid) -> Result<Vec<WatchHistoryEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                user_id,
                content_id,
                resume_position_seconds,
                duration_seconds,
                last_watched_at,
                created_at,
                updated_at
            FROM watch_history
            WHERE user_id = $1
            ORDER BY last_watched_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query user watch history")?;

        let entries = rows
            .into_iter()
            .map(|r| {
                Ok(WatchHistoryEntry {
                    id: r.try_get("id")?,
                    user_id: r.try_get("user_id")?,
                    content_id: r.try_get("content_id")?,
                    resume_position_seconds: r.try_get::<i32, _>("resume_position_seconds")? as u32,
                    duration_seconds: r.try_get::<i32, _>("duration_seconds")? as u32,
                    last_watched_at: r.try_get("last_watched_at")?,
                    created_at: r.try_get("created_at")?,
                    updated_at: r.try_get("updated_at")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }
}

/// Calculate resume position based on current position and duration
///
/// Resume position logic:
/// - Returns `None` if position < 30 seconds (start over)
/// - Returns `None` if position/duration > 0.95 (already finished, 95% threshold)
/// - Otherwise returns `Some(position)` to resume playback
///
/// # Arguments
/// * `position` - Current playback position in seconds
/// * `duration` - Total content duration in seconds
///
/// # Returns
/// * `Some(position)` - Resume from this position
/// * `None` - Start from beginning
///
/// # Examples
/// ```
/// use media_gateway_playback::watch_history::calculate_resume_position;
///
/// // Too early, start from beginning
/// assert_eq!(calculate_resume_position(20, 3600), None);
///
/// // Good resume position
/// assert_eq!(calculate_resume_position(1800, 3600), Some(1800));
///
/// // Already finished (>95%), start from beginning
/// assert_eq!(calculate_resume_position(3500, 3600), None);
/// ```
pub fn calculate_resume_position(position: u32, duration: u32) -> Option<u32> {
    // Avoid division by zero
    if duration == 0 {
        return None;
    }

    // If position < 30 seconds, start from beginning
    if position < 30 {
        tracing::trace!("Position {} < 30s, starting from beginning", position);
        return None;
    }

    // If already watched >95%, start from beginning
    let completion_ratio = position as f32 / duration as f32;
    if completion_ratio > 0.95 {
        tracing::trace!(
            "Completion ratio {:.2}% > 95%, starting from beginning",
            completion_ratio * 100.0
        );
        return None;
    }

    // Resume from saved position
    tracing::trace!(
        "Resuming at {} seconds ({:.1}% completion)",
        position,
        completion_ratio * 100.0
    );
    Some(position)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_resume_position_too_early() {
        // Position < 30 seconds should start from beginning
        assert_eq!(calculate_resume_position(0, 3600), None);
        assert_eq!(calculate_resume_position(15, 3600), None);
        assert_eq!(calculate_resume_position(29, 3600), None);
    }

    #[test]
    fn test_calculate_resume_position_valid() {
        // Valid resume positions
        assert_eq!(calculate_resume_position(30, 3600), Some(30));
        assert_eq!(calculate_resume_position(1800, 3600), Some(1800)); // 50%
        assert_eq!(calculate_resume_position(3000, 3600), Some(3000)); // 83.3%
    }

    #[test]
    fn test_calculate_resume_position_almost_finished() {
        // >95% completion should start from beginning
        assert_eq!(calculate_resume_position(3420, 3600), None); // 95%
        assert_eq!(calculate_resume_position(3500, 3600), None); // 97.2%
        assert_eq!(calculate_resume_position(3600, 3600), None); // 100%
    }

    #[test]
    fn test_calculate_resume_position_edge_cases() {
        // Zero duration
        assert_eq!(calculate_resume_position(100, 0), None);

        // Just before 95% threshold
        assert_eq!(calculate_resume_position(3419, 3600), Some(3419)); // 94.97%

        // Exactly at 95%
        assert_eq!(calculate_resume_position(3420, 3600), None);
    }

    #[test]
    fn test_calculate_resume_position_short_content() {
        // Short content (1 minute)
        assert_eq!(calculate_resume_position(10, 60), None); // < 30s
        assert_eq!(calculate_resume_position(30, 60), Some(30)); // Valid
        assert_eq!(calculate_resume_position(57, 60), None); // 95%
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn create_test_pool() -> Result<PgPool> {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .context("Failed to connect to test database")?;

        Ok(pool)
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test --ignored
    async fn test_update_and_get_resume_position() -> Result<()> {
        let pool = create_test_pool().await?;
        let manager = WatchHistoryManager::new(pool);

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Initially no history
        let resume = manager.get_resume_position(user_id, content_id).await?;
        assert_eq!(resume, None);

        // Update to 50% watched
        manager
            .update_watch_history(user_id, content_id, 1800, 3600)
            .await?;

        // Should resume at 1800
        let resume = manager.get_resume_position(user_id, content_id).await?;
        assert_eq!(resume, Some(1800));

        // Update to 96% watched
        manager
            .update_watch_history(user_id, content_id, 3456, 3600)
            .await?;

        // Should start from beginning (>95%)
        let resume = manager.get_resume_position(user_id, content_id).await?;
        assert_eq!(resume, None);

        // Cleanup
        manager.clear_history(user_id, content_id).await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_clear_history() -> Result<()> {
        let pool = create_test_pool().await?;
        let manager = WatchHistoryManager::new(pool);

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Create history
        manager
            .update_watch_history(user_id, content_id, 1800, 3600)
            .await?;

        // Verify it exists
        let resume = manager.get_resume_position(user_id, content_id).await?;
        assert_eq!(resume, Some(1800));

        // Clear history
        let deleted = manager.clear_history(user_id, content_id).await?;
        assert!(deleted);

        // Verify it's gone
        let resume = manager.get_resume_position(user_id, content_id).await?;
        assert_eq!(resume, None);

        // Clearing again should return false
        let deleted = manager.clear_history(user_id, content_id).await?;
        assert!(!deleted);

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_history_entry() -> Result<()> {
        let pool = create_test_pool().await?;
        let manager = WatchHistoryManager::new(pool);

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Create history
        manager
            .update_watch_history(user_id, content_id, 1800, 3600)
            .await?;

        // Get full entry
        let entry = manager.get_history(user_id, content_id).await?;
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.user_id, user_id);
        assert_eq!(entry.content_id, content_id);
        assert_eq!(entry.resume_position_seconds, 1800);
        assert_eq!(entry.duration_seconds, 3600);

        // Cleanup
        manager.clear_history(user_id, content_id).await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_user_history() -> Result<()> {
        let pool = create_test_pool().await?;
        let manager = WatchHistoryManager::new(pool);

        let user_id = Uuid::new_v4();
        let content1 = Uuid::new_v4();
        let content2 = Uuid::new_v4();
        let content3 = Uuid::new_v4();

        // Create multiple history entries
        manager
            .update_watch_history(user_id, content1, 1800, 3600)
            .await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        manager
            .update_watch_history(user_id, content2, 900, 1800)
            .await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        manager
            .update_watch_history(user_id, content3, 300, 600)
            .await?;

        // Get all user history
        let history = manager.get_user_history(user_id).await?;
        assert_eq!(history.len(), 3);

        // Verify sorted by last_watched_at (newest first)
        assert_eq!(history[0].content_id, content3);
        assert_eq!(history[1].content_id, content2);
        assert_eq!(history[2].content_id, content1);

        // Cleanup
        manager.clear_history(user_id, content1).await?;
        manager.clear_history(user_id, content2).await?;
        manager.clear_history(user_id, content3).await?;

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_upsert_behavior() -> Result<()> {
        let pool = create_test_pool().await?;
        let manager = WatchHistoryManager::new(pool);

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // First update
        manager
            .update_watch_history(user_id, content_id, 1000, 3600)
            .await?;

        let entry1 = manager.get_history(user_id, content_id).await?.unwrap();
        assert_eq!(entry1.resume_position_seconds, 1000);

        // Second update (should update, not insert)
        manager
            .update_watch_history(user_id, content_id, 2000, 3600)
            .await?;

        let entry2 = manager.get_history(user_id, content_id).await?.unwrap();
        assert_eq!(entry2.resume_position_seconds, 2000);
        assert_eq!(entry2.id, entry1.id); // Same ID means UPDATE, not INSERT

        // Verify only one entry exists
        let history = manager.get_user_history(user_id).await?;
        assert_eq!(history.len(), 1);

        // Cleanup
        manager.clear_history(user_id, content_id).await?;

        Ok(())
    }
}
