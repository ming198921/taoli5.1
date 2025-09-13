//! 核心数据类型定义模块
//! 
//! 定义系统中使用的所有核心数据结构和枚举类型

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// 交易所类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExchangeType {
    Binance,
    Okx,
    Huobi,
    Bybit,
    Gate,
    Kucoin,
    Coinbase,
    Kraken,
    Bitfinex,
    Custom(u8), // 自定义交易所类型
}

impl ExchangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExchangeType::Binance => "binance",
            ExchangeType::Okx => "okx",
            ExchangeType::Huobi => "huobi",
            ExchangeType::Bybit => "bybit",
            ExchangeType::Gate => "gate",
            ExchangeType::Kucoin => "kucoin",
            ExchangeType::Coinbase => "coinbase",
            ExchangeType::Kraken => "kraken",
            ExchangeType::Bitfinex => "bitfinex",
            ExchangeType::Custom(_) => "custom",
        }
    }
    
    pub fn supports_spot(&self) -> bool {
        matches!(self, 
            ExchangeType::Binance | ExchangeType::Okx | ExchangeType::Huobi |
            ExchangeType::Bybit | ExchangeType::Gate | ExchangeType::Kucoin |
            ExchangeType::Coinbase | ExchangeType::Kraken | ExchangeType::Bitfinex
        )
    }
    
    pub fn supports_futures(&self) -> bool {
        matches!(self, 
            ExchangeType::Binance | ExchangeType::Okx | ExchangeType::Huobi |
            ExchangeType::Bybit | ExchangeType::Gate
        )
    }
}

/// 重新导出统一的StrategyType定义
pub use common_types::StrategyType;

/// 市场状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketState {
    /// 正常市场状态
    Normal,
    /// 谨慎市场状态
    Caution,
    /// 极端市场状态
    Extreme,
    /// 市场关闭
    Closed,
    /// 维护中
    Maintenance,
}

impl MarketState {
    pub fn risk_multiplier(&self) -> f64 {
        match self {
            MarketState::Normal => 1.0,
            MarketState::Caution => 1.5,
            MarketState::Extreme => 3.0,
            MarketState::Closed => 0.0,
            MarketState::Maintenance => 0.0,
        }
    }
    
    pub fn min_profit_multiplier(&self) -> f64 {
        match self {
            MarketState::Normal => 1.0,
            MarketState::Caution => 1.3,
            MarketState::Extreme => 2.0,
            MarketState::Closed => f64::INFINITY,
            MarketState::Maintenance => f64::INFINITY,
        }
    }
}

/// 订单类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
    StopLossLimit,
    TakeProfitLimit,
    TrailingStop,
    IcebergOrder,
    TimeInForce,
}

/// 订单方向枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 订单状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

/// 风险级别枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    Extreme,
}

impl RiskLevel {
    pub fn max_position_ratio(&self) -> f64 {
        match self {
            RiskLevel::VeryLow => 0.1,   // 10%
            RiskLevel::Low => 0.25,      // 25%
            RiskLevel::Medium => 0.5,    // 50%
            RiskLevel::High => 0.75,     // 75%
            RiskLevel::VeryHigh => 0.9,  // 90%
            RiskLevel::Extreme => 1.0,   // 100%
        }
    }
}

/// 货币对信息
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradingPair {
    pub base: String,
    pub quote: String,
    pub symbol: String,
}

impl TradingPair {
    pub fn new(base: &str, quote: &str) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
            symbol: format!("{}/{}", base.to_uppercase(), quote.to_uppercase()),
        }
    }
    
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        if let Some((base, quote)) = symbol.split_once('/') {
            Some(Self::new(base, quote))
        } else if let Some((base, quote)) = symbol.split_once('-') {
            Some(Self::new(base, quote))
        } else {
            None
        }
    }
    
    pub fn reverse(&self) -> Self {
        Self::new(&self.quote, &self.base)
    }
}

/// 价格数据
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Price {
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub source: &'static str,
}

impl Price {
    pub fn new(value: f64, source: &'static str) -> Self {
        Self {
            value,
            timestamp: Utc::now(),
            source,
        }
    }
    
    pub fn age_ms(&self) -> i64 {
        (Utc::now() - self.timestamp).num_milliseconds()
    }
    
    pub fn is_stale(&self, max_age_ms: i64) -> bool {
        self.age_ms() > max_age_ms
    }
}

/// 深度数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDepth {
    pub bids: Vec<(f64, f64)>, // (价格, 数量)
    pub asks: Vec<(f64, f64)>, // (价格, 数量)
    pub timestamp: DateTime<Utc>,
    pub exchange: String,
    pub symbol: String,
}

impl OrderBookDepth {
    pub fn best_bid(&self) -> Option<f64> {
        self.bids.first().map(|(price, _)| *price)
    }
    
    pub fn best_ask(&self) -> Option<f64> {
        self.asks.first().map(|(price, _)| *price)
    }
    
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }
    
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }
    
    pub fn spread_bps(&self) -> Option<f64> {
        match (self.spread(), self.mid_price()) {
            (Some(spread), Some(mid)) if mid > 0.0 => Some(spread / mid * 10000.0),
            _ => None,
        }
    }
}

/// 成交数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub price: f64,
    pub quantity: f64,
    pub side: OrderSide,
    pub timestamp: DateTime<Utc>,
    pub exchange: String,
    pub symbol: String,
}

/// K线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
    pub interval: String,
}

/// 账户余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
    pub total: f64,
    pub exchange: String,
    pub timestamp: DateTime<Utc>,
}

impl Balance {
    pub fn new(asset: &str, free: f64, locked: f64, exchange: &str) -> Self {
        Self {
            asset: asset.to_uppercase(),
            free,
            locked,
            total: free + locked,
            exchange: exchange.to_string(),
            timestamp: Utc::now(),
        }
    }
    
    pub fn is_sufficient(&self, required: f64) -> bool {
        self.free >= required
    }
    
    pub fn utilization_ratio(&self) -> f64 {
        if self.total > 0.0 {
            self.locked / self.total
        } else {
            0.0
        }
    }
}

/// 订单请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub time_in_force: Option<String>,
    pub client_order_id: Option<String>,
    pub reduce_only: bool,
    pub post_only: bool,
    pub margin_trading: bool,
    pub created_at: DateTime<Utc>,
}

impl OrderRequest {
    pub fn new_market_buy(exchange: &str, symbol: &str, quantity: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            time_in_force: None,
            client_order_id: None,
            reduce_only: false,
            post_only: false,
            margin_trading: false,
            created_at: Utc::now(),
        }
    }
    
    pub fn new_market_sell(exchange: &str, symbol: &str, quantity: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            quantity,
            price: None,
            stop_price: None,
            time_in_force: None,
            client_order_id: None,
            reduce_only: false,
            post_only: false,
            margin_trading: false,
            created_at: Utc::now(),
        }
    }
    
    pub fn new_limit_buy(exchange: &str, symbol: &str, quantity: f64, price: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            time_in_force: None,
            client_order_id: None,
            reduce_only: false,
            post_only: false,
            margin_trading: false,
            created_at: Utc::now(),
        }
    }
    
    pub fn new_limit_sell(exchange: &str, symbol: &str, quantity: f64, price: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            side: OrderSide::Sell,
            order_type: OrderType::Limit,
            quantity,
            price: Some(price),
            stop_price: None,
            time_in_force: None,
            client_order_id: None,
            reduce_only: false,
            post_only: false,
            margin_trading: false,
            created_at: Utc::now(),
        }
    }
}

/// 订单响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub status: OrderStatus,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
    pub average_price: Option<f64>,
    pub fee: Option<f64>,
    pub fee_asset: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub exchange: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub original_quantity: f64,
    pub trades: Vec<TradeInfo>,
}

/// 成交信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInfo {
    pub trade_id: String,
    pub price: f64,
    pub quantity: f64,
    pub fee: f64,
    pub fee_asset: String,
    pub timestamp: DateTime<Utc>,
    pub is_maker: bool,
}

/// 重新导出统一的ArbitrageOpportunity定义用于前后端互通
pub use common_types::ArbitrageOpportunity;


/// 市场数据聚合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub exchanges: HashMap<String, ExchangeMarketData>,
    pub aggregated_price: Option<f64>,
    pub price_variance: f64,
    pub volume_24h: f64,
    pub timestamp: DateTime<Utc>,
    pub data_quality_score: f64,
}

/// 交易所市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeMarketData {
    pub exchange: String,
    pub symbol: String,
    pub last_price: f64,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub price_change_percent_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub timestamp: DateTime<Utc>,
    pub orderbook: Option<OrderBookDepth>,
    pub recent_trades: Vec<Trade>,
}

impl MarketData {
    pub fn calculate_volatility(&self) -> f64 {
        if self.exchanges.len() < 2 {
            return 0.0;
        }
        
        let prices: Vec<f64> = self.exchanges.values()
            .map(|data| data.last_price)
            .collect();
        
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        
        variance.sqrt()
    }
    
    pub fn get_depth_ratio(&self) -> f64 {
        let total_bid_volume: f64 = self.exchanges.values()
            .map(|data| data.bid_volume)
            .sum();
        let total_ask_volume: f64 = self.exchanges.values()
            .map(|data| data.ask_volume)
            .sum();
        
        if total_ask_volume > 0.0 {
            total_bid_volume / total_ask_volume
        } else {
            0.0
        }
    }
    
    pub fn detect_volume_spike(&self) -> f64 {
        // 简单的成交量突变检测
        let avg_volume: f64 = self.exchanges.values()
            .map(|data| data.volume_24h)
            .sum::<f64>() / self.exchanges.len() as f64;
        
        let max_volume = self.exchanges.values()
            .map(|data| data.volume_24h)
            .fold(0.0, f64::max);
        
        if avg_volume > 0.0 {
            max_volume / avg_volume
        } else {
            1.0
        }
    }
}

/// 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub is_running: bool,
    pub market_state: MarketState,
    pub active_strategies: Vec<String>,
    pub total_profit_usd: f64,
    pub total_profit_bps: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub last_update: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub processed_opportunities: u64,
    pub executed_trades: u64,
    pub rejected_opportunities: u64,
    pub system_load: f64,
    pub memory_usage_mb: f64,
    pub active_connections: u32,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            is_running: false,
            market_state: MarketState::Normal,
            active_strategies: Vec::new(),
            total_profit_usd: 0.0,
            total_profit_bps: 0.0,
            success_rate: 0.0,
            error_rate: 0.0,
            last_update: Utc::now(),
            uptime_seconds: 0,
            processed_opportunities: 0,
            executed_trades: 0,
            rejected_opportunities: 0,
            system_load: 0.0,
            memory_usage_mb: 0.0,
            active_connections: 0,
        }
    }
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution_id: String,
    pub opportunity_id: String,
    pub strategy_type: StrategyType,
    pub status: ExecutionStatus,
    pub orders: Vec<OrderResponse>,
    pub total_profit_usd: f64,
    pub total_fees_usd: f64,
    pub net_profit_usd: f64,
    pub execution_time_ms: u64,
    pub slippage: f64,
    pub market_impact: f64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// 执行状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    InProgress,
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
    Timeout,
}

/// 资金分配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundAllocation {
    pub allocation_id: String,
    pub opportunity_id: String,
    pub allocations: HashMap<String, ExchangeAllocation>, // exchange -> allocation
    pub total_required_usd: f64,
    pub total_available_usd: f64,
    pub is_sufficient: bool,
    pub confidence_level: f64,
    pub created_at: DateTime<Utc>,
}

/// 单个交易所的资金分配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAllocation {
    pub exchange: String,
    pub allocations: HashMap<String, f64>, // asset -> amount
    pub total_value_usd: f64,
    pub utilization_ratio: f64,
    pub reserved_for_fees: f64,
    pub margin_buffer: f64,
}

impl FundAllocation {
    pub fn is_sufficient(&self) -> bool {
        self.is_sufficient
    }
    
    pub fn utilization_ratio(&self) -> f64 {
        if self.total_available_usd > 0.0 {
            self.total_required_usd / self.total_available_usd
        } else {
            0.0
        }
    }
}

/// 风险评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub assessment_id: String,
    pub opportunity_id: String,
    pub overall_risk_score: f64,
    pub risk_level: RiskLevel,
    pub is_approved: bool,
    pub risk_factors: HashMap<String, f64>,
    pub recommendations: Vec<String>,
    pub max_position_size: f64,
    pub stop_loss_level: Option<f64>,
    pub take_profit_level: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub rejection_reason: Option<String>,
}

impl RiskAssessment {
    pub fn new() -> Self {
        Self {
            assessment_id: Uuid::new_v4().to_string(),
            opportunity_id: String::new(),
            overall_risk_score: 0.0,
            risk_level: RiskLevel::Medium,
            is_approved: false,
            risk_factors: HashMap::new(),
            recommendations: Vec::new(),
            max_position_size: 0.0,
            stop_loss_level: None,
            take_profit_level: None,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            rejection_reason: None,
        }
    }
    
    pub fn approve(&mut self) {
        self.is_approved = true;
        self.rejection_reason = None;
    }
    
    pub fn reject(&mut self, reason: &str) {
        self.is_approved = false;
        self.rejection_reason = Some(reason.to_string());
    }
    
    pub fn is_approved(&self) -> bool {
        self.is_approved && !self.is_expired()
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// 性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub total_opportunities: u64,
    pub executed_opportunities: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_profit_usd: f64,
    pub total_volume_usd: f64,
    pub average_execution_time_ms: f64,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            total_opportunities: 0,
            executed_opportunities: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_profit_usd: 0.0,
            total_volume_usd: 0.0,
            average_execution_time_ms: 0.0,
            last_update: chrono::Utc::now(),
        }
    }
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: DateTime<Utc>,
    pub total_opportunities: u64,
    pub executed_opportunities: u64,
    pub success_rate: f64,
    pub average_profit_bps: f64,
    pub total_profit_usd: f64,
    pub total_volume_usd: f64,
    pub average_execution_time_ms: f64,
    pub error_rate: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
}

/// 系统配置快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub checksum: String,
    pub config_data: HashMap<String, serde_json::Value>,
}

/// 事件类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    OpportunityDetected,
    OpportunityExecuted,
    OpportunityExpired,
    OrderPlaced,
    OrderFilled,
    OrderCancelled,
    BalanceUpdated,
    RiskAlert,
    SystemStarted,
    SystemStopped,
    ConfigChanged,
    HealthCheckFailed,
    PerformanceReport,
}

/// 系统事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub id: String,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub data: serde_json::Value,
    pub correlation_id: Option<String>,
    pub trace_id: Option<String>,
}

impl SystemEvent {
    pub fn new(event_type: EventType, source: &str, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            source: source.to_string(),
            data,
            correlation_id: None,
            trace_id: None,
        }
    }
    
    pub fn get_type(&self) -> EventType {
        self.event_type
    }
    
    pub fn with_correlation_id(mut self, correlation_id: &str) -> Self {
        self.correlation_id = Some(correlation_id.to_string());
        self
    }
    
    pub fn with_trace_id(mut self, trace_id: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self
    }
} 