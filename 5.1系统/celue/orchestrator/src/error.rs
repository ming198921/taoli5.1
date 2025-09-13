use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("NATS connection error: {0}")]
    NatsConnection(#[from] async_nats::Error),
    
    #[error("Strategy error: {0}")]
    Strategy(String),
    
    #[error("Market data error: {0}")]
    MarketData(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("Risk management error: {0}")]
    RiskManagement(String),
    
    #[error("Metrics error: {0}")]
    Metrics(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Generic error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, Error>; 