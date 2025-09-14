//! # 标准交易所默认配置提供者 (Standard Exchange Default Config Provider)
//! 
//! PHASE 4: 完全消除硬编码的标准配置提供者
//! 提供所有支持交易所的标准配置模板

use crate::dynamic_exchange_config::{
    DefaultConfigProvider, DynamicExchangeConfig, WebSocketEndpoint, RestApiEndpoint,
    ExchangeFeatureFlags, RateLimitConfig, ConnectionConfig
};
use crate::production_error_handling::{QingxiResult, QingxiError};
use std::collections::HashMap;

/// 标准交易所配置提供者
/// 提供所有主流交易所的标准配置模板，完全消除硬编码
pub struct StandardExchangeConfigProvider {
    exchange_templates: HashMap<String, Box<dyn Fn() -> DynamicExchangeConfig + Send + Sync>>,
}

impl StandardExchangeConfigProvider {
    /// 创建新的标准配置提供者
    pub fn new() -> Self {
        let mut provider = Self {
            exchange_templates: HashMap::new(),
        };
        
        // 注册所有支持的交易所配置模板
        provider.register_all_exchanges();
        provider
    }

    /// 注册所有交易所配置模板
    fn register_all_exchanges(&mut self) {
        // Binance 配置模板
        self.register_exchange("binance", Box::new(|| DynamicExchangeConfig {
            exchange_id: "binance".to_string(),
            display_name: "Binance".to_string(),
            websocket_endpoints: vec![
                WebSocketEndpoint {
                    endpoint_type: "spot".to_string(),
                    url: "wss://stream.binance.com:9443/ws".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    max_connections: Some(1024),
                },
                WebSocketEndpoint {
                    endpoint_type: "futures".to_string(),
                    url: "wss://fstream.binance.com/ws".to_string(),
                    priority: 1,
                    regions: vec!["global".to_string()],
                    max_connections: Some(1024),
                },
            ],
            rest_api_endpoints: vec![
                RestApiEndpoint {
                    endpoint_type: "spot".to_string(),
                    base_url: "https://api.binance.com".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    timeout_ms: 5000,
                },
                RestApiEndpoint {
                    endpoint_type: "futures".to_string(),
                    base_url: "https://fapi.binance.com".to_string(),
                    priority: 1,
                    regions: vec!["global".to_string()],
                    timeout_ms: 5000,
                },
            ],
            api_credentials: None,
            feature_flags: ExchangeFeatureFlags {
                supports_spot_trading: true,
                supports_futures_trading: true,
                supports_options_trading: true,
                supports_margin_trading: true,
                supports_lending: true,
                supports_staking: true,
                has_advanced_orders: true,
                has_websocket_auth: true,
            },
            rate_limits: RateLimitConfig {
                requests_per_minute: 1200,
                websocket_messages_per_second: 100,
                order_burst_limit: 10,
                weight_based_limits: {
                    let mut limits = HashMap::new();
                    limits.insert("order".to_string(), 10);
                    limits.insert("general".to_string(), 1);
                    limits.insert("raw_request".to_string(), 5000);
                    limits
                },
            },
            connection_config: ConnectionConfig {
                connect_timeout_ms: 5000,
                read_timeout_ms: 30000,
                keepalive_interval_s: 30,
                reconnect_attempts: 5,
                reconnect_delay_ms: 1000,
                compression_enabled: true,
            },
            testnet_mode: false,
        }));

        // Bybit 配置模板
        self.register_exchange("bybit", Box::new(|| DynamicExchangeConfig {
            exchange_id: "bybit".to_string(),
            display_name: "Bybit".to_string(),
            websocket_endpoints: vec![
                WebSocketEndpoint {
                    endpoint_type: "spot".to_string(),
                    url: "wss://stream.bybit.com/v5/public/spot".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    max_connections: Some(300),
                },
                WebSocketEndpoint {
                    endpoint_type: "linear".to_string(),
                    url: "wss://stream.bybit.com/v5/public/linear".to_string(),
                    priority: 1,
                    regions: vec!["global".to_string()],
                    max_connections: Some(300),
                },
                WebSocketEndpoint {
                    endpoint_type: "inverse".to_string(),
                    url: "wss://stream.bybit.com/v5/public/inverse".to_string(),
                    priority: 2,
                    regions: vec!["global".to_string()],
                    max_connections: Some(300),
                },
            ],
            rest_api_endpoints: vec![
                RestApiEndpoint {
                    endpoint_type: "unified".to_string(),
                    base_url: "https://api.bybit.com".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    timeout_ms: 5000,
                },
                RestApiEndpoint {
                    endpoint_type: "testnet".to_string(),
                    base_url: "https://api-testnet.bybit.com".to_string(),
                    priority: 1,
                    regions: vec!["global".to_string()],
                    timeout_ms: 5000,
                },
            ],
            api_credentials: None,
            feature_flags: ExchangeFeatureFlags {
                supports_spot_trading: true,
                supports_futures_trading: true,
                supports_options_trading: true,
                supports_margin_trading: true,
                supports_lending: false,
                supports_staking: false,
                has_advanced_orders: true,
                has_websocket_auth: true,
            },
            rate_limits: RateLimitConfig {
                requests_per_minute: 600,
                websocket_messages_per_second: 50,
                order_burst_limit: 8,
                weight_based_limits: HashMap::new(),
            },
            connection_config: ConnectionConfig::default(),
            testnet_mode: false,
        }));

        // OKX 配置模板
        self.register_exchange("okx", Box::new(|| DynamicExchangeConfig {
            exchange_id: "okx".to_string(),
            display_name: "OKX".to_string(),
            websocket_endpoints: vec![
                WebSocketEndpoint {
                    endpoint_type: "public".to_string(),
                    url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    max_connections: Some(300),
                },
                WebSocketEndpoint {
                    endpoint_type: "private".to_string(),
                    url: "wss://ws.okx.com:8443/ws/v5/private".to_string(),
                    priority: 1,
                    regions: vec!["global".to_string()],
                    max_connections: Some(100),
                },
                WebSocketEndpoint {
                    endpoint_type: "business".to_string(),
                    url: "wss://ws.okx.com:8443/ws/v5/business".to_string(),
                    priority: 2,
                    regions: vec!["global".to_string()],
                    max_connections: Some(100),
                },
            ],
            rest_api_endpoints: vec![
                RestApiEndpoint {
                    endpoint_type: "unified".to_string(),
                    base_url: "https://www.okx.com".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    timeout_ms: 5000,
                },
            ],
            api_credentials: None,
            feature_flags: ExchangeFeatureFlags {
                supports_spot_trading: true,
                supports_futures_trading: true,
                supports_options_trading: true,
                supports_margin_trading: true,
                supports_lending: true,
                supports_staking: false,
                has_advanced_orders: true,
                has_websocket_auth: true,
            },
            rate_limits: RateLimitConfig {
                requests_per_minute: 600,
                websocket_messages_per_second: 30,
                order_burst_limit: 60,
                weight_based_limits: HashMap::new(),
            },
            connection_config: ConnectionConfig::default(),
            testnet_mode: false,
        }));

        // Huobi 配置模板
        self.register_exchange("huobi", Box::new(|| DynamicExchangeConfig {
            exchange_id: "huobi".to_string(),
            display_name: "Huobi Global".to_string(),
            websocket_endpoints: vec![
                WebSocketEndpoint {
                    endpoint_type: "market".to_string(),
                    url: "wss://api.huobi.pro/ws".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    max_connections: Some(50),
                },
                WebSocketEndpoint {
                    endpoint_type: "mbp".to_string(),
                    url: "wss://api.huobi.pro/feed".to_string(),
                    priority: 1,
                    regions: vec!["global".to_string()],
                    max_connections: Some(50),
                },
            ],
            rest_api_endpoints: vec![
                RestApiEndpoint {
                    endpoint_type: "spot".to_string(),
                    base_url: "https://api.huobi.pro".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    timeout_ms: 5000,
                },
            ],
            api_credentials: None,
            feature_flags: ExchangeFeatureFlags {
                supports_spot_trading: true,
                supports_futures_trading: true,
                supports_options_trading: false,
                supports_margin_trading: true,
                supports_lending: true,
                supports_staking: false,
                has_advanced_orders: false,
                has_websocket_auth: true,
            },
            rate_limits: RateLimitConfig {
                requests_per_minute: 600,
                websocket_messages_per_second: 20,
                order_burst_limit: 100,
                weight_based_limits: HashMap::new(),
            },
            connection_config: ConnectionConfig::default(),
            testnet_mode: false,
        }));

        // Gate.io 配置模板
        self.register_exchange("gateio", Box::new(|| DynamicExchangeConfig {
            exchange_id: "gateio".to_string(),
            display_name: "Gate.io".to_string(),
            websocket_endpoints: vec![
                WebSocketEndpoint {
                    endpoint_type: "spot".to_string(),
                    url: "wss://api.gateio.ws/ws/v4/".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    max_connections: Some(100),
                },
            ],
            rest_api_endpoints: vec![
                RestApiEndpoint {
                    endpoint_type: "spot".to_string(),
                    base_url: "https://api.gateio.ws".to_string(),
                    priority: 0,
                    regions: vec!["global".to_string()],
                    timeout_ms: 5000,
                },
            ],
            api_credentials: None,
            feature_flags: ExchangeFeatureFlags {
                supports_spot_trading: true,
                supports_futures_trading: true,
                supports_options_trading: false,
                supports_margin_trading: true,
                supports_lending: true,
                supports_staking: false,
                has_advanced_orders: true,
                has_websocket_auth: true,
            },
            rate_limits: RateLimitConfig {
                requests_per_minute: 900,
                websocket_messages_per_second: 20,
                order_burst_limit: 20,
                weight_based_limits: HashMap::new(),
            },
            connection_config: ConnectionConfig::default(),
            testnet_mode: false,
        }));

        // Coinbase Pro 配置模板
        self.register_exchange("coinbase", Box::new(|| DynamicExchangeConfig {
            exchange_id: "coinbase".to_string(),
            display_name: "Coinbase Advanced Trade".to_string(),
            websocket_endpoints: vec![
                WebSocketEndpoint {
                    endpoint_type: "advanced".to_string(),
                    url: "wss://advanced-trade-ws.coinbase.com".to_string(),
                    priority: 0,
                    regions: vec!["us".to_string()],
                    max_connections: Some(100),
                },
            ],
            rest_api_endpoints: vec![
                RestApiEndpoint {
                    endpoint_type: "advanced".to_string(),
                    base_url: "https://api.coinbase.com".to_string(),
                    priority: 0,
                    regions: vec!["us".to_string()],
                    timeout_ms: 5000,
                },
            ],
            api_credentials: None,
            feature_flags: ExchangeFeatureFlags {
                supports_spot_trading: true,
                supports_futures_trading: false,
                supports_options_trading: false,
                supports_margin_trading: false,
                supports_lending: false,
                supports_staking: true,
                has_advanced_orders: true,
                has_websocket_auth: true,
            },
            rate_limits: RateLimitConfig {
                requests_per_minute: 300,
                websocket_messages_per_second: 10,
                order_burst_limit: 50,
                weight_based_limits: HashMap::new(),
            },
            connection_config: ConnectionConfig::default(),
            testnet_mode: false,
        }));

        // Kraken 配置模板
        self.register_exchange("kraken", Box::new(|| DynamicExchangeConfig {
            exchange_id: "kraken".to_string(),
            display_name: "Kraken".to_string(),
            websocket_endpoints: vec![
                WebSocketEndpoint {
                    endpoint_type: "public".to_string(),
                    url: "wss://ws.kraken.com".to_string(),
                    priority: 0,
                    regions: vec!["us".to_string(), "eu".to_string()],
                    max_connections: Some(50),
                },
                WebSocketEndpoint {
                    endpoint_type: "auth".to_string(),
                    url: "wss://ws-auth.kraken.com".to_string(),
                    priority: 1,
                    regions: vec!["us".to_string(), "eu".to_string()],
                    max_connections: Some(25),
                },
            ],
            rest_api_endpoints: vec![
                RestApiEndpoint {
                    endpoint_type: "public".to_string(),
                    base_url: "https://api.kraken.com".to_string(),
                    priority: 0,
                    regions: vec!["us".to_string(), "eu".to_string()],
                    timeout_ms: 5000,
                },
            ],
            api_credentials: None,
            feature_flags: ExchangeFeatureFlags {
                supports_spot_trading: true,
                supports_futures_trading: true,
                supports_options_trading: false,
                supports_margin_trading: true,
                supports_lending: false,
                supports_staking: true,
                has_advanced_orders: true,
                has_websocket_auth: true,
            },
            rate_limits: RateLimitConfig {
                requests_per_minute: 120,
                websocket_messages_per_second: 50,
                order_burst_limit: 15,
                weight_based_limits: HashMap::new(),
            },
            connection_config: ConnectionConfig::default(),
            testnet_mode: false,
        }));
    }

    /// 注册单个交易所配置模板
    fn register_exchange(&mut self, exchange_id: &str, template_fn: Box<dyn Fn() -> DynamicExchangeConfig + Send + Sync>) {
        self.exchange_templates.insert(exchange_id.to_string(), template_fn);
    }

    /// 获取所有支持的区域列表
    pub fn get_supported_regions() -> Vec<&'static str> {
        vec!["global", "us", "eu", "asia", "japan", "singapore"]
    }

    /// 获取交易所的区域特定配置
    pub fn get_regional_config(&self, exchange_id: &str, region: &str) -> QingxiResult<DynamicExchangeConfig> {
        let mut config = self.get_default_config(exchange_id)?;
        
        // 根据区域调整配置
        match (exchange_id, region) {
            ("binance", "us") => {
                // Binance US 特定配置
                config.display_name = "Binance US".to_string();
                config.websocket_endpoints[0].url = "wss://stream.binance.us:9443/ws".to_string();
                config.rest_api_endpoints[0].base_url = "https://api.binance.us".to_string();
                config.feature_flags.supports_futures_trading = false;
            },
            ("bybit", "testnet") => {
                // Bybit Testnet 配置
                config.testnet_mode = true;
                for endpoint in &mut config.rest_api_endpoints {
                    if endpoint.endpoint_type == "unified" {
                        endpoint.base_url = "https://api-testnet.bybit.com".to_string();
                    }
                }
            },
            _ => {
                // 默认区域配置已经在模板中
            }
        }
        
        Ok(config)
    }
}

impl DefaultConfigProvider for StandardExchangeConfigProvider {
    fn get_default_config(&self, exchange_id: &str) -> QingxiResult<DynamicExchangeConfig> {
        self.exchange_templates
            .get(exchange_id)
            .map(|template| template())
            .ok_or_else(|| QingxiError::config(format!("Unsupported exchange: {}", exchange_id)))
    }

    fn list_supported_exchanges(&self) -> Vec<String> {
        self.exchange_templates.keys().cloned().collect()
    }
}

impl Default for StandardExchangeConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

