//! 指标收集中间件
//! 
//! 自动收集HTTP请求的性能指标和统计信息，用于监控和分析

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use serde::{Serialize, Deserialize};
use crate::middleware::auth::AuthContext;

/// 指标类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// HTTP指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMetrics {
    pub request_count: u64,
    pub response_times: Vec<Duration>,
    pub status_codes: HashMap<u16, u64>,
    pub methods: HashMap<String, u64>,
    pub paths: HashMap<String, u64>,
    pub user_agents: HashMap<String, u64>,
    pub errors: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub concurrent_requests: u64,
    pub last_request_time: Option<std::time::SystemTime>,
}

impl Default for HttpMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            response_times: Vec::new(),
            status_codes: HashMap::new(),
            methods: HashMap::new(),
            paths: HashMap::new(),
            user_agents: HashMap::new(),
            errors: 0,
            bytes_sent: 0,
            bytes_received: 0,
            concurrent_requests: 0,
            last_request_time: None,
        }
    }
}

/// 业务指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub active_users: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub total_volume: f64,
    pub profit_loss: f64,
    pub arbitrage_opportunities: u64,
    pub api_key_usage: HashMap<String, u64>,
    pub endpoint_usage: HashMap<String, u64>,
}

impl Default for BusinessMetrics {
    fn default() -> Self {
        Self {
            active_users: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_volume: 0.0,
            profit_loss: 0.0,
            arbitrage_opportunities: 0,
            api_key_usage: HashMap::new(),
            endpoint_usage: HashMap::new(),
        }
    }
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub cache_hit_rate: f64,
    pub database_query_time: Duration,
    pub external_api_time: Duration,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time: Duration::ZERO,
            p95_response_time: Duration::ZERO,
            p99_response_time: Duration::ZERO,
            throughput_rps: 0.0,
            error_rate: 0.0,
            cache_hit_rate: 0.0,
            database_query_time: Duration::ZERO,
            external_api_time: Duration::ZERO,
        }
    }
}

/// 指标收集器配置
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collect_detailed_metrics: bool,
    pub collect_user_metrics: bool,
    pub collect_business_metrics: bool,
    pub max_response_time_samples: usize,
    pub metrics_retention_duration: Duration,
    pub excluded_paths: Vec<String>,
    pub track_user_agents: bool,
    pub track_ip_addresses: bool,
    pub performance_percentiles: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collect_detailed_metrics: true,
            collect_user_metrics: true,
            collect_business_metrics: true,
            max_response_time_samples: 10000,
            metrics_retention_duration: Duration::from_secs(3600), // 1小时
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/favicon.ico".to_string(),
            ],
            track_user_agents: true,
            track_ip_addresses: false, // 隐私考虑
            performance_percentiles: vec![50.0, 90.0, 95.0, 99.0, 99.9],
        }
    }
}

impl MetricsConfig {
    pub fn production() -> Self {
        Self {
            enabled: true,
            collect_detailed_metrics: false, // 生产环境减少详细指标收集
            collect_user_metrics: false, // 隐私考虑
            collect_business_metrics: true,
            max_response_time_samples: 5000, // 减少内存使用
            metrics_retention_duration: Duration::from_secs(1800), // 30分钟
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
                "/status".to_string(),
                "/favicon.ico".to_string(),
                "/robots.txt".to_string(),
            ],
            track_user_agents: false,
            track_ip_addresses: false,
            performance_percentiles: vec![95.0, 99.0],
        }
    }

    pub fn development() -> Self {
        Self {
            enabled: true,
            collect_detailed_metrics: true,
            collect_user_metrics: true,
            collect_business_metrics: true,
            max_response_time_samples: 1000, // 开发环境较小样本
            metrics_retention_duration: Duration::from_secs(7200), // 2小时
            excluded_paths: vec![
                "/health".to_string(),
                "/metrics".to_string(),
            ],
            track_user_agents: true,
            track_ip_addresses: true, // 开发环境可以追踪
            performance_percentiles: vec![50.0, 90.0, 95.0, 99.0, 99.9],
        }
    }
}

/// 指标收集器
pub struct MetricsCollector {
    config: MetricsConfig,
    http_metrics: Arc<RwLock<HttpMetrics>>,
    business_metrics: Arc<RwLock<BusinessMetrics>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
    custom_metrics: Arc<RwLock<HashMap<String, f64>>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            http_metrics: Arc::new(RwLock::new(HttpMetrics::default())),
            business_metrics: Arc::new(RwLock::new(BusinessMetrics::default())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            custom_metrics: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// 记录HTTP请求指标
    pub fn record_http_request(
        &self,
        method: &str,
        path: &str,
        status_code: u16,
        response_time: Duration,
        user_agent: Option<&str>,
        user_id: Option<&str>,
        bytes_sent: u64,
        bytes_received: u64,
    ) {
        if !self.config.enabled {
            return;
        }

        let mut http_metrics = self.http_metrics.write().unwrap();
        
        // 基本计数
        http_metrics.request_count += 1;
        http_metrics.last_request_time = Some(std::time::SystemTime::now());

        // 响应时间
        if http_metrics.response_times.len() < self.config.max_response_time_samples {
            http_metrics.response_times.push(response_time);
        } else {
            // 使用滑动窗口，移除最旧的样本
            http_metrics.response_times.remove(0);
            http_metrics.response_times.push(response_time);
        }

        // 状态码统计
        *http_metrics.status_codes.entry(status_code).or_insert(0) += 1;

        // HTTP方法统计
        *http_metrics.methods.entry(method.to_string()).or_insert(0) += 1;

        // 路径统计（只统计路径模式，不包含具体ID）
        let path_pattern = self.normalize_path(path);
        *http_metrics.paths.entry(path_pattern).or_insert(0) += 1;

        // User-Agent统计
        if self.config.track_user_agents {
            if let Some(ua) = user_agent {
                let ua_simplified = self.simplify_user_agent(ua);
                *http_metrics.user_agents.entry(ua_simplified).or_insert(0) += 1;
            }
        }

        // 错误统计
        if status_code >= 400 {
            http_metrics.errors += 1;
        }

        // 字节统计
        http_metrics.bytes_sent += bytes_sent;
        http_metrics.bytes_received += bytes_received;

        // 更新业务指标
        if self.config.collect_business_metrics {
            self.update_business_metrics(path, status_code, user_id);
        }

        // 更新性能指标
        self.update_performance_metrics();
    }

    /// 开始请求（记录并发数）
    pub fn start_request(&self) {
        if !self.config.enabled {
            return;
        }

        let mut http_metrics = self.http_metrics.write().unwrap();
        http_metrics.concurrent_requests += 1;
    }

    /// 结束请求
    pub fn end_request(&self) {
        if !self.config.enabled {
            return;
        }

        let mut http_metrics = self.http_metrics.write().unwrap();
        http_metrics.concurrent_requests = http_metrics.concurrent_requests.saturating_sub(1);
    }

    /// 记录自定义指标
    pub fn record_custom_metric(&self, name: &str, value: f64) {
        if !self.config.enabled {
            return;
        }

        let mut custom_metrics = self.custom_metrics.write().unwrap();
        custom_metrics.insert(name.to_string(), value);
    }

    /// 增加计数器类型的自定义指标
    pub fn increment_custom_counter(&self, name: &str, increment: f64) {
        if !self.config.enabled {
            return;
        }

        let mut custom_metrics = self.custom_metrics.write().unwrap();
        *custom_metrics.entry(name.to_string()).or_insert(0.0) += increment;
    }

    /// 获取所有指标
    pub fn get_all_metrics(&self) -> AllMetrics {
        AllMetrics {
            http: self.http_metrics.read().unwrap().clone(),
            business: self.business_metrics.read().unwrap().clone(),
            performance: self.performance_metrics.read().unwrap().clone(),
            custom: self.custom_metrics.read().unwrap().clone(),
            uptime: self.start_time.elapsed(),
        }
    }

    /// 获取Prometheus格式的指标
    pub fn get_prometheus_metrics(&self) -> String {
        let all_metrics = self.get_all_metrics();
        let mut output = String::new();

        // HTTP指标
        output.push_str(&format!(
            "# HELP http_requests_total Total number of HTTP requests\n# TYPE http_requests_total counter\nhttp_requests_total {}\n\n",
            all_metrics.http.request_count
        ));

        output.push_str(&format!(
            "# HELP http_request_duration_seconds HTTP request latency\n# TYPE http_request_duration_seconds histogram\nhttp_request_duration_seconds_sum {:.3}\nhttp_request_duration_seconds_count {}\n\n",
            all_metrics.http.response_times.iter().map(|d| d.as_secs_f64()).sum::<f64>(),
            all_metrics.http.response_times.len()
        ));

        // 状态码指标
        for (status_code, count) in &all_metrics.http.status_codes {
            output.push_str(&format!(
                "http_responses_total{{status=\"{}\"}} {}\n",
                status_code, count
            ));
        }
        output.push('\n');

        // 错误率
        let error_rate = if all_metrics.http.request_count > 0 {
            all_metrics.http.errors as f64 / all_metrics.http.request_count as f64 * 100.0
        } else {
            0.0
        };
        output.push_str(&format!(
            "# HELP http_error_rate HTTP error rate percentage\n# TYPE http_error_rate gauge\nhttp_error_rate {:.2}\n\n",
            error_rate
        ));

        // 并发请求数
        output.push_str(&format!(
            "# HELP http_concurrent_requests Number of concurrent HTTP requests\n# TYPE http_concurrent_requests gauge\nhttp_concurrent_requests {}\n\n",
            all_metrics.http.concurrent_requests
        ));

        // 业务指标
        if self.config.collect_business_metrics {
            output.push_str(&format!(
                "# HELP business_active_users Number of active users\n# TYPE business_active_users gauge\nbusiness_active_users {}\n\n",
                all_metrics.business.active_users
            ));

            output.push_str(&format!(
                "# HELP business_trades_total Total number of trades\n# TYPE business_trades_total counter\nbusiness_trades_successful_total {}\nbusiness_trades_failed_total {}\n\n",
                all_metrics.business.successful_trades,
                all_metrics.business.failed_trades
            ));
        }

        // 自定义指标
        for (name, value) in &all_metrics.custom {
            output.push_str(&format!(
                "# HELP {} Custom metric\n# TYPE {} gauge\n{} {:.2}\n\n",
                name, name, name, value
            ));
        }

        output
    }

    /// 规范化路径（移除ID等变量部分）
    fn normalize_path(&self, path: &str) -> String {
        // 简单的路径规范化，将数字ID替换为占位符
        let mut normalized = path.to_string();
        
        // 替换UUID模式
        let uuid_regex = regex::Regex::new(r"[a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12}").unwrap();
        normalized = uuid_regex.replace_all(&normalized, ":id").to_string();
        
        // 替换数字ID
        let id_regex = regex::Regex::new(r"/\d+(/|$)").unwrap();
        normalized = id_regex.replace_all(&normalized, "/:id$1").to_string();
        
        normalized
    }

    /// 简化User-Agent字符串
    fn simplify_user_agent(&self, user_agent: &str) -> String {
        if user_agent.contains("Chrome") {
            "Chrome".to_string()
        } else if user_agent.contains("Firefox") {
            "Firefox".to_string()
        } else if user_agent.contains("Safari") && !user_agent.contains("Chrome") {
            "Safari".to_string()
        } else if user_agent.contains("Edge") {
            "Edge".to_string()
        } else if user_agent.contains("curl") {
            "curl".to_string()
        } else if user_agent.contains("Postman") {
            "Postman".to_string()
        } else {
            "Other".to_string()
        }
    }

    /// 更新业务指标
    fn update_business_metrics(&self, path: &str, status_code: u16, user_id: Option<&str>) {
        let mut business_metrics = self.business_metrics.write().unwrap();
        
        // 统计活跃用户
        if let Some(_user_id) = user_id {
            // 在实际实现中，这里应该使用更复杂的逻辑来跟踪唯一活跃用户
            business_metrics.active_users += 1;
        }

        // 统计交易相关指标
        if path.contains("/api/trades/") || path.contains("/api/arbitrage/") {
            if status_code < 400 {
                business_metrics.successful_trades += 1;
            } else {
                business_metrics.failed_trades += 1;
            }
        }

        // 统计端点使用情况
        let endpoint = self.normalize_path(path);
        *business_metrics.endpoint_usage.entry(endpoint).or_insert(0) += 1;
    }

    /// 更新性能指标
    fn update_performance_metrics(&self) {
        let http_metrics = self.http_metrics.read().unwrap();
        let mut performance_metrics = self.performance_metrics.write().unwrap();

        if !http_metrics.response_times.is_empty() {
            // 计算平均响应时间
            let total_time: Duration = http_metrics.response_times.iter().sum();
            performance_metrics.avg_response_time = total_time / http_metrics.response_times.len() as u32;

            // 计算百分位数
            let mut sorted_times = http_metrics.response_times.clone();
            sorted_times.sort();

            let len = sorted_times.len();
            if len > 0 {
                performance_metrics.p95_response_time = sorted_times[(len as f64 * 0.95) as usize];
                performance_metrics.p99_response_time = sorted_times[(len as f64 * 0.99) as usize];
            }

            // 计算吞吐量（请求/秒）
            let uptime_secs = self.start_time.elapsed().as_secs_f64();
            if uptime_secs > 0.0 {
                performance_metrics.throughput_rps = http_metrics.request_count as f64 / uptime_secs;
            }

            // 计算错误率
            if http_metrics.request_count > 0 {
                performance_metrics.error_rate = http_metrics.errors as f64 / http_metrics.request_count as f64;
            }
        }
    }

    /// 检查路径是否应该被排除
    fn should_exclude_path(&self, path: &str) -> bool {
        self.config.excluded_paths.iter().any(|excluded| path.starts_with(excluded))
    }
}

/// 所有指标的聚合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllMetrics {
    pub http: HttpMetrics,
    pub business: BusinessMetrics,
    pub performance: PerformanceMetrics,
    pub custom: HashMap<String, f64>,
    pub uptime: Duration,
}

/// 指标收集中间件函数
pub async fn collect_metrics(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let collector = MetricsCollector::new(MetricsConfig::default());
    collect_metrics_with_config(collector, request, next).await
}

/// 带配置的指标收集中间件函数
pub async fn collect_metrics_with_config(
    collector: MetricsCollector,
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    if !collector.config.enabled {
        return Ok(next.run(request).await);
    }

    let path = request.uri().path();
    
    // 检查是否应该排除此路径
    if collector.should_exclude_path(path) {
        return Ok(next.run(request).await);
    }

    let method = request.method().to_string();
    let user_agent = request.headers().get("user-agent")
        .and_then(|h| h.to_str().ok());
    
    // 从认证上下文获取用户信息
    let user_id = request.extensions().get::<AuthContext>()
        .map(|ctx| ctx.user_id.as_str());

    // 估算请求大小
    let request_size = request.body().size_hint().lower() as u64;

    // 开始请求计时
    collector.start_request();
    let start_time = Instant::now();

    // 执行请求
    let response = next.run(request).await;
    
    // 结束请求
    collector.end_request();
    let response_time = start_time.elapsed();

    // 估算响应大小
    let response_size = response.body().size_hint().lower() as u64;
    let status_code = response.status().as_u16();

    // 记录指标
    collector.record_http_request(
        &method,
        path,
        status_code,
        response_time,
        user_agent,
        user_id,
        response_size,
        request_size,
    );

    Ok(response)
}

/// 开发环境指标收集中间件
pub async fn collect_metrics_dev(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let collector = MetricsCollector::new(MetricsConfig::development());
    collect_metrics_with_config(collector, request, next).await
}

/// 生产环境指标收集中间件
pub async fn collect_metrics_prod(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let collector = MetricsCollector::new(MetricsConfig::production());
    collect_metrics_with_config(collector, request, next).await
}

/// 创建自定义指标收集中间件
pub fn create_metrics_middleware(config: MetricsConfig) -> impl Fn(Request, Next) -> Result<Response, axum::http::StatusCode> + Clone {
    let collector = MetricsCollector::new(config);
    move |request: Request, next: Next| {
        let collector = collector.clone();
        async move {
            collect_metrics_with_config(collector, request, next).await
        }
    }
}

/// 全局指标收集器实例
static GLOBAL_COLLECTOR: std::sync::OnceLock<Arc<MetricsCollector>> = std::sync::OnceLock::new();

/// 获取全局指标收集器
pub fn get_global_metrics_collector() -> Option<&'static Arc<MetricsCollector>> {
    GLOBAL_COLLECTOR.get()
}

/// 初始化全局指标收集器
pub fn init_global_metrics_collector(config: MetricsConfig) {
    let collector = Arc::new(MetricsCollector::new(config));
    GLOBAL_COLLECTOR.set(collector).ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metrics_config() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert!(config.collect_detailed_metrics);
        
        let prod_config = MetricsConfig::production();
        assert!(!prod_config.collect_detailed_metrics);
        assert!(!prod_config.track_user_agents);
        
        let dev_config = MetricsConfig::development();
        assert!(dev_config.collect_detailed_metrics);
        assert!(dev_config.track_user_agents);
    }
    
    #[test]
    fn test_path_normalization() {
        let collector = MetricsCollector::new(MetricsConfig::default());
        
        assert_eq!(collector.normalize_path("/api/users/123"), "/api/users/:id");
        assert_eq!(collector.normalize_path("/api/orders/456/items"), "/api/orders/:id/items");
    }
    
    #[test]
    fn test_user_agent_simplification() {
        let collector = MetricsCollector::new(MetricsConfig::default());
        
        assert_eq!(collector.simplify_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"), "Chrome");
        assert_eq!(collector.simplify_user_agent("curl/7.68.0"), "curl");
        assert_eq!(collector.simplify_user_agent("PostmanRuntime/7.28.4"), "Postman");
    }
    
    #[test]
    fn test_metrics_collection() {
        let collector = MetricsCollector::new(MetricsConfig::default());
        
        collector.record_http_request(
            "GET",
            "/api/test",
            200,
            Duration::from_millis(150),
            Some("test-agent"),
            Some("user123"),
            1024,
            512,
        );
        
        let metrics = collector.get_all_metrics();
        assert_eq!(metrics.http.request_count, 1);
        assert_eq!(metrics.http.status_codes.get(&200), Some(&1));
        assert_eq!(metrics.http.methods.get("GET"), Some(&1));
        assert_eq!(metrics.http.bytes_sent, 1024);
        assert_eq!(metrics.http.bytes_received, 512);
    }
}