//! 5.1套利系统前后端数据互通全面测试
//! 
//! 此测试框架将验证前后端所有数据结构的完整互通性
//! 确保100%的数据兼容性，不遗漏任何字段或类型

use anyhow::{Result, Context};
use colored::*;
use serde::{Serialize, Deserialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use common_types::*;
use comprehensive_frontend_backend_test::*;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("{}", "🚀 5.1套利系统前后端数据互通全面测试开始".bright_green().bold());
    println!("{}", "=" .repeat(80).bright_blue());
    
    let mut test_results = TestResults::new();
    
    // 1. 核心数据结构兼容性测试
    println!("\n{}", "📊 1. 核心数据结构兼容性测试".bright_yellow().bold());
    test_arbitrage_opportunity_compatibility(&mut test_results).await?;
    test_api_response_compatibility(&mut test_results).await?;
    test_system_status_compatibility(&mut test_results).await?;
    test_market_data_compatibility(&mut test_results).await?;
    test_risk_alert_compatibility(&mut test_results).await?;
    
    // 2. API端点数据序列化测试
    println!("\n{}", "🌐 2. API端点数据序列化测试".bright_yellow().bold());
    test_all_api_endpoints(&mut test_results).await?;
    
    // 3. WebSocket实时数据流测试
    println!("\n{}", "📡 3. WebSocket实时数据流测试".bright_yellow().bold());
    test_websocket_data_streams(&mut test_results).await?;
    
    // 4. 策略模块数据兼容性测试
    println!("\n{}", "⚙️ 4. 策略模块数据兼容性测试".bright_yellow().bold());
    test_strategy_module_compatibility(&mut test_results).await?;
    
    // 5. 架构模块数据兼容性测试
    println!("\n{}", "🏗️ 5. 架构模块数据兼容性测试".bright_yellow().bold());
    test_architecture_module_compatibility(&mut test_results).await?;
    
    // 6. 完整的端到端数据流测试
    println!("\n{}", "🔄 6. 端到端数据流测试".bright_yellow().bold());
    test_end_to_end_data_flow(&mut test_results).await?;
    
    // 7. 生成完整测试报告
    println!("\n{}", "📋 7. 生成完整测试报告".bright_yellow().bold());
    generate_comprehensive_report(&test_results).await?;
    
    println!("\n{}", "=" .repeat(80).bright_blue());
    print_final_summary(&test_results);
    
    if test_results.has_failures() {
        std::process::exit(1);
    }
    
    Ok(())
}


fn print_final_summary(results: &TestResults) {
    println!("{}", "📈 测试结果总览".bright_cyan().bold());
    println!("总测试数: {}", results.total_tests.to_string().bright_white());
    println!("通过测试: {}", results.passed_tests.to_string().bright_green());
    println!("失败测试: {}", results.failed_tests.to_string().bright_red());
    println!("成功率: {:.2}%", results.success_rate().to_string().bright_yellow());
    
    println!("\n{}", "🔗 字段兼容性统计".bright_cyan().bold());
    println!("兼容字段: {}", results.compatible_fields.len().to_string().bright_green());
    println!("不兼容字段: {}", results.incompatible_fields.len().to_string().bright_red());
    
    if results.has_failures() {
        println!("\n{}", "⚠️ 存在数据互通问题，需要修复".bright_red().bold());
    } else {
        println!("\n{}", "🎉 所有测试通过，前后端100%数据互通！".bright_green().bold());
    }
}