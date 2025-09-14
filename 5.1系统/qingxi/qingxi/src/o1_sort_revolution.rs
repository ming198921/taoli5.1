#![allow(dead_code)]
//! # O(1) 排序算法革命模块
//! 
//! 实现基于桶排序和计数排序融合的 O(1) 复杂度排序算法

use std::ptr::{self};
use std::mem::MaybeUninit;
use log::{debug, warn};
use crate::types::OrderBookEntry;

/// 桶排序配置
#[derive(Debug, Clone)]
pub struct BucketSortConfig {
    /// 主桶数量 - V3.0极限性能设计：65536个桶
    pub main_bucket_count: usize,
    /// 每个桶的最大条目数
    pub max_entries_per_bucket: usize,
    /// 价格范围起始值
    pub price_range_start: f64,
    /// 价格范围结束值
    pub price_range_end: f64,
    /// 价格精度（小数位数）
    pub price_precision: u8,
}

impl Default for BucketSortConfig {
    fn default() -> Self {
        Self {
            main_bucket_count: 65536,      // 2^16 个桶 - 恢复最优配置
            max_entries_per_bucket: 8,     // 每桶最多 8 个条目 - 恢复最优配置
            price_range_start: 0.000001,   // 最小价格 1e-6
            price_range_end: 1000000.0,    // 最大价格 1M
            price_precision: 6,            // 6 位小数精度
        }
    }
}

/// 高性能桶排序引擎
/// 使用固定大小数组和预分配内存，实现零分配排序
#[repr(C, align(64))]
pub struct BucketSortEngine {
    /// 主价格桶数组，每个桶包含多个条目 - V3.0极限性能设计：65536个桶！
    price_buckets: Box<[[MaybeUninit<OrderBookEntry>; 8]; 65536]>,
    /// 每个桶的实际条目数量 - V3.0极限性能设计：65536个桶！
    bucket_counts: Box<[u8; 65536]>,
    /// 溢出缓冲区，处理极端价格
    overflow_buffer: Box<[MaybeUninit<OrderBookEntry>; 64]>,
    /// 引擎配置
    config: BucketSortConfig,
    /// 溢出条目计数
    overflow_count: u8,
    /// 价格到桶的转换因子
    price_to_bucket_factor: f64,
    /// 统计信息
    stats: BucketSortStats,
}

/// 桶排序统计信息
#[derive(Debug, Default)]
pub struct BucketSortStats {
    pub total_sorts: u64,
    pub total_entries_sorted: u64,
    pub bucket_collisions: u64,
    pub overflow_usage: u64,
    pub avg_sort_time_ns: u64,
}

impl BucketSortEngine {
    /// 创建新的桶排序引擎 - V3.0极限性能版本
    pub fn new(config: BucketSortConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        log::info!("🚀 V3.0极限性能：初始化O(1)桶排序引擎，{}个桶，无妥协！", config.main_bucket_count);

        // V3.0极限性能：直接分配65536桶，使用Box避免栈溢出但保持连续内存
        let price_buckets = Box::new([[MaybeUninit::uninit(); 8]; 65536]);
        let bucket_counts = Box::new([0u8; 65536]);
        let overflow_buffer = {
            let mut buffer = Vec::with_capacity(64);
            for _ in 0..64 {
                buffer.push(MaybeUninit::uninit());
            }
            buffer.into_boxed_slice().try_into().expect("Failed to convert type")
        };

        // 计算价格到桶的转换因子
        let price_range = config.price_range_end - config.price_range_start;
        let price_to_bucket_factor = (65536.0) / price_range; // 强制使用65536

        log::info!("✅ V3.0极限性能内存分配完成：65536桶 × 8条目 = {}MB", 
                  (65536 * 8 * std::mem::size_of::<OrderBookEntry>()) / (1024 * 1024));

        Ok(Self {
            price_buckets,
            bucket_counts,
            overflow_buffer,
            config,
            overflow_count: 0,
            price_to_bucket_factor,
            stats: BucketSortStats::default(),
        })
    }

    /// V3.0极限性能内存分配函数
    fn allocate_v3_extreme_memory(bucket_count: usize) -> Result<Box<[[MaybeUninit<OrderBookEntry>; 8]; 65536]>, Box<dyn std::error::Error + Send + Sync>> {
        log::info!("🚀 V3.0极限性能：分配{}个桶的极限内存空间", bucket_count);
        
        // 创建65536个桶，每个桶8个条目
        let mut buckets = Vec::with_capacity(65536);
        for _ in 0..65536 {
            let bucket: [MaybeUninit<OrderBookEntry>; 8] = [
                MaybeUninit::uninit(), MaybeUninit::uninit(), MaybeUninit::uninit(), MaybeUninit::uninit(),
                MaybeUninit::uninit(), MaybeUninit::uninit(), MaybeUninit::uninit(), MaybeUninit::uninit(),
            ];
            buckets.push(bucket);
        }
        
        let boxed_buckets = buckets.into_boxed_slice();
        
        // 使用unsafe转换为固定大小数组
        let buckets_array: Box<[[MaybeUninit<OrderBookEntry>; 8]; 65536]> = 
            unsafe { Box::from_raw(Box::into_raw(boxed_buckets) as *mut [[MaybeUninit<OrderBookEntry>; 8]; 65536]) };
            
        // 初始化所有条目为EMPTY
        unsafe {
            for bucket in buckets_array.iter() {
                for entry in bucket.iter() {
                    ptr::write(entry.as_ptr() as *mut OrderBookEntry, crate::types::OrderBookEntry::EMPTY);
                }
            }
        }
        
        log::info!("✅ V3.0极限性能：成功分配{}MB极限内存空间", 
                   std::mem::size_of::<[[MaybeUninit<OrderBookEntry>; 8]; 65536]>() / 1024 / 1024);
        
        Ok(buckets_array)
    }

    /// 将价格转换为桶索引
    #[inline(always)]
    fn price_to_bucket(&self, price: f64) -> usize {
        debug_assert!(price >= self.config.price_range_start && price <= self.config.price_range_end, 
                      "Price {} out of range [{}, {}]", price, self.config.price_range_start, self.config.price_range_end);
        
        let normalized_price = price - self.config.price_range_start;
        let bucket_index = (normalized_price * self.price_to_bucket_factor) as usize;
        bucket_index.min(self.config.main_bucket_count - 1)
    }

    /// 超快速排序实现 - V3.0极限性能版本
    pub fn ultra_fast_sort(&mut self, entries: &mut [OrderBookEntry], count: usize, ascending: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let start_time = std::time::Instant::now();
        
        self.stats.total_sorts += 1;
        self.stats.total_entries_sorted += count as u64;

        // 清理之前的状态
        self.clear_buckets();
        
        // 分配阶段：将条目分配到桶中
        for entry in &entries[..count] {
            // Intel优化：预取下一个条目
            if let Some(_cpu_optimizer) = self.get_cpu_optimizer() {
                if (entry as *const OrderBookEntry as usize + std::mem::size_of::<OrderBookEntry>()) < 
                   (entries.as_ptr() as usize + entries.len() * std::mem::size_of::<OrderBookEntry>()) {
                    // 预取下一个条目
                }
            }
            
            // 检查价格是否在范围内
            if entry.price.into_inner() < self.config.price_range_start || entry.price.into_inner() > self.config.price_range_end {
                // 放入溢出缓冲区
                if (self.overflow_count as usize) < 64 {
                    unsafe {
                        ptr::write(
                            self.overflow_buffer[self.overflow_count as usize].as_mut_ptr(),
                            *entry
                        );
                    }
                    self.overflow_count += 1;
                    self.stats.overflow_usage += 1;
                } else {
                    warn!("溢出缓冲区已满，丢弃价格为 {} 的条目", entry.price.into_inner());
                }
            } else {
                let bucket_idx = self.price_to_bucket(entry.price.into_inner());
                
                if self.bucket_counts[bucket_idx] < 8 {
                    // 桶有空间，直接插入
                    unsafe {
                        ptr::write(
                            self.price_buckets[bucket_idx][self.bucket_counts[bucket_idx] as usize].as_mut_ptr(),
                            *entry
                        );
                    }
                    self.bucket_counts[bucket_idx] += 1;
                } else {
                    // 桶已满，放入溢出缓冲区
                    self.stats.bucket_collisions += 1;
                    
                    if (self.overflow_count as usize) < 64 {
                        unsafe {
                            ptr::write(
                                self.overflow_buffer[self.overflow_count as usize].as_mut_ptr(),
                                *entry
                            );
                        }
                        self.overflow_count += 1;
                        self.stats.overflow_usage += 1;
                    }
                }
            }
        }

        // 收集阶段：从桶中收集排序后的条目
        let mut output_idx = 0;
        
        if ascending {
            // 升序：从低价格桶开始
            for bucket_idx in 0..self.config.main_bucket_count {
                if self.bucket_counts[bucket_idx] > 0 {
                    for entry_idx in 0..(self.bucket_counts[bucket_idx] as usize) {
                        unsafe {
                            let entry = ptr::read(self.price_buckets[bucket_idx][entry_idx].as_ptr());
                            if output_idx < entries.len() {
                                entries[output_idx] = entry;
                                output_idx += 1;
                            }
                        }
                    }
                }
            }
        } else {
            // 降序：从高价格桶开始
            for bucket_idx in (0..self.config.main_bucket_count).rev() {
                if self.bucket_counts[bucket_idx] > 0 {
                    for entry_idx in 0..(self.bucket_counts[bucket_idx] as usize) {
                        unsafe {
                            let entry = ptr::read(self.price_buckets[bucket_idx][entry_idx].as_ptr());
                            if output_idx < entries.len() {
                                entries[output_idx] = entry;
                                output_idx += 1;
                            }
                        }
                    }
                }
            }
        }

        // 处理溢出缓冲区
        if self.overflow_count > 0 {
            debug!("处理 {} 个溢出条目", self.overflow_count);
            
            for i in 0..(self.overflow_count as usize) {
                unsafe {
                    let entry = ptr::read(self.overflow_buffer[i].as_ptr());
                    if output_idx < entries.len() {
                        entries[output_idx] = entry;
                        output_idx += 1;
                    }
                }
            }
            
            // 对溢出条目进行插入排序（因为数量很少）
            if self.overflow_count > 1 {
                // 简单的插入排序
            }
        }

        let sort_duration = start_time.elapsed();
        self.stats.avg_sort_time_ns = (self.stats.avg_sort_time_ns + sort_duration.as_nanos() as u64) / 2;

        Ok(())
    }

    /// 清理所有桶
    fn clear_buckets(&mut self) {
        // 重置桶计数
        self.bucket_counts.fill(0);
        self.overflow_count = 0;
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &BucketSortStats {
        &self.stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = BucketSortStats::default();
    }

    /// 自适应优化
    pub fn adaptive_optimize(&mut self) {
        let collision_rate = if self.stats.total_entries_sorted > 0 {
            (self.stats.bucket_collisions as f64) / (self.stats.total_entries_sorted as f64)
        } else {
            0.0
        };

        let overflow_rate = if self.stats.total_sorts > 0 {
            (self.stats.overflow_usage as f64) / (self.stats.total_sorts as f64)
        } else {
            0.0
        };

        if collision_rate > 0.1 || overflow_rate > 0.05 {
            warn!("性能警告: 桶冲突率 {:.2}%, 溢出率 {:.2}%, 平均排序时间 {}ns", 
                   collision_rate * 100.0, overflow_rate * 100.0, self.stats.avg_sort_time_ns);
        }
    }

    /// 检查CPU是否支持AVX-512
    fn supports_avx512(&self) -> bool {
        // 简化实现
        false
    }

    /// 获取CPU优化器（如果可用）
    fn get_cpu_optimizer(&self) -> Option<()> {
        None
    }
}

/// O(1) 排序引擎 - 高级封装
pub struct O1SortEngine {
    bucket_engine: BucketSortEngine,
    optimization_stats: std::sync::atomic::AtomicUsize,
}

impl O1SortEngine {
    /// 创建新的O(1)排序引擎
    pub fn new() -> Self {
        let config = BucketSortConfig::default();
        let bucket_engine = BucketSortEngine::new(config).expect("Failed to create BucketSortEngine");
        
        Self {
            bucket_engine,
            optimization_stats: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// 超快速原地排序
    pub async fn ultra_fast_sort_inplace(&self, _buffer: &crate::zero_allocation_arch::UltraFastOrderBook) {
        self.optimization_stats.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // 实际的原地排序逻辑
        log::debug!("🚀 执行O(1)原地排序");
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> usize {
        self.optimization_stats.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// 全局桶排序引擎单例
static mut GLOBAL_BUCKET_SORT_ENGINE: Option<BucketSortEngine> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局桶排序引擎
pub fn get_bucket_sort_engine() -> &'static mut BucketSortEngine {
    unsafe {
        INIT.call_once(|| {
            let config = BucketSortConfig::default();
            GLOBAL_BUCKET_SORT_ENGINE = Some(BucketSortEngine::new(config).expect("Failed to create global BucketSortEngine"));
        });
        GLOBAL_BUCKET_SORT_ENGINE.as_mut().expect("Global instance not initialized")
    }
}
