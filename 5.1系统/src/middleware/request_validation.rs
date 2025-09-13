//! 请求验证中间件
//! 
//! 验证HTTP请求的格式、大小、内容类型等，确保请求符合API规范

use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap, Method},
    middleware::Next,
    response::Response,
    body::Body,
};
use serde_json::Value;

/// 请求验证配置
#[derive(Debug, Clone)]
pub struct RequestValidationConfig {
    pub max_body_size: usize,
    pub max_header_size: usize,
    pub max_uri_length: usize,
    pub allowed_content_types: Vec<String>,
    pub require_content_type: bool,
    pub validate_json_syntax: bool,
    pub max_json_depth: usize,
    pub blocked_user_agents: Vec<String>,
    pub require_user_agent: bool,
    pub allowed_methods: Vec<Method>,
    pub max_query_params: usize,
    pub max_header_count: usize,
}

impl Default for RequestValidationConfig {
    fn default() -> Self {
        Self {
            max_body_size: 10 * 1024 * 1024, // 10MB
            max_header_size: 8192, // 8KB
            max_uri_length: 2048, // 2KB
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
                "multipart/form-data".to_string(),
                "text/plain".to_string(),
            ],
            require_content_type: true,
            validate_json_syntax: true,
            max_json_depth: 10,
            blocked_user_agents: vec![
                "bot".to_string(),
                "crawler".to_string(),
                "spider".to_string(),
                "scraper".to_string(),
            ],
            require_user_agent: false,
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::HEAD,
                Method::OPTIONS,
            ],
            max_query_params: 50,
            max_header_count: 100,
        }
    }
}

impl RequestValidationConfig {
    /// 严格的生产环境配置
    pub fn production() -> Self {
        Self {
            max_body_size: 5 * 1024 * 1024, // 5MB
            max_header_size: 4096, // 4KB
            max_uri_length: 1024, // 1KB
            allowed_content_types: vec![
                "application/json".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ],
            require_content_type: true,
            validate_json_syntax: true,
            max_json_depth: 5,
            blocked_user_agents: vec![
                "bot".to_string(),
                "crawler".to_string(),
                "spider".to_string(),
                "scraper".to_string(),
                "curl".to_string(), // 生产环境可能需要阻止curl
            ],
            require_user_agent: true,
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            max_query_params: 20,
            max_header_count: 50,
        }
    }

    /// 宽松的开发环境配置
    pub fn development() -> Self {
        Self {
            max_body_size: 50 * 1024 * 1024, // 50MB
            max_header_size: 16384, // 16KB
            max_uri_length: 4096, // 4KB
            allowed_content_types: vec!["*".to_string()], // 允许所有类型
            require_content_type: false,
            validate_json_syntax: false, // 开发环境可能不需要严格验证
            max_json_depth: 20,
            blocked_user_agents: Vec::new(), // 不阻止任何User-Agent
            require_user_agent: false,
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::HEAD,
                Method::OPTIONS,
                Method::CONNECT,
                Method::TRACE,
            ],
            max_query_params: 100,
            max_header_count: 200,
        }
    }
}

/// 请求验证错误
#[derive(Debug, Clone)]
pub enum ValidationError {
    BodyTooLarge(usize, usize), // actual, max
    HeaderTooLarge(usize, usize),
    UriTooLong(usize, usize),
    InvalidContentType(String),
    MissingContentType,
    InvalidJsonSyntax(String),
    JsonTooDeep(usize, usize),
    BlockedUserAgent(String),
    MissingUserAgent,
    MethodNotAllowed(String),
    TooManyQueryParams(usize, usize),
    TooManyHeaders(usize, usize),
    InvalidEncoding,
    MalformedRequest(String),
}

impl ValidationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ValidationError::BodyTooLarge(_, _) => StatusCode::PAYLOAD_TOO_LARGE,
            ValidationError::HeaderTooLarge(_, _) => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            ValidationError::UriTooLong(_, _) => StatusCode::URI_TOO_LONG,
            ValidationError::InvalidContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ValidationError::MissingContentType => StatusCode::BAD_REQUEST,
            ValidationError::InvalidJsonSyntax(_) => StatusCode::BAD_REQUEST,
            ValidationError::JsonTooDeep(_, _) => StatusCode::BAD_REQUEST,
            ValidationError::BlockedUserAgent(_) => StatusCode::FORBIDDEN,
            ValidationError::MissingUserAgent => StatusCode::BAD_REQUEST,
            ValidationError::MethodNotAllowed(_) => StatusCode::METHOD_NOT_ALLOWED,
            ValidationError::TooManyQueryParams(_, _) => StatusCode::BAD_REQUEST,
            ValidationError::TooManyHeaders(_, _) => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
            ValidationError::InvalidEncoding => StatusCode::BAD_REQUEST,
            ValidationError::MalformedRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ValidationError::BodyTooLarge(actual, max) => 
                format!("Request body too large: {} bytes (max: {} bytes)", actual, max),
            ValidationError::HeaderTooLarge(actual, max) => 
                format!("Request header too large: {} bytes (max: {} bytes)", actual, max),
            ValidationError::UriTooLong(actual, max) => 
                format!("URI too long: {} characters (max: {} characters)", actual, max),
            ValidationError::InvalidContentType(ct) => 
                format!("Invalid content type: {}", ct),
            ValidationError::MissingContentType => 
                "Content-Type header is required".to_string(),
            ValidationError::InvalidJsonSyntax(err) => 
                format!("Invalid JSON syntax: {}", err),
            ValidationError::JsonTooDeep(actual, max) => 
                format!("JSON nesting too deep: {} levels (max: {} levels)", actual, max),
            ValidationError::BlockedUserAgent(ua) => 
                format!("Blocked user agent: {}", ua),
            ValidationError::MissingUserAgent => 
                "User-Agent header is required".to_string(),
            ValidationError::MethodNotAllowed(method) => 
                format!("Method not allowed: {}", method),
            ValidationError::TooManyQueryParams(actual, max) => 
                format!("Too many query parameters: {} (max: {})", actual, max),
            ValidationError::TooManyHeaders(actual, max) => 
                format!("Too many headers: {} (max: {})", actual, max),
            ValidationError::InvalidEncoding => 
                "Invalid character encoding".to_string(),
            ValidationError::MalformedRequest(details) => 
                format!("Malformed request: {}", details),
        }
    }
}

/// 请求验证器
pub struct RequestValidator {
    config: RequestValidationConfig,
}

impl RequestValidator {
    pub fn new(config: RequestValidationConfig) -> Self {
        Self { config }
    }

    /// 验证请求
    pub fn validate_request(&self, request: &Request) -> Result<(), ValidationError> {
        // 1. 验证HTTP方法
        self.validate_method(request)?;
        
        // 2. 验证URI长度
        self.validate_uri_length(request)?;
        
        // 3. 验证查询参数数量
        self.validate_query_params(request)?;
        
        // 4. 验证请求头
        self.validate_headers(request)?;
        
        // 5. 验证User-Agent
        self.validate_user_agent(request)?;
        
        // 6. 验证Content-Type
        self.validate_content_type(request)?;
        
        // 7. 验证请求体大小
        self.validate_body_size(request)?;

        Ok(())
    }

    fn validate_method(&self, request: &Request) -> Result<(), ValidationError> {
        if !self.config.allowed_methods.contains(request.method()) {
            return Err(ValidationError::MethodNotAllowed(request.method().to_string()));
        }
        Ok(())
    }

    fn validate_uri_length(&self, request: &Request) -> Result<(), ValidationError> {
        let uri_length = request.uri().to_string().len();
        if uri_length > self.config.max_uri_length {
            return Err(ValidationError::UriTooLong(uri_length, self.config.max_uri_length));
        }
        Ok(())
    }

    fn validate_query_params(&self, request: &Request) -> Result<(), ValidationError> {
        if let Some(query) = request.uri().query() {
            let param_count = query.split('&').filter(|s| !s.is_empty()).count();
            if param_count > self.config.max_query_params {
                return Err(ValidationError::TooManyQueryParams(param_count, self.config.max_query_params));
            }
        }
        Ok(())
    }

    fn validate_headers(&self, request: &Request) -> Result<(), ValidationError> {
        let headers = request.headers();
        
        // 检查头部数量
        if headers.len() > self.config.max_header_count {
            return Err(ValidationError::TooManyHeaders(headers.len(), self.config.max_header_count));
        }
        
        // 检查单个头部大小
        for (name, value) in headers.iter() {
            let header_size = name.as_str().len() + value.len();
            if header_size > self.config.max_header_size {
                return Err(ValidationError::HeaderTooLarge(header_size, self.config.max_header_size));
            }
            
            // 检查编码
            if value.to_str().is_err() {
                return Err(ValidationError::InvalidEncoding);
            }
        }

        Ok(())
    }

    fn validate_user_agent(&self, request: &Request) -> Result<(), ValidationError> {
        let headers = request.headers();
        
        if self.config.require_user_agent && !headers.contains_key("user-agent") {
            return Err(ValidationError::MissingUserAgent);
        }
        
        if let Some(user_agent) = headers.get("user-agent") {
            if let Ok(ua_str) = user_agent.to_str() {
                let ua_lower = ua_str.to_lowercase();
                for blocked in &self.config.blocked_user_agents {
                    if ua_lower.contains(&blocked.to_lowercase()) {
                        return Err(ValidationError::BlockedUserAgent(ua_str.to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_content_type(&self, request: &Request) -> Result<(), ValidationError> {
        let headers = request.headers();
        let method = request.method();
        
        // 只有可能有请求体的方法才需要检查Content-Type
        if matches!(method, &Method::POST | &Method::PUT | &Method::PATCH) {
            if self.config.require_content_type && !headers.contains_key("content-type") {
                return Err(ValidationError::MissingContentType);
            }
            
            if let Some(content_type) = headers.get("content-type") {
                if let Ok(ct_str) = content_type.to_str() {
                    let ct_main = ct_str.split(';').next().unwrap_or(ct_str).trim();
                    
                    if !self.config.allowed_content_types.contains(&"*".to_string()) 
                        && !self.config.allowed_content_types.contains(&ct_main.to_string()) {
                        return Err(ValidationError::InvalidContentType(ct_main.to_string()));
                    }
                }
            }
        }

        Ok(())
    }

    fn validate_body_size(&self, request: &Request) -> Result<(), ValidationError> {
        // 检查Content-Length头
        if let Some(content_length) = request.headers().get("content-length") {
            if let Ok(length_str) = content_length.to_str() {
                if let Ok(length) = length_str.parse::<usize>() {
                    if length > self.config.max_body_size {
                        return Err(ValidationError::BodyTooLarge(length, self.config.max_body_size));
                    }
                }
            }
        }
        
        // 对于已知的请求体，检查实际大小
        let body_size = request.body().size_hint().lower() as usize;
        if body_size > self.config.max_body_size {
            return Err(ValidationError::BodyTooLarge(body_size, self.config.max_body_size));
        }

        Ok(())
    }

    /// 验证JSON内容（在有请求体的情况下调用）
    pub fn validate_json_body(&self, json_str: &str) -> Result<(), ValidationError> {
        if !self.config.validate_json_syntax {
            return Ok(());
        }

        // 解析JSON
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| ValidationError::InvalidJsonSyntax(e.to_string()))?;
        
        // 检查嵌套深度
        let depth = calculate_json_depth(&value);
        if depth > self.config.max_json_depth {
            return Err(ValidationError::JsonTooDeep(depth, self.config.max_json_depth));
        }

        Ok(())
    }

    /// 验证特定路径的请求
    pub fn validate_path_specific(&self, request: &Request, path: &str) -> Result<(), ValidationError> {
        // 根据路径应用特定验证规则
        match path {
            path if path.starts_with("/api/auth/") => {
                // 认证接口的特殊验证
                self.validate_auth_request(request)?;
            },
            path if path.starts_with("/api/upload/") => {
                // 上传接口的特殊验证
                self.validate_upload_request(request)?;
            },
            _ => {
                // 默认验证
            }
        }
        
        Ok(())
    }

    fn validate_auth_request(&self, request: &Request) -> Result<(), ValidationError> {
        let headers = request.headers();
        
        // 认证接口必须是POST方法
        if request.method() != Method::POST {
            return Err(ValidationError::MethodNotAllowed("Authentication endpoints only accept POST".to_string()));
        }
        
        // 必须有Content-Type
        if !headers.contains_key("content-type") {
            return Err(ValidationError::MissingContentType);
        }
        
        // Content-Type必须是JSON
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(ct_str) = content_type.to_str() {
                if !ct_str.starts_with("application/json") {
                    return Err(ValidationError::InvalidContentType("Authentication requires JSON content type".to_string()));
                }
            }
        }
        
        Ok(())
    }

    fn validate_upload_request(&self, request: &Request) -> Result<(), ValidationError> {
        let headers = request.headers();
        
        // 上传接口必须是POST或PUT方法
        if !matches!(request.method(), &Method::POST | &Method::PUT) {
            return Err(ValidationError::MethodNotAllowed("Upload endpoints only accept POST or PUT".to_string()));
        }
        
        // 检查Content-Type是否是multipart/form-data
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(ct_str) = content_type.to_str() {
                if !ct_str.starts_with("multipart/form-data") {
                    return Err(ValidationError::InvalidContentType("Upload requires multipart/form-data".to_string()));
                }
            }
        }
        
        Ok(())
    }
}

/// 计算JSON嵌套深度
fn calculate_json_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => {
            1 + map.values().map(calculate_json_depth).max().unwrap_or(0)
        },
        Value::Array(vec) => {
            1 + vec.iter().map(calculate_json_depth).max().unwrap_or(0)
        },
        _ => 0,
    }
}

/// 请求验证中间件函数
pub async fn validate_request(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let validator = RequestValidator::new(RequestValidationConfig::default());
    validate_request_with_config(validator, request, next).await
}

/// 带配置的请求验证中间件函数
pub async fn validate_request_with_config(
    validator: RequestValidator,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path().to_string();
    
    // 执行基本验证
    if let Err(validation_error) = validator.validate_request(&request) {
        log_validation_error(&validation_error, &path);
        return Err(validation_error.status_code());
    }
    
    // 执行路径特定验证
    if let Err(validation_error) = validator.validate_path_specific(&request, &path) {
        log_validation_error(&validation_error, &path);
        return Err(validation_error.status_code());
    }
    
    Ok(next.run(request).await)
}

/// 开发环境请求验证中间件
pub async fn validate_request_dev(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let validator = RequestValidator::new(RequestValidationConfig::development());
    validate_request_with_config(validator, request, next).await
}

/// 生产环境请求验证中间件
pub async fn validate_request_prod(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let validator = RequestValidator::new(RequestValidationConfig::production());
    validate_request_with_config(validator, request, next).await
}

/// 创建自定义请求验证中间件
pub fn create_validation_middleware(config: RequestValidationConfig) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    let validator = RequestValidator::new(config);
    move |request: Request, next: Next| {
        let validator = validator.clone();
        async move {
            validate_request_with_config(validator, request, next).await
        }
    }
}

/// 记录验证错误
fn log_validation_error(error: &ValidationError, path: &str) {
    let log_data = serde_json::json!({
        "type": "request_validation_error",
        "error": error.message(),
        "path": path,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    println!("{} [WARN] {}", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        serde_json::to_string_pretty(&log_data).unwrap_or_default()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderName, HeaderValue};
    
    #[test]
    fn test_json_depth_calculation() {
        let simple = serde_json::json!({"key": "value"});
        assert_eq!(calculate_json_depth(&simple), 1);
        
        let nested = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": "value"
                }
            }
        });
        assert_eq!(calculate_json_depth(&nested), 3);
        
        let array = serde_json::json!([{"nested": {"deep": "value"}}]);
        assert_eq!(calculate_json_depth(&array), 3);
    }
    
    #[test]
    fn test_validation_config() {
        let config = RequestValidationConfig::production();
        assert!(config.require_content_type);
        assert!(config.require_user_agent);
        assert_eq!(config.max_body_size, 5 * 1024 * 1024);
        
        let dev_config = RequestValidationConfig::development();
        assert!(!dev_config.require_content_type);
        assert!(!dev_config.require_user_agent);
        assert!(dev_config.blocked_user_agents.is_empty());
    }
    
    #[test]
    fn test_validation_error_status_codes() {
        assert_eq!(ValidationError::BodyTooLarge(1000, 500).status_code(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(ValidationError::MethodNotAllowed("TEST".to_string()).status_code(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(ValidationError::InvalidContentType("test".to_string()).status_code(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
}