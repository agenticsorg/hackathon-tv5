//! Tests for circuit breaker implementation

use super::circuit_breaker::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_circuit_breaker_closed_state_allows_calls() {
    let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());

    let result = cb.call(async { Ok::<_, String>("success") }).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    assert_eq!(cb.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_threshold_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(1),
        half_open_max_calls: 2,
    };
    let cb = CircuitBreaker::new("test", config);

    // First 2 failures - should stay closed
    for _ in 0..2 {
        let _ = cb.call(async { Err::<String, _>("error") }).await;
    }
    assert_eq!(cb.state().await, CircuitState::Closed);
    assert_eq!(cb.failure_count().await, 2);

    // 3rd failure - should open
    let _ = cb.call(async { Err::<String, _>("error") }).await;
    assert_eq!(cb.state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_circuit_breaker_rejects_when_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(10),
        half_open_max_calls: 2,
    };
    let cb = CircuitBreaker::new("test", config);

    // Open the circuit
    cb.force_open().await;
    assert_eq!(cb.state().await, CircuitState::Open);

    // Attempt a call - should be rejected
    let result = cb
        .call(async { Ok::<_, String>("should not execute") })
        .await;

    assert!(matches!(
        result,
        Err(CircuitBreakerError::CircuitOpen { .. })
    ));
}

#[tokio::test]
async fn test_circuit_breaker_transitions_to_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_duration: Duration::from_millis(100),
        half_open_max_calls: 2,
    };
    let cb = CircuitBreaker::new("test", config);

    // Open the circuit
    cb.force_open().await;
    assert_eq!(cb.state().await, CircuitState::Open);

    // Wait for timeout
    sleep(Duration::from_millis(150)).await;

    // Next call should transition to half-open
    let result = cb.call(async { Ok::<_, String>("test") }).await;

    // The call should succeed and state should be half-open (or closed if threshold met)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_circuit_breaker_closes_after_successful_recovery() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_duration: Duration::from_millis(100),
        half_open_max_calls: 3,
    };
    let cb = CircuitBreaker::new("test", config);

    // Open the circuit
    cb.force_open().await;

    // Wait for timeout to allow half-open
    sleep(Duration::from_millis(150)).await;

    // Make successful calls equal to success_threshold
    for _ in 0..2 {
        let result = cb.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
    }

    // Circuit should be closed now
    assert_eq!(cb.state().await, CircuitState::Closed);
    assert_eq!(cb.failure_count().await, 0);
}

#[tokio::test]
async fn test_circuit_breaker_reopens_on_half_open_failure() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_duration: Duration::from_millis(100),
        half_open_max_calls: 3,
    };
    let cb = CircuitBreaker::new("test", config);

    // Open the circuit
    cb.force_open().await;

    // Wait for timeout
    sleep(Duration::from_millis(150)).await;

    // Make one successful call to transition to half-open
    let _ = cb.call(async { Ok::<_, String>("success") }).await;

    // Fail the next call - should reopen
    let _ = cb.call(async { Err::<String, _>("error") }).await;

    assert_eq!(cb.state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_circuit_breaker_limits_half_open_calls() {
    use std::sync::Arc;

    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 3,
        timeout_duration: Duration::from_millis(100),
        half_open_max_calls: 2,
    };
    let cb = Arc::new(CircuitBreaker::new("test", config));

    // Open the circuit
    cb.force_open().await;

    // Wait for timeout
    sleep(Duration::from_millis(150)).await;

    // Start two concurrent calls (max allowed)
    let cb_clone1 = Arc::clone(&cb);
    let cb_clone2 = Arc::clone(&cb);

    // These should be allowed
    let handle1 = tokio::spawn(async move {
        cb_clone1
            .call(async {
                sleep(Duration::from_millis(50)).await;
                Ok::<_, String>("success")
            })
            .await
    });

    let handle2 = tokio::spawn(async move {
        cb_clone2
            .call(async {
                sleep(Duration::from_millis(50)).await;
                Ok::<_, String>("success")
            })
            .await
    });

    // Small delay to ensure calls are registered
    sleep(Duration::from_millis(10)).await;

    // This third call should be rejected
    let result = cb
        .call(async { Ok::<_, String>("should not execute") })
        .await;

    assert!(matches!(
        result,
        Err(CircuitBreakerError::TooManyCalls { .. })
    ));

    // Wait for concurrent calls to complete
    let _ = handle1.await;
    let _ = handle2.await;
}

#[tokio::test]
async fn test_circuit_breaker_with_fallback_uses_fallback_when_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(10),
        half_open_max_calls: 2,
    };
    let cb = CircuitBreaker::new("test", config);

    // Open the circuit
    cb.force_open().await;

    // Call with fallback
    let result = cb
        .call_with_fallback(async { Ok::<_, String>("primary") }, || "fallback")
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "fallback");
}

#[tokio::test]
async fn test_circuit_breaker_with_fallback_uses_primary_when_closed() {
    let cb = CircuitBreaker::new("test", CircuitBreakerConfig::default());

    // Call with fallback
    let result = cb
        .call_with_fallback(async { Ok::<_, String>("primary") }, || "fallback")
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "primary");
}

#[tokio::test]
async fn test_circuit_breaker_resets_failure_count_on_success_when_closed() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(1),
        half_open_max_calls: 2,
    };
    let cb = CircuitBreaker::new("test", config);

    // Two failures
    for _ in 0..2 {
        let _ = cb.call(async { Err::<String, _>("error") }).await;
    }
    assert_eq!(cb.failure_count().await, 2);

    // One success - should reset failure count
    let _ = cb.call(async { Ok::<_, String>("success") }).await;
    assert_eq!(cb.failure_count().await, 0);
    assert_eq!(cb.state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_reset() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(10),
        half_open_max_calls: 2,
    };
    let cb = CircuitBreaker::new("test", config);

    // Open the circuit
    cb.force_open().await;
    assert_eq!(cb.state().await, CircuitState::Open);

    // Reset
    cb.reset().await;

    assert_eq!(cb.state().await, CircuitState::Closed);
    assert_eq!(cb.failure_count().await, 0);
    assert_eq!(cb.success_count().await, 0);
}

#[tokio::test]
async fn test_circuit_breaker_metrics() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(1),
        half_open_max_calls: 2,
    };
    let cb = CircuitBreaker::new("test-metrics", config);

    // Make some failures
    for _ in 0..2 {
        let _ = cb.call(async { Err::<String, _>("error") }).await;
    }

    let metrics = cb.metrics().await;
    assert_eq!(metrics.name, "test-metrics");
    assert_eq!(metrics.state, CircuitState::Closed);
    assert_eq!(metrics.failure_count, 2);
}

#[tokio::test]
async fn test_circuit_breaker_config_presets() {
    let platform = CircuitBreakerConfig::platform_api();
    assert_eq!(platform.failure_threshold, 5);
    assert_eq!(platform.timeout_duration, Duration::from_secs(30));
    assert_eq!(platform.half_open_max_calls, 3);

    let pubnub = CircuitBreakerConfig::pubnub();
    assert_eq!(pubnub.failure_threshold, 3);
    assert_eq!(pubnub.timeout_duration, Duration::from_secs(10));
    assert_eq!(pubnub.half_open_max_calls, 2);

    let embedding = CircuitBreakerConfig::embedding_service();
    assert_eq!(embedding.failure_threshold, 5);
    assert_eq!(embedding.timeout_duration, Duration::from_secs(60));
    assert_eq!(embedding.half_open_max_calls, 3);
}

#[tokio::test]
async fn test_circuit_state_display() {
    assert_eq!(CircuitState::Closed.to_string(), "Closed");
    assert_eq!(CircuitState::Open.to_string(), "Open");
    assert_eq!(CircuitState::HalfOpen.to_string(), "HalfOpen");
}
