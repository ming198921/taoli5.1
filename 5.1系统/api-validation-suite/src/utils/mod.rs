use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tracing::{info, warn, error};

/// 工具函数模块

/// 格式化持续时间为人可读格式
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let millis = duration.subsec_millis();

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else if seconds > 0 {
        format!("{}.{}s", seconds, millis / 100)
    } else {
        format!("{}ms", duration.as_millis())
    }
}

/// 计算成功率百分比
pub fn calculate_success_rate(successful: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (successful as f64 / total as f64) * 100.0
    }
}

/// 生成唯一ID
pub fn generate_test_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    format!("test_{}", timestamp)
}

/// 验证URL格式
pub fn validate_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://")
}

/// 计算响应时间百分位数
pub fn calculate_percentile(mut response_times: Vec<Duration>, percentile: f64) -> Duration {
    if response_times.is_empty() {
        return Duration::from_millis(0);
    }

    response_times.sort();
    let index = ((percentile / 100.0) * (response_times.len() - 1) as f64).round() as usize;
    response_times[index.min(response_times.len() - 1)]
}

/// 性能等级评估
pub fn assess_performance_grade(avg_response_time: Duration) -> String {
    let millis = avg_response_time.as_millis();
    
    if millis <= 100 {
        "A+".to_string()
    } else if millis <= 300 {
        "A".to_string()
    } else if millis <= 500 {
        "B".to_string()
    } else if millis <= 1000 {
        "C".to_string()
    } else if millis <= 2000 {
        "D".to_string()
    } else {
        "F".to_string()
    }
}

/// 控制能力等级评估
pub fn assess_control_grade(control_score: f64) -> String {
    if control_score >= 95.0 {
        "优秀".to_string()
    } else if control_score >= 85.0 {
        "良好".to_string()
    } else if control_score >= 75.0 {
        "合格".to_string()
    } else if control_score >= 60.0 {
        "需改进".to_string()
    } else {
        "不合格".to_string()
    }
}

/// 数据完整性等级评估
pub fn assess_data_integrity_grade(integrity_score: f64) -> String {
    if integrity_score >= 98.0 {
        "完美".to_string()
    } else if integrity_score >= 95.0 {
        "优秀".to_string()
    } else if integrity_score >= 90.0 {
        "良好".to_string()
    } else if integrity_score >= 80.0 {
        "合格".to_string()
    } else {
        "不合格".to_string()
    }
}

/// 网络连接测试工具
pub async fn test_network_connectivity(url: &str) -> bool {
    match reqwest::get(url).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// 内存使用统计
pub struct MemoryStats {
    pub used: f64,
    pub available: f64,
    pub usage_percent: f64,
}

/// 获取系统内存统计（模拟）
pub fn get_memory_stats() -> MemoryStats {
    // 在实际实现中，这里应该调用系统API获取真实内存信息
    MemoryStats {
        used: 4.2,      // GB
        available: 8.0, // GB
        usage_percent: 52.5,
    }
}

/// CPU使用率统计
pub struct CpuStats {
    pub usage_percent: f64,
    pub load_average: f64,
}

/// 获取CPU使用率统计（模拟）
pub fn get_cpu_stats() -> CpuStats {
    // 在实际实现中，这里应该调用系统API获取真实CPU信息
    CpuStats {
        usage_percent: 68.3,
        load_average: 1.25,
    }
}

/// 生成测试报告摘要
pub fn generate_test_summary_text(
    total_tests: usize,
    passed: usize,
    failed: usize,
    success_rate: f64,
    avg_response_time: Duration,
    control_score: f64,
) -> String {
    format!(
        r#"
🎯 测试结果摘要
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 总体统计
  • 总测试数: {}
  • 通过测试: {} 
  • 失败测试: {}
  • 成功率: {:.2}%

⏱️ 性能指标
  • 平均响应时间: {}
  • 性能等级: {}

🎮 控制能力
  • 控制评分: {:.1}/100
  • 控制等级: {}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
        "#,
        total_tests,
        passed,
        failed,
        success_rate * 100.0,
        format_duration(avg_response_time),
        assess_performance_grade(avg_response_time),
        control_score,
        assess_control_grade(control_score)
    )
}

/// API分类统计
#[derive(Debug, Clone)]
pub struct ApiCategoryStats {
    pub category: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub avg_response_time: Duration,
}

/// 计算API分类统计
pub fn calculate_category_stats(
    category: &str,
    results: &[crate::tests::TestResult],
) -> ApiCategoryStats {
    let category_results: Vec<_> = results
        .iter()
        .filter(|r| r.category == category)
        .collect();

    let total = category_results.len();
    let passed = category_results.iter().filter(|r| r.success).count();
    let failed = total - passed;
    let success_rate = if total > 0 { passed as f64 / total as f64 } else { 0.0 };

    let avg_response_time = if !category_results.is_empty() {
        let total_time: Duration = category_results.iter().map(|r| r.response_time).sum();
        total_time / category_results.len() as u32
    } else {
        Duration::from_millis(0)
    };

    ApiCategoryStats {
        category: category.to_string(),
        total,
        passed,
        failed,
        success_rate,
        avg_response_time,
    }
}

/// 错误分类统计
pub fn analyze_error_patterns(results: &[crate::tests::TestResult]) -> HashMap<String, usize> {
    let mut error_counts = HashMap::new();

    for result in results {
        if !result.success {
            if let Some(error_msg) = &result.error_message {
                // 简单的错误分类
                let error_type = if error_msg.contains("连接") {
                    "连接错误"
                } else if error_msg.contains("超时") {
                    "超时错误"
                } else if error_msg.contains("HTTP") {
                    "HTTP错误"
                } else if error_msg.contains("格式") {
                    "数据格式错误"
                } else {
                    "其他错误"
                };
                
                *error_counts.entry(error_type.to_string()).or_insert(0) += 1;
            }
        }
    }

    error_counts
}

/// 生成进度条
pub fn generate_progress_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64) as usize;
    let empty = width - filled;
    
    format!(
        "[{}{}] {:.1}%",
        "█".repeat(filled),
        "░".repeat(empty),
        percentage
    )
}

/// 颜色输出工具（用于终端显示）
pub struct ColorText;

impl ColorText {
    pub fn green(text: &str) -> String {
        format!("\x1b[32m{}\x1b[0m", text)
    }
    
    pub fn red(text: &str) -> String {
        format!("\x1b[31m{}\x1b[0m", text)
    }
    
    pub fn yellow(text: &str) -> String {
        format!("\x1b[33m{}\x1b[0m", text)
    }
    
    pub fn blue(text: &str) -> String {
        format!("\x1b[34m{}\x1b[0m", text)
    }
    
    pub fn bold(text: &str) -> String {
        format!("\x1b[1m{}\x1b[0m", text)
    }
}

/// 测试环境信息
#[derive(Debug, Clone)]
pub struct TestEnvironmentInfo {
    pub os: String,
    pub architecture: String,
    pub rust_version: String,
    pub memory_total: String,
    pub cpu_count: usize,
}

/// 获取测试环境信息
pub fn get_test_environment_info() -> TestEnvironmentInfo {
    TestEnvironmentInfo {
        os: std::env::consts::OS.to_string(),
        architecture: std::env::consts::ARCH.to_string(),
        rust_version: "1.75.0".to_string(), // 应该从实际环境获取
        memory_total: "8GB".to_string(), // 应该从系统获取
        cpu_count: 4, // 应该从系统获取
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs(5)), "5.0s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5s");
    }

    #[test]
    fn test_calculate_success_rate() {
        assert_eq!(calculate_success_rate(8, 10), 80.0);
        assert_eq!(calculate_success_rate(0, 0), 0.0);
        assert_eq!(calculate_success_rate(10, 10), 100.0);
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("http://localhost:3000"));
        assert!(validate_url("https://api.example.com"));
        assert!(!validate_url("ftp://example.com"));
        assert!(!validate_url("not_a_url"));
    }

    #[test]
    fn test_assess_performance_grade() {
        assert_eq!(assess_performance_grade(Duration::from_millis(50)), "A+");
        assert_eq!(assess_performance_grade(Duration::from_millis(200)), "A");
        assert_eq!(assess_performance_grade(Duration::from_millis(800)), "B");
        assert_eq!(assess_performance_grade(Duration::from_secs(3)), "F");
    }

    #[test]
    fn test_assess_control_grade() {
        assert_eq!(assess_control_grade(98.0), "优秀");
        assert_eq!(assess_control_grade(88.0), "良好");
        assert_eq!(assess_control_grade(78.0), "合格");
        assert_eq!(assess_control_grade(50.0), "不合格");
    }

    #[test]
    fn test_generate_progress_bar() {
        assert_eq!(generate_progress_bar(50.0, 10), "[█████░░░░░] 50.0%");
        assert_eq!(generate_progress_bar(100.0, 5), "[█████] 100.0%");
        assert_eq!(generate_progress_bar(0.0, 5), "[░░░░░] 0.0%");
    }
}