//! QingXi 5.1 安全错误处理迁移工具
//! 批量替换现有代码中的unsafe unwrap()调用

use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use anyhow::Result;

/// 错误处理迁移工具
pub struct ErrorHandlingMigrator;

impl ErrorHandlingMigrator {
    /// 扫描并修复所有Rust文件中的unwrap()调用
    pub fn migrate_unwrap_calls(workspace_path: &str) -> Result<MigrationReport> {
        let mut report = MigrationReport::new();
        
        // 编译正则表达式
        let unwrap_regex = Regex::new(r"\.unwrap\(\)")?;
        let expect_regex = Regex::new(r"\.expect\([^)]+\)")?;
        
        // 遍历所有Rust文件
        for entry in WalkDir::new(workspace_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let path = entry.path();
            let content = fs::read_to_string(path)?;
            
            // 扫描unwrap调用
            let unwrap_count = unwrap_regex.find_iter(&content).count();
            let expect_count = expect_regex.find_iter(&content).count();
            
            if unwrap_count > 0 || expect_count > 0 {
                report.add_file_issues(
                    path.to_string_lossy().to_string(),
                    unwrap_count,
                    expect_count
                );
            }
        }
        
        Ok(report)
    }

    /// 生成安全替换建议
    pub fn generate_safe_replacements() -> Vec<SafeReplacementPattern> {
        vec![
            SafeReplacementPattern {
                unsafe_pattern: r"\.unwrap\(\)".to_string(),
                safe_pattern: r"SafeWrapper::safe_unwrap_option(#EXPR#, #DEFAULT#, "#CONTEXT#")".to_string(),
                description: "Replace Option.unwrap() with SafeWrapper".to_string(),
                example_before: r"some_option.unwrap()".to_string(),
                example_after: r#"SafeWrapper::safe_unwrap_option(some_option, default_value, "context")"#.to_string(),
            },
            SafeReplacementPattern {
                unsafe_pattern: r"result\.unwrap\(\)".to_string(),
                safe_pattern: r"SafeWrapper::safe_unwrap_result(result, \"#CONTEXT#\")?".to_string(),
                description: "Replace Result.unwrap() with SafeWrapper and error propagation".to_string(),
                example_before: r"result.unwrap()".to_string(),
                example_after: r#"SafeWrapper::safe_unwrap_result(result, "context")?"#.to_string(),
            },
            SafeReplacementPattern {
                unsafe_pattern: r"orderbook\.bids\.first\(\)\.unwrap\(\)".to_string(),
                safe_pattern: r"SafeWrapper::safe_best_bid(&orderbook).ok_or(anyhow::anyhow!(\"No bids available\"))?".to_string(),
                description: "Replace orderbook first bid access".to_string(),
                example_before: r"orderbook.bids.first().unwrap()".to_string(),
                example_after: r#"SafeWrapper::safe_best_bid(&orderbook).ok_or(anyhow::anyhow!("No bids available"))?"#.to_string(),
            },
            SafeReplacementPattern {
                unsafe_pattern: r"orderbook\.asks\.first\(\)\.unwrap\(\)".to_string(),
                safe_pattern: r"SafeWrapper::safe_best_ask(&orderbook).ok_or(anyhow::anyhow!(\"No asks available\"))?".to_string(),
                description: "Replace orderbook first ask access".to_string(),
                example_before: r"orderbook.asks.first().unwrap()".to_string(),
                example_after: r#"SafeWrapper::safe_best_ask(&orderbook).ok_or(anyhow::anyhow!("No asks available"))?"#.to_string(),
            },
            SafeReplacementPattern {
                unsafe_pattern: r"config\.([a-zA-Z_]+)\.unwrap\(\)".to_string(),
                safe_pattern: r"safe_config!(config, $1, default_value)".to_string(),
                description: "Replace config field unwrap with safe_config macro".to_string(),
                example_before: r"config.timeout.unwrap()".to_string(),
                example_after: r"safe_config!(config, timeout, 5000)".to_string(),
            },
        ]
    }
}

/// 迁移报告
#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub files_with_issues: Vec<FileIssues>,
    pub total_unwrap_calls: usize,
    pub total_expect_calls: usize,
    pub total_files: usize,
}

/// 文件问题统计
#[derive(Debug, Clone)]
pub struct FileIssues {
    pub file_path: String,
    pub unwrap_count: usize,
    pub expect_count: usize,
}

/// 安全替换模式
#[derive(Debug, Clone)]
pub struct SafeReplacementPattern {
    pub unsafe_pattern: String,
    pub safe_pattern: String,
    pub description: String,
    pub example_before: String,
    pub example_after: String,
}

impl MigrationReport {
    pub fn new() -> Self {
        Self {
            files_with_issues: Vec::new(),
            total_unwrap_calls: 0,
            total_expect_calls: 0,
            total_files: 0,
        }
    }

    pub fn add_file_issues(&mut self, file_path: String, unwrap_count: usize, expect_count: usize) {
        self.files_with_issues.push(FileIssues {
            file_path,
            unwrap_count,
            expect_count,
        });
        self.total_unwrap_calls += unwrap_count;
        self.total_expect_calls += expect_count;
        self.total_files += 1;
    }

    /// 打印迁移报告
    pub fn print_report(&self) {
        println!("🔍 QingXi 5.1 错误处理迁移报告");
        println!("================================");
        println!("📊 总计发现问题:");
        println!("  - 文件数量: {}", self.total_files);
        println!("  - unwrap() 调用: {}", self.total_unwrap_calls);
        println!("  - expect() 调用: {}", self.total_expect_calls);
        println!();

        if !self.files_with_issues.is_empty() {
            println!("📁 问题文件详情:");
            for file_issue in &self.files_with_issues {
                println!("  📄 {}", file_issue.file_path);
                if file_issue.unwrap_count > 0 {
                    println!("    🔴 unwrap(): {}", file_issue.unwrap_count);
                }
                if file_issue.expect_count > 0 {
                    println!("    🟡 expect(): {}", file_issue.expect_count);
                }
            }
        }

        println!();
        println!("🛠️ 建议的修复步骤:");
        println!("1. 使用 SafeWrapper::safe_unwrap_option() 替代 Option::unwrap()");
        println!("2. 使用 SafeWrapper::safe_unwrap_result() 替代 Result::unwrap()");
        println!("3. 使用 SafeWrapper::safe_best_bid/ask() 替代订单簿直接访问");
        println!("4. 使用 safe_config!() 宏替代配置字段的 unwrap()");
        println!("5. 在函数签名中添加 Result<T> 返回类型以支持错误传播");
    }

    /// 生成修复脚本
    pub fn generate_fix_script(&self) -> String {
        let mut script = String::new();
        script.push_str("#!/bin/bash\n");
        script.push_str("# QingXi 5.1 错误处理自动修复脚本\n");
        script.push_str("# 警告: 请在运行前备份代码!\n\n");

        script.push_str("echo \"🔧 开始修复 unwrap() 调用...\"\n\n");

        for file_issue in &self.files_with_issues {
            if file_issue.unwrap_count > 0 {
                script.push_str(&format!(
                    "echo \"修复文件: {}\"\n",
                    file_issue.file_path
                ));
                script.push_str(&format!(
                    "# 文件有 {} 个 unwrap() 调用需要手动修复\n",
                    file_issue.unwrap_count
                ));
            }
        }

        script.push_str("\necho \"✅ 修复完成! 请运行测试验证修复结果。\"\n");
        script
    }
}

/// 辅助函数：生成安全的错误处理模板
pub fn generate_safe_error_handling_template() -> &'static str {
    r#"
// QingXi 5.1 安全错误处理模板

use crate::error_handling::SafeWrapper;
use anyhow::{Context, Result};

// ✅ 好的做法 - 使用 SafeWrapper
fn safe_example() -> Result<f64> {
    let config = get_config()?;
    
    // 安全的 Option 处理
    let timeout = SafeWrapper::safe_unwrap_option(
        config.timeout, 
        5000, // 默认值
        "config timeout"
    );
    
    // 安全的 Result 处理
    let price_str = "123.45";
    let price = SafeWrapper::safe_unwrap_result(
        price_str.parse::<f64>(),
        "price parsing"
    )?;
    
    // 安全的订单簿访问
    let orderbook = get_orderbook()?;
    let best_bid = SafeWrapper::safe_best_bid(&orderbook)
        .ok_or_else(|| anyhow::anyhow!("No bids available"))?;
    
    Ok(best_bid)
}

// ❌ 避免的做法 - 直接 unwrap
fn unsafe_example() {
    let config = get_config().unwrap(); // 可能panic!
    let timeout = config.timeout.unwrap(); // 可能panic!
    let price = "123.45".parse::<f64>().unwrap(); // 可能panic!
    let best_bid = orderbook.bids.first().unwrap().price; // 可能panic!
}
"#
}

