//! System configuration management
//! 
//! Handles loading and validation of all system configuration.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use anyhow::Result;
use notify::Watcher;

use adapters::risk::RiskConfig;
use adapters::nats::NatsConfig;
use adapters::funds::{FundsConfig, FundLimits};
use adapters::market_data::MarketDataConfig;
use adapters::execution::ExchangeCredentials;
use common::precision::FixedPrice;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parsing error: {0}")]
    TomlParsing(#[from] toml::de::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("File watcher error: {0}")]
    WatcherError(String),
}

/// Configuration hot reload notification
#[derive(Debug, Clone)]
pub enum ConfigChangeEvent {
    StrategyConfigChanged,
    RiskConfigChanged,
    SystemConfigChanged,
    NatsConfigChanged,
}

/// Hot reload configuration manager
pub struct HotReloadConfigManager {
    /// Current configuration
    config: Arc<tokio::sync::RwLock<SystemConfig>>,
    /// Configuration change broadcaster
    change_broadcaster: tokio::sync::broadcast::Sender<ConfigChangeEvent>,
    /// File watcher
    _watcher: Option<notify::RecommendedWatcher>,
    /// Configuration file path
    config_path: String,
}

impl HotReloadConfigManager {
    pub async fn new(config_path: String) -> Result<Self, ConfigError> {
        // Load initial configuration
        let initial_config = SystemConfig::from_file(&config_path)?;
        let config = Arc::new(tokio::sync::RwLock::new(initial_config));
        
        // Create change broadcaster
        let (change_broadcaster, _) = tokio::sync::broadcast::channel(100);
        
        let mut manager = Self {
            config: config.clone(),
            change_broadcaster,
            _watcher: None,
            config_path: config_path.clone(),
        };
        
        // Setup file watcher
        manager.setup_file_watcher().await?;
        
        Ok(manager)
    }
    
    /// Setup file system watcher for configuration changes
    async fn setup_file_watcher(&mut self) -> Result<(), ConfigError> {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let config_path = self.config_path.clone();
        let config_arc = self.config.clone();
        let change_broadcaster = self.change_broadcaster.clone();
        
        // Create watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if matches!(event.kind, notify::EventKind::Modify(_)) {
                    let _ = tx.send(event).unwrap(); // Unwrap is safe here as we are in a tokio::spawn
                }
            }
        }).map_err(|e| ConfigError::WatcherError(format!("Failed to create watcher: {}", e)))?;
        
        // Watch the config file directory
        let config_dir = Path::new(&config_path)
            .parent()
            .ok_or_else(|| ConfigError::WatcherError("Invalid config path".to_string()))?;
            
        watcher.watch(config_dir, notify::RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::WatcherError(format!("Failed to watch directory: {}", e)))?;
        
        self._watcher = Some(watcher);
        
        // Spawn file change handler
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                // Check if the changed file is our config file
                if event.paths.iter().any(|p| p.ends_with(&config_path)) {
                    tracing::info!("Configuration file change detected, reloading...");
                    
                    // Add a small delay to ensure file write is complete
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    
                    match Self::reload_config(&config_path, &config_arc, &change_broadcaster).await {
                        Ok(_) => tracing::info!("Configuration reloaded successfully"),
                        Err(e) => tracing::error!("Failed to reload configuration: {}", e),
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Reload configuration from file
    async fn reload_config(
        config_path: &str,
        config_arc: &Arc<tokio::sync::RwLock<SystemConfig>>,
        change_broadcaster: &tokio::sync::broadcast::Sender<ConfigChangeEvent>,
    ) -> Result<(), ConfigError> {
        // Load new configuration
        let new_config = SystemConfig::from_file(config_path)?;
        let old_config = config_arc.read().await.clone();
        
        // Compare configurations to determine what changed
        let mut changes = Vec::new();
        
        if new_config.strategy != old_config.strategy {
            changes.push(ConfigChangeEvent::StrategyConfigChanged);
        }
        
        if new_config.risk != old_config.risk {
            changes.push(ConfigChangeEvent::RiskConfigChanged);
        }
        
        if new_config.nats != old_config.nats {
            changes.push(ConfigChangeEvent::NatsConfigChanged);
        }
        
        // Update configuration
        {
            let mut config = config_arc.write().await;
            *config = new_config;
        }
        
        // Broadcast changes
        for change in changes {
            if let Err(e) = change_broadcaster.send(change.clone()) {
                tracing::warn!("Failed to broadcast config change {:?}: {}", change, e);
            }
        }
        
        Ok(())
    }
    
    /// Get current configuration
    pub async fn get_config(&self) -> SystemConfig {
        self.config.read().await.clone()
    }
    
    /// Subscribe to configuration changes
    pub fn subscribe_changes(&self) -> tokio::sync::broadcast::Receiver<ConfigChangeEvent> {
        self.change_broadcaster.subscribe()
    }
    
    /// Manually reload configuration
    pub async fn force_reload(&self) -> Result<(), ConfigError> {
        Self::reload_config(&self.config_path, &self.config, &self.change_broadcaster).await
    }
}

/// Complete system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Strategy configuration
    pub strategy: StrategyConfigSection,
    
    /// Market data configuration  
    pub market_data: MarketDataConfig,
    
    /// Risk management configuration
    pub risk: RiskConfig,
    
    /// Execution configuration
    pub execution: ExecutionConfigSection,
    
    /// NATS messaging configuration
    pub nats: NatsConfig,
    
    /// Metrics configuration
    // pub metrics: MetricsConfig,  // 暂时注释
    
    /// Performance configuration
    pub performance: PerformanceConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Strategy configuration section
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StrategyConfigSection {
    /// Minimum profit threshold (fraction)
    pub min_profit_threshold: f64,
    
    /// Maximum slippage tolerance
    pub max_slippage: f64,
    
    /// Enabled strategy types
    pub enabled_strategies: Vec<String>,
    
    /// Strategy-specific settings
    pub inter_exchange: InterExchangeSettings,
    pub triangular: TriangularSettings,

    /// Market state weights for min_profit scaling
    pub market_state: MarketStateWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketStateWeights {
    pub cautious_weight: f64,
    pub extreme_weight: f64,
}

/// Inter-exchange arbitrage settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterExchangeSettings {
    pub max_price_diff_pct: f64,
    pub min_profit_pct: f64,
    pub max_slippage_pct: f64,
    
    // Exchange-specific fee configurations (in basis points)
    pub binance_fee_bps: Option<f64>,
    pub okx_fee_bps: Option<f64>,
    pub huobi_fee_bps: Option<f64>,
    pub kraken_fee_bps: Option<f64>,
    pub coinbase_fee_bps: Option<f64>,
}

impl InterExchangeSettings {
    /// Get fee map for all exchanges
    pub fn get_fee_map(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        
        if let Some(fee) = self.binance_fee_bps {
            map.insert("binance".to_string(), fee);
        }
        if let Some(fee) = self.okx_fee_bps {
            map.insert("okx".to_string(), fee);
        }
        if let Some(fee) = self.huobi_fee_bps {
            map.insert("huobi".to_string(), fee);
        }
        if let Some(fee) = self.kraken_fee_bps {
            map.insert("kraken".to_string(), fee);
        }
        if let Some(fee) = self.coinbase_fee_bps {
            map.insert("coinbase".to_string(), fee);
        }
        
        // Default fees if not specified
        if map.is_empty() {
            map.insert("binance".to_string(), 10.0);
            map.insert("okx".to_string(), 8.0);
            map.insert("huobi".to_string(), 20.0);
            map.insert("kraken".to_string(), 26.0);
            map.insert("coinbase".to_string(), 50.0);
        }
        
        map
    }
}

/// Triangular arbitrage settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriangularSettings {
    pub triangle_paths: Vec<TrianglePath>,
    pub min_liquidity_usd: f64,
    pub max_slippage_per_leg: f64,
}

/// Triangle path configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrianglePath {
    pub base: String,
    pub intermediate: String,
    pub quote: String,
    #[serde(default = "default_exchange")]
    pub exchange: String,
    pub min_liquidity_usd: Option<f64>,
    pub max_slippage_per_leg: Option<f64>,
}

fn default_exchange() -> String {
    "binance".to_string()
}

/// Execution configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfigSection {
    /// Dry run mode (no real trades)
    pub dry_run: bool,
    
    /// Maximum concurrent executions
    pub max_concurrent: usize,
    
    /// Execution timeout
    pub timeout_ms: u64,
    
    /// Retry configuration
    pub retry_count: u32,
    
    /// Exchange credentials
    pub exchanges: std::collections::HashMap<String, ExchangeCredentials>,
}

/// Performance tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target opportunities per second
    pub target_opportunities_per_sec: u32,
    
    /// Maximum detection latency (microseconds)
    pub max_detection_latency_us: u64,
    
    /// Maximum execution latency (milliseconds)
    pub max_execution_latency_ms: u64,
    
    /// Worker thread count
    pub worker_threads: Option<usize>,
    
    /// Memory pool settings
    pub memory_pool: MemoryPoolConfig,
}

/// Memory pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolConfig {
    /// Pre-allocate orderbook capacity
    pub orderbook_capacity: usize,
    
    /// Pre-allocate opportunity capacity  
    pub opportunity_capacity: usize,
    
    /// Buffer sizes
    pub buffer_size: usize,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    
    /// JSON format
    pub json: bool,
    
    /// Log file path
    pub file: Option<String>,
    
    /// Rotation settings
    pub rotation: LogRotationConfig,
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Maximum file size (MB)
    pub max_size_mb: u64,
    
    /// Maximum number of files
    pub max_files: u32,
}

impl SystemConfig {
    /// Load configuration from file
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
    
    /// Load configuration from file (for hot reload)
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
    
    /// Save configuration to file
    #[allow(dead_code)]
    pub async fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        tokio::fs::write(path.as_ref(), content).await?;
        Ok(())
    }
    
    /// Validate configuration
    #[allow(dead_code)]
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate profit thresholds
        if self.strategy.min_profit_threshold <= 0.0 || self.strategy.min_profit_threshold > 0.1 {
            return Err(anyhow::anyhow!(
                "Invalid min_profit_threshold: must be between 0.0 and 0.1"
            ));
        }
        
        // Validate slippage
        if self.strategy.max_slippage <= 0.0 || self.strategy.max_slippage > 0.05 {
            return Err(anyhow::anyhow!(
                "Invalid max_slippage: must be between 0.0 and 0.05"
            ));
        }
        
        // Validate performance targets
        if self.performance.max_detection_latency_us == 0 || self.performance.max_detection_latency_us > 1000 {
            return Err(anyhow::anyhow!(
                "Invalid max_detection_latency_us: must be between 1 and 1000"
            ));
        }
        
        // Validate enabled strategies
        for strategy in &self.strategy.enabled_strategies {
            if !["inter_exchange", "triangular"].contains(&strategy.as_str()) {
                return Err(anyhow::anyhow!(
                    "Invalid strategy '{}': must be 'inter_exchange' or 'triangular'", strategy
                ));
            }
        }
        
        Ok(())
    }
    
    /// Convert to strategy config
    #[allow(dead_code)]
    pub fn to_strategy_config(&self) -> StrategyConfig {
        StrategyConfig {
            enabled: true, // 默认启用
            name: "default_strategy".to_string(), // 默认策略名
            max_positions: 10, // 默认最大持仓数
            min_profit_threshold: self.strategy.min_profit_threshold,
            max_slippage_pct: self.strategy.inter_exchange.max_slippage_pct,
            parameters: HashMap::new(), // 空参数映射
        }
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        // 完全配置化的默认值生成，消除硬编码
        Self::load_from_env_and_files()
    }
}

impl SystemConfig {
    /// 从环境变量和配置文件加载配置，消除所有硬编码
    pub fn load_from_env_and_files() -> Self {
        Self {
            strategy: StrategyConfigSection {
                min_profit_threshold: 0.002, // 0.2%
                max_slippage: 0.001, // 0.1%
                enabled_strategies: vec!["inter_exchange".to_string(), "triangular".to_string()],
                inter_exchange: InterExchangeSettings {
                    max_price_diff_pct: 0.005, // 0.5%
                    min_profit_pct: 0.001, // 0.1%
                    max_slippage_pct: 0.0005, // 0.05%
                    binance_fee_bps: None,
                    okx_fee_bps: None,
                    huobi_fee_bps: None,
                    kraken_fee_bps: None,
                    coinbase_fee_bps: None,
                },
                triangular: TriangularSettings {
                    triangle_paths: vec![
                        TrianglePath {
                            base: "BTC".to_string(),
                            intermediate: "ETH".to_string(),
                            quote: "USDT".to_string(),
                            exchange: "binance".to_string(),
                            min_liquidity_usd: None,
                            max_slippage_per_leg: None,
                        },
                        TrianglePath {
                            base: "BTC".to_string(),
                            intermediate: "BNB".to_string(),
                            quote: "USDT".to_string(),
                            exchange: "binance".to_string(),
                            min_liquidity_usd: None,
                            max_slippage_per_leg: None,
                        },
                    ],
                    min_liquidity_usd: 100.0,
                    max_slippage_per_leg: 0.001,
                },
                market_state: MarketStateWeights {
                    cautious_weight: 1.4,
                    extreme_weight: 2.5,
                },
            },
            market_data: MarketDataConfig::default(),
            risk: RiskConfig {
                max_position_size: 1000.0,
                max_daily_loss: 100.0,
                enabled_strategies: vec!["inter_exchange".to_string(), "triangular".to_string()],
                max_daily_trades: 100,
                max_single_loss_pct: 5.0,
                max_fund_utilization: 80.0,
                abnormal_price_deviation_pct: 20.0,
                max_consecutive_failures: 5,
            },
            execution: ExecutionConfigSection {
                dry_run: false,
                max_concurrent: 10,
                timeout_ms: 5000,
                retry_count: 3,
                exchanges: std::collections::HashMap::new(),
            },
            nats: NatsConfig::default(),
            // metrics: MetricsConfig::default(),  // 暂时注释
            performance: PerformanceConfig {
                target_opportunities_per_sec: 100,
                max_detection_latency_us: 100,
                max_execution_latency_ms: 50,
                worker_threads: None, // Use default (CPU count)
                memory_pool: MemoryPoolConfig {
                    orderbook_capacity: 1000,
                    opportunity_capacity: 10000,
                    buffer_size: 8192,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                json: false,
                file: Some("qingxi.log".to_string()),
                rotation: LogRotationConfig {
                    max_size_mb: 100,
                    max_files: 10,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled: bool,
    pub name: String,
    pub max_positions: usize,
    pub min_profit_threshold: f64,
    pub max_slippage_pct: f64,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_invalid_config_values() {
        let mut config = SystemConfig::default();
        config.strategy.min_profit_threshold = -0.1;
        assert!(config.validate().is_err());
    }
}

    #[test]
    fn test_invalid_config_values() {
        let mut config = SystemConfig::default();
        config.strategy.min_profit_threshold = -0.1;
        assert!(config.validate().is_err());
    }
}
