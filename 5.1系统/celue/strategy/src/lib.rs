//! Strategy module implementing arbitrage detection and execution
//! 
//! This module contains the core strategy implementations for:
//! - Inter-exchange arbitrage
//! - Triangular arbitrage
//! 
//! As specified in STRATEGY_MODULE_V5.1_NATS_DESIGN.md

pub mod context;
pub mod traits;
pub mod dynamic_fee_calculator;
pub mod config_loader;
pub mod market_state;
pub mod plugins;
pub mod production_api;
pub mod types;
pub mod risk_assessment;

pub use context::FeePrecisionRepoImpl;
pub use traits::{ArbitrageStrategy, ExecutionResult, StrategyError, StrategyKind};
pub use dynamic_fee_calculator::*;
pub use types::*;

pub mod depth_analysis;
