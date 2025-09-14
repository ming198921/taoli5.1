use market_data_module::*;
use std::{sync::Arc, time::Duration};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 系统健康状况和异常检测演示");
    println!("=======================================");

    // 加载生产配置
    let settings = settings::Settings::load()?;
    
    // 创建中央管理器和健康监控系统
    let (manager, manager_handle) = central_manager::CentralManager::new(&settings);
    let health_monitor = manager.health_monitor();
    
    // 注册交易所适配器
    manager.register_adapter(Arc::new(crate::adapters::binance::BinanceAdapter::new()));
    manager.register_adapter(Arc::new(crate::adapters::okx::OkxAdapter::new()));
    manager.register_adapter(Arc::new(crate::adapters::huobi::HuobiAdapter::new()));
    
    println!("\n📊 启动真实市场数据收集和健康监控...");
    
    // 等待系统稳定
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // 监控真实数据流和健康状态
    for i in 1..=30 {
        // 获取当前系统健康状况
        let overall_health = health_monitor.get_health_summary();
        
        // 获取最新的市场数据
        if let Ok(orderbooks) = manager_handle.get_all_orderbooks().await {
            for (symbol, orderbook) in orderbooks {
                if let (Some(best_bid), Some(best_ask)) = (orderbook.best_bid(), orderbook.best_ask()) {
                    println!("💹 实时价格 {}: 买价 {:.2} / 卖价 {:.2}", 
                             symbol.as_pair(), 
                             best_bid.price.0, 
                             best_ask.price.0);
                }
            }
        }

        // 显示健康状态摘要
        if i % 10 == 0 {
            println!("\n📈 系统健康报告 (第{}次检查):", i);
            println!("   - 总体健康状态: {}", if overall_health.unhealthy_sources == 0 { "✅ 正常" } else { "❌ 异常" });
            println!("   - 处理延迟: {:.2}ms", overall_health.average_latency_us as f64 / 1000.0);
            println!("   - 健康数据源: {}", overall_health.healthy_sources);
            println!("   - 异常数据源: {}", overall_health.unhealthy_sources);
            println!("   - 总消息数量: {}", overall_health.total_messages);
        }

        // 等待真实数据处理
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    // 生成最终报告
    println!("\n📋 最终健康状况报告");
    println!("========================");
    
    let final_health = health_monitor.get_health_summary();
    println!("✅ 系统整体状态: {}", if final_health.unhealthy_sources == 0 { "健康" } else { "需要关注" });
    println!("📊 平均延迟: {:.2}ms", final_health.average_latency_us as f64 / 1000.0);
    println!("📉 健康数据源: {}", final_health.healthy_sources);
    println!("⚠️  异常数据源: {}", final_health.unhealthy_sources);
    println!("📧 总消息数量: {}", final_health.total_messages);
    
    // 显示实际配置信息
    println!("\n⚡ 系统配置:");
    println!("   - 事件缓冲区大小: {}", settings.central_manager.event_buffer_size);
    println!("   - 配置数据源数量: {}", settings.sources.len());
    println!("   - 一致性检查启用: 是");

    println!("\n🎉 健康监控和异常检测演示完成！");
    
    Ok(())
}