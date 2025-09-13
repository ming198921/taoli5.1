//! 认证和授权中间件
//! 
//! 提供JWT令牌验证、用户认证、角色权限检查等功能

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use common_types::ApiResponse;
use crate::services::auth_service::{AuthService, UserRole, Claims};
use std::sync::Arc;
use tower_cookies::Cookies;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde_json;

/// 认证中间件状态
#[derive(Clone)]
pub struct AuthMiddleware {
    pub jwt_secret: String,
    pub auth_service: Arc<dyn crate::services::AuthServiceTrait>,
}

impl AuthMiddleware {
    pub fn new(jwt_secret: String, auth_service: Arc<dyn crate::services::AuthServiceTrait>) -> Self {
        Self {
            jwt_secret,
            auth_service,
        }
    }
}

/// 认证状态扩展
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub username: String,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub token_exp: i64,
    pub is_admin: bool,
}

/// JWT令牌验证中间件
pub async fn authenticate(
    State(auth_middleware): State<Arc<AuthMiddleware>>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从多个来源尝试获取令牌
    let token = extract_token_from_request(&request, &cookies);
    
    if let Some(token) = token {
        match verify_jwt_token(&token, &auth_middleware.jwt_secret).await {
            Ok(claims) => {
                // 验证用户是否仍然有效
                match auth_middleware.auth_service.get_user_by_id(&claims.sub).await {
                    Ok(user) => {
                        let auth_context = AuthContext {
                            user_id: claims.sub.clone(),
                            username: user.username,
                            role: user.role.clone(),
                            permissions: get_role_permissions(&user.role),
                            token_exp: claims.exp,
                            is_admin: matches!(user.role, UserRole::SuperAdmin | UserRole::Admin),
                        };
                        
                        // 将认证上下文添加到请求扩展中
                        request.extensions_mut().insert(auth_context);
                        Ok(next.run(request).await)
                    },
                    Err(_) => {
                        // 用户不存在或已被禁用
                        Err(StatusCode::UNAUTHORIZED)
                    }
                }
            },
            Err(_) => {
                // 令牌无效
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    } else {
        // 没有找到令牌
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// 可选认证中间件（令牌存在时验证，不存在时允许通过）
pub async fn optional_auth(
    State(auth_middleware): State<Arc<AuthMiddleware>>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_token_from_request(&request, &cookies);
    
    if let Some(token) = token {
        if let Ok(claims) = verify_jwt_token(&token, &auth_middleware.jwt_secret).await {
            if let Ok(user) = auth_middleware.auth_service.get_user_by_id(&claims.sub).await {
                let auth_context = AuthContext {
                    user_id: claims.sub.clone(),
                    username: user.username,
                    role: user.role.clone(),
                    permissions: get_role_permissions(&user.role),
                    token_exp: claims.exp,
                    is_admin: matches!(user.role, UserRole::SuperAdmin | UserRole::Admin),
                };
                request.extensions_mut().insert(auth_context);
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// 角色权限检查中间件
pub fn require_role(required_role: UserRole) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    move |request: Request, next: Next| {
        let required_role = required_role.clone();
        async move {
            if let Some(auth_context) = request.extensions().get::<AuthContext>() {
                if has_required_role(&auth_context.role, &required_role) {
                    Ok(next.run(request).await)
                } else {
                    Err(StatusCode::FORBIDDEN)
                }
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

/// 权限检查中间件
pub fn require_permission(required_permission: &str) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    let required_permission = required_permission.to_string();
    move |request: Request, next: Next| {
        let required_permission = required_permission.clone();
        async move {
            if let Some(auth_context) = request.extensions().get::<AuthContext>() {
                if auth_context.permissions.contains(&required_permission) {
                    Ok(next.run(request).await)
                } else {
                    Err(StatusCode::FORBIDDEN)
                }
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}

/// 管理员权限检查中间件
pub async fn require_admin(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(auth_context) = request.extensions().get::<AuthContext>() {
        if auth_context.is_admin {
            Ok(next.run(request).await)
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// API密钥认证中间件（用于服务间通信）
pub async fn api_key_auth(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    
    if let Some(api_key) = headers.get("X-API-Key") {
        if let Ok(key_str) = api_key.to_str() {
            // 在实际应用中，应该从数据库或配置中验证API密钥
            if is_valid_api_key(key_str).await {
                // 创建服务认证上下文
                let auth_context = AuthContext {
                    user_id: "service_account".to_string(),
                    username: "Service Account".to_string(),
                    role: UserRole::SuperAdmin, // 服务账户具有管理员权限
                    permissions: get_role_permissions(&UserRole::SuperAdmin),
                    token_exp: 0, // API密钥不会过期
                    is_admin: true,
                };
                
                let mut request = request;
                request.extensions_mut().insert(auth_context);
                return Ok(next.run(request).await);
            }
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}

/// IP白名单认证中间件
pub fn ip_whitelist(allowed_ips: Vec<String>) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    move |request: Request, next: Next| {
        let allowed_ips = allowed_ips.clone();
        async move {
            let client_ip = get_client_ip(&request);
            
            if allowed_ips.contains(&client_ip) || allowed_ips.contains(&"*".to_string()) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

// 辅助函数

/// 从请求中提取JWT令牌
fn extract_token_from_request(request: &Request, cookies: &Cookies) -> Option<String> {
    // 1. 首先检查Authorization头
    if let Some(auth_header) = request.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }
    
    // 2. 检查cookie
    if let Some(cookie) = cookies.get("auth_token") {
        return Some(cookie.value().to_string());
    }
    
    // 3. 检查query参数
    if let Some(query) = request.uri().query() {
        for pair in query.split('&') {
            if let Some((key, value)) = pair.split_once('=') {
                if key == "token" {
                    return Some(value.to_string());
                }
            }
        }
    }
    
    None
}

/// 验证JWT令牌
async fn verify_jwt_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let key = DecodingKey::from_secret(secret.as_bytes());
    
    let token_data = decode::<Claims>(token, &key, &validation)?;
    
    // 检查令牌是否过期
    let now = chrono::Utc::now().timestamp();
    if token_data.claims.exp < now {
        return Err(jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::ExpiredSignature));
    }
    
    Ok(token_data.claims)
}

/// 检查用户角色是否满足要求
fn has_required_role(user_role: &UserRole, required_role: &UserRole) -> bool {
    let role_hierarchy = [
        UserRole::Viewer,
        UserRole::Analyst,
        UserRole::Trader,
        UserRole::Admin,
        UserRole::SuperAdmin,
    ];
    
    let user_level = role_hierarchy.iter().position(|r| r == user_role).unwrap_or(0);
    let required_level = role_hierarchy.iter().position(|r| r == required_role).unwrap_or(0);
    
    user_level >= required_level
}

/// 获取角色对应的权限列表
fn get_role_permissions(role: &UserRole) -> Vec<String> {
    match role {
        UserRole::SuperAdmin => vec![
            "system:read".to_string(),
            "system:write".to_string(),
            "system:admin".to_string(),
            "users:read".to_string(),
            "users:write".to_string(),
            "users:admin".to_string(),
            "trading:read".to_string(),
            "trading:write".to_string(),
            "trading:admin".to_string(),
            "monitoring:read".to_string(),
            "monitoring:write".to_string(),
            "monitoring:admin".to_string(),
            "dashboard:read".to_string(),
            "dashboard:write".to_string(),
            "dashboard:admin".to_string(),
        ],
        UserRole::Admin => vec![
            "system:read".to_string(),
            "users:read".to_string(),
            "users:write".to_string(),
            "trading:read".to_string(),
            "trading:write".to_string(),
            "monitoring:read".to_string(),
            "monitoring:write".to_string(),
            "dashboard:read".to_string(),
            "dashboard:write".to_string(),
        ],
        UserRole::Trader => vec![
            "trading:read".to_string(),
            "trading:write".to_string(),
            "monitoring:read".to_string(),
            "dashboard:read".to_string(),
            "dashboard:write".to_string(),
        ],
        UserRole::Analyst => vec![
            "trading:read".to_string(),
            "monitoring:read".to_string(),
            "dashboard:read".to_string(),
        ],
        UserRole::Viewer => vec![
            "monitoring:read".to_string(),
            "dashboard:read".to_string(),
        ],
    }
}

/// 验证API密钥
async fn is_valid_api_key(api_key: &str) -> bool {
    // 在实际应用中，这里应该查询数据库或配置文件
    // 现在使用硬编码的密钥作为示例
    const VALID_API_KEYS: &[&str] = &[
        "arbitrage_system_key_2024",
        "qingxi_service_key_2024",
        "dashboard_service_key_2024",
        "monitoring_service_key_2024",
    ];
    
    VALID_API_KEYS.contains(&api_key)
}

/// 获取客户端IP地址
fn get_client_ip(request: &Request) -> String {
    // 1. 检查X-Forwarded-For头（反向代理）
    if let Some(forwarded_for) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    // 2. 检查X-Real-IP头（Nginx等）
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // 3. 检查连接信息（如果可用）
    // 在实际应用中，这里需要从连接信息中获取IP
    "unknown".to_string()
}

/// 认证中间件函数别名，用于简化使用
pub type RequireAuth = fn(Request, Next) -> Result<Response, StatusCode>;
pub type RequireRole = fn(UserRole) -> fn(Request, Next) -> Result<Response, StatusCode>;

/// 创建认证中间件的工厂函数
pub fn create_auth_middleware(
    jwt_secret: String, 
    auth_service: Arc<dyn crate::services::AuthServiceTrait>
) -> Arc<AuthMiddleware> {
    Arc::new(AuthMiddleware::new(jwt_secret, auth_service))
}

/// 检查当前用户是否有指定权限的辅助函数
pub fn check_permission(auth_context: &AuthContext, permission: &str) -> bool {
    auth_context.permissions.contains(&permission.to_string())
}

/// 检查当前用户是否是管理员的辅助函数
pub fn is_admin(auth_context: &AuthContext) -> bool {
    auth_context.is_admin
}

/// 获取当前认证用户信息的辅助函数
pub fn get_current_user(request: &Request) -> Option<&AuthContext> {
    request.extensions().get::<AuthContext>()
}