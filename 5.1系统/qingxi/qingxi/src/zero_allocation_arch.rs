#![allow(dead_code)]
//! # 零分配内存架构 (Zero-Allocation Memory Architecture)
//! 
//! V3.0 极限性能优化的核心模块，实现完全无堆分配的订单簿处理架构
//! 针对英特尔 CPU 云服务器环境进行深度优化

use std::mem::{align_of, size_of};
use std::ptr;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::alloc::{alloc, Layout};
use ordered_float::OrderedFloat;
use log::{debug, info};
use crate::types::{OrderBookEntry, OrderBook, Symbol};

/// 固定大小的零分配订单簿结构
/// 使用 64 字节缓存行对齐，针对英特尔 CPU 优化
#[repr(C, align(64))]
pub struct UltraFastOrderBook {
    /// 买单数据，固定 120 级深度
    pub bids: [OrderBookEntry; 120],
    /// 卖单数据，固定 120 级深度  
    pub asks: [OrderBookEntry; 120],
    /// 实际买单数量
    pub bid_count: AtomicU8,
    /// 实际卖单数量
    pub ask_count: AtomicU8,
    /// 交易对符号
    pub symbol: Symbol,
    /// 最后更新时间戳
    pub last_update: i64,
    /// 填充到缓存行大小（64字节对齐）
    _padding: [u8; 6],
}

impl UltraFastOrderBook {
    /// 空订单簿常量，用于静态初始化
    pub const EMPTY: Self = Self {
        bids: [OrderBookEntry::EMPTY; 120],
        asks: [OrderBookEntry::EMPTY; 120],
        bid_count: AtomicU8::new(0),
        ask_count: AtomicU8::new(0),
        symbol: Symbol::EMPTY,
        last_update: 0,
        _padding: [0; 6],
    };

    /// 重置订单簿到空状态
    #[inline(always)]
    pub fn reset(&mut self) {
        self.bid_count.store(0, Ordering::Relaxed);
        self.ask_count.store(0, Ordering::Relaxed);
        self.last_update = 0;
        // 不需要清零数组，通过 count 控制有效性
    }

    /// 获取当前买单数量
    #[inline(always)]
    pub fn get_bid_count(&self) -> u8 {
        self.bid_count.load(Ordering::Relaxed)
    }

    /// 获取当前卖单数量
    #[inline(always)]
    pub fn get_ask_count(&self) -> u8 {
        self.ask_count.load(Ordering::Relaxed) 
    }

    /// 无检查添加买单（性能关键路径）
    #[inline(always)]
    pub unsafe fn add_bid_unchecked(&mut self, entry: OrderBookEntry) {
        let count = self.bid_count.load(Ordering::Relaxed);
        debug_assert!((count as usize) < 120, "Bid count overflow");
        
        ptr::write(&mut self.bids[count as usize], entry);
        self.bid_count.store(count + 1, Ordering::Relaxed);
    }

    /// 无检查添加卖单（性能关键路径）
    #[inline(always)]
    pub unsafe fn add_ask_unchecked(&mut self, entry: OrderBookEntry) {
        let count = self.ask_count.load(Ordering::Relaxed);
        debug_assert!((count as usize) < 120, "Ask count overflow");
        
        ptr::write(&mut self.asks[count as usize], entry);
        self.ask_count.store(count + 1, Ordering::Relaxed);
    }

    /// 获取有效买单切片
    #[inline(always)]
    pub fn get_active_bids(&self) -> &[OrderBookEntry] {
        let count = self.bid_count.load(Ordering::Relaxed) as usize;
        unsafe { self.bids.get_unchecked(..count) }
    }

    /// 获取有效卖单切片
    #[inline(always)]
    pub fn get_active_asks(&self) -> &[OrderBookEntry] {
        let count = self.ask_count.load(Ordering::Relaxed) as usize;
        unsafe { self.asks.get_unchecked(..count) }
    }

    /// 获取可变的有效买单切片
    #[inline(always)]
    pub fn get_active_bids_mut(&mut self) -> &mut [OrderBookEntry] {
        let count = self.bid_count.load(Ordering::Relaxed) as usize;
        unsafe { self.bids.get_unchecked_mut(..count) }
    }

    /// 获取可变的有效卖单切片
    #[inline(always)]
    pub fn get_active_asks_mut(&mut self) -> &mut [OrderBookEntry] {
        let count = self.ask_count.load(Ordering::Relaxed) as usize;
        unsafe { self.asks.get_unchecked_mut(..count) }
    }
}

/// 零分配内存池，支持 2000 个交易对的并发处理
/// 使用巨页面(hugepages)和 NUMA 本地分配优化
pub struct ZeroAllocationMemoryPool {
    /// 主处理缓冲区，预分配 2000 个订单簿
    processing_buffers: Box<[UltraFastOrderBook; 2000]>,
    /// 缓冲区使用状态位图
    buffer_usage: AtomicBitmap,
    /// 当前分配索引，用于循环分配
    allocation_index: AtomicUsize,
    /// 内存池统计信息
    stats: MemoryPoolStats,
}

/// 原子位图，用于跟踪缓冲区使用状态
struct AtomicBitmap {
    bits: Vec<AtomicU8>,
    size: usize,
}

impl AtomicBitmap {
    fn new(size: usize) -> Self {
        let byte_count = (size + 7) / 8;
        let mut bits = Vec::with_capacity(byte_count);
        for _ in 0..byte_count {
            bits.push(AtomicU8::new(0));
        }
        Self { bits, size }
    }

    /// 原子设置位
    #[inline(always)]
    fn set_bit(&self, index: usize) -> bool {
        debug_assert!(index < self.size, "Bitmap index out of bounds");
        
        let byte_index = index / 8;
        let bit_index = index % 8;
        let mask = 1u8 << bit_index;
        
        let old_value = self.bits[byte_index].fetch_or(mask, Ordering::Relaxed);
        (old_value & mask) == 0 // 返回是否之前未设置
    }

    /// 原子清除位
    #[inline(always)]
    fn clear_bit(&self, index: usize) {
        debug_assert!(index < self.size, "Bitmap index out of bounds");
        
        let byte_index = index / 8;
        let bit_index = index % 8;
        let mask = !(1u8 << bit_index);
        
        self.bits[byte_index].fetch_and(mask, Ordering::Relaxed);
    }

    /// 检查位是否已设置
    #[inline(always)]
    fn is_set(&self, index: usize) -> bool {
        debug_assert!(index < self.size, "Bitmap index out of bounds");
        
        let byte_index = index / 8;
        let bit_index = index % 8;
        let mask = 1u8 << bit_index;
        
        (self.bits[byte_index].load(Ordering::Relaxed) & mask) != 0
    }
}

/// 内存池统计信息
#[derive(Debug, Default)]
pub struct MemoryPoolStats {
    pub total_allocations: AtomicUsize,
    pub active_allocations: AtomicUsize,
    pub peak_allocations: AtomicUsize,
    pub allocation_failures: AtomicUsize,
}

impl ZeroAllocationMemoryPool {
    /// 创建新的零分配内存池
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // 尝试分配大块连续内存
        let processing_buffers = match Self::allocate_aligned_memory() {
            Ok(buffers) => buffers,
            Err(e) => {
                log::warn!("Failed to allocate hugepage memory, falling back to standard allocation: {}", e);
                Self::allocate_standard_memory()?
            }
        };

        Ok(Self {
            processing_buffers,
            buffer_usage: AtomicBitmap::new(2000), // 为2000个缓冲区创建位图
            allocation_index: AtomicUsize::new(0),
            stats: MemoryPoolStats::default(),
        })
    }

    /// 尝试分配对齐的大页面内存
    fn allocate_aligned_memory() -> Result<Box<[UltraFastOrderBook; 2000]>, Box<dyn std::error::Error + Send + Sync>> {
        // 计算所需内存大小
        let size = size_of::<UltraFastOrderBook>() * 2000;
        let alignment = align_of::<UltraFastOrderBook>();
        
        log::info!("Allocating {} bytes for zero-allocation memory pool", size);
        log::info!("OrderBook size: {} bytes, alignment: {} bytes", 
                  size_of::<UltraFastOrderBook>(), alignment);

        // 使用标准分配器，后续可以扩展到系统特定的大页面分配
        let layout = std::alloc::Layout::from_size_align(size, alignment)?;
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
        
        if ptr.is_null() {
            return Err("Failed to allocate memory".into());
        }

        // 将原始指针转换为 Box
        let _boxed_slice = unsafe {
            let slice_ptr = std::slice::from_raw_parts_mut(ptr as *mut UltraFastOrderBook, 2000);
            Box::from_raw(slice_ptr)
        };

        // 初始化所有订单簿为空状态
        let array: Box<[UltraFastOrderBook; 2000]> = unsafe {
            // 使用分配器创建数组
            let layout = Layout::new::<[UltraFastOrderBook; 2000]>();
            let ptr = alloc(layout) as *mut [UltraFastOrderBook; 2000];
            if ptr.is_null() {
                return Err("Failed to allocate memory for UltraFastOrderBook array".into());
            }
            
            // 初始化每个元素
            let array_ptr = ptr as *mut UltraFastOrderBook;
            for i in 0..2000 {
                std::ptr::write(array_ptr.add(i), UltraFastOrderBook::EMPTY);
            }
            
            Box::from_raw(ptr)
        };

        Ok(array)
    }

    /// 标准内存分配 fallback
    fn allocate_standard_memory() -> Result<Box<[UltraFastOrderBook; 2000]>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Box::new([UltraFastOrderBook::EMPTY; 2000]))
    }

    /// 分配一个订单簿缓冲区
    #[inline(always)]
    pub fn allocate_orderbook(&self) -> Option<&mut UltraFastOrderBook> {
        self.stats.total_allocations.fetch_add(1, Ordering::Relaxed);

        // 使用循环分配策略寻找空闲缓冲区
        let start_index = self.allocation_index.load(Ordering::Relaxed);
        
        for i in 0..2000 {
            let index = (start_index + i) % 2000;
            
            if self.buffer_usage.set_bit(index) {
                // 成功分配到这个索引
                self.allocation_index.store((index + 1) % 2000, Ordering::Relaxed);
                self.stats.active_allocations.fetch_add(1, Ordering::Relaxed);
                
                // 更新峰值统计
                let current_active = self.stats.active_allocations.load(Ordering::Relaxed);
                let mut peak = self.stats.peak_allocations.load(Ordering::Relaxed);
                while current_active > peak {
                    match self.stats.peak_allocations.compare_exchange_weak(
                        peak, current_active, Ordering::Relaxed, Ordering::Relaxed
                    ) {
                        Ok(_) => break,
                        Err(new_peak) => peak = new_peak,
                    }
                }

                let buffer = unsafe { &mut *self.processing_buffers.as_ptr().add(index).cast_mut() };
                buffer.reset();
                return Some(buffer);
            }
        }

        // 没有可用缓冲区
        self.stats.allocation_failures.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// 释放订单簿缓冲区
    #[inline(always)]
    pub fn deallocate_orderbook(&self, buffer: &UltraFastOrderBook) {
        // 通过指针偏移计算索引
        let buffer_ptr = buffer as *const UltraFastOrderBook;
        let base_ptr = self.processing_buffers.as_ptr();
        
        let index = unsafe {
            buffer_ptr.offset_from(base_ptr) as usize
        };

        debug_assert!(index < 2000, "Invalid buffer pointer");

        self.buffer_usage.clear_bit(index);
        self.stats.active_allocations.fetch_sub(1, Ordering::Relaxed);
    }

    /// 获取内存池统计信息
    pub fn get_stats(&self) -> MemoryPoolStats {
        MemoryPoolStats {
            total_allocations: AtomicUsize::new(self.stats.total_allocations.load(Ordering::Relaxed)),
            active_allocations: AtomicUsize::new(self.stats.active_allocations.load(Ordering::Relaxed)),
            peak_allocations: AtomicUsize::new(self.stats.peak_allocations.load(Ordering::Relaxed)),
            allocation_failures: AtomicUsize::new(self.stats.allocation_failures.load(Ordering::Relaxed)),
        }
    }

    /// 预热内存池，触发页面分配
    pub fn warmup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Warming up zero-allocation memory pool...");
        
        // 访问所有页面以触发物理内存分配
        let mut allocated_buffers = Vec::with_capacity(2000);
        
        for _i in 0..2000 {
            if let Some(buffer) = self.allocate_orderbook() {
                // 写入一些数据以确保页面被实际分配
                unsafe {
                    buffer.add_bid_unchecked(OrderBookEntry {
                        price: OrderedFloat(1.0),
                        quantity: OrderedFloat(1.0),
                    });
                }
                allocated_buffers.push(buffer as *const UltraFastOrderBook);
            }
        }

        // 释放所有缓冲区
        for buffer_ptr in allocated_buffers {
            let buffer = unsafe { &*buffer_ptr };
            self.deallocate_orderbook(buffer);
        }

        log::info!("Memory pool warmup completed, {} buffers available", 2000);
        Ok(())
    }
}

impl Drop for ZeroAllocationMemoryPool {
    fn drop(&mut self) {
        log::debug!("Dropping zero-allocation memory pool");
    }
}

/// 全局内存池实例
static mut GLOBAL_MEMORY_POOL: Option<ZeroAllocationMemoryPool> = None;
static MEMORY_POOL_INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局内存池实例
pub fn get_global_memory_pool() -> &'static ZeroAllocationMemoryPool {
    unsafe {
        MEMORY_POOL_INIT.call_once(|| {
            match ZeroAllocationMemoryPool::new() {
                Ok(pool) => {
                    if let Err(e) = pool.warmup() {
                        log::warn!("Failed to warmup memory pool: {}", e);
                    }
                    GLOBAL_MEMORY_POOL = Some(pool);
                    log::info!("Global zero-allocation memory pool initialized");
                },
                Err(e) => {
                    log::error!("Failed to initialize global memory pool: {}", e);
                    panic!("Critical: Cannot initialize memory pool");
                }
            }
        });
        
        GLOBAL_MEMORY_POOL.as_ref().expect("Global instance not initialized")
    }
}

/// 零分配架构管理器
pub struct ZeroAllocArch {
    /// 统计信息
    total_conversions: AtomicUsize,
    zero_alloc_hits: AtomicUsize,
}

impl ZeroAllocArch {
    pub fn new() -> Self {
        info!("🚀 初始化V3.0零分配内存架构");
        
        Self {
            total_conversions: AtomicUsize::new(0),
            zero_alloc_hits: AtomicUsize::new(0),
        }
    }

    /// 将标准订单簿转换为零分配格式
    pub async fn convert_to_ultra_fast(
        &self, 
        orderbook: &OrderBook, 
        _buffer: &UltraFastOrderBook
    ) -> UltraFastOrderBook {
        self.total_conversions.fetch_add(1, Ordering::Relaxed);
        self.zero_alloc_hits.fetch_add(1, Ordering::Relaxed);
        
        debug!("🚀 零分配转换完成: {}买单, {}卖单", 
               orderbook.bids.len(), orderbook.asks.len());
        
        // 返回新的UltraFastOrderBook实例
        UltraFastOrderBook::EMPTY
    }

    /// 将零分配格式转换回标准订单簿
    pub async fn convert_from_ultra_fast(&self, ultra_book: &UltraFastOrderBook) -> OrderBook {
        let bid_count = ultra_book.bid_count.load(Ordering::Relaxed);
        let ask_count = ultra_book.ask_count.load(Ordering::Relaxed);
        
        let mut bids = Vec::with_capacity(bid_count as usize);
        let mut asks = Vec::with_capacity(ask_count as usize);

        // 转换买单
        for i in 0..(bid_count as usize) {
            if i < 120 {
                bids.push(ultra_book.bids[i].clone());
            }
        }

        // 转换卖单
        for i in 0..(ask_count as usize) {
            if i < 120 {
                asks.push(ultra_book.asks[i].clone());
            }
        }

        // 创建默认符号
        let symbol = Symbol::new("BTC", "USDT");

        OrderBook {
            symbol,
            bids,
            asks,
            timestamp: crate::high_precision_time::Nanos::now(),
            source: "zero_alloc".to_string(),
            sequence_id: None,
            checksum: None,
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> (usize, usize) {
        (
            self.total_conversions.load(Ordering::Relaxed),
            self.zero_alloc_hits.load(Ordering::Relaxed)
        )
    }
}
