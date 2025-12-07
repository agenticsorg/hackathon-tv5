use crate::config::{CircuitBreakerServiceConfig, Config};
use crate::error::{ApiError, ApiResult};
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

#[derive(Clone, Debug)]
enum CircuitState {
    Closed,
    Open { opened_at: Instant },
    HalfOpen,
}

/// Serializable state for Redis persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedCircuitState {
    state: String, // "closed", "open", or "half_open"
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<u64>, // Unix timestamp in seconds
    opened_at: Option<u64>,         // Unix timestamp in seconds when circuit opened
}

#[derive(Clone)]
struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<RwLock<u32>>,
    success_count: Arc<RwLock<u32>>,
    last_failure_time: Arc<RwLock<Option<SystemTime>>>,
    config: CircuitBreakerServiceConfig,
    service_name: String,
    redis_manager: Option<Arc<RwLock<ConnectionManager>>>,
}

impl CircuitBreaker {
    fn new(
        config: CircuitBreakerServiceConfig,
        service_name: String,
        redis_manager: Option<Arc<RwLock<ConnectionManager>>>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
            service_name,
            redis_manager,
        }
    }

    /// Get the Redis key for this circuit breaker's state
    fn redis_key(&self) -> String {
        format!("circuit_breaker:{}:state", self.service_name)
    }

    /// Convert current state to persistable format
    async fn to_persisted_state(&self) -> PersistedCircuitState {
        let state = self.state.read().await;
        let failure_count = *self.failure_count.read().await;
        let success_count = *self.success_count.read().await;
        let last_failure_time = self.last_failure_time.read().await;

        let (state_str, opened_at) = match &*state {
            CircuitState::Closed => ("closed".to_string(), None),
            CircuitState::Open { opened_at } => {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .saturating_sub(opened_at.elapsed().as_secs());
                ("open".to_string(), Some(timestamp))
            }
            CircuitState::HalfOpen => ("half_open".to_string(), None),
        };

        let last_failure_timestamp = last_failure_time
            .as_ref()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());

        PersistedCircuitState {
            state: state_str,
            failure_count,
            success_count,
            last_failure_time: last_failure_timestamp,
            opened_at,
        }
    }

    /// Restore state from persisted format
    async fn from_persisted_state(&self, persisted: PersistedCircuitState) {
        let mut state = self.state.write().await;
        let mut failure_count = self.failure_count.write().await;
        let mut success_count = self.success_count.write().await;
        let mut last_failure_time = self.last_failure_time.write().await;

        *failure_count = persisted.failure_count;
        *success_count = persisted.success_count;

        if let Some(timestamp) = persisted.last_failure_time {
            *last_failure_time = Some(UNIX_EPOCH + Duration::from_secs(timestamp));
        }

        *state = match persisted.state.as_str() {
            "open" => {
                if let Some(opened_timestamp) = persisted.opened_at {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    let elapsed = now.saturating_sub(opened_timestamp);
                    let opened_at = Instant::now()
                        .checked_sub(Duration::from_secs(elapsed))
                        .unwrap_or_else(Instant::now);
                    CircuitState::Open { opened_at }
                } else {
                    CircuitState::Open {
                        opened_at: Instant::now(),
                    }
                }
            }
            "half_open" => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        };

        debug!(
            service = %self.service_name,
            state = %persisted.state,
            failure_count = persisted.failure_count,
            success_count = persisted.success_count,
            "Restored circuit breaker state from Redis"
        );
    }

    /// Persist current state to Redis
    async fn persist_state(&self) {
        if let Some(redis_manager) = &self.redis_manager {
            let persisted = self.to_persisted_state().await;
            let key = self.redis_key();

            match serde_json::to_string(&persisted) {
                Ok(json) => {
                    let mut conn = redis_manager.write().await;
                    // Set with 1 hour TTL to auto-cleanup stale circuits
                    let result: Result<(), redis::RedisError> = conn.set_ex(&key, json, 3600).await;

                    if let Err(e) = result {
                        error!(
                            service = %self.service_name,
                            error = %e,
                            "Failed to persist circuit breaker state to Redis"
                        );
                    } else {
                        debug!(
                            service = %self.service_name,
                            state = %persisted.state,
                            "Persisted circuit breaker state to Redis"
                        );
                    }
                }
                Err(e) => {
                    error!(
                        service = %self.service_name,
                        error = %e,
                        "Failed to serialize circuit breaker state"
                    );
                }
            }
        }
    }

    /// Load state from Redis
    async fn load_state(&self) {
        if let Some(redis_manager) = &self.redis_manager {
            let key = self.redis_key();
            let mut conn = redis_manager.write().await;

            let result: Result<Option<String>, redis::RedisError> = conn.get(&key).await;

            match result {
                Ok(Some(json)) => match serde_json::from_str::<PersistedCircuitState>(&json) {
                    Ok(persisted) => {
                        self.from_persisted_state(persisted).await;
                    }
                    Err(e) => {
                        error!(
                            service = %self.service_name,
                            error = %e,
                            "Failed to deserialize circuit breaker state"
                        );
                    }
                },
                Ok(None) => {
                    debug!(
                        service = %self.service_name,
                        "No persisted state found in Redis, using defaults"
                    );
                }
                Err(e) => {
                    error!(
                        service = %self.service_name,
                        error = %e,
                        "Failed to load circuit breaker state from Redis, using defaults"
                    );
                }
            }
        }
    }

    async fn is_open(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, CircuitState::Open { .. })
    }

    #[allow(dead_code)]
    async fn is_half_open(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, CircuitState::HalfOpen)
    }

    async fn check_and_update_state(&self) {
        let mut state = self.state.write().await;
        let mut state_changed = false;

        match &*state {
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= Duration::from_secs(self.config.timeout_seconds) {
                    *state = CircuitState::HalfOpen;
                    state_changed = true;
                    debug!(
                        service = %self.service_name,
                        "Circuit breaker entering half-open state"
                    );
                }
            }
            _ => {}
        }

        drop(state);

        if state_changed {
            self.persist_state().await;
        }
    }

    async fn record_success(&self) {
        let mut success_count = self.success_count.write().await;
        let mut failure_count = self.failure_count.write().await;
        let mut state = self.state.write().await;
        let mut state_changed = false;

        *success_count += 1;

        match &*state {
            CircuitState::HalfOpen => {
                // If we get a success in half-open, close the circuit
                *state = CircuitState::Closed;
                *failure_count = 0;
                *success_count = 0;
                state_changed = true;
                debug!(
                    service = %self.service_name,
                    "Circuit breaker closed after successful half-open request"
                );
            }
            CircuitState::Closed => {
                // Reset failure count on success
                *failure_count = 0;
            }
            _ => {}
        }

        drop(state);
        drop(failure_count);
        drop(success_count);

        if state_changed {
            self.persist_state().await;
        }
    }

    async fn record_failure(&self) {
        let mut failure_count = self.failure_count.write().await;
        let mut state = self.state.write().await;
        let mut last_failure_time = self.last_failure_time.write().await;
        let mut state_changed = false;

        *failure_count += 1;
        *last_failure_time = Some(SystemTime::now());

        match &*state {
            CircuitState::Closed => {
                if *failure_count >= self.config.failure_threshold {
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                    state_changed = true;
                    warn!(
                        service = %self.service_name,
                        failure_count = *failure_count,
                        "Circuit breaker opened due to failures"
                    );
                }
            }
            CircuitState::HalfOpen => {
                // If we fail in half-open, go back to open
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                };
                state_changed = true;
                warn!(
                    service = %self.service_name,
                    "Circuit breaker re-opened after failed half-open request"
                );
            }
            _ => {}
        }

        drop(state);
        drop(failure_count);
        drop(last_failure_time);

        if state_changed {
            self.persist_state().await;
        }
    }

    async fn get_state_string(&self) -> String {
        let state = self.state.read().await;
        match &*state {
            CircuitState::Closed => "closed".to_string(),
            CircuitState::Open { .. } => "open".to_string(),
            CircuitState::HalfOpen => "half_open".to_string(),
        }
    }
}

pub struct CircuitBreakerManager {
    breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    config: Arc<Config>,
    redis_manager: Option<Arc<RwLock<ConnectionManager>>>,
}

impl CircuitBreakerManager {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            config,
            redis_manager: None,
        }
    }

    /// Create a new CircuitBreakerManager with Redis persistence
    pub async fn with_redis(config: Arc<Config>) -> ApiResult<Self> {
        let redis_manager = match Self::create_redis_connection(&config.redis.url).await {
            Ok(conn) => {
                debug!("Circuit breaker Redis persistence enabled");
                Some(Arc::new(RwLock::new(conn)))
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "Failed to connect to Redis for circuit breaker persistence, falling back to in-memory only"
                );
                None
            }
        };

        Ok(Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            config,
            redis_manager,
        })
    }

    /// Create a Redis connection
    async fn create_redis_connection(
        redis_url: &str,
    ) -> Result<ConnectionManager, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        ConnectionManager::new(client).await
    }

    pub async fn get_or_create(&self, service: &str) -> CircuitBreaker {
        let breakers = self.breakers.read().await;

        if let Some(breaker) = breakers.get(service) {
            return breaker.clone();
        }

        drop(breakers);

        let mut breakers = self.breakers.write().await;

        // Double-check after acquiring write lock
        if let Some(breaker) = breakers.get(service) {
            return breaker.clone();
        }

        let breaker = self.create_circuit_breaker(service).await;
        breakers.insert(service.to_string(), breaker.clone());

        breaker
    }

    async fn create_circuit_breaker(&self, service: &str) -> CircuitBreaker {
        let service_config = self
            .config
            .circuit_breaker
            .services
            .get(service)
            .cloned()
            .unwrap_or_else(|| CircuitBreakerServiceConfig {
                failure_threshold: 10,
                timeout_seconds: 2,
                error_rate_threshold: 0.5,
            });

        debug!(
            service = service,
            failure_threshold = service_config.failure_threshold,
            timeout_seconds = service_config.timeout_seconds,
            error_rate = service_config.error_rate_threshold,
            "Created circuit breaker"
        );

        let breaker = CircuitBreaker::new(
            service_config,
            service.to_string(),
            self.redis_manager.clone(),
        );

        // Load persisted state from Redis if available
        breaker.load_state().await;

        breaker
    }

    pub async fn call<F, T, E>(&self, service: &str, operation: F) -> ApiResult<T>
    where
        F: FnOnce() -> Result<T, E> + Send,
        E: std::error::Error + Send + Sync + 'static,
    {
        if !self.config.circuit_breaker.enabled {
            return operation()
                .map_err(|e| ApiError::ProxyError(format!("Service {} error: {}", service, e)));
        }

        let breaker = self.get_or_create(service).await;

        // Check and potentially update the state
        breaker.check_and_update_state().await;

        // If circuit is open, reject the request
        if breaker.is_open().await {
            warn!(service = service, "Circuit breaker open");
            return Err(ApiError::CircuitBreakerOpen(service.to_string()));
        }

        // Execute the operation
        match operation() {
            Ok(result) => {
                breaker.record_success().await;
                Ok(result)
            }
            Err(e) => {
                breaker.record_failure().await;
                warn!(service = service, error = %e, "Service call failed");
                Err(ApiError::ProxyError(format!(
                    "Service {} error: {}",
                    service, e
                )))
            }
        }
    }

    pub async fn get_state(&self, service: &str) -> Option<String> {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(service) {
            Some(breaker.get_state_string().await)
        } else {
            None
        }
    }

    pub async fn get_all_states(&self) -> HashMap<String, String> {
        let breakers = self.breakers.read().await;
        let mut states = HashMap::new();

        for (service, breaker) in breakers.iter() {
            states.insert(service.clone(), breaker.get_state_string().await);
        }

        states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_disabled() {
        let mut config = Config::default();
        config.circuit_breaker.enabled = false;
        let manager = CircuitBreakerManager::new(Arc::new(config));

        let result = manager
            .call("test", || Ok::<_, std::io::Error>("success"))
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = Config::default();
        let manager = CircuitBreakerManager::new(Arc::new(config));

        let result = manager
            .call("discovery", || Ok::<_, std::io::Error>("success"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
