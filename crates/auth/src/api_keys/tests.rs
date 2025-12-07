use super::*;
use crate::api_keys::manager::{ApiKeyManager, CreateApiKeyRequest};
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::test]
async fn test_api_key_lifecycle() {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url).await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            name VARCHAR(255) NOT NULL,
            key_prefix VARCHAR(12) NOT NULL,
            key_hash VARCHAR(64) NOT NULL,
            scopes TEXT[] NOT NULL DEFAULT '{}',
            rate_limit_per_minute INTEGER DEFAULT 60,
            expires_at TIMESTAMP WITH TIME ZONE,
            last_used_at TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            revoked_at TIMESTAMP WITH TIME ZONE,
            UNIQUE(key_prefix)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let manager = ApiKeyManager::new(pool.clone());
    let user_id = Uuid::new_v4();

    let request = CreateApiKeyRequest {
        name: "Production Key".to_string(),
        scopes: vec![
            "read:content".to_string(),
            "read:recommendations".to_string(),
            "write:watchlist".to_string(),
        ],
        rate_limit_per_minute: Some(120),
        expires_in_days: Some(365),
    };

    let created = manager.create_api_key(user_id, request).await.unwrap();
    assert_eq!(created.name, "Production Key");
    assert_eq!(created.scopes.len(), 3);
    assert_eq!(created.rate_limit_per_minute, 120);
    assert!(created.expires_at.is_some());

    let keys = manager.list_user_keys(user_id).await.unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].id, created.id);

    let verified = manager.verify_key(&created.key).await.unwrap();
    assert_eq!(verified.id, created.id);

    manager.revoke_key(user_id, created.id).await.unwrap();

    let verify_result = manager.verify_key(&created.key).await;
    assert!(verify_result.is_err());

    let keys_after_revoke = manager.list_user_keys(user_id).await.unwrap();
    assert_eq!(keys_after_revoke.len(), 0);

    sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(created.id)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_multiple_keys_per_user() {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url).await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            name VARCHAR(255) NOT NULL,
            key_prefix VARCHAR(12) NOT NULL,
            key_hash VARCHAR(64) NOT NULL,
            scopes TEXT[] NOT NULL DEFAULT '{}',
            rate_limit_per_minute INTEGER DEFAULT 60,
            expires_at TIMESTAMP WITH TIME ZONE,
            last_used_at TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            revoked_at TIMESTAMP WITH TIME ZONE,
            UNIQUE(key_prefix)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let manager = ApiKeyManager::new(pool.clone());
    let user_id = Uuid::new_v4();

    let request1 = CreateApiKeyRequest {
        name: "Read-Only Key".to_string(),
        scopes: vec!["read:content".to_string()],
        rate_limit_per_minute: Some(60),
        expires_in_days: None,
    };

    let request2 = CreateApiKeyRequest {
        name: "Read-Write Key".to_string(),
        scopes: vec!["read:content".to_string(), "write:watchlist".to_string()],
        rate_limit_per_minute: Some(120),
        expires_in_days: None,
    };

    let request3 = CreateApiKeyRequest {
        name: "Admin Key".to_string(),
        scopes: vec!["admin:full".to_string()],
        rate_limit_per_minute: Some(240),
        expires_in_days: Some(30),
    };

    let key1 = manager.create_api_key(user_id, request1).await.unwrap();
    let key2 = manager.create_api_key(user_id, request2).await.unwrap();
    let key3 = manager.create_api_key(user_id, request3).await.unwrap();

    let keys = manager.list_user_keys(user_id).await.unwrap();
    assert_eq!(keys.len(), 3);

    let verified1 = manager.verify_key(&key1.key).await.unwrap();
    assert_eq!(verified1.scopes, vec!["read:content"]);
    assert_eq!(verified1.rate_limit_per_minute, 60);

    let verified2 = manager.verify_key(&key2.key).await.unwrap();
    assert_eq!(verified2.scopes.len(), 2);
    assert_eq!(verified2.rate_limit_per_minute, 120);

    let verified3 = manager.verify_key(&key3.key).await.unwrap();
    assert_eq!(verified3.scopes, vec!["admin:full"]);
    assert!(verified3.expires_at.is_some());

    sqlx::query("DELETE FROM api_keys WHERE id = ANY($1)")
        .bind(&[key1.id, key2.id, key3.id])
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_scope_validation() {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url).await.unwrap();

    let manager = ApiKeyManager::new(pool.clone());
    let user_id = Uuid::new_v4();

    let invalid_request = CreateApiKeyRequest {
        name: "Invalid Scopes".to_string(),
        scopes: vec!["invalid:scope".to_string()],
        rate_limit_per_minute: None,
        expires_in_days: None,
    };

    let result = manager.create_api_key(user_id, invalid_request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_last_used() {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    let pool = PgPool::connect(&database_url).await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            name VARCHAR(255) NOT NULL,
            key_prefix VARCHAR(12) NOT NULL,
            key_hash VARCHAR(64) NOT NULL,
            scopes TEXT[] NOT NULL DEFAULT '{}',
            rate_limit_per_minute INTEGER DEFAULT 60,
            expires_at TIMESTAMP WITH TIME ZONE,
            last_used_at TIMESTAMP WITH TIME ZONE,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            revoked_at TIMESTAMP WITH TIME ZONE,
            UNIQUE(key_prefix)
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let manager = ApiKeyManager::new(pool.clone());
    let user_id = Uuid::new_v4();

    let request = CreateApiKeyRequest {
        name: "Last Used Test".to_string(),
        scopes: vec!["read:content".to_string()],
        rate_limit_per_minute: None,
        expires_in_days: None,
    };

    let created = manager.create_api_key(user_id, request).await.unwrap();

    let before_update = manager.verify_key(&created.key).await.unwrap();
    assert!(before_update.last_used_at.is_none());

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    manager.update_last_used(created.id).await.unwrap();

    let after_update = manager.verify_key(&created.key).await.unwrap();
    assert!(after_update.last_used_at.is_some());

    sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(created.id)
        .execute(&pool)
        .await
        .unwrap();
}
