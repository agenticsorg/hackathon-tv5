use actix_web::{test, web, App};

/// Test that all routes are registered correctly
#[actix_web::test]
async fn test_route_registration() {
    let app =
        test::init_service(App::new().configure(media_gateway_discovery::server::configure_routes))
            .await;

    // Test health endpoint
    let req = test::TestRequest::get().uri("/api/v1/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success() || resp.status().is_server_error(),
        "Health endpoint should be registered (may fail without proper state)"
    );

    // Note: Other endpoints require proper application state (database, services)
    // and will return 500 errors without it. This test only verifies routes are registered.
}

/// Test that search routes are registered
#[actix_web::test]
async fn test_search_routes_registered() {
    let app =
        test::init_service(App::new().configure(media_gateway_discovery::server::configure_routes))
            .await;

    // Test search endpoint (POST)
    let req = test::TestRequest::post()
        .uri("/api/v1/search")
        .set_json(&serde_json::json!({
            "query": "test"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    // Route is registered if we get anything other than 404
    assert_ne!(
        resp.status(),
        404,
        "Search POST endpoint should be registered"
    );

    // Test autocomplete endpoint (GET)
    let req = test::TestRequest::get()
        .uri("/api/v1/search/autocomplete?q=test")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "Autocomplete GET endpoint should be registered"
    );
}

/// Test that analytics routes are registered
#[actix_web::test]
async fn test_analytics_routes_registered() {
    let app =
        test::init_service(App::new().configure(media_gateway_discovery::server::configure_routes))
            .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/analytics?period=24h&limit=10")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "Analytics endpoint should be registered"
    );
}

/// Test that quality routes are registered
#[actix_web::test]
async fn test_quality_routes_registered() {
    let app =
        test::init_service(App::new().configure(media_gateway_discovery::server::configure_routes))
            .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/quality/report?threshold=0.6&limit=100")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "Quality report endpoint should be registered"
    );
}

/// Test that admin ranking routes are registered
#[actix_web::test]
async fn test_ranking_routes_registered() {
    let app =
        test::init_service(App::new().configure(media_gateway_discovery::server::configure_routes))
            .await;

    // Test default config endpoints
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/search/ranking")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "Get ranking config endpoint should be registered"
    );

    // Test variants list
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/search/ranking/variants")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "List variants endpoint should be registered"
    );

    // Test specific variant
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/search/ranking/variants/test")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "Get variant endpoint should be registered"
    );

    // Test history
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/search/ranking/history/1")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "Get history endpoint should be registered"
    );
}

/// Test that catalog routes are still registered
#[actix_web::test]
async fn test_catalog_routes_still_registered() {
    let app =
        test::init_service(App::new().configure(media_gateway_discovery::server::configure_routes))
            .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/catalog/content")
        .set_json(&serde_json::json!({
            "title": "Test Movie"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_ne!(
        resp.status(),
        404,
        "Catalog content endpoint should be registered"
    );
}

/// Verify that invalid routes return 404
#[actix_web::test]
async fn test_invalid_routes_return_404() {
    let app =
        test::init_service(App::new().configure(media_gateway_discovery::server::configure_routes))
            .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/nonexistent")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404, "Nonexistent routes should return 404");
}
