use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::search::ranking::{
    NamedRankingConfig, RankingConfig, RankingConfigStore, UpdateRankingConfigRequest,
};

/// Extract admin user ID from JWT token
fn extract_admin_user_id(req: &HttpRequest) -> Result<Uuid, HttpResponse> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Missing Authorization header".to_string(),
            })
        })?
        .to_str()
        .map_err(|_| {
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Invalid Authorization header".to_string(),
            })
        })?;

    if !auth_header.starts_with("Bearer ") {
        return Err(HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid Authorization format".to_string(),
        }));
    }

    let token = &auth_header[7..];

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());

    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| {
        warn!(error = %e, "Failed to decode JWT token");
        HttpResponse::Unauthorized().json(ErrorResponse {
            error: "Invalid or expired token".to_string(),
        })
    })?;

    if !token_data.claims.roles.contains(&"admin".to_string()) {
        return Err(HttpResponse::Forbidden().json(ErrorResponse {
            error: "Admin role required".to_string(),
        }));
    }

    Uuid::parse_str(&token_data.claims.sub).map_err(|_| {
        HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Invalid user ID in token".to_string(),
        })
    })
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    roles: Vec<String>,
    exp: usize,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Serialize)]
struct SuccessResponse {
    message: String,
}

/// GET /api/v1/admin/search/ranking - Get current default ranking config
pub async fn get_ranking_config(
    store: web::Data<Arc<RankingConfigStore>>,
    req: HttpRequest,
) -> impl Responder {
    let admin_id = match extract_admin_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    info!(admin_id = %admin_id, "Admin requested ranking config");

    match store.get_default_config().await {
        Ok(config) => HttpResponse::Ok().json(config),
        Err(e) => {
            error!(error = %e, "Failed to get ranking config");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get ranking config: {}", e),
            })
        }
    }
}

/// PUT /api/v1/admin/search/ranking - Update default ranking config
pub async fn update_ranking_config(
    store: web::Data<Arc<RankingConfigStore>>,
    req: HttpRequest,
    body: web::Json<UpdateRankingConfigRequest>,
) -> impl Responder {
    let admin_id = match extract_admin_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    if let Err(e) = body.validate() {
        warn!(error = %e, "Invalid ranking config weights");
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Invalid weights: {}", e),
        });
    }

    let config = match RankingConfig::new(
        body.vector_weight,
        body.keyword_weight,
        body.quality_weight,
        body.freshness_weight,
        Some(admin_id),
        body.description.clone(),
    ) {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "Failed to create ranking config");
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid config: {}", e),
            });
        }
    };

    match store.set_default_config(&config, Some(admin_id)).await {
        Ok(_) => {
            info!(
                admin_id = %admin_id,
                version = config.version,
                "Updated ranking config"
            );
            HttpResponse::Ok().json(config)
        }
        Err(e) => {
            error!(error = %e, "Failed to update ranking config");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to update ranking config: {}", e),
            })
        }
    }
}

/// Request to create/update named ranking config
#[derive(Debug, Deserialize)]
pub struct CreateNamedRankingConfigRequest {
    pub vector_weight: f64,
    pub keyword_weight: f64,
    pub quality_weight: f64,
    pub freshness_weight: f64,
    pub description: Option<String>,
    pub is_active: bool,
    pub traffic_percentage: Option<u8>,
}

/// GET /api/v1/admin/search/ranking/variants - List all named configs
pub async fn list_ranking_variants(
    store: web::Data<Arc<RankingConfigStore>>,
    req: HttpRequest,
) -> impl Responder {
    let admin_id = match extract_admin_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    info!(admin_id = %admin_id, "Admin requested ranking variants list");

    match store.list_named_configs().await {
        Ok(configs) => HttpResponse::Ok().json(configs),
        Err(e) => {
            error!(error = %e, "Failed to list ranking variants");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to list ranking variants: {}", e),
            })
        }
    }
}

/// GET /api/v1/admin/search/ranking/variants/{name} - Get specific named config
pub async fn get_ranking_variant(
    store: web::Data<Arc<RankingConfigStore>>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let admin_id = match extract_admin_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let name = path.into_inner();
    info!(admin_id = %admin_id, variant_name = %name, "Admin requested ranking variant");

    match store.get_named_config(&name).await {
        Ok(Some(config)) => HttpResponse::Ok().json(config),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: format!("Ranking variant '{}' not found", name),
        }),
        Err(e) => {
            error!(error = %e, "Failed to get ranking variant");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get ranking variant: {}", e),
            })
        }
    }
}

/// PUT /api/v1/admin/search/ranking/variants/{name} - Create/update named config
pub async fn update_ranking_variant(
    store: web::Data<Arc<RankingConfigStore>>,
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<CreateNamedRankingConfigRequest>,
) -> impl Responder {
    let admin_id = match extract_admin_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let name = path.into_inner();

    let config = match RankingConfig::new(
        body.vector_weight,
        body.keyword_weight,
        body.quality_weight,
        body.freshness_weight,
        Some(admin_id),
        body.description.clone(),
    ) {
        Ok(c) => c,
        Err(e) => {
            warn!(error = %e, "Failed to create ranking config");
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid config: {}", e),
            });
        }
    };

    match store
        .set_named_config(
            &name,
            &config,
            body.is_active,
            body.traffic_percentage,
            Some(admin_id),
        )
        .await
    {
        Ok(_) => {
            info!(
                admin_id = %admin_id,
                variant_name = %name,
                is_active = body.is_active,
                "Updated ranking variant"
            );
            HttpResponse::Ok().json(NamedRankingConfig {
                name: name.clone(),
                config,
                is_active: body.is_active,
                traffic_percentage: body.traffic_percentage,
            })
        }
        Err(e) => {
            error!(error = %e, "Failed to update ranking variant");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to update ranking variant: {}", e),
            })
        }
    }
}

/// DELETE /api/v1/admin/search/ranking/variants/{name} - Delete named config
pub async fn delete_ranking_variant(
    store: web::Data<Arc<RankingConfigStore>>,
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let admin_id = match extract_admin_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let name = path.into_inner();

    match store.delete_named_config(&name, Some(admin_id)).await {
        Ok(true) => {
            info!(
                admin_id = %admin_id,
                variant_name = %name,
                "Deleted ranking variant"
            );
            HttpResponse::Ok().json(SuccessResponse {
                message: format!("Ranking variant '{}' deleted successfully", name),
            })
        }
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse {
            error: format!("Ranking variant '{}' not found", name),
        }),
        Err(e) => {
            error!(error = %e, "Failed to delete ranking variant");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to delete ranking variant: {}", e),
            })
        }
    }
}

/// GET /api/v1/admin/search/ranking/history/{version} - Get config version history
pub async fn get_ranking_config_history(
    store: web::Data<Arc<RankingConfigStore>>,
    req: HttpRequest,
    path: web::Path<u32>,
) -> impl Responder {
    let admin_id = match extract_admin_user_id(&req) {
        Ok(id) => id,
        Err(response) => return response,
    };

    let version = path.into_inner();
    info!(admin_id = %admin_id, version = version, "Admin requested ranking config history");

    match store.get_config_history(version).await {
        Ok(Some(config)) => HttpResponse::Ok().json(config),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: format!("Ranking config version {} not found", version),
        }),
        Err(e) => {
            error!(error = %e, "Failed to get ranking config history");
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to get ranking config history: {}", e),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_extract_admin_user_id_missing_header() {
        let req = test::TestRequest::default().to_http_request();
        let result = extract_admin_user_id(&req);
        assert!(result.is_err());
    }

    #[actix_web::test]
    async fn test_extract_admin_user_id_invalid_format() {
        let req = test::TestRequest::default()
            .insert_header(("Authorization", "InvalidToken"))
            .to_http_request();
        let result = extract_admin_user_id(&req);
        assert!(result.is_err());
    }
}
