//! 安全配置管理器 - 消除硬编码问题
//! 提供类型安全的配置访问和验证

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Context, Result};
use tracing::info;

/// 安全配置管理器
pub struct SafeConfigManager {
    config: QingxiSafeConfig,
    secrets: SecretManager,
}

/// QingXi安全配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QingxiSafeConfig {
    pub exchanges: HashMap<String, ExchangeConfig>,
    pub system: SystemConfig,
    pub trading: TradingConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub api_base_url: String,
    pub ws_url: String,
    pub api_key_ref: String,  // 引用密钥管理器中的键名
    pub secret_key_ref: String,
    pub rate_limit: u32,
    pub timeout_ms: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub http_port: u16,
    pub log_level: String,
    pub max_connections: u32,
    pub thread_pool_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub default_min_profit: f64,
    pub max_position_size: f64,
    pub slippage_threshold: f64,
    pub order_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_port: u16,
    pub health_check_interval_ms: u64,
    pub alert_webhook_url: Option<String>,
}

/// 密钥管理器
pub struct SecretManager {
    secrets: HashMap<String, String>,
}

impl SafeConfigManager {
    /// 从配置文件和环境变量安全加载配置
    pub fn load_from_file(config_path: &str) -> Result<Self> {
        // 加载主配置
        let config_content = std::fs::read_to_string(config_path)
            .context("Failed to read config file")?;
        
        let mut config: QingxiSafeConfig = toml::from_str(&config_content)
            .context("Failed to parse config file")?;

        // 验证配置完整性
        Self::validate_config(&config)?;

        // 加载密钥
        let secrets = SecretManager::load_secrets()?;

        // 应用环境变量覆盖
        Self::apply_env_overrides(&mut config)?;

        info!("✅ 安全配置加载完成: {} 个交易所配置", config.exchanges.len());

        Ok(Self { config, secrets })
    }

    /// 验证配置完整性
    fn validate_config(config: &QingxiSafeConfig) -> Result<()> {
        // 验证交易所配置
        for (name, exchange) in &config.exchanges {
            if exchange.api_base_url.is_empty() {
                return Err(anyhow::anyhow!("交易所 {} 缺少 API URL", name));
            }
            if exchange.api_key_ref.is_empty() {
                return Err(anyhow::anyhow!("交易所 {} 缺少 API Key 引用", name));
            }
        }

        // 验证系统配置
        if config.system.http_port == 0 {
            return Err(anyhow::anyhow!("HTTP端口配置无效"));
        }

        // 验证交易配置
        if config.trading.default_min_profit <= 0.0 {
            return Err(anyhow::anyhow!("最小利润阈值必须大于0"));
        }

        Ok(())
    }

    /// 应用环境变量覆盖
    fn apply_env_overrides(config: &mut QingxiSafeConfig) -> Result<()> {
        // HTTP端口覆盖
        if let Ok(port) = std::env::var("QINGXI_HTTP_PORT") {
            config.system.http_port = port.parse()
                .context("Invalid HTTP port in environment")?;
        }

        // 日志级别覆盖
        if let Ok(log_level) = std::env::var("QINGXI_LOG_LEVEL") {
            config.system.log_level = log_level;
        }

        Ok(())
    }

    /// 安全获取交易所配置
    pub fn get_exchange_config(&self, exchange: &str) -> Result<&ExchangeConfig> {
        self.config.exchanges.get(exchange)
            .ok_or_else(|| anyhow::anyhow!("未找到交易所配置: {}", exchange))
    }

    /// 安全获取交易所API密钥
    pub fn get_exchange_credentials(&self, exchange: &str) -> Result<(String, String)> {
        let exchange_config = self.get_exchange_config(exchange)?;
        
        let api_key = self.secrets.get_secret(&exchange_config.api_key_ref)?;
        let secret_key = self.secrets.get_secret(&exchange_config.secret_key_ref)?;
        
        Ok((api_key, secret_key))
    }

    /// 获取系统配置
    pub fn get_system_config(&self) -> &SystemConfig {
        &self.config.system
    }

    /// 获取交易配置
    pub fn get_trading_config(&self) -> &TradingConfig {
        &self.config.trading
    }

    /// 获取监控配置
    pub fn get_monitoring_config(&self) -> &MonitoringConfig {
        &self.config.monitoring
    }

    /// 热重载配置
    pub fn reload_config(&mut self, config_path: &str) -> Result<()> {
        let new_manager = Self::load_from_file(config_path)?;
        self.config = new_manager.config;
        info!("✅ 配置热重载完成");
        Ok(())
    }
}

impl SecretManager {
    /// 加载密钥（从环境变量或密钥文件）
    fn load_secrets() -> Result<Self> {
        let mut secrets = HashMap::new();
        
        // 从环境变量加载密钥
        for (key, value) in std::env::vars() {
            if key.starts_with("QINGXI_SECRET_") {
                let secret_key = key.strip_prefix("QINGXI_SECRET_")
            .ok_or_else(|| anyhow::anyhow!("Invalid secret key format"))?;
                secrets.insert(secret_key.to_lowercase(), value);
            }
        }

        // 可以扩展从密钥文件加载
        // TODO: 支持 HashiCorp Vault, AWS Secrets Manager 等

        info!("✅ 加载了 {} 个密钥配置", secrets.len());
        Ok(Self { secrets })
    }

    /// 安全获取密钥
    fn get_secret(&self, key: &str) -> Result<String> {
        self.secrets.get(key)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("未找到密钥: {}", key))
    }
}

/// 默认配置生成器
impl Default for QingxiSafeConfig {
    fn default() -> Self {
        let mut exchanges = HashMap::new();
        
        // 添加支持的交易所模板配置
        exchanges.insert("binance".to_string(), ExchangeConfig {
            name: "Binance".to_string(),
            api_base_url: "https://api.binance.com".to_string(),
            ws_url: "wss://stream.binance.com:9443".to_string(),
            api_key_ref: "binance_api_key".to_string(),
            secret_key_ref: "binance_secret_key".to_string(),
            rate_limit: 1200,
            timeout_ms: 5000,
            enabled: true,
        });

        exchanges.insert("okx".to_string(), ExchangeConfig {
            name: "OKX".to_string(),
            api_base_url: "https://www.okx.com".to_string(),
            ws_url: "wss://ws.okx.com:8443".to_string(),
            api_key_ref: "okx_api_key".to_string(),
            secret_key_ref: "okx_secret_key".to_string(),
            rate_limit: 600,
            timeout_ms: 5000,
            enabled: true,
        });

        Self {
            exchanges,
            system: SystemConfig {
                http_port: 50061,
                log_level: "info".to_string(),
                max_connections: 1000,
                thread_pool_size: Some(8),
            },
            trading: TradingConfig {
                default_min_profit: 0.005, // 0.5%
                max_position_size: 10000.0,
                slippage_threshold: 0.001,
                order_timeout_ms: 10000,
            },
            monitoring: MonitoringConfig {
                prometheus_port: 9090,
                health_check_interval_ms: 30000,
                alert_webhook_url: None,
            },
        }
    }
}

