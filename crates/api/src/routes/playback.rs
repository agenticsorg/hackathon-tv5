use crate::proxy::{ProxyRequest, ServiceProxy};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

/// Convert actix_http HeaderMap to reqwest HeaderMap
fn convert_headers(
    actix_headers: &actix_web::http::header::HeaderMap,
) -> reqwest::header::HeaderMap {
    let mut reqwest_headers = reqwest::header::HeaderMap::new();

    for (key, value) in actix_headers.iter() {
        if let Ok(header_value) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
            reqwest_headers.insert(key.clone(), header_value);
        }
    }

    reqwest_headers
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/playback")
            .route("/sessions", web::post().to(create_session))
            .route("/sessions/{id}", web::get().to(get_session))
            .route("/sessions/{id}", web::delete().to(delete_session))
            .route("/sessions/{id}/position", web::patch().to(update_position))
            .route(
                "/users/{user_id}/sessions",
                web::get().to(get_user_sessions),
            ),
    );
}

async fn create_session(
    req: HttpRequest,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: "/api/v1/sessions".to_string(),
        method: req.method().clone(),
        headers: convert_headers(req.headers()),
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

async fn get_session(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let session_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/sessions/{}", session_id),
        method: req.method().clone(),
        headers: convert_headers(req.headers()),
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

async fn delete_session(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let session_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/sessions/{}", session_id),
        method: req.method().clone(),
        headers: convert_headers(req.headers()),
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

async fn update_position(
    req: HttpRequest,
    path: web::Path<String>,
    body: web::Bytes,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let session_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/sessions/{}/position", session_id),
        method: req.method().clone(),
        headers: convert_headers(req.headers()),
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

async fn get_user_sessions(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
) -> impl Responder {
    let user_id = path.into_inner();
    let proxy_req = ProxyRequest {
        service: "playback".to_string(),
        path: format!("/api/v1/users/{}/sessions", user_id),
        method: req.method().clone(),
        headers: convert_headers(req.headers()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{HeaderMap as ActixHeaderMap, HeaderName, HeaderValue};

    #[test]
    fn test_convert_headers() {
        let mut actix_headers = ActixHeaderMap::new();
        actix_headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        );
        actix_headers.insert(
            HeaderName::from_static("authorization"),
            HeaderValue::from_static("Bearer token123"),
        );

        let reqwest_headers = convert_headers(&actix_headers);

        assert_eq!(reqwest_headers.len(), 2);
        assert_eq!(
            reqwest_headers.get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(
            reqwest_headers.get("authorization").unwrap(),
            "Bearer token123"
        );
    }

    #[test]
    fn test_convert_headers_empty() {
        let actix_headers = ActixHeaderMap::new();
        let reqwest_headers = convert_headers(&actix_headers);
        assert_eq!(reqwest_headers.len(), 0);
    }

    #[test]
    fn test_convert_headers_multiple_values() {
        let mut actix_headers = ActixHeaderMap::new();
        actix_headers.insert(
            HeaderName::from_static("x-custom-header"),
            HeaderValue::from_static("value1"),
        );
        actix_headers.insert(
            HeaderName::from_static("x-another-header"),
            HeaderValue::from_static("value2"),
        );

        let reqwest_headers = convert_headers(&actix_headers);

        assert_eq!(reqwest_headers.len(), 2);
        assert!(reqwest_headers.contains_key("x-custom-header"));
        assert!(reqwest_headers.contains_key("x-another-header"));
    }
}
