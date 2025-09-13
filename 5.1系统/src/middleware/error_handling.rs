//! 错误处理中间件
//! 
//! 统一处理应用程序错误，提供一致的错误响应格式和日志记录

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use common_types::ApiResponse;
use serde_json::json;
use std::panic::{self, AssertUnwindSafe};
use std::future::Future;
use tower::timeout::error::Elapsed;

/// 应用程序错误类型
#[derive(Debug, Clone)]
pub enum AppError {
    /// 验证错误
    Validation(String),
    /// 认证错误
    Authentication(String),
    /// 授权错误
    Authorization(String),
    /// 业务逻辑错误
    Business(String),
    /// 外部服务错误
    ExternalService(String),
    /// 数据库错误
    Database(String),
    /// 网络错误
    Network(String),
    /// 配置错误
    Configuration(String),
    /// 内部服务器错误
    Internal(String),
    /// 请求超时
    Timeout,
    /// 资源未找到
    NotFound(String),
    /// 请求过多
    TooManyRequests,
    /// 服务不可用
    ServiceUnavailable(String),
}

impl AppError {
    /// 获取错误对应的HTTP状态码
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
            AppError::Authorization(_) => StatusCode::FORBIDDEN,
            AppError::Business(_) => StatusCode::BAD_REQUEST,
            AppError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Network(_) => StatusCode::BAD_GATEWAY,
            AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Timeout => StatusCode::REQUEST_TIMEOUT,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// 获取错误类型
    pub fn error_type(&self) -> &'static str {
        match self {
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Authentication(_) => "AUTHENTICATION_ERROR",
            AppError::Authorization(_) => "AUTHORIZATION_ERROR",
            AppError::Business(_) => "BUSINESS_ERROR",
            AppError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Network(_) => "NETWORK_ERROR",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Timeout => "TIMEOUT_ERROR",
            AppError::NotFound(_) => "NOT_FOUND_ERROR",
            AppError::TooManyRequests => "RATE_LIMIT_ERROR",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE_ERROR",
        }
    }

    /// 获取错误消息
    pub fn message(&self) -> String {
        match self {
            AppError::Validation(msg) => format!("Validation failed: {}", msg),
            AppError::Authentication(msg) => format!("Authentication failed: {}", msg),
            AppError::Authorization(msg) => format!("Authorization failed: {}", msg),
            AppError::Business(msg) => format!("Business logic error: {}", msg),
            AppError::ExternalService(msg) => format!("External service error: {}", msg),
            AppError::Database(msg) => format!("Database error: {}", msg),
            AppError::Network(msg) => format!("Network error: {}", msg),
            AppError::Configuration(msg) => format!("Configuration error: {}", msg),
            AppError::Internal(msg) => format!("Internal error: {}", msg),
            AppError::Timeout => "Request timeout".to_string(),
            AppError::NotFound(msg) => format!("Resource not found: {}", msg),
            AppError::TooManyRequests => "Too many requests".to_string(),
            AppError::ServiceUnavailable(msg) => format!("Service unavailable: {}", msg),
        }
    }

    /// 是否应该记录详细错误信息（用于调试）
    pub fn should_log_details(&self) -> bool {
        match self {
            AppError::Validation(_) => false,
            AppError::Authentication(_) => false,
            AppError::Authorization(_) => false,
            AppError::Business(_) => false,
            AppError::NotFound(_) => false,
            AppError::TooManyRequests => false,
            _ => true, // 其他错误记录详细信息
        }
    }

    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            AppError::Validation(msg) => msg.clone(),
            AppError::Authentication(_) => "Authentication required".to_string(),
            AppError::Authorization(_) => "Access denied".to_string(),
            AppError::Business(msg) => msg.clone(),
            AppError::ExternalService(_) => "External service temporarily unavailable".to_string(),
            AppError::Database(_) => "Database service temporarily unavailable".to_string(),
            AppError::Network(_) => "Network connectivity issue".to_string(),
            AppError::Configuration(_) => "Service configuration error".to_string(),
            AppError::Internal(_) => "Internal server error".to_string(),
            AppError::Timeout => "Request timeout, please try again".to_string(),
            AppError::NotFound(msg) => msg.clone(),
            AppError::TooManyRequests => "Rate limit exceeded, please try again later".to_string(),
            AppError::ServiceUnavailable(_) => "Service temporarily unavailable".to_string(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = ApiResponse::<()>::error_with_details(
            &self.user_message(),
            Some(json!({
                "type": self.error_type(),
                "code": status.as_u16(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }))
        );

        (status, Json(error_response)).into_response()
    }
}

/// 错误处理配置
#[derive(Debug, Clone)]
pub struct ErrorHandlingConfig {
    pub log_errors: bool,
    pub include_stack_trace: bool,
    pub include_request_id: bool,
    pub mask_internal_errors: bool,
    pub custom_error_pages: bool,
}

impl Default for ErrorHandlingConfig {
    fn default() -> Self {
        Self {
            log_errors: true,
            include_stack_trace: false, // 生产环境不包含堆栈跟踪
            include_request_id: true,
            mask_internal_errors: true, // 隐藏内部错误详情
            custom_error_pages: false,
        }
    }
}

impl ErrorHandlingConfig {
    pub fn development() -> Self {
        Self {
            log_errors: true,
            include_stack_trace: true,
            include_request_id: true,
            mask_internal_errors: false, // 开发环境显示详细错误
            custom_error_pages: false,
        }
    }

    pub fn production() -> Self {
        Self {
            log_errors: true,
            include_stack_trace: false,
            include_request_id: true,
            mask_internal_errors: true,
            custom_error_pages: true,
        }
    }
}

/// 错误处理中间件
pub struct ErrorHandlingMiddleware {
    config: ErrorHandlingConfig,
}

impl ErrorHandlingMiddleware {
    pub fn new(config: ErrorHandlingConfig) -> Self {
        Self { config }
    }

    /// 记录错误日志
    fn log_error(&self, error: &AppError, request_id: Option<&str>, path: &str, method: &str) {
        if !self.config.log_errors {
            return;
        }

        let log_level = match error {
            AppError::Validation(_) | AppError::Authentication(_) | AppError::Authorization(_) 
            | AppError::Business(_) | AppError::NotFound(_) => "WARN",
            AppError::TooManyRequests => "INFO",
            _ => "ERROR",
        };

        let mut log_data = json!({
            "type": "application_error",
            "level": log_level,
            "error_type": error.error_type(),
            "error_message": error.message(),
            "user_message": error.user_message(),
            "status_code": error.status_code().as_u16(),
            "method": method,
            "path": path,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(req_id) = request_id {
            log_data["request_id"] = json!(req_id);
        }

        if error.should_log_details() {
            log_data["should_investigate"] = json!(true);
        }

        println!("{} [{}] {}", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            log_level,
            serde_json::to_string_pretty(&log_data).unwrap_or_default()
        );
    }

    /// 创建错误响应
    fn create_error_response(&self, error: AppError, request_id: Option<String>) -> Response {
        let status = error.status_code();
        let mut details = json!({
            "type": error.error_type(),
            "code": status.as_u16(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(req_id) = request_id {
            if self.config.include_request_id {
                details["request_id"] = json!(req_id);
            }
        }

        // 在开发环境中包含更多调试信息
        if !self.config.mask_internal_errors {
            details["debug_message"] = json!(error.message());
        }

        let message = if self.config.mask_internal_errors {
            error.user_message()
        } else {
            error.message()
        };

        let error_response = ApiResponse::<()>::error_with_details(&message, Some(details));
        (status, Json(error_response)).into_response()
    }
}

/// 全局错误处理中间件函数
pub async fn error_handler(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let error_handler = ErrorHandlingMiddleware::new(ErrorHandlingConfig::default());
    error_handler_with_config(error_handler, request, next).await
}

/// 带配置的错误处理中间件函数
pub async fn error_handler_with_config(
    error_handler: ErrorHandlingMiddleware,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let request_id = request.extensions().get::<String>().cloned();

    // 设置恐慌钩子来捕获恐慌
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        // 在异步上下文中运行
        tokio::runtime::Handle::current().block_on(async {
            next.run(request).await
        })
    }));

    match result {
        Ok(response) => {
            // 检查响应状态码，对错误状态码进行额外处理
            let status = response.status();
            if status.is_client_error() || status.is_server_error() {
                // 如果响应已经是错误状态，但没有被正确处理，记录日志
                let app_error = match status {
                    StatusCode::BAD_REQUEST => AppError::Validation("Bad request".to_string()),
                    StatusCode::UNAUTHORIZED => AppError::Authentication("Unauthorized".to_string()),
                    StatusCode::FORBIDDEN => AppError::Authorization("Forbidden".to_string()),
                    StatusCode::NOT_FOUND => AppError::NotFound("Resource not found".to_string()),
                    StatusCode::METHOD_NOT_ALLOWED => AppError::Validation("Method not allowed".to_string()),
                    StatusCode::TOO_MANY_REQUESTS => AppError::TooManyRequests,
                    StatusCode::INTERNAL_SERVER_ERROR => AppError::Internal("Internal server error".to_string()),
                    StatusCode::BAD_GATEWAY => AppError::ExternalService("Bad gateway".to_string()),
                    StatusCode::SERVICE_UNAVAILABLE => AppError::ServiceUnavailable("Service unavailable".to_string()),
                    StatusCode::GATEWAY_TIMEOUT => AppError::Timeout,
                    _ => AppError::Internal(format!("HTTP {}", status.as_u16())),
                };
                
                error_handler.log_error(&app_error, request_id.as_deref(), &path, &method);
            }
            
            Ok(response)
        }
        Err(panic_payload) => {
            // 处理恐慌
            let panic_msg = if let Some(s) = panic_payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic occurred".to_string()
            };

            let app_error = AppError::Internal(format!("Panic occurred: {}", panic_msg));
            error_handler.log_error(&app_error, request_id.as_deref(), &path, &method);

            Ok(error_handler.create_error_response(app_error, request_id))
        }
    }
}

/// 开发环境错误处理中间件
pub async fn error_handler_dev(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let error_handler = ErrorHandlingMiddleware::new(ErrorHandlingConfig::development());
    error_handler_with_config(error_handler, request, next).await
}

/// 生产环境错误处理中间件
pub async fn error_handler_prod(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let error_handler = ErrorHandlingMiddleware::new(ErrorHandlingConfig::production());
    error_handler_with_config(error_handler, request, next).await
}

/// 创建自定义错误处理中间件
pub fn create_error_handler(config: ErrorHandlingConfig) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    let error_handler = ErrorHandlingMiddleware::new(config);
    move |request: Request, next: Next| {
        let error_handler = error_handler.clone();
        async move {
            error_handler_with_config(error_handler, request, next).await
        }
    }
}

/// 处理特定错误类型的辅助函数
pub fn handle_timeout_error(_: Elapsed) -> AppError {
    AppError::Timeout
}

pub fn handle_json_parse_error(err: serde_json::Error) -> AppError {
    AppError::Validation(format!("JSON parse error: {}", err))
}

pub fn handle_validation_error(field: &str, message: &str) -> AppError {
    AppError::Validation(format!("Field '{}': {}", field, message))
}

pub fn handle_auth_error(message: &str) -> AppError {
    AppError::Authentication(message.to_string())
}

pub fn handle_business_error(message: &str) -> AppError {
    AppError::Business(message.to_string())
}

/// 错误转换trait，用于将其他错误类型转换为AppError
pub trait IntoAppError {
    fn into_app_error(self) -> AppError;
}

impl IntoAppError for std::io::Error {
    fn into_app_error(self) -> AppError {
        match self.kind() {
            std::io::ErrorKind::NotFound => AppError::NotFound("Resource not found".to_string()),
            std::io::ErrorKind::PermissionDenied => AppError::Authorization("Permission denied".to_string()),
            std::io::ErrorKind::ConnectionRefused => AppError::ExternalService("Connection refused".to_string()),
            std::io::ErrorKind::TimedOut => AppError::Timeout,
            _ => AppError::Internal(format!("IO error: {}", self)),
        }
    }
}

impl IntoAppError for serde_json::Error {
    fn into_app_error(self) -> AppError {
        AppError::Validation(format!("JSON error: {}", self))
    }
}

impl IntoAppError for reqwest::Error {
    fn into_app_error(self) -> AppError {
        if self.is_timeout() {
            AppError::Timeout
        } else if self.is_connect() {
            AppError::Network(format!("Connection error: {}", self))
        } else if self.is_request() {
            AppError::Validation(format!("Request error: {}", self))
        } else {
            AppError::ExternalService(format!("HTTP client error: {}", self))
        }
    }
}

/// 结果扩展trait，提供便捷的错误转换方法
pub trait ResultExt<T> {
    fn map_err_to_app_error<E: IntoAppError>(self) -> Result<T, AppError>;
    fn map_err_validation(self, message: &str) -> Result<T, AppError>;
    fn map_err_business(self, message: &str) -> Result<T, AppError>;
    fn map_err_internal(self, message: &str) -> Result<T, AppError>;
}

impl<T, E: IntoAppError> ResultExt<T> for Result<T, E> {
    fn map_err_to_app_error(self) -> Result<T, AppError> {
        self.map_err(|e| e.into_app_error())
    }

    fn map_err_validation(self, message: &str) -> Result<T, AppError> {
        self.map_err(|_| AppError::Validation(message.to_string()))
    }

    fn map_err_business(self, message: &str) -> Result<T, AppError> {
        self.map_err(|_| AppError::Business(message.to_string()))
    }

    fn map_err_internal(self, message: &str) -> Result<T, AppError> {
        self.map_err(|_| AppError::Internal(message.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_error_status_codes() {
        assert_eq!(AppError::Validation("test".to_string()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(AppError::Authentication("test".to_string()).status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(AppError::Authorization("test".to_string()).status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::Internal("test".to_string()).status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    #[test]
    fn test_app_error_types() {
        assert_eq!(AppError::Validation("test".to_string()).error_type(), "VALIDATION_ERROR");
        assert_eq!(AppError::Authentication("test".to_string()).error_type(), "AUTHENTICATION_ERROR");
        assert_eq!(AppError::Timeout.error_type(), "TIMEOUT_ERROR");
    }
    
    #[test]
    fn test_error_logging_decision() {
        assert!(!AppError::Validation("test".to_string()).should_log_details());
        assert!(!AppError::Authentication("test".to_string()).should_log_details());
        assert!(AppError::Database("test".to_string()).should_log_details());
        assert!(AppError::Internal("test".to_string()).should_log_details());
    }
}