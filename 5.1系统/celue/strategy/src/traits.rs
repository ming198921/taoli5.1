//! Defines the core traits for arbitrage strategies.

use crate::context::StrategyContext;
use async_trait::async_trait;
use common::{arbitrage::ArbitrageOpportunity, market_data::NormalizedSnapshot};
use thiserror::Error;

/// The kind of an arbitrage strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyKind {
    InterExchange,
    Triangular,
}

/// Errors that can occur during strategy execution.
#[derive(Debug, Error)]
pub enum StrategyError {
    #[error("Execution failed on exchange: {0}")]
    ExecutionFailed(String),
    #[error("Risk check failed: {0}")]
    RiskCheckFailed(String),
    #[error("Insufficient funds for trade")]
    InsufficientFunds,
    #[error("Opportunity is no longer valid (TTL expired)")]
    OpportunityExpired,
}

/// The result of a strategy execution attempt.
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub accepted: bool,
    pub reason: Option<String>,
    pub order_ids: Vec<String>,
}

/// The core trait that all arbitrage strategies must implement.
#[async_trait]
pub trait ArbitrageStrategy: Send + Sync {
    /// A unique, static name for the strategy.
    fn name(&self) -> &'static str;

    /// The kind of the strategy.
    fn kind(&self) -> StrategyKind;

    /// Detects an arbitrage opportunity from a normalized market snapshot.
    ///
    /// This is the **hot path**. It must be synchronous, non-blocking, and avoid
    /// allocations to meet performance targets (<= 100µs).
    fn detect(
        &self,
        ctx: &StrategyContext,
        input: &NormalizedSnapshot,
    ) -> Option<ArbitrageOpportunity>;

    /// Executes a detected arbitrage opportunity.
    ///
    /// This path is asynchronous as it may involve I/O (e.g., sending orders).
    async fn execute(
        &self,
        ctx: &StrategyContext,
        opp: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError>;
}
