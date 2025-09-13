use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, delete, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use common_types::ApiResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// 统一的API网关
pub struct ApiGateway {
    pub config: GatewayConfig,
    pub auth_service: Arc<dyn AuthServiceTrait>,
    pub system_service: Arc<dyn SystemServiceTrait>,
    pub qingxi_service: Arc<dyn QingxiServiceTrait>,
    pub dashboard_service: Arc<dyn DashboardServiceTrait>,
    pub monitoring_service: Arc<dyn MonitoringServiceTrait>,
}

/// 网关配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GatewayConfig {
    pub main_port: u16,
    pub grpc_port: u16,
    pub dashboard_port: u16,
    pub metrics_port: u16,
    pub health_port: u16,
    pub enable_cors: bool,
    pub enable_auth: bool,
    pub enable_rate_limit: bool,
    pub api_prefix: String,
    pub api_version: String,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            main_port: 3000,
            grpc_port: 3001,
            dashboard_port: 3002,
            metrics_port: 3003,
            health_port: 3004,
            enable_cors: true,
            enable_auth: true,
            enable_rate_limit: true,
            api_prefix: "/api".to_string(),
            api_version: "v1".to_string(),
        }
    }
}

/// 服务特征定义
pub trait AuthServiceTrait: Send + Sync {
    fn get_router(&self) -> Router;
}

pub trait SystemServiceTrait: Send + Sync {
    fn get_router(&self) -> Router;
}

pub trait QingxiServiceTrait: Send + Sync {
    fn get_router(&self) -> Router;
}

pub trait DashboardServiceTrait: Send + Sync {
    fn get_router(&self) -> Router;
}

pub trait MonitoringServiceTrait: Send + Sync {
    fn get_router(&self) -> Router;
}

impl ApiGateway {
    /// 创建新的API网关实例
    pub fn new(config: GatewayConfig) -> Self {
        // 这里应该初始化各个服务
        // 暂时使用占位符
        todo!("Initialize services")
    }

    /// 创建统一路由器
    pub fn create_unified_router(&self) -> Router {
        let api_prefix = format!("{}/{}", self.config.api_prefix, self.config.api_version);
        
        let mut router = Router::new()
            // 统一健康检查端点
            .route("/health", get(unified_health_check))
            .route("/healthz", get(unified_health_check))
            
            // API版本信息
            .route(&format!("{}/version", api_prefix), get(get_api_version))
            
            // 嵌套各模块路由
            .nest(&format!("{}/auth", api_prefix), self.auth_service.get_router())
            .nest(&format!("{}/system", api_prefix), self.system_service.get_router())
            .nest(&format!("{}/data", api_prefix), self.qingxi_service.get_router())
            .nest(&format!("{}/dashboard", api_prefix), self.dashboard_service.get_router())
            .nest(&format!("{}/monitoring", api_prefix), self.monitoring_service.get_router());
        
        // 添加中间件
        router = self.apply_middleware(router);
        
        router
    }
    
    /// 应用中间件
    fn apply_middleware(&self, router: Router) -> Router {
        let mut service_builder = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http());
        
        // CORS中间件
        if self.config.enable_cors {
            let cors = CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                .allow_origin(Any)
                .allow_headers(Any)
                .expose_headers([header::CONTENT_TYPE]);
            service_builder = service_builder.layer(cors);
        }
        
        // 认证中间件
        if self.config.enable_auth {
            // router = router.layer(middleware::from_fn(auth_middleware));
        }
        
        // 速率限制中间件
        if self.config.enable_rate_limit {
            // router = router.layer(middleware::from_fn(rate_limit_middleware));
        }
        
        router.layer(service_builder)
    }
}

/// 统一健康检查处理器
async fn unified_health_check() -> impl IntoResponse {
    let response = ApiResponse::success(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "services": {
            "auth": "ok",
            "system": "ok",
            "data": "ok",
            "dashboard": "ok",
            "monitoring": "ok"
        }
    }));
    
    Json(response)
}

/// 获取API版本信息
async fn get_api_version() -> impl IntoResponse {
    let response = ApiResponse::success(serde_json::json!({
        "version": "1.0.0",
        "api_version": "v1",
        "build_time": env!("VERGEN_BUILD_TIMESTAMP", "unknown"),
        "git_commit": env!("VERGEN_GIT_SHA", "unknown"),
    }));
    
    Json(response)
}

/// 认证中间件
pub async fn auth_middleware<B>(
    headers: HeaderMap,
    request: axum::http::Request<B>,
    next: Next<B>,
) -> Response {
    // 跳过健康检查和公开端点
    let path = request.uri().path();
    if path == "/health" || path == "/healthz" || path.contains("/auth/login") {
        return next.run(request).await;
    }
    
    // 验证JWT令牌
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                // TODO: 验证JWT令牌
                return next.run(request).await;
            }
        }
    }
    
    // 未授权响应
    let response = ApiResponse::<()>::error("Unauthorized".to_string());
    (StatusCode::UNAUTHORIZED, Json(response)).into_response()
}

/// 速率限制中间件
pub async fn rate_limit_middleware<B>(
    request: axum::http::Request<B>,
    next: Next<B>,
) -> Response {
    // TODO: 实现速率限制逻辑
    next.run(request).await
}

/// 请求日志中间件
pub async fn request_logging_middleware<B>(
    request: axum::http::Request<B>,
    next: Next<B>,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = duration.as_millis(),
        "Request processed"
    );
    
    response
}

/// 错误处理中间件
pub async fn error_handling_middleware<B>(
    request: axum::http::Request<B>,
    next: Next<B>,
) -> Response {
    let response = next.run(request).await;
    
    // 统一错误响应格式
    if response.status().is_server_error() {
        let error_response = ApiResponse::<()>::error("Internal Server Error".to_string());
        return (response.status(), Json(error_response)).into_response();
    }
    
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_health_check() {
        let response = unified_health_check().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_version() {
        let response = get_api_version().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}