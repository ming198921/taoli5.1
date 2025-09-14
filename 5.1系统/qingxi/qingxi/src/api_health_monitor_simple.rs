//! # API健康监控器 - 极轻量级版本
//! 
//! 修复了AtomicF64问题的简化实现

use crate::types::*;
use crate::errors::*;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn, instrument};
use std::collections::HashMap;

/// 健康监控器 - 简化版本
pub struct ApiHealthMonitorEnhanced {
    total_messages: AtomicU64,
    total_errors: AtomicU64,
    total_latency_ns: AtomicU64,
    
    // 使用AtomicU64存储健康度评分的整数表示
    cached_health_score_x100: AtomicU64, // 乘以100存储，避免浮点数
    last_health_update_ms: AtomicU64,
    
    // 交易所特定指标
    exchange_metrics: Arc<RwLock<HashMap<String, ExchangeHealthMetrics>>>,
    
    // 控制状态
    monitoring_enabled: AtomicBool,
    start_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeHealthMetrics {
    pub exchange_name: String,
    pub health_score: f64,
    pub avg_latency_ms: f64,
    pub error_rate: f64,
    pub message_count: u64,
    pub last_update_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall_health_score: f64,
    pub total_messages: u64,
    pub error_count: u64,
    pub average_latency_ms: f64,
    pub uptime_seconds: u64,
    pub exchange_metrics: HashMap<String, ExchangeHealthMetrics>,
    pub active_alerts: Vec<HealthAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub alert_type: String,
    pub message: String,
    pub severity: String,
    pub timestamp: u64,
    pub exchange: String,
}

impl ApiHealthMonitorEnhanced {
    pub async fn new() -> Result<Self, MarketDataError> {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(Self {
            total_messages: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
            total_latency_ns: AtomicU64::new(0),
            cached_health_score_x100: AtomicU64::new(10000), // 初始100.0分
            last_health_update_ms: AtomicU64::new(start_time),
            exchange_metrics: Arc::new(RwLock::new(HashMap::new())),
            monitoring_enabled: AtomicBool::new(true),
            start_time_ms: start_time,
        })
    }

    /// 记录消息处理指标（极轻量级，<0.0005ms）
    pub fn record_message_processed(&self, exchange: &str, latency: Duration) {
        if !self.monitoring_enabled.load(Ordering::Relaxed) {
            return;
        }

        // 原子操作，无锁更新
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(latency.as_nanos() as u64, Ordering::Relaxed);
        
        // 异步更新交易所指标（不阻塞主线程）
        let exchange_name = exchange.to_string();
        let metrics_clone = self.exchange_metrics.clone();
        tokio::spawn(async move {
            Self::update_exchange_metrics(metrics_clone, exchange_name, latency, false).await;
        });
    }

    /// 记录错误
    pub fn record_error(&self, exchange: &str, _error_message: String) {
        if !self.monitoring_enabled.load(Ordering::Relaxed) {
            return;
        }

        self.total_errors.fetch_add(1, Ordering::Relaxed);
        
        let exchange_name = exchange.to_string();
        let metrics_clone = self.exchange_metrics.clone();
        tokio::spawn(async move {
            Self::update_exchange_metrics(metrics_clone, exchange_name, Duration::from_secs(0), true).await;
        });
    }

    /// 获取健康度评分
    pub fn get_health_score(&self) -> f64 {
        // 返回缓存的评分（转换回浮点数）
        let score_x100 = self.cached_health_score_x100.load(Ordering::Relaxed);
        score_x100 as f64 / 100.0
    }

    /// 生成健康监控报告
    pub async fn generate_health_report(&self) -> HealthReport {
        let total_messages = self.total_messages.load(Ordering::Relaxed);
        let error_count = self.total_errors.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ns.load(Ordering::Relaxed);
        
        let average_latency_ms = if total_messages > 0 {
            (total_latency as f64 / total_messages as f64) / 1_000_000.0
        } else {
            0.0
        };

        let uptime_seconds = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64 - self.start_time_ms) / 1000;

        // 计算健康度评分
        let error_rate = if total_messages > 0 {
            error_count as f64 / total_messages as f64
        } else {
            0.0
        };

        let latency_score = (1.0 - (average_latency_ms / 10.0).min(1.0)).max(0.0);
        let reliability_score = (1.0 - error_rate * 10.0).max(0.0);
        let overall_health = (latency_score * 0.6 + reliability_score * 0.4) * 100.0;

        // 更新缓存
        self.cached_health_score_x100.store((overall_health * 100.0) as u64, Ordering::Relaxed);

        let exchange_metrics = self.exchange_metrics.read().await.clone();
        let active_alerts = self.generate_alerts(&exchange_metrics, overall_health).await;

        HealthReport {
            overall_health_score: overall_health,
            total_messages,
            error_count,
            average_latency_ms,
            uptime_seconds,
            exchange_metrics,
            active_alerts,
        }
    }

    /// 获取活跃告警
    pub async fn get_active_alerts(&self) -> Vec<HealthAlert> {
        let exchange_metrics = self.exchange_metrics.read().await;
        let overall_health = self.get_health_score();
        self.generate_alerts(&exchange_metrics, overall_health).await
    }

    /// 生成告警
    async fn generate_alerts(&self, exchange_metrics: &HashMap<String, ExchangeHealthMetrics>, overall_health: f64) -> Vec<HealthAlert> {
        let mut alerts = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // 整体健康度告警
        if overall_health < 50.0 {
            alerts.push(HealthAlert {
                alert_type: "SystemHealth".to_string(),
                message: format!("Overall system health is low: {:.1}%", overall_health),
                severity: "Critical".to_string(),
                timestamp: now,
                exchange: "system".to_string(),
            });
        } else if overall_health < 70.0 {
            alerts.push(HealthAlert {
                alert_type: "SystemHealth".to_string(),
                message: format!("Overall system health is degraded: {:.1}%", overall_health),
                severity: "Warning".to_string(),
                timestamp: now,
                exchange: "system".to_string(),
            });
        }

        // 交易所特定告警
        for (exchange, metrics) in exchange_metrics {
            if metrics.avg_latency_ms > 100.0 {
                alerts.push(HealthAlert {
                    alert_type: "HighLatency".to_string(),
                    message: format!("High latency detected: {:.2}ms", metrics.avg_latency_ms),
                    severity: if metrics.avg_latency_ms > 500.0 { "Critical" } else { "Warning" }.to_string(),
                    timestamp: now,
                    exchange: exchange.clone(),
                });
            }

            if metrics.error_rate > 0.05 {
                alerts.push(HealthAlert {
                    alert_type: "HighErrorRate".to_string(),
                    message: format!("High error rate: {:.2}%", metrics.error_rate * 100.0),
                    severity: if metrics.error_rate > 0.1 { "Critical" } else { "Warning" }.to_string(),
                    timestamp: now,
                    exchange: exchange.clone(),
                });
            }
        }

        alerts
    }

    /// 更新交易所指标
    async fn update_exchange_metrics(
        metrics: Arc<RwLock<HashMap<String, ExchangeHealthMetrics>>>,
        exchange: String,
        latency: Duration,
        is_error: bool,
    ) {
        let mut metrics_map = metrics.write().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let entry = metrics_map.entry(exchange.clone()).or_insert_with(|| {
            ExchangeHealthMetrics {
                exchange_name: exchange.clone(),
                health_score: 100.0,
                avg_latency_ms: 0.0,
                error_rate: 0.0,
                message_count: 0,
                last_update_ms: now,
            }
        });

        entry.message_count += 1;
        entry.last_update_ms = now;

        // 更新平均延迟（简单移动平均）
        if !is_error {
            let latency_ms = latency.as_secs_f64() * 1000.0;
            entry.avg_latency_ms = (entry.avg_latency_ms * 0.9) + (latency_ms * 0.1);
        }

        // 更新错误率
        if is_error {
            entry.error_rate = (entry.error_rate * 0.95) + 0.05;
        } else {
            entry.error_rate *= 0.99;
        }

        // 计算健康度评分
        let latency_score = (1.0 - (entry.avg_latency_ms / 100.0).min(1.0)).max(0.0);
        let reliability_score = (1.0 - entry.error_rate * 10.0).max(0.0);
        entry.health_score = (latency_score * 0.6 + reliability_score * 0.4) * 100.0;
    }

    /// 启用/禁用监控
    pub fn set_monitoring_enabled(&self, enabled: bool) {
        self.monitoring_enabled.store(enabled, Ordering::Relaxed);
    }

    /// 检查监控是否启用
    pub fn is_monitoring_enabled(&self) -> bool {
        self.monitoring_enabled.load(Ordering::Relaxed)
    }

    /// 重置所有指标
    pub async fn reset_metrics(&self) {
        self.total_messages.store(0, Ordering::Relaxed);
        self.total_errors.store(0, Ordering::Relaxed);
        self.total_latency_ns.store(0, Ordering::Relaxed);
        self.cached_health_score_x100.store(10000, Ordering::Relaxed);
        
        let mut metrics = self.exchange_metrics.write().await;
        metrics.clear();
    }

    /// 获取基本统计信息
    pub fn get_basic_stats(&self) -> (u64, u64, f64) {
        let total_messages = self.total_messages.load(Ordering::Relaxed);
        let total_errors = self.total_errors.load(Ordering::Relaxed);
        let total_latency = self.total_latency_ns.load(Ordering::Relaxed);
        
        let avg_latency_ms = if total_messages > 0 {
            (total_latency as f64 / total_messages as f64) / 1_000_000.0
        } else {
            0.0
        };

        (total_messages, total_errors, avg_latency_ms)
    }
}

