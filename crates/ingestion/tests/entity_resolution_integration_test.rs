use chrono::Utc;
use media_gateway_ingestion::{
    entity_resolution::{EntityMatch, EntityResolver, MatchMethod},
    normalizer::{AvailabilityInfo, CanonicalContent, ContentType, ImageSet},
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;

async fn setup_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:postgres@localhost:5432/media_gateway_test".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
        .execute(&pool)
        .await
        .ok();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS entity_mappings (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            external_id VARCHAR(100) NOT NULL,
            id_type VARCHAR(20) NOT NULL,
            entity_id VARCHAR(100) NOT NULL,
            confidence FLOAT NOT NULL DEFAULT 1.0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(external_id, id_type)
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("Failed to create entity_mappings table");

    pool
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM entity_mappings")
        .execute(pool)
        .await
        .expect("Failed to clean up test data");
}

#[tokio::test]
async fn test_eidr_exact_match_with_persistence() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    let resolver = EntityResolver::new(pool.clone())
        .await
        .expect("Failed to create resolver");

    resolver
        .add_entity(
            "entity_1".to_string(),
            "The Matrix".to_string(),
            Some(1999),
            Some("10.5240/ABCD-1234".to_string()),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add entity");

    let mut content = CanonicalContent {
        platform_content_id: "test".to_string(),
        platform_id: "netflix".to_string(),
        entity_id: None,
        title: "The Matrix".to_string(),
        overview: None,
        content_type: ContentType::Movie,
        release_year: Some(1999),
        runtime_minutes: None,
        genres: vec![],
        external_ids: HashMap::new(),
        availability: AvailabilityInfo {
            regions: vec![],
            subscription_required: false,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: None,
            available_until: None,
        },
        images: ImageSet::default(),
        rating: None,
        user_rating: None,
        embedding: None,
        updated_at: Utc::now(),
    };

    content
        .external_ids
        .insert("eidr".to_string(), "10.5240/ABCD-1234".to_string());

    let result = resolver.resolve(&content).await.unwrap();
    assert_eq!(result.entity_id, Some("entity_1".to_string()));
    assert_eq!(result.confidence, 1.0);
    assert_eq!(result.method, MatchMethod::EidrExact);

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM entity_mappings WHERE id_type = 'eidr'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_imdb_match_with_persistence() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    let resolver = EntityResolver::new(pool.clone())
        .await
        .expect("Failed to create resolver");

    resolver
        .add_entity(
            "entity_2".to_string(),
            "Inception".to_string(),
            Some(2010),
            None,
            Some("tt1375666".to_string()),
            None,
            None,
        )
        .await
        .expect("Failed to add entity");

    let mut content = CanonicalContent {
        platform_content_id: "test2".to_string(),
        platform_id: "hulu".to_string(),
        entity_id: None,
        title: "Inception".to_string(),
        overview: None,
        content_type: ContentType::Movie,
        release_year: Some(2010),
        runtime_minutes: None,
        genres: vec![],
        external_ids: HashMap::new(),
        availability: AvailabilityInfo {
            regions: vec![],
            subscription_required: false,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: None,
            available_until: None,
        },
        images: ImageSet::default(),
        rating: None,
        user_rating: None,
        embedding: None,
        updated_at: Utc::now(),
    };

    content
        .external_ids
        .insert("imdb".to_string(), "tt1375666".to_string());

    let result = resolver.resolve(&content).await.unwrap();
    assert_eq!(result.entity_id, Some("entity_2".to_string()));
    assert_eq!(result.confidence, 0.99);

    if let MatchMethod::ExternalId { source } = result.method {
        assert_eq!(source, "imdb");
    } else {
        panic!("Expected ExternalId method");
    }

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_persistence_across_restarts() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    {
        let resolver = EntityResolver::new(pool.clone())
            .await
            .expect("Failed to create resolver");

        resolver
            .add_entity(
                "entity_3".to_string(),
                "Interstellar".to_string(),
                Some(2014),
                None,
                Some("tt0816692".to_string()),
                Some("157336".to_string()),
                None,
            )
            .await
            .expect("Failed to add entity");
    }

    {
        let resolver = EntityResolver::new(pool.clone())
            .await
            .expect("Failed to create resolver after restart");

        let mut content = CanonicalContent {
            platform_content_id: "test3".to_string(),
            platform_id: "prime".to_string(),
            entity_id: None,
            title: "Interstellar".to_string(),
            overview: None,
            content_type: ContentType::Movie,
            release_year: Some(2014),
            runtime_minutes: None,
            genres: vec![],
            external_ids: HashMap::new(),
            availability: AvailabilityInfo {
                regions: vec![],
                subscription_required: false,
                purchase_price: None,
                rental_price: None,
                currency: None,
                available_from: None,
                available_until: None,
            },
            images: ImageSet::default(),
            rating: None,
            user_rating: None,
            embedding: None,
            updated_at: Utc::now(),
        };

        content
            .external_ids
            .insert("imdb".to_string(), "tt0816692".to_string());

        let result = resolver.resolve(&content).await.unwrap();
        assert_eq!(result.entity_id, Some("entity_3".to_string()));
        assert_eq!(result.confidence, 0.99);
    }

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_cache_performance() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    let resolver = EntityResolver::new(pool.clone())
        .await
        .expect("Failed to create resolver");

    resolver
        .add_entity(
            "entity_4".to_string(),
            "The Dark Knight".to_string(),
            Some(2008),
            Some("10.5240/DARK-KNIGHT".to_string()),
            Some("tt0468569".to_string()),
            None,
            None,
        )
        .await
        .expect("Failed to add entity");

    let mut content = CanonicalContent {
        platform_content_id: "test4".to_string(),
        platform_id: "netflix".to_string(),
        entity_id: None,
        title: "The Dark Knight".to_string(),
        overview: None,
        content_type: ContentType::Movie,
        release_year: Some(2008),
        runtime_minutes: None,
        genres: vec![],
        external_ids: HashMap::new(),
        availability: AvailabilityInfo {
            regions: vec![],
            subscription_required: false,
            purchase_price: None,
            rental_price: None,
            currency: None,
            available_from: None,
            available_until: None,
        },
        images: ImageSet::default(),
        rating: None,
        user_rating: None,
        embedding: None,
        updated_at: Utc::now(),
    };

    content
        .external_ids
        .insert("eidr".to_string(), "10.5240/DARK-KNIGHT".to_string());

    let start = std::time::Instant::now();
    let result1 = resolver.resolve(&content).await.unwrap();
    let first_duration = start.elapsed();

    let start = std::time::Instant::now();
    let result2 = resolver.resolve(&content).await.unwrap();
    let cached_duration = start.elapsed();

    assert_eq!(result1.entity_id, result2.entity_id);
    assert!(cached_duration < first_duration || cached_duration.as_micros() < 5000);

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn test_upsert_semantics() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    let resolver = EntityResolver::new(pool.clone())
        .await
        .expect("Failed to create resolver");

    resolver
        .add_entity(
            "entity_5".to_string(),
            "Blade Runner".to_string(),
            Some(1982),
            None,
            Some("tt0083658".to_string()),
            None,
            None,
        )
        .await
        .expect("Failed to add entity");

    resolver
        .add_entity(
            "entity_5_updated".to_string(),
            "Blade Runner".to_string(),
            Some(1982),
            None,
            Some("tt0083658".to_string()),
            None,
            None,
        )
        .await
        .expect("Failed to update entity");

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM entity_mappings WHERE external_id = 'tt0083658'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(count, 1);

    let entity_id: String = sqlx::query_scalar(
        "SELECT entity_id FROM entity_mappings WHERE external_id = 'tt0083658' AND id_type = 'imdb'"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(entity_id, "entity_5_updated");

    cleanup_test_data(&pool).await;
}
