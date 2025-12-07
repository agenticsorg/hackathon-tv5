use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::{Deserialize, Serialize};
use std::fmt;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Rate limit exceeded")]
    RateLimited { retry_after: u64 },

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Circuit breaker open for service: {0}")]
    CircuitBreakerOpen(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),

    #[error("Proxy error: {0}")]
    ProxyError(String),

    #[error("Timeout")]
    Timeout,
}

impl ApiError {
    pub fn error_code(&self) -> &str {
        match self {
            Self::RateLimited { .. } => "RATE_LIMITED",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::CircuitBreakerOpen(_) => "CIRCUIT_BREAKER_OPEN",
            Self::AuthenticationFailed(_) => "AUTH_FAILED",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::NotFound(_) => "NOT_FOUND",
            Self::InternalError(_) => "INTERNAL_ERROR",
            Self::ProxyError(_) => "PROXY_ERROR",
            Self::Timeout => "TIMEOUT",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::CircuitBreakerOpen(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::AuthenticationFailed(_) => StatusCode::UNAUTHORIZED,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ProxyError(_) => StatusCode::BAD_GATEWAY,
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }

    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after } => Some(*retry_after),
            _ => None,
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status_code()
    }

    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                retry_after: self.retry_after(),
            },
        };

        HttpResponse::build(self.status_code()).json(error_response)
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error.code, self.error.message)
    }
}
