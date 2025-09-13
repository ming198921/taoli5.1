//! 中间件系统 - 统一的请求处理管道
//! 
//! 本模块提供完整的中间件系统，包括：
//! - 认证和授权中间件
//! - 请求日志记录和追踪
//! - CORS（跨域资源共享）处理
//! - 速率限制和请求节流
//! - 错误处理和响应格式化
//! - 请求验证和数据清理
//! - 性能监控和指标收集
//! - 安全头设置

pub mod auth;
pub mod cors;
pub mod logging;
pub mod rate_limiting;
pub mod error_handling;
pub mod request_validation;
pub mod security_headers;
pub mod metrics;
pub mod tracing;

// 重新导出核心中间件
pub use auth::{AuthMiddleware, RequireAuth, RequireRole};
pub use cors::CorsMiddleware;
pub use logging::LoggingMiddleware;
pub use rate_limiting::{RateLimitingMiddleware, RateLimit};
pub use error_handling::ErrorHandlingMiddleware;
pub use request_validation::RequestValidationMiddleware;
pub use security_headers::SecurityHeadersMiddleware;
pub use metrics::MetricsMiddleware;
pub use tracing::TracingMiddleware;

use axum::{Router, middleware};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;

/// 中间件配置
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    pub enable_cors: bool,
    pub enable_auth: bool,
    pub enable_logging: bool,
    pub enable_rate_limiting: bool,
    pub enable_metrics: bool,
    pub enable_tracing: bool,
    pub enable_security_headers: bool,
    pub request_timeout_sec: u64,
    pub max_request_size_mb: usize,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_cors: true,
            enable_auth: true,
            enable_logging: true,
            enable_rate_limiting: true,
            enable_metrics: true,
            enable_tracing: true,
            enable_security_headers: true,
            request_timeout_sec: 30,
            max_request_size_mb: 10,
        }
    }
}

/// 中间件栈构建器
pub struct MiddlewareStack {
    config: MiddlewareConfig,
}

impl MiddlewareStack {
    pub fn new(config: MiddlewareConfig) -> Self {
        Self { config }
    }

    /// 应用所有中间件到路由器
    pub fn apply_to_router<S>(self, router: Router<S>) -> Router<S> 
    where 
        S: Clone + Send + Sync + 'static,
    {
        let mut router = router;

        // 1. 首先应用全局错误处理
        router = router.layer(middleware::from_fn(error_handling::error_handler));

        // 2. 应用安全头
        if self.config.enable_security_headers {
            router = router.layer(middleware::from_fn(security_headers::add_security_headers));
        }

        // 3. 应用CORS
        if self.config.enable_cors {
            router = router.layer(middleware::from_fn(cors::handle_cors));
        }

        // 4. 应用请求追踪
        if self.config.enable_tracing {
            router = router.layer(middleware::from_fn(tracing::trace_requests));
        }

        // 5. 应用日志记录
        if self.config.enable_logging {
            router = router.layer(middleware::from_fn(logging::log_requests));
        }

        // 6. 应用指标收集
        if self.config.enable_metrics {
            router = router.layer(middleware::from_fn(metrics::collect_metrics));
        }

        // 7. 应用速率限制
        if self.config.enable_rate_limiting {
            router = router.layer(middleware::from_fn(rate_limiting::rate_limit));
        }

        // 8. 应用请求验证
        router = router.layer(middleware::from_fn(request_validation::validate_request));

        // 9. 应用超时
        router = router.layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(Duration::from_secs(self.config.request_timeout_sec)))
                .into_inner()
        );

        router
    }

    /// 创建带有认证的中间件栈
    pub fn with_auth(mut self) -> Self {
        self.config.enable_auth = true;
        self
    }

    /// 创建不带认证的中间件栈（用于公开端点）
    pub fn without_auth(mut self) -> Self {
        self.config.enable_auth = false;
        self
    }

    /// 设置请求超时
    pub fn with_timeout(mut self, timeout_sec: u64) -> Self {
        self.config.request_timeout_sec = timeout_sec;
        self
    }

    /// 设置最大请求大小
    pub fn with_max_request_size(mut self, size_mb: usize) -> Self {
        self.config.max_request_size_mb = size_mb;
        self
    }
}

/// 中间件层次结构
/// 
/// 请求处理流程（从外到内）：
/// 1. ErrorHandling - 全局错误捕获和处理
/// 2. SecurityHeaders - 添加安全响应头
/// 3. CORS - 处理跨域请求
/// 4. Tracing - 请求链路追踪
/// 5. Logging - 请求日志记录
/// 6. Metrics - 性能指标收集
/// 7. RateLimiting - 请求频率限制
/// 8. RequestValidation - 请求格式验证
/// 9. Timeout - 请求超时处理
/// 10. Auth - 认证和授权（可选，通过单独的中间件函数应用）
/// 
/// 响应处理流程（从内到外）：
/// 按相反顺序处理响应，确保每层中间件都能正确处理响应数据

/// 创建生产环境中间件栈
pub fn create_production_middleware() -> MiddlewareStack {
    MiddlewareStack::new(MiddlewareConfig {
        enable_cors: true,
        enable_auth: true,
        enable_logging: true,
        enable_rate_limiting: true,
        enable_metrics: true,
        enable_tracing: true,
        enable_security_headers: true,
        request_timeout_sec: 30,
        max_request_size_mb: 10,
    })
}

/// 创建开发环境中间件栈（更宽松的配置）
pub fn create_development_middleware() -> MiddlewareStack {
    MiddlewareStack::new(MiddlewareConfig {
        enable_cors: true,
        enable_auth: false, // 开发环境可能不需要认证
        enable_logging: true,
        enable_rate_limiting: false, // 开发环境不限制请求频率
        enable_metrics: true,
        enable_tracing: true,
        enable_security_headers: false, // 开发环境可能不需要严格的安全头
        request_timeout_sec: 60, // 开发环境允许更长的超时
        max_request_size_mb: 50, // 开发环境允许更大的请求
    })
}

/// 创建API网关专用中间件栈
pub fn create_api_gateway_middleware() -> MiddlewareStack {
    MiddlewareStack::new(MiddlewareConfig {
        enable_cors: true,
        enable_auth: true,
        enable_logging: true,
        enable_rate_limiting: true,
        enable_metrics: true,
        enable_tracing: true,
        enable_security_headers: true,
        request_timeout_sec: 30,
        max_request_size_mb: 10,
    })
}