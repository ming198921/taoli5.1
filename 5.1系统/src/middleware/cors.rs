//! CORS（跨域资源共享）中间件
//! 
//! 处理浏览器的跨域请求，设置合适的CORS头部

use axum::{
    extract::Request,
    http::{HeaderValue, Method, StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};

/// CORS配置
#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<String>,
    pub exposed_headers: Vec<String>,
    pub allow_credentials: bool,
    pub max_age: Option<u64>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:3001".to_string(),
                "http://localhost:8080".to_string(),
                "https://your-domain.com".to_string(),
            ],
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
                Method::HEAD,
            ],
            allowed_headers: vec![
                "accept".to_string(),
                "accept-language".to_string(),
                "content-type".to_string(),
                "content-language".to_string(),
                "origin".to_string(),
                "user-agent".to_string(),
                "authorization".to_string(),
                "x-api-key".to_string(),
                "x-requested-with".to_string(),
                "x-client-version".to_string(),
                "cache-control".to_string(),
            ],
            exposed_headers: vec![
                "x-total-count".to_string(),
                "x-page-count".to_string(),
                "x-rate-limit-remaining".to_string(),
                "x-rate-limit-reset".to_string(),
            ],
            allow_credentials: true,
            max_age: Some(86400), // 24小时
        }
    }
}

/// CORS中间件实现
pub struct CorsMiddleware {
    config: CorsConfig,
}

impl CorsMiddleware {
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(CorsConfig::default())
    }

    /// 创建用于开发环境的宽松CORS配置
    pub fn development() -> Self {
        Self::new(CorsConfig {
            allowed_origins: vec!["*".to_string()], // 开发环境允许所有源
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
                Method::HEAD,
            ],
            allowed_headers: vec!["*".to_string()], // 允许所有头部
            exposed_headers: vec![
                "x-total-count".to_string(),
                "x-page-count".to_string(),
                "x-rate-limit-remaining".to_string(),
                "x-rate-limit-reset".to_string(),
            ],
            allow_credentials: false, // 当允许所有源时，不能同时允许凭据
            max_age: Some(86400),
        })
    }

    /// 创建用于生产环境的严格CORS配置
    pub fn production(allowed_origins: Vec<String>) -> Self {
        Self::new(CorsConfig {
            allowed_origins,
            allowed_methods: vec![
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ],
            allowed_headers: vec![
                "accept".to_string(),
                "content-type".to_string(),
                "origin".to_string(),
                "authorization".to_string(),
                "x-api-key".to_string(),
                "x-requested-with".to_string(),
            ],
            exposed_headers: vec![
                "x-total-count".to_string(),
                "x-rate-limit-remaining".to_string(),
                "x-rate-limit-reset".to_string(),
            ],
            allow_credentials: true,
            max_age: Some(3600), // 1小时
        })
    }

    /// 检查源是否被允许
    fn is_origin_allowed(&self, origin: &str) -> bool {
        self.config.allowed_origins.contains(&"*".to_string()) 
            || self.config.allowed_origins.contains(&origin.to_string())
    }

    /// 检查方法是否被允许
    fn is_method_allowed(&self, method: &Method) -> bool {
        self.config.allowed_methods.contains(method)
    }

    /// 添加CORS头部到响应
    fn add_cors_headers(&self, headers: &mut HeaderMap, origin: Option<&str>) {
        // Access-Control-Allow-Origin
        if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                if self.config.allowed_origins.contains(&"*".to_string()) {
                    headers.insert("access-control-allow-origin", HeaderValue::from_static("*"));
                } else {
                    headers.insert("access-control-allow-origin", HeaderValue::from_str(origin).unwrap_or(HeaderValue::from_static("null")));
                }
            }
        }

        // Access-Control-Allow-Methods
        let methods = self.config.allowed_methods
            .iter()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        headers.insert("access-control-allow-methods", HeaderValue::from_str(&methods).unwrap_or(HeaderValue::from_static("GET, POST")));

        // Access-Control-Allow-Headers
        if self.config.allowed_headers.contains(&"*".to_string()) {
            headers.insert("access-control-allow-headers", HeaderValue::from_static("*"));
        } else {
            let allowed_headers = self.config.allowed_headers.join(", ");
            headers.insert("access-control-allow-headers", HeaderValue::from_str(&allowed_headers).unwrap_or(HeaderValue::from_static("content-type")));
        }

        // Access-Control-Expose-Headers
        if !self.config.exposed_headers.is_empty() {
            let exposed_headers = self.config.exposed_headers.join(", ");
            headers.insert("access-control-expose-headers", HeaderValue::from_str(&exposed_headers).unwrap_or(HeaderValue::from_static("")));
        }

        // Access-Control-Allow-Credentials
        if self.config.allow_credentials {
            headers.insert("access-control-allow-credentials", HeaderValue::from_static("true"));
        }

        // Access-Control-Max-Age
        if let Some(max_age) = self.config.max_age {
            headers.insert("access-control-max-age", HeaderValue::from_str(&max_age.to_string()).unwrap_or(HeaderValue::from_static("3600")));
        }
    }
}

/// CORS中间件函数
pub async fn handle_cors(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let cors = CorsMiddleware::with_default_config();
    handle_cors_with_config(cors, request, next).await
}

/// 带配置的CORS中间件函数
pub async fn handle_cors_with_config(
    cors: CorsMiddleware,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let origin = request.headers()
        .get("origin")
        .and_then(|h| h.to_str().ok());

    let method = request.method();

    // 处理预检请求 (OPTIONS)
    if method == Method::OPTIONS {
        // 检查请求的方法是否被允许
        let requested_method = request.headers()
            .get("access-control-request-method")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<Method>().ok());

        if let Some(req_method) = requested_method {
            if !cors.is_method_allowed(&req_method) {
                return Err(StatusCode::METHOD_NOT_ALLOWED);
            }
        }

        // 检查请求的头部是否被允许
        if let Some(requested_headers) = request.headers().get("access-control-request-headers") {
            if let Ok(headers_str) = requested_headers.to_str() {
                if !cors.config.allowed_headers.contains(&"*".to_string()) {
                    for header in headers_str.split(',').map(|h| h.trim().to_lowercase()) {
                        if !cors.config.allowed_headers.contains(&header) {
                            return Err(StatusCode::FORBIDDEN);
                        }
                    }
                }
            }
        }

        // 返回预检响应
        let mut response = Response::new("".into());
        cors.add_cors_headers(response.headers_mut(), origin);
        *response.status_mut() = StatusCode::NO_CONTENT;
        return Ok(response);
    }

    // 检查源是否被允许
    if let Some(origin_str) = origin {
        if !cors.is_origin_allowed(origin_str) {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // 处理实际请求
    let mut response = next.run(request).await;
    cors.add_cors_headers(response.headers_mut(), origin);

    Ok(response)
}

/// 开发环境CORS中间件
pub async fn handle_dev_cors(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let cors = CorsMiddleware::development();
    handle_cors_with_config(cors, request, next).await
}

/// 生产环境CORS中间件工厂函数
pub fn create_production_cors(allowed_origins: Vec<String>) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    let cors = CorsMiddleware::production(allowed_origins);
    move |request: Request, next: Next| {
        let cors = cors.clone();
        async move {
            handle_cors_with_config(cors, request, next).await
        }
    }
}

/// 自定义CORS中间件工厂函数
pub fn create_custom_cors(config: CorsConfig) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    let cors = CorsMiddleware::new(config);
    move |request: Request, next: Next| {
        let cors = cors.clone();
        async move {
            handle_cors_with_config(cors, request, next).await
        }
    }
}

/// CORS预检请求检查辅助函数
pub fn is_preflight_request(request: &Request) -> bool {
    request.method() == Method::OPTIONS 
        && request.headers().contains_key("origin")
        && request.headers().contains_key("access-control-request-method")
}

/// 获取请求源的辅助函数
pub fn get_origin(request: &Request) -> Option<String> {
    request.headers()
        .get("origin")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
}

/// 验证CORS头部的辅助函数
pub fn validate_cors_headers(request: &Request, allowed_headers: &[String]) -> bool {
    if let Some(requested_headers) = request.headers().get("access-control-request-headers") {
        if let Ok(headers_str) = requested_headers.to_str() {
            for header in headers_str.split(',').map(|h| h.trim().to_lowercase()) {
                if !allowed_headers.contains(&header) && !allowed_headers.contains(&"*".to_string()) {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(config.allow_credentials);
        assert_eq!(config.max_age, Some(86400));
        assert!(config.allowed_methods.contains(&Method::GET));
        assert!(config.allowed_methods.contains(&Method::POST));
    }
    
    #[test]
    fn test_cors_middleware_origin_check() {
        let cors = CorsMiddleware::with_default_config();
        assert!(cors.is_origin_allowed("http://localhost:3000"));
        assert!(!cors.is_origin_allowed("http://malicious-site.com"));
    }
    
    #[test]
    fn test_development_cors() {
        let cors = CorsMiddleware::development();
        assert!(cors.is_origin_allowed("http://any-origin.com"));
        assert!(!cors.config.allow_credentials); // 开发环境不允许凭据
    }
}