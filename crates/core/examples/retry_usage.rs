//! Example usage of the retry utility with exponential backoff
//!
//! This example demonstrates how to use the retry functionality
//! with various retry policies for different scenarios.

use media_gateway_core::error::MediaGatewayError;
use media_gateway_core::retry::{retry_with_backoff, RetryPolicy};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Simulates a flaky network operation that fails intermittently
async fn flaky_network_request(
    attempt_counter: Arc<AtomicU32>,
) -> Result<String, MediaGatewayError> {
    let attempts = attempt_counter.fetch_add(1, Ordering::SeqCst);

    if attempts < 2 {
        // Fail with a retryable error
        Err(MediaGatewayError::NetworkError {
            message: format!("Connection timeout on attempt {}", attempts + 1),
            source: None,
        })
    } else {
        // Success after 2 retries
        Ok(format!("Success after {} attempts", attempts + 1))
    }
}

/// Simulates a database operation with transient failures
async fn database_operation(attempt_counter: Arc<AtomicU32>) -> Result<String, MediaGatewayError> {
    let attempts = attempt_counter.fetch_add(1, Ordering::SeqCst);

    if attempts == 0 {
        // First attempt: connection pool exhausted
        Err(MediaGatewayError::ServiceUnavailableError {
            service: "Database".to_string(),
            retry_after: Some(1),
        })
    } else if attempts == 1 {
        // Second attempt: timeout
        Err(MediaGatewayError::TimeoutError {
            operation: "database query".to_string(),
            duration_ms: 5000,
        })
    } else {
        // Third attempt succeeds
        Ok("Database operation completed".to_string())
    }
}

/// Simulates an operation that fails with a non-retryable error
async fn validation_operation() -> Result<String, MediaGatewayError> {
    Err(MediaGatewayError::ValidationError {
        message: "Invalid user input".to_string(),
        field: Some("email".to_string()),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Retry Utility Examples ===\n");

    // Example 1: Network request with default retry policy
    println!("1. Network request with default retry policy:");
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        || flaky_network_request(counter_clone.clone()),
        RetryPolicy::default(),
        |err: &MediaGatewayError| err.is_retryable(),
    )
    .await;

    match result {
        Ok(msg) => println!("   ✓ {}", msg),
        Err(e) => println!("   ✗ Failed: {}", e),
    }
    println!("   Total attempts: {}\n", counter.load(Ordering::SeqCst));

    // Example 2: Database operation with aggressive retry policy
    println!("2. Database operation with aggressive retry policy:");
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(
        || database_operation(counter_clone.clone()),
        RetryPolicy::aggressive(),
        |err: &MediaGatewayError| err.is_retryable(),
    )
    .await;

    match result {
        Ok(msg) => println!("   ✓ {}", msg),
        Err(e) => println!("   ✗ Failed: {}", e),
    }
    println!("   Total attempts: {}\n", counter.load(Ordering::SeqCst));

    // Example 3: Validation error (non-retryable)
    println!("3. Validation error (should not retry):");
    let counter = Arc::new(AtomicU32::new(0));

    let result = retry_with_backoff(
        || {
            counter.fetch_add(1, Ordering::SeqCst);
            validation_operation()
        },
        RetryPolicy::default(),
        |err: &MediaGatewayError| err.is_retryable(),
    )
    .await;

    match result {
        Ok(msg) => println!("   ✓ {}", msg),
        Err(e) => println!("   ✗ Failed immediately: {}", e),
    }
    println!(
        "   Total attempts: {} (should be 1)\n",
        counter.load(Ordering::SeqCst)
    );

    // Example 4: Custom retry policy
    println!("4. Custom retry policy (10 retries, 50ms base delay):");
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    let custom_policy = RetryPolicy::new(
        10,   // max_retries
        50,   // base_delay_ms
        2000, // max_delay_ms
        true, // jitter
    );

    let result = retry_with_backoff(
        || flaky_network_request(counter_clone.clone()),
        custom_policy,
        |err: &MediaGatewayError| err.is_retryable(),
    )
    .await;

    match result {
        Ok(msg) => println!("   ✓ {}", msg),
        Err(e) => println!("   ✗ Failed: {}", e),
    }
    println!("   Total attempts: {}\n", counter.load(Ordering::SeqCst));

    // Example 5: Gentle retry policy
    println!("5. Non-critical operation with gentle retry policy:");
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    // This will exhaust retries since we only get 2 retries with gentle policy
    let always_fail = || async {
        counter_clone.fetch_add(1, Ordering::SeqCst);
        Err::<String, _>(MediaGatewayError::NetworkError {
            message: "Persistent failure".to_string(),
            source: None,
        })
    };

    let result = retry_with_backoff(
        always_fail,
        RetryPolicy::gentle(),
        |err: &MediaGatewayError| err.is_retryable(),
    )
    .await;

    match result {
        Ok(msg) => println!("   ✓ {}", msg),
        Err(e) => println!("   ✗ All retries exhausted: {}", e),
    }
    println!(
        "   Total attempts: {} (initial + 2 retries)\n",
        counter.load(Ordering::SeqCst)
    );

    println!("=== Examples Complete ===");
    Ok(())
}
