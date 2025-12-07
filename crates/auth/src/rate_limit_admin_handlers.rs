use crate::{
    error::{AuthError, Result},
    middleware::extract_user_context,
    rate_limit_config::{RateLimitConfig, RateLimitConfigStore, UserTier},
};
use actix_web::{delete, get, put, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct RateLimitConfigResponse {
    pub endpoint: String,
    pub tier: String,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
}

impl From<RateLimitConfig> for RateLimitConfigResponse {
    fn from(config: RateLimitConfig) -> Self {
        Self {
            endpoint: config.endpoint,
            tier: config.tier.to_string(),
            requests_per_minute: config.requests_per_minute,
            requests_per_hour: config.requests_per_hour,
            burst_size: config.burst_size,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListRateLimitConfigsResponse {
    pub configs: Vec<RateLimitConfigResponse>,
    pub total: usize,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRateLimitConfigRequest {
    pub tier: String,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
}

impl UpdateRateLimitConfigRequest {
    pub fn validate(&self) -> Result<()> {
        if self.requests_per_minute == 0 && self.requests_per_hour == 0 {
            return Err(AuthError::Internal(
                "At least one rate limit must be non-zero".to_string(),
            ));
        }

        if self.burst_size == 0 {
            return Err(AuthError::Internal(
                "Burst size must be non-zero".to_string(),
            ));
        }

        if self.burst_size < self.requests_per_minute {
            return Err(AuthError::Internal(
                "Burst size must be >= requests per minute".to_string(),
            ));
        }

        Ok(())
    }

    pub fn to_config(&self, endpoint: String) -> Result<RateLimitConfig> {
        self.validate()?;

        let tier: UserTier = self.tier.parse()?;

        Ok(RateLimitConfig::new(
            endpoint,
            tier,
            self.requests_per_minute,
            self.requests_per_hour,
            self.burst_size,
        ))
    }
}

#[derive(Debug, Deserialize)]
pub struct DeleteRateLimitConfigQuery {
    pub tier: String,
}

#[get("/api/v1/admin/rate-limits")]
pub async fn list_rate_limits(
    req: HttpRequest,
    store: web::Data<Arc<RateLimitConfigStore>>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;

    if !admin_context.has_role(&crate::rbac::Role::Admin) {
        return Err(AuthError::Internal("Admin role required".to_string()));
    }

    let configs = store.get_all_configs().await?;
    let total = configs.len();

    let response = ListRateLimitConfigsResponse {
        configs: configs.into_iter().map(Into::into).collect(),
        total,
    };

    Ok(HttpResponse::Ok().json(response))
}

#[get("/api/v1/admin/rate-limits/{endpoint:.*}")]
pub async fn get_rate_limit(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<DeleteRateLimitConfigQuery>,
    store: web::Data<Arc<RateLimitConfigStore>>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;

    if !admin_context.has_role(&crate::rbac::Role::Admin) {
        return Err(AuthError::Internal("Admin role required".to_string()));
    }

    let endpoint = path.into_inner();
    let tier: UserTier = query.tier.parse()?;

    match store.get_config(&endpoint, tier).await? {
        Some(config) => {
            let response: RateLimitConfigResponse = config.into();
            Ok(HttpResponse::Ok().json(response))
        }
        None => {
            let default_config = store.get_default_config(tier).await;
            let response: RateLimitConfigResponse = default_config.into();
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "config": response,
                "is_default": true
            })))
        }
    }
}

#[put("/api/v1/admin/rate-limits/{endpoint:.*}")]
pub async fn update_rate_limit(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<UpdateRateLimitConfigRequest>,
    store: web::Data<Arc<RateLimitConfigStore>>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;

    if !admin_context.has_role(&crate::rbac::Role::Admin) {
        return Err(AuthError::Internal("Admin role required".to_string()));
    }

    let endpoint = path.into_inner();
    let update_req = body.into_inner();

    let config = update_req.to_config(endpoint)?;

    let admin_user_id = Uuid::parse_str(&admin_context.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid admin UUID: {}", e)))?;

    let existing = store.get_config(&config.endpoint, config.tier).await?;
    let action = if existing.is_some() {
        "update"
    } else {
        "create"
    };

    store.set_config(&config).await?;

    store
        .log_config_change(admin_user_id, action, &config)
        .await?;

    let response: RateLimitConfigResponse = config.into();
    Ok(HttpResponse::Ok().json(response))
}

#[delete("/api/v1/admin/rate-limits/{endpoint:.*}")]
pub async fn delete_rate_limit(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<DeleteRateLimitConfigQuery>,
    store: web::Data<Arc<RateLimitConfigStore>>,
) -> Result<impl Responder> {
    let admin_context = extract_user_context(&req)?;

    if !admin_context.has_role(&crate::rbac::Role::Admin) {
        return Err(AuthError::Internal("Admin role required".to_string()));
    }

    let endpoint = path.into_inner();
    let tier: UserTier = query.tier.parse()?;

    let admin_user_id = Uuid::parse_str(&admin_context.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid admin UUID: {}", e)))?;

    let config_before = store.get_config(&endpoint, tier).await?;

    let deleted = store.delete_config(&endpoint, tier).await?;

    if !deleted {
        return Err(AuthError::Internal(
            "Rate limit config not found".to_string(),
        ));
    }

    if let Some(config) = config_before {
        store
            .log_config_change(admin_user_id, "delete", &config)
            .await?;
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Rate limit config deleted successfully",
        "endpoint": endpoint,
        "tier": tier.to_string()
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_rate_limit_config_request_validation() {
        let valid_req = UpdateRateLimitConfigRequest {
            tier: "free".to_string(),
            requests_per_minute: 10,
            requests_per_hour: 100,
            burst_size: 15,
        };
        assert!(valid_req.validate().is_ok());

        let zero_limits = UpdateRateLimitConfigRequest {
            tier: "free".to_string(),
            requests_per_minute: 0,
            requests_per_hour: 0,
            burst_size: 15,
        };
        assert!(zero_limits.validate().is_err());

        let zero_burst = UpdateRateLimitConfigRequest {
            tier: "free".to_string(),
            requests_per_minute: 10,
            requests_per_hour: 100,
            burst_size: 0,
        };
        assert!(zero_burst.validate().is_err());

        let invalid_burst = UpdateRateLimitConfigRequest {
            tier: "free".to_string(),
            requests_per_minute: 10,
            requests_per_hour: 100,
            burst_size: 5,
        };
        assert!(invalid_burst.validate().is_err());
    }

    #[test]
    fn test_update_rate_limit_config_to_config() {
        let req = UpdateRateLimitConfigRequest {
            tier: "premium".to_string(),
            requests_per_minute: 100,
            requests_per_hour: 5000,
            burst_size: 150,
        };

        let config = req.to_config("/api/v1/test".to_string()).unwrap();
        assert_eq!(config.endpoint, "/api/v1/test");
        assert_eq!(config.tier, UserTier::Premium);
        assert_eq!(config.requests_per_minute, 100);
        assert_eq!(config.requests_per_hour, 5000);
        assert_eq!(config.burst_size, 150);
    }

    #[test]
    fn test_rate_limit_config_response_from() {
        let config = RateLimitConfig::new("/api/v1/test".to_string(), UserTier::Free, 30, 1000, 50);

        let response: RateLimitConfigResponse = config.into();
        assert_eq!(response.endpoint, "/api/v1/test");
        assert_eq!(response.tier, "free");
        assert_eq!(response.requests_per_minute, 30);
        assert_eq!(response.requests_per_hour, 1000);
        assert_eq!(response.burst_size, 50);
    }
}
