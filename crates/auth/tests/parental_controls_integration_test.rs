use media_gateway_auth::{
    parental::{
        controls::{
            get_parental_controls, set_parental_controls, ContentRating, ParentalControls,
            SetParentalControlsRequest,
        },
        verification::{verify_pin, VerifyPinRequest},
    },
    AuthError,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/test_auth".to_string());

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

async fn setup_redis() -> redis::Client {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    redis::Client::open(redis_url).expect("Failed to connect to Redis")
}

async fn create_test_user(pool: &PgPool, email: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, display_name)
        VALUES ($1, $2, $3, $4)
        "#,
        user_id,
        email,
        "hashed_password",
        "Test User"
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

async fn cleanup_user(pool: &PgPool, user_id: Uuid) {
    sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(pool)
        .await
        .ok();
}

#[tokio::test]
async fn test_content_rating_hierarchy() {
    assert!(ContentRating::G < ContentRating::PG);
    assert!(ContentRating::PG < ContentRating::PG13);
    assert!(ContentRating::PG13 < ContentRating::R);
    assert!(ContentRating::R < ContentRating::NC17);
}

#[tokio::test]
async fn test_content_rating_from_str() {
    assert_eq!(ContentRating::from_str("G").unwrap(), ContentRating::G);
    assert_eq!(ContentRating::from_str("PG").unwrap(), ContentRating::PG);
    assert_eq!(
        ContentRating::from_str("PG-13").unwrap(),
        ContentRating::PG13
    );
    assert_eq!(
        ContentRating::from_str("pg13").unwrap(),
        ContentRating::PG13
    );
    assert_eq!(ContentRating::from_str("R").unwrap(), ContentRating::R);
    assert_eq!(
        ContentRating::from_str("NC-17").unwrap(),
        ContentRating::NC17
    );
    assert!(ContentRating::from_str("invalid").is_err());
}

#[tokio::test]
async fn test_set_parental_controls_creates_new_controls() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, "parental1@example.com").await;

    let request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("1234".to_string()),
        content_rating_limit: Some("PG-13".to_string()),
        viewing_time_start: Some("06:00".to_string()),
        viewing_time_end: Some("21:00".to_string()),
        blocked_genres: Some(vec!["horror".to_string(), "thriller".to_string()]),
    };

    let controls = set_parental_controls(&pool, user_id, request)
        .await
        .expect("Failed to set parental controls");

    assert!(controls.enabled);
    assert!(controls.pin_hash.is_some());
    assert_eq!(controls.content_rating_limit, ContentRating::PG13);
    assert!(controls.viewing_time_start.is_some());
    assert!(controls.viewing_time_end.is_some());
    assert_eq!(controls.blocked_genres.len(), 2);

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_set_parental_controls_updates_existing_controls() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, "parental2@example.com").await;

    // Set initial controls
    let initial_request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("1234".to_string()),
        content_rating_limit: Some("PG".to_string()),
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: None,
    };

    set_parental_controls(&pool, user_id, initial_request)
        .await
        .expect("Failed to set initial controls");

    // Update controls
    let update_request = SetParentalControlsRequest {
        enabled: true,
        pin: None, // Don't change PIN
        content_rating_limit: Some("PG-13".to_string()),
        viewing_time_start: Some("08:00".to_string()),
        viewing_time_end: Some("20:00".to_string()),
        blocked_genres: Some(vec!["horror".to_string()]),
    };

    let updated_controls = set_parental_controls(&pool, user_id, update_request)
        .await
        .expect("Failed to update controls");

    assert!(updated_controls.enabled);
    assert!(updated_controls.pin_hash.is_some()); // PIN should still be there
    assert_eq!(updated_controls.content_rating_limit, ContentRating::PG13);
    assert!(updated_controls.viewing_time_start.is_some());
    assert_eq!(updated_controls.blocked_genres.len(), 1);

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_get_parental_controls_returns_none_for_unconfigured_user() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, "parental3@example.com").await;

    let controls = get_parental_controls(&pool, user_id)
        .await
        .expect("Failed to get controls");

    assert!(controls.is_none());

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_get_parental_controls_returns_configured_controls() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, "parental4@example.com").await;

    // Set controls
    let request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("5678".to_string()),
        content_rating_limit: Some("R".to_string()),
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: Some(vec!["horror".to_string()]),
    };

    set_parental_controls(&pool, user_id, request)
        .await
        .expect("Failed to set controls");

    // Get controls
    let controls = get_parental_controls(&pool, user_id)
        .await
        .expect("Failed to get controls")
        .expect("Controls should exist");

    assert!(controls.enabled);
    assert_eq!(controls.content_rating_limit, ContentRating::R);
    assert_eq!(controls.blocked_genres, vec!["horror".to_string()]);

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_pin_validation_requires_4_digits() {
    assert!(ParentalControls::validate_pin("1234").is_ok());
    assert!(ParentalControls::validate_pin("123").is_err());
    assert!(ParentalControls::validate_pin("12345").is_err());
    assert!(ParentalControls::validate_pin("abcd").is_err());
    assert!(ParentalControls::validate_pin("12a4").is_err());
}

#[tokio::test]
async fn test_pin_hashing_and_verification() {
    let pin = "1234";
    let hash = ParentalControls::hash_pin(pin).expect("Failed to hash PIN");

    let controls = ParentalControls {
        enabled: true,
        pin_hash: Some(hash),
        content_rating_limit: ContentRating::PG13,
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: Vec::new(),
    };

    assert!(controls.verify_pin("1234").expect("Verification failed"));
    assert!(!controls.verify_pin("4321").expect("Verification failed"));
}

#[tokio::test]
async fn test_verify_pin_with_redis_caching() {
    let pool = setup_test_db().await;
    let redis_client = setup_redis().await;
    let user_id = create_test_user(&pool, "parental5@example.com").await;

    // Set controls with PIN
    let request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("9876".to_string()),
        content_rating_limit: Some("PG".to_string()),
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: None,
    };

    set_parental_controls(&pool, user_id, request)
        .await
        .expect("Failed to set controls");

    // Verify correct PIN
    let verify_request = VerifyPinRequest {
        pin: "9876".to_string(),
    };

    let response = verify_pin(
        &pool,
        &redis_client,
        user_id,
        verify_request,
        "test-jwt-secret",
    )
    .await
    .expect("Failed to verify PIN");

    assert!(response.verified);
    assert!(response.token.is_some());
    assert!(response.expires_at.is_some());

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_verify_pin_rejects_incorrect_pin() {
    let pool = setup_test_db().await;
    let redis_client = setup_redis().await;
    let user_id = create_test_user(&pool, "parental6@example.com").await;

    // Set controls with PIN
    let request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("1111".to_string()),
        content_rating_limit: Some("PG".to_string()),
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: None,
    };

    set_parental_controls(&pool, user_id, request)
        .await
        .expect("Failed to set controls");

    // Verify incorrect PIN
    let verify_request = VerifyPinRequest {
        pin: "2222".to_string(),
    };

    let response = verify_pin(
        &pool,
        &redis_client,
        user_id,
        verify_request,
        "test-jwt-secret",
    )
    .await
    .expect("Failed to verify PIN");

    assert!(!response.verified);
    assert!(response.token.is_none());
    assert!(response.expires_at.is_none());

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_content_filtering_by_rating() {
    let controls = ParentalControls {
        enabled: true,
        pin_hash: None,
        content_rating_limit: ContentRating::PG13,
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: Vec::new(),
    };

    // Should allow content at or below PG-13
    assert!(controls.is_content_allowed(ContentRating::G));
    assert!(controls.is_content_allowed(ContentRating::PG));
    assert!(controls.is_content_allowed(ContentRating::PG13));

    // Should block content above PG-13
    assert!(!controls.is_content_allowed(ContentRating::R));
    assert!(!controls.is_content_allowed(ContentRating::NC17));
}

#[tokio::test]
async fn test_genre_blocking() {
    let controls = ParentalControls {
        enabled: true,
        pin_hash: None,
        content_rating_limit: ContentRating::NC17,
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: vec!["horror".to_string(), "thriller".to_string()],
    };

    assert!(controls.is_genre_blocked("horror"));
    assert!(controls.is_genre_blocked("Horror")); // Case-insensitive
    assert!(controls.is_genre_blocked("THRILLER"));
    assert!(!controls.is_genre_blocked("comedy"));
    assert!(!controls.is_genre_blocked("action"));
}

#[tokio::test]
async fn test_disabled_controls_allow_all_content() {
    let controls = ParentalControls {
        enabled: false,
        pin_hash: None,
        content_rating_limit: ContentRating::G,
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: vec!["horror".to_string()],
    };

    // When disabled, all content should be allowed
    assert!(controls.is_content_allowed(ContentRating::NC17));
    assert!(!controls.is_genre_blocked("horror"));
}

#[tokio::test]
async fn test_invalid_pin_format_returns_validation_error() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, "parental7@example.com").await;

    let request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("abc".to_string()), // Invalid PIN
        content_rating_limit: Some("PG".to_string()),
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: None,
    };

    let result = set_parental_controls(&pool, user_id, request).await;

    assert!(result.is_err());
    match result {
        Err(AuthError::ValidationError(_)) => {}
        _ => panic!("Expected ValidationError"),
    }

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_invalid_content_rating_returns_validation_error() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, "parental8@example.com").await;

    let request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("1234".to_string()),
        content_rating_limit: Some("INVALID".to_string()),
        viewing_time_start: None,
        viewing_time_end: None,
        blocked_genres: None,
    };

    let result = set_parental_controls(&pool, user_id, request).await;

    assert!(result.is_err());
    match result {
        Err(AuthError::ValidationError(_)) => {}
        _ => panic!("Expected ValidationError"),
    }

    cleanup_user(&pool, user_id).await;
}

#[tokio::test]
async fn test_invalid_time_format_returns_validation_error() {
    let pool = setup_test_db().await;
    let user_id = create_test_user(&pool, "parental9@example.com").await;

    let request = SetParentalControlsRequest {
        enabled: true,
        pin: Some("1234".to_string()),
        content_rating_limit: Some("PG".to_string()),
        viewing_time_start: Some("25:00".to_string()), // Invalid time
        viewing_time_end: None,
        blocked_genres: None,
    };

    let result = set_parental_controls(&pool, user_id, request).await;

    assert!(result.is_err());
    match result {
        Err(AuthError::ValidationError(_)) => {}
        _ => panic!("Expected ValidationError"),
    }

    cleanup_user(&pool, user_id).await;
}
