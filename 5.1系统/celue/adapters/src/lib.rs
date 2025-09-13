//! High-frequency arbitrage adapters for external system integration
//! 
//! This crate provides adapters for integrating with external systems:
//! - NATS messaging for loose coupling
//! - Market data adapters for real-time feeds
//! - Risk management adapters
//! - Execution adapters for order placement
//! - Configuration adapters for dynamic updates
//! - Health monitoring for API/module status
//! - Funds management for balance and limits

pub mod nats;
pub mod market_data;
pub mod error;
pub mod risk;
pub mod funds;
pub mod metrics;
pub mod execution;

// Re-export key types
pub use error::{AdapterError, AdapterResult};
pub use metrics::{MetricsRegistry, AdapterMetrics};

/// Core adapter trait that all adapters must implement
#[async_trait::async_trait]
pub trait Adapter: Send + Sync {
    type Config: Send + Sync;
    type Error: std::error::Error + Send + Sync + 'static;

    /// Initialize the adapter with configuration
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;

    /// Start the adapter
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Stop the adapter
    async fn stop(&mut self) -> Result<(), Self::Error>;

    /// Perform a health check
    async fn health_check(&self) -> Result<(), Self::Error>;

    /// Get the adapter name
    fn name(&self) -> &'static str;
}
