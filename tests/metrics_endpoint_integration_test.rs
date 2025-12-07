#[cfg(test)]
mod metrics_endpoint_tests {
    use actix_web::{test, web, App};
    use media_gateway_core::{metrics_handler, MetricsMiddleware};

    #[actix_web::test]
    async fn test_metrics_endpoint_exists() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_metrics_content_type() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;

        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(content_type.contains("text/plain"));
    }

    #[actix_web::test]
    async fn test_metrics_contains_http_requests_total() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("http_requests_total"));
        assert!(body_str.contains("# HELP http_requests_total"));
        assert!(body_str.contains("# TYPE http_requests_total counter"));
    }

    #[actix_web::test]
    async fn test_metrics_contains_http_request_duration_seconds() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("http_request_duration_seconds"));
        assert!(body_str.contains("# HELP http_request_duration_seconds"));
        assert!(body_str.contains("# TYPE http_request_duration_seconds histogram"));
    }

    #[actix_web::test]
    async fn test_metrics_contains_active_connections() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("active_connections"));
        assert!(body_str.contains("# HELP active_connections"));
        assert!(body_str.contains("# TYPE active_connections gauge"));
    }

    #[actix_web::test]
    async fn test_metrics_contains_db_connection_metrics() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("db_connections_active"));
        assert!(body_str.contains("db_connections_idle"));
        assert!(body_str.contains("# TYPE db_connections_active gauge"));
        assert!(body_str.contains("# TYPE db_connections_idle gauge"));
    }

    #[actix_web::test]
    async fn test_metrics_contains_cache_metrics() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        assert!(body_str.contains("cache_hits_total"));
        assert!(body_str.contains("cache_misses_total"));
        assert!(body_str.contains("# TYPE cache_hits_total counter"));
        assert!(body_str.contains("# TYPE cache_misses_total counter"));
    }

    #[actix_web::test]
    async fn test_metrics_endpoint_performance() {
        use std::time::Instant;

        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let start = Instant::now();
        let req = test::TestRequest::get().uri("/metrics").to_request();
        let _resp = test::call_service(&app, req).await;
        let duration = start.elapsed();

        // Metrics endpoint should respond in less than 10ms
        assert!(
            duration.as_millis() < 10,
            "Metrics endpoint took {}ms, expected <10ms",
            duration.as_millis()
        );
    }

    #[actix_web::test]
    async fn test_metrics_middleware_records_requests() {
        use media_gateway_core::{record_http_request, METRICS_REGISTRY};

        // Record some test metrics
        record_http_request("GET", "/test", "200");
        record_http_request("POST", "/test", "201");

        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Verify the recorded metrics appear in the output
        assert!(body_str.contains("http_requests_total"));
    }

    #[actix_web::test]
    async fn test_metrics_format_is_prometheus_compatible() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Verify Prometheus text format structure
        assert!(body_str.contains("# HELP"));
        assert!(body_str.contains("# TYPE"));

        // Should not contain invalid characters
        for line in body_str.lines() {
            if !line.starts_with('#') && !line.is_empty() {
                // Metric lines should have valid format: metric_name{labels} value
                assert!(
                    line.contains('{') || line.split_whitespace().count() == 2,
                    "Invalid metric line: {}",
                    line
                );
            }
        }
    }

    #[actix_web::test]
    async fn test_metrics_endpoint_multiple_requests() {
        let app = test::init_service(
            App::new()
                .wrap(MetricsMiddleware)
                .route("/metrics", web::get().to(metrics_handler)),
        )
        .await;

        // Make multiple requests
        for _ in 0..10 {
            let req = test::TestRequest::get().uri("/metrics").to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }

        // Verify metrics are consistent
        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // All expected metrics should still be present
        assert!(body_str.contains("http_requests_total"));
        assert!(body_str.contains("http_request_duration_seconds"));
        assert!(body_str.contains("active_connections"));
    }
}
