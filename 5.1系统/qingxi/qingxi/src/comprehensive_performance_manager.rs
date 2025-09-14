#![allow(dead_code)]
//! # 简化的性能管理器
//!
//! 基础性能管理和监控

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use log::info;

use crate::types::*;
use crate::errors::MarketDataError;
use crate::cleaner::{OptimizedDataCleaner, DataCleaner};

/// 基础性能配置
#[derive(Debug, Clone)]
pub struct SystemPerformanceConfig {
    pub monitoring_enabled: bool,
    pub optimization_level: OptimizationLevel,
    pub alert_thresholds: AlertThresholds,
}

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    Conservative,
    Balanced,
    Aggressive,
    Maximum,
}

/// 告警阈值配置
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub latency_warning_ms: f64,
    pub latency_critical_ms: f64,
    pub cpu_warning_percent: f64,
    pub cpu_critical_percent: f64,
}

impl Default for SystemPerformanceConfig {
    fn default() -> Self {
        Self {
            monitoring_enabled: true,
            optimization_level: OptimizationLevel::Balanced,
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            latency_warning_ms: 10.0,
            latency_critical_ms: 50.0,
            cpu_warning_percent: 80.0,
            cpu_critical_percent: 95.0,
        }
    }
}

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Excellent,
    Good,
    Warning,
    Critical,
    Unknown,
}

/// 性能统计信息
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_data_processed: u64,
    pub processing_time_total: Duration,
    pub errors_count: u64,
    pub start_time: Instant,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        Self {
            total_data_processed: 0,
            processing_time_total: Duration::ZERO,
            errors_count: 0,
            start_time: Instant::now(),
        }
    }
}

/// 简化的性能管理器
pub struct ComprehensivePerformanceManager {
    config: Arc<RwLock<SystemPerformanceConfig>>,
    data_cleaner: Arc<OptimizedDataCleaner>,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<PerformanceStats>>,
    start_time: Instant,
}

impl ComprehensivePerformanceManager {
    /// 创建新的性能管理器
    pub fn new(
        config: SystemPerformanceConfig,
        input_rx: flume::Receiver<MarketDataSnapshot>,
        output_tx: flume::Sender<MarketDataSnapshot>,
    ) -> Self {
        let data_cleaner = Arc::new(OptimizedDataCleaner::new(input_rx, output_tx));

        Self {
            config: Arc::new(RwLock::new(config)),
            data_cleaner,
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(PerformanceStats::default())),
            start_time: Instant::now(),
        }
    }

    /// 启动性能管理器
    pub async fn start(&self) -> Result<(), MarketDataError> {
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Ok(());
            }
            *running = true;
        }

        info!("🚀 启动简化性能管理器");

        // 启动数据清洗器
        let mut cleaner: OptimizedDataCleaner = Arc::as_ref(&self.data_cleaner).clone();
        cleaner.start().await?;
        info!("✅ 数据清洗器已启动");

        info!("✅ 简化性能管理器启动完成");
        Ok(())
    }

    /// 停止性能管理器
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;

        info!("📊 生成性能统计报告");
        self.log_performance_stats().await;
        
        info!("🛑 简化性能管理器已停止");
    }

    /// 记录性能统计
    async fn log_performance_stats(&self) {
        let stats = self.stats.read().await;
        let uptime = self.start_time.elapsed();
        
        info!("📊 性能统计报告:");
        info!("  - 运行时间: {:?}", uptime);
        info!("  - 处理数据总量: {}", stats.total_data_processed);
        info!("  - 错误数量: {}", stats.errors_count);
        
        if stats.total_data_processed > 0 {
            let avg_processing_time = stats.processing_time_total.as_millis() as f64 / stats.total_data_processed as f64;
            info!("  - 平均处理时间: {:.2}ms", avg_processing_time);
        }
    }

    /// 获取运行状态
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// 获取健康状态
    pub async fn get_health_status(&self) -> HealthStatus {
        if !self.is_running().await {
            return HealthStatus::Unknown;
        }

        let stats = self.stats.read().await;
        let _config = self.config.read().await;
        
        // 简单的健康状态判断
        if stats.errors_count == 0 {
            HealthStatus::Excellent
        } else if stats.errors_count < 10 {
            HealthStatus::Good
        } else if stats.errors_count < 100 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        }
    }

    /// 更新统计信息
    pub async fn update_stats(&self, processed_count: u64, processing_time: Duration, error_count: u64) {
        let mut stats = self.stats.write().await;
        stats.total_data_processed += processed_count;
        stats.processing_time_total += processing_time;
        stats.errors_count += error_count;
    }
}
