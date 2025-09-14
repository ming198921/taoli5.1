//! # 环境变量配置管理模块 (Environment Configuration Module)
//! 
//! PHASE 2: 统一的环境变量驱动配置系统
//! 提供标准化的配置模式，消除硬编码，支持多环境部署

use crate::production_error_handling::{QingxiResult, QingxiError, EnvVar};
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// 环境配置管理器
pub struct EnvironmentConfig {
    exchange_configs: HashMap<String, ExchangeEnvironmentConfig>,
    system_config: SystemEnvironmentConfig,
}

/// 交易所环境配置
#[derive(Debug, Clone)]
pub struct ExchangeEnvironmentConfig {
    pub exchange_id: String,
    pub websocket_url: String,
    pub rest_api_url: Option<String>,
    pub api_key: Option<String>,
    pub secret: Option<String>,
    pub testnet: bool,
}

/// 系统环境配置
#[derive(Debug, Clone)]
pub struct SystemEnvironmentConfig {
    pub server_port: u16,
    pub log_level: String,
    pub cache_ttl_seconds: u64,
    pub memory_pool_size: usize,
    pub worker_threads: Option<usize>,
    pub enable_metrics: bool,
}

impl EnvironmentConfig {
    /// 从环境变量加载完整配置
    pub fn load_from_env() -> QingxiResult<Self> {
        info!("🔧 Loading configuration from environment variables");
        
        let system_config = SystemEnvironmentConfig::load_from_env()?;
        let mut exchange_configs = HashMap::new();
        
        // 支持的交易所列表
        let supported_exchanges = ["binance", "bybit", "okx", "huobi", "gateio"];
        
        for exchange_id in supported_exchanges {
            match ExchangeEnvironmentConfig::load_from_env(exchange_id) {
                Ok(config) => {
                    debug!("✅ Loaded {} configuration from environment", exchange_id);
                    exchange_configs.insert(exchange_id.to_string(), config);
                },
                Err(e) => {
                    warn!("⚠️  Failed to load {} configuration: {}", exchange_id, e);
                    // 继续加载其他交易所，不因为单个交易所配置失败而中断
                }
            }
        }
        
        if exchange_configs.is_empty() {
            return Err(QingxiError::config("No valid exchange configurations found"));
        }
        
        info!("✅ Environment configuration loaded successfully: {} exchanges configured", 
              exchange_configs.len());
        
        Ok(Self {
            exchange_configs,
            system_config,
        })
    }
    
    /// 获取交易所配置
    pub fn get_exchange_config(&self, exchange_id: &str) -> Option<&ExchangeEnvironmentConfig> {
        self.exchange_configs.get(exchange_id)
    }
    
    /// 获取系统配置
    pub fn get_system_config(&self) -> &SystemEnvironmentConfig {
        &self.system_config
    }
    
    /// 验证所有配置
    pub fn validate(&self) -> QingxiResult<()> {
        // 验证系统配置
        self.system_config.validate()?;
        
        // 验证每个交易所配置
        for (exchange_id, config) in &self.exchange_configs {
            config.validate()
                .map_err(|e| QingxiError::config(
                    format!("Exchange {} configuration invalid: {}", exchange_id, e)
                ))?;
        }
        
        info!("✅ All configurations validated successfully");
        Ok(())
    }
    
    /// 生成配置示例和环境变量模板
    pub fn generate_env_template() -> String {
        format!(r#"# Qingxi 5.1 Enhanced System Environment Configuration
# Copy this file to .env or set these variables in your deployment environment

# ============================================================================
# SYSTEM CONFIGURATION
# ============================================================================
QINGXI_SERVER_PORT=8080
QINGXI_LOG_LEVEL=INFO
QINGXI_CACHE_TTL_SECONDS=300
QINGXI_MEMORY_POOL_SIZE=2000
QINGXI_WORKER_THREADS=4
QINGXI_ENABLE_METRICS=true

# ============================================================================
# BINANCE CONFIGURATION
# ============================================================================
QINGXI_BINANCE_WS_URL=wss://stream.binance.com:9443/ws
QINGXI_BINANCE_API_URL=https://api.binance.com/api/v3
QINGXI_BINANCE_API_KEY=your_binance_api_key_here
QINGXI_BINANCE_SECRET=your_binance_secret_here
QINGXI_BINANCE_TESTNET=false

# ============================================================================
# BYBIT CONFIGURATION  
# ============================================================================
QINGXI_BYBIT_WS_URL=wss://stream.bybit.com/v5/public/spot
QINGXI_BYBIT_API_URL=https://api.bybit.com
QINGXI_BYBIT_API_KEY=your_bybit_api_key_here
QINGXI_BYBIT_SECRET=your_bybit_secret_here
QINGXI_BYBIT_TESTNET=false

# ============================================================================
# OKX CONFIGURATION
# ============================================================================
QINGXI_OKX_WS_URL=wss://ws.okx.com:8443/ws/v5/public
QINGXI_OKX_API_URL=https://www.okx.com/api/v5
QINGXI_OKX_API_KEY=your_okx_api_key_here
QINGXI_OKX_SECRET=your_okx_secret_here
QINGXI_OKX_PASSPHRASE=your_okx_passphrase_here
QINGXI_OKX_TESTNET=false

# ============================================================================
# HUOBI CONFIGURATION
# ============================================================================
QINGXI_HUOBI_WS_URL=wss://api.huobi.pro/ws
QINGXI_HUOBI_API_URL=https://api.huobi.pro
QINGXI_HUOBI_API_KEY=your_huobi_api_key_here
QINGXI_HUOBI_SECRET=your_huobi_secret_here
QINGXI_HUOBI_TESTNET=false

# ============================================================================
# GATE.IO CONFIGURATION
# ============================================================================
QINGXI_GATEIO_WS_URL=wss://api.gateio.ws/ws/v4/
QINGXI_GATEIO_API_URL=https://api.gateio.ws/api/v4
QINGXI_GATEIO_API_KEY=your_gateio_api_key_here
QINGXI_GATEIO_SECRET=your_gateio_secret_here
QINGXI_GATEIO_TESTNET=false

# ============================================================================
# DEPLOYMENT NOTES
# ============================================================================
# 1. Replace 'your_*_here' values with actual API credentials
# 2. Set TESTNET=true for testing environments
# 3. Adjust MEMORY_POOL_SIZE based on your system resources
# 4. LOG_LEVEL options: TRACE, DEBUG, INFO, WARN, ERROR
# 5. WORKER_THREADS: omit for auto-detection based on CPU cores
"#)
    }
}

impl ExchangeEnvironmentConfig {
    /// 从环境变量加载交易所配置
    pub fn load_from_env(exchange_id: &str) -> QingxiResult<Self> {
        let exchange_upper = exchange_id.to_uppercase();
        
        let websocket_url = EnvVar::get_string(&format!("QINGXI_{}_WS_URL", exchange_upper))
            .or_else(|_| Err(QingxiError::environment(
                format!("QINGXI_{}_WS_URL", exchange_upper),
                "WebSocket URL is required"
            )))?;
        
        let rest_api_url = EnvVar::get_string(&format!("QINGXI_{}_API_URL", exchange_upper))
            .ok();
        
        let api_key = EnvVar::get_string(&format!("QINGXI_{}_API_KEY", exchange_upper))
            .ok();
        
        let secret = EnvVar::get_string(&format!("QINGXI_{}_SECRET", exchange_upper))
            .ok();
        
        let testnet = EnvVar::get_bool(&format!("QINGXI_{}_TESTNET", exchange_upper))
            .unwrap_or(false);
        
        Ok(Self {
            exchange_id: exchange_id.to_string(),
            websocket_url,
            rest_api_url,
            api_key,
            secret,
            testnet,
        })
    }
    
    /// 验证交易所配置
    pub fn validate(&self) -> QingxiResult<()> {
        use crate::production_error_handling::ConfigValidator;
        
        // 验证WebSocket URL
        ConfigValidator::validate_url(&self.websocket_url)?;
        
        // 验证REST API URL（如果存在）
        if let Some(ref api_url) = self.rest_api_url {
            ConfigValidator::validate_url(api_url)?;
        }
        
        // 验证API密钥（如果存在）
        if let Some(ref api_key) = self.api_key {
            ConfigValidator::validate_api_key(api_key, &self.exchange_id)?;
        }
        
        // 检查API密钥和密钥的一致性
        if self.api_key.is_some() && self.secret.is_none() {
            return Err(QingxiError::config(
                format!("Exchange {} has API key but missing secret", self.exchange_id)
            ));
        }
        
        Ok(())
    }
}

impl SystemEnvironmentConfig {
    /// 从环境变量加载系统配置
    pub fn load_from_env() -> QingxiResult<Self> {
        let server_port = EnvVar::get_parsed("QINGXI_SERVER_PORT")
            .unwrap_or(8080);
        
        let log_level = EnvVar::get_string("QINGXI_LOG_LEVEL")
            .unwrap_or_else(|_| "INFO".to_string());
        
        let cache_ttl_seconds = EnvVar::get_parsed("QINGXI_CACHE_TTL_SECONDS")
            .unwrap_or(300);
        
        let memory_pool_size = EnvVar::get_parsed("QINGXI_MEMORY_POOL_SIZE")
            .unwrap_or(2000);
        
        let worker_threads = EnvVar::get_parsed("QINGXI_WORKER_THREADS")
            .ok();
        
        let enable_metrics = EnvVar::get_bool("QINGXI_ENABLE_METRICS")
            .unwrap_or(true);
        
        Ok(Self {
            server_port,
            log_level,
            cache_ttl_seconds,
            memory_pool_size,
            worker_threads,
            enable_metrics,
        })
    }
    
    /// 验证系统配置
    pub fn validate(&self) -> QingxiResult<()> {
        use crate::production_error_handling::ConfigValidator;
        
        // 验证端口号
        ConfigValidator::validate_port(self.server_port)?;
        
        // 验证日志级别
        let valid_log_levels = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(QingxiError::config(
                format!("Invalid log level: {}. Valid options: {:?}", 
                       self.log_level, valid_log_levels)
            ));
        }
        
        // 验证内存池大小
        if self.memory_pool_size < 100 {
            return Err(QingxiError::config(
                "Memory pool size must be at least 100"
            ));
        }
        
        if self.memory_pool_size > 10000 {
            warn!("Large memory pool size ({}), ensure sufficient system memory", 
                  self.memory_pool_size);
        }
        
        // 验证工作线程数
        if let Some(threads) = self.worker_threads {
            if threads == 0 {
                return Err(QingxiError::config(
                    "Worker threads must be greater than 0"
                ));
            }
            
            if threads > num_cpus::get() * 2 {
                warn!("Worker threads ({}) exceeds 2x CPU cores ({})", 
                      threads, num_cpus::get());
            }
        }
        
        Ok(())
    }
}

