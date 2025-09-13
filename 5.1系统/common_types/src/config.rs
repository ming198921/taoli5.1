use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    pub system: SystemConfig,
    pub exchanges: HashMap<String, ExchangeConfig>,
    pub strategies: HashMap<String, StrategyConfig>,
    pub monitoring: MonitoringConfig,
    pub performance: PerformanceConfig,
    pub risk: RiskConfig,
    pub data_cleaning: DataCleaningConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub log_level: LogLevel,
    pub thread_pool_size: usize,
    pub max_memory_gb: f64,
    pub heartbeat_interval: Duration,
    pub shutdown_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub enabled: bool,
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: Option<String>,
    pub testnet: bool,
    pub rate_limit: RateLimitConfig,
    pub websocket: WebsocketConfig,
    pub rest_api: RestApiConfig,
    pub symbols: Vec<String>,
    pub trading_fees: TradingFeeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub requests_per_minute: u32,
    pub weight_per_second: Option<u32>,
    pub order_per_second: Option<u32>,
    pub burst_capacity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebsocketConfig {
    pub url: String,
    pub reconnect_interval: Duration,
    pub max_reconnect_attempts: u32,
    pub ping_interval: Duration,
    pub subscribe_timeout: Duration,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestApiConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub retry_attempts: u32,
    pub retry_delay: Duration,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingFeeConfig {
    pub maker_fee: f64,
    pub taker_fee: f64,
    pub vip_level: Option<u8>,
    pub fee_discount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub name: String,
    pub enabled: bool,
    pub strategy_type: StrategyType,
    pub parameters: StrategyParameters,
    pub risk_limits: RiskLimits,
    pub execution: ExecutionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Triangular,
    InterExchange,
    Statistical,
    MarketMaking,
    HighFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyParameters {
    pub min_profit_rate: f64,
    pub max_position_size: f64,
    pub min_volume: f64,
    pub max_slippage: f64,
    pub confidence_threshold: f64,
    pub lookback_period: Duration,
    pub update_interval: Duration,
    pub custom_params: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_daily_loss: f64,
    pub max_position_value: f64,
    pub max_leverage: f64,
    pub stop_loss_percentage: f64,
    pub max_concurrent_trades: u32,
    pub max_exposure_per_symbol: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub execution_mode: ExecutionMode,
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    pub post_only: bool,
    pub reduce_only: bool,
    pub iceberg_size: Option<f64>,
    pub execution_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    Aggressive,
    Passive,
    Smart,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    StopLimit,
    TakeProfit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    GTC,
    IOC,
    FOK,
    GTX,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub metrics: MetricsConfig,
    pub alerts: AlertConfig,
    pub logging: LoggingConfig,
    pub health_check: HealthCheckConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub prometheus_endpoint: String,
    pub collection_interval: Duration,
    pub retention_period: Duration,
    pub export_format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Prometheus,
    Json,
    Csv,
    InfluxDB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub email_alerts: bool,
    pub sms_alerts: bool,
    pub webhook_url: Option<String>,
    pub alert_thresholds: HashMap<String, f64>,
    pub cooldown_period: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub file_path: String,
    pub max_file_size: usize,
    pub max_files: usize,
    pub format: LogFormat,
    pub include_timestamps: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Plain,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub endpoint: String,
    pub interval: Duration,
    pub timeout: Duration,
    pub failure_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub optimization_level: OptimizationLevel,
    pub simd_enabled: bool,
    pub avx512_enabled: bool,
    pub cache_size: usize,
    pub buffer_pool_size: usize,
    pub zero_copy: bool,
    pub numa_aware: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Maximum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub risk_model: RiskModel,
    pub var_confidence: f64,
    pub stress_test_scenarios: Vec<StressScenario>,
    pub correlation_window: Duration,
    pub risk_update_interval: Duration,
    pub max_var: f64,
    pub max_drawdown: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskModel {
    Historical,
    MonteCarlo,
    Parametric,
    AI,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenario {
    pub name: String,
    pub market_shock: f64,
    pub volatility_multiplier: f64,
    pub correlation_break: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCleaningConfig {
    pub enabled: bool,
    pub outlier_detection: OutlierDetectionConfig,
    pub data_validation: DataValidationConfig,
    pub compression: CompressionConfig,
    pub retention: RetentionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierDetectionConfig {
    pub method: OutlierMethod,
    pub threshold: f64,
    pub window_size: usize,
    pub min_samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutlierMethod {
    ZScore,
    IQR,
    IsolationForest,
    LocalOutlierFactor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataValidationConfig {
    pub check_timestamps: bool,
    pub check_price_ranges: bool,
    pub check_volume_ranges: bool,
    pub check_sequence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub algorithm: CompressionAlgorithm,
    pub level: u32,
    pub min_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Zstd,
    Lz4,
    Snappy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    pub raw_data_days: u32,
    pub aggregated_data_days: u32,
    pub archive_enabled: bool,
    pub archive_path: Option<String>,
}

pub struct ConfigBuilder {
    config: UnifiedConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: UnifiedConfig::default(),
        }
    }

    pub fn with_system(mut self, system: SystemConfig) -> Self {
        self.config.system = system;
        self
    }

    pub fn add_exchange(mut self, name: String, exchange: ExchangeConfig) -> Self {
        self.config.exchanges.insert(name, exchange);
        self
    }

    pub fn add_strategy(mut self, name: String, strategy: StrategyConfig) -> Self {
        self.config.strategies.insert(name, strategy);
        self
    }

    pub fn with_monitoring(mut self, monitoring: MonitoringConfig) -> Self {
        self.config.monitoring = monitoring;
        self
    }

    pub fn with_performance(mut self, performance: PerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }

    pub fn with_risk(mut self, risk: RiskConfig) -> Self {
        self.config.risk = risk;
        self
    }

    pub fn with_data_cleaning(mut self, data_cleaning: DataCleaningConfig) -> Self {
        self.config.data_cleaning = data_cleaning;
        self
    }

    pub fn build(self) -> UnifiedConfig {
        self.config
    }
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            exchanges: HashMap::new(),
            strategies: HashMap::new(),
            monitoring: MonitoringConfig::default(),
            performance: PerformanceConfig::default(),
            risk: RiskConfig::default(),
            data_cleaning: DataCleaningConfig::default(),
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: "Arbitrage System 5.1".to_string(),
            version: "5.1.0".to_string(),
            environment: Environment::Development,
            log_level: LogLevel::Info,
            thread_pool_size: num_cpus::get(),
            max_memory_gb: 8.0,
            heartbeat_interval: Duration::from_secs(30),
            shutdown_timeout: Duration::from_secs(60),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics: MetricsConfig::default(),
            alerts: AlertConfig::default(),
            logging: LoggingConfig::default(),
            health_check: HealthCheckConfig::default(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            prometheus_endpoint: "http://localhost:9090".to_string(),
            collection_interval: Duration::from_secs(10),
            retention_period: Duration::from_secs(86400 * 30),
            export_format: ExportFormat::Prometheus,
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            email_alerts: false,
            sms_alerts: false,
            webhook_url: None,
            alert_thresholds: HashMap::new(),
            cooldown_period: Duration::from_secs(300),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            file_path: "/var/log/arbitrage/system.log".to_string(),
            max_file_size: 100 * 1024 * 1024,
            max_files: 10,
            format: LogFormat::Json,
            include_timestamps: true,
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Aggressive,
            simd_enabled: true,
            avx512_enabled: cfg!(target_feature = "avx512f"),
            cache_size: 64 * 1024 * 1024,
            buffer_pool_size: 1024,
            zero_copy: true,
            numa_aware: false,
        }
    }
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            risk_model: RiskModel::Historical,
            var_confidence: 0.95,
            stress_test_scenarios: vec![],
            correlation_window: Duration::from_secs(86400),
            risk_update_interval: Duration::from_secs(60),
            max_var: 0.05,
            max_drawdown: 0.10,
        }
    }
}

impl Default for DataCleaningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            outlier_detection: OutlierDetectionConfig::default(),
            data_validation: DataValidationConfig::default(),
            compression: CompressionConfig::default(),
            retention: RetentionConfig::default(),
        }
    }
}

impl Default for OutlierDetectionConfig {
    fn default() -> Self {
        Self {
            method: OutlierMethod::ZScore,
            threshold: 3.0,
            window_size: 100,
            min_samples: 10,
        }
    }
}

impl Default for DataValidationConfig {
    fn default() -> Self {
        Self {
            check_timestamps: true,
            check_price_ranges: true,
            check_volume_ranges: true,
            check_sequence: true,
        }
    }
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            min_size: 1024,
        }
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            raw_data_days: 7,
            aggregated_data_days: 90,
            archive_enabled: false,
            archive_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .with_system(SystemConfig::default())
            .add_exchange("binance".to_string(), ExchangeConfig {
                name: "binance".to_string(),
                enabled: true,
                api_key: "test".to_string(),
                secret_key: "test".to_string(),
                passphrase: None,
                testnet: false,
                rate_limit: RateLimitConfig {
                    requests_per_second: 10,
                    requests_per_minute: 600,
                    weight_per_second: Some(1200),
                    order_per_second: Some(10),
                    burst_capacity: 20,
                },
                websocket: WebsocketConfig {
                    url: "wss://stream.binance.com:9443/ws".to_string(),
                    reconnect_interval: Duration::from_secs(5),
                    max_reconnect_attempts: 10,
                    ping_interval: Duration::from_secs(30),
                    subscribe_timeout: Duration::from_secs(10),
                    buffer_size: 1024,
                },
                rest_api: RestApiConfig {
                    base_url: "https://api.binance.com".to_string(),
                    timeout: Duration::from_secs(30),
                    retry_attempts: 3,
                    retry_delay: Duration::from_secs(1),
                    proxy: None,
                },
                symbols: vec!["BTCUSDT".to_string()],
                trading_fees: TradingFeeConfig {
                    maker_fee: 0.001,
                    taker_fee: 0.001,
                    vip_level: None,
                    fee_discount: None,
                },
            })
            .build();

        assert_eq!(config.system.name, "Arbitrage System 5.1");
        assert!(config.exchanges.contains_key("binance"));
    }
}