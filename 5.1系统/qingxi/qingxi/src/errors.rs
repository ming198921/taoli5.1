#![allow(dead_code)]
//! # 错误类型定义 - 系统唯一权威来源
//!
//! 本模块定义了整个qingxi系统中使用的所有错误类型。
//! 这是系统错误处理的权威定义，所有其他模块必须导入并使用这些类型。

use serde_json::Error as SerdeJsonError;
use std::num::ParseFloatError;
use thiserror::Error;
use tonic::Status;

/// 核心市场数据错误 - 系统权威错误类型
#[derive(Error, Debug, Clone)]
pub enum MarketDataError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Connection failed for {exchange}: {details}")]
    Connection { exchange: String, details: String },

    #[error("WebSocket communication error for {exchange}: {details}")]
    Communication { exchange: String, details: String },

    #[error("Failed to parse message from {exchange}: {details}")]
    Parse { exchange: String, details: String },

    #[error("Adapter not found for: {0}")]
    AdapterNotFound(String),

    #[error("Operation not supported: {0}")]
    UnsupportedOperation(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Subscription error for {exchange}: {details}")]
    Subscription { exchange: String, details: String },

    #[error("Rate limit exceeded for {exchange}")]
    RateLimit { exchange: String },

    #[error("Authentication failed for {exchange}: {details}")]
    Authentication { exchange: String, details: String },

    #[error("Data quality error: {0}")]
    DataQuality(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Data cleaning failed: {0}")]
    CleaningFailed(String),

    #[error("Data validation failed: {0}")]
    ValidationFailed(String),
}

/// 自动转换实现 - From traits
impl From<SerdeJsonError> for MarketDataError {
    fn from(e: SerdeJsonError) -> Self {
        MarketDataError::Parse {
            exchange: "unknown".to_string(),
            details: e.to_string(),
        }
    }
}

impl From<ParseFloatError> for MarketDataError {
    fn from(e: ParseFloatError) -> Self {
        MarketDataError::Parse {
            exchange: "unknown".to_string(),
            details: e.to_string(),
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for MarketDataError {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        MarketDataError::Communication {
            exchange: "unknown".to_string(),
            details: e.to_string(),
        }
    }
}

impl From<anyhow::Error> for MarketDataError {
    fn from(e: anyhow::Error) -> Self {
        MarketDataError::Connection {
            exchange: "unknown".to_string(),
            details: e.to_string(),
        }
    }
}

/// API层错误类型
#[derive(Error, Debug, Clone)]
pub enum MarketDataApiError {
    #[error("Data not available: {0}")]
    DataUnavailable(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// gRPC错误转换
impl From<MarketDataApiError> for Status {
    fn from(error: MarketDataApiError) -> Self {
        match error {
            MarketDataApiError::DataUnavailable(msg) => Status::not_found(msg),
            MarketDataApiError::InvalidRequest(msg) => Status::invalid_argument(msg),
            MarketDataApiError::ServiceUnavailable(msg) => Status::unavailable(msg),
            MarketDataApiError::InternalError(msg) => Status::internal(msg),
        }
    }
}

impl From<MarketDataError> for MarketDataApiError {
    fn from(error: MarketDataError) -> Self {
        match error {
            MarketDataError::Configuration(msg) => MarketDataApiError::InvalidRequest(msg),
            MarketDataError::AdapterNotFound(msg) => MarketDataApiError::DataUnavailable(msg),
            MarketDataError::UnsupportedOperation(msg) => MarketDataApiError::InvalidRequest(msg),
            _ => MarketDataApiError::InternalError(error.to_string()),
        }
    }
}
