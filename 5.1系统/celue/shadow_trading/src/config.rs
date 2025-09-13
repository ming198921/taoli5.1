//! 影子交易系统配置

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 影子交易系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowTradingConfig {
    /// 虚拟账户配置
    pub account: VirtualAccountConfig,
    /// 执行引擎配置
    pub execution: ExecutionEngineConfig,
    /// 市场模拟配置
    pub market_simulation: MarketSimulationConfig,
    /// 性能分析配置
    pub performance: PerformanceAnalysisConfig,
    /// 风险管理配置
    pub risk_limits: RiskLimitsConfig,
    /// 订单匹配配置
    pub matching: OrderMatchingConfig,
    /// 指标收集配置
    pub metrics: MetricsConfig,
}

/// 虚拟账户配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualAccountConfig {
    /// 默认基础货币
    pub base_currency: String,
    /// 最大持仓数量
    pub max_positions: usize,
    /// 最小余额阈值
    pub min_balance_threshold: f64,
    /// 是否启用保证金交易
    pub enable_margin: bool,
    /// 最大杠杆倍数
    pub max_leverage: f64,
    /// 手续费率
    pub fee_rates: HashMap<String, f64>,
}

impl Default for VirtualAccountConfig {
    fn default() -> Self {
        let mut fee_rates = HashMap::new();
        fee_rates.insert("maker".to_string(), 0.001); // 0.1%
        fee_rates.insert("taker".to_string(), 0.001); // 0.1%
        
        Self {
            base_currency: "USDT".to_string(),
            max_positions: 100,
            min_balance_threshold: 0.01,
            enable_margin: false,
            max_leverage: 1.0,
            fee_rates,
        }
    }
}

/// 执行引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEngineConfig {
    /// 最大并发订单数
    pub max_concurrent_orders: usize,
    /// 订单超时时间（秒）
    pub order_timeout_seconds: u64,
    /// 部分成交最小间隔（毫秒）
    pub partial_fill_interval_ms: u64,
    /// 是否启用滑点模拟
    pub enable_slippage_simulation: bool,
    /// 平均滑点基点
    pub average_slippage_bps: f64,
    /// 最大滑点基点
    pub max_slippage_bps: f64,
    /// 市价单立即成交概率
    pub market_order_immediate_fill_prob: f64,
    /// 限价单成交概率模型
    pub limit_order_fill_model: FillModel,
}

impl Default for ExecutionEngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_orders: 1000,
            order_timeout_seconds: 3600, // 1小时
            partial_fill_interval_ms: 100,
            enable_slippage_simulation: true,
            average_slippage_bps: 5.0, // 0.05%
            max_slippage_bps: 50.0, // 0.5%
            market_order_immediate_fill_prob: 0.95,
            limit_order_fill_model: FillModel::Probability,
        }
    }
}

/// 成交模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FillModel {
    /// 基于概率的成交模型
    Probability,
    /// 基于订单簿深度的模型
    OrderBookDepth,
    /// 基于历史数据的模型
    Historical,
}

/// 市场模拟配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSimulationConfig {
    /// 支持的交易对
    pub supported_symbols: Vec<String>,
    /// 价格更新间隔（毫秒）
    pub price_update_interval_ms: u64,
    /// 波动率模型
    pub volatility_model: VolatilityModel,
    /// 基础波动率
    pub base_volatility: f64,
    /// 趋势持续时间（秒）
    pub trend_duration_seconds: u64,
    /// 是否启用跳跃模拟
    pub enable_jump_simulation: bool,
    /// 跳跃概率
    pub jump_probability: f64,
    /// 平均跳跃幅度
    pub average_jump_size: f64,
}

impl Default for MarketSimulationConfig {
    fn default() -> Self {
        Self {
            supported_symbols: vec![
                "BTC/USDT".to_string(),
                "ETH/USDT".to_string(),
                "BNB/USDT".to_string(),
                "ADA/USDT".to_string(),
                "SOL/USDT".to_string(),
            ],
            price_update_interval_ms: 100,
            volatility_model: VolatilityModel::GBM,
            base_volatility: 0.02, // 2% daily volatility
            trend_duration_seconds: 3600, // 1小时
            enable_jump_simulation: true,
            jump_probability: 0.01, // 1%
            average_jump_size: 0.005, // 0.5%
        }
    }
}

/// 波动率模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolatilityModel {
    /// 几何布朗运动
    GBM,
    /// 赫斯顿模型
    Heston,
    /// GARCH模型
    GARCH,
}

/// 性能分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisConfig {
    /// 基准收益率（年化）
    pub benchmark_return: f64,
    /// 无风险利率（年化）
    pub risk_free_rate: f64,
    /// 回测窗口大小
    pub lookback_window_days: u32,
    /// 计算间隔（秒）
    pub calculation_interval_seconds: u64,
    /// 是否启用实时VaR计算
    pub enable_realtime_var: bool,
    /// VaR置信水平
    pub var_confidence_level: f64,
    /// 最大回撤计算窗口
    pub max_drawdown_window_days: u32,
}

impl Default for PerformanceAnalysisConfig {
    fn default() -> Self {
        Self {
            benchmark_return: 0.08, // 8% 年化收益率
            risk_free_rate: 0.02, // 2% 无风险利率
            lookback_window_days: 252, // 一年交易日
            calculation_interval_seconds: 300, // 5分钟
            enable_realtime_var: true,
            var_confidence_level: 0.95, // 95% 置信水平
            max_drawdown_window_days: 1000, // 约4年
        }
    }
}

/// 风险限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimitsConfig {
    /// 最大单日损失限制（百分比）
    pub max_daily_loss_pct: f64,
    /// 最大总损失限制（百分比）
    pub max_total_loss_pct: f64,
    /// 最大持仓集中度（百分比）
    pub max_position_concentration_pct: f64,
    /// 最大单笔订单金额（基础货币）
    pub max_order_value: f64,
    /// 最大VaR限制
    pub max_var_limit: f64,
    /// 是否在违规时自动停止交易
    pub auto_stop_on_violation: bool,
    /// 风险检查间隔（秒）
    pub risk_check_interval_seconds: u64,
    /// 是否启用动态风险限制
    pub enable_dynamic_limits: bool,
}

impl Default for RiskLimitsConfig {
    fn default() -> Self {
        Self {
            max_daily_loss_pct: 0.05, // 5% 日损失限制
            max_total_loss_pct: 0.20, // 20% 总损失限制
            max_position_concentration_pct: 0.30, // 30% 持仓集中度限制
            max_order_value: 50000.0, // $50,000 最大订单
            max_var_limit: 0.10, // 10% VaR限制
            auto_stop_on_violation: true,
            risk_check_interval_seconds: 30, // 30秒检查一次
            enable_dynamic_limits: false,
        }
    }
}

/// 订单匹配配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMatchingConfig {
    /// 匹配算法
    pub matching_algorithm: MatchingAlgorithm,
    /// 最小价格变动单位
    pub tick_sizes: HashMap<String, f64>,
    /// 最小交易数量
    pub min_quantities: HashMap<String, f64>,
    /// 订单簿最大深度
    pub max_orderbook_depth: usize,
    /// 是否启用自成交保护
    pub enable_self_trade_prevention: bool,
    /// 延迟模拟配置
    pub latency_simulation: LatencySimulation,
}

impl Default for OrderMatchingConfig {
    fn default() -> Self {
        let mut tick_sizes = HashMap::new();
        tick_sizes.insert("BTC/USDT".to_string(), 0.01);
        tick_sizes.insert("ETH/USDT".to_string(), 0.01);
        tick_sizes.insert("default".to_string(), 0.0001);
        
        let mut min_quantities = HashMap::new();
        min_quantities.insert("BTC/USDT".to_string(), 0.0001);
        min_quantities.insert("ETH/USDT".to_string(), 0.001);
        min_quantities.insert("default".to_string(), 0.01);
        
        Self {
            matching_algorithm: MatchingAlgorithm::PriceTimePriority,
            tick_sizes,
            min_quantities,
            max_orderbook_depth: 1000,
            enable_self_trade_prevention: true,
            latency_simulation: LatencySimulation::default(),
        }
    }
}

/// 匹配算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchingAlgorithm {
    /// 价格时间优先
    PriceTimePriority,
    /// 价格数量优先  
    PriceQuantityPriority,
    /// Pro-Rata匹配
    ProRata,
}

/// 延迟模拟配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencySimulation {
    /// 是否启用延迟模拟
    pub enabled: bool,
    /// 基础延迟（微秒）
    pub base_latency_us: u64,
    /// 延迟抖动范围（微秒）
    pub jitter_range_us: u64,
    /// 网络拥塞模拟
    pub enable_congestion: bool,
    /// 拥塞概率
    pub congestion_probability: f64,
    /// 拥塞时的额外延迟（微秒）
    pub congestion_extra_latency_us: u64,
}

impl Default for LatencySimulation {
    fn default() -> Self {
        Self {
            enabled: true,
            base_latency_us: 1000, // 1ms
            jitter_range_us: 500, // ±0.5ms
            enable_congestion: true,
            congestion_probability: 0.05, // 5%
            congestion_extra_latency_us: 10000, // 10ms
        }
    }
}

/// 指标收集配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// 是否启用指标收集
    pub enabled: bool,
    /// 指标推送间隔（秒）
    pub push_interval_seconds: u64,
    /// 历史数据保留天数
    pub retention_days: u32,
    /// Prometheus推送网关地址
    pub prometheus_pushgateway_url: Option<String>,
    /// 作业名称
    pub job_name: String,
    /// 实例标识
    pub instance_id: String,
    /// 额外标签
    pub labels: HashMap<String, String>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            push_interval_seconds: 60, // 1分钟
            retention_days: 30,
            prometheus_pushgateway_url: None,
            job_name: "shadow_trading".to_string(),
            instance_id: "shadow_trading_01".to_string(),
            labels: HashMap::new(),
        }
    }
}

impl Default for ShadowTradingConfig {
    fn default() -> Self {
        Self {
            account: VirtualAccountConfig::default(),
            execution: ExecutionEngineConfig::default(),
            market_simulation: MarketSimulationConfig::default(),
            performance: PerformanceAnalysisConfig::default(),
            risk_limits: RiskLimitsConfig::default(),
            matching: OrderMatchingConfig::default(),
            metrics: MetricsConfig::default(),
        }
    }
}

impl ShadowTradingConfig {
    /// 从TOML文件加载配置
    pub fn from_toml_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: ShadowTradingConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// 保存配置到TOML文件
    pub fn save_to_toml_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置参数
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 验证账户配置
        if self.account.max_positions == 0 {
            return Err("Max positions must be greater than 0".into());
        }

        if self.account.max_leverage <= 0.0 {
            return Err("Max leverage must be positive".into());
        }

        // 验证执行引擎配置
        if self.execution.max_concurrent_orders == 0 {
            return Err("Max concurrent orders must be greater than 0".into());
        }

        if self.execution.average_slippage_bps < 0.0 || self.execution.max_slippage_bps < 0.0 {
            return Err("Slippage values must be non-negative".into());
        }

        if self.execution.market_order_immediate_fill_prob < 0.0 || 
           self.execution.market_order_immediate_fill_prob > 1.0 {
            return Err("Market order fill probability must be between 0 and 1".into());
        }

        // 验证市场模拟配置
        if self.market_simulation.supported_symbols.is_empty() {
            return Err("At least one trading symbol must be supported".into());
        }

        if self.market_simulation.base_volatility <= 0.0 {
            return Err("Base volatility must be positive".into());
        }

        // 验证风险限制配置
        if self.risk_limits.max_daily_loss_pct <= 0.0 || self.risk_limits.max_daily_loss_pct > 1.0 {
            return Err("Max daily loss percentage must be between 0 and 1".into());
        }

        if self.risk_limits.max_total_loss_pct <= 0.0 || self.risk_limits.max_total_loss_pct > 1.0 {
            return Err("Max total loss percentage must be between 0 and 1".into());
        }

        // 验证性能分析配置
        if self.performance.var_confidence_level <= 0.0 || self.performance.var_confidence_level >= 1.0 {
            return Err("VaR confidence level must be between 0 and 1 (exclusive)".into());
        }

        // 验证订单匹配配置
        if self.matching.max_orderbook_depth == 0 {
            return Err("Max orderbook depth must be greater than 0".into());
        }

        Ok(())
    }

    /// 应用环境变量覆盖
    pub fn apply_env_overrides(&mut self) {
        // 账户配置环境变量
        if let Ok(val) = std::env::var("SHADOW_MAX_POSITIONS") {
            if let Ok(positions) = val.parse::<usize>() {
                self.account.max_positions = positions;
            }
        }

        if let Ok(val) = std::env::var("SHADOW_MAX_LEVERAGE") {
            if let Ok(leverage) = val.parse::<f64>() {
                self.account.max_leverage = leverage;
            }
        }

        // 风险限制环境变量
        if let Ok(val) = std::env::var("SHADOW_MAX_DAILY_LOSS") {
            if let Ok(loss) = val.parse::<f64>() {
                self.risk_limits.max_daily_loss_pct = loss;
            }
        }

        if let Ok(val) = std::env::var("SHADOW_AUTO_STOP") {
            if let Ok(auto_stop) = val.parse::<bool>() {
                self.risk_limits.auto_stop_on_violation = auto_stop;
            }
        }

        // 指标配置环境变量
        if let Ok(url) = std::env::var("PROMETHEUS_PUSHGATEWAY_URL") {
            self.metrics.prometheus_pushgateway_url = Some(url);
        }

        if let Ok(instance) = std::env::var("SHADOW_INSTANCE_ID") {
            self.metrics.instance_id = instance;
        }
    }

    /// 创建用于开发的配置
    pub fn for_development() -> Self {
        let mut config = Self::default();
        
        // 降低风险限制用于测试
        config.risk_limits.max_daily_loss_pct = 0.10; // 10%
        config.risk_limits.max_total_loss_pct = 0.50; // 50%
        config.risk_limits.auto_stop_on_violation = false;
        
        // 更快的更新间隔
        config.market_simulation.price_update_interval_ms = 50;
        config.performance.calculation_interval_seconds = 60;
        config.metrics.push_interval_seconds = 30;
        
        // 启用更多模拟功能
        config.execution.enable_slippage_simulation = true;
        config.market_simulation.enable_jump_simulation = true;
        config.matching.latency_simulation.enabled = true;
        
        config
    }

    /// 创建用于生产的配置
    pub fn for_production() -> Self {
        let mut config = Self::default();
        
        // 严格的风险控制
        config.risk_limits.max_daily_loss_pct = 0.02; // 2%
        config.risk_limits.max_total_loss_pct = 0.10; // 10%
        config.risk_limits.auto_stop_on_violation = true;
        
        // 保守的性能设置
        config.market_simulation.price_update_interval_ms = 1000;
        config.performance.calculation_interval_seconds = 300;
        config.metrics.push_interval_seconds = 60;
        
        // 增加订单簿深度
        config.matching.max_orderbook_depth = 5000;
        
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = ShadowTradingConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_development_config() {
        let config = ShadowTradingConfig::for_development();
        assert!(config.validate().is_ok());
        assert_eq!(config.risk_limits.max_daily_loss_pct, 0.10);
        assert!(!config.risk_limits.auto_stop_on_violation);
    }

    #[test]
    fn test_production_config() {
        let config = ShadowTradingConfig::for_production();
        assert!(config.validate().is_ok());
        assert_eq!(config.risk_limits.max_daily_loss_pct, 0.02);
        assert!(config.risk_limits.auto_stop_on_violation);
    }

    #[test]
    fn test_config_validation_failures() {
        let mut config = ShadowTradingConfig::default();
        
        // 测试无效的最大持仓数
        config.account.max_positions = 0;
        assert!(config.validate().is_err());
        
        config.account.max_positions = 100;
        
        // 测试无效的杠杆
        config.account.max_leverage = -1.0;
        assert!(config.validate().is_err());
        
        config.account.max_leverage = 1.0;
        
        // 测试无效的损失限制
        config.risk_limits.max_daily_loss_pct = 1.5;
        assert!(config.validate().is_err());
        
        config.risk_limits.max_daily_loss_pct = 0.05;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_env_overrides() {
        std::env::set_var("SHADOW_MAX_POSITIONS", "500");
        std::env::set_var("SHADOW_MAX_LEVERAGE", "5.0");
        std::env::set_var("SHADOW_AUTO_STOP", "false");
        
        let mut config = ShadowTradingConfig::default();
        config.apply_env_overrides();
        
        assert_eq!(config.account.max_positions, 500);
        assert_eq!(config.account.max_leverage, 5.0);
        assert!(!config.risk_limits.auto_stop_on_violation);
        
        // 清理环境变量
        std::env::remove_var("SHADOW_MAX_POSITIONS");
        std::env::remove_var("SHADOW_MAX_LEVERAGE");
        std::env::remove_var("SHADOW_AUTO_STOP");
    }
}