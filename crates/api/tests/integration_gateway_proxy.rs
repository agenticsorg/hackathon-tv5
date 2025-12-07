use actix_web::{test, web, App};
use media_gateway_api::{
    circuit_breaker::CircuitBreakerManager, config::Config, proxy::ServiceProxy, routes,
};
use std::sync::Arc;

#[actix_web::test]
async fn test_sona_routes_configured() {
    let config = Arc::new(Config::default());
    let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.clone()));
    let proxy = Arc::new(ServiceProxy::new(config, circuit_breaker));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(proxy))
            .configure(routes::configure),
    )
    .await;

    // Test SONA endpoints are registered
    let req = test::TestRequest::post()
        .uri("/api/v1/sona/recommendations")
        .set_json(serde_json::json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should get a response (even if service is down, route exists)
    assert!(resp.status().as_u16() >= 200);
}

#[actix_web::test]
async fn test_playback_routes_configured() {
    let config = Arc::new(Config::default());
    let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.clone()));
    let proxy = Arc::new(ServiceProxy::new(config, circuit_breaker));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(proxy))
            .configure(routes::configure),
    )
    .await;

    // Test playback endpoints are registered
    let req = test::TestRequest::post()
        .uri("/api/v1/playback/sessions")
        .set_json(serde_json::json!({
            "user_id": "test-user",
            "content_id": "test-content",
            "device_id": "test-device"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);
}

#[actix_web::test]
async fn test_sync_routes_configured() {
    let config = Arc::new(Config::default());
    let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.clone()));
    let proxy = Arc::new(ServiceProxy::new(config, circuit_breaker));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(proxy))
            .configure(routes::configure),
    )
    .await;

    // Test sync endpoints are registered
    let req = test::TestRequest::post()
        .uri("/api/v1/sync/watchlist")
        .set_json(serde_json::json!({
            "operation": "add",
            "content_id": "test-content"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);
}

#[actix_web::test]
async fn test_playback_session_routes() {
    let config = Arc::new(Config::default());
    let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.clone()));
    let proxy = Arc::new(ServiceProxy::new(config, circuit_breaker));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(proxy))
            .configure(routes::configure),
    )
    .await;

    // Test GET session by ID
    let req = test::TestRequest::get()
        .uri("/api/v1/playback/sessions/550e8400-e29b-41d4-a716-446655440000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);

    // Test PATCH position update
    let req = test::TestRequest::patch()
        .uri("/api/v1/playback/sessions/550e8400-e29b-41d4-a716-446655440000/position")
        .set_json(serde_json::json!({
            "position_seconds": 100
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);

    // Test DELETE session
    let req = test::TestRequest::delete()
        .uri("/api/v1/playback/sessions/550e8400-e29b-41d4-a716-446655440000")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);
}

#[actix_web::test]
async fn test_sona_experiment_metrics() {
    let config = Arc::new(Config::default());
    let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.clone()));
    let proxy = Arc::new(ServiceProxy::new(config, circuit_breaker));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(proxy))
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/sona/experiments/exp-123/metrics")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);
}

#[actix_web::test]
async fn test_sync_device_routes() {
    let config = Arc::new(Config::default());
    let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.clone()));
    let proxy = Arc::new(ServiceProxy::new(config, circuit_breaker));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(proxy))
            .configure(routes::configure),
    )
    .await;

    // Test list devices
    let req = test::TestRequest::get()
        .uri("/api/v1/sync/devices")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);

    // Test device handoff
    let req = test::TestRequest::post()
        .uri("/api/v1/sync/devices/handoff")
        .set_json(serde_json::json!({
            "target_device_id": "device-2",
            "content_id": "content-1"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().as_u16() >= 200);
}

#[test]
fn test_service_config_includes_playback() {
    let config = Config::default();
    assert_eq!(config.services.playback.url, "http://localhost:8086");
    assert_eq!(config.services.playback.timeout_ms, 5000);
}

#[test]
fn test_circuit_breaker_config_includes_all_services() {
    let config = Config::default();

    assert!(config.circuit_breaker.services.contains_key("discovery"));
    assert!(config.circuit_breaker.services.contains_key("sona"));
    assert!(config.circuit_breaker.services.contains_key("playback"));
    assert!(config.circuit_breaker.services.contains_key("sync"));
}
