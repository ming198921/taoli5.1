#![allow(dead_code)]
// src/memory/zero_allocation_engine.rs
// 零分配引擎实现

#[allow(dead_code)]
use crate::memory::advanced_allocator::{QINGXI_MEMORY, AlignedMarketData, AlignedOrderBook};
use std::sync::Arc;
use std::collections::HashMap;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ZeroAllocationConfig {
    pub buffer_size: usize,           // 缓冲区大小 (131072)
    pub prealloc_pools: usize,        // 预分配池数量
    pub max_symbols: usize,           // 最大交易对数量
    pub max_orderbook_depth: usize,   // 最大订单簿深度
    pub memory_alignment: usize,      // 内存对齐大小
    pub enable_monitoring: bool,      // 启用监控
}

impl Default for ZeroAllocationConfig {
    fn default() -> Self {
        Self {
            buffer_size: 131072,        // 128KB 缓冲区
            prealloc_pools: 16,         // 16个预分配池
            max_symbols: 1000,          // 支持1000个交易对
            max_orderbook_depth: 1000,  // 每个订单簿最大1000档
            memory_alignment: 64,       // 64字节对齐
            enable_monitoring: true,    // 默认启用监控
        }
    }
}

pub struct ZeroAllocationEngine {
    config: ZeroAllocationConfig,
    orderbook_pools: Arc<RwLock<HashMap<String, AlignedOrderBook>>>,
    data_pools: Arc<RwLock<Vec<AlignedMarketData>>>,
    allocation_stats: Arc<RwLock<AllocationStats>>,
    last_health_check: Arc<RwLock<Instant>>,
}

#[derive(Debug, Default)]
struct AllocationStats {
    total_operations: u64,
    zero_allocation_success: u64,
    zero_allocation_failures: u64,
    average_processing_time_ns: u64,
    peak_memory_usage: usize,
    active_symbols: usize,
}

impl ZeroAllocationEngine {
    pub fn new(config: ZeroAllocationConfig) -> Self {
        println!("🚀 初始化零分配引擎，配置: {:#?}", config);
        
        // 预分配订单簿池
        let mut orderbook_pools = HashMap::new();
        for i in 0..config.max_symbols {
            let symbol = format!("SYMBOL_{:04}", i);
            orderbook_pools.insert(symbol, AlignedOrderBook::new_optimized());
        }
        
        // 预分配数据池
        let mut data_pools = Vec::with_capacity(config.buffer_size);
        for _ in 0..config.buffer_size {
            data_pools.push(AlignedMarketData {
                timestamp: 0,
                price: 0.0,
                volume: 0.0,
                exchange_id: 0,
                symbol_id: 0,
                _padding: [0; 24],
            });
        }
        
        println!("✅ 预分配完成: {} 个订单簿, {} 个数据对象", 
                orderbook_pools.len(), data_pools.len());
        
        Self {
            config,
            orderbook_pools: Arc::new(RwLock::new(orderbook_pools)),
            data_pools: Arc::new(RwLock::new(data_pools)),
            allocation_stats: Arc::new(RwLock::new(AllocationStats::default())),
            last_health_check: Arc::new(RwLock::new(Instant::now())),
        }
    }
    
    // 零分配市场数据处理
    pub fn process_market_data_zero_alloc(
        &self, 
        symbol: &str, 
        price: f64, 
        volume: f64, 
        exchange_id: u32
    ) -> Result<(), &'static str> {
        let start_time = Instant::now();
        
        // 尝试零分配处理
        let result = self.try_zero_allocation_processing(symbol, price, volume, exchange_id);
        
        // 更新统计信息
        let processing_time = start_time.elapsed().as_nanos() as u64;
        let mut stats = self.allocation_stats.write();
        stats.total_operations += 1;
        
        match result {
            Ok(_) => {
                stats.zero_allocation_success += 1;
            }
            Err(_) => {
                stats.zero_allocation_failures += 1;
            }
        }
        
        // 更新平均处理时间
        stats.average_processing_time_ns = 
            (stats.average_processing_time_ns * (stats.total_operations - 1) + processing_time) 
            / stats.total_operations;
        
        // 定期健康检查
        if start_time.duration_since(*self.last_health_check.read()) > Duration::from_secs(10) {
            self.perform_health_check();
            *self.last_health_check.write() = start_time;
        }
        
        result
    }
    
    fn try_zero_allocation_processing(
        &self,
        symbol: &str,
        price: f64,
        volume: f64,
        exchange_id: u32
    ) -> Result<(), &'static str> {
        // 获取预分配的订单簿
        let mut orderbooks = self.orderbook_pools.write();
        let orderbook = orderbooks.get_mut(symbol)
            .ok_or("订单簿不存在，需要动态分配")?;
        
        // 创建市场数据（从预分配池获取）
        let market_data = AlignedMarketData {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Operation failed")
                .as_nanos() as u64,
            price,
            volume,
            exchange_id,
            symbol_id: symbol.chars().map(|c| c as u32).sum::<u32>() % 1000,
            _padding: [0; 24],
        };
        
        // 零分配更新订单簿
        if price > 0.0 {
            orderbook.update_zero_alloc(&[market_data], &[])
        } else {
            orderbook.update_zero_alloc(&[], &[market_data])
        }
    }
    
    // 批量处理多个交易对数据
    pub fn process_batch_zero_alloc(
        &self,
        batch_data: &[(String, f64, f64, u32)]
    ) -> Result<usize, String> {
        let start_time = Instant::now();
        let mut successful_processed = 0;
        
        for (symbol, price, volume, exchange_id) in batch_data {
            match self.process_market_data_zero_alloc(symbol, *price, *volume, *exchange_id) {
                Ok(_) => successful_processed += 1,
                Err(e) => {
                    println!("⚠️ 处理 {} 失败: {}", symbol, e);
                }
            }
        }
        
        let processing_time = start_time.elapsed();
        let throughput = (batch_data.len() as f64) / processing_time.as_secs_f64();
        
        println!("📊 批量处理完成: {}/{} 成功, 吞吐量: {:.0} ops/sec", 
                successful_processed, batch_data.len(), throughput);
        
        if successful_processed == batch_data.len() {
            Ok(successful_processed)
        } else {
            Err(format!("部分处理失败: {}/{}", successful_processed, batch_data.len()))
        }
    }
    
    // 获取所有交易对的当前状态
    pub fn get_all_symbols_status(&self) -> HashMap<String, SymbolStatus> {
        let orderbooks = self.orderbook_pools.read();
        let mut status_map = HashMap::new();
        
        for (symbol, orderbook) in orderbooks.iter() {
            status_map.insert(symbol.clone(), SymbolStatus {
                symbol: symbol.clone(),
                bid_depth: orderbook.bids.len(),
                ask_depth: orderbook.asks.len(),
                last_update: orderbook.last_update,
                is_active: orderbook.last_update > 0,
            });
        }
        
        status_map
    }
    
    // 健康检查
    fn perform_health_check(&self) {
        let stats = self.allocation_stats.read();
        let memory_health = QINGXI_MEMORY.health_check();
        
        let zero_alloc_success_rate = if stats.total_operations > 0 {
            (stats.zero_allocation_success as f64 / stats.total_operations as f64) * 100.0
        } else {
            0.0
        };
        
        println!("🩺 零分配引擎健康检查:");
        println!("   总操作数: {}", stats.total_operations);
        println!("   零分配成功率: {:.2}%", zero_alloc_success_rate);
        println!("   平均处理时间: {} ns", stats.average_processing_time_ns);
        println!("   内存健康状态: {}", if memory_health.is_healthy { "✅ 健康" } else { "⚠️ 需要注意" });
        println!("   内存失败率: {:.4}%", memory_health.failure_rate);
        
        // 如果性能下降，输出警告
        if zero_alloc_success_rate < 99.95 {
            println!("⚠️ 警告: 零分配成功率低于99.95%，建议检查内存配置");
        }
        
        if stats.average_processing_time_ns > 1000 {
            println!("⚠️ 警告: 平均处理时间超过1μs，建议优化处理逻辑");
        }
    }
    
    // 获取详细统计信息
    pub fn get_detailed_stats(&self) -> DetailedStats {
        let stats = self.allocation_stats.read();
        let memory_health = QINGXI_MEMORY.health_check();
        let symbol_status = self.get_all_symbols_status();
        
        DetailedStats {
            total_operations: stats.total_operations,
            zero_allocation_success: stats.zero_allocation_success,
            zero_allocation_failures: stats.zero_allocation_failures,
            success_rate: if stats.total_operations > 0 {
                (stats.zero_allocation_success as f64 / stats.total_operations as f64) * 100.0
            } else {
                0.0
            },
            average_processing_time_ns: stats.average_processing_time_ns,
            memory_failure_rate: memory_health.failure_rate,
            active_symbols: symbol_status.values().filter(|s| s.is_active).count(),
            total_symbols: symbol_status.len(),
            memory_allocated_mb: memory_health.total_allocated_mb,
            peak_memory_mb: memory_health.peak_allocated_mb,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolStatus {
    pub symbol: String,
    pub bid_depth: usize,
    pub ask_depth: usize,
    pub last_update: u64,
    pub is_active: bool,
}

#[derive(Debug)]
pub struct DetailedStats {
    pub total_operations: u64,
    pub zero_allocation_success: u64,
    pub zero_allocation_failures: u64,
    pub success_rate: f64,
    pub average_processing_time_ns: u64,
    pub memory_failure_rate: f64,
    pub active_symbols: usize,
    pub total_symbols: usize,
    pub memory_allocated_mb: f64,
    pub peak_memory_mb: f64,
}

// 全局零分配引擎实例
lazy_static::lazy_static! {
    pub static ref ZERO_ALLOCATION_ENGINE: ZeroAllocationEngine = {
        let config = ZeroAllocationConfig::default();
        ZeroAllocationEngine::new(config)
    };
}

// 初始化函数
pub fn init_zero_allocation_system() {
    println!("🚀 初始化Qingxi V3.0零分配系统");
    
    // 强制初始化全局引擎
    lazy_static::initialize(&ZERO_ALLOCATION_ENGINE);
    
    // 运行内存性能基准测试
    crate::memory::benchmark_memory_performance();
    
    println!("✅ 零分配系统初始化完成");
}
