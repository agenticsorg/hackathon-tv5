use media_gateway_api::circuit_breaker::CircuitBreakerManager;
use media_gateway_api::config::{CircuitBreakerServiceConfig, Config};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Helper to get a test Redis URL from environment or use default
fn get_test_redis_url() -> String {
    std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
}

/// Helper to clean up Redis keys after tests
async fn cleanup_redis_keys(redis_url: &str, pattern: &str) -> Result<(), redis::RedisError> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = ConnectionManager::new(client).await?;

    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(pattern)
        .query_async(&mut conn)
        .await?;

    if !keys.is_empty() {
        let _: () = redis::cmd("DEL").arg(&keys).query_async(&mut conn).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_persist_closed_state() {
    let redis_url = get_test_redis_url();
    cleanup_redis_keys(&redis_url, "circuit_breaker:test_persist_closed:*")
        .await
        .ok();

    let mut config = Config::default();
    config.redis.url = redis_url.clone();
    config.circuit_breaker.enabled = true;

    let manager = CircuitBreakerManager::with_redis(Arc::new(config))
        .await
        .expect("Failed to create manager with Redis");

    // Create circuit breaker and record success
    let result = manager
        .call("test_persist_closed", || Ok::<_, std::io::Error>("success"))
        .await;

    assert!(result.is_ok());

    // Verify state in Redis
    let client = redis::Client::open(redis_url.as_str()).unwrap();
    let mut conn = ConnectionManager::new(client).await.unwrap();

    let state_json: Option<String> = conn
        .get("circuit_breaker:test_persist_closed:state")
        .await
        .ok();

    assert!(state_json.is_some(), "State should be persisted to Redis");

    cleanup_redis_keys(&redis_url, "circuit_breaker:test_persist_closed:*")
        .await
        .ok();
}

#[tokio::test]
async fn test_circuit_breaker_persist_open_state() {
    let redis_url = get_test_redis_url();
    cleanup_redis_keys(&redis_url, "circuit_breaker:test_persist_open:*")
        .await
        .ok();

    let mut config = Config::default();
    config.redis.url = redis_url.clone();
    config.circuit_breaker.enabled = true;

    // Set low failure threshold
    let mut service_config = CircuitBreakerServiceConfig {
        failure_threshold: 2,
        timeout_seconds: 60,
        error_rate_threshold: 0.5,
    };
    config
        .circuit_breaker
        .services
        .insert("test_persist_open".to_string(), service_config.clone());

    let manager = CircuitBreakerManager::with_redis(Arc::new(config))
        .await
        .expect("Failed to create manager with Redis");

    // Trigger failures to open the circuit
    for _ in 0..3 {
        let _ = manager
            .call("test_persist_open", || {
                Err::<String, _>(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "test failure",
                ))
            })
            .await;
    }

    // Wait a bit for persistence
    sleep(Duration::from_millis(100)).await;

    // Verify state in Redis
    let client = redis::Client::open(redis_url.as_str()).unwrap();
    let mut conn = ConnectionManager::new(client).await.unwrap();

    let state_json: Option<String> = conn
        .get("circuit_breaker:test_persist_open:state")
        .await
        .unwrap();

    assert!(state_json.is_some(), "State should be persisted to Redis");

    let state_data: serde_json::Value = serde_json::from_str(&state_json.unwrap()).unwrap();
    assert_eq!(state_data["state"], "open");
    assert!(state_data["failure_count"].as_u64().unwrap() >= 2);

    cleanup_redis_keys(&redis_url, "circuit_breaker:test_persist_open:*")
        .await
        .ok();
}

#[tokio::test]
async fn test_circuit_breaker_load_state_from_redis() {
    let redis_url = get_test_redis_url();
    cleanup_redis_keys(&redis_url, "circuit_breaker:test_load_state:*")
        .await
        .ok();

    // First, create a circuit breaker and trigger failures
    {
        let mut config = Config::default();
        config.redis.url = redis_url.clone();
        config.circuit_breaker.enabled = true;

        let mut service_config = CircuitBreakerServiceConfig {
            failure_threshold: 2,
            timeout_seconds: 60,
            error_rate_threshold: 0.5,
        };
        config
            .circuit_breaker
            .services
            .insert("test_load_state".to_string(), service_config.clone());

        let manager1 = CircuitBreakerManager::with_redis(Arc::new(config))
            .await
            .expect("Failed to create manager with Redis");

        // Trigger failures to open the circuit
        for _ in 0..3 {
            let _ = manager1
                .call("test_load_state", || {
                    Err::<String, _>(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "test failure",
                    ))
                })
                .await;
        }

        // Wait for persistence
        sleep(Duration::from_millis(100)).await;

        // Verify circuit is open
        let state = manager1.get_state("test_load_state").await;
        assert_eq!(state, Some("open".to_string()));
    }

    // Now create a new manager and verify it loads the state
    {
        let mut config = Config::default();
        config.redis.url = redis_url.clone();
        config.circuit_breaker.enabled = true;

        let mut service_config = CircuitBreakerServiceConfig {
            failure_threshold: 2,
            timeout_seconds: 60,
            error_rate_threshold: 0.5,
        };
        config
            .circuit_breaker
            .services
            .insert("test_load_state".to_string(), service_config.clone());

        let manager2 = CircuitBreakerManager::with_redis(Arc::new(config))
            .await
            .expect("Failed to create manager with Redis");

        // Access the circuit breaker (this should load from Redis)
        let breaker = manager2.get_or_create("test_load_state").await;

        // Verify it loaded the open state
        assert!(
            breaker.is_open().await,
            "Circuit should be open after loading from Redis"
        );
    }

    cleanup_redis_keys(&redis_url, "circuit_breaker:test_load_state:*")
        .await
        .ok();
}

#[tokio::test]
async fn test_circuit_breaker_redis_ttl() {
    let redis_url = get_test_redis_url();
    cleanup_redis_keys(&redis_url, "circuit_breaker:test_ttl:*")
        .await
        .ok();

    let mut config = Config::default();
    config.redis.url = redis_url.clone();
    config.circuit_breaker.enabled = true;

    let manager = CircuitBreakerManager::with_redis(Arc::new(config))
        .await
        .expect("Failed to create manager with Redis");

    // Trigger a state change
    let _ = manager
        .call("test_ttl", || Ok::<_, std::io::Error>("success"))
        .await;

    sleep(Duration::from_millis(100)).await;

    // Verify TTL is set
    let client = redis::Client::open(redis_url.as_str()).unwrap();
    let mut conn = ConnectionManager::new(client).await.unwrap();

    let ttl: i64 = conn.ttl("circuit_breaker:test_ttl:state").await.unwrap();

    assert!(
        ttl > 0 && ttl <= 3600,
        "TTL should be set to 1 hour (3600 seconds)"
    );

    cleanup_redis_keys(&redis_url, "circuit_breaker:test_ttl:*")
        .await
        .ok();
}

#[tokio::test]
async fn test_circuit_breaker_fallback_without_redis() {
    // Test that circuit breaker works without Redis connection
    let mut config = Config::default();
    config.redis.url = "redis://invalid-host:9999".to_string();
    config.circuit_breaker.enabled = true;

    let manager = CircuitBreakerManager::with_redis(Arc::new(config))
        .await
        .expect("Manager should be created even if Redis is unavailable");

    // Should still work in-memory
    let result = manager
        .call("test_fallback", || Ok::<_, std::io::Error>("success"))
        .await;

    assert!(
        result.is_ok(),
        "Circuit breaker should work in-memory without Redis"
    );
}

#[tokio::test]
async fn test_circuit_breaker_state_transition_persistence() {
    let redis_url = get_test_redis_url();
    cleanup_redis_keys(&redis_url, "circuit_breaker:test_transition:*")
        .await
        .ok();

    let mut config = Config::default();
    config.redis.url = redis_url.clone();
    config.circuit_breaker.enabled = true;

    let service_config = CircuitBreakerServiceConfig {
        failure_threshold: 2,
        timeout_seconds: 1, // Short timeout for testing
        error_rate_threshold: 0.5,
    };
    config
        .circuit_breaker
        .services
        .insert("test_transition".to_string(), service_config.clone());

    let manager = CircuitBreakerManager::with_redis(Arc::new(config))
        .await
        .expect("Failed to create manager with Redis");

    // 1. Closed -> Open
    for _ in 0..3 {
        let _ = manager
            .call("test_transition", || {
                Err::<String, _>(std::io::Error::new(std::io::ErrorKind::Other, "failure"))
            })
            .await;
    }

    sleep(Duration::from_millis(100)).await;
    let state = manager.get_state("test_transition").await;
    assert_eq!(state, Some("open".to_string()));

    // 2. Open -> Half-Open (after timeout)
    sleep(Duration::from_secs(2)).await;

    // Access the breaker to trigger state check
    let breaker = manager.get_or_create("test_transition").await;
    breaker.check_and_update_state().await;

    sleep(Duration::from_millis(100)).await;
    let state = manager.get_state("test_transition").await;
    assert_eq!(state, Some("half_open".to_string()));

    // 3. Half-Open -> Closed (on success)
    let _ = manager
        .call("test_transition", || Ok::<_, std::io::Error>("success"))
        .await;

    sleep(Duration::from_millis(100)).await;
    let state = manager.get_state("test_transition").await;
    assert_eq!(state, Some("closed".to_string()));

    cleanup_redis_keys(&redis_url, "circuit_breaker:test_transition:*")
        .await
        .ok();
}

#[tokio::test]
async fn test_circuit_breaker_multiple_instances_share_state() {
    let redis_url = get_test_redis_url();
    cleanup_redis_keys(&redis_url, "circuit_breaker:test_shared:*")
        .await
        .ok();

    let mut config1 = Config::default();
    config1.redis.url = redis_url.clone();
    config1.circuit_breaker.enabled = true;

    let service_config = CircuitBreakerServiceConfig {
        failure_threshold: 2,
        timeout_seconds: 60,
        error_rate_threshold: 0.5,
    };
    config1
        .circuit_breaker
        .services
        .insert("test_shared".to_string(), service_config.clone());

    // Create first manager and open circuit
    let manager1 = CircuitBreakerManager::with_redis(Arc::new(config1))
        .await
        .expect("Failed to create manager1");

    for _ in 0..3 {
        let _ = manager1
            .call("test_shared", || {
                Err::<String, _>(std::io::Error::new(std::io::ErrorKind::Other, "failure"))
            })
            .await;
    }

    sleep(Duration::from_millis(200)).await;

    // Create second manager (simulating another gateway instance)
    let mut config2 = Config::default();
    config2.redis.url = redis_url.clone();
    config2.circuit_breaker.enabled = true;
    config2
        .circuit_breaker
        .services
        .insert("test_shared".to_string(), service_config.clone());

    let manager2 = CircuitBreakerManager::with_redis(Arc::new(config2))
        .await
        .expect("Failed to create manager2");

    // Access the circuit breaker in manager2
    let breaker2 = manager2.get_or_create("test_shared").await;

    // It should load the open state from Redis
    assert!(
        breaker2.is_open().await,
        "Second instance should see circuit as open"
    );

    cleanup_redis_keys(&redis_url, "circuit_breaker:test_shared:*")
        .await
        .ok();
}
