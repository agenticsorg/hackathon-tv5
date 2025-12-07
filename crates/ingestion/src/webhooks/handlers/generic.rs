//! Generic webhook handler for platforms without specific implementations

use crate::webhooks::{
    verify_hmac_signature, ProcessedWebhook, ProcessingStatus, WebhookDeduplicator, WebhookError,
    WebhookEventType, WebhookHandler, WebhookPayload, WebhookResult,
};
use async_trait::async_trait;
use chrono::Utc;

/// Generic webhook handler
pub struct GenericWebhookHandler {
    platform: String,
}

impl GenericWebhookHandler {
    /// Create a new generic webhook handler
    pub fn new(platform: String) -> Self {
        Self { platform }
    }
}

#[async_trait]
impl WebhookHandler for GenericWebhookHandler {
    fn platform_id(&self) -> &'static str {
        // Note: This is a limitation of the trait design
        // In production, you might want to use a different approach
        "generic"
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
        tracing::info!(
            "Processing generic webhook: platform={} event={:?}",
            webhook.platform,
            webhook.event_type
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_id() {
        let handler = GenericWebhookHandler::new("hulu".to_string());
        assert_eq!(handler.platform_id(), "generic");
    }

    #[test]
    fn test_parse_valid_payload() {
        let handler = GenericWebhookHandler::new("hulu".to_string());

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "hulu".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "id": "12345",
                "title": "Test Show"
            }),
            signature: "sha256=test".to_string(),
        };

        let body = serde_json::to_vec(&webhook).unwrap();
        let result = handler.parse_payload(&body);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.platform, "hulu");
        assert_eq!(parsed.event_type, WebhookEventType::ContentAdded);
    }

    #[test]
    fn test_parse_invalid_json() {
        let handler = GenericWebhookHandler::new("hulu".to_string());

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
        let handler = GenericWebhookHandler::new("hulu".to_string());

        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentUpdated,
            platform: "hulu".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "id": "12345",
                "title": "Updated Show"
            }),
            signature: "sha256=test".to_string(),
        };

        let result = handler.process_event(webhook).await;

        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.status, ProcessingStatus::Completed);
        assert!(processed.error.is_none());
    }

    #[test]
    fn test_verify_signature() {
        let handler = GenericWebhookHandler::new("hulu".to_string());

        let payload = b"test payload";
        let secret = "test-secret";

        // Generate valid signature
        use crate::webhooks::verification::generate_hmac_signature;
        let signature = generate_hmac_signature(payload, secret).unwrap();

        let result = handler.verify_signature(payload, &signature, secret);

        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
