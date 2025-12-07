use crate::middleware::auth::{get_user_context, AuthMiddleware};
use crate::middleware::request_id::get_request_id;
use crate::proxy::{ProxyRequest, ServiceProxy};
use crate::rate_limit::RateLimiter;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;
use tracing::debug;

fn convert_headers(
    actix_headers: &actix_web::http::header::HeaderMap,
) -> reqwest::header::HeaderMap {
    let mut reqwest_headers = reqwest::header::HeaderMap::new();
    for (key, value) in actix_headers.iter() {
        if let Ok(name) = reqwest::header::HeaderName::from_bytes(key.as_str().as_bytes()) {
            if let Ok(val) = reqwest::header::HeaderValue::from_bytes(value.as_bytes()) {
                reqwest_headers.insert(name, val);
            }
        }
    }
    reqwest_headers
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/content")
            .wrap(AuthMiddleware::optional())
            .route("/{id}", web::get().to(get_content))
            .route("/{id}/availability", web::get().to(get_availability))
            .route("/trending", web::get().to(get_trending)),
    )
    .service(
        web::scope("/movies")
            .wrap(AuthMiddleware::optional())
            .route("/popular", web::get().to(get_popular_movies)),
    )
    .service(
        web::scope("/tv")
            .wrap(AuthMiddleware::optional())
            .route("/popular", web::get().to(get_popular_tv)),
    );
}

async fn get_content(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let content_id = path.into_inner();
    let request_id = get_request_id(&req).unwrap_or_default();

    debug!(
        request_id = %request_id,
        content_id = %content_id,
        "Get content request"
    );

    // Check rate limit
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            // Forward to discovery service
            let proxy_req = ProxyRequest {
                service: "discovery".to_string(),
                path: format!("/api/v1/content/{}", content_id),
                method: req.method().clone(),
                headers: convert_headers(req.headers()),
                body: None,
                query: req
                    .query_string()
                    .is_empty()
                    .then(|| req.query_string().to_string()),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);

                    // Add rate limit headers
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    // Forward response headers
                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_availability(
    req: HttpRequest,
    path: web::Path<String>,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    let content_id = path.into_inner();

    // Check rate limit
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "discovery".to_string(),
                path: format!("/api/v1/content/{}/availability", content_id),
                method: req.method().clone(),
                headers: convert_headers(req.headers()),
                body: None,
                query: req
                    .query_string()
                    .is_empty()
                    .then(|| req.query_string().to_string()),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_trending(
    req: HttpRequest,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    // Check rate limit
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "discovery".to_string(),
                path: "/api/v1/content/trending".to_string(),
                method: req.method().clone(),
                headers: convert_headers(req.headers()),
                body: None,
                query: req
                    .query_string()
                    .is_empty()
                    .then(|| req.query_string().to_string()),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_popular_movies(
    req: HttpRequest,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    // Check rate limit
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "discovery".to_string(),
                path: "/api/v1/movies/popular".to_string(),
                method: req.method().clone(),
                headers: convert_headers(req.headers()),
                body: None,
                query: req
                    .query_string()
                    .is_empty()
                    .then(|| req.query_string().to_string()),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}

async fn get_popular_tv(
    req: HttpRequest,
    proxy: web::Data<Arc<ServiceProxy>>,
    rate_limiter: web::Data<Arc<RateLimiter>>,
) -> impl Responder {
    // Check rate limit
    let user_ctx = get_user_context(&req);
    let user_id = user_ctx
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    let tier = user_ctx
        .as_ref()
        .map(|u| u.tier.as_str())
        .unwrap_or("anonymous");

    match rate_limiter.check_rate_limit(user_id, tier).await {
        Ok(rate_info) => {
            let proxy_req = ProxyRequest {
                service: "discovery".to_string(),
                path: "/api/v1/tv/popular".to_string(),
                method: req.method().clone(),
                headers: convert_headers(req.headers()),
                body: None,
                query: req
                    .query_string()
                    .is_empty()
                    .then(|| req.query_string().to_string()),
            };

            match proxy.forward(proxy_req).await {
                Ok(response) => {
                    let mut http_response = HttpResponse::build(response.status);
                    http_response.insert_header(("X-RateLimit-Limit", rate_info.limit.to_string()));
                    http_response
                        .insert_header(("X-RateLimit-Remaining", rate_info.remaining.to_string()));
                    http_response.insert_header(("X-RateLimit-Reset", rate_info.reset.to_string()));

                    for (key, value) in response.headers.iter() {
                        http_response.insert_header((key.clone(), value.clone()));
                    }

                    http_response.body(response.body)
                }
                Err(err) => HttpResponse::from_error(err),
            }
        }
        Err(err) => HttpResponse::from_error(err),
    }
}
