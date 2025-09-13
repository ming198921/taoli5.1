//! 追踪中间件
//!
//! 提供自动trace_id注入的中间件组件

use anyhow::Result;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{info_span, Instrument};
use uuid::Uuid;

use super::cross_service_propagation::{CrossServicePropagator, PropagationContext};

/// 追踪中间件层
#[derive(Clone)]
pub struct TraceLayer {
    propagator: Arc<CrossServicePropagator>,
    service_name: String,
}

impl TraceLayer {
    pub fn new(propagator: Arc<CrossServicePropagator>, service_name: String) -> Self {
        Self {
            propagator,
            service_name,
        }
    }
}

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceService {
            inner,
            propagator: Arc::clone(&self.propagator),
            service_name: self.service_name.clone(),
        }
    }
}

/// 追踪服务
#[derive(Clone)]
pub struct TraceService<S> {
    inner: S,
    propagator: Arc<CrossServicePropagator>,
    service_name: String,
}

/// HTTP请求类型（简化版）
pub struct HttpRequest {
    pub headers: HashMap<String, String>,
    pub method: String,
    pub uri: String,
    pub body: Vec<u8>,
}

/// HTTP响应类型（简化版）
pub struct HttpResponse {
    pub headers: HashMap<String, String>,
    pub status: u16,
    pub body: Vec<u8>,
}

impl<S> Service<HttpRequest> for TraceService<S>
where
    S: Service<HttpRequest, Response = HttpResponse> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = HttpResponse;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: HttpRequest) -> Self::Future {
        let propagator = Arc::clone(&self.propagator);
        let service_name = self.service_name.clone();
        let mut inner = self.inner.clone();
        
        Box::pin(async move {
            // 提取或创建追踪上下文
            let operation_name = format!("{} {}", req.method, req.uri);
            let context = match propagator
                .extract_from_http_headers(&req.headers, &service_name, &operation_name)
                .await
            {
                Ok(Some(ctx)) => ctx,
                Ok(None) => propagator.create_root_context(&service_name, &operation_name).await,
                Err(e) => {
                    tracing::warn!("Failed to extract trace context: {}", e);
                    propagator.create_root_context(&service_name, &operation_name).await
                }
            };

            // 创建tracing span
            let span = info_span!(
                "http_request",
                trace_id = %context.trace_id,
                span_id = %context.current_span_id,
                parent_span_id = context.parent_span_id.as_deref().unwrap_or(""),
                service = %context.service_name,
                operation = %context.operation_name,
                http.method = %req.method,
                http.uri = %req.uri,
                depth = context.depth
            );

            async move {
                // 注入追踪信息到请求头
                if let Err(e) = propagator.inject_http_headers(&mut req.headers, &context).await {
                    tracing::warn!("Failed to inject trace headers: {}", e);
                }

                // 调用内部服务
                let mut response = inner.call(req).await?;

                // 注入追踪信息到响应头
                if let Err(e) = propagator.inject_http_headers(&mut response.headers, &context).await {
                    tracing::warn!("Failed to inject response trace headers: {}", e);
                }

                Ok(response)
            }
            .instrument(span)
            .await
        })
    }
}

/// NATS消息中间件
pub struct NatsTraceMiddleware {
    propagator: Arc<CrossServicePropagator>,
    service_name: String,
}

impl NatsTraceMiddleware {
    pub fn new(propagator: Arc<CrossServicePropagator>, service_name: String) -> Self {
        Self {
            propagator,
            service_name,
        }
    }

    /// 处理NATS消息发布
    pub async fn handle_publish(
        &self,
        subject: &str,
        message: &mut NatsMessage,
        operation: &str,
    ) -> Result<PropagationContext> {
        // 创建或使用现有的追踪上下文
        let context = if let Some(current_context) = get_current_trace_context().await {
            self.propagator
                .create_child_context(&current_context, operation)
                .await
        } else {
            self.propagator
                .create_root_context(&self.service_name, operation)
                .await
        };

        // 注入追踪信息到消息头
        self.propagator
            .inject_nats_message(&mut message.headers, &context)
            .await?;

        // 添加NATS特定信息
        message.headers.insert("nats.subject".to_string(), subject.to_string());
        message.headers.insert("nats.publisher".to_string(), self.service_name.clone());

        tracing::info!(
            trace_id = %context.trace_id,
            span_id = %context.current_span_id,
            subject = %subject,
            "Published NATS message with trace context"
        );

        Ok(context)
    }

    /// 处理NATS消息订阅
    pub async fn handle_subscription<F, Fut>(
        &self,
        subject: &str,
        message: &NatsMessage,
        handler: F,
    ) -> Result<()>
    where
        F: FnOnce(PropagationContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send,
    {
        let operation = format!("nats_subscription:{}", subject);
        
        // 提取或创建追踪上下文
        let context = match self
            .propagator
            .extract_from_nats_message(&message.headers, &self.service_name, &operation)
            .await
        {
            Ok(Some(ctx)) => ctx,
            Ok(None) => self.propagator.create_root_context(&self.service_name, &operation).await,
            Err(e) => {
                tracing::warn!("Failed to extract NATS trace context: {}", e);
                self.propagator.create_root_context(&self.service_name, &operation).await
            }
        };

        // 创建tracing span
        let span = info_span!(
            "nats_message_handler",
            trace_id = %context.trace_id,
            span_id = %context.current_span_id,
            parent_span_id = context.parent_span_id.as_deref().unwrap_or(""),
            service = %context.service_name,
            nats.subject = %subject,
            depth = context.depth
        );

        // 在span上下文中执行处理器
        handler(context).instrument(span).await
    }
}

/// NATS消息结构（简化版）
pub struct NatsMessage {
    pub headers: HashMap<String, String>,
    pub data: Vec<u8>,
    pub reply: Option<String>,
}

/// 获取当前线程的追踪上下文（模拟实现）
async fn get_current_trace_context() -> Option<PropagationContext> {
    // 这里应该从当前的tracing span中提取上下文
    // 实际实现中会使用tracing的当前span信息
    None
}

/// 异步任务追踪包装器
pub struct TracedTask<F> {
    future: F,
    context: PropagationContext,
}

impl<F> TracedTask<F> {
    pub fn new(future: F, context: PropagationContext) -> Self {
        Self { future, context }
    }
}

impl<F> Future for TracedTask<F>
where
    F: Future + Send,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let context = &self.context;
        let span = info_span!(
            "traced_task",
            trace_id = %context.trace_id,
            span_id = %context.current_span_id,
            parent_span_id = context.parent_span_id.as_deref().unwrap_or(""),
            service = %context.service_name,
            operation = %context.operation_name
        );

        let _enter = span.enter();
        
        // SAFETY: 我们只是转发poll调用，不移动future
        let future = unsafe { self.as_mut().map_unchecked_mut(|s| &mut s.future) };
        future.poll(cx)
    }
}

/// 自动追踪任务生成宏
#[macro_export]
macro_rules! spawn_traced {
    ($propagator:expr, $service:expr, $operation:expr, $future:expr) => {
        {
            let context = $propagator.create_root_context($service, $operation).await;
            let traced_future = TracedTask::new($future, context);
            tokio::spawn(traced_future)
        }
    };
    
    ($propagator:expr, $parent_context:expr, $operation:expr, $future:expr) => {
        {
            let context = $propagator.create_child_context($parent_context, $operation).await;
            let traced_future = TracedTask::new($future, context);
            tokio::spawn(traced_future)
        }
    };
}

/// 数据库查询追踪包装器
pub struct TracedDatabaseQuery {
    propagator: Arc<CrossServicePropagator>,
    service_name: String,
}

impl TracedDatabaseQuery {
    pub fn new(propagator: Arc<CrossServicePropagator>, service_name: String) -> Self {
        Self {
            propagator,
            service_name,
        }
    }

    pub async fn execute_query<F, R>(
        &self,
        query: &str,
        parent_context: Option<&PropagationContext>,
        executor: F,
    ) -> Result<R>
    where
        F: Future<Output = Result<R>> + Send,
    {
        let operation = format!("db_query:{}", query.split_whitespace().next().unwrap_or("unknown"));
        
        let context = match parent_context {
            Some(parent) => self.propagator.create_child_context(parent, &operation).await,
            None => self.propagator.create_root_context(&self.service_name, &operation).await,
        };

        let span = info_span!(
            "database_query",
            trace_id = %context.trace_id,
            span_id = %context.current_span_id,
            parent_span_id = context.parent_span_id.as_deref().unwrap_or(""),
            db.query = %query,
            service = %context.service_name
        );

        executor.instrument(span).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cross_service_propagation::PropagationConfig;

    #[tokio::test]
    async fn test_nats_trace_middleware() {
        let config = PropagationConfig::default();
        let propagator = Arc::new(CrossServicePropagator::new(config));
        let middleware = NatsTraceMiddleware::new(Arc::clone(&propagator), "test-service".to_string());

        // 测试消息发布
        let mut message = NatsMessage {
            headers: HashMap::new(),
            data: b"test message".to_vec(),
            reply: None,
        };

        let context = middleware.handle_publish("test.subject", &mut message, "publish_test").await.unwrap();
        
        assert!(!context.trace_id.is_empty());
        assert!(!context.current_span_id.is_empty());
        assert!(message.headers.contains_key("trace_id"));

        // 测试消息订阅
        let result = middleware.handle_subscription(
            "test.subject",
            &message,
            |_ctx| async { Ok(()) }
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_traced_task() {
        let config = PropagationConfig::default();
        let propagator = Arc::new(CrossServicePropagator::new(config));
        let context = propagator.create_root_context("test-service", "test-task").await;

        let future = async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            42
        };

        let traced_task = TracedTask::new(future, context);
        let result = traced_task.await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_database_query_tracing() {
        let config = PropagationConfig::default();
        let propagator = Arc::new(CrossServicePropagator::new(config));
        let db_tracer = TracedDatabaseQuery::new(Arc::clone(&propagator), "db-service".to_string());

        let query = "SELECT * FROM users WHERE id = ?";
        let result = db_tracer.execute_query(
            query,
            None,
            async { Ok("query_result".to_string()) }
        ).await.unwrap();

        assert_eq!(result, "query_result");
    }
}