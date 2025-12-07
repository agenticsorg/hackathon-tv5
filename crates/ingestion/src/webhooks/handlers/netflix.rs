//! Netflix webhook handler

use crate::webhooks::{
    verify_hmac_signature, ProcessedWebhook, ProcessingStatus, WebhookDeduplicator, WebhookError,
    WebhookEventType, WebhookHandler, WebhookPayload, WebhookResult,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Netflix webhook handler
pub struct NetflixWebhookHandler;

impl NetflixWebhookHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NetflixWebhookHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WebhookHandler for NetflixWebhookHandler {
    fn platform_id(&self) -> &'static str {
        "netflix"
    }

    fn verify_signature(
        &self,
        payload: &[u8],
        signature: &str,
        secret: &str,
    ) -> WebhookResult<bool> {
        verify_hmac_signature(payload, signature, secret)
    }

    fn parse_payload(&self, body: &[u8]) -> WebhookResult<WebhookPayload> {
        let payload: WebhookPayload = serde_json::from_slice(body)
            .map_err(|e| WebhookError::InvalidPayload(format!("Failed to parse JSON: {}", e)))?;

        // Validate platform
        if payload.platform != "netflix" {
            return Err(WebhookError::InvalidPayload(format!(
                "Invalid platform: expected 'netflix', got '{}'",
                payload.platform
            )));
        }

        // Validate event type
        match payload.event_type {
            WebhookEventType::ContentAdded
            | WebhookEventType::ContentUpdated
            | WebhookEventType::ContentRemoved => {}
        }

        // Validate payload structure
        if !payload.payload.is_object() {
            return Err(WebhookError::InvalidPayload(
                "Payload must be an object".to_string(),
            ));
        }

        Ok(payload)
    }

    async fn process_event(&self, webhook: WebhookPayload) -> WebhookResult<ProcessedWebhook> {
        // Extract Netflix-specific data
        let content_id = webhook
            .payload
            .get("content_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| WebhookError::InvalidPayload("Missing content_id".to_string()))?;

        tracing::info!(
            "Processing Netflix webhook: event={:?} content_id={}",
            webhook.event_type,
            content_id
        );

        let event_id = WebhookDeduplicator::compute_hash(&webhook);

        Ok(ProcessedWebhook {
            event_id,
            webhook,
            processed_at: Utc::now(),
            status: ProcessingStatus::Completed,
            error: None,
        })
    }
}

/// Netflix webhook payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetflixWebhookData {
    pub content_id: String,
    pub title: Option<String>,
    pub content_type: Option<String>,
    pub regions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_id() {
        let handler = NetflixWebhookHandler::new();
        assert_eq!(handler.platform_id(), "netflix");
    }

    #[test]
    fn test_parse_valid_payload() {
        let handler = NetflixWebhookHandler::new();

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "content_id": "12345",
                "title": "Test Movie",
                "content_type": "movie"
            }),
            signature: "sha256=test".to_string(),
        };

        let body = serde_json::to_vec(&webhook).unwrap();
        let result = handler.parse_payload(&body);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.platform, "netflix");
        assert_eq!(parsed.event_type, WebhookEventType::ContentAdded);
    }

    #[test]
    fn test_parse_invalid_platform() {
        let handler = NetflixWebhookHandler::new();

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "hulu".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"content_id": "12345"}),
            signature: "sha256=test".to_string(),
        };

        let body = serde_json::to_vec(&webhook).unwrap();
        let result = handler.parse_payload(&body);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WebhookError::InvalidPayload(_)
        ));
    }

    #[test]
    fn test_parse_invalid_json() {
        let handler = NetflixWebhookHandler::new();

        let body = b"invalid json";
        let result = handler.parse_payload(body);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WebhookError::InvalidPayload(_)
        ));
    }

    #[tokio::test]
    async fn test_process_event() {
        let handler = NetflixWebhookHandler::new();

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "content_id": "12345",
                "title": "Test Movie"
            }),
            signature: "sha256=test".to_string(),
        };

        let result = handler.process_event(webhook).await;

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.status, ProcessingStatus::Completed);
        assert!(processed.error.is_none());
    }

    #[tokio::test]
    async fn test_process_event_missing_content_id() {
        let handler = NetflixWebhookHandler::new();

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({"title": "Test Movie"}),
            signature: "sha256=test".to_string(),
        };

        let result = handler.process_event(webhook).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WebhookError::InvalidPayload(_)
        ));
    }
}
