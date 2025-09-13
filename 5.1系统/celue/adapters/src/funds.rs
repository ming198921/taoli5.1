//! Funds management adapter for balance, limit and constraint tracking
//! Implements fund availability checks and balance management

use crate::{AdapterError, AdapterResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// Balance information for a specific asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBalance {
    /// Asset symbol (e.g., "BTC", "USDT")
    pub asset: String,
    /// Exchange name
    pub exchange: String,
    /// Free balance available for trading
    pub free: f64,
    /// Locked balance (in orders)
    pub locked: f64,
    /// Total balance (free + locked)
    pub total: f64,
    /// Last update timestamp (nanoseconds)
    pub updated_ns: u64,
}

/// Fund limits and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundLimits {
    /// Maximum position size per symbol (in USD)
    pub max_position_per_symbol: f64,
    /// Maximum total exposure (in USD)
    pub max_total_exposure: f64,
    /// Minimum order size per symbol (in USD)
    pub min_order_sizes: HashMap<String, f64>,
    /// Maximum leverage allowed
    pub max_leverage: f64,
    /// Reserve ratio (0.0 - 1.0)
    pub reserve_ratio: f64,
}

impl Default for FundLimits {
    fn default() -> Self {
        Self {
            max_position_per_symbol: 50_000.0,
            max_total_exposure: 500_000.0,
            min_order_sizes: HashMap::new(),
            max_leverage: 1.0,
            reserve_ratio: 0.1,
        }
    }
}

/// Fund allocation result
#[derive(Debug, Clone)]
pub struct FundAllocation {
    /// Whether funds are available
    pub available: bool,
    /// Maximum available amount
    pub max_amount: f64,
    /// Reason if not available
    pub reason: Option<String>,
}

/// Configuration for funds adapter
#[derive(Debug, Clone, Deserialize)]
pub struct FundsConfig {
    /// Update interval in seconds
    pub update_interval_secs: u64,
    /// Fund limits
    pub limits: FundLimits,
}

impl Default for FundsConfig {
    fn default() -> Self {
        Self {
            update_interval_secs: 60,
            limits: FundLimits::default(),
        }
    }
}

/// Funds management adapter
pub struct FundsAdapter {
    /// Asset balances by exchange and asset
    balances: Arc<RwLock<HashMap<(String, String), AssetBalance>>>,
    /// Fund limits
    limits: Arc<RwLock<FundLimits>>,
    /// Current positions by symbol (in USD)
    positions: Arc<RwLock<HashMap<String, f64>>>,
    /// Configuration
    config: FundsConfig,
}

impl FundsAdapter {
    /// Create a new funds adapter
    pub fn new(config: FundsConfig) -> Self {
        Self {
            balances: Arc::new(RwLock::new(HashMap::new())),
            limits: Arc::new(RwLock::new(config.limits.clone())),
            positions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Update balance for an asset
    pub fn update_balance(&self, balance: AssetBalance) {
        let key = (balance.exchange.clone(), balance.asset.clone());
        self.balances.write().insert(key, balance);
    }

    /// Get balance for an asset
    pub fn get_balance(&self, exchange: &str, asset: &str) -> Option<AssetBalance> {
        let key = (exchange.to_string(), asset.to_string());
        self.balances.read().get(&key).cloned()
    }

    /// Check if funds are available for allocation
    pub fn check_allocation(&self, symbol: &str, _exchange: &str, amount_usd: f64) -> FundAllocation {
        let limits = self.limits.read();
        
        // Check minimum order size
        if let Some(&min_size) = limits.min_order_sizes.get(symbol) {
            if amount_usd < min_size {
                return FundAllocation {
                    available: false,
                    max_amount: 0.0,
                    reason: Some(format!("Below minimum order size: ${:.2}", min_size)),
                };
            }
        }
        
        // Check position limits
        let current_position = self.positions.read().get(symbol).copied().unwrap_or(0.0);
        if current_position + amount_usd > limits.max_position_per_symbol {
            return FundAllocation {
                available: false,
                max_amount: limits.max_position_per_symbol - current_position,
                reason: Some(format!("Exceeds position limit for {}", symbol)),
            };
        }
        
        // Check total exposure
        let total_exposure: f64 = self.positions.read().values().sum();
        if total_exposure + amount_usd > limits.max_total_exposure {
            return FundAllocation {
                available: false,
                max_amount: limits.max_total_exposure - total_exposure,
                reason: Some("Exceeds total exposure limit".to_string()),
            };
        }
        
        // Funds available
        FundAllocation {
            available: true,
            max_amount: amount_usd,
            reason: None,
        }
    }

    /// Allocate funds for a trade
    pub fn allocate_funds(&self, symbol: &str, exchange: &str, amount_usd: f64) -> AdapterResult<()> {
        // Check allocation
        let allocation = self.check_allocation(symbol, exchange, amount_usd);
        if !allocation.available {
            return Err(AdapterError::Validation { 
                message: allocation.reason.unwrap_or_else(|| "Funds not available".to_string())
            });
        }

        // Update position
        let mut positions = self.positions.write();
        *positions.entry(symbol.to_string()).or_insert(0.0) += amount_usd;
        
        Ok(())
    }

    /// Release funds after trade completion
    pub fn release_funds(&self, symbol: &str, amount_usd: f64) {
        let mut positions = self.positions.write();
        if let Some(position) = positions.get_mut(symbol) {
            *position = (*position - amount_usd).max(0.0);
        }
    }

    /// Update fund limits
    pub fn update_limits(&self, limits: FundLimits) {
        *self.limits.write() = limits;
    }

    /// Get current positions
    pub fn get_positions(&self) -> HashMap<String, f64> {
        self.positions.read().clone()
    }
}

#[async_trait]
impl crate::Adapter for FundsAdapter {
    type Config = FundsConfig;
    type Error = AdapterError;

    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = config;
        *self.limits.write() = self.config.limits.clone();
        Ok(())
    }

    async fn start(&mut self) -> Result<(), Self::Error> {
        // Funds adapter is always ready
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn name(&self) -> &'static str {
        "funds"
    }
} 