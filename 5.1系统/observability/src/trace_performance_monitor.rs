//! 追踪性能监控器
//!
//! 监控追踪系统本身的性能，确保追踪不会成为系统瓶颈

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::cross_service_propagation::PropagationContext;

/// 追踪性能监控器
pub struct TracePerformanceMonitor {
    /// 监控配置
    config: MonitorConfig,
    /// 性能指标
    metrics: Arc<TraceMetrics>,
    /// 最近的性能样本
    recent_samples: Arc<RwLock<Vec<PerformanceSample>>>,
    /// 性能报告生成器
    reporter: PerformanceReporter,
}

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// 是否启用性能监控
    pub enabled: bool,
    /// 采样率 (0.0 - 1.0)
    pub sampling_rate: f64,
    /// 样本保留时间（秒）
    pub sample_retention_seconds: u64,
    /// 最大样本数量
    pub max_samples: usize,
    /// 性能报告间隔（秒）
    pub report_interval_seconds: u64,
    /// 警告阈值
    pub warning_thresholds: PerformanceThresholds,
}

/// 性能阈值
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// 头注入延迟警告阈值（微秒）
    pub header_injection_latency_micros: u64,
    /// 头提取延迟警告阈值（微秒）
    pub header_extraction_latency_micros: u64,
    /// 上下文序列化延迟警告阈值（微秒）
    pub context_serialization_latency_micros: u64,
    /// 内存使用警告阈值（字节）
    pub memory_usage_warning_bytes: usize,
}

/// 追踪系统性能指标
#[derive(Debug)]
pub struct TraceMetrics {
    // 操作计数
    pub header_injections: AtomicU64,
    pub header_extractions: AtomicU64,
    pub context_creations: AtomicU64,
    pub context_serializations: AtomicU64,
    
    // 延迟统计（微秒）
    pub total_injection_latency_micros: AtomicU64,
    pub total_extraction_latency_micros: AtomicU64,
    pub total_serialization_latency_micros: AtomicU64,
    
    // 错误统计
    pub injection_errors: AtomicU64,
    pub extraction_errors: AtomicU64,
    pub serialization_errors: AtomicU64,
    
    // 内存使用
    pub current_contexts_count: AtomicUsize,
    pub peak_contexts_count: AtomicUsize,
    pub estimated_memory_usage_bytes: AtomicUsize,
    
    // 系统启动时间
    pub start_time: Instant,
}

/// 性能样本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSample {
    pub timestamp: DateTime<Utc>,
    pub operation: String,
    pub latency_micros: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub context_depth: u32,
    pub payload_size_bytes: usize,
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: u64,
    pub summary: PerformanceSummary,
    pub warnings: Vec<PerformanceWarning>,
    pub recommendations: Vec<String>,
}

/// 性能摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    // 操作统计
    pub total_operations: u64,
    pub operations_per_second: f64,
    pub error_rate_percent: f64,
    
    // 延迟统计
    pub avg_injection_latency_micros: f64,
    pub avg_extraction_latency_micros: f64,
    pub p95_injection_latency_micros: u64,
    pub p95_extraction_latency_micros: u64,
    
    // 内存统计
    pub current_memory_usage_mb: f64,
    pub peak_memory_usage_mb: f64,
    pub context_count: usize,
}

/// 性能警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWarning {
    pub warning_type: WarningType,
    pub message: String,
    pub current_value: f64,
    pub threshold_value: f64,
    pub severity: WarningSeverity,
}

/// 警告类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningType {
    HighLatency,
    HighErrorRate,
    MemoryUsage,
    ContextLeakage,
    ThroughputDegradation,
}

/// 警告严重性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 性能报告生成器
#[derive(Debug)]
pub struct PerformanceReporter {
    last_report_time: Arc<RwLock<Instant>>,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 0.01, // 1% 采样
            sample_retention_seconds: 3600, // 1小时
            max_samples: 10000,
            report_interval_seconds: 300, // 5分钟
            warning_thresholds: PerformanceThresholds {
                header_injection_latency_micros: 1000, // 1ms
                header_extraction_latency_micros: 500,  // 0.5ms
                context_serialization_latency_micros: 2000, // 2ms
                memory_usage_warning_bytes: 100 * 1024 * 1024, // 100MB
            },
        }
    }
}

impl TraceMetrics {
    pub fn new() -> Self {
        Self {
            header_injections: AtomicU64::new(0),
            header_extractions: AtomicU64::new(0),
            context_creations: AtomicU64::new(0),
            context_serializations: AtomicU64::new(0),
            total_injection_latency_micros: AtomicU64::new(0),
            total_extraction_latency_micros: AtomicU64::new(0),
            total_serialization_latency_micros: AtomicU64::new(0),
            injection_errors: AtomicU64::new(0),
            extraction_errors: AtomicU64::new(0),
            serialization_errors: AtomicU64::new(0),
            current_contexts_count: AtomicUsize::new(0),
            peak_contexts_count: AtomicUsize::new(0),
            estimated_memory_usage_bytes: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }
}

impl TracePerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new(config: MonitorConfig) -> Self {
        let reporter = PerformanceReporter {
            last_report_time: Arc::new(RwLock::new(Instant::now())),
        };
        
        Self {
            config,
            metrics: Arc::new(TraceMetrics::new()),
            recent_samples: Arc::new(RwLock::new(Vec::new())),
            reporter,
        }
    }
    
    /// 记录头注入性能
    pub async fn record_header_injection(&self, latency: Duration, success: bool, context: &PropagationContext) {
        if !self.config.enabled || !self.should_sample() {
            return;
        }
        
        let latency_micros = latency.as_micros() as u64;
        
        // 更新指标
        self.metrics.header_injections.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_injection_latency_micros.fetch_add(latency_micros, Ordering::Relaxed);
        
        if !success {
            self.metrics.injection_errors.fetch_add(1, Ordering::Relaxed);
        }
        
        // 记录样本
        let sample = PerformanceSample {
            timestamp: Utc::now(),
            operation: "header_injection".to_string(),
            latency_micros,
            success,
            error_message: None,
            context_depth: context.depth,
            payload_size_bytes: self.estimate_context_size(context),
        };
        
        self.add_sample(sample).await;
        
        // 检查是否超过阈值
        if latency_micros > self.config.warning_thresholds.header_injection_latency_micros {
            warn!(
                latency_micros = latency_micros,
                threshold = self.config.warning_thresholds.header_injection_latency_micros,
                trace_id = %context.trace_id,
                "Header injection latency exceeds threshold"
            );
        }
    }
    
    /// 记录头提取性能
    pub async fn record_header_extraction(&self, latency: Duration, success: bool, error_msg: Option<String>) {
        if !self.config.enabled || !self.should_sample() {
            return;
        }
        
        let latency_micros = latency.as_micros() as u64;
        
        // 更新指标
        self.metrics.header_extractions.fetch_add(1, Ordering::Relaxed);
        self.metrics.total_extraction_latency_micros.fetch_add(latency_micros, Ordering::Relaxed);
        
        if !success {
            self.metrics.extraction_errors.fetch_add(1, Ordering::Relaxed);
        }
        
        // 记录样本
        let sample = PerformanceSample {
            timestamp: Utc::now(),
            operation: "header_extraction".to_string(),
            latency_micros,
            success,
            error_message: error_msg,
            context_depth: 0, // 未知深度
            payload_size_bytes: 0, // 未知大小
        };
        
        self.add_sample(sample).await;
        
        // 检查是否超过阈值
        if latency_micros > self.config.warning_thresholds.header_extraction_latency_micros {
            warn!(
                latency_micros = latency_micros,
                threshold = self.config.warning_thresholds.header_extraction_latency_micros,
                "Header extraction latency exceeds threshold"
            );
        }
    }
    
    /// 记录上下文创建
    pub async fn record_context_creation(&self, context: &PropagationContext) {
        if !self.config.enabled {
            return;
        }
        
        self.metrics.context_creations.fetch_add(1, Ordering::Relaxed);
        
        // 更新内存使用统计
        let current_count = self.metrics.current_contexts_count.fetch_add(1, Ordering::Relaxed) + 1;
        let peak_count = self.metrics.peak_contexts_count.load(Ordering::Relaxed);
        
        if current_count > peak_count {
            self.metrics.peak_contexts_count.store(current_count, Ordering::Relaxed);
        }
        
        // 估算内存使用
        let context_size = self.estimate_context_size(context);
        self.metrics.estimated_memory_usage_bytes.fetch_add(context_size, Ordering::Relaxed);
    }
    
    /// 记录上下文销毁
    pub async fn record_context_destruction(&self, context: &PropagationContext) {
        if !self.config.enabled {
            return;
        }
        
        self.metrics.current_contexts_count.fetch_sub(1, Ordering::Relaxed);
        
        // 减少内存使用估算
        let context_size = self.estimate_context_size(context);
        self.metrics.estimated_memory_usage_bytes.fetch_sub(context_size, Ordering::Relaxed);
    }
    
    /// 生成性能报告
    pub async fn generate_report(&self) -> Result<PerformanceReport> {
        let now = Instant::now();
        let report_duration = {
            let last_time = self.reporter.last_report_time.read().await;
            now.duration_since(*last_time)
        };
        
        // 更新最后报告时间
        {
            let mut last_time = self.reporter.last_report_time.write().await;
            *last_time = now;
        }
        
        let summary = self.calculate_summary(report_duration).await;
        let warnings = self.check_warnings(&summary).await;
        let recommendations = self.generate_recommendations(&summary, &warnings).await;
        
        let report = PerformanceReport {
            timestamp: Utc::now(),
            duration_seconds: report_duration.as_secs(),
            summary,
            warnings,
            recommendations,
        };
        
        info!(
            duration_seconds = report.duration_seconds,
            operations_per_second = report.summary.operations_per_second,
            error_rate = report.summary.error_rate_percent,
            warnings_count = report.warnings.len(),
            "Generated trace performance report"
        );
        
        Ok(report)
    }
    
    /// 启动性能监控任务
    pub fn start_monitoring_task(&self) -> tokio::task::JoinHandle<()> {
        let monitor = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(monitor.config.report_interval_seconds)
            );
            
            loop {
                interval.tick().await;
                
                match monitor.generate_report().await {
                    Ok(report) => {
                        debug!("Performance report generated successfully");
                        
                        // 如果有严重警告，记录详细信息
                        for warning in &report.warnings {
                            if warning.severity >= WarningSeverity::High {
                                warn!(
                                    warning_type = ?warning.warning_type,
                                    message = %warning.message,
                                    current_value = warning.current_value,
                                    threshold = warning.threshold_value,
                                    severity = ?warning.severity,
                                    "High severity performance warning"
                                );
                            }
                        }
                    },
                    Err(e) => {
                        warn!("Failed to generate performance report: {}", e);
                    }
                }
                
                // 清理旧样本
                monitor.cleanup_old_samples().await;
            }
        })
    }
    
    /// 克隆监控器（用于异步任务）
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
            recent_samples: Arc::clone(&self.recent_samples),
            reporter: PerformanceReporter {
                last_report_time: Arc::clone(&self.reporter.last_report_time),
            },
        }
    }
    
    /// 判断是否应该采样
    fn should_sample(&self) -> bool {
        fastrand::f64() < self.config.sampling_rate
    }
    
    /// 估算上下文大小
    fn estimate_context_size(&self, context: &PropagationContext) -> usize {
        // 基础结构大小
        let base_size = std::mem::size_of::<PropagationContext>();
        
        // 字符串字段大小
        let strings_size = context.trace_id.len() +
            context.parent_span_id.as_ref().map(|s| s.len()).unwrap_or(0) +
            context.current_span_id.len() +
            context.service_name.len() +
            context.operation_name.len();
        
        // 标签大小
        let tags_size: usize = context.tags
            .iter()
            .map(|(k, v)| k.len() + v.len())
            .sum();
        
        // 日志大小
        let logs_size: usize = context.logs
            .iter()
            .map(|log| log.message.len() + 
                 log.fields.iter().map(|(k, v)| k.len() + v.to_string().len()).sum::<usize>())
            .sum();
        
        base_size + strings_size + tags_size + logs_size
    }
    
    /// 添加性能样本
    async fn add_sample(&self, sample: PerformanceSample) {
        let mut samples = self.recent_samples.write().await;
        samples.push(sample);
        
        // 限制样本数量
        if samples.len() > self.config.max_samples {
            samples.drain(0..samples.len() - self.config.max_samples);
        }
    }
    
    /// 清理旧样本
    async fn cleanup_old_samples(&self) {
        let cutoff_time = Utc::now() - chrono::Duration::seconds(self.config.sample_retention_seconds as i64);
        
        let mut samples = self.recent_samples.write().await;
        samples.retain(|sample| sample.timestamp > cutoff_time);
    }
    
    /// 计算性能摘要
    async fn calculate_summary(&self, duration: Duration) -> PerformanceSummary {
        let total_injections = self.metrics.header_injections.load(Ordering::Relaxed);
        let total_extractions = self.metrics.header_extractions.load(Ordering::Relaxed);
        let total_operations = total_injections + total_extractions;
        
        let total_errors = self.metrics.injection_errors.load(Ordering::Relaxed) +
                          self.metrics.extraction_errors.load(Ordering::Relaxed);
        
        let operations_per_second = if duration.as_secs() > 0 {
            total_operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };
        
        let error_rate_percent = if total_operations > 0 {
            (total_errors as f64 / total_operations as f64) * 100.0
        } else {
            0.0
        };
        
        let avg_injection_latency = if total_injections > 0 {
            self.metrics.total_injection_latency_micros.load(Ordering::Relaxed) as f64 / total_injections as f64
        } else {
            0.0
        };
        
        let avg_extraction_latency = if total_extractions > 0 {
            self.metrics.total_extraction_latency_micros.load(Ordering::Relaxed) as f64 / total_extractions as f64
        } else {
            0.0
        };
        
        // 计算P95延迟（从样本中）
        let samples = self.recent_samples.read().await;
        let (p95_injection, p95_extraction) = self.calculate_p95_latencies(&samples);
        
        PerformanceSummary {
            total_operations,
            operations_per_second,
            error_rate_percent,
            avg_injection_latency_micros: avg_injection_latency,
            avg_extraction_latency_micros: avg_extraction_latency,
            p95_injection_latency_micros: p95_injection,
            p95_extraction_latency_micros: p95_extraction,
            current_memory_usage_mb: self.metrics.estimated_memory_usage_bytes.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0),
            peak_memory_usage_mb: self.metrics.peak_contexts_count.load(Ordering::Relaxed) as f64 * 1024.0 / (1024.0 * 1024.0), // 估算
            context_count: self.metrics.current_contexts_count.load(Ordering::Relaxed),
        }
    }
    
    /// 计算P95延迟
    fn calculate_p95_latencies(&self, samples: &[PerformanceSample]) -> (u64, u64) {
        let mut injection_latencies: Vec<u64> = samples
            .iter()
            .filter(|s| s.operation == "header_injection" && s.success)
            .map(|s| s.latency_micros)
            .collect();
        
        let mut extraction_latencies: Vec<u64> = samples
            .iter()
            .filter(|s| s.operation == "header_extraction" && s.success)
            .map(|s| s.latency_micros)
            .collect();
        
        injection_latencies.sort_unstable();
        extraction_latencies.sort_unstable();
        
        let p95_injection = if injection_latencies.is_empty() {
            0
        } else {
            let index = ((injection_latencies.len() as f64) * 0.95) as usize;
            injection_latencies.get(index.min(injection_latencies.len() - 1)).copied().unwrap_or(0)
        };
        
        let p95_extraction = if extraction_latencies.is_empty() {
            0
        } else {
            let index = ((extraction_latencies.len() as f64) * 0.95) as usize;
            extraction_latencies.get(index.min(extraction_latencies.len() - 1)).copied().unwrap_or(0)
        };
        
        (p95_injection, p95_extraction)
    }
    
    /// 检查性能警告
    async fn check_warnings(&self, summary: &PerformanceSummary) -> Vec<PerformanceWarning> {
        let mut warnings = Vec::new();
        
        // 检查延迟警告
        if summary.avg_injection_latency_micros > self.config.warning_thresholds.header_injection_latency_micros as f64 {
            warnings.push(PerformanceWarning {
                warning_type: WarningType::HighLatency,
                message: "Average header injection latency exceeds threshold".to_string(),
                current_value: summary.avg_injection_latency_micros,
                threshold_value: self.config.warning_thresholds.header_injection_latency_micros as f64,
                severity: WarningSeverity::Medium,
            });
        }
        
        if summary.avg_extraction_latency_micros > self.config.warning_thresholds.header_extraction_latency_micros as f64 {
            warnings.push(PerformanceWarning {
                warning_type: WarningType::HighLatency,
                message: "Average header extraction latency exceeds threshold".to_string(),
                current_value: summary.avg_extraction_latency_micros,
                threshold_value: self.config.warning_thresholds.header_extraction_latency_micros as f64,
                severity: WarningSeverity::Medium,
            });
        }
        
        // 检查错误率
        if summary.error_rate_percent > 5.0 {
            warnings.push(PerformanceWarning {
                warning_type: WarningType::HighErrorRate,
                message: format!("Error rate is {:.2}%", summary.error_rate_percent),
                current_value: summary.error_rate_percent,
                threshold_value: 5.0,
                severity: if summary.error_rate_percent > 10.0 {
                    WarningSeverity::High
                } else {
                    WarningSeverity::Medium
                },
            });
        }
        
        // 检查内存使用
        let memory_threshold_mb = self.config.warning_thresholds.memory_usage_warning_bytes as f64 / (1024.0 * 1024.0);
        if summary.current_memory_usage_mb > memory_threshold_mb {
            warnings.push(PerformanceWarning {
                warning_type: WarningType::MemoryUsage,
                message: format!("Memory usage is {:.2}MB", summary.current_memory_usage_mb),
                current_value: summary.current_memory_usage_mb,
                threshold_value: memory_threshold_mb,
                severity: WarningSeverity::High,
            });
        }
        
        warnings
    }
    
    /// 生成建议
    async fn generate_recommendations(&self, summary: &PerformanceSummary, warnings: &[PerformanceWarning]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        for warning in warnings {
            match warning.warning_type {
                WarningType::HighLatency => {
                    recommendations.push("Consider reducing trace context payload size".to_string());
                    recommendations.push("Optimize header serialization performance".to_string());
                },
                WarningType::HighErrorRate => {
                    recommendations.push("Review error logs for common failure patterns".to_string());
                    recommendations.push("Consider fallback mechanisms for trace propagation".to_string());
                },
                WarningType::MemoryUsage => {
                    recommendations.push("Implement more aggressive context cleanup".to_string());
                    recommendations.push("Consider reducing context TTL".to_string());
                },
                WarningType::ContextLeakage => {
                    recommendations.push("Check for missing context destruction calls".to_string());
                },
                WarningType::ThroughputDegradation => {
                    recommendations.push("Consider increasing async task pool size".to_string());
                },
            }
        }
        
        // 通用建议
        if summary.operations_per_second > 10000.0 {
            recommendations.push("Consider reducing sampling rate in high-traffic scenarios".to_string());
        }
        
        if summary.context_count > 1000 {
            recommendations.push("Monitor for potential context leaks".to_string());
        }
        
        recommendations.sort();
        recommendations.dedup();
        
        recommendations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let config = MonitorConfig::default();
        let monitor = TracePerformanceMonitor::new(config);
        
        // 测试基本功能
        assert!(monitor.config.enabled);
        assert_eq!(monitor.metrics.header_injections.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_performance_recording() {
        let config = MonitorConfig {
            sampling_rate: 1.0, // 100% 采样用于测试
            ..Default::default()
        };
        let monitor = TracePerformanceMonitor::new(config);
        
        let context = PropagationContext {
            trace_id: "test-trace-id".to_string(),
            parent_span_id: None,
            current_span_id: "test-span-id".to_string(),
            service_name: "test-service".to_string(),
            operation_name: "test-operation".to_string(),
            depth: 0,
            created_at: Utc::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
        };
        
        // 记录性能数据
        monitor.record_header_injection(Duration::from_micros(500), true, &context).await;
        monitor.record_context_creation(&context).await;
        
        // 验证指标更新
        assert_eq!(monitor.metrics.header_injections.load(Ordering::Relaxed), 1);
        assert_eq!(monitor.metrics.context_creations.load(Ordering::Relaxed), 1);
        assert_eq!(monitor.metrics.current_contexts_count.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let config = MonitorConfig::default();
        let monitor = TracePerformanceMonitor::new(config);
        
        let report = monitor.generate_report().await.unwrap();
        assert!(report.timestamp <= Utc::now());
        assert_eq!(report.summary.total_operations, 0);
    }
}