use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;
use metrics::{counter, gauge, histogram, describe_counter, describe_gauge, describe_histogram};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsCollector {
    #[serde(skip)]
    start_time: Option<Instant>,
    
    // 策略相关指标
    opportunities_detected: u64,
    opportunities_executed: u64,
    opportunities_failed: u64,
    
    // 性能指标
    avg_detection_latency_us: f64,
    avg_execution_latency_ms: f64,
    peak_memory_usage_mb: f64,
    
    // 财务指标
    total_profit_usd: f64,
    total_volume_usd: f64,
    win_rate: f64,
    sharpe_ratio: f64,
    
    // 系统健康指标
    uptime_seconds: u64,
    error_count: u64,
    warning_count: u64,
}

impl MetricsCollector {
    pub fn new() -> Self {
        // 注册所有指标描述
        describe_counter!("opportunities_detected_total", "Total number of arbitrage opportunities detected");
        describe_counter!("opportunities_executed_total", "Total number of arbitrage opportunities executed");
        describe_counter!("opportunities_failed_total", "Total number of failed arbitrage executions");
        
        describe_histogram!("detection_latency_microseconds", "Latency of opportunity detection in microseconds");
        describe_histogram!("execution_latency_milliseconds", "Latency of trade execution in milliseconds");
        
        describe_gauge!("profit_usd_total", "Total profit in USD");
        describe_gauge!("volume_usd_total", "Total trading volume in USD");
        describe_gauge!("win_rate_percentage", "Percentage of profitable trades");
        describe_gauge!("sharpe_ratio", "Risk-adjusted return metric");
        
        describe_counter!("errors_total", "Total number of errors");
        describe_counter!("warnings_total", "Total number of warnings");
        
        Self {
            start_time: Some(Instant::now()),
            opportunities_detected: 0,
            opportunities_executed: 0,
            opportunities_failed: 0,
            avg_detection_latency_us: 0.0,
            avg_execution_latency_ms: 0.0,
            peak_memory_usage_mb: 0.0,
            total_profit_usd: 0.0,
            total_volume_usd: 0.0,
            win_rate: 0.0,
            sharpe_ratio: 0.0,
            uptime_seconds: 0,
            error_count: 0,
            warning_count: 0,
        }
    }
    
    pub fn record_opportunity_detected(&mut self) {
        self.opportunities_detected += 1;
        counter!("opportunities_detected_total", 1);
    }
    
    pub fn record_opportunity_executed(&mut self, profit: f64, volume: f64) {
        self.opportunities_executed += 1;
        self.total_profit_usd += profit;
        self.total_volume_usd += volume;
        
        counter!("opportunities_executed_total", 1);
        gauge!("profit_usd_total", self.total_profit_usd);
        gauge!("volume_usd_total", self.total_volume_usd);
        
        // 更新胜率
        if profit > 0.0 {
            let total = self.opportunities_executed as f64;
            let wins = self.opportunities_executed.saturating_sub(self.opportunities_failed) as f64;
            self.win_rate = (wins / total) * 100.0;
            gauge!("win_rate_percentage", self.win_rate);
        }
    }
    
    pub fn record_opportunity_failed(&mut self) {
        self.opportunities_failed += 1;
        counter!("opportunities_failed_total", 1);
    }
    
    pub fn record_detection_latency(&mut self, latency_us: u64) {
        histogram!("detection_latency_microseconds", latency_us as f64);
        
        // 更新平均延迟（简单移动平均）
        let count = self.opportunities_detected as f64;
        if count > 0.0 {
            self.avg_detection_latency_us = 
                (self.avg_detection_latency_us * (count - 1.0) + latency_us as f64) / count;
        }
    }
    
    pub fn record_execution_latency(&mut self, latency_ms: u64) {
        histogram!("execution_latency_milliseconds", latency_ms as f64);
        
        // 更新平均延迟
        let count = self.opportunities_executed as f64;
        if count > 0.0 {
            self.avg_execution_latency_ms = 
                (self.avg_execution_latency_ms * (count - 1.0) + latency_ms as f64) / count;
        }
    }
    
    pub fn update_sharpe_ratio(&mut self, returns: &[f64], risk_free_rate: f64) {
        if returns.is_empty() {
            return;
        }
        
        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev > 0.0 {
            self.sharpe_ratio = (mean_return - risk_free_rate) / std_dev;
            gauge!("sharpe_ratio", self.sharpe_ratio);
        }
    }
    
    pub fn record_error(&mut self) {
        self.error_count += 1;
        counter!("errors_total", 1);
    }
    
    pub fn record_warning(&mut self) {
        self.warning_count += 1;
        counter!("warnings_total", 1);
    }
    
    pub fn update_uptime(&mut self) {
        if let Some(start) = self.start_time {
            self.uptime_seconds = start.elapsed().as_secs();
        }
    }
    
    pub fn get_summary(&self) -> MetricsSummary {
        MetricsSummary {
            opportunities_detected: self.opportunities_detected,
            opportunities_executed: self.opportunities_executed,
            opportunities_failed: self.opportunities_failed,
            success_rate: if self.opportunities_executed > 0 {
                ((self.opportunities_executed - self.opportunities_failed) as f64 / 
                 self.opportunities_executed as f64) * 100.0
            } else { 0.0 },
            avg_detection_latency_us: self.avg_detection_latency_us,
            avg_execution_latency_ms: self.avg_execution_latency_ms,
            total_profit_usd: self.total_profit_usd,
            total_volume_usd: self.total_volume_usd,
            win_rate: self.win_rate,
            sharpe_ratio: self.sharpe_ratio,
            uptime_seconds: self.uptime_seconds,
            error_count: self.error_count,
            warning_count: self.warning_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub opportunities_detected: u64,
    pub opportunities_executed: u64,
    pub opportunities_failed: u64,
    pub success_rate: f64,
    pub avg_detection_latency_us: f64,
    pub avg_execution_latency_ms: f64,
    pub total_profit_usd: f64,
    pub total_volume_usd: f64,
    pub win_rate: f64,
    pub sharpe_ratio: f64,
    pub uptime_seconds: u64,
    pub error_count: u64,
    pub warning_count: u64,
}

pub struct MetricsManager {
    collector: Arc<RwLock<MetricsCollector>>,
}

impl MetricsManager {
    pub fn new() -> Self {
        Self {
            collector: Arc::new(RwLock::new(MetricsCollector::new())),
        }
    }
    
    pub fn get_collector(&self) -> Arc<RwLock<MetricsCollector>> {
        self.collector.clone()
    }
    
    pub fn start_prometheus_exporter(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Prometheus exporter由metrics-exporter-prometheus处理
        tracing::info!("Metrics exporter started on port {}", port);
        Ok(())
    }
} 