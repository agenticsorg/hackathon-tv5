use chrono::Utc;
use media_gateway_discovery::catalog::{
    AvailabilityUpdate, CatalogService, ContentResponse, CreateContentRequest, UpdateContentRequest,
};
use qdrant_client::Qdrant;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_service() -> (CatalogService, PgPool) {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::query("TRUNCATE content, platform_ids, content_genres, content_ratings, platform_availability CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to clean test database");

    let qdrant_url =
        std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());
    let qdrant_client = Arc::new(
        Qdrant::from_url(&qdrant_url)
            .build()
            .expect("Failed to create Qdrant client"),
    );

    let openai_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "test_key".to_string());

    let service = CatalogService::new(
        pool.clone(),
        qdrant_client,
        "test_media_content".to_string(),
        openai_key,
        "https://api.openai.com/v1/embeddings".to_string(),
    );

    (service, pool)
}

#[tokio::test]
async fn test_create_content_success() {
    let (service, _pool) = setup_test_service().await;

    let request = CreateContentRequest {
        title: "Test Movie".to_string(),
        content_type: media_gateway_discovery::catalog::types::ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "nf_test_123".to_string(),
        overview: Some("A great test movie".to_string()),
        release_year: Some(2024),
        runtime_minutes: Some(120),
        genres: vec!["action".to_string(), "drama".to_string()],
        rating: Some("PG-13".to_string()),
        images: media_gateway_discovery::catalog::types::ImageSet::default(),
    };

    let result = service.create_content(request.clone()).await;
    assert!(
        result.is_ok(),
        "Failed to create content: {:?}",
        result.err()
    );

    let content = result.unwrap();
    assert_eq!(content.title, "Test Movie");
    assert_eq!(content.platform, "netflix");
    assert_eq!(content.release_year, Some(2024));
    assert_eq!(content.genres.len(), 2);
}

#[tokio::test]
async fn test_create_content_validation_error() {
    let (service, _pool) = setup_test_service().await;

    let request = CreateContentRequest {
        title: "".to_string(),
        content_type: media_gateway_discovery::catalog::types::ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "nf_test_123".to_string(),
        overview: None,
        release_year: None,
        runtime_minutes: None,
        genres: vec![],
        rating: None,
        images: media_gateway_discovery::catalog::types::ImageSet::default(),
    };

    let result = service.create_content(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_content_success() {
    let (service, _pool) = setup_test_service().await;

    let create_request = CreateContentRequest {
        title: "Retrievable Movie".to_string(),
        content_type: media_gateway_discovery::catalog::types::ContentType::Movie,
        platform: "disney_plus".to_string(),
        platform_content_id: "dp_test_456".to_string(),
        overview: Some("Test overview".to_string()),
        release_year: Some(2023),
        runtime_minutes: Some(95),
        genres: vec!["comedy".to_string()],
        rating: Some("PG".to_string()),
        images: media_gateway_discovery::catalog::types::ImageSet::default(),
    };

    let created = service.create_content(create_request).await.unwrap();

    let retrieved = service.get_content(created.id).await;
    assert!(retrieved.is_ok());

    let content = retrieved.unwrap();
    assert!(content.is_some());

    let content = content.unwrap();
    assert_eq!(content.id, created.id);
    assert_eq!(content.title, "Retrievable Movie");
    assert_eq!(content.platform, "disney_plus");
}

#[tokio::test]
async fn test_get_content_not_found() {
    let (service, _pool) = setup_test_service().await;

    let random_id = Uuid::new_v4();
    let result = service.get_content(random_id).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_update_content_success() {
    let (service, _pool) = setup_test_service().await;

    let create_request = CreateContentRequest {
        title: "Original Title".to_string(),
        content_type: media_gateway_discovery::catalog::types::ContentType::Series,
        platform: "hulu".to_string(),
        platform_content_id: "hulu_test_789".to_string(),
        overview: Some("Original overview".to_string()),
        release_year: Some(2022),
        runtime_minutes: Some(45),
        genres: vec!["drama".to_string()],
        rating: Some("TV-14".to_string()),
        images: media_gateway_discovery::catalog::types::ImageSet::default(),
    };

    let created = service.create_content(create_request).await.unwrap();

    let update_request = UpdateContentRequest {
        title: Some("Updated Title".to_string()),
        overview: Some("Updated overview".to_string()),
        genres: Some(vec!["drama".to_string(), "thriller".to_string()]),
        rating: Some("TV-MA".to_string()),
        images: None,
    };

    let updated = service.update_content(created.id, update_request).await;
    assert!(
        updated.is_ok(),
        "Failed to update content: {:?}",
        updated.err()
    );

    let content = updated.unwrap();
    assert_eq!(content.title, "Updated Title");
    assert_eq!(content.overview, Some("Updated overview".to_string()));
    assert_eq!(content.genres.len(), 2);
    assert!(content.genres.contains(&"thriller".to_string()));
}

#[tokio::test]
async fn test_delete_content_success() {
    let (service, _pool) = setup_test_service().await;

    let create_request = CreateContentRequest {
        title: "To Be Deleted".to_string(),
        content_type: media_gateway_discovery::catalog::types::ContentType::Movie,
        platform: "prime_video".to_string(),
        platform_content_id: "pv_test_999".to_string(),
        overview: None,
        release_year: Some(2021),
        runtime_minutes: Some(110),
        genres: vec!["horror".to_string()],
        rating: None,
        images: media_gateway_discovery::catalog::types::ImageSet::default(),
    };

    let created = service.create_content(create_request).await.unwrap();

    let delete_result = service.delete_content(created.id).await;
    assert!(delete_result.is_ok());

    let retrieved = service.get_content(created.id).await.unwrap();
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_update_availability_success() {
    let (service, _pool) = setup_test_service().await;

    let create_request = CreateContentRequest {
        title: "Available Content".to_string(),
        content_type: media_gateway_discovery::catalog::types::ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "nf_avail_123".to_string(),
        overview: Some("Test".to_string()),
        release_year: Some(2024),
        runtime_minutes: Some(100),
        genres: vec!["action".to_string()],
        rating: None,
        images: media_gateway_discovery::catalog::types::ImageSet::default(),
    };

    let created = service.create_content(create_request).await.unwrap();

    let availability_update = AvailabilityUpdate {
        regions: vec!["US".to_string(), "CA".to_string()],
        subscription_required: true,
        purchase_price: None,
        rental_price: None,
        available_from: Some(Utc::now()),
        available_until: None,
    };

    let result = service
        .update_availability(created.id, availability_update)
        .await;
    assert!(
        result.is_ok(),
        "Failed to update availability: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_create_content_with_kafka() {
    let kafka_brokers =
        std::env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());

    let (mut service, _pool) = setup_test_service().await;

    if let Ok(service_with_kafka) = service.with_kafka(&kafka_brokers) {
        service = service_with_kafka;
    }

    let request = CreateContentRequest {
        title: "Kafka Test Movie".to_string(),
        content_type: media_gateway_discovery::catalog::types::ContentType::Movie,
        platform: "netflix".to_string(),
        platform_content_id: "nf_kafka_test".to_string(),
        overview: Some("Testing Kafka integration".to_string()),
        release_year: Some(2024),
        runtime_minutes: Some(115),
        genres: vec!["sci-fi".to_string()],
        rating: Some("PG-13".to_string()),
        images: media_gateway_discovery::catalog::types::ImageSet::default(),
    };

    let result = service.create_content(request).await;
    assert!(result.is_ok());
}
