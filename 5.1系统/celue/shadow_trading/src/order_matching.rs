//! 订单匹配引擎

use crate::config::{OrderMatchingConfig, MatchingAlgorithm, LatencySimulation};
use crate::execution_engine::{OrderSide, ShadowOrder};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

/// 交易执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    pub trade_id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub quantity: f64,
    pub price: f64,
    pub value: f64,
    pub fees: f64,
    pub executed_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// 匹配结果
#[derive(Debug, Clone)]
pub enum MatchingResult {
    NoMatch,
    PartialMatch(Vec<TradeExecution>),
    FullMatch(Vec<TradeExecution>),
}

/// 订单簿条目
#[derive(Debug, Clone)]
struct OrderBookEntry {
    order_id: String,
    price: f64,
    quantity: f64,
    timestamp: DateTime<Utc>,
}

/// 订单簿
#[derive(Debug, Clone)]
struct OrderBook {
    symbol: String,
    bids: BTreeMap<OrderedFloat, VecDeque<OrderBookEntry>>, // 买单按价格降序
    asks: BTreeMap<OrderedFloat, VecDeque<OrderBookEntry>>, // 卖单按价格升序
}

/// 用于BTreeMap排序的浮点数包装器
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl From<f64> for OrderedFloat {
    fn from(f: f64) -> Self {
        OrderedFloat(f)
    }
}

impl OrderBook {
    fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    fn add_order(&mut self, order: &ShadowOrder) {
        let price = order.price.unwrap_or(0.0);
        let entry = OrderBookEntry {
            order_id: order.id.clone(),
            price,
            quantity: order.quantity,
            timestamp: order.created_at,
        };

        match order.side {
            OrderSide::Buy => {
                self.bids.entry(OrderedFloat(price))
                    .or_insert_with(VecDeque::new)
                    .push_back(entry);
            }
            OrderSide::Sell => {
                self.asks.entry(OrderedFloat(price))
                    .or_insert_with(VecDeque::new)
                    .push_back(entry);
            }
        }
    }

    fn remove_order(&mut self, order_id: &str, side: OrderSide) -> Option<OrderBookEntry> {
        let levels = match side {
            OrderSide::Buy => &mut self.bids,
            OrderSide::Sell => &mut self.asks,
        };

        for (_, queue) in levels.iter_mut() {
            if let Some(pos) = queue.iter().position(|entry| entry.order_id == order_id) {
                return Some(queue.remove(pos).unwrap());
            }
        }

        // 清理空的价格级别
        levels.retain(|_, queue| !queue.is_empty());
        None
    }

    fn get_best_bid(&self) -> Option<f64> {
        self.bids.keys().last().map(|k| k.0)
    }

    fn get_best_ask(&self) -> Option<f64> {
        self.asks.keys().next().map(|k| k.0)
    }

    fn get_spread(&self) -> Option<f64> {
        match (self.get_best_ask(), self.get_best_bid()) {
            (Some(ask), Some(bid)) => Some(ask - bid),
            _ => None,
        }
    }
}

/// 订单匹配引擎
pub struct OrderMatchingEngine {
    config: OrderMatchingConfig,
    order_books: Arc<RwLock<HashMap<String, OrderBook>>>,
    trade_history: Arc<RwLock<VecDeque<TradeExecution>>>,
    running: Arc<RwLock<bool>>,
}

impl OrderMatchingEngine {
    pub fn new(config: OrderMatchingConfig) -> Result<Self> {
        Ok(Self {
            config,
            order_books: Arc::new(RwLock::new(HashMap::new())),
            trade_history: Arc::new(RwLock::new(VecDeque::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        info!("Order matching engine started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Order matching engine stopped");
        Ok(())
    }

    pub async fn add_order(&self, order: &ShadowOrder) -> Result<MatchingResult> {
        if order.price.is_none() {
            return Ok(MatchingResult::NoMatch); // 市价单不加入订单簿
        }

        // 模拟网络延迟
        if self.config.latency_simulation.enabled {
            self.simulate_latency().await;
        }

        let symbol = &order.symbol;
        let mut order_books = self.order_books.write().await;
        
        // 确保订单簿存在
        if !order_books.contains_key(symbol) {
            order_books.insert(symbol.clone(), OrderBook::new(symbol.clone()));
        }

        let order_book = order_books.get_mut(symbol).unwrap();
        
        // 尝试匹配
        let matching_result = self.try_match_order(order_book, order).await?;
        
        // 如果未完全成交，将剩余部分加入订单簿
        if let MatchingResult::PartialMatch(_) | MatchingResult::NoMatch = matching_result {
            order_book.add_order(order);
        }

        Ok(matching_result)
    }

    pub async fn cancel_order(&self, order_id: &str, symbol: &str, side: OrderSide) -> Result<bool> {
        let mut order_books = self.order_books.write().await;
        
        if let Some(order_book) = order_books.get_mut(symbol) {
            Ok(order_book.remove_order(order_id, side).is_some())
        } else {
            Ok(false)
        }
    }

    pub async fn get_order_book(&self, symbol: &str) -> Option<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        let order_books = self.order_books.read().await;
        
        if let Some(order_book) = order_books.get(symbol) {
            let mut bids = Vec::new();
            let mut asks = Vec::new();

            // 收集买单数据（价格降序）
            for (price, queue) in order_book.bids.iter().rev() {
                let total_quantity: f64 = queue.iter().map(|entry| entry.quantity).sum();
                bids.push((price.0, total_quantity));
            }

            // 收集卖单数据（价格升序）
            for (price, queue) in order_book.asks.iter() {
                let total_quantity: f64 = queue.iter().map(|entry| entry.quantity).sum();
                asks.push((price.0, total_quantity));
            }

            Some((bids, asks))
        } else {
            None
        }
    }

    pub async fn get_best_prices(&self, symbol: &str) -> Option<(Option<f64>, Option<f64>)> {
        let order_books = self.order_books.read().await;
        
        if let Some(order_book) = order_books.get(symbol) {
            Some((order_book.get_best_bid(), order_book.get_best_ask()))
        } else {
            None
        }
    }

    async fn try_match_order(&self, order_book: &mut OrderBook, order: &ShadowOrder) -> Result<MatchingResult> {
        let mut executions = Vec::new();
        let mut remaining_quantity = order.quantity;
        let order_price = order.price.unwrap();

        match order.side {
            OrderSide::Buy => {
                // 买单匹配最低的卖单
                let mut asks_to_remove = Vec::new();
                
                for (price_key, ask_queue) in order_book.asks.iter_mut() {
                    let ask_price = price_key.0;
                    
                    if ask_price <= order_price && remaining_quantity > 0.0 {
                        let mut queue_empty = false;
                        
                        while let Some(mut ask_entry) = ask_queue.pop_front() {
                            let trade_quantity = remaining_quantity.min(ask_entry.quantity);
                            
                            // 创建交易执行记录
                            let execution = TradeExecution {
                                trade_id: uuid::Uuid::new_v4().to_string(),
                                order_id: order.id.clone(),
                                symbol: order.symbol.clone(),
                                side: order.side,
                                quantity: trade_quantity,
                                price: ask_price,
                                value: trade_quantity * ask_price,
                                fees: 0.0,
                                executed_at: Utc::now(),
                                metadata: HashMap::new(),
                            };
                            
                            executions.push(execution);
                            remaining_quantity -= trade_quantity;
                            
                            if ask_entry.quantity > trade_quantity {
                                // 部分成交，剩余部分放回队列
                                ask_entry.quantity -= trade_quantity;
                                ask_queue.push_front(ask_entry);
                                break;
                            }
                            
                            if ask_queue.is_empty() {
                                queue_empty = true;
                                break;
                            }
                            
                            if remaining_quantity <= 0.0 {
                                break;
                            }
                        }
                        
                        if queue_empty {
                            asks_to_remove.push(*price_key);
                        }
                    }
                }
                
                // 移除空的价格级别
                for price_key in asks_to_remove {
                    order_book.asks.remove(&price_key);
                }
            }
            OrderSide::Sell => {
                // 卖单匹配最高的买单
                let mut bids_to_remove = Vec::new();
                
                for (price_key, bid_queue) in order_book.bids.iter_mut().rev() {
                    let bid_price = price_key.0;
                    
                    if bid_price >= order_price && remaining_quantity > 0.0 {
                        let mut queue_empty = false;
                        
                        while let Some(mut bid_entry) = bid_queue.pop_front() {
                            let trade_quantity = remaining_quantity.min(bid_entry.quantity);
                            
                            let execution = TradeExecution {
                                trade_id: uuid::Uuid::new_v4().to_string(),
                                order_id: order.id.clone(),
                                symbol: order.symbol.clone(),
                                side: order.side,
                                quantity: trade_quantity,
                                price: bid_price,
                                value: trade_quantity * bid_price,
                                fees: 0.0,
                                executed_at: Utc::now(),
                                metadata: HashMap::new(),
                            };
                            
                            executions.push(execution);
                            remaining_quantity -= trade_quantity;
                            
                            if bid_entry.quantity > trade_quantity {
                                bid_entry.quantity -= trade_quantity;
                                bid_queue.push_front(bid_entry);
                                break;
                            }
                            
                            if bid_queue.is_empty() {
                                queue_empty = true;
                                break;
                            }
                            
                            if remaining_quantity <= 0.0 {
                                break;
                            }
                        }
                        
                        if queue_empty {
                            bids_to_remove.push(*price_key);
                        }
                    }
                }
                
                // 移除空的价格级别
                for price_key in bids_to_remove {
                    order_book.bids.remove(&price_key);
                }
            }
        }

        // 记录交易历史
        if !executions.is_empty() {
            let mut trade_history = self.trade_history.write().await;
            for execution in &executions {
                trade_history.push_back(execution.clone());
                
                // 限制历史记录大小
                if trade_history.len() > 100000 {
                    trade_history.pop_front();
                }
            }
        }

        // 返回匹配结果
        if executions.is_empty() {
            Ok(MatchingResult::NoMatch)
        } else if remaining_quantity > 0.0 {
            Ok(MatchingResult::PartialMatch(executions))
        } else {
            Ok(MatchingResult::FullMatch(executions))
        }
    }

    async fn simulate_latency(&self) {
        if !self.config.latency_simulation.enabled {
            return;
        }

        let mut rng = rand::thread_rng();
        let base_latency = self.config.latency_simulation.base_latency_us;
        let jitter = rng.gen_range(0..self.config.latency_simulation.jitter_range_us);
        let mut total_latency = base_latency + jitter;

        // 网络拥塞模拟
        if self.config.latency_simulation.enable_congestion {
            if rng.gen::<f64>() < self.config.latency_simulation.congestion_probability {
                total_latency += self.config.latency_simulation.congestion_extra_latency_us;
            }
        }

        if total_latency > 0 {
            sleep(Duration::from_micros(total_latency)).await;
        }
    }

    pub async fn get_trade_history(&self, limit: Option<usize>) -> Vec<TradeExecution> {
        let history = self.trade_history.read().await;
        if let Some(limit) = limit {
            history.iter().rev().take(limit).cloned().collect()
        } else {
            history.iter().cloned().collect()
        }
    }

    pub async fn get_market_depth(&self, symbol: &str, depth: usize) -> Option<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        let order_books = self.order_books.read().await;
        
        if let Some(order_book) = order_books.get(symbol) {
            let mut bids = Vec::new();
            let mut asks = Vec::new();

            // 收集买单深度
            for (price, queue) in order_book.bids.iter().rev().take(depth) {
                let total_quantity: f64 = queue.iter().map(|entry| entry.quantity).sum();
                bids.push((price.0, total_quantity));
            }

            // 收集卖单深度
            for (price, queue) in order_book.asks.iter().take(depth) {
                let total_quantity: f64 = queue.iter().map(|entry| entry.quantity).sum();
                asks.push((price.0, total_quantity));
            }

            Some((bids, asks))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_engine::{ShadowOrder, OrderType, OrderStatus};
    use std::collections::HashMap;

    fn create_test_order(side: OrderSide, price: f64, quantity: f64) -> ShadowOrder {
        ShadowOrder {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: "test".to_string(),
            symbol: "BTC/USDT".to_string(),
            side,
            quantity,
            price: Some(price),
            order_type: OrderType::Limit,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            filled_quantity: 0.0,
            average_price: None,
            fees: 0.0,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_matching_engine_creation() {
        let config = OrderMatchingConfig::default();
        let engine = OrderMatchingEngine::new(config);
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_order_matching() {
        let config = OrderMatchingConfig::default();
        let engine = OrderMatchingEngine::new(config).unwrap();
        engine.start().await.unwrap();

        // 添加卖单
        let sell_order = create_test_order(OrderSide::Sell, 50000.0, 1.0);
        let result = engine.add_order(&sell_order).await.unwrap();
        assert!(matches!(result, MatchingResult::NoMatch));

        // 添加匹配的买单
        let buy_order = create_test_order(OrderSide::Buy, 50000.0, 1.0);
        let result = engine.add_order(&buy_order).await.unwrap();
        assert!(matches!(result, MatchingResult::FullMatch(_)));

        engine.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_partial_matching() {
        let config = OrderMatchingConfig::default();
        let engine = OrderMatchingEngine::new(config).unwrap();
        engine.start().await.unwrap();

        // 添加小的卖单
        let sell_order = create_test_order(OrderSide::Sell, 50000.0, 0.5);
        engine.add_order(&sell_order).await.unwrap();

        // 添加大的买单
        let buy_order = create_test_order(OrderSide::Buy, 50000.0, 1.0);
        let result = engine.add_order(&buy_order).await.unwrap();
        assert!(matches!(result, MatchingResult::PartialMatch(_)));

        engine.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_order_book_depth() {
        let config = OrderMatchingConfig::default();
        let engine = OrderMatchingEngine::new(config).unwrap();
        engine.start().await.unwrap();

        // 添加多个订单
        let orders = vec![
            create_test_order(OrderSide::Buy, 49900.0, 1.0),
            create_test_order(OrderSide::Buy, 49800.0, 2.0),
            create_test_order(OrderSide::Sell, 50100.0, 1.5),
            create_test_order(OrderSide::Sell, 50200.0, 2.5),
        ];

        for order in orders {
            engine.add_order(&order).await.unwrap();
        }

        let depth = engine.get_market_depth("BTC/USDT", 5).await;
        assert!(depth.is_some());

        let (bids, asks) = depth.unwrap();
        assert!(!bids.is_empty());
        assert!(!asks.is_empty());

        engine.stop().await.unwrap();
    }
}