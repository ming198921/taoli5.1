//! 全链路可观测性系统
//! 
//! 提供分布式追踪、指标收集、日志聚合和性能监控
//! 支持OpenTelemetry标准和多种后端存储

// 模块声明已清理 - 删除不存在的模块

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{info, warn, error, debug, instrument, Span};
use uuid::Uuid;

// pub use声明已清理 - 删除不存在模块的导出

/// 观测性事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityEvent {
    pub event_id: String,
    pub event_type: ObservabilityEventType,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub service: String,
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
    pub data: serde_json::Value,
    pub tags: HashMap<String, String>,
    pub severity: ObservabilitySeverity,
}

/// 观测性事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservabilityEventType {
    Trace,
    Metric,
    Log,
    Profile,
    Health,
    Alert,
    Correlation,
}

/// 观测性严重性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// 系统组件状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub component_name: String,
    pub health_status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub error_rate: f64,
    pub throughput: f64,
    pub resource_usage: ResourceUsage,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub network_in_bps: u64,
    pub network_out_bps: u64,
}

/// 性能洞察
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsight {
    pub insight_id: String,
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub severity: ObservabilitySeverity,
    pub affected_components: Vec<String>,
    pub metrics: HashMap<String, f64>,
    pub recommendations: Vec<String>,
    pub detected_at: DateTime<Utc>,
}

/// 洞察类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InsightType {
    PerformanceBottleneck,
    ResourceExhaustion,
    AnomalyDetected,
    TrendAnalysis,
    CapacityPlanning,
    OptimizationOpportunity,
}

/// 全链路可观测性系统
pub struct FullStackObservability {
    /// 配置
    config: ObservabilityConfig,
    /// 分布式追踪器
    tracer: Arc<DistributedTracer>,
    /// 指标收集器
    metrics_collector: Arc<MetricsCollector>,
    /// 日志聚合器
    log_aggregator: Arc<LogAggregator>,
    /// 性能分析器
    profiler: Arc<PerformanceProfiler>,
    /// 健康检查器
    health_checker: Arc<HealthChecker>,
    /// 告警管理器
    alert_manager: Arc<AlertManager>,
    /// 仪表盘管理器
    dashboard_manager: Arc<DashboardManager>,
    /// 事件关联器
    event_correlator: Arc<EventCorrelator>,
    /// 事件广播器
    event_tx: broadcast::Sender<ObservabilityEvent>,
    /// 内部事件处理器
    internal_tx: mpsc::UnboundedSender<InternalEvent>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 组件状态
    component_status: Arc<RwLock<HashMap<String, ComponentStatus>>>,
    /// 性能洞察
    insights: Arc<RwLock<Vec<PerformanceInsight>>>,
}

/// 内部事件类型
#[derive(Debug, Clone)]
enum InternalEvent {
    NewObservabilityEvent(ObservabilityEvent),
    ComponentStatusUpdate(ComponentStatus),
    PerformanceInsight(PerformanceInsight),
    HealthCheck,
    MetricsCollection,
    AnalysisRun,
}

impl FullStackObservability {
    /// 创建新的全链路可观测性系统
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        let tracer = Arc::new(DistributedTracer::new(config.tracing.clone()).await?);
        let metrics_collector = Arc::new(MetricsCollector::new(config.metrics.clone()).await?);
        let log_aggregator = Arc::new(LogAggregator::new(config.logging.clone()).await?);
        let profiler = Arc::new(PerformanceProfiler::new(config.profiling.clone())?);
        let health_checker = Arc::new(HealthChecker::new(config.health.clone()).await?);
        let alert_manager = Arc::new(AlertManager::new(config.alerting.clone()).await?);
        let dashboard_manager = Arc::new(DashboardManager::new(config.dashboard.clone()).await?);
        let event_correlator = Arc::new(EventCorrelator::new(config.correlation.clone()).await?);

        let (event_tx, _) = broadcast::channel(10000);
        let (internal_tx, internal_rx) = mpsc::unbounded_channel();

        let observability = Self {
            config,
            tracer,
            metrics_collector,
            log_aggregator,
            profiler,
            health_checker,
            alert_manager,
            dashboard_manager,
            event_correlator,
            event_tx,
            internal_tx,
            running: Arc::new(RwLock::new(false)),
            component_status: Arc::new(RwLock::new(HashMap::new())),
            insights: Arc::new(RwLock::new(Vec::new())),
        };

        // 启动内部事件处理器
        observability.start_event_processor(internal_rx).await;

        info!("Full-stack observability system initialized successfully");
        Ok(observability)
    }

    /// 启动可观测性系统
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Observability system is already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        // 启动各个组件
        self.tracer.start().await?;
        self.metrics_collector.start().await?;
        self.log_aggregator.start().await?;
        self.profiler.start().await?;
        self.health_checker.start().await?;
        self.alert_manager.start().await?;
        self.dashboard_manager.start().await?;
        self.event_correlator.start().await?;

        // 启动后台任务
        self.start_background_tasks().await;

        info!("Full-stack observability system started successfully");
        Ok(())
    }

    /// 停止可观测性系统
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            warn!("Observability system is not running");
            return Ok(());
        }
        *running = false;

        // 停止各个组件
        self.tracer.stop().await?;
        self.metrics_collector.stop().await?;
        self.log_aggregator.stop().await?;
        self.profiler.stop().await?;
        self.health_checker.stop().await?;
        self.alert_manager.stop().await?;
        self.dashboard_manager.stop().await?;
        self.event_correlator.stop().await?;

        info!("Full-stack observability system stopped successfully");
        Ok(())
    }

    /// 创建新的分布式追踪Span
    #[instrument(skip(self))]
    pub async fn start_span(&self, operation_name: &str, parent_context: Option<TraceContext>) -> Result<SpanInfo> {
        let span_info = self.tracer.start_span(operation_name, parent_context).await?;
        
        // 记录追踪事件
        let event = ObservabilityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: ObservabilityEventType::Trace,
            timestamp: Utc::now(),
            source: "tracer".to_string(),
            service: "observability".to_string(),
            trace_id: Some(span_info.trace_id.clone()),
            span_id: Some(span_info.span_id.clone()),
            data: serde_json::json!({
                "operation_name": operation_name,
                "parent_span_id": span_info.parent_span_id
            }),
            tags: HashMap::new(),
            severity: ObservabilitySeverity::Info,
        };
        
        self.emit_event(event).await?;
        Ok(span_info)
    }

    /// 结束追踪Span
    pub async fn finish_span(&self, span_info: SpanInfo, tags: Option<HashMap<String, String>>) -> Result<()> {
        self.tracer.finish_span(span_info.clone(), tags.clone()).await?;
        
        // 记录Span结束事件
        let event = ObservabilityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: ObservabilityEventType::Trace,
            timestamp: Utc::now(),
            source: "tracer".to_string(),
            service: "observability".to_string(),
            trace_id: Some(span_info.trace_id),
            span_id: Some(span_info.span_id),
            data: serde_json::json!({
                "action": "span_finished",
                "tags": tags
            }),
            tags: HashMap::new(),
            severity: ObservabilitySeverity::Info,
        };
        
        self.emit_event(event).await?;
        Ok(())
    }

    /// 记录指标
    pub async fn record_metric(&self, name: &str, value: f64, metric_type: MetricType, tags: HashMap<String, String>) -> Result<()> {
        let metric_point = MetricPoint {
            name: name.to_string(),
            value,
            metric_type,
            tags: tags.clone(),
            timestamp: Utc::now(),
        };
        
        self.metrics_collector.record_metric(metric_point).await?;
        
        // 记录指标事件
        let event = ObservabilityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: ObservabilityEventType::Metric,
            timestamp: Utc::now(),
            source: "metrics".to_string(),
            service: "observability".to_string(),
            trace_id: None,
            span_id: None,
            data: serde_json::json!({
                "metric_name": name,
                "value": value,
                "type": metric_type
            }),
            tags,
            severity: ObservabilitySeverity::Info,
        };
        
        self.emit_event(event).await?;
        Ok(())
    }

    /// 记录日志
    pub async fn log(&self, level: LogLevel, message: &str, context: HashMap<String, serde_json::Value>) -> Result<()> {
        let log_entry = LogEntry {
            level,
            message: message.to_string(),
            timestamp: Utc::now(),
            service: "observability".to_string(),
            context,
            trace_id: None,
            span_id: None,
        };
        
        self.log_aggregator.log(log_entry).await?;
        
        // 记录日志事件
        let event = ObservabilityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: ObservabilityEventType::Log,
            timestamp: Utc::now(),
            source: "logging".to_string(),
            service: "observability".to_string(),
            trace_id: None,
            span_id: None,
            data: serde_json::json!({
                "level": level,
                "message": message,
                "context": context
            }),
            tags: HashMap::new(),
            severity: match level {
                LogLevel::Error => ObservabilitySeverity::High,
                LogLevel::Warn => ObservabilitySeverity::Medium,
                LogLevel::Info => ObservabilitySeverity::Info,
                LogLevel::Debug => ObservabilitySeverity::Low,
            },
        };
        
        self.emit_event(event).await?;
        Ok(())
    }

    /// 开始性能分析
    pub async fn start_profiling(&self, component: &str) -> Result<String> {
        let profile_id = self.profiler.start_profiling(component).await?;
        
        // 记录性能分析事件
        let event = ObservabilityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: ObservabilityEventType::Profile,
            timestamp: Utc::now(),
            source: "profiler".to_string(),
            service: "observability".to_string(),
            trace_id: None,
            span_id: None,
            data: serde_json::json!({
                "action": "profiling_started",
                "component": component,
                "profile_id": profile_id
            }),
            tags: HashMap::new(),
            severity: ObservabilitySeverity::Info,
        };
        
        self.emit_event(event).await?;
        Ok(profile_id)
    }

    /// 停止性能分析并获取结果
    pub async fn stop_profiling(&self, profile_id: &str) -> Result<ProfileData> {
        let profile_data = self.profiler.stop_profiling(profile_id).await?;
        
        // 记录性能分析完成事件
        let event = ObservabilityEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: ObservabilityEventType::Profile,
            timestamp: Utc::now(),
            source: "profiler".to_string(),
            service: "observability".to_string(),
            trace_id: None,
            span_id: None,
            data: serde_json::json!({
                "action": "profiling_completed",
                "profile_id": profile_id,
                "duration_ms": profile_data.duration_ms,
                "cpu_usage": profile_data.cpu_usage,
                "memory_usage": profile_data.memory_usage
            }),
            tags: HashMap::new(),
            severity: ObservabilitySeverity::Info,
        };
        
        self.emit_event(event).await?;
        Ok(profile_data)
    }

    /// 获取系统健康状态
    pub async fn get_system_health(&self) -> Result<HashMap<String, ComponentHealth>> {
        self.health_checker.check_all_components().await
    }

    /// 获取组件状态
    pub async fn get_component_status(&self, component: &str) -> Option<ComponentStatus> {
        let status = self.component_status.read().await;
        status.get(component).cloned()
    }

    /// 获取所有组件状态
    pub async fn get_all_component_status(&self) -> HashMap<String, ComponentStatus> {
        let status = self.component_status.read().await;
        status.clone()
    }

    /// 获取性能洞察
    pub async fn get_performance_insights(&self) -> Vec<PerformanceInsight> {
        let insights = self.insights.read().await;
        insights.clone()
    }

    /// 创建自定义仪表盘
    pub async fn create_dashboard(&self, name: &str, widgets: Vec<Widget>) -> Result<String> {
        self.dashboard_manager.create_dashboard(name, widgets).await
    }

    /// 获取仪表盘
    pub async fn get_dashboard(&self, dashboard_id: &str) -> Result<Dashboard> {
        self.dashboard_manager.get_dashboard(dashboard_id).await
    }

    /// 订阅观测性事件
    pub fn subscribe_events(&self) -> broadcast::Receiver<ObservabilityEvent> {
        self.event_tx.subscribe()
    }

    /// 添加告警规则
    pub async fn add_alert_rule(&self, rule: AlertRule) -> Result<String> {
        self.alert_manager.add_rule(rule).await
    }

    /// 获取系统统计信息
    pub async fn get_system_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        // 基本状态
        let running = *self.running.read().await;
        stats.insert("running".to_string(), serde_json::Value::Bool(running));

        // 组件数量
        let status = self.component_status.read().await;
        let component_count = status.len();
        stats.insert("monitored_components".to_string(), serde_json::Value::Number(component_count.into()));

        // 洞察数量
        let insights = self.insights.read().await;
        let insight_count = insights.len();
        stats.insert("performance_insights".to_string(), serde_json::Value::Number(insight_count.into()));

        // 获取各组件统计信息
        if let Ok(tracer_stats) = self.tracer.get_stats().await {
            for (key, value) in tracer_stats {
                stats.insert(format!("tracer_{}", key), value);
            }
        }

        if let Ok(metrics_stats) = self.metrics_collector.get_stats().await {
            for (key, value) in metrics_stats {
                stats.insert(format!("metrics_{}", key), value);
            }
        }

        stats
    }

    /// 发送观测性事件
    async fn emit_event(&self, event: ObservabilityEvent) -> Result<()> {
        // 广播事件
        let _ = self.event_tx.send(event.clone());
        
        // 发送到内部处理器
        self.internal_tx.send(InternalEvent::NewObservabilityEvent(event))?;
        
        Ok(())
    }

    /// 启动内部事件处理器
    async fn start_event_processor(&self, mut internal_rx: mpsc::UnboundedReceiver<InternalEvent>) {
        let observability = Arc::new(self);
        
        tokio::spawn(async move {
            while let Some(event) = internal_rx.recv().await {
                if let Err(e) = observability.process_internal_event(event).await {
                    error!(error = %e, "Failed to process internal event");
                }
            }
        });
    }

    /// 处理内部事件
    async fn process_internal_event(&self, event: InternalEvent) -> Result<()> {
        match event {
            InternalEvent::NewObservabilityEvent(obs_event) => {
                // 发送给事件关联器进行分析
                self.event_correlator.process_event(&obs_event).await?;
                
                // 检查是否需要触发告警
                self.alert_manager.check_alert_conditions(&obs_event).await?;
            }
            InternalEvent::ComponentStatusUpdate(status) => {
                let mut component_status = self.component_status.write().await;
                component_status.insert(status.component_name.clone(), status);
            }
            InternalEvent::PerformanceInsight(insight) => {
                let mut insights = self.insights.write().await;
                insights.push(insight);
                
                // 限制洞察数量
                if insights.len() > 1000 {
                    insights.remove(0);
                }
            }
            InternalEvent::HealthCheck => {
                self.perform_health_check().await?;
            }
            InternalEvent::MetricsCollection => {
                self.collect_system_metrics().await?;
            }
            InternalEvent::AnalysisRun => {
                self.run_performance_analysis().await?;
            }
        }

        Ok(())
    }

    /// 启动后台任务
    async fn start_background_tasks(&self) {
        let observability = Arc::new(self);

        // 健康检查任务
        {
            let obs_clone = Arc::clone(&observability);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // 1分钟

                loop {
                    interval.tick().await;
                    
                    let running = *obs_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = obs_clone.internal_tx.send(InternalEvent::HealthCheck);
                }
            });
        }

        // 指标收集任务
        {
            let obs_clone = Arc::clone(&observability);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30)); // 30秒

                loop {
                    interval.tick().await;
                    
                    let running = *obs_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = obs_clone.internal_tx.send(InternalEvent::MetricsCollection);
                }
            });
        }

        // 性能分析任务
        {
            let obs_clone = Arc::clone(&observability);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5分钟

                loop {
                    interval.tick().await;
                    
                    let running = *obs_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = obs_clone.internal_tx.send(InternalEvent::AnalysisRun);
                }
            });
        }

        info!("Background tasks started");
    }

    /// 执行健康检查
    async fn perform_health_check(&self) -> Result<()> {
        let health_status = self.health_checker.check_all_components().await?;
        
        for (component_name, health) in health_status {
            let status = ComponentStatus {
                component_name: component_name.clone(),
                health_status: health.status,
                last_check: Utc::now(),
                response_time_ms: health.response_time_ms,
                error_rate: health.error_rate,
                throughput: health.throughput,
                resource_usage: ResourceUsage {
                    cpu_percent: health.cpu_percent,
                    memory_percent: health.memory_percent,
                    disk_percent: health.disk_percent,
                    network_in_bps: health.network_in_bps,
                    network_out_bps: health.network_out_bps,
                },
            };
            
            self.internal_tx.send(InternalEvent::ComponentStatusUpdate(status))?;
        }

        Ok(())
    }

    /// 收集系统指标
    async fn collect_system_metrics(&self) -> Result<()> {
        // 收集系统级指标
        let cpu_usage = self.get_cpu_usage().await;
        let memory_usage = self.get_memory_usage().await;
        let disk_usage = self.get_disk_usage().await;

        // 记录系统指标
        self.record_metric("system.cpu.usage", cpu_usage, MetricType::Gauge, HashMap::new()).await?;
        self.record_metric("system.memory.usage", memory_usage, MetricType::Gauge, HashMap::new()).await?;
        self.record_metric("system.disk.usage", disk_usage, MetricType::Gauge, HashMap::new()).await?;

        Ok(())
    }

    /// 运行性能分析
    async fn run_performance_analysis(&self) -> Result<()> {
        // 检测性能瓶颈
        if let Some(bottleneck) = self.detect_performance_bottleneck().await {
            let insight = PerformanceInsight {
                insight_id: Uuid::new_v4().to_string(),
                insight_type: InsightType::PerformanceBottleneck,
                title: "Performance Bottleneck Detected".to_string(),
                description: bottleneck,
                severity: ObservabilitySeverity::High,
                affected_components: vec!["system".to_string()],
                metrics: HashMap::new(),
                recommendations: vec![
                    "Optimize database queries".to_string(),
                    "Scale horizontally".to_string(),
                    "Add caching layer".to_string(),
                ],
                detected_at: Utc::now(),
            };
            
            self.internal_tx.send(InternalEvent::PerformanceInsight(insight))?;
        }

        Ok(())
    }

    /// 获取CPU使用率（模拟）
    async fn get_cpu_usage(&self) -> f64 {
        // 在实际实现中，这里应该获取真实的CPU使用率
        rand::random::<f64>() * 100.0
    }

    /// 获取内存使用率（模拟）
    async fn get_memory_usage(&self) -> f64 {
        // 在实际实现中，这里应该获取真实的内存使用率
        rand::random::<f64>() * 100.0
    }

    /// 获取磁盘使用率（模拟）
    async fn get_disk_usage(&self) -> f64 {
        // 在实际实现中，这里应该获取真实的磁盘使用率
        rand::random::<f64>() * 100.0
    }

    /// 检测性能瓶颈（模拟）
    async fn detect_performance_bottleneck(&self) -> Option<String> {
        // 在实际实现中，这里应该分析各种指标来检测瓶颈
        if rand::random::<f64>() > 0.8 {
            Some("High database query latency detected".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ObservabilityConfig;

    #[tokio::test]
    async fn test_observability_system_creation() {
        let config = ObservabilityConfig::default();
        let observability = FullStackObservability::new(config).await;
        assert!(observability.is_ok());
    }

    #[tokio::test]
    async fn test_observability_lifecycle() {
        let config = ObservabilityConfig::default();
        let observability = FullStackObservability::new(config).await.unwrap();
        
        // 测试启动
        let start_result = observability.start().await;
        assert!(start_result.is_ok());
        
        // 测试停止
        let stop_result = observability.stop().await;
        assert!(stop_result.is_ok());
    }

    #[tokio::test]
    async fn test_metric_recording() {
        let config = ObservabilityConfig::default();
        let observability = FullStackObservability::new(config).await.unwrap();
        observability.start().await.unwrap();

        let mut tags = HashMap::new();
        tags.insert("component".to_string(), "test".to_string());

        let result = observability.record_metric(
            "test.metric",
            42.0,
            MetricType::Counter,
            tags
        ).await;
        assert!(result.is_ok());

        observability.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_span_lifecycle() {
        let config = ObservabilityConfig::default();
        let observability = FullStackObservability::new(config).await.unwrap();
        observability.start().await.unwrap();

        // 开始span
        let span_info = observability.start_span("test_operation", None).await;
        assert!(span_info.is_ok());

        let span_info = span_info.unwrap();
        assert!(!span_info.trace_id.is_empty());
        assert!(!span_info.span_id.is_empty());

        // 结束span
        let result = observability.finish_span(span_info, None).await;
        assert!(result.is_ok());

        observability.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = ObservabilityConfig::default();
        let observability = FullStackObservability::new(config).await.unwrap();

        let _receiver = observability.subscribe_events();
        // 订阅应该成功，不抛出错误
    }

    #[tokio::test]
    async fn test_system_stats() {
        let config = ObservabilityConfig::default();
        let observability = FullStackObservability::new(config).await.unwrap();

        let stats = observability.get_system_stats().await;
        assert!(stats.contains_key("running"));
        assert!(stats.contains_key("monitored_components"));
    }
}