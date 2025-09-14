#![allow(dead_code)]
//! # 动态配置管理模块
//!
//! 提供运行时动态调整系统参数的能力，避免频繁创建配置文件
//! 支持前端实时调整订单薄深度、币种列表、交易所参数等

use crate::{
    adapters::dynamic_registry::DynamicAdapterRegistry,
    types::{Symbol, MarketSourceConfig},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tracing::info;

/// 动态配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicParams {
    /// 订单薄深度配置 (交易所 -> 深度)
    pub orderbook_depths: HashMap<String, u32>,
    /// 币种列表配置 (交易所 -> 币种列表)
    pub symbols_config: HashMap<String, Vec<String>>,
    /// 交易所启用状态
    pub exchange_enabled: HashMap<String, bool>,
    /// 全局参数
    pub global_params: GlobalParams,
}

/// 全局动态参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalParams {
    /// 全局最大订单薄深度限制
    pub max_orderbook_depth: u32,
    /// 全局最大币种数量限制
    pub max_symbols_per_exchange: u32,
    /// 重连间隔 (秒)
    pub reconnect_interval_sec: u64,
    /// 最大重连尝试次数
    pub max_reconnect_attempts: u32,
    /// 事件缓冲区大小
    pub event_buffer_size: usize,
}

impl Default for DynamicParams {
    fn default() -> Self {
        let mut orderbook_depths = HashMap::new();
        orderbook_depths.insert("binance".to_string(), 100);
        orderbook_depths.insert("okx".to_string(), 120);
        orderbook_depths.insert("huobi".to_string(), 80);
        orderbook_depths.insert("bybit".to_string(), 60);

        let mut exchange_enabled = HashMap::new();
        exchange_enabled.insert("binance".to_string(), true);
        exchange_enabled.insert("okx".to_string(), true);
        exchange_enabled.insert("huobi".to_string(), true);
        exchange_enabled.insert("bybit".to_string(), true);

        Self {
            orderbook_depths,
            symbols_config: Self::default_symbols_config(),
            exchange_enabled,
            global_params: GlobalParams::default(),
        }
    }
}

impl Default for GlobalParams {
    fn default() -> Self {
        Self {
            max_orderbook_depth: 120,
            max_symbols_per_exchange: 100,
            reconnect_interval_sec: 5,
            max_reconnect_attempts: 10,
            event_buffer_size: 2000,
        }
    }
}

impl DynamicParams {
    /// 获取50个主流币种的默认配置
    fn default_symbols_config() -> HashMap<String, Vec<String>> {
        let main_symbols = vec![
            "BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "SOLUSDT",
            "XRPUSDT", "DOTUSDT", "DOGEUSDT", "AVAXUSDT", "SHIBUSDT",
            "MATICUSDT", "LTCUSDT", "UNIUSDT", "LINKUSDT", "ATOMUSDT",
            "NEARUSDT", "APTUSDT", "ICPUSDT", "FILUSDT", "TRXUSDT",
            "VETUSDT", "MANAUSDT", "SANDUSDT", "CHZUSDT", "ALGOUSDT",
            "HBARUSDT", "FTMUSDT", "AXSUSDT", "THETAUSDT", "XLMUSDT",
            "EOSUSDT", "AAVEUSDT", "GRTUSDT", "ENJUSDT", "MKRUSDT",
            "CRVUSDT", "XTZUSDT", "BATUSDT", "ZECUSDT", "DASHUSDT",
            "COMPUSDT", "YFIUSDT", "SNXUSDT", "SUSHIUSDT", "1INCHUSDT",
            "RNDRUSDT", "LDOUSDT", "OPUSDT", "ARBUSDT", "PEPEUSDT"
        ];

        let mut config = HashMap::new();
        
        // Binance - 标准USDT格式
        config.insert("binance".to_string(), 
            main_symbols.iter().map(|s| s.to_string()).collect());

        // OKX - 使用"-"分隔符
        config.insert("okx".to_string(),
            main_symbols.iter().map(|s| s.replace("USDT", "-USDT")).collect());

        // Huobi - 使用小写
        config.insert("huobi".to_string(),
            main_symbols.iter().map(|s| s.to_lowercase()).collect());

        // Bybit - 标准USDT格式
        config.insert("bybit".to_string(),
            main_symbols.iter().map(|s| s.to_string()).collect());

        config
    }

    /// 验证参数有效性
    pub fn validate(&self) -> Result<(), String> {
        // 验证订单薄深度
        for (exchange, depth) in &self.orderbook_depths {
            if *depth == 0 {
                return Err(format!("Invalid orderbook depth for {}: must be > 0", exchange));
            }
            if *depth > self.global_params.max_orderbook_depth {
                return Err(format!("Orderbook depth for {} ({}) exceeds global limit ({})", 
                    exchange, depth, self.global_params.max_orderbook_depth));
            }
        }

        // 验证币种数量
        for (exchange, symbols) in &self.symbols_config {
            if symbols.len() > self.global_params.max_symbols_per_exchange as usize {
                return Err(format!("Too many symbols for {} ({}), limit is {}", 
                    exchange, symbols.len(), self.global_params.max_symbols_per_exchange));
            }
        }

        Ok(())
    }
}

/// 动态配置管理器
pub struct DynamicConfigManager {
    current_params: Arc<RwLock<DynamicParams>>,
    adapter_registry: Arc<DynamicAdapterRegistry>,
}

impl DynamicConfigManager {
    pub fn new(adapter_registry: Arc<DynamicAdapterRegistry>) -> Self {
        Self {
            current_params: Arc::new(RwLock::new(DynamicParams::default())),
            adapter_registry,
        }
    }

    /// 获取当前参数 (别名方法)
    pub fn get_current_params(&self) -> DynamicParams {
        self.get_params()
    }

    /// 获取当前参数
    pub fn get_params(&self) -> DynamicParams {
        self.current_params.read()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| {
                error!("Failed to acquire read lock on dynamic params, returning defaults");
                DynamicParams::default()
            })
    }

    /// 应用预定义模板
    pub fn apply_template(&self, template_name: &str) -> Result<(), String> {
        let template_params = match template_name {
            "four_exchanges_50_symbols" => ConfigTemplates::four_exchanges_50_symbols(),
            "high_frequency_test" => ConfigTemplates::high_frequency_test(),
            "stability_test" => ConfigTemplates::stability_test(),
            _ => return Err(format!("Unknown template: {}", template_name)),
        };
        
        self.update_params(template_params)
    }

    /// 列出可用模板
    pub fn list_available_templates(&self) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "four_exchanges_50_symbols",
                "description": "4个交易所50币种测试配置",
                "features": ["跨交易所测试", "50个主流币种", "不同订单薄深度"]
            }),
            serde_json::json!({
                "name": "high_frequency_test",
                "description": "高频交易测试配置",
                "features": ["高频场景", "最大深度", "小缓冲区"]
            }),
            serde_json::json!({
                "name": "stability_test",
                "description": "稳定性测试配置",
                "features": ["保守参数", "高重连次数", "中等币种数量"]
            })
        ]
    }

    /// 设置订单薄深度 (别名方法)
    pub fn set_orderbook_depth(&self, exchange: &str, depth: u32) -> Result<(), String> {
        self.update_orderbook_depth(exchange, depth)
    }

    /// 设置币种列表 (别名方法)
    pub fn set_symbols(&self, exchange: &str, symbols: Vec<String>) -> Result<(), String> {
        self.update_symbols(exchange, symbols)
    }

    /// 设置交易所启用状态 (别名方法)
    pub fn set_exchange_enabled(&self, exchange: &str, enabled: bool) -> Result<(), String> {
        self.toggle_exchange(exchange, enabled)
    }

    /// 更新订单薄深度
    pub fn update_orderbook_depth(&self, exchange: &str, depth: u32) -> Result<(), String> {
        let mut params = self.current_params.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        
        if depth > params.global_params.max_orderbook_depth {
            return Err(format!("Depth {} exceeds global limit {}", 
                depth, params.global_params.max_orderbook_depth));
        }
        
        params.orderbook_config.insert(exchange.to_string(), depth);
        info!("Updated orderbook depth for {}: {}", exchange, depth);
        Ok(())
    }    /// 更新币种列表
    pub fn update_symbols(&self, exchange: &str, symbols: Vec<String>) -> Result<(), String> {
        let params = self.current_params.read()
            .map_err(|e| format!("Failed to acquire read lock: {}", e))?;
        
        if symbols.len() > params.global_params.max_symbols_per_exchange as usize {
            return Err(format!("Too many symbols ({}), limit is {}", 
                symbols.len(), params.global_params.max_symbols_per_exchange));
        }

        drop(params);
        let mut params = self.current_params.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        params.symbols_config.insert(exchange.to_string(), symbols.clone());
        info!("Updated symbols for {}: {} symbols", exchange, symbols.len());
        Ok(())
    }

    /// 启用/禁用交易所
    pub fn toggle_exchange(&self, exchange: &str, enabled: bool) -> Result<(), String> {
        let mut params = self.current_params.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        params.exchange_enabled.insert(exchange.to_string(), enabled);
        info!("Exchange {} is now {}", exchange, if enabled { "enabled" } else { "disabled" });
        Ok(())
    }

    /// 批量更新参数
    pub fn update_params(&self, new_params: DynamicParams) -> Result<(), String> {
        // 验证新参数
        new_params.validate()?;

        let mut current = self.current_params.write()
            .map_err(|e| format!("Failed to acquire write lock: {}", e))?;
        *current = new_params;
        info!("Dynamic parameters updated successfully");
        Ok(())
    }

    /// 生成MarketSourceConfig列表
    pub async fn generate_source_configs(&self) -> Vec<MarketSourceConfig> {
        let params = self.get_params();
        let mut configs = Vec::new();

        for (exchange_id, enabled) in &params.exchange_enabled {
            if !*enabled {
                continue;
            }

            let symbols = params.symbols_config.get(exchange_id).cloned().unwrap_or_default();
            let _depth = params.orderbook_depths.get(exchange_id).cloned().unwrap_or(50);

            // 获取适配器元数据
            if let Some(metadata) = self.adapter_registry.get_adapter_metadata(exchange_id).await {
                let ws_url = metadata.websocket_url_template.clone();
                let rest_url = metadata.rest_api_url_template.clone();
                
                let config = MarketSourceConfig {
                    exchange_id: exchange_id.clone(),
                    enabled: true,
                    websocket_url: Some(metadata.websocket_url_template),
                    rest_api_url: Some(metadata.rest_api_url_template),
                    api_key: None,
                    api_secret: None,
                    api_passphrase: None,
                    symbols: symbols.into_iter().filter_map(|s| Symbol::from_pair(&s)).collect(),
                    ws_endpoint: ws_url,
                    rest_endpoint: Some(rest_url),
                    channel: Some("orderbook".to_string()),
                    heartbeat: None,
                    reconnect_interval_sec: Some(params.global_params.reconnect_interval_sec),
                    max_reconnect_attempts: Some(params.global_params.max_reconnect_attempts),
                };
                configs.push(config);
            }
        }

        configs
    }

    /// 获取特定交易所的配置
    pub async fn get_exchange_config(&self, exchange_id: &str) -> Option<MarketSourceConfig> {
        let configs = self.generate_source_configs().await;
        configs.into_iter().find(|c| c.exchange_id == exchange_id)
    }

    /// 重置为默认参数
    pub fn reset_to_defaults(&self) {
        match self.current_params.write() {
            Ok(mut params) => {
                *params = DynamicParams::default();
                info!("Dynamic parameters reset to defaults");
            }
            Err(e) => {
                error!("Failed to reset dynamic parameters: {}", e);
            }
        }
    }

    /// 导出参数为JSON
    pub fn export_params_json(&self) -> Result<String, String> {
        let params = self.get_params();
        serde_json::to_string_pretty(&params)
            .map_err(|e| format!("Failed to serialize params: {}", e))
    }

    /// 从JSON导入参数
    pub fn import_params_json(&self, json: &str) -> Result<(), String> {
        let params: DynamicParams = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        self.update_params(params)
    }

    /// 获取参数摘要
    pub fn get_params_summary(&self) -> HashMap<String, serde_json::Value> {
        let params = self.get_params();
        let mut summary = HashMap::new();

        summary.insert("total_exchanges".to_string(), 
            serde_json::json!(params.exchange_enabled.len()));
        
        summary.insert("enabled_exchanges".to_string(),
            serde_json::json!(params.exchange_enabled.iter()
                .filter(|(_, enabled)| **enabled)
                .map(|(ex, _)| ex.clone())
                .collect::<Vec<_>>()));

        summary.insert("total_symbols".to_string(),
            serde_json::json!(params.symbols_config.values()
                .map(|symbols| symbols.len())
                .sum::<usize>()));

        summary.insert("orderbook_depths".to_string(),
            serde_json::json!(params.orderbook_depths));

        summary.insert("global_limits".to_string(),
            serde_json::json!({
                "max_orderbook_depth": params.global_params.max_orderbook_depth,
                "max_symbols_per_exchange": params.global_params.max_symbols_per_exchange
            }));

        summary
    }
}

/// 预定义配置模板
pub struct ConfigTemplates;

impl ConfigTemplates {
    /// 4个交易所50币种测试配置
    pub fn four_exchanges_50_symbols() -> DynamicParams {
        let mut params = DynamicParams::default();
        
        // 设置不同的订单薄深度以测试跨交易所差异
        params.orderbook_depths.insert("binance".to_string(), 100);
        params.orderbook_depths.insert("okx".to_string(), 120);
        params.orderbook_depths.insert("huobi".to_string(), 80);
        params.orderbook_depths.insert("bybit".to_string(), 60);
        
        params
    }

    /// 高频交易测试配置
    pub fn high_frequency_test() -> DynamicParams {
        let mut params = DynamicParams::default();
        
        // 减少币种，增加深度
        params.global_params.max_symbols_per_exchange = 20;
        params.orderbook_depths.iter_mut().for_each(|(_, depth)| *depth = 120);
        
        // 减少缓冲区大小以测试高频场景
        params.global_params.event_buffer_size = 5000;
        params.global_params.reconnect_interval_sec = 3;
        
        params
    }

    /// 稳定性测试配置
    pub fn stability_test() -> DynamicParams {
        let mut params = DynamicParams::default();
        
        // 保守的参数设置
        params.orderbook_depths.iter_mut().for_each(|(_, depth)| *depth = 50);
        params.global_params.max_symbols_per_exchange = 30;
        params.global_params.event_buffer_size = 1000;
        params.global_params.max_reconnect_attempts = 20;
        
        params
    }
}
