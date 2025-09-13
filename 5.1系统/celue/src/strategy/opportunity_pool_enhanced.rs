//! 🚀 全局套利机会池增强功能
//! 
//! 实现剩余5%功能：
//! - 动态回测与参数优化
//! - 路径失效自动回溯
//! - 评分权重自动调优

use crate::types::{ArbitrageOpportunityCore, OpportunityEvaluation, StrategyError};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

/// 回测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// 回测窗口大小（秒）
    pub window_size_seconds: i64,
    /// 最小样本数量
    pub min_sample_size: usize,
    /// 参数调整步长
    pub adjustment_step: f64,
    /// 最大调整幅度
    pub max_adjustment_range: f64,
    /// 评估间隔（秒）
    pub evaluation_interval_seconds: u64,
    /// 性能历史最大记录数
    pub max_performance_history: usize,
    /// 成功率评分乘数
    pub success_rate_score_multiplier: f64,
    /// 速度评分基准（毫秒）
    pub speed_score_baseline_ms: f64,
    /// 速度评分乘数
    pub speed_score_multiplier: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            window_size_seconds: 3600, // 1小时
            min_sample_size: 100,
            adjustment_step: 0.05,
            max_adjustment_range: 0.3,
            evaluation_interval_seconds: 300, // 5分钟
            max_performance_history: 100,
            success_rate_score_multiplier: 100.0,
            speed_score_baseline_ms: 1000.0,
            speed_score_multiplier: 10.0,
        }
    }
}

/// 历史执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub opportunity_id: String,
    pub strategy_type: String,
    pub timestamp: DateTime<Utc>,
    pub expected_profit: f64,
    pub actual_profit: f64,
    pub execution_time_ms: u64,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub path_info: PathInfo,
}

/// 路径信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathInfo {
    pub exchanges: Vec<String>,
    pub symbols: Vec<String>,
    pub execution_steps: Vec<ExecutionStep>,
}

/// 执行步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub step_type: StepType,
    pub exchange: String,
    pub symbol: String,
    pub side: String,
    pub price: f64,
    pub quantity: f64,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    OrderPlace,
    OrderFill,
    OrderCancel,
    Transfer,
}

/// 权重优化器
#[derive(Debug, Clone)]
pub struct WeightOptimizer {
    /// 当前权重
    pub current_weights: ScoreWeights,
    /// 历史性能记录
    pub performance_history: VecDeque<PerformanceMetric>,
    /// 最佳权重记录
    pub best_weights: ScoreWeights,
    /// 最佳性能得分
    pub best_performance: f64,
}

/// 评分权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub profit_weight: f64,
    pub liquidity_weight: f64,
    pub latency_weight: f64,
    pub success_rate_weight: f64,
    pub risk_weight: f64,
    pub freshness_weight: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            profit_weight: 0.30,
            liquidity_weight: 0.25,
            latency_weight: 0.15,
            success_rate_weight: 0.15,
            risk_weight: 0.10,
            freshness_weight: 0.05,
        }
    }
}

/// 性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMetric {
    pub timestamp: DateTime<Utc>,
    pub total_profit: f64,
    pub success_rate: f64,
    pub avg_execution_time: f64,
    pub opportunities_processed: u64,
}

/// 路径失效追踪器
pub struct PathFailureTracker {
    /// 失效路径记录
    failed_paths: Arc<RwLock<HashMap<String, FailedPath>>>,
    /// 配置
    config: FailureTrackingConfig,
}

/// 失效路径信息
#[derive(Debug, Clone)]
pub struct FailedPath {
    pub path_id: String,
    pub failure_count: u32,
    pub last_failure: DateTime<Utc>,
    pub failure_reasons: Vec<String>,
    pub recovery_attempts: u32,
    pub blacklisted_until: Option<DateTime<Utc>>,
}

/// 失效追踪配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureTrackingConfig {
    /// 失败阈值
    pub failure_threshold: u32,
    /// 黑名单时长（秒）
    pub blacklist_duration_seconds: i64,
    /// 恢复尝试次数
    pub max_recovery_attempts: u32,
    /// 衰减因子
    pub decay_factor: f64,
}

impl Default for FailureTrackingConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 3,
            blacklist_duration_seconds: 1800, // 30分钟
            max_recovery_attempts: 5,
            decay_factor: 0.9,
        }
    }
}

/// 🚀 动态回测引擎
pub struct DynamicBacktestEngine {
    /// 回测配置
    config: BacktestConfig,
    /// 执行历史
    execution_history: Arc<RwLock<VecDeque<ExecutionRecord>>>,
    /// 权重优化器
    weight_optimizer: Arc<RwLock<WeightOptimizer>>,
    /// 路径失效追踪器
    path_tracker: Arc<PathFailureTracker>,
}

impl DynamicBacktestEngine {
    pub fn new(config: BacktestConfig) -> Self {
        Self {
            config,
            execution_history: Arc::new(RwLock::new(VecDeque::new())),
            weight_optimizer: Arc::new(RwLock::new(WeightOptimizer {
                current_weights: ScoreWeights::default(),
                performance_history: VecDeque::new(),
                best_weights: ScoreWeights::default(),
                best_performance: 0.0,
            })),
            path_tracker: Arc::new(PathFailureTracker {
                failed_paths: Arc::new(RwLock::new(HashMap::new())),
                config: FailureTrackingConfig::default(),
            }),
        }
    }

    /// 记录执行结果
    pub async fn record_execution(&self, record: ExecutionRecord) -> Result<(), StrategyError> {
        let mut history = self.execution_history.write().await;
        
        // 维护窗口大小
        let cutoff_time = Utc::now() - Duration::seconds(self.config.window_size_seconds);
        while history.front().map_or(false, |r| r.timestamp < cutoff_time) {
            history.pop_front();
        }
        
        // 添加新记录
        history.push_back(record.clone());
        
        // 更新路径失效信息
        if !record.success {
            self.path_tracker.track_failure(&record).await?;
        } else {
            self.path_tracker.track_success(&record).await?;
        }
        
        Ok(())
    }

    /// 执行动态回测
    pub async fn run_backtest(&self) -> Result<BacktestResult, StrategyError> {
        let history = self.execution_history.read().await;
        
        if history.len() < self.config.min_sample_size {
            return Err(StrategyError::InsufficientData(
                format!("需要至少 {} 个样本，当前只有 {} 个", 
                    self.config.min_sample_size, history.len())
            ));
        }
        
        // 计算性能指标
        let mut total_profit = 0.0;
        let mut successful_count = 0;
        let mut total_execution_time = 0;
        let opportunities_processed = history.len() as u64;
        
        for record in history.iter() {
            total_profit += record.actual_profit;
            if record.success {
                successful_count += 1;
            }
            total_execution_time += record.execution_time_ms as i64;
        }
        
        let success_rate = successful_count as f64 / opportunities_processed as f64;
        let avg_execution_time = total_execution_time as f64 / opportunities_processed as f64;
        
        let performance = PerformanceMetric {
            timestamp: Utc::now(),
            total_profit,
            success_rate,
            avg_execution_time,
            opportunities_processed,
        };
        
        // 优化权重
        self.optimize_weights(&performance).await?;
        
        Ok(BacktestResult {
            performance,
            optimized_weights: self.weight_optimizer.read().await.current_weights.clone(),
            failed_paths: self.path_tracker.get_failed_paths().await,
            recommendations: self.generate_recommendations(&performance).await,
        })
    }

    /// 优化权重
    async fn optimize_weights(&self, performance: &PerformanceMetric) -> Result<(), StrategyError> {
        let mut optimizer = self.weight_optimizer.write().await;
        
        // 添加到历史
        optimizer.performance_history.push_back(performance.clone());
        if optimizer.performance_history.len() > self.config.max_performance_history {
            optimizer.performance_history.pop_front();
        }
        
        // 计算综合性能得分
        let performance_score = self.calculate_performance_score(performance);
        
        // 如果性能提升，保存当前权重
        if performance_score > optimizer.best_performance {
            optimizer.best_weights = optimizer.current_weights.clone();
            optimizer.best_performance = performance_score;
        }
        
        // 基于梯度下降调整权重
        if optimizer.performance_history.len() >= 2 {
            let prev_performance = &optimizer.performance_history[optimizer.performance_history.len() - 2];
            let gradient = self.calculate_gradient(performance, prev_performance);
            
            // 调整权重
            self.adjust_weights(&mut optimizer.current_weights, &gradient);
        }
        
        Ok(())
    }

    /// 计算性能得分
    fn calculate_performance_score(&self, performance: &PerformanceMetric) -> f64 {
        // 综合考虑利润、成功率和执行时间
        let profit_score = performance.total_profit.max(0.0);
        let success_score = performance.success_rate * self.config.success_rate_score_multiplier;
        let speed_score = (self.config.speed_score_baseline_ms / performance.avg_execution_time.max(1.0)) 
            * self.config.speed_score_multiplier;
        
        profit_score * 0.5 + success_score * 0.3 + speed_score * 0.2
    }

    /// 计算梯度
    fn calculate_gradient(&self, current: &PerformanceMetric, previous: &PerformanceMetric) -> ScoreWeights {
        let profit_diff = current.total_profit - previous.total_profit;
        let success_diff = current.success_rate - previous.success_rate;
        let speed_diff = previous.avg_execution_time - current.avg_execution_time;
        
        ScoreWeights {
            profit_weight: profit_diff.signum() * self.config.adjustment_step,
            liquidity_weight: 0.0, // 保持稳定
            latency_weight: speed_diff.signum() * self.config.adjustment_step,
            success_rate_weight: success_diff.signum() * self.config.adjustment_step,
            risk_weight: -profit_diff.signum() * self.config.adjustment_step * 0.5,
            freshness_weight: 0.0, // 保持稳定
        }
    }

    /// 调整权重
    fn adjust_weights(&self, weights: &mut ScoreWeights, gradient: &ScoreWeights) {
        // 应用梯度
        weights.profit_weight = (weights.profit_weight + gradient.profit_weight)
            .clamp(0.1, 0.5);
        weights.liquidity_weight = (weights.liquidity_weight + gradient.liquidity_weight)
            .clamp(0.15, 0.35);
        weights.latency_weight = (weights.latency_weight + gradient.latency_weight)
            .clamp(0.05, 0.25);
        weights.success_rate_weight = (weights.success_rate_weight + gradient.success_rate_weight)
            .clamp(0.1, 0.3);
        weights.risk_weight = (weights.risk_weight + gradient.risk_weight)
            .clamp(0.05, 0.2);
        weights.freshness_weight = (weights.freshness_weight + gradient.freshness_weight)
            .clamp(0.02, 0.1);
        
        // 归一化
        let total = weights.profit_weight + weights.liquidity_weight + 
                   weights.latency_weight + weights.success_rate_weight + 
                   weights.risk_weight + weights.freshness_weight;
        
        weights.profit_weight /= total;
        weights.liquidity_weight /= total;
        weights.latency_weight /= total;
        weights.success_rate_weight /= total;
        weights.risk_weight /= total;
        weights.freshness_weight /= total;
    }

    /// 生成优化建议
    async fn generate_recommendations(&self, performance: &PerformanceMetric) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if performance.success_rate < 0.7 {
            recommendations.push("成功率偏低，建议提高机会筛选标准".to_string());
        }
        
        if performance.avg_execution_time > 500.0 {
            recommendations.push("执行延迟过高，建议优化网络连接或使用更快的交易所API".to_string());
        }
        
        if performance.total_profit < 0.0 {
            recommendations.push("总体亏损，建议检查手续费计算和滑点控制".to_string());
        }
        
        let failed_paths = self.path_tracker.get_failed_paths().await;
        if failed_paths.len() > 5 {
            recommendations.push(format!("发现 {} 条失效路径，建议更新交易对配置", failed_paths.len()));
        }
        
        recommendations
    }

    /// 获取优化后的权重
    pub async fn get_optimized_weights(&self) -> ScoreWeights {
        self.weight_optimizer.read().await.current_weights.clone()
    }
}

impl PathFailureTracker {
    /// 追踪失败
    async fn track_failure(&self, record: &ExecutionRecord) -> Result<(), StrategyError> {
        let path_id = self.generate_path_id(&record.path_info);
        let mut failed_paths = self.failed_paths.write().await;
        
        let failed_path = failed_paths.entry(path_id.clone()).or_insert_with(|| FailedPath {
            path_id: path_id.clone(),
            failure_count: 0,
            last_failure: Utc::now(),
            failure_reasons: Vec::new(),
            recovery_attempts: 0,
            blacklisted_until: None,
        });
        
        failed_path.failure_count += 1;
        failed_path.last_failure = Utc::now();
        
        if let Some(reason) = &record.failure_reason {
            failed_path.failure_reasons.push(reason.clone());
            if failed_path.failure_reasons.len() > 10 {
                failed_path.failure_reasons.remove(0);
            }
        }
        
        // 检查是否需要加入黑名单
        if failed_path.failure_count >= self.config.failure_threshold {
            failed_path.blacklisted_until = Some(
                Utc::now() + Duration::seconds(self.config.blacklist_duration_seconds)
            );
            tracing::warn!("路径 {} 已加入黑名单", path_id);
        }
        
        Ok(())
    }

    /// 追踪成功
    async fn track_success(&self, record: &ExecutionRecord) -> Result<(), StrategyError> {
        let path_id = self.generate_path_id(&record.path_info);
        let mut failed_paths = self.failed_paths.write().await;
        
        if let Some(failed_path) = failed_paths.get_mut(&path_id) {
            // 应用衰减
            failed_path.failure_count = 
                (failed_path.failure_count as f64 * self.config.decay_factor) as u32;
            
            // 如果失败次数降到0，移除记录
            if failed_path.failure_count == 0 {
                failed_paths.remove(&path_id);
            }
        }
        
        Ok(())
    }

    /// 获取失效路径
    async fn get_failed_paths(&self) -> Vec<FailedPath> {
        let failed_paths = self.failed_paths.read().await;
        let now = Utc::now();
        
        failed_paths.values()
            .filter(|path| {
                // 过滤掉已经过期的黑名单
                path.blacklisted_until.map_or(true, |until| until > now)
            })
            .cloned()
            .collect()
    }

    /// 检查路径是否可用
    pub async fn is_path_available(&self, path_info: &PathInfo) -> bool {
        let path_id = self.generate_path_id(path_info);
        let failed_paths = self.failed_paths.read().await;
        
        if let Some(failed_path) = failed_paths.get(&path_id) {
            if let Some(blacklisted_until) = failed_path.blacklisted_until {
                return Utc::now() > blacklisted_until;
            }
        }
        
        true
    }

    /// 生成路径ID
    fn generate_path_id(&self, path_info: &PathInfo) -> String {
        format!("{}-{}", 
            path_info.exchanges.join("_"),
            path_info.symbols.join("_")
        )
    }
}

/// 回测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub performance: PerformanceMetric,
    pub optimized_weights: ScoreWeights,
    pub failed_paths: Vec<FailedPath>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dynamic_backtest() {
        let engine = DynamicBacktestEngine::new(BacktestConfig::default());
        
        // 模拟执行记录
        for i in 0..150 {
            let record = ExecutionRecord {
                opportunity_id: format!("opp_{}", i),
                strategy_type: "triangular".to_string(),
                timestamp: Utc::now() - Duration::seconds(i as i64 * 10),
                expected_profit: 0.002,
                actual_profit: if i % 3 == 0 { -0.001 } else { 0.0015 },
                execution_time_ms: 100 + (i % 50) as u64,
                success: i % 3 != 0,
                failure_reason: if i % 3 == 0 { Some("Slippage too high".to_string()) } else { None },
                path_info: PathInfo {
                    exchanges: vec!["binance".to_string(), "okx".to_string()],
                    symbols: vec!["BTC/USDT".to_string()],
                    execution_steps: vec![],
                },
            };
            
            engine.record_execution(record).await.unwrap();
        }
        
        // 运行回测
        let result = engine.run_backtest().await.unwrap();
        
        assert!(result.performance.opportunities_processed >= 100);
        assert!(!result.recommendations.is_empty());
        
        // 检查权重优化
        let weights = engine.get_optimized_weights().await;
        assert!((weights.profit_weight + weights.liquidity_weight + 
                weights.latency_weight + weights.success_rate_weight + 
                weights.risk_weight + weights.freshness_weight - 1.0).abs() < 0.001);
    }
} 