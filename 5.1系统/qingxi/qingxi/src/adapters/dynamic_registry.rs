#![allow(dead_code)]
//! # 动态适配器注册系统
//!
//! 提供可扩展的交易所适配器管理能力，支持动态注册、配置验证和模板生成。

use crate::{
    adapters::{ExchangeAdapter, binance::BinanceAdapter, huobi::HuobiAdapter, okx::OkxAdapter, bybit::BybitAdapter, gateio::GateioAdapter},
    errors::MarketDataError,
    types::MarketSourceConfig,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// 适配器元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterMetadata {
    pub exchange_id: String,
    pub display_name: String,
    pub version: String,
    pub supported_channels: Vec<String>,
    pub supported_symbols: Vec<String>,
    pub required_config_fields: Vec<ConfigField>,
    pub optional_config_fields: Vec<ConfigField>,
    pub default_config: serde_json::Value,
    pub websocket_url_template: String,
    pub rest_api_url_template: String,
    pub rate_limits: RateLimits,
    pub features: AdapterFeatures,
}

/// 配置字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigField {
    pub name: String,
    pub field_type: ConfigFieldType,
    pub description: String,
    pub validation_rules: Vec<ValidationRule>,
    pub default_value: Option<String>,
}

/// 配置字段类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFieldType {
    String,
    Integer,
    Float,
    Boolean,
    Url,
    ApiKey,
    Symbol,
    Channel,
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    Required,
    ValidUrl,
    ApiKeyFormat,
    PositiveInteger,
    InRange(i64, i64),
    OneOf(Vec<String>),
    Regex(String),
}

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub websocket_connections: u32,
}

/// 适配器特性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterFeatures {
    pub supports_heartbeat: bool,
    pub supports_orderbook_snapshot: bool,
    pub supports_trades: bool,
    pub supports_authentication: bool,
    pub compression_type: Option<CompressionType>,
}

/// 压缩类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Deflate,
}

/// 配置模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub exchange_id: String,
    pub template_content: String,
    pub schema: JsonSchema,
    pub examples: Vec<ConfigExample>,
}

/// JSON Schema定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchema {
    pub schema_type: String,
    pub properties: serde_json::Map<String, serde_json::Value>,
    pub required: Vec<String>,
}

/// 配置示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigExample {
    pub name: String,
    pub description: String,
    pub config: serde_json::Value,
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    MissingRequiredField(String),
    EmptyRequiredField(String),
    InvalidUrl(String),
    InvalidApiKey(String),
    InvalidInteger(String),
    OutOfRange(String, i64, i64),
    InvalidOption(String, Vec<String>),
    PatternMismatch(String, String),
    InvalidPattern(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingRequiredField(field) => {
                write!(f, "Required field '{}' is missing", field)
            }
            ValidationError::EmptyRequiredField(field) => {
                write!(f, "Required field '{}' cannot be empty", field)
            }
            ValidationError::InvalidUrl(field) => {
                write!(f, "Field '{}' must be a valid URL", field)
            }
            ValidationError::InvalidApiKey(field) => {
                write!(f, "Field '{}' must be a valid API key (minimum 16 characters)", field)
            }
            ValidationError::InvalidInteger(field) => {
                write!(f, "Field '{}' must be a positive integer", field)
            }
            ValidationError::OutOfRange(field, min, max) => {
                write!(f, "Field '{}' must be between {} and {}", field, min, max)
            }
            ValidationError::InvalidOption(field, options) => {
                write!(f, "Field '{}' must be one of: {:?}", field, options)
            }
            ValidationError::PatternMismatch(field, pattern) => {
                write!(f, "Field '{}' does not match pattern: {}", field, pattern)
            }
            ValidationError::InvalidPattern(pattern) => {
                write!(f, "Invalid regex pattern: {}", pattern)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// 适配器工厂函数类型
pub type AdapterFactory = Box<dyn Fn(&MarketSourceConfig) -> Result<Arc<dyn ExchangeAdapter>, MarketDataError> + Send + Sync>;

/// 已注册的适配器
#[derive(Clone)]
pub struct RegisteredAdapter {
    pub metadata: AdapterMetadata,
    pub factory: Arc<AdapterFactory>,
    pub config_template: ConfigTemplate,
}

/// 动态适配器注册表
pub struct DynamicAdapterRegistry {
    adapters: Arc<RwLock<HashMap<String, RegisteredAdapter>>>,
}

impl DynamicAdapterRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        let registry = Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // 注册内置适配器
        tokio::spawn({
            let registry = registry.clone();
            async move {
                if let Err(e) = registry.register_builtin_adapters().await {
                    error!("Failed to register builtin adapters: {}", e);
                }
            }
        });
        
        registry
    }
    
    /// 注册适配器
    pub async fn register_adapter(
        &self,
        metadata: AdapterMetadata,
        factory: AdapterFactory,
    ) -> Result<(), MarketDataError> {
        // 验证元数据
        self.validate_metadata(&metadata).await?;
        
        // 创建配置模板
        let template = self.create_template(&metadata).await?;
        
        let registered = RegisteredAdapter {
            metadata: metadata.clone(),
            factory: Arc::new(factory),
            config_template: template,
        };
        
        let mut adapters = self.adapters.write().await;
        adapters.insert(metadata.exchange_id.clone(), registered);
        
        info!("Registered adapter: {}", metadata.exchange_id);
        Ok(())
    }
    
    /// 根据配置创建适配器实例
    pub async fn create_adapter_with_config(
        &self,
        exchange_id: &str,
        config: &MarketSourceConfig,
    ) -> Result<Arc<dyn ExchangeAdapter>, MarketDataError> {
        let adapters = self.adapters.read().await;
        let registered = adapters.get(exchange_id)
            .ok_or_else(|| MarketDataError::Configuration(
                format!("Unknown exchange adapter: {}", exchange_id)
            ))?;
            
        // 验证配置
        self.validate_config(&registered.metadata, config).await?;
        
        // 创建适配器实例
        (registered.factory)(config)
    }
    
    /// 创建适配器实例（使用默认配置）
    pub async fn create_adapter(&self, exchange_id: &str) -> Option<Arc<dyn ExchangeAdapter>> {
        let adapters = self.adapters.read().await;
        if let Some(registered) = adapters.get(exchange_id) {
            // 使用默认配置创建适配器
            let default_config = MarketSourceConfig::new(exchange_id, "");
            match (registered.factory)(&default_config) {
                Ok(adapter) => Some(adapter),
                Err(e) => {
                    warn!("Failed to create adapter for {}: {}", exchange_id, e);
                    None
                }
            }
        } else {
            None
        }
    }
    
    /// 验证配置（公共方法）
    pub async fn validate_config_public(&self, exchange_id: &str, config: &MarketSourceConfig) -> Result<(), MarketDataError> {
        let adapters = self.adapters.read().await;
        let registered = adapters.get(exchange_id)
            .ok_or_else(|| MarketDataError::Configuration(
                format!("Unknown exchange adapter: {}", exchange_id)
            ))?;
        
        self.validate_config(&registered.metadata, config).await
    }
    
    /// 获取适配器元数据
    pub async fn get_adapter_metadata(&self, exchange_id: &str) -> Option<AdapterMetadata> {
        let adapters = self.adapters.read().await;
        adapters.get(exchange_id).map(|a| a.metadata.clone())
    }
    
    /// 获取所有注册的适配器
    pub async fn list_adapters(&self) -> Vec<AdapterMetadata> {
        let adapters = self.adapters.read().await;
        adapters.values().map(|a| a.metadata.clone()).collect()
    }
    
    /// 获取所有已注册的交易所ID
    pub async fn registered_exchanges(&self) -> Vec<String> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }
    
    /// 生成配置模板
    pub async fn generate_config_template(&self, exchange_id: &str) -> Result<ConfigTemplate, MarketDataError> {
        let adapters = self.adapters.read().await;
        let registered = adapters.get(exchange_id)
            .ok_or_else(|| MarketDataError::Configuration(
                format!("Unknown exchange adapter: {}", exchange_id)
            ))?;
        Ok(registered.config_template.clone())
    }
    
    /// 检查适配器是否已注册
    pub async fn is_registered(&self, exchange_id: &str) -> bool {
        let adapters = self.adapters.read().await;
        adapters.contains_key(exchange_id)
    }
    
    /// 注册内置适配器
    async fn register_builtin_adapters(&self) -> Result<(), MarketDataError> {
        // 注册 Binance 适配器
        let binance_metadata = AdapterMetadata {
            exchange_id: "binance".to_string(),
            display_name: "Binance".to_string(),
            version: "1.0.0".to_string(),
            supported_channels: vec!["orderbook".to_string(), "trades".to_string()],
            supported_symbols: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            required_config_fields: vec![
                ConfigField {
                    name: "websocket_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "WebSocket连接URL".to_string(),
                    validation_rules: vec![ValidationRule::Required, ValidationRule::ValidUrl],
                    default_value: Some("wss://stream.binance.com:9443/ws".to_string()),
                },
            ],
            optional_config_fields: vec![
                ConfigField {
                    name: "rest_api_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "REST API URL".to_string(),
                    validation_rules: vec![ValidationRule::ValidUrl],
                    default_value: Some("https://api.binance.com/api/v3".to_string()),
                },
            ],
            default_config: serde_json::json!({
                "exchange_id": "binance",
                "enabled": true,
                "websocket_url": "wss://stream.binance.com:9443/ws",
                "rest_api_url": "https://api.binance.com/api/v3"
            }),
            websocket_url_template: "wss://stream.binance.com:9443/ws".to_string(),
            rest_api_url_template: "https://api.binance.com/api/v3".to_string(),
            rate_limits: RateLimits {
                requests_per_second: 10,
                burst_size: 100,
                websocket_connections: 5,
            },
            features: AdapterFeatures {
                supports_heartbeat: false,
                supports_orderbook_snapshot: true,
                supports_trades: true,
                supports_authentication: false,
                compression_type: Some(CompressionType::None),
            },
        };
        
        let binance_factory: AdapterFactory = Box::new(|config| {
            Ok(Arc::new(BinanceAdapter::new_with_config(config)))
        });
        
        self.register_adapter(binance_metadata, binance_factory).await?;
        
        // 注册 Huobi 适配器
        let huobi_metadata = AdapterMetadata {
            exchange_id: "huobi".to_string(),
            display_name: "Huobi".to_string(),
            version: "1.0.0".to_string(),
            supported_channels: vec!["orderbook".to_string(), "trades".to_string()],
            supported_symbols: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            required_config_fields: vec![
                ConfigField {
                    name: "websocket_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "WebSocket连接URL".to_string(),
                    validation_rules: vec![ValidationRule::Required, ValidationRule::ValidUrl],
                    default_value: Some("wss://api.huobi.pro/ws".to_string()),
                },
            ],
            optional_config_fields: vec![
                ConfigField {
                    name: "rest_api_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "REST API URL".to_string(),
                    validation_rules: vec![ValidationRule::ValidUrl],
                    default_value: Some("https://api.huobi.pro".to_string()),
                },
            ],
            default_config: serde_json::json!({
                "exchange_id": "huobi",
                "enabled": true,
                "websocket_url": "wss://api.huobi.pro/ws",
                "rest_api_url": "https://api.huobi.pro"
            }),
            websocket_url_template: "wss://api.huobi.pro/ws".to_string(),
            rest_api_url_template: "https://api.huobi.pro".to_string(),
            rate_limits: RateLimits {
                requests_per_second: 10,
                burst_size: 100,
                websocket_connections: 5,
            },
            features: AdapterFeatures {
                supports_heartbeat: true,
                supports_orderbook_snapshot: true,
                supports_trades: true,
                supports_authentication: false,
                compression_type: Some(CompressionType::Gzip),
            },
        };
        
        let huobi_factory: AdapterFactory = Box::new(|config| {
            Ok(Arc::new(HuobiAdapter::new_with_config(config)))
        });
        
        self.register_adapter(huobi_metadata, huobi_factory).await?;
        
        // 注册 OKX 适配器
        let okx_metadata = AdapterMetadata {
            exchange_id: "okx".to_string(),
            display_name: "OKX".to_string(),
            version: "1.0.0".to_string(),
            supported_channels: vec!["orderbook".to_string(), "trades".to_string()],
            supported_symbols: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            required_config_fields: vec![
                ConfigField {
                    name: "websocket_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "WebSocket连接URL".to_string(),
                    validation_rules: vec![ValidationRule::Required, ValidationRule::ValidUrl],
                    default_value: Some("wss://ws.okx.com:8443/ws/v5/public".to_string()),
                },
            ],
            optional_config_fields: vec![
                ConfigField {
                    name: "rest_api_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "REST API URL".to_string(),
                    validation_rules: vec![ValidationRule::ValidUrl],
                    default_value: Some("https://www.okx.com/api/v5".to_string()),
                },
            ],
            default_config: serde_json::json!({
                "exchange_id": "okx",
                "enabled": true,
                "websocket_url": "wss://ws.okx.com:8443/ws/v5/public",
                "rest_api_url": "https://www.okx.com/api/v5"
            }),
            websocket_url_template: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            rest_api_url_template: "https://www.okx.com/api/v5".to_string(),
            rate_limits: RateLimits {
                requests_per_second: 20,
                burst_size: 200,
                websocket_connections: 5,
            },
            features: AdapterFeatures {
                supports_heartbeat: true,
                supports_orderbook_snapshot: true,
                supports_trades: true,
                supports_authentication: false,
                compression_type: Some(CompressionType::None),
            },
        };
        
        let okx_factory: AdapterFactory = Box::new(|config| {
            Ok(Arc::new(OkxAdapter::new_with_config(config)))
        });
        
        self.register_adapter(okx_metadata, okx_factory).await?;
        
        // 注册 Bybit 适配器
        let bybit_metadata = AdapterMetadata {
            exchange_id: "bybit".to_string(),
            display_name: "Bybit".to_string(),
            version: "1.0.0".to_string(),
            supported_channels: vec!["orderbook".to_string(), "trades".to_string()],
            supported_symbols: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            required_config_fields: vec![
                ConfigField {
                    name: "websocket_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "WebSocket连接URL".to_string(),
                    validation_rules: vec![ValidationRule::Required, ValidationRule::ValidUrl],
                    default_value: Some("wss://stream.bybit.com/v5/public/spot".to_string()),
                },
            ],
            optional_config_fields: vec![
                ConfigField {
                    name: "rest_api_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "REST API URL".to_string(),
                    validation_rules: vec![ValidationRule::ValidUrl],
                    default_value: Some("https://api.bybit.com".to_string()),
                },
            ],
            default_config: serde_json::json!({
                "exchange_id": "bybit",
                "enabled": true,
                "websocket_url": "wss://stream.bybit.com/v5/public/spot",
                "rest_api_url": "https://api.bybit.com"
            }),
            websocket_url_template: "wss://stream.bybit.com/v5/public/spot".to_string(),
            rest_api_url_template: "https://api.bybit.com".to_string(),
            rate_limits: RateLimits {
                requests_per_second: 20,
                burst_size: 200,
                websocket_connections: 5,
            },
            features: AdapterFeatures {
                supports_heartbeat: true,
                supports_orderbook_snapshot: true,
                supports_trades: true,
                supports_authentication: false,
                compression_type: Some(CompressionType::None),
            },
        };
        
        let bybit_factory: AdapterFactory = Box::new(|config| {
            Ok(Arc::new(BybitAdapter::new_with_config(config)))
        });
        
        self.register_adapter(bybit_metadata, bybit_factory).await?;
        
        // 注册 Gate.io 适配器
        let gateio_metadata = AdapterMetadata {
            exchange_id: "gateio".to_string(),
            display_name: "Gate.io".to_string(),
            version: "1.0.0".to_string(),
            supported_channels: vec!["orderbook".to_string(), "trades".to_string()],
            supported_symbols: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            required_config_fields: vec![
                ConfigField {
                    name: "websocket_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "WebSocket连接URL".to_string(),
                    validation_rules: vec![ValidationRule::Required, ValidationRule::ValidUrl],
                    default_value: Some("wss://api.gateio.ws/ws/v4/".to_string()),
                },
            ],
            optional_config_fields: vec![
                ConfigField {
                    name: "rest_api_url".to_string(),
                    field_type: ConfigFieldType::Url,
                    description: "REST API URL".to_string(),
                    validation_rules: vec![ValidationRule::ValidUrl],
                    default_value: Some("https://api.gateio.ws".to_string()),
                },
            ],
            default_config: serde_json::json!({
                "exchange_id": "gateio",
                "enabled": true,
                "websocket_url": "wss://api.gateio.ws/ws/v4/",
                "rest_api_url": "https://api.gateio.ws"
            }),
            websocket_url_template: "wss://api.gateio.ws/ws/v4/".to_string(),
            rest_api_url_template: "https://api.gateio.ws".to_string(),
            rate_limits: RateLimits {
                requests_per_second: 10,
                burst_size: 100,
                websocket_connections: 5,
            },
            features: AdapterFeatures {
                supports_heartbeat: true,
                supports_orderbook_snapshot: true,
                supports_trades: true,
                supports_authentication: false,
                compression_type: Some(CompressionType::None),
            },
        };
        
        let gateio_factory: AdapterFactory = Box::new(|config| {
            Ok(Arc::new(GateioAdapter::new_with_config(config)))
        });
        
        self.register_adapter(gateio_metadata, gateio_factory).await?;
        
        info!("Successfully registered {} builtin adapters", 5);
        Ok(())
    }
    
    /// 验证元数据
    async fn validate_metadata(&self, metadata: &AdapterMetadata) -> Result<(), MarketDataError> {
        if metadata.exchange_id.is_empty() {
            return Err(MarketDataError::Configuration("Exchange ID cannot be empty".to_string()));
        }
        
        if metadata.display_name.is_empty() {
            return Err(MarketDataError::Configuration("Display name cannot be empty".to_string()));
        }
        
        if metadata.supported_channels.is_empty() {
            return Err(MarketDataError::Configuration("At least one supported channel must be specified".to_string()));
        }
        
        Ok(())
    }
    
    /// 验证配置
    async fn validate_config(
        &self,
        metadata: &AdapterMetadata,
        config: &MarketSourceConfig,
    ) -> Result<(), MarketDataError> {
        // 验证必需字段
        for field in &metadata.required_config_fields {
            let value = match field.name.as_str() {
                "websocket_url" => config.get_websocket_url(),
                "rest_api_url" => config.get_rest_api_url().unwrap_or(""),
                _ => {
                    warn!("Unknown required field: {}", field.name);
                    continue;
                }
            };
            
            if let Err(e) = self.validate_field_with_rules(&field.name, value, &field.validation_rules) {
                return Err(MarketDataError::Configuration(e.to_string()));
            }
        }
        
        // 验证可选字段（如果存在）
        for field in &metadata.optional_config_fields {
            let value = match field.name.as_str() {
                "websocket_url" => config.get_websocket_url(),
                "rest_api_url" => config.get_rest_api_url().unwrap_or(""),
                _ => continue,
            };
            
            if !value.is_empty() {
                if let Err(e) = self.validate_field_with_rules(&field.name, value, &field.validation_rules) {
                    return Err(MarketDataError::Configuration(e.to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    /// 验证字段值
    fn validate_field_with_rules(
        &self,
        field_name: &str,
        value: &str,
        rules: &[ValidationRule],
    ) -> Result<(), ValidationError> {
        for rule in rules {
            match rule {
                ValidationRule::Required => {
                    if value.is_empty() {
                        return Err(ValidationError::EmptyRequiredField(field_name.to_string()));
                    }
                }
                ValidationRule::ValidUrl => {
                    if !value.starts_with("http://") && !value.starts_with("https://") 
                        && !value.starts_with("wss://") && !value.starts_with("ws://") {
                        return Err(ValidationError::InvalidUrl(field_name.to_string()));
                    }
                }
                ValidationRule::ApiKeyFormat => {
                    if value.len() < 16 {
                        return Err(ValidationError::InvalidApiKey(field_name.to_string()));
                    }
                }
                ValidationRule::PositiveInteger => {
                    if value.parse::<i64>().map_or(true, |n| n <= 0) {
                        return Err(ValidationError::InvalidInteger(field_name.to_string()));
                    }
                }
                ValidationRule::InRange(min, max) => {
                    if let Ok(n) = value.parse::<i64>() {
                        if n < *min || n > *max {
                            return Err(ValidationError::OutOfRange(field_name.to_string(), *min, *max));
                        }
                    }
                }
                ValidationRule::OneOf(options) => {
                    if !options.contains(&value.to_string()) {
                        return Err(ValidationError::InvalidOption(field_name.to_string(), options.clone()));
                    }
                }
                ValidationRule::Regex(pattern) => {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|_| ValidationError::InvalidPattern(pattern.clone()))?;
                    if !regex.is_match(value) {
                        return Err(ValidationError::PatternMismatch(field_name.to_string(), pattern.clone()));
                    }
                }
            }
        }
        Ok(())
    }
    
    /// 创建配置模板
    async fn create_template(&self, metadata: &AdapterMetadata) -> Result<ConfigTemplate, MarketDataError> {
        let template_content = self.generate_template_content(metadata)?;
        let schema = self.generate_json_schema(metadata);
        let examples = self.generate_examples(metadata);
        
        Ok(ConfigTemplate {
            exchange_id: metadata.exchange_id.clone(),
            template_content,
            schema,
            examples,
        })
    }
    
    /// 生成模板内容
    fn generate_template_content(&self, metadata: &AdapterMetadata) -> Result<String, MarketDataError> {
        let mut template = String::new();
        
        template.push_str(&format!("# {} 交易所配置\n", metadata.display_name));
        template.push_str(&format!("# 版本: {}\n\n", metadata.version));
        
        template.push_str(&format!("[[sources]]\n"));
        template.push_str(&format!("exchange_id = \"{}\"\n", metadata.exchange_id));
        template.push_str("enabled = true\n\n");
        
        // 添加必需字段
        for field in &metadata.required_config_fields {
            template.push_str(&format!("# {} (必需)\n", field.description));
            if let Some(default) = &field.default_value {
                template.push_str(&format!("{} = \"{}\"\n", field.name, default));
            } else {
                template.push_str(&format!("{} = \"\"\n", field.name));
            }
            template.push('\n');
        }
        
        // 添加可选字段
        for field in &metadata.optional_config_fields {
            template.push_str(&format!("# {} (可选)\n", field.description));
            if let Some(default) = &field.default_value {
                template.push_str(&format!("{} = \"{}\"\n", field.name, default));
            } else {
                template.push_str(&format!("# {} = \"\"\n", field.name));
            }
            template.push('\n');
        }
        
        // 添加订阅配置
        template.push_str("# 订阅配置\n");
        template.push_str("subscriptions = [\n");
        for channel in &metadata.supported_channels {
            template.push_str("    {\n");
            template.push_str("        symbol = \"BTC/USDT\",\n");
            template.push_str(&format!("        channel = \"{}\"\n", channel));
            template.push_str("    },\n");
        }
        template.push_str("]\n");
        
        Ok(template)
    }
    
    /// 生成JSON Schema
    fn generate_json_schema(&self, metadata: &AdapterMetadata) -> JsonSchema {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        
        // 添加必需字段
        for field in &metadata.required_config_fields {
            properties.insert(field.name.clone(), self.field_to_schema_property(field));
            required.push(field.name.clone());
        }
        
        // 添加可选字段
        for field in &metadata.optional_config_fields {
            properties.insert(field.name.clone(), self.field_to_schema_property(field));
        }
        
        JsonSchema {
            schema_type: "object".to_string(),
            properties,
            required,
        }
    }
    
    /// 字段转换为Schema属性
    fn field_to_schema_property(&self, field: &ConfigField) -> serde_json::Value {
        let mut property = serde_json::Map::new();
        
        match field.field_type {
            ConfigFieldType::String => {
                property.insert("type".to_string(), serde_json::json!("string"));
            },
            ConfigFieldType::Integer => {
                property.insert("type".to_string(), serde_json::json!("integer"));
            },
            ConfigFieldType::Float => {
                property.insert("type".to_string(), serde_json::json!("number"));
            },
            ConfigFieldType::Boolean => {
                property.insert("type".to_string(), serde_json::json!("boolean"));
            },
            ConfigFieldType::Url => {
                property.insert("type".to_string(), serde_json::json!("string"));
                property.insert("format".to_string(), serde_json::json!("uri"));
            },
            ConfigFieldType::ApiKey => {
                property.insert("type".to_string(), serde_json::json!("string"));
                property.insert("minLength".to_string(), serde_json::json!(16));
            },
            _ => {
                property.insert("type".to_string(), serde_json::json!("string"));
            },
        }
        
        property.insert("description".to_string(), serde_json::json!(field.description));
        
        if let Some(default) = &field.default_value {
            property.insert("default".to_string(), serde_json::json!(default));
        }
        
        serde_json::Value::Object(property)
    }
    
    /// 生成配置示例
    fn generate_examples(&self, metadata: &AdapterMetadata) -> Vec<ConfigExample> {
        vec![
            ConfigExample {
                name: "基本配置".to_string(),
                description: format!("{}交易所的基本配置示例", metadata.display_name),
                config: metadata.default_config.clone(),
            }
        ]
    }
}

impl Clone for DynamicAdapterRegistry {
    fn clone(&self) -> Self {
        Self {
            adapters: Arc::clone(&self.adapters),
        }
    }
}

impl Default for DynamicAdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
