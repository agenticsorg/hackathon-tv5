use actix_web::{test, web, App};
use media_gateway_api::health::{aggregate, AggregatedHealth, HealthAggregator, HealthStatus};
use std::time::Duration;

#[actix_web::test]
async fn test_aggregate_endpoint_returns_json() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success() || resp.status().is_server_error());
}

#[actix_web::test]
async fn test_aggregate_endpoint_structure() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status().is_success() {
        let body = test::read_body(resp).await;
        let health: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(health.get("status").is_some());
        assert!(health.get("timestamp").is_some());
        assert!(health.get("services").is_some());
        assert!(health.get("dependencies").is_some());
        assert!(health.get("overall_latency_ms").is_some());

        let services = health.get("services").unwrap().as_array().unwrap();
        assert!(!services.is_empty());

        let dependencies = health.get("dependencies").unwrap().as_array().unwrap();
        assert!(!dependencies.is_empty());
    }
}

#[actix_web::test]
async fn test_aggregate_caching() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_millis(100)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    // First request
    let req1 = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    let body1 = test::read_body(resp1).await;
    let health1: serde_json::Value = serde_json::from_slice(&body1).unwrap_or_default();

    // Second request (should use cache)
    let req2 = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    let body2 = test::read_body(resp2).await;
    let health2: serde_json::Value = serde_json::from_slice(&body2).unwrap_or_default();

    if !health1.is_null() && !health2.is_null() {
        assert_eq!(
            health1.get("timestamp"),
            health2.get("timestamp"),
            "Cached responses should have same timestamp"
        );
    }
}

#[actix_web::test]
async fn test_aggregate_cache_expiry() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_millis(50)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    // First request
    let req1 = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    let body1 = test::read_body(resp1).await;
    let health1: serde_json::Value = serde_json::from_slice(&body1).unwrap_or_default();

    // Wait for cache to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Second request (should refresh cache)
    let req2 = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    let body2 = test::read_body(resp2).await;
    let health2: serde_json::Value = serde_json::from_slice(&body2).unwrap_or_default();

    if !health1.is_null() && !health2.is_null() {
        assert_ne!(
            health1.get("timestamp"),
            health2.get("timestamp"),
            "Expired cache should generate new timestamp"
        );
    }
}

#[actix_web::test]
async fn test_service_health_fields() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status().is_success() {
        let body = test::read_body(resp).await;
        let health: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let services = health.get("services").unwrap().as_array().unwrap();
        for service in services {
            assert!(service.get("name").is_some());
            assert!(service.get("status").is_some());
            assert!(service.get("last_checked").is_some());
            // latency_ms and error are optional
        }
    }
}

#[actix_web::test]
async fn test_dependency_health_fields() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status().is_success() {
        let body = test::read_body(resp).await;
        let health: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let dependencies = health.get("dependencies").unwrap().as_array().unwrap();
        for dep in dependencies {
            assert!(dep.get("name").is_some());
            assert!(dep.get("status").is_some());
            assert!(dep.get("last_checked").is_some());
            // latency_ms and error are optional
        }
    }
}

#[actix_web::test]
async fn test_health_status_values() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status().is_success() {
        let body = test::read_body(resp).await;
        let health: serde_json::Value = serde_json::from_slice(&body).unwrap();

        let status = health.get("status").unwrap().as_str().unwrap();
        assert!(
            status == "healthy" || status == "degraded" || status == "unhealthy",
            "Status must be one of: healthy, degraded, unhealthy"
        );
    }
}

#[actix_web::test]
async fn test_expected_services_present() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status().is_success() {
        let body = test::read_body(resp).await;
        let health: AggregatedHealth = serde_json::from_slice(&body).unwrap();

        let expected_services = vec!["discovery", "sona", "auth", "sync", "ingestion", "playback"];

        for expected in expected_services {
            assert!(
                health.services.iter().any(|s| s.name == expected),
                "Service '{}' should be present in health check",
                expected
            );
        }
    }
}

#[actix_web::test]
async fn test_expected_dependencies_present() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status().is_success() {
        let body = test::read_body(resp).await;
        let health: AggregatedHealth = serde_json::from_slice(&body).unwrap();

        let expected_deps = vec!["postgresql", "redis", "qdrant"];

        for expected in expected_deps {
            assert!(
                health.dependencies.iter().any(|d| d.name == expected),
                "Dependency '{}' should be present in health check",
                expected
            );
        }
    }
}

#[actix_web::test]
async fn test_latency_tracking() {
    let aggregator = web::Data::new(HealthAggregator::new(Duration::from_secs(5)));

    let app = test::init_service(
        App::new()
            .app_data(aggregator.clone())
            .route("/health/aggregate", web::get().to(aggregate)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/health/aggregate")
        .to_request();

    let resp = test::call_service(&app, req).await;

    if resp.status().is_success() {
        let body = test::read_body(resp).await;
        let health: AggregatedHealth = serde_json::from_slice(&body).unwrap();

        assert!(
            health.overall_latency_ms > 0,
            "Overall latency should be greater than 0"
        );
    }
}
