//! Error types for adapters

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("Adapter not initialized")]
    NotInitialized,
    
    #[error("Adapter already running")]
    AlreadyRunning,
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Generic error: {message}")]
    Generic { message: String },
    
    #[error("Validation error: {message}")]
    Validation { message: String },
    
    #[error("Timeout error: operation timed out after {duration_ms}ms")]
    Timeout { duration_ms: u64 },
    
    #[error("NATS publish error: {0}")]
    NatsPublish(String),
    
    #[error("NATS request error: {0}")]
    NatsRequest(String),
    
    #[error("NATS subscribe error: {0}")]
    NatsSubscribe(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type AdapterResult<T> = Result<T, AdapterError>;
