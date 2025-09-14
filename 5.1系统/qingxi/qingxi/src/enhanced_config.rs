#![allow(dead_code)]
//! # Qingxi 5.1 增强配置系统
//! 
//! 基于TOML的配置加载和管理，支持热重载和配置验证

use crate::data_distribution::DistributorConfig;
use crate::api_health_monitor_enhanced::HealthMonitorConfig;
use crate::system_enhanced::{EnhancedConfig, StorageMode, LatencyConfig};
use serde::{Serialize, Deserialize};
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, warn};

/// 完整的增强配置结构 - 与TOML文件对应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QingxiEnhancedConfiguration {
    /// 通用配置
    pub general: GeneralConfig,
    
    /// API服务器配置
    pub api_server: ApiServerConfig,
    
    /// 增强功能配置
    pub enhancements: EnhancementFlags,
    
    /// 存储策略配置
    pub storage_strategy: StorageStrategyConfig,
    
    /// 数据分发配置
    pub data_distribution: DataDistributionConfig,
    
    /// 性能优化配置
    pub performance_optimization: PerformanceOptimizationConfig,
    
    /// 延迟监控配置
    pub latency_monitoring: LatencyMonitoringConfig,
    
    /// CPU亲和性配置
    pub cpu_affinity: CpuAffinityConfig,
    
    /// 健康监控配置
    pub health_monitoring: HealthMonitoringConfig,
    
    /// 数据质量配置
    pub data_quality: DataQualityConfig,
    
    /// 审计配置
    pub audit_storage: AuditStorageConfig,
    
    /// 机器学习训练数据配置
    pub ml_training_data: MlTrainingDataConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub log_level: String,
    pub metrics_enabled: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementFlags {
    /// 是否启用数据分发功能
    pub enable_data_distribution: bool,
    /// 是否启用交叉校验
    pub enable_cross_validation: bool,
    /// 是否启用健康监控
    pub enable_health_monitoring: bool,
    /// 是否启用性能优化
    pub enable_performance_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStrategyConfig {
    pub mode: String, // "realtime" | "async" | "sync" | "realtime_with_audit"
    pub audit_storage_enabled: bool,
    pub training_data_collection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDistributionConfig {
    pub strategy_topic: String,
    pub arbitrage_topic: String,
    pub risk_topic: String,
    pub buffer_size: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOptimizationConfig {
    pub enable_adaptive_connections: bool,
    pub enable_binary_apis: bool,
    pub enable_kernel_tuning: bool,
    pub enable_cpu_affinity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMonitoringConfig {
    pub target_strategy_latency_ms: f64,
    pub latency_window_size: usize,
    pub connection_switch_threshold_ms: f64,
    pub switch_cooldown_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuAffinityConfig {
    #[serde(default)]
    pub network_thread_cores: Vec<usize>,
    #[serde(default)]
    pub processing_thread_cores: Vec<usize>,
    #[serde(default)]
    pub storage_thread_cores: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringConfig {
    pub health_check_interval_seconds: u64,
    pub alert_threshold_score: f64,
    pub enable_auto_recovery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityConfig {
    pub enable_cross_exchange_validation: bool,
    pub max_price_variance_percent: f64,
    pub min_confidence_score: f64,
    pub anomaly_detection_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStorageConfig {
    pub storage_path: String,
    pub retention_days: u32,
    pub compress_after_days: u32,
    pub backup_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlTrainingDataConfig {
    pub collection_enabled: bool,
    pub storage_path: String,
    pub feature_extraction_enabled: bool,
    pub sample_rate_percent: f64,
}

impl Default for QingxiEnhancedConfiguration {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                log_level: "info".to_string(),
                metrics_enabled: true,
                version: "5.1.1-enhanced".to_string(),
            },
            api_server: ApiServerConfig {
                host: "0.0.0.0".to_string(),
                port: 50051,
            },
            enhancements: EnhancementFlags {
                enable_data_distribution: true,
                enable_cross_validation: true,
                enable_health_monitoring: true,
                enable_performance_optimization: true,
            },
            storage_strategy: StorageStrategyConfig {
                mode: "realtime_with_audit".to_string(),
                audit_storage_enabled: true,
                training_data_collection: false,
            },
            data_distribution: DataDistributionConfig {
                strategy_topic: "qingxi.strategy.data".to_string(),
                arbitrage_topic: "qingxi.arbitrage.opportunities".to_string(),
                risk_topic: "qingxi.risk.alerts".to_string(),
                buffer_size: 50000,
                batch_size: 100,
                flush_interval_ms: 10,
            },
            performance_optimization: PerformanceOptimizationConfig {
                enable_adaptive_connections: true,
                enable_binary_apis: false,
                enable_kernel_tuning: true,
                enable_cpu_affinity: true,
            },
            latency_monitoring: LatencyMonitoringConfig {
                target_strategy_latency_ms: 0.2,
                latency_window_size: 1000,
                connection_switch_threshold_ms: 5.0,
                switch_cooldown_seconds: 30,
            },
            cpu_affinity: CpuAffinityConfig {
                network_thread_cores: vec![0, 1],
                processing_thread_cores: vec![2, 3],
                storage_thread_cores: vec![4, 5],
            },
            health_monitoring: HealthMonitoringConfig {
                health_check_interval_seconds: 10,
                alert_threshold_score: 50.0,
                enable_auto_recovery: true,
            },
            data_quality: DataQualityConfig {
                enable_cross_exchange_validation: true,
                max_price_variance_percent: 1.0,
                min_confidence_score: 0.8,
                anomaly_detection_threshold: 0.8,
            },
            audit_storage: AuditStorageConfig {
                storage_path: "data/audit".to_string(),
                retention_days: 30,
                compress_after_days: 7,
                backup_enabled: true,
            },
            ml_training_data: MlTrainingDataConfig {
                collection_enabled: true,
                storage_path: "data/ml_training".to_string(),
                feature_extraction_enabled: true,
                sample_rate_percent: 10.0,
            },
        }
    }
}

/// 配置加载器和管理器
pub struct ConfigurationManager {
    config: QingxiEnhancedConfiguration,
    config_file_path: String,
    is_watching: std::sync::atomic::AtomicBool,
}

impl ConfigurationManager {
    /// 从文件加载配置
    pub async fn load_from_file<P: AsRef<Path>>(config_path: P) -> Result<Self, ConfigError> {
        let config_path_str = config_path.as_ref().to_string_lossy().to_string();
        
        let config = if config_path.as_ref().exists() {
            info!("Loading configuration from: {}", config_path_str);
            let content = fs::read_to_string(&config_path).await
                .map_err(|e| ConfigError::FileReadError(e.to_string()))?;
            
            let parsed_config: QingxiEnhancedConfiguration = toml::from_str(&content)
                .map_err(|e| ConfigError::ParseError(e.to_string()))?;
            
            // 验证配置
            Self::validate_configuration(&parsed_config)?;
            
            info!("Configuration loaded successfully");
            parsed_config
        } else {
            warn!("Configuration file not found, using default configuration");
            let default_config = QingxiEnhancedConfiguration::default();
            
            // 创建默认配置文件
            Self::save_configuration_to_file(&default_config, &config_path).await?;
            info!("Default configuration saved to: {}", config_path_str);
            
            default_config
        };
        
        Ok(Self {
            config,
            config_file_path: config_path_str,
            is_watching: std::sync::atomic::AtomicBool::new(false),
        })
    }
    
    /// 创建默认配置管理器
    pub fn default() -> Self {
        Self {
            config: QingxiEnhancedConfiguration::default(),
            config_file_path: "qingxi_enhanced_config.toml".to_string(),
            is_watching: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    /// 验证配置的有效性
    fn validate_configuration(config: &QingxiEnhancedConfiguration) -> Result<(), ConfigError> {
        // 验证端口范围
        if config.api_server.port == 0 || config.api_server.port > 65535 {
            return Err(ConfigError::ValidationError("Invalid API server port".to_string()));
        }
        
        // 验证延迟配置
        if config.latency_monitoring.target_strategy_latency_ms <= 0.0 {
            return Err(ConfigError::ValidationError("Target strategy latency must be positive".to_string()));
        }
        
        // 验证存储模式
        match config.storage_strategy.mode.as_str() {
            "realtime" | "async" | "sync" | "realtime_with_audit" => {},
            _ => return Err(ConfigError::ValidationError("Invalid storage mode".to_string())),
        }
        
        // 验证数据质量参数
        if config.data_quality.max_price_variance_percent <= 0.0 {
            return Err(ConfigError::ValidationError("Max price variance must be positive".to_string()));
        }
        
        if config.data_quality.min_confidence_score < 0.0 || config.data_quality.min_confidence_score > 1.0 {
            return Err(ConfigError::ValidationError("Min confidence score must be between 0 and 1".to_string()));
        }
        
        // 验证机器学习配置
        if config.ml_training_data.sample_rate_percent < 0.0 || config.ml_training_data.sample_rate_percent > 100.0 {
            return Err(ConfigError::ValidationError("Sample rate must be between 0 and 100".to_string()));
        }
        
        // 验证审计配置
        if config.audit_storage.retention_days == 0 {
            return Err(ConfigError::ValidationError("Audit retention days must be positive".to_string()));
        }
        
        if config.audit_storage.compress_after_days > config.audit_storage.retention_days {
            return Err(ConfigError::ValidationError("Compress after days cannot exceed retention days".to_string()));
        }
        
        info!("Configuration validation passed");
        Ok(())
    }
    
    /// 保存配置到文件
    async fn save_configuration_to_file<P: AsRef<Path>>(
        config: &QingxiEnhancedConfiguration, 
        path: P
    ) -> Result<(), ConfigError> {
        let toml_content = toml::to_string_pretty(config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
        
        // 确保目录存在
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).await
                    .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
            }
        }
        
        fs::write(&path, toml_content).await
            .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
        
        Ok(())
    }
    
    /// 获取配置引用
    pub fn get_config(&self) -> &QingxiEnhancedConfiguration {
        &self.config
    }
    
    /// 转换为EnhancedConfig
    pub fn to_enhanced_config(&self) -> EnhancedConfig {
        let storage_mode = match self.config.storage_strategy.mode.as_str() {
            "realtime" => StorageMode::Realtime,
            "async" => StorageMode::Async,
            "sync" => StorageMode::Sync,
            "realtime_with_audit" => StorageMode::RealtimeWithAudit,
            _ => StorageMode::RealtimeWithAudit, // 默认值
        };
        
        EnhancedConfig {
            enable_data_distribution: self.config.enhancements.enable_data_distribution,
            enable_cross_validation: self.config.enhancements.enable_cross_validation,
            enable_health_monitoring: self.config.enhancements.enable_health_monitoring,
            enable_performance_optimization: self.config.enhancements.enable_performance_optimization,
            storage_mode,
            audit_storage_enabled: self.config.storage_strategy.audit_storage_enabled,
            training_data_collection: self.config.storage_strategy.training_data_collection,
            distribution_config: DistributorConfig {
                audit_queue_capacity: self.config.data_distribution.buffer_size,
                strategy_latency_target_ns: (self.config.latency_monitoring.target_strategy_latency_ms * 1_000_000.0) as u64,
                enable_quality_scoring: self.config.data_quality.enable_cross_exchange_validation,
                enable_audit_storage: self.config.storage_strategy.audit_storage_enabled,
                latency_window_size: self.config.latency_monitoring.latency_window_size,
            },
            latency_config: LatencyConfig {
                target_strategy_latency_ns: (self.config.latency_monitoring.target_strategy_latency_ms * 1_000_000.0) as u64,
                latency_window_size: self.config.latency_monitoring.latency_window_size,
                connection_switch_threshold_ms: self.config.latency_monitoring.connection_switch_threshold_ms,
                switch_cooldown_seconds: self.config.latency_monitoring.switch_cooldown_seconds,
            },
        }
    }
    
    /// 转换为健康监控配置
    pub fn to_health_monitor_config(&self) -> HealthMonitorConfig {
        HealthMonitorConfig {
            health_cache_ttl_ms: self.config.health_monitoring.health_check_interval_seconds * 1000,
            latency_window_size: self.config.latency_monitoring.latency_window_size,
            error_rate_threshold: 0.01, // 1% 固定阈值
            latency_threshold_ms: self.config.latency_monitoring.target_strategy_latency_ms,
            min_quality_score: self.config.data_quality.min_confidence_score,
            message_rate_window_seconds: 60,
        }
    }
    
    /// 热重载配置（从文件重新加载）
    pub async fn reload_configuration(&mut self) -> Result<bool, ConfigError> {
        info!("Reloading configuration from: {}", self.config_file_path);
        
        if !Path::new(&self.config_file_path).exists() {
            return Err(ConfigError::FileNotFound(self.config_file_path.clone()));
        }
        
        let content = fs::read_to_string(&self.config_file_path).await
            .map_err(|e| ConfigError::FileReadError(e.to_string()))?;
        
        let new_config: QingxiEnhancedConfiguration = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        // 验证新配置
        Self::validate_configuration(&new_config)?;
        
        // 检查配置是否有变化
        let config_changed = !self.configs_equal(&self.config, &new_config);
        
        if config_changed {
            info!("Configuration changed, updating...");
            self.config = new_config;
        } else {
            debug!("Configuration unchanged");
        }
        
        Ok(config_changed)
    }
    
    /// 比较两个配置是否相等
    fn configs_equal(&self, config1: &QingxiEnhancedConfiguration, config2: &QingxiEnhancedConfiguration) -> bool {
        // 简化的比较（实际项目中应该序列化后比较或实现PartialEq）
        config1.enhancements.enable_data_distribution == config2.enhancements.enable_data_distribution &&
        config1.enhancements.enable_cross_validation == config2.enhancements.enable_cross_validation &&
        config1.enhancements.enable_health_monitoring == config2.enhancements.enable_health_monitoring &&
        config1.storage_strategy.mode == config2.storage_strategy.mode &&
        config1.latency_monitoring.target_strategy_latency_ms == config2.latency_monitoring.target_strategy_latency_ms
    }
    
    /// 启动配置文件监控（热重载）
    pub async fn start_config_watching(&self) -> Result<(), ConfigError> {
        if self.is_watching.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }
        
        self.is_watching.store(true, std::sync::atomic::Ordering::Relaxed);
        
        // 实际项目中可以使用 notify crate 监控文件变化
        // 这里用简单的定期检查模拟
        let config_path = self.config_file_path.clone();
        let is_watching = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(self.is_watching.load(std::sync::atomic::Ordering::Relaxed)));
        
        tokio::spawn(async move {
            let mut last_modified = std::time::SystemTime::UNIX_EPOCH;
            
            while is_watching.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(metadata) = std::fs::metadata(&config_path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified > last_modified {
                            info!("Configuration file changed, triggering reload...");
                            last_modified = modified;
                            // 实际项目中应该发送信号给主程序重载配置
                        }
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        });
        
        info!("Configuration file watching started");
        Ok(())
    }
    
    /// 停止配置文件监控
    pub fn stop_config_watching(&self) {
        self.is_watching.store(false, std::sync::atomic::Ordering::Relaxed);
        info!("Configuration file watching stopped");
    }
    
    /// 导出配置为TOML字符串
    pub fn export_to_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }
    
    /// 保存当前配置到文件
    pub async fn save_current_config(&self) -> Result<(), ConfigError> {
        Self::save_configuration_to_file(&self.config, &self.config_file_path).await
    }
    
    /// 更新特定配置项
    pub fn update_enhancement_flags(&mut self, flags: EnhancementFlags) -> Result<(), ConfigError> {
        self.config.enhancements = flags;
        info!("Enhancement flags updated");
        Ok(())
    }
    
    pub fn update_storage_strategy(&mut self, strategy: StorageStrategyConfig) -> Result<(), ConfigError> {
        // 验证存储模式
        match strategy.mode.as_str() {
            "realtime" | "async" | "sync" | "realtime_with_audit" => {},
            _ => return Err(ConfigError::ValidationError("Invalid storage mode".to_string())),
        }
        
        self.config.storage_strategy = strategy;
        info!("Storage strategy updated");
        Ok(())
    }
    
    pub fn update_latency_config(&mut self, latency_config: LatencyMonitoringConfig) -> Result<(), ConfigError> {
        if latency_config.target_strategy_latency_ms <= 0.0 {
            return Err(ConfigError::ValidationError("Target strategy latency must be positive".to_string()));
        }
        
        self.config.latency_monitoring = latency_config;
        info!("Latency monitoring config updated");
        Ok(())
    }
    
    /// 获取配置摘要（用于日志和监控）
    pub fn get_config_summary(&self) -> ConfigurationSummary {
        ConfigurationSummary {
            version: self.config.general.version.clone(),
            enhancement_flags: self.config.enhancements.clone(),
            storage_mode: self.config.storage_strategy.mode.clone(),
            target_latency_ms: self.config.latency_monitoring.target_strategy_latency_ms,
            audit_enabled: self.config.storage_strategy.audit_storage_enabled,
            health_monitoring_enabled: self.config.enhancements.enable_health_monitoring,
            config_file_path: self.config_file_path.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigurationSummary {
    pub version: String,
    pub enhancement_flags: EnhancementFlags,
    pub storage_mode: String,
    pub target_latency_ms: f64,
    pub audit_enabled: bool,
    pub health_monitoring_enabled: bool,
    pub config_file_path: String,
}

/// 配置错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Failed to read file: {0}")]
    FileReadError(String),
    
    #[error("Failed to write file: {0}")]
    FileWriteError(String),
    
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
    
    #[error("Failed to serialize configuration: {0}")]
    SerializationError(String),
}

/// 生成默认配置文件的工具函数
pub async fn generate_default_config_file<P: AsRef<Path>>(path: P) -> Result<(), ConfigError> {
    let default_config = QingxiEnhancedConfiguration::default();
    
    let toml_content = toml::to_string_pretty(&default_config)
        .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
    
    // 添加注释说明
    let commented_content = format!(
        r#"# Qingxi 5.1 增强配置文件
# 此文件包含所有增强功能的配置选项
# 修改后需要重启系统或触发热重载

{}

# 配置说明：
# - storage_strategy.mode: 存储策略
#   * "realtime": 无存储，最低延迟 (~0.17ms)
#   * "async": 异步存储，不阻塞主线程 (~0.17ms 主线程)
#   * "sync": 传统同步存储 (~0.35ms)
#   * "realtime_with_audit": 实时策略+后台审计存储 (~0.17ms 主线程)
#
# - latency_monitoring.target_strategy_latency_ms: 策略延迟目标（毫秒）
# - data_quality.max_price_variance_percent: 最大价格偏差百分比
# - ml_training_data.sample_rate_percent: 机器学习训练数据采样率
"#,
        toml_content
    );
    
    // 确保目录存在
    if let Some(parent) = path.as_ref().parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
        }
    }
    
    tokio::fs::write(&path, commented_content).await
        .map_err(|e| ConfigError::FileWriteError(e.to_string()))?;
    
    info!("Default configuration file generated: {}", path.as_ref().display());
    Ok(())
}

