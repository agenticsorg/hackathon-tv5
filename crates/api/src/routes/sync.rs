use crate::proxy::{ProxyRequest, ServiceProxy};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sync")
            .route("/watchlist", web::post().to(sync_watchlist))
            .route("/progress", web::post().to(sync_progress))
            .route("/devices", web::get().to(list_devices))
            .route("/devices/handoff", web::post().to(device_handoff)),
    );
}

async fn sync_watchlist(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    // Convert actix-web headers to reqwest headers
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in req.headers() {
        if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            headers.insert(key.clone(), value);
        }
    }

    let proxy_req = ProxyRequest {
        service: "sync".to_string(),
        path: "/api/v1/sync/watchlist".to_string(),
        method: req.method().clone(),
        headers,
        body: Some(body),
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status).body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn sync_progress(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    // Convert actix-web headers to reqwest headers
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in req.headers() {
        if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            headers.insert(key.clone(), value);
        }
    }

    let proxy_req = ProxyRequest {
        service: "sync".to_string(),
        path: "/api/v1/sync/progress".to_string(),
        method: req.method().clone(),
        headers,
        body: Some(body),
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status).body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn list_devices(req: HttpRequest, proxy: web::Data<Arc<ServiceProxy>>) -> impl Responder {
    // Convert actix-web headers to reqwest headers
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in req.headers() {
        if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            headers.insert(key.clone(), value);
        }
    }

    let proxy_req = ProxyRequest {
        service: "sync".to_string(),
        path: "/api/v1/devices".to_string(),
        method: req.method().clone(),
        headers,
        body: None,
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status).body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn device_handoff(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    // Convert actix-web headers to reqwest headers
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in req.headers() {
        if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            headers.insert(key.clone(), value);
        }
    }

    let proxy_req = ProxyRequest {
        service: "sync".to_string(),
        path: "/api/v1/devices/handoff".to_string(),
        method: req.method().clone(),
        headers,
        body: Some(body),
        query: req.uri().query().map(String::from),
    };

    match proxy.forward(proxy_req).await {
        Ok(response) => HttpResponse::build(response.status).body(response.body),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
