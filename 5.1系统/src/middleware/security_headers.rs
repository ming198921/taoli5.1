//! 安全头中间件
//! 
//! 自动为所有响应添加安全相关的HTTP头部，提高应用程序的安全性

use axum::{
    extract::Request,
    http::{HeaderValue, HeaderMap},
    middleware::Next,
    response::Response,
};

/// 安全头配置
#[derive(Debug, Clone)]
pub struct SecurityHeadersConfig {
    // Content Security Policy
    pub csp: Option<String>,
    // X-Frame-Options
    pub frame_options: Option<String>,
    // X-Content-Type-Options
    pub content_type_options: bool,
    // X-XSS-Protection  
    pub xss_protection: Option<String>,
    // Strict-Transport-Security
    pub hsts: Option<HstsConfig>,
    // Referrer-Policy
    pub referrer_policy: Option<String>,
    // Permissions-Policy
    pub permissions_policy: Option<String>,
    // X-Permitted-Cross-Domain-Policies
    pub cross_domain_policy: Option<String>,
    // X-Download-Options
    pub download_options: bool,
    // Cache-Control for sensitive pages
    pub cache_control: Option<String>,
    // Server header (remove or customize)
    pub hide_server: bool,
    pub custom_server: Option<String>,
}

/// HSTS配置
#[derive(Debug, Clone)]
pub struct HstsConfig {
    pub max_age: u32,
    pub include_subdomains: bool,
    pub preload: bool,
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            csp: Some(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' 'unsafe-eval' https://cdnjs.cloudflare.com; \
                 style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; \
                 font-src 'self' https://fonts.gstatic.com; \
                 img-src 'self' data: https:; \
                 connect-src 'self' wss: ws:; \
                 frame-ancestors 'none'".to_string()
            ),
            frame_options: Some("DENY".to_string()),
            content_type_options: true,
            xss_protection: Some("1; mode=block".to_string()),
            hsts: Some(HstsConfig {
                max_age: 31536000, // 1年
                include_subdomains: true,
                preload: false, // 默认不开启preload
            }),
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some(
                "geolocation=(), microphone=(), camera=(), \
                 payment=(), usb=(), magnetometer=(), gyroscope=()".to_string()
            ),
            cross_domain_policy: Some("none".to_string()),
            download_options: true,
            cache_control: Some("no-cache, no-store, must-revalidate".to_string()),
            hide_server: true,
            custom_server: Some("Arbitrage-System/1.0".to_string()),
        }
    }
}

impl SecurityHeadersConfig {
    /// 开发环境配置（较为宽松）
    pub fn development() -> Self {
        Self {
            csp: Some(
                "default-src 'self'; \
                 script-src 'self' 'unsafe-inline' 'unsafe-eval' *; \
                 style-src 'self' 'unsafe-inline' *; \
                 font-src 'self' *; \
                 img-src 'self' data: *; \
                 connect-src 'self' *; \
                 frame-ancestors 'self' localhost:*".to_string()
            ),
            frame_options: Some("SAMEORIGIN".to_string()),
            content_type_options: true,
            xss_protection: Some("1; mode=block".to_string()),
            hsts: None, // 开发环境不需要HSTS
            referrer_policy: Some("same-origin".to_string()),
            permissions_policy: None, // 开发环境不限制
            cross_domain_policy: Some("master-only".to_string()),
            download_options: false,
            cache_control: None, // 开发环境不需要缓存控制
            hide_server: false, // 开发环境显示服务器信息
            custom_server: None,
        }
    }

    /// 生产环境配置（严格安全）
    pub fn production() -> Self {
        Self {
            csp: Some(
                "default-src 'self'; \
                 script-src 'self'; \
                 style-src 'self'; \
                 font-src 'self'; \
                 img-src 'self' data:; \
                 connect-src 'self'; \
                 frame-ancestors 'none'; \
                 base-uri 'self'; \
                 form-action 'self'".to_string()
            ),
            frame_options: Some("DENY".to_string()),
            content_type_options: true,
            xss_protection: Some("1; mode=block".to_string()),
            hsts: Some(HstsConfig {
                max_age: 63072000, // 2年
                include_subdomains: true,
                preload: true,
            }),
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            permissions_policy: Some(
                "geolocation=(), microphone=(), camera=(), payment=(), \
                 usb=(), magnetometer=(), gyroscope=(), accelerometer=(), \
                 ambient-light-sensor=(), autoplay=(), encrypted-media=(), \
                 fullscreen=(), midi=(), speaker=()".to_string()
            ),
            cross_domain_policy: Some("none".to_string()),
            download_options: true,
            cache_control: Some("no-cache, no-store, must-revalidate, private".to_string()),
            hide_server: true,
            custom_server: None, // 生产环境不暴露服务器信息
        }
    }

    /// API专用配置
    pub fn api_only() -> Self {
        Self {
            csp: Some("default-src 'none'".to_string()), // API不需要加载资源
            frame_options: Some("DENY".to_string()),
            content_type_options: true,
            xss_protection: None, // API不需要XSS保护
            hsts: Some(HstsConfig {
                max_age: 31536000,
                include_subdomains: true,
                preload: false,
            }),
            referrer_policy: Some("no-referrer".to_string()),
            permissions_policy: Some("*=()".to_string()), // 禁用所有权限
            cross_domain_policy: Some("none".to_string()),
            download_options: false,
            cache_control: Some("no-cache, no-store, must-revalidate".to_string()),
            hide_server: true,
            custom_server: Some("API/1.0".to_string()),
        }
    }
}

/// 安全头中间件实现
pub struct SecurityHeadersMiddleware {
    config: SecurityHeadersConfig,
}

impl SecurityHeadersMiddleware {
    pub fn new(config: SecurityHeadersConfig) -> Self {
        Self { config }
    }

    /// 添加所有安全头到响应
    pub fn add_headers_to_response(&self, headers: &mut HeaderMap) {
        // Content Security Policy
        if let Some(csp) = &self.config.csp {
            headers.insert("content-security-policy", HeaderValue::from_str(csp).unwrap());
        }

        // X-Frame-Options
        if let Some(frame_options) = &self.config.frame_options {
            headers.insert("x-frame-options", HeaderValue::from_str(frame_options).unwrap());
        }

        // X-Content-Type-Options
        if self.config.content_type_options {
            headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
        }

        // X-XSS-Protection
        if let Some(xss_protection) = &self.config.xss_protection {
            headers.insert("x-xss-protection", HeaderValue::from_str(xss_protection).unwrap());
        }

        // Strict-Transport-Security
        if let Some(hsts) = &self.config.hsts {
            let mut hsts_value = format!("max-age={}", hsts.max_age);
            if hsts.include_subdomains {
                hsts_value.push_str("; includeSubDomains");
            }
            if hsts.preload {
                hsts_value.push_str("; preload");
            }
            headers.insert("strict-transport-security", HeaderValue::from_str(&hsts_value).unwrap());
        }

        // Referrer-Policy
        if let Some(referrer_policy) = &self.config.referrer_policy {
            headers.insert("referrer-policy", HeaderValue::from_str(referrer_policy).unwrap());
        }

        // Permissions-Policy
        if let Some(permissions_policy) = &self.config.permissions_policy {
            headers.insert("permissions-policy", HeaderValue::from_str(permissions_policy).unwrap());
        }

        // X-Permitted-Cross-Domain-Policies
        if let Some(cross_domain_policy) = &self.config.cross_domain_policy {
            headers.insert("x-permitted-cross-domain-policies", HeaderValue::from_str(cross_domain_policy).unwrap());
        }

        // X-Download-Options
        if self.config.download_options {
            headers.insert("x-download-options", HeaderValue::from_static("noopen"));
        }

        // Cache-Control (for sensitive endpoints)
        if let Some(cache_control) = &self.config.cache_control {
            headers.insert("cache-control", HeaderValue::from_str(cache_control).unwrap());
            headers.insert("pragma", HeaderValue::from_static("no-cache"));
            headers.insert("expires", HeaderValue::from_static("0"));
        }

        // 自定义Server头或隐藏
        if self.config.hide_server {
            headers.remove("server");
            if let Some(custom_server) = &self.config.custom_server {
                headers.insert("server", HeaderValue::from_str(custom_server).unwrap());
            }
        }

        // 添加一些额外的安全头
        headers.insert("x-robots-tag", HeaderValue::from_static("noindex, nofollow, nosnippet, noarchive"));
        headers.insert("x-request-id", HeaderValue::from_str(&uuid::Uuid::new_v4().to_string()).unwrap());
        
        // 移除可能泄露信息的头部
        headers.remove("x-powered-by");
        headers.remove("x-aspnet-version");
        headers.remove("x-aspnetmvc-version");
    }

    /// 根据请求路径调整安全头
    pub fn adjust_headers_for_path(&self, headers: &mut HeaderMap, path: &str) {
        match path {
            // 对于API端点，使用更严格的CSP
            path if path.starts_with("/api/") => {
                headers.insert("content-security-policy", HeaderValue::from_static("default-src 'none'"));
            },
            // 对于认证端点，添加额外保护
            path if path.contains("/auth/") => {
                headers.insert("cache-control", HeaderValue::from_static("no-cache, no-store, must-revalidate, private"));
                headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
            },
            // 对于上传端点，允许multipart
            path if path.contains("/upload/") => {
                if let Some(csp) = headers.get("content-security-policy") {
                    if let Ok(csp_str) = csp.to_str() {
                        let new_csp = format!("{}; form-action 'self'", csp_str);
                        headers.insert("content-security-policy", HeaderValue::from_str(&new_csp).unwrap());
                    }
                }
            },
            // 对于WebSocket端点
            path if path.contains("/ws/") || path.contains("/websocket/") => {
                if let Some(csp) = headers.get("content-security-policy") {
                    if let Ok(csp_str) = csp.to_str() {
                        let new_csp = format!("{}; connect-src 'self' ws: wss:", csp_str);
                        headers.insert("content-security-policy", HeaderValue::from_str(&new_csp).unwrap());
                    }
                }
            },
            _ => {
                // 默认配置已经应用
            }
        }
    }
}

/// 安全头中间件函数
pub async fn add_security_headers(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let middleware = SecurityHeadersMiddleware::new(SecurityHeadersConfig::default());
    add_security_headers_with_config(middleware, request, next).await
}

/// 带配置的安全头中间件函数
pub async fn add_security_headers_with_config(
    middleware: SecurityHeadersMiddleware,
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let path = request.uri().path().to_string();
    
    let mut response = next.run(request).await;
    
    // 添加安全头
    middleware.add_headers_to_response(response.headers_mut());
    
    // 根据路径调整安全头
    middleware.adjust_headers_for_path(response.headers_mut(), &path);
    
    Ok(response)
}

/// 开发环境安全头中间件
pub async fn add_security_headers_dev(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let middleware = SecurityHeadersMiddleware::new(SecurityHeadersConfig::development());
    add_security_headers_with_config(middleware, request, next).await
}

/// 生产环境安全头中间件
pub async fn add_security_headers_prod(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let middleware = SecurityHeadersMiddleware::new(SecurityHeadersConfig::production());
    add_security_headers_with_config(middleware, request, next).await
}

/// API专用安全头中间件
pub async fn add_api_security_headers(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let middleware = SecurityHeadersMiddleware::new(SecurityHeadersConfig::api_only());
    add_security_headers_with_config(middleware, request, next).await
}

/// 创建自定义安全头中间件
pub fn create_security_headers_middleware(config: SecurityHeadersConfig) -> impl Fn(Request, Next) -> Result<Response, axum::http::StatusCode> + Clone {
    let middleware = SecurityHeadersMiddleware::new(config);
    move |request: Request, next: Next| {
        let middleware = middleware.clone();
        async move {
            add_security_headers_with_config(middleware, request, next).await
        }
    }
}

/// 检查HTTPS的辅助函数
pub fn is_https_request(request: &Request) -> bool {
    // 检查X-Forwarded-Proto头（反向代理）
    if let Some(proto) = request.headers().get("x-forwarded-proto") {
        if let Ok(proto_str) = proto.to_str() {
            return proto_str.to_lowercase() == "https";
        }
    }
    
    // 检查X-Forwarded-SSL头
    if let Some(ssl) = request.headers().get("x-forwarded-ssl") {
        if let Ok(ssl_str) = ssl.to_str() {
            return ssl_str.to_lowercase() == "on";
        }
    }
    
    // 检查协议
    request.uri().scheme_str() == Some("https")
}

/// 获取内容安全策略违规报告的处理函数
pub async fn handle_csp_report(
    request: Request,
    _next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // 记录CSP违规报告
    let log_data = serde_json::json!({
        "type": "csp_violation_report",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "user_agent": request.headers().get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown"),
        "remote_addr": request.headers().get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .or_else(|| request.headers().get("x-real-ip").and_then(|h| h.to_str().ok()))
            .unwrap_or("unknown"),
    });

    println!("{} [WARN] CSP Violation Report: {}", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
        serde_json::to_string_pretty(&log_data).unwrap_or_default()
    );

    // 返回204 No Content
    Ok(Response::builder()
        .status(204)
        .body(axum::body::Body::empty())
        .unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    
    #[test]
    fn test_security_headers_config() {
        let config = SecurityHeadersConfig::default();
        assert!(config.content_type_options);
        assert!(config.hide_server);
        assert!(config.hsts.is_some());
        
        let dev_config = SecurityHeadersConfig::development();
        assert!(!dev_config.hide_server);
        assert!(dev_config.hsts.is_none());
        
        let prod_config = SecurityHeadersConfig::production();
        assert!(prod_config.hsts.is_some());
        assert!(prod_config.hsts.as_ref().unwrap().preload);
    }
    
    #[test]
    fn test_hsts_header_generation() {
        let config = HstsConfig {
            max_age: 31536000,
            include_subdomains: true,
            preload: true,
        };
        
        let middleware = SecurityHeadersMiddleware::new(SecurityHeadersConfig {
            hsts: Some(config),
            ..Default::default()
        });
        
        let mut headers = HeaderMap::new();
        middleware.add_headers_to_response(&mut headers);
        
        let hsts_value = headers.get("strict-transport-security").unwrap();
        let hsts_str = hsts_value.to_str().unwrap();
        
        assert!(hsts_str.contains("max-age=31536000"));
        assert!(hsts_str.contains("includeSubDomains"));
        assert!(hsts_str.contains("preload"));
    }
    
    #[test]
    fn test_api_config() {
        let config = SecurityHeadersConfig::api_only();
        assert_eq!(config.csp.as_ref().unwrap(), "default-src 'none'");
        assert_eq!(config.referrer_policy.as_ref().unwrap(), "no-referrer");
    }
}