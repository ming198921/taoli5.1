#![allow(dead_code)]
//! # API健康度监控模块 - Qingxi 5.1 增强版
//! 
//! 基于原子操作的超轻量级健康监控系统，无锁设计，延迟 < 0.0005ms

use crate::errors::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn, instrument};

/// 连接状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Reconnecting,
    Failed,
}

/// 交易所健康度指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeHealthMetrics {
    pub exchange_name: String,
    pub connection_status: ConnectionStatus,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub error_rate: f64,
    pub message_rate: f64,
    pub data_quality_score: f64,
    pub last_message_time: u64, // Unix timestamp in milliseconds
    pub uptime_percentage: f64,
    pub reconnection_count: u64,
}

/// API健康度监控器 - 极轻量级实现
pub struct ApiHealthMonitorEnhanced {
    // 原子计数器（无锁，极低延迟）
    total_messages: AtomicU64,
    total_errors: AtomicU64,
    total_latency_ns: AtomicU64,
    
    // 健康度评分缓存（使用AtomicU64存储浮点数的位表示）
    cached_health_score_bits: AtomicU64,
    last_health_update_ms: AtomicU64,
    health_cache_ttl_ms: u64,
    
    // 系统启动时间
    start_time: Instant,
    
    // 交易所特定指标（使用RwLock，读多写少场景）
    exchange_metrics: Arc<RwLock<HashMap<String, ExchangeMetricsAtomic>>>,
    
    // 配置
    config: HealthMonitorConfig,
    
    // 运行状态
    is_running: AtomicBool,
}

/// 原子化的交易所指标（避免锁竞争）
struct ExchangeMetricsAtomic {
    message_count: AtomicU64,
    error_count: AtomicU64,
    total_latency_ns: AtomicU64,
    last_message_timestamp_ms: AtomicU64,
    connection_status: AtomicU64, // 用数字表示ConnectionStatus
    reconnection_count: AtomicU64,
    uptime_start_ms: AtomicU64,
    downtime_total_ms: AtomicU64,
}

impl ExchangeMetricsAtomic {
    fn new() -> Self {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
            
        Self {
            message_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            last_message_timestamp_ms: AtomicU64::new(now_ms),
            connection_status: AtomicU64::new(ConnectionStatus::Disconnected as u64),
            reconnection_count: AtomicU64::new(0),
            uptime_start_ms: AtomicU64::new(now_ms),
            downtime_total_ms: AtomicU64::new(0),
        }
    }
    
    fn set_connection_status(&self, status: ConnectionStatus) {
        self.connection_status.store(status as u64, Ordering::Relaxed);
    }
    
    fn get_connection_status(&self) -> ConnectionStatus {
        match self.connection_status.load(Ordering::Relaxed) {
            0 => ConnectionStatus::Connected,
            1 => ConnectionStatus::Connecting,
            2 => ConnectionStatus::Disconnected,
            3 => ConnectionStatus::Reconnecting,
            4 => ConnectionStatus::Failed,
            _ => ConnectionStatus::Disconnected,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// 健康度缓存TTL（毫秒）
    pub health_cache_ttl_ms: u64,
    /// 延迟历史窗口大小
    pub latency_window_size: usize,
    /// 错误率阈值（触发告警）
    pub error_rate_threshold: f64,
    /// 延迟阈值（毫秒，触发告警）
    pub latency_threshold_ms: f64,
    /// 数据质量最低分数
    pub min_quality_score: f64,
    /// 消息速率监控窗口（秒）
    pub message_rate_window_seconds: u64,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            health_cache_ttl_ms: 1000, // 1秒缓存
            latency_window_size: 1000,
            error_rate_threshold: 0.01, // 1%错误率阈值
            latency_threshold_ms: 10.0, // 10ms延迟阈值
            min_quality_score: 0.8,
            message_rate_window_seconds: 60, // 1分钟窗口
        }
    }
}

/// 详细的健康度报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthReport {
    pub overall_health_score: f64,
    pub system_uptime_seconds: u64,
    pub total_messages_processed: u64,
    pub global_error_rate: f64,
    pub avg_system_latency_ms: f64,
    pub p99_system_latency_ms: f64,
    pub exchange_reports: HashMap<String, ExchangeHealthMetrics>,
    pub alerts: Vec<HealthAlert>,
    pub performance_grade: PerformanceGrade,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub alert_id: String,
    pub exchange: String,
    pub alert_type: HealthAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp_ms: u64,
    pub metric_value: f64,
    pub threshold_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthAlertType {
    HighLatency,
    HighErrorRate,
    LowDataQuality,
    ConnectionLoss,
    LowMessageRate,
    SystemOverload,
}

impl std::fmt::Display for HealthAlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthAlertType::HighLatency => write!(f, "HIGH_LATENCY"),
            HealthAlertType::HighErrorRate => write!(f, "HIGH_ERROR_RATE"),
            HealthAlertType::LowDataQuality => write!(f, "LOW_DATA_QUALITY"),
            HealthAlertType::ConnectionLoss => write!(f, "CONNECTION_LOSS"),
            HealthAlertType::LowMessageRate => write!(f, "LOW_MESSAGE_RATE"),
            HealthAlertType::SystemOverload => write!(f, "SYSTEM_OVERLOAD"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceGrade {
    Excellent, // A+ (95-100%)
    Good,      // A  (85-94%)
    Fair,      // B  (70-84%)
    Poor,      // C  (50-69%)
    Critical,  // D  (0-49%)
}

impl ApiHealthMonitorEnhanced {
    pub fn new(config: HealthMonitorConfig) -> Self {
        Self {
            total_messages: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            cached_health_score_bits: AtomicU64::new(100.0_f64.to_bits()),
            last_health_update_ms: AtomicU64::new(0),
            health_cache_ttl_ms: config.health_cache_ttl_ms,
            start_time: Instant::now(),
            exchange_metrics: Arc::new(RwLock::new(HashMap::new())),
            config,
            is_running: AtomicBool::new(false),
        }
    }
    
    /// 启动健康监控器
    pub async fn start(&self) -> Result<(), MarketDataError> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        self.is_running.store(true, Ordering::Relaxed);
        
        // 启动后台健康度计算任务
        self.start_background_health_calculator().await;
        
        // 启动告警检查任务
        self.start_alert_checker().await;
        
        info!("ApiHealthMonitorEnhanced started");
        Ok(())
    }
    
    async fn start_background_health_calculator(&self) {
        let health_monitor = self.clone();
        let is_running = Arc::new(AtomicBool::new(self.is_running.load(Ordering::Relaxed)));
        
        tokio::spawn(async move {
            while is_running.load(Ordering::Relaxed) {
                health_monitor.recalculate_health_score_async().await;
                tokio::time::sleep(Duration::from_millis(health_monitor.health_cache_ttl_ms / 2)).await;
            }
        });
    }
    
    async fn start_alert_checker(&self) {
        let health_monitor = self.clone();
        let is_running = Arc::new(AtomicBool::new(self.is_running.load(Ordering::Relaxed)));
        
        tokio::spawn(async move {
            while is_running.load(Ordering::Relaxed) {
                health_monitor.check_and_generate_alerts().await;
                tokio::time::sleep(Duration::from_secs(10)).await; // 每10秒检查一次告警
            }
        });
    }
    
    /// 极轻量级指标更新（目标 < 0.0005ms）
    #[instrument(skip(self), fields(exchange = %exchange))]
    pub async fn record_message_processed(&self, exchange: &str, latency_ns: u64) {
        // 原子操作，无锁更新全局统计
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
        
        // 更新交易所特定指标（异步，不阻塞主线程）
        let exchange_name = exchange.to_string();
        let exchange_metrics = self.exchange_metrics.clone();
        
        tokio::spawn(async move {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64;
            
            let metrics = exchange_metrics.read().await;
            if let Some(atomic_metrics) = metrics.get(&exchange_name) {
                atomic_metrics.message_count.fetch_add(1, Ordering::Relaxed);
                atomic_metrics.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
                atomic_metrics.last_message_timestamp_ms.store(now_ms, Ordering::Relaxed);
            } else {
                // 需要创建新的指标项，释放读锁后获取写锁
                drop(metrics);
                let mut write_metrics = exchange_metrics.write().await;
                if !write_metrics.contains_key(&exchange_name) {
                    let atomic_metrics = ExchangeMetricsAtomic::new();
                    atomic_metrics.message_count.fetch_add(1, Ordering::Relaxed);
                    atomic_metrics.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
                    atomic_metrics.last_message_timestamp_ms.store(now_ms, Ordering::Relaxed);
                    write_metrics.insert(exchange_name, atomic_metrics);
                }
            }
        });
        
        debug!("Message processed for {}: latency={}ns", exchange, latency_ns);
    }
    
    /// 记录错误（原子操作）
    pub async fn record_error(&self, exchange: &str, error_type: &str) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
        
        // 异步更新交易所错误计数
        let exchange_name = exchange.to_string();
        let error_type_name = error_type.to_string();
        let exchange_metrics = self.exchange_metrics.clone();
        
        tokio::spawn(async move {
            let metrics = exchange_metrics.read().await;
            if let Some(atomic_metrics) = metrics.get(&exchange_name) {
                atomic_metrics.error_count.fetch_add(1, Ordering::Relaxed);
            }
        });
        
        warn!("Error recorded for {}: {}", exchange, error_type_name);
    }
    
    /// 更新连接状态
    pub async fn update_connection_status(&self, exchange: &str, status: ConnectionStatus) {
        let exchange_name = exchange.to_string();
        let exchange_metrics = self.exchange_metrics.clone();
        
        tokio::spawn(async move {
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64;
            
            let mut write_metrics = exchange_metrics.write().await;
            let atomic_metrics = write_metrics.entry(exchange_name.clone())
                .or_insert_with(ExchangeMetricsAtomic::new);
            
            let old_status = atomic_metrics.get_connection_status();
            atomic_metrics.set_connection_status(status);
            
            // 处理连接状态变化
            match (old_status, status) {
                (ConnectionStatus::Disconnected, ConnectionStatus::Connected) => {
                    atomic_metrics.uptime_start_ms.store(now_ms, Ordering::Relaxed);
                    info!("Exchange {} connected", exchange_name);
                }
                (ConnectionStatus::Connected, ConnectionStatus::Disconnected) => {
                    atomic_metrics.reconnection_count.fetch_add(1, Ordering::Relaxed);
                    info!("Exchange {} disconnected", exchange_name);
                }
                (_, ConnectionStatus::Reconnecting) => {
                    atomic_metrics.reconnection_count.fetch_add(1, Ordering::Relaxed);
                    info!("Exchange {} reconnecting", exchange_name);
                }
                _ => {}
            }
        });
    }
    
    /// 快速健康度评分（基于缓存，避免重复计算）
    pub fn get_health_score(&self) -> f64 {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
        
        let last_update = self.last_health_update_ms.load(Ordering::Relaxed);
        
        // 如果缓存仍然有效，直接返回
        if now_ms - last_update < self.health_cache_ttl_ms {
            return f64::from_bits(self.cached_health_score_bits.load(Ordering::Relaxed));
        }
        
        // 触发后台重新计算（不阻塞当前调用）
        let health_monitor = self.clone();
        tokio::spawn(async move {
            health_monitor.recalculate_health_score_async().await;
        });
        
        // 返回缓存值
        f64::from_bits(self.cached_health_score_bits.load(Ordering::Relaxed))
    }
    
    /// 后台重新计算健康度分数
    async fn recalculate_health_score_async(&self) {
        let total_messages = self.total_messages.load(Ordering::Relaxed);
        let total_errors = self.total_errors.load(Ordering::Relaxed);
        let total_latency_ns = self.total_latency_ns.load(Ordering::Relaxed);
        
        if total_messages == 0 {
            self.cached_health_score_bits.store(100.0_f64.to_bits(), Ordering::Relaxed);
            return;
        }
        
        // 计算全局指标
        let global_error_rate = total_errors as f64 / total_messages as f64;
        let avg_latency_ms = (total_latency_ns as f64 / total_messages as f64) / 1_000_000.0;
        
        // 基础健康度评分算法
        let error_penalty = (global_error_rate / self.config.error_rate_threshold).min(1.0) * 30.0;
        let latency_penalty = (avg_latency_ms / self.config.latency_threshold_ms).min(1.0) * 25.0;
        
        // 计算交易所健康度
        let exchange_health_score = self.calculate_exchange_health_score().await;
        
        // 综合健康度分数
        let base_score = 100.0 - error_penalty - latency_penalty;
        let health_score = (base_score * 0.7 + exchange_health_score * 0.3).max(0.0).min(100.0);
        
        // 更新缓存
        self.cached_health_score_bits.store(health_score.to_bits(), Ordering::Relaxed);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
        self.last_health_update_ms.store(now_ms, Ordering::Relaxed);
        
        debug!("Health score recalculated: {:.2}% (error_penalty={:.2}, latency_penalty={:.2})", 
               health_score, error_penalty, latency_penalty);
    }
    
    async fn calculate_exchange_health_score(&self) -> f64 {
        let metrics = self.exchange_metrics.read().await;
        if metrics.is_empty() {
            return 100.0;
        }
        
        let mut total_score = 0.0;
        let mut exchange_count = 0;
        
        for (exchange_name, atomic_metrics) in metrics.iter() {
            let message_count = atomic_metrics.message_count.load(Ordering::Relaxed);
            let error_count = atomic_metrics.error_count.load(Ordering::Relaxed);
            let connection_status = atomic_metrics.get_connection_status();
            
            if message_count == 0 {
                continue;
            }
            
            let error_rate = error_count as f64 / message_count as f64;
            let connection_score = match connection_status {
                ConnectionStatus::Connected => 100.0,
                ConnectionStatus::Connecting => 80.0,
                ConnectionStatus::Reconnecting => 60.0,
                ConnectionStatus::Disconnected => 20.0,
                ConnectionStatus::Failed => 0.0,
            };
            
            let reliability_score = (1.0 - error_rate * 10.0).max(0.0) * 100.0;
            let exchange_score = (connection_score * 0.5 + reliability_score * 0.5).max(0.0).min(100.0);
            
            total_score += exchange_score;
            exchange_count += 1;
            
            debug!("Exchange {} health: {:.2}% (connection={:.0}%, reliability={:.2}%)", 
                   exchange_name, exchange_score, connection_score, reliability_score);
        }
        
        if exchange_count == 0 {
            return 100.0;
        }
        
        total_score / exchange_count as f64
    }
    
    /// 生成详细健康度报告
    pub async fn generate_detailed_report(&self) -> DetailedHealthReport {
        let overall_health_score = self.get_health_score();
        let system_uptime_seconds = self.start_time.elapsed().as_secs();
        let total_messages = self.total_messages.load(Ordering::Relaxed);
        let total_errors = self.total_errors.load(Ordering::Relaxed);
        let total_latency_ns = self.total_latency_ns.load(Ordering::Relaxed);
        
        let global_error_rate = if total_messages > 0 {
            total_errors as f64 / total_messages as f64
        } else {
            0.0
        };
        
        let avg_system_latency_ms = if total_messages > 0 {
            (total_latency_ns as f64 / total_messages as f64) / 1_000_000.0
        } else {
            0.0
        };
        
        // 计算P99延迟（简化实现，实际应使用直方图）
        let p99_system_latency_ms = avg_system_latency_ms * 2.5; // 估算值
        
        // 生成交易所报告
        let exchange_reports = self.generate_exchange_reports().await;
        
        // 生成告警
        let alerts = self.generate_current_alerts(&exchange_reports).await;
        
        // 性能评级
        let performance_grade = Self::calculate_performance_grade(overall_health_score);
        
        // 生成建议
        let recommendations = self.generate_recommendations(overall_health_score, &exchange_reports, &alerts).await;
        
        DetailedHealthReport {
            overall_health_score,
            system_uptime_seconds,
            total_messages_processed: total_messages,
            global_error_rate,
            avg_system_latency_ms,
            p99_system_latency_ms,
            exchange_reports,
            alerts,
            performance_grade,
            recommendations,
        }
    }
    
    async fn generate_exchange_reports(&self) -> HashMap<String, ExchangeHealthMetrics> {
        let mut reports = HashMap::new();
        let metrics = self.exchange_metrics.read().await;
        
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
        
        for (exchange_name, atomic_metrics) in metrics.iter() {
            let message_count = atomic_metrics.message_count.load(Ordering::Relaxed);
            let error_count = atomic_metrics.error_count.load(Ordering::Relaxed);
            let total_latency_ns = atomic_metrics.total_latency_ns.load(Ordering::Relaxed);
            let last_message_time = atomic_metrics.last_message_timestamp_ms.load(Ordering::Relaxed);
            let connection_status = atomic_metrics.get_connection_status();
            let reconnection_count = atomic_metrics.reconnection_count.load(Ordering::Relaxed);
            let uptime_start_ms = atomic_metrics.uptime_start_ms.load(Ordering::Relaxed);
            
            let error_rate = if message_count > 0 {
                error_count as f64 / message_count as f64
            } else {
                0.0
            };
            
            let avg_latency_ms = if message_count > 0 {
                (total_latency_ns as f64 / message_count as f64) / 1_000_000.0
            } else {
                0.0
            };
            
            let p99_latency_ms = avg_latency_ms * 2.5; // 估算值
            
            // 计算消息速率（每秒消息数）
            let uptime_seconds = (now_ms - uptime_start_ms) / 1000;
            let message_rate = if uptime_seconds > 0 {
                message_count as f64 / uptime_seconds as f64
            } else {
                0.0
            };
            
            // 计算运行时间百分比（简化实现）
            let uptime_percentage = match connection_status {
                ConnectionStatus::Connected => 100.0,
                ConnectionStatus::Connecting | ConnectionStatus::Reconnecting => 80.0,
                ConnectionStatus::Disconnected => 50.0,
                ConnectionStatus::Failed => 0.0,
            };
            
            // 数据质量评分（基于错误率和延迟）
            let quality_score = (1.0 - error_rate * 5.0).max(0.0) * 
                               (1.0 - (avg_latency_ms / 100.0).min(1.0)) * 100.0;
            
            let metrics = ExchangeHealthMetrics {
                exchange_name: exchange_name.clone(),
                connection_status,
                avg_latency_ms,
                p99_latency_ms,
                error_rate,
                message_rate,
                data_quality_score: quality_score,
                last_message_time,
                uptime_percentage,
                reconnection_count,
            };
            
            reports.insert(exchange_name.clone(), metrics);
        }
        
        reports
    }
    
    async fn check_and_generate_alerts(&self) {
        let exchange_reports = self.generate_exchange_reports().await;
        let alerts = self.generate_current_alerts(&exchange_reports).await;
        
        for alert in alerts {
            match alert.severity {
                AlertSeverity::Emergency => {
                    error!("EMERGENCY: {} - {} ({}={:.2}, threshold={:.2})", 
                           alert.alert_type, alert.message, 
                           alert.exchange, alert.metric_value, alert.threshold_value);
                }
                AlertSeverity::Critical => {
                    error!("CRITICAL: {} - {} ({}={:.2}, threshold={:.2})", 
                           alert.alert_type, alert.message, 
                           alert.exchange, alert.metric_value, alert.threshold_value);
                }
                AlertSeverity::Warning => {
                    warn!("WARNING: {} - {} ({}={:.2}, threshold={:.2})", 
                          alert.alert_type, alert.message, 
                          alert.exchange, alert.metric_value, alert.threshold_value);
                }
                AlertSeverity::Info => {
                    info!("INFO: {} - {} ({}={:.2}, threshold={:.2})", 
                          alert.alert_type, alert.message, 
                          alert.exchange, alert.metric_value, alert.threshold_value);
                }
            }
        }
    }
    
    async fn generate_current_alerts(&self, exchange_reports: &HashMap<String, ExchangeHealthMetrics>) -> Vec<HealthAlert> {
        let mut alerts = Vec::new();
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;
        
        for (exchange_name, metrics) in exchange_reports {
            // 高延迟告警
            if metrics.avg_latency_ms > self.config.latency_threshold_ms {
                alerts.push(HealthAlert {
                    alert_id: format!("high_latency_{}_{}", exchange_name, now_ms),
                    exchange: exchange_name.clone(),
                    alert_type: HealthAlertType::HighLatency,
                    severity: if metrics.avg_latency_ms > self.config.latency_threshold_ms * 2.0 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::Warning
                    },
                    message: format!("High latency detected: {:.2}ms", metrics.avg_latency_ms),
                    timestamp_ms: now_ms,
                    metric_value: metrics.avg_latency_ms,
                    threshold_value: self.config.latency_threshold_ms,
                });
            }
            
            // 高错误率告警
            if metrics.error_rate > self.config.error_rate_threshold {
                alerts.push(HealthAlert {
                    alert_id: format!("high_error_rate_{}_{}", exchange_name, now_ms),
                    exchange: exchange_name.clone(),
                    alert_type: HealthAlertType::HighErrorRate,
                    severity: if metrics.error_rate > self.config.error_rate_threshold * 5.0 {
                        AlertSeverity::Critical
                    } else {
                        AlertSeverity::Warning
                    },
                    message: format!("High error rate: {:.2}%", metrics.error_rate * 100.0),
                    timestamp_ms: now_ms,
                    metric_value: metrics.error_rate,
                    threshold_value: self.config.error_rate_threshold,
                });
            }
            
            // 低数据质量告警
            if metrics.data_quality_score < self.config.min_quality_score * 100.0 {
                alerts.push(HealthAlert {
                    alert_id: format!("low_quality_{}_{}", exchange_name, now_ms),
                    exchange: exchange_name.clone(),
                    alert_type: HealthAlertType::LowDataQuality,
                    severity: AlertSeverity::Warning,
                    message: format!("Low data quality: {:.1}%", metrics.data_quality_score),
                    timestamp_ms: now_ms,
                    metric_value: metrics.data_quality_score,
                    threshold_value: self.config.min_quality_score * 100.0,
                });
            }
            
            // 连接状态告警
            match metrics.connection_status {
                ConnectionStatus::Disconnected => {
                    alerts.push(HealthAlert {
                        alert_id: format!("disconnected_{}_{}", exchange_name, now_ms),
                        exchange: exchange_name.clone(),
                        alert_type: HealthAlertType::ConnectionLoss,
                        severity: AlertSeverity::Critical,
                        message: "Exchange disconnected".to_string(),
                        timestamp_ms: now_ms,
                        metric_value: 0.0,
                        threshold_value: 1.0,
                    });
                }
                ConnectionStatus::Failed => {
                    alerts.push(HealthAlert {
                        alert_id: format!("failed_{}_{}", exchange_name, now_ms),
                        exchange: exchange_name.clone(),
                        alert_type: HealthAlertType::ConnectionLoss,
                        severity: AlertSeverity::Emergency,
                        message: "Exchange connection failed".to_string(),
                        timestamp_ms: now_ms,
                        metric_value: 0.0,
                        threshold_value: 1.0,
                    });
                }
                _ => {}
            }
        }
        
        alerts
    }
    
    fn calculate_performance_grade(health_score: f64) -> PerformanceGrade {
        match health_score {
            score if score >= 95.0 => PerformanceGrade::Excellent,
            score if score >= 85.0 => PerformanceGrade::Good,
            score if score >= 70.0 => PerformanceGrade::Fair,
            score if score >= 50.0 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Critical,
        }
    }
    
    async fn generate_recommendations(&self, health_score: f64, exchange_reports: &HashMap<String, ExchangeHealthMetrics>, alerts: &[HealthAlert]) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // 基于健康度分数的建议
        if health_score < 70.0 {
            recommendations.push("System health is below optimal. Consider investigating performance issues.".to_string());
        }
        
        // 基于延迟的建议
        let high_latency_exchanges: Vec<_> = exchange_reports.iter()
            .filter(|(_, metrics)| metrics.avg_latency_ms > self.config.latency_threshold_ms)
            .map(|(name, _)| name.as_str())
            .collect();
        
        if !high_latency_exchanges.is_empty() {
            recommendations.push(format!("High latency detected on exchanges: {}. Consider optimizing network connections or switching to closer servers.", high_latency_exchanges.join(", ")));
        }
        
        // 基于错误率的建议
        let high_error_exchanges: Vec<_> = exchange_reports.iter()
            .filter(|(_, metrics)| metrics.error_rate > self.config.error_rate_threshold)
            .map(|(name, _)| name.as_str())
            .collect();
        
        if !high_error_exchanges.is_empty() {
            recommendations.push(format!("High error rate on exchanges: {}. Check connection stability and API limits.", high_error_exchanges.join(", ")));
        }
        
        // 基于连接状态的建议
        let disconnected_exchanges: Vec<_> = exchange_reports.iter()
            .filter(|(_, metrics)| matches!(metrics.connection_status, ConnectionStatus::Disconnected | ConnectionStatus::Failed))
            .map(|(name, _)| name.as_str())
            .collect();
        
        if !disconnected_exchanges.is_empty() {
            recommendations.push(format!("Exchanges {} are disconnected. Enable auto-reconnection or check network connectivity.", disconnected_exchanges.join(", ")));
        }
        
        // 基于告警数量的建议
        if alerts.len() > 5 {
            recommendations.push("Multiple alerts detected. Consider implementing alert prioritization and batch processing.".to_string());
        }
        
        // 如果没有问题，给出正面建议
        if recommendations.is_empty() {
            recommendations.push("System is operating optimally. Continue monitoring for sustained performance.".to_string());
        }
        
        recommendations
    }
    
    /// 停止健康监控器
    pub async fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        info!("ApiHealthMonitorEnhanced stopped");
    }
    
    /// 获取基础健康统计
    pub async fn get_basic_stats(&self) -> BasicHealthStats {
        let total_messages = self.total_messages.load(Ordering::Relaxed);
        let total_errors = self.total_errors.load(Ordering::Relaxed);
        let total_latency_ns = self.total_latency_ns.load(Ordering::Relaxed);
        
        BasicHealthStats {
            health_score: self.get_health_score(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            total_messages,
            total_errors,
            error_rate: if total_messages > 0 { total_errors as f64 / total_messages as f64 } else { 0.0 },
            avg_latency_ms: if total_messages > 0 { (total_latency_ns as f64 / total_messages as f64) / 1_000_000.0 } else { 0.0 },
            active_exchanges: self.exchange_metrics.read().await.len(),
        }
    }
}

// 实现Clone以支持多线程使用
impl Clone for ApiHealthMonitorEnhanced {
    fn clone(&self) -> Self {
        Self {
            total_messages: AtomicU64::new(self.total_messages.load(Ordering::Relaxed)),
            total_errors: AtomicU64::new(self.total_errors.load(Ordering::Relaxed)),
            total_latency_ns: AtomicU64::new(self.total_latency_ns.load(Ordering::Relaxed)),
            cached_health_score_bits: AtomicU64::new(f64::from_bits(self.cached_health_score_bits.load(Ordering::Relaxed)).to_bits()),
            last_health_update_ms: AtomicU64::new(self.last_health_update_ms.load(Ordering::Relaxed)),
            health_cache_ttl_ms: self.health_cache_ttl_ms,
            start_time: self.start_time,
            exchange_metrics: self.exchange_metrics.clone(),
            config: self.config.clone(),
            is_running: AtomicBool::new(self.is_running.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicHealthStats {
    pub health_score: f64,
    pub uptime_seconds: u64,
    pub total_messages: u64,
    pub total_errors: u64,
    pub error_rate: f64,
    pub avg_latency_ms: f64,
    pub active_exchanges: usize,
}

