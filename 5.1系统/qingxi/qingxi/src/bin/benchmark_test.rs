#![allow(dead_code)]
//! # 24小时性能优化验证测试
//! 
//! 🚀 阶段3: 运行综合基准测试，验证SOL清洗从3ms到0.5-0.8ms的优化效果

use market_data_module::performance_benchmark::*;
use market_data_module::types::*;
use market_data_module::cleaner::optimized_cleaner::OptimizedDataCleaner;
use market_data_module::bucket_orderbook::BucketOrderBook;
use market_data_module::simd_optimizations::SimdDataValidator;
use std::time::Instant;
use ordered_float::OrderedFloat;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志 - 简化版本
    println!("🎯 === 24小时Qingxi性能优化验证测试 ===");
    
    println!("🎯 === 24小时Qingxi性能优化验证测试 ===");
    println!("💡 目标: SOL币种清洗时间从3ms优化到0.5-0.8ms");
    println!("🚀 预期性能提升: 4-6倍");
    println!();
    
    // 创建基准测试器
    let mut benchmark = PerformanceBenchmark::new();
    
    // 运行综合基准测试
    let results = benchmark.benchmark_sol_cleaning_performance().await;
    
    // 打印详细结果
    print_detailed_results(&results);
    
    // 运行额外的性能测试
    println!("\n🔧 === 额外性能验证测试 ===");
    run_bucket_orderbook_test();
    run_simd_validation_test();
    run_memory_pool_test().await;
    
    println!("\n🎉 === 24小时性能优化验证完成 ===");
    
    Ok(())
}

/// 打印详细的基准测试结果
fn print_detailed_results(results: &BenchmarkResults) {
    println!("📊 === 性能测试结果详情 ===");
    
    println!("\n🔍 阶段1优化 (配置优化+内存池):");
    if !results.stage1_times.is_empty() {
        let avg_stage1 = results.stage1_times.iter().sum::<f64>() / results.stage1_times.len() as f64;
        println!("  平均清洗时间: {:.3} ms", avg_stage1);
        println!("  最快清洗时间: {:.3} ms", results.stage1_times.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        println!("  最慢清洗时间: {:.3} ms", results.stage1_times.iter().fold(0.0f64, |a, &b| a.max(b)));
    }
    
    println!("\n⚡ 阶段2优化 (桶排序+AVX-512+并行):");
    if !results.stage2_times.is_empty() {
        let avg_stage2 = results.stage2_times.iter().sum::<f64>() / results.stage2_times.len() as f64;
        println!("  平均清洗时间: {:.3} ms", avg_stage2);
        println!("  最快清洗时间: {:.3} ms", results.stage2_times.iter().fold(f64::INFINITY, |a, &b| a.min(b)));
        println!("  最慢清洗时间: {:.3} ms", results.stage2_times.iter().fold(0.0f64, |a, &b| a.max(b)));
    }
    
    println!("\n🚀 综合性能评估:");
    println!("  性能提升倍数: {:.2}x", results.improvement_factor);
    println!("  目标达成状态: {}", if results.target_achieved { "✅ 已达成" } else { "❌ 需继续优化" });
    
    if results.target_achieved {
        println!("  🎉 恭喜！成功将SOL清洗时间优化到0.5-0.8ms范围！");
    } else {
        println!("  💡 建议: 继续实施阶段3和阶段4优化方案");
    }
}

/// 桶排序订单簿性能测试
fn run_bucket_orderbook_test() {
    println!("\n🪣 桶排序订单簿性能测试:");
    
    let mut bucket_book = BucketOrderBook::new(
        "SOLUSDT".to_string(), 
        "test".to_string(), 
        (100.0, 200.0)
    );
    
    // 预热
    for i in 0..1000 {
        let price = 150.0 + (i as f64 % 50.0) * 0.01;
        bucket_book.update_bid(price, 1.0);
    }
    
    // 性能测试
    let iterations = 10000;
    let start = Instant::now();
    
    for i in 0..iterations {
        let price = 150.0 + (i as f64 % 100.0) * 0.01;
        bucket_book.update_bid(price, 1.0 + (i as f64 % 5.0));
        bucket_book.update_ask(price + 0.01, 1.0 + (i as f64 % 3.0));
    }
    
    let elapsed = start.elapsed();
    let avg_update_time = elapsed.as_nanos() as f64 / (iterations * 2) as f64; // bid + ask
    
    println!("  {}次更新操作完成", iterations * 2);
    println!("  总耗时: {:?}", elapsed);
    println!("  平均每次更新: {:.1} ns", avg_update_time);
    
    let (updates, hits, hit_rate) = bucket_book.get_performance_stats();
    println!("  桶命中率: {:.1}% ({}/{})", hit_rate * 100.0, hits, updates);
    
    if avg_update_time < 1000.0 { // 小于1μs
        println!("  ✅ 桶排序性能优秀!");
    } else {
        println!("  ⚠️  桶排序性能需要优化");
    }
}

/// SIMD验证性能测试
fn run_simd_validation_test() {
    println!("\n⚡ SIMD价格验证性能测试:");
    
    let validator = SimdDataValidator::new(100.0, 200.0, 0.1, 0.05);
    
    // 生成测试数据
    let test_prices: Vec<f64> = (0..10000)
        .map(|i| 150.0 + (i as f64 * 0.001) % 50.0)
        .collect();
    
    // 预热
    let _ = validator.validate_prices_batch(&test_prices[..100]);
    
    // 性能测试
    let start = Instant::now();
    let valid_count = validator.validate_prices_batch(&test_prices);
    let elapsed = start.elapsed();
    
    println!("  验证{}个价格", test_prices.len());
    println!("  有效价格数: {}", valid_count);
    println!("  总耗时: {:?}", elapsed);
    println!("  平均每个价格: {:.1} ns", elapsed.as_nanos() as f64 / test_prices.len() as f64);
    
    if elapsed.as_micros() < 100 { // 小于100μs
        println!("  ✅ SIMD验证性能优秀!");
    } else {
        println!("  ⚠️  SIMD验证性能需要优化");
    }
}

/// 内存池性能测试
async fn run_memory_pool_test() {
    println!("\n💾 内存池性能测试:");
    
    let (_tx, rx) = flume::unbounded();
    let (output_tx, _) = flume::unbounded();
    let cleaner = OptimizedDataCleaner::new(rx, output_tx);
    
    // 创建测试订单簿
    let mut test_orderbook = OrderBook {
        symbol: Symbol::from_string("SOLUSDT").unwrap_or_else(|_| Symbol::new("SOL", "USDT")),
        source: "test".to_string(),
        bids: Vec::with_capacity(100),
        asks: Vec::with_capacity(100),
        timestamp: market_data_module::high_precision_time::Nanos::now(),
        sequence_id: Some(1),
        checksum: None,
    };
    
    // 填充测试数据
    for i in 0..50 {
        test_orderbook.bids.push(OrderBookEntry {
            price: OrderedFloat(150.0 - i as f64 * 0.01),
            quantity: OrderedFloat(1.0 + i as f64 * 0.1),
        });
        test_orderbook.asks.push(OrderBookEntry {
            price: OrderedFloat(150.01 + i as f64 * 0.01),
            quantity: OrderedFloat(1.0 + i as f64 * 0.1),
        });
    }
    
    // 性能测试
    let iterations = 1000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _ = cleaner.clean_orderbook_ultrafast(Some(test_orderbook.clone())).await;
    }
    
    let elapsed = start.elapsed();
    let avg_time = elapsed.as_micros() as f64 / iterations as f64;
    
    println!("  {}次清洗操作完成", iterations);
    println!("  总耗时: {:?}", elapsed);
    println!("  平均每次清洗: {:.1} μs", avg_time);
    
    if avg_time < 800.0 { // 小于0.8ms
        println!("  ✅ 内存池性能达到目标!");
    } else if avg_time < 1500.0 { // 小于1.5ms
        println!("  ⚡ 内存池性能良好，接近目标");
    } else {
        println!("  ⚠️  内存池性能需要进一步优化");
    }
}
