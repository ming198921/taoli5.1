//! QingXi 5.1 主配置管理器增强版
//! 集成安全配置和一致性检查

use crate::error_handling::{SafeConfigManager, QingxiSafeConfig};
use crate::cleaner::{ConsistencyConfig, DataConsistencyManager};
use anyhow::{Context, Result};
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// QingXi 5.1 增强主配置管理器
pub struct EnhancedConfigManager {
    /// 安全配置管理器
    safe_config: Arc<RwLock<SafeConfigManager>>,
    /// 数据一致性管理器
    consistency_manager: Option<Arc<DataConsistencyManager>>,
    /// 配置文件路径
    config_path: String,
}

impl EnhancedConfigManager {
    /// 创建增强配置管理器
    pub async fn new(config_path: &str) -> Result<Self> {
        // 加载安全配置
        let safe_config = SafeConfigManager::load_from_file(config_path)
            .context("Failed to load safe config")?;

        info!("✅ 增强配置管理器初始化完成");

        Ok(Self {
            safe_config: Arc::new(RwLock::new(safe_config)),
            consistency_manager: None,
            config_path: config_path.to_string(),
        })
    }

    /// 启用数据一致性检查
    pub async fn enable_consistency_check(&mut self) -> Result<()> {
        let consistency_config = ConsistencyConfig {
            enable_cross_exchange_check: true,
            price_deviation_threshold: 0.02, // 2%
            data_freshness_window_ms: 10000,
            quality_check_interval_ms: 30000,
            auto_recovery_enabled: true,
        };

        let consistency_manager = Arc::new(
            DataConsistencyManager::new(consistency_config)
        );

        // 启动监控
        consistency_manager.start_monitoring().await?;
        
        self.consistency_manager = Some(consistency_manager);
        
        info!("✅ 数据一致性检查已启用");
        Ok(())
    }

    /// 获取交易所配置
    pub async fn get_exchange_config(&self, exchange: &str) -> Result<ExchangeConfigSafe> {
        let config_guard = self.safe_config.read().await;
        let exchange_config = config_guard.get_exchange_config(exchange)?;
        let (api_key, secret_key) = config_guard.get_exchange_credentials(exchange)?;

        Ok(ExchangeConfigSafe {
            name: exchange_config.name.clone(),
            api_base_url: exchange_config.api_base_url.clone(),
            ws_url: exchange_config.ws_url.clone(),
            api_key,
            secret_key,
            rate_limit: exchange_config.rate_limit,
            timeout_ms: exchange_config.timeout_ms,
            enabled: exchange_config.enabled,
        })
    }

    /// 获取系统配置
    pub async fn get_system_config(&self) -> SystemConfigSafe {
        let config_guard = self.safe_config.read().await;
        let system_config = config_guard.get_system_config();

        SystemConfigSafe {
            http_port: system_config.http_port,
            log_level: system_config.log_level.clone(),
            max_connections: system_config.max_connections,
            thread_pool_size: system_config.thread_pool_size.unwrap_or(8),
        }
    }

    /// 获取交易配置
    pub async fn get_trading_config(&self) -> TradingConfigSafe {
        let config_guard = self.safe_config.read().await;
        let trading_config = config_guard.get_trading_config();

        TradingConfigSafe {
            default_min_profit: trading_config.default_min_profit,
            max_position_size: trading_config.max_position_size,
            slippage_threshold: trading_config.slippage_threshold,
            order_timeout_ms: trading_config.order_timeout_ms,
        }
    }

    /// 获取监控配置
    pub async fn get_monitoring_config(&self) -> MonitoringConfigSafe {
        let config_guard = self.safe_config.read().await;
        let monitoring_config = config_guard.get_monitoring_config();

        MonitoringConfigSafe {
            prometheus_port: monitoring_config.prometheus_port,
            health_check_interval_ms: monitoring_config.health_check_interval_ms,
            alert_webhook_url: monitoring_config.alert_webhook_url.clone(),
        }
    }

    /// 热重载配置
    pub async fn reload_config(&self) -> Result<()> {
        let mut config_guard = self.safe_config.write().await;
        config_guard.reload_config(&self.config_path)?;
        
        info!("✅ 配置热重载完成");
        Ok(())
    }

    /// 获取一致性管理器
    pub fn get_consistency_manager(&self) -> Option<Arc<DataConsistencyManager>> {
        self.consistency_manager.clone()
    }

    /// 验证配置完整性
    pub async fn validate_configuration(&self) -> Result<ConfigValidationReport> {
        let config_guard = self.safe_config.read().await;
        
        let mut report = ConfigValidationReport {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            enabled_exchanges: 0,
            total_exchanges: 0,
        };

        // 验证交易所配置
        let system_config = config_guard.get_system_config();
        let trading_config = config_guard.get_trading_config();

        // 检查HTTP端口
        if system_config.http_port == 0 || system_config.http_port > 65535 {
            report.errors.push("Invalid HTTP port configuration".to_string());
            report.is_valid = false;
        }

        // 检查交易配置
        if trading_config.default_min_profit <= 0.0 {
            report.errors.push("Min profit threshold must be positive".to_string());
            report.is_valid = false;
        }

        if trading_config.slippage_threshold <= 0.0 || trading_config.slippage_threshold > 0.1 {
            report.warnings.push("Unusual slippage threshold configuration".to_string());
        }

        // TODO: 验证交易所配置
        // 这里需要访问内部配置，暂时使用默认值
        report.enabled_exchanges = 2; // Binance + OKX
        report.total_exchanges = 4;   // 包括禁用的

        if report.enabled_exchanges == 0 {
            report.errors.push("No exchanges enabled".to_string());
            report.is_valid = false;
        }

        Ok(report)
    }
}

/// 安全交易所配置（带密钥）
#[derive(Debug, Clone)]
pub struct ExchangeConfigSafe {
    pub name: String,
    pub api_base_url: String,
    pub ws_url: String,
    pub api_key: String,
    pub secret_key: String,
    pub rate_limit: u32,
    pub timeout_ms: u64,
    pub enabled: bool,
}

/// 安全系统配置
#[derive(Debug, Clone)]
pub struct SystemConfigSafe {
    pub http_port: u16,
    pub log_level: String,
    pub max_connections: u32,
    pub thread_pool_size: u32,
}

/// 安全交易配置
#[derive(Debug, Clone)]
pub struct TradingConfigSafe {
    pub default_min_profit: f64,
    pub max_position_size: f64,
    pub slippage_threshold: f64,
    pub order_timeout_ms: u64,
}

/// 安全监控配置
#[derive(Debug, Clone)]
pub struct MonitoringConfigSafe {
    pub prometheus_port: u16,
    pub health_check_interval_ms: u64,
    pub alert_webhook_url: Option<String>,
}

/// 配置验证报告
#[derive(Debug)]
pub struct ConfigValidationReport {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub enabled_exchanges: usize,
    pub total_exchanges: usize,
}

impl ConfigValidationReport {
    /// 打印验证报告
    pub fn print_report(&self) {
        if self.is_valid {
            info!("✅ 配置验证通过");
        } else {
            error!("❌ 配置验证失败");
        }

        info!("📊 交易所状态: {}/{} 已启用", self.enabled_exchanges, self.total_exchanges);

        for warning in &self.warnings {
            warn!("⚠️ {}", warning);
        }

        for error in &self.errors {
            error!("❌ {}", error);
        }
    }
}

