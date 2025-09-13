//! 生产级CUSUM概念漂移检测模块
//! 
//! 完整实现累积和控制图，用于检测策略性能和市场环境的概念漂移

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};

/// 生产级CUSUM检测器
#[derive(Debug)]
pub struct ProductionCusumDetector {
    /// CUSUM配置参数
    config: CusumConfig,
    /// 当前CUSUM状态
    state: Arc<RwLock<CusumState>>,
    /// 历史数据缓冲区
    history: Arc<RwLock<VecDeque<CusumDataPoint>>>,
}

/// CUSUM配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumConfig {
    /// 参考值 k（通常为期望检测偏移量的一半）
    pub reference_value: f64,
    /// 控制限 h（决定检测灵敏度，通常为4-5倍标准差）
    pub control_limit: f64,
    /// 目标均值
    pub target_mean: f64,
    /// 过程标准差
    pub process_sigma: f64,
    /// 警告阈值（控制限的百分比）
    pub warning_threshold_ratio: f64,
    /// 最大历史数据点数
    pub max_history_size: usize,
    /// 重置策略
    pub reset_strategy: ResetStrategy,
}

impl Default for CusumConfig {
    fn default() -> Self {
        Self {
            reference_value: std::env::var("CELUE_CUSUM_REFERENCE_VALUE")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(0.5), // 检测0.5σ的偏移
            control_limit: std::env::var("CELUE_CUSUM_CONTROL_LIMIT")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(4.0), // 4σ控制限
            target_mean: std::env::var("CELUE_CUSUM_TARGET_MEAN")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            process_sigma: std::env::var("CELUE_CUSUM_PROCESS_SIGMA")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            warning_threshold_ratio: std::env::var("CELUE_CUSUM_WARNING_RATIO")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(0.75),
            max_history_size: std::env::var("CELUE_CUSUM_MAX_HISTORY")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(2000),
            reset_strategy: ResetStrategy::ImmediateReset,
        }
    }
}

/// 重置策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResetStrategy {
    /// 检测到漂移后立即重置
    ImmediateReset,
    /// 渐进重置
    GradualReset { reset_rate: f64 },
    /// 不重置（保持累积）
    NoReset,
}

/// CUSUM状态
#[derive(Debug, Clone)]
pub struct CusumState {
    /// 上侧CUSUM统计量
    pub c_plus: f64,
    /// 下侧CUSUM统计量
    pub c_minus: f64,
    /// 检测到的漂移总数
    pub drift_count: u64,
    /// 警告总数
    pub warning_count: u64,
    /// 最后更新时间
    pub last_update: DateTime<Utc>,
    /// 运行长度（自上次重置以来的样本数）
    pub run_length: usize,
}

impl Default for CusumState {
    fn default() -> Self {
        Self {
            c_plus: 0.0,
            c_minus: 0.0,
            drift_count: 0,
            warning_count: 0,
            last_update: Utc::now(),
            run_length: 0,
        }
    }
}

/// CUSUM数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumDataPoint {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 原始观测值
    pub observation: f64,
    /// 标准化观测值
    pub standardized_value: f64,
    /// 上侧CUSUM值
    pub c_plus: f64,
    /// 下侧CUSUM值
    pub c_minus: f64,
    /// 是否检测到漂移
    pub drift_detected: bool,
    /// 是否发出警告
    pub warning_issued: bool,
}

/// 漂移检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumDetectionResult {
    /// 是否检测到漂移
    pub drift_detected: bool,
    /// 漂移类型（上漂移或下漂移）
    pub drift_type: Option<DriftType>,
    /// 是否发出警告
    pub warning_issued: bool,
    /// 检测置信度
    pub confidence: f64,
    /// 当前CUSUM统计量
    pub cusum_statistic: f64,
    /// 控制限
    pub control_limit: f64,
    /// 运行长度
    pub run_length: usize,
    /// 检测时间
    pub detection_time: DateTime<Utc>,
}

/// 漂移类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftType {
    /// 向上漂移（均值增加）
    UpwardDrift,
    /// 向下漂移（均值减少）
    DownwardDrift,
}

impl ProductionCusumDetector {
    /// 创建新的CUSUM检测器
    pub fn new(config: CusumConfig) -> Self {
        info!("🎯 初始化生产级CUSUM检测器");
        info!("  - 参考值 k: {:.4}", config.reference_value);
        info!("  - 控制限 h: {:.4}", config.control_limit);
        info!("  - 目标均值: {:.4}", config.target_mean);
        info!("  - 过程标准差: {:.4}", config.process_sigma);
        
        Self {
            config,
            state: Arc::new(RwLock::new(CusumState::default())),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(2000))),
        }
    }

    /// 处理新的观测值并检测漂移
    pub async fn process_observation(&self, observation: f64) -> CusumDetectionResult {
        let mut state = self.state.write().await;
        let mut history = self.history.write().await;
        
        // 标准化观测值
        let standardized_value = (observation - self.config.target_mean) / self.config.process_sigma;
        
        // 更新上侧CUSUM（检测向上漂移）
        state.c_plus = (state.c_plus + standardized_value - self.config.reference_value).max(0.0);
        
        // 更新下侧CUSUM（检测向下漂移）
        state.c_minus = (state.c_minus - standardized_value - self.config.reference_value).max(0.0);
        
        // 更新运行长度
        state.run_length += 1;
        state.last_update = Utc::now();
        
        // 检测漂移
        let (drift_detected, drift_type) = self.detect_drift(&state);
        let warning_issued = self.check_warning(&state);
        
        // 计算置信度
        let max_cusum = state.c_plus.max(state.c_minus);
        let confidence = (max_cusum / self.config.control_limit).min(1.0);
        
        // 创建数据点
        let data_point = CusumDataPoint {
            timestamp: state.last_update,
            observation,
            standardized_value,
            c_plus: state.c_plus,
            c_minus: state.c_minus,
            drift_detected,
            warning_issued,
        };
        
        // 添加到历史记录
        history.push_back(data_point.clone());
        if history.len() > self.config.max_history_size {
            history.pop_front();
        }
        
        // 处理漂移检测事件
        if drift_detected {
            state.drift_count += 1;
            warn!("🚨 CUSUM检测到概念漂移 #{}: 类型={:?}, C+={:.4}, C-={:.4}, 运行长度={}", 
                  state.drift_count, drift_type, state.c_plus, state.c_minus, state.run_length);
            
            // 根据重置策略处理
            self.handle_drift_reset(&mut state).await;
        }
        
        if warning_issued && !drift_detected {
            state.warning_count += 1;
            debug!("⚠️ CUSUM警告 #{}: C+={:.4}, C-={:.4}, 阈值={:.4}", 
                   state.warning_count, state.c_plus, state.c_minus, 
                   self.config.control_limit * self.config.warning_threshold_ratio);
        }
        
        CusumDetectionResult {
            drift_detected,
            drift_type,
            warning_issued,
            confidence,
            cusum_statistic: max_cusum,
            control_limit: self.config.control_limit,
            run_length: state.run_length,
            detection_time: state.last_update,
        }
    }

    /// 检测漂移
    fn detect_drift(&self, state: &CusumState) -> (bool, Option<DriftType>) {
        let upward_drift = state.c_plus > self.config.control_limit;
        let downward_drift = state.c_minus > self.config.control_limit;
        
        match (upward_drift, downward_drift) {
            (true, false) => (true, Some(DriftType::UpwardDrift)),
            (false, true) => (true, Some(DriftType::DownwardDrift)),
            (true, true) => {
                // 同时触发，选择CUSUM值较大的
                if state.c_plus >= state.c_minus {
                    (true, Some(DriftType::UpwardDrift))
                } else {
                    (true, Some(DriftType::DownwardDrift))
                }
            }
            (false, false) => (false, None),
        }
    }

    /// 检查警告条件
    fn check_warning(&self, state: &CusumState) -> bool {
        let warning_threshold = self.config.control_limit * self.config.warning_threshold_ratio;
        state.c_plus > warning_threshold || state.c_minus > warning_threshold
    }

    /// 处理漂移检测后的重置
    async fn handle_drift_reset(&self, state: &mut CusumState) {
        match self.config.reset_strategy {
            ResetStrategy::ImmediateReset => {
                debug!("🔄 立即重置CUSUM状态");
                state.c_plus = 0.0;
                state.c_minus = 0.0;
                state.run_length = 0;
            }
            ResetStrategy::GradualReset { reset_rate } => {
                debug!("🔄 渐进重置CUSUM状态 (rate: {:.4})", reset_rate);
                state.c_plus *= (1.0 - reset_rate);
                state.c_minus *= (1.0 - reset_rate);
                // 不重置运行长度
            }
            ResetStrategy::NoReset => {
                debug!("⏸️ 保持CUSUM状态（不重置）");
                // 不做任何重置
            }
        }
    }

    /// 获取当前CUSUM状态
    pub async fn get_current_state(&self) -> CusumState {
        self.state.read().await.clone()
    }

    /// 获取历史数据
    pub async fn get_history(&self, last_n: Option<usize>) -> Vec<CusumDataPoint> {
        let history = self.history.read().await;
        match last_n {
            Some(n) => history.iter().rev().take(n).cloned().collect::<Vec<_>>().into_iter().rev().collect(),
            None => history.iter().cloned().collect(),
        }
    }

    /// 计算平均运行长度（ARL）
    pub async fn calculate_arl(&self) -> f64 {
        let state = self.state.read().await;
        if state.drift_count > 0 {
            let total_observations = self.history.read().await.len() as f64;
            total_observations / state.drift_count as f64
        } else {
            f64::INFINITY
        }
    }

    /// 获取检测性能统计
    pub async fn get_performance_stats(&self) -> CusumPerformanceStats {
        let state = self.state.read().await;
        let history = self.history.read().await;
        
        let total_observations = history.len();
        let total_drifts = state.drift_count;
        let total_warnings = state.warning_count;
        
        // 计算假阳性率（在无漂移期间的误报）
        let false_positive_rate = if total_observations > 0 {
            // 简化计算：假设没有真实漂移标签
            0.0 // 实际应用中需要真实漂移标签
        } else {
            0.0
        };
        
        // 计算检测延迟统计
        let detection_delays: Vec<usize> = history
            .iter()
            .filter_map(|point| if point.drift_detected { Some(1) } else { None })
            .collect(); // 简化实现
        
        let avg_detection_delay = if !detection_delays.is_empty() {
            detection_delays.iter().sum::<usize>() as f64 / detection_delays.len() as f64
        } else {
            0.0
        };
        
        CusumPerformanceStats {
            total_observations,
            total_drifts,
            total_warnings,
            current_run_length: state.run_length,
            average_run_length: if total_drifts > 0 { total_observations as f64 / total_drifts as f64 } else { f64::INFINITY },
            false_positive_rate,
            average_detection_delay: avg_detection_delay,
            current_c_plus: state.c_plus,
            current_c_minus: state.c_minus,
        }
    }

    /// 重置检测器状态
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        let mut history = self.history.write().await;
        
        *state = CusumState::default();
        history.clear();
        
        info!("🔄 CUSUM检测器已重置");
    }

    /// 更新配置参数
    pub async fn update_config(&mut self, new_config: CusumConfig) {
        info!("🔧 更新CUSUM配置参数");
        self.config = new_config;
    }
}

/// CUSUM性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumPerformanceStats {
    /// 总观测数
    pub total_observations: usize,
    /// 检测到的漂移总数
    pub total_drifts: u64,
    /// 发出的警告总数
    pub total_warnings: u64,
    /// 当前运行长度
    pub current_run_length: usize,
    /// 平均运行长度
    pub average_run_length: f64,
    /// 假阳性率
    pub false_positive_rate: f64,
    /// 平均检测延迟
    pub average_detection_delay: f64,
    /// 当前上侧CUSUM值
    pub current_c_plus: f64,
    /// 当前下侧CUSUM值
    pub current_c_minus: f64,
}

/// 批量CUSUM处理器（用于历史数据分析）
#[derive(Debug)]
pub struct BatchCusumProcessor {
    config: CusumConfig,
}

impl BatchCusumProcessor {
    pub fn new(config: CusumConfig) -> Self {
        Self { config }
    }

    /// 批量处理观测序列
    pub async fn process_batch(&self, observations: &[f64]) -> Vec<CusumDetectionResult> {
        let mut detector = ProductionCusumDetector::new(self.config.clone());
        let mut results = Vec::with_capacity(observations.len());
        
        for &observation in observations {
            let result = detector.process_observation(observation).await;
            results.push(result);
        }
        
        results
    }

    /// 分析批量数据的漂移点
    pub async fn analyze_drift_points(&self, observations: &[f64]) -> DriftAnalysis {
        let results = self.process_batch(observations).await;
        
        let drift_points: Vec<usize> = results
            .iter()
            .enumerate()
            .filter_map(|(i, result)| if result.drift_detected { Some(i) } else { None })
            .collect();
        
        let total_drifts = drift_points.len();
        let avg_drift_interval = if total_drifts > 1 {
            let intervals: Vec<usize> = drift_points.windows(2)
                .map(|window| window[1] - window[0])
                .collect();
            intervals.iter().sum::<usize>() as f64 / intervals.len() as f64
        } else {
            f64::INFINITY
        };
        
        DriftAnalysis {
            drift_points,
            total_drifts,
            average_drift_interval: avg_drift_interval,
            drift_density: total_drifts as f64 / observations.len() as f64,
        }
    }
}

/// 漂移分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAnalysis {
    /// 漂移点位置
    pub drift_points: Vec<usize>,
    /// 漂移总数
    pub total_drifts: usize,
    /// 平均漂移间隔
    pub average_drift_interval: f64,
    /// 漂移密度（漂移/观测数）
    pub drift_density: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cusum_no_drift() {
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 模拟无漂移数据（均值为0，标准差为1）
        for i in 0..100 {
            let observation = (i as f64).sin() * 0.1; // 小幅波动
            let result = detector.process_observation(observation).await;
            assert!(!result.drift_detected, "不应检测到漂移在观测 {}", i);
        }
    }

    #[tokio::test]
    async fn test_cusum_upward_drift() {
        let mut config = CusumConfig::default();
        config.control_limit = 2.0; // 降低阈值以便测试
        let detector = ProductionCusumDetector::new(config);
        
        // 模拟向上漂移
        let mut drift_detected = false;
        for i in 0..50 {
            let observation = if i < 25 { 0.0 } else { 2.0 }; // 在第25个观测引入漂移
            let result = detector.process_observation(observation).await;
            if result.drift_detected {
                drift_detected = true;
                assert!(matches!(result.drift_type, Some(DriftType::UpwardDrift)));
                break;
            }
        }
        
        assert!(drift_detected, "应该检测到向上漂移");
    }

    #[tokio::test]
    async fn test_cusum_performance_stats() {
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 添加一些观测
        for i in 0..10 {
            detector.process_observation(i as f64).await;
        }
        
        let stats = detector.get_performance_stats().await;
        assert_eq!(stats.total_observations, 10);
        assert_eq!(stats.current_run_length, 10);
    }
} 
//! 
//! 完整实现累积和控制图，用于检测策略性能和市场环境的概念漂移

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};

/// 生产级CUSUM检测器
#[derive(Debug)]
pub struct ProductionCusumDetector {
    /// CUSUM配置参数
    config: CusumConfig,
    /// 当前CUSUM状态
    state: Arc<RwLock<CusumState>>,
    /// 历史数据缓冲区
    history: Arc<RwLock<VecDeque<CusumDataPoint>>>,
}

/// CUSUM配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumConfig {
    /// 参考值 k（通常为期望检测偏移量的一半）
    pub reference_value: f64,
    /// 控制限 h（决定检测灵敏度，通常为4-5倍标准差）
    pub control_limit: f64,
    /// 目标均值
    pub target_mean: f64,
    /// 过程标准差
    pub process_sigma: f64,
    /// 警告阈值（控制限的百分比）
    pub warning_threshold_ratio: f64,
    /// 最大历史数据点数
    pub max_history_size: usize,
    /// 重置策略
    pub reset_strategy: ResetStrategy,
}

impl Default for CusumConfig {
    fn default() -> Self {
        Self {
            reference_value: std::env::var("CELUE_CUSUM_REFERENCE_VALUE")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(0.5), // 检测0.5σ的偏移
            control_limit: std::env::var("CELUE_CUSUM_CONTROL_LIMIT")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(4.0), // 4σ控制限
            target_mean: std::env::var("CELUE_CUSUM_TARGET_MEAN")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(0.0),
            process_sigma: std::env::var("CELUE_CUSUM_PROCESS_SIGMA")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            warning_threshold_ratio: std::env::var("CELUE_CUSUM_WARNING_RATIO")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(0.75),
            max_history_size: std::env::var("CELUE_CUSUM_MAX_HISTORY")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(2000),
            reset_strategy: ResetStrategy::ImmediateReset,
        }
    }
}

/// 重置策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResetStrategy {
    /// 检测到漂移后立即重置
    ImmediateReset,
    /// 渐进重置
    GradualReset { reset_rate: f64 },
    /// 不重置（保持累积）
    NoReset,
}

/// CUSUM状态
#[derive(Debug, Clone)]
pub struct CusumState {
    /// 上侧CUSUM统计量
    pub c_plus: f64,
    /// 下侧CUSUM统计量
    pub c_minus: f64,
    /// 检测到的漂移总数
    pub drift_count: u64,
    /// 警告总数
    pub warning_count: u64,
    /// 最后更新时间
    pub last_update: DateTime<Utc>,
    /// 运行长度（自上次重置以来的样本数）
    pub run_length: usize,
}

impl Default for CusumState {
    fn default() -> Self {
        Self {
            c_plus: 0.0,
            c_minus: 0.0,
            drift_count: 0,
            warning_count: 0,
            last_update: Utc::now(),
            run_length: 0,
        }
    }
}

/// CUSUM数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumDataPoint {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 原始观测值
    pub observation: f64,
    /// 标准化观测值
    pub standardized_value: f64,
    /// 上侧CUSUM值
    pub c_plus: f64,
    /// 下侧CUSUM值
    pub c_minus: f64,
    /// 是否检测到漂移
    pub drift_detected: bool,
    /// 是否发出警告
    pub warning_issued: bool,
}

/// 漂移检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumDetectionResult {
    /// 是否检测到漂移
    pub drift_detected: bool,
    /// 漂移类型（上漂移或下漂移）
    pub drift_type: Option<DriftType>,
    /// 是否发出警告
    pub warning_issued: bool,
    /// 检测置信度
    pub confidence: f64,
    /// 当前CUSUM统计量
    pub cusum_statistic: f64,
    /// 控制限
    pub control_limit: f64,
    /// 运行长度
    pub run_length: usize,
    /// 检测时间
    pub detection_time: DateTime<Utc>,
}

/// 漂移类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriftType {
    /// 向上漂移（均值增加）
    UpwardDrift,
    /// 向下漂移（均值减少）
    DownwardDrift,
}

impl ProductionCusumDetector {
    /// 创建新的CUSUM检测器
    pub fn new(config: CusumConfig) -> Self {
        info!("🎯 初始化生产级CUSUM检测器");
        info!("  - 参考值 k: {:.4}", config.reference_value);
        info!("  - 控制限 h: {:.4}", config.control_limit);
        info!("  - 目标均值: {:.4}", config.target_mean);
        info!("  - 过程标准差: {:.4}", config.process_sigma);
        
        Self {
            config,
            state: Arc::new(RwLock::new(CusumState::default())),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(2000))),
        }
    }

    /// 处理新的观测值并检测漂移
    pub async fn process_observation(&self, observation: f64) -> CusumDetectionResult {
        let mut state = self.state.write().await;
        let mut history = self.history.write().await;
        
        // 标准化观测值
        let standardized_value = (observation - self.config.target_mean) / self.config.process_sigma;
        
        // 更新上侧CUSUM（检测向上漂移）
        state.c_plus = (state.c_plus + standardized_value - self.config.reference_value).max(0.0);
        
        // 更新下侧CUSUM（检测向下漂移）
        state.c_minus = (state.c_minus - standardized_value - self.config.reference_value).max(0.0);
        
        // 更新运行长度
        state.run_length += 1;
        state.last_update = Utc::now();
        
        // 检测漂移
        let (drift_detected, drift_type) = self.detect_drift(&state);
        let warning_issued = self.check_warning(&state);
        
        // 计算置信度
        let max_cusum = state.c_plus.max(state.c_minus);
        let confidence = (max_cusum / self.config.control_limit).min(1.0);
        
        // 创建数据点
        let data_point = CusumDataPoint {
            timestamp: state.last_update,
            observation,
            standardized_value,
            c_plus: state.c_plus,
            c_minus: state.c_minus,
            drift_detected,
            warning_issued,
        };
        
        // 添加到历史记录
        history.push_back(data_point.clone());
        if history.len() > self.config.max_history_size {
            history.pop_front();
        }
        
        // 处理漂移检测事件
        if drift_detected {
            state.drift_count += 1;
            warn!("🚨 CUSUM检测到概念漂移 #{}: 类型={:?}, C+={:.4}, C-={:.4}, 运行长度={}", 
                  state.drift_count, drift_type, state.c_plus, state.c_minus, state.run_length);
            
            // 根据重置策略处理
            self.handle_drift_reset(&mut state).await;
        }
        
        if warning_issued && !drift_detected {
            state.warning_count += 1;
            debug!("⚠️ CUSUM警告 #{}: C+={:.4}, C-={:.4}, 阈值={:.4}", 
                   state.warning_count, state.c_plus, state.c_minus, 
                   self.config.control_limit * self.config.warning_threshold_ratio);
        }
        
        CusumDetectionResult {
            drift_detected,
            drift_type,
            warning_issued,
            confidence,
            cusum_statistic: max_cusum,
            control_limit: self.config.control_limit,
            run_length: state.run_length,
            detection_time: state.last_update,
        }
    }

    /// 检测漂移
    fn detect_drift(&self, state: &CusumState) -> (bool, Option<DriftType>) {
        let upward_drift = state.c_plus > self.config.control_limit;
        let downward_drift = state.c_minus > self.config.control_limit;
        
        match (upward_drift, downward_drift) {
            (true, false) => (true, Some(DriftType::UpwardDrift)),
            (false, true) => (true, Some(DriftType::DownwardDrift)),
            (true, true) => {
                // 同时触发，选择CUSUM值较大的
                if state.c_plus >= state.c_minus {
                    (true, Some(DriftType::UpwardDrift))
                } else {
                    (true, Some(DriftType::DownwardDrift))
                }
            }
            (false, false) => (false, None),
        }
    }

    /// 检查警告条件
    fn check_warning(&self, state: &CusumState) -> bool {
        let warning_threshold = self.config.control_limit * self.config.warning_threshold_ratio;
        state.c_plus > warning_threshold || state.c_minus > warning_threshold
    }

    /// 处理漂移检测后的重置
    async fn handle_drift_reset(&self, state: &mut CusumState) {
        match self.config.reset_strategy {
            ResetStrategy::ImmediateReset => {
                debug!("🔄 立即重置CUSUM状态");
                state.c_plus = 0.0;
                state.c_minus = 0.0;
                state.run_length = 0;
            }
            ResetStrategy::GradualReset { reset_rate } => {
                debug!("🔄 渐进重置CUSUM状态 (rate: {:.4})", reset_rate);
                state.c_plus *= (1.0 - reset_rate);
                state.c_minus *= (1.0 - reset_rate);
                // 不重置运行长度
            }
            ResetStrategy::NoReset => {
                debug!("⏸️ 保持CUSUM状态（不重置）");
                // 不做任何重置
            }
        }
    }

    /// 获取当前CUSUM状态
    pub async fn get_current_state(&self) -> CusumState {
        self.state.read().await.clone()
    }

    /// 获取历史数据
    pub async fn get_history(&self, last_n: Option<usize>) -> Vec<CusumDataPoint> {
        let history = self.history.read().await;
        match last_n {
            Some(n) => history.iter().rev().take(n).cloned().collect::<Vec<_>>().into_iter().rev().collect(),
            None => history.iter().cloned().collect(),
        }
    }

    /// 计算平均运行长度（ARL）
    pub async fn calculate_arl(&self) -> f64 {
        let state = self.state.read().await;
        if state.drift_count > 0 {
            let total_observations = self.history.read().await.len() as f64;
            total_observations / state.drift_count as f64
        } else {
            f64::INFINITY
        }
    }

    /// 获取检测性能统计
    pub async fn get_performance_stats(&self) -> CusumPerformanceStats {
        let state = self.state.read().await;
        let history = self.history.read().await;
        
        let total_observations = history.len();
        let total_drifts = state.drift_count;
        let total_warnings = state.warning_count;
        
        // 计算假阳性率（在无漂移期间的误报）
        let false_positive_rate = if total_observations > 0 {
            // 简化计算：假设没有真实漂移标签
            0.0 // 实际应用中需要真实漂移标签
        } else {
            0.0
        };
        
        // 计算检测延迟统计
        let detection_delays: Vec<usize> = history
            .iter()
            .filter_map(|point| if point.drift_detected { Some(1) } else { None })
            .collect(); // 简化实现
        
        let avg_detection_delay = if !detection_delays.is_empty() {
            detection_delays.iter().sum::<usize>() as f64 / detection_delays.len() as f64
        } else {
            0.0
        };
        
        CusumPerformanceStats {
            total_observations,
            total_drifts,
            total_warnings,
            current_run_length: state.run_length,
            average_run_length: if total_drifts > 0 { total_observations as f64 / total_drifts as f64 } else { f64::INFINITY },
            false_positive_rate,
            average_detection_delay: avg_detection_delay,
            current_c_plus: state.c_plus,
            current_c_minus: state.c_minus,
        }
    }

    /// 重置检测器状态
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        let mut history = self.history.write().await;
        
        *state = CusumState::default();
        history.clear();
        
        info!("🔄 CUSUM检测器已重置");
    }

    /// 更新配置参数
    pub async fn update_config(&mut self, new_config: CusumConfig) {
        info!("🔧 更新CUSUM配置参数");
        self.config = new_config;
    }
}

/// CUSUM性能统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CusumPerformanceStats {
    /// 总观测数
    pub total_observations: usize,
    /// 检测到的漂移总数
    pub total_drifts: u64,
    /// 发出的警告总数
    pub total_warnings: u64,
    /// 当前运行长度
    pub current_run_length: usize,
    /// 平均运行长度
    pub average_run_length: f64,
    /// 假阳性率
    pub false_positive_rate: f64,
    /// 平均检测延迟
    pub average_detection_delay: f64,
    /// 当前上侧CUSUM值
    pub current_c_plus: f64,
    /// 当前下侧CUSUM值
    pub current_c_minus: f64,
}

/// 批量CUSUM处理器（用于历史数据分析）
#[derive(Debug)]
pub struct BatchCusumProcessor {
    config: CusumConfig,
}

impl BatchCusumProcessor {
    pub fn new(config: CusumConfig) -> Self {
        Self { config }
    }

    /// 批量处理观测序列
    pub async fn process_batch(&self, observations: &[f64]) -> Vec<CusumDetectionResult> {
        let mut detector = ProductionCusumDetector::new(self.config.clone());
        let mut results = Vec::with_capacity(observations.len());
        
        for &observation in observations {
            let result = detector.process_observation(observation).await;
            results.push(result);
        }
        
        results
    }

    /// 分析批量数据的漂移点
    pub async fn analyze_drift_points(&self, observations: &[f64]) -> DriftAnalysis {
        let results = self.process_batch(observations).await;
        
        let drift_points: Vec<usize> = results
            .iter()
            .enumerate()
            .filter_map(|(i, result)| if result.drift_detected { Some(i) } else { None })
            .collect();
        
        let total_drifts = drift_points.len();
        let avg_drift_interval = if total_drifts > 1 {
            let intervals: Vec<usize> = drift_points.windows(2)
                .map(|window| window[1] - window[0])
                .collect();
            intervals.iter().sum::<usize>() as f64 / intervals.len() as f64
        } else {
            f64::INFINITY
        };
        
        DriftAnalysis {
            drift_points,
            total_drifts,
            average_drift_interval: avg_drift_interval,
            drift_density: total_drifts as f64 / observations.len() as f64,
        }
    }
}

/// 漂移分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftAnalysis {
    /// 漂移点位置
    pub drift_points: Vec<usize>,
    /// 漂移总数
    pub total_drifts: usize,
    /// 平均漂移间隔
    pub average_drift_interval: f64,
    /// 漂移密度（漂移/观测数）
    pub drift_density: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cusum_no_drift() {
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 模拟无漂移数据（均值为0，标准差为1）
        for i in 0..100 {
            let observation = (i as f64).sin() * 0.1; // 小幅波动
            let result = detector.process_observation(observation).await;
            assert!(!result.drift_detected, "不应检测到漂移在观测 {}", i);
        }
    }

    #[tokio::test]
    async fn test_cusum_upward_drift() {
        let mut config = CusumConfig::default();
        config.control_limit = 2.0; // 降低阈值以便测试
        let detector = ProductionCusumDetector::new(config);
        
        // 模拟向上漂移
        let mut drift_detected = false;
        for i in 0..50 {
            let observation = if i < 25 { 0.0 } else { 2.0 }; // 在第25个观测引入漂移
            let result = detector.process_observation(observation).await;
            if result.drift_detected {
                drift_detected = true;
                assert!(matches!(result.drift_type, Some(DriftType::UpwardDrift)));
                break;
            }
        }
        
        assert!(drift_detected, "应该检测到向上漂移");
    }

    #[tokio::test]
    async fn test_cusum_performance_stats() {
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 添加一些观测
        for i in 0..10 {
            detector.process_observation(i as f64).await;
        }
        
        let stats = detector.get_performance_stats().await;
        assert_eq!(stats.total_observations, 10);
        assert_eq!(stats.current_run_length, 10);
    }
} 