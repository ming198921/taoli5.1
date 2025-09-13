//! Runtime Enforcement System Demonstration
//! 
//! 展示如何使用运行时限制强制执行系统来监控和保护系统运行

use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, Level};

use architecture::{
    config::system_limits::{SystemLimits, SystemLimitsValidator},
    runtime_enforcement::{RuntimeEnforcer, EnforcementConfig, EnforcementAction},
};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Runtime Enforcement System Demo");

    // 创建系统限制配置
    let system_limits = SystemLimits {
        max_supported_exchanges: 3,     // 降低限制用于演示
        max_supported_symbols: 10,
        max_symbols_per_exchange: 5,
        max_concurrent_opportunities: 100,
        max_order_batch_size: 20,
        ..Default::default()
    };

    // 创建系统限制验证器
    let validator = Arc::new(SystemLimitsValidator::new(system_limits));

    // 创建强制执行配置
    let enforcement_config = EnforcementConfig {
        monitoring_interval_seconds: 5,  // 快速监控用于演示
        auto_enforcement_enabled: true,
        critical_violation_shutdown: true,
        high_risk_warning_threshold: 70.0,
        critical_risk_threshold: 85.0,
        ..Default::default()
    };

    // 创建运行时强制执行器
    let enforcer = RuntimeEnforcer::new(validator.clone(), enforcement_config);

    // 创建shutdown channel
    let (shutdown_tx, shutdown_rx) = mpsc::unbounded_channel::<EnforcementAction>();

    // 获取健康状态监听器
    let mut health_receiver = enforcer.get_health_receiver();

    // 启动强制执行器（在后台）
    let enforcer_handle = {
        let enforcer = enforcer.clone();
        tokio::spawn(async move {
            if let Err(e) = enforcer.start(shutdown_rx).await {
                eprintln!("Runtime enforcer error: {}", e);
            }
        })
    };

    // 启动健康状态监听器
    let health_monitor_handle = tokio::spawn(async move {
        while health_receiver.changed().await.is_ok() {
            let health = health_receiver.borrow().clone();
            info!(
                "System Health Update: {:?}, Risk: {:?}, Violations: {}",
                health.overall_health,
                health.risk_level,
                health.active_violations.len()
            );

            if health.emergency_stop_active {
                info!("EMERGENCY STOP DETECTED - Demo will exit");
                break;
            }
        }
    });

    // 模拟系统使用场景
    info!("=== Demo Scenario 1: Normal Operation ===");
    
    // 注册交易所（正常情况）
    let result = validator.register_exchange("binance").await?;
    assert!(result.is_valid);
    info!("✓ Registered binance exchange");

    let result = validator.register_exchange("okx").await?;
    assert!(result.is_valid);
    info!("✓ Registered okx exchange");

    // 添加一些交易对
    let result = validator.register_symbol("binance", "BTC/USDT").await?;
    assert!(result.is_valid);
    info!("✓ Registered BTC/USDT on binance");

    let result = validator.register_symbol("okx", "ETH/USDT").await?;
    assert!(result.is_valid);
    info!("✓ Registered ETH/USDT on okx");

    // 等待监控周期
    tokio::time::sleep(Duration::from_secs(6)).await;

    info!("=== Demo Scenario 2: Approaching Limits ===");
    
    // 尝试接近限制
    let result = validator.register_exchange("huobi").await?;
    assert!(result.is_valid);
    info!("✓ Registered huobi exchange (at limit)");

    // 添加更多交易对接近限制
    for symbol in ["ADA/USDT", "DOT/USDT", "SOL/USDT", "AVAX/USDT"] {
        let result = validator.register_symbol("binance", symbol).await?;
        if result.is_valid {
            info!("✓ Registered {} on binance", symbol);
        } else {
            info!("⚠ Failed to register {} on binance: {:?}", symbol, result.violations);
        }
    }

    tokio::time::sleep(Duration::from_secs(6)).await;

    info!("=== Demo Scenario 3: Limit Violations ===");
    
    // 尝试超出交易所限制
    let result = validator.register_exchange("bybit").await?;
    if !result.is_valid {
        info!("✗ Exchange limit exceeded: {:?}", result.violations);
    }

    // 尝试超出交易对限制
    for i in 1..15 {
        let symbol = format!("TOKEN{}/USDT", i);
        let result = validator.register_symbol("huobi", &symbol).await?;
        if !result.is_valid {
            info!("✗ Symbol limit exceeded for {}: {:?}", symbol, result.violations);
            break;
        } else {
            info!("✓ Registered {} on huobi", symbol);
        }
    }

    tokio::time::sleep(Duration::from_secs(6)).await;

    info!("=== Demo Scenario 4: System Status Report ===");
    
    // 获取系统状态
    let status = validator.get_system_status().await;
    info!("System Status:");
    info!("  Exchanges: {}/{}", status.current_exchange_count, status.limits.max_supported_exchanges);
    info!("  Symbols: {}/{}", status.current_symbol_count, status.limits.max_supported_symbols);
    info!("  Compliance: {:.1}%", status.compliance_status.overall_compliance_percent);
    info!("  Risk Level: {:?}", status.compliance_status.risk_level);
    info!("  Recent Violations: {}", status.recent_violations.len());

    // 获取强制执行统计
    let stats = enforcer.get_enforcement_stats().await;
    info!("Enforcement Stats:");
    info!("  Total Violations: {}", stats.total_violations_detected);
    info!("  Warnings Issued: {}", stats.warnings_issued);
    info!("  Throttle Actions: {}", stats.throttle_actions_taken);
    info!("  Emergency Shutdowns: {}", stats.emergency_shutdowns_triggered);

    info!("=== Demo Scenario 5: Testing Enforcement Actions ===");
    
    // 模拟并发操作限制测试
    let result = validator.validate_concurrent_opportunities(150).await?;
    if result.is_valid {
        info!("✓ Concurrent opportunities validation passed");
    } else {
        info!("⚠ Concurrent opportunities validation failed: {:?}", result.violations);
    }

    // 模拟订单批次大小测试
    let result = validator.validate_order_batch_size(25).await?;
    if result.is_valid {
        info!("✓ Order batch size validation passed");
    } else {
        info!("⚠ Order batch size validation failed: {:?}", result.violations);
    }

    info!("=== Demo Scenario 6: Cleanup and Shutdown ===");
    
    // 清理资源
    validator.unregister_exchange("huobi").await?;
    info!("✓ Unregistered huobi exchange");

    // 等待最后的监控周期
    tokio::time::sleep(Duration::from_secs(6)).await;

    // 停止强制执行器
    enforcer.stop().await?;
    info!("✓ Runtime enforcer stopped");

    // 取消任务
    enforcer_handle.abort();
    health_monitor_handle.abort();

    info!("Runtime Enforcement System Demo completed successfully!");

    Ok(())
}

/// 辅助函数：模拟系统压力测试
#[allow(dead_code)]
async fn simulate_stress_test(
    validator: Arc<SystemLimitsValidator>,
    enforcer: &RuntimeEnforcer,
) -> Result<()> {
    info!("=== Stress Test: Rapid Registration ===");
    
    // 快速注册多个资源以触发限制
    for i in 1..30 {
        let exchange = format!("exchange_{}", i);
        let result = validator.register_exchange(&exchange).await?;
        
        if !result.is_valid {
            info!("Stress test triggered violation at exchange {}: {:?}", i, result.violations);
            break;
        }
        
        // 为每个交易所添加交易对
        for j in 1..10 {
            let symbol = format!("TOKEN{}/USDT", j);
            let result = validator.register_symbol(&exchange, &symbol).await?;
            
            if !result.is_valid {
                info!("Symbol registration failed: {:?}", result.violations);
                break;
            }
        }
        
        // 检查是否触发紧急停机
        if enforcer.is_emergency_stopped().await {
            info!("Emergency stop triggered during stress test");
            break;
        }
        
        // 短暂暂停
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Ok(())
}