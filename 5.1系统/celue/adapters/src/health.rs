//! Health monitoring adapter for tracking API and module health
//! Implements health checks and status aggregation across components

use crate::{AdapterError, AdapterResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy and operating normally
    Healthy,
    /// Component is degraded but still operational
    Degraded,
    /// Component is unhealthy and may not be operational
    Unhealthy,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Healthy
    }
}

/// Health information for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub component: String,
    /// Current health status
    pub status: HealthStatus,
    /// Last successful check timestamp (nanoseconds)
    pub last_check_ns: u64,
    /// Success rate over the last minute (0.0 - 1.0)
    pub success_rate: f64,
    /// Average latency in microseconds
    pub avg_latency_us: f64,
    /// Additional metrics or messages
    pub details: HashMap<String, String>,
}

/// Health snapshot across all components
#[derive(Debug, Clone, Default)]
pub struct HealthSnapshot {
    /// Individual component health statuses
    components: Arc<RwLock<HashMap<String, ComponentHealth>>>,
    /// Overall system health
    overall_status: Arc<RwLock<HealthStatus>>,
}

impl HealthSnapshot {
    /// Create a new health snapshot
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            overall_status: Arc::new(RwLock::new(HealthStatus::Healthy)),
        }
    }

    /// Update component health
    pub fn update_component(&self, health: ComponentHealth) {
        let mut components = self.components.write();
        components.insert(health.component.clone(), health);
        
        // Recalculate overall status
        let overall = self.calculate_overall_status(&components);
        *self.overall_status.write() = overall;
    }

    /// Get current health status for a component
    pub fn get_component(&self, component: &str) -> Option<ComponentHealth> {
        self.components.read().get(component).cloned()
    }

    /// Get overall system health
    pub fn overall_status(&self) -> HealthStatus {
        *self.overall_status.read()
    }

    /// Get all component health statuses
    pub fn all_components(&self) -> Vec<ComponentHealth> {
        self.components.read().values().cloned().collect()
    }

    /// Calculate overall health based on component statuses
    fn calculate_overall_status(&self, components: &HashMap<String, ComponentHealth>) -> HealthStatus {
        let mut unhealthy_count = 0;
        let mut degraded_count = 0;

        for health in components.values() {
            match health.status {
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Healthy => {}
            }
        }

        // If any critical component is unhealthy, system is unhealthy
        if unhealthy_count > 0 {
            HealthStatus::Unhealthy
        } else if degraded_count > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Configuration for health adapter
#[derive(Debug, Clone, Deserialize)]
pub struct HealthConfig {
    /// Check interval in seconds
    pub check_interval_secs: u64,
    /// Timeout for health checks in seconds
    pub check_timeout_secs: u64,
    /// Degraded threshold (success rate)
    pub degraded_threshold: f64,
    /// Unhealthy threshold (success rate)
    pub unhealthy_threshold: f64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            check_timeout_secs: 5,
            degraded_threshold: 0.8,
            unhealthy_threshold: 0.5,
        }
    }
}

/// Health monitoring adapter
pub struct HealthAdapter {
    snapshot: Arc<HealthSnapshot>,
    config: HealthConfig,
}

impl HealthAdapter {
    /// Create a new health adapter
    pub fn new(config: HealthConfig) -> Self {
        Self {
            snapshot: Arc::new(HealthSnapshot::new()),
            config,
        }
    }

    /// Get current health snapshot
    pub fn snapshot(&self) -> Arc<HealthSnapshot> {
        self.snapshot.clone()
    }
}

#[async_trait]
impl crate::Adapter for HealthAdapter {
    type Config = HealthConfig;
    type Error = AdapterError;

    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        self.config = config;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), Self::Error> {
        // Health adapter is always ready
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn health_check(&self) -> Result<(), Self::Error> {
        // Self health check
        Ok(())
    }

    fn name(&self) -> &'static str {
        "health"
    }
}

/// Health check trait for components
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check and return status
    async fn check_health(&self) -> AdapterResult<ComponentHealth>;
} 