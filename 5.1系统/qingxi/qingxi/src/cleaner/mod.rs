#![allow(dead_code)]
//! # 数据清洗模块
//!
//! 负责清洗和规范化从交易所收集的原始市场数据。

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};
use async_trait::async_trait;

use crate::types::*;
use crate::errors::MarketDataError;

/// 清洗统计信息
#[derive(Debug, Default, Clone)]
pub struct CleaningStats {
    pub total_processed: u64,
    pub total_time: std::time::Duration,
    pub simd_optimizations: u64,
    pub memory_allocations_saved: u64,
    pub orderbooks_processed: u64,
    pub bucket_optimizations: u64,
}

// 仅保留核心优化清洗器
pub mod optimized_cleaner;

// 高级清洗器模块
pub mod progressive_cleaner;

pub use optimized_cleaner::OptimizedDataCleaner;
pub use progressive_cleaner::ProgressiveDataCleaner;

/// 数据清洗器特性
#[async_trait]
pub trait DataCleaner: Send + Sync {
    /// 清洗市场数据
    async fn clean(&self, data: MarketDataSnapshot) -> Result<MarketDataSnapshot, MarketDataError>;
    
    /// 启动清洗处理
    async fn start(&mut self) -> Result<(), MarketDataError>;
    
    /// 停止清洗处理
    async fn stop(&mut self) -> Result<(), MarketDataError>;
    
    /// 获取性能统计信息
    async fn get_stats(&self) -> CleaningStats {
        CleaningStats::default()
    }
    
    /// 重置统计信息
    async fn reset_stats(&self) {
        // 默认实现为空
    }
}

/// 基础数据清洗器
pub struct BaseDataCleaner {
    /// 输入通道
    input_rx: Arc<RwLock<Option<flume::Receiver<MarketDataSnapshot>>>>,
    /// 输出通道
    output_tx: flume::Sender<MarketDataSnapshot>,
    /// 是否应该停止
    should_stop: Arc<RwLock<bool>>,
    /// 处理任务句柄
    task_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl BaseDataCleaner {
    /// 创建新的数据清洗器
    pub fn new(
        input_rx: flume::Receiver<MarketDataSnapshot>,
        output_tx: flume::Sender<MarketDataSnapshot>,
    ) -> Self {
        Self {
            input_rx: Arc::new(RwLock::new(Some(input_rx))),
            output_tx,
            should_stop: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(RwLock::new(None)),
        }
    }
    
    /// 检查是否应该停止
    #[allow(dead_code)]
    async fn check_should_stop(&self) -> bool {
        let stop_lock = self.should_stop.read().await;
        *stop_lock
    }
    
    /// 设置停止标志
    async fn set_should_stop(&self, value: bool) {
        let mut stop_lock = self.should_stop.write().await;
        *stop_lock = value;
    }
    
    /// 标准化订单簿
    fn normalize_orderbook(&self, mut orderbook: OrderBook) -> OrderBook {
        // 🚀 OPTIMIZED: 使用不稳定排序提升性能
        orderbook.bids.sort_unstable_by(|a, b| b.price.cmp(&a.price));
        orderbook.asks.sort_unstable_by(|a, b| a.price.cmp(&b.price));
        
        // 🚀 OPTIMIZED: 原地过滤，避免额外分配
        orderbook.bids.retain(|entry| entry.quantity > 0.0.into());
        orderbook.asks.retain(|entry| entry.quantity > 0.0.into());
        
        orderbook
    }
    
    /// 增强的订单簿验证 - 支持空数据处理
    fn validate_orderbook(&self, orderbook: &OrderBook) -> Result<(), MarketDataError> {
        // 🚀 ENHANCED: 增强的空数据处理逻辑
        if orderbook.bids.is_empty() && orderbook.asks.is_empty() {
            warn!("🔍 完全空的订单簿: 交易所={}, 符号={}", orderbook.source, orderbook.symbol.as_pair());
            return Ok(()); // 允许完全空的订单簿
        }
        
        if orderbook.bids.is_empty() {
            info!("📊 订单簿缺少买单: 交易所={}, 符号={}", orderbook.source, orderbook.symbol.as_pair());
            return Ok(()); // 允许单边空订单簿
        }
        
        if orderbook.asks.is_empty() {
            info!("📊 订单簿缺少卖单: 交易所={}, 符号={}", orderbook.source, orderbook.symbol.as_pair());
            return Ok(()); // 允许单边空订单簿
        }
        
        // 🚀 OPTIMIZED: 直接访问第一个元素，避免Option处理开销
        let best_bid_price = orderbook.bids[0].price;
        let best_ask_price = orderbook.asks[0].price;
        
        if best_bid_price >= best_ask_price {
            warn!("⚠️  价格倒挂: 买一价 ({}) >= 卖一价 ({}) - 交易所={}, 符号={}", 
                  best_bid_price.0, best_ask_price.0, orderbook.source, orderbook.symbol.as_pair());
            // 记录但不阻止数据处理
        }
        
        Ok(())
    }
    
    /// 清洗订单簿数据
    fn clean_orderbook(&self, orderbook: Option<OrderBook>) -> Option<OrderBook> {
        match orderbook {
            Some(ob) => {
                let normalized = self.normalize_orderbook(ob);
                match self.validate_orderbook(&normalized) {
                    Ok(_) => Some(normalized),
                    Err(e) => {
                        warn!("订单簿验证失败: {}", e);
                        None
                    }
                }
            },
            None => None,
        }
    }
    
    /// 清洗交易数据
    fn clean_trades(&self, mut trades: Vec<TradeUpdate>) -> Vec<TradeUpdate> {
        // 🚀 OPTIMIZED: 原地过滤零数量交易
        trades.retain(|trade| trade.quantity > 0.0.into());
        
        // 🚀 OPTIMIZED: 使用不稳定排序提升性能
        trades.sort_unstable_by_key(|trade| trade.timestamp);
        
        trades
    }
}

#[async_trait]
impl DataCleaner for BaseDataCleaner {
    async fn clean(&self, mut data: MarketDataSnapshot) -> Result<MarketDataSnapshot, MarketDataError> {
        // 清洗订单簿数据
        data.orderbook = self.clean_orderbook(data.orderbook);
        
        // 清洗交易数据
        data.trades = self.clean_trades(data.trades);
        
        Ok(data)
    }
    
    async fn start(&mut self) -> Result<(), MarketDataError> {
        // 检查是否已经启动
        {
            let task_handle = self.task_handle.read().await;
            if task_handle.is_some() {
                return Ok(());
            }
        }
        
        // 重置停止标志
        self.set_should_stop(false).await;
        
        // 获取输入通道
        let input_rx = {
            let mut rx_lock = self.input_rx.write().await;
            rx_lock.take().ok_or_else(|| MarketDataError::InternalError(
                "输入通道已被消费".to_string()
            ))?
        };
        
        let cleaner = Arc::new(self.clone());
        let should_stop = self.should_stop.clone();
        let output_tx = self.output_tx.clone();
        
        // 启动处理任务
        let handle = tokio::spawn(async move {
            info!("数据清洗器已启动");
            
            while !*should_stop.read().await {
                match input_rx.recv_async().await {
                    Ok(data) => {
                        match cleaner.clean(data).await {
                            Ok(cleaned_data) => {
                                if let Err(e) = output_tx.send_async(cleaned_data).await {
                                    error!("发送清洗后的数据失败: {}", e);
                                }
                            },
                            Err(e) => {
                                warn!("数据清洗失败: {}", e);
                            }
                        }
                    },
                    Err(_) => {
                        info!("输入通道已关闭，停止数据清洗器");
                        break;
                    }
                }
            }
            
            info!("数据清洗器已停止");
        });
        
        // 保存任务句柄
        {
            let mut task_handle = self.task_handle.write().await;
            *task_handle = Some(handle);
        }
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), MarketDataError> {
        info!("停止数据清洗器");
        self.set_should_stop(true).await;
        
        // 等待任务完成
        if let Some(handle) = {
            let mut task_handle = self.task_handle.write().await;
            task_handle.take()
        } {
            if !handle.is_finished() {
                // 等待任务完成，但不阻塞太久
                match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
                    Ok(Ok(())) => {
                        info!("清洗任务已完成");
                    }
                    Ok(Err(e)) => {
                        error!("清洗任务失败: {:?}", e);
                    }
                    Err(_) => {
                        error!("清洗任务超时");
                        // handle已经被移动，不能再使用
                    }
                }
            }
        }
        
        Ok(())
    }
}

// 实现Clone以便在start方法中创建Arc
impl Clone for BaseDataCleaner {
    fn clone(&self) -> Self {
        Self {
            input_rx: self.input_rx.clone(),
            output_tx: self.output_tx.clone(),
            should_stop: self.should_stop.clone(),
            task_handle: self.task_handle.clone(),
        }
    }
}
