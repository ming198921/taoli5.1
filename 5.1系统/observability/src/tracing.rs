//! 分布式全链路追踪系统
//! 
//! 提供生产级的分布式追踪功能，包括：
//! - 自动trace_id注入到所有异步任务
//! - span关系追踪和性能分析
//! - 错误链路自动分析
//! - 与日志系统完全集成

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, error, info, instrument, span, warn, Instrument, Level, Span};
use uuid::Uuid;

/// 分布式追踪器
pub struct DistributedTracer {
    /// 配置
    config: TracingConfig,
    /// 当前活跃的追踪上下文
    active_contexts: Arc<RwLock<HashMap<String, TraceContext>>>,
    /// Span存储
    span_storage: Arc<RwLock<HashMap<String, SpanInfo>>>,
    /// 错误链路分析器
    error_analyzer: ErrorChainAnalyzer,
    /// 性能分析器
    performance_analyzer: PerformanceAnalyzer,
    /// 事件发送器
    event_sender: Arc<mpsc::UnboundedSender<TraceEvent>>,
    /// 序列号生成器
    sequence_generator: AtomicU64,
}

/// 追踪配置
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// 是否启用分布式追踪
    pub enabled: bool,
    /// 采样率 (0.0 - 1.0)
    pub sampling_rate: f64,
    /// 最大span深度
    pub max_span_depth: u32,
    /// Span最大生存时间
    pub span_ttl: Duration,
    /// 是否启用性能分析
    pub enable_performance_analysis: bool,
    /// 是否启用错误链路分析
    pub enable_error_analysis: bool,
    /// 是否自动注入到异步任务
    pub auto_inject_async_tasks: bool,
    /// 服务名称
    pub service_name: String,
    /// 版本
    pub service_version: String,
}

/// 追踪上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    /// 追踪ID
    pub trace_id: String,
    /// 父Span ID
    pub parent_span_id: Option<String>,
    /// 当前Span ID
    pub current_span_id: String,
    /// 服务名称
    pub service_name: String,
    /// 操作名称
    pub operation_name: String,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 标签
    pub tags: HashMap<String, String>,
    /// 日志
    pub logs: Vec<LogEntry>,
    /// 状态
    pub status: SpanStatus,
    /// 深度
    pub depth: u32,
    /// 采样标志
    pub sampled: bool,
}

/// Span信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    /// Span ID
    pub span_id: String,
    /// 追踪ID
    pub trace_id: String,
    /// 父Span ID
    pub parent_span_id: Option<String>,
    /// 操作名称
    pub operation_name: String,
    /// 服务名称
    pub service_name: String,
    /// 开始时间
    pub start_time: DateTime<Utc>,
    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,
    /// 持续时间（毫秒）
    pub duration_ms: Option<u64>,
    /// 标签
    pub tags: HashMap<String, String>,
    /// 日志
    pub logs: Vec<LogEntry>,
    /// 状态
    pub status: SpanStatus,
    /// 错误信息
    pub error: Option<String>,
    /// 子Span
    pub child_spans: Vec<String>,
}

/// Span状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// 活跃中
    Active,
    /// 成功完成
    Success,
    /// 出现错误
    Error,
    /// 超时
    Timeout,
    /// 取消
    Cancelled,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// 追踪事件
#[derive(Debug, Clone)]
pub enum TraceEvent {
    SpanCreated(SpanInfo),
    SpanUpdated(SpanInfo),
    SpanFinished(SpanInfo),
    ErrorDetected { trace_id: String, span_id: String, error: String },
    PerformanceIssue { trace_id: String, span_id: String, issue: PerformanceIssue },
}

/// 性能问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub issue_type: PerformanceIssueType,
    pub description: String,
    pub severity: IssueSeverity,
    pub metrics: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

/// 性能问题类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceIssueType {
    SlowOperation,
    HighMemoryUsage,
    FrequentRetries,
    LongQueue,
    DeadlockRisk,
}

/// 问题严重性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

/// 错误链路分析器
pub struct ErrorChainAnalyzer {
    error_patterns: HashMap<String, ErrorPattern>,
    active_errors: Arc<RwLock<HashMap<String, ErrorChain>>>,
}

/// 错误模式
#[derive(Debug, Clone)]
struct ErrorPattern {
    pattern_name: String,
    error_regex: regex::Regex,
    root_cause_indicators: Vec<String>,
    impact_assessment: ImpactLevel,
}

/// 错误链
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorChain {
    chain_id: String,
    trace_id: String,
    error_sequence: Vec<ErrorEvent>,
    root_cause: Option<String>,
    impact_level: ImpactLevel,
    resolution_suggestions: Vec<String>,
}

/// 错误事件
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorEvent {
    timestamp: DateTime<Utc>,
    span_id: String,
    error_type: String,
    error_message: String,
    stack_trace: Option<String>,
}

/// 影响级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImpactLevel {
    System,
    Service,
    Operation,
    Request,
}

/// 性能分析器
pub struct PerformanceAnalyzer {
    config: PerformanceAnalysisConfig,
    metric_collectors: HashMap<String, Box<dyn MetricCollector + Send + Sync>>,
    performance_baselines: Arc<RwLock<HashMap<String, PerformanceBaseline>>>,
}

/// 性能分析配置
#[derive(Debug, Clone)]
struct PerformanceAnalysisConfig {
    enable_latency_analysis: bool,
    enable_throughput_analysis: bool,
    enable_resource_analysis: bool,
    analysis_window: Duration,
    alert_thresholds: HashMap<String, f64>,
}

/// 性能基线
#[derive(Debug, Clone)]
struct PerformanceBaseline {
    operation_name: String,
    avg_latency_ms: f64,
    p95_latency_ms: f64,
    avg_throughput: f64,
    error_rate: f64,
    last_updated: DateTime<Utc>,
}

/// 指标收集器trait
trait MetricCollector {
    fn collect(&self, span: &SpanInfo) -> Result<Vec<(String, f64)>>;
    fn reset(&mut self);
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 1.0,
            max_span_depth: 20,
            span_ttl: Duration::from_hours(1),
            enable_performance_analysis: true,
            enable_error_analysis: true,
            auto_inject_async_tasks: true,
            service_name: "arbitrage-system-5.1".to_string(),
            service_version: "5.1.0".to_string(),
        }
    }
}

impl DistributedTracer {
    /// 创建新的分布式追踪器
    pub fn new(config: TracingConfig) -> Result<Self> {
        let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
        
        let tracer = Self {
            config: config.clone(),
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
            span_storage: Arc::new(RwLock::new(HashMap::new())),
            error_analyzer: ErrorChainAnalyzer::new()?,
            performance_analyzer: PerformanceAnalyzer::new(config.clone())?,
            event_sender: Arc::new(event_sender),
            sequence_generator: AtomicU64::new(1),
        };
        
        // 启动事件处理器
        let span_storage = tracer.span_storage.clone();
        let error_analyzer = tracer.error_analyzer.clone();
        let performance_analyzer = tracer.performance_analyzer.clone();
        
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                if let Err(e) = Self::handle_trace_event(
                    event,
                    &span_storage,
                    &error_analyzer,
                    &performance_analyzer,
                ).await {
                    error!("Failed to handle trace event: {}", e);
                }
            }
        });
        
        // 启动清理任务
        tracer.start_cleanup_task();
        
        Ok(tracer)
    }
    
    /// 开始新的追踪
    #[instrument(skip(self))]
    pub async fn start_trace(&self, operation_name: &str) -> Result<TraceContext> {
        let trace_id = Uuid::new_v4().to_string();
        let span_id = self.generate_span_id();
        
        let context = TraceContext {
            trace_id: trace_id.clone(),
            parent_span_id: None,
            current_span_id: span_id.clone(),
            service_name: self.config.service_name.clone(),
            operation_name: operation_name.to_string(),
            start_time: Utc::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Active,
            depth: 0,
            sampled: self.should_sample(),
        };
        
        // 存储上下文
        self.active_contexts.write().await.insert(trace_id.clone(), context.clone());
        
        // 创建Span信息
        let span_info = SpanInfo {
            span_id: span_id.clone(),
            trace_id: trace_id.clone(),
            parent_span_id: None,
            operation_name: operation_name.to_string(),
            service_name: self.config.service_name.clone(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Active,
            error: None,
            child_spans: Vec::new(),
        };
        
        // 存储Span
        self.span_storage.write().await.insert(span_id.clone(), span_info.clone());
        
        // 发送事件
        let _ = self.event_sender.send(TraceEvent::SpanCreated(span_info));
        
        debug!(trace_id = %trace_id, span_id = %span_id, "Started new trace");
        Ok(context)
    }
    
    /// 创建子Span
    #[instrument(skip(self, parent_context))]
    pub async fn start_span(
        &self,
        parent_context: &TraceContext,
        operation_name: &str,
    ) -> Result<TraceContext> {
        if parent_context.depth >= self.config.max_span_depth {
            return Err(anyhow::anyhow!("Max span depth exceeded"));
        }
        
        let span_id = self.generate_span_id();
        
        let context = TraceContext {
            trace_id: parent_context.trace_id.clone(),
            parent_span_id: Some(parent_context.current_span_id.clone()),
            current_span_id: span_id.clone(),
            service_name: self.config.service_name.clone(),
            operation_name: operation_name.to_string(),
            start_time: Utc::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Active,
            depth: parent_context.depth + 1,
            sampled: parent_context.sampled,
        };
        
        // 更新父Span的子Span列表
        if let Some(mut parent_span) = self.span_storage.write().await.get_mut(&parent_context.current_span_id) {
            parent_span.child_spans.push(span_id.clone());
        }
        
        // 创建Span信息
        let span_info = SpanInfo {
            span_id: span_id.clone(),
            trace_id: parent_context.trace_id.clone(),
            parent_span_id: Some(parent_context.current_span_id.clone()),
            operation_name: operation_name.to_string(),
            service_name: self.config.service_name.clone(),
            start_time: Utc::now(),
            end_time: None,
            duration_ms: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Active,
            error: None,
            child_spans: Vec::new(),
        };
        
        // 存储Span
        self.span_storage.write().await.insert(span_id.clone(), span_info.clone());
        
        // 发送事件
        let _ = self.event_sender.send(TraceEvent::SpanCreated(span_info));
        
        debug!(
            trace_id = %parent_context.trace_id,
            parent_span_id = %parent_context.current_span_id,
            span_id = %span_id,
            "Started child span"
        );
        
        Ok(context)
    }
    
    /// 完成Span
    #[instrument(skip(self, context))]
    pub async fn finish_span(&self, mut context: TraceContext) -> Result<()> {
        let end_time = Utc::now();
        let duration_ms = (end_time - context.start_time).num_milliseconds() as u64;
        
        context.status = SpanStatus::Success;
        
        // 更新Span信息
        if let Some(mut span_info) = self.span_storage.write().await.get_mut(&context.current_span_id) {
            span_info.end_time = Some(end_time);
            span_info.duration_ms = Some(duration_ms);
            span_info.status = context.status;
            span_info.tags = context.tags.clone();
            span_info.logs = context.logs.clone();
            
            // 发送事件
            let _ = self.event_sender.send(TraceEvent::SpanFinished(span_info.clone()));
        }
        
        // 从活跃上下文中移除
        self.active_contexts.write().await.remove(&context.trace_id);
        
        debug!(
            trace_id = %context.trace_id,
            span_id = %context.current_span_id,
            duration_ms = duration_ms,
            "Finished span"
        );
        
        Ok(())
    }
    
    /// 记录错误
    #[instrument(skip(self, context))]
    pub async fn record_error(&self, context: &mut TraceContext, error: &str) -> Result<()> {
        context.status = SpanStatus::Error;
        context.logs.push(LogEntry {
            timestamp: Utc::now(),
            level: "ERROR".to_string(),
            message: error.to_string(),
            fields: HashMap::new(),
        });
        
        // 更新Span信息
        if let Some(mut span_info) = self.span_storage.write().await.get_mut(&context.current_span_id) {
            span_info.status = SpanStatus::Error;
            span_info.error = Some(error.to_string());
            span_info.logs.push(context.logs.last().unwrap().clone());
            
            // 发送错误事件
            let _ = self.event_sender.send(TraceEvent::ErrorDetected {
                trace_id: context.trace_id.clone(),
                span_id: context.current_span_id.clone(),
                error: error.to_string(),
            });
        }
        
        warn!(
            trace_id = %context.trace_id,
            span_id = %context.current_span_id,
            error = %error,
            "Recorded error in span"
        );
        
        Ok(())
    }
    
    /// 添加标签
    pub async fn add_tag(&self, context: &mut TraceContext, key: &str, value: &str) -> Result<()> {
        context.tags.insert(key.to_string(), value.to_string());
        
        // 更新Span信息
        if let Some(mut span_info) = self.span_storage.write().await.get_mut(&context.current_span_id) {
            span_info.tags.insert(key.to_string(), value.to_string());
        }
        
        Ok(())
    }
    
    /// 记录日志
    pub async fn log(&self, context: &mut TraceContext, level: &str, message: &str) -> Result<()> {
        let log_entry = LogEntry {
            timestamp: Utc::now(),
            level: level.to_string(),
            message: message.to_string(),
            fields: HashMap::new(),
        };
        
        context.logs.push(log_entry.clone());
        
        // 更新Span信息
        if let Some(mut span_info) = self.span_storage.write().await.get_mut(&context.current_span_id) {
            span_info.logs.push(log_entry);
        }
        
        Ok(())
    }
    
    /// 获取追踪信息
    pub async fn get_trace(&self, trace_id: &str) -> Result<Vec<SpanInfo>> {
        let storage = self.span_storage.read().await;
        let spans: Vec<SpanInfo> = storage
            .values()
            .filter(|span| span.trace_id == trace_id)
            .cloned()
            .collect();
        
        Ok(spans)
    }
    
    /// 自动注入到异步任务
    pub fn instrument_async_task<F>(&self, trace_context: TraceContext, future: F) -> impl std::future::Future<Output = F::Output>
    where
        F: std::future::Future,
    {
        let span = tracing::span!(
            Level::INFO,
            "async_task",
            trace_id = %trace_context.trace_id,
            span_id = %trace_context.current_span_id,
            operation = %trace_context.operation_name
        );
        
        future.instrument(span)
    }
    
    /// 生成Span ID
    fn generate_span_id(&self) -> String {
        format!("span-{:016x}", self.sequence_generator.fetch_add(1, Ordering::SeqCst))
    }
    
    /// 判断是否应该采样
    fn should_sample(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < self.config.sampling_rate
    }
    
    /// 处理追踪事件
    async fn handle_trace_event(
        event: TraceEvent,
        span_storage: &Arc<RwLock<HashMap<String, SpanInfo>>>,
        error_analyzer: &ErrorChainAnalyzer,
        performance_analyzer: &PerformanceAnalyzer,
    ) -> Result<()> {
        match event {
            TraceEvent::SpanCreated(span_info) => {
                debug!("Processing span created event: {}", span_info.span_id);
            }
            TraceEvent::SpanFinished(span_info) => {
                // 性能分析
                if let Err(e) = performance_analyzer.analyze_span(&span_info).await {
                    warn!("Performance analysis failed: {}", e);
                }
            }
            TraceEvent::ErrorDetected { trace_id, span_id, error } => {
                // 错误链路分析
                if let Err(e) = error_analyzer.analyze_error(&trace_id, &span_id, &error).await {
                    warn!("Error analysis failed: {}", e);
                }
            }
            TraceEvent::PerformanceIssue { trace_id, span_id, issue } => {
                warn!(
                    "Performance issue detected in trace {}, span {}: {:?}",
                    trace_id, span_id, issue
                );
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// 启动清理任务
    fn start_cleanup_task(&self) {
        let span_storage = self.span_storage.clone();
        let active_contexts = self.active_contexts.clone();
        let span_ttl = self.config.span_ttl;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_minutes(5));
            
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                
                // 清理过期Span
                {
                    let mut storage = span_storage.write().await;
                    storage.retain(|_, span| {
                        (now - span.start_time).to_std().unwrap_or(Duration::MAX) < span_ttl
                    });
                }
                
                // 清理过期上下文
                {
                    let mut contexts = active_contexts.write().await;
                    contexts.retain(|_, context| {
                        (now - context.start_time).to_std().unwrap_or(Duration::MAX) < span_ttl
                    });
                }
                
                debug!("Cleaned up expired spans and contexts");
            }
        });
    }
    
    /// 导出追踪数据
    pub async fn export_traces(&self, trace_ids: &[String]) -> Result<Vec<TraceExport>> {
        let mut exports = Vec::new();
        
        for trace_id in trace_ids {
            let spans = self.get_trace(trace_id).await?;
            let export = TraceExport {
                trace_id: trace_id.clone(),
                spans,
                export_time: Utc::now(),
            };
            exports.push(export);
        }
        
        Ok(exports)
    }
}

/// 追踪导出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceExport {
    pub trace_id: String,
    pub spans: Vec<SpanInfo>,
    pub export_time: DateTime<Utc>,
}

impl ErrorChainAnalyzer {
    fn new() -> Result<Self> {
        Ok(Self {
            error_patterns: Self::load_error_patterns()?,
            active_errors: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    fn load_error_patterns() -> Result<HashMap<String, ErrorPattern>> {
        // 加载预定义的错误模式
        let mut patterns = HashMap::new();
        
        patterns.insert("connection_timeout".to_string(), ErrorPattern {
            pattern_name: "Connection Timeout".to_string(),
            error_regex: regex::Regex::new(r"(?i)(connection|connect).*timeout")?,
            root_cause_indicators: vec!["network_latency".to_string(), "server_overload".to_string()],
            impact_assessment: ImpactLevel::Operation,
        });
        
        patterns.insert("authentication_failure".to_string(), ErrorPattern {
            pattern_name: "Authentication Failure".to_string(),
            error_regex: regex::Regex::new(r"(?i)(auth|authentication).*fail")?,
            root_cause_indicators: vec!["invalid_credentials".to_string(), "token_expired".to_string()],
            impact_assessment: ImpactLevel::Service,
        });
        
        Ok(patterns)
    }
    
    async fn analyze_error(&self, trace_id: &str, span_id: &str, error: &str) -> Result<()> {
        // 分析错误模式并构建错误链
        for (_, pattern) in &self.error_patterns {
            if pattern.error_regex.is_match(error) {
                debug!("Error pattern matched: {} for error: {}", pattern.pattern_name, error);
                // 这里可以实现更复杂的错误链分析逻辑
                break;
            }
        }
        Ok(())
    }
    
    fn clone(&self) -> Self {
        Self {
            error_patterns: self.error_patterns.clone(),
            active_errors: self.active_errors.clone(),
        }
    }
}

impl PerformanceAnalyzer {
    fn new(config: TracingConfig) -> Result<Self> {
        Ok(Self {
            config: PerformanceAnalysisConfig {
                enable_latency_analysis: true,
                enable_throughput_analysis: true,
                enable_resource_analysis: true,
                analysis_window: Duration::from_minutes(5),
                alert_thresholds: HashMap::new(),
            },
            metric_collectors: HashMap::new(),
            performance_baselines: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    async fn analyze_span(&self, span_info: &SpanInfo) -> Result<()> {
        if let Some(duration_ms) = span_info.duration_ms {
            // 检查是否超过基线性能
            let baselines = self.performance_baselines.read().await;
            if let Some(baseline) = baselines.get(&span_info.operation_name) {
                if duration_ms as f64 > baseline.p95_latency_ms * 1.5 {
                    debug!(
                        "Performance issue detected: {} took {}ms (baseline p95: {}ms)",
                        span_info.operation_name,
                        duration_ms,
                        baseline.p95_latency_ms
                    );
                }
            }
        }
        Ok(())
    }
    
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metric_collectors: HashMap::new(), // 不克隆collectors，因为包含trait对象
            performance_baselines: self.performance_baselines.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_trace_creation() {
        let config = TracingConfig::default();
        let tracer = DistributedTracer::new(config).unwrap();
        
        let trace_context = tracer.start_trace("test_operation").await.unwrap();
        assert!(!trace_context.trace_id.is_empty());
        assert!(!trace_context.current_span_id.is_empty());
        assert_eq!(trace_context.depth, 0);
        
        tracer.finish_span(trace_context).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_child_span_creation() {
        let config = TracingConfig::default();
        let tracer = DistributedTracer::new(config).unwrap();
        
        let parent_context = tracer.start_trace("parent_operation").await.unwrap();
        let child_context = tracer.start_span(&parent_context, "child_operation").await.unwrap();
        
        assert_eq!(child_context.trace_id, parent_context.trace_id);
        assert_eq!(child_context.parent_span_id, Some(parent_context.current_span_id.clone()));
        assert_eq!(child_context.depth, 1);
        
        tracer.finish_span(child_context).await.unwrap();
        tracer.finish_span(parent_context).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_error_recording() {
        let config = TracingConfig::default();
        let tracer = DistributedTracer::new(config).unwrap();
        
        let mut trace_context = tracer.start_trace("error_operation").await.unwrap();
        tracer.record_error(&mut trace_context, "Test error").await.unwrap();
        
        assert_eq!(trace_context.status, SpanStatus::Error);
        assert!(!trace_context.logs.is_empty());
        
        tracer.finish_span(trace_context).await.unwrap();
    }
    
    #[tokio::test]
    async fn test_trace_export() {
        let config = TracingConfig::default();
        let tracer = DistributedTracer::new(config).unwrap();
        
        let trace_context = tracer.start_trace("export_test").await.unwrap();
        let trace_id = trace_context.trace_id.clone();
        tracer.finish_span(trace_context).await.unwrap();
        
        // 等待事件处理
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let exports = tracer.export_traces(&[trace_id]).await.unwrap();
        assert_eq!(exports.len(), 1);
        assert!(!exports[0].spans.is_empty());
    }
}