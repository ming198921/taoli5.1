//! 存储管理器实现

use crate::{
    config::ConfigCenter,
    errors::Result,
    types::{SystemState, PerformanceStats},
};
use std::sync::Arc;

/// 存储管理器
pub struct StorageManager {
    #[allow(dead_code)]
    config: Arc<ConfigCenter>,
}

impl StorageManager {
    pub async fn new(config: &Arc<ConfigCenter>) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
    
    pub async fn save_system_state(&self, _state: &SystemState) -> Result<()> {
        Ok(())
    }
    
    pub async fn save_performance_stats(&self, _stats: &PerformanceStats) -> Result<()> {
        Ok(())
    }
} 