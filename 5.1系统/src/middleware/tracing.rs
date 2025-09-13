//! 请求追踪中间件
//! 
//! 为每个HTTP请求创建唯一的追踪ID，用于跨服务的请求链路追踪和调试

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, HeaderName},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use serde::{Serialize, Deserialize};
use crate::middleware::auth::AuthContext;

/// 追踪上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service_name: String,
    pub operation_name: String,
    pub start_time: SystemTime,
    pub duration: Option<Duration>,
    pub status: TraceStatus,
    pub tags: HashMap<String, String>,
    pub logs: Vec<TraceLog>,
}

/// 追踪状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TraceStatus {
    Ok,
    Error,
    Timeout,
    Cancelled,
}

/// 追踪日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceLog {
    pub timestamp: SystemTime,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// 追踪配置
#[derive(Debug, Clone)]
pub struct TracingConfig {
    pub enabled: bool,
    pub service_name: String,
    pub sample_rate: f64, // 0.0 - 1.0
    pub max_spans_per_trace: usize,
    pub max_logs_per_span: usize,
    pub trace_timeout: Duration,
    pub export_traces: bool,
    pub export_endpoint: Option<String>,
    pub trace_header_name: String,
    pub span_header_name: String,
    pub baggage_header_prefix: String,
    pub excluded_paths: Vec<String>,
    pub include_request_body: bool,
    pub include_response_body: bool,
    pub max_body_size: usize,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            service_name: "arbitrage-api-gateway".to_string(),
            sample_rate: 1.0, // 默认全部采样
            max_spans_per_trace: 100,
            max_logs_per_span: 50,
            trace_timeout: Duration::from_secs(300), // 5分钟
            export_traces: false, // 默认不导出
            export_endpoint: None,
            trace_header_name: "X-Trace-Id".to_string(),
            span_header_name: "X-Span-Id".to_string(),
            baggage_header_prefix: "X-Baggage-".to_string(),
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/favicon.ico".to_string(),
            ],
            include_request_body: false,
            include_response_body: false,
            max_body_size: 4096, // 4KB
        }
    }
}

impl TracingConfig {
    pub fn production() -> Self {
        Self {
            enabled: true,
            service_name: "arbitrage-api-gateway".to_string(),
            sample_rate: 0.1, // 生产环境采样10%
            max_spans_per_trace: 50,
            max_logs_per_span: 20,
            trace_timeout: Duration::from_secs(60),
            export_traces: true,
            export_endpoint: Some("http://jaeger-collector:14268/api/traces".to_string()),
            trace_header_name: "X-Trace-Id".to_string(),
            span_header_name: "X-Span-Id".to_string(),
            baggage_header_prefix: "X-Baggage-".to_string(),
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/status".to_string(),
                "/favicon.ico".to_string(),
                "/robots.txt".to_string(),
            ],
            include_request_body: false, // 生产环境不包含请求体
            include_response_body: false,
            max_body_size: 1024,
        }
    }

    pub fn development() -> Self {
        Self {
            enabled: true,
            service_name: "arbitrage-api-gateway-dev".to_string(),
            sample_rate: 1.0, // 开发环境全部采样
            max_spans_per_trace: 200,
            max_logs_per_span: 100,
            trace_timeout: Duration::from_secs(600), // 10分钟
            export_traces: false, // 开发环境通常不导出
            export_endpoint: None,
            trace_header_name: "X-Trace-Id".to_string(),
            span_header_name: "X-Span-Id".to_string(),
            baggage_header_prefix: "X-Baggage-".to_string(),
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
            ],
            include_request_body: true, // 开发环境包含请求体以便调试
            include_response_body: true,
            max_body_size: 16384, // 16KB
        }
    }
}

/// 追踪器
pub struct Tracer {
    config: TracingConfig,
    active_traces: Arc<RwLock<HashMap<String, TraceContext>>>,
    trace_storage: Arc<RwLock<Vec<TraceContext>>>,
}

impl Tracer {
    pub fn new(config: TracingConfig) -> Self {
        Self {
            config,
            active_traces: Arc::new(RwLock::new(HashMap::new())),
            trace_storage: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 开始一个新的追踪
    pub fn start_trace(&self, operation_name: &str, parent_trace_id: Option<&str>) -> Option<TraceContext> {
        if !self.config.enabled {
            return None;
        }

        // 采样决策
        if !self.should_sample() {
            return None;
        }

        let trace_id = parent_trace_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let span_id = Uuid::new_v4().to_string();
        
        let mut trace_context = TraceContext {
            trace_id: trace_id.clone(),
            span_id,
            parent_span_id: None,
            service_name: self.config.service_name.clone(),
            operation_name: operation_name.to_string(),
            start_time: SystemTime::now(),
            duration: None,
            status: TraceStatus::Ok,
            tags: HashMap::new(),
            logs: Vec::new(),
        };

        // 添加基本标签
        trace_context.tags.insert("service.name".to_string(), self.config.service_name.clone());
        trace_context.tags.insert("service.version".to_string(), "1.0.0".to_string());

        // 存储到活跃追踪中
        self.active_traces.write().unwrap().insert(trace_id.clone(), trace_context.clone());

        Some(trace_context)
    }

    /// 完成追踪
    pub fn finish_trace(&self, trace_id: &str, status: TraceStatus) {
        if !self.config.enabled {
            return;
        }

        let mut active_traces = self.active_traces.write().unwrap();
        if let Some(mut trace_context) = active_traces.remove(trace_id) {
            trace_context.duration = Some(trace_context.start_time.elapsed().unwrap_or(Duration::ZERO));
            trace_context.status = status;

            // 存储完成的追踪
            let mut storage = self.trace_storage.write().unwrap();
            storage.push(trace_context.clone());

            // 限制存储的追踪数量
            if storage.len() > 1000 {
                storage.drain(0..100); // 移除最旧的100个追踪
            }

            // 导出追踪（如果配置了）
            if self.config.export_traces {
                self.export_trace(&trace_context);
            }
        }
    }

    /// 向追踪添加标签
    pub fn add_tag(&self, trace_id: &str, key: &str, value: &str) {
        if !self.config.enabled {
            return;
        }

        let mut active_traces = self.active_traces.write().unwrap();
        if let Some(trace_context) = active_traces.get_mut(trace_id) {
            trace_context.tags.insert(key.to_string(), value.to_string());
        }
    }

    /// 向追踪添加日志
    pub fn add_log(&self, trace_id: &str, level: &str, message: &str, fields: HashMap<String, String>) {
        if !self.config.enabled {
            return;
        }

        let mut active_traces = self.active_traces.write().unwrap();
        if let Some(trace_context) = active_traces.get_mut(trace_id) {
            if trace_context.logs.len() < self.config.max_logs_per_span {
                let log = TraceLog {
                    timestamp: SystemTime::now(),
                    level: level.to_string(),
                    message: message.to_string(),
                    fields,
                };
                trace_context.logs.push(log);
            }
        }
    }

    /// 获取所有已完成的追踪
    pub fn get_completed_traces(&self) -> Vec<TraceContext> {
        self.trace_storage.read().unwrap().clone()
    }

    /// 查找特定的追踪
    pub fn get_trace(&self, trace_id: &str) -> Option<TraceContext> {
        // 首先检查活跃的追踪
        if let Some(trace) = self.active_traces.read().unwrap().get(trace_id) {
            return Some(trace.clone());
        }

        // 然后检查已完成的追踪
        self.trace_storage.read().unwrap()
            .iter()
            .find(|trace| trace.trace_id == trace_id)
            .cloned()
    }

    /// 采样决策
    fn should_sample(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.config.sample_rate
    }

    /// 导出追踪
    fn export_trace(&self, trace: &TraceContext) {
        // 这里应该实现实际的追踪导出逻辑
        // 例如发送到Jaeger、Zipkin等追踪系统
        if let Some(endpoint) = &self.config.export_endpoint {
            // 异步发送追踪数据
            let trace_json = serde_json::to_string(trace).unwrap_or_default();
            println!("Exporting trace to {}: {}", endpoint, trace_json);
        }
    }

    /// 从请求头提取追踪信息
    pub fn extract_trace_context(&self, headers: &HeaderMap) -> Option<(String, Option<String>)> {
        let trace_id = headers.get(&self.config.trace_header_name)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        let span_id = headers.get(&self.config.span_header_name)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        trace_id.map(|tid| (tid, span_id))
    }

    /// 将追踪信息注入到响应头
    pub fn inject_trace_context(&self, headers: &mut HeaderMap, trace_context: &TraceContext) {
        if let Ok(trace_id_value) = HeaderValue::from_str(&trace_context.trace_id) {
            headers.insert(
                HeaderName::from_bytes(self.config.trace_header_name.as_bytes()).unwrap(),
                trace_id_value
            );
        }

        if let Ok(span_id_value) = HeaderValue::from_str(&trace_context.span_id) {
            headers.insert(
                HeaderName::from_bytes(self.config.span_header_name.as_bytes()).unwrap(),
                span_id_value
            );
        }
    }

    /// 检查路径是否应该被排除
    fn should_exclude_path(&self, path: &str) -> bool {
        self.config.excluded_paths.iter().any(|excluded| path.starts_with(excluded))
    }
}

/// 追踪中间件函数
pub async fn trace_requests(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let tracer = Tracer::new(TracingConfig::default());
    trace_requests_with_config(tracer, request, next).await
}

/// 带配置的追踪中间件函数
pub async fn trace_requests_with_config(
    tracer: Tracer,
    mut request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    if !tracer.config.enabled {
        return Ok(next.run(request).await);
    }

    let path = request.uri().path();
    
    // 检查是否应该排除此路径
    if tracer.should_exclude_path(path) {
        return Ok(next.run(request).await);
    }

    let method = request.method().to_string();
    let operation_name = format!("{} {}", method, path);

    // 从请求头提取或创建追踪上下文
    let (trace_id, parent_span_id) = tracer.extract_trace_context(request.headers())
        .unwrap_or_else(|| (Uuid::new_v4().to_string(), None));

    // 开始追踪
    let trace_context = tracer.start_trace(&operation_name, Some(&trace_id));
    
    if let Some(mut context) = trace_context {
        context.parent_span_id = parent_span_id;
        
        // 添加请求相关的标签
        tracer.add_tag(&context.trace_id, "http.method", &method);
        tracer.add_tag(&context.trace_id, "http.url", &request.uri().to_string());
        tracer.add_tag(&context.trace_id, "http.scheme", request.uri().scheme_str().unwrap_or("http"));
        tracer.add_tag(&context.trace_id, "http.path", path);
        
        if let Some(query) = request.uri().query() {
            tracer.add_tag(&context.trace_id, "http.query", query);
        }

        if let Some(user_agent) = request.headers().get("user-agent").and_then(|h| h.to_str().ok()) {
            tracer.add_tag(&context.trace_id, "http.user_agent", user_agent);
        }

        // 添加用户信息（如果可用）
        if let Some(auth_context) = request.extensions().get::<AuthContext>() {
            tracer.add_tag(&context.trace_id, "user.id", &auth_context.user_id);
            tracer.add_tag(&context.trace_id, "user.name", &auth_context.username);
            tracer.add_tag(&context.trace_id, "user.role", &format!("{:?}", auth_context.role));
        }

        // 将追踪上下文添加到请求扩展中，供后续中间件使用
        request.extensions_mut().insert(context.clone());

        // 添加请求开始日志
        tracer.add_log(&context.trace_id, "INFO", "Request started", {
            let mut fields = HashMap::new();
            fields.insert("method".to_string(), method.clone());
            fields.insert("path".to_string(), path.to_string());
            fields
        });

        // 处理请求
        let start_time = Instant::now();
        let response = next.run(request).await;
        let duration = start_time.elapsed();

        // 添加响应相关的标签和日志
        let status_code = response.status().as_u16();
        tracer.add_tag(&context.trace_id, "http.status_code", &status_code.to_string());
        tracer.add_tag(&context.trace_id, "http.response_size", &response.body().size_hint().lower().to_string());

        // 确定追踪状态
        let trace_status = match status_code {
            200..=299 => TraceStatus::Ok,
            400..=499 => TraceStatus::Error,
            500..=599 => TraceStatus::Error,
            _ => TraceStatus::Ok,
        };

        // 添加响应完成日志
        tracer.add_log(&context.trace_id, "INFO", "Request completed", {
            let mut fields = HashMap::new();
            fields.insert("status_code".to_string(), status_code.to_string());
            fields.insert("duration_ms".to_string(), duration.as_millis().to_string());
            fields
        });

        // 如果是错误状态，添加错误日志
        if matches!(trace_status, TraceStatus::Error) {
            tracer.add_log(&context.trace_id, "ERROR", "Request resulted in error", {
                let mut fields = HashMap::new();
                fields.insert("status_code".to_string(), status_code.to_string());
                fields.insert("error_category".to_string(), 
                    if status_code >= 500 { "server_error" } else { "client_error" }.to_string());
                fields
            });
        }

        // 完成追踪
        tracer.finish_trace(&context.trace_id, trace_status);

        // 将追踪信息注入响应头
        let mut response = response;
        tracer.inject_trace_context(response.headers_mut(), &context);

        Ok(response)
    } else {
        // 如果没有创建追踪上下文（可能由于采样），直接处理请求
        Ok(next.run(request).await)
    }
}

/// 开发环境追踪中间件
pub async fn trace_requests_dev(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let tracer = Tracer::new(TracingConfig::development());
    trace_requests_with_config(tracer, request, next).await
}

/// 生产环境追踪中间件
pub async fn trace_requests_prod(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let tracer = Tracer::new(TracingConfig::production());
    trace_requests_with_config(tracer, request, next).await
}

/// 创建自定义追踪中间件
pub fn create_tracing_middleware(config: TracingConfig) -> impl Fn(Request, Next) -> Result<Response, axum::http::StatusCode> + Clone {
    let tracer = Tracer::new(config);
    move |request: Request, next: Next| {
        let tracer = tracer.clone();
        async move {
            trace_requests_with_config(tracer, request, next).await
        }
    }
}

/// 从请求扩展中获取追踪上下文的辅助函数
pub fn get_trace_context(request: &Request) -> Option<&TraceContext> {
    request.extensions().get::<TraceContext>()
}

/// 全局追踪器实例
static GLOBAL_TRACER: std::sync::OnceLock<Arc<Tracer>> = std::sync::OnceLock::new();

/// 获取全局追踪器
pub fn get_global_tracer() -> Option<&'static Arc<Tracer>> {
    GLOBAL_TRACER.get()
}

/// 初始化全局追踪器
pub fn init_global_tracer(config: TracingConfig) {
    let tracer = Arc::new(Tracer::new(config));
    GLOBAL_TRACER.set(tracer).ok();
}

/// 追踪宏，用于在代码中手动添加追踪点
#[macro_export]
macro_rules! trace_span {
    ($operation:expr) => {
        if let Some(tracer) = get_global_tracer() {
            tracer.start_trace($operation, None)
        } else {
            None
        }
    };
    ($operation:expr, $parent_trace_id:expr) => {
        if let Some(tracer) = get_global_tracer() {
            tracer.start_trace($operation, Some($parent_trace_id))
        } else {
            None
        }
    };
}

#[macro_export]
macro_rules! trace_log {
    ($trace_id:expr, $level:expr, $message:expr) => {
        if let Some(tracer) = get_global_tracer() {
            tracer.add_log($trace_id, $level, $message, HashMap::new())
        }
    };
    ($trace_id:expr, $level:expr, $message:expr, $fields:expr) => {
        if let Some(tracer) = get_global_tracer() {
            tracer.add_log($trace_id, $level, $message, $fields)
        }
    };
}

#[macro_export]
macro_rules! trace_tag {
    ($trace_id:expr, $key:expr, $value:expr) => {
        if let Some(tracer) = get_global_tracer() {
            tracer.add_tag($trace_id, $key, $value)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tracing_config() {
        let config = TracingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.sample_rate, 1.0);
        
        let prod_config = TracingConfig::production();
        assert_eq!(prod_config.sample_rate, 0.1);
        assert!(!prod_config.include_request_body);
        
        let dev_config = TracingConfig::development();
        assert_eq!(dev_config.sample_rate, 1.0);
        assert!(dev_config.include_request_body);
    }
    
    #[test]
    fn test_trace_context_creation() {
        let tracer = Tracer::new(TracingConfig::default());
        let trace = tracer.start_trace("test_operation", None);
        
        assert!(trace.is_some());
        let trace_context = trace.unwrap();
        assert_eq!(trace_context.operation_name, "test_operation");
        assert_eq!(trace_context.service_name, "arbitrage-api-gateway");
        assert_eq!(trace_context.status, TraceStatus::Ok);
    }
    
    #[test]
    fn test_sampling() {
        let mut config = TracingConfig::default();
        config.sample_rate = 0.0;
        let tracer = Tracer::new(config);
        
        // 采样率为0时应该不创建追踪
        let trace = tracer.start_trace("test", None);
        assert!(trace.is_none());
    }
    
    #[test]
    fn test_trace_completion() {
        let tracer = Tracer::new(TracingConfig::default());
        let trace = tracer.start_trace("test_operation", None).unwrap();
        let trace_id = trace.trace_id.clone();
        
        tracer.finish_trace(&trace_id, TraceStatus::Ok);
        
        // 追踪应该从活跃列表中移除并添加到存储中
        assert!(!tracer.active_traces.read().unwrap().contains_key(&trace_id));
        assert!(!tracer.trace_storage.read().unwrap().is_empty());
    }
}