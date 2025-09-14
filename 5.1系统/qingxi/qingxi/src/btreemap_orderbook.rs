#![allow(dead_code)]
//! # BTreeMap优化订单簿模块
//!
//! 使用BTreeMap替代Vec实现高性能订单簿操作
//! 提供O(log n)插入、删除和O(1)最优价格访问
//!
//! ## 性能优势
//! - O(log n)插入/删除 vs Vec的O(n)
//! - O(1)最优价格访问
//! - 自动排序维护
//! - 内存效率优化

use crate::types::*;
use std::collections::BTreeMap;
use ordered_float::OrderedFloat;

/// 高性能BTreeMap订单簿实现
#[derive(Debug, Clone)]
pub struct OptimizedOrderBook {
    pub symbol: String,
    /// 买盘：价格 -> 数量（降序排列）
    pub bids: BTreeMap<OrderedFloat<f64>, f64>,
    /// 卖盘：价格 -> 数量（升序排列）
    pub asks: BTreeMap<OrderedFloat<f64>, f64>,
    pub timestamp: u64,
    /// 性能统计
    performance_stats: OrderBookStats,
}

impl OptimizedOrderBook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            timestamp: 0,
            performance_stats: OrderBookStats::default(),
        }
    }

    /// 更新买盘价格
    pub fn update_bid(&mut self, price: f64, quantity: f64) {
        let start = std::time::Instant::now();
        
        let price_key = OrderedFloat(price);
        if quantity > 0.0 {
            self.bids.insert(price_key, quantity);
        } else {
            self.bids.remove(&price_key);
        }
        
        self.performance_stats.bid_updates += 1;
        self.performance_stats.update_time_ns += start.elapsed().as_nanos() as u64;
    }

    /// 更新卖盘价格
    pub fn update_ask(&mut self, price: f64, quantity: f64) {
        let start = std::time::Instant::now();
        
        let price_key = OrderedFloat(price);
        if quantity > 0.0 {
            self.asks.insert(price_key, quantity);
        } else {
            self.asks.remove(&price_key);
        }
        
        self.performance_stats.ask_updates += 1;
        self.performance_stats.update_time_ns += start.elapsed().as_nanos() as u64;
    }

    /// 获取最佳买价（O(1)）
    pub fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.iter().next_back().map(|(price, &quantity)| (price.0, quantity))
    }

    /// 获取最佳卖价（O(1)）
    pub fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.iter().next().map(|(price, &quantity)| (price.0, quantity))
    }

    /// 获取买卖价差
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid_price, _)), Some((ask_price, _))) => {
                Some(ask_price - bid_price)
            }
            _ => None,
        }
    }

    /// 获取中间价
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid_price, _)), Some((ask_price, _))) => {
                Some((bid_price + ask_price) / 2.0)
            }
            _ => None,
        }
    }

    /// 获取指定深度的买盘
    pub fn get_bids(&self, depth: usize) -> Vec<OrderBookEntry> {
        self.bids.iter()
            .rev()
            .take(depth)
            .map(|(price, &quantity)| OrderBookEntry {
                price: OrderedFloat(price.0),
                quantity: OrderedFloat(quantity),
            })
            .collect()
    }

    /// 获取指定深度的卖盘
    pub fn get_asks(&self, depth: usize) -> Vec<OrderBookEntry> {
        self.asks.iter()
            .take(depth)
            .map(|(price, &quantity)| OrderBookEntry {
                price: OrderedFloat(price.0),
                quantity: OrderedFloat(quantity),
            })
            .collect()
    }

    /// 批量更新订单簿
    pub fn update_batch(&mut self, updates: Vec<OrderBookUpdate>) {
        let start = std::time::Instant::now();
        
        for update in updates {
            match update.side {
                OrderSide::Buy => self.update_bid(update.price, update.quantity),
                OrderSide::Sell => self.update_ask(update.price, update.quantity),
            }
        }
        
        self.performance_stats.batch_updates += 1;
        self.performance_stats.batch_update_time_ns += start.elapsed().as_nanos() as u64;
    }

    /// 从传统OrderBook转换
    pub fn from_traditional_orderbook(orderbook: &OrderBook) -> Self {
        let mut optimized = Self::new(orderbook.symbol.to_string());
        optimized.timestamp = orderbook.timestamp.as_u64();

        // 转换买盘
        for entry in &orderbook.bids {
            optimized.update_bid(entry.price.0, entry.quantity.0);
        }

        // 转换卖盘
        for entry in &orderbook.asks {
            optimized.update_ask(entry.price.0, entry.quantity.0);
        }

        optimized
    }

    /// 转换为传统OrderBook
    pub fn to_traditional_orderbook(&self) -> OrderBook {
        OrderBook {
            symbol: Symbol::from_string(&self.symbol).unwrap_or_else(|_| Symbol::new("UNKNOWN", "USDT")),
            bids: self.get_bids(100), // 获取前100档
            asks: self.get_asks(100),
            timestamp: crate::high_precision_time::Nanos::from_u64(self.timestamp),
            checksum: None,
            sequence_id: None,
            source: "optimized".to_string(),
        }
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &OrderBookStats {
        &self.performance_stats
    }

    /// 重置性能统计
    pub fn reset_performance_stats(&mut self) {
        self.performance_stats = OrderBookStats::default();
    }

    /// 计算订单簿质量评分
    pub fn quality_score(&self) -> f64 {
        let mut score = 0.0;

        // 深度评分（0-30分）
        let bid_depth = self.bids.len();
        let ask_depth = self.asks.len();
        let depth_score = ((bid_depth + ask_depth).min(20) as f64 / 20.0) * 30.0;
        score += depth_score;

        // 价差评分（0-25分）
        if let (Some((bid_price, _)), Some((ask_price, _))) = (self.best_bid(), self.best_ask()) {
            let spread = ask_price - bid_price;
            let mid_price = (bid_price + ask_price) / 2.0;
            let spread_ratio = spread / mid_price;
            
            // 获取配置化的评分参数
            let max_score = if let Ok(settings) = crate::settings::Settings::load() {
                settings.algorithm_scoring.liquidity_score_max
            } else {
                25.0
            };
            
            // 价差越小评分越高
            let spread_score = (1.0 - spread_ratio.min(0.05) / 0.05) * max_score;
            score += spread_score;
        }

        // 流动性评分（0-25分）
        let bid_volume: f64 = self.bids.values().sum();
        let ask_volume: f64 = self.asks.values().sum();
        let total_volume = bid_volume + ask_volume;
        
        // 获取配置化的流动性参数
        let (baseline, max_score) = if let Ok(settings) = crate::settings::Settings::load() {
            (settings.algorithm_scoring.liquidity_score_baseline, 
             settings.algorithm_scoring.liquidity_score_max)
        } else {
            (1000.0, 25.0)
        };
        
        let liquidity_score = (total_volume / baseline).min(1.0) * max_score;
        score += liquidity_score;

        // 平衡性评分（0-20分）
        if bid_volume > 0.0 && ask_volume > 0.0 {
            let balance_ratio = bid_volume.min(ask_volume) / bid_volume.max(ask_volume);
            let balance_score = balance_ratio * 20.0;
            score += balance_score;
        }

        score.min(100.0)
    }

    /// 清理异常数据
    pub fn cleanup_anomalies(&mut self, max_spread_ratio: f64) {
        if let Some(mid_price) = self.mid_price() {
            let max_spread = mid_price * max_spread_ratio;

            // 清理异常买盘（价格过低）
            let min_bid_price = mid_price - max_spread;
            self.bids.retain(|price, _| price.0 >= min_bid_price);

            // 清理异常卖盘（价格过高）
            let max_ask_price = mid_price + max_spread;
            self.asks.retain(|price, _| price.0 <= max_ask_price);
        }
    }

    /// 智能深度截断
    pub fn truncate_depth(&mut self, max_depth: usize) {
        // 保留最优的max_depth档价格
        if self.bids.len() > max_depth {
            let keys_to_remove: Vec<_> = self.bids.keys()
                .take(self.bids.len() - max_depth)
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                self.bids.remove(&key);
            }
        }

        if self.asks.len() > max_depth {
            let keys_to_remove: Vec<_> = self.asks.keys()
                .skip(max_depth)
                .cloned()
                .collect();
            
            for key in keys_to_remove {
                self.asks.remove(&key);
            }
        }
    }
}

/// 订单簿更新事件
#[derive(Debug, Clone)]
pub struct OrderBookUpdate {
    pub price: f64,
    pub quantity: f64,
    pub side: OrderSide,
}

/// 订单方向
#[derive(Debug, Clone, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// 订单簿性能统计
#[derive(Debug, Clone, Default)]
pub struct OrderBookStats {
    pub bid_updates: u64,
    pub ask_updates: u64,
    pub batch_updates: u64,
    pub update_time_ns: u64,
    pub batch_update_time_ns: u64,
    pub access_time_ns: u64,
    pub access_count: u64,
}

impl OrderBookStats {
    pub fn average_update_time(&self) -> f64 {
        let total_updates = self.bid_updates + self.ask_updates;
        if total_updates > 0 {
            self.update_time_ns as f64 / total_updates as f64
        } else {
            0.0
        }
    }

    pub fn average_batch_update_time(&self) -> f64 {
        if self.batch_updates > 0 {
            self.batch_update_time_ns as f64 / self.batch_updates as f64
        } else {
            0.0
        }
    }

    pub fn average_access_time(&self) -> f64 {
        if self.access_count > 0 {
            self.access_time_ns as f64 / self.access_count as f64
        } else {
            0.0
        }
    }
}

/// BTreeMap订单簿管理器
pub struct OptimizedOrderBookManager {
    /// 订单簿映射：symbol -> OptimizedOrderBook
    orderbooks: BTreeMap<String, OptimizedOrderBook>,
    /// 全局统计
    global_stats: GlobalOrderBookStats,
}

impl OptimizedOrderBookManager {
    pub fn new() -> Self {
        Self {
            orderbooks: BTreeMap::new(),
            global_stats: GlobalOrderBookStats::default(),
        }
    }

    /// 获取或创建订单簿
    pub fn get_or_create_orderbook(&mut self, symbol: &str) -> &mut OptimizedOrderBook {
        self.orderbooks.entry(symbol.to_string())
            .or_insert_with(|| OptimizedOrderBook::new(symbol.to_string()))
    }

    /// 更新订单簿
    pub fn update_orderbook(&mut self, symbol: &str, updates: Vec<OrderBookUpdate>) {
        let start = std::time::Instant::now();
        
        let orderbook = self.get_or_create_orderbook(symbol);
        orderbook.update_batch(updates);
        
        self.global_stats.total_updates += 1;
        self.global_stats.total_update_time_ns += start.elapsed().as_nanos() as u64;
    }

    /// 获取订单簿
    pub fn get_orderbook(&self, symbol: &str) -> Option<&OptimizedOrderBook> {
        self.orderbooks.get(symbol)
    }

    /// 获取所有订单簿的质量评分
    pub fn get_all_quality_scores(&self) -> Vec<(String, f64)> {
        self.orderbooks.iter()
            .map(|(symbol, orderbook)| (symbol.clone(), orderbook.quality_score()))
            .collect()
    }

    /// 清理所有订单簿的异常数据
    pub fn cleanup_all_anomalies(&mut self, max_spread_ratio: f64) {
        for orderbook in self.orderbooks.values_mut() {
            orderbook.cleanup_anomalies(max_spread_ratio);
        }
    }

    /// 获取全局统计
    pub fn get_global_stats(&self) -> &GlobalOrderBookStats {
        &self.global_stats
    }
}

/// 全局订单簿统计
#[derive(Debug, Clone, Default)]
pub struct GlobalOrderBookStats {
    pub total_updates: u64,
    pub total_update_time_ns: u64,
    pub active_orderbooks: usize,
    pub total_depth: usize,
}

impl GlobalOrderBookStats {
    pub fn average_update_time(&self) -> f64 {
        if self.total_updates > 0 {
            self.total_update_time_ns as f64 / self.total_updates as f64
        } else {
            0.0
        }
    }
}
