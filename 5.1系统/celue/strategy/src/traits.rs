//! Strategy traits - now using unified definitions from common_types

// Re-export the unified strategy definitions from common_types
pub use common_types::{
    ArbitrageStrategy, StrategyContext, StrategyKind, StrategyError, 
    ExecutionResult, NormalizedSnapshot, ExchangeSnapshot, ResourceRequirements
};
