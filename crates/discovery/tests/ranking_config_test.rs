use media_gateway_discovery::{RankingConfig, RankingConfigStore};
use uuid::Uuid;

#[tokio::test]
async fn test_ranking_config_default() {
    let config = RankingConfig::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.total_weight(), 1.0);
    assert_eq!(config.version, 1);
}

#[tokio::test]
async fn test_ranking_config_validation_success() {
    let config = RankingConfig::new(0.35, 0.30, 0.20, 0.15, None, None);
    assert!(config.is_ok());

    let config = config.unwrap();
    assert_eq!(config.vector_weight, 0.35);
    assert_eq!(config.keyword_weight, 0.30);
    assert_eq!(config.quality_weight, 0.20);
    assert_eq!(config.freshness_weight, 0.15);
}

#[tokio::test]
async fn test_ranking_config_validation_sum_error() {
    let config = RankingConfig::new(0.5, 0.3, 0.2, 0.2, None, None);
    assert!(config.is_err());
    assert!(config.unwrap_err().to_string().contains("must sum to 1.0"));
}

#[tokio::test]
async fn test_ranking_config_validation_negative_error() {
    let config = RankingConfig::new(0.5, -0.1, 0.3, 0.3, None, None);
    assert!(config.is_err());
    assert!(config
        .unwrap_err()
        .to_string()
        .contains("must be non-negative"));
}

#[tokio::test]
async fn test_ranking_config_store_get_default() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let config = store.get_default_config().await.unwrap();
    assert!(config.validate().is_ok());
    assert_eq!(config.total_weight(), 1.0);
}

#[tokio::test]
async fn test_ranking_config_store_update_default() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let admin_id = Uuid::new_v4();
    let new_config = RankingConfig::new(
        0.4,
        0.3,
        0.2,
        0.1,
        Some(admin_id),
        Some("Test update".to_string()),
    )
    .unwrap();

    store
        .set_default_config(&new_config, Some(admin_id))
        .await
        .unwrap();

    let retrieved = store.get_default_config().await.unwrap();
    assert_eq!(retrieved.vector_weight, 0.4);
    assert_eq!(retrieved.keyword_weight, 0.3);
    assert_eq!(retrieved.quality_weight, 0.2);
    assert_eq!(retrieved.freshness_weight, 0.1);
    assert_eq!(retrieved.created_by, Some(admin_id));
    assert!(retrieved.version > 0);
}

#[tokio::test]
async fn test_ranking_config_store_named_config() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let admin_id = Uuid::new_v4();
    let config = RankingConfig::new(
        0.5,
        0.25,
        0.15,
        0.1,
        Some(admin_id),
        Some("High vector weight variant".to_string()),
    )
    .unwrap();

    store
        .set_named_config("high_vector", &config, true, Some(50), Some(admin_id))
        .await
        .unwrap();

    let named = store.get_named_config("high_vector").await.unwrap();
    assert!(named.is_some());

    let named = named.unwrap();
    assert_eq!(named.name, "high_vector");
    assert_eq!(named.config.vector_weight, 0.5);
    assert_eq!(named.is_active, true);
    assert_eq!(named.traffic_percentage, Some(50));
}

#[tokio::test]
async fn test_ranking_config_store_list_named_configs() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let admin_id = Uuid::new_v4();

    let config1 = RankingConfig::new(0.5, 0.25, 0.15, 0.1, Some(admin_id), None).unwrap();
    let config2 = RankingConfig::new(0.3, 0.4, 0.2, 0.1, Some(admin_id), None).unwrap();

    store
        .set_named_config("variant_a", &config1, true, Some(50), Some(admin_id))
        .await
        .unwrap();

    store
        .set_named_config("variant_b", &config2, false, Some(0), Some(admin_id))
        .await
        .unwrap();

    let configs = store.list_named_configs().await.unwrap();
    assert!(configs.len() >= 2);

    let names: Vec<String> = configs.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"variant_a".to_string()));
    assert!(names.contains(&"variant_b".to_string()));
}

#[tokio::test]
async fn test_ranking_config_store_delete_named_config() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let admin_id = Uuid::new_v4();
    let config = RankingConfig::new(0.4, 0.3, 0.2, 0.1, Some(admin_id), None).unwrap();

    store
        .set_named_config("to_delete", &config, true, None, Some(admin_id))
        .await
        .unwrap();

    let deleted = store
        .delete_named_config("to_delete", Some(admin_id))
        .await
        .unwrap();
    assert!(deleted);

    let retrieved = store.get_named_config("to_delete").await.unwrap();
    assert!(retrieved.is_none());

    let deleted_again = store
        .delete_named_config("to_delete", Some(admin_id))
        .await
        .unwrap();
    assert!(!deleted_again);
}

#[tokio::test]
async fn test_ranking_config_store_get_config_for_variant() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let admin_id = Uuid::new_v4();
    let variant_config = RankingConfig::new(
        0.6,
        0.2,
        0.1,
        0.1,
        Some(admin_id),
        Some("Variant".to_string()),
    )
    .unwrap();

    store
        .set_named_config(
            "test_variant",
            &variant_config,
            true,
            Some(100),
            Some(admin_id),
        )
        .await
        .unwrap();

    let config_for_variant = store
        .get_config_for_variant(Some("test_variant"))
        .await
        .unwrap();
    assert_eq!(config_for_variant.vector_weight, 0.6);

    let config_for_default = store.get_config_for_variant(None).await.unwrap();
    assert!(config_for_default.validate().is_ok());

    store
        .delete_named_config("test_variant", Some(admin_id))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_ranking_config_store_inactive_variant_falls_back_to_default() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let admin_id = Uuid::new_v4();
    let inactive_config = RankingConfig::new(
        0.6,
        0.2,
        0.1,
        0.1,
        Some(admin_id),
        Some("Inactive".to_string()),
    )
    .unwrap();

    store
        .set_named_config(
            "inactive_variant",
            &inactive_config,
            false,
            Some(0),
            Some(admin_id),
        )
        .await
        .unwrap();

    let config = store
        .get_config_for_variant(Some("inactive_variant"))
        .await
        .unwrap();

    let default_config = store.get_default_config().await.unwrap();
    assert_eq!(config.vector_weight, default_config.vector_weight);

    store
        .delete_named_config("inactive_variant", Some(admin_id))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_ranking_config_versioning() {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/media_gateway".to_string());

    let db_pool = match sqlx::PgPool::connect(&db_url).await {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Skipping test: PostgreSQL not available");
            return;
        }
    };

    let store = match RankingConfigStore::new(&redis_url, db_pool).await {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Skipping test: Redis not available");
            return;
        }
    };

    let admin_id = Uuid::new_v4();

    let config1 = RankingConfig::new(0.4, 0.3, 0.2, 0.1, Some(admin_id), None).unwrap();
    store
        .set_default_config(&config1, Some(admin_id))
        .await
        .unwrap();
    let version1 = store.get_default_config().await.unwrap().version;

    let config2 = RankingConfig::new(0.35, 0.35, 0.2, 0.1, Some(admin_id), None).unwrap();
    store
        .set_default_config(&config2, Some(admin_id))
        .await
        .unwrap();
    let version2 = store.get_default_config().await.unwrap().version;

    assert!(version2 > version1);

    let history = store.get_config_history(version2).await.unwrap();
    assert!(history.is_some());
    assert_eq!(history.unwrap().vector_weight, 0.35);
}
