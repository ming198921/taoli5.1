use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub timestamp: DateTime<Utc>,
}

impl<T> StandardResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "Success".to_string(),
            data: Some(data),
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            timestamp: Utc::now(),
        }
    }
}

// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatus {
    pub id: String,
    pub symbol: String,
    pub side: String, // "buy", "sell"
    pub order_type: String, // "market", "limit", "stop_loss"
    pub status: String, // "pending", "filled", "partially_filled", "cancelled", "rejected"
    pub quantity: f64,
    pub price: Option<f64>,
    pub filled_quantity: f64,
    pub average_fill_price: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub exchange: String,
    pub commission: f64,
    pub time_in_force: String, // "GTC", "IOC", "FOK"
}

// 持仓信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: String,
    pub symbol: String,
    pub exchange: String,
    pub side: String, // "long", "short"
    pub quantity: f64,
    pub average_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub margin_used: f64,
    pub leverage: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub risk_metrics: PositionRiskMetrics,
}

// 资金状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundStatus {
    pub account_id: String,
    pub exchange: String,
    pub currency: String,
    pub total_balance: f64,
    pub available_balance: f64,
    pub frozen_balance: f64,
    pub margin_balance: f64,
    pub unrealized_pnl: f64,
    pub equity: f64,
    pub margin_ratio: f64,
    pub last_updated: DateTime<Utc>,
}

// 风险指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub account_id: String,
    pub total_exposure: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub var_95: f64, // Value at Risk 95%
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub max_consecutive_losses: u32,
    pub current_consecutive_losses: u32,
    pub risk_score: f64, // 0-100
    pub leverage_utilization: f64,
    pub concentration_risk: HashMap<String, f64>,
}

// 持仓风险指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRiskMetrics {
    pub position_value: f64,
    pub risk_exposure: f64,
    pub stop_loss_price: Option<f64>,
    pub take_profit_price: Option<f64>,
    pub liquidation_price: Option<f64>,
    pub margin_ratio: f64,
    pub delta: f64,
    pub gamma: f64,
    pub theta: f64,
    pub vega: f64,
}

// 交易执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    pub id: String,
    pub order_id: String,
    pub symbol: String,
    pub exchange: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub commission: f64,
    pub commission_asset: String,
    pub executed_at: DateTime<Utc>,
    pub execution_type: String, // "maker", "taker"
    pub trade_id: String,
}

// 套利机会
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub symbol: String,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub price_difference: f64,
    pub profit_percentage: f64,
    pub estimated_profit: f64,
    pub volume: f64,
    pub detected_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String, // "active", "executed", "expired"
    pub risk_level: String, // "low", "medium", "high"
}

// 交易信号
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSignal {
    pub id: String,
    pub symbol: String,
    pub signal_type: String, // "buy", "sell", "hold"
    pub strength: f64, // 0.0 - 1.0
    pub confidence: f64, // 0.0 - 1.0
    pub generated_at: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub source: String, // "technical_analysis", "ml_model", "arbitrage"
    pub parameters: HashMap<String, f64>,
    pub description: String,
}

// 市场数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub symbol: String,
    pub exchange: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
    pub change_percentage_24h: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub bid_price: f64,
    pub ask_price: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub timestamp: DateTime<Utc>,
    pub orderbook_snapshot: Option<OrderbookSnapshot>,
}

// 订单簿快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookSnapshot {
    pub symbol: String,
    pub exchange: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
}

// 价格层级
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: f64,
    pub quantity: f64,
}

// 交易策略状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStatus {
    pub id: String,
    pub name: String,
    pub status: String, // "active", "paused", "stopped", "error"
    pub pnl: f64,
    pub total_trades: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub win_rate: f64,
    pub average_profit: f64,
    pub average_loss: f64,
    pub max_drawdown: f64,
    pub current_positions: u32,
    pub started_at: DateTime<Utc>,
    pub last_trade_at: Option<DateTime<Utc>>,
    pub parameters: HashMap<String, serde_json::Value>,
}

// 交易费用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingFee {
    pub exchange: String,
    pub symbol: String,
    pub maker_fee: f64,
    pub taker_fee: f64,
    pub withdrawal_fee: f64,
    pub deposit_fee: f64,
    pub last_updated: DateTime<Utc>,
}

// 流动性信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityInfo {
    pub symbol: String,
    pub exchange: String,
    pub bid_liquidity: f64,
    pub ask_liquidity: f64,
    pub spread_percentage: f64,
    pub market_impact: f64,
    pub depth_1_percent: f64,
    pub depth_5_percent: f64,
    pub timestamp: DateTime<Utc>,
}

// 交易限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingLimits {
    pub exchange: String,
    pub symbol: String,
    pub min_order_size: f64,
    pub max_order_size: f64,
    pub min_price_increment: f64,
    pub min_quantity_increment: f64,
    pub max_leverage: f64,
    pub margin_requirement: f64,
    pub daily_volume_limit: Option<f64>,
    pub position_limit: Option<f64>,
}

// 交易会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingSession {
    pub id: String,
    pub exchange: String,
    pub status: String, // "active", "closed", "pre_market", "after_hours"
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub timezone: String,
    pub trading_hours: Vec<TradingHours>,
}

// 交易时间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingHours {
    pub day_of_week: String,
    pub open_time: String,
    pub close_time: String,
    pub is_trading_day: bool,
}