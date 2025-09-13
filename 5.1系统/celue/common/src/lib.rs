pub mod arbitrage;
pub mod market_data;
pub mod precision;
pub mod types;

pub use arbitrage::{PrecisionArbitrageOpportunity, ArbitrageLeg, Side};
// 同时导出统一的ArbitrageOpportunity供一般用途使用
pub use common_types::ArbitrageOpportunity;
pub use market_data::{NormalizedSnapshot, OrderBook};
pub use precision::{FixedPrice, FixedQuantity};
pub use types::{Exchange, Symbol, ExecutionResult, TraceId, IdempotencyKey};
