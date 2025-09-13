//! 请求日志记录中间件
//! 
//! 记录HTTP请求和响应的详细信息，用于调试、监控和审计

use axum::{
    extract::{Request, MatchedPath},
    middleware::Next,
    response::Response,
};
use std::time::{Duration, Instant};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::middleware::auth::AuthContext;

/// 日志级别
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }
}

/// 请求日志配置
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub log_requests: bool,
    pub log_responses: bool,
    pub log_request_body: bool,
    pub log_response_body: bool,
    pub log_headers: bool,
    pub log_query_params: bool,
    pub max_body_size: usize,
    pub excluded_paths: Vec<String>,
    pub included_headers: Vec<String>,
    pub excluded_headers: Vec<String>,
    pub log_level: LogLevel,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_request_body: false, // 出于安全考虑默认不记录请求体
            log_response_body: false, // 出于性能考虑默认不记录响应体
            log_headers: true,
            log_query_params: true,
            max_body_size: 4096, // 4KB
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/favicon.ico".to_string(),
            ],
            included_headers: vec![
                "user-agent".to_string(),
                "content-type".to_string(),
                "content-length".to_string(),
                "origin".to_string(),
                "referer".to_string(),
                "x-forwarded-for".to_string(),
                "x-real-ip".to_string(),
            ],
            excluded_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "password".to_string(),
            ],
            log_level: LogLevel::Info,
        }
    }
}

/// 开发环境日志配置
impl LoggingConfig {
    pub fn development() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_request_body: true,
            log_response_body: true,
            log_headers: true,
            log_query_params: true,
            max_body_size: 16384, // 16KB
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
            ],
            included_headers: vec![
                "user-agent".to_string(),
                "content-type".to_string(),
                "content-length".to_string(),
                "origin".to_string(),
                "referer".to_string(),
                "authorization".to_string(), // 开发环境可以记录
                "x-forwarded-for".to_string(),
                "x-real-ip".to_string(),
            ],
            excluded_headers: vec![
                "cookie".to_string(),
                "password".to_string(),
            ],
            log_level: LogLevel::Debug,
        }
    }

    pub fn production() -> Self {
        Self {
            log_requests: true,
            log_responses: true,
            log_request_body: false,
            log_response_body: false,
            log_headers: true,
            log_query_params: false, // 生产环境可能包含敏感信息
            max_body_size: 1024, // 1KB
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/status".to_string(),
                "/favicon.ico".to_string(),
                "/robots.txt".to_string(),
            ],
            included_headers: vec![
                "user-agent".to_string(),
                "content-type".to_string(),
                "content-length".to_string(),
                "origin".to_string(),
                "x-forwarded-for".to_string(),
                "x-real-ip".to_string(),
            ],
            excluded_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
            ],
            log_level: LogLevel::Info,
        }
    }
}

/// 请求上下文信息
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub start_time: Instant,
    pub method: String,
    pub uri: String,
    pub path: String,
    pub query: Option<String>,
    pub version: String,
    pub headers: Value,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub user_id: Option<String>,
    pub username: Option<String>,
}

/// 响应上下文信息
#[derive(Debug, Clone)]
pub struct ResponseContext {
    pub status_code: u16,
    pub headers: Value,
    pub body_size: usize,
    pub duration: Duration,
}

/// 日志记录中间件实现
pub struct LoggingMiddleware {
    config: LoggingConfig,
}

impl LoggingMiddleware {
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(LoggingConfig::default())
    }

    fn should_log_path(&self, path: &str) -> bool {
        !self.config.excluded_paths.iter().any(|excluded| path.starts_with(excluded))
    }

    fn filter_headers(&self, headers: &axum::http::HeaderMap) -> Value {
        let mut filtered_headers = serde_json::Map::new();
        
        for (name, value) in headers.iter() {
            let header_name = name.as_str().to_lowercase();
            
            // 检查是否在排除列表中
            if self.config.excluded_headers.iter().any(|excluded| header_name.contains(excluded)) {
                filtered_headers.insert(header_name, Value::String("[FILTERED]".to_string()));
                continue;
            }
            
            // 检查是否在包含列表中（如果列表不为空）
            if !self.config.included_headers.is_empty() {
                if !self.config.included_headers.iter().any(|included| header_name == included.to_lowercase()) {
                    continue;
                }
            }
            
            if let Ok(header_value) = value.to_str() {
                filtered_headers.insert(header_name, Value::String(header_value.to_string()));
            }
        }
        
        Value::Object(filtered_headers)
    }

    fn get_client_ip(&self, request: &Request) -> String {
        // 优先级：X-Forwarded-For > X-Real-IP > 连接IP
        let headers = request.headers();
        
        if let Some(forwarded_for) = headers.get("x-forwarded-for") {
            if let Ok(forwarded_str) = forwarded_for.to_str() {
                if let Some(first_ip) = forwarded_str.split(',').next() {
                    return first_ip.trim().to_string();
                }
            }
        }
        
        if let Some(real_ip) = headers.get("x-real-ip") {
            if let Ok(ip_str) = real_ip.to_str() {
                return ip_str.to_string();
            }
        }
        
        // 在实际应用中，这里应该从连接信息获取真实IP
        "unknown".to_string()
    }

    fn create_request_context(&self, request: &Request) -> RequestContext {
        let request_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();
        let method = request.method().to_string();
        let uri = request.uri().to_string();
        let path = request.uri().path().to_string();
        let query = request.uri().query().map(|q| q.to_string());
        let version = format!("{:?}", request.version());
        let headers = if self.config.log_headers {
            self.filter_headers(request.headers())
        } else {
            Value::Object(serde_json::Map::new())
        };
        let client_ip = self.get_client_ip(request);
        let user_agent = request.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        // 从认证上下文获取用户信息
        let (user_id, username) = if let Some(auth_context) = request.extensions().get::<AuthContext>() {
            (Some(auth_context.user_id.clone()), Some(auth_context.username.clone()))
        } else {
            (None, None)
        };

        RequestContext {
            request_id,
            start_time,
            method,
            uri,
            path,
            query,
            version,
            headers,
            client_ip,
            user_agent,
            user_id,
            username,
        }
    }

    fn log_request(&self, context: &RequestContext) {
        if !self.config.log_requests {
            return;
        }

        let mut log_data = json!({
            "type": "http_request",
            "request_id": context.request_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "method": context.method,
            "uri": context.uri,
            "path": context.path,
            "version": context.version,
            "client_ip": context.client_ip,
        });

        if let Some(query) = &context.query {
            if self.config.log_query_params {
                log_data["query"] = Value::String(query.clone());
            }
        }

        if let Some(user_agent) = &context.user_agent {
            log_data["user_agent"] = Value::String(user_agent.clone());
        }

        if let Some(user_id) = &context.user_id {
            log_data["user_id"] = Value::String(user_id.clone());
        }

        if let Some(username) = &context.username {
            log_data["username"] = Value::String(username.clone());
        }

        if self.config.log_headers {
            log_data["headers"] = context.headers.clone();
        }

        println!("{} [{}] {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            self.config.log_level.as_str(),
            serde_json::to_string_pretty(&log_data).unwrap_or_default()
        );
    }

    fn log_response(&self, request_context: &RequestContext, response_context: &ResponseContext) {
        if !self.config.log_responses {
            return;
        }

        let log_data = json!({
            "type": "http_response",
            "request_id": request_context.request_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "method": request_context.method,
            "path": request_context.path,
            "status_code": response_context.status_code,
            "duration_ms": response_context.duration.as_millis(),
            "body_size": response_context.body_size,
            "client_ip": request_context.client_ip,
            "user_id": request_context.user_id,
            "username": request_context.username,
            "headers": response_context.headers,
        });

        let level = match response_context.status_code {
            200..=299 => LogLevel::Info,
            300..=399 => LogLevel::Info,
            400..=499 => LogLevel::Warn,
            500..=599 => LogLevel::Error,
            _ => LogLevel::Info,
        };

        println!("{} [{}] {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            level.as_str(),
            serde_json::to_string_pretty(&log_data).unwrap_or_default()
        );
    }
}

/// 日志记录中间件函数
pub async fn log_requests(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let logging = LoggingMiddleware::with_default_config();
    log_requests_with_config(logging, request, next).await
}

/// 带配置的日志记录中间件函数
pub async fn log_requests_with_config(
    logging: LoggingMiddleware,
    mut request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // 检查是否应该记录此路径
    let path = request.uri().path();
    if !logging.should_log_path(path) {
        return Ok(next.run(request).await);
    }

    // 创建请求上下文
    let request_context = logging.create_request_context(&request);
    
    // 将请求ID添加到请求扩展中，用于后续的关联
    request.extensions_mut().insert(request_context.request_id.clone());
    
    // 记录请求
    logging.log_request(&request_context);

    // 执行请求处理
    let response = next.run(request).await;
    
    // 计算响应时间
    let duration = request_context.start_time.elapsed();
    
    // 创建响应上下文
    let response_context = ResponseContext {
        status_code: response.status().as_u16(),
        headers: logging.filter_headers(response.headers()),
        body_size: response.body().size_hint().lower() as usize,
        duration,
    };
    
    // 记录响应
    logging.log_response(&request_context, &response_context);
    
    Ok(response)
}

/// 开发环境日志中间件
pub async fn log_requests_dev(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let logging = LoggingMiddleware::new(LoggingConfig::development());
    log_requests_with_config(logging, request, next).await
}

/// 生产环境日志中间件
pub async fn log_requests_prod(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let logging = LoggingMiddleware::new(LoggingConfig::production());
    log_requests_with_config(logging, request, next).await
}

/// 创建自定义日志中间件
pub fn create_logging_middleware(config: LoggingConfig) -> impl Fn(Request, Next) -> Result<Response, axum::http::StatusCode> + Clone {
    let logging = LoggingMiddleware::new(config);
    move |request: Request, next: Next| {
        let logging = logging.clone();
        async move {
            log_requests_with_config(logging, request, next).await
        }
    }
}

/// 获取当前请求ID的辅助函数
pub fn get_request_id(request: &Request) -> Option<String> {
    request.extensions().get::<String>().cloned()
}

/// 结构化日志宏
#[macro_export]
macro_rules! log_info {
    ($message:expr) => {
        println!("{} [INFO] {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), $message);
    };
    ($message:expr, $($arg:tt)*) => {
        println!("{} [INFO] {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), 
            format!($message, $($arg)*)
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($message:expr) => {
        eprintln!("{} [ERROR] {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), $message);
    };
    ($message:expr, $($arg:tt)*) => {
        eprintln!("{} [ERROR] {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), 
            format!($message, $($arg)*)
        );
    };
}

#[macro_export]
macro_rules! log_warn {
    ($message:expr) => {
        println!("{} [WARN] {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), $message);
    };
    ($message:expr, $($arg:tt)*) => {
        println!("{} [WARN] {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), 
            format!($message, $($arg)*)
        );
    };
}

#[macro_export]
macro_rules! log_debug {
    ($message:expr) => {
        println!("{} [DEBUG] {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), $message);
    };
    ($message:expr, $($arg:tt)*) => {
        println!("{} [DEBUG] {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"), 
            format!($message, $($arg)*)
        );
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderName, HeaderValue};
    
    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(config.log_requests);
        assert!(config.log_responses);
        assert!(!config.log_request_body); // 安全考虑
        assert!(!config.log_response_body); // 性能考虑
    }
    
    #[test]
    fn test_development_config() {
        let config = LoggingConfig::development();
        assert!(config.log_request_body);
        assert!(config.log_response_body);
        assert_eq!(config.max_body_size, 16384);
    }
    
    #[test]
    fn test_production_config() {
        let config = LoggingConfig::production();
        assert!(!config.log_request_body);
        assert!(!config.log_response_body);
        assert!(!config.log_query_params); // 可能包含敏感信息
    }
    
    #[test]
    fn test_header_filtering() {
        let logging = LoggingMiddleware::with_default_config();
        let mut headers = HeaderMap::new();
        headers.insert(HeaderName::from_static("user-agent"), HeaderValue::from_static("test-agent"));
        headers.insert(HeaderName::from_static("authorization"), HeaderValue::from_static("Bearer secret"));
        
        let filtered = logging.filter_headers(&headers);
        assert!(filtered["user-agent"].as_str().unwrap() == "test-agent");
        assert!(filtered["authorization"].as_str().unwrap() == "[FILTERED]");
    }
}