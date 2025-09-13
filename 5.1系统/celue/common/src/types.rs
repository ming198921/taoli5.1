use serde::{Deserialize, Serialize};

/// Execution result for trade operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub details: String,
    pub executed_quantity: Option<crate::precision::FixedQuantity>,
    pub average_price: Option<crate::precision::FixedPrice>,
    pub order_ids: Vec<String>,
    pub opportunity_id: String,
    pub trace_id: Option<String>,
}

impl ExecutionResult {
    pub fn accepted(opportunity_id: String, order_ids: Vec<String>, trace_id: Option<String>) -> Self { Self { success: true, details: "accepted".into(), executed_quantity: None, average_price: None, order_ids, opportunity_id, trace_id } }
    pub fn rejected(opportunity_id: String, reason: String, trace_id: Option<String>) -> Self { Self { success: false, details: reason, executed_quantity: None, average_price: None, order_ids: vec![], opportunity_id, trace_id } }
    pub fn partial(opportunity_id: String, order_ids: Vec<String>, details: String, trace_id: Option<String>) -> Self { Self { success: false, details, executed_quantity: None, average_price: None, order_ids, opportunity_id, trace_id } }
}

/// Exchange identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Exchange(String);

impl Exchange {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Trading symbol (e.g., "BTCUSDT")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self(symbol.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(String);

impl TraceId {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
}

/// Performance metrics for strategy operations
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub opportunities_detected: u64,
    pub opportunities_executed: u64,
    pub total_profit: f64,
    pub average_execution_time_ns: u64,
}

/// Order side - Buy or Sell
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Strategy execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Paper trading - no real trades
    Paper,
    /// Live trading with real money
    Live,
    /// Simulation with historical data
    Simulation,
}

/// Strategy type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyKind {
    InterExchange,
    Triangular,
    Statistical,
    PairTrading,
}

/// Risk limits for strategy execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_position_size: crate::precision::FixedQuantity,
    pub max_daily_loss: crate::precision::FixedPrice,
    pub max_drawdown_pct: f64,
}

impl Default for RiskLimits {
    fn default() -> Self {
        Self {
            max_position_size: crate::precision::FixedQuantity::from_f64(1.0, 8),
            max_daily_loss: crate::precision::FixedPrice::from_f64(1000.0, 2),
            max_drawdown_pct: 0.05, // 5%
        }
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
