//! 系统配置模块 - 消除所有硬编码
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemLimits {
    pub limits: LimitConfig,
    pub performance: PerformanceConfig,
    pub monitoring: MonitoringConfig,
    pub paths: PathConfig,
    pub version: VersionConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LimitConfig {
    pub max_supported_exchanges: usize,
    pub max_supported_symbols: usize,
    pub max_concurrent_opportunities: usize,
    pub max_order_batch_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    pub target_latency_microseconds: u64,
    pub target_throughput_ops_per_sec: u64,
    pub target_success_rate_percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    pub health_check_interval_seconds: u64,
    pub api_health_check_interval_seconds: u64,
    pub metrics_collection_interval_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathConfig {
    pub default_config_path: String,
    pub default_secrets_path: String,
    pub default_exchanges_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionConfig {
    pub version: String,
    pub build_timestamp: String,
    pub git_sha: String,
}

// 全局配置实例
static SYSTEM_CONFIG: OnceLock<Arc<RwLock<SystemLimits>>> = OnceLock::new();

/// 获取系统配置
pub fn get_system_config() -> Arc<RwLock<SystemLimits>> {
    SYSTEM_CONFIG.get_or_init(|| {
        let config_path = std::env::var("SYSTEM_CONFIG_PATH")
            .unwrap_or_else(|_| "config/system_limits.toml".to_string());
        
        let config = load_config(&config_path)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load config from {}: {}", config_path, e);
                eprintln!("Using default configuration");
                SystemLimits::default()
            });
        
        Arc::new(RwLock::new(config))
    }).clone()
}

/// 加载配置文件
fn load_config(path: &str) -> Result<SystemLimits, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: SystemLimits = toml::from_str(&content)?;
    Ok(config)
}

/// 热重载配置
pub fn reload_system_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::var("SYSTEM_CONFIG_PATH")
        .unwrap_or_else(|_| "config/system_limits.toml".to_string());
    
    let new_config = load_config(&config_path)?;
    
    if let Some(config) = SYSTEM_CONFIG.get() {
        let mut guard = config.write();
        *guard = new_config;
    }
    
    Ok(())
}

impl Default for SystemLimits {
    fn default() -> Self {
        Self {
            limits: LimitConfig {
                max_supported_exchanges: 20,
                max_supported_symbols: 50,
                max_concurrent_opportunities: 1000,
                max_order_batch_size: 50,
            },
            performance: PerformanceConfig {
                target_latency_microseconds: 500,
                target_throughput_ops_per_sec: 10000,
                target_success_rate_percent: 99.9,
            },
            monitoring: MonitoringConfig {
                health_check_interval_seconds: 30,
                api_health_check_interval_seconds: 10,
                metrics_collection_interval_seconds: 5,
            },
            paths: PathConfig {
                default_config_path: "./config/system.toml".to_string(),
                default_secrets_path: "./config/secrets.toml".to_string(),
                default_exchanges_path: "./config/exchanges.toml".to_string(),
            },
            version: VersionConfig {
                version: "5.1.0".to_string(),
                build_timestamp: "2024-08-25T14:30:00Z".to_string(),
                git_sha: "development".to_string(),
            },
        }
    }
}

// 便捷访问函数
pub fn max_supported_exchanges() -> usize {
    get_system_config().read().limits.max_supported_exchanges
}

pub fn max_supported_symbols() -> usize {
    get_system_config().read().limits.max_supported_symbols
}

pub fn max_concurrent_opportunities() -> usize {
    get_system_config().read().limits.max_concurrent_opportunities
}

pub fn max_order_batch_size() -> usize {
    get_system_config().read().limits.max_order_batch_size
}

pub fn target_latency_microseconds() -> u64 {
    get_system_config().read().performance.target_latency_microseconds
}

pub fn target_throughput_ops_per_sec() -> u64 {
    get_system_config().read().performance.target_throughput_ops_per_sec
}

pub fn target_success_rate_percent() -> f64 {
    get_system_config().read().performance.target_success_rate_percent
}

pub fn health_check_interval_seconds() -> u64 {
    get_system_config().read().monitoring.health_check_interval_seconds
}

pub fn api_health_check_interval_seconds() -> u64 {
    get_system_config().read().monitoring.api_health_check_interval_seconds
}

pub fn metrics_collection_interval_seconds() -> u64 {
    get_system_config().read().monitoring.metrics_collection_interval_seconds
}

pub fn version() -> String {
    get_system_config().read().version.version.clone()
}

pub fn build_timestamp() -> String {
    get_system_config().read().version.build_timestamp.clone()
}

pub fn git_sha() -> String {
    get_system_config().read().version.git_sha.clone()
}