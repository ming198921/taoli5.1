//! 结构化配置类型定义
//! 
//! 替代serde_json::Value的结构化配置，提供类型安全的配置访问

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 市场状态配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStateConfig {
    /// 更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 价格变化阈值
    pub price_change_threshold: f64,
    /// 成交量变化阈值
    pub volume_change_threshold: f64,
    /// 极端波动率阈值
    pub extreme_volatility_threshold: f64,
    /// 谨慎波动率阈值
    pub caution_volatility_threshold: f64,
    /// 波动率权重
    pub volatility_weight: f64,
    /// 极端深度阈值
    pub extreme_depth_threshold: f64,
    /// 谨慎深度阈值
    pub caution_depth_threshold: f64,
    /// 深度权重
    pub depth_weight: f64,
    /// 极端成交量阈值
    pub extreme_volume_threshold: f64,
    /// 成交量权重
    pub volume_weight: f64,
    /// 极端API错误阈值
    pub extreme_api_error_threshold: f64,
    /// API权重
    pub api_weight: f64,
    /// 外部事件权重
    pub external_event_weight: f64,
    /// 极端阈值
    pub extreme_threshold: f64,
    /// 谨慎阈值
    pub caution_threshold: f64,
}

impl Default for MarketStateConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000,
            price_change_threshold: 0.01,
            volume_change_threshold: 0.05,
            extreme_volatility_threshold: 0.05,
            caution_volatility_threshold: 0.02,
            volatility_weight: 1.0,
            extreme_depth_threshold: 0.1,
            caution_depth_threshold: 0.3,
            depth_weight: 1.0,
            extreme_volume_threshold: 3.0,
            volume_weight: 0.5,
            extreme_api_error_threshold: 0.1,
            api_weight: 2.0,
            external_event_weight: 1.5,
            extreme_threshold: 8.0,
            caution_threshold: 5.0,
        }
    }
}

/// 最小利润配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinProfitConfig {
    /// 最小利润百分比
    pub min_profit_percentage: f64,
    /// 最小利润USDT
    pub min_profit_usdt: f64,
    /// 正常市场最小利润
    pub normal_min_profit: f64,
    /// 谨慎市场最小利润
    pub caution_min_profit: f64,
    /// 极端市场最小利润
    pub extreme_min_profit: f64,
    /// 自适应调整开关
    pub adaptive_adjustment: bool,
    /// 成功率阈值
    pub success_rate_threshold: f64,
    /// 调整因子
    pub adjustment_factor: f64,
}

impl Default for MinProfitConfig {
    fn default() -> Self {
        Self {
            min_profit_percentage: 0.001,
            min_profit_usdt: 1.0,
            normal_min_profit: 0.001,
            caution_min_profit: 0.0015,
            extreme_min_profit: 0.003,
            adaptive_adjustment: true,
            success_rate_threshold: 0.8,
            adjustment_factor: 0.1,
        }
    }
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 指标收集间隔（秒）
    pub metrics_interval_seconds: u64,
    /// 性能监控配置
    pub performance: PerformanceConfig,
    /// 告警阈值
    pub alert_thresholds: AlertThresholds,
}

/// 性能监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 启用性能监控
    pub enable_performance_monitoring: bool,
    /// 延迟阈值（毫秒）
    pub latency_threshold_ms: u64,
    /// 吞吐量阈值（操作/秒）
    pub throughput_threshold_ops: u64,
    /// 成功率阈值
    pub success_rate_threshold: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            latency_threshold_ms: 500,
            throughput_threshold_ops: 1000,
            success_rate_threshold: 0.99,
        }
    }
}

/// 告警阈值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// CPU使用率告警阈值
    pub cpu_usage: f64,
    /// 内存使用率告警阈值
    pub memory_usage: f64,
    /// 错误率告警阈值
    pub error_rate: f64,
    /// 响应时间告警阈值（毫秒）
    pub response_time_ms: u64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cpu_usage: 80.0,
            memory_usage: 90.0,
            error_rate: 0.01,
            response_time_ms: 1000,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_interval_seconds: 30,
            performance: PerformanceConfig::default(),
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// 交易所配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub api_passphrase: Option<String>,
    pub sandbox_mode: bool,
    pub rate_limit: u32,
    pub websocket_url: String,
    pub rest_api_url: String,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            enabled: true,
            api_key: "".to_string(),
            api_secret: "".to_string(),
            api_passphrase: None,
            sandbox_mode: true,
            rate_limit: 10,
            websocket_url: "".to_string(),
            rest_api_url: "".to_string(),
        }
    }
}

/// 风险限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimitsConfig {
    /// 最大敞口
    pub max_exposure: f64,
    /// 最大仓位
    pub max_position: f64,
    /// 最大日损失
    pub max_daily_loss: f64,
    /// 单笔交易最大金额
    pub max_trade_amount: f64,
    /// 停损比例
    pub stop_loss_ratio: f64,
}

impl Default for RiskLimitsConfig {
    fn default() -> Self {
        Self {
            max_exposure: 100000.0,
            max_position: 50000.0,
            max_daily_loss: 5000.0,
            max_trade_amount: 10000.0,
            stop_loss_ratio: 0.02,
        }
    }
}

/// 结构化配置中心
#[derive(Debug, Clone)]
pub struct StructuredConfigCenter {
    /// 市场状态配置
    pub market_state_config: MarketStateConfig,
    /// 最小利润配置
    pub min_profit_config: MinProfitConfig,
    /// 监控配置
    pub monitoring_config: MonitoringConfig,
    /// 风险限制配置
    pub risk_limits_config: RiskLimitsConfig,
    /// 自定义配置项
    pub custom_configs: HashMap<String, serde_json::Value>,
}

impl StructuredConfigCenter {
    /// 创建新的结构化配置中心
    pub fn new() -> Self {
        Self {
            market_state_config: MarketStateConfig::default(),
            min_profit_config: MinProfitConfig::default(),
            monitoring_config: MonitoringConfig::default(),
            risk_limits_config: RiskLimitsConfig::default(),
            custom_configs: HashMap::new(),
        }
    }

    /// 从配置文件加载
    pub async fn load_from_file(_config_path: &str) -> Result<Self> {
        // 实际实现中会从配置文件读取
        // 这里为了编译通过返回默认配置
        Ok(Self::new())
    }

    /// 获取市场状态配置
    pub fn get_market_state_config(&self) -> &MarketStateConfig {
        &self.market_state_config
    }

    /// 获取最小利润配置
    pub fn get_min_profit_config(&self) -> &MinProfitConfig {
        &self.min_profit_config
    }

    /// 获取监控配置
    pub fn get_monitoring_config(&self) -> &MonitoringConfig {
        &self.monitoring_config
    }

    /// 获取风险限制配置
    pub fn get_risk_limits_config(&self) -> &RiskLimitsConfig {
        &self.risk_limits_config
    }

    /// 更新配置
    pub fn update_config<T: Serialize>(&mut self, key: &str, value: T) -> Result<()> {
        let json_value = serde_json::to_value(value)?;
        self.custom_configs.insert(key.to_string(), json_value);
        Ok(())
    }

    /// 更新市场状态配置
    pub fn update_market_state_config(&mut self, config: MarketStateConfig) {
        self.market_state_config = config;
    }

    /// 更新最小利润配置
    pub fn update_min_profit_config(&mut self, config: MinProfitConfig) {
        self.min_profit_config = config;
    }

    /// 更新监控配置
    pub fn update_monitoring_config(&mut self, config: MonitoringConfig) {
        self.monitoring_config = config;
    }

    /// 更新风险限制配置
    pub fn update_risk_limits_config(&mut self, config: RiskLimitsConfig) {
        self.risk_limits_config = config;
    }

    /// 序列化配置到JSON
    pub fn to_json(&self) -> Result<String> {
        let config_json = serde_json::json!({
            "market_state_config": self.market_state_config,
            "min_profit_config": self.min_profit_config,
            "monitoring_config": self.monitoring_config,
            "risk_limits_config": self.risk_limits_config,
            "custom_configs": self.custom_configs
        });
        Ok(serde_json::to_string_pretty(&config_json)?)
    }

    /// 从JSON反序列化配置
    pub fn from_json(json_str: &str) -> Result<Self> {
        let config_value: serde_json::Value = serde_json::from_str(json_str)?;
        
        let market_state_config = if let Some(config) = config_value.get("market_state_config") {
            serde_json::from_value(config.clone())?
        } else {
            MarketStateConfig::default()
        };

        let min_profit_config = if let Some(config) = config_value.get("min_profit_config") {
            serde_json::from_value(config.clone())?
        } else {
            MinProfitConfig::default()
        };

        let monitoring_config = if let Some(config) = config_value.get("monitoring_config") {
            serde_json::from_value(config.clone())?
        } else {
            MonitoringConfig::default()
        };

        let risk_limits_config = if let Some(config) = config_value.get("risk_limits_config") {
            serde_json::from_value(config.clone())?
        } else {
            RiskLimitsConfig::default()
        };

        let custom_configs = if let Some(configs) = config_value.get("custom_configs") {
            serde_json::from_value(configs.clone())?
        } else {
            HashMap::new()
        };

        Ok(Self {
            market_state_config,
            min_profit_config,
            monitoring_config,
            risk_limits_config,
            custom_configs,
        })
    }
}

impl Default for StructuredConfigCenter {
    fn default() -> Self {
        Self::new()
    }
}

/// 自动配置检测
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConfigDetection {
    /// 启用性能适应
    pub enable_performance_adaptation: bool,
    /// 最大自动缩放因子
    pub max_auto_scale_factor: f64,
    /// 启用硬件检测
    pub enable_hardware_detection: bool,
    /// 启用网络延迟优化
    pub enable_network_optimization: bool,
}

impl Default for AutoConfigDetection {
    fn default() -> Self {
        Self {
            enable_performance_adaptation: true,
            max_auto_scale_factor: 4.0,
            enable_hardware_detection: true,
            enable_network_optimization: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_config_creation() {
        let config = StructuredConfigCenter::new();
        assert_eq!(config.market_state_config.update_interval_ms, 1000);
        assert_eq!(config.min_profit_config.min_profit_percentage, 0.001);
        assert!(config.monitoring_config.performance.enable_performance_monitoring);
    }

    #[test]
    fn test_config_serialization() {
        let config = StructuredConfigCenter::new();
        let json_str = config.to_json().unwrap();
        assert!(json_str.contains("market_state_config"));
        assert!(json_str.contains("min_profit_config"));
    }

    #[test]
    fn test_config_deserialization() {
        let config = StructuredConfigCenter::new();
        let json_str = config.to_json().unwrap();
        let restored_config = StructuredConfigCenter::from_json(&json_str).unwrap();
        
        assert_eq!(
            config.market_state_config.update_interval_ms, 
            restored_config.market_state_config.update_interval_ms
        );
    }

    #[test]
    fn test_config_updates() {
        let mut config = StructuredConfigCenter::new();
        
        // 更新市场状态配置
        let mut market_config = config.market_state_config.clone();
        market_config.extreme_threshold = 10.0;
        config.update_market_state_config(market_config);
        
        assert_eq!(config.market_state_config.extreme_threshold, 10.0);
    }
}