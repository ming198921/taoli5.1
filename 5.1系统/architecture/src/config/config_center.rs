//! 配置中心模块 - 提供集中式配置管理

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};
use crate::config::{StructuredConfigCenter, ExchangeConfig};

/// 配置中心
#[derive(Debug, Clone)]
pub struct ConfigCenter {
    /// 配置数据
    config_data: Arc<RwLock<HashMap<String, serde_json::Value>>>,
}

/// 配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub key: String,
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub category: Option<String>,
}

impl ConfigCenter {
    /// 创建新的配置中心
    pub fn new() -> Self {
        Self {
            config_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 设置配置项
    #[instrument(skip(self, value))]
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json_value = serde_json::to_value(value)?;
        let mut data = self.config_data.write().await;
        data.insert(key.to_string(), json_value);
        info!("Config updated: {}", key);
        Ok(())
    }

    /// 获取配置项
    #[instrument(skip(self))]
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let data = self.config_data.read().await;
        if let Some(value) = data.get(key) {
            let result = serde_json::from_value(value.clone())?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// 获取字符串配置
    pub async fn get_string(&self, key: &str) -> Option<String> {
        self.get::<String>(key).await.unwrap_or(None)
    }

    /// 获取整数配置
    pub async fn get_i64(&self, key: &str) -> Option<i64> {
        self.get::<i64>(key).await.unwrap_or(None)
    }

    /// 获取浮点数配置
    pub async fn get_f64(&self, key: &str) -> Option<f64> {
        self.get::<f64>(key).await.unwrap_or(None)
    }

    /// 获取布尔配置
    pub async fn get_bool(&self, key: &str) -> Option<bool> {
        self.get::<bool>(key).await.unwrap_or(None)
    }

    /// 删除配置项
    #[instrument(skip(self))]
    pub async fn remove(&self, key: &str) -> bool {
        let mut data = self.config_data.write().await;
        data.remove(key).is_some()
    }

    /// 获取所有配置键
    pub async fn keys(&self) -> Vec<String> {
        let data = self.config_data.read().await;
        data.keys().cloned().collect()
    }

    /// 清空所有配置
    pub async fn clear(&self) {
        let mut data = self.config_data.write().await;
        data.clear();
    }

    /// 从配置文件加载配置中心
    pub async fn load(_config_path: &str) -> Result<Self> {
        // 实际实现中会从配置文件加载，这里为了编译通过返回默认实例
        Ok(Self::new())
    }

    /// 获取系统配置
    pub async fn get_system_config(&self) -> Result<crate::config::SystemConfig> {
        // 返回默认配置
        Ok(crate::config::SystemConfig::default())
    }

    /// 获取风险配置
    pub async fn get_risk_config(&self) -> Result<crate::config::RiskConfig> {
        // 返回默认配置
        Ok(crate::config::RiskConfig::default())
    }

    /// 获取策略配置
    pub async fn get_strategy_configs(&self) -> Result<Vec<crate::config::StrategyConfig>> {
        // 返回空配置列表
        Ok(Vec::new())
    }

    /// 获取市场状态配置
    pub async fn get_market_state_config(&self) -> serde_json::Value {
        // 返回默认的市场状态配置
        serde_json::json!({
            "update_interval_ms": 1000,
            "price_change_threshold": 0.01,
            "volume_change_threshold": 0.05
        })
    }

    /// 获取最小利润配置
    pub async fn get_min_profit_config(&self) -> serde_json::Value {
        // 返回默认的最小利润配置
        serde_json::json!({
            "min_profit_percentage": 0.001,
            "min_profit_usdt": 1.0
        })
    }

    /// 更新配置
    pub async fn update_config(&self, key: &str, value: serde_json::Value) -> Result<()> {
        let mut data = self.config_data.write().await;
        data.insert(key.to_string(), value);
        Ok(())
    }

    /// 获取监控配置
    pub async fn get_monitoring_config(&self) -> serde_json::Value {
        // 返回默认的监控配置
        serde_json::json!({
            "metrics_interval_seconds": 30,
            "alert_thresholds": {
                "cpu_usage": 80.0,
                "memory_usage": 90.0,
                "error_rate": 0.01
            }
        })
    }

    /// 获取结构化配置中心
    pub async fn get_structured_config(&self) -> Result<StructuredConfigCenter> {
        // 返回默认的结构化配置
        Ok(StructuredConfigCenter::new())
    }

    /// 获取交易所配置
    pub async fn get_exchange_configs(&self) -> Vec<ExchangeConfig> {
        // 返回默认的交易所配置
        vec![
            ExchangeConfig {
                name: "binance".to_string(),
                enabled: true,
                api_key: "".to_string(),
                api_secret: "".to_string(),
                api_passphrase: None,
                sandbox_mode: true,
                rate_limit: 10,
                websocket_url: "wss://stream.binance.com:9443/ws".to_string(),
                rest_api_url: "https://api.binance.com".to_string(),
            }
        ]
    }
}

impl Default for ConfigCenter {
    fn default() -> Self {
        Self::new()
    }
}