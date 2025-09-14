use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tokio::sync::RwLock;
use uuid::Uuid;

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

/// 市场数据快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataSnapshot {
    pub symbol: String,
    pub exchange: String,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_quantity: f64,
    pub ask_quantity: f64,
    pub mid_price: f64,
    pub spread: f64,
    pub volume_24h: f64,
    pub timestamp: DateTime<Utc>,
    pub api_latency_ms: u64,
}

/// 套利机会定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunityCore {
    pub id: String,
    pub strategy_type: StrategyType,
    pub symbol: String,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub quantity: f64,
    pub profit_estimate: f64,
    pub profit_percentage: f64,
    pub fees_estimate: f64,
    pub slippage_estimate: f64,
    pub execution_path: Vec<String>,
    pub capital_requirement: f64,
    pub risk_score: f64,
    pub confidence_score: f64,
    pub valid_until: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub market_data: Vec<MarketDataSnapshot>,
}

impl ArbitrageOpportunityCore {
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

/// 核心策略Trait
pub trait ArbitrageStrategy: Send + Sync {
    /// 策略唯一标识
    fn strategy_id(&self) -> &str;
    
    /// 策略类型
    fn strategy_type(&self) -> StrategyType;
    
    /// 策略描述
    fn description(&self) -> &str;
    
    /// 检测套利机会
    fn detect_opportunities(
        &self,
        market_data: &HashMap<String, HashMap<String, MarketDataSnapshot>>,
        min_profit: f64,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ArbitrageOpportunityCore>, StrategyError>> + Send + '_>>;
    
    /// 评估机会质量
    fn evaluate_opportunity(
        &self,
        opportunity: &ArbitrageOpportunityCore,
    ) -> Pin<Box<dyn Future<Output = Result<OpportunityEvaluation, StrategyError>> + Send + '_>>;
    
    /// 执行套利
    fn execute_opportunity(
        &self,
        opportunity: &ArbitrageOpportunityCore,
    ) -> Pin<Box<dyn Future<Output = Result<StrategyExecutionResult, StrategyError>> + Send + '_>>;
    
    /// 获取策略配置
    fn get_config(&self) -> HashMap<String, serde_json::Value>;
    
    /// 更新策略配置
    fn update_config(
        &mut self, 
        config: HashMap<String, serde_json::Value>
    ) -> Pin<Box<dyn Future<Output = Result<(), StrategyError>> + Send + '_>>;
    
    /// 策略健康检查
    fn health_check(&self) -> Pin<Box<dyn Future<Output = Result<(), StrategyError>> + Send + '_>>;
    
    /// 获取资源需求
    fn resource_requirements(&self) -> ResourceRequirements;
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub strategy_id: String,
    pub enabled: bool,
    pub priority: u32,
    pub max_capital_allocation: f64,
    pub risk_parameters: HashMap<String, f64>,
    pub execution_parameters: HashMap<String, serde_json::Value>,
    pub monitoring_enabled: bool,
    pub auto_recovery: bool,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            strategy_id: String::new(),
            enabled: true,
            priority: 1,
            max_capital_allocation: 10000.0,
            risk_parameters: HashMap::new(),
            execution_parameters: HashMap::new(),
            monitoring_enabled: true,
            auto_recovery: true,
        }
    }
} 
 
 

