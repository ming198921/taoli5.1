use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tokio::sync::RwLock;
use uuid::Uuid;
use common_types::ArbitrageOpportunity;

/// 策略执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyExecutionResult {
    pub strategy_id: String,
    pub opportunity_id: String,
    pub success: bool,
    pub profit_realized: f64,
    pub execution_time_ms: u64,
    pub slippage: f64,
    pub fees_paid: f64,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// 策略性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetrics {
    pub strategy_id: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub avg_execution_time_ms: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub consecutive_losses: u32,
    pub is_active: bool,
    pub failure_reason: Option<String>,
}

impl Default for StrategyMetrics {
    fn default() -> Self {
        Self {
            strategy_id: String::new(),
            total_executions: 0,
            successful_executions: 0,
            total_profit: 0.0,
            total_loss: 0.0,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            avg_execution_time_ms: 0.0,
            last_execution: None,
            consecutive_losses: 0,
            is_active: true,
            failure_reason: None,
        }
    }
}

/// 套利机会评估标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityEvaluation {
    pub opportunity_id: String,
    pub strategy_type: StrategyType,
    pub profit_estimate: f64,
    pub liquidity_score: f64,
    pub capital_requirement: f64,
    pub execution_delay_estimate_ms: u64,
    pub risk_exposure: f64,
    pub confidence_score: f64,
    pub weighted_score: f64,
    pub priority: OpportunityPriority,
    pub timestamp: DateTime<Utc>,
}

/// 策略类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyType {
    InterExchange,
    Triangular,
    // 预留扩展
    Statistical,
    CrossPair,
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::InterExchange => write!(f, "InterExchange"),
            StrategyType::Triangular => write!(f, "Triangular"),
            StrategyType::Statistical => write!(f, "Statistical"),
            StrategyType::CrossPair => write!(f, "CrossPair"),
        }
    }
}

/// 机会优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OpportunityPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// 策略错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyError {
    InsufficientLiquidity,
    InsufficientCapital,
    APIError(String),
    MarketClosed,
    RiskLimitExceeded,
    ConfigurationError(String),
    ExecutionTimeout,
    InvalidOpportunity,
    StrategyDisabled,
    ModelTrainingError(String),
    PredictionError(String),
    FeatureEngineeringError(String),
    ValidationError(String),
    InsufficientData(String),
    ModelNotFound(String),
}

impl std::fmt::Display for StrategyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyError::InsufficientLiquidity => write!(f, "Insufficient liquidity"),
            StrategyError::InsufficientCapital => write!(f, "Insufficient capital"),
            StrategyError::APIError(msg) => write!(f, "API error: {}", msg),
            StrategyError::MarketClosed => write!(f, "Market closed"),
            StrategyError::RiskLimitExceeded => write!(f, "Risk limit exceeded"),
            StrategyError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            StrategyError::ExecutionTimeout => write!(f, "Execution timeout"),
            StrategyError::InvalidOpportunity => write!(f, "Invalid opportunity"),
            StrategyError::StrategyDisabled => write!(f, "Strategy disabled"),
            StrategyError::ModelTrainingError(msg) => write!(f, "Model training error: {}", msg),
            StrategyError::PredictionError(msg) => write!(f, "Prediction error: {}", msg),
            StrategyError::FeatureEngineeringError(msg) => write!(f, "Feature engineering error: {}", msg),
            StrategyError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            StrategyError::InsufficientData(msg) => write!(f, "Insufficient data: {}", msg),
            StrategyError::ModelNotFound(msg) => write!(f, "Model not found: {}", msg),
        }
    }
}

impl std::error::Error for StrategyError {}

// 使用统一的MarketData定义替代重复的MarketDataSnapshot
pub use common_types::{MarketData, MarketDataSnapshot, AlignedMarketData};

impl ArbitrageOpportunity {
    pub fn new(strategy_type: StrategyType, symbol: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            strategy_type,
            symbol,
            buy_exchange: String::new(),
            sell_exchange: String::new(),
            buy_price: 0.0,
            sell_price: 0.0,
            quantity: 0.0,
            profit_estimate: 0.0,
            profit_percentage: 0.0,
            fees_estimate: 0.0,
            slippage_estimate: 0.0,
            execution_path: Vec::new(),
            capital_requirement: 0.0,
            risk_score: 0.0,
            confidence_score: 0.0,
            valid_until: Utc::now() + chrono::Duration::seconds(30),
            created_at: Utc::now(),
            market_data: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        Utc::now() < self.valid_until
    }

    pub fn net_profit(&self) -> f64 {
        self.profit_estimate - self.fees_estimate - self.slippage_estimate
    }
}

// The unified ArbitrageStrategy trait is now defined in common_types
// Use common_types::ArbitrageStrategy instead of defining it here

/// 资源需求定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_capital: f64,
    pub max_capital: f64,
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub api_rate_limit: u32,
    pub exchange_connections: Vec<String>,
}

/// 策略状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyStatus {
    Active,
    Paused,
    Disabled,
    Error,
    Maintenance,
}

/// 策略配置
/// Strategy configuration - using unified definition to eliminate duplication
/// The unified BaseStrategyConfig includes all the fields we need:
/// - id (maps to strategy_id)
/// - enabled 
/// - priority
/// - max_capital_allocation
/// - risk_parameters
/// - execution_parameters
/// - monitoring_enabled
/// - auto_recovery
/// And provides additional standardized fields for better consistency
pub use common_types::{BaseStrategyConfig as StrategyConfig, BaseConfig}; 
 
 

