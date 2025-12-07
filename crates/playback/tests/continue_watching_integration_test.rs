//! Integration tests for Continue Watching API
//!
//! These tests use a real PostgreSQL database connection

mod common;

use actix_web::{test, web, App};
use common::MockContentMetadataProvider;
use media_gateway_playback::continue_watching::{ContinueWatchingService, SyncServiceClient};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Ensure table exists
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS playback_progress (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            content_id UUID NOT NULL,
            platform_id VARCHAR(50) NOT NULL,
            progress_seconds INTEGER NOT NULL DEFAULT 0,
            duration_seconds INTEGER NOT NULL,
            progress_percentage FLOAT NOT NULL DEFAULT 0.0,
            last_position_ms BIGINT NOT NULL DEFAULT 0,
            is_completed BOOLEAN NOT NULL DEFAULT FALSE,
            device_id UUID,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            CONSTRAINT unique_user_content_platform UNIQUE(user_id, content_id, platform_id)
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create test table");

    pool
}

async fn cleanup_test_data(pool: &sqlx::PgPool, user_id: Uuid) {
    sqlx::query("DELETE FROM playback_progress WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
}

#[actix_web::test]
async fn test_get_continue_watching_empty() {
    let pool = setup_test_db().await;
    let user_id = Uuid::new_v4();

    let service = Arc::new(ContinueWatchingService::new(
        pool.clone(),
        Arc::new(MockContentMetadataProvider),
    ));

    let app = test::init_service(App::new().app_data(web::Data::new(service.clone())).route(
        "/api/v1/playback/continue-watching",
        web::get().to(
            |service: web::Data<Arc<ContinueWatchingService>>,
             query: web::Query<std::collections::HashMap<String, String>>| async move {
                let user_id = query.get("user_id").unwrap().parse::<Uuid>().unwrap();
                let limit = query.get("limit").and_then(|l| l.parse::<i64>().ok());
                let result = service.get_continue_watching(user_id, limit).await;
                match result {
                    Ok(response) => actix_web::HttpResponse::Ok().json(response),
                    Err(e) => e.error_response(),
                }
            },
        ),
    ))
    .await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/v1/playback/continue-watching?user_id={}",
            user_id
        ))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["total"], 0);
    assert_eq!(body["items"].as_array().unwrap().len(), 0);

    cleanup_test_data(&pool, user_id).await;
}

#[actix_web::test]
async fn test_update_progress_creates_record() {
    let pool = setup_test_db().await;
    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let service = Arc::new(ContinueWatchingService::new(
        pool.clone(),
        Arc::new(MockContentMetadataProvider),
    ));

    let app = test::init_service(App::new().app_data(web::Data::new(service.clone())).route(
        "/api/v1/playback/progress",
        web::post().to(
            |service: web::Data<Arc<ContinueWatchingService>>,
             req: web::Json<serde_json::Value>| async move {
                let user_id = req["user_id"].as_str().unwrap().parse::<Uuid>().unwrap();
                let progress_request =
                    media_gateway_playback::continue_watching::ProgressUpdateRequest {
                        content_id: req["content_id"].as_str().unwrap().parse::<Uuid>().unwrap(),
                        platform_id: req["platform_id"].as_str().unwrap().to_string(),
                        progress_seconds: req["progress_seconds"].as_i64().unwrap() as i32,
                        duration_seconds: req["duration_seconds"].as_i64().unwrap() as i32,
                        device_id: req["device_id"]
                            .as_str()
                            .and_then(|s| s.parse::<Uuid>().ok()),
                    };
                let result = service.update_progress(user_id, progress_request).await;
                match result {
                    Ok(response) => actix_web::HttpResponse::Ok().json(response),
                    Err(e) => e.error_response(),
                }
            },
        ),
    ))
    .await;

    let payload = json!({
        "user_id": user_id.to_string(),
        "content_id": content_id.to_string(),
        "platform_id": "netflix",
        "progress_seconds": 1200,
        "duration_seconds": 6000
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/playback/progress")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["content_id"], content_id.to_string());
    assert_eq!(body["progress_percentage"], 20.0);
    assert_eq!(body["is_completed"], false);

    cleanup_test_data(&pool, user_id).await;
}

#[actix_web::test]
async fn test_full_workflow_progress_and_continue_watching() {
    let pool = setup_test_db().await;
    let user_id = Uuid::new_v4();

    let service = Arc::new(ContinueWatchingService::new(
        pool.clone(),
        Arc::new(MockContentMetadataProvider),
    ));

    // Create multiple progress records
    let content_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();

    for (i, content_id) in content_ids.iter().enumerate() {
        let request = media_gateway_playback::continue_watching::ProgressUpdateRequest {
            content_id: *content_id,
            platform_id: "netflix".to_string(),
            progress_seconds: 1000 + (i as i32 * 500),
            duration_seconds: 6000,
            device_id: None,
        };

        service.update_progress(user_id, request).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Get continue watching list
    let result = service
        .get_continue_watching(user_id, Some(10))
        .await
        .unwrap();

    assert_eq!(result.total, 3);
    assert_eq!(result.items.len(), 3);

    // Verify ordering (most recent first)
    assert_eq!(result.items[0].content_id, content_ids[2]);
    assert_eq!(result.items[1].content_id, content_ids[1]);
    assert_eq!(result.items[2].content_id, content_ids[0]);

    // Verify all items have correct metadata
    for item in &result.items {
        assert_eq!(item.platform, "netflix");
        assert!(item.progress_percentage > 0.0 && item.progress_percentage < 95.0);
        assert_eq!(item.resume_position_ms, item.progress_seconds as i64 * 1000);
    }

    cleanup_test_data(&pool, user_id).await;
}

#[actix_web::test]
async fn test_completion_threshold_excludes_from_continue_watching() {
    let pool = setup_test_db().await;
    let user_id = Uuid::new_v4();

    let service = Arc::new(ContinueWatchingService::new(
        pool.clone(),
        Arc::new(MockContentMetadataProvider),
    ));

    // Add incomplete content
    let incomplete_id = Uuid::new_v4();
    let incomplete_request = media_gateway_playback::continue_watching::ProgressUpdateRequest {
        content_id: incomplete_id,
        platform_id: "netflix".to_string(),
        progress_seconds: 3000,
        duration_seconds: 6000,
        device_id: None,
    };
    service
        .update_progress(user_id, incomplete_request)
        .await
        .unwrap();

    // Add completed content (95%+)
    let completed_id = Uuid::new_v4();
    let completed_request = media_gateway_playback::continue_watching::ProgressUpdateRequest {
        content_id: completed_id,
        platform_id: "hulu".to_string(),
        progress_seconds: 5700,
        duration_seconds: 6000,
        device_id: None,
    };
    service
        .update_progress(user_id, completed_request)
        .await
        .unwrap();

    // Get continue watching list
    let result = service.get_continue_watching(user_id, None).await.unwrap();

    // Only incomplete should be in the list
    assert_eq!(result.total, 1);
    assert_eq!(result.items[0].content_id, incomplete_id);
    assert_eq!(result.items[0].platform, "netflix");

    cleanup_test_data(&pool, user_id).await;
}

#[actix_web::test]
async fn test_update_progress_conflict_resolution() {
    let pool = setup_test_db().await;
    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let service = Arc::new(ContinueWatchingService::new(
        pool.clone(),
        Arc::new(MockContentMetadataProvider),
    ));

    // First update
    let request1 = media_gateway_playback::continue_watching::ProgressUpdateRequest {
        content_id,
        platform_id: "netflix".to_string(),
        progress_seconds: 1200,
        duration_seconds: 6000,
        device_id: None,
    };
    let response1 = service.update_progress(user_id, request1).await.unwrap();
    assert_eq!(response1.progress_percentage, 20.0);

    // Second update (conflict should resolve via upsert)
    let request2 = media_gateway_playback::continue_watching::ProgressUpdateRequest {
        content_id,
        platform_id: "netflix".to_string(),
        progress_seconds: 3600,
        duration_seconds: 6000,
        device_id: Some(Uuid::new_v4()),
    };
    let response2 = service.update_progress(user_id, request2).await.unwrap();
    assert_eq!(response2.progress_percentage, 60.0);

    // Verify only one record exists
    let continue_watching = service.get_continue_watching(user_id, None).await.unwrap();
    assert_eq!(continue_watching.total, 1);
    assert_eq!(continue_watching.items[0].progress_seconds, 3600);

    cleanup_test_data(&pool, user_id).await;
}

#[actix_web::test]
async fn test_cleanup_stale_progress() {
    let pool = setup_test_db().await;
    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let service = Arc::new(ContinueWatchingService::new(
        pool.clone(),
        Arc::new(MockContentMetadataProvider),
    ));

    // Create completed progress
    let request = media_gateway_playback::continue_watching::ProgressUpdateRequest {
        content_id,
        platform_id: "netflix".to_string(),
        progress_seconds: 5700,
        duration_seconds: 6000,
        device_id: None,
    };
    service.update_progress(user_id, request).await.unwrap();

    // Manually update timestamp to simulate old record
    sqlx::query(
        "UPDATE playback_progress SET updated_at = NOW() - INTERVAL '31 days' WHERE user_id = $1"
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    // Run cleanup
    let deleted_count = service.cleanup_stale_progress(30).await.unwrap();
    assert_eq!(deleted_count, 1);

    // Verify record was deleted
    let continue_watching = service.get_continue_watching(user_id, None).await.unwrap();
    assert_eq!(continue_watching.total, 0);

    cleanup_test_data(&pool, user_id).await;
}

#[actix_web::test]
async fn test_multiple_platforms_same_content() {
    let pool = setup_test_db().await;
    let user_id = Uuid::new_v4();
    let content_id = Uuid::new_v4();

    let service = Arc::new(ContinueWatchingService::new(
        pool.clone(),
        Arc::new(MockContentMetadataProvider),
    ));

    // Watch same content on different platforms
    let netflix_request = media_gateway_playback::continue_watching::ProgressUpdateRequest {
        content_id,
        platform_id: "netflix".to_string(),
        progress_seconds: 1200,
        duration_seconds: 6000,
        device_id: None,
    };
    service
        .update_progress(user_id, netflix_request)
        .await
        .unwrap();

    let hulu_request = media_gateway_playback::continue_watching::ProgressUpdateRequest {
        content_id,
        platform_id: "hulu".to_string(),
        progress_seconds: 2400,
        duration_seconds: 6000,
        device_id: None,
    };
    service
        .update_progress(user_id, hulu_request)
        .await
        .unwrap();

    // Should have two separate records
    let result = service.get_continue_watching(user_id, None).await.unwrap();
    assert_eq!(result.total, 2);

    let platforms: Vec<&str> = result.items.iter().map(|i| i.platform.as_str()).collect();
    assert!(platforms.contains(&"netflix"));
    assert!(platforms.contains(&"hulu"));

    cleanup_test_data(&pool, user_id).await;
}
