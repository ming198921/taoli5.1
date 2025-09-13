use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use reqwest::Client;
use tracing::{info, warn, error, debug};

pub mod api_completeness;
pub mod data_integrity;
pub mod system_control;
pub mod e2e_workflows;
pub mod performance;

pub use api_completeness::*;
pub use data_integrity::*;
pub use system_control::*;
pub use e2e_workflows::*;
pub use performance::*;

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub concurrency: usize,
    pub repeats: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub api_name: String,
    pub category: String,
    pub method: String,
    pub endpoint: String,
    pub success: bool,
    pub response_time: Duration,
    pub status_code: Option<u16>,
    pub error_message: Option<String>,
    pub data_integrity_score: f64,
    pub control_capability_score: f64,
    pub response_size_bytes: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Default)]
pub struct TestResults {
    results: HashMap<String, Vec<TestResult>>,
    start_time: Option<Instant>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            start_time: Some(Instant::now()),
        }
    }

    pub fn add_results(&mut self, category: String, results: Vec<TestResult>) {
        self.results.insert(category, results);
    }

    pub fn get_summary(&self) -> TestSummary {
        let mut total_tests = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut total_response_time = Duration::ZERO;
        let mut category_stats = HashMap::new();

        for (category, results) in &self.results {
            let category_passed = results.iter().filter(|r| r.success).count();
            let category_total = results.len();
            
            category_stats.insert(category.clone(), CategoryStats {
                total: category_total,
                passed: category_passed,
            });

            total_tests += category_total;
            passed += category_passed;
            failed += category_total - category_passed;

            for result in results {
                total_response_time += result.response_time;
            }
        }

        TestSummary {
            total_tests,
            passed,
            failed,
            success_rate: if total_tests > 0 { passed as f64 / total_tests as f64 } else { 0.0 },
            avg_response_time: if total_tests > 0 { 
                total_response_time / total_tests as u32 
            } else { 
                Duration::ZERO 
            },
            category_stats,
        }
    }

    pub fn has_failures(&self) -> bool {
        self.results.values().any(|results| results.iter().any(|r| !r.success))
    }

    pub fn get_failures(&self) -> Vec<&TestResult> {
        self.results.values()
            .flatten()
            .filter(|r| !r.success)
            .collect()
    }

    pub fn calculate_control_score(&self) -> f64 {
        let all_results: Vec<&TestResult> = self.results.values().flatten().collect();
        
        if all_results.is_empty() {
            return 0.0;
        }

        let control_score = all_results.iter()
            .map(|r| r.control_capability_score)
            .sum::<f64>() / all_results.len() as f64;

        control_score
    }
}

#[derive(Debug)]
pub struct TestSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub success_rate: f64,
    pub avg_response_time: Duration,
    pub category_stats: HashMap<String, CategoryStats>,
}

#[derive(Debug)]
pub struct CategoryStats {
    pub total: usize,
    pub passed: usize,
}

#[derive(Clone)]
pub struct TestExecutor {
    config: TestConfig,
    client: Client,
}

impl TestExecutor {
    pub async fn new(config: TestConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()?;

        Ok(Self { config, client })
    }

    /// 测试基础连通性
    pub async fn test_connectivity(&self) -> Vec<TestResult> {
        info!("🔗 开始基础连通性测试...");
        
        let mut results = Vec::new();
        let services = vec![
            ("统一网关", "3000"),
            ("日志监控服务", "4001"),
            ("清洗配置服务", "4002"),
            ("策略监控服务", "4003"),
            ("性能调优服务", "4004"),
            ("交易监控服务", "4005"),
            ("AI模型服务", "4006"),
            ("配置管理服务", "4007"),
        ];

        for (service_name, port) in services {
            let url = format!("http://localhost:{}/health", port);
            let start = Instant::now();
            
            match self.client.get(&url).send().await {
                Ok(response) => {
                    let response_time = start.elapsed();
                    let status_code = response.status().as_u16();
                    let success = response.status().is_success();
                    
                    results.push(TestResult {
                        api_name: format!("{} Health Check", service_name),
                        category: "connectivity".to_string(),
                        method: "GET".to_string(),
                        endpoint: url,
                        success,
                        response_time,
                        status_code: Some(status_code),
                        error_message: if success { None } else { Some(format!("HTTP {}", status_code)) },
                        data_integrity_score: if success { 100.0 } else { 0.0 },
                        control_capability_score: if success { 100.0 } else { 0.0 },
                        response_size_bytes: 0,
                        timestamp: chrono::Utc::now(),
                    });
                    
                    if success {
                        info!("✅ {} 连通性测试通过 ({}ms)", service_name, response_time.as_millis());
                    } else {
                        warn!("⚠️ {} 连通性测试失败: HTTP {}", service_name, status_code);
                    }
                },
                Err(e) => {
                    let response_time = start.elapsed();
                    results.push(TestResult {
                        api_name: format!("{} Health Check", service_name),
                        category: "connectivity".to_string(),
                        method: "GET".to_string(),
                        endpoint: url,
                        success: false,
                        response_time,
                        status_code: None,
                        error_message: Some(e.to_string()),
                        data_integrity_score: 0.0,
                        control_capability_score: 0.0,
                        response_size_bytes: 0,
                        timestamp: chrono::Utc::now(),
                    });
                    
                    error!("❌ {} 连通性测试失败: {}", service_name, e);
                }
            }
        }

        results
    }

    /// 测试所有387个API接口
    pub async fn test_all_387_apis(&self) -> Vec<TestResult> {
        info!("🧪 开始387个API完整性测试...");
        
        let mut all_results = Vec::new();
        
        // 日志监控服务 - 45个API
        info!("📝 测试日志监控服务 (45个API)...");
        let logging_results = self.test_logging_apis().await;
        all_results.extend(logging_results);
        
        // 清洗配置服务 - 52个API
        info!("🧹 测试清洗配置服务 (52个API)...");
        let cleaning_results = self.test_cleaning_apis().await;
        all_results.extend(cleaning_results);
        
        // 策略监控服务 - 38个API
        info!("🎯 测试策略监控服务 (38个API)...");
        let strategy_results = self.test_strategy_apis().await;
        all_results.extend(strategy_results);
        
        // 性能调优服务 - 67个API
        info!("⚡ 测试性能调优服务 (67个API)...");
        let performance_results = self.test_performance_apis().await;
        all_results.extend(performance_results);
        
        // 交易监控服务 - 41个API
        info!("💹 测试交易监控服务 (41个API)...");
        let trading_results = self.test_trading_apis().await;
        all_results.extend(trading_results);
        
        // AI模型服务 - 48个API
        info!("🤖 测试AI模型服务 (48个API)...");
        let ai_results = self.test_ai_apis().await;
        all_results.extend(ai_results);
        
        // 配置管理服务 - 96个API
        info!("⚙️ 测试配置管理服务 (96个API)...");
        let config_results = self.test_config_apis().await;
        all_results.extend(config_results);

        info!("✅ 387个API测试完成，成功: {}, 失败: {}", 
              all_results.iter().filter(|r| r.success).count(),
              all_results.iter().filter(|r| !r.success).count());

        all_results
    }

    /// 测试数据传输完整性
    pub async fn test_data_integrity(&self) -> Vec<TestResult> {
        self.test_data_integrity_detailed().await
    }

    /// 测试系统控制能力
    pub async fn test_system_control(&self) -> Vec<TestResult> {
        self.test_system_control_detailed().await
    }

    /// 端到端业务流程测试
    pub async fn test_end_to_end_workflows(&self) -> Vec<TestResult> {
        self.test_end_to_end_workflows_detailed().await
    }

    /// 性能和并发测试
    pub async fn test_performance_and_concurrency(&self) -> Vec<TestResult> {
        self.test_performance_and_concurrency_detailed().await
    }
}