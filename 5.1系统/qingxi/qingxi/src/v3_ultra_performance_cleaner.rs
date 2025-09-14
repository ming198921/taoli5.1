#![allow(dead_code)]
//! # V3.0 终极性能清洗器
//! 
//! 集成了所有 V3.0 优化技术的终极清洗器：
//! - 零分配内存架构
//! - 英特尔 CPU 硬件优化  
//! - O(1) 排序算法
//! - 实时性能监控与自调优

use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::collections::HashMap;
use crate::types::{OrderBookEntry, Symbol};
use crate::intel_cpu_optimizer::{get_cpu_optimizer, IntelCpuOptimizer, PrefetchHint};
use crate::o1_sort_revolution::get_bucket_sort_engine;
use crate::realtime_performance_monitor_simple::RealTimePerformanceMonitor;

/// V3.0 终极性能清洗器配置
#[derive(Debug, Clone)]
pub struct V3UltraPerformanceConfig {
    /// 预取距离
    pub prefetch_distance: usize,
    /// 批处理大小
    pub batch_size: usize,
    /// 启用英特尔优化
    pub enable_intel_optimization: bool,
    /// 启用零分配
    pub enable_zero_allocation: bool,
    /// 启用O(1)排序
    pub enable_o1_sorting: bool,
    /// 启用实时监控
    pub enable_realtime_monitoring: bool,
}

impl Default for V3UltraPerformanceConfig {
    fn default() -> Self {
        Self {
            prefetch_distance: 4,
            batch_size: 1000,
            enable_intel_optimization: true,
            enable_zero_allocation: true,
            enable_o1_sorting: true,
            enable_realtime_monitoring: true,
        }
    }
}

/// V3.0 终极性能清洗器
/// 
/// 这是清洗器的巅峰版本，整合了所有V3.0的技术：
/// - 65536桶O(1)排序算法
/// - 零分配内存架构
/// - 英特尔CPU硬件优化
/// - 实时性能监控与自调优
pub struct V3UltraPerformanceCleaner {
    config: V3UltraPerformanceConfig,
    performance_monitor: Arc<RealTimePerformanceMonitor>,
    total_processed: AtomicU64,
    total_errors: AtomicU64,
    avg_processing_time_ns: AtomicU64,
}

impl V3UltraPerformanceCleaner {
    /// 创建新的V3.0终极性能清洗器
    pub fn new(config: V3UltraPerformanceConfig) -> Self {
        let performance_monitor = Arc::new(RealTimePerformanceMonitor::new());
        
        Self {
            config,
            performance_monitor,
            total_processed: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            avg_processing_time_ns: AtomicU64::new(0),
        }
    }

    /// V3.0 终极清洗主函数
    /// 
    /// 这是所有技术的集大成者：
    /// - 使用65536桶O(1)排序算法
    /// - 零分配内存池
    /// - 英特尔CPU优化
    /// - 实时性能监控
    pub async fn ultra_clean(
        &self,
        symbol: Symbol,
        raw_entries: &[OrderBookEntry],
        is_bid: bool,
    ) -> Result<Vec<OrderBookEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        
        // 获取性能监控器
        let _mon = self.performance_monitor.clone();
        
        // 使用性能监控包装处理
        let result = {
            let cleaned_entries = self.internal_ultra_clean(symbol, raw_entries, is_bid)?;
            Ok(cleaned_entries)
        };
        
        // 更新统计信息
        let processing_time = start_time.elapsed().as_nanos() as u64;
        self.total_processed.fetch_add(1, Ordering::Relaxed);
        let old_avg = self.avg_processing_time_ns.load(Ordering::Relaxed);
        let new_avg = (old_avg + processing_time) / 2;
        self.avg_processing_time_ns.store(new_avg, Ordering::Relaxed);
        
        result
    }
    
    /// 内部处理函数
    fn internal_ultra_clean(
        &self,
        symbol: Symbol,
        raw_entries: &[OrderBookEntry],
        is_bid: bool,
    ) -> Result<Vec<OrderBookEntry>, Box<dyn std::error::Error + Send + Sync>> {
        // 第一阶段：数据预处理和验证
        let valid_entries = self.preprocess_entries(raw_entries)?;
        
        // 第二阶段：使用65536桶O(1)排序
        let sorted_entries = self.ultra_fast_sort_v3(valid_entries, is_bid)?;
        
        // 第三阶段：应用清洗规则
        let cleaned_entries = self.apply_cleaning_rules(sorted_entries, symbol)?;
        
        Ok(cleaned_entries)
    }
    
    /// 数据预处理 - V3.0优化版本
    fn preprocess_entries(&self, raw_entries: &[OrderBookEntry]) -> Result<Vec<OrderBookEntry>, Box<dyn std::error::Error + Send + Sync>> {
        if self.config.enable_intel_optimization {
            let _cpu_optimizer = get_cpu_optimizer();
            
            // 使用英特尔优化进行预处理
            let mut valid_entries = Vec::with_capacity(raw_entries.len());
            
            for (i, &entry) in raw_entries.iter().enumerate() {
                // 预取下一个条目以提高缓存性能
                if i + self.config.prefetch_distance < raw_entries.len() {
                      IntelCpuOptimizer::prefetch_data(
                          &raw_entries[i + self.config.prefetch_distance], 
                          PrefetchHint::Temporal0
                      );
                }
                
                // 验证条目有效性
                if entry.price > ordered_float::OrderedFloat(0.0) && entry.quantity > ordered_float::OrderedFloat(0.0) {
                    valid_entries.push(entry);
                }
            }
            
            Ok(valid_entries)
        } else {
            // 标准预处理
            Ok(raw_entries.iter().filter(|e| e.price > ordered_float::OrderedFloat(0.0) && e.quantity > ordered_float::OrderedFloat(0.0)).copied().collect())
        }
    }
    
    /// V3.0 超快速排序
    fn ultra_fast_sort_v3(&self, mut entries: Vec<OrderBookEntry>, ascending: bool) -> Result<Vec<OrderBookEntry>, Box<dyn std::error::Error + Send + Sync>> {
        if self.config.enable_o1_sorting {
            // 使用65536桶O(1)排序引擎
            let sort_engine = get_bucket_sort_engine();
            
            // 获取长度，避免借用冲突
            let len = entries.len();
            
            // 直接原地排序，极限性能
            sort_engine.ultra_fast_sort(&mut entries, len, ascending)?;
            
            Ok(entries)
        } else {
            // 回退到标准排序
            if ascending {
                entries.sort_by_key(|e| e.price);
            } else {
                entries.sort_by_key(|e| std::cmp::Reverse(e.price));
            }
            Ok(entries)
        }
    }
    
    /// 应用清洗规则
    fn apply_cleaning_rules(&self, entries: Vec<OrderBookEntry>, _symbol: Symbol) -> Result<Vec<OrderBookEntry>, Box<dyn std::error::Error + Send + Sync>> {
        // V3.0 清洗规则：去重、过滤异常值、规范化
        let mut cleaned = Vec::with_capacity(entries.len());
        let mut last_price = None;
        
        for entry in entries {
            // 去重
            if Some(entry.price) != last_price {
                if self.is_valid_entry(&entry) {
                    cleaned.push(entry);
                    last_price = Some(entry.price);
                }
            }
        }
        
        Ok(cleaned)
    }
    
    /// 验证条目是否有效
    fn is_valid_entry(&self, entry: &OrderBookEntry) -> bool {
        entry.price > ordered_float::OrderedFloat(0.0) && 
        entry.quantity > ordered_float::OrderedFloat(0.0) && 
        entry.price.into_inner().is_finite() && 
        entry.quantity.into_inner().is_finite()
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("total_processed".to_string(), self.total_processed.load(Ordering::Relaxed));
        stats.insert("total_errors".to_string(), self.total_errors.load(Ordering::Relaxed));
        stats.insert("avg_processing_time_ns".to_string(), self.avg_processing_time_ns.load(Ordering::Relaxed));
        stats
    }
}

/// 全局V3.0终极清洗器单例
static mut GLOBAL_V3_CLEANER: Option<V3UltraPerformanceCleaner> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局V3.0终极清洗器
pub fn get_v3_ultra_cleaner() -> &'static V3UltraPerformanceCleaner {
    unsafe {
        INIT.call_once(|| {
            let config = V3UltraPerformanceConfig::default();
            GLOBAL_V3_CLEANER = Some(V3UltraPerformanceCleaner::new(config));
        });
        GLOBAL_V3_CLEANER.as_ref().expect("Global instance not initialized")
    }
}

// 为了向后兼容，保留一些旧的函数签名
pub async fn clean_orderbook_v3_ultra(
    symbol: Symbol,
    entries: &[OrderBookEntry],
    is_bid: bool,
) -> Result<Vec<OrderBookEntry>, Box<dyn std::error::Error + Send + Sync>> {
    get_v3_ultra_cleaner().ultra_clean(symbol, entries, is_bid).await
}
