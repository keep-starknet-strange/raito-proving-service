use axum::{
    extract::Request,
    http::{HeaderValue, Method},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tracing::info;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::any())
        .allow_credentials(false)
}

pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().path().to_string();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status().as_u16();

    info!(
        method = %method,
        uri = %uri,
        status = status,
        duration_ms = duration.as_millis(),
        "HTTP request processed"
    );

    response
}

pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}
