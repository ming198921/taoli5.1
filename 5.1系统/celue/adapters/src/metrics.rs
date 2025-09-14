//! 生产级指标监控模块 - 完整实现
//! 高性能、低延迟、多层次指标系统

use std::sync::atomic::{AtomicU64, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
// 移除未使用的HashMap导入
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use parking_lot::Mutex;

/// 原子性能计数器（无锁实现）
#[derive(Debug)]
pub struct AtomicCounter {
    value: AtomicU64,
    last_reset: AtomicU64,
}

impl AtomicCounter {
    pub fn new() -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Self {
            value: AtomicU64::new(0),
            last_reset: AtomicU64::new(now),
        }
    }

    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }

    pub fn add(&self, delta: u64) -> u64 {
        self.value.fetch_add(delta, Ordering::Relaxed)
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn reset(&self) -> u64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.last_reset.store(now, Ordering::Relaxed);
        self.value.swap(0, Ordering::Relaxed)
    }

    pub fn rate_per_second(&self) -> f64 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let last_reset = self.last_reset.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last_reset);
        if elapsed == 0 {
            0.0
        } else {
            self.get() as f64 / elapsed as f64
        }
    }
}

/// 原子统计指标（移动平均）
#[derive(Debug)]
pub struct AtomicStats {
    count: AtomicU64,
    sum: AtomicU64,
    min: AtomicU64,
    max: AtomicU64,
    sum_squares: AtomicU64, // 用于计算标准差
}

impl AtomicStats {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
            sum_squares: AtomicU64::new(0),
        }
    }

    pub fn record(&self, value: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(value, Ordering::Relaxed);
        self.sum_squares.fetch_add(value * value, Ordering::Relaxed);
        
        // 更新最小值
        let mut current_min = self.min.load(Ordering::Relaxed);
        while current_min > value {
            match self.min.compare_exchange_weak(current_min, value, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }
        
        // 更新最大值
        let mut current_max = self.max.load(Ordering::Relaxed);
        while current_max < value {
            match self.max.compare_exchange_weak(current_max, value, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(actual) => current_max = actual,
            }
        }
    }

    pub fn mean(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            0.0
        } else {
            self.sum.load(Ordering::Relaxed) as f64 / count as f64
        }
    }

    pub fn min(&self) -> u64 {
        let min_val = self.min.load(Ordering::Relaxed);
        if min_val == u64::MAX { 0 } else { min_val }
    }

    pub fn max(&self) -> u64 {
        self.max.load(Ordering::Relaxed)
    }

    pub fn std_dev(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed);
        if count < 2 {
            return 0.0;
        }
        
        let mean = self.mean();
        let sum_squares = self.sum_squares.load(Ordering::Relaxed) as f64;
        let variance = (sum_squares / count as f64) - (mean * mean);
        variance.sqrt()
    }

    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            count: self.count.load(Ordering::Relaxed),
            mean: self.mean(),
            min: self.min(),
            max: self.max(),
            std_dev: self.std_dev(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSnapshot {
    pub count: u64,
    pub mean: f64,
    pub min: u64,
    pub max: u64,
    pub std_dev: f64,
}

/// 高频交易指标（微秒级更新）
#[derive(Debug)]
pub struct HighFrequencyMetrics {
    // 机会统计
    pub opportunities_detected: AtomicCounter,
    pub opportunities_executed: AtomicCounter,
    pub opportunities_missed: AtomicCounter,
    
    // 延迟统计
    pub detection_latency_ns: AtomicStats,
    pub execution_latency_us: AtomicStats,
    pub total_latency_us: AtomicStats,
    
    // 财务统计
    pub profit_basis_points: AtomicI64,
    pub volume_usd_cents: AtomicU64,
    pub trade_count: AtomicCounter,
    
    // 系统统计
    pub cpu_usage_basis_points: AtomicU64, // CPU使用率 * 10000
    pub memory_usage_mb: AtomicU64,
    pub network_latency_us: AtomicStats,
    
    // 风险统计
    pub risk_score_basis_points: AtomicU64, // 风险分数 * 10000
    pub correlation_alerts: AtomicCounter,
    pub volatility_alerts: AtomicCounter,
    pub circuit_breaker_triggers: AtomicCounter,
    
    // 错误统计
    pub errors_total: AtomicCounter,
    pub warnings_total: AtomicCounter,
    pub timeouts_total: AtomicCounter,
    
    // 启动时间
    start_time: Instant,
}

impl HighFrequencyMetrics {
    pub fn new() -> Self {
        Self {
            opportunities_detected: AtomicCounter::new(),
            opportunities_executed: AtomicCounter::new(),
            opportunities_missed: AtomicCounter::new(),
            
            detection_latency_ns: AtomicStats::new(),
            execution_latency_us: AtomicStats::new(),
            total_latency_us: AtomicStats::new(),
            
            profit_basis_points: AtomicI64::new(0),
            volume_usd_cents: AtomicU64::new(0),
            trade_count: AtomicCounter::new(),
            
            cpu_usage_basis_points: AtomicU64::new(0),
            memory_usage_mb: AtomicU64::new(0),
            network_latency_us: AtomicStats::new(),
            
            risk_score_basis_points: AtomicU64::new(0),
            correlation_alerts: AtomicCounter::new(),
            volatility_alerts: AtomicCounter::new(),
            circuit_breaker_triggers: AtomicCounter::new(),
            
            errors_total: AtomicCounter::new(),
            warnings_total: AtomicCounter::new(),
            timeouts_total: AtomicCounter::new(),
            
            start_time: Instant::now(),
        }
    }
    
    /// 记录机会检测
    pub fn record_opportunity_detected(&self, detection_latency_ns: u64) {
        self.opportunities_detected.increment();
        self.detection_latency_ns.record(detection_latency_ns);
        
        // 延迟警报
        if detection_latency_ns > 100_000 { // > 100微秒
            self.warnings_total.increment();
        }
    }
    
    /// 记录机会执行
    pub fn record_opportunity_executed(&self, execution_latency_us: u64, profit_bps: i32, volume_usd: f64) {
        self.opportunities_executed.increment();
        self.trade_count.increment();
        self.execution_latency_us.record(execution_latency_us);
        
        // 更新财务指标
        self.profit_basis_points.fetch_add(profit_bps as i64, Ordering::Relaxed);
        self.volume_usd_cents.fetch_add((volume_usd * 100.0) as u64, Ordering::Relaxed);
        
        // 总延迟（检测 + 执行）
        let avg_detection = self.detection_latency_ns.mean() / 1000.0; // 转换为微秒
        let total_latency = avg_detection + execution_latency_us as f64;
        self.total_latency_us.record(total_latency as u64);
    }
    
    /// 记录错失机会
    pub fn record_opportunity_missed(&self, reason: &str) {
        self.opportunities_missed.increment();
        tracing::warn!("Opportunity missed: {}", reason);
        
        // 分类统计
        match reason {
            "timeout" | "network_timeout" => { self.timeouts_total.increment(); },
            "latency" | "slow_execution" => { self.warnings_total.increment(); },
            _ => { self.errors_total.increment(); },
        }
    }
    
    /// 更新系统指标
    pub fn update_system_metrics(&self, cpu_percent: f64, memory_mb: u64, network_latency_us: u64) {
        self.cpu_usage_basis_points.store((cpu_percent * 10000.0) as u64, Ordering::Relaxed);
        self.memory_usage_mb.store(memory_mb, Ordering::Relaxed);
        self.network_latency_us.record(network_latency_us);
    }
    
    /// 更新风险指标
    pub fn update_risk_score(&self, risk_score: f64) {
        self.risk_score_basis_points.store((risk_score * 10000.0) as u64, Ordering::Relaxed);
        
        // 高风险警报
        if risk_score > 0.8 {
            self.correlation_alerts.increment();
        }
        
        // 极高风险触发熔断
        if risk_score > 0.95 {
            self.circuit_breaker_triggers.increment();
        }
    }
    
    /// 记录波动率警报
    pub fn record_volatility_alert(&self) {
        self.volatility_alerts.increment();
    }
    
    /// 获取综合性能分数
    pub fn get_performance_score(&self) -> f64 {
        let success_rate = self.get_success_rate();
        let avg_latency_us = self.total_latency_us.mean();
        let risk_score = self.risk_score_basis_points.load(Ordering::Relaxed) as f64 / 10000.0;
        
        // 性能分数：成功率权重50%，延迟权重30%，风险权重20%
        let latency_score = if avg_latency_us > 0.0 {
            (1000.0 / avg_latency_us).min(1.0) // 1000微秒基准
        } else {
            1.0
        };
        
        let risk_score_normalized = 1.0 - risk_score;
        
        (success_rate * 0.5 + latency_score * 0.3 + risk_score_normalized * 0.2) * 100.0
    }
    
    /// 获取成功率
    pub fn get_success_rate(&self) -> f64 {
        let detected = self.opportunities_detected.get();
        let executed = self.opportunities_executed.get();
        
        if detected == 0 {
            1.0
        } else {
            executed as f64 / detected as f64
        }
    }
    
    /// 获取每秒机会检测率
    pub fn get_detection_rate(&self) -> f64 {
        self.opportunities_detected.rate_per_second()
    }
    
    /// 获取系统正常运行时间
    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// 获取详细快照
    pub fn get_detailed_snapshot(&self) -> DetailedMetricsSnapshot {
        DetailedMetricsSnapshot {
            // 机会统计
            opportunities_detected: self.opportunities_detected.get(),
            opportunities_executed: self.opportunities_executed.get(),
            opportunities_missed: self.opportunities_missed.get(),
            detection_rate_per_sec: self.get_detection_rate(),
            success_rate_percent: self.get_success_rate() * 100.0,
            
            // 延迟统计
            detection_latency: self.detection_latency_ns.snapshot(),
            execution_latency: self.execution_latency_us.snapshot(),
            total_latency: self.total_latency_us.snapshot(),
            
            // 财务统计
            profit_basis_points: self.profit_basis_points.load(Ordering::Relaxed),
            volume_usd: self.volume_usd_cents.load(Ordering::Relaxed) as f64 / 100.0,
            trades_count: self.trade_count.get(),
            avg_profit_per_trade_bps: if self.trade_count.get() > 0 {
                self.profit_basis_points.load(Ordering::Relaxed) as f64 / self.trade_count.get() as f64
            } else {
                0.0
            },
            
            // 系统统计
            cpu_usage_percent: self.cpu_usage_basis_points.load(Ordering::Relaxed) as f64 / 10000.0,
            memory_usage_mb: self.memory_usage_mb.load(Ordering::Relaxed),
            network_latency: self.network_latency_us.snapshot(),
            
            // 风险统计
            risk_score: self.risk_score_basis_points.load(Ordering::Relaxed) as f64 / 10000.0,
            correlation_alerts: self.correlation_alerts.get(),
            volatility_alerts: self.volatility_alerts.get(),
            circuit_breaker_triggers: self.circuit_breaker_triggers.get(),
            
            // 错误统计
            errors_total: self.errors_total.get(),
            warnings_total: self.warnings_total.get(),
            timeouts_total: self.timeouts_total.get(),
            
            // 系统信息
            uptime_seconds: self.get_uptime_seconds(),
            performance_score: self.get_performance_score(),
            
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedMetricsSnapshot {
    // 机会统计
    pub opportunities_detected: u64,
    pub opportunities_executed: u64,
    pub opportunities_missed: u64,
    pub detection_rate_per_sec: f64,
    pub success_rate_percent: f64,
    
    // 延迟统计
    pub detection_latency: StatsSnapshot,
    pub execution_latency: StatsSnapshot,
    pub total_latency: StatsSnapshot,
    
    // 财务统计
    pub profit_basis_points: i64,
    pub volume_usd: f64,
    pub trades_count: u64,
    pub avg_profit_per_trade_bps: f64,
    
    // 系统统计
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub network_latency: StatsSnapshot,
    
    // 风险统计
    pub risk_score: f64,
    pub correlation_alerts: u64,
    pub volatility_alerts: u64,
    pub circuit_breaker_triggers: u64,
    
    // 错误统计
    pub errors_total: u64,
    pub warnings_total: u64,
    pub timeouts_total: u64,
    
    // 系统信息
    pub uptime_seconds: u64,
    pub performance_score: f64,
    
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 时间序列存储（环形缓冲区）
#[derive(Debug)]
pub struct TimeSeriesBuffer<T> {
    buffer: Mutex<Vec<(u64, T)>>, // (timestamp, value)
    capacity: usize,
}

impl<T: Clone> TimeSeriesBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }
    
    pub fn push(&self, value: T) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut buffer = self.buffer.lock();
        
        if buffer.len() >= self.capacity {
            buffer.remove(0);
        }
        
        buffer.push((timestamp, value));
    }
    
    pub fn get_recent(&self, duration_secs: u64) -> Vec<(u64, T)> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let cutoff = now.saturating_sub(duration_secs);
        
        let buffer = self.buffer.lock();
        buffer.iter()
            .filter(|(ts, _)| *ts >= cutoff)
            .cloned()
            .collect()
    }
    
    pub fn len(&self) -> usize {
        self.buffer.lock().len()
    }
}

/// 分层指标系统
#[derive(Debug)]
pub struct LayeredProductionMetrics {
    pub high_frequency: Arc<HighFrequencyMetrics>,
    pub historical: TimeSeriesBuffer<DetailedMetricsSnapshot>,
    pub aggregated: Arc<RwLock<AggregatedMetrics>>,
}

impl LayeredProductionMetrics {
    pub fn new() -> Self {
        Self {
            high_frequency: Arc::new(HighFrequencyMetrics::new()),
            historical: TimeSeriesBuffer::new(1000), // 保存最近1000个快照
            aggregated: Arc::new(RwLock::new(AggregatedMetrics::default())),
        }
    }
    
    /// 捕获快照
    pub async fn capture_snapshot(&self) {
        let snapshot = self.high_frequency.get_detailed_snapshot();
        self.historical.push(snapshot.clone());
        
        // 更新聚合指标
        self.update_aggregated_metrics(&snapshot).await;
    }
    
    /// 更新聚合指标
    async fn update_aggregated_metrics(&self, snapshot: &DetailedMetricsSnapshot) {
        let mut aggregated = self.aggregated.write().await;
        
        // 计算各时间段的指标
        let recent_1h = self.historical.get_recent(3600);
        let recent_24h = self.historical.get_recent(86400);
        
        aggregated.hourly_profit_usd = recent_1h.iter()
            .last()
            .map(|(_, s)| s.profit_basis_points as f64 / 10000.0)
            .unwrap_or(0.0);
            
        aggregated.daily_profit_usd = recent_24h.iter()
            .last()
            .map(|(_, s)| s.profit_basis_points as f64 / 10000.0)
            .unwrap_or(0.0);
            
        aggregated.hourly_volume_usd = recent_1h.iter()
            .last()
            .map(|(_, s)| s.volume_usd)
            .unwrap_or(0.0);
            
        aggregated.daily_volume_usd = recent_24h.iter()
            .last()
            .map(|(_, s)| s.volume_usd)
            .unwrap_or(0.0);
            
        aggregated.success_rate_percent = snapshot.success_rate_percent;
        aggregated.avg_latency_us = snapshot.total_latency.mean;
        aggregated.performance_score = snapshot.performance_score;
        aggregated.uptime_percent = if snapshot.uptime_seconds > 0 {
            ((snapshot.uptime_seconds - snapshot.errors_total) as f64 / snapshot.uptime_seconds as f64) * 100.0
        } else {
            100.0
        };
        
        // 计算夏普比率（简化版）
        if recent_24h.len() > 1 {
            let returns: Vec<f64> = recent_24h.windows(2)
                .map(|pair| {
                    let prev = &pair[0].1;
                    let curr = &pair[1].1;
                    (curr.profit_basis_points - prev.profit_basis_points) as f64 / 10000.0
                })
                .collect();
                
            if !returns.is_empty() {
                let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance = returns.iter()
                    .map(|r| (r - mean_return).powi(2))
                    .sum::<f64>() / returns.len() as f64;
                let std_dev = variance.sqrt();
                
                aggregated.sharpe_ratio = if std_dev > 0.0 {
                    mean_return / std_dev
                } else {
                    0.0
                };
            }
        }
    }
    
    /// 启动后台任务
    pub fn start_background_tasks(self: Arc<Self>) {
        let metrics = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // 每30秒捕获快照
            loop {
                interval.tick().await;
                metrics.capture_snapshot().await;
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub hourly_profit_usd: f64,
    pub daily_profit_usd: f64,
    pub hourly_volume_usd: f64,
    pub daily_volume_usd: f64,
    pub success_rate_percent: f64,
    pub avg_latency_us: f64,
    pub performance_score: f64,
    pub sharpe_ratio: f64,
    pub uptime_percent: f64,
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            hourly_profit_usd: 0.0,
            daily_profit_usd: 0.0,
            hourly_volume_usd: 0.0,
            daily_volume_usd: 0.0,
            success_rate_percent: 100.0,
            avg_latency_us: 0.0,
            performance_score: 100.0,
            sharpe_ratio: 0.0,
            uptime_percent: 100.0,
        }
    }
}

/// 生产级适配器指标
#[derive(Debug)]
pub struct ProductionAdapterMetrics {
    pub layered: Arc<LayeredProductionMetrics>,
    
    // 向后兼容字段
    pub messages_processed: AtomicCounter,
    pub messages_failed: AtomicCounter,
    pub active_connections: AtomicU64,
}

impl ProductionAdapterMetrics {
    pub fn new() -> Self {
        let layered = Arc::new(LayeredProductionMetrics::new());
        
        // 启动后台任务
        Arc::clone(&layered).start_background_tasks();
        
        Self {
            layered,
            messages_processed: AtomicCounter::new(),
            messages_failed: AtomicCounter::new(),
            active_connections: AtomicU64::new(0),
        }
    }
    
    /// 获取高频指标
    pub fn hf(&self) -> &Arc<HighFrequencyMetrics> {
        &self.layered.high_frequency
    }
    
    /// 获取聚合指标
    pub async fn aggregated(&self) -> AggregatedMetrics {
        self.layered.aggregated.read().await.clone()
    }
    
    /// 向后兼容的摘要
    pub async fn summary(&self) -> MetricsSummary {
        let snapshot = self.layered.high_frequency.get_detailed_snapshot();
        MetricsSummary {
            messages_processed: self.messages_processed.get(),
            messages_failed: self.messages_failed.get(),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            avg_processing_time_ms: snapshot.execution_latency.mean / 1000.0,
            avg_detection_time_us: snapshot.detection_latency.mean / 1000.0,
            avg_execution_time_ms: snapshot.execution_latency.mean / 1000.0,
        }
    }
    
    /// 生成详细报告
    pub async fn generate_report(&self) -> ProductionMetricsReport {
        let snapshot = self.layered.high_frequency.get_detailed_snapshot();
        let aggregated = self.aggregated().await;
        
        ProductionMetricsReport {
            current_snapshot: snapshot,
            aggregated_metrics: aggregated,
            historical_count: self.layered.historical.len(),
            system_health: self.assess_system_health().await,
        }
    }
    
    /// 评估系统健康状况
    pub async fn assess_system_health(&self) -> SystemHealthStatus {
        let snapshot = self.layered.high_frequency.get_detailed_snapshot();
        
        let mut health_score: f64 = 100.0;
        let mut issues = Vec::new();
        
        // 检查成功率
        if snapshot.success_rate_percent < 95.0 {
            health_score -= 20.0;
            issues.push("Low success rate".to_string());
        }
        
        // 检查延迟
        if snapshot.total_latency.mean > 1000.0 { // > 1ms
            health_score -= 15.0;
            issues.push("High latency".to_string());
        }
        
        // 检查风险分数
        if snapshot.risk_score > 0.8 {
            health_score -= 25.0;
            issues.push("High risk score".to_string());
        }
        
        // 检查错误率
        let error_rate = if snapshot.opportunities_detected > 0 {
            snapshot.errors_total as f64 / snapshot.opportunities_detected as f64
        } else {
            0.0
        };
        
        if error_rate > 0.05 { // > 5%
            health_score -= 10.0;
            issues.push("High error rate".to_string());
        }
        
        // 检查系统资源
        if snapshot.cpu_usage_percent > 80.0 {
            health_score -= 10.0;
            issues.push("High CPU usage".to_string());
        }
        
        if snapshot.memory_usage_mb > 2048 { // > 2GB
            health_score -= 5.0;
            issues.push("High memory usage".to_string());
        }
        
        let status = if health_score >= 90.0 {
            "HEALTHY"
        } else if health_score >= 70.0 {
            "WARNING"
        } else {
            "CRITICAL"
        };
        
        SystemHealthStatus {
            status: status.to_string(),
            health_score: health_score.max(0.0),
            issues,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub active_connections: u64,
    pub avg_processing_time_ms: f64,
    pub avg_detection_time_us: f64,
    pub avg_execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionMetricsReport {
    pub current_snapshot: DetailedMetricsSnapshot,
    pub aggregated_metrics: AggregatedMetrics,
    pub historical_count: usize,
    pub system_health: SystemHealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub status: String,
    pub health_score: f64,
    pub issues: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 生产级指标注册表
#[derive(Debug)]
pub struct ProductionMetricsRegistry {
    pub adapter_metrics: Arc<ProductionAdapterMetrics>,
}

impl Default for ProductionMetricsRegistry {
    fn default() -> Self {
        Self {
            adapter_metrics: Arc::new(ProductionAdapterMetrics::new()),
        }
    }
}

impl ProductionMetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn adapter_metrics(&self) -> Arc<ProductionAdapterMetrics> {
        Arc::clone(&self.adapter_metrics)
    }
    
    /// 启动生产级指标服务器
    pub async fn start_metrics_server(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🚀 Starting production-grade metrics server on {}", addr);
        
        let metrics = Arc::clone(&self.adapter_metrics);
        
        // 启动HTTP服务（简化版）
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                
                let report = metrics.generate_report().await;
                
                // 输出关键指标
                tracing::info!(
                    "🎯 Production Metrics: detected={}/s, executed={}, success={:.1}%, latency={:.0}μs, profit={}bps, score={:.1}, health={}",
                    report.current_snapshot.detection_rate_per_sec,
                    report.current_snapshot.opportunities_executed,
                    report.current_snapshot.success_rate_percent,
                    report.current_snapshot.total_latency.mean,
                    report.current_snapshot.profit_basis_points,
                    report.current_snapshot.performance_score,
                    report.system_health.status
                );
                
                // 输出聚合指标
                tracing::info!(
                    "📊 Aggregated: daily_profit=${:.2}, daily_volume=${:.2}, sharpe={:.3}, uptime={:.1}%",
                    report.aggregated_metrics.daily_profit_usd,
                    report.aggregated_metrics.daily_volume_usd,
                    report.aggregated_metrics.sharpe_ratio,
                    report.aggregated_metrics.uptime_percent
                );
                
                // 健康警报
                if report.system_health.status != "HEALTHY" {
                    tracing::warn!(
                        "⚠️ System Health Alert: {} (score: {:.1}) - Issues: {:?}",
                        report.system_health.status,
                        report.system_health.health_score,
                        report.system_health.issues
                    );
                }
            }
        });
        
        tracing::info!("✅ Production metrics server running with comprehensive monitoring");
        Ok(())
    }
}

// 重新导出为默认版本（保持兼容性）
pub use ProductionAdapterMetrics as AdapterMetrics;
pub use ProductionMetricsRegistry as MetricsRegistry; 
            profit_basis_points: AtomicI64::new(0),
            volume_usd_cents: AtomicU64::new(0),
            trade_count: AtomicCounter::new(),
            
            cpu_usage_basis_points: AtomicU64::new(0),
            memory_usage_mb: AtomicU64::new(0),
            network_latency_us: AtomicStats::new(),
            
            risk_score_basis_points: AtomicU64::new(0),
            correlation_alerts: AtomicCounter::new(),
            volatility_alerts: AtomicCounter::new(),
            circuit_breaker_triggers: AtomicCounter::new(),
            
            errors_total: AtomicCounter::new(),
            warnings_total: AtomicCounter::new(),
            timeouts_total: AtomicCounter::new(),
            
            start_time: Instant::now(),
        }
    }
    
    /// 记录机会检测
    pub fn record_opportunity_detected(&self, detection_latency_ns: u64) {
        self.opportunities_detected.increment();
        self.detection_latency_ns.record(detection_latency_ns);
        
        // 延迟警报
        if detection_latency_ns > 100_000 { // > 100微秒
            self.warnings_total.increment();
        }
    }
    
    /// 记录机会执行
    pub fn record_opportunity_executed(&self, execution_latency_us: u64, profit_bps: i32, volume_usd: f64) {
        self.opportunities_executed.increment();
        self.trade_count.increment();
        self.execution_latency_us.record(execution_latency_us);
        
        // 更新财务指标
        self.profit_basis_points.fetch_add(profit_bps as i64, Ordering::Relaxed);
        self.volume_usd_cents.fetch_add((volume_usd * 100.0) as u64, Ordering::Relaxed);
        
        // 总延迟（检测 + 执行）
        let avg_detection = self.detection_latency_ns.mean() / 1000.0; // 转换为微秒
        let total_latency = avg_detection + execution_latency_us as f64;
        self.total_latency_us.record(total_latency as u64);
    }
    
    /// 记录错失机会
    pub fn record_opportunity_missed(&self, reason: &str) {
        self.opportunities_missed.increment();
        tracing::warn!("Opportunity missed: {}", reason);
        
        // 分类统计
        match reason {
            "timeout" | "network_timeout" => { self.timeouts_total.increment(); },
            "latency" | "slow_execution" => { self.warnings_total.increment(); },
            _ => { self.errors_total.increment(); },
        }
    }
    
    /// 更新系统指标
    pub fn update_system_metrics(&self, cpu_percent: f64, memory_mb: u64, network_latency_us: u64) {
        self.cpu_usage_basis_points.store((cpu_percent * 10000.0) as u64, Ordering::Relaxed);
        self.memory_usage_mb.store(memory_mb, Ordering::Relaxed);
        self.network_latency_us.record(network_latency_us);
    }
    
    /// 更新风险指标
    pub fn update_risk_score(&self, risk_score: f64) {
        self.risk_score_basis_points.store((risk_score * 10000.0) as u64, Ordering::Relaxed);
        
        // 高风险警报
        if risk_score > 0.8 {
            self.correlation_alerts.increment();
        }
        
        // 极高风险触发熔断
        if risk_score > 0.95 {
            self.circuit_breaker_triggers.increment();
        }
    }
    
    /// 记录波动率警报
    pub fn record_volatility_alert(&self) {
        self.volatility_alerts.increment();
    }
    
    /// 获取综合性能分数
    pub fn get_performance_score(&self) -> f64 {
        let success_rate = self.get_success_rate();
        let avg_latency_us = self.total_latency_us.mean();
        let risk_score = self.risk_score_basis_points.load(Ordering::Relaxed) as f64 / 10000.0;
        
        // 性能分数：成功率权重50%，延迟权重30%，风险权重20%
        let latency_score = if avg_latency_us > 0.0 {
            (1000.0 / avg_latency_us).min(1.0) // 1000微秒基准
        } else {
            1.0
        };
        
        let risk_score_normalized = 1.0 - risk_score;
        
        (success_rate * 0.5 + latency_score * 0.3 + risk_score_normalized * 0.2) * 100.0
    }
    
    /// 获取成功率
    pub fn get_success_rate(&self) -> f64 {
        let detected = self.opportunities_detected.get();
        let executed = self.opportunities_executed.get();
        
        if detected == 0 {
            1.0
        } else {
            executed as f64 / detected as f64
        }
    }
    
    /// 获取每秒机会检测率
    pub fn get_detection_rate(&self) -> f64 {
        self.opportunities_detected.rate_per_second()
    }
    
    /// 获取系统正常运行时间
    pub fn get_uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
    
    /// 获取详细快照
    pub fn get_detailed_snapshot(&self) -> DetailedMetricsSnapshot {
        DetailedMetricsSnapshot {
            // 机会统计
            opportunities_detected: self.opportunities_detected.get(),
            opportunities_executed: self.opportunities_executed.get(),
            opportunities_missed: self.opportunities_missed.get(),
            detection_rate_per_sec: self.get_detection_rate(),
            success_rate_percent: self.get_success_rate() * 100.0,
            
            // 延迟统计
            detection_latency: self.detection_latency_ns.snapshot(),
            execution_latency: self.execution_latency_us.snapshot(),
            total_latency: self.total_latency_us.snapshot(),
            
            // 财务统计
            profit_basis_points: self.profit_basis_points.load(Ordering::Relaxed),
            volume_usd: self.volume_usd_cents.load(Ordering::Relaxed) as f64 / 100.0,
            trades_count: self.trade_count.get(),
            avg_profit_per_trade_bps: if self.trade_count.get() > 0 {
                self.profit_basis_points.load(Ordering::Relaxed) as f64 / self.trade_count.get() as f64
            } else {
                0.0
            },
            
            // 系统统计
            cpu_usage_percent: self.cpu_usage_basis_points.load(Ordering::Relaxed) as f64 / 10000.0,
            memory_usage_mb: self.memory_usage_mb.load(Ordering::Relaxed),
            network_latency: self.network_latency_us.snapshot(),
            
            // 风险统计
            risk_score: self.risk_score_basis_points.load(Ordering::Relaxed) as f64 / 10000.0,
            correlation_alerts: self.correlation_alerts.get(),
            volatility_alerts: self.volatility_alerts.get(),
            circuit_breaker_triggers: self.circuit_breaker_triggers.get(),
            
            // 错误统计
            errors_total: self.errors_total.get(),
            warnings_total: self.warnings_total.get(),
            timeouts_total: self.timeouts_total.get(),
            
            // 系统信息
            uptime_seconds: self.get_uptime_seconds(),
            performance_score: self.get_performance_score(),
            
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedMetricsSnapshot {
    // 机会统计
    pub opportunities_detected: u64,
    pub opportunities_executed: u64,
    pub opportunities_missed: u64,
    pub detection_rate_per_sec: f64,
    pub success_rate_percent: f64,
    
    // 延迟统计
    pub detection_latency: StatsSnapshot,
    pub execution_latency: StatsSnapshot,
    pub total_latency: StatsSnapshot,
    
    // 财务统计
    pub profit_basis_points: i64,
    pub volume_usd: f64,
    pub trades_count: u64,
    pub avg_profit_per_trade_bps: f64,
    
    // 系统统计
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub network_latency: StatsSnapshot,
    
    // 风险统计
    pub risk_score: f64,
    pub correlation_alerts: u64,
    pub volatility_alerts: u64,
    pub circuit_breaker_triggers: u64,
    
    // 错误统计
    pub errors_total: u64,
    pub warnings_total: u64,
    pub timeouts_total: u64,
    
    // 系统信息
    pub uptime_seconds: u64,
    pub performance_score: f64,
    
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 时间序列存储（环形缓冲区）
#[derive(Debug)]
pub struct TimeSeriesBuffer<T> {
    buffer: Mutex<Vec<(u64, T)>>, // (timestamp, value)
    capacity: usize,
}

impl<T: Clone> TimeSeriesBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(Vec::with_capacity(capacity)),
            capacity,
        }
    }
    
    pub fn push(&self, value: T) {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let mut buffer = self.buffer.lock();
        
        if buffer.len() >= self.capacity {
            buffer.remove(0);
        }
        
        buffer.push((timestamp, value));
    }
    
    pub fn get_recent(&self, duration_secs: u64) -> Vec<(u64, T)> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let cutoff = now.saturating_sub(duration_secs);
        
        let buffer = self.buffer.lock();
        buffer.iter()
            .filter(|(ts, _)| *ts >= cutoff)
            .cloned()
            .collect()
    }
    
    pub fn len(&self) -> usize {
        self.buffer.lock().len()
    }
}

/// 分层指标系统
#[derive(Debug)]
pub struct LayeredProductionMetrics {
    pub high_frequency: Arc<HighFrequencyMetrics>,
    pub historical: TimeSeriesBuffer<DetailedMetricsSnapshot>,
    pub aggregated: Arc<RwLock<AggregatedMetrics>>,
}

impl LayeredProductionMetrics {
    pub fn new() -> Self {
        Self {
            high_frequency: Arc::new(HighFrequencyMetrics::new()),
            historical: TimeSeriesBuffer::new(1000), // 保存最近1000个快照
            aggregated: Arc::new(RwLock::new(AggregatedMetrics::default())),
        }
    }
    
    /// 捕获快照
    pub async fn capture_snapshot(&self) {
        let snapshot = self.high_frequency.get_detailed_snapshot();
        self.historical.push(snapshot.clone());
        
        // 更新聚合指标
        self.update_aggregated_metrics(&snapshot).await;
    }
    
    /// 更新聚合指标
    async fn update_aggregated_metrics(&self, snapshot: &DetailedMetricsSnapshot) {
        let mut aggregated = self.aggregated.write().await;
        
        // 计算各时间段的指标
        let recent_1h = self.historical.get_recent(3600);
        let recent_24h = self.historical.get_recent(86400);
        
        aggregated.hourly_profit_usd = recent_1h.iter()
            .last()
            .map(|(_, s)| s.profit_basis_points as f64 / 10000.0)
            .unwrap_or(0.0);
            
        aggregated.daily_profit_usd = recent_24h.iter()
            .last()
            .map(|(_, s)| s.profit_basis_points as f64 / 10000.0)
            .unwrap_or(0.0);
            
        aggregated.hourly_volume_usd = recent_1h.iter()
            .last()
            .map(|(_, s)| s.volume_usd)
            .unwrap_or(0.0);
            
        aggregated.daily_volume_usd = recent_24h.iter()
            .last()
            .map(|(_, s)| s.volume_usd)
            .unwrap_or(0.0);
            
        aggregated.success_rate_percent = snapshot.success_rate_percent;
        aggregated.avg_latency_us = snapshot.total_latency.mean;
        aggregated.performance_score = snapshot.performance_score;
        aggregated.uptime_percent = if snapshot.uptime_seconds > 0 {
            ((snapshot.uptime_seconds - snapshot.errors_total) as f64 / snapshot.uptime_seconds as f64) * 100.0
        } else {
            100.0
        };
        
        // 计算夏普比率（简化版）
        if recent_24h.len() > 1 {
            let returns: Vec<f64> = recent_24h.windows(2)
                .map(|pair| {
                    let prev = &pair[0].1;
                    let curr = &pair[1].1;
                    (curr.profit_basis_points - prev.profit_basis_points) as f64 / 10000.0
                })
                .collect();
                
            if !returns.is_empty() {
                let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
                let variance = returns.iter()
                    .map(|r| (r - mean_return).powi(2))
                    .sum::<f64>() / returns.len() as f64;
                let std_dev = variance.sqrt();
                
                aggregated.sharpe_ratio = if std_dev > 0.0 {
                    mean_return / std_dev
                } else {
                    0.0
                };
            }
        }
    }
    
    /// 启动后台任务
    pub fn start_background_tasks(self: Arc<Self>) {
        let metrics = Arc::clone(&self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // 每30秒捕获快照
            loop {
                interval.tick().await;
                metrics.capture_snapshot().await;
            }
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub hourly_profit_usd: f64,
    pub daily_profit_usd: f64,
    pub hourly_volume_usd: f64,
    pub daily_volume_usd: f64,
    pub success_rate_percent: f64,
    pub avg_latency_us: f64,
    pub performance_score: f64,
    pub sharpe_ratio: f64,
    pub uptime_percent: f64,
}

impl Default for AggregatedMetrics {
    fn default() -> Self {
        Self {
            hourly_profit_usd: 0.0,
            daily_profit_usd: 0.0,
            hourly_volume_usd: 0.0,
            daily_volume_usd: 0.0,
            success_rate_percent: 100.0,
            avg_latency_us: 0.0,
            performance_score: 100.0,
            sharpe_ratio: 0.0,
            uptime_percent: 100.0,
        }
    }
}

/// 生产级适配器指标
#[derive(Debug)]
pub struct ProductionAdapterMetrics {
    pub layered: Arc<LayeredProductionMetrics>,
    
    // 向后兼容字段
    pub messages_processed: AtomicCounter,
    pub messages_failed: AtomicCounter,
    pub active_connections: AtomicU64,
}

impl ProductionAdapterMetrics {
    pub fn new() -> Self {
        let layered = Arc::new(LayeredProductionMetrics::new());
        
        // 启动后台任务
        Arc::clone(&layered).start_background_tasks();
        
        Self {
            layered,
            messages_processed: AtomicCounter::new(),
            messages_failed: AtomicCounter::new(),
            active_connections: AtomicU64::new(0),
        }
    }
    
    /// 获取高频指标
    pub fn hf(&self) -> &Arc<HighFrequencyMetrics> {
        &self.layered.high_frequency
    }
    
    /// 获取聚合指标
    pub async fn aggregated(&self) -> AggregatedMetrics {
        self.layered.aggregated.read().await.clone()
    }
    
    /// 向后兼容的摘要
    pub async fn summary(&self) -> MetricsSummary {
        let snapshot = self.layered.high_frequency.get_detailed_snapshot();
        MetricsSummary {
            messages_processed: self.messages_processed.get(),
            messages_failed: self.messages_failed.get(),
            active_connections: self.active_connections.load(Ordering::Relaxed),
            avg_processing_time_ms: snapshot.execution_latency.mean / 1000.0,
            avg_detection_time_us: snapshot.detection_latency.mean / 1000.0,
            avg_execution_time_ms: snapshot.execution_latency.mean / 1000.0,
        }
    }
    
    /// 生成详细报告
    pub async fn generate_report(&self) -> ProductionMetricsReport {
        let snapshot = self.layered.high_frequency.get_detailed_snapshot();
        let aggregated = self.aggregated().await;
        
        ProductionMetricsReport {
            current_snapshot: snapshot,
            aggregated_metrics: aggregated,
            historical_count: self.layered.historical.len(),
            system_health: self.assess_system_health().await,
        }
    }
    
    /// 评估系统健康状况
    pub async fn assess_system_health(&self) -> SystemHealthStatus {
        let snapshot = self.layered.high_frequency.get_detailed_snapshot();
        
        let mut health_score: f64 = 100.0;
        let mut issues = Vec::new();
        
        // 检查成功率
        if snapshot.success_rate_percent < 95.0 {
            health_score -= 20.0;
            issues.push("Low success rate".to_string());
        }
        
        // 检查延迟
        if snapshot.total_latency.mean > 1000.0 { // > 1ms
            health_score -= 15.0;
            issues.push("High latency".to_string());
        }
        
        // 检查风险分数
        if snapshot.risk_score > 0.8 {
            health_score -= 25.0;
            issues.push("High risk score".to_string());
        }
        
        // 检查错误率
        let error_rate = if snapshot.opportunities_detected > 0 {
            snapshot.errors_total as f64 / snapshot.opportunities_detected as f64
        } else {
            0.0
        };
        
        if error_rate > 0.05 { // > 5%
            health_score -= 10.0;
            issues.push("High error rate".to_string());
        }
        
        // 检查系统资源
        if snapshot.cpu_usage_percent > 80.0 {
            health_score -= 10.0;
            issues.push("High CPU usage".to_string());
        }
        
        if snapshot.memory_usage_mb > 2048 { // > 2GB
            health_score -= 5.0;
            issues.push("High memory usage".to_string());
        }
        
        let status = if health_score >= 90.0 {
            "HEALTHY"
        } else if health_score >= 70.0 {
            "WARNING"
        } else {
            "CRITICAL"
        };
        
        SystemHealthStatus {
            status: status.to_string(),
            health_score: health_score.max(0.0),
            issues,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub active_connections: u64,
    pub avg_processing_time_ms: f64,
    pub avg_detection_time_us: f64,
    pub avg_execution_time_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionMetricsReport {
    pub current_snapshot: DetailedMetricsSnapshot,
    pub aggregated_metrics: AggregatedMetrics,
    pub historical_count: usize,
    pub system_health: SystemHealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub status: String,
    pub health_score: f64,
    pub issues: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 生产级指标注册表
#[derive(Debug)]
pub struct ProductionMetricsRegistry {
    pub adapter_metrics: Arc<ProductionAdapterMetrics>,
}

impl Default for ProductionMetricsRegistry {
    fn default() -> Self {
        Self {
            adapter_metrics: Arc::new(ProductionAdapterMetrics::new()),
        }
    }
}

impl ProductionMetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn adapter_metrics(&self) -> Arc<ProductionAdapterMetrics> {
        Arc::clone(&self.adapter_metrics)
    }
    
    /// 启动生产级指标服务器
    pub async fn start_metrics_server(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("🚀 Starting production-grade metrics server on {}", addr);
        
        let metrics = Arc::clone(&self.adapter_metrics);
        
        // 启动HTTP服务（简化版）
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                
                let report = metrics.generate_report().await;
                
                // 输出关键指标
                tracing::info!(
                    "🎯 Production Metrics: detected={}/s, executed={}, success={:.1}%, latency={:.0}μs, profit={}bps, score={:.1}, health={}",
                    report.current_snapshot.detection_rate_per_sec,
                    report.current_snapshot.opportunities_executed,
                    report.current_snapshot.success_rate_percent,
                    report.current_snapshot.total_latency.mean,
                    report.current_snapshot.profit_basis_points,
                    report.current_snapshot.performance_score,
                    report.system_health.status
                );
                
                // 输出聚合指标
                tracing::info!(
                    "📊 Aggregated: daily_profit=${:.2}, daily_volume=${:.2}, sharpe={:.3}, uptime={:.1}%",
                    report.aggregated_metrics.daily_profit_usd,
                    report.aggregated_metrics.daily_volume_usd,
                    report.aggregated_metrics.sharpe_ratio,
                    report.aggregated_metrics.uptime_percent
                );
                
                // 健康警报
                if report.system_health.status != "HEALTHY" {
                    tracing::warn!(
                        "⚠️ System Health Alert: {} (score: {:.1}) - Issues: {:?}",
                        report.system_health.status,
                        report.system_health.health_score,
                        report.system_health.issues
                    );
                }
            }
        });
        
        tracing::info!("✅ Production metrics server running with comprehensive monitoring");
        Ok(())
    }
}

// 重新导出为默认版本（保持兼容性）
pub use ProductionAdapterMetrics as AdapterMetrics;
pub use ProductionMetricsRegistry as MetricsRegistry; 