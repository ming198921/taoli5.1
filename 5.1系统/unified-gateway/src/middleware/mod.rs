use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::info;

pub mod auth;
pub mod rate_limit;
pub mod metrics;

pub use auth::AuthLayer;
pub use rate_limit::RateLimitLayer;
pub use metrics::MetricsLayer;

pub async fn logging_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed();
    
    info!(
        "{} {} - {} - {:?}",
        method,
        uri,
        response.status(),
        duration
    );
    
    Ok(response)
}