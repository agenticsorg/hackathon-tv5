//! Shared configuration loader module for Media Gateway services
//!
//! This module provides a unified configuration loading system with environment variable
//! parsing, validation, and support for .env files. All configuration uses the
//! `MEDIA_GATEWAY_` prefix for environment variables.
//!
//! # Features
//!
//! - Environment variable parsing with typed values
//! - .env file support via dotenvy
//! - Configuration validation with clear error messages
//! - Default values for optional fields
//! - URL, port, and timeout validation
//! - Configuration override hierarchy: defaults < .env < environment
//!
//! # Example
//!
//! ```no_run
//! use media_gateway_core::config::{ConfigLoader, DatabaseConfig, RedisConfig, ServiceConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load .env file (optional)
//! dotenvy::dotenv().ok();
//!
//! // Load and validate configurations
//! let db_config = DatabaseConfig::from_env()?;
//! let redis_config = RedisConfig::from_env()?;
//! let service_config = ServiceConfig::from_env()?;
//!
//! // Validate all configs
//! db_config.validate()?;
//! redis_config.validate()?;
//! service_config.validate()?;
//! # Ok(())
//! # }
//! ```

use crate::error::MediaGatewayError;
use std::time::Duration;
use url::Url;

/// Configuration loader trait
///
/// Provides standardized methods for loading and validating configuration from
/// environment variables.
pub trait ConfigLoader: Sized {
    /// Load configuration from environment variables
    ///
    /// Reads environment variables with the `MEDIA_GATEWAY_` prefix and constructs
    /// a configuration instance with defaults for missing optional values.
    ///
    /// # Errors
    ///
    /// Returns a `ConfigurationError` if:
    /// - Required environment variables are missing
    /// - Environment variable values cannot be parsed
    /// - Values are outside acceptable ranges
    fn from_env() -> Result<Self, MediaGatewayError>;

    /// Validate configuration values
    ///
    /// Performs validation checks on all configuration fields to ensure they meet
    /// requirements (e.g., valid URLs, port ranges, positive timeouts).
    ///
    /// # Errors
    ///
    /// Returns a `ConfigurationError` if any validation check fails.
    fn validate(&self) -> Result<(), MediaGatewayError>;
}

/// Database configuration
///
/// Configuration for PostgreSQL database connections with connection pooling settings.
///
/// # Environment Variables
///
/// - `MEDIA_GATEWAY_DATABASE_URL` (required): PostgreSQL connection URL
/// - `MEDIA_GATEWAY_DATABASE_MAX_CONNECTIONS` (optional): Maximum pool connections (default: 20)
/// - `MEDIA_GATEWAY_DATABASE_MIN_CONNECTIONS` (optional): Minimum pool connections (default: 2)
/// - `MEDIA_GATEWAY_DATABASE_CONNECT_TIMEOUT` (optional): Connection timeout in seconds (default: 30)
/// - `MEDIA_GATEWAY_DATABASE_IDLE_TIMEOUT` (optional): Idle connection timeout in seconds (default: 600)
///
/// # Example
///
/// ```bash
/// export MEDIA_GATEWAY_DATABASE_URL="postgresql://user:pass@localhost:5432/media_gateway"
/// export MEDIA_GATEWAY_DATABASE_MAX_CONNECTIONS="50"
/// export MEDIA_GATEWAY_DATABASE_MIN_CONNECTIONS="5"
/// export MEDIA_GATEWAY_DATABASE_CONNECT_TIMEOUT="60"
/// ```
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Connection timeout duration
    pub connect_timeout: Duration,
    /// Idle connection timeout duration
    pub idle_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgresql://localhost/media_gateway".to_string(),
            max_connections: 20,
            min_connections: 2,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        }
    }
}

impl ConfigLoader for DatabaseConfig {
    fn from_env() -> Result<Self, MediaGatewayError> {
        let url = std::env::var("MEDIA_GATEWAY_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .map_err(|_| MediaGatewayError::ConfigurationError {
                message: "DATABASE_URL or MEDIA_GATEWAY_DATABASE_URL must be set".to_string(),
                key: Some("MEDIA_GATEWAY_DATABASE_URL".to_string()),
            })?;

        let max_connections = parse_env_var(
            "MEDIA_GATEWAY_DATABASE_MAX_CONNECTIONS",
            DatabaseConfig::default().max_connections,
        )?;

        let min_connections = parse_env_var(
            "MEDIA_GATEWAY_DATABASE_MIN_CONNECTIONS",
            DatabaseConfig::default().min_connections,
        )?;

        let connect_timeout_secs = parse_env_var("MEDIA_GATEWAY_DATABASE_CONNECT_TIMEOUT", 30u64)?;

        let idle_timeout_secs = parse_env_var("MEDIA_GATEWAY_DATABASE_IDLE_TIMEOUT", 600u64)?;

        Ok(Self {
            url,
            max_connections,
            min_connections,
            connect_timeout: Duration::from_secs(connect_timeout_secs),
            idle_timeout: Duration::from_secs(idle_timeout_secs),
        })
    }

    fn validate(&self) -> Result<(), MediaGatewayError> {
        // Validate URL format
        Url::parse(&self.url).map_err(|e| MediaGatewayError::ConfigurationError {
            message: format!("Invalid DATABASE_URL: {}", e),
            key: Some("MEDIA_GATEWAY_DATABASE_URL".to_string()),
        })?;

        // Validate connection counts
        if self.max_connections == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "max_connections must be greater than 0".to_string(),
                key: Some("MEDIA_GATEWAY_DATABASE_MAX_CONNECTIONS".to_string()),
            });
        }

        if self.min_connections > self.max_connections {
            return Err(MediaGatewayError::ConfigurationError {
                message: format!(
                    "min_connections ({}) cannot exceed max_connections ({})",
                    self.min_connections, self.max_connections
                ),
                key: Some("MEDIA_GATEWAY_DATABASE_MIN_CONNECTIONS".to_string()),
            });
        }

        // Validate timeouts
        if self.connect_timeout.as_secs() == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "connect_timeout must be greater than 0 seconds".to_string(),
                key: Some("MEDIA_GATEWAY_DATABASE_CONNECT_TIMEOUT".to_string()),
            });
        }

        if self.idle_timeout.as_secs() == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "idle_timeout must be greater than 0 seconds".to_string(),
                key: Some("MEDIA_GATEWAY_DATABASE_IDLE_TIMEOUT".to_string()),
            });
        }

        Ok(())
    }
}

/// Redis configuration
///
/// Configuration for Redis cache connections with connection pooling settings.
///
/// # Environment Variables
///
/// - `MEDIA_GATEWAY_REDIS_URL` (required): Redis connection URL
/// - `MEDIA_GATEWAY_REDIS_MAX_CONNECTIONS` (optional): Maximum pool connections (default: 10)
/// - `MEDIA_GATEWAY_REDIS_CONNECTION_TIMEOUT` (optional): Connection timeout in seconds (default: 10)
/// - `MEDIA_GATEWAY_REDIS_RESPONSE_TIMEOUT` (optional): Response timeout in seconds (default: 5)
///
/// # Example
///
/// ```bash
/// export MEDIA_GATEWAY_REDIS_URL="redis://localhost:6379/0"
/// export MEDIA_GATEWAY_REDIS_MAX_CONNECTIONS="20"
/// export MEDIA_GATEWAY_REDIS_CONNECTION_TIMEOUT="15"
/// ```
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout duration
    pub connection_timeout: Duration,
    /// Response timeout duration
    pub response_timeout: Duration,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379/0".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(10),
            response_timeout: Duration::from_secs(5),
        }
    }
}

impl ConfigLoader for RedisConfig {
    fn from_env() -> Result<Self, MediaGatewayError> {
        let url = std::env::var("MEDIA_GATEWAY_REDIS_URL")
            .or_else(|_| std::env::var("REDIS_URL"))
            .map_err(|_| MediaGatewayError::ConfigurationError {
                message: "REDIS_URL or MEDIA_GATEWAY_REDIS_URL must be set".to_string(),
                key: Some("MEDIA_GATEWAY_REDIS_URL".to_string()),
            })?;

        let max_connections = parse_env_var(
            "MEDIA_GATEWAY_REDIS_MAX_CONNECTIONS",
            RedisConfig::default().max_connections,
        )?;

        let connection_timeout_secs =
            parse_env_var("MEDIA_GATEWAY_REDIS_CONNECTION_TIMEOUT", 10u64)?;

        let response_timeout_secs = parse_env_var("MEDIA_GATEWAY_REDIS_RESPONSE_TIMEOUT", 5u64)?;

        Ok(Self {
            url,
            max_connections,
            connection_timeout: Duration::from_secs(connection_timeout_secs),
            response_timeout: Duration::from_secs(response_timeout_secs),
        })
    }

    fn validate(&self) -> Result<(), MediaGatewayError> {
        // Validate URL format
        Url::parse(&self.url).map_err(|e| MediaGatewayError::ConfigurationError {
            message: format!("Invalid REDIS_URL: {}", e),
            key: Some("MEDIA_GATEWAY_REDIS_URL".to_string()),
        })?;

        // Validate max connections
        if self.max_connections == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "max_connections must be greater than 0".to_string(),
                key: Some("MEDIA_GATEWAY_REDIS_MAX_CONNECTIONS".to_string()),
            });
        }

        // Validate timeouts
        if self.connection_timeout.as_secs() == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "connection_timeout must be greater than 0 seconds".to_string(),
                key: Some("MEDIA_GATEWAY_REDIS_CONNECTION_TIMEOUT".to_string()),
            });
        }

        if self.response_timeout.as_secs() == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "response_timeout must be greater than 0 seconds".to_string(),
                key: Some("MEDIA_GATEWAY_REDIS_RESPONSE_TIMEOUT".to_string()),
            });
        }

        Ok(())
    }
}

/// Service configuration
///
/// Configuration for HTTP service settings including host, port, workers, and logging.
///
/// # Environment Variables
///
/// - `MEDIA_GATEWAY_SERVICE_HOST` (optional): Service bind host (default: "0.0.0.0")
/// - `MEDIA_GATEWAY_SERVICE_PORT` (optional): Service bind port (default: 8080)
/// - `MEDIA_GATEWAY_SERVICE_WORKERS` (optional): Number of worker threads (default: CPU count)
/// - `MEDIA_GATEWAY_SERVICE_LOG_LEVEL` (optional): Log level (default: "info")
/// - `MEDIA_GATEWAY_SERVICE_REQUEST_TIMEOUT` (optional): Request timeout in seconds (default: 60)
///
/// # Example
///
/// ```bash
/// export MEDIA_GATEWAY_SERVICE_HOST="127.0.0.1"
/// export MEDIA_GATEWAY_SERVICE_PORT="3000"
/// export MEDIA_GATEWAY_SERVICE_WORKERS="4"
/// export MEDIA_GATEWAY_SERVICE_LOG_LEVEL="debug"
/// ```
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Service bind host
    pub host: String,
    /// Service bind port
    pub port: u16,
    /// Number of worker threads
    pub workers: usize,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Request timeout duration
    pub request_timeout: Duration,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: num_cpus::get(),
            log_level: "info".to_string(),
            request_timeout: Duration::from_secs(60),
        }
    }
}

impl ConfigLoader for ServiceConfig {
    fn from_env() -> Result<Self, MediaGatewayError> {
        let host = std::env::var("MEDIA_GATEWAY_SERVICE_HOST")
            .or_else(|_| std::env::var("HOST"))
            .unwrap_or_else(|_| ServiceConfig::default().host);

        let port = parse_env_var("MEDIA_GATEWAY_SERVICE_PORT", ServiceConfig::default().port)
            .or_else(|_| parse_env_var("PORT", ServiceConfig::default().port))?;

        let workers = parse_env_var(
            "MEDIA_GATEWAY_SERVICE_WORKERS",
            ServiceConfig::default().workers,
        )?;

        let log_level = std::env::var("MEDIA_GATEWAY_SERVICE_LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG"))
            .unwrap_or_else(|_| ServiceConfig::default().log_level);

        let request_timeout_secs = parse_env_var("MEDIA_GATEWAY_SERVICE_REQUEST_TIMEOUT", 60u64)?;

        Ok(Self {
            host,
            port,
            workers,
            log_level,
            request_timeout: Duration::from_secs(request_timeout_secs),
        })
    }

    fn validate(&self) -> Result<(), MediaGatewayError> {
        // Validate port range
        if self.port == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "port must be greater than 0".to_string(),
                key: Some("MEDIA_GATEWAY_SERVICE_PORT".to_string()),
            });
        }

        // Validate workers
        if self.workers == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "workers must be greater than 0".to_string(),
                key: Some("MEDIA_GATEWAY_SERVICE_WORKERS".to_string()),
            });
        }

        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.to_lowercase().as_str()) {
            return Err(MediaGatewayError::ConfigurationError {
                message: format!(
                    "Invalid log_level '{}'. Must be one of: {}",
                    self.log_level,
                    valid_log_levels.join(", ")
                ),
                key: Some("MEDIA_GATEWAY_SERVICE_LOG_LEVEL".to_string()),
            });
        }

        // Validate request timeout
        if self.request_timeout.as_secs() == 0 {
            return Err(MediaGatewayError::ConfigurationError {
                message: "request_timeout must be greater than 0 seconds".to_string(),
                key: Some("MEDIA_GATEWAY_SERVICE_REQUEST_TIMEOUT".to_string()),
            });
        }

        Ok(())
    }
}

/// Helper function to parse environment variable with default value
///
/// # Type Parameters
///
/// * `T` - The type to parse into (must implement FromStr)
///
/// # Arguments
///
/// * `key` - The environment variable key
/// * `default` - The default value if the variable is not set
///
/// # Returns
///
/// The parsed value or default if not set
///
/// # Errors
///
/// Returns a `ConfigurationError` if the value cannot be parsed
fn parse_env_var<T>(key: &str, default: T) -> Result<T, MediaGatewayError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    std::env::var(key)
        .ok()
        .map(|v| {
            v.parse::<T>()
                .map_err(|e| MediaGatewayError::ConfigurationError {
                    message: format!("Failed to parse {}: {}", key, e),
                    key: Some(key.to_string()),
                })
        })
        .unwrap_or(Ok(default))
}

/// Load .env file if present
///
/// This is a convenience function that loads environment variables from a .env file
/// using dotenvy. It does not return an error if the .env file is not found.
///
/// # Example
///
/// ```no_run
/// use media_gateway_core::config::load_dotenv;
///
/// // Load .env file at the start of your application
/// load_dotenv();
/// ```
pub fn load_dotenv() {
    if let Err(e) = dotenvy::dotenv() {
        // Only log if it's not a "file not found" error
        if !e.to_string().contains("not found") {
            eprintln!("Warning: Failed to load .env file: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// Helper to set environment variable for test
    fn set_test_env(key: &str, value: &str) {
        env::set_var(key, value);
    }

    /// Helper to remove environment variable after test
    fn clear_test_env(key: &str) {
        env::remove_var(key);
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.connect_timeout, Duration::from_secs(30));
        assert_eq!(config.idle_timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_database_config_from_env() {
        set_test_env("MEDIA_GATEWAY_DATABASE_URL", "postgresql://localhost/test");
        set_test_env("MEDIA_GATEWAY_DATABASE_MAX_CONNECTIONS", "50");
        set_test_env("MEDIA_GATEWAY_DATABASE_MIN_CONNECTIONS", "5");

        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.url, "postgresql://localhost/test");
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.min_connections, 5);

        clear_test_env("MEDIA_GATEWAY_DATABASE_URL");
        clear_test_env("MEDIA_GATEWAY_DATABASE_MAX_CONNECTIONS");
        clear_test_env("MEDIA_GATEWAY_DATABASE_MIN_CONNECTIONS");
    }

    #[test]
    fn test_database_config_validation_invalid_url() {
        let mut config = DatabaseConfig::default();
        config.url = "not-a-valid-url".to_string();

        let result = config.validate();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            MediaGatewayError::ConfigurationError { .. }
        ));
    }

    #[test]
    fn test_database_config_validation_zero_max_connections() {
        let mut config = DatabaseConfig::default();
        config.url = "postgresql://localhost/test".to_string();
        config.max_connections = 0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_database_config_validation_min_exceeds_max() {
        let mut config = DatabaseConfig::default();
        config.url = "postgresql://localhost/test".to_string();
        config.min_connections = 30;
        config.max_connections = 20;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert_eq!(config.response_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_redis_config_from_env() {
        set_test_env("MEDIA_GATEWAY_REDIS_URL", "redis://localhost:6379/1");
        set_test_env("MEDIA_GATEWAY_REDIS_MAX_CONNECTIONS", "20");

        let config = RedisConfig::from_env().unwrap();
        assert_eq!(config.url, "redis://localhost:6379/1");
        assert_eq!(config.max_connections, 20);

        clear_test_env("MEDIA_GATEWAY_REDIS_URL");
        clear_test_env("MEDIA_GATEWAY_REDIS_MAX_CONNECTIONS");
    }

    #[test]
    fn test_redis_config_validation_invalid_url() {
        let mut config = RedisConfig::default();
        config.url = "invalid-redis-url".to_string();

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_config_default() {
        let config = ServiceConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.workers > 0);
    }

    #[test]
    fn test_service_config_from_env() {
        set_test_env("MEDIA_GATEWAY_SERVICE_HOST", "127.0.0.1");
        set_test_env("MEDIA_GATEWAY_SERVICE_PORT", "3000");
        set_test_env("MEDIA_GATEWAY_SERVICE_WORKERS", "4");
        set_test_env("MEDIA_GATEWAY_SERVICE_LOG_LEVEL", "debug");

        let config = ServiceConfig::from_env().unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.workers, 4);
        assert_eq!(config.log_level, "debug");

        clear_test_env("MEDIA_GATEWAY_SERVICE_HOST");
        clear_test_env("MEDIA_GATEWAY_SERVICE_PORT");
        clear_test_env("MEDIA_GATEWAY_SERVICE_WORKERS");
        clear_test_env("MEDIA_GATEWAY_SERVICE_LOG_LEVEL");
    }

    #[test]
    fn test_service_config_validation_invalid_log_level() {
        let mut config = ServiceConfig::default();
        config.log_level = "invalid".to_string();

        let result = config.validate();
        assert!(result.is_err());
        match result.unwrap_err() {
            MediaGatewayError::ConfigurationError { message, .. } => {
                assert!(message.contains("Invalid log_level"));
            }
            _ => panic!("Expected ConfigurationError"),
        }
    }

    #[test]
    fn test_service_config_validation_zero_port() {
        let mut config = ServiceConfig::default();
        config.port = 0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_service_config_validation_zero_workers() {
        let mut config = ServiceConfig::default();
        config.workers = 0;

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_env_var_with_default() {
        let result: u32 = parse_env_var("NON_EXISTENT_VAR", 42).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_env_var_with_value() {
        set_test_env("TEST_PARSE_VAR", "100");
        let result: u32 = parse_env_var("TEST_PARSE_VAR", 42).unwrap();
        assert_eq!(result, 100);
        clear_test_env("TEST_PARSE_VAR");
    }

    #[test]
    fn test_parse_env_var_invalid_value() {
        set_test_env("TEST_INVALID_VAR", "not-a-number");
        let result: Result<u32, _> = parse_env_var("TEST_INVALID_VAR", 42);
        assert!(result.is_err());
        clear_test_env("TEST_INVALID_VAR");
    }

    #[test]
    fn test_database_url_fallback() {
        // Test that DATABASE_URL is used as fallback
        set_test_env("DATABASE_URL", "postgresql://fallback/test");
        let config = DatabaseConfig::from_env().unwrap();
        assert_eq!(config.url, "postgresql://fallback/test");
        clear_test_env("DATABASE_URL");
    }

    #[test]
    fn test_redis_url_fallback() {
        // Test that REDIS_URL is used as fallback
        set_test_env("REDIS_URL", "redis://fallback:6379");
        let config = RedisConfig::from_env().unwrap();
        assert_eq!(config.url, "redis://fallback:6379");
        clear_test_env("REDIS_URL");
    }

    #[test]
    fn test_service_port_fallback() {
        // Test that PORT is used as fallback
        set_test_env("PORT", "9000");
        let config = ServiceConfig::from_env().unwrap();
        assert_eq!(config.port, 9000);
        clear_test_env("PORT");
    }
}
