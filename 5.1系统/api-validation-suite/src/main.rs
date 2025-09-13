use clap::Parser;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

mod tests;
mod utils;
mod validators;
mod reports;

use tests::*;
use utils::*;
use reports::*;

#[derive(Parser, Debug)]
#[command(name = "api-validator")]
#[command(about = "严谨的387个API接口完整性验证测试套件")]
pub struct Args {
    /// 测试基础URL
    #[arg(short, long, default_value = "http://localhost:3000")]
    base_url: String,

    /// 并发测试线程数
    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,

    /// 每个API的测试重复次数
    #[arg(short, long, default_value_t = 3)]
    repeats: usize,

    /// 是否运行性能测试
    #[arg(long)]
    performance: bool,

    /// 是否运行端到端测试
    #[arg(long)]
    e2e: bool,

    /// 测试超时时间(秒)
    #[arg(short, long, default_value_t = 30)]
    timeout: u64,

    /// 输出报告格式 (json, html, text)
    #[arg(long, default_value = "html")]
    report_format: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("api_validation_suite=debug")
        .init();

    let args = Args::parse();
    
    info!("🚀 启动387个API接口严谨验证测试");
    info!("📋 测试配置:");
    info!("   基础URL: {}", args.base_url);
    info!("   并发数: {}", args.concurrency);
    info!("   重复次数: {}", args.repeats);
    info!("   超时时间: {}秒", args.timeout);

    let start_time = Instant::now();
    
    // 创建测试配置
    let config = TestConfig {
        base_url: args.base_url.clone(),
        timeout: Duration::from_secs(args.timeout),
        concurrency: args.concurrency,
        repeats: args.repeats,
    };

    // 创建测试执行器
    let executor = TestExecutor::new(config).await?;
    
    // 执行完整的387个API测试
    let mut test_results = TestResults::new();

    // 1. 基础连通性测试
    info!("🔗 阶段1: 基础连通性测试");
    let connectivity_results = executor.test_connectivity().await;
    test_results.add_results("connectivity".to_string(), connectivity_results);

    // 2. 所有387个API完整性测试
    info!("🧪 阶段2: 387个API完整性测试");
    let api_results = executor.test_all_387_apis().await;
    test_results.add_results("api_completeness".to_string(), api_results);

    // 3. 数据传输完整性测试
    info!("📦 阶段3: 数据传输完整性测试");
    let data_results = executor.test_data_integrity().await;
    test_results.add_results("data_integrity".to_string(), data_results);

    // 4. 系统控制能力测试
    info!("🎮 阶段4: 系统控制能力测试");
    let control_results = executor.test_system_control().await;
    test_results.add_results("system_control".to_string(), control_results);

    // 5. 端到端业务流程测试
    if args.e2e {
        info!("🔄 阶段5: 端到端业务流程测试");
        let e2e_results = executor.test_end_to_end_workflows().await;
        test_results.add_results("e2e_workflows".to_string(), e2e_results);
    }

    // 6. 性能和并发测试
    if args.performance {
        info!("⚡ 阶段6: 性能和并发测试");
        let perf_results = executor.test_performance_and_concurrency().await;
        test_results.add_results("performance".to_string(), perf_results);
    }

    let total_duration = start_time.elapsed();

    // 生成详细测试报告
    info!("📊 生成测试报告...");
    let report_generator = ReportGenerator::new();
    let report = report_generator.generate_comprehensive_report(&test_results, total_duration).await;

    // 输出报告
    match args.report_format.as_str() {
        "json" => {
            tokio::fs::write("reports/api_validation_report.json", 
                serde_json::to_string_pretty(&report)?).await?;
            info!("📄 JSON报告已保存到: reports/api_validation_report.json");
        },
        "html" => {
            let html_report = report_generator.generate_html_report(&report).await;
            tokio::fs::create_dir_all("reports").await?;
            tokio::fs::write("reports/api_validation_report.html", html_report).await?;
            info!("📄 HTML报告已保存到: reports/api_validation_report.html");
        },
        _ => {
            println!("{}", report_generator.generate_text_report(&report));
        }
    }

    // 输出测试结果摘要
    print_test_summary(&test_results);

    // 返回退出代码
    if test_results.has_failures() {
        error!("❌ 测试失败，某些API接口存在问题");
        std::process::exit(1);
    } else {
        info!("✅ 所有387个API接口测试通过！");
        std::process::exit(0);
    }
}

fn print_test_summary(results: &TestResults) {
    println!("\n🎯 387个API接口验证测试总结");
    println!("══════════════════════════════════════════");
    
    let summary = results.get_summary();
    println!("📊 总体统计:");
    println!("   总测试数: {}", summary.total_tests);
    println!("   通过数量: {}", summary.passed);
    println!("   失败数量: {}", summary.failed);
    println!("   成功率: {:.2}%", summary.success_rate * 100.0);
    println!("   平均响应时间: {:?}", summary.avg_response_time);
    
    println!("\n🔍 分类结果:");
    for (category, stats) in summary.category_stats.iter() {
        println!("   {}: {}/{} ({:.1}%)", 
                 category, stats.passed, stats.total, 
                 stats.passed as f64 / stats.total as f64 * 100.0);
    }

    if summary.failed > 0 {
        println!("\n⚠️ 失败的API接口:");
        for failure in results.get_failures() {
            println!("   - {} ({}): {}", failure.api_name, failure.category, failure.error_message.as_ref().unwrap_or(&"Unknown error".to_string()));
        }
    }

    println!("\n🚀 系统控制能力评估:");
    let control_score = results.calculate_control_score();
    println!("   控制完整性得分: {:.1}/100", control_score);
    
    if control_score >= 95.0 {
        println!("   评级: 优秀 - 100%控制系统 ✅");
    } else if control_score >= 80.0 {
        println!("   评级: 良好 - 基本控制系统 ⚠️");
    } else {
        println!("   评级: 需改进 - 控制能力不足 ❌");
    }
    
    println!("══════════════════════════════════════════");
}