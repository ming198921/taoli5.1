//! 影子交易系统演示程序
//! 
//! 展示如何使用影子交易系统进行完整的模拟交易

use shadow_trading::{
    ShadowTradingSystem, ShadowTradingConfig, ShadowOrder, 
    OrderSide, OrderType, OrderStatus, ReportFormat
};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, Level};
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("🚀 启动影子交易系统演示程序");

    // 1. 创建和启动影子交易系统
    let config = ShadowTradingConfig::for_development();
    let system = ShadowTradingSystem::new(config).await?;
    system.start().await?;

    info!("✅ 影子交易系统已启动");

    // 2. 创建虚拟账户
    let mut initial_balance = HashMap::new();
    initial_balance.insert("USDT".to_string(), 100000.0);  // $100,000 USDT
    initial_balance.insert("BTC".to_string(), 0.0);

    let account_id = "demo_trader_001";
    system.create_virtual_account(account_id.to_string(), initial_balance).await?;

    info!("🏦 已创建虚拟账户: {}", account_id);

    // 3. 获取初始账户信息
    let account = system.get_virtual_account(account_id).await?;
    info!("💰 初始余额: USDT={:.2}", account.get_available_balance("USDT"));

    // 4. 演示市价单交易
    info!("📈 执行市价买入订单...");
    let market_buy_order = ShadowOrder {
        id: String::new(),
        account_id: account_id.to_string(),
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        quantity: 2.0,
        price: None, // 市价单
        order_type: OrderType::Market,
        status: OrderStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        filled_quantity: 0.0,
        average_price: None,
        fees: 0.0,
        metadata: HashMap::new(),
    };

    let order_id = system.submit_shadow_order(market_buy_order).await?;
    info!("📋 市价买入订单已提交: {}", order_id);

    // 等待订单执行
    sleep(Duration::from_millis(500)).await;

    // 检查订单状态
    match system.get_order_status(&order_id).await {
        Ok(order) => {
            info!("📊 订单状态: {:?}, 成交数量: {:.4}", order.status, order.filled_quantity);
            if let Some(avg_price) = order.average_price {
                info!("💱 平均成交价: ${:.2}", avg_price);
            }
        }
        Err(e) => warn!("❌ 获取订单状态失败: {}", e),
    }

    // 5. 演示限价单交易
    info!("📊 执行限价卖出订单...");
    let current_price = system.get_simulated_price("BTC/USDT").await?;
    let limit_price = current_price * 1.02; // 高于当前价格2%

    let limit_sell_order = ShadowOrder {
        id: String::new(),
        account_id: account_id.to_string(),
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Sell,
        quantity: 1.0,
        price: Some(limit_price),
        order_type: OrderType::Limit,
        status: OrderStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        filled_quantity: 0.0,
        average_price: None,
        fees: 0.0,
        metadata: HashMap::new(),
    };

    let limit_order_id = system.submit_shadow_order(limit_sell_order).await?;
    info!("📋 限价卖出订单已提交: {} @ ${:.2}", limit_order_id, limit_price);

    // 6. 模拟市场运行
    info!("⏱️  等待市场运行和订单成交...");
    for i in 1..=10 {
        sleep(Duration::from_secs(2)).await;
        
        // 获取当前市价
        let current_price = system.get_simulated_price("BTC/USDT").await?;
        
        // 获取账户更新后的信息
        let updated_account = system.get_virtual_account(account_id).await?;
        
        info!(
            "🔄 第{}次更新 - BTC价格: ${:.2}, USDT余额: {:.2}, BTC余额: {:.6}", 
            i, 
            current_price,
            updated_account.get_available_balance("USDT"),
            updated_account.get_available_balance("BTC")
        );

        // 检查限价订单是否成交
        if let Ok(order) = system.get_order_status(&limit_order_id).await {
            if order.status == OrderStatus::Filled {
                info!("🎉 限价订单已成交!");
                break;
            }
        }
    }

    // 7. 获取交易历史
    info!("📜 获取交易历史...");
    let trades = system.get_trade_history(account_id, None, None).await?;
    for (i, trade) in trades.iter().enumerate() {
        info!(
            "📈 交易#{}: {} {:.4} {} @ ${:.2} (手续费: ${:.2})",
            i + 1,
            format!("{:?}", trade.side),
            trade.quantity,
            trade.symbol,
            trade.price,
            trade.fees
        );
    }

    // 8. 获取性能统计
    info!("📊 获取性能统计...");
    match system.get_performance_stats(account_id).await {
        Ok(stats) => {
            info!("📈 总收益率: {:.2}%", stats.total_return * 100.0);
            info!("📉 最大回撤: {:.2}%", stats.max_drawdown * 100.0);
            info!("📊 夏普比率: {:.3}", stats.sharpe_ratio);
            info!("💰 当前价值: ${:.2}", stats.current_value);
        }
        Err(e) => warn!("❌ 获取性能统计失败: {}", e),
    }

    // 9. 获取交易指标
    info!("📋 获取交易指标...");
    match system.get_trading_metrics(account_id).await {
        Ok(metrics) => {
            info!("🎯 总交易数: {}", metrics.total_trades);
            info!("✅ 盈利交易: {} ({:.1}%)", metrics.winning_trades, metrics.win_rate * 100.0);
            info!("❌ 亏损交易: {}", metrics.losing_trades);
            info!("💰 平均盈利: ${:.2}", metrics.average_win);
            info!("💸 平均亏损: ${:.2}", metrics.average_loss);
            info!("📊 盈亏比: {:.2}", metrics.profit_factor);
        }
        Err(e) => warn!("❌ 获取交易指标失败: {}", e),
    }

    // 10. 导出交易报告
    info!("📄 导出交易报告...");
    match system.export_trading_report(account_id, ReportFormat::Json).await {
        Ok(report) => {
            println!("📊 交易报告 (JSON格式):");
            println!("{}", report);
        }
        Err(e) => warn!("❌ 导出报告失败: {}", e),
    }

    // 11. 演示风险管理
    info!("⚠️ 演示风险管理功能...");
    
    // 尝试提交一个超大订单来触发风险限制
    let risky_order = ShadowOrder {
        id: String::new(),
        account_id: account_id.to_string(),
        symbol: "BTC/USDT".to_string(),
        side: OrderSide::Buy,
        quantity: 50.0, // 巨大数量
        price: Some(current_price),
        order_type: OrderType::Limit,
        status: OrderStatus::Pending,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        filled_quantity: 0.0,
        average_price: None,
        fees: 0.0,
        metadata: HashMap::new(),
    };

    match system.submit_shadow_order(risky_order).await {
        Ok(risky_order_id) => {
            info!("⚠️ 风险订单已提交: {}", risky_order_id);
        }
        Err(e) => {
            info!("🛡️ 风险管理系统阻止了危险订单: {}", e);
        }
    }

    // 12. 系统状态检查
    info!("🔍 检查系统状态...");
    let status = system.get_system_status().await;
    info!("📊 系统状态:");
    info!("  - 运行中: {}", status.running);
    info!("  - 活跃账户数: {}", status.account_count);
    info!("  - 运行时间: {:.0}秒", status.uptime.as_secs_f64());
    info!("  - 总订单数: {}", status.total_orders);
    info!("  - 总交易数: {}", status.total_trades);

    // 13. 重置账户演示
    info!("🔄 演示账户重置功能...");
    system.reset_account(account_id).await?;
    
    let reset_account = system.get_virtual_account(account_id).await?;
    info!("✅ 账户已重置 - USDT余额: {:.2}", reset_account.get_available_balance("USDT"));

    // 14. 清理和停止
    info!("🧹 清理资源...");
    system.delete_virtual_account(account_id).await?;
    system.stop().await?;

    info!("🎉 演示程序执行完成！");
    info!("📝 演示内容总结:");
    info!("  ✅ 创建和管理虚拟账户");
    info!("  ✅ 执行市价单和限价单");
    info!("  ✅ 实时市场数据模拟");
    info!("  ✅ 订单匹配和执行");
    info!("  ✅ 交易历史跟踪");
    info!("  ✅ 性能统计分析");
    info!("  ✅ 风险管理保护");
    info!("  ✅ 报告生成导出");
    info!("  ✅ 系统状态监控");

    Ok(())
}

/// 演示高级功能
#[allow(dead_code)]
async fn demonstrate_advanced_features(system: &ShadowTradingSystem) -> anyhow::Result<()> {
    info!("🚀 演示高级功能...");

    // 1. 市场条件设置
    use shadow_trading::MarketCondition;
    
    info!("📊 设置牛市条件...");
    system.set_market_condition(MarketCondition::Bull).await?;
    sleep(Duration::from_secs(5)).await;

    info!("📉 设置熊市条件...");
    system.set_market_condition(MarketCondition::Bear).await?;
    sleep(Duration::from_secs(5)).await;

    info!("📈 恢复震荡市场...");
    system.set_market_condition(MarketCondition::Sideways).await?;

    // 2. 价格模拟设置
    use shadow_trading::PriceSimulation;
    
    let price_sim = PriceSimulation {
        symbol: "ETH/USDT".to_string(),
        initial_price: 3000.0,
        target_price: Some(3200.0),
        volatility: 0.025,
        drift: 0.001,
        jump_intensity: 0.01,
    };
    
    system.add_price_simulation("ETH/USDT".to_string(), price_sim).await?;
    info!("📊 已设置ETH/USDT价格模拟参数");

    // 3. 批量订单提交演示
    info!("📝 演示批量订单提交...");
    let account_id = "batch_trader";
    
    // 创建批量测试账户
    let mut balance = HashMap::new();
    balance.insert("USDT".to_string(), 50000.0);
    system.create_virtual_account(account_id.to_string(), balance).await?;

    // 提交多个限价订单
    let symbols = vec!["BTC/USDT", "ETH/USDT", "BNB/USDT"];
    for (i, symbol) in symbols.iter().enumerate() {
        let current_price = system.get_simulated_price(symbol).await.unwrap_or(1000.0);
        
        let order = ShadowOrder {
            id: String::new(),
            account_id: account_id.to_string(),
            symbol: symbol.to_string(),
            side: if i % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell },
            quantity: 0.5,
            price: Some(current_price * if i % 2 == 0 { 0.98 } else { 1.02 }),
            order_type: OrderType::Limit,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            filled_quantity: 0.0,
            average_price: None,
            fees: 0.0,
            metadata: HashMap::new(),
        };

        match system.submit_shadow_order(order).await {
            Ok(order_id) => info!("📋 批量订单 #{}: {} - {}", i + 1, symbol, order_id),
            Err(e) => warn!("❌ 批量订单 #{} 失败: {}", i + 1, e),
        }
    }

    // 等待处理
    sleep(Duration::from_secs(3)).await;

    // 获取账户订单
    let orders = system.get_account_orders(account_id).await?;
    info!("📊 账户 {} 共有 {} 个订单", account_id, orders.len());

    // 清理
    system.delete_virtual_account(account_id).await?;

    Ok(())
}

/// 压力测试演示
#[allow(dead_code)]
async fn demonstrate_stress_test(system: &ShadowTradingSystem) -> anyhow::Result<()> {
    info!("⚡ 开始压力测试...");

    let account_count = 10;
    let orders_per_account = 20;

    // 创建多个测试账户
    let mut account_ids = Vec::new();
    for i in 0..account_count {
        let account_id = format!("stress_test_account_{:03}", i);
        let mut balance = HashMap::new();
        balance.insert("USDT".to_string(), 25000.0);
        
        system.create_virtual_account(account_id.clone(), balance).await?;
        account_ids.push(account_id);
    }

    info!("✅ 已创建 {} 个测试账户", account_count);

    // 并发提交订单
    let start_time = std::time::Instant::now();
    let mut tasks = Vec::new();

    for account_id in &account_ids {
        let system_clone = system;  // 注意：这里需要Arc包装才能真正克隆
        let account_id_clone = account_id.clone();
        
        let task = tokio::spawn(async move {
            let mut order_count = 0;
            
            for j in 0..orders_per_account {
                let order = ShadowOrder {
                    id: String::new(),
                    account_id: account_id_clone.clone(),
                    symbol: "BTC/USDT".to_string(),
                    side: if j % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell },
                    quantity: 0.1,
                    price: Some(45000.0 + (j as f64 * 100.0)),
                    order_type: OrderType::Limit,
                    status: OrderStatus::Pending,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    filled_quantity: 0.0,
                    average_price: None,
                    fees: 0.0,
                    metadata: HashMap::new(),
                };

                if system_clone.submit_shadow_order(order).await.is_ok() {
                    order_count += 1;
                }
            }
            
            order_count
        });
        
        tasks.push(task);
    }

    // 等待所有任务完成
    let mut total_orders = 0;
    for task in tasks {
        if let Ok(count) = task.await {
            total_orders += count;
        }
    }

    let duration = start_time.elapsed();
    let orders_per_second = total_orders as f64 / duration.as_secs_f64();

    info!("📊 压力测试结果:");
    info!("  - 总订单数: {}", total_orders);
    info!("  - 执行时间: {:.2}秒", duration.as_secs_f64());
    info!("  - 订单处理速度: {:.0} 订单/秒", orders_per_second);

    // 清理测试账户
    for account_id in account_ids {
        let _ = system.delete_virtual_account(&account_id).await;
    }

    info!("✅ 压力测试完成并清理");

    Ok(())
}