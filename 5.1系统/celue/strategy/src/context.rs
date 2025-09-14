//! Strategy execution context providing access to shared resources
//! Provides runtime information to strategies for decision making

use std::collections::HashMap;
use std::sync::Arc;

use common::types::Exchange;
use common::precision::FixedPrice;
use crate::market_state::MarketState;
use crate::config_loader::ConfigLoader;

// 策略指标类型定义
pub type StrategyMetrics = Arc<adapters::metrics::AdapterMetrics>;

/// 策略上下文配置
#[derive(Debug, Clone)]
pub struct StrategyContextConfig {
    pub inter_exchange_slippage_per_leg_pct: f64,
    pub inter_exchange_min_liquidity_usd: f64,
}

impl Default for StrategyContextConfig {
    fn default() -> Self {
        Self {
            // 从环境变量加载，避免硬编码
            inter_exchange_slippage_per_leg_pct: std::env::var("CELUE_CONTEXT_INTER_EXCHANGE_SLIPPAGE")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(0.002), // 保守默认值：0.2%
            inter_exchange_min_liquidity_usd: std::env::var("CELUE_CONTEXT_MIN_LIQUIDITY_USD")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(50000.0), // 保守默认值：$50K
        }
    }
}

/// 策略执行上下文 - 核心配置驱动架构
/// 专为v3.0三角套利算法和跨交易所套利优化
// Debug removed due to complex trait objects
pub struct StrategyContext {
    pub fee_precision_repo: Arc<dyn FeePrecisionRepo>,
    min_profit_cache: Arc<parking_lot::RwLock<HashMap<String, FixedPrice>>>,
    exchange_weights: Arc<parking_lot::RwLock<HashMap<String, f64>>>,
    current_market_state: Arc<parking_lot::RwLock<MarketState>>,
    strategy_metrics: StrategyMetrics,
    pub inter_exchange_slippage_per_leg_pct: f64,
    // v3.0算法不使用预配置的三角套利参数，全部动态计算
    pub inter_exchange_min_liquidity_usd: f64,
    // 配置加载器（可选）- 用于动态配置加载
    config_loader: Option<Arc<parking_lot::RwLock<ConfigLoader>>>,
}

impl StrategyContext {
    pub fn new(
        fee_repo: Arc<dyn FeePrecisionRepo>,
        strategy_metrics: StrategyMetrics,
    ) -> Self {
        Self::with_config(fee_repo, strategy_metrics, Default::default())
    }

    /// 使用指定配置创建策略上下文
    pub fn with_config(
        fee_repo: Arc<dyn FeePrecisionRepo>,
        strategy_metrics: StrategyMetrics,
        config: StrategyContextConfig,
    ) -> Self {
        Self {
            fee_precision_repo: fee_repo,
            min_profit_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            exchange_weights: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            current_market_state: Arc::new(parking_lot::RwLock::new(MarketState::Regular)),
            strategy_metrics,
            inter_exchange_slippage_per_leg_pct: config.inter_exchange_slippage_per_leg_pct,
            inter_exchange_min_liquidity_usd: config.inter_exchange_min_liquidity_usd,
            config_loader: None, // 默认不启用配置加载器
        }
    }

    pub fn get_taker_fee(&self, exchange: &Exchange) -> Option<FixedPrice> {
        self.fee_precision_repo.get_taker_fee(exchange)
    }

    pub fn get_maker_fee(&self, exchange: &Exchange) -> Option<FixedPrice> {
        self.fee_precision_repo.get_maker_fee(exchange)
    }

    pub fn current_min_profit_pct(&self) -> FixedPrice {
        // 动态获取策略特定的最小利润率，如果未设置则使用配置文件中的值
        self.get_min_profit_for_strategy("default")
            .unwrap_or_else(|| {
                // 从config_loader获取动态配置，而非硬编码
                if let Some(config_loader) = &self.config_loader {
                    if let Some(guard) = config_loader.try_read() {
                        let config = guard.get_config();
                        let base_bps = config.min_profit.base_bps as u32;
                        let current_state = self.get_market_state();
                        let weight = match current_state {
                            MarketState::Regular => config.min_profit.market_state_weights.regular,
                            MarketState::Cautious => config.min_profit.market_state_weights.cautious,
                            MarketState::Extreme => config.min_profit.market_state_weights.extreme,
                        };
                        let dynamic_bps = (base_bps as f64 * weight) as u32;
                        return FixedPrice::from_raw((dynamic_bps * 100) as i64, 6);
                    }
                }
                
                // 最后的安全后备值：从环境变量获取
                let base_bps = std::env::var("CELUE_FALLBACK_MIN_PROFIT_BPS")
                    .ok().and_then(|s| s.parse().ok())
                    .unwrap_or(100u32); // 最保守：1.0%
                    
                let current_state = self.get_market_state();
                let weight = match current_state {
                    MarketState::Regular => std::env::var("CELUE_FALLBACK_WEIGHT_REGULAR")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(1.0),
                    MarketState::Cautious => std::env::var("CELUE_FALLBACK_WEIGHT_CAUTIOUS")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(2.0),
                    MarketState::Extreme => std::env::var("CELUE_FALLBACK_WEIGHT_EXTREME")
                        .ok().and_then(|s| s.parse().ok())
                        .unwrap_or(3.0),
                };
                let dynamic_bps = (base_bps as f64 * weight) as u32;
                FixedPrice::from_raw((dynamic_bps * 100) as i64, 6)
            })
    }

    pub fn set_min_profit_for_strategy(&self, strategy: &str, min_profit: FixedPrice) {
        self.min_profit_cache.write().insert(strategy.to_string(), min_profit);
    }

    pub fn get_min_profit_for_strategy(&self, strategy: &str) -> Option<FixedPrice> {
        self.min_profit_cache.read().get(strategy).copied()
    }

    pub fn update_market_state(&self, new_state: MarketState) {
        *self.current_market_state.write() = new_state;
    }

    pub fn get_market_state(&self) -> MarketState {
        *self.current_market_state.read()
    }

    pub fn get_exchange_weight(&self, exchange: &str) -> f64 {
        self.exchange_weights.read().get(exchange).copied().unwrap_or(1.0)
    }

    pub fn set_exchange_weight(&self, exchange: &str, weight: f64) {
        self.exchange_weights.write().insert(exchange.to_string(), weight);
    }

    pub fn metrics(&self) -> &StrategyMetrics {
        &self.strategy_metrics
    }
}

/// 手续费和精度仓库接口 - 完全可配置化
pub trait FeePrecisionRepo: Send + Sync {
    fn get_taker_fee(&self, exchange: &Exchange) -> Option<FixedPrice>;
    fn get_maker_fee(&self, exchange: &Exchange) -> Option<FixedPrice>;
    fn get_price_precision(&self, exchange: &Exchange, symbol: &str) -> Option<u8>;
    fn get_quantity_precision(&self, exchange: &Exchange, symbol: &str) -> Option<u8>;
    fn get_step_size_for_symbol(&self, symbol: &str) -> Option<f64>;
    fn get_tick_size_for_symbol(&self, symbol: &str) -> Option<f64>;
    fn get_fee_rate_bps_for_exchange(&self, exchange: &str) -> Option<f64>;
}

/// 手续费和精度仓库实现 - 基于配置文件
pub struct FeePrecisionRepoImpl {
    taker_fees: HashMap<Exchange, FixedPrice>,
    maker_fees: HashMap<Exchange, FixedPrice>,
    price_precisions: HashMap<(Exchange, String), u8>,
    quantity_precisions: HashMap<(Exchange, String), u8>,
    step_sizes: HashMap<String, f64>,
    tick_sizes: HashMap<String, f64>,
    fee_rates_bps: HashMap<String, f64>,
}

impl FeePrecisionRepoImpl {
    /// 从配置创建仓库实例
    pub fn from_config(config: &FeePrecisionConfig) -> Self {
        let mut repo = Self {
            taker_fees: HashMap::new(),
            maker_fees: HashMap::new(),
            price_precisions: HashMap::new(),
            quantity_precisions: HashMap::new(),
            step_sizes: HashMap::new(),
            tick_sizes: HashMap::new(),
            fee_rates_bps: HashMap::new(),
        };

        // 从配置加载数据
        for (exchange_name, exchange_config) in &config.exchanges {
            let exchange = Exchange::new(exchange_name.clone());
            repo.taker_fees.insert(exchange.clone(), FixedPrice::from_f64(exchange_config.taker_fee, 6));
            repo.maker_fees.insert(exchange.clone(), FixedPrice::from_f64(exchange_config.maker_fee, 6));
            repo.fee_rates_bps.insert(exchange_name.clone(), exchange_config.fee_rate_bps);
        }

        repo
    }
}

impl Default for FeePrecisionRepoImpl {
    fn default() -> Self {
        let config = FeePrecisionConfig::default();
        Self::from_config(&config)
    }
}

impl FeePrecisionRepo for FeePrecisionRepoImpl {
    fn get_taker_fee(&self, exchange: &Exchange) -> Option<FixedPrice> {
        self.taker_fees.get(exchange).copied()
    }

    fn get_maker_fee(&self, exchange: &Exchange) -> Option<FixedPrice> {
        self.maker_fees.get(exchange).copied()
    }

    fn get_price_precision(&self, exchange: &Exchange, symbol: &str) -> Option<u8> {
        self.price_precisions.get(&(exchange.clone(), symbol.to_string())).copied()
    }

    fn get_quantity_precision(&self, exchange: &Exchange, symbol: &str) -> Option<u8> {
        self.quantity_precisions.get(&(exchange.clone(), symbol.to_string())).copied()
    }

    fn get_step_size_for_symbol(&self, symbol: &str) -> Option<f64> {
        self.step_sizes.get(symbol).copied()
    }

    fn get_tick_size_for_symbol(&self, symbol: &str) -> Option<f64> {
        self.tick_sizes.get(symbol).copied()
    }

    fn get_fee_rate_bps_for_exchange(&self, exchange: &str) -> Option<f64> {
        self.fee_rates_bps.get(exchange).copied()
    }
}

/// 手续费精度配置 - 完全外部化配置
#[derive(Debug, Clone)]
pub struct FeePrecisionConfig {
    pub exchanges: HashMap<String, ExchangeConfig>,
}

#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    pub taker_fee: f64,
    pub maker_fee: f64,
    pub fee_rate_bps: f64,
}

impl Default for FeePrecisionConfig {
    fn default() -> Self {
        let mut exchanges = HashMap::new();
        
        exchanges.insert("binance".to_string(), ExchangeConfig {
            taker_fee: 0.001, // 0.1%
            maker_fee: 0.001,
            fee_rate_bps: 10.0,
        });
        
        exchanges.insert("okx".to_string(), ExchangeConfig {
            taker_fee: 0.001,
            maker_fee: 0.001,
            fee_rate_bps: 10.0,
        });
        
        exchanges.insert("bybit".to_string(), ExchangeConfig {
            taker_fee: 0.001,
            maker_fee: 0.001,
            fee_rate_bps: 10.0,
        });

        Self { exchanges }
    }
}

    pub taker_fee: f64,
    pub maker_fee: f64,
    pub fee_rate_bps: f64,
}

impl Default for FeePrecisionConfig {
    fn default() -> Self {
        let mut exchanges = HashMap::new();
        
        exchanges.insert("binance".to_string(), ExchangeConfig {
            taker_fee: 0.001, // 0.1%
            maker_fee: 0.001,
            fee_rate_bps: 10.0,
        });
        
        exchanges.insert("okx".to_string(), ExchangeConfig {
            taker_fee: 0.001,
            maker_fee: 0.001,
            fee_rate_bps: 10.0,
        });
        
        exchanges.insert("bybit".to_string(), ExchangeConfig {
            taker_fee: 0.001,
            maker_fee: 0.001,
            fee_rate_bps: 10.0,
        });

        Self { exchanges }
    }
}
