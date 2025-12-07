//! Integration tests for webhook system
//!
//! These tests require Redis to be running.
//! Set REDIS_URL environment variable or use default: redis://localhost:6379

use chrono::Utc;
use media_gateway_ingestion::webhooks::{
    handlers::{GenericWebhookHandler, NetflixWebhookHandler},
    verification::generate_hmac_signature,
};
use media_gateway_ingestion::{
    ContentEvent, EventProducer, PlatformWebhookConfig, RedisWebhookQueue, WebhookDeduplicator,
    WebhookEventType, WebhookHandler, WebhookMetrics, WebhookPayload, WebhookProcessor,
    WebhookQueue, WebhookReceiver,
};
use std::sync::Arc;

fn get_redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
}

async fn setup_receiver() -> Option<Arc<WebhookReceiver>> {
    let redis_url = get_redis_url();

    let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
        Ok(q) => Arc::new(q) as Arc<dyn WebhookQueue>,
        Err(_) => {
            eprintln!("Redis not available - skipping test");
            return None;
        }
    };

    let deduplicator = match WebhookDeduplicator::new(&redis_url, Some(1)) {
        Ok(d) => Arc::new(d),
        Err(_) => return None,
    };

    let metrics = Arc::new(WebhookMetrics::new());
    Some(Arc::new(WebhookReceiver::new(queue, deduplicator, metrics)))
}

#[tokio::test]
async fn test_end_to_end_webhook_flow() {
    let receiver = match setup_receiver().await {
        Some(r) => r,
        None => return,
    };

    // Register Netflix handler
    let handler = Box::new(NetflixWebhookHandler::new());
    let config = PlatformWebhookConfig {
        platform: "netflix".to_string(),
        secret: "test-secret-e2e".to_string(),
        rate_limit: 100,
        enabled: true,
    };

    receiver.register_handler(handler, config).await.unwrap();

    // Create webhook payload
    let webhook = WebhookPayload {
        event_type: WebhookEventType::ContentAdded,
        platform: "netflix".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "content_id": "e2e-test-12345",
            "title": "Test Movie",
            "content_type": "movie"
        }),
        signature: "sha256=placeholder".to_string(),
    };

    let body = serde_json::to_vec(&webhook).unwrap();
    let signature = generate_hmac_signature(&body, "test-secret-e2e").unwrap();

    // Receive webhook
    let event_id = receiver
        .receive("netflix", &body, &signature)
        .await
        .unwrap();
    assert!(!event_id.is_empty());

    // Check metrics
    let metrics = receiver.metrics();
    let snapshot = metrics.snapshot();
    assert!(snapshot.received > 0);

    // Clean up
    let redis_client = redis::Client::open(get_redis_url()).unwrap();
    let mut conn = redis_client.get_async_connection().await.unwrap();
    let key = format!("webhook:hash:{}", event_id);
    let _: () = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_duplicate_webhook_rejection() {
    let receiver = match setup_receiver().await {
        Some(r) => r,
        None => return,
    };

    // Register handler
    let handler = Box::new(NetflixWebhookHandler::new());
    let config = PlatformWebhookConfig {
        platform: "netflix".to_string(),
        secret: "test-secret-dup".to_string(),
        rate_limit: 100,
        enabled: true,
    };

    receiver.register_handler(handler, config).await.unwrap();

    // Create webhook
    let webhook = WebhookPayload {
        event_type: WebhookEventType::ContentAdded,
        platform: "netflix".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "content_id": "dup-test-12345",
            "title": "Duplicate Test"
        }),
        signature: "sha256=placeholder".to_string(),
    };

    let body = serde_json::to_vec(&webhook).unwrap();
    let signature = generate_hmac_signature(&body, "test-secret-dup").unwrap();

    // First receive should succeed
    let event_id1 = receiver
        .receive("netflix", &body, &signature)
        .await
        .unwrap();

    // Second receive should also succeed but return same event_id (duplicate)
    let event_id2 = receiver
        .receive("netflix", &body, &signature)
        .await
        .unwrap();

    assert_eq!(event_id1, event_id2);

    // Check metrics - duplicates counter should be incremented
    let metrics = receiver.metrics();
    let snapshot = metrics.snapshot();
    assert!(snapshot.duplicates > 0);

    // Clean up
    let redis_client = redis::Client::open(get_redis_url()).unwrap();
    let mut conn = redis_client.get_async_connection().await.unwrap();
    let key = format!("webhook:hash:{}", event_id1);
    let _: () = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_invalid_signature_rejection() {
    let receiver = match setup_receiver().await {
        Some(r) => r,
        None => return,
    };

    // Register handler
    let handler = Box::new(NetflixWebhookHandler::new());
    let config = PlatformWebhookConfig {
        platform: "netflix".to_string(),
        secret: "correct-secret".to_string(),
        rate_limit: 100,
        enabled: true,
    };

    receiver.register_handler(handler, config).await.unwrap();

    // Create webhook with wrong signature
    let webhook = WebhookPayload {
        event_type: WebhookEventType::ContentAdded,
        platform: "netflix".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "content_id": "sig-test-12345",
            "title": "Signature Test"
        }),
        signature: "sha256=placeholder".to_string(),
    };

    let body = serde_json::to_vec(&webhook).unwrap();
    let wrong_signature = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

    // Should fail with invalid signature
    let result = receiver.receive("netflix", &body, wrong_signature).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_queue_processing() {
    let redis_url = get_redis_url();

    let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
        Ok(q) => Arc::new(q) as Arc<dyn WebhookQueue>,
        Err(_) => {
            eprintln!("Redis not available - skipping test");
            return;
        }
    };

    // Create test webhook
    let webhook = WebhookPayload {
        event_type: WebhookEventType::ContentUpdated,
        platform: "netflix".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "content_id": "queue-test-12345",
            "title": "Queue Test Movie"
        }),
        signature: "sha256=test".to_string(),
    };

    // Enqueue
    let message_id = queue.enqueue(webhook.clone()).await.unwrap();
    assert!(!message_id.is_empty());

    // Dequeue
    let result = queue.dequeue("test-consumer").await.unwrap();
    assert!(result.is_some());

    let (dequeued_id, dequeued_webhook) = result.unwrap();
    assert_eq!(dequeued_webhook.platform, "netflix");
    assert_eq!(
        dequeued_webhook.event_type,
        WebhookEventType::ContentUpdated
    );

    // Ack
    queue.ack(&dequeued_id).await.unwrap();

    // Stats
    let stats = queue.stats().await.unwrap();
    assert!(stats.pending_count >= 0);

    // Clean up
    let redis_client = redis::Client::open(redis_url).unwrap();
    let mut conn = redis_client.get_async_connection().await.unwrap();
    let stream_key = "webhooks:incoming:netflix";
    let _: () = redis::cmd("DEL")
        .arg(stream_key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_rate_limiting() {
    let receiver = match setup_receiver().await {
        Some(r) => r,
        None => return,
    };

    // Register handler with low rate limit
    let handler = Box::new(NetflixWebhookHandler::new());
    let config = PlatformWebhookConfig {
        platform: "netflix".to_string(),
        secret: "test-secret-rate".to_string(),
        rate_limit: 2, // Only 2 requests per minute
        enabled: true,
    };

    receiver.register_handler(handler, config).await.unwrap();

    let secret = "test-secret-rate";

    // Send 3 webhooks rapidly
    for i in 0..3 {
        let webhook = WebhookPayload {
            event_type: WebhookEventType::ContentAdded,
            platform: "netflix".to_string(),
            timestamp: Utc::now(),
            payload: serde_json::json!({
                "content_id": format!("rate-test-{}", i),
                "title": format!("Rate Test {}", i)
            }),
            signature: "sha256=placeholder".to_string(),
        };

        let body = serde_json::to_vec(&webhook).unwrap();
        let signature = generate_hmac_signature(&body, secret).unwrap();

        let result = receiver.receive("netflix", &body, &signature).await;

        if i < 2 {
            // First 2 should succeed
            assert!(result.is_ok(), "Request {} should succeed", i);
        } else {
            // Third should be rate limited
            assert!(result.is_err(), "Request {} should be rate limited", i);
        }
    }

    // Check metrics
    let metrics = receiver.metrics();
    let snapshot = metrics.snapshot();
    assert!(snapshot.rate_limited > 0);
}

#[tokio::test]
async fn test_generic_handler() {
    let receiver = match setup_receiver().await {
        Some(r) => r,
        None => return,
    };

    // Register generic handler for Hulu
    let handler = Box::new(GenericWebhookHandler::new("hulu".to_string()));
    let config = PlatformWebhookConfig {
        platform: "hulu".to_string(),
        secret: "hulu-secret".to_string(),
        rate_limit: 100,
        enabled: true,
    };

    receiver.register_handler(handler, config).await.unwrap();

    // Create Hulu webhook
    let webhook = WebhookPayload {
        event_type: WebhookEventType::ContentRemoved,
        platform: "hulu".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "id": "hulu-12345",
            "title": "Removed Show"
        }),
        signature: "sha256=placeholder".to_string(),
    };

    let body = serde_json::to_vec(&webhook).unwrap();
    let signature = generate_hmac_signature(&body, "hulu-secret").unwrap();

    // Receive webhook
    let event_id = receiver.receive("hulu", &body, &signature).await.unwrap();
    assert!(!event_id.is_empty());

    // Clean up
    let redis_client = redis::Client::open(get_redis_url()).unwrap();
    let mut conn = redis_client.get_async_connection().await.unwrap();
    let key = format!("webhook:hash:{}", event_id);
    let _: () = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_webhook_pipeline_integration_end_to_end() {
    let receiver = match setup_receiver().await {
        Some(r) => r,
        None => return,
    };

    let handler = Box::new(NetflixWebhookHandler::new());
    let config = PlatformWebhookConfig {
        platform: "netflix".to_string(),
        secret: "test-e2e-pipeline".to_string(),
        rate_limit: 100,
        enabled: true,
    };

    receiver.register_handler(handler, config).await.unwrap();

    let webhook = WebhookPayload {
        event_type: WebhookEventType::ContentAdded,
        platform: "netflix".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "content_id": "e2e-pipeline-999",
            "title": "End to End Test Movie",
            "content_type": "movie",
            "year": 2024,
            "regions": ["US", "CA"]
        }),
        signature: "sha256=placeholder".to_string(),
    };

    let body = serde_json::to_vec(&webhook).unwrap();
    let signature = generate_hmac_signature(&body, "test-e2e-pipeline").unwrap();

    let event_id = receiver
        .receive("netflix", &body, &signature)
        .await
        .unwrap();
    assert!(!event_id.is_empty());

    let metrics = receiver.metrics();
    let snapshot = metrics.snapshot();
    assert!(snapshot.received > 0);
    assert!(snapshot.processed > 0);

    let stats = receiver.queue_stats().await.unwrap();
    assert!(stats.pending_count >= 0);

    let redis_client = redis::Client::open(get_redis_url()).unwrap();
    let mut conn = redis_client.get_async_connection().await.unwrap();
    let key = format!("webhook:hash:{}", event_id);
    let _: () = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_queue_metrics_tracking_integration() {
    let redis_url = get_redis_url();

    let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
        Ok(q) => Arc::new(q),
        Err(_) => {
            eprintln!("Redis not available - skipping test");
            return;
        }
    };

    let webhook = WebhookPayload {
        event_type: WebhookEventType::ContentAdded,
        platform: "netflix".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "content_id": "metrics-integration-001",
            "title": "Metrics Test"
        }),
        signature: "sha256=test".to_string(),
    };

    queue.enqueue(webhook.clone()).await.unwrap();

    let stats_before = queue.stats().await.unwrap();
    let initial_total = stats_before.total_processed;

    let dequeued = queue.dequeue("metrics-consumer").await.unwrap();
    assert!(dequeued.is_some());

    let stats_during = queue.stats().await.unwrap();
    assert_eq!(stats_during.processing_count, 1);

    let (msg_id, _) = dequeued.unwrap();
    queue.ack(&msg_id).await.unwrap();

    let stats_after = queue.stats().await.unwrap();
    assert_eq!(stats_after.processing_count, 0);
    assert_eq!(stats_after.total_processed, initial_total + 1);

    let redis_client = redis::Client::open(redis_url).unwrap();
    let mut conn = redis_client.get_async_connection().await.unwrap();
    let stream_key = "webhooks:incoming:netflix";
    let _: () = redis::cmd("DEL")
        .arg(stream_key)
        .query_async(&mut conn)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_error_recovery_dead_letter_queue() {
    let redis_url = get_redis_url();

    let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
        Ok(q) => Arc::new(q) as Arc<dyn WebhookQueue>,
        Err(_) => {
            eprintln!("Redis not available - skipping test");
            return;
        }
    };

    let invalid_webhook = WebhookPayload {
        event_type: WebhookEventType::ContentAdded,
        platform: "netflix".to_string(),
        timestamp: Utc::now(),
        payload: serde_json::json!({
            "title": "Missing content_id - should fail"
        }),
        signature: "sha256=test-dlq".to_string(),
    };

    queue.enqueue(invalid_webhook.clone()).await.unwrap();

    let dequeued = queue.dequeue("dlq-test-consumer").await.unwrap();
    assert!(dequeued.is_some());

    use media_gateway_ingestion::{ProcessedWebhook, ProcessingStatus};
    let processed = ProcessedWebhook {
        event_id: "dlq-test-event".to_string(),
        webhook: invalid_webhook,
        processed_at: Utc::now(),
        status: ProcessingStatus::Failed,
        error: Some("Processing failed - invalid payload".to_string()),
    };

    queue.dead_letter(processed).await.unwrap();

    let stats = queue.stats().await.unwrap();
    assert!(stats.dead_letter_count > 0);

    let redis_client = redis::Client::open(redis_url).unwrap();
    let mut conn = redis_client.get_async_connection().await.unwrap();
    let dlq_key = "webhooks:dlq:netflix";
    let _: () = redis::cmd("DEL")
        .arg(dlq_key)
        .query_async(&mut conn)
        .await
        .unwrap();
}
