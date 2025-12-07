//! Circuit breaker implementation for external service resilience
//!
//! Implements the circuit breaker pattern to prevent cascading failures
//! when calling external services. The circuit breaker has three states:
//! - Closed: Normal operation, requests pass through
//! - Open: Service is failing, reject requests immediately
//! - Half-Open: Testing if service has recovered

use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation, requests pass through
    Closed,
    /// Service is failing, reject requests immediately
    Open,
    /// Testing if service has recovered
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,
    /// Number of successes to close from half-open state
    pub success_threshold: u32,
    /// Duration to wait before attempting recovery (half-open)
    pub timeout_duration: Duration,
    /// Maximum number of concurrent calls in half-open state
    pub half_open_max_calls: u32,
}

impl CircuitBreakerConfig {
    /// Configuration for platform APIs (moderate sensitivity)
    pub fn platform_api() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }

    /// Configuration for PubNub (high sensitivity)
    pub fn pubnub() -> Self {
        Self {
            failure_threshold: 3,
            success_threshold: 2,
            timeout_duration: Duration::from_secs(10),
            half_open_max_calls: 2,
        }
    }

    /// Configuration for embedding service (low sensitivity)
    pub fn embedding_service() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout_duration: Duration::from_secs(60),
            half_open_max_calls: 3,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self::platform_api()
    }
}

/// Circuit breaker error
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open, rejecting requests
    #[error("Circuit breaker is open for {circuit_name}")]
    CircuitOpen { circuit_name: String },

    /// Too many calls in half-open state
    #[error("Too many calls in half-open state for {circuit_name}")]
    TooManyCalls { circuit_name: String },

    /// The underlying operation failed
    #[error("Operation failed: {0}")]
    OperationFailed(E),

    /// Redis state synchronization failed
    #[error("Redis sync failed: {0}")]
    RedisSyncError(String),
}

/// Internal state tracker
#[derive(Debug)]
struct StateTracker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    half_open_calls: u32,
}

impl StateTracker {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            half_open_calls: 0,
        }
    }
}

/// Circuit breaker for protecting external service calls
pub struct CircuitBreaker {
    name: String,
    config: CircuitBreakerConfig,
    state_tracker: Arc<RwLock<StateTracker>>,
    redis_client: Option<redis::Client>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    ///
    /// # Arguments
    /// * `name` - Circuit breaker name (used for metrics and logging)
    /// * `config` - Circuit breaker configuration
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        let name = name.into();
        info!(
            circuit_name = %name,
            failure_threshold = config.failure_threshold,
            timeout_duration_secs = config.timeout_duration.as_secs(),
            "Creating circuit breaker"
        );

        Self {
            name,
            config,
            state_tracker: Arc::new(RwLock::new(StateTracker::new())),
            redis_client: None,
        }
    }

    /// Create a circuit breaker with Redis-backed distributed state
    ///
    /// # Arguments
    /// * `name` - Circuit breaker name
    /// * `config` - Circuit breaker configuration
    /// * `redis_client` - Redis client for state synchronization
    pub fn with_redis(
        name: impl Into<String>,
        config: CircuitBreakerConfig,
        redis_client: redis::Client,
    ) -> Self {
        let mut cb = Self::new(name, config);
        cb.redis_client = Some(redis_client);
        cb
    }

    /// Get the circuit breaker name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the current state
    pub async fn state(&self) -> CircuitState {
        let tracker = self.state_tracker.read().await;
        tracker.state
    }

    /// Get failure count
    pub async fn failure_count(&self) -> u32 {
        let tracker = self.state_tracker.read().await;
        tracker.failure_count
    }

    /// Get success count (in half-open state)
    pub async fn success_count(&self) -> u32 {
        let tracker = self.state_tracker.read().await;
        tracker.success_count
    }

    /// Execute a fallible operation protected by the circuit breaker
    ///
    /// # Arguments
    /// * `f` - Async operation to execute
    ///
    /// # Returns
    /// - Ok(T) if operation succeeded
    /// - Err(CircuitBreakerError::CircuitOpen) if circuit is open
    /// - Err(CircuitBreakerError::OperationFailed) if operation failed
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        // Check if we can make the call
        self.before_call().await?;

        // Execute the operation
        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(err))
            }
        }
    }

    /// Execute operation with fallback if circuit is open
    ///
    /// # Arguments
    /// * `f` - Async operation to execute
    /// * `fallback` - Fallback function to call if circuit is open
    ///
    /// # Returns
    /// Result from operation or fallback value
    pub async fn call_with_fallback<F, T, E, FB>(
        &self,
        f: F,
        fallback: FB,
    ) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
        FB: FnOnce() -> T,
    {
        match self.call(f).await {
            Ok(result) => Ok(result),
            Err(CircuitBreakerError::CircuitOpen { .. }) => {
                debug!(
                    circuit_name = %self.name,
                    "Circuit open, using fallback"
                );
                Ok(fallback())
            }
            Err(err) => Err(err),
        }
    }

    /// Check if call is allowed and update state
    async fn before_call<E>(&self) -> Result<(), CircuitBreakerError<E>> {
        let mut tracker = self.state_tracker.write().await;

        match tracker.state {
            CircuitState::Closed => {
                // Normal operation
                Ok(())
            }
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = tracker.last_failure_time {
                    if last_failure.elapsed() >= self.config.timeout_duration {
                        // Transition to half-open
                        info!(
                            circuit_name = %self.name,
                            "Circuit transitioning to half-open"
                        );
                        tracker.state = CircuitState::HalfOpen;
                        tracker.half_open_calls = 0;
                        tracker.success_count = 0;
                        self.sync_to_redis(&tracker).await;
                        Ok(())
                    } else {
                        // Still open
                        Err(CircuitBreakerError::CircuitOpen {
                            circuit_name: self.name.clone(),
                        })
                    }
                } else {
                    Err(CircuitBreakerError::CircuitOpen {
                        circuit_name: self.name.clone(),
                    })
                }
            }
            CircuitState::HalfOpen => {
                // Limit concurrent calls in half-open state
                if tracker.half_open_calls >= self.config.half_open_max_calls {
                    Err(CircuitBreakerError::TooManyCalls {
                        circuit_name: self.name.clone(),
                    })
                } else {
                    tracker.half_open_calls += 1;
                    Ok(())
                }
            }
        }
    }

    /// Handle successful operation
    async fn on_success(&self) {
        let mut tracker = self.state_tracker.write().await;

        match tracker.state {
            CircuitState::Closed => {
                // Reset failure count on success
                if tracker.failure_count > 0 {
                    debug!(
                        circuit_name = %self.name,
                        "Resetting failure count after success"
                    );
                    tracker.failure_count = 0;
                }
            }
            CircuitState::HalfOpen => {
                tracker.success_count += 1;
                tracker.half_open_calls = tracker.half_open_calls.saturating_sub(1);

                if tracker.success_count >= self.config.success_threshold {
                    // Close the circuit
                    info!(
                        circuit_name = %self.name,
                        success_count = tracker.success_count,
                        "Circuit closing after successful recovery"
                    );
                    tracker.state = CircuitState::Closed;
                    tracker.failure_count = 0;
                    tracker.success_count = 0;
                    tracker.last_failure_time = None;
                    self.sync_to_redis(&tracker).await;
                }
            }
            CircuitState::Open => {
                // Should not happen, but handle gracefully
                warn!(
                    circuit_name = %self.name,
                    "Unexpected success in open state"
                );
            }
        }
    }

    /// Handle failed operation
    async fn on_failure(&self) {
        let mut tracker = self.state_tracker.write().await;

        match tracker.state {
            CircuitState::Closed => {
                tracker.failure_count += 1;
                tracker.last_failure_time = Some(Instant::now());

                if tracker.failure_count >= self.config.failure_threshold {
                    // Open the circuit
                    warn!(
                        circuit_name = %self.name,
                        failure_count = tracker.failure_count,
                        "Circuit opening due to failures"
                    );
                    tracker.state = CircuitState::Open;
                    self.sync_to_redis(&tracker).await;
                }
            }
            CircuitState::HalfOpen => {
                // Single failure in half-open reopens the circuit
                warn!(
                    circuit_name = %self.name,
                    "Circuit reopening after failure in half-open state"
                );
                tracker.state = CircuitState::Open;
                tracker.failure_count = self.config.failure_threshold;
                tracker.success_count = 0;
                tracker.half_open_calls = 0;
                tracker.last_failure_time = Some(Instant::now());
                self.sync_to_redis(&tracker).await;
            }
            CircuitState::Open => {
                // Already open, just update timestamp
                tracker.last_failure_time = Some(Instant::now());
            }
        }
    }

    /// Synchronize state to Redis (if configured)
    async fn sync_to_redis(&self, tracker: &StateTracker) {
        if let Some(redis_client) = &self.redis_client {
            let state_key = format!("circuit:{}:state", self.name);
            let failures_key = format!("circuit:{}:failures", self.name);
            let last_failure_key = format!("circuit:{}:last_failure", self.name);

            if let Ok(mut conn) = redis_client.get_async_connection().await {
                use redis::AsyncCommands;

                let state_str = match tracker.state {
                    CircuitState::Closed => "closed",
                    CircuitState::Open => "open",
                    CircuitState::HalfOpen => "half_open",
                };

                let _: Result<(), redis::RedisError> = conn.set(&state_key, state_str).await;
                let _: Result<(), redis::RedisError> =
                    conn.set(&failures_key, tracker.failure_count).await;

                if let Some(last_failure) = tracker.last_failure_time {
                    let timestamp = last_failure.elapsed().as_secs();
                    let _: Result<(), redis::RedisError> =
                        conn.set(&last_failure_key, timestamp).await;
                }
            }
        }
    }

    /// Reset the circuit breaker to closed state (for testing/admin)
    pub async fn reset(&self) {
        let mut tracker = self.state_tracker.write().await;
        info!(circuit_name = %self.name, "Resetting circuit breaker");
        tracker.state = CircuitState::Closed;
        tracker.failure_count = 0;
        tracker.success_count = 0;
        tracker.last_failure_time = None;
        tracker.half_open_calls = 0;
        self.sync_to_redis(&tracker).await;
    }

    /// Force the circuit to open (for testing)
    #[cfg(test)]
    pub async fn force_open(&self) {
        let mut tracker = self.state_tracker.write().await;
        tracker.state = CircuitState::Open;
        tracker.failure_count = self.config.failure_threshold;
        tracker.last_failure_time = Some(Instant::now());
    }

    /// Get metrics snapshot
    pub async fn metrics(&self) -> CircuitBreakerMetrics {
        let tracker = self.state_tracker.read().await;
        CircuitBreakerMetrics {
            name: self.name.clone(),
            state: tracker.state,
            failure_count: tracker.failure_count,
            success_count: tracker.success_count,
            half_open_calls: tracker.half_open_calls,
        }
    }
}

/// Circuit breaker metrics snapshot
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub name: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub half_open_calls: u32,
}
