use actix_web::{test, web, App, HttpResponse};
use media_gateway_auth::{
    middleware::AuthMiddleware,
    rate_limit_admin_handlers::{
        delete_rate_limit, get_rate_limit, list_rate_limits, update_rate_limit,
        UpdateRateLimitConfigRequest,
    },
    rate_limit_config::{RateLimitConfigStore, UserTier},
};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

fn setup_redis_client() -> redis::Client {
    redis::Client::open("redis://127.0.0.1:6379").expect("Failed to create Redis client")
}

async fn setup_db_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database")
}

fn create_test_jwt() -> String {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        role: String,
        exp: usize,
    }

    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        role: "admin".to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
    };

    let secret = "test-secret-key-for-jwt-signing";
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[actix_web::test]
async fn test_rate_limit_config_store_set_and_get() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = RateLimitConfigStore::new(redis_client, db_pool);

    let config = media_gateway_auth::rate_limit_config::RateLimitConfig::new(
        "/api/v1/test".to_string(),
        UserTier::Free,
        50,
        500,
        75,
    );

    store.set_config(&config).await.unwrap();

    let retrieved = store
        .get_config("/api/v1/test", UserTier::Free)
        .await
        .unwrap();
    assert!(retrieved.is_some());

    let retrieved_config = retrieved.unwrap();
    assert_eq!(retrieved_config.endpoint, "/api/v1/test");
    assert_eq!(retrieved_config.tier, UserTier::Free);
    assert_eq!(retrieved_config.requests_per_minute, 50);
    assert_eq!(retrieved_config.requests_per_hour, 500);
    assert_eq!(retrieved_config.burst_size, 75);

    store
        .delete_config("/api/v1/test", UserTier::Free)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_rate_limit_config_store_get_all() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = RateLimitConfigStore::new(redis_client, db_pool);

    let config1 = media_gateway_auth::rate_limit_config::RateLimitConfig::new(
        "/api/v1/users".to_string(),
        UserTier::Free,
        30,
        300,
        50,
    );

    let config2 = media_gateway_auth::rate_limit_config::RateLimitConfig::new(
        "/api/v1/posts".to_string(),
        UserTier::Premium,
        100,
        1000,
        150,
    );

    store.set_config(&config1).await.unwrap();
    store.set_config(&config2).await.unwrap();

    let all_configs = store.get_all_configs().await.unwrap();
    assert!(all_configs.len() >= 2);

    store
        .delete_config("/api/v1/users", UserTier::Free)
        .await
        .unwrap();
    store
        .delete_config("/api/v1/posts", UserTier::Premium)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_rate_limit_config_wildcard_matching() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = RateLimitConfigStore::new(redis_client, db_pool);

    let wildcard_config = media_gateway_auth::rate_limit_config::RateLimitConfig::new(
        "/api/v1/*".to_string(),
        UserTier::Free,
        40,
        400,
        60,
    );

    store.set_config(&wildcard_config).await.unwrap();

    let matched = store
        .get_matching_config("/api/v1/users/123", UserTier::Free)
        .await
        .unwrap();
    assert!(matched.is_some());

    let matched_config = matched.unwrap();
    assert_eq!(matched_config.endpoint, "/api/v1/*");
    assert_eq!(matched_config.requests_per_minute, 40);

    store
        .delete_config("/api/v1/*", UserTier::Free)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_rate_limit_config_default_fallback() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = RateLimitConfigStore::new(redis_client, db_pool);

    let effective = store
        .get_effective_config("/api/v1/nonexistent", UserTier::Anonymous)
        .await
        .unwrap();

    assert_eq!(effective.tier, UserTier::Anonymous);
    assert_eq!(effective.requests_per_minute, 10);
    assert_eq!(effective.requests_per_hour, 100);
    assert_eq!(effective.burst_size, 15);
}

#[actix_web::test]
async fn test_list_rate_limits_endpoint() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = Arc::new(RateLimitConfigStore::new(redis_client, db_pool));

    let config = media_gateway_auth::rate_limit_config::RateLimitConfig::new(
        "/api/v1/test-list".to_string(),
        UserTier::Free,
        25,
        250,
        40,
    );
    store.set_config(&config).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
            .service(list_rate_limits),
    )
    .await;

    let jwt = create_test_jwt();

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/rate-limits")
        .insert_header(("Authorization", format!("Bearer {}", jwt)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    store
        .delete_config("/api/v1/test-list", UserTier::Free)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_update_rate_limit_endpoint() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = Arc::new(RateLimitConfigStore::new(redis_client, db_pool));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
            .service(update_rate_limit),
    )
    .await;

    let jwt = create_test_jwt();

    let update_req = UpdateRateLimitConfigRequest {
        tier: "premium".to_string(),
        requests_per_minute: 150,
        requests_per_hour: 7500,
        burst_size: 200,
    };

    let req = test::TestRequest::put()
        .uri("/api/v1/admin/rate-limits/api/v1/test-update")
        .insert_header(("Authorization", format!("Bearer {}", jwt)))
        .set_json(&update_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let retrieved = store
        .get_config("/api/v1/test-update", UserTier::Premium)
        .await
        .unwrap();
    assert!(retrieved.is_some());

    let config = retrieved.unwrap();
    assert_eq!(config.requests_per_minute, 150);
    assert_eq!(config.requests_per_hour, 7500);
    assert_eq!(config.burst_size, 200);

    store
        .delete_config("/api/v1/test-update", UserTier::Premium)
        .await
        .unwrap();
}

#[actix_web::test]
async fn test_delete_rate_limit_endpoint() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = Arc::new(RateLimitConfigStore::new(redis_client, db_pool));

    let config = media_gateway_auth::rate_limit_config::RateLimitConfig::new(
        "/api/v1/test-delete".to_string(),
        UserTier::Free,
        20,
        200,
        30,
    );
    store.set_config(&config).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
            .service(delete_rate_limit),
    )
    .await;

    let jwt = create_test_jwt();

    let req = test::TestRequest::delete()
        .uri("/api/v1/admin/rate-limits/api/v1/test-delete?tier=free")
        .insert_header(("Authorization", format!("Bearer {}", jwt)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let retrieved = store
        .get_config("/api/v1/test-delete", UserTier::Free)
        .await
        .unwrap();
    assert!(retrieved.is_none());
}

#[actix_web::test]
async fn test_rate_limit_config_validation() {
    let invalid_zero_limits = UpdateRateLimitConfigRequest {
        tier: "free".to_string(),
        requests_per_minute: 0,
        requests_per_hour: 0,
        burst_size: 10,
    };
    assert!(invalid_zero_limits.validate().is_err());

    let invalid_burst = UpdateRateLimitConfigRequest {
        tier: "free".to_string(),
        requests_per_minute: 50,
        requests_per_hour: 500,
        burst_size: 30,
    };
    assert!(invalid_burst.validate().is_err());

    let valid = UpdateRateLimitConfigRequest {
        tier: "free".to_string(),
        requests_per_minute: 50,
        requests_per_hour: 500,
        burst_size: 75,
    };
    assert!(valid.validate().is_ok());
}

#[actix_web::test]
async fn test_get_rate_limit_endpoint_with_default() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = Arc::new(RateLimitConfigStore::new(redis_client, db_pool));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
            .service(get_rate_limit),
    )
    .await;

    let jwt = create_test_jwt();

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/rate-limits/api/v1/nonexistent?tier=anonymous")
        .insert_header(("Authorization", format!("Bearer {}", jwt)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[tokio::test]
async fn test_user_tier_defaults() {
    let redis_client = setup_redis_client();
    if redis_client
        .get_multiplexed_async_connection()
        .await
        .is_err()
    {
        println!("Redis not available, skipping test");
        return;
    }

    let db_pool = setup_db_pool().await;
    let store = RateLimitConfigStore::new(redis_client, db_pool);

    let anonymous = store.get_default_config(UserTier::Anonymous).await;
    assert_eq!(anonymous.requests_per_minute, 10);
    assert_eq!(anonymous.requests_per_hour, 100);

    let free = store.get_default_config(UserTier::Free).await;
    assert_eq!(free.requests_per_minute, 30);
    assert_eq!(free.requests_per_hour, 1000);

    let premium = store.get_default_config(UserTier::Premium).await;
    assert_eq!(premium.requests_per_minute, 100);
    assert_eq!(premium.requests_per_hour, 5000);

    let enterprise = store.get_default_config(UserTier::Enterprise).await;
    assert_eq!(enterprise.requests_per_minute, 500);
    assert_eq!(enterprise.requests_per_hour, 50000);
}
