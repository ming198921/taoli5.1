use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::strategy::core::{
    ArbitrageStrategy, StrategyType, StrategyStatus, StrategyConfig, 
    StrategyMetrics, StrategyError
};

/// 注册中心统计信息
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_strategies: usize,
    pub active_strategies: usize,
    pub failed_strategies: usize,
    pub paused_strategies: usize,
    pub strategies_by_type: HashMap<StrategyType, usize>,
    pub average_uptime_hours: f64,
    pub memory_usage_mb: f64,
    pub last_updated: DateTime<Utc>,
}

/// 策略注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRegistration {
    pub strategy_id: String,
    pub strategy_type: StrategyType,
    pub version: String,
    pub description: String,
    pub status: StrategyStatus,
    pub config: StrategyConfig,
    pub metrics: StrategyMetrics,
    pub registered_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub resource_usage: ResourceUsage,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u32,
    pub network_io_bytes: u64,
    pub api_calls_per_minute: u32,
    pub active_opportunities: u32,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            network_io_bytes: 0,
            api_calls_per_minute: 0,
            active_opportunities: 0,
        }
    }
}

/// 策略注册中心
pub struct StrategyRegistry {
    /// 已注册的策略实例
    strategies: Arc<RwLock<HashMap<String, Arc<RwLock<dyn ArbitrageStrategy>>>>>,
    /// 策略注册信息
    registrations: Arc<RwLock<HashMap<String, StrategyRegistration>>>,
    /// 策略类型索引
    type_index: Arc<RwLock<HashMap<StrategyType, Vec<String>>>>,
    /// 策略依赖图
    dependency_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl StrategyRegistry {
    /// 创建新的策略注册中心
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            registrations: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册策略
    pub async fn register_strategy(
        &self,
        strategy: Arc<RwLock<dyn ArbitrageStrategy>>,
        config: StrategyConfig,
        dependencies: Vec<String>,
    ) -> Result<(), StrategyError> {
        let strategy_guard = strategy.read().await;
        let strategy_id = strategy_guard.strategy_id().to_string();
        let strategy_type = strategy_guard.strategy_type();
        let description = strategy_guard.description().to_string();
        drop(strategy_guard);

        // 验证依赖关系
        self.validate_dependencies(&dependencies).await?;

        // 创建注册信息
        let registration = StrategyRegistration {
            strategy_id: strategy_id.clone(),
            strategy_type,
            version: "1.0.0".to_string(),
            description,
            status: StrategyStatus::Active,
            config,
            metrics: StrategyMetrics {
                strategy_id: strategy_id.clone(),
                ..Default::default()
            },
            registered_at: Utc::now(),
            last_updated: Utc::now(),
            dependencies: dependencies.clone(),
            resource_usage: ResourceUsage::default(),
        };

        // 注册策略
        {
            let mut strategies = self.strategies.write().await;
            let mut registrations = self.registrations.write().await;
            let mut type_index = self.type_index.write().await;
            let mut dependency_graph = self.dependency_graph.write().await;

            strategies.insert(strategy_id.clone(), strategy);
            registrations.insert(strategy_id.clone(), registration);
            
            // 更新类型索引
            type_index
                .entry(strategy_type)
                .or_insert_with(Vec::new)
                .push(strategy_id.clone());

            // 更新依赖图
            dependency_graph.insert(strategy_id.clone(), dependencies);
        }

        tracing::info!(
            strategy_id = %strategy_id,
            strategy_type = %strategy_type,
            "Strategy registered successfully"
        );

        Ok(())
    }

    /// 注销策略
    pub async fn unregister_strategy(&self, strategy_id: &str) -> Result<(), StrategyError> {
        // 检查是否有其他策略依赖此策略
        self.check_dependents(strategy_id).await?;

        let mut strategies = self.strategies.write().await;
        let mut registrations = self.registrations.write().await;
        let mut type_index = self.type_index.write().await;
        let mut dependency_graph = self.dependency_graph.write().await;

        if let Some(registration) = registrations.remove(strategy_id) {
            strategies.remove(strategy_id);
            
            // 从类型索引中移除
            if let Some(type_strategies) = type_index.get_mut(&registration.strategy_type) {
                type_strategies.retain(|id| id != strategy_id);
                if type_strategies.is_empty() {
                    type_index.remove(&registration.strategy_type);
                }
            }

            // 从依赖图中移除
            dependency_graph.remove(strategy_id);

            tracing::info!(
                strategy_id = %strategy_id,
                "Strategy unregistered successfully"
            );

            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", strategy_id)
            ))
        }
    }

    /// 获取策略实例
    pub async fn get_strategy(&self, strategy_id: &str) -> Option<Arc<RwLock<dyn ArbitrageStrategy>>> {
        let strategies = self.strategies.read().await;
        strategies.get(strategy_id).cloned()
    }

    /// 获取策略注册信息
    pub async fn get_registration(&self, strategy_id: &str) -> Option<StrategyRegistration> {
        let registrations = self.registrations.read().await;
        registrations.get(strategy_id).cloned()
    }

    /// 根据类型获取策略列表
    pub async fn get_strategies_by_type(&self, strategy_type: StrategyType) -> Vec<String> {
        let type_index = self.type_index.read().await;
        type_index.get(&strategy_type).cloned().unwrap_or_default()
    }

    /// 获取所有活跃策略
    pub async fn get_active_strategies(&self) -> Vec<String> {
        let registrations = self.registrations.read().await;
        registrations
            .values()
            .filter(|reg| reg.status == StrategyStatus::Active)
            .map(|reg| reg.strategy_id.clone())
            .collect()
    }

    /// 更新策略状态
    pub async fn update_strategy_status(
        &self,
        strategy_id: &str,
        status: StrategyStatus,
    ) -> Result<(), StrategyError> {
        let mut registrations = self.registrations.write().await;
        
        if let Some(registration) = registrations.get_mut(strategy_id) {
            registration.status = status;
            registration.last_updated = Utc::now();
            
            tracing::info!(
                strategy_id = %strategy_id,
                status = ?status,
                "Strategy status updated"
            );
            
            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", strategy_id)
            ))
        }
    }

    /// 更新策略配置
    pub async fn update_strategy_config(
        &self,
        strategy_id: &str,
        config: StrategyConfig,
    ) -> Result<(), StrategyError> {
        // 更新注册信息中的配置
        {
            let mut registrations = self.registrations.write().await;
            if let Some(registration) = registrations.get_mut(strategy_id) {
                registration.config = config.clone();
                registration.last_updated = Utc::now();
            } else {
                return Err(StrategyError::ConfigurationError(
                    format!("Strategy {} not found", strategy_id)
                ));
            }
        }

        // 更新策略实例配置
        if let Some(strategy) = self.get_strategy(strategy_id).await {
            let mut strategy_guard = strategy.write().await;
            let config_map = serde_json::to_value(&config)
                .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            strategy_guard.update_config(config_map).await?;
        }

        tracing::info!(
            strategy_id = %strategy_id,
            "Strategy configuration updated"
        );

        Ok(())
    }

    /// 更新策略指标
    pub async fn update_strategy_metrics(
        &self,
        strategy_id: &str,
        metrics: StrategyMetrics,
    ) -> Result<(), StrategyError> {
        let mut registrations = self.registrations.write().await;
        
        if let Some(registration) = registrations.get_mut(strategy_id) {
            registration.metrics = metrics;
            registration.last_updated = Utc::now();
            
            tracing::debug!(
                strategy_id = %strategy_id,
                "Strategy metrics updated"
            );
            
            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", strategy_id)
            ))
        }
    }

    /// 获取策略统计信息
    pub async fn get_registry_stats(&self) -> RegistryStats {
        let registrations = self.registrations.read().await;
        let type_index = self.type_index.read().await;

        let total_strategies = registrations.len();
        let active_strategies = registrations
            .values()
            .filter(|reg| reg.status == StrategyStatus::Active)
            .count();

        let strategies_by_type = type_index
            .iter()
            .map(|(strategy_type, strategies)| (*strategy_type, strategies.len()))
            .collect();

        // 计算实际的失败策略数
        let failed_strategies = strategies.values()
            .filter(|strategy| strategy.state == StrategyState::Failed)
            .count();
        
        // 计算实际的暂停策略数
        let paused_strategies = strategies.values()
            .filter(|strategy| strategy.state == StrategyState::Paused)
            .count();
        
        // 计算实际的平均运行时间
        let total_uptime_seconds: f64 = strategies.values()
            .map(|strategy| {
                let elapsed = strategy.last_updated.signed_duration_since(strategy.creation_time);
                elapsed.num_seconds() as f64
            })
            .sum();
        let average_uptime_hours = if total_strategies > 0 {
            total_uptime_seconds / 3600.0 / total_strategies as f64
        } else {
            0.0
        };
        
        // 计算实际的内存使用（基于策略数量和状态估算）
        let memory_usage_mb = self.estimate_memory_usage(&strategies).await;

        RegistryStats {
            total_strategies,
            active_strategies,
            failed_strategies,
            paused_strategies,
            strategies_by_type,
            average_uptime_hours,
            memory_usage_mb,
            last_updated: Utc::now(),
        }
    }

    /// 验证依赖关系
    async fn validate_dependencies(&self, dependencies: &[String]) -> Result<(), StrategyError> {
        let registrations = self.registrations.read().await;
        
        for dep in dependencies {
            if !registrations.contains_key(dep) {
                return Err(StrategyError::ConfigurationError(
                    format!("Dependency strategy {} not found", dep)
                ));
            }
        }
        
        Ok(())
    }

    /// 检查是否有其他策略依赖指定策略
    async fn check_dependents(&self, strategy_id: &str) -> Result<(), StrategyError> {
        let dependency_graph = self.dependency_graph.read().await;
        
        for (dependent_id, dependencies) in dependency_graph.iter() {
            if dependencies.contains(&strategy_id.to_string()) {
                return Err(StrategyError::ConfigurationError(
                    format!("Strategy {} is depended on by strategy {}", strategy_id, dependent_id)
                ));
            }
        }
        
        Ok(())
    }

    /// 执行策略健康检查
    pub async fn health_check(&self) -> HashMap<String, Result<(), StrategyError>> {
        let strategies = self.strategies.read().await;
        let mut results = HashMap::new();

        for (strategy_id, strategy) in strategies.iter() {
            let strategy_guard = strategy.read().await;
            let result = strategy_guard.health_check().await;
            results.insert(strategy_id.clone(), result);
        }

        results
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}




 
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::strategy::core::{
    ArbitrageStrategy, StrategyType, StrategyStatus, StrategyConfig, 
    StrategyMetrics, StrategyError
};

/// 注册中心统计信息
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_strategies: usize,
    pub active_strategies: usize,
    pub failed_strategies: usize,
    pub paused_strategies: usize,
    pub strategies_by_type: HashMap<StrategyType, usize>,
    pub average_uptime_hours: f64,
    pub memory_usage_mb: f64,
    pub last_updated: DateTime<Utc>,
}

/// 策略注册信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRegistration {
    pub strategy_id: String,
    pub strategy_type: StrategyType,
    pub version: String,
    pub description: String,
    pub status: StrategyStatus,
    pub config: StrategyConfig,
    pub metrics: StrategyMetrics,
    pub registered_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub dependencies: Vec<String>,
    pub resource_usage: ResourceUsage,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u32,
    pub network_io_bytes: u64,
    pub api_calls_per_minute: u32,
    pub active_opportunities: u32,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            network_io_bytes: 0,
            api_calls_per_minute: 0,
            active_opportunities: 0,
        }
    }
}

/// 策略注册中心
pub struct StrategyRegistry {
    /// 已注册的策略实例
    strategies: Arc<RwLock<HashMap<String, Arc<RwLock<dyn ArbitrageStrategy>>>>>,
    /// 策略注册信息
    registrations: Arc<RwLock<HashMap<String, StrategyRegistration>>>,
    /// 策略类型索引
    type_index: Arc<RwLock<HashMap<StrategyType, Vec<String>>>>,
    /// 策略依赖图
    dependency_graph: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl StrategyRegistry {
    /// 创建新的策略注册中心
    pub fn new() -> Self {
        Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            registrations: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册策略
    pub async fn register_strategy(
        &self,
        strategy: Arc<RwLock<dyn ArbitrageStrategy>>,
        config: StrategyConfig,
        dependencies: Vec<String>,
    ) -> Result<(), StrategyError> {
        let strategy_guard = strategy.read().await;
        let strategy_id = strategy_guard.strategy_id().to_string();
        let strategy_type = strategy_guard.strategy_type();
        let description = strategy_guard.description().to_string();
        drop(strategy_guard);

        // 验证依赖关系
        self.validate_dependencies(&dependencies).await?;

        // 创建注册信息
        let registration = StrategyRegistration {
            strategy_id: strategy_id.clone(),
            strategy_type,
            version: "1.0.0".to_string(),
            description,
            status: StrategyStatus::Active,
            config,
            metrics: StrategyMetrics {
                strategy_id: strategy_id.clone(),
                ..Default::default()
            },
            registered_at: Utc::now(),
            last_updated: Utc::now(),
            dependencies: dependencies.clone(),
            resource_usage: ResourceUsage::default(),
        };

        // 注册策略
        {
            let mut strategies = self.strategies.write().await;
            let mut registrations = self.registrations.write().await;
            let mut type_index = self.type_index.write().await;
            let mut dependency_graph = self.dependency_graph.write().await;

            strategies.insert(strategy_id.clone(), strategy);
            registrations.insert(strategy_id.clone(), registration);
            
            // 更新类型索引
            type_index
                .entry(strategy_type)
                .or_insert_with(Vec::new)
                .push(strategy_id.clone());

            // 更新依赖图
            dependency_graph.insert(strategy_id.clone(), dependencies);
        }

        tracing::info!(
            strategy_id = %strategy_id,
            strategy_type = %strategy_type,
            "Strategy registered successfully"
        );

        Ok(())
    }

    /// 注销策略
    pub async fn unregister_strategy(&self, strategy_id: &str) -> Result<(), StrategyError> {
        // 检查是否有其他策略依赖此策略
        self.check_dependents(strategy_id).await?;

        let mut strategies = self.strategies.write().await;
        let mut registrations = self.registrations.write().await;
        let mut type_index = self.type_index.write().await;
        let mut dependency_graph = self.dependency_graph.write().await;

        if let Some(registration) = registrations.remove(strategy_id) {
            strategies.remove(strategy_id);
            
            // 从类型索引中移除
            if let Some(type_strategies) = type_index.get_mut(&registration.strategy_type) {
                type_strategies.retain(|id| id != strategy_id);
                if type_strategies.is_empty() {
                    type_index.remove(&registration.strategy_type);
                }
            }

            // 从依赖图中移除
            dependency_graph.remove(strategy_id);

            tracing::info!(
                strategy_id = %strategy_id,
                "Strategy unregistered successfully"
            );

            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", strategy_id)
            ))
        }
    }

    /// 获取策略实例
    pub async fn get_strategy(&self, strategy_id: &str) -> Option<Arc<RwLock<dyn ArbitrageStrategy>>> {
        let strategies = self.strategies.read().await;
        strategies.get(strategy_id).cloned()
    }

    /// 获取策略注册信息
    pub async fn get_registration(&self, strategy_id: &str) -> Option<StrategyRegistration> {
        let registrations = self.registrations.read().await;
        registrations.get(strategy_id).cloned()
    }

    /// 根据类型获取策略列表
    pub async fn get_strategies_by_type(&self, strategy_type: StrategyType) -> Vec<String> {
        let type_index = self.type_index.read().await;
        type_index.get(&strategy_type).cloned().unwrap_or_default()
    }

    /// 获取所有活跃策略
    pub async fn get_active_strategies(&self) -> Vec<String> {
        let registrations = self.registrations.read().await;
        registrations
            .values()
            .filter(|reg| reg.status == StrategyStatus::Active)
            .map(|reg| reg.strategy_id.clone())
            .collect()
    }

    /// 更新策略状态
    pub async fn update_strategy_status(
        &self,
        strategy_id: &str,
        status: StrategyStatus,
    ) -> Result<(), StrategyError> {
        let mut registrations = self.registrations.write().await;
        
        if let Some(registration) = registrations.get_mut(strategy_id) {
            registration.status = status;
            registration.last_updated = Utc::now();
            
            tracing::info!(
                strategy_id = %strategy_id,
                status = ?status,
                "Strategy status updated"
            );
            
            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", strategy_id)
            ))
        }
    }

    /// 更新策略配置
    pub async fn update_strategy_config(
        &self,
        strategy_id: &str,
        config: StrategyConfig,
    ) -> Result<(), StrategyError> {
        // 更新注册信息中的配置
        {
            let mut registrations = self.registrations.write().await;
            if let Some(registration) = registrations.get_mut(strategy_id) {
                registration.config = config.clone();
                registration.last_updated = Utc::now();
            } else {
                return Err(StrategyError::ConfigurationError(
                    format!("Strategy {} not found", strategy_id)
                ));
            }
        }

        // 更新策略实例配置
        if let Some(strategy) = self.get_strategy(strategy_id).await {
            let mut strategy_guard = strategy.write().await;
            let config_map = serde_json::to_value(&config)
                .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?
                .as_object()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            
            strategy_guard.update_config(config_map).await?;
        }

        tracing::info!(
            strategy_id = %strategy_id,
            "Strategy configuration updated"
        );

        Ok(())
    }

    /// 更新策略指标
    pub async fn update_strategy_metrics(
        &self,
        strategy_id: &str,
        metrics: StrategyMetrics,
    ) -> Result<(), StrategyError> {
        let mut registrations = self.registrations.write().await;
        
        if let Some(registration) = registrations.get_mut(strategy_id) {
            registration.metrics = metrics;
            registration.last_updated = Utc::now();
            
            tracing::debug!(
                strategy_id = %strategy_id,
                "Strategy metrics updated"
            );
            
            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", strategy_id)
            ))
        }
    }

    /// 获取策略统计信息
    pub async fn get_registry_stats(&self) -> RegistryStats {
        let registrations = self.registrations.read().await;
        let type_index = self.type_index.read().await;

        let total_strategies = registrations.len();
        let active_strategies = registrations
            .values()
            .filter(|reg| reg.status == StrategyStatus::Active)
            .count();

        let strategies_by_type = type_index
            .iter()
            .map(|(strategy_type, strategies)| (*strategy_type, strategies.len()))
            .collect();

        RegistryStats {
            total_strategies,
            active_strategies,
            failed_strategies: 0, // TODO: 计算实际的失败策略数
            paused_strategies: 0, // TODO: 计算实际的暂停策略数
            strategies_by_type,
            average_uptime_hours: 0.0, // TODO: 计算实际的平均运行时间
            memory_usage_mb: 0.0, // TODO: 计算实际的内存使用
            last_updated: Utc::now(),
        }
    }

    /// 验证依赖关系
    async fn validate_dependencies(&self, dependencies: &[String]) -> Result<(), StrategyError> {
        let registrations = self.registrations.read().await;
        
        for dep in dependencies {
            if !registrations.contains_key(dep) {
                return Err(StrategyError::ConfigurationError(
                    format!("Dependency strategy {} not found", dep)
                ));
            }
        }
        
        Ok(())
    }

    /// 检查是否有其他策略依赖指定策略
    async fn check_dependents(&self, strategy_id: &str) -> Result<(), StrategyError> {
        let dependency_graph = self.dependency_graph.read().await;
        
        for (dependent_id, dependencies) in dependency_graph.iter() {
            if dependencies.contains(&strategy_id.to_string()) {
                return Err(StrategyError::ConfigurationError(
                    format!("Strategy {} is depended on by strategy {}", strategy_id, dependent_id)
                ));
            }
        }
        
        Ok(())
    }

    /// 执行策略健康检查
    pub async fn health_check(&self) -> HashMap<String, Result<(), StrategyError>> {
        let strategies = self.strategies.read().await;
        let mut results = HashMap::new();

        for (strategy_id, strategy) in strategies.iter() {
            let strategy_guard = strategy.read().await;
            let result = strategy_guard.health_check().await;
            results.insert(strategy_id.clone(), result);
        }

        results
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}




 






        for (strategy_id, strategy) in strategies.iter() {
            let strategy_guard = strategy.read().await;
            let result = strategy_guard.health_check().await;
            results.insert(strategy_id.clone(), result);
        }

        results
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}




 




