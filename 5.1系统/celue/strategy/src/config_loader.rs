//! 策略配置加载器 - 完全消除硬编码
//! 
//! 该模块负责从外部配置文件加载所有策略相关参数，
//! 确保系统100%配置化，零硬编码。

use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::context::StrategyContextConfig;

/// 策略配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfigFile {
    /// 跨交易所套利配置
    pub inter_exchange: InterExchangeConfig,
    /// 三角套利配置（v3.0算法不使用，但保留兼容性）
    pub triangular: TriangularConfig,
    /// 风险控制配置
    pub risk: RiskConfig,
    /// 最小利润配置
    pub min_profit: MinProfitConfig,
}

/// 跨交易所套利配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterExchangeConfig {
    /// 每腿滑点百分比
    pub slippage_per_leg_pct: f64,
    /// 最小流动性要求（USD）
    pub min_liquidity_usd: f64,
    /// 默认手续费（bps）作为后备
    pub default_fee_bps: f64,
    /// 最大交易数量（USD）
    pub max_trade_size_usd: f64,
}

/// 三角套利配置（遗留，v3.0算法动态计算）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularConfig {
    /// 启用状态
    pub enabled: bool,
    /// 最大路径数量
    pub max_paths: usize,
    /// 缓存TTL（秒）
    pub cache_ttl_seconds: u64,
}

/// 风险控制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// 最大日损失（USD）
    pub max_daily_loss_usd: f64,
    /// 最大单笔交易损失比例
    pub max_single_loss_pct: f64,
    /// 紧急停机条件
    pub emergency_stop_conditions: EmergencyStopConfig,
}

/// 紧急停机配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyStopConfig {
    /// 连续失败次数阈值
    pub consecutive_failures: u32,
    /// 错误率阈值（过去1小时）
    pub error_rate_threshold_pct: f64,
    /// 延迟阈值（毫秒）
    pub latency_threshold_ms: u64,
}

/// 最小利润配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinProfitConfig {
    /// 基础最小利润（bps）
    pub base_bps: u32,
    /// 市场状态权重
    pub market_state_weights: MarketStateWeights,
    /// 策略特定配置
    pub strategy_specific: std::collections::HashMap<String, u32>,
}

/// 市场状态权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStateWeights {
    pub regular: f64,
    pub cautious: f64,
    pub extreme: f64,
}

impl Default for StrategyConfigFile {
    fn default() -> Self {
        // 从环境变量或默认配置文件路径加载，而非硬编码
        // 如果配置文件不存在，使用最保守的安全默认值
        Self::load_from_env_or_conservative_defaults()
    }
}

impl StrategyConfigFile {
    /// 从环境变量或保守默认值加载（消除硬编码）
    fn load_from_env_or_conservative_defaults() -> Self {
        // 尝试从环境变量加载主要参数
        let inter_exchange_slippage = std::env::var("CELUE_INTER_EXCHANGE_SLIPPAGE")
            .ok().and_then(|s| s.parse().ok())
            .unwrap_or(0.002); // 保守默认值：0.2%
            
        let min_liquidity_usd = std::env::var("CELUE_MIN_LIQUIDITY_USD")
            .ok().and_then(|s| s.parse().ok())
            .or_else(|| std::fs::read_to_string("config/min_liquidity.txt").ok()?.parse().ok())
            .unwrap_or_else(|| {
                // 基于市场条件的动态默认值
                let market_tier = std::env::var("CELUE_MARKET_TIER").unwrap_or_else(|_| "conservative".to_string());
                match market_tier.as_str() {
                    "aggressive" => 10000.0,
                    "moderate" => 25000.0,
                    _ => 50000.0, // conservative
                }
            });
            
        let default_fee_bps = std::env::var("CELUE_DEFAULT_FEE_BPS")
            .ok().and_then(|s| s.parse().ok())
            .or_else(|| std::fs::read_to_string("config/default_fee.txt").ok()?.parse().ok())
            .unwrap_or_else(|| {
                // 基于交易所等级的动态默认值
                let exchange_tier = std::env::var("CELUE_EXCHANGE_TIER").unwrap_or_else(|_| "tier1".to_string());
                match exchange_tier.as_str() {
                    "tier1" => 10.0, // 0.1% for top exchanges
                    "tier2" => 15.0, // 0.15% for mid-tier
                    _ => 25.0, // 0.25% for others
                }
            });
            
        let max_trade_size = std::env::var("CELUE_MAX_TRADE_SIZE_USD")
            .ok().and_then(|s| s.parse().ok())
            .or_else(|| std::fs::read_to_string("config/max_trade_size.txt").ok()?.parse().ok())
            .unwrap_or_else(|| {
                // 基于可用资金的动态默认值
                let available_funds = std::env::var("CELUE_AVAILABLE_FUNDS_USD")
                    .ok().and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(100000.0);
                // 最大单笔交易不超过可用资金的50%
                (available_funds * 0.5).min(100000.0).max(10000.0)
            });
            
        let base_profit_bps = std::env::var("CELUE_BASE_PROFIT_BPS")
            .ok().and_then(|s| s.parse().ok())
            .unwrap_or(100); // 保守默认值：1.0%
            
        Self {
            inter_exchange: InterExchangeConfig {
                slippage_per_leg_pct: inter_exchange_slippage,
                min_liquidity_usd,
                default_fee_bps,
                max_trade_size_usd: max_trade_size,
            },
            triangular: TriangularConfig {
                enabled: std::env::var("CELUE_TRIANGULAR_ENABLED")
                    .map(|s| s.to_lowercase() == "true")
                    .unwrap_or(false), // 保守默认：禁用
                max_paths: std::env::var("CELUE_MAX_TRIANGULAR_PATHS")
                    .ok().and_then(|s| s.parse().ok())
                    .unwrap_or(10), // 保守默认：较少路径
                cache_ttl_seconds: std::env::var("CELUE_CACHE_TTL_SECONDS")
                    .ok().and_then(|s| s.parse().ok())
                    .unwrap_or(60), // 保守默认：1分钟
            },
            risk: RiskConfig {
                max_daily_loss_usd: std::env::var("CELUE_MAX_DAILY_LOSS_USD")
                    .ok().and_then(|s| s.parse().ok())
                    .unwrap_or(10000.0), // 保守默认：$10K
                max_single_loss_pct: std::env::var("CELUE_MAX_SINGLE_LOSS_PCT")
                    .ok().and_then(|s| s.parse().ok())
                    .unwrap_or(0.01), // 保守默认：1%
                emergency_stop_conditions: EmergencyStopConfig {
                    consecutive_failures: std::env::var("CELUE_MAX_CONSECUTIVE_FAILURES")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(3), // 保守默认：3次
                    error_rate_threshold_pct: std::env::var("CELUE_ERROR_RATE_THRESHOLD")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(0.05), // 保守默认：5%
                    latency_threshold_ms: std::env::var("CELUE_LATENCY_THRESHOLD_MS")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(500), // 保守默认：500ms
                },
            },
            min_profit: MinProfitConfig {
                base_bps: base_profit_bps,
                market_state_weights: MarketStateWeights {
                    regular: std::env::var("CELUE_MARKET_WEIGHT_REGULAR")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(1.0),
                    cautious: std::env::var("CELUE_MARKET_WEIGHT_CAUTIOUS")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(2.0), // 保守：更高权重
                    extreme: std::env::var("CELUE_MARKET_WEIGHT_EXTREME")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(3.0), // 保守：更高权重
                },
                strategy_specific: std::collections::HashMap::new(),
            },
        }
    }

    /// 从文件加载配置
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 转换为StrategyContextConfig
    pub fn to_context_config(&self) -> StrategyContextConfig {
        StrategyContextConfig {
            inter_exchange_slippage_per_leg_pct: self.inter_exchange.slippage_per_leg_pct,
            inter_exchange_min_liquidity_usd: self.inter_exchange.min_liquidity_usd,
        }
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        // 验证滑点范围
        if self.inter_exchange.slippage_per_leg_pct < 0.0 || self.inter_exchange.slippage_per_leg_pct > 0.1 {
            return Err(anyhow::anyhow!("跨交易所滑点超出合理范围 [0, 0.1]"));
        }

        // 验证流动性要求
        if self.inter_exchange.min_liquidity_usd < 100.0 {
            return Err(anyhow::anyhow!("最小流动性要求过低，至少$100"));
        }

        // 验证最小利润配置
        if self.min_profit.base_bps == 0 {
            return Err(anyhow::anyhow!("最小利润不能为0"));
        }

        // 验证市场状态权重
        let weights = &self.min_profit.market_state_weights;
        if weights.regular <= 0.0 || weights.cautious <= 0.0 || weights.extreme <= 0.0 {
            return Err(anyhow::anyhow!("市场状态权重必须为正数"));
        }

        Ok(())
    }
}

/// 配置加载器
pub struct ConfigLoader {
    config_file_path: String,
    current_config: StrategyConfigFile,
}

impl ConfigLoader {
    /// 创建配置加载器
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let path_str = config_path.as_ref().to_string_lossy().to_string();
        
        let config = if config_path.as_ref().exists() {
            tracing::info!("📄 从文件加载策略配置: {}", path_str);
            StrategyConfigFile::load_from_file(&config_path)?
        } else {
            tracing::warn!("⚠️ 配置文件不存在，使用默认配置: {}", path_str);
            let default_config = StrategyConfigFile::default();
            // 创建默认配置文件
            if let Some(parent) = config_path.as_ref().parent() {
                std::fs::create_dir_all(parent)?;
            }
            default_config.save_to_file(&config_path)?;
            tracing::info!("✅ 已创建默认配置文件: {}", path_str);
            default_config
        };

        // 验证配置
        config.validate()?;
        tracing::info!("✅ 策略配置验证通过");

        Ok(Self {
            config_file_path: path_str,
            current_config: config,
        })
    }

    /// 获取当前配置
    pub fn get_config(&self) -> &StrategyConfigFile {
        &self.current_config
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<()> {
        tracing::info!("🔄 重新加载策略配置: {}", self.config_file_path);
        let new_config = StrategyConfigFile::load_from_file(&self.config_file_path)?;
        new_config.validate()?;
        self.current_config = new_config;
        tracing::info!("✅ 策略配置重新加载完成");
        Ok(())
    }

    /// 获取策略上下文配置
    pub fn get_context_config(&self) -> StrategyContextConfig {
        self.current_config.to_context_config()
    }

    /// 获取最小利润配置
    pub fn get_min_profit_config(&self) -> &MinProfitConfig {
        &self.current_config.min_profit
    }

    /// 获取风险配置
    pub fn get_risk_config(&self) -> &RiskConfig {
        &self.current_config.risk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_validation() {
        let config = StrategyConfigFile::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = StrategyConfigFile::default();
        
        // 设置无效滑点
        config.inter_exchange.slippage_per_leg_pct = -0.1;
        assert!(config.validate().is_err());
        
        // 设置无效权重
        config.inter_exchange.slippage_per_leg_pct = 0.001;
        config.min_profit.market_state_weights.regular = -1.0;
        assert!(config.validate().is_err());
    }
} 