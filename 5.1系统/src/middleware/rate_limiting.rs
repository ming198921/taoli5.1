//! 速率限制中间件
//! 
//! 实现基于IP、用户和API密钥的请求频率限制，防止滥用和DoS攻击

use axum::{
    extract::Request,
    http::{StatusCode, HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use crate::middleware::auth::AuthContext;

/// 速率限制策略
#[derive(Debug, Clone)]
pub enum RateLimitStrategy {
    /// 固定窗口：在固定时间窗口内允许固定数量的请求
    FixedWindow {
        window_size: Duration,
        max_requests: u32,
    },
    /// 滑动窗口：在滑动时间窗口内允许固定数量的请求
    SlidingWindow {
        window_size: Duration,
        max_requests: u32,
    },
    /// 令牌桶：以固定速率补充令牌，每个请求消耗一个令牌
    TokenBucket {
        capacity: u32,
        refill_rate: u32, // 每秒补充的令牌数
    },
    /// 漏桶：以固定速率处理请求
    LeakyBucket {
        capacity: u32,
        leak_rate: u32, // 每秒处理的请求数
    },
}

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub default_strategy: RateLimitStrategy,
    pub per_ip_strategy: Option<RateLimitStrategy>,
    pub per_user_strategy: Option<RateLimitStrategy>,
    pub per_api_key_strategy: Option<RateLimitStrategy>,
    pub whitelist_ips: Vec<String>,
    pub blacklist_ips: Vec<String>,
    pub custom_limits: HashMap<String, RateLimitStrategy>, // path -> strategy
    pub burst_allowance: u32,
    pub cleanup_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_strategy: RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60), // 1分钟
                max_requests: 100, // 每分钟100个请求
            },
            per_ip_strategy: Some(RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60),
                max_requests: 60, // 每个IP每分钟60个请求
            }),
            per_user_strategy: Some(RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60),
                max_requests: 200, // 认证用户每分钟200个请求
            }),
            per_api_key_strategy: Some(RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60),
                max_requests: 1000, // API密钥每分钟1000个请求
            }),
            whitelist_ips: vec![
                "127.0.0.1".to_string(),
                "::1".to_string(),
                "localhost".to_string(),
            ],
            blacklist_ips: Vec::new(),
            custom_limits: HashMap::new(),
            burst_allowance: 10, // 允许突发10个额外请求
            cleanup_interval: Duration::from_secs(300), // 5分钟清理一次过期数据
        }
    }
}

impl RateLimitConfig {
    /// 开发环境配置（更宽松）
    pub fn development() -> Self {
        Self {
            enabled: false, // 开发环境默认禁用限流
            default_strategy: RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60),
                max_requests: 1000,
            },
            per_ip_strategy: Some(RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60),
                max_requests: 500,
            }),
            per_user_strategy: Some(RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60),
                max_requests: 2000,
            }),
            per_api_key_strategy: Some(RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(60),
                max_requests: 10000,
            }),
            whitelist_ips: vec!["*".to_string()], // 开发环境允许所有IP
            blacklist_ips: Vec::new(),
            custom_limits: HashMap::new(),
            burst_allowance: 50,
            cleanup_interval: Duration::from_secs(600),
        }
    }

    /// 生产环境配置（更严格）
    pub fn production() -> Self {
        let mut custom_limits = HashMap::new();
        
        // 为不同的端点设置不同的限制
        custom_limits.insert("/api/auth/login".to_string(), RateLimitStrategy::FixedWindow {
            window_size: Duration::from_secs(300), // 5分钟
            max_requests: 5, // 登录尝试限制更严格
        });
        
        custom_limits.insert("/api/auth/register".to_string(), RateLimitStrategy::FixedWindow {
            window_size: Duration::from_secs(3600), // 1小时
            max_requests: 3, // 注册限制更严格
        });

        Self {
            enabled: true,
            default_strategy: RateLimitStrategy::TokenBucket {
                capacity: 100,
                refill_rate: 50, // 每秒50个令牌
            },
            per_ip_strategy: Some(RateLimitStrategy::TokenBucket {
                capacity: 50,
                refill_rate: 10, // 每秒10个令牌
            }),
            per_user_strategy: Some(RateLimitStrategy::TokenBucket {
                capacity: 200,
                refill_rate: 100,
            }),
            per_api_key_strategy: Some(RateLimitStrategy::TokenBucket {
                capacity: 1000,
                refill_rate: 500,
            }),
            whitelist_ips: vec![
                "127.0.0.1".to_string(),
                "::1".to_string(),
            ],
            blacklist_ips: Vec::new(),
            custom_limits,
            burst_allowance: 5,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

/// 速率限制状态
#[derive(Debug, Clone)]
pub struct RateLimitState {
    pub requests: u32,
    pub window_start: Instant,
    pub tokens: f64, // 用于令牌桶算法
    pub last_refill: Instant,
    pub request_times: Vec<Instant>, // 用于滑动窗口算法
}

impl RateLimitState {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            requests: 0,
            window_start: now,
            tokens: 0.0,
            last_refill: now,
            request_times: Vec::new(),
        }
    }
}

/// 速率限制器
#[derive(Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    states: Arc<RwLock<HashMap<String, RateLimitState>>>,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            states: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// 检查请求是否允许通过
    pub fn is_allowed(&self, key: &str, strategy: &RateLimitStrategy) -> (bool, RateLimitInfo) {
        if !self.config.enabled {
            return (true, RateLimitInfo::default());
        }

        let mut states = self.states.write().unwrap();
        let state = states.entry(key.to_string()).or_insert_with(RateLimitState::new);
        
        let (allowed, info) = match strategy {
            RateLimitStrategy::FixedWindow { window_size, max_requests } => {
                self.check_fixed_window(state, *window_size, *max_requests)
            },
            RateLimitStrategy::SlidingWindow { window_size, max_requests } => {
                self.check_sliding_window(state, *window_size, *max_requests)
            },
            RateLimitStrategy::TokenBucket { capacity, refill_rate } => {
                self.check_token_bucket(state, *capacity, *refill_rate)
            },
            RateLimitStrategy::LeakyBucket { capacity, leak_rate } => {
                self.check_leaky_bucket(state, *capacity, *leak_rate)
            },
        };

        // 定期清理过期状态
        self.cleanup_if_needed();

        (allowed, info)
    }

    fn check_fixed_window(&self, state: &mut RateLimitState, window_size: Duration, max_requests: u32) -> (bool, RateLimitInfo) {
        let now = Instant::now();
        
        // 检查是否需要重置窗口
        if now.duration_since(state.window_start) >= window_size {
            state.requests = 0;
            state.window_start = now;
        }
        
        let allowed = state.requests < max_requests;
        if allowed {
            state.requests += 1;
        }
        
        let remaining = max_requests.saturating_sub(state.requests);
        let reset_time = state.window_start + window_size;
        let retry_after = if allowed { 0 } else { 
            reset_time.duration_since(now).as_secs() 
        };
        
        (allowed, RateLimitInfo {
            limit: max_requests,
            remaining,
            reset_time: reset_time.duration_since(UNIX_EPOCH.into()).unwrap_or_default().as_secs(),
            retry_after,
        })
    }

    fn check_sliding_window(&self, state: &mut RateLimitState, window_size: Duration, max_requests: u32) -> (bool, RateLimitInfo) {
        let now = Instant::now();
        
        // 移除窗口外的请求
        state.request_times.retain(|&time| now.duration_since(time) <= window_size);
        
        let current_requests = state.request_times.len() as u32;
        let allowed = current_requests < max_requests;
        
        if allowed {
            state.request_times.push(now);
        }
        
        let remaining = max_requests.saturating_sub(current_requests + if allowed { 1 } else { 0 });
        let retry_after = if allowed { 0 } else {
            // 计算最早的请求何时会过期
            state.request_times.first()
                .map(|&earliest| window_size.saturating_sub(now.duration_since(earliest)).as_secs())
                .unwrap_or(0)
        };
        
        (allowed, RateLimitInfo {
            limit: max_requests,
            remaining,
            reset_time: (now + window_size).duration_since(UNIX_EPOCH.into()).unwrap_or_default().as_secs(),
            retry_after,
        })
    }

    fn check_token_bucket(&self, state: &mut RateLimitState, capacity: u32, refill_rate: u32) -> (bool, RateLimitInfo) {
        let now = Instant::now();
        let time_passed = now.duration_since(state.last_refill).as_secs_f64();
        
        // 初始化令牌桶（首次使用时）
        if state.tokens == 0.0 && state.last_refill == state.window_start {
            state.tokens = capacity as f64;
        }
        
        // 补充令牌
        state.tokens = (state.tokens + refill_rate as f64 * time_passed).min(capacity as f64);
        state.last_refill = now;
        
        let allowed = state.tokens >= 1.0;
        if allowed {
            state.tokens -= 1.0;
        }
        
        let remaining = state.tokens.floor() as u32;
        let retry_after = if allowed { 0 } else {
            (1.0 / refill_rate as f64) as u64 // 等待下一个令牌的时间
        };
        
        (allowed, RateLimitInfo {
            limit: capacity,
            remaining,
            reset_time: 0, // 令牌桶没有固定的重置时间
            retry_after,
        })
    }

    fn check_leaky_bucket(&self, state: &mut RateLimitState, capacity: u32, leak_rate: u32) -> (bool, RateLimitInfo) {
        let now = Instant::now();
        let time_passed = now.duration_since(state.last_refill).as_secs_f64();
        
        // 漏掉一些请求
        let leaked = (leak_rate as f64 * time_passed) as u32;
        state.requests = state.requests.saturating_sub(leaked);
        state.last_refill = now;
        
        let allowed = state.requests < capacity;
        if allowed {
            state.requests += 1;
        }
        
        let remaining = capacity.saturating_sub(state.requests);
        let retry_after = if allowed { 0 } else {
            (1.0 / leak_rate as f64) as u64
        };
        
        (allowed, RateLimitInfo {
            limit: capacity,
            remaining,
            reset_time: 0, // 漏桶没有固定的重置时间
            retry_after,
        })
    }

    fn cleanup_if_needed(&self) {
        let mut last_cleanup = self.last_cleanup.write().unwrap();
        let now = Instant::now();
        
        if now.duration_since(*last_cleanup) >= self.config.cleanup_interval {
            let mut states = self.states.write().unwrap();
            let cutoff = now - self.config.cleanup_interval * 2; // 清理2个间隔前的数据
            
            states.retain(|_, state| {
                // 保留最近活跃的状态
                state.window_start > cutoff || state.last_refill > cutoff
            });
            
            *last_cleanup = now;
        }
    }

    fn get_client_ip(&self, request: &Request) -> String {
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
        
        "unknown".to_string()
    }

    fn is_ip_whitelisted(&self, ip: &str) -> bool {
        self.config.whitelist_ips.contains(&"*".to_string()) 
            || self.config.whitelist_ips.contains(&ip.to_string())
    }

    fn is_ip_blacklisted(&self, ip: &str) -> bool {
        self.config.blacklist_ips.contains(&ip.to_string())
    }
}

/// 速率限制信息
#[derive(Debug, Clone, Default)]
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_time: u64,
    pub retry_after: u64,
}

/// 速率限制中间件函数
pub async fn rate_limit(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = RateLimiter::new(RateLimitConfig::default());
    rate_limit_with_config(rate_limiter, request, next).await
}

/// 带配置的速率限制中间件函数
pub async fn rate_limit_with_config(
    rate_limiter: RateLimiter,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !rate_limiter.config.enabled {
        return Ok(next.run(request).await);
    }

    let client_ip = rate_limiter.get_client_ip(&request);
    
    // 检查IP黑名单
    if rate_limiter.is_ip_blacklisted(&client_ip) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // 检查IP白名单
    if rate_limiter.is_ip_whitelisted(&client_ip) {
        return Ok(next.run(request).await);
    }

    let path = request.uri().path();
    
    // 确定使用的限流策略
    let strategy = rate_limiter.config.custom_limits.get(path)
        .unwrap_or(&rate_limiter.config.default_strategy);

    // 构建限流键
    let mut limit_keys = Vec::new();
    
    // 1. 基于IP的限制
    if let Some(ip_strategy) = &rate_limiter.config.per_ip_strategy {
        limit_keys.push((format!("ip:{}", client_ip), ip_strategy));
    }
    
    // 2. 基于用户的限制
    if let Some(user_strategy) = &rate_limiter.config.per_user_strategy {
        if let Some(auth_context) = request.extensions().get::<AuthContext>() {
            limit_keys.push((format!("user:{}", auth_context.user_id), user_strategy));
        }
    }
    
    // 3. 基于API密钥的限制
    if let Some(api_key_strategy) = &rate_limiter.config.per_api_key_strategy {
        if let Some(api_key) = request.headers().get("x-api-key") {
            if let Ok(key_str) = api_key.to_str() {
                limit_keys.push((format!("api_key:{}", key_str), api_key_strategy));
            }
        }
    }
    
    // 4. 默认限制
    limit_keys.push((format!("default:{}", client_ip), strategy));

    // 检查所有限制
    let mut final_info = RateLimitInfo::default();
    for (key, strategy) in limit_keys {
        let (allowed, info) = rate_limiter.is_allowed(&key, strategy);
        if !allowed {
            // 添加速率限制响应头
            let mut response = Response::new("Too Many Requests".into());
            *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", HeaderValue::from_str(&info.limit.to_string()).unwrap());
            headers.insert("X-RateLimit-Remaining", HeaderValue::from_str(&info.remaining.to_string()).unwrap());
            headers.insert("X-RateLimit-Reset", HeaderValue::from_str(&info.reset_time.to_string()).unwrap());
            
            if info.retry_after > 0 {
                headers.insert("Retry-After", HeaderValue::from_str(&info.retry_after.to_string()).unwrap());
            }
            
            return Ok(response);
        }
        final_info = info; // 保存最后一个成功的信息
    }

    // 请求被允许，添加速率限制信息头
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("X-RateLimit-Limit", HeaderValue::from_str(&final_info.limit.to_string()).unwrap());
    headers.insert("X-RateLimit-Remaining", HeaderValue::from_str(&final_info.remaining.to_string()).unwrap());
    
    Ok(response)
}

/// 开发环境速率限制中间件
pub async fn rate_limit_dev(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = RateLimiter::new(RateLimitConfig::development());
    rate_limit_with_config(rate_limiter, request, next).await
}

/// 生产环境速率限制中间件
pub async fn rate_limit_prod(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = RateLimiter::new(RateLimitConfig::production());
    rate_limit_with_config(rate_limiter, request, next).await
}

/// 创建自定义速率限制中间件
pub fn create_rate_limit_middleware(config: RateLimitConfig) -> impl Fn(Request, Next) -> Result<Response, StatusCode> + Clone {
    let rate_limiter = RateLimiter::new(config);
    move |request: Request, next: Next| {
        let rate_limiter = rate_limiter.clone();
        async move {
            rate_limit_with_config(rate_limiter, request, next).await
        }
    }
}

/// 速率限制类型别名
pub type RateLimit = fn(Request, Next) -> Result<Response, StatusCode>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert!(matches!(config.default_strategy, RateLimitStrategy::FixedWindow { .. }));
    }
    
    #[test]
    fn test_fixed_window_rate_limiting() {
        let config = RateLimitConfig {
            enabled: true,
            default_strategy: RateLimitStrategy::FixedWindow {
                window_size: Duration::from_secs(1),
                max_requests: 2,
            },
            ..Default::default()
        };
        
        let limiter = RateLimiter::new(config);
        
        // 前两个请求应该被允许
        assert!(limiter.is_allowed("test", &RateLimitStrategy::FixedWindow {
            window_size: Duration::from_secs(1),
            max_requests: 2,
        }).0);
        
        assert!(limiter.is_allowed("test", &RateLimitStrategy::FixedWindow {
            window_size: Duration::from_secs(1),
            max_requests: 2,
        }).0);
        
        // 第三个请求应该被拒绝
        assert!(!limiter.is_allowed("test", &RateLimitStrategy::FixedWindow {
            window_size: Duration::from_secs(1),
            max_requests: 2,
        }).0);
    }
    
    #[test]
    fn test_token_bucket_rate_limiting() {
        let strategy = RateLimitStrategy::TokenBucket {
            capacity: 2,
            refill_rate: 1,
        };
        
        let config = RateLimitConfig {
            enabled: true,
            default_strategy: strategy.clone(),
            ..Default::default()
        };
        
        let limiter = RateLimiter::new(config);
        
        // 初始应该有2个令牌
        assert!(limiter.is_allowed("test", &strategy).0);
        assert!(limiter.is_allowed("test", &strategy).0);
        
        // 现在应该没有令牌了
        assert!(!limiter.is_allowed("test", &strategy).0);
    }
}