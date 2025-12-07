/// Example: Circuit Breaker with Redis Persistence
///
/// This example demonstrates how to use the circuit breaker with Redis persistence
/// to share state across multiple gateway instances.
///
/// Prerequisites:
/// - Redis running on localhost:6379 (or set REDIS_URL environment variable)
///
/// Run with:
/// ```bash
/// REDIS_URL=redis://localhost:6379 cargo run --package media-gateway-api --example circuit_breaker_with_redis
/// ```
use media_gateway_api::circuit_breaker::CircuitBreakerManager;
use media_gateway_api::config::{CircuitBreakerServiceConfig, Config};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Starting Circuit Breaker Redis Persistence Example");

    // Create configuration with Redis
    let mut config = Config::default();
    config.redis.url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    config.circuit_breaker.enabled = true;

    // Configure circuit breaker for test service
    config.circuit_breaker.services.insert(
        "test_service".to_string(),
        CircuitBreakerServiceConfig {
            failure_threshold: 3,
            timeout_seconds: 5,
            error_rate_threshold: 0.5,
        },
    );

    let config = Arc::new(config);

    // Create circuit breaker manager with Redis persistence
    info!("Initializing circuit breaker manager with Redis persistence...");
    let manager = CircuitBreakerManager::with_redis(config.clone()).await?;

    info!("Circuit breaker manager initialized successfully");

    // Simulate successful requests
    info!("\n--- Phase 1: Successful Requests ---");
    for i in 1..=3 {
        let result = manager
            .call("test_service", || {
                info!("Executing successful request {}", i);
                Ok::<_, std::io::Error>(format!("Success {}", i))
            })
            .await;

        match result {
            Ok(value) => info!("Request {} succeeded: {}", i, value),
            Err(e) => error!("Request {} failed: {}", i, e),
        }

        sleep(Duration::from_millis(500)).await;
    }

    // Check circuit state
    if let Some(state) = manager.get_state("test_service").await {
        info!("Circuit state after successful requests: {}", state);
    }

    // Simulate failures to open the circuit
    info!("\n--- Phase 2: Triggering Circuit Breaker (Failures) ---");
    for i in 1..=5 {
        let result = manager
            .call("test_service", || {
                info!("Simulating failure {}", i);
                Err::<String, _>(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Simulated failure {}", i),
                ))
            })
            .await;

        match result {
            Ok(_) => info!("Request {} succeeded (unexpected)", i),
            Err(e) => error!("Request {} failed: {}", i, e),
        }

        if let Some(state) = manager.get_state("test_service").await {
            info!("Circuit state: {}", state);
        }

        sleep(Duration::from_millis(500)).await;
    }

    // Try to make request with open circuit
    info!("\n--- Phase 3: Requests with Open Circuit ---");
    for i in 1..=2 {
        let result = manager
            .call("test_service", || {
                info!("This should not execute due to open circuit");
                Ok::<_, std::io::Error>("Should not reach here")
            })
            .await;

        match result {
            Ok(value) => info!("Request {} succeeded: {}", i, value),
            Err(e) => error!("Request {} rejected by circuit breaker: {}", i, e),
        }

        sleep(Duration::from_millis(500)).await;
    }

    // Wait for circuit to enter half-open state
    info!("\n--- Phase 4: Waiting for Circuit Timeout (5 seconds) ---");
    info!("Circuit will transition to half-open state...");
    sleep(Duration::from_secs(6)).await;

    if let Some(state) = manager.get_state("test_service").await {
        info!("Circuit state after timeout: {}", state);
    }

    // Make a successful request to close the circuit
    info!("\n--- Phase 5: Recovering Circuit (Half-Open -> Closed) ---");
    let result = manager
        .call("test_service", || {
            info!("Executing recovery request");
            Ok::<_, std::io::Error>("Recovery success")
        })
        .await;

    match result {
        Ok(value) => info!("Recovery request succeeded: {}", value),
        Err(e) => error!("Recovery request failed: {}", e),
    }

    if let Some(state) = manager.get_state("test_service").await {
        info!("Circuit state after recovery: {}", state);
    }

    // Demonstrate multi-instance state sharing
    info!("\n--- Phase 6: Multi-Instance State Sharing ---");
    info!("Creating second circuit breaker manager instance...");

    let manager2 = CircuitBreakerManager::with_redis(config.clone()).await?;

    if let Some(state) = manager2.get_state("test_service").await {
        info!("Second instance sees circuit state: {}", state);
    }

    // Trigger failure in first instance
    info!("Triggering failures in first instance...");
    for i in 1..=4 {
        let _ = manager
            .call("test_service", || {
                Err::<String, _>(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Multi-instance test failure",
                ))
            })
            .await;
    }

    sleep(Duration::from_secs(1)).await;

    // Check state in second instance
    info!("Checking state in second instance...");
    let breaker2 = manager2.get_or_create("test_service").await;

    let result = manager2
        .call("test_service", || {
            Ok::<_, std::io::Error>("Second instance request")
        })
        .await;

    match result {
        Ok(value) => info!("Second instance request succeeded: {}", value),
        Err(e) => info!("Second instance request correctly rejected: {}", e),
    }

    // Display all circuit states
    info!("\n--- Final State Summary ---");
    let all_states = manager.get_all_states().await;
    for (service, state) in all_states {
        info!("Service '{}': {}", service, state);
    }

    info!("\n--- Example Complete ---");
    info!("Check Redis to see persisted state:");
    info!("  redis-cli GET circuit_breaker:test_service:state");
    info!("  redis-cli TTL circuit_breaker:test_service:state");

    Ok(())
}
