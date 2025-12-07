//! Background cleanup task for stale playback progress

use crate::continue_watching::ContinueWatchingService;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

const CLEANUP_INTERVAL_HOURS: u64 = 24;
const STALE_PROGRESS_DAYS: i32 = 30;

/// Background task that periodically cleans up stale progress
pub async fn run_cleanup_task(service: Arc<ContinueWatchingService>) {
    let mut interval = time::interval(Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600));

    info!(
        "Starting cleanup task: interval={}h, stale_threshold={}d",
        CLEANUP_INTERVAL_HOURS, STALE_PROGRESS_DAYS
    );

    loop {
        interval.tick().await;

        info!("Running stale progress cleanup...");

        match service.cleanup_stale_progress(STALE_PROGRESS_DAYS).await {
            Ok(deleted_count) => {
                info!(
                    "Cleanup completed successfully: deleted {} stale progress records",
                    deleted_count
                );
            }
            Err(e) => {
                error!("Cleanup task failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::continue_watching::{ContentMetadataProvider, ContinueWatchingService};
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    /// Mock content metadata provider for testing
    struct MockContentMetadataProvider;

    #[async_trait::async_trait]
    impl ContentMetadataProvider for MockContentMetadataProvider {
        async fn get_content_title(
            &self,
            content_id: Uuid,
            platform: &str,
        ) -> Result<String, String> {
            Ok(format!("Content {} on {}", content_id, platform))
        }
    }

    async fn setup_test_db() -> sqlx::PgPool {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    #[tokio::test]
    async fn test_cleanup_task_deletes_old_records() {
        let pool = setup_test_db().await;
        let service = Arc::new(ContinueWatchingService::new(
            pool.clone(),
            Arc::new(MockContentMetadataProvider),
        ));

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Create completed progress
        let request = crate::continue_watching::ProgressUpdateRequest {
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 5700,
            duration_seconds: 6000,
            device_id: None,
        };
        service.update_progress(user_id, request).await.unwrap();

        // Make it stale
        sqlx::query(
            "UPDATE playback_progress SET updated_at = NOW() - INTERVAL '31 days' WHERE user_id = $1"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        // Run cleanup
        let deleted = service
            .cleanup_stale_progress(STALE_PROGRESS_DAYS)
            .await
            .unwrap();
        assert!(deleted > 0);

        // Cleanup test data
        sqlx::query("DELETE FROM playback_progress WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_cleanup_task_preserves_recent_records() {
        let pool = setup_test_db().await;
        let service = Arc::new(ContinueWatchingService::new(
            pool.clone(),
            Arc::new(MockContentMetadataProvider),
        ));

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        // Create recent progress
        let request = crate::continue_watching::ProgressUpdateRequest {
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: None,
        };
        service.update_progress(user_id, request).await.unwrap();

        // Run cleanup
        let deleted = service
            .cleanup_stale_progress(STALE_PROGRESS_DAYS)
            .await
            .unwrap();

        // Should not delete recent records
        let continue_watching = service.get_continue_watching(user_id, None).await.unwrap();
        assert_eq!(continue_watching.total, 1);

        // Cleanup test data
        sqlx::query("DELETE FROM playback_progress WHERE user_id = $1")
            .bind(user_id)
            .execute(&pool)
            .await
            .ok();
    }
}
