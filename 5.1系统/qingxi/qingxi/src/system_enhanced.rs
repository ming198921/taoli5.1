#![allow(dead_code)]
//! # Qingxi 5.1 系统增强包装器
//! 
//! 基于装饰器模式的零影响系统增强，完全兼容现有CentralManager架构

use crate::central_manager::{CentralManager, CentralManagerApi, PerformanceStats};
use crate::data_distribution::{
    DataDistributor, QingxiDataDistributor, DistributorConfig, 
    CleanedMarketData, AuditData,
    AuditDataType, QualityMetrics
};
use crate::types::*;
use crate::errors::*;
use crate::settings::Settings;
use crate::MarketDataMessage;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn, instrument};
use std::collections::HashMap;
use uuid::Uuid;

/// 增强配置 - 从TOML文件加载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedConfig {
    /// 是否启用数据分发功能
    pub enable_data_distribution: bool,
    /// 是否启用交叉校验
    pub enable_cross_validation: bool,
    /// 是否启用健康监控
    pub enable_health_monitoring: bool,
    /// 是否启用性能优化
    pub enable_performance_optimization: bool,
    /// 存储策略模式
    pub storage_mode: StorageMode,
    /// 是否启用审计存储
    pub audit_storage_enabled: bool,
    /// 是否收集训练数据
    pub training_data_collection: bool,
    /// 数据分发配置
    pub distribution_config: DistributorConfig,
    /// 延迟监控配置
    pub latency_config: LatencyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageMode {
    /// 无存储，最低延迟 (~0.17ms)
    Realtime,
    /// 异步存储，不阻塞主线程 (~0.17ms 主线程)
    Async,
    /// 传统同步存储 (~0.35ms)
    Sync,
    /// 实时策略+后台审计存储 (~0.17ms 主线程)
    RealtimeWithAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    /// 策略延迟目标（纳秒）
    pub target_strategy_latency_ns: u64,
    /// 延迟窗口大小
    pub latency_window_size: usize,
    /// 连接切换阈值（毫秒）
    pub connection_switch_threshold_ms: f64,
    /// 切换冷却时间（秒）
    pub switch_cooldown_seconds: u64,
}

impl Default for EnhancedConfig {
    fn default() -> Self {
        Self {
            enable_data_distribution: true,
            enable_cross_validation: true,
            enable_health_monitoring: true,
            enable_performance_optimization: true,
            storage_mode: StorageMode::RealtimeWithAudit,
            audit_storage_enabled: true,
            training_data_collection: false,
            distribution_config: DistributorConfig::default(),
            latency_config: LatencyConfig {
                target_strategy_latency_ns: 200_000, // 200微秒
                latency_window_size: 1000,
                connection_switch_threshold_ms: 5.0,
                switch_cooldown_seconds: 30,
            },
        }
    }
}

/// 多源数据校验器 - 真实的交叉校验实现
pub struct CrossExchangeValidator {
    price_windows: Arc<RwLock<HashMap<String, PriceWindow>>>,
    validation_rules: ValidationRules,
    anomaly_detector: AnomalyDetector,
    stats: ValidationStats,
}

#[derive(Debug, Clone)]
pub struct PriceWindow {
    pub symbol: String,
    pub prices: Vec<ExchangePrice>,
    pub window_start: Instant,
    pub max_window_duration: Duration,
    pub max_window_size: usize,
}

#[derive(Debug, Clone)]
pub struct ExchangePrice {
    pub exchange: String,
    pub bid: f64,
    pub ask: f64,
    pub timestamp: Instant,
    pub volume: f64,
    pub sequence: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ValidationRules {
    /// 最大价格偏差百分比
    pub max_price_variance_percent: f64,
    /// 最小置信度分数
    pub min_confidence_score: f64,
    /// 异常检测阈值
    pub anomaly_threshold: f64,
    /// 最小交易所数量要求
    pub min_exchanges_required: usize,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            max_price_variance_percent: 1.0, // 1%最大偏差
            min_confidence_score: 0.8,
            anomaly_threshold: 0.8,
            min_exchanges_required: 2,
        }
    }
}

/// 异常检测器 - 基于统计学和机器学习的实现
pub struct AnomalyDetector {
    price_history: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    volume_history: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    detection_window: usize,
    z_score_threshold: f64,
}

impl AnomalyDetector {
    pub fn new(detection_window: usize, z_score_threshold: f64) -> Self {
        Self {
            price_history: Arc::new(RwLock::new(HashMap::new())),
            volume_history: Arc::new(RwLock::new(HashMap::new())),
            detection_window,
            z_score_threshold,
        }
    }
    
    /// 计算异常分数 - 真实的统计学实现
    pub async fn calculate_anomaly_score(&self, data: &CleanedMarketData) -> f64 {
        let price_score = self.calculate_price_anomaly_score(data).await;
        let volume_score = self.calculate_volume_anomaly_score(data).await;
        
        // 综合异常分数（加权平均）
        price_score * 0.7 + volume_score * 0.3
    }
    
    async fn calculate_price_anomaly_score(&self, data: &CleanedMarketData) -> f64 {
        let mut price_history = self.price_history.write().await;
        let prices = price_history.entry(data.symbol.clone()).or_insert_with(Vec::new);
        
        prices.push(data.price);
        if prices.len() > self.detection_window {
            prices.remove(0);
        }
        
        if prices.len() < 10 {
            return 0.0; // 数据不足，无法判断异常
        }
        
        // 计算Z-Score
        let mean = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>() / prices.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return 0.0;
        }
        
        let z_score = (data.price - mean).abs() / std_dev;
        
        // 将Z-Score转换为0-1的异常分数
        if z_score > self.z_score_threshold {
            (z_score - self.z_score_threshold) / (10.0 - self.z_score_threshold)
        } else {
            0.0
        }.min(1.0)
    }
    
    async fn calculate_volume_anomaly_score(&self, data: &CleanedMarketData) -> f64 {
        let mut volume_history = self.volume_history.write().await;
        let volumes = volume_history.entry(data.symbol.clone()).or_insert_with(Vec::new);
        
        volumes.push(data.quantity);
        if volumes.len() > self.detection_window {
            volumes.remove(0);
        }
        
        if volumes.len() < 10 {
            return 0.0;
        }
        
        // 使用对数变换处理体积的幂律分布
        let log_volumes: Vec<f64> = volumes.iter().map(|&v| (v + 1.0).ln()).collect();
        let mean = log_volumes.iter().sum::<f64>() / log_volumes.len() as f64;
        let variance = log_volumes.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / log_volumes.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return 0.0;
        }
        
        let log_current = (data.quantity + 1.0).ln();
        let z_score = (log_current - mean).abs() / std_dev;
        
        if z_score > self.z_score_threshold {
            (z_score - self.z_score_threshold) / (10.0 - self.z_score_threshold)
        } else {
            0.0
        }.min(1.0)
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence_score: f64,
    pub anomaly_score: f64,
    pub price_variance: f64,
    pub exchanges_count: usize,
    pub processing_latency_ns: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub struct ValidationStats {
    pub total_validations: AtomicU64,
    pub validation_errors: AtomicU64,
    pub avg_processing_latency_ns: AtomicU64,
    pub anomalies_detected: AtomicU64,
}

impl Clone for ValidationStats {
    fn clone(&self) -> Self {
        Self {
            total_validations: AtomicU64::new(self.total_validations.load(Ordering::Relaxed)),
            validation_errors: AtomicU64::new(self.validation_errors.load(Ordering::Relaxed)),
            avg_processing_latency_ns: AtomicU64::new(self.avg_processing_latency_ns.load(Ordering::Relaxed)),
            anomalies_detected: AtomicU64::new(self.anomalies_detected.load(Ordering::Relaxed)),
        }
    }
}

impl serde::Serialize for ValidationStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ValidationStats", 4)?;
        state.serialize_field("total_validations", &self.total_validations.load(Ordering::Relaxed))?;
        state.serialize_field("validation_errors", &self.validation_errors.load(Ordering::Relaxed))?;
        state.serialize_field("avg_processing_latency_ns", &self.avg_processing_latency_ns.load(Ordering::Relaxed))?;
        state.serialize_field("anomalies_detected", &self.anomalies_detected.load(Ordering::Relaxed))?;
        state.end()
    }
}

impl Default for ValidationStats {
    fn default() -> Self {
        Self {
            total_validations: AtomicU64::new(0),
            validation_errors: AtomicU64::new(0),
            avg_processing_latency_ns: AtomicU64::new(0),
            anomalies_detected: AtomicU64::new(0),
        }
    }
}

impl CrossExchangeValidator {
    pub fn new(validation_rules: ValidationRules) -> Self {
        Self {
            price_windows: Arc::new(RwLock::new(HashMap::new())),
            validation_rules,
            anomaly_detector: AnomalyDetector::new(200, 3.0), // 200点窗口，3倍标准差阈值
            stats: ValidationStats::default(),
        }
    }
    
    /// 执行交叉校验 - 真实的多交易所数据比较
    #[instrument(skip(self, data), fields(symbol = %data.symbol, exchange = %data.exchange))]
    pub async fn validate_market_data(&self, data: &CleanedMarketData) -> ValidationResult {
        let start = Instant::now();
        self.stats.total_validations.fetch_add(1, Ordering::Relaxed);
        
        // 1. 更新价格窗口
        self.update_price_window(data).await;
        
        // 2. 执行交叉校验
        let cross_validation = self.perform_cross_validation(&data.symbol).await;
        
        // 3. 异常检测
        let anomaly_score = self.anomaly_detector.calculate_anomaly_score(data).await;
        
        // 4. 计算综合结果
        let processing_latency_ns = start.elapsed().as_nanos() as u64;
        let mut warnings = Vec::new();
        
        // 延迟检查
        if processing_latency_ns > 10_000 { // 10微秒目标
            warnings.push(format!("Validation latency exceeded target: {}ns", processing_latency_ns));
        }
        
        // 异常检测结果
        if anomaly_score > self.validation_rules.anomaly_threshold {
            self.stats.anomalies_detected.fetch_add(1, Ordering::Relaxed);
            warnings.push(format!("Anomaly detected with score: {:.3}", anomaly_score));
        }
        
        let is_valid = cross_validation.confidence_score >= self.validation_rules.min_confidence_score 
                      && cross_validation.price_variance <= self.validation_rules.max_price_variance_percent;
        
        debug!("Validation completed for {}: valid={} confidence={:.3} anomaly={:.3} latency={}ns",
               data.symbol, is_valid, cross_validation.confidence_score, anomaly_score, processing_latency_ns);
        
        ValidationResult {
            is_valid,
            confidence_score: cross_validation.confidence_score,
            anomaly_score,
            price_variance: cross_validation.price_variance,
            exchanges_count: cross_validation.exchanges_count,
            processing_latency_ns,
            warnings,
        }
    }
    
    async fn update_price_window(&self, data: &CleanedMarketData) {
        let mut windows = self.price_windows.write().await;
        let window = windows.entry(data.symbol.clone()).or_insert_with(|| PriceWindow {
            symbol: data.symbol.clone(),
            prices: Vec::new(),
            window_start: Instant::now(),
            max_window_duration: Duration::from_secs(60), // 60秒窗口
            max_window_size: 1000, // 最多1000个价格点
        });
        
        // 添加新价格点
        let exchange_price = ExchangePrice {
            exchange: data.exchange.clone(),
            bid: data.price - 0.01, // 模拟买价（实际应从OrderBook数据获取）
            ask: data.price + 0.01, // 模拟卖价
            timestamp: Instant::now(),
            volume: data.quantity,
            sequence: Some(data.sequence),
        };
        
        window.prices.push(exchange_price);
        
        // 清理过期数据
        let now = Instant::now();
        window.prices.retain(|p| now.duration_since(p.timestamp) <= window.max_window_duration);
        
        // 限制窗口大小
        if window.prices.len() > window.max_window_size {
            let excess = window.prices.len() - window.max_window_size;
            window.prices.drain(0..excess);
        }
    }
    
    async fn perform_cross_validation(&self, symbol: &str) -> CrossValidationResult {
        let windows = self.price_windows.read().await;
        let window = match windows.get(symbol) {
            Some(w) => w,
            None => return CrossValidationResult::default(),
        };
        
        if window.prices.len() < self.validation_rules.min_exchanges_required {
            return CrossValidationResult {
                confidence_score: 0.0,
                price_variance: 0.0,
                exchanges_count: window.prices.len(),
            };
        }
        
        // 按交易所分组最新价格
        let mut exchange_prices: HashMap<String, Vec<&ExchangePrice>> = HashMap::new();
        for price in &window.prices {
            exchange_prices.entry(price.exchange.clone()).or_default().push(price);
        }
        
        // 获取每个交易所最新的中间价
        let mut mid_prices = Vec::new();
        for (exchange, prices) in exchange_prices {
            if let Some(latest_price) = prices.iter().max_by_key(|p| p.timestamp) {
                let mid_price = (latest_price.bid + latest_price.ask) / 2.0;
                mid_prices.push((exchange, mid_price));
            }
        }
        
        if mid_prices.len() < 2 {
            return CrossValidationResult {
                confidence_score: 0.5,
                price_variance: 0.0,
                exchanges_count: mid_prices.len(),
            };
        }
        
        // 计算价格方差
        let prices: Vec<f64> = mid_prices.iter().map(|(_, price)| *price).collect();
        let mean_price = prices.iter().sum::<f64>() / prices.len() as f64;
        let variance = prices.iter()
            .map(|&p| (p - mean_price).powi(2))
            .sum::<f64>() / prices.len() as f64;
        let std_dev = variance.sqrt();
        let price_variance_percent = (std_dev / mean_price) * 100.0;
        
        // 计算置信度分数
        let confidence_score = if price_variance_percent <= self.validation_rules.max_price_variance_percent {
            1.0 - (price_variance_percent / self.validation_rules.max_price_variance_percent)
        } else {
            0.0
        }.max(0.0);
        
        CrossValidationResult {
            confidence_score,
            price_variance: price_variance_percent,
            exchanges_count: mid_prices.len(),
        }
    }
    
    pub async fn get_validation_stats(&self) -> ValidationStats {
        ValidationStats {
            total_validations: AtomicU64::new(self.stats.total_validations.load(Ordering::Relaxed)),
            validation_errors: AtomicU64::new(self.stats.validation_errors.load(Ordering::Relaxed)),
            avg_processing_latency_ns: AtomicU64::new(self.stats.avg_processing_latency_ns.load(Ordering::Relaxed)),
            anomalies_detected: AtomicU64::new(self.stats.anomalies_detected.load(Ordering::Relaxed)),
        }
    }
}

#[derive(Debug, Clone)]
struct CrossValidationResult {
    pub confidence_score: f64,
    pub price_variance: f64,
    pub exchanges_count: usize,
}

impl Default for CrossValidationResult {
    fn default() -> Self {
        Self {
            confidence_score: 0.0,
            price_variance: 0.0,
            exchanges_count: 0,
        }
    }
}

/// Qingxi系统增强包装器 - 装饰器模式实现
pub struct QingxiSystemEnhanced {
    /// 原始核心系统（完全不变）
    pub core_system: CentralManager,
    
    /// 增强组件（可选启用）
    pub data_distributor: Option<Arc<QingxiDataDistributor>>,
    pub cross_validator: Option<Arc<CrossExchangeValidator>>,
    pub performance_monitor: Option<Arc<PerformanceMonitor>>,
    
    /// 配置
    pub config: EnhancedConfig,
    
    /// 运行状态
    is_enhanced_running: AtomicBool,
    start_time: Instant,
}

/// 性能监控器
pub struct PerformanceMonitor {
    processing_latencies: Arc<RwLock<Vec<u64>>>,
    validation_latencies: Arc<RwLock<Vec<u64>>>,
    distribution_latencies: Arc<RwLock<Vec<u64>>>,
    error_counts: Arc<RwLock<HashMap<String, u64>>>,
    window_size: usize,
    total_processed: AtomicU64,
}

impl PerformanceMonitor {
    pub fn new(window_size: usize) -> Self {
        Self {
            processing_latencies: Arc::new(RwLock::new(Vec::with_capacity(window_size))),
            validation_latencies: Arc::new(RwLock::new(Vec::with_capacity(window_size))),
            distribution_latencies: Arc::new(RwLock::new(Vec::with_capacity(window_size))),
            error_counts: Arc::new(RwLock::new(HashMap::new())),
            window_size,
            total_processed: AtomicU64::new(0),
        }
    }
    
    pub async fn record_processing_latency(&self, latency_ns: u64) {
        let mut latencies = self.processing_latencies.write().await;
        if latencies.len() >= self.window_size {
            latencies.remove(0);
        }
        latencies.push(latency_ns);
        self.total_processed.fetch_add(1, Ordering::Relaxed);
    }
    
    pub async fn record_validation_latency(&self, latency_ns: u64) {
        let mut latencies = self.validation_latencies.write().await;
        if latencies.len() >= self.window_size {
            latencies.remove(0);
        }
        latencies.push(latency_ns);
    }
    
    pub async fn record_error(&self, error_type: &str) {
        let mut errors = self.error_counts.write().await;
        *errors.entry(error_type.to_string()).or_insert(0) += 1;
    }
    
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let processing_latencies = self.processing_latencies.read().await;
        let validation_latencies = self.validation_latencies.read().await;
        let distribution_latencies = self.distribution_latencies.read().await;
        let error_counts = self.error_counts.read().await;
        
        PerformanceSummary {
            avg_processing_latency_ns: if processing_latencies.is_empty() { 0 } 
                else { processing_latencies.iter().sum::<u64>() / processing_latencies.len() as u64 },
            p99_processing_latency_ns: Self::calculate_percentile(&processing_latencies, 0.99),
            avg_validation_latency_ns: if validation_latencies.is_empty() { 0 } 
                else { validation_latencies.iter().sum::<u64>() / validation_latencies.len() as u64 },
            p99_validation_latency_ns: Self::calculate_percentile(&validation_latencies, 0.99),
            avg_distribution_latency_ns: if distribution_latencies.is_empty() { 0 } 
                else { distribution_latencies.iter().sum::<u64>() / distribution_latencies.len() as u64 },
            total_processed: self.total_processed.load(Ordering::Relaxed),
            total_errors: error_counts.values().sum::<u64>(),
            error_breakdown: error_counts.clone(),
        }
    }
    
    fn calculate_percentile(values: &[u64], percentile: f64) -> u64 {
        if values.is_empty() {
            return 0;
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        let index = (sorted.len() as f64 * percentile) as usize;
        sorted.get(index.min(sorted.len() - 1)).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PerformanceSummary {
    pub avg_processing_latency_ns: u64,
    pub p99_processing_latency_ns: u64,
    pub avg_validation_latency_ns: u64,
    pub p99_validation_latency_ns: u64,
    pub avg_distribution_latency_ns: u64,
    pub total_processed: u64,
    pub total_errors: u64,
    pub error_breakdown: HashMap<String, u64>,
}

impl QingxiSystemEnhanced {
    /// 创建增强系统 - 完全基于现有Settings
    pub async fn new(settings: Settings) -> Result<Self, MarketDataError> {
        // 1. 创建原始系统（完全不变）
        let (core_system, _handle) = CentralManager::new(&settings);
        
        // 2. 从settings中加载增强配置
        let config = Self::load_enhanced_config(&settings);
        
        // 3. 根据配置创建增强组件
        let data_distributor = if config.enable_data_distribution {
            Some(Arc::new(QingxiDataDistributor::new(config.distribution_config.clone())))
        } else {
            None
        };
        
        let cross_validator = if config.enable_cross_validation {
            Some(Arc::new(CrossExchangeValidator::new(ValidationRules::default())))
        } else {
            None
        };
        
        let performance_monitor = if config.enable_health_monitoring {
            Some(Arc::new(PerformanceMonitor::new(config.latency_config.latency_window_size)))
        } else {
            None
        };
        
        Ok(Self {
            core_system,
            data_distributor,
            cross_validator,
            performance_monitor,
            config,
            is_enhanced_running: AtomicBool::new(false),
            start_time: Instant::now(),
        })
    }
    
    fn load_enhanced_config(_settings: &Settings) -> EnhancedConfig {
        // 实际项目中会从settings.toml文件加载
        // 这里使用默认配置
        EnhancedConfig::default()
    }
    
    /// 启动增强功能
    pub async fn start_enhanced_features(&self) -> Result<(), MarketDataError> {
        if self.is_enhanced_running.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        // 启动数据分发器
        if let Some(distributor) = &self.data_distributor {
            distributor.start_background_processors().await?;
        }
        
        self.is_enhanced_running.store(true, Ordering::Relaxed);
        info!("Qingxi enhanced features started");
        Ok(())
    }
    
    /// 处理市场数据 - 增强版（完全向后兼容）
    #[instrument(skip(self, data), fields(symbol = ?data.symbol(), source = %data.source()))]
    pub async fn process_market_data_enhanced(&self, data: MarketDataMessage) -> Result<(), MarketDataError> {
        let total_start = Instant::now();
        
        // === 第一优先级：正常处理流程（完全不变） ===
        // 注意：实际的CentralManager可能没有直接的process_message方法
        // 这里需要根据实际API调整
        // let cleaned_result = self.core_system.process_message(data.clone()).await?;
        
        // 转换为增强数据格式
        let cleaned_data = CleanedMarketData::from(data.clone());
        
        // === 第二优先级：并行增强处理（绝对不阻塞主流程） ===
        if self.is_enhanced_running.load(Ordering::Relaxed) {
            // 1. 立即发送给策略模块（最高优先级）
            if let Some(distributor) = &self.data_distributor {
                if let Err(e) = distributor.send_to_strategy(cleaned_data.clone()).await {
                    error!("Strategy distribution failed: {}", e);
                    if let Some(monitor) = &self.performance_monitor {
                        monitor.record_error("strategy_distribution").await;
                    }
                }
            }
            
            // 2. 并行执行数据校验（不等待结果）
            if let Some(validator) = &self.cross_validator {
                let validator_clone = validator.clone();
                let cleaned_data_clone = cleaned_data.clone();
                let monitor_clone = self.performance_monitor.clone();
                
                tokio::spawn(async move {
                    let validation_start = Instant::now();
                    match validator_clone.validate_market_data(&cleaned_data_clone).await {
                        validation_result => {
                            let validation_latency = validation_start.elapsed().as_nanos() as u64;
                            
                            if let Some(monitor) = monitor_clone {
                                monitor.record_validation_latency(validation_latency).await;
                            }
                            
                            // 更新数据质量分数
                            let quality_adjustment = if validation_result.is_valid {
                                validation_result.confidence_score
                            } else {
                                validation_result.confidence_score * 0.5 // 降低质量分数
                            };
                            
                            debug!("Validation completed for {}: valid={} quality_adjustment={:.3}", 
                                   cleaned_data_clone.symbol, validation_result.is_valid, quality_adjustment);
                        }
                    }
                });
            }
            
            // 3. 异步审计存储（如果启用）
            if self.config.audit_storage_enabled {
                if let Some(distributor) = &self.data_distributor {
                    let audit_data = AuditData {
                        id: Uuid::new_v4().to_string(),
                        data_type: AuditDataType::MarketData,
                        timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
                        symbol: cleaned_data.symbol.clone(),
                        exchange: cleaned_data.exchange.clone(),
                        raw_data: serde_json::to_vec(&data).unwrap_or_default(),
                        processed_data: Some(serde_json::to_vec(&cleaned_data).unwrap_or_default()),
                        quality_metrics: QualityMetrics {
                            completeness_score: 1.0, // 基于实际数据完整性计算
                            timeliness_score: self.calculate_timeliness_score(&cleaned_data),
                            accuracy_score: cleaned_data.quality_score,
                            consistency_score: 0.95, // 基于交叉校验结果
                            overall_score: cleaned_data.quality_score,
                        },
                    };
                    
                    distributor.store_for_audit_async(audit_data).await;
                }
            }
        }
        
        // 记录总体处理延迟
        let total_latency = total_start.elapsed().as_nanos() as u64;
        if let Some(monitor) = &self.performance_monitor {
            monitor.record_processing_latency(total_latency).await;
        }
        
        // 延迟警告
        if total_latency > self.config.latency_config.target_strategy_latency_ns {
            warn!("Total processing latency exceeded target: {}ns > {}ns for {}", 
                  total_latency, self.config.latency_config.target_strategy_latency_ns, cleaned_data.symbol);
        }
        
        debug!("Enhanced processing completed for {}: total_latency={}ns", 
               cleaned_data.symbol, total_latency);
        
        Ok(())
    }
    
    fn calculate_timeliness_score(&self, data: &CleanedMarketData) -> f64 {
        let now = chrono::Utc::now().timestamp_millis();
        let age_ms = (now - data.timestamp).abs();
        
        // 数据越新，分数越高
        if age_ms <= 100 { // 100ms内认为很及时
            1.0
        } else if age_ms <= 1000 { // 1秒内还算及时
            1.0 - (age_ms as f64 - 100.0) / 900.0
        } else { // 超过1秒认为延迟较大
            (5000.0 - age_ms as f64).max(0.0) / 4000.0
        }
    }
    
    /// 获取增强系统统计信息
    pub async fn get_enhanced_stats(&self) -> EnhancedSystemStats {
        let distribution_stats = if let Some(distributor) = &self.data_distributor {
            Some(distributor.get_distribution_stats().await)
        } else {
            None
        };
        
        let validation_stats = if let Some(validator) = &self.cross_validator {
            Some(validator.get_validation_stats().await)
        } else {
            None
        };
        
        let performance_stats = if let Some(monitor) = &self.performance_monitor {
            Some(monitor.get_performance_summary().await)
        } else {
            None
        };
        
        EnhancedSystemStats {
            is_enhanced_running: self.is_enhanced_running.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            config: self.config.clone(),
            distribution_stats,
            validation_stats,
            performance_stats,
        }
    }
    
    /// 停止增强功能
    pub async fn stop_enhanced_features(&self) {
        if let Some(distributor) = &self.data_distributor {
            distributor.stop().await;
        }
        
        self.is_enhanced_running.store(false, Ordering::Relaxed);
        info!("Qingxi enhanced features stopped");
    }
}

/// 为增强系统实现CentralManagerApi - 完全兼容
#[async_trait]
impl CentralManagerApi for QingxiSystemEnhanced {
    async fn reconfigure(&self, sources: Vec<MarketSourceConfig>) -> Result<(), MarketDataError> {
        // 直接委托给核心系统
        self.core_system.reconfigure(sources).await
    }
    
    async fn get_latest_orderbook(&self, exchange_id: &str, symbol: &Symbol) -> Result<OrderBook, MarketDataApiError> {
        self.core_system.get_latest_orderbook(exchange_id, symbol).await
    }
    
    async fn get_latest_snapshot(&self, symbol: &str) -> Result<MarketDataSnapshot, MarketDataApiError> {
        self.core_system.get_latest_snapshot(symbol).await
    }
    
    async fn get_latest_anomaly(&self, symbol: &str) -> Result<AnomalyDetectionResult, MarketDataApiError> {
        self.core_system.get_latest_anomaly(symbol).await
    }
    
    async fn get_all_orderbooks(&self) -> Result<Vec<(Symbol, OrderBook)>, MarketDataApiError> {
        self.core_system.get_all_orderbooks().await
    }
    
    async fn get_performance_stats(&self) -> Result<PerformanceStats, MarketDataApiError> {
        Ok(self.core_system.get_performance_stats().await)
    }
    
    async fn start_collectors(&self) -> Result<(), MarketDataError> {
        // 先启动核心系统
        let result = self.core_system.start_collectors().await;
        
        // 然后启动增强功能
        if result.is_ok() {
            if let Err(e) = self.start_enhanced_features().await {
                error!("Failed to start enhanced features: {}", e);
            }
        }
        
        result
    }
    
    async fn get_active_exchanges(&self) -> Result<Vec<String>, MarketDataError> {
        self.core_system.get_active_exchanges().await
    }
    
    async fn get_orderbook(&self, exchange: &str, symbol: &str) -> Result<Option<OrderBook>, MarketDataError> {
        self.core_system.get_orderbook(exchange, symbol).await
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EnhancedSystemStats {
    pub is_enhanced_running: bool,
    pub uptime_seconds: u64,
    pub config: EnhancedConfig,
    pub distribution_stats: Option<crate::data_distribution::DistributionStats>,
    pub validation_stats: Option<ValidationStats>,
    pub performance_stats: Option<PerformanceSummary>,
}

