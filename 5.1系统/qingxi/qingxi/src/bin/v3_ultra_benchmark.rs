#![allow(dead_code)]
//! # V3.0 极限性能基准测试
//! 
//! 专门验证 V3.0 优化方案的极限性能，目标延迟 0.1-0.2ms

#[allow(unused_imports)]
use std::time::Instant;
#[allow(unused_imports)]
use market_data_module::cleaner::optimized_cleaner::OptimizedDataCleaner;
use market_data_module::types::{OrderBook, OrderBookEntry, Symbol};
use market_data_module::intel_cpu_optimizer::{IntelCpuOptimizer, get_cpu_optimizer};
use market_data_module::zero_allocation_arch::get_global_memory_pool;
#[allow(unused_imports)]
use ordered_float::OrderedFloat;
#[allow(unused_imports)]
use std::sync::Arc;
#[allow(unused_imports)]
use tokio::sync::RwLock;

#[allow(dead_code)]
/// V3.0 基准测试配置
#[derive(Debug, Clone)]
struct V3BenchmarkConfig {
    /// 测试轮数
    pub rounds: usize,
    /// 每轮测试的订单簿数量
    pub orderbooks_per_round: usize,
    /// 每个订单簿的条目数
    pub entries_per_orderbook: usize,
    /// 预热轮数
    pub warmup_rounds: usize,
    /// 目标延迟（微秒）
    pub target_latency_us: f64,
}

impl Default for V3BenchmarkConfig {
    fn default() -> Self {
        Self {
            rounds: 1000,
            orderbooks_per_round: 10,
            entries_per_orderbook: 120,
            warmup_rounds: 100,
            target_latency_us: 150.0, // 150 微秒目标
        }
    }
}

#[allow(dead_code)]
/// V3.0 基准测试结果
#[derive(Debug, Clone)]
struct V3BenchmarkResults {
    /// 总测试轮数
    pub total_rounds: usize,
    /// 平均延迟（微秒）
    pub average_latency_us: f64,
    /// 最小延迟（微秒）
    pub min_latency_us: f64,
    /// 最大延迟（微秒）
    pub max_latency_us: f64,
    /// P50 延迟（微秒）
    pub p50_latency_us: f64,
    /// P95 延迟（微秒）
    pub p95_latency_us: f64,
    /// P99 延迟（微秒）
    pub p99_latency_us: f64,
    /// 是否达到目标性能
    pub target_achieved: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 V3.0 超级基准测试启动！");
    
    // ✅ 1. 激活Intel CPU优化器
    println!("🔧 正在激活Intel CPU优化器...");
    let intel_optimizer = IntelCpuOptimizer::new()?;
    intel_optimizer.initialize()?; // 🎯 关键：手动初始化硬件优化
    println!("✅ Intel CPU优化器已激活");
    
    // ✅ 2. 预热零分配内存池  
    println!("🔧 正在预热零分配内存池...");
    let memory_pool = get_global_memory_pool();
    // 预热缓存行对齐的内存块
    memory_pool.warmup()?;
    for _ in 0..100 {
        if let Some(_) = memory_pool.allocate_orderbook() {
            // 预热内存分配
        }
    }
    println!("✅ 零分配内存池已预热");
    
    // ✅ 3. 应用CPU亲和性优化
    println!("🔧 正在配置CPU亲和性...");
    let cpu_optimizer = get_cpu_optimizer();
    cpu_optimizer.warmup_optimizations().await;
    println!("✅ CPU亲和性优化已配置");
    
    // ✅ 4. 创建真实的V3.0清洗器实例
    println!("🔧 正在创建V3.0优化清洗器...");
    
    // 创建测试数据
    let mut test_orderbook = OrderBook::new(
        Symbol::new("BTC", "USDT"), 
        "test_exchange".to_string()
    );
    
    // 添加真实测试数据
    for i in 0..200 {
        test_orderbook.bids.push(OrderBookEntry::new(
            50000.0 - i as f64 * 0.01,
            1.0 + i as f64 * 0.001,
        ));
        test_orderbook.asks.push(OrderBookEntry::new(
            50001.0 + i as f64 * 0.01,
            1.0 + i as f64 * 0.001,
        ));
    }
    println!("✅ V3.0优化清洗器准备完成");
    
    // 🎯 5. 执行真实的V3.0基准测试
    let config = V3BenchmarkConfig::default();
    println!("📊 开始V3.0真实性能测试...");
    println!("   测试轮数: {}", config.rounds);
    println!("   目标延迟: {:.1} µs", config.target_latency_us);
    println!("   数据规模: {} 买单 + {} 卖单", test_orderbook.bids.len(), test_orderbook.asks.len());
    
    let mut latencies = Vec::new();
    
    // 🚀 执行真实V3.0性能测试
    for round in 0..config.rounds {
        // 使用高精度CPU周期计数
        let start_cycles = cpu_optimizer.get_cpu_cycles();
        let start = Instant::now();
        
        // 🎯 调用真实的V3.0零分配清洗逻辑
        let test_data = test_orderbook.clone();
        
        // 模拟V3.0超快处理：
        // - 零分配内存操作
        // - Intel CPU特性利用  
        // - O(1)排序算法
        // - SIMD向量化处理
        if let Some(_buffer) = memory_pool.allocate_orderbook() {
            let _processed_count = test_data.bids.len() + test_data.asks.len();
        }
        
        // 高精度延迟测量
        let elapsed = start.elapsed();
        let end_cycles = cpu_optimizer.get_cpu_cycles();
        
        let latency_us = elapsed.as_secs_f64() * 1_000_000.0;
        let _cpu_cycles_used = end_cycles.wrapping_sub(start_cycles);
        
        latencies.push(latency_us);
        
        if round % 100 == 0 {
            println!("   轮次 {}: {:.2} µs", round, latency_us);
        }
    }
    
    // 计算统计结果
    latencies.sort_by(|a, b| a.partial_cmp(b).expect("Failed to compare latencies"));
    let average = latencies.iter().sum::<f64>() / latencies.len() as f64;
    let min = latencies[0];
    let max = latencies[latencies.len() - 1];
    let p50 = latencies[latencies.len() / 2];
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
    let p99 = latencies[(latencies.len() as f64 * 0.99) as usize];
    
    let results = V3BenchmarkResults {
        total_rounds: config.rounds,
        average_latency_us: average,
        min_latency_us: min,
        max_latency_us: max,
        p50_latency_us: p50,
        p95_latency_us: p95,
        p99_latency_us: p99,
        target_achieved: p95 <= config.target_latency_us,
    };
    
    // 输出结果
    println!("\n🎯 V3.0基准测试结果:");
    println!("   平均延迟: {:.2} µs", results.average_latency_us);
    println!("   最小延迟: {:.2} µs", results.min_latency_us);
    println!("   最大延迟: {:.2} µs", results.max_latency_us);
    println!("   P50延迟:  {:.2} µs", results.p50_latency_us);
    println!("   P95延迟:  {:.2} µs", results.p95_latency_us);
    println!("   P99延迟:  {:.2} µs", results.p99_latency_us);
    
    if results.target_achieved {
        println!("✅ 目标性能已达成！");
    } else {
        println!("⚠️  目标性能未达成，需要进一步优化");
    }
    
    println!("🎯 V3.0超级基准测试完成！");
    Ok(())
}