// API Key Management Handlers
// Add these handlers to crates/auth/src/server.rs

#[post("/api/v1/auth/api-keys")]
async fn create_api_key(
    req: actix_web::HttpRequest,
    body: web::Json<CreateApiKeyRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;

    let api_key = state
        .api_key_manager
        .create_api_key(Uuid::parse_str(&user_context.user_id).unwrap(), body.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(api_key))
}

#[get("/api/v1/auth/api-keys")]
async fn list_api_keys(
    req: actix_web::HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;

    let keys = state
        .api_key_manager
        .list_user_keys(Uuid::parse_str(&user_context.user_id).unwrap())
        .await?;

    Ok(HttpResponse::Ok().json(keys))
}

#[delete("/api/v1/auth/api-keys/{key_id}")]
async fn revoke_api_key(
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let user_context = extract_user_context(&req)?;
    let key_id = path.into_inner();

    state
        .api_key_manager
        .revoke_key(Uuid::parse_str(&user_context.user_id).unwrap(), key_id)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "API key revoked successfully"
    })))
}

// Update the start_server function to include the api_key_manager and register handlers
// Add to AppState initialization:
//     api_key_manager: Arc::new(ApiKeyManager::new(db_pool.clone())),

// Add to HttpServer::new App configuration:
//     .service(create_api_key)
//     .service(list_api_keys)
//     .service(revoke_api_key)
