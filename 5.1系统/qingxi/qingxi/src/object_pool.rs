#![allow(dead_code)]
use crate::types::{MarketDataSnapshot, OrderBook};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 高性能对象池，支持多线程复用。
pub struct ObjectPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    factory: Arc<dyn Fn() -> T + Send + Sync>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    pub fn new(factory: impl Fn() -> T + Send + Sync + 'static, max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            factory: Arc::new(factory),
            max_size,
        }
    }

    pub async fn get(&self) -> T {
        let mut pool = self.pool.lock().await;
        pool.pop_front().unwrap_or_else(|| (self.factory)())
    }

    pub async fn release(&self, obj: T) {
        let mut pool = self.pool.lock().await;
        if pool.len() < self.max_size {
            pool.push_back(obj);
        }
    }

    // 保持向后兼容性
    pub async fn put(&self, obj: T) {
        self.release(obj).await;
    }
}

pub trait Resettable {
    fn reset(&mut self);
}

// Implement Resettable for your types
impl Resettable for MarketDataSnapshot {
    fn reset(&mut self) {
        self.orderbook = None;
        self.trades.clear();
    }
}

impl Resettable for OrderBook {
    fn reset(&mut self) {
        self.bids.clear();
        self.asks.clear();
    }
}
