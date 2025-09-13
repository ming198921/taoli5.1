//! 统一配置架构 - 消除所有Config结构体重复
//! 
//! 这个模块提供了系统范围内的统一配置管理，消除了263个重复的Config结构体

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 系统统一配置根结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSystemConfig {
    /// 系统核心配置
    pub core: CoreConfig,
    /// 交易所配置
    pub exchanges: Vec<ExchangeConfig>,
    /// 策略配置
    pub strategy: StrategyConfig,
    /// 风险管理配置
    pub risk: RiskConfig,
    /// 性能优化配置
    pub performance: PerformanceConfig,
    /// 监控配置
    pub monitoring: MonitoringConfig,
    /// 网络和API配置
    pub network: NetworkConfig,
    /// 数据管理配置
    pub data: DataConfig,
}

/// 系统核心配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// 系统ID
    pub system_id: String,
    /// 环境类型 (dev, staging, prod)
    pub environment: String,
    /// 系统版本
    pub version: String,
    /// 日志级别
    pub log_level: String,
    /// 系统限制
    pub limits: SystemLimitsConfig,
}

/// 统一交易所配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    /// 交易所名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// API配置
    pub api: ApiConfig,
    /// 连接配置
    pub connection: ConnectionConfig,
    /// 速率限制配置
    pub rate_limits: RateLimitConfig,
    /// 交易费用配置
    pub trading_fees: TradingFeeConfig,
    /// 精度配置
    pub precision: PrecisionConfig,
    /// 重试配置
    pub retry: RetryConfig,
}

/// 统一策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// 启用的策略列表
    pub enabled_strategies: Vec<String>,
    /// 最小利润阈值
    pub min_profit_threshold: f64,
    /// 最大滑点
    pub max_slippage: f64,
    /// 三角套利配置
    pub triangular: TriangularConfig,
    /// 跨交易所套利配置
    pub inter_exchange: InterExchangeConfig,
    /// 执行配置
    pub execution: ExecutionConfig,
    /// 调度器配置
    pub scheduler: SchedulerConfig,
}

/// 统一风险管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// 是否启用风险控制
    pub enabled: bool,
    /// 最大日损失 (USD)
    pub max_daily_loss_usd: f64,
    /// 最大仓位 (USD)
    pub max_position_usd: f64,
    /// 紧急停止配置
    pub emergency_stop: EmergencyStopConfig,
    /// 动态阈值配置
    pub dynamic_thresholds: DynamicThresholdConfig,
    /// 资金管理配置
    pub fund_management: FundManagementConfig,
}

/// 统一性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// CPU亲和性配置
    pub cpu_affinity: CpuAffinityConfig,
    /// 内存池配置
    pub memory_pool: MemoryPoolConfig,
    /// SIMD优化配置
    pub simd: SIMDConfig,
    /// 零拷贝配置
    pub zero_copy: ZeroCopyConfig,
    /// 高频配置
    pub high_frequency: HighFrequencyConfig,
    /// 延迟监控配置
    pub latency_monitoring: LatencyMonitoringConfig,
}

/// 统一监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 是否启用监控
    pub enabled: bool,
    /// 指标配置
    pub metrics: MetricsConfig,
    /// 告警配置
    pub alerts: AlertConfig,
    /// 健康检查配置
    pub health_check: HealthCheckConfig,
    /// 追踪配置
    pub tracing: TracingConfig,
    /// 可视化配置
    pub visualization: VisualizationConfig,
}

/// 统一网络和API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 代理配置
    pub proxy: Option<ProxyConfig>,
    /// 安全配置
    pub security: SecurityConfig,
    /// 心跳配置
    pub heartbeat: HeartbeatConfig,
    /// 连接池配置
    pub connection_pool: ConnectionPoolConfig,
}

/// 统一数据管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    /// 存储配置
    pub storage: StorageConfig,
    /// 数据清理配置
    pub cleaning: DataCleaningConfig,
    /// 数据验证配置
    pub validation: DataValidationConfig,
    /// 缓存配置
    pub cache: CacheConfig,
    /// 批处理配置
    pub batch: BatchConfig,
    /// 数据分发配置
    pub distribution: DataDistributionConfig,
}

// ============= 子配置结构体定义 =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLimitsConfig {
    pub max_concurrent_strategies: usize,
    pub max_market_connections: usize,
    pub max_memory_usage_mb: usize,
    pub max_cpu_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub base_url: String,
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>,
    pub timeout_ms: u64,
    pub max_requests_per_second: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub connect_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub write_timeout_ms: u64,
    pub keep_alive: bool,
    pub pool_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_capacity: u32,
    pub retry_after_ms: u64,
    pub adaptive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingFeeConfig {
    pub maker_fee_rate: f64,
    pub taker_fee_rate: f64,
    pub withdrawal_fees: HashMap<String, f64>,
    pub fee_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecisionConfig {
    pub price_precision: HashMap<String, u8>,
    pub quantity_precision: HashMap<String, u8>,
    pub min_order_size: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularConfig {
    pub enabled: bool,
    pub min_path_length: usize,
    pub max_path_length: usize,
    pub path_discovery_interval_ms: u64,
    pub validation_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterExchangeConfig {
    pub enabled: bool,
    pub max_price_deviation: f64,
    pub execution_timeout_ms: u64,
    pub funding_rate_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub execution_timeout_ms: u64,
    pub order_retry_count: u32,
    pub partial_fill_handling: bool,
    pub slippage_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub worker_threads: usize,
    pub queue_capacity: usize,
    pub scheduling_interval_ms: u64,
    pub priority_levels: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyStopConfig {
    pub enabled: bool,
    pub consecutive_failures: u32,
    pub failure_rate_threshold: f64,
    pub cooldown_period_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicThresholdConfig {
    pub enabled: bool,
    pub adjustment_interval_ms: u64,
    pub sensitivity_factor: f64,
    pub volatility_window_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundManagementConfig {
    pub max_fund_utilization: f64,
    pub position_sizing_method: String,
    pub risk_free_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuAffinityConfig {
    pub enabled: bool,
    pub core_binding: Vec<usize>,
    pub numa_awareness: bool,
    pub thread_priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolConfig {
    pub enabled: bool,
    pub pool_size_mb: usize,
    pub chunk_size: usize,
    pub preallocation_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIMDConfig {
    pub enabled: bool,
    pub instruction_set: String, // "avx512", "avx2", "sse4"
    pub auto_detect: bool,
    pub fallback_to_scalar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCopyConfig {
    pub enabled: bool,
    pub buffer_size: usize,
    pub memory_mapping: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighFrequencyConfig {
    pub enabled: bool,
    pub tick_processing_threads: usize,
    pub order_processing_threads: usize,
    pub latency_target_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMonitoringConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub alert_threshold_ms: f64,
    pub histogram_buckets: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval_ms: u64,
    pub retention_days: u32,
    pub export_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub channels: Vec<String>, // "email", "slack", "telegram"
    pub severity_levels: Vec<String>,
    pub aggregation_window_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub check_interval_ms: u64,
    pub timeout_ms: u64,
    pub failure_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub enabled: bool,
    pub sampling_rate: f64,
    pub service_name: String,
    pub export_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub enabled: bool,
    pub dashboard_port: u16,
    pub refresh_interval_ms: u64,
    pub chart_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub socks_proxy: Option<String>,
    pub bypass_hosts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub tls_enabled: bool,
    pub certificate_validation: bool,
    pub cipher_suites: Vec<String>,
    pub api_key_rotation_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub enabled: bool,
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub max_missed_heartbeats: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    pub max_connections: usize,
    pub idle_timeout_ms: u64,
    pub connection_lifetime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub engine: String, // "rocksdb", "postgresql", "redis"
    pub path: String,
    pub max_size_gb: u64,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCleaningConfig {
    pub enabled: bool,
    pub cleaning_interval_ms: u64,
    pub retention_days: u32,
    pub outlier_detection: OutlierDetectionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetectionConfig {
    pub method: String, // "zscore", "iqr", "isolation_forest"
    pub threshold: f64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationConfig {
    pub enabled: bool,
    pub schema_validation: bool,
    pub range_checks: bool,
    pub consistency_checks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub cache_type: String, // "redis", "memory", "hybrid"
    pub ttl_seconds: u64,
    pub max_size_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub enabled: bool,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    pub max_memory_usage_mb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDistributionConfig {
    pub enabled: bool,
    pub replication_factor: u32,
    pub consistency_level: String,
    pub partition_strategy: String,
}

impl Default for UnifiedSystemConfig {
    fn default() -> Self {
        Self {
            core: CoreConfig::default(),
            exchanges: vec![],
            strategy: StrategyConfig::default(),
            risk: RiskConfig::default(),
            performance: PerformanceConfig::default(),
            monitoring: MonitoringConfig::default(),
            network: NetworkConfig::default(),
            data: DataConfig::default(),
        }
    }
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            system_id: "arbitrage-system-5.1".to_string(),
            environment: "production".to_string(),
            version: "5.1.0".to_string(),
            log_level: "info".to_string(),
            limits: SystemLimitsConfig::default(),
        }
    }
}

impl Default for SystemLimitsConfig {
    fn default() -> Self {
        Self {
            max_concurrent_strategies: 10,
            max_market_connections: 50,
            max_memory_usage_mb: 8192,
            max_cpu_usage_percent: 80.0,
        }
    }
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            enabled_strategies: vec!["triangular".to_string(), "inter_exchange".to_string()],
            min_profit_threshold: 0.001,
            max_slippage: 0.002,
            triangular: TriangularConfig::default(),
            inter_exchange: InterExchangeConfig::default(),
            execution: ExecutionConfig::default(),
            scheduler: SchedulerConfig::default(),
        }
    }
}

impl Default for TriangularConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_path_length: 3,
            max_path_length: 4,
            path_discovery_interval_ms: 100,
            validation_timeout_ms: 50,
        }
    }
}

impl Default for InterExchangeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_price_deviation: 0.01,
            execution_timeout_ms: 1000,
            funding_rate_threshold: 0.05,
        }
    }
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            execution_timeout_ms: 5000,
            order_retry_count: 3,
            partial_fill_handling: true,
            slippage_tolerance: 0.005,
        }
    }
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            queue_capacity: 10000,
            scheduling_interval_ms: 10,
            priority_levels: 5,
        }
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_daily_loss_usd: 10000.0,
            max_position_usd: 50000.0,
            emergency_stop: EmergencyStopConfig::default(),
            dynamic_thresholds: DynamicThresholdConfig::default(),
            fund_management: FundManagementConfig::default(),
        }
    }
}

impl Default for EmergencyStopConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            consecutive_failures: 5,
            failure_rate_threshold: 0.8,
            cooldown_period_ms: 300000, // 5分钟
        }
    }
}

impl Default for DynamicThresholdConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            adjustment_interval_ms: 60000, // 1分钟
            sensitivity_factor: 0.1,
            volatility_window_size: 100,
        }
    }
}

impl Default for FundManagementConfig {
    fn default() -> Self {
        Self {
            max_fund_utilization: 0.8,
            position_sizing_method: "kelly".to_string(),
            risk_free_rate: 0.02,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cpu_affinity: CpuAffinityConfig::default(),
            memory_pool: MemoryPoolConfig::default(),
            simd: SIMDConfig::default(),
            zero_copy: ZeroCopyConfig::default(),
            high_frequency: HighFrequencyConfig::default(),
            latency_monitoring: LatencyMonitoringConfig::default(),
        }
    }
}

impl Default for SIMDConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            instruction_set: "avx512".to_string(),
            auto_detect: true,
            fallback_to_scalar: true,
        }
    }
}

impl Default for HighFrequencyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tick_processing_threads: 4,
            order_processing_threads: 2,
            latency_target_ns: 100_000, // 100μs
        }
    }
}

impl Default for CpuAffinityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            core_binding: vec![0, 1, 2, 3],
            numa_awareness: true,
            thread_priority: 90,
        }
    }
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pool_size_mb: 1024,
            chunk_size: 4096,
            preallocation_percentage: 0.8,
        }
    }
}

impl Default for ZeroCopyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 65536,
            memory_mapping: true,
        }
    }
}

impl Default for LatencyMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 0.01, // 1%采样
            alert_threshold_ms: 10.0,
            histogram_buckets: vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0],
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics: MetricsConfig::default(),
            alerts: AlertConfig::default(),
            health_check: HealthCheckConfig::default(),
            tracing: TracingConfig::default(),
            visualization: VisualizationConfig::default(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval_ms: 1000,
            retention_days: 30,
            export_format: "prometheus".to_string(),
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec!["email".to_string(), "slack".to_string()],
            severity_levels: vec!["info".to_string(), "warning".to_string(), "error".to_string(), "critical".to_string()],
            aggregation_window_ms: 60000, // 1分钟
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_ms: 30000, // 30秒
            timeout_ms: 5000,
            failure_threshold: 3,
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 0.1, // 10%采样
            service_name: "arbitrage-system".to_string(),
            export_endpoint: "http://localhost:14268/api/traces".to_string(),
        }
    }
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dashboard_port: 3000,
            refresh_interval_ms: 5000,
            chart_types: vec!["line".to_string(), "bar".to_string(), "heatmap".to_string()],
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            security: SecurityConfig::default(),
            heartbeat: HeartbeatConfig::default(),
            connection_pool: ConnectionPoolConfig::default(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: true,
            certificate_validation: true,
            cipher_suites: vec!["TLS_AES_256_GCM_SHA384".to_string()],
            api_key_rotation_hours: 24,
        }
    }
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_ms: 30000, // 30秒
            timeout_ms: 5000,
            max_missed_heartbeats: 3,
        }
    }
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            idle_timeout_ms: 300000, // 5分钟
            connection_lifetime_ms: 3600000, // 1小时
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            cleaning: DataCleaningConfig::default(),
            validation: DataValidationConfig::default(),
            cache: CacheConfig::default(),
            batch: BatchConfig::default(),
            distribution: DataDistributionConfig::default(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            engine: "rocksdb".to_string(),
            path: "/opt/arbitrage/data".to_string(),
            max_size_gb: 100,
            compression_enabled: true,
        }
    }
}

impl Default for DataCleaningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cleaning_interval_ms: 3600000, // 1小时
            retention_days: 30,
            outlier_detection: OutlierDetectionConfig::default(),
        }
    }
}

impl Default for OutlierDetectionConfig {
    fn default() -> Self {
        Self {
            method: "zscore".to_string(),
            threshold: 3.0,
            enabled: true,
        }
    }
}

impl Default for DataValidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            schema_validation: true,
            range_checks: true,
            consistency_checks: true,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_type: "redis".to_string(),
            ttl_seconds: 3600, // 1小时
            max_size_mb: 512,
        }
    }
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_size: 1000,
            flush_interval_ms: 5000,
            max_memory_usage_mb: 256,
        }
    }
}

impl Default for DataDistributionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            replication_factor: 3,
            consistency_level: "quorum".to_string(),
            partition_strategy: "hash".to_string(),
        }
    }
}

/// 配置加载和管理工具
impl UnifiedSystemConfig {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置有效性
    pub fn validate(&self) -> anyhow::Result<()> {
        // 验证核心配置
        if self.core.system_id.is_empty() {
            return Err(anyhow::anyhow!("System ID cannot be empty"));
        }

        // 验证交易所配置
        if self.exchanges.is_empty() {
            return Err(anyhow::anyhow!("At least one exchange must be configured"));
        }

        for exchange in &self.exchanges {
            if exchange.name.is_empty() {
                return Err(anyhow::anyhow!("Exchange name cannot be empty"));
            }
            if exchange.api.api_key.is_empty() {
                return Err(anyhow::anyhow!("API key cannot be empty for exchange {}", exchange.name));
            }
        }

        // 验证策略配置
        if self.strategy.enabled_strategies.is_empty() {
            return Err(anyhow::anyhow!("At least one strategy must be enabled"));
        }

        if self.strategy.min_profit_threshold <= 0.0 {
            return Err(anyhow::anyhow!("Min profit threshold must be positive"));
        }

        // 验证风险配置
        if self.risk.max_daily_loss_usd <= 0.0 {
            return Err(anyhow::anyhow!("Max daily loss must be positive"));
        }

        if self.risk.max_position_usd <= 0.0 {
            return Err(anyhow::anyhow!("Max position size must be positive"));
        }

        Ok(())
    }

    /// 获取特定交易所的配置
    pub fn get_exchange_config(&self, exchange_name: &str) -> Option<&ExchangeConfig> {
        self.exchanges.iter().find(|e| e.name == exchange_name)
    }

    /// 更新交易所配置
    pub fn update_exchange_config(&mut self, exchange_name: &str, config: ExchangeConfig) {
        if let Some(existing) = self.exchanges.iter_mut().find(|e| e.name == exchange_name) {
            *existing = config;
        } else {
            self.exchanges.push(config);
        }
    }

    /// 检查策略是否启用
    pub fn is_strategy_enabled(&self, strategy_name: &str) -> bool {
        self.strategy.enabled_strategies.contains(&strategy_name.to_string())
    }
}

/// 配置构建器，用于程序化创建配置
#[derive(Default)]
pub struct ConfigBuilder {
    config: UnifiedSystemConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system_id(mut self, system_id: impl Into<String>) -> Self {
        self.config.core.system_id = system_id.into();
        self
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.config.core.environment = environment.into();
        self
    }

    pub fn add_exchange(mut self, exchange: ExchangeConfig) -> Self {
        self.config.exchanges.push(exchange);
        self
    }

    pub fn with_strategy_config(mut self, strategy: StrategyConfig) -> Self {
        self.config.strategy = strategy;
        self
    }

    pub fn with_risk_config(mut self, risk: RiskConfig) -> Self {
        self.config.risk = risk;
        self
    }

    pub fn build(self) -> anyhow::Result<UnifiedSystemConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = UnifiedSystemConfig::default();
        assert!(config.validate().is_err()); // Should fail because no exchanges configured

        let mut config = config;
        config.exchanges.push(ExchangeConfig {
            name: "binance".to_string(),
            enabled: true,
            api: ApiConfig {
                base_url: "https://api.binance.com".to_string(),
                api_key: "test_key".to_string(),
                api_secret: "test_secret".to_string(),
                passphrase: None,
                timeout_ms: 5000,
                max_requests_per_second: 10,
            },
            connection: ConnectionConfig::default(),
            rate_limits: RateLimitConfig::default(),
            trading_fees: TradingFeeConfig {
                maker_fee_rate: 0.001,
                taker_fee_rate: 0.001,
                withdrawal_fees: HashMap::new(),
                fee_optimization: true,
            },
            precision: PrecisionConfig {
                price_precision: HashMap::new(),
                quantity_precision: HashMap::new(),
                min_order_size: HashMap::new(),
            },
            retry: RetryConfig::default(),
        });

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .with_system_id("test-system")
            .with_environment("test")
            .add_exchange(ExchangeConfig {
                name: "binance".to_string(),
                enabled: true,
                api: ApiConfig {
                    base_url: "https://api.binance.com".to_string(),
                    api_key: "test_key".to_string(),
                    api_secret: "test_secret".to_string(),
                    passphrase: None,
                    timeout_ms: 5000,
                    max_requests_per_second: 10,
                },
                connection: ConnectionConfig::default(),
                rate_limits: RateLimitConfig::default(),
                trading_fees: TradingFeeConfig {
                    maker_fee_rate: 0.001,
                    taker_fee_rate: 0.001,
                    withdrawal_fees: HashMap::new(),
                    fee_optimization: true,
                },
                precision: PrecisionConfig {
                    price_precision: HashMap::new(),
                    quantity_precision: HashMap::new(),
                    min_order_size: HashMap::new(),
                },
                retry: RetryConfig::default(),
            })
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.core.system_id, "test-system");
        assert_eq!(config.core.environment, "test");
        assert_eq!(config.exchanges.len(), 1);
        assert_eq!(config.exchanges[0].name, "binance");
    }
}