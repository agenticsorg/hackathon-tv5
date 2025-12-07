use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use super::service::CatalogService;
use super::types::{AvailabilityUpdate, CreateContentRequest, UpdateContentRequest};

pub struct CatalogState {
    pub catalog_service: Arc<CatalogService>,
    pub jwt_secret: String,
}

fn verify_admin(req: &HttpRequest, jwt_secret: &str) -> Result<(), actix_web::Error> {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
        role: Option<String>,
    }

    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Missing authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid authorization format"))?;

    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token"))?;

    if token_data.claims.role.as_deref() != Some("admin") {
        return Err(actix_web::error::ErrorForbidden("Admin access required"));
    }

    Ok(())
}

async fn create_content(
    req: HttpRequest,
    data: web::Data<CatalogState>,
    payload: web::Json<CreateContentRequest>,
) -> impl Responder {
    if let Err(e) = verify_admin(&req, &data.jwt_secret) {
        return e.into();
    }

    match data
        .catalog_service
        .create_content(payload.into_inner())
        .await
    {
        Ok(content) => HttpResponse::Created().json(content),
        Err(e) => {
            tracing::error!("Failed to create content: {}", e);
            HttpResponse::BadRequest().json(json!({
                "error": "Failed to create content",
                "message": e.to_string()
            }))
        }
    }
}

async fn get_content(
    req: HttpRequest,
    data: web::Data<CatalogState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = verify_admin(&req, &data.jwt_secret) {
        return e.into();
    }

    let content_id = path.into_inner();

    match data.catalog_service.get_content(content_id).await {
        Ok(Some(content)) => HttpResponse::Ok().json(content),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Content not found",
            "id": content_id
        })),
        Err(e) => {
            tracing::error!("Failed to get content: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to get content",
                "message": e.to_string()
            }))
        }
    }
}

async fn update_content(
    req: HttpRequest,
    data: web::Data<CatalogState>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateContentRequest>,
) -> impl Responder {
    if let Err(e) = verify_admin(&req, &data.jwt_secret) {
        return e.into();
    }

    let content_id = path.into_inner();

    match data
        .catalog_service
        .update_content(content_id, payload.into_inner())
        .await
    {
        Ok(content) => HttpResponse::Ok().json(content),
        Err(e) => {
            tracing::error!("Failed to update content: {}", e);
            HttpResponse::BadRequest().json(json!({
                "error": "Failed to update content",
                "message": e.to_string()
            }))
        }
    }
}

async fn delete_content(
    req: HttpRequest,
    data: web::Data<CatalogState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    if let Err(e) = verify_admin(&req, &data.jwt_secret) {
        return e.into();
    }

    let content_id = path.into_inner();

    match data.catalog_service.delete_content(content_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => {
            tracing::error!("Failed to delete content: {}", e);
            HttpResponse::BadRequest().json(json!({
                "error": "Failed to delete content",
                "message": e.to_string()
            }))
        }
    }
}

async fn update_availability(
    req: HttpRequest,
    data: web::Data<CatalogState>,
    path: web::Path<Uuid>,
    payload: web::Json<AvailabilityUpdate>,
) -> impl Responder {
    if let Err(e) = verify_admin(&req, &data.jwt_secret) {
        return e.into();
    }

    let content_id = path.into_inner();

    match data
        .catalog_service
        .update_availability(content_id, payload.into_inner())
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "Availability updated successfully"
        })),
        Err(e) => {
            tracing::error!("Failed to update availability: {}", e);
            HttpResponse::BadRequest().json(json!({
                "error": "Failed to update availability",
                "message": e.to_string()
            }))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/admin/catalog")
            .route("/content", web::post().to(create_content))
            .route("/content/{id}", web::get().to(get_content))
            .route("/content/{id}", web::patch().to(update_content))
            .route("/content/{id}", web::delete().to(delete_content))
            .route(
                "/content/{id}/availability",
                web::post().to(update_availability),
            ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_unauthorized_access() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(CatalogState {
                    catalog_service: Arc::new(CatalogService::new(
                        sqlx::PgPool::connect("postgresql://localhost/test")
                            .await
                            .unwrap_or_else(|_| panic!("test requires postgres")),
                        Arc::new(Qdrant::from_url("http://localhost:6334").build().unwrap()),
                        "test_collection".to_string(),
                        "test_key".to_string(),
                        "https://api.openai.com/v1/embeddings".to_string(),
                    )),
                    jwt_secret: "test_secret".to_string(),
                }))
                .configure(configure_routes),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/catalog/content")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }
}
