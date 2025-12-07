//! Integration tests for expiring content API endpoint
//!
//! Tests the GET /api/v1/content/expiring endpoint
//!
//! Run with: cargo test --test expiration_api_test -- --test-threads=1

use actix_web::{test, web, App};
use chrono::{Duration, Utc};
use media_gateway_ingestion::normalizer::{
    AvailabilityInfo, CanonicalContent, ContentType, ImageSet,
};
use media_gateway_ingestion::repository::{ContentRepository, PostgresContentRepository};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use uuid::Uuid;

mod handlers {
    pub use media_gateway_ingestion::handlers::*;
}

/// Database URL for integration tests
fn get_test_database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/media_gateway_test".to_string())
}

/// Setup test database pool
async fn setup_test_pool() -> sqlx::PgPool {
    let database_url = get_test_database_url();

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Create test content with expiration date
fn create_expiring_content(
    title: &str,
    platform: &str,
    region: &str,
    expires_in_days: i64,
) -> CanonicalContent {
    let mut external_ids = HashMap::new();
    external_ids.insert(
        "imdb".to_string(),
        format!("tt{}", Uuid::new_v4().as_u128() % 10000000),
    );

    let expires_at = Utc::now() + Duration::days(expires_in_days);

    CanonicalContent {
        platform_content_id: format!("api_test_{}", Uuid::new_v4()),
        platform_id: platform.to_string(),
        entity_id: None,
        title: title.to_string(),
        overview: Some("Test content for API".to_string()),
        content_type: ContentType::Movie,
        release_year: Some(2024),
        runtime_minutes: Some(120),
        genres: vec!["Drama".to_string()],
        external_ids,
        availability: AvailabilityInfo {
            regions: vec![region.to_string()],
            subscription_required: true,
            purchase_price: None,
            rental_price: None,
            currency: Some("USD".to_string()),
            available_from: Some(Utc::now()),
            available_until: Some(expires_at),
        },
        images: ImageSet::default(),
        rating: Some("PG-13".to_string()),
        user_rating: Some(7.5),
        embedding: None,
        updated_at: Utc::now(),
    }
}

/// Cleanup helper
async fn cleanup_test_content(pool: &sqlx::PgPool, content_id: Uuid) {
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(content_id)
        .execute(pool)
        .await
        .expect("Failed to cleanup test content");
}

#[actix_web::test]
#[ignore]
async fn test_get_expiring_content_default_params() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create content expiring in 5 days
    let content = create_expiring_content("API Test Movie", "netflix", "US", 5);
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to insert content");

    // Create test app
    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
        "/api/v1/content/expiring",
        web::get().to(handlers::get_expiring_content),
    ))
    .await;

    // Make request
    let req = test::TestRequest::get()
        .uri("/api/v1/content/expiring")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Parse response
    let body: Value = test::read_body_json(resp).await;

    assert!(body["total"].as_u64().unwrap() > 0);
    assert_eq!(body["window_days"].as_i64().unwrap(), 7); // Default is 7 days

    let items = body["items"].as_array().expect("items should be array");
    let found = items
        .iter()
        .any(|item| item["content_id"].as_str().unwrap() == content_id.to_string());

    assert!(found, "Should find our test content");

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[actix_web::test]
#[ignore]
async fn test_get_expiring_content_custom_days() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create content expiring in 5 days
    let content = create_expiring_content("Custom Days Movie", "netflix", "US", 5);
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to insert content");

    // Create test app
    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
        "/api/v1/content/expiring",
        web::get().to(handlers::get_expiring_content),
    ))
    .await;

    // Request with 10 days window (should find content)
    let req = test::TestRequest::get()
        .uri("/api/v1/content/expiring?days=10")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["window_days"].as_i64().unwrap(), 10);

    let items = body["items"].as_array().expect("items should be array");
    let found = items
        .iter()
        .any(|item| item["content_id"].as_str().unwrap() == content_id.to_string());

    assert!(found, "Should find content with 10-day window");

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}

#[actix_web::test]
#[ignore]
async fn test_get_expiring_content_platform_filter() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create content on different platforms
    let netflix_content = create_expiring_content("Netflix Movie", "netflix", "US", 5);
    let disney_content = create_expiring_content("Disney Movie", "disney", "US", 5);

    let netflix_id = repo
        .upsert(&netflix_content)
        .await
        .expect("Failed to insert");
    let disney_id = repo
        .upsert(&disney_content)
        .await
        .expect("Failed to insert");

    // Create test app
    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
        "/api/v1/content/expiring",
        web::get().to(handlers::get_expiring_content),
    ))
    .await;

    // Request with platform filter
    let req = test::TestRequest::get()
        .uri("/api/v1/content/expiring?platform=netflix")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let items = body["items"].as_array().expect("items should be array");

    let has_netflix = items
        .iter()
        .any(|item| item["content_id"].as_str().unwrap() == netflix_id.to_string());

    let has_disney = items
        .iter()
        .any(|item| item["content_id"].as_str().unwrap() == disney_id.to_string());

    assert!(has_netflix, "Should find Netflix content");
    assert!(
        !has_disney,
        "Should not find Disney content with Netflix filter"
    );

    // Cleanup
    cleanup_test_content(&pool, netflix_id).await;
    cleanup_test_content(&pool, disney_id).await;
}

#[actix_web::test]
#[ignore]
async fn test_get_expiring_content_region_filter() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create content in different regions
    let us_content = create_expiring_content("US Movie", "netflix", "US", 5);
    let uk_content = create_expiring_content("UK Movie", "netflix", "UK", 5);

    let us_id = repo.upsert(&us_content).await.expect("Failed to insert");
    let uk_id = repo.upsert(&uk_content).await.expect("Failed to insert");

    // Create test app
    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
        "/api/v1/content/expiring",
        web::get().to(handlers::get_expiring_content),
    ))
    .await;

    // Request with region filter
    let req = test::TestRequest::get()
        .uri("/api/v1/content/expiring?region=US")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let items = body["items"].as_array().expect("items should be array");

    let has_us = items
        .iter()
        .any(|item| item["content_id"].as_str().unwrap() == us_id.to_string());

    let has_uk = items
        .iter()
        .any(|item| item["content_id"].as_str().unwrap() == uk_id.to_string());

    assert!(has_us, "Should find US content");
    assert!(!has_uk, "Should not find UK content with US filter");

    // Cleanup
    cleanup_test_content(&pool, us_id).await;
    cleanup_test_content(&pool, uk_id).await;
}

#[actix_web::test]
#[ignore]
async fn test_get_expiring_content_limit() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create multiple expiring content items
    let mut content_ids = Vec::new();
    for i in 1..=5 {
        let content =
            create_expiring_content(&format!("Limit Test Movie {}", i), "netflix", "US", 5);
        let id = repo.upsert(&content).await.expect("Failed to insert");
        content_ids.push(id);
    }

    // Create test app
    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
        "/api/v1/content/expiring",
        web::get().to(handlers::get_expiring_content),
    ))
    .await;

    // Request with limit
    let req = test::TestRequest::get()
        .uri("/api/v1/content/expiring?limit=2")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let items = body["items"].as_array().expect("items should be array");

    // Should respect the limit
    assert!(items.len() <= 2, "Should limit results to 2");

    // Cleanup
    for id in content_ids {
        cleanup_test_content(&pool, id).await;
    }
}

#[actix_web::test]
#[ignore]
async fn test_get_expiring_content_response_format() {
    let pool = setup_test_pool().await;
    let repo = PostgresContentRepository::new(pool.clone());

    // Create test content
    let content = create_expiring_content("Format Test Movie", "netflix", "US", 5);
    let content_id = repo
        .upsert(&content)
        .await
        .expect("Failed to insert content");

    // Create test app
    let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
        "/api/v1/content/expiring",
        web::get().to(handlers::get_expiring_content),
    ))
    .await;

    // Make request
    let req = test::TestRequest::get()
        .uri("/api/v1/content/expiring")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;

    // Verify response structure
    assert!(body["total"].is_number());
    assert!(body["window_days"].is_number());
    assert!(body["items"].is_array());

    // Verify item structure
    let items = body["items"].as_array().unwrap();
    if let Some(item) = items.first() {
        assert!(item["content_id"].is_string());
        assert!(item["title"].is_string());
        assert!(item["platform"].is_string());
        assert!(item["region"].is_string());
        assert!(item["expires_at"].is_string());
        assert!(item["days_until_expiration"].is_number());
    }

    // Cleanup
    cleanup_test_content(&pool, content_id).await;
}
