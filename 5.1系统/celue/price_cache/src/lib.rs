//! BinaryHeap全局最优价格缓存系统
//! 
//! 提供高性能的价格数据缓存，支持实时价格更新、多层级缓存和智能预热策略
//! 使用BinaryHeap实现最优价格快速查找和套利机会发现

pub mod config;
pub mod heap_cache;
pub mod price_index;
pub mod arbitrage_detector;
pub mod cache_manager;
pub mod persistence;
// pub mod metrics; // 已清理 - 模块不存在
pub mod preloader;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{info, warn, error, debug, instrument};

pub use config::PriceCacheConfig;
pub use heap_cache::{PriceHeap, PriceEntry, HeapType};
pub use price_index::{PriceIndex, IndexKey, IndexStats};
pub use arbitrage_detector::{ArbitrageDetector, ArbitrageOpportunity, OpportunityType};
pub use cache_manager::{CacheManager, CacheLayer, CacheStats};
pub use persistence::{PersistenceManager, PersistenceStrategy};
pub use metrics::PriceCacheMetrics;
pub use preloader::{CachePreloader, PreloadStrategy};

/// 价格数据点
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PricePoint {
    /// 交易所名称
    pub exchange: String,
    /// 交易对
    pub symbol: String,
    /// 买入价格
    pub bid: f64,
    /// 卖出价格
    pub ask: f64,
    /// 中间价
    pub mid_price: f64,
    /// 价差
    pub spread: f64,
    /// 成交量
    pub volume: f64,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 数据质量分数
    pub quality_score: f64,
    /// 延迟（毫秒）
    pub latency_ms: u64,
}

impl PricePoint {
    pub fn new(
        exchange: String,
        symbol: String,
        bid: f64,
        ask: f64,
        volume: f64,
    ) -> Self {
        let mid_price = (bid + ask) / 2.0;
        let spread = ask - bid;
        let quality_score = Self::calculate_quality_score(spread, volume);
        
        Self {
            exchange,
            symbol,
            bid,
            ask,
            mid_price,
            spread,
            volume,
            timestamp: Utc::now(),
            quality_score,
            latency_ms: 0,
        }
    }

    fn calculate_quality_score(spread: f64, volume: f64) -> f64 {
        // 基于价差和成交量的质量评分
        let spread_score = if spread > 0.0 { (1.0 / spread).min(100.0) } else { 0.0 };
        let volume_score = (volume / 1000.0).min(100.0);
        (spread_score * 0.3 + volume_score * 0.7) / 100.0
    }

    pub fn is_valid(&self) -> bool {
        self.bid > 0.0 && 
        self.ask > 0.0 && 
        self.ask >= self.bid &&
        self.volume >= 0.0 &&
        self.timestamp > Utc::now() - Duration::minutes(1)
    }

    pub fn age_seconds(&self) -> i64 {
        Utc::now().signed_duration_since(self.timestamp).num_seconds()
    }
}

// 为了在BinaryHeap中使用，实现排序特性
impl PartialOrd for PricePoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PricePoint {
    fn cmp(&self, other: &Self) -> Ordering {
        // 按照质量分数排序，质量高的优先
        self.quality_score.partial_cmp(&other.quality_score).unwrap_or(Ordering::Equal)
    }
}

impl Eq for PricePoint {}

/// 价格更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateEvent {
    pub price_point: PricePoint,
    pub update_type: PriceUpdateType,
    pub cache_hit: bool,
    pub processing_time_us: u64,
}

/// 价格更新类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriceUpdateType {
    New,
    Update,
    Stale,
    Invalid,
}

/// 套利机会事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageEvent {
    pub opportunity: ArbitrageOpportunity,
    pub detected_at: DateTime<Utc>,
    pub expiry_estimate: DateTime<Utc>,
}

/// BinaryHeap全局最优价格缓存系统
pub struct GlobalOptimalPriceCache {
    /// 配置
    config: PriceCacheConfig,
    /// 价格堆缓存
    price_heaps: Arc<RwLock<HashMap<String, PriceHeap>>>,
    /// 价格索引
    price_index: Arc<PriceIndex>,
    /// 套利检测器
    arbitrage_detector: Arc<ArbitrageDetector>,
    /// 缓存管理器
    cache_manager: Arc<CacheManager>,
    /// 持久化管理器
    persistence_manager: Arc<PersistenceManager>,
    /// 指标收集器
    metrics: Arc<PriceCacheMetrics>,
    /// 缓存预加载器
    preloader: Arc<CachePreloader>,
    /// 价格更新事件广播器
    price_update_tx: broadcast::Sender<PriceUpdateEvent>,
    /// 套利机会事件广播器
    arbitrage_tx: broadcast::Sender<ArbitrageEvent>,
    /// 内部事件处理器
    event_tx: mpsc::UnboundedSender<InternalEvent>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 全局最优价格缓存（按交易对分组）
    optimal_prices: Arc<RwLock<HashMap<String, BestPrices>>>,
}

/// 内部事件类型
#[derive(Debug, Clone)]
enum InternalEvent {
    PriceUpdate(PricePoint),
    ArbitrageDetected(ArbitrageOpportunity),
    CacheCleanup,
    CacheWarmup,
    HealthCheck,
    PersistenceSync,
}

/// 最佳价格信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPrices {
    /// 最佳买入价格点
    pub best_bid: PricePoint,
    /// 最佳卖出价格点
    pub best_ask: PricePoint,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 价格源数量
    pub source_count: usize,
}

impl BestPrices {
    pub fn new(first_price: PricePoint) -> Self {
        Self {
            best_bid: first_price.clone(),
            best_ask: first_price,
            last_updated: Utc::now(),
            source_count: 1,
        }
    }

    pub fn update(&mut self, price: PricePoint) {
        // 更新最佳买入价格
        if price.bid > self.best_bid.bid || 
           (price.bid == self.best_bid.bid && price.quality_score > self.best_bid.quality_score) {
            self.best_bid = price.clone();
        }

        // 更新最佳卖出价格
        if price.ask < self.best_ask.ask ||
           (price.ask == self.best_ask.ask && price.quality_score > self.best_ask.quality_score) {
            self.best_ask = price;
        }

        self.last_updated = Utc::now();
    }

    pub fn get_spread(&self) -> f64 {
        self.best_ask.ask - self.best_bid.bid
    }

    pub fn get_mid_price(&self) -> f64 {
        (self.best_bid.bid + self.best_ask.ask) / 2.0
    }
}

impl GlobalOptimalPriceCache {
    /// 创建新的全局最优价格缓存系统
    pub async fn new(config: PriceCacheConfig) -> Result<Self> {
        let price_index = Arc::new(PriceIndex::new(config.index.clone()).await?);
        let arbitrage_detector = Arc::new(ArbitrageDetector::new(config.arbitrage.clone())?);
        let cache_manager = Arc::new(CacheManager::new(config.cache.clone())?);
        let persistence_manager = Arc::new(PersistenceManager::new(config.persistence.clone()).await?);
        let metrics = Arc::new(PriceCacheMetrics::new(config.metrics.clone())?);
        let preloader = Arc::new(CachePreloader::new(config.preload.clone())?);

        let (price_update_tx, _) = broadcast::channel(10000);
        let (arbitrage_tx, _) = broadcast::channel(1000);
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let cache = Self {
            config,
            price_heaps: Arc::new(RwLock::new(HashMap::new())),
            price_index,
            arbitrage_detector,
            cache_manager,
            persistence_manager,
            metrics,
            preloader,
            price_update_tx,
            arbitrage_tx,
            event_tx,
            running: Arc::new(RwLock::new(false)),
            optimal_prices: Arc::new(RwLock::new(HashMap::new())),
        };

        // 启动内部事件处理器
        cache.start_event_processor(event_rx).await;

        info!("Global optimal price cache initialized successfully");
        Ok(cache)
    }

    /// 启动价格缓存系统
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Price cache is already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        // 启动各个组件
        self.price_index.start().await?;
        self.arbitrage_detector.start().await?;
        self.cache_manager.start().await?;
        self.persistence_manager.start().await?;
        self.metrics.start().await?;

        // 预加载缓存
        self.preloader.warmup_cache().await?;

        // 启动后台任务
        self.start_background_tasks().await;

        info!("Global optimal price cache started successfully");
        Ok(())
    }

    /// 停止价格缓存系统
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            warn!("Price cache is not running");
            return Ok(());
        }
        *running = false;

        // 持久化当前数据
        self.sync_to_persistence().await?;

        // 停止各个组件
        self.price_index.stop().await?;
        self.arbitrage_detector.stop().await?;
        self.cache_manager.stop().await?;
        self.persistence_manager.stop().await?;
        self.metrics.stop().await?;

        info!("Global optimal price cache stopped successfully");
        Ok(())
    }

    /// 更新价格数据
    #[instrument(skip(self, price), fields(exchange = %price.exchange, symbol = %price.symbol))]
    pub async fn update_price(&self, price: PricePoint) -> Result<()> {
        let start_time = std::time::Instant::now();

        if !price.is_valid() {
            warn!(
                exchange = %price.exchange,
                symbol = %price.symbol,
                "Invalid price point received"
            );
            return Err(anyhow::anyhow!("Invalid price point"));
        }

        // 发送内部事件进行异步处理
        self.event_tx.send(InternalEvent::PriceUpdate(price))?;

        let processing_time = start_time.elapsed().as_micros() as u64;
        self.metrics.price_update_latency(processing_time).await;

        Ok(())
    }

    /// 获取最优价格
    pub async fn get_best_prices(&self, symbol: &str) -> Option<BestPrices> {
        let optimal_prices = self.optimal_prices.read().await;
        optimal_prices.get(symbol).cloned()
    }

    /// 获取指定交易所的价格
    pub async fn get_exchange_price(&self, exchange: &str, symbol: &str) -> Option<PricePoint> {
        let heap_key = format!("{}:{}", exchange, symbol);
        let heaps = self.price_heaps.read().await;
        
        if let Some(heap) = heaps.get(&heap_key) {
            heap.peek_best().await
        } else {
            None
        }
    }

    /// 查找套利机会
    pub async fn find_arbitrage_opportunities(&self, symbol: &str) -> Result<Vec<ArbitrageOpportunity>> {
        let optimal_prices = self.optimal_prices.read().await;
        
        if let Some(best_prices) = optimal_prices.get(symbol) {
            self.arbitrage_detector.detect_opportunities(symbol, vec![
                best_prices.best_bid.clone(),
                best_prices.best_ask.clone(),
            ]).await
        } else {
            Ok(Vec::new())
        }
    }

    /// 获取价格历史
    pub async fn get_price_history(
        &self,
        exchange: &str,
        symbol: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<PricePoint>> {
        self.persistence_manager.get_price_history(exchange, symbol, from, to).await
    }

    /// 订阅价格更新事件
    pub fn subscribe_price_updates(&self) -> broadcast::Receiver<PriceUpdateEvent> {
        self.price_update_tx.subscribe()
    }

    /// 订阅套利机会事件
    pub fn subscribe_arbitrage_opportunities(&self) -> broadcast::Receiver<ArbitrageEvent> {
        self.arbitrage_tx.subscribe()
    }

    /// 获取缓存统计信息
    pub async fn get_cache_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        // 基本统计
        let heaps = self.price_heaps.read().await;
        let heap_count = heaps.len();
        stats.insert("heap_count".to_string(), serde_json::Value::Number(heap_count.into()));

        let optimal_prices = self.optimal_prices.read().await;
        let symbol_count = optimal_prices.len();
        stats.insert("tracked_symbols".to_string(), serde_json::Value::Number(symbol_count.into()));

        // 内存使用情况
        let mut total_entries = 0;
        for heap in heaps.values() {
            total_entries += heap.len().await;
        }
        stats.insert("total_cached_prices".to_string(), serde_json::Value::Number(total_entries.into()));

        // 获取各组件的统计信息
        if let Ok(index_stats) = self.price_index.get_stats().await {
            for (key, value) in index_stats {
                stats.insert(format!("index_{}", key), value);
            }
        }

        if let Ok(cache_stats) = self.cache_manager.get_stats().await {
            for (key, value) in cache_stats {
                stats.insert(format!("cache_{}", key), value);
            }
        }

        stats
    }

    /// 预热缓存
    pub async fn warmup_cache(&self, symbols: Vec<String>) -> Result<()> {
        self.preloader.warmup_symbols(symbols).await
    }

    /// 清理过期数据
    pub async fn cleanup_expired_data(&self, max_age: Duration) -> Result<usize> {
        let cutoff_time = Utc::now() - max_age;
        let mut cleaned_count = 0;

        // 清理堆缓存中的过期数据
        let mut heaps = self.price_heaps.write().await;
        for heap in heaps.values_mut() {
            cleaned_count += heap.cleanup_expired(cutoff_time).await;
        }

        // 清理全局最优价格中的过期数据
        let mut optimal_prices = self.optimal_prices.write().await;
        optimal_prices.retain(|_, best_prices| {
            best_prices.last_updated > cutoff_time
        });

        info!(cleaned_count = cleaned_count, "Cleaned up expired price data");
        Ok(cleaned_count)
    }

    /// 处理价格更新的内部逻辑
    async fn process_price_update(&self, price: PricePoint) -> Result<()> {
        let symbol = price.symbol.clone();
        let heap_key = format!("{}:{}", price.exchange, price.symbol);
        
        // 更新价格堆
        {
            let mut heaps = self.price_heaps.write().await;
            let heap = heaps.entry(heap_key).or_insert_with(|| {
                PriceHeap::new(price.exchange.clone(), price.symbol.clone(), HeapType::Both)
            });
            heap.push(price.clone()).await?;
        }

        // 更新价格索引
        self.price_index.index_price(&price).await?;

        // 更新全局最优价格
        {
            let mut optimal_prices = self.optimal_prices.write().await;
            match optimal_prices.get_mut(&symbol) {
                Some(best_prices) => {
                    best_prices.update(price.clone());
                }
                None => {
                    optimal_prices.insert(symbol.clone(), BestPrices::new(price.clone()));
                }
            }
        }

        // 检测套利机会
        if let Ok(opportunities) = self.arbitrage_detector.detect_opportunities(&symbol, vec![price.clone()]).await {
            for opportunity in opportunities {
                let arbitrage_event = ArbitrageEvent {
                    opportunity: opportunity.clone(),
                    detected_at: Utc::now(),
                    expiry_estimate: Utc::now() + Duration::seconds(30), // 估计30秒过期
                };
                
                let _ = self.arbitrage_tx.send(arbitrage_event);
                self.event_tx.send(InternalEvent::ArbitrageDetected(opportunity))?;
            }
        }

        // 广播价格更新事件
        let update_event = PriceUpdateEvent {
            price_point: price,
            update_type: PriceUpdateType::Update,
            cache_hit: self.was_cache_hit(&price.exchange, &price.symbol).await,
            processing_time_us: self.calculate_processing_time_us().await,
        };
        let _ = self.price_update_tx.send(update_event);

        // 更新指标
        self.metrics.price_updated(&symbol).await;

        Ok(())
    }

    /// 启动内部事件处理器
    async fn start_event_processor(&self, mut event_rx: mpsc::UnboundedReceiver<InternalEvent>) {
        let cache = Arc::new(self);
        
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                if let Err(e) = cache.process_internal_event(event).await {
                    error!(error = %e, "Failed to process internal event");
                }
            }
        });
    }

    /// 处理内部事件
    async fn process_internal_event(&self, event: InternalEvent) -> Result<()> {
        match event {
            InternalEvent::PriceUpdate(price) => {
                self.process_price_update(price).await?;
            }
            InternalEvent::ArbitrageDetected(opportunity) => {
                info!(
                    symbol = %opportunity.symbol,
                    profit_bps = opportunity.profit_bps,
                    "Arbitrage opportunity detected"
                );
                self.metrics.arbitrage_detected(&opportunity.symbol).await;
            }
            InternalEvent::CacheCleanup => {
                let cleaned = self.cleanup_expired_data(Duration::minutes(5)).await?;
                debug!(cleaned_count = cleaned, "Cache cleanup completed");
            }
            InternalEvent::CacheWarmup => {
                // 获取热门交易对进行预热
                let popular_symbols = self.get_popular_symbols().await;
                self.warmup_cache(popular_symbols).await?;
            }
            InternalEvent::HealthCheck => {
                self.perform_health_check().await?;
            }
            InternalEvent::PersistenceSync => {
                self.sync_to_persistence().await?;
            }
        }

        Ok(())
    }

    /// 启动后台任务
    async fn start_background_tasks(&self) {
        let cache = Arc::new(self);

        // 清理任务
        {
            let cache_clone = Arc::clone(&cache);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5分钟

                loop {
                    interval.tick().await;
                    
                    let running = *cache_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = cache_clone.event_tx.send(InternalEvent::CacheCleanup);
                }
            });
        }

        // 持久化同步任务
        {
            let cache_clone = Arc::clone(&cache);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // 1分钟

                loop {
                    interval.tick().await;
                    
                    let running = *cache_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = cache_clone.event_tx.send(InternalEvent::PersistenceSync);
                }
            });
        }

        // 健康检查任务
        {
            let cache_clone = Arc::clone(&cache);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(600)); // 10分钟

                loop {
                    interval.tick().await;
                    
                    let running = *cache_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = cache_clone.event_tx.send(InternalEvent::HealthCheck);
                }
            });
        }

        info!("Background tasks started");
    }

    /// 获取热门交易对
    async fn get_popular_symbols(&self) -> Vec<String> {
        // 这里应该基于实际的交易量或访问频率
        // 为简化起见，返回一些常见的交易对
        vec![
            "BTC/USDT".to_string(),
            "ETH/USDT".to_string(),
            "BNB/USDT".to_string(),
            "ADA/USDT".to_string(),
            "SOL/USDT".to_string(),
        ]
    }

    /// 执行健康检查
    async fn perform_health_check(&self) -> Result<()> {
        let heaps = self.price_heaps.read().await;
        let mut unhealthy_heaps = 0;

        for (key, heap) in heaps.iter() {
            if heap.len().await == 0 {
                unhealthy_heaps += 1;
                warn!(heap_key = %key, "Empty price heap detected");
            }
        }

        if unhealthy_heaps > 0 {
            warn!(unhealthy_count = unhealthy_heaps, "Health check found issues");
        }

        self.metrics.health_check_completed(unhealthy_heaps == 0).await;
        Ok(())
    }

    /// 同步到持久化存储
    async fn sync_to_persistence(&self) -> Result<()> {
        let optimal_prices = self.optimal_prices.read().await;
        
        for (symbol, best_prices) in optimal_prices.iter() {
            self.persistence_manager.store_best_prices(symbol, best_prices).await?;
        }

        debug!("Synced {} symbols to persistence", optimal_prices.len());
        Ok(())
    }
    
    /// 检查是否为缓存命中
    async fn was_cache_hit(&self, exchange: &str, symbol: &str) -> bool {
        let optimal_prices = self.optimal_prices.read().await;
        let key = format!("{}:{}", exchange, symbol);
        
        // 检查最近是否有相同的价格查询
        if let Some(cached_price) = optimal_prices.get(&key) {
            let now = std::time::Instant::now();
            let elapsed = cached_price.timestamp.elapsed();
            // 如果缓存在5秒内，认为是命中
            elapsed < std::time::Duration::from_secs(5)
        } else {
            false
        }
    }
    
    /// 计算处理时间（微秒）
    async fn calculate_processing_time_us(&self) -> u64 {
        // 模拟处理时间测量
        let start = std::time::Instant::now();
        
        // 简单的处理时间估算（基于缓存大小）
        let optimal_prices = self.optimal_prices.read().await;
        let cache_size = optimal_prices.len();
        drop(optimal_prices);
        
        let elapsed = start.elapsed();
        let base_time_us = elapsed.as_micros() as u64;
        
        // 基于缓存大小调整处理时间
        let processing_time = base_time_us + (cache_size as u64 * 10); // 每个缓存项增加10微秒
        
        processing_time.min(10000) // 最大10毫秒
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_price_point_creation() {
        let price = PricePoint::new(
            "binance".to_string(),
            "BTC/USDT".to_string(),
            45000.0,
            45010.0,
            1000.0,
        );

        assert!(price.is_valid());
        assert_eq!(price.mid_price, 45005.0);
        assert_eq!(price.spread, 10.0);
        assert!(price.quality_score > 0.0);
    }

    #[tokio::test]
    async fn test_best_prices_update() {
        let price1 = PricePoint::new(
            "binance".to_string(),
            "BTC/USDT".to_string(),
            45000.0,
            45010.0,
            1000.0,
        );

        let mut best_prices = BestPrices::new(price1);

        let price2 = PricePoint::new(
            "okx".to_string(),
            "BTC/USDT".to_string(),
            45005.0,  // 更高的买价
            45008.0,  // 更低的卖价
            1500.0,
        );

        best_prices.update(price2);

        assert_eq!(best_prices.best_bid.bid, 45005.0);
        assert_eq!(best_prices.best_ask.ask, 45008.0);
        assert_eq!(best_prices.get_spread(), 3.0);
    }

    #[tokio::test]
    async fn test_cache_lifecycle() {
        let config = PriceCacheConfig::default();
        let cache = GlobalOptimalPriceCache::new(config).await;
        assert!(cache.is_ok());

        let cache = cache.unwrap();
        
        // 测试启动和停止
        let start_result = cache.start().await;
        assert!(start_result.is_ok());

        let stop_result = cache.stop().await;
        assert!(stop_result.is_ok());
    }

    #[tokio::test]
    async fn test_price_update() {
        let config = PriceCacheConfig::default();
        let cache = GlobalOptimalPriceCache::new(config).await.unwrap();
        cache.start().await.unwrap();

        let price = PricePoint::new(
            "test_exchange".to_string(),
            "BTC/USDT".to_string(),
            45000.0,
            45010.0,
            1000.0,
        );

        let result = cache.update_price(price.clone()).await;
        assert!(result.is_ok());

        // 给异步处理一些时间
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let best_prices = cache.get_best_prices("BTC/USDT").await;
        assert!(best_prices.is_some());

        cache.stop().await.unwrap();
    }
}