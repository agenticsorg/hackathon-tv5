//! Exponential backoff retry utility
//!
//! Provides configurable retry mechanisms with exponential backoff and jitter
//! for handling transient failures in distributed systems.
//!
//! # Examples
//!
//! ```
//! use media_gateway_core::retry::{RetryPolicy, retry_with_backoff};
//!
//! async fn fallible_operation() -> Result<String, std::io::Error> {
//!     // Your operation that might fail
//!     Ok("success".to_string())
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use default policy
//! let result = retry_with_backoff(
//!     || async { fallible_operation().await },
//!     RetryPolicy::default(),
//!     |err: &std::io::Error| err.kind() == std::io::ErrorKind::ConnectionRefused,
//! ).await?;
//!
//! // Use aggressive policy for critical operations
//! let result = retry_with_backoff(
//!     || async { fallible_operation().await },
//!     RetryPolicy::aggressive(),
//!     |_| true,
//! ).await?;
//! # Ok(())
//! # }
//! ```

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Retry policy configuration for exponential backoff
///
/// Defines the behavior of retry attempts including maximum retries,
/// delay parameters, and whether to apply jitter.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 means no retries, only initial attempt)
    pub max_retries: u32,

    /// Base delay in milliseconds for the first retry
    pub base_delay_ms: u64,

    /// Maximum delay in milliseconds to cap exponential growth
    pub max_delay_ms: u64,

    /// Whether to add random jitter to delays (recommended for distributed systems)
    pub jitter: bool,
}

impl Default for RetryPolicy {
    /// Creates a default retry policy with sensible defaults
    ///
    /// - max_retries: 3
    /// - base_delay_ms: 100
    /// - max_delay_ms: 5000
    /// - jitter: true
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Creates a new retry policy with custom parameters
    ///
    /// # Arguments
    ///
    /// * `max_retries` - Maximum number of retry attempts
    /// * `base_delay_ms` - Base delay in milliseconds
    /// * `max_delay_ms` - Maximum delay in milliseconds
    /// * `jitter` - Whether to apply random jitter
    ///
    /// # Examples
    ///
    /// ```
    /// use media_gateway_core::retry::RetryPolicy;
    ///
    /// let policy = RetryPolicy::new(5, 200, 10000, true);
    /// assert_eq!(policy.max_retries, 5);
    /// assert_eq!(policy.base_delay_ms, 200);
    /// ```
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64, jitter: bool) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms,
            jitter,
        }
    }

    /// Creates an aggressive retry policy for critical operations
    ///
    /// - max_retries: 5
    /// - base_delay_ms: 50
    /// - max_delay_ms: 5000
    /// - jitter: true
    ///
    /// Use this for operations that must succeed and can tolerate more retry attempts.
    ///
    /// # Examples
    ///
    /// ```
    /// use media_gateway_core::retry::RetryPolicy;
    ///
    /// let policy = RetryPolicy::aggressive();
    /// assert_eq!(policy.max_retries, 5);
    /// assert_eq!(policy.base_delay_ms, 50);
    /// ```
    pub fn aggressive() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 50,
            max_delay_ms: 5000,
            jitter: true,
        }
    }

    /// Creates a gentle retry policy for non-critical operations
    ///
    /// - max_retries: 2
    /// - base_delay_ms: 500
    /// - max_delay_ms: 3000
    /// - jitter: true
    ///
    /// Use this for operations that are not time-sensitive and should back off more gracefully.
    ///
    /// # Examples
    ///
    /// ```
    /// use media_gateway_core::retry::RetryPolicy;
    ///
    /// let policy = RetryPolicy::gentle();
    /// assert_eq!(policy.max_retries, 2);
    /// assert_eq!(policy.base_delay_ms, 500);
    /// ```
    pub fn gentle() -> Self {
        Self {
            max_retries: 2,
            base_delay_ms: 500,
            max_delay_ms: 3000,
            jitter: true,
        }
    }

    /// Calculates the delay for a given retry attempt
    ///
    /// Uses exponential backoff: delay = min(base * 2^attempt, max_delay)
    /// Optionally adds random jitter: delay + random(0, delay * 0.3)
    ///
    /// # Arguments
    ///
    /// * `attempt` - The retry attempt number (0-indexed)
    ///
    /// # Returns
    ///
    /// Duration to wait before the next retry attempt
    fn calculate_delay(&self, attempt: u32) -> Duration {
        // Calculate exponential backoff: base * 2^attempt
        let exponential_delay = self
            .base_delay_ms
            .saturating_mul(2_u64.saturating_pow(attempt));

        // Cap at maximum delay
        let capped_delay = exponential_delay.min(self.max_delay_ms);

        // Apply jitter if enabled
        let final_delay = if self.jitter {
            let jitter_range = (capped_delay as f64 * 0.3) as u64;
            let jitter = if jitter_range > 0 {
                // Use a simple random number generation based on current time
                // For production-grade randomness, this is replaced with rand crate
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos() as u64;
                nanos % (jitter_range + 1)
            } else {
                0
            };
            capped_delay.saturating_add(jitter)
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay)
    }
}

/// Retries an async operation with exponential backoff
///
/// Executes the provided async closure and retries on failure according to the retry policy.
/// Only retries when the predicate function returns true for the error.
///
/// # Type Parameters
///
/// * `F` - Factory function that creates the future to retry
/// * `Fut` - Future type returned by the factory
/// * `T` - Success type
/// * `E` - Error type
///
/// # Arguments
///
/// * `operation` - Async closure that produces a future to execute
/// * `policy` - Retry policy configuration
/// * `is_retryable` - Predicate to determine if an error should trigger a retry
///
/// # Returns
///
/// Result of the operation after retries are exhausted or success is achieved
///
/// # Examples
///
/// ```
/// use media_gateway_core::retry::{RetryPolicy, retry_with_backoff};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut attempt = 0;
/// let result = retry_with_backoff(
///     || async {
///         attempt += 1;
///         if attempt < 3 {
///             Err("temporary failure")
///         } else {
///             Ok("success")
///         }
///     },
///     RetryPolicy::default(),
///     |_: &str| true,
/// ).await;
///
/// assert!(result.is_ok());
/// # Ok(())
/// # }
/// ```
pub async fn retry_with_backoff<F, Fut, T, E, P>(
    mut operation: F,
    policy: RetryPolicy,
    is_retryable: P,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: Fn(&E) -> bool,
{
    let mut attempt = 0;

    loop {
        // Execute the operation
        match operation().await {
            Ok(result) => {
                // Success - return immediately
                tracing::debug!(
                    attempt = attempt,
                    total_attempts = attempt + 1,
                    "Operation succeeded"
                );
                return Ok(result);
            }
            Err(error) => {
                // Check if we should retry
                if attempt >= policy.max_retries {
                    // Exhausted all retries
                    tracing::warn!(
                        attempt = attempt,
                        max_retries = policy.max_retries,
                        "All retry attempts exhausted"
                    );
                    return Err(error);
                }

                // Check if error is retryable
                if !is_retryable(&error) {
                    // Non-retryable error - fail immediately
                    tracing::debug!(
                        attempt = attempt,
                        "Error is not retryable, failing immediately"
                    );
                    return Err(error);
                }

                // Calculate delay and wait
                let delay = policy.calculate_delay(attempt);
                tracing::debug!(
                    attempt = attempt,
                    delay_ms = delay.as_millis(),
                    max_retries = policy.max_retries,
                    "Retrying after delay"
                );

                sleep(delay).await;
                attempt += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);
        assert_eq!(policy.base_delay_ms, 100);
        assert_eq!(policy.max_delay_ms, 5000);
        assert!(policy.jitter);
    }

    #[test]
    fn test_retry_policy_aggressive() {
        let policy = RetryPolicy::aggressive();
        assert_eq!(policy.max_retries, 5);
        assert_eq!(policy.base_delay_ms, 50);
        assert_eq!(policy.max_delay_ms, 5000);
        assert!(policy.jitter);
    }

    #[test]
    fn test_retry_policy_gentle() {
        let policy = RetryPolicy::gentle();
        assert_eq!(policy.max_retries, 2);
        assert_eq!(policy.base_delay_ms, 500);
        assert_eq!(policy.max_delay_ms, 3000);
        assert!(policy.jitter);
    }

    #[test]
    fn test_retry_policy_new() {
        let policy = RetryPolicy::new(10, 200, 8000, false);
        assert_eq!(policy.max_retries, 10);
        assert_eq!(policy.base_delay_ms, 200);
        assert_eq!(policy.max_delay_ms, 8000);
        assert!(!policy.jitter);
    }

    #[test]
    fn test_calculate_delay_exponential_progression() {
        let policy = RetryPolicy::new(5, 100, 10000, false);

        // Verify exponential progression: 100, 200, 400, 800, 1600
        let delay0 = policy.calculate_delay(0);
        assert_eq!(delay0.as_millis(), 100);

        let delay1 = policy.calculate_delay(1);
        assert_eq!(delay1.as_millis(), 200);

        let delay2 = policy.calculate_delay(2);
        assert_eq!(delay2.as_millis(), 400);

        let delay3 = policy.calculate_delay(3);
        assert_eq!(delay3.as_millis(), 800);

        let delay4 = policy.calculate_delay(4);
        assert_eq!(delay4.as_millis(), 1600);
    }

    #[test]
    fn test_calculate_delay_max_cap() {
        let policy = RetryPolicy::new(10, 100, 500, false);

        // After several attempts, delay should be capped at max_delay_ms
        let delay5 = policy.calculate_delay(5); // 100 * 2^5 = 3200, capped at 500
        assert_eq!(delay5.as_millis(), 500);

        let delay10 = policy.calculate_delay(10); // Should still be capped
        assert_eq!(delay10.as_millis(), 500);
    }

    #[test]
    fn test_calculate_delay_with_jitter() {
        let policy = RetryPolicy::new(3, 1000, 5000, true);

        // With jitter enabled, delay should be base + random amount
        // We can't test exact values due to randomness, but we can test bounds
        let delay = policy.calculate_delay(0);
        let delay_ms = delay.as_millis();

        // Should be at least base_delay_ms
        assert!(delay_ms >= 1000);

        // Should be at most base_delay_ms + 30% jitter
        assert!(delay_ms <= 1300);
    }

    #[tokio::test]
    async fn test_retry_succeeds_immediately() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, String>("success")
                }
            },
            RetryPolicy::default(),
            |_: &String| true,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only called once
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    let count = c.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("temporary failure")
                    } else {
                        Ok("success")
                    }
                }
            },
            RetryPolicy::new(5, 10, 100, false), // Fast retries for testing
            |_: &&str| true,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_exhausts_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>("persistent failure")
                }
            },
            RetryPolicy::new(3, 10, 100, false),
            |_: &&str| true,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "persistent failure");
        assert_eq!(counter.load(Ordering::SeqCst), 4); // Initial + 3 retries
    }

    #[tokio::test]
    async fn test_non_retryable_error_fails_immediately() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>("non-retryable")
                }
            },
            RetryPolicy::default(),
            |err: &&str| *err != "non-retryable", // This error is NOT retryable
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "non-retryable");
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only called once
    }

    #[tokio::test]
    async fn test_retry_with_media_gateway_error() {
        use crate::error::MediaGatewayError;

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    let count = c.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(MediaGatewayError::NetworkError {
                            message: "Connection timeout".to_string(),
                            source: None,
                        })
                    } else {
                        Ok("success")
                    }
                }
            },
            RetryPolicy::new(5, 10, 100, false),
            |err: &MediaGatewayError| err.is_retryable(),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_non_retryable_media_gateway_error() {
        use crate::error::MediaGatewayError;

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>(MediaGatewayError::ValidationError {
                        message: "Invalid input".to_string(),
                        field: None,
                    })
                }
            },
            RetryPolicy::default(),
            |err: &MediaGatewayError| err.is_retryable(),
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Fails immediately
    }

    #[tokio::test]
    async fn test_aggressive_policy_more_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>("always fails")
                }
            },
            RetryPolicy::aggressive(),
            |_: &&str| true,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 6); // Initial + 5 retries
    }

    #[tokio::test]
    async fn test_gentle_policy_fewer_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>("always fails")
                }
            },
            RetryPolicy::gentle(),
            |_: &&str| true,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_zero_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_with_backoff(
            || {
                let c = counter_clone.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err::<(), _>("failure")
                }
            },
            RetryPolicy::new(0, 100, 1000, false),
            |_: &&str| true,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only initial attempt
    }
}
