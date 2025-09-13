use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::config::RateLimitConfig;

#[derive(Clone)]
pub struct RateLimitLayer {
    config: RateLimitConfig,
    limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            config: self.config.clone(),
            limiters: self.limiters.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    config: RateLimitConfig,
    limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
}

struct RateLimiter {
    tokens: u32,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(initial_tokens: u32) -> Self {
        Self {
            tokens: initial_tokens,
            last_refill: Instant::now(),
        }
    }

    fn try_consume(&mut self, tokens_per_minute: u32, burst_size: u32) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        
        // 补充令牌
        let tokens_to_add = (elapsed.as_secs_f64() / 60.0 * tokens_per_minute as f64) as u32;
        self.tokens = (self.tokens + tokens_to_add).min(burst_size);
        self.last_refill = now;
        
        // 尝试消费令牌
        if self.tokens > 0 {
            self.tokens -= 1;
            true
        } else {
            false
        }
    }
}

impl<S> Service<Request<Body>> for RateLimitMiddleware<S>
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
        let config = self.config.clone();
        let limiters = self.limiters.clone();
        let future = self.inner.call(req);
        
        Box::pin(async move {
            // 如果限流未启用，直接通过
            if !config.enabled {
                return future.await;
            }
            
            // 获取客户端标识（这里简化使用IP）
            let client_id = "default".to_string(); // TODO: 从请求中提取真实IP
            
            // 检查限流
            let mut limiters = limiters.write().await;
            let limiter = limiters.entry(client_id)
                .or_insert_with(|| RateLimiter::new(config.burst_size));
            
            if !limiter.try_consume(config.requests_per_minute, config.burst_size) {
                // 超过限流，返回429
                return Ok(Response::builder()
                    .status(StatusCode::TOO_MANY_REQUESTS)
                    .body(Body::from("Rate limit exceeded"))
                    .unwrap());
            }
            
            drop(limiters);
            future.await
        })
    }
}