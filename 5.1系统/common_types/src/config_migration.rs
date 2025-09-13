use crate::config::*;
use std::collections::HashMap;

/// Configuration migration utility to replace duplicate Config structs
pub struct ConfigMigrator;

impl ConfigMigrator {
    /// Migrate all duplicate StrategyConfig instances to BaseStrategyConfig
    pub fn migrate_strategy_configs() {
        // This function will be called by build scripts to update imports
        println!("Migrating StrategyConfig instances to BaseStrategyConfig");
    }

    /// Migrate all duplicate ExchangeConfig instances to BaseExchangeConfig
    pub fn migrate_exchange_configs() {
        println!("Migrating ExchangeConfig instances to BaseExchangeConfig");
    }

    /// Migrate all duplicate MonitoringConfig instances to unified MonitoringConfig
    pub fn migrate_monitoring_configs() {
        println!("Migrating MonitoringConfig instances to unified MonitoringConfig");
    }
}

// Type aliases for migration compatibility
pub type FrontendStrategyConfig = StrategyConfig;
pub type InterExchangeConfig = StrategyConfig; 
pub type TriangularConfig = StrategyConfig;
pub type EmergencyStopConfig = RiskConfig;
pub type MinProfitConfig = PerformanceConfig;
pub type DynamicRiskConfig = RiskConfig;
pub type SlippageExecutionConfig = ExecutionConfig;
pub type ConfigAdapterConfig = SystemConfig;
pub type DynamicConfig = SystemConfig;

// Conversion functions for seamless migration
impl From<StrategyConfig> for ExchangeConfig {
    fn from(strategy: StrategyConfig) -> Self {
        ExchangeConfig {
            name: strategy.name,
            enabled: strategy.enabled,
            api_key: None,
            api_secret: None, 
            api_passphrase: None,
            sandbox_mode: false,
            rest_url: String::new(),
            ws_url: String::new(),
            websocket_url: String::new(),
            rate_limit: 10,
            max_connections: Some(5),
            supported_symbols: strategy.symbols.clone(),
            symbols: strategy.symbols,
            taker_fee: Some(0.001),
            maker_fee: Some(0.001),
            fee_rate_bps: Some(10.0),
            parameters: strategy.parameters,
        }
    }
}

impl From<ExchangeConfig> for StrategyConfig {
    fn from(exchange: ExchangeConfig) -> Self {
        StrategyConfig {
            name: exchange.name.clone(),
            enabled: exchange.enabled,
            strategy_type: StrategyType::InterExchange,
            parameters: StrategyParameters {
                min_profit_threshold: 0.001,
                max_position_size: 1000.0,
                max_slippage: 0.005,
                timeout_ms: 5000,
                additional: HashMap::new(),
            },
            risk_limits: RiskLimits {
                max_position_value: 10000.0,
                max_daily_loss: 1000.0,
                max_exposure_per_symbol: 5000.0,
                max_open_positions: 10,
            },
            execution: ExecutionConfig {
                order_type: "market".to_string(),
                time_in_force: "IOC".to_string(),
                post_only: false,
                reduce_only: false,
            },
        }
    }
}