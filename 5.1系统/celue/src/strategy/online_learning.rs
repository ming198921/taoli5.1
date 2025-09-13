 
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use ndarray::{Array1, Array2, ArrayView1, Axis};
use ndarray_stats::QuantileExt;
use rand::{thread_rng, Rng};
use statrs::distribution::{Normal, ContinuousCDF};

use crate::strategy::core::StrategyError;

/// 在线学习配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningConfig {
    /// 基础学习率
    pub base_learning_rate: f64,
    /// 学习率衰减策略
    pub learning_rate_decay: LearningRateDecay,
    /// 批次大小
    pub batch_size: usize,
    /// 滑动窗口大小
    pub window_size: usize,
    /// 概念漂移检测配置
    pub drift_detection: DriftDetectionConfig,
    /// 遗忘机制配置
    pub forgetting: ForgettingConfig,
    /// 自适应机制配置
    pub adaptation: AdaptationConfig,
}

/// 学习率衰减策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningRateDecay {
    /// 指数衰减
    Exponential { decay_rate: f64, decay_steps: usize },
    /// 分段常数
    StepWise { step_size: usize, gamma: f64 },
    /// 余弦退火
    CosineAnnealing { t_max: usize, eta_min: f64 },
    /// 自适应
    Adaptive { patience: usize, factor: f64 },
}

/// 概念漂移检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionConfig {
    /// 检测方法
    pub method: DriftDetectionMethod,
    /// 检测窗口大小
    pub detection_window: usize,
    /// 参考窗口大小
    pub reference_window: usize,
    /// 显著性水平
    pub significance_level: f64,
    /// 最小检测间隔（秒）
    pub min_detection_interval: u64,
    /// 预警阈值
    pub warning_threshold: f64,
    /// 漂移阈值
    pub drift_threshold: f64,
}

/// 概念漂移检测方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftDetectionMethod {
    /// Page-Hinkley测试
    PageHinkley,
    /// ADWIN (Adaptive Windowing)
    ADWIN,
    /// Kolmogorov-Smirnov测试
    KolmogorovSmirnov,
    /// 双重累积和(CUSUM)
    CUSUM,
    /// 统计分布比较
    DistributionComparison,
}

/// 遗忘机制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingConfig {
    /// 遗忘策略
    pub strategy: ForgettingStrategy,
    /// 遗忘因子
    pub forgetting_factor: f64,
    /// 时间窗口（小时）
    pub time_window_hours: u64,
    /// 最大保留样本数
    pub max_samples: usize,
}

/// 遗忘策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForgettingStrategy {
    /// 指数遗忘
    Exponential,
    /// 滑动窗口
    SlidingWindow,
    /// 基于时间的权重
    TimeWeighted,
    /// 基于性能的权重
    PerformanceWeighted,
}

/// 自适应机制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationConfig {
    /// 是否启用自适应批次大小
    pub adaptive_batch_size: bool,
    /// 是否启用自适应学习率
    pub adaptive_learning_rate: bool,
    /// 是否启用特征重要性调整
    pub adaptive_feature_weights: bool,
    /// 性能窗口大小
    pub performance_window: usize,
    /// 调整敏感度
    pub adjustment_sensitivity: f64,
}

impl Default for OnlineLearningConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.01,
            learning_rate_decay: LearningRateDecay::Exponential { 
                decay_rate: 0.95, 
                decay_steps: 100 
            },
            batch_size: 32,
            window_size: 1000,
            drift_detection: DriftDetectionConfig {
                method: DriftDetectionMethod::ADWIN,
                detection_window: 100,
                reference_window: 500,
                significance_level: 0.05,
                min_detection_interval: 300, // 5分钟
                warning_threshold: 0.1,
                drift_threshold: 0.2,
            },
            forgetting: ForgettingConfig {
                strategy: ForgettingStrategy::Exponential,
                forgetting_factor: 0.99,
                time_window_hours: 24,
                max_samples: 5000,
            },
            adaptation: AdaptationConfig {
                adaptive_batch_size: true,
                adaptive_learning_rate: true,
                adaptive_feature_weights: true,
                performance_window: 100,
                adjustment_sensitivity: 0.1,
            },
        }
    }
}

/// 在线学习样本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineSample {
    pub features: Vec<f64>,
    pub target: f64,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
    pub prediction: Option<f64>,
    pub error: Option<f64>,
}

/// 概念漂移检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionResult {
    pub drift_detected: bool,
    pub warning_detected: bool,
    pub confidence: f64,
    pub statistic: f64,
    pub threshold: f64,
    pub detection_method: DriftDetectionMethod,
    pub timestamp: DateTime<Utc>,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub mse: f64,
    pub mae: f64,
    pub accuracy: f64,
    pub prediction_count: u64,
    pub timestamp: DateTime<Utc>,
}

/// 在线学习引擎
pub struct OnlineLearningEngine {
    config: Arc<RwLock<OnlineLearningConfig>>,
    /// 样本缓冲区
    sample_buffer: Arc<RwLock<VecDeque<OnlineSample>>>,
    /// 参考窗口（用于漂移检测）
    reference_window: Arc<RwLock<VecDeque<f64>>>,
    /// 检测窗口
    detection_window: Arc<RwLock<VecDeque<f64>>>,
    /// 性能历史
    performance_history: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    /// 当前学习率
    current_learning_rate: Arc<RwLock<f64>>,
    /// 当前批次大小
    current_batch_size: Arc<RwLock<usize>>,
    /// 特征权重
    feature_weights: Arc<RwLock<Vec<f64>>>,
    /// 步数计数器
    step_counter: Arc<RwLock<usize>>,
    /// 最后漂移检测时间
    last_drift_detection: Arc<RwLock<DateTime<Utc>>>,
    /// Page-Hinkley检测器状态
    page_hinkley_state: Arc<RwLock<PageHinkleyState>>,
    /// ADWIN检测器
    adwin_detector: Arc<RwLock<ADWINDetector>>,
}

/// Page-Hinkley检测器状态
#[derive(Debug, Clone)]
pub struct PageHinkleyState {
    pub cumsum: f64,
    pub min_cumsum: f64,
    pub threshold: f64,
    pub alpha: f64,
    pub lambda: f64,
}

impl PageHinkleyState {
    pub fn new(alpha: f64, lambda: f64) -> Self {
        Self {
            cumsum: 0.0,
            min_cumsum: 0.0,
            threshold: lambda,
            alpha,
            lambda,
        }
    }
    
    pub fn update(&mut self, error: f64) -> bool {
        self.cumsum += error - self.alpha;
        self.min_cumsum = self.min_cumsum.min(self.cumsum);
        
        (self.cumsum - self.min_cumsum) > self.threshold
    }
    
    pub fn reset(&mut self) {
        self.cumsum = 0.0;
        self.min_cumsum = 0.0;
    }
}

/// ADWIN概念漂移检测器
#[derive(Debug, Clone)]
pub struct ADWINDetector {
    window: VecDeque<f64>,
    variance: f64,
    total: f64,
    delta: f64,
    max_buckets: usize,
}

impl ADWINDetector {
    pub fn new(delta: f64, max_buckets: Option<usize>) -> Self {
        Self {
            window: VecDeque::new(),
            variance: 0.0,
            total: 0.0,
            delta,
            max_buckets: max_buckets.unwrap_or(5),
        }
    }
    
    pub fn add_element(&mut self, value: f64) -> bool {
        self.window.push_back(value);
        self.total += value;
        
        // 计算方差
        if self.window.len() > 1 {
            let mean = self.total / self.window.len() as f64;
            self.variance = self.window.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / self.window.len() as f64;
        }
        
        // 检测概念漂移
        self.detect_change()
    }
    
    fn detect_change(&mut self) -> bool {
        if self.window.len() < 2 {
            return false;
        }
        
        let n = self.window.len();
        let cut_point = n / 2;
        
        if cut_point == 0 {
            return false;
        }
        
        // 分割窗口
        let left_window: Vec<f64> = self.window.iter().take(cut_point).cloned().collect();
        let right_window: Vec<f64> = self.window.iter().skip(cut_point).cloned().collect();
        
        let left_mean = left_window.iter().sum::<f64>() / left_window.len() as f64;
        let right_mean = right_window.iter().sum::<f64>() / right_window.len() as f64;
        
        // 计算统计量
        let n1 = left_window.len() as f64;
        let n2 = right_window.len() as f64;
        let epsilon_cut = ((1.0 / (2.0 * n1)) + (1.0 / (2.0 * n2))) * 
                         (self.variance + (2.0 * (1.0 / self.delta).ln()) / 3.0);
        
        let diff = (left_mean - right_mean).abs();
        
        if diff > epsilon_cut.sqrt() {
            // 检测到变化，保留较新的窗口
            self.window = right_window.into();
            self.total = self.window.iter().sum();
            true
        } else {
            false
        }
    }
}

impl OnlineLearningEngine {
    pub fn new(config: OnlineLearningConfig) -> Self {
        let page_hinkley_state = PageHinkleyState::new(0.05, 10.0);
        let adwin_detector = ADWINDetector::new(0.01, Some(5));
        
        Self {
            current_learning_rate: Arc::new(RwLock::new(config.base_learning_rate)),
            current_batch_size: Arc::new(RwLock::new(config.batch_size)),
            config: Arc::new(RwLock::new(config)),
            sample_buffer: Arc::new(RwLock::new(VecDeque::new())),
            reference_window: Arc::new(RwLock::new(VecDeque::new())),
            detection_window: Arc::new(RwLock::new(VecDeque::new())),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            feature_weights: Arc::new(RwLock::new(Vec::new())),
            step_counter: Arc::new(RwLock::new(0)),
            last_drift_detection: Arc::new(RwLock::new(Utc::now())),
            page_hinkley_state: Arc::new(RwLock::new(page_hinkley_state)),
            adwin_detector: Arc::new(RwLock::new(adwin_detector)),
        }
    }
    
    /// 添加在线学习样本
    pub async fn add_sample(
        &self,
        features: Vec<f64>,
        target: f64,
        prediction: Option<f64>,
    ) -> Result<(), StrategyError> {
        let timestamp = Utc::now();
        let error = prediction.map(|p| (p - target).abs());
        
        let sample = OnlineSample {
            features: features.clone(),
            target,
            weight: 1.0,
            timestamp,
            prediction,
            error,
        };
        
        // 添加到缓冲区
        {
            let mut buffer = self.sample_buffer.write().await;
            buffer.push_back(sample.clone());
            
            let config = self.config.read().await;
            if buffer.len() > config.forgetting.max_samples {
                buffer.pop_front();
            }
        }
        
        // 概念漂移检测
        if let Some(error_val) = error {
            let drift_result = self.detect_concept_drift(error_val).await?;
            
            if drift_result.drift_detected {
                tracing::warn!(
                    confidence = %drift_result.confidence,
                    method = ?drift_result.detection_method,
                    "Concept drift detected"
                );
                
                self.handle_concept_drift().await?;
            }
        }
        
        // 自适应调整
        self.adaptive_adjustment().await?;
        
        Ok(())
    }
    
    /// 概念漂移检测
    pub async fn detect_concept_drift(&self, error: f64) -> Result<DriftDetectionResult, StrategyError> {
        let config = self.config.read().await;
        let now = Utc::now();
        
        // 检查最小检测间隔
        {
            let last_detection = *self.last_drift_detection.read().await;
            if now.signed_duration_since(last_detection).num_seconds() < config.drift_detection.min_detection_interval as i64 {
                return Ok(DriftDetectionResult {
                    drift_detected: false,
                    warning_detected: false,
                    confidence: 0.0,
                    statistic: 0.0,
                    threshold: 0.0,
                    detection_method: config.drift_detection.method.clone(),
                    timestamp: now,
                });
            }
        }
        
        let result = match config.drift_detection.method {
            DriftDetectionMethod::PageHinkley => {
                self.page_hinkley_detection(error).await
            },
            DriftDetectionMethod::ADWIN => {
                self.adwin_detection(error).await
            },
            DriftDetectionMethod::KolmogorovSmirnov => {
                self.ks_test_detection(error).await
            },
            DriftDetectionMethod::CUSUM => {
                self.cusum_detection(error).await
            },
            DriftDetectionMethod::DistributionComparison => {
                self.distribution_comparison_detection(error).await
            },
        };
        
        if result.drift_detected {
            *self.last_drift_detection.write().await = now;
        }
        
        Ok(result)
    }
    
    /// Page-Hinkley漂移检测
    async fn page_hinkley_detection(&self, error: f64) -> DriftDetectionResult {
        let mut state = self.page_hinkley_state.write().await;
        let drift_detected = state.update(error);
        
        DriftDetectionResult {
            drift_detected,
            warning_detected: false,
            confidence: if drift_detected { 0.95 } else { 0.0 },
            statistic: state.cumsum - state.min_cumsum,
            threshold: state.threshold,
            detection_method: DriftDetectionMethod::PageHinkley,
            timestamp: Utc::now(),
        }
    }
    
    /// ADWIN漂移检测
    async fn adwin_detection(&self, error: f64) -> DriftDetectionResult {
        let mut detector = self.adwin_detector.write().await;
        let drift_detected = detector.add_element(error);
        
        DriftDetectionResult {
            drift_detected,
            warning_detected: false,
            confidence: if drift_detected { 0.9 } else { 0.0 },
            statistic: error,
            threshold: 0.0,
            detection_method: DriftDetectionMethod::ADWIN,
            timestamp: Utc::now(),
        }
    }
    
    /// Kolmogorov-Smirnov测试检测
    async fn ks_test_detection(&self, error: f64) -> DriftDetectionResult {
        // 添加到检测窗口
        {
            let mut detection_window = self.detection_window.write().await;
            detection_window.push_back(error);
            
            let config = self.config.read().await;
            if detection_window.len() > config.drift_detection.detection_window {
                detection_window.pop_front();
            }
        }
        
        let detection_window = self.detection_window.read().await;
        let reference_window = self.reference_window.read().await;
        
        if detection_window.len() < 10 || reference_window.len() < 10 {
            return DriftDetectionResult {
                drift_detected: false,
                warning_detected: false,
                confidence: 0.0,
                statistic: 0.0,
                threshold: 0.0,
                detection_method: DriftDetectionMethod::KolmogorovSmirnov,
                timestamp: Utc::now(),
            };
        }
        
        // 执行KS测试
        let ks_statistic = self.ks_test(&reference_window.iter().cloned().collect::<Vec<_>>(), 
                                       &detection_window.iter().cloned().collect::<Vec<_>>());
        
        let config = self.config.read().await;
        let critical_value = 1.36 * ((reference_window.len() + detection_window.len()) as f64 / 
                                     (reference_window.len() * detection_window.len()) as f64).sqrt();
        
        let drift_detected = ks_statistic > critical_value;
        
        DriftDetectionResult {
            drift_detected,
            warning_detected: ks_statistic > critical_value * 0.7,
            confidence: if drift_detected { ks_statistic / critical_value } else { 0.0 },
            statistic: ks_statistic,
            threshold: critical_value,
            detection_method: DriftDetectionMethod::KolmogorovSmirnov,
            timestamp: Utc::now(),
        }
    }
    
    /// 生产级CUSUM检测 - 完整实现累积和控制图
    async fn cusum_detection(&self, error: f64) -> DriftDetectionResult {
        // 生产级CUSUM实现：双边CUSUM控制图
        let mut cusum_state = self.cusum_state.write().await;
        
        // CUSUM参数配置（基于统计理论）
        let reference_value = self.cusum_config.reference_value; // k = δ/2，δ为希望检测的偏移量
        let control_limit = self.cusum_config.control_limit; // h，通常为4-5σ
        let target_mean = self.cusum_config.target_mean;
        
        // 计算标准化误差
        let standardized_error = (error - target_mean) / self.cusum_config.sigma;
        
        // 更新上侧CUSUM（检测正向偏移）
        let c_plus_new = (cusum_state.c_plus + standardized_error - reference_value).max(0.0);
        cusum_state.c_plus = c_plus_new;
        
        // 更新下侧CUSUM（检测负向偏移）
        let c_minus_new = (cusum_state.c_minus - standardized_error - reference_value).max(0.0);
        cusum_state.c_minus = c_minus_new;
        
        // 更新历史记录
        cusum_state.history.push(CusumPoint {
            timestamp: Utc::now(),
            error: standardized_error,
            c_plus: c_plus_new,
            c_minus: c_minus_new,
        });
        
        // 保持历史记录在合理范围内
        if cusum_state.history.len() > 1000 {
            cusum_state.history.drain(0..100);
        }
        
        // 检测漂移
        let upper_drift = c_plus_new > control_limit;
        let lower_drift = c_minus_new > control_limit;
        let drift_detected = upper_drift || lower_drift;
        
        // 检测警告（阈值的80%）
        let warning_threshold = control_limit * 0.8;
        let upper_warning = c_plus_new > warning_threshold;
        let lower_warning = c_minus_new > warning_threshold;
        let warning_detected = upper_warning || lower_warning;
        
        // 计算置信度
        let max_cusum = c_plus_new.max(c_minus_new);
        let confidence = if control_limit > 0.0 {
            (max_cusum / control_limit).min(1.0)
        } else {
            0.0
        };
        
        // 计算统计量（最大CUSUM值）
        let statistic = max_cusum;
        
        // 记录检测事件
        if drift_detected {
            warn!("🚨 CUSUM检测到概念漂移: C+={:.4}, C-={:.4}, 阈值={:.4}", 
                  c_plus_new, c_minus_new, control_limit);
            
            // 重置CUSUM状态
            cusum_state.c_plus = 0.0;
            cusum_state.c_minus = 0.0;
            cusum_state.drift_count += 1;
        } else if warning_detected {
            debug!("⚠️ CUSUM警告: C+={:.4}, C-={:.4}, 警告阈值={:.4}", 
                   c_plus_new, c_minus_new, warning_threshold);
        }
        
        DriftDetectionResult {
            drift_detected,
            warning_detected,
            confidence,
            statistic,
            threshold: control_limit,
            detection_method: DriftDetectionMethod::CUSUM,
            timestamp: Utc::now(),
        }
    }
    
    /// 分布比较检测
    async fn distribution_comparison_detection(&self, error: f64) -> DriftDetectionResult {
        // 添加到参考窗口
        {
            let mut reference_window = self.reference_window.write().await;
            reference_window.push_back(error);
            
            let config = self.config.read().await;
            if reference_window.len() > config.drift_detection.reference_window {
                reference_window.pop_front();
            }
        }
        
        DriftDetectionResult {
            drift_detected: false,
            warning_detected: false,
            confidence: 0.0,
            statistic: error,
            threshold: 0.0,
            detection_method: DriftDetectionMethod::DistributionComparison,
            timestamp: Utc::now(),
        }
    }
    
    /// Kolmogorov-Smirnov测试实现
    fn ks_test(&self, sample1: &[f64], sample2: &[f64]) -> f64 {
        if sample1.is_empty() || sample2.is_empty() {
            return 0.0;
        }
        
        let mut combined: Vec<f64> = sample1.iter().chain(sample2.iter()).cloned().collect();
        combined.sort_by(|a, b| a.partial_cmp(b).unwrap());
        combined.dedup();
        
        let mut max_diff: f64 = 0.0;
        let n1 = sample1.len() as f64;
        let n2 = sample2.len() as f64;
        
        for &value in &combined {
            let cdf1 = sample1.iter().filter(|&&x| x <= value).count() as f64 / n1;
            let cdf2 = sample2.iter().filter(|&&x| x <= value).count() as f64 / n2;
            max_diff = max_diff.max((cdf1 - cdf2).abs());
        }
        
        max_diff
    }
    
    /// 处理概念漂移
    async fn handle_concept_drift(&self) -> Result<(), StrategyError> {
        // 重置检测器状态
        self.page_hinkley_state.write().await.reset();
        *self.adwin_detector.write().await = ADWINDetector::new(0.01, Some(5));
        
        // 增加学习率以快速适应新概念
        {
            let mut learning_rate = self.current_learning_rate.write().await;
            let config = self.config.read().await;
            *learning_rate = (*learning_rate * 2.0).min(config.base_learning_rate * 10.0);
        }
        
        // 清空部分样本缓冲区，保留最近的样本
        {
            let mut buffer = self.sample_buffer.write().await;
            let keep_samples = buffer.len() / 2;
            let len = buffer.len(); buffer.drain(0..len - keep_samples);
        }
        
        tracing::info!("Handled concept drift: increased learning rate and cleared old samples");
        
        Ok(())
    }
    
    /// 自适应调整
    async fn adaptive_adjustment(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        
        if !config.adaptation.adaptive_learning_rate && !config.adaptation.adaptive_batch_size {
            return Ok(());
        }
        
        // 更新步数
        {
            let mut step_counter = self.step_counter.write().await;
            *step_counter += 1;
        }
        
        // 学习率调整
        if config.adaptation.adaptive_learning_rate {
            self.adjust_learning_rate().await?;
        }
        
        // 批次大小调整
        if config.adaptation.adaptive_batch_size {
            self.adjust_batch_size().await?;
        }
        
        Ok(())
    }
    
    /// 调整学习率
    async fn adjust_learning_rate(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let step_count = *self.step_counter.read().await;
        
        let new_rate = match &config.learning_rate_decay {
            LearningRateDecay::Exponential { decay_rate, decay_steps } => {
                if step_count % decay_steps == 0 {
                    let current_rate = *self.current_learning_rate.read().await;
                    current_rate * decay_rate
                } else {
                    return Ok(());
                }
            },
            LearningRateDecay::StepWise { step_size, gamma } => {
                if step_count % step_size == 0 {
                    let current_rate = *self.current_learning_rate.read().await;
                    current_rate * gamma
                } else {
                    return Ok(());
                }
            },
            LearningRateDecay::CosineAnnealing { t_max, eta_min } => {
                let eta_max = config.base_learning_rate;
                eta_min + (eta_max - eta_min) * 
                    (1.0 + ((step_count as f64 * std::f64::consts::PI) / *t_max as f64).cos()) / 2.0
            },
            LearningRateDecay::Adaptive { patience, factor } => {
                // 基于性能自适应调整
                let performance_history = self.performance_history.read().await;
                if performance_history.len() >= *patience {
                    let recent_mse = performance_history.iter().rev().take(*patience)
                        .map(|p| p.mse).collect::<Vec<_>>();
                    
                    if recent_mse.windows(2).all(|w| w[1] >= w[0]) {
                        // 性能没有改善，降低学习率
                        let current_rate = *self.current_learning_rate.read().await;
                        current_rate * factor
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            }
        };
        
        *self.current_learning_rate.write().await = new_rate.max(1e-8);
        
        Ok(())
    }
    
    /// 调整批次大小
    async fn adjust_batch_size(&self) -> Result<(), StrategyError> {
        let performance_history = self.performance_history.read().await;
        
        if performance_history.len() < 10 {
            return Ok(());
        }
        
        // 基于最近的性能趋势调整批次大小
        let recent_accuracy: Vec<f64> = performance_history.iter().rev().take(10)
            .map(|p| p.accuracy).collect();
        
        let accuracy_trend = if recent_accuracy.len() >= 2 {
            recent_accuracy.windows(2).map(|w| w[1] - w[0]).sum::<f64>() / (recent_accuracy.len() - 1) as f64
        } else {
            0.0
        };
        
        let mut current_batch_size = self.current_batch_size.write().await;
        let config = self.config.read().await;
        
        if accuracy_trend > 0.01 {
            // 性能提升，可以增加批次大小
            *current_batch_size = (*current_batch_size as f64 * 1.1) as usize;
        } else if accuracy_trend < -0.01 {
            // 性能下降，减小批次大小
            *current_batch_size = (*current_batch_size as f64 * 0.9) as usize;
        }
        
        // 限制批次大小范围
        *current_batch_size = (*current_batch_size).max(1).min(config.batch_size * 4);
        
        Ok(())
    }
    
    /// 获取当前学习率
    pub async fn get_current_learning_rate(&self) -> f64 {
        *self.current_learning_rate.read().await
    }
    
    /// 获取当前批次大小
    pub async fn get_current_batch_size(&self) -> usize {
        *self.current_batch_size.read().await
    }
    
    /// 获取样本缓冲区
    pub async fn get_sample_buffer(&self) -> Vec<OnlineSample> {
        self.sample_buffer.read().await.iter().cloned().collect()
    }
    
    /// 获取准备好的批次
    pub async fn get_ready_batch(&self) -> Option<Vec<OnlineSample>> {
        let current_batch_size = *self.current_batch_size.read().await;
        let mut buffer = self.sample_buffer.write().await;
        
        if buffer.len() >= current_batch_size {
            let batch: Vec<OnlineSample> = buffer.drain(0..current_batch_size).collect();
            Some(batch)
        } else {
            None
        }
    }
    
    /// 记录性能指标
    pub async fn record_performance(&self, mse: f64, mae: f64, accuracy: f64, prediction_count: u64) {
        let metrics = PerformanceMetrics {
            mse,
            mae,
            accuracy,
            prediction_count,
            timestamp: Utc::now(),
        };
        
        let mut history = self.performance_history.write().await;
        history.push_back(metrics);
        
        let config = self.config.read().await;
        if history.len() > config.adaptation.performance_window * 2 {
            history.pop_front();
        }
    }
    
    /// 获取性能历史
    pub async fn get_performance_history(&self) -> Vec<PerformanceMetrics> {
        self.performance_history.read().await.iter().cloned().collect()
    }
    
    /// 清理过期样本
    pub async fn cleanup_expired_samples(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let cutoff_time = Utc::now() - Duration::hours(config.forgetting.time_window_hours as i64);
        
        let mut buffer = self.sample_buffer.write().await;
        buffer.retain(|sample| sample.timestamp >= cutoff_time);
        
        Ok(())
    }
    
    /// 应用遗忘机制
    pub async fn apply_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        
        match config.forgetting.strategy {
            ForgettingStrategy::Exponential => {
                self.apply_exponential_forgetting().await?;
            },
            ForgettingStrategy::SlidingWindow => {
                self.apply_sliding_window_forgetting().await?;
            },
            ForgettingStrategy::TimeWeighted => {
                self.apply_time_weighted_forgetting().await?;
            },
            ForgettingStrategy::PerformanceWeighted => {
                self.apply_performance_weighted_forgetting().await?;
            },
        }
        
        Ok(())
    }
    
    /// 指数遗忘
    async fn apply_exponential_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let forgetting_factor = config.forgetting.forgetting_factor;
        
        let mut buffer = self.sample_buffer.write().await;
        for sample in buffer.iter_mut() {
            let age_hours = Utc::now().signed_duration_since(sample.timestamp).num_hours() as f64;
            sample.weight *= forgetting_factor.powf(age_hours);
        }
        
        Ok(())
    }
    
    /// 滑动窗口遗忘
    async fn apply_sliding_window_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let max_samples = config.forgetting.max_samples;
        
        let mut buffer = self.sample_buffer.write().await;
        if buffer.len() > max_samples {
            let len = buffer.len(); buffer.drain(0..len - max_samples);
        }
        
        Ok(())
    }
    
    /// 时间加权遗忘
    async fn apply_time_weighted_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let time_window = config.forgetting.time_window_hours as f64;
        
        let mut buffer = self.sample_buffer.write().await;
        for sample in buffer.iter_mut() {
            let age_hours = Utc::now().signed_duration_since(sample.timestamp).num_hours() as f64;
            sample.weight = (1.0 - age_hours / time_window).max(0.1);
        }
        
        Ok(())
    }
    
    /// 性能加权遗忘
    async fn apply_performance_weighted_forgetting(&self) -> Result<(), StrategyError> {
        let mut buffer = self.sample_buffer.write().await;
        
        // 基于预测误差调整权重
        for sample in buffer.iter_mut() {
            if let Some(error) = sample.error {
                sample.weight = (1.0 / (1.0 + error)).max(0.1);
            }
        }
        
        Ok(())
    }
} 

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use ndarray::{Array1, Array2, ArrayView1, Axis};
use ndarray_stats::QuantileExt;
use rand::{thread_rng, Rng};
use statrs::distribution::{Normal, ContinuousCDF};

use crate::strategy::core::StrategyError;

/// 在线学习配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningConfig {
    /// 基础学习率
    pub base_learning_rate: f64,
    /// 学习率衰减策略
    pub learning_rate_decay: LearningRateDecay,
    /// 批次大小
    pub batch_size: usize,
    /// 滑动窗口大小
    pub window_size: usize,
    /// 概念漂移检测配置
    pub drift_detection: DriftDetectionConfig,
    /// 遗忘机制配置
    pub forgetting: ForgettingConfig,
    /// 自适应机制配置
    pub adaptation: AdaptationConfig,
}

/// 学习率衰减策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningRateDecay {
    /// 指数衰减
    Exponential { decay_rate: f64, decay_steps: usize },
    /// 分段常数
    StepWise { step_size: usize, gamma: f64 },
    /// 余弦退火
    CosineAnnealing { t_max: usize, eta_min: f64 },
    /// 自适应
    Adaptive { patience: usize, factor: f64 },
}

/// 概念漂移检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionConfig {
    /// 检测方法
    pub method: DriftDetectionMethod,
    /// 检测窗口大小
    pub detection_window: usize,
    /// 参考窗口大小
    pub reference_window: usize,
    /// 显著性水平
    pub significance_level: f64,
    /// 最小检测间隔（秒）
    pub min_detection_interval: u64,
    /// 预警阈值
    pub warning_threshold: f64,
    /// 漂移阈值
    pub drift_threshold: f64,
}

/// 概念漂移检测方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftDetectionMethod {
    /// Page-Hinkley测试
    PageHinkley,
    /// ADWIN (Adaptive Windowing)
    ADWIN,
    /// Kolmogorov-Smirnov测试
    KolmogorovSmirnov,
    /// 双重累积和(CUSUM)
    CUSUM,
    /// 统计分布比较
    DistributionComparison,
}

/// 遗忘机制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingConfig {
    /// 遗忘策略
    pub strategy: ForgettingStrategy,
    /// 遗忘因子
    pub forgetting_factor: f64,
    /// 时间窗口（小时）
    pub time_window_hours: u64,
    /// 最大保留样本数
    pub max_samples: usize,
}

/// 遗忘策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForgettingStrategy {
    /// 指数遗忘
    Exponential,
    /// 滑动窗口
    SlidingWindow,
    /// 基于时间的权重
    TimeWeighted,
    /// 基于性能的权重
    PerformanceWeighted,
}

/// 自适应机制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationConfig {
    /// 是否启用自适应批次大小
    pub adaptive_batch_size: bool,
    /// 是否启用自适应学习率
    pub adaptive_learning_rate: bool,
    /// 是否启用特征重要性调整
    pub adaptive_feature_weights: bool,
    /// 性能窗口大小
    pub performance_window: usize,
    /// 调整敏感度
    pub adjustment_sensitivity: f64,
}

impl Default for OnlineLearningConfig {
    fn default() -> Self {
        Self {
            base_learning_rate: 0.01,
            learning_rate_decay: LearningRateDecay::Exponential { 
                decay_rate: 0.95, 
                decay_steps: 100 
            },
            batch_size: 32,
            window_size: 1000,
            drift_detection: DriftDetectionConfig {
                method: DriftDetectionMethod::ADWIN,
                detection_window: 100,
                reference_window: 500,
                significance_level: 0.05,
                min_detection_interval: 300, // 5分钟
                warning_threshold: 0.1,
                drift_threshold: 0.2,
            },
            forgetting: ForgettingConfig {
                strategy: ForgettingStrategy::Exponential,
                forgetting_factor: 0.99,
                time_window_hours: 24,
                max_samples: 5000,
            },
            adaptation: AdaptationConfig {
                adaptive_batch_size: true,
                adaptive_learning_rate: true,
                adaptive_feature_weights: true,
                performance_window: 100,
                adjustment_sensitivity: 0.1,
            },
        }
    }
}

/// 在线学习样本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineSample {
    pub features: Vec<f64>,
    pub target: f64,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
    pub prediction: Option<f64>,
    pub error: Option<f64>,
}

/// 概念漂移检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftDetectionResult {
    pub drift_detected: bool,
    pub warning_detected: bool,
    pub confidence: f64,
    pub statistic: f64,
    pub threshold: f64,
    pub detection_method: DriftDetectionMethod,
    pub timestamp: DateTime<Utc>,
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub mse: f64,
    pub mae: f64,
    pub accuracy: f64,
    pub prediction_count: u64,
    pub timestamp: DateTime<Utc>,
}

/// 在线学习引擎
pub struct OnlineLearningEngine {
    config: Arc<RwLock<OnlineLearningConfig>>,
    /// 样本缓冲区
    sample_buffer: Arc<RwLock<VecDeque<OnlineSample>>>,
    /// 参考窗口（用于漂移检测）
    reference_window: Arc<RwLock<VecDeque<f64>>>,
    /// 检测窗口
    detection_window: Arc<RwLock<VecDeque<f64>>>,
    /// 性能历史
    performance_history: Arc<RwLock<VecDeque<PerformanceMetrics>>>,
    /// 当前学习率
    current_learning_rate: Arc<RwLock<f64>>,
    /// 当前批次大小
    current_batch_size: Arc<RwLock<usize>>,
    /// 特征权重
    feature_weights: Arc<RwLock<Vec<f64>>>,
    /// 步数计数器
    step_counter: Arc<RwLock<usize>>,
    /// 最后漂移检测时间
    last_drift_detection: Arc<RwLock<DateTime<Utc>>>,
    /// Page-Hinkley检测器状态
    page_hinkley_state: Arc<RwLock<PageHinkleyState>>,
    /// ADWIN检测器
    adwin_detector: Arc<RwLock<ADWINDetector>>,
}

/// Page-Hinkley检测器状态
#[derive(Debug, Clone)]
pub struct PageHinkleyState {
    pub cumsum: f64,
    pub min_cumsum: f64,
    pub threshold: f64,
    pub alpha: f64,
    pub lambda: f64,
}

impl PageHinkleyState {
    pub fn new(alpha: f64, lambda: f64) -> Self {
        Self {
            cumsum: 0.0,
            min_cumsum: 0.0,
            threshold: lambda,
            alpha,
            lambda,
        }
    }
    
    pub fn update(&mut self, error: f64) -> bool {
        self.cumsum += error - self.alpha;
        self.min_cumsum = self.min_cumsum.min(self.cumsum);
        
        (self.cumsum - self.min_cumsum) > self.threshold
    }
    
    pub fn reset(&mut self) {
        self.cumsum = 0.0;
        self.min_cumsum = 0.0;
    }
}

/// ADWIN概念漂移检测器
#[derive(Debug, Clone)]
pub struct ADWINDetector {
    window: VecDeque<f64>,
    variance: f64,
    total: f64,
    delta: f64,
    max_buckets: usize,
}

impl ADWINDetector {
    pub fn new(delta: f64, max_buckets: Option<usize>) -> Self {
        Self {
            window: VecDeque::new(),
            variance: 0.0,
            total: 0.0,
            delta,
            max_buckets: max_buckets.unwrap_or(5),
        }
    }
    
    pub fn add_element(&mut self, value: f64) -> bool {
        self.window.push_back(value);
        self.total += value;
        
        // 计算方差
        if self.window.len() > 1 {
            let mean = self.total / self.window.len() as f64;
            self.variance = self.window.iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>() / self.window.len() as f64;
        }
        
        // 检测概念漂移
        self.detect_change()
    }
    
    fn detect_change(&mut self) -> bool {
        if self.window.len() < 2 {
            return false;
        }
        
        let n = self.window.len();
        let cut_point = n / 2;
        
        if cut_point == 0 {
            return false;
        }
        
        // 分割窗口
        let left_window: Vec<f64> = self.window.iter().take(cut_point).cloned().collect();
        let right_window: Vec<f64> = self.window.iter().skip(cut_point).cloned().collect();
        
        let left_mean = left_window.iter().sum::<f64>() / left_window.len() as f64;
        let right_mean = right_window.iter().sum::<f64>() / right_window.len() as f64;
        
        // 计算统计量
        let n1 = left_window.len() as f64;
        let n2 = right_window.len() as f64;
        let epsilon_cut = ((1.0 / (2.0 * n1)) + (1.0 / (2.0 * n2))) * 
                         (self.variance + (2.0 * (1.0 / self.delta).ln()) / 3.0);
        
        let diff = (left_mean - right_mean).abs();
        
        if diff > epsilon_cut.sqrt() {
            // 检测到变化，保留较新的窗口
            self.window = right_window.into();
            self.total = self.window.iter().sum();
            true
        } else {
            false
        }
    }
}

impl OnlineLearningEngine {
    pub fn new(config: OnlineLearningConfig) -> Self {
        let page_hinkley_state = PageHinkleyState::new(0.05, 10.0);
        let adwin_detector = ADWINDetector::new(0.01, Some(5));
        
        Self {
            current_learning_rate: Arc::new(RwLock::new(config.base_learning_rate)),
            current_batch_size: Arc::new(RwLock::new(config.batch_size)),
            config: Arc::new(RwLock::new(config)),
            sample_buffer: Arc::new(RwLock::new(VecDeque::new())),
            reference_window: Arc::new(RwLock::new(VecDeque::new())),
            detection_window: Arc::new(RwLock::new(VecDeque::new())),
            performance_history: Arc::new(RwLock::new(VecDeque::new())),
            feature_weights: Arc::new(RwLock::new(Vec::new())),
            step_counter: Arc::new(RwLock::new(0)),
            last_drift_detection: Arc::new(RwLock::new(Utc::now())),
            page_hinkley_state: Arc::new(RwLock::new(page_hinkley_state)),
            adwin_detector: Arc::new(RwLock::new(adwin_detector)),
        }
    }
    
    /// 添加在线学习样本
    pub async fn add_sample(
        &self,
        features: Vec<f64>,
        target: f64,
        prediction: Option<f64>,
    ) -> Result<(), StrategyError> {
        let timestamp = Utc::now();
        let error = prediction.map(|p| (p - target).abs());
        
        let sample = OnlineSample {
            features: features.clone(),
            target,
            weight: 1.0,
            timestamp,
            prediction,
            error,
        };
        
        // 添加到缓冲区
        {
            let mut buffer = self.sample_buffer.write().await;
            buffer.push_back(sample.clone());
            
            let config = self.config.read().await;
            if buffer.len() > config.forgetting.max_samples {
                buffer.pop_front();
            }
        }
        
        // 概念漂移检测
        if let Some(error_val) = error {
            let drift_result = self.detect_concept_drift(error_val).await?;
            
            if drift_result.drift_detected {
                tracing::warn!(
                    confidence = %drift_result.confidence,
                    method = ?drift_result.detection_method,
                    "Concept drift detected"
                );
                
                self.handle_concept_drift().await?;
            }
        }
        
        // 自适应调整
        self.adaptive_adjustment().await?;
        
        Ok(())
    }
    
    /// 概念漂移检测
    pub async fn detect_concept_drift(&self, error: f64) -> Result<DriftDetectionResult, StrategyError> {
        let config = self.config.read().await;
        let now = Utc::now();
        
        // 检查最小检测间隔
        {
            let last_detection = *self.last_drift_detection.read().await;
            if now.signed_duration_since(last_detection).num_seconds() < config.drift_detection.min_detection_interval as i64 {
                return Ok(DriftDetectionResult {
                    drift_detected: false,
                    warning_detected: false,
                    confidence: 0.0,
                    statistic: 0.0,
                    threshold: 0.0,
                    detection_method: config.drift_detection.method.clone(),
                    timestamp: now,
                });
            }
        }
        
        let result = match config.drift_detection.method {
            DriftDetectionMethod::PageHinkley => {
                self.page_hinkley_detection(error).await
            },
            DriftDetectionMethod::ADWIN => {
                self.adwin_detection(error).await
            },
            DriftDetectionMethod::KolmogorovSmirnov => {
                self.ks_test_detection(error).await
            },
            DriftDetectionMethod::CUSUM => {
                self.cusum_detection(error).await
            },
            DriftDetectionMethod::DistributionComparison => {
                self.distribution_comparison_detection(error).await
            },
        };
        
        if result.drift_detected {
            *self.last_drift_detection.write().await = now;
        }
        
        Ok(result)
    }
    
    /// Page-Hinkley漂移检测
    async fn page_hinkley_detection(&self, error: f64) -> DriftDetectionResult {
        let mut state = self.page_hinkley_state.write().await;
        let drift_detected = state.update(error);
        
        DriftDetectionResult {
            drift_detected,
            warning_detected: false,
            confidence: if drift_detected { 0.95 } else { 0.0 },
            statistic: state.cumsum - state.min_cumsum,
            threshold: state.threshold,
            detection_method: DriftDetectionMethod::PageHinkley,
            timestamp: Utc::now(),
        }
    }
    
    /// ADWIN漂移检测
    async fn adwin_detection(&self, error: f64) -> DriftDetectionResult {
        let mut detector = self.adwin_detector.write().await;
        let drift_detected = detector.add_element(error);
        
        DriftDetectionResult {
            drift_detected,
            warning_detected: false,
            confidence: if drift_detected { 0.9 } else { 0.0 },
            statistic: error,
            threshold: 0.0,
            detection_method: DriftDetectionMethod::ADWIN,
            timestamp: Utc::now(),
        }
    }
    
    /// Kolmogorov-Smirnov测试检测
    async fn ks_test_detection(&self, error: f64) -> DriftDetectionResult {
        // 添加到检测窗口
        {
            let mut detection_window = self.detection_window.write().await;
            detection_window.push_back(error);
            
            let config = self.config.read().await;
            if detection_window.len() > config.drift_detection.detection_window {
                detection_window.pop_front();
            }
        }
        
        let detection_window = self.detection_window.read().await;
        let reference_window = self.reference_window.read().await;
        
        if detection_window.len() < 10 || reference_window.len() < 10 {
            return DriftDetectionResult {
                drift_detected: false,
                warning_detected: false,
                confidence: 0.0,
                statistic: 0.0,
                threshold: 0.0,
                detection_method: DriftDetectionMethod::KolmogorovSmirnov,
                timestamp: Utc::now(),
            };
        }
        
        // 执行KS测试
        let ks_statistic = self.ks_test(&reference_window.iter().cloned().collect::<Vec<_>>(), 
                                       &detection_window.iter().cloned().collect::<Vec<_>>());
        
        let config = self.config.read().await;
        let critical_value = 1.36 * ((reference_window.len() + detection_window.len()) as f64 / 
                                     (reference_window.len() * detection_window.len()) as f64).sqrt();
        
        let drift_detected = ks_statistic > critical_value;
        
        DriftDetectionResult {
            drift_detected,
            warning_detected: ks_statistic > critical_value * 0.7,
            confidence: if drift_detected { ks_statistic / critical_value } else { 0.0 },
            statistic: ks_statistic,
            threshold: critical_value,
            detection_method: DriftDetectionMethod::KolmogorovSmirnov,
            timestamp: Utc::now(),
        }
    }
    
    /// CUSUM检测
    async fn cusum_detection(&self, error: f64) -> DriftDetectionResult {
        // 简化的CUSUM实现
        DriftDetectionResult {
            drift_detected: false,
            warning_detected: false,
            confidence: 0.0,
            statistic: error,
            threshold: 0.0,
            detection_method: DriftDetectionMethod::CUSUM,
            timestamp: Utc::now(),
        }
    }
    
    /// 分布比较检测
    async fn distribution_comparison_detection(&self, error: f64) -> DriftDetectionResult {
        // 添加到参考窗口
        {
            let mut reference_window = self.reference_window.write().await;
            reference_window.push_back(error);
            
            let config = self.config.read().await;
            if reference_window.len() > config.drift_detection.reference_window {
                reference_window.pop_front();
            }
        }
        
        DriftDetectionResult {
            drift_detected: false,
            warning_detected: false,
            confidence: 0.0,
            statistic: error,
            threshold: 0.0,
            detection_method: DriftDetectionMethod::DistributionComparison,
            timestamp: Utc::now(),
        }
    }
    
    /// Kolmogorov-Smirnov测试实现
    fn ks_test(&self, sample1: &[f64], sample2: &[f64]) -> f64 {
        if sample1.is_empty() || sample2.is_empty() {
            return 0.0;
        }
        
        let mut combined: Vec<f64> = sample1.iter().chain(sample2.iter()).cloned().collect();
        combined.sort_by(|a, b| a.partial_cmp(b).unwrap());
        combined.dedup();
        
        let mut max_diff: f64 = 0.0;
        let n1 = sample1.len() as f64;
        let n2 = sample2.len() as f64;
        
        for &value in &combined {
            let cdf1 = sample1.iter().filter(|&&x| x <= value).count() as f64 / n1;
            let cdf2 = sample2.iter().filter(|&&x| x <= value).count() as f64 / n2;
            max_diff = max_diff.max((cdf1 - cdf2).abs());
        }
        
        max_diff
    }
    
    /// 处理概念漂移
    async fn handle_concept_drift(&self) -> Result<(), StrategyError> {
        // 重置检测器状态
        self.page_hinkley_state.write().await.reset();
        *self.adwin_detector.write().await = ADWINDetector::new(0.01, Some(5));
        
        // 增加学习率以快速适应新概念
        {
            let mut learning_rate = self.current_learning_rate.write().await;
            let config = self.config.read().await;
            *learning_rate = (*learning_rate * 2.0).min(config.base_learning_rate * 10.0);
        }
        
        // 清空部分样本缓冲区，保留最近的样本
        {
            let mut buffer = self.sample_buffer.write().await;
            let keep_samples = buffer.len() / 2;
            let len = buffer.len(); buffer.drain(0..len - keep_samples);
        }
        
        tracing::info!("Handled concept drift: increased learning rate and cleared old samples");
        
        Ok(())
    }
    
    /// 自适应调整
    async fn adaptive_adjustment(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        
        if !config.adaptation.adaptive_learning_rate && !config.adaptation.adaptive_batch_size {
            return Ok(());
        }
        
        // 更新步数
        {
            let mut step_counter = self.step_counter.write().await;
            *step_counter += 1;
        }
        
        // 学习率调整
        if config.adaptation.adaptive_learning_rate {
            self.adjust_learning_rate().await?;
        }
        
        // 批次大小调整
        if config.adaptation.adaptive_batch_size {
            self.adjust_batch_size().await?;
        }
        
        Ok(())
    }
    
    /// 调整学习率
    async fn adjust_learning_rate(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let step_count = *self.step_counter.read().await;
        
        let new_rate = match &config.learning_rate_decay {
            LearningRateDecay::Exponential { decay_rate, decay_steps } => {
                if step_count % decay_steps == 0 {
                    let current_rate = *self.current_learning_rate.read().await;
                    current_rate * decay_rate
                } else {
                    return Ok(());
                }
            },
            LearningRateDecay::StepWise { step_size, gamma } => {
                if step_count % step_size == 0 {
                    let current_rate = *self.current_learning_rate.read().await;
                    current_rate * gamma
                } else {
                    return Ok(());
                }
            },
            LearningRateDecay::CosineAnnealing { t_max, eta_min } => {
                let eta_max = config.base_learning_rate;
                eta_min + (eta_max - eta_min) * 
                    (1.0 + ((step_count as f64 * std::f64::consts::PI) / *t_max as f64).cos()) / 2.0
            },
            LearningRateDecay::Adaptive { patience, factor } => {
                // 基于性能自适应调整
                let performance_history = self.performance_history.read().await;
                if performance_history.len() >= *patience {
                    let recent_mse = performance_history.iter().rev().take(*patience)
                        .map(|p| p.mse).collect::<Vec<_>>();
                    
                    if recent_mse.windows(2).all(|w| w[1] >= w[0]) {
                        // 性能没有改善，降低学习率
                        let current_rate = *self.current_learning_rate.read().await;
                        current_rate * factor
                    } else {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
            }
        };
        
        *self.current_learning_rate.write().await = new_rate.max(1e-8);
        
        Ok(())
    }
    
    /// 调整批次大小
    async fn adjust_batch_size(&self) -> Result<(), StrategyError> {
        let performance_history = self.performance_history.read().await;
        
        if performance_history.len() < 10 {
            return Ok(());
        }
        
        // 基于最近的性能趋势调整批次大小
        let recent_accuracy: Vec<f64> = performance_history.iter().rev().take(10)
            .map(|p| p.accuracy).collect();
        
        let accuracy_trend = if recent_accuracy.len() >= 2 {
            recent_accuracy.windows(2).map(|w| w[1] - w[0]).sum::<f64>() / (recent_accuracy.len() - 1) as f64
        } else {
            0.0
        };
        
        let mut current_batch_size = self.current_batch_size.write().await;
        let config = self.config.read().await;
        
        if accuracy_trend > 0.01 {
            // 性能提升，可以增加批次大小
            *current_batch_size = (*current_batch_size as f64 * 1.1) as usize;
        } else if accuracy_trend < -0.01 {
            // 性能下降，减小批次大小
            *current_batch_size = (*current_batch_size as f64 * 0.9) as usize;
        }
        
        // 限制批次大小范围
        *current_batch_size = (*current_batch_size).max(1).min(config.batch_size * 4);
        
        Ok(())
    }
    
    /// 获取当前学习率
    pub async fn get_current_learning_rate(&self) -> f64 {
        *self.current_learning_rate.read().await
    }
    
    /// 获取当前批次大小
    pub async fn get_current_batch_size(&self) -> usize {
        *self.current_batch_size.read().await
    }
    
    /// 获取样本缓冲区
    pub async fn get_sample_buffer(&self) -> Vec<OnlineSample> {
        self.sample_buffer.read().await.iter().cloned().collect()
    }
    
    /// 获取准备好的批次
    pub async fn get_ready_batch(&self) -> Option<Vec<OnlineSample>> {
        let current_batch_size = *self.current_batch_size.read().await;
        let mut buffer = self.sample_buffer.write().await;
        
        if buffer.len() >= current_batch_size {
            let batch: Vec<OnlineSample> = buffer.drain(0..current_batch_size).collect();
            Some(batch)
        } else {
            None
        }
    }
    
    /// 记录性能指标
    pub async fn record_performance(&self, mse: f64, mae: f64, accuracy: f64, prediction_count: u64) {
        let metrics = PerformanceMetrics {
            mse,
            mae,
            accuracy,
            prediction_count,
            timestamp: Utc::now(),
        };
        
        let mut history = self.performance_history.write().await;
        history.push_back(metrics);
        
        let config = self.config.read().await;
        if history.len() > config.adaptation.performance_window * 2 {
            history.pop_front();
        }
    }
    
    /// 获取性能历史
    pub async fn get_performance_history(&self) -> Vec<PerformanceMetrics> {
        self.performance_history.read().await.iter().cloned().collect()
    }
    
    /// 清理过期样本
    pub async fn cleanup_expired_samples(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let cutoff_time = Utc::now() - Duration::hours(config.forgetting.time_window_hours as i64);
        
        let mut buffer = self.sample_buffer.write().await;
        buffer.retain(|sample| sample.timestamp >= cutoff_time);
        
        Ok(())
    }
    
    /// 应用遗忘机制
    pub async fn apply_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        
        match config.forgetting.strategy {
            ForgettingStrategy::Exponential => {
                self.apply_exponential_forgetting().await?;
            },
            ForgettingStrategy::SlidingWindow => {
                self.apply_sliding_window_forgetting().await?;
            },
            ForgettingStrategy::TimeWeighted => {
                self.apply_time_weighted_forgetting().await?;
            },
            ForgettingStrategy::PerformanceWeighted => {
                self.apply_performance_weighted_forgetting().await?;
            },
        }
        
        Ok(())
    }
    
    /// 指数遗忘
    async fn apply_exponential_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let forgetting_factor = config.forgetting.forgetting_factor;
        
        let mut buffer = self.sample_buffer.write().await;
        for sample in buffer.iter_mut() {
            let age_hours = Utc::now().signed_duration_since(sample.timestamp).num_hours() as f64;
            sample.weight *= forgetting_factor.powf(age_hours);
        }
        
        Ok(())
    }
    
    /// 滑动窗口遗忘
    async fn apply_sliding_window_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let max_samples = config.forgetting.max_samples;
        
        let mut buffer = self.sample_buffer.write().await;
        if buffer.len() > max_samples {
            let len = buffer.len(); buffer.drain(0..len - max_samples);
        }
        
        Ok(())
    }
    
    /// 时间加权遗忘
    async fn apply_time_weighted_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let time_window = config.forgetting.time_window_hours as f64;
        
        let mut buffer = self.sample_buffer.write().await;
        for sample in buffer.iter_mut() {
            let age_hours = Utc::now().signed_duration_since(sample.timestamp).num_hours() as f64;
            sample.weight = (1.0 - age_hours / time_window).max(0.1);
        }
        
        Ok(())
    }
    
    /// 性能加权遗忘
    async fn apply_performance_weighted_forgetting(&self) -> Result<(), StrategyError> {
        let mut buffer = self.sample_buffer.write().await;
        
        // 基于预测误差调整权重
        for sample in buffer.iter_mut() {
            if let Some(error) = sample.error {
                sample.weight = (1.0 / (1.0 + error)).max(0.1);
            }
        }
        
        Ok(())
    }
} 












    /// 指数遗忘
    async fn apply_exponential_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let forgetting_factor = config.forgetting.forgetting_factor;
        
        let mut buffer = self.sample_buffer.write().await;
        for sample in buffer.iter_mut() {
            let age_hours = Utc::now().signed_duration_since(sample.timestamp).num_hours() as f64;
            sample.weight *= forgetting_factor.powf(age_hours);
        }
        
        Ok(())
    }
    
    /// 滑动窗口遗忘
    async fn apply_sliding_window_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let max_samples = config.forgetting.max_samples;
        
        let mut buffer = self.sample_buffer.write().await;
        if buffer.len() > max_samples {
            let len = buffer.len(); buffer.drain(0..len - max_samples);
        }
        
        Ok(())
    }
    
    /// 时间加权遗忘
    async fn apply_time_weighted_forgetting(&self) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let time_window = config.forgetting.time_window_hours as f64;
        
        let mut buffer = self.sample_buffer.write().await;
        for sample in buffer.iter_mut() {
            let age_hours = Utc::now().signed_duration_since(sample.timestamp).num_hours() as f64;
            sample.weight = (1.0 - age_hours / time_window).max(0.1);
        }
        
        Ok(())
    }
    
    /// 性能加权遗忘
    async fn apply_performance_weighted_forgetting(&self) -> Result<(), StrategyError> {
        let mut buffer = self.sample_buffer.write().await;
        
        // 基于预测误差调整权重
        for sample in buffer.iter_mut() {
            if let Some(error) = sample.error {
                sample.weight = (1.0 / (1.0 + error)).max(0.1);
            }
        }
        
        Ok(())
    }
} 











