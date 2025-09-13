use crate::api_gateway::AuthServiceTrait;
use crate::routes;
use axum::Router;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use std::collections::HashMap;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use chrono::{DateTime, Duration, Utc};
use tracing::{info, warn, error};
use bcrypt::{hash, verify, DEFAULT_COST};
use uuid::Uuid;

/// JWT Claims结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // 用户ID
    pub username: String,   // 用户名
    pub role: String,       // 角色
    pub permissions: Vec<String>, // 权限列表
    pub exp: i64,          // 过期时间
    pub iat: i64,          // 签发时间
    pub jti: String,       // JWT ID
    pub device_id: Option<String>, // 设备ID
    pub ip_address: Option<String>, // IP地址
}

/// 用户角色枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    SuperAdmin,   // 超级管理员
    Admin,        // 管理员
    Trader,       // 交易员
    Analyst,      // 分析师
    Viewer,       // 只读用户
}

impl UserRole {
    pub fn get_permissions(&self) -> Vec<String> {
        match self {
            UserRole::SuperAdmin => vec![
                "system:*".to_string(),
                "user:*".to_string(),
                "trading:*".to_string(),
                "analytics:*".to_string(),
            ],
            UserRole::Admin => vec![
                "system:read".to_string(),
                "system:config".to_string(),
                "user:read".to_string(),
                "user:create".to_string(),
                "user:update".to_string(),
                "trading:*".to_string(),
                "analytics:*".to_string(),
            ],
            UserRole::Trader => vec![
                "trading:read".to_string(),
                "trading:execute".to_string(),
                "analytics:read".to_string(),
                "system:read".to_string(),
            ],
            UserRole::Analyst => vec![
                "analytics:*".to_string(),
                "system:read".to_string(),
                "trading:read".to_string(),
            ],
            UserRole::Viewer => vec![
                "system:read".to_string(),
                "trading:read".to_string(),
                "analytics:read".to_string(),
            ],
        }
    }
    
    pub fn as_string(&self) -> String {
        match self {
            UserRole::SuperAdmin => "super_admin".to_string(),
            UserRole::Admin => "admin".to_string(),
            UserRole::Trader => "trader".to_string(),
            UserRole::Analyst => "analyst".to_string(),
            UserRole::Viewer => "viewer".to_string(),
        }
    }
    
    pub fn from_string(s: &str) -> Self {
        match s {
            "super_admin" => UserRole::SuperAdmin,
            "admin" => UserRole::Admin,
            "trader" => UserRole::Trader,
            "analyst" => UserRole::Analyst,
            "viewer" => UserRole::Viewer,
            _ => UserRole::Viewer,
        }
    }
}

/// 用户状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Locked,
    PendingActivation,
}

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub login_attempts: u32,
    pub last_failed_login: Option<DateTime<Utc>>,
    pub password_changed_at: DateTime<Utc>,
    pub two_factor_enabled: bool,
    pub two_factor_secret: Option<String>,
    pub profile: UserProfile,
}

/// 用户资料
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub department: Option<String>,
    pub position: Option<String>,
    pub timezone: String,
    pub language: String,
    pub notification_preferences: NotificationPreferences,
}

/// 通知偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_alerts: bool,
    pub sms_alerts: bool,
    pub push_notifications: bool,
    pub trading_alerts: bool,
    pub system_alerts: bool,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            first_name: None,
            last_name: None,
            phone: None,
            department: None,
            position: None,
            timezone: "UTC".to_string(),
            language: "zh-CN".to_string(),
            notification_preferences: NotificationPreferences {
                email_alerts: true,
                sms_alerts: false,
                push_notifications: true,
                trading_alerts: true,
                system_alerts: true,
            },
        }
    }
}

/// 用户会话
#[derive(Debug, Clone)]
pub struct UserSession {
    pub session_id: String,
    pub user_id: String,
    pub username: String,
    pub role: UserRole,
    pub token: String,
    pub refresh_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub is_active: bool,
}

/// 登录尝试记录
#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub username: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub failure_reason: Option<String>,
}

/// 认证服务配置
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub refresh_token_expiry_days: i64,
    pub max_sessions_per_user: usize,
    pub enable_refresh_token: bool,
    pub password_min_length: usize,
    pub password_require_uppercase: bool,
    pub password_require_lowercase: bool,
    pub password_require_numbers: bool,
    pub password_require_symbols: bool,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: i64,
    pub session_timeout_minutes: i64,
    pub enable_two_factor: bool,
    pub rate_limit_attempts: u32,
    pub rate_limit_window_minutes: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "5d6f7c8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c".to_string()),
            jwt_expiry_hours: 8,
            refresh_token_expiry_days: 30,
            max_sessions_per_user: 3,
            enable_refresh_token: true,
            password_min_length: 12,
            password_require_uppercase: true,
            password_require_lowercase: true,
            password_require_numbers: true,
            password_require_symbols: true,
            max_login_attempts: 5,
            lockout_duration_minutes: 30,
            session_timeout_minutes: 120,
            enable_two_factor: false,
            rate_limit_attempts: 10,
            rate_limit_window_minutes: 15,
        }
    }
}

/// 认证服务实现
pub struct AuthService {
    config: AuthConfig,
    users: Arc<RwLock<HashMap<String, User>>>,
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
    login_attempts: Arc<RwLock<Vec<LoginAttempt>>>,
    blocked_ips: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());
        
        let now = Utc::now();
        
        // 初始化系统用户
        let mut users = HashMap::new();
        
        // 超级管理员
        let super_admin = User {
            id: Uuid::new_v4().to_string(),
            username: "superadmin".to_string(),
            password_hash: Self::hash_password("SuperAdmin@2025!"),
            email: "superadmin@arbitrage-system.com".to_string(),
            role: UserRole::SuperAdmin,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
            last_login: None,
            login_attempts: 0,
            last_failed_login: None,
            password_changed_at: now,
            two_factor_enabled: false,
            two_factor_secret: None,
            profile: UserProfile {
                first_name: Some("Super".to_string()),
                last_name: Some("Administrator".to_string()),
                department: Some("System".to_string()),
                position: Some("Super Administrator".to_string()),
                ..Default::default()
            },
        };
        
        // 系统管理员
        let admin = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password_hash: Self::hash_password("Admin@2025!"),
            email: "admin@arbitrage-system.com".to_string(),
            role: UserRole::Admin,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
            last_login: None,
            login_attempts: 0,
            last_failed_login: None,
            password_changed_at: now,
            two_factor_enabled: false,
            two_factor_secret: None,
            profile: UserProfile {
                first_name: Some("System".to_string()),
                last_name: Some("Administrator".to_string()),
                department: Some("IT".to_string()),
                position: Some("System Administrator".to_string()),
                ..Default::default()
            },
        };
        
        // 交易员
        let trader = User {
            id: Uuid::new_v4().to_string(),
            username: "trader".to_string(),
            password_hash: Self::hash_password("Trader@2025!"),
            email: "trader@arbitrage-system.com".to_string(),
            role: UserRole::Trader,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
            last_login: None,
            login_attempts: 0,
            last_failed_login: None,
            password_changed_at: now,
            two_factor_enabled: false,
            two_factor_secret: None,
            profile: UserProfile {
                first_name: Some("Senior".to_string()),
                last_name: Some("Trader".to_string()),
                department: Some("Trading".to_string()),
                position: Some("Senior Arbitrage Trader".to_string()),
                ..Default::default()
            },
        };
        
        // 分析师
        let analyst = User {
            id: Uuid::new_v4().to_string(),
            username: "analyst".to_string(),
            password_hash: Self::hash_password("Analyst@2025!"),
            email: "analyst@arbitrage-system.com".to_string(),
            role: UserRole::Analyst,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
            last_login: None,
            login_attempts: 0,
            last_failed_login: None,
            password_changed_at: now,
            two_factor_enabled: false,
            two_factor_secret: None,
            profile: UserProfile {
                first_name: Some("Market".to_string()),
                last_name: Some("Analyst".to_string()),
                department: Some("Research".to_string()),
                position: Some("Senior Market Analyst".to_string()),
                ..Default::default()
            },
        };
        
        users.insert("superadmin".to_string(), super_admin);
        users.insert("admin".to_string(), admin);
        users.insert("trader".to_string(), trader);
        users.insert("analyst".to_string(), analyst);
        
        Self {
            config,
            users: Arc::new(RwLock::new(users)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            login_attempts: Arc::new(RwLock::new(Vec::new())),
            blocked_ips: Arc::new(RwLock::new(HashMap::new())),
            encoding_key,
            decoding_key,
        }
    }
    
    /// 用户登录
    pub async fn login(&self, username: &str, password: &str, ip_address: Option<String>, user_agent: Option<String>) -> Result<UserSession, String> {
        // 检查IP是否被封禁
        if let Some(ip) = &ip_address {
            if self.is_ip_blocked(ip).await {
                return Err("IP地址已被临时封禁，请稍后再试".to_string());
            }
        }
        
        // 检查登录频率限制
        if self.is_rate_limited(username, &ip_address).await {
            return Err("登录尝试过于频繁，请稍后再试".to_string());
        }
        
        let users = self.users.read().await;
        
        if let Some(user) = users.get(username) {
            // 检查用户状态
            match user.status {
                UserStatus::Active => {},
                UserStatus::Inactive => {
                    self.record_login_attempt(username, &ip_address, &user_agent, false, Some("账号未激活")).await;
                    return Err("用户账号未激活".to_string());
                },
                UserStatus::Suspended => {
                    self.record_login_attempt(username, &ip_address, &user_agent, false, Some("账号已被暂停")).await;
                    return Err("用户账号已被暂停".to_string());
                },
                UserStatus::Locked => {
                    self.record_login_attempt(username, &ip_address, &user_agent, false, Some("账号已被锁定")).await;
                    return Err("用户账号已被锁定".to_string());
                },
                UserStatus::PendingActivation => {
                    self.record_login_attempt(username, &ip_address, &user_agent, false, Some("账号等待激活")).await;
                    return Err("用户账号等待激活".to_string());
                },
            }
            
            // 检查账号是否因登录失败次数过多而被锁定
            if user.login_attempts >= self.config.max_login_attempts {
                if let Some(last_failed) = user.last_failed_login {
                    let lockout_duration = Duration::minutes(self.config.lockout_duration_minutes);
                    if Utc::now() - last_failed < lockout_duration {
                        self.record_login_attempt(username, &ip_address, &user_agent, false, Some("账号因多次失败被锁定")).await;
                        return Err("账号因多次登录失败被暂时锁定，请稍后再试".to_string());
                    }
                }
            }
            
            // 验证密码
            if self.verify_password(password, &user.password_hash) {
                drop(users);
                
                // 创建会话
                let session = self.create_session_with_details(user, ip_address.clone(), user_agent.clone()).await?;
                
                // 更新用户信息
                let mut users_mut = self.users.write().await;
                if let Some(user_mut) = users_mut.get_mut(username) {
                    user_mut.last_login = Some(Utc::now());
                    user_mut.login_attempts = 0; // 重置失败计数
                    user_mut.last_failed_login = None;
                }
                
                // 记录成功登录
                self.record_login_attempt(username, &ip_address, &user_agent, true, None).await;
                
                info!("用户 {} 登录成功，IP: {:?}", username, ip_address);
                Ok(session)
            } else {
                // 密码错误，增加失败计数
                let mut users_mut = self.users.write().await;
                if let Some(user_mut) = users_mut.get_mut(username) {
                    user_mut.login_attempts += 1;
                    user_mut.last_failed_login = Some(Utc::now());
                }
                
                self.record_login_attempt(username, &ip_address, &user_agent, false, Some("密码错误")).await;
                
                warn!("用户 {} 登录失败：密码错误，IP: {:?}", username, ip_address);
                Err("用户名或密码错误".to_string())
            }
        } else {
            self.record_login_attempt(username, &ip_address, &user_agent, false, Some("用户不存在")).await;
            warn!("登录失败：用户 {} 不存在，IP: {:?}", username, ip_address);
            Err("用户名或密码错误".to_string())
        }
    }
    
    /// 验证JWT令牌
    pub async fn verify_token(&self, token: &str) -> Result<Claims, String> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_nbf = true;
        
        match decode::<Claims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                let sessions = self.sessions.read().await;
                if let Some(session) = sessions.get(token) {
                    if session.is_active && session.expires_at > Utc::now() {
                        // 更新最后活动时间
                        drop(sessions);
                        let mut sessions_mut = self.sessions.write().await;
                        if let Some(session_mut) = sessions_mut.get_mut(token) {
                            session_mut.last_activity = Utc::now();
                        }
                        
                        Ok(token_data.claims)
                    } else {
                        Err("会话已过期或无效".to_string())
                    }
                } else {
                    Err("会话不存在".to_string())
                }
            }
            Err(err) => {
                error!("JWT验证失败: {}", err);
                Err("令牌验证失败".to_string())
            }
        }
    }
    
    /// 刷新令牌
    pub async fn refresh_token(&self, token: &str) -> Result<UserSession, String> {
        let sessions = self.sessions.read().await;
        
        if let Some(session) = sessions.get(token) {
            if !session.is_active {
                return Err("会话已失效".to_string());
            }
            
            let user_id = session.user_id.clone();
            let ip_address = session.ip_address.clone();
            let user_agent = session.user_agent.clone();
            
            drop(sessions);
            
            // 获取用户信息
            let users = self.users.read().await;
            if let Some(user) = users.values().find(|u| u.id == user_id) {
                if user.status != UserStatus::Active {
                    return Err("用户状态不允许刷新令牌".to_string());
                }
                
                // 撤销旧令牌
                self.revoke_session(token).await?;
                
                // 创建新会话
                let new_session = self.create_session_with_details(user, ip_address, user_agent).await?;
                
                info!("用户 {} 成功刷新令牌", user.username);
                Ok(new_session)
            } else {
                Err("用户不存在".to_string())
            }
        } else {
            Err("令牌不存在".to_string())
        }
    }
    
    /// 用户登出
    pub async fn logout(&self, token: &str) -> Result<(), String> {
        self.revoke_session(token).await
    }
    
    /// 哈希密码
    fn hash_password(password: &str) -> String {
        hash(password, DEFAULT_COST).expect("密码哈希失败")
    }
    
    /// 验证密码
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }
    
    /// 获取用户信息
    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        let users = self.users.read().await;
        users.values().find(|u| u.id == user_id).cloned()
    }
    
    /// 通过用户名获取用户
    pub async fn get_user_by_username(&self, username: &str) -> Option<User> {
        let users = self.users.read().await;
        users.get(username).cloned()
    }
    
    /// 获取用户资料
    pub async fn get_user_profile(&self, user_id: &str) -> Option<serde_json::Value> {
        if let Some(user) = self.get_user(user_id).await {
            Some(serde_json::json!({
                "id": user.id,
                "username": user.username,
                "email": user.email,
                "role": user.role,
                "status": user.status,
                "created_at": user.created_at,
                "last_login": user.last_login,
                "profile": user.profile,
                "two_factor_enabled": user.two_factor_enabled
            }))
        } else {
            None
        }
    }
}

// 私有方法实现 
impl AuthService {
    /// 创建带详细信息的用户会话
    async fn create_session_with_details(&self, user: &User, ip_address: Option<String>, user_agent: Option<String>) -> Result<UserSession, String> {
        let now = Utc::now();
        let expires_at = now + Duration::hours(self.config.jwt_expiry_hours);
        let session_id = Uuid::new_v4().to_string();
        let jti = Uuid::new_v4().to_string();
        
        let permissions = user.role.get_permissions();
        
        let claims = Claims {
            sub: user.id.clone(),
            username: user.username.clone(),
            role: user.role.as_string(),
            permissions: permissions.clone(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            jti: jti.clone(),
            device_id: None,
            ip_address: ip_address.clone(),
        };
        
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| format!("JWT编码失败: {}", e))?;
        
        let refresh_token = if self.config.enable_refresh_token {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        };
        
        let session = UserSession {
            session_id: session_id.clone(),
            user_id: user.id.clone(),
            username: user.username.clone(),
            role: user.role.clone(),
            token: token.clone(),
            refresh_token,
            created_at: now,
            expires_at,
            last_activity: now,
            ip_address,
            user_agent,
            device_id: None,
            is_active: true,
        };
        
        // 清理该用户的过期会话
        self.cleanup_user_sessions(&user.id).await;
        
        // 检查会话数量限制
        let sessions = self.sessions.read().await;
        let user_sessions_count = sessions.values()
            .filter(|s| s.user_id == user.id && s.is_active)
            .count();
            
        if user_sessions_count >= self.config.max_sessions_per_user {
            drop(sessions);
            // 删除最旧的会话
            self.remove_oldest_user_session(&user.id).await;
        } else {
            drop(sessions);
        }
        
        // 存储会话
        let mut sessions_mut = self.sessions.write().await;
        sessions_mut.insert(token.clone(), session.clone());
        
        info!("为用户 {} 创建新会话，会话ID: {}", user.username, session_id);
        Ok(session)
    }
    
    /// 清理用户过期会话
    async fn cleanup_user_sessions(&self, user_id: &str) {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        
        sessions.retain(|_, session| {
            if session.user_id == user_id {
                session.expires_at > now && session.is_active
            } else {
                true
            }
        });
    }
    
    /// 删除用户最旧的会话
    async fn remove_oldest_user_session(&self, user_id: &str) {
        let mut sessions = self.sessions.write().await;
        
        let mut user_sessions: Vec<(String, UserSession)> = sessions
            .iter()
            .filter(|(_, s)| s.user_id == user_id && s.is_active)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        
        if !user_sessions.is_empty() {
            // 按创建时间排序，删除最旧的
            user_sessions.sort_by_key(|(_, s)| s.created_at);
            let oldest_token = &user_sessions[0].0;
            sessions.remove(oldest_token);
            
            info!("删除用户 {} 最旧的会话", user_id);
        }
    }
    
    /// 撤销会话
    async fn revoke_session(&self, token: &str) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(token) {
            session.is_active = false;
            info!("撤销会话，用户: {}", session.username);
            Ok(())
        } else {
            Err("会话不存在".to_string())
        }
    }
    
    /// 检查IP是否被封禁
    async fn is_ip_blocked(&self, ip: &str) -> bool {
        let blocked_ips = self.blocked_ips.read().await;
        if let Some(blocked_until) = blocked_ips.get(ip) {
            *blocked_until > Utc::now()
        } else {
            false
        }
    }
    
    /// 检查登录频率限制
    async fn is_rate_limited(&self, username: &str, ip_address: &Option<String>) -> bool {
        let attempts = self.login_attempts.read().await;
        let cutoff_time = Utc::now() - Duration::minutes(self.config.rate_limit_window_minutes);
        
        let recent_attempts = attempts
            .iter()
            .filter(|attempt| {
                attempt.timestamp > cutoff_time && 
                (attempt.username == username || 
                 (ip_address.is_some() && attempt.ip_address == *ip_address))
            })
            .count();
        
        recent_attempts >= self.config.rate_limit_attempts as usize
    }
    
    /// 记录登录尝试
    async fn record_login_attempt(&self, username: &str, ip_address: &Option<String>, user_agent: &Option<String>, success: bool, failure_reason: Option<&str>) {
        let attempt = LoginAttempt {
            username: username.to_string(),
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            timestamp: Utc::now(),
            success,
            failure_reason: failure_reason.map(|s| s.to_string()),
        };
        
        let mut attempts = self.login_attempts.write().await;
        attempts.push(attempt);
        
        // 保持记录数量在合理范围内
        const MAX_ATTEMPTS_RECORDS: usize = 10000;
        if attempts.len() > MAX_ATTEMPTS_RECORDS {
            attempts.drain(0..attempts.len() - MAX_ATTEMPTS_RECORDS);
        }
        
        // 如果失败次数过多，封禁IP
        if !success && ip_address.is_some() {
            self.check_and_block_ip(ip_address.as_ref().unwrap()).await;
        }
    }
    
    /// 检查并封禁IP
    async fn check_and_block_ip(&self, ip: &str) {
        let attempts = self.login_attempts.read().await;
        let recent_time = Utc::now() - Duration::minutes(15);
        
        let recent_failures = attempts
            .iter()
            .filter(|attempt| {
                attempt.timestamp > recent_time &&
                attempt.ip_address.as_deref() == Some(ip) &&
                !attempt.success
            })
            .count();
        
        if recent_failures >= 10 {
            drop(attempts);
            let mut blocked_ips = self.blocked_ips.write().await;
            let block_until = Utc::now() + Duration::hours(1);
            blocked_ips.insert(ip.to_string(), block_until);
            
            warn!("IP {} 因多次失败登录被封禁至 {}", ip, block_until);
        }
    }
}

impl AuthServiceTrait for AuthService {
    fn get_router(&self) -> Router {
        routes::auth::routes(Arc::new(self.clone()))
    }
}

impl Clone for AuthService {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            users: Arc::clone(&self.users),
            sessions: Arc::clone(&self.sessions),
            login_attempts: Arc::clone(&self.login_attempts),
            blocked_ips: Arc::clone(&self.blocked_ips),
            encoding_key: self.encoding_key.clone(),
            decoding_key: self.decoding_key.clone(),
        }
    }
}