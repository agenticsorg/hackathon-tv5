use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub services: ServicesConfig,
    pub rate_limit: RateLimitConfig,
    pub circuit_breaker: CircuitBreakerConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub discovery: ServiceEndpoint,
    pub sona: ServiceEndpoint,
    pub sync: ServiceEndpoint,
    pub auth: ServiceEndpoint,
    pub playback: ServiceEndpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub tiers: HashMap<String, RateLimitTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitTier {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub burst: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub enabled: bool,
    pub services: HashMap<String, CircuitBreakerServiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerServiceConfig {
    pub failure_threshold: u32,
    pub timeout_seconds: u64,
    pub error_rate_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        let mut rate_limit_tiers = HashMap::new();
        rate_limit_tiers.insert(
            "anonymous".to_string(),
            RateLimitTier {
                requests_per_second: 5,
                requests_per_minute: 100,
                burst: 10,
            },
        );
        rate_limit_tiers.insert(
            "free".to_string(),
            RateLimitTier {
                requests_per_second: 10,
                requests_per_minute: 200,
                burst: 20,
            },
        );
        rate_limit_tiers.insert(
            "pro".to_string(),
            RateLimitTier {
                requests_per_second: 50,
                requests_per_minute: 1000,
                burst: 100,
            },
        );
        rate_limit_tiers.insert(
            "enterprise".to_string(),
            RateLimitTier {
                requests_per_second: 200,
                requests_per_minute: 5000,
                burst: 400,
            },
        );

        let mut circuit_breaker_services = HashMap::new();
        circuit_breaker_services.insert(
            "discovery".to_string(),
            CircuitBreakerServiceConfig {
                failure_threshold: 20,
                timeout_seconds: 3,
                error_rate_threshold: 0.5,
            },
        );
        circuit_breaker_services.insert(
            "sona".to_string(),
            CircuitBreakerServiceConfig {
                failure_threshold: 10,
                timeout_seconds: 2,
                error_rate_threshold: 0.4,
            },
        );
        circuit_breaker_services.insert(
            "playback".to_string(),
            CircuitBreakerServiceConfig {
                failure_threshold: 15,
                timeout_seconds: 3,
                error_rate_threshold: 0.4,
            },
        );
        circuit_breaker_services.insert(
            "sync".to_string(),
            CircuitBreakerServiceConfig {
                failure_threshold: 15,
                timeout_seconds: 3,
                error_rate_threshold: 0.5,
            },
        );

        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: num_cpus::get(),
                max_connections: 25000,
            },
            services: ServicesConfig {
                discovery: ServiceEndpoint {
                    url: "http://localhost:8081".to_string(),
                    timeout_ms: 5000,
                },
                sona: ServiceEndpoint {
                    url: "http://localhost:8082".to_string(),
                    timeout_ms: 3000,
                },
                sync: ServiceEndpoint {
                    url: "http://localhost:8083".to_string(),
                    timeout_ms: 5000,
                },
                auth: ServiceEndpoint {
                    url: "http://localhost:8084".to_string(),
                    timeout_ms: 3000,
                },
                playback: ServiceEndpoint {
                    url: "http://localhost:8086".to_string(),
                    timeout_ms: 5000,
                },
            },
            rate_limit: RateLimitConfig {
                enabled: true,
                tiers: rate_limit_tiers,
            },
            circuit_breaker: CircuitBreakerConfig {
                enabled: true,
                services: circuit_breaker_services,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
            },
        }
    }
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let mut config = Config::default();

        if let Ok(host) = std::env::var("API_GATEWAY_HOST") {
            config.server.host = host;
        }

        if let Ok(port) = std::env::var("API_GATEWAY_PORT") {
            config.server.port = port.parse()?;
        }

        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            config.redis.url = redis_url;
        }

        if let Ok(discovery_url) = std::env::var("DISCOVERY_SERVICE_URL") {
            config.services.discovery.url = discovery_url;
        }

        if let Ok(sona_url) = std::env::var("SONA_SERVICE_URL") {
            config.services.sona.url = sona_url;
        }

        if let Ok(sync_url) = std::env::var("SYNC_SERVICE_URL") {
            config.services.sync.url = sync_url;
        }

        if let Ok(auth_url) = std::env::var("AUTH_SERVICE_URL") {
            config.services.auth.url = auth_url;
        }

        if let Ok(playback_url) = std::env::var("PLAYBACK_SERVICE_URL") {
            config.services.playback.url = playback_url;
        }

        Ok(config)
    }
}
