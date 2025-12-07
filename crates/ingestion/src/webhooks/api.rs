//! Webhook API endpoints

use crate::webhooks::{WebhookError, WebhookReceiver, WebhookRegistration};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Webhook API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/webhooks")
            .route("/{platform}", web::post().to(receive_webhook))
            .route("/register", web::post().to(register_webhook))
            .route("/metrics", web::get().to(get_metrics))
            .route("/stats", web::get().to(get_stats)),
    );
}

/// Receive webhook endpoint
///
/// POST /api/v1/webhooks/{platform}
async fn receive_webhook(
    receiver: web::Data<Arc<WebhookReceiver>>,
    platform: web::Path<String>,
    req: HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    // Extract signature from header
    let signature = match req.headers().get("X-Webhook-Signature") {
        Some(sig) => match sig.to_str() {
            Ok(s) => s,
            Err(_) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: "Invalid signature header".to_string(),
                })
            }
        },
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Missing X-Webhook-Signature header".to_string(),
            })
        }
    };

    // Process webhook
    match receiver.receive(&platform, &body, signature).await {
        Ok(event_id) => HttpResponse::Ok().json(WebhookResponse {
            event_id,
            status: "accepted".to_string(),
        }),
        Err(WebhookError::InvalidSignature(msg)) => {
            HttpResponse::Unauthorized().json(ErrorResponse { error: msg })
        }
        Err(WebhookError::RateLimitExceeded(platform)) => {
            HttpResponse::TooManyRequests().json(ErrorResponse {
                error: format!("Rate limit exceeded for platform: {}", platform),
            })
        }
        Err(WebhookError::UnsupportedPlatform(platform)) => {
            HttpResponse::NotFound().json(ErrorResponse {
                error: format!("Platform not supported: {}", platform),
            })
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Processing error: {}", e),
        }),
    }
}

/// Register webhook endpoint
///
/// POST /api/v1/webhooks/register
async fn register_webhook(
    _receiver: web::Data<Arc<WebhookReceiver>>,
    registration: web::Json<WebhookRegistration>,
) -> impl Responder {
    // TODO: Implement webhook registration
    // This would typically:
    // 1. Validate the registration request
    // 2. Store webhook configuration in database
    // 3. Set up handler with provided secret
    // 4. Return webhook URL

    HttpResponse::Ok().json(RegisterResponse {
        webhook_url: format!("/api/v1/webhooks/{}", registration.platform),
        platform: registration.platform.clone(),
        secret: "configured".to_string(),
    })
}

/// Get metrics endpoint
///
/// GET /api/v1/webhooks/metrics
async fn get_metrics(receiver: web::Data<Arc<WebhookReceiver>>) -> impl Responder {
    let metrics = receiver.metrics();
    let snapshot = metrics.snapshot();

    HttpResponse::Ok().json(MetricsResponse {
        received: snapshot.received,
        processed: snapshot.processed,
        failed: snapshot.failed,
        duplicates: snapshot.duplicates,
        rate_limited: snapshot.rate_limited,
        success_rate: snapshot.success_rate(),
        failure_rate: snapshot.failure_rate(),
    })
}

/// Get queue statistics endpoint
///
/// GET /api/v1/webhooks/stats
async fn get_stats(receiver: web::Data<Arc<WebhookReceiver>>) -> impl Responder {
    match receiver.queue_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to get stats: {}", e),
        }),
    }
}

/// Webhook response
#[derive(Debug, Serialize)]
struct WebhookResponse {
    event_id: String,
    status: String,
}

/// Registration response
#[derive(Debug, Serialize)]
struct RegisterResponse {
    webhook_url: String,
    platform: String,
    secret: String,
}

/// Metrics response
#[derive(Debug, Serialize)]
struct MetricsResponse {
    received: u64,
    processed: u64,
    failed: u64,
    duplicates: u64,
    rate_limited: u64,
    success_rate: f64,
    failure_rate: f64,
}

/// Error response
#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::webhooks::{
        handlers::NetflixWebhookHandler, queue::RedisWebhookQueue, PlatformWebhookConfig,
        WebhookDeduplicator, WebhookMetrics, WebhookQueue,
    };
    use actix_web::{test, App};

    async fn create_test_receiver() -> Option<Arc<WebhookReceiver>> {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let queue = match RedisWebhookQueue::new(&redis_url, None, None, None) {
            Ok(q) => Arc::new(q) as Arc<dyn WebhookQueue>,
            Err(_) => return None,
        };

        let deduplicator = match WebhookDeduplicator::new(&redis_url, Some(1)) {
            Ok(d) => Arc::new(d),
            Err(_) => return None,
        };

        let metrics = Arc::new(WebhookMetrics::new());
        let receiver = Arc::new(WebhookReceiver::new(queue, deduplicator, metrics));

        // Register Netflix handler
        let handler = Box::new(NetflixWebhookHandler::new());
        let config = PlatformWebhookConfig {
            platform: "netflix".to_string(),
            secret: "test-secret".to_string(),
            rate_limit: 100,
            enabled: true,
        };

        receiver.register_handler(handler, config).await.ok()?;

        Some(receiver)
    }

    #[actix_web::test]
    async fn test_receive_webhook_endpoint() {
        let receiver = match create_test_receiver().await {
            Some(r) => r,
            None => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(receiver.clone()))
                .configure(configure_routes),
        )
        .await;

        let webhook_payload = serde_json::json!({
            "event_type": "content_added",
            "platform": "netflix",
            "timestamp": "2025-12-06T00:00:00Z",
            "payload": {"content_id": "test123"},
            "signature": "sha256=test"
        });

        let body = serde_json::to_vec(&webhook_payload).unwrap();

        // Generate valid signature
        use crate::webhooks::verification::generate_hmac_signature;
        let signature = generate_hmac_signature(&body, "test-secret").unwrap();

        let req = test::TestRequest::post()
            .uri("/api/v1/webhooks/netflix")
            .insert_header(("X-Webhook-Signature", signature))
            .set_payload(body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_receive_webhook_missing_signature() {
        let receiver = match create_test_receiver().await {
            Some(r) => r,
            None => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(receiver))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/webhooks/netflix")
            .set_payload(b"{}")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_get_metrics_endpoint() {
        let receiver = match create_test_receiver().await {
            Some(r) => r,
            None => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(receiver))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/v1/webhooks/metrics")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_register_webhook_endpoint() {
        let receiver = match create_test_receiver().await {
            Some(r) => r,
            None => {
                eprintln!("Skipping test: Redis not available");
                return;
            }
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(receiver))
                .configure(configure_routes),
        )
        .await;

        let registration = WebhookRegistration {
            platform: "hulu".to_string(),
            url: "https://example.com/webhook".to_string(),
            event_types: vec![],
            secret: "test-secret".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/webhooks/register")
            .set_json(&registration)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
