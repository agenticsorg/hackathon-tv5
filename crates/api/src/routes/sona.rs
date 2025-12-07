use crate::proxy::{ProxyRequest, ServiceProxy};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sona")
            .route("/recommendations", web::post().to(get_recommendations))
            .route(
                "/personalization/score",
                web::post().to(score_personalization),
            )
            .route(
                "/experiments/{id}/metrics",
                web::get().to(get_experiment_metrics),
            ),
    );
}

async fn get_recommendations(
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
        service: "sona".to_string(),
        path: "/recommendations".to_string(),
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

async fn score_personalization(
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
        service: "sona".to_string(),
        path: "/personalization/score".to_string(),
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

async fn get_experiment_metrics(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let experiment_id = path.into_inner();

    // Convert actix-web headers to reqwest headers
    let mut headers = reqwest::header::HeaderMap::new();
    for (key, value) in req.headers() {
        if let Ok(value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            headers.insert(key.clone(), value);
        }
    }

    let proxy_req = ProxyRequest {
        service: "sona".to_string(),
        path: format!("/experiments/{}/metrics", experiment_id),
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
