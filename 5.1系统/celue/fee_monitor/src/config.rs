//! 费率监控配置模块
//! 
//! 提供可配置化的费率监控参数，支持5分钟间隔优化

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 费率监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeMonitorConfig {
    /// 监控间隔（秒）- 默认5分钟 (300秒)
    pub monitoring_interval_seconds: u64,
    
    /// WebSocket配置
    pub websocket_config: WebSocketConfig,
    
    /// 费率跟踪配置
    pub tracking_config: TrackingConfig,
    
    /// 优化配置
    pub optimization_config: OptimizationConfig,
    
    /// 预测配置
    pub prediction_config: PredictionConfig,
    
    /// 警报配置
    pub alert_config: AlertConfig,
    
    /// 存储配置
    pub storage_config: StorageConfig,
    
    /// 支持的交易所列表
    pub supported_exchanges: Vec<ExchangeConfig>,
}

/// WebSocket配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// 连接超时（秒）
    pub connection_timeout_seconds: u64,
    
    /// 重连间隔（秒）
    pub reconnect_interval_seconds: u64,
    
    /// 最大重连次数
    pub max_reconnect_attempts: u32,
    
    /// 心跳间隔（秒）
    pub heartbeat_interval_seconds: u64,
    
    /// 是否启用压缩
    pub enable_compression: bool,
    
    /// 每个交易所的最大并发连接数
    pub max_concurrent_connections_per_exchange: u32,
}

/// 跟踪配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingConfig {
    /// 历史数据保留天数
    pub history_retention_days: u32,
    
    /// 费率变化阈值（百分比）
    pub fee_change_threshold_percent: f64,
    
    /// 异常检测阈值
    pub anomaly_detection_threshold: f64,
    
    /// 数据采样率
    pub data_sampling_rate: f64,
    
    /// 是否启用实时计算
    pub enable_real_time_calculation: bool,
}

/// 优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// 优化算法类型
    pub algorithm_type: OptimizationAlgorithm,
    
    /// 优化目标
    pub optimization_target: OptimizationTarget,
    
    /// 风险权重
    pub risk_weight: f64,
    
    /// 最小改进阈值
    pub min_improvement_threshold: f64,
    
    /// 是否启用自适应优化
    pub enable_adaptive_optimization: bool,
}

/// 优化算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationAlgorithm {
    GreedyFirst,
    WeightedAverage,
    MachineLearning,
    Hybrid,
}

/// 优化目标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationTarget {
    MinimizeFees,
    MaximizeVolume,
    OptimizeSpread,
    Balanced,
}

/// 预测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    /// 预测模型类型
    pub model_type: PredictionModel,
    
    /// 预测窗口大小（分钟）
    pub prediction_window_minutes: u32,
    
    /// 训练数据窗口（小时）
    pub training_window_hours: u32,
    
    /// 模型更新间隔（小时）
    pub model_update_interval_hours: u32,
    
    /// 预测准确性阈值
    pub accuracy_threshold: f64,
}

/// 预测模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PredictionModel {
    LinearRegression,
    ARIMA,
    LSTM,
    EnsembleModel,
}

/// 警报配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// 是否启用警报
    pub enabled: bool,
    
    /// 高费率警报阈值（基点）
    pub high_fee_alert_threshold_bps: u32,
    
    /// 费率异常警报阈值（百分比）
    pub fee_anomaly_alert_threshold_percent: f64,
    
    /// 连接丢失警报延迟（秒）
    pub connection_loss_alert_delay_seconds: u64,
    
    /// 警报通知渠道
    pub notification_channels: Vec<NotificationChannel>,
    
    /// 警报抑制时间（分钟）
    pub alert_suppression_minutes: u32,
}

/// 通知渠道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email { recipients: Vec<String> },
    Slack { webhook_url: String },
    Discord { webhook_url: String },
    Log,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 存储后端类型
    pub backend_type: StorageBackend,
    
    /// 数据库连接字符串
    pub connection_string: Option<String>,
    
    /// 批量写入大小
    pub batch_write_size: u32,
    
    /// 写入间隔（秒）
    pub write_interval_seconds: u64,
    
    /// 数据压缩
    pub enable_compression: bool,
    
    /// 数据分区策略
    pub partitioning_strategy: PartitioningStrategy,
}

/// 存储后端
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackend {
    Memory,
    Redis,
    ClickHouse,
    InfluxDB,
    PostgreSQL,
}

/// 分区策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartitioningStrategy {
    Daily,
    Weekly,
    Monthly,
    ByExchange,
}

/// 交易所配置
/// Exchange configuration - using unified definition to eliminate duplication
/// The unified BaseExchangeConfig provides all the fields we need:
/// - name, enabled, api_key, supported_symbols (direct mapping)
/// - websocket_url (direct mapping)
/// - secret_key maps to api_secret
/// Plus additional standardized fields for consistency across the system
pub use common_types::{BaseExchangeConfig as ExchangeConfig, BaseConfig};
}

impl Default for FeeMonitorConfig {
    fn default() -> Self {
        Self {
            // 设置默认监控间隔为5分钟
            monitoring_interval_seconds: 300, // 5分钟
            
            websocket_config: WebSocketConfig {
                connection_timeout_seconds: 30,
                reconnect_interval_seconds: 5,
                max_reconnect_attempts: 10,
                heartbeat_interval_seconds: 30,
                enable_compression: true,
                max_concurrent_connections_per_exchange: 5,
            },
            
            tracking_config: TrackingConfig {
                history_retention_days: 30,
                fee_change_threshold_percent: 1.0,
                anomaly_detection_threshold: 2.5,
                data_sampling_rate: 1.0,
                enable_real_time_calculation: true,
            },
            
            optimization_config: OptimizationConfig {
                algorithm_type: OptimizationAlgorithm::Hybrid,
                optimization_target: OptimizationTarget::Balanced,
                risk_weight: 0.3,
                min_improvement_threshold: 0.1,
                enable_adaptive_optimization: true,
            },
            
            prediction_config: PredictionConfig {
                model_type: PredictionModel::EnsembleModel,
                prediction_window_minutes: 30,
                training_window_hours: 24,
                model_update_interval_hours: 6,
                accuracy_threshold: 0.8,
            },
            
            alert_config: AlertConfig {
                enabled: true,
                high_fee_alert_threshold_bps: 50,
                fee_anomaly_alert_threshold_percent: 10.0,
                connection_loss_alert_delay_seconds: 60,
                notification_channels: vec![NotificationChannel::Log],
                alert_suppression_minutes: 5,
            },
            
            storage_config: StorageConfig {
                backend_type: StorageBackend::ClickHouse,
                connection_string: None,
                batch_write_size: 1000,
                write_interval_seconds: 60,
                enable_compression: true,
                partitioning_strategy: PartitioningStrategy::Daily,
            },
            
            supported_exchanges: vec![
                ExchangeConfig {
                    name: "binance".to_string(),
                    enabled: true,
                    websocket_url: "wss://stream.binance.com:9443/ws".to_string(),
                    api_key: None,
                    secret_key: None,
                    supported_symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
                    fee_mapping: HashMap::new(),
                    custom_config: HashMap::new(),
                },
                ExchangeConfig {
                    name: "okx".to_string(),
                    enabled: true,
                    websocket_url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
                    api_key: None,
                    secret_key: None,
                    supported_symbols: vec!["BTC-USDT".to_string(), "ETH-USDT".to_string()],
                    fee_mapping: HashMap::new(),
                    custom_config: HashMap::new(),
                },
            ],
        }
    }
}

impl FeeMonitorConfig {
    /// 从文件加载配置
    pub async fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub async fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }
    
    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        // 验证监控间隔
        if self.monitoring_interval_seconds < 60 {
            return Err("Monitoring interval must be at least 60 seconds".to_string());
        }
        
        if self.monitoring_interval_seconds > 3600 {
            return Err("Monitoring interval should not exceed 1 hour".to_string());
        }
        
        // 验证交易所配置
        if self.supported_exchanges.is_empty() {
            return Err("At least one exchange must be configured".to_string());
        }
        
        for exchange in &self.supported_exchanges {
            if exchange.enabled && exchange.websocket_url.is_empty() {
                return Err(format!("WebSocket URL is required for enabled exchange: {}", exchange.name));
            }
        }
        
        // 验证警报配置
        if self.alert_config.enabled && self.alert_config.notification_channels.is_empty() {
            return Err("At least one notification channel must be configured when alerts are enabled".to_string());
        }
        
        Ok(())
    }
    
    /// 获取启用的交易所
    pub fn get_enabled_exchanges(&self) -> Vec<&ExchangeConfig> {
        self.supported_exchanges.iter().filter(|e| e.enabled).collect()
    }
    
    /// 设置监控间隔
    pub fn set_monitoring_interval(&mut self, seconds: u64) -> Result<(), String> {
        if seconds < 60 || seconds > 3600 {
            return Err("Monitoring interval must be between 60 and 3600 seconds".to_string());
        }
        self.monitoring_interval_seconds = seconds;
        Ok(())
    }
    
    /// 设置5分钟监控间隔
    pub fn set_five_minute_monitoring(&mut self) {
        self.monitoring_interval_seconds = 300; // 5分钟
    }
    
    /// 检查是否为5分钟间隔
    pub fn is_five_minute_monitoring(&self) -> bool {
        self.monitoring_interval_seconds == 300
    }
    
    /// 获取监控间隔的Duration
    pub fn get_monitoring_interval_duration(&self) -> Duration {
        Duration::from_secs(self.monitoring_interval_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = FeeMonitorConfig::default();
        assert_eq!(config.monitoring_interval_seconds, 300); // 5分钟
        assert!(config.is_five_minute_monitoring());
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_five_minute_monitoring() {
        let mut config = FeeMonitorConfig::default();
        
        // 测试设置5分钟间隔
        config.set_five_minute_monitoring();
        assert!(config.is_five_minute_monitoring());
        
        // 测试修改间隔
        assert!(config.set_monitoring_interval(600).is_ok()); // 10分钟
        assert!(!config.is_five_minute_monitoring());
        
        // 测试无效间隔
        assert!(config.set_monitoring_interval(30).is_err()); // 小于最小值
        assert!(config.set_monitoring_interval(7200).is_err()); // 大于最大值
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = FeeMonitorConfig::default();
        
        // 有效配置
        assert!(config.validate().is_ok());
        
        // 无效监控间隔
        config.monitoring_interval_seconds = 30;
        assert!(config.validate().is_err());
        
        config.monitoring_interval_seconds = 300;
        
        // 没有启用的交易所
        config.supported_exchanges.clear();
        assert!(config.validate().is_err());
    }
    
    #[tokio::test]
    async fn test_config_file_operations() {
        use tempfile::NamedTempFile;
        
        let config = FeeMonitorConfig::default();
        
        // 创建临时文件
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();
        
        // 保存配置
        assert!(config.save_to_file(temp_path).await.is_ok());
        
        // 加载配置
        let loaded_config = FeeMonitorConfig::from_file(temp_path).await.unwrap();
        assert_eq!(loaded_config.monitoring_interval_seconds, config.monitoring_interval_seconds);
        assert_eq!(loaded_config.supported_exchanges.len(), config.supported_exchanges.len());
    }
}