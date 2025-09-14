#![allow(dead_code)]
//! # 桶排序订单簿实现 - O(1)更新性能
//! 
//! 🚀 阶段2优化：实现桶排序订单簿，目标实现O(1)更新性能

#[allow(dead_code)]
use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use pdqsort;

/// 🚀 桶排序订单簿 - 针对高频更新优化
#[derive(Debug, Clone)]
pub struct BucketOrderBook {
    pub symbol: String,
    pub timestamp: u64,
    pub source: String,
    
    // 价格桶 - 核心数据结构
    bid_buckets: Vec<Vec<OrderBookEntry>>,
    ask_buckets: Vec<Vec<OrderBookEntry>>,
    
    // 价格映射配置
    price_min: f64,
    #[allow(dead_code)]  // 预留字段，用于价格范围验证
    price_max: f64,
    price_scale: f64,
    bucket_count: usize,
    
    // 性能统计
    update_count: u64,
    bucket_hits: u64,
}

impl BucketOrderBook {
    /// 创建新的桶排序订单簿
    pub fn new(symbol: String, source: String, price_range: (f64, f64)) -> Self {
        const BUCKET_COUNT: usize = 4096;
        let price_scale = BUCKET_COUNT as f64 / (price_range.1 - price_range.0);
        
        Self {
            symbol,
            source,
            timestamp: 0,
            bid_buckets: vec![Vec::with_capacity(8); BUCKET_COUNT],
            ask_buckets: vec![Vec::with_capacity(8); BUCKET_COUNT],
            price_min: price_range.0,
            price_max: price_range.1,
            price_scale,
            bucket_count: BUCKET_COUNT,
            update_count: 0,
            bucket_hits: 0,
        }
    }
    
    /// 🚀 O(1)价格到桶索引映射
    #[inline]
    fn price_to_bucket(&self, price: f64) -> usize {
        let normalized = (price - self.price_min) * self.price_scale;
        (normalized as usize).min(self.bucket_count - 1)
    }
    
    /// 🚀 O(1)买单更新
    pub fn update_bid(&mut self, price: f64, quantity: f64) {
        self.update_count += 1;
        let bucket_idx = self.price_to_bucket(price);
        let bucket = &mut self.bid_buckets[bucket_idx];
        
        // 在小桶内查找 - 平均O(1)，最坏O(k)其中k很小
        if let Some(entry) = bucket.iter_mut().find(|e| e.price.0 == price) {
            if quantity > 0.0 {
                entry.quantity = quantity.into();
                self.bucket_hits += 1;
            } else {
                bucket.retain(|e| e.price.0 != price);
            }
        } else if quantity > 0.0 && bucket.len() < 8 {
            bucket.push(OrderBookEntry {
                price: price.into(),
                quantity: quantity.into(),
            });
            self.bucket_hits += 1;
        }
    }
    
    /// 🚀 O(1)卖单更新
    pub fn update_ask(&mut self, price: f64, quantity: f64) {
        self.update_count += 1;
        let bucket_idx = self.price_to_bucket(price);
        let bucket = &mut self.ask_buckets[bucket_idx];
        
        if let Some(entry) = bucket.iter_mut().find(|e| e.price.0 == price) {
            if quantity > 0.0 {
                entry.quantity = quantity.into();
                self.bucket_hits += 1;
            } else {
                bucket.retain(|e| e.price.0 != price);
            }
        } else if quantity > 0.0 && bucket.len() < 8 {
            bucket.push(OrderBookEntry {
                price: price.into(),
                quantity: quantity.into(),
            });
            self.bucket_hits += 1;
        }
    }
    
    /// 🚀 获取最佳买价 - O(k)其中k是非空桶数
    pub fn best_bid(&self) -> Option<OrderBookEntry> {
        for bucket in self.bid_buckets.iter().rev() {
            if !bucket.is_empty() {
                let best = bucket.iter().max_by(|a, b| a.price.cmp(&b.price))?;
                return Some(best.clone());
            }
        }
        None
    }
    
    /// 🚀 获取最佳卖价 - O(k)其中k是非空桶数  
    pub fn best_ask(&self) -> Option<OrderBookEntry> {
        for bucket in self.ask_buckets.iter() {
            if !bucket.is_empty() {
                let best = bucket.iter().min_by(|a, b| a.price.cmp(&b.price))?;
                return Some(best.clone());
            }
        }
        None
    }
    
    /// 🚀 导出为标准OrderBook格式
    pub fn to_orderbook(&self) -> OrderBook {
        let mut bids = Vec::new();
        let mut asks = Vec::new();
        
        // 收集所有买单并排序
        for bucket in &self.bid_buckets {
            bids.extend_from_slice(bucket);
        }
        pdqsort::sort_by(&mut bids, |a, b| b.price.cmp(&a.price));
        
        // 收集所有卖单并排序
        for bucket in &self.ask_buckets {
            asks.extend_from_slice(bucket);
        }
        pdqsort::sort_by(&mut asks, |a, b| a.price.cmp(&b.price));
        
        OrderBook {
            symbol: Symbol::from_string(&self.symbol).unwrap_or_else(|_| Symbol::new("UNKNOWN", "USDT")),
            source: self.source.clone(),
            timestamp: crate::high_precision_time::Nanos::from_millis(self.timestamp as i64),
            sequence_id: None,
            checksum: None,
            bids,
            asks,
        }
    }
    
    /// 🚀 从标准OrderBook批量导入
    pub fn from_orderbook(&mut self, orderbook: &OrderBook) {
        self.symbol = orderbook.symbol.as_pair();
        self.source = orderbook.source.clone();
        self.timestamp = orderbook.timestamp.as_millis() as u64;
        
        // 清空现有桶
        for bucket in &mut self.bid_buckets {
            bucket.clear();
        }
        for bucket in &mut self.ask_buckets {
            bucket.clear();
        }
        
        // 批量插入买单
        for entry in &orderbook.bids {
            self.update_bid(entry.price.0, entry.quantity.0);
        }
        
        // 批量插入卖单
        for entry in &orderbook.asks {
            self.update_ask(entry.price.0, entry.quantity.0);
        }
    }
    
    /// 🚀 阶段2核心优化：批量更新条目
    pub fn update_with_entries(&mut self, bids: Vec<OrderBookEntry>, asks: Vec<OrderBookEntry>) {
        // 清空旧数据
        for bucket in &mut self.bid_buckets {
            bucket.clear();
        }
        for bucket in &mut self.ask_buckets {
            bucket.clear();
        }

        // 批量添加买单
        for bid in bids {
            self.add_bid(bid);
        }

        // 批量添加卖单  
        for ask in asks {
            self.add_ask(ask);
        }

        self.timestamp = chrono::Utc::now().timestamp_millis() as u64;
        self.update_count += 1;
    }

    /// 🚀 转换为标准OrderBook格式 - 优化版本
    pub fn to_standard_orderbook(&self) -> OrderBook {
        let mut bids = Vec::with_capacity(1000);
        let mut asks = Vec::with_capacity(1000);
        
        // 🚀 串行收集所有条目（避免并行迭代器复杂性）
        for bucket in &self.bid_buckets {
            bids.extend_from_slice(bucket);
        }
            
        for bucket in &self.ask_buckets {
            asks.extend_from_slice(bucket);
        }
        
        // 🚀 使用pdqsort高性能排序
        if bids.len() > 1 {
            pdqsort::sort_by(&mut bids, |a, b| b.price.cmp(&a.price));
        }
        
        if asks.len() > 1 {
            pdqsort::sort_by(&mut asks, |a, b| a.price.cmp(&b.price));
        }
        
        // 转换symbol - 使用Symbol::from_string创建
        let symbol = Symbol::from_string(&self.symbol).unwrap_or_else(|_| Symbol::new("UNKNOWN", "USDT"));
        
        OrderBook {
            symbol,
            source: self.source.clone(),
            timestamp: crate::high_precision_time::Nanos::from_millis(self.timestamp as i64),
            bids,
            asks,
            sequence_id: None,
            checksum: None,
        }
    }

    /// 性能统计信息
    pub fn get_performance_stats(&self) -> (u64, u64, f64) {
        let hit_rate = if self.update_count > 0 {
            self.bucket_hits as f64 / self.update_count as f64
        } else {
            0.0
        };
        (self.update_count, self.bucket_hits, hit_rate)
    }
    
    /// 🚀 性能统计
    pub fn performance_stats(&self) -> (u64, u64, f64) {
        let hit_rate = if self.update_count > 0 {
            self.bucket_hits as f64 / self.update_count as f64
        } else {
            0.0
        };
        (self.update_count, self.bucket_hits, hit_rate)
    }
    
    /// 🚀 重置性能统计
    pub fn reset_performance_stats(&mut self) {
        self.update_count = 0;
        self.bucket_hits = 0;
    }
    
    /// 🚀 添加买单
    pub fn add_bid(&mut self, entry: OrderBookEntry) {
        let price = entry.price.into_inner();
        let quantity = entry.quantity.into_inner();
        self.update_bid(price, quantity);
    }
    
    /// 🚀 添加卖单
    pub fn add_ask(&mut self, entry: OrderBookEntry) {
        let price = entry.price.into_inner();
        let quantity = entry.quantity.into_inner();
        self.update_ask(price, quantity);
    }
}

/// 🚀 桶排序订单簿管理器
pub struct BucketOrderBookManager {
    books: Arc<RwLock<HashMap<(String, String), BucketOrderBook>>>,
    price_ranges: HashMap<String, (f64, f64)>,
}

impl BucketOrderBookManager {
    pub fn new() -> Self {
        let mut price_ranges = HashMap::new();
        
        // 预设常见交易对的价格范围
        price_ranges.insert("BTCUSDT".to_string(), (20000.0, 100000.0));
        price_ranges.insert("ETHUSDT".to_string(), (1000.0, 10000.0));
        price_ranges.insert("SOLUSDT".to_string(), (10.0, 500.0));
        price_ranges.insert("BNBUSDT".to_string(), (200.0, 1000.0));
        
        Self {
            books: Arc::new(RwLock::new(HashMap::new())),
            price_ranges,
        }
    }
    
    /// 🚀 获取或创建桶排序订单簿
    pub async fn get_or_create_bucket_book(&self, symbol: &str, source: &str) -> BucketOrderBook {
        let key = (symbol.to_string(), source.to_string());
        
        {
            let books = self.books.read().await;
            if let Some(book) = books.get(&key) {
                return book.clone();
            }
        }
        
        // 创建新的桶排序订单簿
        let price_range = self.price_ranges.get(symbol)
            .copied()
            .unwrap_or((1.0, 1000000.0)); // 默认范围
            
        let book = BucketOrderBook::new(symbol.to_string(), source.to_string(), price_range);
        
        {
            let mut books = self.books.write().await;
            books.insert(key, book.clone());
        }
        
        book
    }
    
    /// 🚀 更新桶排序订单簿
    pub async fn update_bucket_book(&self, symbol: &str, source: &str, orderbook: &OrderBook) {
        let key = (symbol.to_string(), source.to_string());
        
        let mut books = self.books.write().await;
        if let Some(book) = books.get_mut(&key) {
            book.from_orderbook(orderbook);
        }
    }
    
    /// 🚀 获取所有桶排序订单簿的性能统计
    pub async fn get_all_performance_stats(&self) -> Vec<(String, String, u64, u64, f64)> {
        let books = self.books.read().await;
        let mut stats = Vec::new();
        
        for ((symbol, source), book) in books.iter() {
            let (updates, hits, hit_rate) = book.performance_stats();
            stats.push((symbol.clone(), source.clone(), updates, hits, hit_rate));
        }
        
        stats
    }
}
