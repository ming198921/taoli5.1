//! Strategy execution context providing access to shared resources
//! Provides runtime information to strategies for decision making

use std::collections::HashMap;
use std::sync::Arc;

use common::types::Exchange;
use common::precision::FixedPrice;
use crate::market_state::MarketState;
use crate::StrategyConfig;
use crate::MarketStateEvaluator;
use crate::MinProfitAdjuster; 
use crate::RiskManager;
use crate::MarketDataSnapshot;
use crate::production_api::ProductionApiManager;

// 策略指标结构体
#[derive(Debug, Default)]
pub struct StrategyMetrics {
    pub total_opportunities: std::sync::atomic::AtomicUsize,
    pub successful_trades: std::sync::atomic::AtomicUsize,
    pub failed_trades: std::sync::atomic::AtomicUsize,
    pub total_profit: parking_lot::RwLock<f64>,
}

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

/// 策略上下文 - 为策略提供市场数据、配置和工具
pub struct StrategyContext {
    /// 手续费和精度仓库
    pub fee_precision_repo: Arc<dyn FeePrecisionRepo>,
    /// 策略配置
    pub strategy_config: StrategyConfig,
    /// 市场状态评估器
    pub market_state_evaluator: Arc<dyn MarketStateEvaluator>,
    /// min_profit动态调整器
    pub min_profit_adjuster: Arc<dyn MinProfitAdjuster>,
    /// 风险管理器
    pub risk_manager: Arc<dyn RiskManager>,
    /// 📈 实时市场数据缓存
    pub market_data_cache: Arc<parking_lot::RwLock<HashMap<String, MarketDataSnapshot>>>,
    /// 🚀 生产级API管理器
    pub production_api_manager: Option<Arc<ProductionApiManager>>,
    /// 配置加载器
    pub config_loader: Option<Arc<crate::config_loader::ConfigLoader>>,
    /// min_profit缓存
    pub min_profit_cache: Arc<parking_lot::RwLock<HashMap<String, FixedPrice>>>,
    /// 当前市场状态
    pub current_market_state: Arc<parking_lot::RwLock<MarketState>>,
    /// 交易所权重
    pub exchange_weights: Arc<parking_lot::RwLock<HashMap<String, f64>>>,
    /// 策略指标
    pub strategy_metrics: Arc<StrategyMetrics>,
    /// 跨交易所每腿滑点百分比
    pub inter_exchange_slippage_per_leg_pct: f64,
}

impl StrategyContext {
    /// 创建新的策略上下文
    pub fn new(
        fee_precision_repo: Arc<dyn FeePrecisionRepo>,
        strategy_config: StrategyConfig,
        market_state_evaluator: Arc<dyn MarketStateEvaluator>,
        min_profit_adjuster: Arc<dyn MinProfitAdjuster>,
        risk_manager: Arc<dyn RiskManager>,
        production_api_manager: Option<Arc<ProductionApiManager>>,
    ) -> Self {
        Self {
            fee_precision_repo,
            strategy_config,
            market_state_evaluator,
            min_profit_adjuster,
            risk_manager,
            market_data_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            production_api_manager,
            config_loader: None,
            min_profit_cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            current_market_state: Arc::new(parking_lot::RwLock::new(MarketState::Regular)),
            exchange_weights: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            strategy_metrics: Arc::new(StrategyMetrics::default()),
            inter_exchange_slippage_per_leg_pct: 0.001, // 默认0.1%滑点
        }
    }

    /// 获取生产API管理器
    pub fn get_production_api_manager(&self) -> Option<&Arc<ProductionApiManager>> {
        self.production_api_manager.as_ref()
    }

    /// 设置生产API管理器
    pub fn set_production_api_manager(&mut self, manager: Arc<ProductionApiManager>) {
        self.production_api_manager = Some(manager);
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
                    let config = config_loader.get_config();
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
