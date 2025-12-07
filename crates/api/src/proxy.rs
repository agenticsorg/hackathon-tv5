use crate::circuit_breaker::CircuitBreakerManager;
use crate::config::Config;
use crate::error::{ApiError, ApiResult};
use actix_web::web::Bytes;
use reqwest::{header::HeaderMap, Method, StatusCode};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};

pub struct ServiceProxy {
    client: reqwest::Client,
    config: Arc<Config>,
    circuit_breaker: Arc<CircuitBreakerManager>,
}

pub struct ProxyRequest {
    pub service: String,
    pub path: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
    pub query: Option<String>,
}

pub struct ProxyResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

impl ServiceProxy {
    pub fn new(config: Arc<Config>, circuit_breaker: Arc<CircuitBreakerManager>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(100)
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            circuit_breaker,
        }
    }

    pub async fn forward(&self, request: ProxyRequest) -> ApiResult<ProxyResponse> {
        let service_url = self.get_service_url(&request.service)?;

        let url = if let Some(query) = &request.query {
            format!("{}{}?{}", service_url, request.path, query)
        } else {
            format!("{}{}", service_url, request.path)
        };

        debug!(
            service = request.service,
            method = %request.method,
            url = %url,
            "Forwarding request"
        );

        let service = request.service.clone();
        let method = request.method.clone();
        let headers = request.headers.clone();
        let body = request.body.clone();
        let client = self.client.clone();

        // Use circuit breaker for the request
        let response = self
            .circuit_breaker
            .call(&service, move || {
                let url = url.clone();
                let client = client.clone();
                let method = method.clone();
                let headers = headers.clone();
                let body = body.clone();

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async move {
                        let mut req = client.request(method, &url);

                        // Forward headers
                        for (key, value) in headers.iter() {
                            req = req.header(key, value);
                        }

                        // Add body if present
                        if let Some(body_bytes) = body {
                            req = req.body(body_bytes);
                        }

                        req.send().await.map_err(|e| {
                            error!(error = %e, "Request failed");
                            std::io::Error::new(std::io::ErrorKind::Other, e)
                        })
                    })
                })
            })
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response
            .bytes()
            .await
            .map_err(|e| ApiError::ProxyError(e.to_string()))?;

        debug!(
            service = request.service,
            status = status.as_u16(),
            "Received response"
        );

        Ok(ProxyResponse {
            status,
            headers,
            body,
        })
    }

    fn get_service_url(&self, service: &str) -> ApiResult<String> {
        let url = match service {
            "discovery" => &self.config.services.discovery.url,
            "sona" => &self.config.services.sona.url,
            "sync" => &self.config.services.sync.url,
            "auth" => &self.config.services.auth.url,
            "playback" => &self.config.services.playback.url,
            _ => {
                return Err(ApiError::BadRequest(format!(
                    "Unknown service: {}",
                    service
                )))
            }
        };

        Ok(url.clone())
    }

    pub async fn get_service_health(&self, service: &str) -> ApiResult<bool> {
        let service_url = self.get_service_url(service)?;
        let url = format!("{}/health", service_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_service_url() {
        let config = Arc::new(Config::default());
        let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.clone()));
        let proxy = ServiceProxy::new(config, circuit_breaker);

        assert!(proxy.get_service_url("discovery").is_ok());
        assert!(proxy.get_service_url("sona").is_ok());
        assert!(proxy.get_service_url("sync").is_ok());
        assert!(proxy.get_service_url("auth").is_ok());
        assert!(proxy.get_service_url("playback").is_ok());
        assert!(proxy.get_service_url("invalid").is_err());
    }
}
