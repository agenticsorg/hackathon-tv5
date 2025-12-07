//! Resilience patterns for external service integration
//!
//! This module provides circuit breaker and fallback mechanisms for handling
//! failures in external service calls, preventing cascading failures and
//! providing graceful degradation.

pub mod circuit_breaker;

pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitState,
};

#[cfg(test)]
mod tests;
