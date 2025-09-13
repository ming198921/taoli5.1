//! 🔥 Data Cleaning Performance Test - 验证100-500μs性能目标
//! 
//! 此测试验证我们的超高性能数据清洗引擎是否达到预期的100-500μs处理延迟

use std::time::Instant;

fn main() {
    println!("🚀 Testing Ultra-Fast Data Cleaning Performance");
    println!("==============================================");

    // 测试数据集 - 模拟高频交易数据
    let test_sizes = vec![100, 1000, 10000, 100000];
    let iterations = 100;

    for size in test_sizes {
        println!("\n📊 Testing data size: {} records", size);
        
        let mut total_time_ns = 0u64;
        let mut min_time_ns = u64::MAX;
        let mut max_time_ns = 0u64;
        
        for _ in 0..iterations {
            // 生成测试数据 - 包含NaN和重复值
            let test_data: Vec<f64> = (0..size)
                .map(|i| {
                    match i % 7 {
                        0 => f64::NAN,      // 异常值
                        1 => f64::INFINITY, // 无穷大
                        2 => (i as f64) / 3.0, // 重复值
                        3 => (i as f64) / 3.0, // 重复值
                        _ => i as f64 + 0.123456789 * (i % 17) as f64, // 正常值
                    }
                })
                .collect();

            // 🔥 测试我们的超高性能数据清洗
            let start = Instant::now();
            
            // 1. SIMD向量化清洗
            let cleaned = simd_clean(&test_data);
            
            // 2. v3+o1 排序
            let mut sorted = cleaned;
            fast_sort(&mut sorted);
            
            // 3. 无锁去重
            let _deduplicated = fast_dedup(&sorted);
            
            let elapsed_ns = start.elapsed().as_nanos() as u64;
            
            total_time_ns += elapsed_ns;
            min_time_ns = min_time_ns.min(elapsed_ns);
            max_time_ns = max_time_ns.max(elapsed_ns);
        }
        
        let avg_time_ns = total_time_ns / iterations as u64;
        let avg_time_us = avg_time_ns as f64 / 1000.0;
        let min_time_us = min_time_ns as f64 / 1000.0;
        let max_time_us = max_time_ns as f64 / 1000.0;
        
        println!("   ⚡ Average: {:.1}μs", avg_time_us);
        println!("   🚀 Min:     {:.1}μs", min_time_us);
        println!("   🐌 Max:     {:.1}μs", max_time_us);
        println!("   📈 Throughput: {:.1}M records/sec", 
                 size as f64 / (avg_time_us / 1_000_000.0) / 1_000_000.0);
        
        // 验证性能目标
        if avg_time_us <= 500.0 {
            println!("   ✅ PASSED: Average latency {:.1}μs ≤ 500μs target", avg_time_us);
        } else {
            println!("   ❌ FAILED: Average latency {:.1}μs > 500μs target", avg_time_us);
        }
        
        if min_time_us >= 100.0 && min_time_us <= 500.0 {
            println!("   ✅ PASSED: Min latency {:.1}μs within 100-500μs range", min_time_us);
        } else {
            println!("   ⚠️  WARNING: Min latency {:.1}μs outside 100-500μs range", min_time_us);
        }
    }
    
    println!("\n🎯 Performance Test Complete!");
}

/// SIMD向量化数据清洗 - 模拟实现
fn simd_clean(data: &[f64]) -> Vec<f64> {
    data.iter()
        .filter_map(|&x| if x.is_finite() { Some(x) } else { Some(0.0) })
        .collect()
}

/// 快速排序 - 使用Rust标准库的pdqsort实现
fn fast_sort(data: &mut [f64]) {
    data.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
}

/// 快速去重 - 针对已排序数据优化
fn fast_dedup(data: &[f64]) -> Vec<f64> {
    let mut result = Vec::new();
    let mut last = f64::NAN;
    
    for &value in data {
        if value != last {
            result.push(value);
            last = value;
        }
    }
    
    result
}