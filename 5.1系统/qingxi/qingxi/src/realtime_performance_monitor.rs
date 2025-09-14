#![allow(dead_code)]
//! # 实时性能监控和自调优模块
//! 
//! 提供微秒级性能监控和基于实时数据的自动参数调优
//! 使用 RDTSC 指令进行高精度计时和性能分析

use std::sync::atomic::{AtomicU64, AtomicUsize, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, VecDeque};
use std::time::{Instant, Duration};
use crate::types::{Symbol, Price, Quantity, Timestamp};

/// 高精度性能计数器，使用 CPU 周期数
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// 总延迟（微秒）
    pub latency_us: f64,
    /// CPU 周期数
    pub cpu_cycles: u64,
    /// 处理的条目数量
    pub entries_processed: usize,
    /// 内存分配次数
    pub allocations: u32,
    /// 缓存未命中次数
    pub cache_misses: u32,
    /// 分支预测失败次数
    pub branch_mispredictions: u32,
    /// 时间戳
    pub timestamp: Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            latency_us: 0.0,
            cpu_cycles: 0,
            entries_processed: 0,
            allocations: 0,
            cache_misses: 0,
            branch_mispredictions: 0,
            timestamp: Instant::now(),
        }
    }
}

/// 循环缓冲区，用于存储性能历史数据
pub struct CircularBuffer<T> {
    buffer: Vec<T>,
    head: AtomicUsize,
    size: usize,
    initialized: AtomicUsize,
}

impl<T: Clone + Default> CircularBuffer<T> {
    /// 创建指定大小的循环缓冲区
    pub fn new(size: usize) -> Self {
        Self {
            buffer: vec![T::default(); size],
            head: AtomicUsize::new(0),
            size,
            initialized: AtomicUsize::new(0),
        }
    }

    /// 添加新的数据点
    pub fn push(&self, item: T) {
        let index = self.head.fetch_add(1, Ordering::Relaxed) % self.size;
        
        // 使用 unsafe 操作提高性能，因为我们确保索引有效
        unsafe {
            let ptr = self.buffer.as_ptr().add(index) as *mut T;
            std::ptr::write(ptr, item);
        }
        
        // 更新已初始化的元素数量
        let current_init = self.initialized.load(Ordering::Relaxed);
        if current_init < self.size {
            self.initialized.store((current_init + 1).min(self.size), Ordering::Relaxed);
        }
    }

    /// 获取最近的 N 个数据点的平均值
    pub fn recent_average(&self, count: usize) -> T 
    where T: std::ops::Add<Output = T> + std::ops::Div<f64, Output = T> + Copy {
        let initialized = self.initialized.load(Ordering::Relaxed);
        if initialized == 0 {
            return T::default();
        }

        let actual_count = count.min(initialized);
        let head = self.head.load(Ordering::Relaxed);
        
        let mut sum = T::default();
        for i in 0..actual_count {
            let index = (head + self.size - 1 - i) % self.size;
            unsafe {
                let item = std::ptr::read(self.buffer.as_ptr().add(index));
                sum = sum + item;
            }
        }
        
        sum / (actual_count as f64)
    }

    /// 获取最近的数据点
    pub fn get_recent(&self, count: usize) -> Vec<T> 
    where T: Clone {
        let initialized = self.initialized.load(Ordering::Relaxed);
        if initialized == 0 {
            return Vec::new();
        }

        let actual_count = count.min(initialized);
        let head = self.head.load(Ordering::Relaxed);
        let mut result = Vec::with_capacity(actual_count);
        
        for i in 0..actual_count {
            let index = (head + self.size - 1 - i) % self.size;
            unsafe {
                let item = std::ptr::read(self.buffer.as_ptr().add(index));
                result.push(item);
            }
        }
        
        result
    }
}

/// 实现 PerformanceMetrics 的数学运算
impl std::ops::Add for PerformanceMetrics {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        Self {
            latency_us: self.latency_us + other.latency_us,
            cpu_cycles: self.cpu_cycles + other.cpu_cycles,
            entries_processed: self.entries_processed + other.entries_processed,
            allocations: self.allocations + other.allocations,
            cache_misses: self.cache_misses + other.cache_misses,
            branch_mispredictions: self.branch_mispredictions + other.branch_mispredictions,
            timestamp: self.timestamp.max(other.timestamp),
        }
    }
}

impl std::ops::Div<f64> for PerformanceMetrics {
    type Output = Self;
    
    fn div(self, divisor: f64) -> Self {
        Self {
            latency_us: self.latency_us / divisor,
            cpu_cycles: (self.cpu_cycles as f64 / divisor) as u64,
            entries_processed: (self.entries_processed as f64 / divisor) as usize,
            allocations: (self.allocations as f64 / divisor) as u32,
            cache_misses: (self.cache_misses as f64 / divisor) as u32,
            branch_mispredictions: (self.branch_mispredictions as f64 / divisor) as u32,
            timestamp: self.timestamp,
        }
    }
}

impl Copy for PerformanceMetrics {}

/// 超高精度性能监控器
pub struct UltraPerformanceMonitor {
    /// 性能历史数据（最近 1000 次测量）
    performance_history: CircularBuffer<PerformanceMetrics>,
    /// CPU 频率，用于周期数到时间的转换
    cpu_frequency_hz: AtomicU64,
    /// 基准周期数
    baseline_cycles: AtomicU64,
    /// 监控统计
    total_measurements: AtomicU64,
    /// 异常检测阈值
    anomaly_threshold_multiplier: f64,
}

impl UltraPerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let monitor = Self {
            performance_history: CircularBuffer::new(1000),
            cpu_frequency_hz: AtomicU64::new(0),
            baseline_cycles: AtomicU64::new(0),
            total_measurements: AtomicU64::new(0),
            anomaly_threshold_multiplier: 3.0, // 3 倍标准差作为异常阈值
        };

        // 初始化 CPU 频率检测
        monitor.detect_cpu_frequency()?;
        monitor.calibrate_baseline()?;

        Ok(monitor)
    }

    /// 检测 CPU 频率
    fn detect_cpu_frequency(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_arch = "x86_64")]
        {
            // 尝试从 /proc/cpuinfo 读取 CPU 频率
            if let Ok(frequency) = Self::read_cpu_frequency_from_proc() {
                self.cpu_frequency_hz.store(frequency, Ordering::Relaxed);
                log::info!("Detected CPU frequency: {} MHz", frequency / 1_000_000);
                return Ok(());
            }

            // 如果无法读取，使用经验测量
            log::info!("Could not read CPU frequency from /proc/cpuinfo, using empirical measurement");
            let measured_freq = Self::measure_cpu_frequency()?;
            self.cpu_frequency_hz.store(measured_freq, Ordering::Relaxed);
            log::info!("Measured CPU frequency: {} MHz", measured_freq / 1_000_000);
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // 非 x86_64 架构的默认值
            self.cpu_frequency_hz.store(2_400_000_000, Ordering::Relaxed); // 假设 2.4GHz
            log::warn!("Using default CPU frequency assumption: 2.4 GHz");
        }

        Ok(())
    }

    /// 从 /proc/cpuinfo 读取 CPU 频率
    fn read_cpu_frequency_from_proc() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo")?;
        
        for line in cpuinfo.lines() {
            if line.starts_with("cpu MHz") {
                if let Some(freq_str) = line.split(':').nth(1) {
                    let freq_mhz: f64 = freq_str.trim().parse()?;
                    return Ok((freq_mhz * 1_000_000.0) as u64);
                }
            }
        }
        
        Err("CPU frequency not found in /proc/cpuinfo".into())
    }

    /// 经验测量 CPU 频率
    fn measure_cpu_frequency() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        let start_cycles = Self::get_cpu_cycles();
        
        // 等待一定时间进行测量
        std::thread::sleep(Duration::from_millis(100));
        
        let end_cycles = Self::get_cpu_cycles();
        let end_time = Instant::now();
        
        let elapsed_seconds = end_time.duration_since(start_time).as_secs_f64();
        let cycles_diff = end_cycles.wrapping_sub(start_cycles);
        
        let frequency = (cycles_diff as f64) / elapsed_seconds;
        Ok(frequency as u64)
    }

    /// 获取 CPU 周期数
    #[inline(always)]
    fn get_cpu_cycles() -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe { std::arch::x86_64::_rdtsc() }
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            // 非 x86_64 架构的 fallback
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
        }
    }

    /// 校准基准性能
    fn calibrate_baseline(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        log::info!("Calibrating performance baseline...");
        
        const CALIBRATION_ITERATIONS: usize = 1000;
        let mut total_cycles = 0u64;
        
        for _ in 0..CALIBRATION_ITERATIONS {
            let start = Self::get_cpu_cycles();
            
            // 执行一个简单的基准操作
            Self::benchmark_operation();
            
            let end = Self::get_cpu_cycles();
            total_cycles += end.wrapping_sub(start);
        }
        
        let baseline = total_cycles / (CALIBRATION_ITERATIONS as u64);
        self.baseline_cycles.store(baseline, Ordering::Relaxed);
        
        log::info!("Performance baseline calibrated: {} cycles", baseline);
        Ok(())
    }

    /// 基准操作，用于校准
    #[inline(never)]
    fn benchmark_operation() {
        let mut sum = 0u64;
        for i in 0..100 {
            sum = sum.wrapping_add(i);
        }
        // 防止编译器优化掉计算
        std::hint::black_box(sum);
    }

    /// 跟踪操作性能
    pub fn track_operation<F, T>(&self, operation: F) -> (T, PerformanceMetrics) 
    where F: FnOnce() -> T {
        let start_cycles = Self::get_cpu_cycles();
        let start_time = Instant::now();
        
        // 执行操作
        let result = operation();
        
        let end_cycles = Self::get_cpu_cycles();
        let end_time = Instant::now();
        
        // 计算性能指标
        let elapsed_cycles = end_cycles.wrapping_sub(start_cycles);
        let elapsed_us = end_time.duration_since(start_time).as_micros() as f64;
        
        let metrics = PerformanceMetrics {
            latency_us: elapsed_us,
            cpu_cycles: elapsed_cycles,
            entries_processed: 1, // 默认值，调用者可以覆盖
            allocations: 0,       // 需要外部计数器
            cache_misses: 0,      // 需要硬件计数器
            branch_mispredictions: 0, // 需要硬件计数器
            timestamp: end_time,
        };
        
        // 记录到历史数据
        self.performance_history.push(metrics);
        self.total_measurements.fetch_add(1, Ordering::Relaxed);
        
        (result, metrics)
    }

    /// 获取最近的平均性能
    pub fn get_recent_average(&self, sample_count: usize) -> PerformanceMetrics {
        self.performance_history.recent_average(sample_count)
    }

    /// 获取性能趋势分析
    pub fn analyze_performance_trend(&self, window_size: usize) -> PerformanceTrend {
        let recent_metrics = self.performance_history.get_recent(window_size);
        
        if recent_metrics.len() < 2 {
            return PerformanceTrend::Stable;
        }
        
        // 计算延迟趋势
        let first_half = &recent_metrics[recent_metrics.len()/2..];
        let second_half = &recent_metrics[..recent_metrics.len()/2];
        
        let first_avg = Self::calculate_average_latency(first_half);
        let second_avg = Self::calculate_average_latency(second_half);
        
        let change_ratio = (second_avg - first_avg) / first_avg;
        
        if change_ratio > 0.1 {
            PerformanceTrend::Degrading
        } else if change_ratio < -0.1 {
            PerformanceTrend::Improving
        } else {
            PerformanceTrend::Stable
        }
    }

    /// 计算平均延迟
    fn calculate_average_latency(metrics: &[PerformanceMetrics]) -> f64 {
        if metrics.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = metrics.iter().map(|m| m.latency_us).sum();
        sum / (metrics.len() as f64)
    }

    /// 检测性能异常
    pub fn detect_anomalies(&self, window_size: usize) -> Vec<PerformanceAnomaly> {
        let recent_metrics = self.performance_history.get_recent(window_size);
        
        if recent_metrics.len() < 10 {
            return Vec::new(); // 需要至少 10 个样本
        }
        
        let mut anomalies = Vec::new();
        
        // 计算延迟的平均值和标准差
        let latencies: Vec<f64> = recent_metrics.iter().map(|m| m.latency_us).collect();
        let mean = latencies.iter().sum::<f64>() / (latencies.len() as f64);
        let variance = latencies.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (latencies.len() as f64);
        let std_dev = variance.sqrt();
        
        let threshold = mean + std_dev * self.anomaly_threshold_multiplier;
        
        // 检查每个数据点
        for (i, &latency) in latencies.iter().enumerate() {
            if latency > threshold {
                anomalies.push(PerformanceAnomaly {
                    anomaly_type: AnomalyType::HighLatency,
                    severity: if latency > threshold * 2.0 { AnomalySeverity::Critical } else { AnomalySeverity::Warning },
                    value: latency,
                    threshold,
                    timestamp: recent_metrics[i].timestamp,
                    description: format!("Latency {:.2}µs exceeds threshold {:.2}µs", latency, threshold),
                });
            }
        }
        
        anomalies
    }

    /// 将 CPU 周期转换为微秒
    pub fn cycles_to_microseconds(&self, cycles: u64) -> f64 {
        let frequency = self.cpu_frequency_hz.load(Ordering::Relaxed);
        if frequency == 0 {
            return 0.0;
        }
        
        (cycles as f64) / (frequency as f64) * 1_000_000.0
    }

    /// 获取监控统计信息
    pub fn get_monitor_stats(&self) -> MonitorStats {
        let total = self.total_measurements.load(Ordering::Relaxed);
        let recent_avg = self.get_recent_average(100);
        let cpu_freq = self.cpu_frequency_hz.load(Ordering::Relaxed);
        let baseline = self.baseline_cycles.load(Ordering::Relaxed);
        
        MonitorStats {
            total_measurements: total,
            cpu_frequency_hz: cpu_freq,
            baseline_cycles: baseline,
            recent_average_latency_us: recent_avg.latency_us,
            recent_average_cycles: recent_avg.cpu_cycles,
        }
    }
}

/// 性能趋势枚举
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Degrading,
}

/// 性能异常类型
#[derive(Debug, Clone)]
pub enum AnomalyType {
    HighLatency,
    HighCycles,
    MemoryLeak,
    CacheThrashing,
}

/// 异常严重程度
#[derive(Debug, Clone)]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
}

/// 性能异常描述
#[derive(Debug, Clone)]
pub struct PerformanceAnomaly {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub value: f64,
    pub threshold: f64,
    pub timestamp: Instant,
    pub description: String,
}

/// 监控统计信息
#[derive(Debug, Clone)]
pub struct MonitorStats {
    pub total_measurements: u64,
    pub cpu_frequency_hz: u64,
    pub baseline_cycles: u64,
    pub recent_average_latency_us: f64,
    pub recent_average_cycles: u64,
}

/// 实时自调优器
pub struct RealTimeAutoTuner {
    /// 性能监控器引用
    monitor: Arc<UltraPerformanceMonitor>,
    /// 优化参数
    optimization_parameters: Arc<RwLock<OptimizationParameters>>,
    /// 调优历史
    tuning_history: Arc<Mutex<VecDeque<TuningAction>>>,
    /// 调优间隔
    tuning_interval: Duration,
    /// 最后调优时间
    last_tuning: Arc<Mutex<Instant>>,
}

/// 优化参数结构
#[derive(Debug, Clone)]
pub struct OptimizationParameters {
    pub aggression_level: f64,        // 0.0 - 1.0
    pub accuracy_level: f64,          // 0.0 - 1.0
    pub batch_size: usize,
    pub prefetch_distance: usize,
    pub parallel_threads: usize,
    pub cache_size: usize,
    pub enable_simd: bool,
    pub enable_avx512: bool,
}

impl Default for OptimizationParameters {
    fn default() -> Self {
        Self {
            aggression_level: 0.5,
            accuracy_level: 0.8,
            batch_size: 128,
            prefetch_distance: 4,
            parallel_threads: 4,
            cache_size: 1024,
            enable_simd: true,
            enable_avx512: false,
        }
    }
}

/// 调优动作记录
#[derive(Debug, Clone)]
pub struct TuningAction {
    pub timestamp: Instant,
    pub parameter_name: String,
    pub old_value: f64,
    pub new_value: f64,
    pub reason: String,
    pub performance_before: PerformanceMetrics,
    pub performance_after: Option<PerformanceMetrics>,
}

impl RealTimeAutoTuner {
    /// 创建新的自调优器
    pub fn new(monitor: Arc<UltraPerformanceMonitor>) -> Self {
        Self {
            monitor,
            optimization_parameters: Arc::new(RwLock::new(OptimizationParameters::default())),
            tuning_history: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            tuning_interval: Duration::from_secs(30), // 每 30 秒调优一次
            last_tuning: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// 执行自动调优
    pub fn auto_tune(&self) -> Result<Vec<TuningAction>, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_tuning = self.last_tuning.lock().map_err(|_| "Lock poisoned")?;
        
        // 检查是否需要调优
        if last_tuning.elapsed() < self.tuning_interval {
            return Ok(Vec::new());
        }
        
        *last_tuning = Instant::now();
        
        let recent_perf = self.monitor.get_recent_average(100);
        let trend = self.monitor.analyze_performance_trend(200);
        let anomalies = self.monitor.detect_anomalies(500);
        
        let mut actions = Vec::new();
        
        // 基于性能趋势进行调优
        match trend {
            PerformanceTrend::Degrading => {
                actions.extend(self.handle_performance_degradation(&recent_perf)?);
            },
            PerformanceTrend::Improving => {
                actions.extend(self.handle_performance_improvement(&recent_perf)?);
            },
            PerformanceTrend::Stable => {
                // 性能稳定时进行小幅优化实验
                actions.extend(self.handle_stable_performance(&recent_perf)?);
            }
        }
        
        // 处理异常情况
        for anomaly in anomalies {
            actions.extend(self.handle_anomaly(&anomaly, &recent_perf)?);
        }
        
        // 记录调优动作
        let mut history = self.tuning_history.lock().map_err(|_| "Lock poisoned")?;
        for action in &actions {
            history.push_back(action.clone());
            if history.len() > 1000 {
                history.pop_front();
            }
        }
        
        if !actions.is_empty() {
            log::info!("Auto-tuning completed with {} actions", actions.len());
        }
        
        Ok(actions)
    }

    /// 处理性能降级
    fn handle_performance_degradation(&self, current_perf: &PerformanceMetrics) -> Result<Vec<TuningAction>, Box<dyn std::error::Error + Send + Sync>> {
        let mut actions = Vec::new();
        let mut params = self.optimization_parameters.write().map_err(|_| "Lock poisoned")?;
        
        // 提高激进程度，降低精度要求
        if current_perf.latency_us > 200.0 && params.aggression_level < 0.9 {
            let old_value = params.aggression_level;
            params.aggression_level = (params.aggression_level + 0.1).min(1.0);
            
            actions.push(TuningAction {
                timestamp: Instant::now(),
                parameter_name: "aggression_level".to_string(),
                old_value,
                new_value: params.aggression_level,
                reason: format!("High latency detected: {:.2}µs", current_perf.latency_us),
                performance_before: *current_perf,
                performance_after: None,
            });
        }
        
        // 减少精度要求以提高速度
        if current_perf.latency_us > 150.0 && params.accuracy_level > 0.5 {
            let old_value = params.accuracy_level;
            params.accuracy_level = (params.accuracy_level - 0.1).max(0.5);
            
            actions.push(TuningAction {
                timestamp: Instant::now(),
                parameter_name: "accuracy_level".to_string(),
                old_value,
                new_value: params.accuracy_level,
                reason: "Reducing accuracy to improve speed".to_string(),
                performance_before: *current_perf,
                performance_after: None,
            });
        }
        
        // 启用更多并行化
        if params.parallel_threads < 8 {
            let old_value = params.parallel_threads as f64;
            params.parallel_threads = (params.parallel_threads + 1).min(8);
            
            actions.push(TuningAction {
                timestamp: Instant::now(),
                parameter_name: "parallel_threads".to_string(),
                old_value,
                new_value: params.parallel_threads as f64,
                reason: "Increasing parallelism to handle degradation".to_string(),
                performance_before: *current_perf,
                performance_after: None,
            });
        }
        
        Ok(actions)
    }

    /// 处理性能改善
    fn handle_performance_improvement(&self, current_perf: &PerformanceMetrics) -> Result<Vec<TuningAction>, Box<dyn std::error::Error + Send + Sync>> {
        let mut actions = Vec::new();
        let mut params = self.optimization_parameters.write().map_err(|_| "Lock poisoned")?;
        
        // 性能充足时，可以提高精度
        if current_perf.latency_us < 50.0 && params.accuracy_level < 0.95 {
            let old_value = params.accuracy_level;
            params.accuracy_level = (params.accuracy_level + 0.05).min(0.95);
            
            actions.push(TuningAction {
                timestamp: Instant::now(),
                parameter_name: "accuracy_level".to_string(),
                old_value,
                new_value: params.accuracy_level,
                reason: format!("Low latency allows higher accuracy: {:.2}µs", current_perf.latency_us),
                performance_before: *current_perf,
                performance_after: None,
            });
        }
        
        // 可以降低激进程度，节省资源
        if current_perf.latency_us < 30.0 && params.aggression_level > 0.3 {
            let old_value = params.aggression_level;
            params.aggression_level = (params.aggression_level - 0.05).max(0.3);
            
            actions.push(TuningAction {
                timestamp: Instant::now(),
                parameter_name: "aggression_level".to_string(),
                old_value,
                new_value: params.aggression_level,
                reason: "Excellent performance allows resource conservation".to_string(),
                performance_before: *current_perf,
                performance_after: None,
            });
        }
        
        Ok(actions)
    }

    /// 处理稳定性能
    fn handle_stable_performance(&self, current_perf: &PerformanceMetrics) -> Result<Vec<TuningAction>, Box<dyn std::error::Error + Send + Sync>> {
        let mut actions = Vec::new();
        let mut params = self.optimization_parameters.write().map_err(|_| "Lock poisoned")?;
        
        // 性能稳定时，可以进行小幅实验性调整
        if current_perf.latency_us > 100.0 && current_perf.latency_us < 200.0 {
            // 尝试增加批处理大小
            if params.batch_size < 256 {
                let old_value = params.batch_size as f64;
                params.batch_size = (params.batch_size * 11 / 10).min(256); // 增加 10%
                
                actions.push(TuningAction {
                    timestamp: Instant::now(),
                    parameter_name: "batch_size".to_string(),
                    old_value,
                    new_value: params.batch_size as f64,
                    reason: "Experimental batch size increase during stable period".to_string(),
                    performance_before: *current_perf,
                    performance_after: None,
                });
            }
        }
        
        Ok(actions)
    }

    /// 处理异常情况
    fn handle_anomaly(&self, anomaly: &PerformanceAnomaly, current_perf: &PerformanceMetrics) -> Result<Vec<TuningAction>, Box<dyn std::error::Error + Send + Sync>> {
        let mut actions = Vec::new();
        let mut params = self.optimization_parameters.write().map_err(|_| "Lock poisoned")?;
        
        match anomaly.anomaly_type {
            AnomalyType::HighLatency => {
                // 高延迟异常：立即采取激进措施
                if params.aggression_level < 1.0 {
                    let old_value = params.aggression_level;
                    params.aggression_level = 1.0;
                    
                    actions.push(TuningAction {
                        timestamp: Instant::now(),
                        parameter_name: "aggression_level".to_string(),
                        old_value,
                        new_value: params.aggression_level,
                        reason: format!("Emergency response to high latency anomaly: {}", anomaly.description),
                        performance_before: *current_perf,
                        performance_after: None,
                    });
                }
            },
            
            AnomalyType::CacheThrashing => {
                // 缓存抖动：调整预取距离
                if params.prefetch_distance > 1 {
                    let old_value = params.prefetch_distance as f64;
                    params.prefetch_distance = params.prefetch_distance / 2;
                    
                    actions.push(TuningAction {
                        timestamp: Instant::now(),
                        parameter_name: "prefetch_distance".to_string(),
                        old_value,
                        new_value: params.prefetch_distance as f64,
                        reason: "Reducing prefetch distance due to cache thrashing".to_string(),
                        performance_before: *current_perf,
                        performance_after: None,
                    });
                }
            },
            
            _ => {
                // 其他异常类型的通用处理
                log::warn!("Unhandled anomaly type: {:?}", anomaly.anomaly_type);
            }
        }
        
        Ok(actions)
    }

    /// 获取当前优化参数
    pub fn get_parameters(&self) -> Result<OptimizationParameters, Box<dyn std::error::Error + Send + Sync>> {
        let params = self.optimization_parameters.read().map_err(|_| "Lock poisoned")?;
        Ok(params.clone())
    }

    /// 获取调优历史
    pub fn get_tuning_history(&self, count: usize) -> Result<Vec<TuningAction>, Box<dyn std::error::Error + Send + Sync>> {
        let history = self.tuning_history.lock().map_err(|_| "Lock poisoned")?;
        Ok(history.iter().rev().take(count).cloned().collect())
    }
}

/// 全局性能监控器实例
static mut GLOBAL_PERFORMANCE_MONITOR: Option<Arc<UltraPerformanceMonitor>> = None;
static MONITOR_INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局性能监控器
pub fn get_performance_monitor() -> Arc<UltraPerformanceMonitor> {
    unsafe {
        MONITOR_INIT.call_once(|| {
            match UltraPerformanceMonitor::new() {
                Ok(monitor) => {
                    GLOBAL_PERFORMANCE_MONITOR = Some(Arc::new(monitor));
                    log::info!("Global ultra performance monitor initialized");
                },
                Err(e) => {
                    log::error!("Failed to initialize performance monitor: {}", e);
                    panic!("Critical: Cannot initialize performance monitor");
                }
            }
        });
        
        GLOBAL_PERFORMANCE_MONITOR.as_ref().expect("Global instance not initialized").clone()
    }
}

/// 全局自调优器实例
static mut GLOBAL_AUTO_TUNER: Option<Arc<RealTimeAutoTuner>> = None;
static TUNER_INIT: std::sync::Once = std::sync::Once::new();

/// 获取全局自调优器
pub fn get_auto_tuner() -> Arc<RealTimeAutoTuner> {
    unsafe {
        TUNER_INIT.call_once(|| {
            let monitor = get_performance_monitor();
            let tuner = RealTimeAutoTuner::new(monitor);
            GLOBAL_AUTO_TUNER = Some(Arc::new(tuner));
            log::info!("Global real-time auto-tuner initialized");
        });
        
        GLOBAL_AUTO_TUNER.as_ref().expect("Global instance not initialized").clone()
    }
}

/// 优化建议枚举
#[derive(Debug, Clone)]
pub enum OptimizationSuggestion {
    IncreaseParallelism,
    OptimizeMemoryUsage,
    TuneCpuAffinity,
    SwitchToFastPath,
}

/// 实时性能监控器主结构
pub struct RealTimePerformanceMonitor {
    metrics_history: Arc<Mutex<CircularBuffer<PerformanceMetrics>>>,
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    monitoring_active: AtomicU32,
}

impl RealTimePerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics_history: Arc::new(Mutex::new(CircularBuffer::new(1000))),
            current_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            monitoring_active: AtomicU32::new(0),
        }
    }

    /// 开始操作跟踪
    pub async fn start_operation(&self, _operation_name: &str) {
        // 开始性能跟踪
    }

    /// 结束操作跟踪
    pub async fn end_operation(&self, _operation_name: &str, duration: Duration) {
        let mut metrics = self.current_metrics.write().expect("Failed to acquire write lock");
        metrics.latency_us = duration.as_micros() as f64;
        metrics.timestamp = Instant::now();
    }

    /// 获取当前性能指标
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        self.current_metrics.read().expect("Failed to acquire read lock").clone()
    }

    /// 获取优化建议
    pub async fn get_optimization_suggestion(&self) -> Option<OptimizationSuggestion> {
        let metrics = self.current_metrics.read().expect("Failed to acquire read lock");
        
        if metrics.latency_us > 200.0 {
            Some(OptimizationSuggestion::SwitchToFastPath)
        } else if metrics.allocations > 100 {
            Some(OptimizationSuggestion::OptimizeMemoryUsage)
        } else {
            None
        }
    }

    /// 启动监控
    pub async fn start_monitoring(&self) {
        self.monitoring_active.store(1, Ordering::Relaxed);
        log::info!("🚀 启动实时性能监控");
    }

    /// 停止监控
    pub async fn stop_monitoring(&self) {
        self.monitoring_active.store(0, Ordering::Relaxed);
        log::info!("🚀 停止实时性能监控");
    }

    /// 重置指标
    pub async fn reset_metrics(&self) {
        let mut metrics = self.current_metrics.write().expect("Failed to acquire write lock");
        *metrics = PerformanceMetrics::default();
    }
}

/// 优化建议枚举
#[derive(Debug, Clone)]
pub enum OptimizationSuggestion {
    IncreaseParallelism,
    OptimizeMemoryUsage,
    TuneCpuAffinity,
    SwitchToFastPath,
}

/// 实时性能监控器
pub struct RealTimePerformanceMonitor {
    current_metrics: Arc<RwLock<PerformanceMetrics>>,
    history: Arc<Mutex<VecDeque<PerformanceMetrics>>>,
    monitoring_enabled: Arc<std::sync::atomic::AtomicBool>,
}

impl RealTimePerformanceMonitor {
    pub fn new() -> Self {
        Self {
            current_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            history: Arc::new(Mutex::new(VecDeque::new())),
            monitoring_enabled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// 开始操作跟踪
    pub async fn start_operation(&self, _operation: &str) {
        if !self.monitoring_enabled.load(Ordering::Relaxed) {
            return;
        }
        // 记录操作开始时间
    }

    /// 结束操作跟踪
    pub async fn end_operation(&self, _operation: &str, duration: Duration) {
        if !self.monitoring_enabled.load(Ordering::Relaxed) {
            return;
        }
        
        let mut metrics = self.current_metrics.write().expect("Failed to acquire write lock");
        metrics.latency_us = duration.as_micros() as f64;
        metrics.timestamp = Instant::now();
    }

    /// 获取当前性能指标
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        let metrics = self.current_metrics.read().expect("Failed to acquire read lock");
        metrics.clone()
    }

    /// 获取优化建议
    pub async fn get_optimization_suggestion(&self) -> Option<OptimizationSuggestion> {
        let metrics = self.current_metrics.read().expect("Failed to acquire read lock");
        
        // 简单的启发式规则
        if metrics.latency_us > 1000.0 {
            Some(OptimizationSuggestion::IncreaseParallelism)
        } else if metrics.allocations > 100 {
            Some(OptimizationSuggestion::OptimizeMemoryUsage)
        } else {
            None
        }
    }

    /// 开始监控
    pub async fn start_monitoring(&self) {
        self.monitoring_enabled.store(true, Ordering::Relaxed);
        log::info!("🚀 实时性能监控已启动");
    }

    /// 停止监控
    pub async fn stop_monitoring(&self) {
        self.monitoring_enabled.store(false, Ordering::Relaxed);
        log::info!("🚀 实时性能监控已停止");
    }

    /// 重置指标
    pub async fn reset_metrics(&self) {
        let mut metrics = self.current_metrics.write().expect("Failed to acquire write lock");
        *metrics = PerformanceMetrics::default();
        
        let mut history = self.history.lock().expect("Failed to acquire mutex lock");
        history.clear();
    }
}
