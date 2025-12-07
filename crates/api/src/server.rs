use crate::circuit_breaker::CircuitBreakerManager;
use crate::config::Config;
use crate::health::{self, HealthAggregator, HealthChecker};
use crate::middleware::{LoggingMiddleware, RequestIdMiddleware};
use crate::proxy::ServiceProxy;
use crate::rate_limit::RateLimiter;
use crate::routes;
use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub struct Server {
    config: Arc<Config>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .json()
            .init();

        info!("Starting API Gateway");
        info!("Version: {}", env!("CARGO_PKG_VERSION"));
        info!("Host: {}", self.config.server.host);
        info!("Port: {}", self.config.server.port);

        // Initialize circuit breaker manager
        let circuit_breaker = Arc::new(CircuitBreakerManager::new(self.config.clone()));
        info!("Circuit breaker initialized");

        // Initialize service proxy
        let proxy = Arc::new(ServiceProxy::new(
            self.config.clone(),
            circuit_breaker.clone(),
        ));
        info!("Service proxy initialized");

        // Initialize rate limiter
        let rate_limiter = Arc::new(RateLimiter::new(self.config.clone()).await?);
        info!("Rate limiter initialized");

        // Initialize health checker
        let health_checker = Arc::new(HealthChecker::new(proxy.clone(), circuit_breaker.clone()));
        info!("Health checker initialized");

        // Initialize health aggregator
        let health_aggregator = Arc::new(HealthAggregator::new(Duration::from_secs(5)));
        info!("Health aggregator initialized");

        let bind_addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        info!("Binding to {}", bind_addr);

        // Create HTTP server
        HttpServer::new(move || {
            // Configure CORS
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .expose_headers(vec![
                    "X-Request-ID",
                    "X-Correlation-ID",
                    "X-RateLimit-Limit",
                    "X-RateLimit-Remaining",
                    "X-RateLimit-Reset",
                ])
                .max_age(3600);

            App::new()
                // Add shared state
                .app_data(web::Data::new(proxy.clone()))
                .app_data(web::Data::new(rate_limiter.clone()))
                .app_data(web::Data::new(circuit_breaker.clone()))
                .app_data(web::Data::new(health_checker.clone()))
                .app_data(web::Data::new(health_aggregator.clone()))
                // Add middleware
                .wrap(cors)
                .wrap(LoggingMiddleware)
                .wrap(RequestIdMiddleware)
                // Health endpoints (no authentication required)
                .route("/health", web::get().to(health::health))
                .route("/health/ready", web::get().to(health::readiness))
                .route("/health/live", web::get().to(health::liveness))
                .route("/health/aggregate", web::get().to(health::aggregate))
                // Health dashboard static files
                .service(
                    fs::Files::new("/dashboard/health", "apps/health-dashboard")
                        .index_file("index.html")
                        .use_last_modified(true),
                )
                // API routes
                .configure(routes::configure)
        })
        .workers(self.config.server.workers)
        .max_connections(self.config.server.max_connections)
        .bind(&bind_addr)?
        .run()
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = Config::default();
        let _server = Server::new(config);
    }
}
