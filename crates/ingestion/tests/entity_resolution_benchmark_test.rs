use chrono::Utc;
use media_gateway_ingestion::{
    entity_resolution::EntityResolver,
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

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_entity_mappings_external ON entity_mappings(external_id, id_type)")
        .execute(&pool)
        .await
        .ok();
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_entity_mappings_entity ON entity_mappings(entity_id)",
    )
    .execute(&pool)
    .await
    .ok();

    pool
}

async fn cleanup_test_data(pool: &PgPool) {
    sqlx::query("DELETE FROM entity_mappings")
        .execute(pool)
        .await
        .expect("Failed to clean up test data");
}

#[tokio::test]
async fn benchmark_cache_lookup_performance() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    let resolver = EntityResolver::new(pool.clone())
        .await
        .expect("Failed to create resolver");

    resolver
        .add_entity(
            "bench_entity_1".to_string(),
            "Benchmark Movie".to_string(),
            Some(2020),
            Some("10.5240/BENCH-1".to_string()),
            None,
            None,
            None,
        )
        .await
        .expect("Failed to add entity");

    let mut content = CanonicalContent {
        platform_content_id: "bench1".to_string(),
        platform_id: "netflix".to_string(),
        entity_id: None,
        title: "Benchmark Movie".to_string(),
        overview: None,
        content_type: ContentType::Movie,
        release_year: Some(2020),
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
        .insert("eidr".to_string(), "10.5240/BENCH-1".to_string());

    resolver.resolve(&content).await.unwrap();

    let iterations = 100;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let result = resolver.resolve(&content).await.unwrap();
        assert_eq!(result.entity_id, Some("bench_entity_1".to_string()));
    }

    let total_duration = start.elapsed();
    let avg_duration_micros = total_duration.as_micros() / iterations;

    println!("Average cache lookup time: {}μs", avg_duration_micros);
    assert!(
        avg_duration_micros < 5000,
        "Cache lookup should be <5ms (5000μs), got {}μs",
        avg_duration_micros
    );

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn benchmark_database_lookup_performance() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    let resolver = EntityResolver::new(pool.clone())
        .await
        .expect("Failed to create resolver");

    for i in 0..10 {
        resolver
            .add_entity(
                format!("entity_{}", i),
                format!("Movie {}", i),
                Some(2020 + i),
                None,
                Some(format!("tt{:07}", i)),
                None,
                None,
            )
            .await
            .expect("Failed to add entity");
    }

    let iterations = 50;
    let mut total_duration = std::time::Duration::ZERO;

    for i in 0..iterations {
        let mut content = CanonicalContent {
            platform_content_id: format!("bench{}", i),
            platform_id: "netflix".to_string(),
            entity_id: None,
            title: format!("Movie {}", i % 10),
            overview: None,
            content_type: ContentType::Movie,
            release_year: Some(2020 + (i % 10)),
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
            .insert("imdb".to_string(), format!("tt{:07}", i % 10));

        let start = std::time::Instant::now();
        let result = resolver.resolve(&content).await.unwrap();
        total_duration += start.elapsed();

        assert_eq!(result.entity_id, Some(format!("entity_{}", i % 10)));
    }

    let avg_duration_micros = total_duration.as_micros() / iterations;

    println!(
        "Average database lookup time (with cache): {}μs",
        avg_duration_micros
    );
    assert!(
        avg_duration_micros < 20000,
        "Database lookup should be <20ms (20000μs), got {}μs",
        avg_duration_micros
    );

    cleanup_test_data(&pool).await;
}

#[tokio::test]
async fn benchmark_persistence_write_performance() {
    let pool = setup_test_pool().await;
    cleanup_test_data(&pool).await;

    let resolver = EntityResolver::new(pool.clone())
        .await
        .expect("Failed to create resolver");

    let iterations = 100;
    let start = std::time::Instant::now();

    for i in 0..iterations {
        resolver
            .add_entity(
                format!("write_entity_{}", i),
                format!("Write Test {}", i),
                Some(2020),
                None,
                Some(format!("tt{:07}", 1000 + i)),
                None,
                None,
            )
            .await
            .expect("Failed to add entity");
    }

    let total_duration = start.elapsed();
    let avg_duration_micros = total_duration.as_micros() / iterations;

    println!("Average persistence write time: {}μs", avg_duration_micros);
    assert!(
        avg_duration_micros < 50000,
        "Write should be <50ms (50000μs), got {}μs",
        avg_duration_micros
    );

    cleanup_test_data(&pool).await;
}
