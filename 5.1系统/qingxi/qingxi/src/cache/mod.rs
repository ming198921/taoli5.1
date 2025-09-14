#![allow(dead_code)]
//! # 多级缓存系统
//!
//! 提供L1(内存)、L2(SSD)、L3(网络)多级缓存，优化市场数据访问性能

use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug, instrument};

use crate::types::*;
use crate::errors::MarketDataError;
use crate::lockfree::{MarketDataLockFreeBuffer};

/// 缓存层级
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheLevel {
    L1Memory,
    L2Disk,
    L3Network,
}

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    ttl: Duration,
}

impl<T> CacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            data,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }

    fn access(&mut self) -> &T {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        &self.data
    }
}

/// L1内存缓存
pub struct L1MemoryCache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    max_size: usize,
    default_ttl: Duration,
    lockfree_buffer: Option<Arc<MarketDataLockFreeBuffer>>,
}

impl<K, V> L1MemoryCache<K, V> 
where 
    K: std::hash::Hash + Eq + Clone + Send + Sync + std::fmt::Debug + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            default_ttl,
            lockfree_buffer: None,
        }
    }

    pub fn with_lockfree_buffer(mut self, buffer: Arc<MarketDataLockFreeBuffer>) -> Self {
        self.lockfree_buffer = Some(buffer);
        self
    }

    #[instrument(skip(self, value))]
    pub async fn insert(&self, key: K, value: V) -> Result<(), MarketDataError> {
        let mut data = self.data.write().await;
        
        // 检查是否需要清理过期条目
        if data.len() >= self.max_size {
            self.evict_expired(&mut data).await;
            
            // 如果仍然超出限制，使用LRU策略移除
            if data.len() >= self.max_size {
                self.evict_lru(&mut data).await;
            }
        }

        let entry = CacheEntry::new(value, self.default_ttl);
        data.insert(key, entry);
        
        debug!("Inserted entry into L1 cache, size: {}", data.len());
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut data = self.data.write().await;
        
        if let Some(entry) = data.get_mut(key) {
            if entry.is_expired() {
                data.remove(key);
                return None;
            }
            
            let value = entry.access().clone();
            return Some(value);
        }
        
        None
    }

    async fn evict_expired(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        let expired_keys: Vec<K> = data.iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(k, _)| k.clone())
            .collect();
        
        for key in expired_keys {
            data.remove(&key);
        }
        
        if !data.is_empty() {
            debug!("Evicted expired entries from L1 cache");
        }
    }

    async fn evict_lru(&self, data: &mut HashMap<K, CacheEntry<V>>) {
        if let Some((lru_key, _)) = data.iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, v)| (k.clone(), v.last_accessed))
        {
            data.remove(&lru_key);
            debug!("Evicted LRU entry from L1 cache");
        }
    }

    pub async fn stats(&self) -> L1CacheStats {
        let data = self.data.read().await;
        L1CacheStats {
            size: data.len(),
            max_size: self.max_size,
            hit_ratio: 0.0, // 需要额外跟踪命中率
        }
    }

    /// 启动后台清理任务
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let data = self.data.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let mut cache_data = data.write().await;
                let initial_size = cache_data.len();
                
                let expired_keys: Vec<K> = cache_data.iter()
                    .filter(|(_, entry)| entry.is_expired())
                    .map(|(k, _)| k.clone())
                    .collect();
                
                for key in expired_keys {
                    cache_data.remove(&key);
                }
                
                let cleaned_count = initial_size - cache_data.len();
                if cleaned_count > 0 {
                    debug!("Cleaned {} expired entries from L1 cache", cleaned_count);
                }
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct L1CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_ratio: f64,
}

/// L2磁盘缓存
pub struct L2DiskCache {
    cache_dir: std::path::PathBuf,
    max_size_mb: u64,
    default_ttl: Duration,
}

impl L2DiskCache {
    pub fn new(cache_dir: std::path::PathBuf, max_size_mb: u64, default_ttl: Duration) -> Result<Self, MarketDataError> {
        // 创建缓存目录
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| MarketDataError::InternalError(format!("Failed to create cache directory: {}", e)))?;
        
        Ok(Self {
            cache_dir,
            max_size_mb,
            default_ttl,
        })
    }

    #[instrument(skip(self, data))]
    pub async fn store<T>(&self, key: &str, data: &T) -> Result<(), MarketDataError> 
    where 
        T: Serialize,
    {
        // 🚀 清理文件名，将非法字符替换为安全字符
        let safe_key = key
            .replace(':', "_")
            .replace('/', "_")
            .replace('\\', "_")
            .replace('<', "_")
            .replace('>', "_")
            .replace('|', "_")
            .replace('?', "_")
            .replace('*', "_")
            .replace('"', "_");
        
        let file_path = self.cache_dir.join(format!("{}.cache", safe_key));
        
        let serialized = bincode::serialize(data)
            .map_err(|e| MarketDataError::InternalError(format!("Serialization failed: {}", e)))?;
        
        tokio::fs::write(&file_path, serialized).await
            .map_err(|e| MarketDataError::InternalError(format!("Failed to write cache file: {}", e)))?;
        
        debug!("Stored data to L2 cache: {} -> {}", key, safe_key);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn load<T>(&self, key: &str) -> Result<Option<T>, MarketDataError> 
    where 
        T: for<'de> Deserialize<'de>,
    {
        // 🚀 清理文件名，与store方法保持一致
        let safe_key = key
            .replace(':', "_")
            .replace('/', "_")
            .replace('\\', "_")
            .replace('<', "_")
            .replace('>', "_")
            .replace('|', "_")
            .replace('?', "_")
            .replace('*', "_")
            .replace('"', "_");
        
        let file_path = self.cache_dir.join(format!("{}.cache", safe_key));
        
        if !file_path.exists() {
            return Ok(None);
        }

        // 检查文件是否过期
        if let Ok(metadata) = tokio::fs::metadata(&file_path).await {
            if let Ok(created) = metadata.created() {
                if created.elapsed().unwrap_or(Duration::MAX) > self.default_ttl {
                    // 删除过期文件
                    let _ = tokio::fs::remove_file(&file_path).await;
                    return Ok(None);
                }
            }
        }

        let data = tokio::fs::read(&file_path).await
            .map_err(|e| MarketDataError::InternalError(format!("Failed to read cache file: {}", e)))?;
        
        let deserialized = bincode::deserialize(&data)
            .map_err(|e| MarketDataError::InternalError(format!("Deserialization failed: {}", e)))?;
        
        debug!("Loaded data from L2 cache: {}", key);
        Ok(Some(deserialized))
    }

    pub async fn cleanup(&self) -> Result<(), MarketDataError> {
        let mut dir = tokio::fs::read_dir(&self.cache_dir).await
            .map_err(|e| MarketDataError::InternalError(format!("Failed to read cache directory: {}", e)))?;
        
        let mut total_size = 0u64;
        let mut files = Vec::new();
        
        while let Some(entry) = dir.next_entry().await
            .map_err(|e| MarketDataError::InternalError(format!("Failed to read directory entry: {}", e)))? {
            
            if let Ok(metadata) = entry.metadata().await {
                total_size += metadata.len();
                files.push((entry.path(), metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH)));
            }
        }
        
        // 如果超出大小限制，删除最旧的文件
        if total_size > self.max_size_mb * 1024 * 1024 {
            files.sort_by_key(|(_, modified)| *modified);
            
            for (path, _) in files.iter().take(files.len() / 2) {
                let _ = tokio::fs::remove_file(path).await;
            }
            
            info!("Cleaned up L2 cache due to size limit");
        }
        
        Ok(())
    }
}

/// L3网络缓存（Redis等）
pub struct L3NetworkCache {
    // 这里可以集成Redis、Memcached等网络缓存
    // 为了简化，暂时使用本地实现
    _placeholder: (),
}

impl L3NetworkCache {
    pub fn new() -> Self {
        Self {
            _placeholder: (),
        }
    }

    pub async fn get<T>(&self, _key: &str) -> Result<Option<T>, MarketDataError> 
    where 
        T: for<'de> Deserialize<'de>,
    {
        // 网络缓存实现
        Ok(None)
    }

    pub async fn set<T>(&self, _key: &str, _value: &T, _ttl: Duration) -> Result<(), MarketDataError> 
    where 
        T: Serialize,
    {
        // 网络缓存实现
        Ok(())
    }
}

/// 多级缓存管理器
pub struct MultiLevelCache {
    l1_cache: L1MemoryCache<String, MarketDataSnapshot>,
    l2_cache: L2DiskCache,
    l3_cache: L3NetworkCache,
    stats: Arc<Mutex<MultiLevelCacheStats>>,
}

impl MultiLevelCache {
    pub fn new(
        l1_max_size: usize,
        l2_max_size: usize,
        _l3_max_size: usize,
    ) -> Self {
        let l1_cache = L1MemoryCache::new(l1_max_size, Duration::from_secs(3600));
        let cache_dir = std::path::PathBuf::from("/tmp/qingxi_cache");
        let l2_cache = L2DiskCache::new(cache_dir, l2_max_size as u64, Duration::from_secs(7200))
            .unwrap_or_else(|_| {
                // 如果创建失败，使用临时目录
                let temp_dir = std::env::temp_dir().join("qingxi_l2_cache");
                L2DiskCache::new(temp_dir, l2_max_size as u64, Duration::from_secs(7200))
                    .expect("Failed to create L2 cache")
            });
        let l3_cache = L3NetworkCache::new();
        
        Self {
            l1_cache,
            l2_cache,
            l3_cache,
            stats: Arc::new(Mutex::new(MultiLevelCacheStats::default())),
        }
    }

    pub fn new_detailed(
        l1_max_size: usize,
        l1_ttl: Duration,
        l2_cache_dir: std::path::PathBuf,
        l2_max_size_mb: u64,
        l2_ttl: Duration,
    ) -> Result<Self, MarketDataError> {
        let l1_cache = L1MemoryCache::new(l1_max_size, l1_ttl);
        let l2_cache = L2DiskCache::new(l2_cache_dir, l2_max_size_mb, l2_ttl)?;
        let l3_cache = L3NetworkCache::new();
        
        Ok(Self {
            l1_cache,
            l2_cache,
            l3_cache,
            stats: Arc::new(Mutex::new(MultiLevelCacheStats::default())),
        })
    }

    #[instrument(skip(self, value))]
    pub async fn put(&self, key: String, value: OrderBook, level: CacheLevel) -> Result<(), MarketDataError> {
        let mut stats = self.stats.lock().await;
        
        match level {
            CacheLevel::L1Memory => {
                // 为了兼容性，将 OrderBook 转换为 MarketDataSnapshot
                let snapshot = MarketDataSnapshot {
                    orderbook: Some(value),
                    trades: Vec::new(),
                    timestamp: crate::high_precision_time::Nanos::now(),
                    source: "cache".to_string(),
                };
                
                if let Err(e) = self.l1_cache.insert(key.clone(), snapshot.clone()).await {
                    warn!("Failed to store to L1 cache: {}", e);
                    stats.l1_errors += 1;
                } else {
                    stats.l1_writes += 1;
                }
                
                // 同时异步存储到L2
                if let Err(e) = self.l2_cache.store(&key, &snapshot).await {
                    warn!("Failed to store to L2 cache: {}", e);
                    stats.l2_errors += 1;
                } else {
                    stats.l2_writes += 1;
                }
            }
            CacheLevel::L2Disk => {
                let snapshot = MarketDataSnapshot {
                    orderbook: Some(value),
                    trades: Vec::new(),
                    timestamp: crate::high_precision_time::Nanos::now(),
                    source: "cache".to_string(),
                };
                
                if let Err(e) = self.l2_cache.store(&key, &snapshot).await {
                    warn!("Failed to store to L2 cache: {}", e);
                    stats.l2_errors += 1;
                } else {
                    stats.l2_writes += 1;
                }
            }
            CacheLevel::L3Network => {
                let snapshot = MarketDataSnapshot {
                    orderbook: Some(value),
                    trades: Vec::new(),
                    timestamp: crate::high_precision_time::Nanos::now(),
                    source: "cache".to_string(),
                };
                
                if let Err(e) = self.l3_cache.set(&key, &snapshot, Duration::from_secs(3600)).await {
                    warn!("Failed to store to L3 cache: {}", e);
                    stats.l3_errors += 1;
                } else {
                    stats.l3_writes += 1;
                }
            }
        }
        
        Ok(())
    }

    #[instrument(skip(self, value))]
    pub async fn put_snapshot(&self, key: &str, value: MarketDataSnapshot) -> Result<(), MarketDataError> {
        let mut stats = self.stats.lock().await;
        
        // 存储到L1缓存
        if let Err(e) = self.l1_cache.insert(key.to_string(), value.clone()).await {
            warn!("Failed to store to L1 cache: {}", e);
            stats.l1_errors += 1;
        } else {
            stats.l1_writes += 1;
        }
        
        // 异步存储到L2缓存
        if let Err(e) = self.l2_cache.store(key, &value).await {
            warn!("Failed to store to L2 cache: {}", e);
            stats.l2_errors += 1;
        } else {
            stats.l2_writes += 1;
        }
        
        // 异步存储到L3缓存
        if let Err(e) = self.l3_cache.set(key, &value, Duration::from_secs(3600)).await {
            warn!("Failed to store to L3 cache: {}", e);
            stats.l3_errors += 1;
        } else {
            stats.l3_writes += 1;
        }
        
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &str) -> Result<Option<MarketDataSnapshot>, MarketDataError> {
        let mut stats = self.stats.lock().await;
        
        // 先尝试L1缓存
        if let Some(value) = self.l1_cache.get(&key.to_string()).await {
            stats.l1_hits += 1;
            debug!("Cache hit at L1 level for key: {}", key);
            return Ok(Some(value));
        }
        stats.l1_misses += 1;
        
        // 尝试L2缓存
        match self.l2_cache.load::<MarketDataSnapshot>(key).await? {
            Some(value) => {
                stats.l2_hits += 1;
                // 将数据提升到L1缓存
                let _ = self.l1_cache.insert(key.to_string(), value.clone()).await;
                debug!("Cache hit at L2 level for key: {}", key);
                return Ok(Some(value));
            }
            None => {
                stats.l2_misses += 1;
            }
        }
        
        // 尝试L3缓存
        match self.l3_cache.get::<MarketDataSnapshot>(key).await? {
            Some(value) => {
                stats.l3_hits += 1;
                // 将数据提升到L1和L2缓存
                let _ = self.l1_cache.insert(key.to_string(), value.clone()).await;
                let _ = self.l2_cache.store(key, &value).await;
                debug!("Cache hit at L3 level for key: {}", key);
                return Ok(Some(value));
            }
            None => {
                stats.l3_misses += 1;
            }
        }
        
        debug!("Cache miss for key: {}", key);
        Ok(None)
    }

    pub async fn invalidate(&self, key: &str) -> Result<(), MarketDataError> {
        // 从所有缓存层移除
        // L1缓存没有直接的删除方法，需要扩展
        // L2缓存删除文件 - 使用安全的文件名
        let safe_key = key
            .replace(':', "_")
            .replace('/', "_")
            .replace('\\', "_")
            .replace('<', "_")
            .replace('>', "_")
            .replace('|', "_")
            .replace('?', "_")
            .replace('*', "_")
            .replace('"', "_");
        
        let file_path = self.l2_cache.cache_dir.join(format!("{}.cache", safe_key));
        let _ = tokio::fs::remove_file(file_path).await;
        
        // L3缓存删除（需要实现）
        
        Ok(())
    }

    pub async fn stats(&self) -> MultiLevelCacheStats {
        self.stats.lock().await.clone()
    }

    /// 获取简化的缓存统计信息用于性能监控
    pub async fn get_stats(&self) -> CacheStatsSimple {
        let stats = self.stats.lock().await;
        CacheStatsSimple {
            hit_rate: stats.overall_hit_ratio(),
            l1_hit_rate: stats.l1_hit_ratio(),
            l2_hit_rate: stats.l2_hit_ratio(),
            l3_hit_rate: stats.l3_hit_ratio(),
            total_operations: stats.l1_hits + stats.l1_misses + stats.l2_hits + stats.l2_misses + stats.l3_hits + stats.l3_misses,
        }
    }

    /// 启动后台维护任务
    pub fn start_maintenance_tasks(&self) -> Vec<tokio::task::JoinHandle<()>> {
        let mut tasks = Vec::new();
        
        // L1缓存清理任务
        tasks.push(self.l1_cache.start_cleanup_task());
        
        // L2缓存清理任务
        let l2_cache = self.l2_cache.clone();
        tasks.push(tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5分钟
            
            loop {
                interval.tick().await;
                if let Err(e) = l2_cache.cleanup().await {
                    error!("L2 cache cleanup failed: {}", e);
                }
            }
        }));
        
        tasks
    }
}

// 克隆实现
impl Clone for L2DiskCache {
    fn clone(&self) -> Self {
        Self {
            cache_dir: self.cache_dir.clone(),
            max_size_mb: self.max_size_mb,
            default_ttl: self.default_ttl,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MultiLevelCacheStats {
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub l1_writes: u64,
    pub l1_errors: u64,
    pub l2_hits: u64,
    pub l2_misses: u64,
    pub l2_writes: u64,
    pub l2_errors: u64,
    pub l3_hits: u64,
    pub l3_misses: u64,
    pub l3_writes: u64,
    pub l3_errors: u64,
}

impl MultiLevelCacheStats {
    pub fn l1_hit_ratio(&self) -> f64 {
        let total = self.l1_hits + self.l1_misses;
        if total > 0 {
            self.l1_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn l2_hit_ratio(&self) -> f64 {
        let total = self.l2_hits + self.l2_misses;
        if total > 0 {
            self.l2_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn l3_hit_ratio(&self) -> f64 {
        let total = self.l3_hits + self.l3_misses;
        if total > 0 {
            self.l3_hits as f64 / total as f64
        } else {
            0.0
        }
    }

    pub fn overall_hit_ratio(&self) -> f64 {
        let total_hits = self.l1_hits + self.l2_hits + self.l3_hits;
        let total_requests = total_hits + self.l1_misses + self.l2_misses + self.l3_misses;
        if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }
}

/// 简化的缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStatsSimple {
    pub hit_rate: f64,
    pub l1_hit_rate: f64,
    pub l2_hit_rate: f64,
    pub l3_hit_rate: f64,
    pub total_operations: u64,
}
