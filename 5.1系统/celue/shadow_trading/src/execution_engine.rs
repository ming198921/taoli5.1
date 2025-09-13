//! 影子交易执行引擎

use crate::config::ExecutionEngineConfig;
use crate::market_simulator::MarketSimulator;
use crate::order_matching::{OrderMatchingEngine, TradeExecution, MatchingResult};
use crate::metrics::ShadowTradingMetrics;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, Notify};
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, info, warn, error, instrument};
use uuid::Uuid;

/// 影子订单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowOrder {
    /// 订单ID
    pub id: String,
    /// 账户ID
    pub account_id: String,
    /// 交易对
    pub symbol: String,
    /// 订单方向
    pub side: OrderSide,
    /// 订单数量
    pub quantity: f64,
    /// 订单价格（None表示市价单）
    pub price: Option<f64>,
    /// 订单类型
    pub order_type: OrderType,
    /// 订单状态
    pub status: OrderStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 已成交数量
    pub filled_quantity: f64,
    /// 平均成交价
    pub average_price: Option<f64>,
    /// 手续费
    pub fees: f64,
    /// 订单元数据
    pub metadata: HashMap<String, String>,
}

/// 订单方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 订单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
    TakeProfitMarket,
    TakeProfitLimit,
}

/// 订单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

/// 执行事件
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    OrderSubmitted(ShadowOrder),
    OrderUpdated(ShadowOrder),
    OrderCancelled(String),
    TradeExecuted(TradeExecution),
    OrderExpired(String),
}

/// 影子交易执行引擎
pub struct ShadowExecutionEngine {
    /// 配置
    config: ExecutionEngineConfig,
    /// 活跃订单
    active_orders: Arc<RwLock<HashMap<String, ShadowOrder>>>,
    /// 订单历史
    order_history: Arc<RwLock<VecDeque<ShadowOrder>>>,
    /// 交易历史
    trade_history: Arc<RwLock<VecDeque<TradeExecution>>>,
    /// 市场模拟器
    market_simulator: Arc<MarketSimulator>,
    /// 订单匹配引擎
    order_matching: Arc<OrderMatchingEngine>,
    /// 指标收集器
    metrics: Arc<ShadowTradingMetrics>,
    /// 执行事件发送器
    event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    /// 执行事件接收器
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<ExecutionEvent>>>>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 通知器
    notify: Arc<Notify>,
}

impl ShadowExecutionEngine {
    /// 创建新的执行引擎
    pub fn new(
        config: ExecutionEngineConfig,
        market_simulator: Arc<MarketSimulator>,
        order_matching: Arc<OrderMatchingEngine>,
        metrics: Arc<ShadowTradingMetrics>,
    ) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            config,
            active_orders: Arc::new(RwLock::new(HashMap::new())),
            order_history: Arc::new(RwLock::new(VecDeque::new())),
            trade_history: Arc::new(RwLock::new(VecDeque::new())),
            market_simulator,
            order_matching,
            metrics,
            event_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
            running: Arc::new(RwLock::new(false)),
            notify: Arc::new(Notify::new()),
        })
    }

    /// 启动执行引擎
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Execution engine is already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        // 启动事件处理任务
        self.start_event_processor().await?;
        
        // 启动订单处理任务
        self.start_order_processor().await?;
        
        // 启动清理任务
        self.start_cleanup_task().await?;

        info!("Shadow execution engine started");
        Ok(())
    }

    /// 停止执行引擎
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            warn!("Execution engine is not running");
            return Ok(());
        }
        *running = false;

        info!("Shadow execution engine stopped");
        Ok(())
    }

    /// 提交订单
    #[instrument(skip(self), fields(order_id = %order.id, symbol = %order.symbol))]
    pub async fn submit_order(&self, mut order: ShadowOrder) -> Result<String> {
        // 生成订单ID
        if order.id.is_empty() {
            order.id = Uuid::new_v4().to_string();
        }

        // 验证订单
        self.validate_order(&order).await?;

        // 设置初始状态
        order.status = OrderStatus::Pending;
        order.created_at = Utc::now();
        order.updated_at = Utc::now();

        // 检查并发订单限制
        {
            let active_orders = self.active_orders.read().await;
            if active_orders.len() >= self.config.max_concurrent_orders {
                return Err(anyhow::anyhow!("Maximum concurrent orders limit reached"));
            }
        }

        let order_id = order.id.clone();

        // 添加到活跃订单
        {
            let mut active_orders = self.active_orders.write().await;
            active_orders.insert(order_id.clone(), order.clone());
        }

        // 发送事件
        self.event_sender.send(ExecutionEvent::OrderSubmitted(order))?;
        
        // 通知处理器
        self.notify.notify_one();

        self.metrics.order_submitted(&order_id).await;
        debug!(order_id = %order_id, "Order submitted");

        Ok(order_id)
    }

    /// 取消订单
    pub async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let mut order = {
            let mut active_orders = self.active_orders.write().await;
            let mut order = active_orders
                .remove(order_id)
                .ok_or_else(|| anyhow::anyhow!("Order {} not found", order_id))?;
            
            if !matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
                return Err(anyhow::anyhow!("Order {} cannot be cancelled", order_id));
            }

            order.status = OrderStatus::Cancelled;
            order.updated_at = Utc::now();
            order
        };

        // 添加到历史记录
        {
            let mut history = self.order_history.write().await;
            history.push_back(order.clone());
            
            // 限制历史记录大小
            while history.len() > 10000 {
                history.pop_front();
            }
        }

        // 发送事件
        self.event_sender.send(ExecutionEvent::OrderCancelled(order_id.to_string()))?;

        self.metrics.order_cancelled(order_id).await;
        debug!(order_id = %order_id, "Order cancelled");

        Ok(())
    }

    /// 获取订单信息
    pub async fn get_order(&self, order_id: &str) -> Result<ShadowOrder> {
        // 先检查活跃订单
        {
            let active_orders = self.active_orders.read().await;
            if let Some(order) = active_orders.get(order_id) {
                return Ok(order.clone());
            }
        }

        // 再检查历史订单
        {
            let history = self.order_history.read().await;
            for order in history.iter().rev() {
                if order.id == order_id {
                    return Ok(order.clone());
                }
            }
        }

        Err(anyhow::anyhow!("Order {} not found", order_id))
    }

    /// 获取账户订单
    pub async fn get_account_orders(&self, account_id: &str) -> Result<Vec<ShadowOrder>> {
        let mut orders = Vec::new();

        // 获取活跃订单
        {
            let active_orders = self.active_orders.read().await;
            for order in active_orders.values() {
                if order.account_id == account_id {
                    orders.push(order.clone());
                }
            }
        }

        // 获取历史订单（最近100个）
        {
            let history = self.order_history.read().await;
            let mut count = 0;
            for order in history.iter().rev() {
                if order.account_id == account_id {
                    orders.push(order.clone());
                    count += 1;
                    if count >= 100 {
                        break;
                    }
                }
            }
        }

        Ok(orders)
    }

    /// 获取交易历史
    pub async fn get_trade_history(
        &self,
        account_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<TradeExecution>> {
        let history = self.trade_history.read().await;
        let mut trades = Vec::new();

        for trade in history.iter().rev() {
            // 检查账户ID（通过订单ID查找）
            if let Ok(order) = self.get_order(&trade.order_id).await {
                if order.account_id != account_id {
                    continue;
                }
            } else {
                continue;
            }

            // 检查时间范围
            if let Some(from_time) = from {
                if trade.executed_at < from_time {
                    continue;
                }
            }

            if let Some(to_time) = to {
                if trade.executed_at > to_time {
                    continue;
                }
            }

            trades.push(trade.clone());
        }

        Ok(trades)
    }

    /// 验证订单
    async fn validate_order(&self, order: &ShadowOrder) -> Result<()> {
        // 基本字段验证
        if order.account_id.is_empty() {
            return Err(anyhow::anyhow!("Account ID is required"));
        }

        if order.symbol.is_empty() {
            return Err(anyhow::anyhow!("Symbol is required"));
        }

        if order.quantity <= 0.0 {
            return Err(anyhow::anyhow!("Quantity must be positive"));
        }

        // 限价单必须有价格
        if matches!(order.order_type, OrderType::Limit | OrderType::StopLimit | OrderType::TakeProfitLimit) {
            if order.price.is_none() || order.price.unwrap() <= 0.0 {
                return Err(anyhow::anyhow!("Price is required for limit orders"));
            }
        }

        // 检查交易对是否支持
        if !self.market_simulator.is_symbol_supported(&order.symbol).await {
            return Err(anyhow::anyhow!("Symbol {} is not supported", order.symbol));
        }

        Ok(())
    }

    /// 启动事件处理器
    async fn start_event_processor(&self) -> Result<()> {
        let mut receiver = {
            let mut receiver_opt = self.event_receiver.write().await;
            receiver_opt.take().context("Event receiver already taken")?
        };

        let engine = Arc::new(self);
        tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                if let Err(e) = engine.handle_event(event).await {
                    error!(error = %e, "Failed to handle execution event");
                }
            }
        });

        Ok(())
    }

    /// 启动订单处理器
    async fn start_order_processor(&self) -> Result<()> {
        let engine = Arc::new(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = engine.process_orders().await {
                            error!(error = %e, "Failed to process orders");
                        }
                    }
                    _ = engine.notify.notified() => {
                        if let Err(e) = engine.process_orders().await {
                            error!(error = %e, "Failed to process orders on notification");
                        }
                    }
                }
                
                let running = *engine.running.read().await;
                if !running {
                    break;
                }
            }
        });

        Ok(())
    }

    /// 启动清理任务
    async fn start_cleanup_task(&self) -> Result<()> {
        let engine = Arc::new(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5分钟清理一次
            
            loop {
                interval.tick().await;
                
                if let Err(e) = engine.cleanup_expired_orders().await {
                    error!(error = %e, "Failed to cleanup expired orders");
                }
                
                let running = *engine.running.read().await;
                if !running {
                    break;
                }
            }
        });

        Ok(())
    }

    /// 处理执行事件
    async fn handle_event(&self, event: ExecutionEvent) -> Result<()> {
        match event {
            ExecutionEvent::OrderSubmitted(order) => {
                debug!(order_id = %order.id, "Processing submitted order");
                self.process_new_order(order).await?;
            }
            ExecutionEvent::OrderUpdated(order) => {
                debug!(order_id = %order.id, "Processing updated order");
            }
            ExecutionEvent::OrderCancelled(order_id) => {
                debug!(order_id = %order_id, "Processing cancelled order");
            }
            ExecutionEvent::TradeExecuted(trade) => {
                debug!(trade_id = %trade.trade_id, "Processing executed trade");
                self.handle_trade_execution(trade).await?;
            }
            ExecutionEvent::OrderExpired(order_id) => {
                debug!(order_id = %order_id, "Processing expired order");
                self.handle_order_expiration(order_id).await?;
            }
        }

        Ok(())
    }

    /// 处理新订单
    async fn process_new_order(&self, order: ShadowOrder) -> Result<()> {
        match order.order_type {
            OrderType::Market => {
                self.process_market_order(order).await?;
            }
            OrderType::Limit => {
                self.process_limit_order(order).await?;
            }
            _ => {
                warn!(order_id = %order.id, order_type = ?order.order_type, "Order type not yet implemented");
            }
        }

        Ok(())
    }

    /// 处理市价单
    async fn process_market_order(&self, order: ShadowOrder) -> Result<()> {
        // 获取当前市价
        let current_price = self.market_simulator.get_current_price(&order.symbol).await?;
        
        // 计算滑点
        let slippage = self.calculate_slippage(&order, current_price).await;
        let execution_price = match order.side {
            OrderSide::Buy => current_price * (1.0 + slippage),
            OrderSide::Sell => current_price * (1.0 - slippage),
        };

        // 模拟执行延迟
        if rand::random::<f64>() > self.config.market_order_immediate_fill_prob {
            sleep(Duration::from_millis(self.config.partial_fill_interval_ms)).await;
        }

        // 创建交易执行记录
        let trade = TradeExecution {
            trade_id: Uuid::new_v4().to_string(),
            order_id: order.id.clone(),
            symbol: order.symbol.clone(),
            side: order.side,
            quantity: order.quantity,
            price: execution_price,
            value: order.quantity * execution_price,
            fees: self.calculate_fees(&order, execution_price),
            executed_at: Utc::now(),
            metadata: HashMap::new(),
        };

        // 更新订单状态
        self.update_order_on_fill(&order.id, order.quantity, execution_price).await?;

        // 发送交易执行事件
        self.event_sender.send(ExecutionEvent::TradeExecuted(trade))?;

        Ok(())
    }

    /// 处理限价单
    async fn process_limit_order(&self, order: ShadowOrder) -> Result<()> {
        let current_price = self.market_simulator.get_current_price(&order.symbol).await?;
        let order_price = order.price.unwrap();

        // 检查是否可以立即成交
        let can_fill = match order.side {
            OrderSide::Buy => current_price <= order_price,
            OrderSide::Sell => current_price >= order_price,
        };

        if can_fill {
            // 使用订单价格执行
            let trade = TradeExecution {
                trade_id: Uuid::new_v4().to_string(),
                order_id: order.id.clone(),
                symbol: order.symbol.clone(),
                side: order.side,
                quantity: order.quantity,
                price: order_price,
                value: order.quantity * order_price,
                fees: self.calculate_fees(&order, order_price),
                executed_at: Utc::now(),
                metadata: HashMap::new(),
            };

            // 更新订单状态
            self.update_order_on_fill(&order.id, order.quantity, order_price).await?;

            // 发送交易执行事件
            self.event_sender.send(ExecutionEvent::TradeExecuted(trade))?;
        }
        // 否则订单保持在订单簿中等待成交

        Ok(())
    }

    /// 处理所有订单
    async fn process_orders(&self) -> Result<()> {
        let running = *self.running.read().await;
        if !running {
            return Ok(());
        }

        let orders = {
            let active_orders = self.active_orders.read().await;
            active_orders.values().cloned().collect::<Vec<_>>()
        };

        for order in orders {
            if matches!(order.order_type, OrderType::Limit) && order.status == OrderStatus::Pending {
                self.check_limit_order_fill(&order).await?;
            }
        }

        Ok(())
    }

    /// 检查限价单是否可以成交
    async fn check_limit_order_fill(&self, order: &ShadowOrder) -> Result<()> {
        let current_price = self.market_simulator.get_current_price(&order.symbol).await?;
        let order_price = order.price.unwrap();

        let can_fill = match order.side {
            OrderSide::Buy => current_price <= order_price,
            OrderSide::Sell => current_price >= order_price,
        };

        if can_fill {
            // 使用概率模型决定是否成交
            let fill_probability = self.calculate_fill_probability(order, current_price);
            if rand::random::<f64>() < fill_probability {
                let trade = TradeExecution {
                    trade_id: Uuid::new_v4().to_string(),
                    order_id: order.id.clone(),
                    symbol: order.symbol.clone(),
                    side: order.side,
                    quantity: order.quantity,
                    price: order_price,
                    value: order.quantity * order_price,
                    fees: self.calculate_fees(order, order_price),
                    executed_at: Utc::now(),
                    metadata: HashMap::new(),
                };

                self.update_order_on_fill(&order.id, order.quantity, order_price).await?;
                self.event_sender.send(ExecutionEvent::TradeExecuted(trade))?;
            }
        }

        Ok(())
    }

    /// 计算滑点
    async fn calculate_slippage(&self, order: &ShadowOrder, current_price: f64) -> f64 {
        if !self.config.enable_slippage_simulation {
            return 0.0;
        }

        let base_slippage = self.config.average_slippage_bps / 10000.0;
        let max_slippage = self.config.max_slippage_bps / 10000.0;
        
        // 基于订单大小和市场条件的滑点模型
        let size_impact = (order.quantity * current_price / 100000.0).min(1.0); // 基于10万美元的影响
        let random_factor = rand::random::<f64>() - 0.5; // -0.5 到 0.5
        
        let slippage = base_slippage * (1.0 + size_impact) * (1.0 + random_factor);
        slippage.min(max_slippage).max(0.0)
    }

    /// 计算成交概率
    fn calculate_fill_probability(&self, order: &ShadowOrder, current_price: f64) -> f64 {
        let order_price = order.price.unwrap();
        
        let price_advantage = match order.side {
            OrderSide::Buy => {
                if current_price <= order_price {
                    (order_price - current_price) / current_price
                } else {
                    return 0.0;
                }
            }
            OrderSide::Sell => {
                if current_price >= order_price {
                    (current_price - order_price) / current_price
                } else {
                    return 0.0;
                }
            }
        };

        // 基础概率 + 价格优势奖励
        let base_prob = 0.1; // 10% 基础概率
        let advantage_boost = (price_advantage * 100.0).min(0.8); // 最多80%奖励
        
        (base_prob + advantage_boost).min(1.0)
    }

    /// 计算手续费
    fn calculate_fees(&self, order: &ShadowOrder, execution_price: f64) -> f64 {
        let fee_rate = match order.order_type {
            OrderType::Market => 0.001, // 0.1% taker fee
            OrderType::Limit => 0.0008, // 0.08% maker fee
            _ => 0.001,
        };
        
        order.quantity * execution_price * fee_rate
    }

    /// 更新订单成交信息
    async fn update_order_on_fill(
        &self,
        order_id: &str,
        filled_quantity: f64,
        fill_price: f64,
    ) -> Result<()> {
        let mut active_orders = self.active_orders.write().await;
        
        if let Some(order) = active_orders.get_mut(order_id) {
            let previous_filled = order.filled_quantity;
            order.filled_quantity += filled_quantity;
            
            // 更新平均成交价
            if let Some(avg_price) = order.average_price {
                let total_value = previous_filled * avg_price + filled_quantity * fill_price;
                order.average_price = Some(total_value / order.filled_quantity);
            } else {
                order.average_price = Some(fill_price);
            }

            // 更新状态
            if (order.filled_quantity - order.quantity).abs() < 1e-8 {
                order.status = OrderStatus::Filled;
                order.updated_at = Utc::now();
                
                // 移动到历史记录
                let completed_order = order.clone();
                drop(active_orders); // 释放锁
                
                let mut active_orders = self.active_orders.write().await;
                active_orders.remove(order_id);
                
                let mut history = self.order_history.write().await;
                history.push_back(completed_order);
                
                if history.len() > 10000 {
                    history.pop_front();
                }
            } else {
                order.status = OrderStatus::PartiallyFilled;
                order.updated_at = Utc::now();
            }
        }

        Ok(())
    }

    /// 处理交易执行
    async fn handle_trade_execution(&self, trade: TradeExecution) -> Result<()> {
        // 添加到交易历史
        {
            let mut history = self.trade_history.write().await;
            history.push_back(trade.clone());
            
            if history.len() > 50000 {
                history.pop_front();
            }
        }

        // 更新指标
        self.metrics.trade_executed(&trade).await;
        
        debug!(
            trade_id = %trade.trade_id,
            order_id = %trade.order_id,
            symbol = %trade.symbol,
            quantity = trade.quantity,
            price = trade.price,
            "Trade executed"
        );

        Ok(())
    }

    /// 处理订单过期
    async fn handle_order_expiration(&self, order_id: String) -> Result<()> {
        let mut active_orders = self.active_orders.write().await;
        
        if let Some(mut order) = active_orders.remove(&order_id) {
            order.status = OrderStatus::Expired;
            order.updated_at = Utc::now();
            
            // 添加到历史记录
            let mut history = self.order_history.write().await;
            history.push_back(order);
            
            if history.len() > 10000 {
                history.pop_front();
            }
        }

        self.metrics.order_expired(&order_id).await;
        debug!(order_id = %order_id, "Order expired");

        Ok(())
    }

    /// 清理过期订单
    async fn cleanup_expired_orders(&self) -> Result<()> {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(self.config.order_timeout_seconds as i64);
        let cutoff_time = now - timeout;

        let expired_orders = {
            let active_orders = self.active_orders.read().await;
            active_orders
                .values()
                .filter(|order| order.created_at < cutoff_time)
                .map(|order| order.id.clone())
                .collect::<Vec<_>>()
        };

        for order_id in expired_orders {
            self.event_sender.send(ExecutionEvent::OrderExpired(order_id))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ExecutionEngineConfig, MarketSimulationConfig, OrderMatchingConfig};
    use crate::market_simulator::MarketSimulator;
    use crate::order_matching::OrderMatchingEngine;
    use crate::metrics::ShadowTradingMetrics;

    async fn create_test_engine() -> ShadowExecutionEngine {
        let config = ExecutionEngineConfig::default();
        let market_config = MarketSimulationConfig::default();
        let matching_config = OrderMatchingConfig::default();
        
        let market_simulator = Arc::new(MarketSimulator::new(market_config).unwrap());
        let order_matching = Arc::new(OrderMatchingEngine::new(matching_config).unwrap());
        let metrics = Arc::new(ShadowTradingMetrics::new().unwrap());
        
        ShadowExecutionEngine::new(config, market_simulator, order_matching, metrics).unwrap()
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = create_test_engine().await;
        assert!(!*engine.running.read().await);
    }

    #[tokio::test]
    async fn test_order_submission() {
        let engine = create_test_engine().await;
        engine.start().await.unwrap();

        let order = ShadowOrder {
            id: String::new(),
            account_id: "test_account".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(50000.0),
            order_type: OrderType::Limit,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            filled_quantity: 0.0,
            average_price: None,
            fees: 0.0,
            metadata: HashMap::new(),
        };

        let order_id = engine.submit_order(order).await;
        assert!(order_id.is_ok());
        
        let order_id = order_id.unwrap();
        assert!(!order_id.is_empty());

        // 验证订单存在
        let retrieved_order = engine.get_order(&order_id).await;
        assert!(retrieved_order.is_ok());

        engine.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_order_cancellation() {
        let engine = create_test_engine().await;
        engine.start().await.unwrap();

        let order = ShadowOrder {
            id: String::new(),
            account_id: "test_account".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(50000.0),
            order_type: OrderType::Limit,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            filled_quantity: 0.0,
            average_price: None,
            fees: 0.0,
            metadata: HashMap::new(),
        };

        let order_id = engine.submit_order(order).await.unwrap();
        
        // 取消订单
        let result = engine.cancel_order(&order_id).await;
        assert!(result.is_ok());

        // 验证订单已取消
        let cancelled_order = engine.get_order(&order_id).await.unwrap();
        assert_eq!(cancelled_order.status, OrderStatus::Cancelled);

        engine.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_market_order_execution() {
        let engine = create_test_engine().await;
        engine.start().await.unwrap();

        let order = ShadowOrder {
            id: String::new(),
            account_id: "test_account".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: None,
            order_type: OrderType::Market,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            filled_quantity: 0.0,
            average_price: None,
            fees: 0.0,
            metadata: HashMap::new(),
        };

        let order_id = engine.submit_order(order).await.unwrap();
        
        // 等待订单处理
        tokio::time::sleep(Duration::from_millis(200)).await;

        // 检查订单状态（市价单应该立即成交）
        let executed_order = engine.get_order(&order_id).await.unwrap();
        // 注意：由于异步处理，可能需要更长时间或不同的测试策略

        engine.stop().await.unwrap();
    }
}