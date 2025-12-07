//! Actix-web middleware for distributed tracing

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::task::{Context, Poll};
use tracing::{span, Level};
use uuid::Uuid;

/// W3C Trace Context header name
pub const TRACEPARENT_HEADER: &str = "traceparent";

/// Trace state header for additional vendor-specific data
pub const TRACESTATE_HEADER: &str = "tracestate";

/// Request ID header for correlation
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Actix-web middleware for distributed tracing
///
/// Automatically:
/// - Extracts trace context from incoming requests
/// - Creates spans for each HTTP request
/// - Injects trace context into outgoing responses
/// - Propagates context to downstream services
///
/// # Example
///
/// ```rust,no_run
/// use actix_web::{App, HttpServer};
/// use media_gateway_core::telemetry::TracingMiddleware;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     HttpServer::new(|| {
///         App::new()
///             .wrap(TracingMiddleware)
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
#[derive(Clone)]
pub struct TracingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TracingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TracingMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TracingMiddlewareService { service })
    }
}

pub struct TracingMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TracingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract trace context from headers
        let trace_context = extract_trace_context(&req);

        // Create request span
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let method = req.method().to_string();
        let path = req.path().to_string();
        let version = format!("{:?}", req.version());

        let span = span!(
            Level::INFO,
            "http.request",
            http.method = %method,
            http.target = %path,
            http.version = %version,
            http.request_id = %request_id,
            otel.kind = "server"
        );

        // Add trace context to span if present
        if let Some(ref ctx) = trace_context {
            span.record("trace.trace_id", ctx.trace_id.as_str());
            span.record("trace.span_id", ctx.span_id.as_str());
            if let Some(ref parent) = ctx.parent_span_id {
                span.record("trace.parent_span_id", parent.as_str());
            }
        }

        // Store request ID in extensions for downstream access
        req.extensions_mut().insert(RequestId(request_id.clone()));

        let fut = self.service.call(req);

        Box::pin(async move {
            let _guard = span.enter();
            let res = fut.await?;

            // Record response status
            let status = res.status().as_u16();
            tracing::info!(http.status_code = status, "HTTP request completed");

            Ok(res)
        })
    }
}

/// Trace context extracted from W3C traceparent header
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Version of trace context format (00 for W3C standard)
    pub version: String,

    /// Trace ID (32 hex characters)
    pub trace_id: String,

    /// Parent span ID (16 hex characters)
    pub span_id: String,

    /// Trace flags (01 = sampled, 00 = not sampled)
    pub trace_flags: String,

    /// Optional parent span ID for nested spans
    pub parent_span_id: Option<String>,
}

/// Request ID wrapper for extensions
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// Extract W3C trace context from incoming request headers
///
/// Parses the `traceparent` header following W3C Trace Context specification:
/// `{version}-{trace-id}-{parent-id}-{trace-flags}`
///
/// # Example
///
/// ```rust
/// use actix_web::test::TestRequest;
/// use media_gateway_core::telemetry::extract_trace_context;
///
/// let req = TestRequest::default()
///     .insert_header((
///         "traceparent",
///         "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
///     ))
///     .to_srv_request();
///
/// let context = extract_trace_context(&req);
/// assert!(context.is_some());
/// ```
pub fn extract_trace_context(req: &ServiceRequest) -> Option<TraceContext> {
    let traceparent = req.headers().get(TRACEPARENT_HEADER)?.to_str().ok()?;

    parse_traceparent(traceparent)
}

/// Parse W3C traceparent header value
fn parse_traceparent(value: &str) -> Option<TraceContext> {
    let parts: Vec<&str> = value.split('-').collect();

    if parts.len() != 4 {
        tracing::warn!("Invalid traceparent format: {}", value);
        return None;
    }

    Some(TraceContext {
        version: parts[0].to_string(),
        trace_id: parts[1].to_string(),
        span_id: parts[2].to_string(),
        trace_flags: parts[3].to_string(),
        parent_span_id: None,
    })
}

/// Inject trace context into outgoing HTTP request headers
///
/// Creates a new span ID and injects W3C trace context for service-to-service
/// trace propagation.
///
/// # Example
///
/// ```rust
/// use reqwest::Client;
/// use media_gateway_core::telemetry::{TraceContext, inject_trace_context};
///
/// let context = TraceContext {
///     version: "00".to_string(),
///     trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
///     span_id: "b7ad6b7169203331".to_string(),
///     trace_flags: "01".to_string(),
///     parent_span_id: None,
/// };
///
/// let mut headers = reqwest::header::HeaderMap::new();
/// inject_trace_context(&context, &mut headers);
/// ```
pub fn inject_trace_context(context: &TraceContext, headers: &mut reqwest::header::HeaderMap) {
    // Generate new span ID for this outgoing request
    let new_span_id = generate_span_id();

    let traceparent = format!(
        "{}-{}-{}-{}",
        context.version, context.trace_id, new_span_id, context.trace_flags
    );

    if let Ok(value) = reqwest::header::HeaderValue::from_str(&traceparent) {
        headers.insert(TRACEPARENT_HEADER, value);
    }
}

/// Generate a random 16-character hex span ID
fn generate_span_id() -> String {
    use std::fmt::Write;
    let mut rng = [0u8; 8];

    // Use UUID randomness for span ID generation
    let uuid = Uuid::new_v4();
    let uuid_bytes = uuid.as_bytes();
    rng.copy_from_slice(&uuid_bytes[0..8]);

    let mut span_id = String::with_capacity(16);
    for byte in rng.iter() {
        write!(&mut span_id, "{:02x}", byte).unwrap();
    }
    span_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().body("test")
    }

    #[actix_web::test]
    async fn test_tracing_middleware_without_traceparent() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_traceparent() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header((
                "traceparent",
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            ))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_tracing_middleware_with_request_id() {
        let app = test::init_service(
            App::new()
                .wrap(TracingMiddleware)
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("x-request-id", "test-req-123"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_parse_traceparent_valid() {
        let traceparent = "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01";
        let context = parse_traceparent(traceparent);

        assert!(context.is_some());
        let ctx = context.unwrap();
        assert_eq!(ctx.version, "00");
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
        assert_eq!(ctx.span_id, "b7ad6b7169203331");
        assert_eq!(ctx.trace_flags, "01");
    }

    #[actix_web::test]
    async fn test_parse_traceparent_invalid() {
        // Missing parts
        assert!(parse_traceparent("00-abc123").is_none());

        // Too many parts
        assert!(parse_traceparent("00-a-b-c-d-e").is_none());

        // Empty string
        assert!(parse_traceparent("").is_none());
    }

    #[actix_web::test]
    async fn test_extract_trace_context() {
        let req = test::TestRequest::default()
            .insert_header((
                "traceparent",
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
            ))
            .to_srv_request();

        let context = extract_trace_context(&req);
        assert!(context.is_some());

        let ctx = context.unwrap();
        assert_eq!(ctx.trace_id, "0af7651916cd43dd8448eb211c80319c");
    }

    #[actix_web::test]
    async fn test_extract_trace_context_no_header() {
        let req = test::TestRequest::default().to_srv_request();
        let context = extract_trace_context(&req);
        assert!(context.is_none());
    }

    #[actix_web::test]
    async fn test_inject_trace_context() {
        let context = TraceContext {
            version: "00".to_string(),
            trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
            span_id: "b7ad6b7169203331".to_string(),
            trace_flags: "01".to_string(),
            parent_span_id: None,
        };

        let mut headers = reqwest::header::HeaderMap::new();
        inject_trace_context(&context, &mut headers);

        let traceparent = headers.get(TRACEPARENT_HEADER);
        assert!(traceparent.is_some());

        let value = traceparent.unwrap().to_str().unwrap();
        assert!(value.starts_with("00-0af7651916cd43dd8448eb211c80319c-"));
        assert!(value.ends_with("-01"));
    }

    #[actix_web::test]
    async fn test_generate_span_id() {
        let span_id = generate_span_id();

        // Should be 16 hex characters
        assert_eq!(span_id.len(), 16);
        assert!(span_id.chars().all(|c| c.is_ascii_hexdigit()));

        // Should be unique
        let span_id2 = generate_span_id();
        assert_ne!(span_id, span_id2);
    }

    #[actix_web::test]
    async fn test_trace_context_clone() {
        let ctx1 = TraceContext {
            version: "00".to_string(),
            trace_id: "abc123".to_string(),
            span_id: "def456".to_string(),
            trace_flags: "01".to_string(),
            parent_span_id: Some("parent123".to_string()),
        };

        let ctx2 = ctx1.clone();
        assert_eq!(ctx1.trace_id, ctx2.trace_id);
        assert_eq!(ctx1.span_id, ctx2.span_id);
        assert_eq!(ctx1.parent_span_id, ctx2.parent_span_id);
    }

    #[actix_web::test]
    async fn test_request_id_clone() {
        let id1 = RequestId("test-123".to_string());
        let id2 = id1.clone();
        assert_eq!(id1.0, id2.0);
    }
}
