//! # 动态交易所配置管理模块 (Dynamic Exchange Configuration Module)
//! 
//! PHASE 4: 消除所有硬编码，实现完全动态配置系统
//! 提供运行时交易所配置发现和动态切换能力

use crate::production_error_handling::{QingxiResult, QingxiError, EnvVar};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{info, warn, debug, error};
use serde::{Deserialize, Serialize};

/// 动态交易所配置管理器
/// 完全消除硬编码，支持运行时配置变更
pub struct DynamicExchangeConfigManager {
    /// 当前活跃的交易所配置
    active_configs: Arc<RwLock<HashMap<String, Arc<DynamicExchangeConfig>>>>,
    /// 配置变更监听器
    config_watchers: Arc<RwLock<Vec<Box<dyn ConfigChangeListener + Send + Sync>>>>,
    /// 默认配置提供者
    default_provider: Arc<dyn DefaultConfigProvider + Send + Sync>,
}

/// 动态交易所配置
/// 支持运行时配置变更和验证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicExchangeConfig {
    pub exchange_id: String,
    pub display_name: String,
    pub websocket_endpoints: Vec<WebSocketEndpoint>,
    pub rest_api_endpoints: Vec<RestApiEndpoint>,
    pub api_credentials: Option<ApiCredentials>,
    pub feature_flags: ExchangeFeatureFlags,
    pub rate_limits: RateLimitConfig,
    pub connection_config: ConnectionConfig,
    pub testnet_mode: bool,
}

/// WebSocket端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketEndpoint {
    pub endpoint_type: String, // "spot", "futures", "options"
    pub url: String,
    pub priority: u8, // 0=highest, 255=lowest
    pub regions: Vec<String>, // ["global", "us", "asia"]
    pub max_connections: Option<u32>,
}

/// REST API端点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestApiEndpoint {
    pub endpoint_type: String,
    pub base_url: String,
    pub priority: u8,
    pub regions: Vec<String>,
    pub timeout_ms: u64,
}

/// API凭证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCredentials {
    pub api_key: String,
    pub secret: String,
    pub passphrase: Option<String>, // For OKX
    pub sandbox: bool,
}

/// 交易所功能标志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeFeatureFlags {
    pub supports_spot_trading: bool,
    pub supports_futures_trading: bool,
    pub supports_options_trading: bool,
    pub supports_margin_trading: bool,
    pub supports_lending: bool,
    pub supports_staking: bool,
    pub has_advanced_orders: bool,
    pub has_websocket_auth: bool,
}

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub websocket_messages_per_second: u32,
    pub order_burst_limit: u32,
    pub weight_based_limits: HashMap<String, u32>,
}

/// 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub connect_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub keepalive_interval_s: u64,
    pub reconnect_attempts: u32,
    pub reconnect_delay_ms: u64,
    pub compression_enabled: bool,
}

/// 配置变更监听器
pub trait ConfigChangeListener {
    fn on_config_changed(&self, exchange_id: &str, config: &DynamicExchangeConfig) -> QingxiResult<()>;
    fn on_config_removed(&self, exchange_id: &str) -> QingxiResult<()>;
}

/// 默认配置提供者
pub trait DefaultConfigProvider {
    fn get_default_config(&self, exchange_id: &str) -> QingxiResult<DynamicExchangeConfig>;
    fn list_supported_exchanges(&self) -> Vec<String>;
}

impl DynamicExchangeConfigManager {
    /// 创建新的动态配置管理器
    pub fn new(default_provider: Arc<dyn DefaultConfigProvider + Send + Sync>) -> Self {
        Self {
            active_configs: Arc::new(RwLock::new(HashMap::new())),
            config_watchers: Arc::new(RwLock::new(Vec::new())),
            default_provider,
        }
    }

    /// 从环境变量和默认配置初始化
    pub fn initialize_from_environment(&self) -> QingxiResult<()> {
        info!("🔧 Initializing dynamic exchange configurations from environment");

        let supported_exchanges = self.default_provider.list_supported_exchanges();
        let mut loaded_count = 0;

        for exchange_id in supported_exchanges {
            match self.load_exchange_config(&exchange_id) {
                Ok(config) => {
                    self.set_config(exchange_id.clone(), config)?;
                    loaded_count += 1;
                    debug!("✅ Loaded dynamic config for {}", exchange_id);
                },
                Err(e) => {
                    warn!("⚠️ Failed to load config for {}: {}", exchange_id, e);
                    // 尝试使用默认配置作为fallback
                    if let Ok(default_config) = self.default_provider.get_default_config(&exchange_id) {
                        self.set_config(exchange_id.clone(), default_config)?;
                        loaded_count += 1;
                        info!("✅ Used default config for {}", exchange_id);
                    }
                }
            }
        }

        if loaded_count == 0 {
            return Err(QingxiError::config("No exchange configurations could be loaded"));
        }

        info!("✅ Dynamic exchange configuration initialized: {} exchanges loaded", loaded_count);
        Ok(())
    }

    /// 加载单个交易所配置
    fn load_exchange_config(&self, exchange_id: &str) -> QingxiResult<DynamicExchangeConfig> {
        let exchange_upper = exchange_id.to_uppercase();

        // 获取基础配置
        let mut default_config = self.default_provider.get_default_config(exchange_id)?;

        // 从环境变量覆盖配置
        self.override_config_from_env(&mut default_config, &exchange_upper)?;

        // 验证配置
        self.validate_config(&default_config)?;

        Ok(default_config)
    }

    /// 从环境变量覆盖配置
    fn override_config_from_env(&self, config: &mut DynamicExchangeConfig, exchange_upper: &str) -> QingxiResult<()> {
        // 覆盖WebSocket端点
        if let Ok(ws_url) = EnvVar::get_string(&format!("QINGXI_{}_WS_URL", exchange_upper)) {
            if let Some(primary_ws) = config.websocket_endpoints.get_mut(0) {
                primary_ws.url = ws_url;
            }
        }

        // 覆盖REST API端点
        if let Ok(api_url) = EnvVar::get_string(&format!("QINGXI_{}_API_URL", exchange_upper)) {
            if let Some(primary_api) = config.rest_api_endpoints.get_mut(0) {
                primary_api.base_url = api_url;
            }
        }

        // 设置API凭证
        let api_key = EnvVar::get_string(&format!("QINGXI_{}_API_KEY", exchange_upper)).ok();
        let secret = EnvVar::get_string(&format!("QINGXI_{}_SECRET", exchange_upper)).ok();
        let passphrase = EnvVar::get_string(&format!("QINGXI_{}_PASSPHRASE", exchange_upper)).ok();

        if let (Some(key), Some(sec)) = (api_key, secret) {
            // 验证不是占位符
            if !key.contains("your_") && !key.contains("PLACEHOLDER") && 
               !sec.contains("your_") && !sec.contains("PLACEHOLDER") {
                config.api_credentials = Some(ApiCredentials {
                    api_key: key,
                    secret: sec,
                    passphrase,
                    sandbox: EnvVar::get_bool(&format!("QINGXI_{}_TESTNET", exchange_upper)).unwrap_or(false),
                });
            } else {
                warn!("⚠️ API credentials for {} appear to be placeholders, skipping", config.exchange_id);
            }
        }

        // 覆盖testnet模式
        config.testnet_mode = EnvVar::get_bool(&format!("QINGXI_{}_TESTNET", exchange_upper)).unwrap_or(false);

        // 覆盖连接配置
        if let Ok(timeout) = EnvVar::get_parsed::<u64, String>(format!("QINGXI_{}_TIMEOUT_MS", exchange_upper)) {
            config.connection_config.connect_timeout_ms = timeout;
            config.connection_config.read_timeout_ms = timeout;
        }

        Ok(())
    }

    /// 验证配置
    fn validate_config(&self, config: &DynamicExchangeConfig) -> QingxiResult<()> {
        use crate::production_error_handling::ConfigValidator;

        // 验证WebSocket端点
        for endpoint in &config.websocket_endpoints {
            ConfigValidator::validate_url(&endpoint.url)?;
        }

        // 验证REST API端点
        for endpoint in &config.rest_api_endpoints {
            ConfigValidator::validate_url(&endpoint.base_url)?;
        }

        // 验证API凭证
        if let Some(ref creds) = config.api_credentials {
            ConfigValidator::validate_api_key(&creds.api_key, &config.exchange_id)?;
        }

        // 验证速率限制
        if config.rate_limits.requests_per_minute == 0 {
            return Err(QingxiError::config("Rate limit cannot be zero"));
        }

        Ok(())
    }

    /// 设置配置
    pub fn set_config(&self, exchange_id: String, config: DynamicExchangeConfig) -> QingxiResult<()> {
        let config_arc = Arc::new(config);
        
        {
            let mut configs = self.active_configs.write()
                .map_err(|_| QingxiError::config("Failed to acquire config write lock"))?;
            configs.insert(exchange_id.clone(), config_arc.clone());
        }

        // 通知监听器
        self.notify_config_changed(&exchange_id, &config_arc)?;

        info!("✅ Configuration updated for exchange: {}", exchange_id);
        Ok(())
    }

    /// 获取配置
    pub fn get_config(&self, exchange_id: &str) -> QingxiResult<Arc<DynamicExchangeConfig>> {
        let configs = self.active_configs.read()
            .map_err(|_| QingxiError::config("Failed to acquire config read lock"))?;
        
        configs.get(exchange_id)
            .cloned()
            .ok_or_else(|| QingxiError::config(format!("No configuration found for exchange: {}", exchange_id)))
    }

    /// 列出所有配置的交易所
    pub fn list_configured_exchanges(&self) -> QingxiResult<Vec<String>> {
        let configs = self.active_configs.read()
            .map_err(|_| QingxiError::config("Failed to acquire config read lock"))?;
        
        Ok(configs.keys().cloned().collect())
    }

    /// 添加配置变更监听器
    pub fn add_config_listener(&self, listener: Box<dyn ConfigChangeListener + Send + Sync>) -> QingxiResult<()> {
        let mut watchers = self.config_watchers.write()
            .map_err(|_| QingxiError::config("Failed to acquire watchers write lock"))?;
        watchers.push(listener);
        Ok(())
    }

    /// 通知配置变更
    fn notify_config_changed(&self, exchange_id: &str, config: &DynamicExchangeConfig) -> QingxiResult<()> {
        let watchers = self.config_watchers.read()
            .map_err(|_| QingxiError::config("Failed to acquire watchers read lock"))?;
        
        for watcher in watchers.iter() {
            if let Err(e) = watcher.on_config_changed(exchange_id, config) {
                error!("Config change notification failed for {}: {}", exchange_id, e);
            }
        }
        
        Ok(())
    }

    /// 热重载配置
    pub fn reload_config(&self, exchange_id: &str) -> QingxiResult<()> {
        info!("🔄 Hot reloading configuration for {}", exchange_id);
        
        let new_config = self.load_exchange_config(exchange_id)?;
        self.set_config(exchange_id.to_string(), new_config)?;
        
        info!("✅ Configuration hot reloaded for {}", exchange_id);
        Ok(())
    }

    /// 生成配置摘要
    pub fn generate_config_summary(&self) -> QingxiResult<String> {
        let configs = self.active_configs.read()
            .map_err(|_| QingxiError::config("Failed to acquire config read lock"))?;
        
        let mut summary = String::from("📊 Dynamic Exchange Configuration Summary\n");
        summary.push_str(&format!("Total configured exchanges: {}\n\n", configs.len()));
        
        for (exchange_id, config) in configs.iter() {
            summary.push_str(&format!("🏛️ {}\n", exchange_id.to_uppercase()));
            summary.push_str(&format!("  Name: {}\n", config.display_name));
            summary.push_str(&format!("  WebSocket Endpoints: {}\n", config.websocket_endpoints.len()));
            summary.push_str(&format!("  REST API Endpoints: {}\n", config.rest_api_endpoints.len()));
            summary.push_str(&format!("  Has Credentials: {}\n", config.api_credentials.is_some()));
            summary.push_str(&format!("  Testnet Mode: {}\n", config.testnet_mode));
            summary.push_str(&format!("  Rate Limit: {} req/min\n", config.rate_limits.requests_per_minute));
            summary.push_str("\n");
        }
        
        Ok(summary)
    }
}

impl Default for ExchangeFeatureFlags {
    fn default() -> Self {
        Self {
            supports_spot_trading: true,
            supports_futures_trading: false,
            supports_options_trading: false,
            supports_margin_trading: false,
            supports_lending: false,
            supports_staking: false,
            has_advanced_orders: false,
            has_websocket_auth: false,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 1200,
            websocket_messages_per_second: 100,
            order_burst_limit: 10,
            weight_based_limits: HashMap::new(),
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout_ms: 5000,
            read_timeout_ms: 30000,
            keepalive_interval_s: 30,
            reconnect_attempts: 5,
            reconnect_delay_ms: 1000,
            compression_enabled: true,
        }
    }
}

