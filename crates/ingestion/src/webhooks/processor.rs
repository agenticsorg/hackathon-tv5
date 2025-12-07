//! Webhook processor that integrates with ingestion pipeline

use crate::{
    events::{ContentEvent, ContentIngestedEvent, ContentUpdatedEvent, EventProducer},
    normalizer::{AvailabilityInfo, CanonicalContent, ContentType, ImageSet, RawContent},
    repository::ContentRepository,
    webhooks::{
        ProcessedWebhook, ProcessingStatus, WebhookDeduplicator, WebhookError, WebhookEventType,
        WebhookPayload, WebhookResult,
    },
};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

/// Webhook processor that integrates webhooks with the ingestion pipeline
pub struct WebhookProcessor {
    repository: Arc<dyn ContentRepository>,
    event_producer: Option<Arc<dyn EventProducer>>,
}

impl WebhookProcessor {
    /// Create a new webhook processor
    pub fn new(
        repository: Arc<dyn ContentRepository>,
        event_producer: Option<Arc<dyn EventProducer>>,
    ) -> Self {
        Self {
            repository,
            event_producer,
        }
    }

    /// Process a webhook event and integrate with ingestion pipeline
    pub async fn process_webhook(
        &self,
        webhook: WebhookPayload,
    ) -> WebhookResult<ProcessedWebhook> {
        let event_id = WebhookDeduplicator::compute_hash(&webhook);

        tracing::info!(
            "Processing webhook: platform={} event_type={:?} event_id={}",
            webhook.platform,
            webhook.event_type,
            event_id
        );

        match webhook.event_type {
            WebhookEventType::ContentAdded => self.handle_content_added(&webhook, &event_id).await,
            WebhookEventType::ContentUpdated => {
                self.handle_content_updated(&webhook, &event_id).await
            }
            WebhookEventType::ContentRemoved => {
                self.handle_content_removed(&webhook, &event_id).await
            }
        }
    }

    /// Handle content added event
    async fn handle_content_added(
        &self,
        webhook: &WebhookPayload,
        event_id: &str,
    ) -> WebhookResult<ProcessedWebhook> {
        let platform_content_id = webhook
            .payload
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WebhookError::InvalidPayload("Missing content_id".to_string()))?;

        let title = webhook
            .payload
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled");

        let content_type_str = webhook
            .payload
            .get("content_type")
            .and_then(|v| v.as_str())
            .unwrap_or("movie");

        let content_type = match content_type_str {
            "movie" => ContentType::Movie,
            "series" | "tv_show" => ContentType::Series,
            "episode" => ContentType::Episode,
            "short" => ContentType::Short,
            "documentary" => ContentType::Documentary,
            _ => ContentType::Movie,
        };

        let regions = webhook
            .payload
            .get("regions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(|| vec!["US".to_string()]);

        let canonical = CanonicalContent {
            platform_content_id: platform_content_id.to_string(),
            platform_id: webhook.platform.clone(),
            entity_id: None,
            title: title.to_string(),
            overview: webhook
                .payload
                .get("overview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            content_type,
            release_year: webhook
                .payload
                .get("year")
                .and_then(|v| v.as_i64())
                .map(|y| y as i32),
            runtime_minutes: webhook
                .payload
                .get("runtime")
                .and_then(|v| v.as_i64())
                .map(|r| r as i32),
            genres: vec![],
            external_ids: std::collections::HashMap::new(),
            availability: AvailabilityInfo {
                regions,
                subscription_required: true,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: Some(Utc::now()),
                available_until: None,
            },
            images: ImageSet::default(),
            rating: None,
            user_rating: None,
            embedding: None,
            updated_at: Utc::now(),
        };

        let content_id = match self.repository.upsert(&canonical).await {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Failed to upsert content: {}", e);
                return Ok(ProcessedWebhook {
                    event_id: event_id.to_string(),
                    webhook: webhook.clone(),
                    processed_at: Utc::now(),
                    status: ProcessingStatus::Failed,
                    error: Some(format!("Database error: {}", e)),
                });
            }
        };

        tracing::info!(
            "Content added: platform={} platform_content_id={} content_id={}",
            webhook.platform,
            platform_content_id,
            content_id
        );

        if let Some(producer) = &self.event_producer {
            let event = ContentEvent::Ingested(ContentIngestedEvent::new(
                content_id,
                webhook.platform.clone(),
                platform_content_id.to_string(),
                content_type_str.to_string(),
                title.to_string(),
                vec!["title".to_string(), "content_type".to_string()],
            ));

            if let Err(e) = producer.publish_event(event).await {
                tracing::warn!("Failed to publish content ingested event: {}", e);
            } else {
                tracing::debug!(
                    "Published content ingested event for content_id={}",
                    content_id
                );
            }
        }

        Ok(ProcessedWebhook {
            event_id: event_id.to_string(),
            webhook: webhook.clone(),
            processed_at: Utc::now(),
            status: ProcessingStatus::Completed,
            error: None,
        })
    }

    /// Handle content updated event
    async fn handle_content_updated(
        &self,
        webhook: &WebhookPayload,
        event_id: &str,
    ) -> WebhookResult<ProcessedWebhook> {
        let platform_content_id = webhook
            .payload
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WebhookError::InvalidPayload("Missing content_id".to_string()))?;

        let content_id = match self
            .repository
            .find_by_platform_id(platform_content_id, &webhook.platform)
            .await
        {
            Ok(Some(id)) => id,
            Ok(None) => {
                tracing::warn!(
                    "Content not found for update: platform={} platform_content_id={}",
                    webhook.platform,
                    platform_content_id
                );
                return Ok(ProcessedWebhook {
                    event_id: event_id.to_string(),
                    webhook: webhook.clone(),
                    processed_at: Utc::now(),
                    status: ProcessingStatus::Failed,
                    error: Some("Content not found".to_string()),
                });
            }
            Err(e) => {
                tracing::error!("Failed to lookup content: {}", e);
                return Ok(ProcessedWebhook {
                    event_id: event_id.to_string(),
                    webhook: webhook.clone(),
                    processed_at: Utc::now(),
                    status: ProcessingStatus::Failed,
                    error: Some(format!("Database error: {}", e)),
                });
            }
        };

        tracing::info!(
            "Content updated: platform={} platform_content_id={} content_id={}",
            webhook.platform,
            platform_content_id,
            content_id
        );

        if let Some(producer) = &self.event_producer {
            let updated_fields: Vec<String> = webhook
                .payload
                .as_object()
                .map(|obj| obj.keys().map(|k| k.to_string()).collect())
                .unwrap_or_default();

            let event = ContentEvent::Updated(ContentUpdatedEvent::new(
                content_id,
                updated_fields,
                "webhook".to_string(),
            ));

            if let Err(e) = producer.publish_event(event).await {
                tracing::warn!("Failed to publish content updated event: {}", e);
            } else {
                tracing::debug!(
                    "Published content updated event for content_id={}",
                    content_id
                );
            }
        }

        Ok(ProcessedWebhook {
            event_id: event_id.to_string(),
            webhook: webhook.clone(),
            processed_at: Utc::now(),
            status: ProcessingStatus::Completed,
            error: None,
        })
    }

    /// Handle content removed event
    async fn handle_content_removed(
        &self,
        webhook: &WebhookPayload,
        event_id: &str,
    ) -> WebhookResult<ProcessedWebhook> {
        let platform_content_id = webhook
            .payload
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WebhookError::InvalidPayload("Missing content_id".to_string()))?;

        tracing::info!(
            "Content removed: platform={} platform_content_id={}",
            webhook.platform,
            platform_content_id
        );

        Ok(ProcessedWebhook {
            event_id: event_id.to_string(),
            webhook: webhook.clone(),
            processed_at: Utc::now(),
            status: ProcessingStatus::Completed,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::MockEventProducer;
    use crate::repository::PostgresContentRepository;
    use sqlx::PgPool;

    async fn setup_test_processor() -> Option<WebhookProcessor> {
        let database_url = std::env::var("DATABASE_URL").ok()?;
        let pool = PgPool::connect(&database_url).await.ok()?;

        let repository = Arc::new(PostgresContentRepository::new(pool));
        let event_producer = Arc::new(MockEventProducer::new(
            crate::events::KafkaConfig::from_env().ok()?,
        ));

        Some(WebhookProcessor::new(repository, Some(event_producer)))
    }

    #[tokio::test]
    async fn test_process_content_added_webhook() {
        if let Some(processor) = setup_test_processor().await {
            let webhook = WebhookPayload {
                event_type: WebhookEventType::ContentAdded,
                platform: "netflix".to_string(),
                timestamp: Utc::now(),
                payload: serde_json::json!({
                    "content_id": "test-processor-123",
                    "title": "Processor Test Movie",
                    "content_type": "movie",
                    "year": 2024
                }),
                signature: "sha256=test".to_string(),
            };

            let result = processor.process_webhook(webhook).await;
            assert!(result.is_ok());

            let processed = result.unwrap();
            assert_eq!(processed.status, ProcessingStatus::Completed);
            assert!(processed.error.is_none());
        }
    }
}
