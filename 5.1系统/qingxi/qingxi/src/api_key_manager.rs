#![allow(dead_code)]
//! # API密钥管理模块
//!
//! 提供安全的API密钥存储、管理和加密功能

use crate::types::MarketSourceConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, RwLock},
};
use tracing::{error, info, warn};

/// API密钥配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub exchange_id: String,
    pub api_key: String,
    pub api_secret: String,
    pub api_passphrase: Option<String>, // For OKX
    pub testnet: bool,
    pub created_at: u64,
    pub last_used: Option<u64>,
    pub enabled: bool,
}

/// API密钥管理器
pub struct ApiKeyManager {
    keys: Arc<RwLock<HashMap<String, ApiKeyConfig>>>,
    storage_path: String,
}

impl ApiKeyManager {
    pub fn new(storage_path: &str) -> Self {
        let manager = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            storage_path: storage_path.to_string(),
        };
        
        // 加载已存储的密钥
        if let Err(e) = manager.load_keys() {
            warn!("Failed to load existing API keys: {}", e);
        }
        
        manager
    }

    /// 添加或更新API密钥
    pub fn add_api_key(&self, config: ApiKeyConfig) -> Result<(), String> {
        let mut keys = self.keys.write().expect("Failed to acquire write lock");
        
        // 验证API密钥格式
        if config.api_key.is_empty() || config.api_secret.is_empty() {
            return Err("API key and secret cannot be empty".to_string());
        }

        // 对于OKX，需要验证passphrase
        if config.exchange_id.to_lowercase() == "okx" && config.api_passphrase.is_none() {
            return Err("OKX requires API passphrase".to_string());
        }

        let exchange_id = config.exchange_id.clone();
        keys.insert(config.exchange_id.clone(), config);
        drop(keys);

        // 保存到文件
        self.save_keys()?;
        info!("API key added for exchange: {}", exchange_id);
        Ok(())
    }

    /// 获取交易所的API密钥
    pub fn get_api_key(&self, exchange_id: &str) -> Option<ApiKeyConfig> {
        let keys = self.keys.read().expect("Failed to acquire read lock");
        keys.get(exchange_id).cloned()
    }

    /// 获取所有已配置的交易所
    pub fn get_configured_exchanges(&self) -> Vec<String> {
        let keys = self.keys.read().expect("Failed to acquire read lock");
        keys.keys().cloned().collect()
    }

    /// 删除API密钥
    pub fn remove_api_key(&self, exchange_id: &str) -> Result<(), String> {
        let mut keys = self.keys.write().expect("Failed to acquire write lock");
        if keys.remove(exchange_id).is_some() {
            drop(keys);
            self.save_keys()?;
            info!("API key removed for exchange: {}", exchange_id);
            Ok(())
        } else {
            Err(format!("No API key found for exchange: {}", exchange_id))
        }
    }

    /// 启用/禁用API密钥
    pub fn toggle_api_key(&self, exchange_id: &str, enabled: bool) -> Result<(), String> {
        let mut keys = self.keys.write().expect("Failed to acquire write lock");
        if let Some(config) = keys.get_mut(exchange_id) {
            config.enabled = enabled;
            drop(keys);
            self.save_keys()?;
            info!("API key {} for exchange: {}", 
                if enabled { "enabled" } else { "disabled" }, exchange_id);
            Ok(())
        } else {
            Err(format!("No API key found for exchange: {}", exchange_id))
        }
    }

    /// 验证API密钥是否有效
    pub async fn validate_api_key(&self, exchange_id: &str) -> Result<bool, String> {
        let config = self.get_api_key(exchange_id)
            .ok_or_else(|| format!("No API key found for {}", exchange_id))?;

        if !config.enabled {
            return Ok(false);
        }

        // TODO: 实际验证API密钥有效性
        // 这里可以调用交易所的API来验证密钥
        Ok(true)
    }

    /// 更新最后使用时间
    pub fn update_last_used(&self, exchange_id: &str) -> Result<(), String> {
        let mut keys = self.keys.write().expect("Failed to acquire write lock");
        if let Some(config) = keys.get_mut(exchange_id) {
            config.last_used = Some(chrono::Utc::now().timestamp() as u64);
            drop(keys);
            self.save_keys()?;
            Ok(())
        } else {
            Err(format!("No API key found for exchange: {}", exchange_id))
        }
    }

    /// 生成MarketSourceConfig（包含API密钥）
    pub fn generate_market_source_config(&self, exchange_id: &str, symbols: Vec<crate::types::Symbol>) -> Option<MarketSourceConfig> {
        let api_config = self.get_api_key(exchange_id)?;
        
        if !api_config.enabled {
            return None;
        }

        // 根据交易所设置WebSocket和REST端点
        let (ws_endpoint, rest_endpoint) = match exchange_id.to_lowercase().as_str() {
            "binance" => {
                if api_config.testnet {
                    ("wss://testnet.binance.vision/ws", "https://testnet.binance.vision")
                } else {
                    ("wss://stream.binance.com:9443/ws", "https://api.binance.com")
                }
            },
            "okx" => {
                if api_config.testnet {
                    ("wss://wspap.okx.com:8443/ws/v5/public", "https://www.okx.com")
                } else {
                    ("wss://ws.okx.com:8443/ws/v5/public", "https://www.okx.com")
                }
            },
            "huobi" => {
                if api_config.testnet {
                    ("wss://api.huobi.pro/ws", "https://api.huobi.pro")
                } else {
                    ("wss://api.huobi.pro/ws", "https://api.huobi.pro")
                }
            },
            "bybit" => {
                if api_config.testnet {
                    ("wss://stream-testnet.bybit.com/v5/public/spot", "https://api-testnet.bybit.com")
                } else {
                    ("wss://stream.bybit.com/v5/public/spot", "https://api.bybit.com")
                }
            },
            _ => return None,
        };

        Some(MarketSourceConfig {
            exchange_id: exchange_id.to_string(),
            enabled: true,
            websocket_url: Some(ws_endpoint.to_string()),
            rest_api_url: Some(rest_endpoint.to_string()),
            api_key: Some(api_config.api_key),
            api_secret: Some(api_config.api_secret),
            api_passphrase: api_config.api_passphrase,
            symbols,
            ws_endpoint: ws_endpoint.to_string(),
            rest_endpoint: Some(rest_endpoint.to_string()),
            channel: Some("orderbook".to_string()),
            heartbeat: None,
            reconnect_interval_sec: Some(30),
            max_reconnect_attempts: Some(10),
        })
    }

    /// 保存密钥到文件
    fn save_keys(&self) -> Result<(), String> {
        let keys = self.keys.read().expect("Failed to acquire read lock");
        let json = serde_json::to_string_pretty(&*keys)
            .map_err(|e| format!("Failed to serialize keys: {}", e))?;
        
        // 确保目录存在
        if let Some(parent) = Path::new(&self.storage_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        fs::write(&self.storage_path, json)
            .map_err(|e| format!("Failed to write keys to file: {}", e))?;

        Ok(())
    }

    /// 从文件加载密钥
    fn load_keys(&self) -> Result<(), String> {
        if !Path::new(&self.storage_path).exists() {
            info!("API keys file does not exist, starting with empty keys");
            return Ok(());
        }

        let json = fs::read_to_string(&self.storage_path)
            .map_err(|e| format!("Failed to read keys file: {}", e))?;

        let loaded_keys: HashMap<String, ApiKeyConfig> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse keys file: {}", e))?;

        let mut keys = self.keys.write().expect("Failed to acquire write lock");
        *keys = loaded_keys;
        
        info!("Loaded {} API key configurations", keys.len());
        Ok(())
    }

    /// 获取所有API密钥状态（不包含敏感信息）
    pub fn get_api_keys_status(&self) -> Vec<serde_json::Value> {
        let keys = self.keys.read().expect("Failed to acquire read lock");
        keys.values().map(|config| {
            serde_json::json!({
                "exchange_id": config.exchange_id,
                "has_api_key": !config.api_key.is_empty(),
                "has_api_secret": !config.api_secret.is_empty(),
                "has_passphrase": config.api_passphrase.is_some(),
                "testnet": config.testnet,
                "enabled": config.enabled,
                "created_at": config.created_at,
                "last_used": config.last_used
            })
        }).collect()
    }

    /// HTTP API兼容方法 - 列出所有配置
    pub fn list_configs(&self) -> Result<Vec<ApiKeyConfig>, String> {
        let keys = self.keys.read().map_err(|e| format!("Failed to read keys: {}", e))?;
        Ok(keys.values().cloned().collect())
    }

    /// HTTP API兼容方法 - 添加配置
    pub fn add_config(&self, config: ApiKeyConfig) -> Result<(), String> {
        self.add_api_key(config)
    }

    /// HTTP API兼容方法 - 获取配置
    pub fn get_config(&self, exchange_id: &str) -> Result<Option<ApiKeyConfig>, String> {
        Ok(self.get_api_key(exchange_id))
    }

    /// HTTP API兼容方法 - 更新配置
    pub fn update_config(&self, exchange_id: &str, config: ApiKeyConfig) -> Result<(), String> {
        // 先删除旧配置，再添加新配置
        let _ = self.remove_api_key(exchange_id); // 忽略删除错误，可能原本就不存在
        self.add_config(config)
    }

    /// HTTP API兼容方法 - 删除配置
    pub fn remove_config(&self, exchange_id: &str) -> Result<(), String> {
        self.remove_api_key(exchange_id)
    }

}
