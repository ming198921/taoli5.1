use axum::{
    body::Body,
    http::Request,
    response::Response,
};
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

#[derive(Clone)]
pub struct MetricsLayer;

impl MetricsLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct MetricsMiddleware<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for MetricsMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let start = Instant::now();
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        
        let future = self.inner.call(req);
        
        Box::pin(async move {
            let response = future.await;
            let duration = start.elapsed();
            
            // TODO: 记录指标到Prometheus
            tracing::debug!(
                "Request {} {} completed in {:?}",
                method,
                path,
                duration
            );
            
            response
        })
    }
}