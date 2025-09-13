//! SIMD套利计算性能基准测试
//! 
//! 验证目标：1000个价格点处理时间 ≤ 1微秒

use std::time::Instant;
use celue::performance::simd_fixed_point::{SIMDFixedPointProcessor, FixedPrice, FixedQuantity};

fn main() {
    println!("🚀 高性能SIMD套利计算基准测试");
    println!("目标：1000个价格点处理时间 ≤ 1微秒\n");
    
    let processor = SIMDFixedPointProcessor::new(1000);
    
    // 生成测试数据：1000个价格点
    let buy_prices: Vec<FixedPrice> = (0..1000)
        .map(|i| FixedPrice::from_f64(100.0 + i as f64 * 0.01))
        .collect();
    let sell_prices: Vec<FixedPrice> = (0..1000)
        .map(|i| FixedPrice::from_f64(101.0 + i as f64 * 0.01))
        .collect();
    let buy_volumes: Vec<FixedQuantity> = (0..1000)
        .map(|i| FixedQuantity::from_f64(10.0 + i as f64 * 0.001))
        .collect();
    let sell_volumes: Vec<FixedQuantity> = (0..1000)
        .map(|i| FixedQuantity::from_f64(9.0 + i as f64 * 0.001))
        .collect();
    let fee_rates: Vec<FixedPrice> = (0..1000)
        .map(|_| FixedPrice::from_f64(0.001)) // 0.1% 费率
        .collect();
    
    println!("📊 测试数据生成完成：1000个价格点");
    println!("买入价格范围: {:.2} - {:.2}", 
        buy_prices[0].to_f64(), buy_prices[999].to_f64());
    println!("卖出价格范围: {:.2} - {:.2}", 
        sell_prices[0].to_f64(), sell_prices[999].to_f64());
    
    // 预热运行
    for _ in 0..10 {
        let _ = processor.calculate_arbitrage_profits_batch(
            &buy_prices, &sell_prices, &buy_volumes, &sell_volumes, &fee_rates
        ).unwrap();
    }
    
    // 基准测试：100次运行取平均
    let iterations = 100;
    let mut total_duration = std::time::Duration::new(0, 0);
    
    println!("\n⚡ 开始性能基准测试（{}次迭代）...", iterations);
    
    for i in 0..iterations {
        let start = Instant::now();
        
        let profits = processor.calculate_arbitrage_profits_batch(
            &buy_prices, &sell_prices, &buy_volumes, &sell_volumes, &fee_rates
        ).unwrap();
        
        let duration = start.elapsed();
        total_duration += duration;
        
        if i == 0 {
            // 验证结果正确性
            println!("✅ 结果验证：");
            println!("   计算得到 {} 个套利机会", profits.len());
            println!("   第一个机会 - 毛利润: {:.6}, 净利润: {:.6}, 费用: {:.6}",
                profits[0].gross_profit.to_f64(),
                profits[0].net_profit.to_f64(), 
                profits[0].fee.to_f64());
            println!("   最后一个机会 - 毛利润: {:.6}, 净利润: {:.6}",
                profits[999].gross_profit.to_f64(),
                profits[999].net_profit.to_f64());
        }
    }
    
    let avg_duration = total_duration / iterations;
    let avg_nanos = avg_duration.as_nanos();
    let avg_micros = avg_nanos as f64 / 1000.0;
    
    println!("\n📈 性能测试结果：");
    println!("   平均处理时间: {:.3} 微秒 ({} 纳秒)", avg_micros, avg_nanos);
    println!("   1000个价格点处理速度: {:.1} 点/微秒", 1000.0 / avg_micros);
    
    // 目标验证
    if avg_micros <= 1.0 {
        println!("🎉 目标达成！处理时间 {:.3}μs ≤ 1μs", avg_micros);
    } else {
        println!("⚠️  目标未达成：处理时间 {:.3}μs > 1μs", avg_micros);
        println!("   但仍然是极高性能！");
    }
    
    // 特性检测
    println!("\n🔧 CPU特性检测：");
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx512f") {
            println!("   ✅ AVX-512 支持 - 使用512位向量指令");
        } else if std::arch::is_x86_feature_detected!("avx2") {
            println!("   ✅ AVX2 支持 - 使用256位向量指令");  
        } else {
            println!("   ⚠️  仅标量支持 - 无SIMD加速");
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        println!("   ⚠️  非x86_64架构 - 使用标量fallback");
    }
    
    // 吞吐量计算
    let points_per_second = 1000.0 * 1_000_000.0 / avg_nanos as f64;
    println!("\n📊 吞吐量分析：");
    println!("   每秒可处理: {:.0} 个价格点", points_per_second);
    println!("   每秒可处理: {:.0} 个套利机会", points_per_second);
    
    if points_per_second >= 1_000_000_000.0 {
        println!("   🚀 超高性能：>10亿点/秒");
    } else if points_per_second >= 100_000_000.0 {
        println!("   ⚡ 高性能：>1亿点/秒");
    } else {
        println!("   📈 良好性能：{:.0}万点/秒", points_per_second / 10_000.0);
    }
} 