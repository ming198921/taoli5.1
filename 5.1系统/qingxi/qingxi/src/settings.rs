#![allow(dead_code)]
// src/settings.rs
//! # Configuration Management Module
use crate::types::{ConsistencyThresholds, MarketSourceConfig};
use serde::Deserialize;

fn default_metrics_port_offset() -> u16 { 1 }
fn default_health_port_offset() -> u16 { 2 }
fn default_http_port_offset() -> u16 { 10 }
fn default_orderbook_depth_limit() -> usize { 10 }
fn default_symbols_list_limit() -> usize { 50 }
fn default_performance_stats_interval() -> u64 { 30 }
fn default_system_readiness_timeout() -> u64 { 60 }
fn default_command_channel_size() -> usize { 128 }
fn default_internal_channel_size() -> usize { 1000 }
fn default_cleaner_buffer_size() -> usize { 1000 }
fn default_network_worker_threads() -> usize { 3 }
fn default_network_cpu_cores() -> Vec<usize> { vec![2, 3, 4] }
fn default_processing_worker_threads() -> usize { 1 }
fn default_processing_cpu_core() -> usize { 5 }
fn default_main_worker_threads() -> usize { 2 }
fn default_cache_hit_rate_threshold() -> f64 { 0.8 }
fn default_buffer_usage_threshold() -> f64 { 0.8 }
fn default_compression_ratio_threshold() -> f64 { 2.0 }
fn default_data_freshness_warning_ms() -> u64 { 1000 }
fn default_data_freshness_critical_ms() -> u64 { 5000 }
fn default_max_orderbook_count() -> usize { 10000 }
fn default_max_batch_size() -> usize { 1000 }
fn default_max_memory_usage_mb() -> usize { 1024 }
fn default_auto_create_dirs() -> bool { true }
fn default_snapshot_pool_size() -> usize { 500 }
fn default_default_vec_capacity() -> usize { 100 }

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub general: GeneralSettings,
    pub api_server: ApiServerSettings,
    pub central_manager: CentralManagerSettings,
    #[serde(default)]
    pub sources: Vec<MarketSourceConfig>,
    pub consistency_thresholds: ConsistencyThresholds,
    pub reasoner: ReasonerSettings,
    pub anomaly_detection: AnomalyDetectionSettings,
    #[serde(default)]
    pub performance: PerformanceSettings,
    #[serde(default)]
    pub threading: ThreadingSettings,
    #[serde(default)]
    pub quality_thresholds: QualityThresholds,
    #[serde(default)]
    pub cache: CacheSettings,
    #[serde(default)]
    pub memory_pools: MemoryPoolSettings,
    #[serde(default)]
    pub algorithm_scoring: AlgorithmScoringSettings,
    #[serde(default)]
    pub memory_allocator: MemoryAllocatorSettings,
    #[serde(default)]
    pub cleaner: CleanerSettings,
    #[serde(default)]
    pub batch: BatchSettings,
    #[serde(default)]
    pub benchmark: BenchmarkSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralSettings {
    pub log_level: String,
    pub metrics_enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiServerSettings {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_metrics_port_offset")]
    pub metrics_port_offset: u16,
    #[serde(default = "default_health_port_offset")]
    pub health_port_offset: u16,
    #[serde(default = "default_http_port_offset")]
    pub http_port_offset: u16,
    #[serde(default = "default_orderbook_depth_limit")]
    pub orderbook_depth_limit: usize,
    #[serde(default = "default_symbols_list_limit")]
    pub symbols_list_limit: usize,
}

impl ApiServerSettings {
    /// 获取API服务器绑定地址，支持环境变量覆盖
    pub fn get_host(&self) -> String {
        std::env::var("QINGXI_API_SERVER__HOST")
            .unwrap_or_else(|_| self.host.clone())
    }
    
    /// 获取API服务器端口，支持环境变量覆盖
    pub fn get_port(&self) -> u16 {
        std::env::var("QINGXI_API_SERVER__PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CentralManagerSettings {
    pub event_buffer_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReasonerSettings {
    pub api_endpoint: String,
}

impl ReasonerSettings {
    pub fn get_api_endpoint(&self) -> String {
        std::env::var("QINGXI_REASONER_ENDPOINT")
            .unwrap_or_else(|_| self.api_endpoint.clone())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnomalyDetectionSettings {
    pub spread_threshold: f64,
    pub volume_threshold: f64,
    pub price_change_threshold: f64,
    pub spread_threshold_percentage: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PerformanceSettings {
    #[serde(default = "default_performance_stats_interval")]
    pub performance_stats_interval_sec: u64,
    #[serde(default = "default_system_readiness_timeout")]
    pub system_readiness_timeout_sec: u64,
    #[serde(default = "default_command_channel_size")]
    pub command_channel_size: usize,
    #[serde(default = "default_internal_channel_size")]
    pub internal_channel_size: usize,
    #[serde(default = "default_cleaner_buffer_size")]
    pub cleaner_input_buffer_size: usize,
    #[serde(default = "default_cleaner_buffer_size")]
    pub cleaner_output_buffer_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThreadingSettings {
    #[serde(default = "default_network_worker_threads")]
    pub network_worker_threads: usize,
    #[serde(default = "default_network_cpu_cores")]
    pub network_cpu_cores: Vec<usize>,
    #[serde(default = "default_processing_worker_threads")]
    pub processing_worker_threads: usize,
    #[serde(default = "default_processing_cpu_core")]
    pub processing_cpu_core: usize,
    #[serde(default = "default_main_worker_threads")]
    pub main_worker_threads: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QualityThresholds {
    #[serde(default = "default_cache_hit_rate_threshold")]
    pub cache_hit_rate_threshold: f64,
    #[serde(default = "default_buffer_usage_threshold")]
    pub buffer_usage_threshold: f64,
    #[serde(default = "default_compression_ratio_threshold")]
    pub compression_ratio_threshold: f64,
    #[serde(default = "default_data_freshness_warning_ms")]
    pub data_freshness_warning_ms: u64,
    #[serde(default = "default_data_freshness_critical_ms")]
    pub data_freshness_critical_ms: u64,
    #[serde(default = "default_max_orderbook_count")]
    pub max_orderbook_count: usize,
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    #[serde(default = "default_max_memory_usage_mb")]
    pub max_memory_usage_mb: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheSettings {
    pub l2_directory: String,
    pub l3_directory: String,
    pub log_directory: String,
    #[serde(default = "default_auto_create_dirs")]
    pub auto_create_dirs: bool,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_path =
            std::env::var("QINGXI_CONFIG_PATH").unwrap_or_else(|_| "configs/qingxi".to_string());

        config::Config::builder()
            .add_source(config::File::with_name(&config_path).required(true))
            .add_source(config::Environment::with_prefix("QINGXI").separator("__"))
            .build()?
            .try_deserialize()
    }

    pub fn get_api_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port())
    }

    pub fn get_metrics_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port() + self.api_server.metrics_port_offset)
    }

    pub fn get_health_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port() + self.api_server.health_port_offset)
    }

    pub fn get_http_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port() + self.api_server.http_port_offset)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                log_level: "info".to_string(),
                metrics_enabled: true,
            },
            api_server: ApiServerSettings {
                host: "0.0.0.0".to_string(),
                port: 50051,
                metrics_port_offset: 1,
                health_port_offset: 2,
                http_port_offset: 10,
                orderbook_depth_limit: 20,
                symbols_list_limit: 100,
            },
            central_manager: CentralManagerSettings { 
                event_buffer_size: 1000 
            },
            sources: vec![],
            consistency_thresholds: crate::types::ConsistencyThresholds {
                price_diff_percentage: 0.5,
                timestamp_diff_ms: 5000,
                sequence_gap_threshold: 10,
                spread_threshold_percentage: 1.0,
                critical_spread_threshold_percentage: 2.0,
                max_time_diff_ms: 10000.0,
                volume_consistency_threshold: 0.5,
            },
            reasoner: ReasonerSettings {
                api_endpoint: "http://reasoner-service:8081".to_string(),
            },
            anomaly_detection: AnomalyDetectionSettings {
                spread_threshold: 2.0,
                volume_threshold: 100.0,
                price_change_threshold: 5.0,
                spread_threshold_percentage: 1.0,
            },
            performance: PerformanceSettings::default(),
            threading: ThreadingSettings::default(),
            quality_thresholds: QualityThresholds::default(),
            cache: CacheSettings::default(),
            memory_pools: MemoryPoolSettings::default(),
            algorithm_scoring: AlgorithmScoringSettings::default(),
            memory_allocator: MemoryAllocatorSettings::default(),
            cleaner: CleanerSettings::default(),
            batch: BatchSettings::default(),
            benchmark: BenchmarkSettings::default(),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            performance_stats_interval_sec: default_performance_stats_interval(),
            system_readiness_timeout_sec: default_system_readiness_timeout(),
            command_channel_size: default_command_channel_size(),
            internal_channel_size: default_internal_channel_size(),
            cleaner_input_buffer_size: default_cleaner_buffer_size(),
            cleaner_output_buffer_size: default_cleaner_buffer_size(),
        }
    }
}

impl Default for ThreadingSettings {
    fn default() -> Self {
        Self {
            network_worker_threads: default_network_worker_threads(),
            network_cpu_cores: default_network_cpu_cores(),
            processing_worker_threads: default_processing_worker_threads(),
            processing_cpu_core: default_processing_cpu_core(),
            main_worker_threads: default_main_worker_threads(),
        }
    }
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            cache_hit_rate_threshold: default_cache_hit_rate_threshold(),
            buffer_usage_threshold: default_buffer_usage_threshold(),
            compression_ratio_threshold: default_compression_ratio_threshold(),
            data_freshness_warning_ms: default_data_freshness_warning_ms(),
            data_freshness_critical_ms: default_data_freshness_critical_ms(),
            max_orderbook_count: default_max_orderbook_count(),
            max_batch_size: default_max_batch_size(),
            max_memory_usage_mb: default_max_memory_usage_mb(),
        }
    }
}

impl Default for CacheSettings {
    fn default() -> Self {
        Self {
            l2_directory: "cache/l2".to_string(),
            l3_directory: "cache/l3".to_string(),
            log_directory: "logs/cache".to_string(),
            auto_create_dirs: default_auto_create_dirs(),
        }
    }
}

// 新增配置结构
fn default_channel_buffer_size() -> usize { 1000 }

// WebSocket网络配置默认值
fn default_connection_timeout_sec() -> u64 { 30 }
fn default_read_timeout_sec() -> u64 { 60 }
fn default_write_timeout_sec() -> u64 { 10 }
fn default_heartbeat_interval_sec() -> u64 { 30 }
fn default_max_reconnect_attempts() -> u32 { 5 }
fn default_reconnect_initial_delay_sec() -> u64 { 5 }
fn default_reconnect_max_delay_sec() -> u64 { 300 }
fn default_reconnect_backoff_multiplier() -> f64 { 1.5 }
fn default_max_frame_size() -> usize { 16 * 1024 * 1024 } // 16MB
fn default_message_buffer_size() -> usize { 1000 }
fn default_enable_tls_verification() -> bool { true }
fn default_tcp_keepalive_sec() -> u64 { 60 }
fn default_tcp_nodelay() -> bool { true }
fn default_orderbook_entry_pool_size() -> usize { 1000 }
fn default_trade_update_pool_size() -> usize { 5000 }
#[allow(dead_code)]
fn default_depth_score_max() -> f64 { 30.0 }
#[allow(dead_code)]
fn default_liquidity_score_baseline() -> f64 { 1000.0 }
#[allow(dead_code)]
fn default_liquidity_score_max() -> f64 { 25.0 }

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryPoolSettings {
    pub orderbook_entry_pool_size: usize,
    pub trade_update_pool_size: usize,
    pub cleaner_buffer_size: usize,
    pub snapshot_pool_size: usize,
    pub default_vec_capacity: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlgorithmScoringSettings {
    pub liquidity_score_baseline: f64,
    pub liquidity_score_max: f64,
    pub spread_score_max: f64,
    pub spread_ratio_threshold: f64,
    pub balance_score_max: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryAllocatorSettings {
    pub zero_allocation_buffer_size: usize,
    pub large_buffer_size: usize,
    pub huge_buffer_size: usize,
    pub small_chunks_capacity: usize,
    pub chunk_sizes: Vec<usize>,
    pub large_buffer_sizes: Vec<usize>,
    pub size_threshold: usize,
    pub test_iterations: usize,
    pub test_allocation_sizes: Vec<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CleanerSettings {
    pub memory_pool_size: usize,
    pub batch_size: usize,
    pub orderbook_capacity: usize,
    pub zero_alloc_buffer_count: usize,
    pub thread_count: usize,
    pub target_latency_ns: u64,
    pub orderbook_bid_capacity: usize,
    pub orderbook_ask_capacity: usize,
    pub reserved_bid_capacity: usize,
    pub reserved_ask_capacity: usize,
    pub volume_top_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchSettings {
    pub max_batch_size: usize,
    pub max_wait_time_ms: u64,
    pub concurrency: usize,
    pub enable_compression: bool,
    pub buffer_multiplier: usize,  // 缓冲区倍数
}

#[derive(Debug, Clone, Deserialize)]
pub struct BenchmarkSettings {
    pub iterations: usize,
    pub orderbook_depth: usize,
    pub trade_count: usize,
    pub warmup_iterations: usize,
    pub verbose_output: bool,
}

impl Default for MemoryAllocatorSettings {
    fn default() -> Self {
        Self {
            zero_allocation_buffer_size: 131072,
            large_buffer_size: 262144,
            huge_buffer_size: 1048576,
            small_chunks_capacity: 1024,
            chunk_sizes: vec![64, 128, 256, 512, 1024],
            large_buffer_sizes: vec![131072, 262144, 524288, 1048576],
            size_threshold: 1024,
            test_iterations: 1000000,
            test_allocation_sizes: vec![128, 1024, 4096],
        }
    }
}

impl Default for CleanerSettings {
    fn default() -> Self {
        Self {
            memory_pool_size: 65536,
            batch_size: 10000,
            orderbook_capacity: 1000,
            zero_alloc_buffer_count: 65536,
            thread_count: 4,
            target_latency_ns: 100000,
            orderbook_bid_capacity: 1000,
            orderbook_ask_capacity: 1000,
            reserved_bid_capacity: 1000,
            reserved_ask_capacity: 1000,
            volume_top_count: 10,
        }
    }
}

impl Default for BatchSettings {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            max_wait_time_ms: 10,
            concurrency: 4,
            enable_compression: false,
            buffer_multiplier: 10,
        }
    }
}

impl Default for BenchmarkSettings {
    fn default() -> Self {
        Self {
            iterations: 1000,
            orderbook_depth: 20,
            trade_count: 50,
            warmup_iterations: 100,
            verbose_output: false,
        }
    }
}

impl Default for MemoryPoolSettings {
    fn default() -> Self {
        Self {
            orderbook_entry_pool_size: default_orderbook_entry_pool_size(),
            trade_update_pool_size: default_trade_update_pool_size(),
            cleaner_buffer_size: default_cleaner_buffer_size(),
            snapshot_pool_size: default_snapshot_pool_size(),
            default_vec_capacity: default_default_vec_capacity(),
        }
    }
}

impl Default for AlgorithmScoringSettings {
    fn default() -> Self {
        Self {
            liquidity_score_baseline: 1000.0,
            liquidity_score_max: 25.0,
            spread_score_max: 25.0,
            spread_ratio_threshold: 0.05,
            balance_score_max: 20.0,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct WebSocketNetworkSettings {
    /// WebSocket连接超时时间（秒）
    #[serde(default = "default_connection_timeout_sec")]
    pub connection_timeout_sec: u64,
    
    /// WebSocket读取超时时间（秒）
    #[serde(default = "default_read_timeout_sec")]
    pub read_timeout_sec: u64,
    
    /// WebSocket写入超时时间（秒）
    #[serde(default = "default_write_timeout_sec")]
    pub write_timeout_sec: u64,
    
    /// 心跳间隔时间（秒）
    #[serde(default = "default_heartbeat_interval_sec")]
    pub heartbeat_interval_sec: u64,
    
    /// 最大重连尝试次数
    #[serde(default = "default_max_reconnect_attempts")]
    pub max_reconnect_attempts: u32,
    
    /// 重连初始延迟时间（秒）
    #[serde(default = "default_reconnect_initial_delay_sec")]
    pub reconnect_initial_delay_sec: u64,
    
    /// 重连最大延迟时间（秒）
    #[serde(default = "default_reconnect_max_delay_sec")]
    pub reconnect_max_delay_sec: u64,
    
    /// 重连退避倍数
    #[serde(default = "default_reconnect_backoff_multiplier")]
    pub reconnect_backoff_multiplier: f64,
    
    /// WebSocket最大帧大小（字节）
    #[serde(default = "default_max_frame_size")]
    pub max_frame_size: usize,
    
    /// 消息缓冲区大小
    #[serde(default = "default_message_buffer_size")]
    pub message_buffer_size: usize,
    
    /// 启用TLS证书验证
    #[serde(default = "default_enable_tls_verification")]
    pub enable_tls_verification: bool,
    
    /// TCP保活时间（秒）
    #[serde(default = "default_tcp_keepalive_sec")]
    pub tcp_keepalive_sec: u64,
    
    /// 启用TCP_NODELAY（禁用Nagle算法）
    #[serde(default = "default_tcp_nodelay")]
    pub tcp_nodelay: bool,
}

impl Default for WebSocketNetworkSettings {
    fn default() -> Self {
        Self {
            connection_timeout_sec: default_connection_timeout_sec(),
            read_timeout_sec: default_read_timeout_sec(),
            write_timeout_sec: default_write_timeout_sec(),
            heartbeat_interval_sec: default_heartbeat_interval_sec(),
            max_reconnect_attempts: default_max_reconnect_attempts(),
            reconnect_initial_delay_sec: default_reconnect_initial_delay_sec(),
            reconnect_max_delay_sec: default_reconnect_max_delay_sec(),
            reconnect_backoff_multiplier: default_reconnect_backoff_multiplier(),
            max_frame_size: default_max_frame_size(),
            message_buffer_size: default_message_buffer_size(),
            enable_tls_verification: default_enable_tls_verification(),
            tcp_keepalive_sec: default_tcp_keepalive_sec(),
            tcp_nodelay: default_tcp_nodelay(),
        }
    }
}

impl WebSocketNetworkSettings {
    /// 获取连接超时Duration
    pub fn get_connection_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.connection_timeout_sec)
    }
    
    /// 获取读取超时Duration
    pub fn get_read_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.read_timeout_sec)
    }
    
    /// 获取写入超时Duration
    pub fn get_write_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.write_timeout_sec)
    }
    
    /// 获取心跳间隔Duration
    pub fn get_heartbeat_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.heartbeat_interval_sec)
    }
    
    /// 获取重连初始延迟Duration
    pub fn get_reconnect_initial_delay(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.reconnect_initial_delay_sec)
    }
    
    /// 获取重连最大延迟Duration
    pub fn get_reconnect_max_delay(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.reconnect_max_delay_sec)
    }
    
    /// 计算指数退避延迟时间
    pub fn get_exponential_backoff_delay(&self, attempt: u32) -> std::time::Duration {
        let base_delay = self.reconnect_initial_delay_sec as f64;
        let max_delay = self.reconnect_max_delay_sec as f64;
        let multiplier = self.reconnect_backoff_multiplier;
        
        let delay = base_delay * multiplier.powi(attempt as i32);
        let final_delay = delay.min(max_delay);
        
        std::time::Duration::from_secs(final_delay as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_network_settings_default() {
        let settings = WebSocketNetworkSettings::default();
        
        // 测试默认值
        assert_eq!(settings.connection_timeout_sec, 30);
        assert_eq!(settings.read_timeout_sec, 60);
        assert_eq!(settings.write_timeout_sec, 10);
        assert_eq!(settings.heartbeat_interval_sec, 30);
        assert_eq!(settings.max_reconnect_attempts, 5);
        assert_eq!(settings.reconnect_initial_delay_sec, 5);
        assert_eq!(settings.reconnect_max_delay_sec, 300);
        assert_eq!(settings.reconnect_backoff_multiplier, 1.5);
        assert_eq!(settings.max_frame_size, 16 * 1024 * 1024); // 16MB
        assert_eq!(settings.message_buffer_size, 1000);
        assert_eq!(settings.enable_tls_verification, true);
        assert_eq!(settings.tcp_keepalive_sec, 60);
        assert_eq!(settings.tcp_nodelay, true);
    }

    #[test]
    fn test_websocket_duration_methods() {
        let settings = WebSocketNetworkSettings::default();
        
        // 测试Duration转换方法
        assert_eq!(settings.get_connection_timeout(), std::time::Duration::from_secs(30));
        assert_eq!(settings.get_read_timeout(), std::time::Duration::from_secs(60));
        assert_eq!(settings.get_write_timeout(), std::time::Duration::from_secs(10));
        assert_eq!(settings.get_heartbeat_interval(), std::time::Duration::from_secs(30));
        assert_eq!(settings.get_reconnect_initial_delay(), std::time::Duration::from_secs(5));
        assert_eq!(settings.get_reconnect_max_delay(), std::time::Duration::from_secs(300));
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let settings = WebSocketNetworkSettings::default();
        
        // 测试指数退避计算
        let delay_0 = settings.get_exponential_backoff_delay(0);
        let delay_1 = settings.get_exponential_backoff_delay(1);
        let delay_2 = settings.get_exponential_backoff_delay(2);
        let delay_3 = settings.get_exponential_backoff_delay(3);
        
        // 第0次尝试: 5秒
        assert_eq!(delay_0, std::time::Duration::from_secs(5));
        // 第1次尝试: 5 * 1.5 = 7.5秒 -> 7秒
        assert_eq!(delay_1, std::time::Duration::from_secs(7));
        // 第2次尝试: 5 * 1.5^2 = 11.25秒 -> 11秒
        assert_eq!(delay_2, std::time::Duration::from_secs(11));
        // 第3次尝试: 5 * 1.5^3 = 16.875秒 -> 16秒
        assert_eq!(delay_3, std::time::Duration::from_secs(16));
        
        // 测试达到最大延迟
        let delay_large = settings.get_exponential_backoff_delay(20);
        assert_eq!(delay_large, settings.get_reconnect_max_delay());
    }

    #[test]
    fn test_settings_integration() {
        let settings = Settings::default();
        
        // 测试WebSocket网络设置集成到主Settings中
        assert_eq!(settings.websocket_network.connection_timeout_sec, 30);
        assert_eq!(settings.websocket_network.max_reconnect_attempts, 5);
        assert_eq!(settings.websocket_network.enable_tls_verification, true);
    }

    #[test]
    fn test_production_ready_values() {
        let settings = WebSocketNetworkSettings::default();
        
        // 验证生产级配置
        assert!(settings.connection_timeout_sec > 0, "连接超时必须大于0");
        assert!(settings.read_timeout_sec > settings.connection_timeout_sec, "读取超时应该大于连接超时");
        assert!(settings.max_reconnect_attempts > 0, "重连次数必须大于0");
        assert!(settings.reconnect_backoff_multiplier > 1.0, "退避倍数应该大于1.0以确保延迟递增");
        assert!(settings.max_frame_size >= 1024, "最大帧大小应该至少1KB");
        assert!(settings.enable_tls_verification, "生产环境应该启用TLS验证");
        assert!(settings.tcp_nodelay, "应该启用TCP_NODELAY以减少延迟");
    }
}
