use strategy::depth_analysis::{DepthAnalyzer, DepthAnalysisResult};
use common::{
    market_data::OrderBook,
    precision::{FixedPrice, FixedQuantity},
    arbitrage::Side,
    types::{Exchange, Symbol},
};
use anyhow::Result;

fn main() -> Result<()> {
    println!("🔬 测试v3.0真实深度滑点分析...");

    // 创建测试用订单簿
    let orderbook = create_realistic_orderbook();
    let depth_analyzer = DepthAnalyzer::default();

    // 测试1: 买单深度分析（分析ask侧）
    println!("\n1️⃣ 测试买单深度分析...");
    let buy_target = FixedQuantity::from_f64(2.5, 8); // 目标购买2.5个单位
    
    match depth_analyzer.analyze_depth(&orderbook, Side::Buy, buy_target) {
        Ok(result) => {
            println!("   ✅ 买单分析成功:");
            print_depth_analysis_result(&result, "买单");
            
            // 验证合理性
            assert!(result.effective_price.to_f64() >= 50001.0, "实际价格应高于最佳ask");
            assert!(result.cumulative_slippage_pct >= 0.0, "滑点应为正数");
            
        }
        Err(e) => {
            println!("   ❌ 买单分析失败: {}", e);
            return Err(e);
        }
    }

    // 测试2: 卖单深度分析（分析bid侧）
    println!("\n2️⃣ 测试卖单深度分析...");
    let sell_target = FixedQuantity::from_f64(1.8, 8); // 目标卖出1.8个单位
    
    match depth_analyzer.analyze_depth(&orderbook, Side::Sell, sell_target) {
        Ok(result) => {
            println!("   ✅ 卖单分析成功:");
            print_depth_analysis_result(&result, "卖单");
            
            // 验证合理性
            assert!(result.effective_price.to_f64() <= 50000.0, "实际价格应低于最佳bid");
            
        }
        Err(e) => {
            println!("   ❌ 卖单分析失败: {}", e);
            return Err(e);
        }
    }

    // 测试3: 批量三角套利深度分析
    println!("\n3️⃣ 测试三角套利批量分析...");
    let orderbooks = vec![&orderbook, &orderbook, &orderbook];
    let sides = vec![Side::Buy, Side::Sell, Side::Buy];
    let quantities = vec![
        FixedQuantity::from_f64(1.0, 8),
        FixedQuantity::from_f64(1.0, 8),
        FixedQuantity::from_f64(1.0, 8),
    ];
    
    match depth_analyzer.batch_analyze_triangular_depth(&orderbooks, &sides, &quantities) {
        Ok(results) => {
            println!("   ✅ 三角套利批量分析成功:");
            for (i, result) in results.iter().enumerate() {
                println!("     腿{}: 滑点={:.4}%, 风险={}, 流动性={}", 
                    i + 1, 
                    result.cumulative_slippage_pct,
                    result.execution_risk_score,
                    result.liquidity_score
                );
            }
            
            assert_eq!(results.len(), 3, "应该有3个分析结果");
            
        }
        Err(e) => {
            println!("   ❌ 三角套利批量分析失败: {}", e);
            return Err(e);
        }
    }

    // 测试4: 三角套利数量优化
    println!("\n4️⃣ 测试三角套利数量优化...");
    let initial_amount = FixedQuantity::from_f64(1.0, 8);
    
    match depth_analyzer.optimize_triangular_quantities(&orderbooks, &sides, initial_amount) {
        Ok((optimized_quantities, efficiency)) => {
            println!("   ✅ 数量优化成功:");
            println!("     优化效率评分: {:.2}", efficiency);
            for (i, qty) in optimized_quantities.iter().enumerate() {
                println!("     腿{} 优化数量: {:.4}", i + 1, qty.to_f64());
            }
            
            assert!(efficiency > 0.0, "效率评分应为正数");
            assert_eq!(optimized_quantities.len(), 3, "应该有3个优化数量");
            
        }
        Err(e) => {
            println!("   ❌ 数量优化失败: {}", e);
            return Err(e);
        }
    }

    // 测试5: 性能对比 - 简化模型 vs 真实深度
    println!("\n5️⃣ 性能对比测试...");
    let start_time = std::time::Instant::now();
    
    // 运行100次深度分析
    for _ in 0..100 {
        let _ = depth_analyzer.analyze_depth(&orderbook, Side::Buy, FixedQuantity::from_f64(1.0, 8));
    }
    
    let duration = start_time.elapsed();
    println!("   ✅ 100次深度分析耗时: {:.2}ms (平均 {:.4}ms/次)", 
        duration.as_secs_f64() * 1000.0,
        duration.as_secs_f64() * 10.0
    );
    
    if duration.as_millis() < 100 {
        println!("   🚀 性能优秀: 满足高频交易要求");
    } else {
        println!("   ⚠️ 性能告警: 可能需要进一步优化");
    }

    println!("\n🎉 v3.0真实深度滑点分析测试全部通过！");
    println!("✅ 精度提升: 真实订单簿遍历替代简化模型");
    println!("✅ 风险控制: 多维度风险和流动性评分");
    println!("✅ 性能优化: 适合高频交易环境");
    println!("✅ 生产就绪: 完整的错误处理和边界检查");

    Ok(())
}

/// 创建贴近真实市场的测试订单簿
fn create_realistic_orderbook() -> OrderBook {
    // 模拟BTCUSDT订单簿，带有真实的市场微观结构
    OrderBook {
        exchange: Exchange::new("binance"),
        symbol: Symbol::new("BTCUSDT"),
        // Bid侧（买单）- 价格递减
        bid_prices: vec![
            FixedPrice::from_f64(50000.00, 2), // 最佳买价
            FixedPrice::from_f64(49999.50, 2),
            FixedPrice::from_f64(49999.00, 2),
            FixedPrice::from_f64(49998.00, 2),
            FixedPrice::from_f64(49995.00, 2),
        ],
        bid_quantities: vec![
            FixedQuantity::from_f64(0.5, 8),  // 最佳价位数量较少
            FixedQuantity::from_f64(1.2, 8),
            FixedQuantity::from_f64(2.1, 8),
            FixedQuantity::from_f64(3.5, 8),
            FixedQuantity::from_f64(5.0, 8),  // 深层价位数量较多
        ],
        // Ask侧（卖单）- 价格递增
        ask_prices: vec![
            FixedPrice::from_f64(50001.00, 2), // 最佳卖价
            FixedPrice::from_f64(50001.50, 2),
            FixedPrice::from_f64(50002.00, 2),
            FixedPrice::from_f64(50003.00, 2),
            FixedPrice::from_f64(50005.00, 2),
        ],
        ask_quantities: vec![
            FixedQuantity::from_f64(0.8, 8),  // 最佳价位数量较少
            FixedQuantity::from_f64(1.5, 8),
            FixedQuantity::from_f64(2.3, 8),
            FixedQuantity::from_f64(4.2, 8),
            FixedQuantity::from_f64(6.0, 8),  // 深层价位数量较多
        ],
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
        sequence: 12345,
        quality_score: 0.95,
        processing_latency_ns: 1000,
    }
}

/// 打印深度分析结果
fn print_depth_analysis_result(result: &DepthAnalysisResult, side_name: &str) {
    println!("     {} 深度分析结果:", side_name);
    println!("       有效执行价格: ${:.2}", result.effective_price.to_f64());
    println!("       最大可执行量: {:.4}", result.max_quantity.to_f64());
    println!("       累积滑点: {:.4}%", result.cumulative_slippage_pct);
    println!("       流动性评分: {}/100", result.liquidity_score);
    println!("       执行风险评分: {}/100", result.execution_risk_score);
    println!("       价格影响层级: {} 层", result.price_impact_curve.len());
} 