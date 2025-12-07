use actix_web::{patch, post, web, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use uuid::Uuid;

use crate::error::{AuthError, Result};
use crate::middleware::extract_user_context;
use crate::parental::controls::{
    get_parental_controls, set_parental_controls, ParentalControlsPublic,
    SetParentalControlsRequest, SetParentalControlsResponse,
};
use crate::parental::verification::{verify_pin, VerifyPinRequest};

/// Handler state for parental controls
pub struct ParentalControlsState {
    pub db_pool: sqlx::PgPool,
    pub redis_client: redis::Client,
    pub jwt_secret: String,
}

/// Get current user's parental controls
#[derive(Debug, Serialize)]
pub struct GetParentalControlsResponse {
    pub parental_controls: Option<ParentalControlsPublic>,
}

pub async fn get_user_parental_controls(
    pool: &sqlx::PgPool,
    user_id: Uuid,
) -> Result<GetParentalControlsResponse> {
    let controls = get_parental_controls(pool, user_id).await?;

    Ok(GetParentalControlsResponse {
        parental_controls: controls.map(|c| c.into()),
    })
}

/// PATCH /api/v1/users/me/parental-controls
#[patch("/api/v1/users/me/parental-controls")]
pub async fn update_parental_controls(
    req: HttpRequest,
    body: web::Json<SetParentalControlsRequest>,
    state: web::Data<ParentalControlsState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;
    let user_id = Uuid::parse_str(&user_context.user_id)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    let controls = set_parental_controls(&state.db_pool, user_id, body.into_inner()).await?;

    Ok(HttpResponse::Ok().json(SetParentalControlsResponse {
        success: true,
        parental_controls: controls.into(),
    }))
}

/// POST /api/v1/users/me/parental-controls/verify-pin
#[post("/api/v1/users/me/parental-controls/verify-pin")]
pub async fn verify_parental_pin(
    req: HttpRequest,
    body: web::Json<VerifyPinRequest>,
    state: web::Data<ParentalControlsState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;
    let user_id = Uuid::parse_str(&user_context.user_id)
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    let response = verify_pin(
        &state.db_pool,
        &state.redis_client,
        user_id,
        body.into_inner(),
        &state.jwt_secret,
    )
    .await?;

    if response.verified {
        Ok(HttpResponse::Ok().json(response))
    } else {
        Ok(HttpResponse::Unauthorized().json(response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_parental_controls_response_serialization() {
        let response = GetParentalControlsResponse {
            parental_controls: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("parental_controls"));
    }
}
