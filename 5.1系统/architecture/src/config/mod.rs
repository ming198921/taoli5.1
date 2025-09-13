//! 配置模块
//! 
//! 包含系统配置、限制验证和参数管理

pub mod system_limits;
pub mod config_center;
pub mod structured_config;

pub use system_limits::{
    SystemLimitsValidator,
    SystemLimits,
    ValidationResult,
    ViolationType,
    ViolationSeverity,
    SystemStatus,
    ComplianceStatus,
    RiskLevel,
    RuntimeStats,
    LimitViolation,
};
pub use config_center::{ConfigCenter, ConfigItem};
pub use structured_config::{
    StructuredConfigCenter,
    MarketStateConfig,
    MinProfitConfig,
    MonitoringConfig,
    PerformanceConfig,
    AlertThresholds,
    RiskLimitsConfig,
    ExchangeConfig,
    AutoConfigDetection,
};

// 导出配置类型
use serde::{Deserialize, Serialize};

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemConfig {
    pub enable_monitoring: bool,
    pub enable_performance_optimization: bool,
    pub enable_auto_recovery: bool,
}

/// 风险配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskConfig {
    pub max_exposure_usd: f64,
    pub max_position_usd: f64,
    pub max_daily_loss_usd: f64,
    pub stop_loss_percentage: f64,
}

/// 策略配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StrategyConfig {
    pub strategy_id: String,
    pub strategy_type: String,
    pub enabled: bool,
    pub parameters: serde_json::Value,
}