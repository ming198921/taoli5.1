//! Error types for the strategy system

use thiserror::Error;

#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Market data unavailable for symbol {symbol} on exchange {exchange}")]
    MarketDataUnavailable { symbol: String, exchange: String },
    
    #[error("Insufficient liquidity: required {required}, available {available}")]
    InsufficientLiquidity { required: f64, available: f64 },
    
    #[error("Profit threshold not met: profit {profit}% < threshold {threshold}%")]
    ProfitThresholdNotMet { profit: f64, threshold: f64 },
    
    #[error("Risk check failed: {reason}")]
    RiskCheckFailed { reason: String },
    
    #[error("Execution timeout after {timeout_ms}ms")]
    ExecutionTimeout { timeout_ms: u64 },
    
    #[error("Invalid precision for symbol {symbol}: {precision}")]
    InvalidPrecision { symbol: String, precision: String },
    
    #[error("Exchange API error: {exchange} - {message}")]
    ExchangeApiError { exchange: String, message: String },
    
    #[error("Data consistency error: {message}")]
    DataConsistency { message: String },
    
    #[error("Internal system error: {0}")]
    Internal(String),
}

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Failed to start orchestrator: {0}")]
    StartupFailed(String),
    
    #[error("NATS connection error: {0}")]
    NatsConnectionError(String),
    
    #[error("Configuration loading error: {0}")]
    ConfigurationError(String),
    
    #[error("Strategy initialization failed: {strategy} - {error}")]
    StrategyInitError { strategy: String, error: String },
    
    #[error("Graceful shutdown failed: {0}")]
    ShutdownError(String),
}

pub type StrategyResult<T> = Result<T, StrategyError>;
pub type OrchestratorResult<T> = Result<T, OrchestratorError>;
