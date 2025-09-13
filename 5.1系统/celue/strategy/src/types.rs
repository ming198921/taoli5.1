//! 策略模块类型定义
//! 
//! 定义策略系统中使用的核心类型和数据结构

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 策略配置 - 重新导出统一的配置定义
/// 使用统一的BaseStrategyConfig以消除重复定义
pub use common_types::{BaseStrategyConfig as StrategyConfig, BaseConfig};

/// 直接使用统一的市场数据定义 - 不再重复定义
pub use arbitrage_architecture::types::{
    MarketData, 
    ExchangeMarketData, 
    MarketState as ArchitectureMarketState,
    OrderBookDepth,
    Trade
};

/// 为了向后兼容，提供MarketDataSnapshot的类型别名
pub type MarketDataSnapshot = ExchangeMarketData;

/// 市场状态评估器trait  
#[async_trait]
pub trait MarketStateEvaluator: Send + Sync {
    /// 评估市场状态 - 使用统一的MarketState定义
    async fn evaluate_market_state(&self, data: &MarketDataSnapshot) -> anyhow::Result<ArchitectureMarketState>;
    
    /// 获取波动率
    async fn get_volatility(&self, symbol: &str) -> anyhow::Result<f64>;
    
    /// 获取流动性深度
    async fn get_market_depth(&self, symbol: &str) -> anyhow::Result<f64>;
}

/// 最小利润调整器trait
#[async_trait]
pub trait MinProfitAdjuster: Send + Sync {
    /// 调整最小利润阈值 - 使用统一的MarketState定义
    async fn adjust_min_profit(
        &self,
        base_profit: f64,
        market_state: ArchitectureMarketState,
        success_rate: f64,
    ) -> anyhow::Result<f64>;
    
    /// 获取当前成功率
    async fn get_success_rate(&self, symbol: &str) -> anyhow::Result<f64>;
}

/// 风险管理器trait
#[async_trait]
pub trait RiskManager: Send + Sync {
    /// 评估交易风险
    async fn assess_trade_risk(&self, trade: &TradeProposal) -> anyhow::Result<RiskAssessment>;
    
    /// 检查风险限制
    async fn check_risk_limits(&self, portfolio: &Portfolio) -> anyhow::Result<bool>;
    
    /// 获取当前风险敞口
    async fn get_current_exposure(&self) -> anyhow::Result<f64>;
}

/// 交易提案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeProposal {
    /// 交易对
    pub symbol: String,
    /// 交易所
    pub exchange: String,
    /// 交易方向（买/卖）
    pub side: TradeSide,
    /// 交易数量
    pub quantity: f64,
    /// 预期价格
    pub expected_price: f64,
    /// 预期利润
    pub expected_profit: f64,
    /// 风险评分
    pub risk_score: f64,
}

/// 交易方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 风险评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 风险评分（0-100）
    pub risk_score: f64,
    /// 建议动作
    pub recommended_action: RecommendedAction,
    /// 风险因子
    pub risk_factors: Vec<RiskFactor>,
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 建议动作
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendedAction {
    Approve,
    ApproveWithCaution,
    Reject,
    RequireReview,
}

/// 风险因子
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// 因子名称
    pub name: String,
    /// 因子权重
    pub weight: f64,
    /// 当前值
    pub current_value: f64,
    /// 阈值
    pub threshold: f64,
    /// 是否超出阈值
    pub is_exceeded: bool,
}

/// 投资组合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    /// 总价值
    pub total_value: f64,
    /// 现金余额
    pub cash_balance: f64,
    /// 持仓
    pub positions: HashMap<String, Position>,
    /// 未结订单
    pub pending_orders: Vec<Order>,
}

/// 持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// 交易对
    pub symbol: String,
    /// 数量
    pub quantity: f64,
    /// 平均成本
    pub average_cost: f64,
    /// 当前市值
    pub market_value: f64,
    /// 未实现盈亏
    pub unrealized_pnl: f64,
}

/// 订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// 订单ID
    pub order_id: String,
    /// 交易对
    pub symbol: String,
    /// 交易所
    pub exchange: String,
    /// 订单类型
    pub order_type: OrderType,
    /// 交易方向
    pub side: TradeSide,
    /// 数量
    pub quantity: f64,
    /// 价格
    pub price: f64,
    /// 订单状态
    pub status: OrderStatus,
    /// 创建时间
    pub created_at: i64,
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}


/// 基础市场状态评估器实现
#[derive(Debug, Clone)]
pub struct BasicMarketStateEvaluator {
    /// 波动率阈值配置
    pub volatility_thresholds: VolatilityThresholds,
}

#[derive(Debug, Clone)]
pub struct VolatilityThresholds {
    pub normal_threshold: f64,
    pub caution_threshold: f64,
    pub extreme_threshold: f64,
}

impl Default for VolatilityThresholds {
    fn default() -> Self {
        Self {
            normal_threshold: 0.01,
            caution_threshold: 0.03,
            extreme_threshold: 0.05,
        }
    }
}

impl Default for BasicMarketStateEvaluator {
    fn default() -> Self {
        Self {
            volatility_thresholds: VolatilityThresholds::default(),
        }
    }
}

#[async_trait]
impl MarketStateEvaluator for BasicMarketStateEvaluator {
    async fn evaluate_market_state(&self, _data: &MarketDataSnapshot) -> anyhow::Result<ArchitectureMarketState> {
        // 简化实现，实际中会分析市场数据
        Ok(ArchitectureMarketState::Normal)
    }
    
    async fn get_volatility(&self, _symbol: &str) -> anyhow::Result<f64> {
        // 简化实现，返回模拟波动率
        Ok(0.02)
    }
    
    async fn get_market_depth(&self, _symbol: &str) -> anyhow::Result<f64> {
        // 简化实现，返回模拟流动性深度
        Ok(0.5)
    }
}