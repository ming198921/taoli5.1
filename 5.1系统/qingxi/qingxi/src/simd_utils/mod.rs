#![allow(dead_code)]
//! # SIMD Utils Module
use crate::errors::MarketDataError;

pub fn parse_json(exchange: &str, data: &mut [u8]) -> Result<serde_json::Value, MarketDataError> {
    serde_json::from_slice(data).map_err(|e| MarketDataError::Parse {
        exchange: exchange.to_string(),
        details: format!("JSON parsing error: {e}"),
    })
}

pub struct JsonNumber;
impl JsonNumber {
    pub fn new(_bytes: &[u8]) -> Self {
        Self
    }

    pub fn as_f64(&self) -> f64 {
        0.0 // 简化实现
    }
}

pub struct JsonString;
impl JsonString {
    pub fn new(_bytes: &[u8]) -> Self {
        Self
    }

    pub fn as_str(&self) -> &str {
        "" // 简化实现
    }
}

pub struct JsonFieldLocator;
impl JsonFieldLocator {
    pub fn new(_json: &[u8]) -> Self {
        Self
    }

    pub fn locate_field(&self, _field_name: &str) -> Option<&[u8]> {
        None // 简化实现
    }
}
