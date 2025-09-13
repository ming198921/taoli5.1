//! Configuration adapter for dynamic updates

use crate::{Adapter, AdapterError};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigAdapterConfig {
    pub config_source: String,
    pub refresh_interval: std::time::Duration,
}

impl Default for ConfigAdapterConfig {
    fn default() -> Self {
        Self {
            config_source: "file://config.toml".to_string(),
            refresh_interval: std::time::Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicConfig {
    pub version: u64,
    pub data: serde_json::Value,
    // Optional structured fields for fast-path lookup
    pub fee_bps_per_exchange: Option<std::collections::HashMap<String, f64>>, // e.g., {"binance": 10.0}
    pub price_scale_per_symbol: Option<std::collections::HashMap<String, u8>>, // e.g., {"BTCUSDT": 2}
    pub qty_scale_per_symbol: Option<std::collections::HashMap<String, u8>>,   // e.g., {"BTCUSDT": 8}
    pub tick_size_per_symbol: Option<std::collections::HashMap<String, f64>>,  // e.g., {"BTCUSDT": 0.01}
    pub step_size_per_symbol: Option<std::collections::HashMap<String, f64>>,  // e.g., {"BTCUSDT": 0.0001}
}

pub struct ConfigAdapter {
    config: Option<ConfigAdapterConfig>,
    running: Arc<parking_lot::Mutex<bool>>,
}

impl ConfigAdapter {
    pub fn new() -> Self {
        Self {
            config: None,
            running: Arc::new(parking_lot::Mutex::new(false)),
        }
    }
}

#[async_trait::async_trait]
impl Adapter for ConfigAdapter {
    type Config = ConfigAdapterConfig;
    type Error = AdapterError;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = Some(config);
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        *self.running.lock() = true;
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Self::Error> {
        *self.running.lock() = false;
        Ok(())
    }
    
    async fn health_check(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "config_adapter"
    }
}

impl Default for ConfigAdapter {
    fn default() -> Self {
        Self::new()
    }
}
