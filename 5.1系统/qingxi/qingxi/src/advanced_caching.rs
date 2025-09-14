#![allow(dead_code)]
//! # 高级缓存策略模块
//!
//! 实现智能缓存预加载、缓存命中率优化和缓存预热策略
//! 
//! ## 核心特性
//! - 智能缓存预加载
//! - LRU + 频率衰减算法
//! - 缓存预热机制
//! - 分层缓存架构
//! - 实时命中率监控

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use std::hash::Hash;
use tokio::time::interval;
use tracing::info;

use crate::types::*;

/// 缓存项元数据
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    /// 缓存数据
    data: T,
    /// 创建时间
    created_at: Instant,
    /// 最后访问时间
    last_accessed: Instant,
    /// 访问次数
    access_count: u64,
    /// 权重分数（基于访问频率和时间衰减）
    weight_score: f64,
    /// 数据大小（字节）
    size_bytes: usize,
    /// TTL (Time To Live)
    ttl: Option<Duration>,
}

impl<T> CacheEntry<T> {
    fn new(data: T, size_bytes: usize, ttl: Option<Duration>) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            weight_score: 1.0,
            size_bytes,
            ttl,
        }
    }

    fn access(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        self.update_weight_score();
    }

    fn update_weight_score(&mut self) {
        let age_factor = self.last_accessed.duration_since(self.created_at).as_secs_f64();
        let frequency_factor = self.access_count as f64;
        
        // 时间衰减因子（越新权重越高）
        let time_decay = (-age_factor / 3600.0).exp(); // 1小时衰减周期
        
        // 频率权重（访问次数越多权重越高）
        let frequency_weight = (frequency_factor + 1.0).ln();
        
        self.weight_score = time_decay * frequency_weight;
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 最大内存使用量（字节）
    pub max_memory_bytes: usize,
    /// 默认TTL
    pub default_ttl: Option<Duration>,
    /// 清理间隔
    pub cleanup_interval: Duration,
    /// 预热并发数
    pub preload_concurrency: usize,
    /// 命中率统计窗口大小
    pub hit_rate_window_size: usize,
    /// 是否启用预测性预加载
    pub enable_predictive_preload: bool,
    /// 预加载阈值（命中率低于此值时触发）
    pub preload_threshold: f64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            default_ttl: Some(Duration::from_secs(3600)), // 1小时
            cleanup_interval: Duration::from_secs(60),
            preload_concurrency: 4,
            hit_rate_window_size: 1000,
            enable_predictive_preload: true,
            preload_threshold: 0.8,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub preload_count: u64,
    pub current_entries: usize,
    pub current_memory_bytes: usize,
    pub hit_rate: f64,
    pub average_access_time_ns: u64,
    pub cache_efficiency_score: f64,
}

impl CacheStats {
    pub fn update_hit_rate(&mut self) {
        let total = self.hit_count + self.miss_count;
        if total > 0 {
            self.hit_rate = self.hit_count as f64 / total as f64;
        }
    }

    pub fn calculate_efficiency_score(&mut self) {
        // 综合命中率、内存使用率等因素计算缓存效率
        let memory_efficiency = if self.current_memory_bytes > 0 {
            1.0 - (self.current_memory_bytes as f64 / (100 * 1024 * 1024) as f64).min(1.0)
        } else {
            1.0
        };

        self.cache_efficiency_score = (self.hit_rate * 0.7) + (memory_efficiency * 0.3);
    }
}

/// 智能缓存管理器
pub struct IntelligentCacheManager<K, V> 
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// 主缓存存储
    cache: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// 配置
    config: CacheConfig,
    /// 统计信息
    stats: Arc<RwLock<CacheStats>>,
    /// 访问历史（用于预测）
    access_history: Arc<Mutex<VecDeque<(K, Instant)>>>,
    /// 预加载队列
    preload_queue: Arc<Mutex<VecDeque<K>>>,
    /// 预加载函数
    preload_fn: Option<Arc<dyn Fn(&K) -> Option<V> + Send + Sync>>,
    /// 停止标志
    shutdown_flag: Arc<std::sync::atomic::AtomicBool>,
}

impl<K, V> IntelligentCacheManager<K, V>
where
    K: Clone + Hash + Eq + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// 创建新的缓存管理器
    pub fn new(config: CacheConfig) -> Self {
        let manager = Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            access_history: Arc::new(Mutex::new(VecDeque::new())),
            preload_queue: Arc::new(Mutex::new(VecDeque::new())),
            preload_fn: None,
            shutdown_flag: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        };

        manager
    }

    /// 设置预加载函数
    pub fn set_preload_function<F>(&mut self, func: F)
    where
        F: Fn(&K) -> Option<V> + Send + Sync + 'static,
    {
        self.preload_fn = Some(Arc::new(func));
    }

    /// 启动后台任务
    pub async fn start_background_tasks(&self) {
        let cache_clone = self.cache.clone();
        let stats_clone = self.stats.clone();
        let config = self.config.clone();
        let shutdown_flag = self.shutdown_flag.clone();

        // 启动清理任务
        let _cleanup_task = tokio::spawn(async move {
            let mut cleanup_interval = interval(config.cleanup_interval);
            
            while !shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
                cleanup_interval.tick().await;
                Self::cleanup_expired_entries(&cache_clone, &stats_clone).await;
            }
        });

        // 启动预加载任务
        if self.config.enable_predictive_preload && self.preload_fn.is_some() {
            let preload_queue = self.preload_queue.clone();
            let cache_clone = self.cache.clone();
            let stats_clone = self.stats.clone();
            let preload_fn = self.preload_fn.clone()
                .expect("Preload function should be available");
            let shutdown_flag = self.shutdown_flag.clone();

            let _preload_task = tokio::spawn(async move {
                while !shutdown_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    if let Some(key) = {
                        let mut queue = preload_queue.lock()
                            .expect("Failed to acquire preload queue lock");
                        queue.pop_front()
                    } {
                        if let Some(value) = preload_fn(&key) {
                            Self::insert_internal(&cache_clone, &stats_clone, key, value, None).await;
                            
                            let mut stats = stats_clone.write()
                                .expect("Failed to acquire stats write lock");
                            stats.preload_count += 1;
                        }
                    } else {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            });
        }
    }

    /// 获取缓存项（泛型版本，当K=String时使用）
    pub async fn get(&self, key: &str) -> Option<V>
    where
        K: From<String>,
    {
        self.get_with_key(&K::from(key.to_string())).await
    }
    
    /// 使用String键获取缓存项
    pub async fn get_with_key(&self, key: &K) -> Option<V> {
        let start_time = Instant::now();
        
        // 记录访问历史
        {
            let mut history = self.access_history.lock()
                .expect("Failed to acquire access history lock");
            history.push_back((key.clone(), Instant::now()));
            
            // 限制历史记录大小
            if history.len() > self.config.hit_rate_window_size {
                history.pop_front();
            }
        }

        let result = {
            let mut cache = self.cache.write()
                .expect("Failed to acquire cache write lock");
            if let Some(entry) = cache.get_mut(key) {
                if !entry.is_expired() {
                    entry.access();
                    Some(entry.data.clone())
                } else {
                    cache.remove(key);
                    None
                }
            } else {
                None
            }
        };

        // 更新统计
        {
            let mut stats = self.stats.write()
                .expect("Failed to acquire stats write lock");
            if result.is_some() {
                stats.hit_count += 1;
            } else {
                stats.miss_count += 1;
                
                // 如果命中率低于阈值，触发预加载
                if self.config.enable_predictive_preload {
                    stats.update_hit_rate();
                    if stats.hit_rate < self.config.preload_threshold {
                        self.trigger_predictive_preload(key.clone()).await;
                    }
                }
            }
            
            stats.update_hit_rate();
            stats.average_access_time_ns = start_time.elapsed().as_nanos() as u64;
        }

        result
    }

    /// 插入缓存项
    pub async fn insert(&self, key: K, value: V, ttl: Option<Duration>) {
        Self::insert_internal(&self.cache, &self.stats, key, value, ttl).await;
    }

    /// 插入缓存项（别名）
    pub async fn put(&self, key: K, value: V) {
        self.insert(key, value, self.config.default_ttl).await;
    }

    /// 内部插入方法
    async fn insert_internal(
        cache: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        stats: &Arc<RwLock<CacheStats>>,
        key: K,
        value: V,
        ttl: Option<Duration>,
    ) {
        let size_bytes = std::mem::size_of::<V>(); // 简化的大小计算
        let entry = CacheEntry::new(value, size_bytes, ttl);

        {
            let mut cache_map = cache.write()
                .expect("Failed to acquire cache write lock");
            cache_map.insert(key, entry);
        }

        // 检查是否需要清理
        let needs_cleanup = {
            let cache_map = cache.read()
                .expect("Failed to acquire cache read lock");
            let stats_guard = stats.read()
                .expect("Failed to acquire stats read lock");
            cache_map.len() > 10000 || // 配置的最大条目数
            stats_guard.current_memory_bytes > 100 * 1024 * 1024 // 配置的最大内存
        };

        if needs_cleanup {
            Self::evict_least_valuable_entries(cache, stats).await;
        }
    }

    /// 触发预测性预加载
    async fn trigger_predictive_preload(&self, key: K) {
        let predicted_keys = self.predict_next_access_keys(&key).await;
        
        {
            let mut queue = self.preload_queue.lock()
                .expect("Failed to acquire preload queue lock");
            for predicted_key in predicted_keys {
                queue.push_back(predicted_key);
            }
        }
    }

    /// 预测下一个可能访问的键
    async fn predict_next_access_keys(&self, _current_key: &K) -> Vec<K> {
        // 简化的预测算法，基于访问历史
        let history = self.access_history.lock()
            .expect("Failed to acquire access history lock");
        let mut frequency_map = HashMap::new();
        
        // 统计最近访问的键的频率
        for (key, _) in history.iter().rev().take(100) {
            *frequency_map.entry(key.clone()).or_insert(0) += 1;
        }

        // 返回频率最高的几个键
        let mut sorted_keys: Vec<_> = frequency_map.into_iter().collect();
        sorted_keys.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted_keys.into_iter()
            .take(5)
            .map(|(key, _)| key)
            .collect()
    }

    /// 清理过期条目
    async fn cleanup_expired_entries(
        cache: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        stats: &Arc<RwLock<CacheStats>>,
    ) {
        let expired_keys: Vec<K> = {
            let cache_map = cache.read()
                .expect("Failed to acquire cache read lock");
            cache_map.iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(key, _)| key.clone())
                .collect()
        };

        if !expired_keys.is_empty() {
            let mut cache_map = cache.write()
                .expect("Failed to acquire cache write lock");
            let mut stats_guard = stats.write()
                .expect("Failed to acquire stats write lock");
            
            for key in expired_keys {
                if let Some(entry) = cache_map.remove(&key) {
                    stats_guard.current_memory_bytes = 
                        stats_guard.current_memory_bytes.saturating_sub(entry.size_bytes);
                    stats_guard.eviction_count += 1;
                }
            }
            
            stats_guard.current_entries = cache_map.len();
            stats_guard.calculate_efficiency_score();
        }
    }

    /// 淘汰最不重要的条目
    async fn evict_least_valuable_entries(
        cache: &Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
        stats: &Arc<RwLock<CacheStats>>,
    ) {
        let evict_keys: Vec<K> = {
            let mut cache_map = cache.write()
                .expect("Failed to acquire cache write lock");
            
            // 更新所有条目的权重分数
            for entry in cache_map.values_mut() {
                entry.update_weight_score();
            }

            // 收集权重最低的条目
            let mut entries: Vec<_> = cache_map.iter().collect();
            entries.sort_by(|a, b| a.1.weight_score.partial_cmp(&b.1.weight_score)
                .expect("Failed to compare weight scores"));
            
            // 淘汰最低权重的25%条目
            let evict_count = cache_map.len() / 4;
            entries.into_iter()
                .take(evict_count)
                .map(|(key, _)| key.clone())
                .collect()
        };

        if !evict_keys.is_empty() {
            let mut cache_map = cache.write()
                .expect("Failed to acquire cache write lock");
            let mut stats_guard = stats.write()
                .expect("Failed to acquire stats write lock");
            
            for key in evict_keys {
                if let Some(entry) = cache_map.remove(&key) {
                    stats_guard.current_memory_bytes = 
                        stats_guard.current_memory_bytes.saturating_sub(entry.size_bytes);
                    stats_guard.eviction_count += 1;
                }
            }
            
            stats_guard.current_entries = cache_map.len();
            stats_guard.calculate_efficiency_score();
        }
    }

    /// 批量预热缓存
    pub async fn warm_up(&self, keys: Vec<K>) {
        if let Some(preload_fn) = &self.preload_fn {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.preload_concurrency));
            let mut tasks = Vec::new();

        for key in keys {
            let permit = semaphore.clone().acquire_owned().await
                .expect("Failed to acquire semaphore permit");
            let cache = self.cache.clone();
            let stats = self.stats.clone();
            let preload_fn = preload_fn.clone();
            let key_clone = key.clone();                let task = tokio::spawn(async move {
                    let _permit = permit;
                    if let Some(value) = preload_fn(&key_clone) {
                        Self::insert_internal(&cache, &stats, key_clone, value, None).await;
                    }
                });

                tasks.push(task);
            }

            // 等待所有预热任务完成
            for task in tasks {
                let _ = task.await;
            }

            info!("缓存预热完成");
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read()
            .expect("Failed to acquire stats read lock");
        stats.clone()
    }

    /// 关闭缓存管理器
    pub async fn shutdown(&self) {
        self.shutdown_flag.store(true, std::sync::atomic::Ordering::Relaxed);
        info!("缓存管理器已关闭");
    }
}

/// 市场数据缓存管理器（特化版本）
pub struct MarketDataCacheManager {
    orderbook_cache: IntelligentCacheManager<String, OrderBook>,
    trade_cache: IntelligentCacheManager<String, Vec<crate::types::TradeUpdate>>,
    price_cache: IntelligentCacheManager<String, f64>,
}

impl MarketDataCacheManager {
    pub fn new() -> Self {
        let orderbook_config = CacheConfig {
            max_entries: 1000,
            max_memory_bytes: 50 * 1024 * 1024, // 50MB for orderbooks
            default_ttl: Some(Duration::from_secs(60)),
            ..Default::default()
        };

        let trade_config = CacheConfig {
            max_entries: 5000,
            max_memory_bytes: 30 * 1024 * 1024, // 30MB for trades
            default_ttl: Some(Duration::from_secs(300)),
            ..Default::default()
        };

        let price_config = CacheConfig {
            max_entries: 10000,
            max_memory_bytes: 10 * 1024 * 1024, // 10MB for prices
            default_ttl: Some(Duration::from_secs(30)),
            ..Default::default()
        };

        Self {
            orderbook_cache: IntelligentCacheManager::new(orderbook_config),
            trade_cache: IntelligentCacheManager::new(trade_config),
            price_cache: IntelligentCacheManager::new(price_config),
        }
    }

    /// 获取订单簿
    pub async fn get_orderbook(&self, symbol: &str) -> Option<OrderBook> {
        let symbol_string = symbol.to_string();
        self.orderbook_cache.get_with_key(&symbol_string).await
    }

    /// 缓存订单簿
    pub async fn cache_orderbook(&self, symbol: String, orderbook: OrderBook) {
        self.orderbook_cache.insert(symbol, orderbook, None).await;
    }

    /// 获取交易数据
    pub async fn get_trades(&self, symbol: &str) -> Option<Vec<crate::types::TradeUpdate>> {
        let symbol_string = symbol.to_string();
        self.trade_cache.get_with_key(&symbol_string).await
    }

    /// 缓存交易数据
    pub async fn cache_trades(&self, symbol: String, trades: Vec<crate::types::TradeUpdate>) {
        self.trade_cache.insert(symbol, trades, None).await;
    }

    /// 获取价格
    pub async fn get_price(&self, symbol: &str) -> Option<f64> {
        let symbol_string = symbol.to_string();
        self.price_cache.get_with_key(&symbol_string).await
    }

    /// 缓存价格
    pub async fn cache_price(&self, symbol: String, price: f64) {
        self.price_cache.insert(symbol, price, None).await;
    }

    /// 启动所有后台任务
    pub async fn start_all_background_tasks(&self) {
        self.orderbook_cache.start_background_tasks().await;
        self.trade_cache.start_background_tasks().await;
        self.price_cache.start_background_tasks().await;
    }

    /// 获取综合统计
    pub fn get_comprehensive_stats(&self) -> MarketDataCacheStats {
        let orderbook_stats = self.orderbook_cache.get_stats();
        let trade_stats = self.trade_cache.get_stats();
        let price_stats = self.price_cache.get_stats();

        MarketDataCacheStats {
            orderbook_stats: orderbook_stats.clone(),
            trade_stats: trade_stats.clone(),
            price_stats: price_stats.clone(),
            overall_hit_rate: (orderbook_stats.hit_rate + trade_stats.hit_rate + price_stats.hit_rate) / 3.0,
            total_memory_usage: orderbook_stats.current_memory_bytes + 
                              trade_stats.current_memory_bytes + 
                              price_stats.current_memory_bytes,
        }
    }
}

/// 市场数据缓存统计
#[derive(Debug, Clone)]
pub struct MarketDataCacheStats {
    pub orderbook_stats: CacheStats,
    pub trade_stats: CacheStats,
    pub price_stats: CacheStats,
    pub overall_hit_rate: f64,
    pub total_memory_usage: usize,
}
