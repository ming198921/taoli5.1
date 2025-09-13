//! 统一的数据类型定义 - 重新导出架构版本的权威定义
//! 此模块提供整个系统的权威数据结构定义，确保前后端完美互通

pub mod config;
// 暂时禁用有编译错误的模块，专注于核心优化
// pub mod config_migration;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use thiserror::Error;

/// 交易对符号类型
pub type Symbol = String;

/// 交易所名称类型  
pub type Exchange = String;

// ================== UNIFIED STRATEGY TRAIT DEFINITIONS ==================

/// The kind of an arbitrage strategy - unified definition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyKind {
    /// Cross-exchange arbitrage
    InterExchange,
    /// Triangular arbitrage 
    Triangular,
    /// Future-spot arbitrage
    FuturesSpot,
    /// Calendar spread arbitrage
    CalendarSpread,
    /// Statistical arbitrage
    Statistical,
    /// Market making
    MarketMaking,
}

impl StrategyKind {
    pub fn requires_multiple_exchanges(&self) -> bool {
        matches!(self, 
            StrategyKind::InterExchange | 
            StrategyKind::FuturesSpot
        )
    }
    
    pub fn is_high_frequency(&self) -> bool {
        matches!(self, 
            StrategyKind::InterExchange | 
            StrategyKind::Triangular |
            StrategyKind::MarketMaking
        )
    }
}

/// Unified strategy errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum StrategyError {
    #[error("Execution failed on exchange: {0}")]
    ExecutionFailed(String),
    #[error("Risk check failed: {0}")]
    RiskCheckFailed(String),
    #[error("Insufficient funds for trade")]
    InsufficientFunds,
    #[error("Opportunity is no longer valid (TTL expired)")]
    OpportunityExpired,
    #[error("Insufficient liquidity")]
    InsufficientLiquidity,
    #[error("API error: {0}")]
    APIError(String),
    #[error("Market closed")]
    MarketClosed,
    #[error("Risk limit exceeded")]
    RiskLimitExceeded,
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Execution timeout")]
    ExecutionTimeout,
    #[error("Invalid opportunity")]
    InvalidOpportunity,
    #[error("Strategy disabled")]
    StrategyDisabled,
}

/// The result of a strategy execution attempt - unified definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub accepted: bool,
    pub reason: Option<String>,
    pub order_ids: Vec<String>,
    pub executed_quantity: f64,
    pub realized_profit: f64,
    pub execution_time_ms: u64,
    pub slippage: f64,
    pub fees_paid: f64,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self {
            accepted: false,
            reason: None,
            order_ids: Vec::new(),
            executed_quantity: 0.0,
            realized_profit: 0.0,
            execution_time_ms: 0,
            slippage: 0.0,
            fees_paid: 0.0,
        }
    }
}

/// Fee precision repository trait for dynamic fee calculation
pub trait FeePrecisionRepo: Send + Sync {
    fn get_fee(&self, exchange: &str, fee_type: &str) -> Option<f64>;
}

/// Strategy context interface - for dependency injection
pub trait StrategyContext: Send + Sync {
    fn get_config(&self) -> HashMap<String, serde_json::Value>;
    fn min_profit_threshold(&self) -> f64;
    fn max_position_size(&self, symbol: &str) -> f64;
    fn exchange_fee(&self, exchange: &str) -> f64;
    
    // 扩展方法支持现有代码
    fn current_min_profit_pct(&self) -> f64 {
        self.min_profit_threshold()
    }
    
    fn get_taker_fee(&self, exchange: &str) -> f64 {
        self.exchange_fee(exchange)
    }
    
    fn get_maker_fee(&self, exchange: &str) -> f64 {
        self.exchange_fee(exchange) * 0.8  // 通常maker费率更低
    }
    
    // 访问器方法，避免直接字段访问
    fn fee_precision_repo(&self) -> Option<&dyn FeePrecisionRepo> {
        None  // 默认实现返回None
    }
    
    // 滑点配置访问方法
    fn inter_exchange_slippage_per_leg_pct(&self) -> f64 {
        0.001  // 默认0.1%滑点
    }
}

/// Normalized market data snapshot for strategy input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSnapshot {
    pub symbol: String,
    pub exchanges: HashMap<String, ExchangeSnapshot>,
    pub timestamp_ns: u64,
}

/// Exchange-specific snapshot data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeSnapshot {
    pub exchange: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_quantity: f64,
    pub ask_quantity: f64,
    pub mid_price: f64,
    pub spread: f64,
    pub volume_24h: f64,
    pub timestamp_ns: u64,
    pub latency_ms: u64,
}

/// THE UNIFIED ARBITRAGE STRATEGY TRAIT
/// This consolidates all strategy definitions across the codebase
#[async_trait]
pub trait ArbitrageStrategy: Send + Sync {
    /// A unique, static name for the strategy.
    fn name(&self) -> &'static str;

    /// The kind of the strategy.
    fn kind(&self) -> StrategyKind;

    /// Strategy description for logging/monitoring
    fn description(&self) -> &str {
        "Generic arbitrage strategy"
    }

    /// Detects an arbitrage opportunity from a normalized market snapshot.
    ///
    /// This is the **hot path**. It must be synchronous, non-blocking, and avoid
    /// allocations to meet performance targets (<= 100µs).
    fn detect(
        &self,
        ctx: &dyn StrategyContext,
        input: &NormalizedSnapshot,
    ) -> Option<ArbitrageOpportunity>;

    /// Executes a detected arbitrage opportunity.
    ///
    /// This path is asynchronous as it may involve I/O (e.g., sending orders).
    async fn execute(
        &self,
        ctx: &dyn StrategyContext,
        opp: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError>;

    /// Evaluates the quality of an opportunity (optional advanced method)
    async fn evaluate_opportunity(
        &self,
        _ctx: &dyn StrategyContext,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<f64, StrategyError> {
        // Default implementation returns confidence score
        Ok(opportunity.confidence_score)
    }

    /// Health check for the strategy
    async fn health_check(&self) -> Result<(), StrategyError> {
        Ok(())
    }

    /// Get strategy configuration
    fn get_config(&self) -> HashMap<String, serde_json::Value> {
        HashMap::new()
    }

    /// Update strategy configuration (mutable strategies only)
    async fn update_config(
        &mut self, 
        _config: HashMap<String, serde_json::Value>
    ) -> Result<(), StrategyError> {
        Err(StrategyError::ConfigurationError("Configuration updates not supported".to_string()))
    }

    /// Get resource requirements for this strategy
    fn resource_requirements(&self) -> ResourceRequirements {
        ResourceRequirements::default()
    }
}

/// Resource requirements for strategy execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_capital: f64,
    pub max_capital: f64,
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub api_rate_limit: u32,
    pub exchange_connections: Vec<String>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_capital: 100.0,
            max_capital: 10000.0,
            cpu_cores: 1,
            memory_mb: 512,
            api_rate_limit: 100,
            exchange_connections: Vec::new(),
        }
    }
}

/// 机会状态枚举 - 与前端完全匹配
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OpportunityStatus {
    Active,
    Executed, 
    Expired,
    Cancelled,
}

impl Default for OpportunityStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// 复杂的套利机会结构体 - 包含风险评分、流动性等高级字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub strategy_type: StrategyType,
    pub exchange_pair: Option<(String, String)>,  // 跨交易所套利
    pub triangle_path: Option<Vec<String>>,       // 三角套利路径
    pub symbol: String,
    pub estimated_profit: f64,                    // 估算利润
    pub net_profit: f64,                         // 净利润（扣除手续费）
    pub profit_bps: f64,                         // 利润基点
    pub liquidity_score: f64,                    // 流动性评分 0-1
    pub confidence_score: f64,                   // 置信度评分 0-1
    pub estimated_latency_ms: u64,               // 预估执行延迟
    pub risk_score: f64,                         // 风险评分 0-1
    pub required_funds: HashMap<String, serde_json::Value>, // 所需资金
    pub market_impact: f64,                      // 市场冲击成本
    pub slippage_estimate: f64,                  // 滑点估算
    pub timestamp: String,                       // ISO 8601格式
    pub expires_at: String,                      // ISO 8601格式  
    pub priority: u8,                            // 优先级 0-255
    pub tags: Vec<String>,                       // 标签
    pub status: OpportunityStatus,               // 机会状态
    /// 额外的元数据字段
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 策略类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyType {
    /// 跨交易所套利
    InterExchangeArbitrage,
    /// 三角套利
    TriangularArbitrage,
    /// 期货现货套利
    FuturesSpotArbitrage,
    /// 跨期套利
    CalendarSpreadArbitrage,
    /// 统计套利
    StatisticalArbitrage,
    /// 做市商策略
    MarketMaking,
    /// 动量策略
    Momentum,
    /// 均值回归策略
    MeanReversion,
    /// 网格策略
    GridTrading,
    /// 自定义策略
    Custom(String),
}

impl StrategyType {
    pub fn requires_multiple_exchanges(&self) -> bool {
        matches!(self, 
            StrategyType::InterExchangeArbitrage | 
            StrategyType::FuturesSpotArbitrage
        )
    }
    
    pub fn is_high_frequency(&self) -> bool {
        matches!(self, 
            StrategyType::InterExchangeArbitrage | 
            StrategyType::TriangularArbitrage |
            StrategyType::MarketMaking
        )
    }
    
    pub fn default_min_profit_bps(&self) -> f64 {
        match self {
            StrategyType::InterExchangeArbitrage => 5.0,      // 5个基点
            StrategyType::TriangularArbitrage => 3.0,         // 3个基点
            StrategyType::FuturesSpotArbitrage => 10.0,       // 10个基点
            StrategyType::CalendarSpreadArbitrage => 15.0,    // 15个基点
            StrategyType::StatisticalArbitrage => 20.0,       // 20个基点
            StrategyType::MarketMaking => 2.0,                // 2个基点
            StrategyType::Momentum => 30.0,                   // 30个基点
            StrategyType::MeanReversion => 25.0,              // 25个基点
            StrategyType::GridTrading => 10.0,                // 10个基点
            StrategyType::Custom(_) => 10.0,                  // 默认10个基点
        }
    }
}

impl ArbitrageOpportunity {
    pub fn is_expired(&self) -> bool {
        if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(&self.expires_at) {
            chrono::Utc::now() > expires_at
        } else {
            true // 如果解析失败，认为已过期
        }
    }
    
    pub fn age_ms(&self) -> i64 {
        if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&self.timestamp) {
            let now = chrono::Utc::now();
            let timestamp_utc = timestamp.with_timezone(&chrono::Utc);
            (now - timestamp_utc).num_milliseconds().max(0)
        } else {
            0
        }
    }
    
    pub fn time_to_expiry_ms(&self) -> i64 {
        if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(&self.expires_at) {
            let expires_at_utc = expires_at.with_timezone(&chrono::Utc);
            (expires_at_utc - chrono::Utc::now()).num_milliseconds().max(0)
        } else {
            0
        }
    }
    
    pub fn is_profitable(&self, min_profit_bps: f64) -> bool {
        self.profit_bps >= min_profit_bps
    }
    
    pub fn risk_adjusted_return(&self) -> f64 {
        if self.risk_score > 0.0 {
            self.net_profit / self.risk_score
        } else {
            self.net_profit
        }
    }

    /// 获取净利润百分比 (兼容旧代码) 
    pub fn net_profit_pct(&self) -> f64 {
        self.profit_bps / 10000.0  // 转换基点为小数格式
    }

    /// 设置机会状态
    pub fn set_status(&mut self, status: OpportunityStatus) {
        self.status = status;
    }

    /// 设置额外的元数据
    pub fn set_metadata(&mut self, key: &str, value: serde_json::Value) {
        self.metadata.insert(key.to_string(), value);
    }

    /// 获取expires_at作为DateTime (兼容旧代码)
    pub fn expires_at_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::parse_from_rfc3339(&self.expires_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|_| chrono::Utc::now())
    }

    /// 获取模拟的legs (兼容旧代码)
    /// 注意：这只是为了兼容旧的风控代码，新代码应该使用标准字段
    pub fn legs(&self) -> Vec<LegSimulation> {
        let mut legs = Vec::new();
        
        // 从exchange_pair构造基本的买卖legs
        if let Some((buy_exchange, sell_exchange)) = &self.exchange_pair {
            legs.push(LegSimulation {
                exchange: buy_exchange.clone(),
                price: self.estimated_profit / 2.0,  // 简化计算
                quantity: 1.0, // 默认数量
                side: "buy".to_string(),
            });
            legs.push(LegSimulation {
                exchange: sell_exchange.clone(),
                price: (self.estimated_profit / 2.0) + self.estimated_profit,  // 简化计算
                quantity: 1.0, // 默认数量
                side: "sell".to_string(),
            });
        }
        
        // 如果有三角套利路径，构造更复杂的legs
        if let Some(triangle_path) = &self.triangle_path {
            legs.clear();
            for (i, exchange) in triangle_path.iter().enumerate() {
                let side = if i % 2 == 0 { "buy" } else { "sell" };
                legs.push(LegSimulation {
                    exchange: exchange.clone(),
                    price: self.estimated_profit / triangle_path.len() as f64,
                    quantity: 1.0,
                    side: side.to_string(),
                });
            }
        }
        
        legs
    }

    /// 创建新的套利机会
    pub fn new(
        id: String,
        symbol: String,
        buy_exchange: String,
        sell_exchange: String,
        buy_price: f64,
        sell_price: f64,
        volume_available: f64,
        ttl_ms: u64,
    ) -> Self {
        let now = chrono::Utc::now();
        let estimated_profit = (sell_price - buy_price) * volume_available;
        let profit_bps = if buy_price > 0.0 {
            ((sell_price - buy_price) / buy_price) * 10000.0 // 基点
        } else {
            0.0
        };
        
        Self {
            id,
            strategy_type: StrategyType::InterExchangeArbitrage,
            exchange_pair: Some((buy_exchange, sell_exchange)),
            triangle_path: None,
            symbol,
            estimated_profit,
            net_profit: estimated_profit,
            profit_bps,
            liquidity_score: 0.8, // 默认中等流动性
            confidence_score: 0.9, // 默认高置信度
            estimated_latency_ms: 100, // 默认100ms延迟
            risk_score: 0.3, // 默认低风险
            required_funds: HashMap::new(),
            market_impact: 0.01,
            slippage_estimate: 0.005,
            timestamp: now.to_rfc3339(),
            expires_at: (now + chrono::Duration::milliseconds(ttl_ms as i64)).to_rfc3339(),
            priority: 128,
            tags: Vec::new(),
            status: OpportunityStatus::Active,
            metadata: HashMap::new(),
        }
    }

    /// 创建三角套利机会 (兼容triangular.rs的new_with_legs调用)
    pub fn new_triangular(
        strategy_name: &str,
        legs: Vec<LegSimulation>,
        net_profit_usd: f64,
        net_profit_pct: f64,
        timestamp_ns: u64,
    ) -> Self {
        // 从legs中提取关键信息来构造统一的ArbitrageOpportunity
        let triangle_path = legs.iter().map(|leg| leg.exchange.clone()).collect::<Vec<_>>();
        let symbol = format!("TRIANGULAR_{}", legs.first().map(|l| l.exchange.as_str()).unwrap_or("UNKNOWN"));
        let _total_volume: f64 = legs.iter().map(|leg| leg.quantity).sum();
        
        // 将纳秒时间戳转换为秒
        let timestamp_s = timestamp_ns / 1_000_000_000;
        let detected_at = chrono::DateTime::from_timestamp(timestamp_s as i64, (timestamp_ns % 1_000_000_000) as u32)
            .unwrap_or_else(|| chrono::Utc::now());
        
        Self {
            id: format!("{}_{}", strategy_name, uuid::Uuid::new_v4()),
            strategy_type: StrategyType::TriangularArbitrage,
            exchange_pair: None,
            triangle_path: Some(triangle_path),
            symbol,
            estimated_profit: net_profit_usd,
            net_profit: net_profit_usd,
            profit_bps: net_profit_pct * 10000.0, // 转换为基点
            liquidity_score: 0.7, // 三角套利流动性通常较低
            confidence_score: 0.8,
            estimated_latency_ms: 150, // 三角套利通常延迟更高
            risk_score: 0.4, // 风险稍高
            required_funds: HashMap::new(),
            market_impact: 0.02,
            slippage_estimate: 0.01,
            timestamp: detected_at.to_rfc3339(),
            expires_at: (detected_at + chrono::Duration::milliseconds(150)).to_rfc3339(),
            priority: 120,
            tags: vec![strategy_name.to_string()],
            status: OpportunityStatus::Active,
            metadata: HashMap::new(),
        }
    }
}

/// API响应包装器 - 与前端ApiResponse类型匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub timestamp: Option<i64>, // Unix timestamp in milliseconds - 与前端完全匹配
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
            timestamp: Some(chrono::Utc::now().timestamp_millis()),
        }
    }

    /// 创建错误响应
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.clone()),
            message: Some(message),
            timestamp: Some(chrono::Utc::now().timestamp_millis()),
        }
    }
}

/// 系统状态信息 - 与前端SystemStatus类型匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub active_opportunities: u32,
    pub total_processed: u64,
    pub error_count: u32,
    pub last_update: String,
}

/// 风险警报 - 与前端RiskAlert类型匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlert {
    pub id: String,
    pub level: AlertLevel,
    pub message: String,
    pub created_at: String,
    pub acknowledged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Low,
    Medium,
    High,
    Critical,
}

// ================== UNIFIED CONFIGURATION SYSTEM ==================

/// Base configuration trait - all configuration types should implement this
pub trait BaseConfig: Send + Sync + Clone {
    /// Validate the configuration
    fn validate(&self) -> Result<(), ConfigError>;
    
    /// Get configuration version/hash for change detection
    fn version(&self) -> String {
        // Default implementation uses a simple hash
        format!("{:?}", std::ptr::addr_of!(*self))
    }
    
    /// Merge with another configuration (for dynamic updates)
    fn merge(&mut self, other: &Self) -> Result<(), ConfigError>;
}

/// Configuration validation errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    #[error("Invalid value for field {field}: {reason}")]
    InvalidValue { field: String, reason: String },
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(String),
    #[error("Configuration conflict: {0}")]
    Conflict(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Base strategy configuration - consolidates all StrategyConfig duplicates
/// This replaces the duplicated StrategyConfig structs across the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseStrategyConfig {
    // Core identification fields
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
    
    // Financial parameters
    pub min_profit_threshold: f64,
    pub max_profit_threshold: Option<f64>,
    pub max_position_size: f64,
    pub max_capital_allocation: f64,
    pub max_risk_exposure: f64,
    
    // Runtime configuration
    pub priority: u32,
    pub monitoring_enabled: bool,
    pub auto_recovery: bool,
    
    // Market configuration
    pub symbols: Vec<String>,
    pub exchanges: Vec<String>,
    
    // Risk management
    pub risk_level: String,
    pub risk_parameters: HashMap<String, f64>,
    
    // Execution configuration
    pub execution_parameters: HashMap<String, serde_json::Value>,
    
    // Custom parameters for strategy-specific configuration
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for BaseStrategyConfig {
    fn default() -> Self {
        Self {
            id: format!("strategy_{}", chrono::Utc::now().timestamp_millis()),
            name: "default_strategy".to_string(),
            version: "1.0.0".to_string(),
            enabled: true,
            min_profit_threshold: 0.001,
            max_profit_threshold: None,
            max_position_size: 1000.0,
            max_capital_allocation: 10000.0,
            max_risk_exposure: 10000.0,
            priority: 1,
            monitoring_enabled: true,
            auto_recovery: true,
            symbols: vec!["BTC/USDT".to_string(), "ETH/USDT".to_string()],
            exchanges: vec!["binance".to_string(), "okx".to_string()],
            risk_level: "medium".to_string(),
            risk_parameters: HashMap::new(),
            execution_parameters: HashMap::new(),
            parameters: HashMap::new(),
        }
    }
}

impl BaseConfig for BaseStrategyConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.name.is_empty() {
            return Err(ConfigError::RequiredFieldMissing("name".to_string()));
        }
        if self.min_profit_threshold < 0.0 {
            return Err(ConfigError::InvalidValue {
                field: "min_profit_threshold".to_string(),
                reason: "Must be non-negative".to_string(),
            });
        }
        if self.max_position_size <= 0.0 {
            return Err(ConfigError::InvalidValue {
                field: "max_position_size".to_string(),
                reason: "Must be positive".to_string(),
            });
        }
        if self.exchanges.is_empty() {
            return Err(ConfigError::RequiredFieldMissing("exchanges".to_string()));
        }
        Ok(())
    }
    
    fn merge(&mut self, other: &Self) -> Result<(), ConfigError> {
        // Update fields that can be safely merged
        if !other.name.is_empty() && other.name != "default_strategy" {
            self.name = other.name.clone();
        }
        if other.enabled != self.enabled {
            self.enabled = other.enabled;
        }
        if other.min_profit_threshold != 0.001 {
            self.min_profit_threshold = other.min_profit_threshold;
        }
        if other.max_position_size != 1000.0 {
            self.max_position_size = other.max_position_size;
        }
        
        // Merge parameters
        for (key, value) in &other.parameters {
            self.parameters.insert(key.clone(), value.clone());
        }
        
        // Merge risk parameters
        for (key, value) in &other.risk_parameters {
            self.risk_parameters.insert(key.clone(), *value);
        }
        
        // Merge execution parameters
        for (key, value) in &other.execution_parameters {
            self.execution_parameters.insert(key.clone(), value.clone());
        }
        
        Ok(())
    }
}

/// Base exchange configuration - consolidates all ExchangeConfig duplicates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseExchangeConfig {
    // Core identification
    pub name: String,
    pub enabled: bool,
    
    // API configuration
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_passphrase: Option<String>,
    pub sandbox_mode: bool,
    
    // Connection configuration
    pub rest_url: String,
    pub ws_url: String,
    pub websocket_url: String, // Alternative field name for compatibility
    
    // Rate limiting and performance
    pub rate_limit: u32,
    pub max_connections: Option<u32>,
    
    // Trading configuration
    pub supported_symbols: Vec<String>,
    pub symbols: Vec<String>, // Alternative field name for compatibility
    
    // Fee configuration
    pub taker_fee: Option<f64>,
    pub maker_fee: Option<f64>,
    pub fee_rate_bps: Option<f64>,
    
    // Custom parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for BaseExchangeConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            enabled: true,
            api_key: None,
            api_secret: None,
            api_passphrase: None,
            sandbox_mode: true,
            rest_url: "".to_string(),
            ws_url: "".to_string(),
            websocket_url: "".to_string(),
            rate_limit: 10,
            max_connections: Some(5),
            supported_symbols: Vec::new(),
            symbols: Vec::new(),
            taker_fee: Some(0.001),
            maker_fee: Some(0.001),
            fee_rate_bps: Some(10.0),
            parameters: HashMap::new(),
        }
    }
}

impl BaseConfig for BaseExchangeConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.name.is_empty() {
            return Err(ConfigError::RequiredFieldMissing("name".to_string()));
        }
        if self.enabled && self.api_key.is_none() {
            return Err(ConfigError::RequiredFieldMissing("api_key".to_string()));
        }
        if self.enabled && self.api_secret.is_none() {
            return Err(ConfigError::RequiredFieldMissing("api_secret".to_string()));
        }
        if self.rate_limit == 0 {
            return Err(ConfigError::InvalidValue {
                field: "rate_limit".to_string(),
                reason: "Must be positive".to_string(),
            });
        }
        Ok(())
    }
    
    fn merge(&mut self, other: &Self) -> Result<(), ConfigError> {
        if !other.name.is_empty() && other.name != "default" {
            self.name = other.name.clone();
        }
        self.enabled = other.enabled;
        
        if other.api_key.is_some() {
            self.api_key = other.api_key.clone();
        }
        if other.api_secret.is_some() {
            self.api_secret = other.api_secret.clone();
        }
        if !other.rest_url.is_empty() {
            self.rest_url = other.rest_url.clone();
        }
        if !other.ws_url.is_empty() {
            self.ws_url = other.ws_url.clone();
        }
        
        // Merge parameters
        for (key, value) in &other.parameters {
            self.parameters.insert(key.clone(), value.clone());
        }
        
        Ok(())
    }
}

/// Legacy StrategyConfig type alias for backward compatibility
/// Use BaseStrategyConfig for new code
pub type StrategyConfig = BaseStrategyConfig;

/// Legacy ExchangeConfig type alias for backward compatibility  
/// Use BaseExchangeConfig for new code
pub type UnifiedExchangeConfig = BaseExchangeConfig;

/// 腿部模拟结构体 - 用于兼容旧的legs访问模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegSimulation {
    pub exchange: String,
    pub price: f64,
    pub quantity: f64,
    pub side: String,
}

impl LegSimulation {
    /// 计算成本 (兼容旧的cost访问模式)
    pub fn cost(&self) -> CostSimulation {
        CostSimulation {
            value: self.price * self.quantity,
        }
    }
}

/// 成本模拟结构体 - 用于兼容旧的cost访问模式
#[derive(Debug, Clone)]
pub struct CostSimulation {
    pub value: f64,
}

impl CostSimulation {
    pub fn to_f64(&self) -> f64 {
        self.value
    }
}

// ================== UNIFIED MARKET DATA SYSTEM ==================

/// 统一的市场数据结构体 - 系统权威定义
/// 这个结构体整合了所有MarketData变体的字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    /// 交易对符号 (如 "BTC/USDT")
    pub symbol: String,
    /// 交易所名称
    pub exchange: String,
    /// 时间戳 (毫秒级Unix时间戳)
    pub timestamp: u64,
    /// 买一价
    pub bid_price: f64,
    /// 卖一价
    pub ask_price: f64,
    /// 买一量
    pub bid_volume: f64,
    /// 卖一量
    pub ask_volume: f64,
    /// 最新成交价
    pub last_price: f64,
    /// 24小时成交量
    pub volume_24h: f64,
    /// 24小时涨跌幅
    pub change_24h: f64,
    /// 额外的元数据字段
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            exchange: String::new(),
            timestamp: 0,
            bid_price: 0.0,
            ask_price: 0.0,
            bid_volume: 0.0,
            ask_volume: 0.0,
            last_price: 0.0,
            volume_24h: 0.0,
            change_24h: 0.0,
            metadata: HashMap::new(),
        }
    }
}

impl MarketData {
    /// 计算中间价
    pub fn mid_price(&self) -> f64 {
        if self.bid_price > 0.0 && self.ask_price > 0.0 {
            (self.bid_price + self.ask_price) / 2.0
        } else {
            self.last_price
        }
    }
    
    /// 计算价差
    pub fn spread(&self) -> f64 {
        if self.ask_price > self.bid_price {
            self.ask_price - self.bid_price
        } else {
            0.0
        }
    }
    
    /// 计算价差百分比
    pub fn spread_pct(&self) -> f64 {
        let mid = self.mid_price();
        if mid > 0.0 {
            self.spread() / mid * 100.0
        } else {
            0.0
        }
    }
    
    /// 检查数据是否有效
    pub fn is_valid(&self) -> bool {
        !self.symbol.is_empty() 
            && !self.exchange.is_empty()
            && self.bid_price > 0.0 
            && self.ask_price > 0.0
            && self.ask_price >= self.bid_price
            && self.timestamp > 0
    }
    
    /// 检查数据是否过期 (默认30秒过期)
    pub fn is_stale(&self, max_age_ms: u64) -> bool {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        now.saturating_sub(self.timestamp) > max_age_ms
    }
}

/// 对齐的市场数据 - 用于高频交易优化
#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct AlignedMarketData {
    pub data: MarketData,
    /// 填充字节确保64字节对齐
    pub _padding: [u8; 0],
}

impl From<MarketData> for AlignedMarketData {
    fn from(data: MarketData) -> Self {
        Self {
            data,
            _padding: [],
        }
    }
}

impl std::ops::Deref for AlignedMarketData {
    type Target = MarketData;
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for AlignedMarketData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// 市场数据快照 - 类型别名
pub type MarketDataSnapshot = MarketData;

/// 清理后的市场数据 - 类型别名
pub type CleanedMarketData = MarketData;

/// 原始市场数据 - 类型别名
pub type RawMarketData = MarketData;

/// 市场数据输入 - 类型别名
pub type MarketDataInput = MarketData;

/// Celue市场数据 - 类型别名
pub type CelueMarketData = MarketData;

/// 前端市场数据 - 类型别名
pub type FrontendMarketData = MarketData;

/// 市场数据事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataEvent {
    /// 事件类型
    pub event_type: String,
    /// 市场数据
    pub market_data: MarketData,
    /// 事件时间戳
    pub event_timestamp: u64,
}

/// 交易所市场数据 - 扩展版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeMarketData {
    /// 基础市场数据
    #[serde(flatten)]
    pub market_data: MarketData,
    /// 订单簿深度数据
    pub order_book: Option<OrderBookData>,
    /// 交易历史
    pub recent_trades: Vec<TradeData>,
    /// 延迟信息
    pub latency_ms: u64,
}

/// 订单簿数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookData {
    pub bids: Vec<(f64, f64)>, // (价格, 数量)
    pub asks: Vec<(f64, f64)>, // (价格, 数量)
    pub timestamp: u64,
}

/// 交易数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeData {
    pub price: f64,
    pub volume: f64,
    pub side: String, // "buy" or "sell"
    pub timestamp: u64,
}

// 重新导出常用类型，便于其他模块使用
pub use OpportunityStatus::*;
pub use config::*;
// pub use config_migration::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::success(vec!["test_data"]);
        let json = serde_json::to_string(&response).expect("序列化失败");
        
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":[\"test_data\"]"));
    }
}