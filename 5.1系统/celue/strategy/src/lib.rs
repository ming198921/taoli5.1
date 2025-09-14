//! Strategy module implementing arbitrage detection and execution
//! 
//! This module contains the core strategy implementations for:
//! - Inter-exchange arbitrage
//! - Triangular arbitrage
//! 
//! As specified in STRATEGY_MODULE_V5.1_NATS_DESIGN.md

pub mod traits;
pub mod context;
pub mod min_profit;
pub mod market_state;
pub mod plugins;
pub mod config_loader;
pub mod depth_analysis;
pub mod dynamic_fee_calculator;

pub use context::{StrategyContext, StrategyContextConfig, FeePrecisionRepo, FeePrecisionRepoImpl};
pub use market_state::{MarketState, AtomicMarketState};
pub use min_profit::MinProfitModel;
pub use traits::{ArbitrageStrategy, ExecutionResult};

/// Strategy configuration
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub min_profit_threshold: f64,
    pub max_slippage: f64,
    pub enabled_strategies: Vec<String>,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            min_profit_threshold: 0.002,
            max_slippage: 0.001,
            enabled_strategies: vec!["inter_exchange".to_string(), "triangular".to_string()],
        }
    }
}
