//! Continue watching service with cross-device sync integration

use crate::progress::{ProgressRecord, ProgressRepository, UpdateProgressRequest};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

const DEFAULT_LIMIT: i64 = 20;
const COMPLETION_THRESHOLD: f32 = 95.0;

/// Continue watching item with metadata
#[derive(Debug, Serialize, Clone)]
pub struct ContinueWatchingItem {
    pub content_id: Uuid,
    pub title: String,
    pub platform: String,
    pub progress_percentage: f32,
    pub progress_seconds: i32,
    pub duration_seconds: i32,
    pub last_watched: DateTime<Utc>,
    pub resume_position_ms: i64,
}

/// Continue watching list response
#[derive(Debug, Serialize)]
pub struct ContinueWatchingResponse {
    pub items: Vec<ContinueWatchingItem>,
    pub total: usize,
}

/// Request to update progress
#[derive(Debug, Deserialize)]
pub struct ProgressUpdateRequest {
    pub content_id: Uuid,
    pub platform_id: String,
    pub progress_seconds: i32,
    pub duration_seconds: i32,
    pub device_id: Option<Uuid>,
}

/// Progress update response
#[derive(Debug, Serialize)]
pub struct ProgressUpdateResponse {
    pub content_id: Uuid,
    pub progress_percentage: f32,
    pub is_completed: bool,
    pub updated_at: DateTime<Utc>,
}

/// Content metadata provider trait for fetching content details
#[async_trait::async_trait]
pub trait ContentMetadataProvider: Send + Sync {
    async fn get_content_title(&self, content_id: Uuid, platform: &str) -> Result<String, String>;
}

/// HTTP-based content metadata provider
pub struct HttpContentMetadataProvider {
    http_client: reqwest::Client,
    catalog_service_url: String,
}

impl HttpContentMetadataProvider {
    pub fn new(catalog_service_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            catalog_service_url,
        }
    }
}

#[async_trait::async_trait]
impl ContentMetadataProvider for HttpContentMetadataProvider {
    async fn get_content_title(&self, content_id: Uuid, platform: &str) -> Result<String, String> {
        let url = format!(
            "{}/api/v1/content/{}/metadata?platform={}",
            self.catalog_service_url, content_id, platform
        );

        match self.http_client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                match resp.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(title) = json.get("title").and_then(|t| t.as_str()) {
                            Ok(title.to_string())
                        } else {
                            Ok(format!("Unknown Content {}", content_id))
                        }
                    }
                    Err(_) => Ok(format!("Unknown Content {}", content_id)),
                }
            }
            _ => Ok(format!("Unknown Content {}", content_id)),
        }
    }
}

/// Sync service integration for cross-device sync
pub struct SyncServiceClient {
    http_client: reqwest::Client,
    sync_service_url: String,
}

impl SyncServiceClient {
    pub fn new(sync_service_url: String) -> Self {
        Self {
            http_client: reqwest::Client::new(),
            sync_service_url,
        }
    }

    /// Notify sync service of progress update (fire-and-forget)
    pub async fn notify_progress_update(&self, user_id: Uuid, record: &ProgressRecord) {
        let progress_data = serde_json::json!({
            "user_id": user_id,
            "content_id": record.content_id,
            "platform_id": record.platform_id,
            "progress_seconds": record.progress_seconds,
            "duration_seconds": record.duration_seconds,
            "device_id": record.device_id,
            "timestamp": record.updated_at.to_rfc3339(),
        });

        let url = format!("{}/api/v1/sync/progress", self.sync_service_url);
        let client = self.http_client.clone();

        tokio::spawn(async move {
            match client.post(&url).json(&progress_data).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::debug!("Sync service notified of progress update");
                }
                Ok(resp) => {
                    tracing::warn!("Sync service responded with status: {}", resp.status());
                }
                Err(e) => {
                    tracing::warn!("Failed to notify sync service: {}", e);
                }
            }
        });
    }
}

/// Continue watching service
pub struct ContinueWatchingService {
    progress_repo: ProgressRepository,
    metadata_provider: Arc<dyn ContentMetadataProvider>,
    sync_client: Option<Arc<SyncServiceClient>>,
}

impl ContinueWatchingService {
    /// Create new continue watching service
    pub fn new(pool: PgPool, metadata_provider: Arc<dyn ContentMetadataProvider>) -> Self {
        Self {
            progress_repo: ProgressRepository::new(pool),
            metadata_provider,
            sync_client: None,
        }
    }

    /// Add sync service integration
    pub fn with_sync_service(mut self, sync_client: Arc<SyncServiceClient>) -> Self {
        self.sync_client = Some(sync_client);
        self
    }

    /// Get continue watching list for a user
    pub async fn get_continue_watching(
        &self,
        user_id: Uuid,
        limit: Option<i64>,
    ) -> Result<ContinueWatchingResponse, ContinueWatchingError> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT);
        let records = self
            .progress_repo
            .get_user_incomplete_progress(user_id, limit)
            .await
            .map_err(|e| ContinueWatchingError::Storage(e.to_string()))?;

        let mut items = Vec::new();
        for record in records {
            let title = self
                .metadata_provider
                .get_content_title(record.content_id, &record.platform_id)
                .await
                .unwrap_or_else(|_| format!("Unknown Content {}", record.content_id));

            items.push(ContinueWatchingItem {
                content_id: record.content_id,
                title,
                platform: record.platform_id,
                progress_percentage: record.progress_percentage,
                progress_seconds: record.progress_seconds,
                duration_seconds: record.duration_seconds,
                last_watched: record.updated_at,
                resume_position_ms: record.last_position_ms,
            });
        }

        let total = items.len();

        Ok(ContinueWatchingResponse { items, total })
    }

    /// Update playback progress with sync service notification
    pub async fn update_progress(
        &self,
        user_id: Uuid,
        request: ProgressUpdateRequest,
    ) -> Result<ProgressUpdateResponse, ContinueWatchingError> {
        let update_request = UpdateProgressRequest {
            user_id,
            content_id: request.content_id,
            platform_id: request.platform_id,
            progress_seconds: request.progress_seconds,
            duration_seconds: request.duration_seconds,
            device_id: request.device_id,
        };

        let record = self
            .progress_repo
            .upsert_progress(update_request)
            .await
            .map_err(|e| ContinueWatchingError::Storage(e.to_string()))?;

        // Notify sync service
        if let Some(sync_client) = &self.sync_client {
            sync_client.notify_progress_update(user_id, &record).await;
        }

        Ok(ProgressUpdateResponse {
            content_id: record.content_id,
            progress_percentage: record.progress_percentage,
            is_completed: record.is_completed,
            updated_at: record.updated_at,
        })
    }

    /// Cleanup stale progress (scheduled task)
    pub async fn cleanup_stale_progress(&self, days: i32) -> Result<u64, ContinueWatchingError> {
        self.progress_repo
            .cleanup_stale_progress(days)
            .await
            .map_err(|e| ContinueWatchingError::Storage(e.to_string()))
    }
}

/// Continue watching service errors
#[derive(Debug, thiserror::Error)]
pub enum ContinueWatchingError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("External service error: {0}")]
    ExternalService(String),
}

impl actix_web::ResponseError for ContinueWatchingError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            ContinueWatchingError::InvalidRequest(msg) => actix_web::HttpResponse::BadRequest()
                .json(serde_json::json!({
                    "error": msg
                })),
            _ => actix_web::HttpResponse::InternalServerError().json(serde_json::json!({
                "error": self.to_string()
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    /// Mock content metadata provider for testing
    pub struct MockContentMetadataProvider;

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

    async fn setup_test_db() -> PgPool {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
        });

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database")
    }

    async fn cleanup_test_data(pool: &PgPool, user_id: Uuid) {
        sqlx::query("DELETE FROM playback_progress WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_get_continue_watching_empty() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();
        let result = service.get_continue_watching(user_id, None).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.total, 0);
        assert!(response.items.is_empty());
    }

    #[tokio::test]
    async fn test_update_progress_creates_record() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = ProgressUpdateRequest {
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: Some(Uuid::new_v4()),
        };

        let result = service.update_progress(user_id, request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.content_id, content_id);
        assert_eq!(response.progress_percentage, 20.0);
        assert!(!response.is_completed);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_update_progress_marks_completed() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = ProgressUpdateRequest {
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 5700,
            duration_seconds: 6000,
            device_id: None,
        };

        let result = service.update_progress(user_id, request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.progress_percentage, 95.0);
        assert!(response.is_completed);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_get_continue_watching_with_items() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();

        for i in 0..3 {
            let request = ProgressUpdateRequest {
                content_id: Uuid::new_v4(),
                platform_id: "netflix".to_string(),
                progress_seconds: 1000 + (i * 100),
                duration_seconds: 6000,
                device_id: None,
            };
            service.update_progress(user_id, request).await.unwrap();
        }

        let result = service.get_continue_watching(user_id, Some(10)).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.total, 3);
        assert_eq!(response.items.len(), 3);

        for item in &response.items {
            assert_eq!(item.platform, "netflix");
            assert!(item.progress_percentage < COMPLETION_THRESHOLD);
            assert_eq!(item.resume_position_ms, item.progress_seconds as i64 * 1000);
        }

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_continue_watching_excludes_completed() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();

        let incomplete_request = ProgressUpdateRequest {
            content_id: Uuid::new_v4(),
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: None,
        };
        service
            .update_progress(user_id, incomplete_request)
            .await
            .unwrap();

        let completed_request = ProgressUpdateRequest {
            content_id: Uuid::new_v4(),
            platform_id: "hulu".to_string(),
            progress_seconds: 5700,
            duration_seconds: 6000,
            device_id: None,
        };
        service
            .update_progress(user_id, completed_request)
            .await
            .unwrap();

        let result = service.get_continue_watching(user_id, None).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items[0].platform, "netflix");

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_cleanup_stale_progress() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request = ProgressUpdateRequest {
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 5700,
            duration_seconds: 6000,
            device_id: None,
        };

        service.update_progress(user_id, request).await.unwrap();

        sqlx::query(
            "UPDATE playback_progress SET updated_at = NOW() - INTERVAL '31 days' WHERE user_id = $1"
        )
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

        let result = service.cleanup_stale_progress(30).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_continue_watching_ordering() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();
        let mut content_ids = Vec::new();

        for _ in 0..3 {
            let content_id = Uuid::new_v4();
            content_ids.push(content_id);

            let request = ProgressUpdateRequest {
                content_id,
                platform_id: "netflix".to_string(),
                progress_seconds: 1200,
                duration_seconds: 6000,
                device_id: None,
            };
            service.update_progress(user_id, request).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let result = service.get_continue_watching(user_id, None).await.unwrap();
        assert_eq!(result.total, 3);

        assert_eq!(result.items[0].content_id, content_ids[2]);
        assert_eq!(result.items[1].content_id, content_ids[1]);
        assert_eq!(result.items[2].content_id, content_ids[0]);

        cleanup_test_data(&pool, user_id).await;
    }

    #[tokio::test]
    async fn test_update_progress_updates_existing() {
        let pool = setup_test_db().await;
        let service =
            ContinueWatchingService::new(pool.clone(), Arc::new(MockContentMetadataProvider));

        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();

        let request1 = ProgressUpdateRequest {
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1200,
            duration_seconds: 6000,
            device_id: None,
        };
        service.update_progress(user_id, request1).await.unwrap();

        let request2 = ProgressUpdateRequest {
            content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 3600,
            duration_seconds: 6000,
            device_id: None,
        };
        let result = service.update_progress(user_id, request2).await.unwrap();

        assert_eq!(result.progress_percentage, 60.0);

        let continue_watching = service.get_continue_watching(user_id, None).await.unwrap();
        assert_eq!(continue_watching.total, 1);
        assert_eq!(continue_watching.items[0].progress_seconds, 3600);

        cleanup_test_data(&pool, user_id).await;
    }
}
