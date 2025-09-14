#![allow(dead_code)]
// src/memory/advanced_allocator.rs
// Qingxi V3.0 高级内存优化模块

#[allow(dead_code)]
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::RwLock;

// 从配置获取内存分配器参数的函数
fn get_buffer_sizes() -> (usize, usize, usize) {
    if let Ok(settings) = crate::settings::Settings::load() {
        (
            settings.memory_allocator.zero_allocation_buffer_size,
            settings.memory_allocator.large_buffer_size,
            settings.memory_allocator.huge_buffer_size,
        )
    } else {
        (131072, 262144, 1048576) // 默认值
    }
}

#[allow(dead_code)]
fn get_small_chunks_capacity() -> usize {
    if let Ok(settings) = crate::settings::Settings::load() {
        settings.memory_allocator.small_chunks_capacity
    } else {
        1024
    }
}

// 2. Per-thread内存池结构
#[repr(align(64))] // Cache line alignment
pub struct ThreadLocalPool {
    small_chunks: Vec<*mut u8>,      // 64B-1KB chunks
    medium_chunks: Vec<*mut u8>,     // 1KB-64KB chunks
    large_chunks: Vec<*mut u8>,      // 64KB-1MB chunks
    allocated_bytes: AtomicUsize,
    peak_allocated: AtomicUsize,
    allocation_count: AtomicUsize,
    thread_id: usize,
}

#[allow(dead_code)]
impl ThreadLocalPool {
    fn new(thread_id: usize) -> Self {
        let small_capacity = get_small_chunks_capacity();
        let mut pool = Self {
            small_chunks: Vec::with_capacity(small_capacity),
            medium_chunks: Vec::with_capacity(256),
            large_chunks: Vec::with_capacity(64),
            allocated_bytes: AtomicUsize::new(0),
            peak_allocated: AtomicUsize::new(0),
            allocation_count: AtomicUsize::new(0),
            thread_id,
        };
        
        // 3. 内存预热机制
        pool.preheat_memory();
        pool
    }
    
    // 内存预热：预分配常用大小的内存块
    fn preheat_memory(&mut self) {
        println!("🔥 线程 {} 内存预热开始...", self.thread_id);
        
        // 获取配置化的块大小
        let chunk_sizes = if let Ok(settings) = crate::settings::Settings::load() {
            settings.memory_allocator.chunk_sizes
        } else {
            vec![64, 128, 256, 512, 1024]
        };
        
        // 预热小块内存 
        for size in chunk_sizes {
            for _ in 0..32 {
                let layout = Layout::from_size_align(size, 64).expect("Invalid layout parameters");
                unsafe {
                    let ptr = System.alloc_zeroed(layout);
                    if !ptr.is_null() {
                        self.small_chunks.push(ptr);
                    }
                }
            }
        }
        
        // 预热中等内存 (2KB, 4KB, 8KB, 16KB, 32KB, 64KB)
        for size in [2048, 4096, 8192, 16384, 32768, 65536] {
            for _ in 0..16 {
                let layout = Layout::from_size_align(size, 64).expect("Invalid layout parameters");
                unsafe {
                    let ptr = System.alloc_zeroed(layout);
                    if !ptr.is_null() {
                        self.medium_chunks.push(ptr);
                    }
                }
            }
        }
        
        // 预热大块内存，使用配置化的缓冲区大小
        let buffer_sizes = get_buffer_sizes();
        let large_buffer_sizes = if let Ok(settings) = crate::settings::Settings::load() {
            settings.memory_allocator.large_buffer_sizes
        } else {
            vec![buffer_sizes.0, buffer_sizes.1, 512*1024, buffer_sizes.2]
        };
        
        for size in large_buffer_sizes {
            for _ in 0..8 {
                let layout = Layout::from_size_align(size, 64).expect("Invalid layout parameters");
                unsafe {
                    let ptr = System.alloc_zeroed(layout);
                    if !ptr.is_null() {
                        self.large_chunks.push(ptr);
                    }
                }
            }
        }
        
        let total_preheated = self.small_chunks.len() + self.medium_chunks.len() + self.large_chunks.len();
        println!("✅ 线程 {} 预热完成，预分配 {} 个内存块", self.thread_id, total_preheated);
    }
    
    // 4. 优化的内存分配器
    fn allocate_optimized(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        // 获取大小阈值
        let size_threshold = if let Ok(settings) = crate::settings::Settings::load() {
            settings.memory_allocator.size_threshold
        } else {
            1024
        };
        
        // 选择合适的内存池
        let pool = if size <= size_threshold {
            &mut self.small_chunks
        } else if size <= 65536 {
            &mut self.medium_chunks
        } else {
            &mut self.large_chunks
        };
        
        // 尝试从预分配池获取
        if let Some(ptr) = pool.pop() {
            self.allocated_bytes.fetch_add(size, Ordering::Relaxed);
            
            // 更新峰值内存使用
            let current = self.allocated_bytes.load(Ordering::Relaxed);
            let mut peak = self.peak_allocated.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_allocated.compare_exchange_weak(
                    peak, current, Ordering::Relaxed, Ordering::Relaxed
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
            
            return Some(ptr);
        }
        
        // 如果预分配池为空，直接分配
        unsafe {
            let layout = Layout::from_size_align(size, align.max(64)).ok()?;
            let ptr = System.alloc_zeroed(layout);
            if !ptr.is_null() {
                self.allocated_bytes.fetch_add(size, Ordering::Relaxed);
                Some(ptr)
            } else {
                None
            }
        }
    }
    
    fn deallocate_optimized(&mut self, ptr: *mut u8, size: usize) {
        self.allocated_bytes.fetch_sub(size, Ordering::Relaxed);
        
        // 获取大小阈值
        let size_threshold = if let Ok(settings) = crate::settings::Settings::load() {
            settings.memory_allocator.size_threshold
        } else {
            1024
        };
        
        // 根据大小返回到对应的池
        let pool = if size <= size_threshold {
            &mut self.small_chunks
        } else if size <= 65536 {
            &mut self.medium_chunks
        } else {
            &mut self.large_chunks
        };
        
        // 如果池未满，回收内存块
        if pool.len() < pool.capacity() {
            pool.push(ptr);
        } else {
            // 池已满，直接释放
            unsafe {
                let layout = Layout::from_size_align(size, 64).expect("Invalid layout parameters");
                System.dealloc(ptr, layout);
            }
        }
    }
    
    // 获取内存统计信息
    pub fn get_stats(&self) -> MemoryStats {
        MemoryStats {
            thread_id: self.thread_id,
            allocated_bytes: self.allocated_bytes.load(Ordering::Relaxed),
            peak_allocated: self.peak_allocated.load(Ordering::Relaxed),
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            small_pool_size: self.small_chunks.len(),
            medium_pool_size: self.medium_chunks.len(),
            large_pool_size: self.large_chunks.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub thread_id: usize,
    pub allocated_bytes: usize,
    pub peak_allocated: usize,
    pub allocation_count: usize,
    pub small_pool_size: usize,
    pub medium_pool_size: usize,
    pub large_pool_size: usize,
}

// 线程安全的简化内存管理器
pub struct QingxiMemoryManager {
    global_stats: Arc<RwLock<GlobalMemoryStats>>,
    buffer_size: usize,
}

unsafe impl Sync for QingxiMemoryManager {}
unsafe impl Send for QingxiMemoryManager {}

#[derive(Debug, Default, Clone)]
pub struct GlobalMemoryStats {
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub peak_memory: usize,
    pub allocation_failures: usize,
    pub zero_allocation_hits: usize,
    pub zero_allocation_misses: usize,
}

impl QingxiMemoryManager {
    pub fn new() -> Self {
        println!("🚀 初始化Qingxi V3.0高级内存管理器");
        let buffer_size = get_buffer_sizes().0; // 获取零分配缓冲区大小
        Self {
            global_stats: Arc::new(RwLock::new(GlobalMemoryStats::default())),
            buffer_size,
        }
    }
    
    // 高级分配函数
    pub fn allocate_advanced(&self, size: usize, _align: usize) -> Option<*mut u8> {
        unsafe {
            let layout = Layout::from_size_align(size, 64).ok()?;
            let ptr = System.alloc_zeroed(layout);
            if !ptr.is_null() {
                // 更新全局统计
                if let Ok(mut stats) = self.global_stats.write() {
                    stats.total_allocated += size;
                    stats.zero_allocation_hits += 1;
                }
                Some(ptr)
            } else {
                // 分配失败
                if let Ok(mut stats) = self.global_stats.write() {
                    stats.allocation_failures += 1;
                    stats.zero_allocation_misses += 1;
                }
                None
            }
        }
    }
    
    // 高级释放函数
    pub fn deallocate_advanced(&self, ptr: *mut u8, size: usize) {
        unsafe {
            let layout = Layout::from_size_align(size, 64).expect("Invalid layout parameters");
            System.dealloc(ptr, layout);
            
            // 更新全局统计
            if let Ok(mut stats) = self.global_stats.write() {
                stats.total_deallocated += size;
            }
        }
    }
    
    // 获取全局内存统计
    pub fn get_global_stats(&self) -> GlobalMemoryStats {
        self.global_stats.read().expect("Failed to acquire read lock").clone()
    }
    
    // 获取所有线程池统计（简化版）
    pub fn get_all_thread_stats(&self) -> Vec<MemoryStats> {
        vec![MemoryStats {
            thread_id: 0,
            allocated_bytes: 0,
            peak_allocated: 0,
            allocation_count: 0,
            small_pool_size: 0,
            medium_pool_size: 0,
            large_pool_size: 0,
        }]
    }
    
    // 内存健康检查
    pub fn health_check(&self) -> MemoryHealthReport {
        let global_stats = self.get_global_stats();
        
        let failure_rate = if global_stats.zero_allocation_hits + global_stats.zero_allocation_misses > 0 {
            (global_stats.allocation_failures as f64) / 
            ((global_stats.zero_allocation_hits + global_stats.zero_allocation_misses) as f64) * 100.0
        } else {
            0.0
        };
        
        MemoryHealthReport {
            is_healthy: failure_rate < 0.01, // 目标：失败率 < 0.01%
            failure_rate,
            total_allocated_mb: global_stats.total_allocated as f64 / 1024.0 / 1024.0,
            peak_allocated_mb: global_stats.peak_memory as f64 / 1024.0 / 1024.0,
            active_threads: 1,
            recommendation: if failure_rate > 0.05 {
                "建议增加预分配内存池大小".to_string()
            } else if failure_rate > 0.01 {
                "内存使用接近阈值，建议监控".to_string()
            } else {
                "内存管理状态良好".to_string()
            }
        }
    }
}

#[derive(Debug)]
pub struct MemoryHealthReport {
    pub is_healthy: bool,
    pub failure_rate: f64,
    pub total_allocated_mb: f64,
    pub peak_allocated_mb: f64,
    pub active_threads: usize,
    pub recommendation: String,
}

// 静态全局内存管理器实例
lazy_static::lazy_static! {
    pub static ref QINGXI_MEMORY: QingxiMemoryManager = QingxiMemoryManager::new();
}

// 5. 数据结构对齐优化
#[repr(align(64))] // Cache line对齐
#[derive(Debug, Clone, Copy)]
pub struct AlignedMarketData {
    pub timestamp: u64,
    pub price: f64,
    pub volume: f64,
    pub exchange_id: u32,
    pub symbol_id: u32,
    // 填充到64字节边界
    pub _padding: [u8; 24],
}

#[repr(align(64))]
pub struct AlignedOrderBook {
    pub bids: Vec<AlignedMarketData>,
    pub asks: Vec<AlignedMarketData>,
    pub last_update: u64,
    // 预分配空间避免频繁分配
    _reserved_bids: Vec<AlignedMarketData>,
    _reserved_asks: Vec<AlignedMarketData>,
}

impl AlignedOrderBook {
    pub fn new_optimized() -> Self {
        let (bid_capacity, ask_capacity, reserved_bid_capacity, reserved_ask_capacity) = 
            if let Ok(settings) = crate::settings::Settings::load() {
                (
                    settings.cleaner.orderbook_bid_capacity,
                    settings.cleaner.orderbook_ask_capacity,
                    settings.cleaner.reserved_bid_capacity,
                    settings.cleaner.reserved_ask_capacity
                )
            } else {
                (1000, 1000, 1000, 1000)
            };
            
        Self {
            bids: Vec::with_capacity(bid_capacity),
            asks: Vec::with_capacity(ask_capacity),
            last_update: 0,
            _reserved_bids: Vec::with_capacity(reserved_bid_capacity),
            _reserved_asks: Vec::with_capacity(reserved_ask_capacity),
        }
    }
    
    // 零分配更新函数
    pub fn update_zero_alloc(&mut self, bid_data: &[AlignedMarketData], ask_data: &[AlignedMarketData]) -> Result<(), &'static str> {
        // 检查容量是否足够
        if self.bids.capacity() < bid_data.len() || self.asks.capacity() < ask_data.len() {
            return Err("容量不足，需要重新分配");
        }
        
        // 零分配更新
        self.bids.clear();
        self.asks.clear();
        
        for data in bid_data {
            self.bids.push(*data);
        }
        
        for data in ask_data {
            self.asks.push(*data);
        }
        
        self.last_update = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Operation failed")
            .as_nanos() as u64;
        
        Ok(())
    }
}

// 性能测试函数
pub fn benchmark_memory_performance() {
    println!("🧪 开始内存性能基准测试...");
    
    let start = std::time::Instant::now();
    let (iterations, test_sizes) = if let Ok(settings) = crate::settings::Settings::load() {
        (settings.memory_allocator.test_iterations, settings.memory_allocator.test_allocation_sizes)
    } else {
        (1000000, vec![128, 1024, 4096])
    };
    
    // 测试新的内存分配器
    for i in 0..iterations {
        let size_index = i % test_sizes.len();
        let size = test_sizes[size_index];
        
        if let Some(ptr) = QINGXI_MEMORY.allocate_advanced(size, 64) {
            QINGXI_MEMORY.deallocate_advanced(ptr, size);
        }
    }
    
    let duration = start.elapsed();
    println!("✅ 完成 {} 次内存操作，耗时: {:?}", iterations, duration);
    println!("   平均每次操作: {:.2} ns", duration.as_nanos() as f64 / iterations as f64);
    
    // 打印健康报告
    let health = QINGXI_MEMORY.health_check();
    println!("📊 内存健康报告: {:#?}", health);
}
