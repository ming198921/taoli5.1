use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use crate::strategy::core::{StrategyError, StrategyExecutionResult, StrategyType};

/// 故障统计信息
#[derive(Debug, Clone)]
pub struct FailureStatistics {
    pub total_failures: usize,
    pub auto_recoveries: usize,
    pub manual_reviews: usize,
    pub approved_recoveries: usize,
    pub active_strategies: usize,
    pub failed_strategies: usize,
    pub recovery_success_rate: f64,
    pub average_recovery_time_ms: f64,
}

/// 失效检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetectionConfig {
    /// 连续亏损阈值
    pub consecutive_loss_threshold: u32,
    
    /// 最大回撤阈值 (百分比)
    pub max_drawdown_threshold: f64,
    
    /// 夏普比率阈值
    pub sharpe_ratio_threshold: f64,
    
    /// 动态调整参数
    pub dynamic_adjustment: DynamicAdjustmentConfig,
    
    /// 自动恢复配置
    pub auto_recovery: AutoRecoveryConfig,
    
    /// 人工复核配置
    pub manual_review: ManualReviewConfig,
}

/// 动态调整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicAdjustmentConfig {
    /// 是否启用动态调整
    pub enabled: bool,
    
    /// 市场波动率影响因子
    pub volatility_factor: f64,
    
    /// 流动性影响因子
    pub liquidity_factor: f64,
    
    /// 调整频率 (小时)
    pub adjustment_frequency_hours: u32,
    
    /// 最大调整倍数
    pub max_adjustment_multiplier: f64,
}

/// 自动恢复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRecoveryConfig {
    /// 是否启用自动恢复
    pub enabled: bool,
    
    /// 恢复检查间隔 (分钟)
    pub check_interval_minutes: u32,
    
    /// 成功率恢复阈值
    pub success_rate_threshold: f64,
    
    /// 连续成功次数要求
    pub consecutive_success_required: u32,
    
    /// 恢复后观察期 (小时)
    pub observation_period_hours: u32,
}

/// 人工复核配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualReviewConfig {
    /// 是否需要人工复核
    pub required: bool,
    
    /// 复核超时时间 (小时)
    pub review_timeout_hours: u32,
    
    /// 紧急情况自动通过
    pub emergency_auto_approve: bool,
    
    /// 复核人员列表
    pub reviewers: Vec<String>,
}

impl Default for FailureDetectionConfig {
    fn default() -> Self {
        Self {
            consecutive_loss_threshold: 10,
            max_drawdown_threshold: 0.1, // 10%
            sharpe_ratio_threshold: 0.5,
            dynamic_adjustment: DynamicAdjustmentConfig {
                enabled: true,
                volatility_factor: 1.5,
                liquidity_factor: 1.2,
                adjustment_frequency_hours: 6,
                max_adjustment_multiplier: 2.0,
            },
            auto_recovery: AutoRecoveryConfig {
                enabled: true,
                check_interval_minutes: 30,
                success_rate_threshold: 0.8,
                consecutive_success_required: 5,
                observation_period_hours: 24,
            },
            manual_review: ManualReviewConfig {
                required: true,
                review_timeout_hours: 2,
                emergency_auto_approve: true,
                reviewers: vec!["admin".to_string(), "risk_manager".to_string()],
            },
        }
    }
}

/// 策略状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyStatus {
    Active,
    Paused,
    Failed,
    UnderReview,
    Recovery,
    Disabled,
}

/// 失效原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureReason {
    ConsecutiveLosses {
        count: u32,
        threshold: u32,
    },
    MaxDrawdownExceeded {
        current_drawdown: f64,
        threshold: f64,
    },
    LowSharpeRatio {
        current_ratio: f64,
        threshold: f64,
    },
    ManualDisable {
        operator: String,
        reason: String,
    },
    SystemError {
        error_type: String,
        details: String,
    },
}

/// 策略性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformanceMetrics {
    pub strategy_id: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub consecutive_losses: u32,
    pub consecutive_wins: u32,
    pub avg_execution_time_ms: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl Default for StrategyPerformanceMetrics {
    fn default() -> Self {
        Self {
            strategy_id: String::new(),
            total_executions: 0,
            successful_executions: 0,
            total_profit: 0.0,
            total_loss: 0.0,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            consecutive_losses: 0,
            consecutive_wins: 0,
            avg_execution_time_ms: 0.0,
            last_execution: None,
            last_updated: Utc::now(),
        }
    }
}

/// 失效检测记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetectionRecord {
    pub record_id: String,
    pub strategy_id: String,
    pub detection_timestamp: DateTime<Utc>,
    pub failure_reason: FailureReason,
    pub performance_snapshot: StrategyPerformanceMetrics,
    pub auto_recovery_enabled: bool,
    pub manual_review_required: bool,
    pub status: FailureDetectionStatus,
}

/// 失效检测状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureDetectionStatus {
    Detected,
    UnderReview,
    Approved,
    Rejected,
    AutoRecovered,
}

/// 人工复核记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualReviewRecord {
    pub review_id: String,
    pub failure_record_id: String,
    pub reviewer: String,
    pub review_timestamp: DateTime<Utc>,
    pub decision: ReviewDecision,
    pub comments: String,
    pub recovery_plan: Option<RecoveryPlan>,
}

/// 复核决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewDecision {
    Approve,        // 批准恢复
    Reject,         // 拒绝恢复
    Modify,         // 修改策略后恢复
    Investigate,    // 需要进一步调查
}

/// 恢复计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub plan_id: String,
    pub strategy_modifications: HashMap<String, serde_json::Value>,
    pub monitoring_period_hours: u32,
    pub success_criteria: SuccessCriteria,
    pub rollback_conditions: Vec<String>,
}

/// 成功标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub min_success_rate: f64,
    pub min_consecutive_successes: u32,
    pub max_drawdown_limit: f64,
    pub min_sharpe_ratio: f64,
}

/// 策略失效检测器
pub struct StrategyFailureDetector {
    /// 配置
    config: Arc<RwLock<FailureDetectionConfig>>,
    
    /// 策略性能指标
    performance_metrics: Arc<RwLock<HashMap<String, StrategyPerformanceMetrics>>>,
    
    /// 策略状态
    strategy_status: Arc<RwLock<HashMap<String, StrategyStatus>>>,
    
    /// 执行历史记录 (用于计算指标)
    execution_history: Arc<RwLock<HashMap<String, VecDeque<StrategyExecutionResult>>>>,
    
    /// 失效检测记录
    failure_records: Arc<RwLock<Vec<FailureDetectionRecord>>>,
    
    /// 人工复核记录
    review_records: Arc<RwLock<Vec<ManualReviewRecord>>>,
    
    /// 恢复计划
    recovery_plans: Arc<RwLock<HashMap<String, RecoveryPlan>>>,
    
    /// 最后检查时间
    last_check_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl StrategyFailureDetector {
    /// 创建新的失效检测器
    pub fn new(config: Option<FailureDetectionConfig>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config.unwrap_or_default())),
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            strategy_status: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(HashMap::new())),
            failure_records: Arc::new(RwLock::new(Vec::new())),
            review_records: Arc::new(RwLock::new(Vec::new())),
            recovery_plans: Arc::new(RwLock::new(HashMap::new())),
            last_check_time: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录策略执行结果
    pub async fn record_execution(&self, result: StrategyExecutionResult) -> Result<(), StrategyError> {
        // 更新执行历史
        {
            let mut history = self.execution_history.write().await;
            let strategy_history = history.entry(result.strategy_id.clone()).or_insert_with(VecDeque::new);
            strategy_history.push_back(result.clone());
            
            // 保留最近1000条记录
            if strategy_history.len() > 1000 {
                strategy_history.pop_front();
            }
        }

        // 更新性能指标
        self.update_performance_metrics(&result).await?;
        
        // 检查失效条件
        self.check_failure_conditions(&result.strategy_id).await?;

        Ok(())
    }

    /// 获取策略状态
    pub async fn get_strategy_status(&self, strategy_id: &str) -> StrategyStatus {
        let status_map = self.strategy_status.read().await;
        status_map.get(strategy_id).copied().unwrap_or(StrategyStatus::Active)
    }

    /// 获取策略性能指标
    pub async fn get_performance_metrics(&self, strategy_id: &str) -> Option<StrategyPerformanceMetrics> {
        let metrics = self.performance_metrics.read().await;
        metrics.get(strategy_id).cloned()
    }

    /// 手动禁用策略
    pub async fn disable_strategy_manual(
        &self,
        strategy_id: &str,
        operator: String,
        reason: String,
    ) -> Result<String, StrategyError> {
        // 更新状态
        {
            let mut status_map = self.strategy_status.write().await;
            status_map.insert(strategy_id.to_string(), StrategyStatus::Disabled);
        }

        // 创建失效记录
        let performance_snapshot = self.get_performance_metrics(strategy_id).await
            .unwrap_or_default();

        let failure_record = FailureDetectionRecord {
            record_id: uuid::Uuid::new_v4().to_string(),
            strategy_id: strategy_id.to_string(),
            detection_timestamp: Utc::now(),
            failure_reason: FailureReason::ManualDisable { operator, reason },
            performance_snapshot,
            auto_recovery_enabled: false,
            manual_review_required: false,
            status: FailureDetectionStatus::Approved,
        };

        {
            let mut records = self.failure_records.write().await;
            records.push(failure_record.clone());
        }

        tracing::warn!(
            strategy_id = %strategy_id,
            record_id = %failure_record.record_id,
            "Strategy manually disabled"
        );

        Ok(failure_record.record_id)
    }

    /// 提交人工复核
    pub async fn submit_manual_review(
        &self,
        failure_record_id: &str,
        reviewer: String,
        decision: ReviewDecision,
        comments: String,
        recovery_plan: Option<RecoveryPlan>,
    ) -> Result<String, StrategyError> {
        let review_record = ManualReviewRecord {
            review_id: uuid::Uuid::new_v4().to_string(),
            failure_record_id: failure_record_id.to_string(),
            reviewer,
            review_timestamp: Utc::now(),
            decision: decision.clone(),
            comments,
            recovery_plan: recovery_plan.clone(),
        };

        // 更新失效记录状态
        {
            let mut records = self.failure_records.write().await;
            if let Some(failure_record) = records.iter_mut().find(|r| r.record_id == failure_record_id) {
                failure_record.status = match decision {
                    ReviewDecision::Approve => FailureDetectionStatus::Approved,
                    ReviewDecision::Reject => FailureDetectionStatus::Rejected,
                    _ => FailureDetectionStatus::UnderReview,
                };
            }
        }

        // 如果批准恢复，更新策略状态
        if matches!(decision, ReviewDecision::Approve) {
            if let Some(failure_record) = self.failure_records.read().await
                .iter().find(|r| r.record_id == failure_record_id) {
                
                {
                    let mut status_map = self.strategy_status.write().await;
                    status_map.insert(failure_record.strategy_id.clone(), StrategyStatus::Recovery);
                }

                // 保存恢复计划
                if let Some(plan) = recovery_plan {
                    let mut plans = self.recovery_plans.write().await;
                    plans.insert(failure_record.strategy_id.clone(), plan);
                }
            }
        }

        // 保存复核记录
        {
            let mut reviews = self.review_records.write().await;
            reviews.push(review_record.clone());
        }

        tracing::info!(
            review_id = %review_record.review_id,
            failure_record_id = %failure_record_id,
            reviewer = %review_record.reviewer,
            decision = ?decision,
            "Manual review submitted"
        );

        Ok(review_record.review_id)
    }

    /// 检查自动恢复条件
    pub async fn check_auto_recovery(&self) -> Result<Vec<String>, StrategyError> {
        let config = self.config.read().await;
        if !config.auto_recovery.enabled {
            return Ok(Vec::new());
        }

        let mut recovered_strategies = Vec::new();
        let status_map = self.strategy_status.read().await;
        let metrics_map = self.performance_metrics.read().await;

        for (strategy_id, &status) in status_map.iter() {
            if status == StrategyStatus::Failed || status == StrategyStatus::Recovery {
                if let Some(metrics) = metrics_map.get(strategy_id) {
                    if self.meets_recovery_criteria(metrics, &config.auto_recovery).await {
                        recovered_strategies.push(strategy_id.clone());
                    }
                }
            }
        }

        // 应用自动恢复
        for strategy_id in &recovered_strategies {
            self.apply_auto_recovery(strategy_id).await?;
        }

        Ok(recovered_strategies)
    }

    /// 获取失效检测统计
    pub async fn get_failure_statistics(&self) -> FailureStatistics {
        let records = self.failure_records.read().await;
        let reviews = self.review_records.read().await;
        let status_map = self.strategy_status.read().await;

        let total_failures = records.len();
        let auto_recoveries = records.iter()
            .filter(|r| matches!(r.status, FailureDetectionStatus::AutoRecovered))
            .count();
        
        let manual_reviews = reviews.len();
        let approved_recoveries = reviews.iter()
            .filter(|r| matches!(r.decision, ReviewDecision::Approve))
            .count();

        let active_strategies = status_map.values()
            .filter(|&&status| status == StrategyStatus::Active)
            .count();
        
        let failed_strategies = status_map.values()
            .filter(|&&status| status == StrategyStatus::Failed)
            .count();

        FailureStatistics {
            total_failures,
            auto_recoveries,
            manual_reviews,
            approved_recoveries,
            active_strategies,
            failed_strategies,
            recovery_success_rate: if total_failures > 0 {
                (auto_recoveries + approved_recoveries) as f64 / total_failures as f64
            } else {
                0.0
            },
            average_recovery_time_ms: self.calculate_average_recovery_time().await,
        }
    }

    /// 更新失效检测配置
    pub async fn update_config(&self, new_config: FailureDetectionConfig) -> Result<(), StrategyError> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        tracing::info!("Failure detection configuration updated");
        Ok(())
    }

    /// 更新性能指标
    async fn update_performance_metrics(&self, result: &StrategyExecutionResult) -> Result<(), StrategyError> {
        let mut metrics_map = self.performance_metrics.write().await;
        let metrics = metrics_map.entry(result.strategy_id.clone())
            .or_insert_with(|| StrategyPerformanceMetrics {
                strategy_id: result.strategy_id.clone(),
                ..Default::default()
            });

        // 更新基本统计
        metrics.total_executions += 1;
        if result.success {
            metrics.successful_executions += 1;
            metrics.consecutive_wins += 1;
            metrics.consecutive_losses = 0;
        } else {
            metrics.consecutive_losses += 1;
            metrics.consecutive_wins = 0;
        }

        // 更新盈亏
        if result.profit_realized > 0.0 {
            metrics.total_profit += result.profit_realized;
        } else {
            metrics.total_loss += result.profit_realized.abs();
        }

        // 更新胜率
        metrics.win_rate = metrics.successful_executions as f64 / metrics.total_executions as f64;

        // 更新回撤
        metrics.current_drawdown = self.calculate_current_drawdown(&result.strategy_id).await;
        metrics.max_drawdown = metrics.max_drawdown.max(metrics.current_drawdown);

        // 更新夏普比率
        metrics.sharpe_ratio = self.calculate_sharpe_ratio(&result.strategy_id).await;

        // 更新平均执行时间
        let total_time = metrics.avg_execution_time_ms * (metrics.total_executions - 1) as f64 + result.execution_time_ms as f64;
        metrics.avg_execution_time_ms = total_time / metrics.total_executions as f64;

        metrics.last_execution = Some(result.timestamp);
        metrics.last_updated = Utc::now();

        Ok(())
    }

    /// 检查失效条件
    async fn check_failure_conditions(&self, strategy_id: &str) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let metrics_opt = {
            let metrics_map = self.performance_metrics.read().await;
            metrics_map.get(strategy_id).cloned()
        };

        if let Some(metrics) = metrics_opt {
            let mut failure_reason = None;

            // 检查连续亏损
            if metrics.consecutive_losses >= config.consecutive_loss_threshold {
                failure_reason = Some(FailureReason::ConsecutiveLosses {
                    count: metrics.consecutive_losses,
                    threshold: config.consecutive_loss_threshold,
                });
            }

            // 检查最大回撤
            if metrics.current_drawdown > config.max_drawdown_threshold {
                failure_reason = Some(FailureReason::MaxDrawdownExceeded {
                    current_drawdown: metrics.current_drawdown,
                    threshold: config.max_drawdown_threshold,
                });
            }

            // 检查夏普比率
            if metrics.total_executions > 20 && metrics.sharpe_ratio < config.sharpe_ratio_threshold {
                failure_reason = Some(FailureReason::LowSharpeRatio {
                    current_ratio: metrics.sharpe_ratio,
                    threshold: config.sharpe_ratio_threshold,
                });
            }

            if let Some(reason) = failure_reason {
                self.trigger_failure_detection(strategy_id, reason, metrics).await?;
            }
        }

        Ok(())
    }

    /// 触发失效检测
    async fn trigger_failure_detection(
        &self,
        strategy_id: &str,
        reason: FailureReason,
        metrics: StrategyPerformanceMetrics,
    ) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        
        // 更新策略状态
        {
            let mut status_map = self.strategy_status.write().await;
            status_map.insert(strategy_id.to_string(), StrategyStatus::Failed);
        }

        // 创建失效记录
        let failure_record = FailureDetectionRecord {
            record_id: uuid::Uuid::new_v4().to_string(),
            strategy_id: strategy_id.to_string(),
            detection_timestamp: Utc::now(),
            failure_reason: reason,
            performance_snapshot: metrics,
            auto_recovery_enabled: config.auto_recovery.enabled,
            manual_review_required: config.manual_review.required,
            status: if config.manual_review.required {
                FailureDetectionStatus::UnderReview
            } else {
                FailureDetectionStatus::Detected
            },
        };

        {
            let mut records = self.failure_records.write().await;
            records.push(failure_record.clone());
        }

        tracing::warn!(
            strategy_id = %strategy_id,
            record_id = %failure_record.record_id,
            reason = ?failure_record.failure_reason,
            "Strategy failure detected"
        );

        Ok(())
    }

    /// 检查是否满足恢复条件
    async fn meets_recovery_criteria(
        &self,
        metrics: &StrategyPerformanceMetrics,
        recovery_config: &AutoRecoveryConfig,
    ) -> bool {
        metrics.win_rate >= recovery_config.success_rate_threshold
            && metrics.consecutive_wins >= recovery_config.consecutive_success_required
            && metrics.current_drawdown < 0.05 // 5%以下回撤
    }

    /// 应用自动恢复
    async fn apply_auto_recovery(&self, strategy_id: &str) -> Result<(), StrategyError> {
        {
            let mut status_map = self.strategy_status.write().await;
            status_map.insert(strategy_id.to_string(), StrategyStatus::Active);
        }

        // 更新失效记录状态
        {
            let mut records = self.failure_records.write().await;
            if let Some(record) = records.iter_mut()
                .filter(|r| r.strategy_id == strategy_id)
                .last() {
                record.status = FailureDetectionStatus::AutoRecovered;
            }
        }

        tracing::info!(
            strategy_id = %strategy_id,
            "Strategy auto-recovered"
        );

        Ok(())
    }

    /// 计算当前回撤
    async fn calculate_current_drawdown(&self, strategy_id: &str) -> f64 {
        let history = self.execution_history.read().await;
        if let Some(executions) = history.get(strategy_id) {
            if executions.is_empty() {
                return 0.0;
            }

            let mut peak = 0.0;
            let mut current_value = 0.0;
            let mut max_drawdown: f64 = 0.0;

            for execution in executions {
                current_value += execution.profit_realized;
                if current_value > peak {
                    peak = current_value;
                }
                let drawdown = (peak - current_value) / peak.max(1.0);
                max_drawdown = max_drawdown.max(drawdown);
            }

            max_drawdown
        } else {
            0.0
        }
    }

    /// 计算夏普比率
    async fn calculate_sharpe_ratio(&self, strategy_id: &str) -> f64 {
        let history = self.execution_history.read().await;
        if let Some(executions) = history.get(strategy_id) {
            if executions.len() < 10 {
                return 0.0;
            }

            let returns: Vec<f64> = executions.iter().map(|e| e.profit_realized).collect();
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
            
            if returns.len() < 2 {
                return 0.0;
            }

            let variance = returns.iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>() / (returns.len() - 1) as f64;
            
            let std_dev = variance.sqrt();
            
            if std_dev > 0.0 {
                mean_return / std_dev
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}


impl Default for StrategyFailureDetector {
    fn default() -> Self {
        Self::new(None)
    }
}






use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use crate::strategy::core::{StrategyError, StrategyExecutionResult, StrategyType};

/// 故障统计信息
#[derive(Debug, Clone)]
pub struct FailureStatistics {
    pub total_failures: usize,
    pub auto_recoveries: usize,
    pub manual_reviews: usize,
    pub approved_recoveries: usize,
    pub active_strategies: usize,
    pub failed_strategies: usize,
    pub recovery_success_rate: f64,
    pub average_recovery_time_ms: f64,
}

/// 失效检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
        }
    }
}

/// 策略状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyStatus {
    Active,
    Paused,
    Failed,
    UnderReview,
    Recovery,
    Disabled,
}

/// 失效原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureReason {
    ConsecutiveLosses {
        count: u32,
        threshold: u32,
    },
    MaxDrawdownExceeded {
        current_drawdown: f64,
        threshold: f64,
    },
    LowSharpeRatio {
        current_ratio: f64,
        threshold: f64,
    },
    ManualDisable {
        operator: String,
        reason: String,
    },
    SystemError {
        error_type: String,
        details: String,
    },
}

/// 策略性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformanceMetrics {
    pub strategy_id: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub consecutive_losses: u32,
    pub consecutive_wins: u32,
    pub avg_execution_time_ms: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
}

impl Default for StrategyPerformanceMetrics {
    fn default() -> Self {
        Self {
            strategy_id: String::new(),
            total_executions: 0,
            successful_executions: 0,
            total_profit: 0.0,
            total_loss: 0.0,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
            sharpe_ratio: 0.0,
            win_rate: 0.0,
            consecutive_losses: 0,
            consecutive_wins: 0,
            avg_execution_time_ms: 0.0,
            last_execution: None,
            last_updated: Utc::now(),
        }
    }
}

/// 失效检测记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureDetectionRecord {
    pub record_id: String,
    pub strategy_id: String,
    pub detection_timestamp: DateTime<Utc>,
    pub failure_reason: FailureReason,
    pub performance_snapshot: StrategyPerformanceMetrics,
    pub auto_recovery_enabled: bool,
    pub manual_review_required: bool,
    pub status: FailureDetectionStatus,
}

/// 失效检测状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureDetectionStatus {
    Detected,
    UnderReview,
    Approved,
    Rejected,
    AutoRecovered,
}

/// 人工复核记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualReviewRecord {
    pub review_id: String,
    pub failure_record_id: String,
    pub reviewer: String,
    pub review_timestamp: DateTime<Utc>,
    pub decision: ReviewDecision,
    pub comments: String,
    pub recovery_plan: Option<RecoveryPlan>,
}

/// 复核决策
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewDecision {
    Approve,        // 批准恢复
    Reject,         // 拒绝恢复
    Modify,         // 修改策略后恢复
    Investigate,    // 需要进一步调查
}

/// 恢复计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPlan {
    pub plan_id: String,
    pub strategy_modifications: HashMap<String, serde_json::Value>,
    pub monitoring_period_hours: u32,
    pub success_criteria: SuccessCriteria,
    pub rollback_conditions: Vec<String>,
}

/// 成功标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub min_success_rate: f64,
    pub min_consecutive_successes: u32,
    pub max_drawdown_limit: f64,
    pub min_sharpe_ratio: f64,
}

/// 策略失效检测器
pub struct StrategyFailureDetector {
    /// 配置
    config: Arc<RwLock<FailureDetectionConfig>>,
    
    /// 策略性能指标
    performance_metrics: Arc<RwLock<HashMap<String, StrategyPerformanceMetrics>>>,
    
    /// 策略状态
    strategy_status: Arc<RwLock<HashMap<String, StrategyStatus>>>,
    
    /// 执行历史记录 (用于计算指标)
    execution_history: Arc<RwLock<HashMap<String, VecDeque<StrategyExecutionResult>>>>,
    
    /// 失效检测记录
    failure_records: Arc<RwLock<Vec<FailureDetectionRecord>>>,
    
    /// 人工复核记录
    review_records: Arc<RwLock<Vec<ManualReviewRecord>>>,
    
    /// 恢复计划
    recovery_plans: Arc<RwLock<HashMap<String, RecoveryPlan>>>,
    
    /// 最后检查时间
    last_check_time: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl StrategyFailureDetector {
    /// 创建新的失效检测器
    pub fn new(config: Option<FailureDetectionConfig>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config.unwrap_or_default())),
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            strategy_status: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(HashMap::new())),
            failure_records: Arc::new(RwLock::new(Vec::new())),
            review_records: Arc::new(RwLock::new(Vec::new())),
            recovery_plans: Arc::new(RwLock::new(HashMap::new())),
            last_check_time: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 记录策略执行结果
    pub async fn record_execution(&self, result: StrategyExecutionResult) -> Result<(), StrategyError> {
        // 更新执行历史
        {
            let mut history = self.execution_history.write().await;
            let strategy_history = history.entry(result.strategy_id.clone()).or_insert_with(VecDeque::new);
            strategy_history.push_back(result.clone());
            
            // 保留最近1000条记录
            if strategy_history.len() > 1000 {
                strategy_history.pop_front();
            }
        }

        // 更新性能指标
        self.update_performance_metrics(&result).await?;
        
        // 检查失效条件
        self.check_failure_conditions(&result.strategy_id).await?;

        Ok(())
    }

    /// 获取策略状态
    pub async fn get_strategy_status(&self, strategy_id: &str) -> StrategyStatus {
        let status_map = self.strategy_status.read().await;
        status_map.get(strategy_id).copied().unwrap_or(StrategyStatus::Active)
    }

    /// 获取策略性能指标
    pub async fn get_performance_metrics(&self, strategy_id: &str) -> Option<StrategyPerformanceMetrics> {
        let metrics = self.performance_metrics.read().await;
        metrics.get(strategy_id).cloned()
    }

    /// 手动禁用策略
    pub async fn disable_strategy_manual(
        &self,
        strategy_id: &str,
        operator: String,
        reason: String,
    ) -> Result<String, StrategyError> {
        // 更新状态
        {
            let mut status_map = self.strategy_status.write().await;
            status_map.insert(strategy_id.to_string(), StrategyStatus::Disabled);
        }

        // 创建失效记录
        let performance_snapshot = self.get_performance_metrics(strategy_id).await
            .unwrap_or_default();

        let failure_record = FailureDetectionRecord {
            record_id: uuid::Uuid::new_v4().to_string(),
            strategy_id: strategy_id.to_string(),
            detection_timestamp: Utc::now(),
            failure_reason: FailureReason::ManualDisable { operator, reason },
            performance_snapshot,
            auto_recovery_enabled: false,
            manual_review_required: false,
            status: FailureDetectionStatus::Approved,
        };

        {
            let mut records = self.failure_records.write().await;
            records.push(failure_record.clone());
        }

        tracing::warn!(
            strategy_id = %strategy_id,
            record_id = %failure_record.record_id,
            "Strategy manually disabled"
        );

        Ok(failure_record.record_id)
    }

    /// 提交人工复核
    pub async fn submit_manual_review(
        &self,
        failure_record_id: &str,
        reviewer: String,
        decision: ReviewDecision,
        comments: String,
        recovery_plan: Option<RecoveryPlan>,
    ) -> Result<String, StrategyError> {
        let review_record = ManualReviewRecord {
            review_id: uuid::Uuid::new_v4().to_string(),
            failure_record_id: failure_record_id.to_string(),
            reviewer,
            review_timestamp: Utc::now(),
            decision: decision.clone(),
            comments,
            recovery_plan: recovery_plan.clone(),
        };

        // 更新失效记录状态
        {
            let mut records = self.failure_records.write().await;
            if let Some(failure_record) = records.iter_mut().find(|r| r.record_id == failure_record_id) {
                failure_record.status = match decision {
                    ReviewDecision::Approve => FailureDetectionStatus::Approved,
                    ReviewDecision::Reject => FailureDetectionStatus::Rejected,
                    _ => FailureDetectionStatus::UnderReview,
                };
            }
        }

        // 如果批准恢复，更新策略状态
        if matches!(decision, ReviewDecision::Approve) {
            if let Some(failure_record) = self.failure_records.read().await
                .iter().find(|r| r.record_id == failure_record_id) {
                
                {
                    let mut status_map = self.strategy_status.write().await;
                    status_map.insert(failure_record.strategy_id.clone(), StrategyStatus::Recovery);
                }

                // 保存恢复计划
                if let Some(plan) = recovery_plan {
                    let mut plans = self.recovery_plans.write().await;
                    plans.insert(failure_record.strategy_id.clone(), plan);
                }
            }
        }

        // 保存复核记录
        {
            let mut reviews = self.review_records.write().await;
            reviews.push(review_record.clone());
        }

        tracing::info!(
            review_id = %review_record.review_id,
            failure_record_id = %failure_record_id,
            reviewer = %review_record.reviewer,
            decision = ?decision,
            "Manual review submitted"
        );

        Ok(review_record.review_id)
    }

    /// 检查自动恢复条件
    pub async fn check_auto_recovery(&self) -> Result<Vec<String>, StrategyError> {
        let config = self.config.read().await;
        if !config.auto_recovery.enabled {
            return Ok(Vec::new());
        }

        let mut recovered_strategies = Vec::new();
        let status_map = self.strategy_status.read().await;
        let metrics_map = self.performance_metrics.read().await;

        for (strategy_id, &status) in status_map.iter() {
            if status == StrategyStatus::Failed || status == StrategyStatus::Recovery {
                if let Some(metrics) = metrics_map.get(strategy_id) {
                    if self.meets_recovery_criteria(metrics, &config.auto_recovery).await {
                        recovered_strategies.push(strategy_id.clone());
                    }
                }
            }
        }

        // 应用自动恢复
        for strategy_id in &recovered_strategies {
            self.apply_auto_recovery(strategy_id).await?;
        }

        Ok(recovered_strategies)
    }

    /// 获取失效检测统计
    pub async fn get_failure_statistics(&self) -> FailureStatistics {
        let records = self.failure_records.read().await;
        let reviews = self.review_records.read().await;
        let status_map = self.strategy_status.read().await;

        let total_failures = records.len();
        let auto_recoveries = records.iter()
            .filter(|r| matches!(r.status, FailureDetectionStatus::AutoRecovered))
            .count();
        
        let manual_reviews = reviews.len();
        let approved_recoveries = reviews.iter()
            .filter(|r| matches!(r.decision, ReviewDecision::Approve))
            .count();

        let active_strategies = status_map.values()
            .filter(|&&status| status == StrategyStatus::Active)
            .count();
        
        let failed_strategies = status_map.values()
            .filter(|&&status| status == StrategyStatus::Failed)
            .count();

        FailureStatistics {
            total_failures,
            auto_recoveries,
            manual_reviews,
            approved_recoveries,
            active_strategies,
            failed_strategies,
            recovery_success_rate: if total_failures > 0 {
                (auto_recoveries + approved_recoveries) as f64 / total_failures as f64
            } else {
                0.0
            },
            average_recovery_time_ms: self.calculate_average_recovery_time().await,
        }
    }

    /// 更新失效检测配置
    pub async fn update_config(&self, new_config: FailureDetectionConfig) -> Result<(), StrategyError> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        tracing::info!("Failure detection configuration updated");
        Ok(())
    }

    /// 更新性能指标
    async fn update_performance_metrics(&self, result: &StrategyExecutionResult) -> Result<(), StrategyError> {
        let mut metrics_map = self.performance_metrics.write().await;
        let metrics = metrics_map.entry(result.strategy_id.clone())
            .or_insert_with(|| StrategyPerformanceMetrics {
                strategy_id: result.strategy_id.clone(),
                ..Default::default()
            });

        // 更新基本统计
        metrics.total_executions += 1;
        if result.success {
            metrics.successful_executions += 1;
            metrics.consecutive_wins += 1;
            metrics.consecutive_losses = 0;
        } else {
            metrics.consecutive_losses += 1;
            metrics.consecutive_wins = 0;
        }

        // 更新盈亏
        if result.profit_realized > 0.0 {
            metrics.total_profit += result.profit_realized;
        } else {
            metrics.total_loss += result.profit_realized.abs();
        }

        // 更新胜率
        metrics.win_rate = metrics.successful_executions as f64 / metrics.total_executions as f64;

        // 更新回撤
        metrics.current_drawdown = self.calculate_current_drawdown(&result.strategy_id).await;
        metrics.max_drawdown = metrics.max_drawdown.max(metrics.current_drawdown);

        // 更新夏普比率
        metrics.sharpe_ratio = self.calculate_sharpe_ratio(&result.strategy_id).await;

        // 更新平均执行时间
        let total_time = metrics.avg_execution_time_ms * (metrics.total_executions - 1) as f64 + result.execution_time_ms as f64;
        metrics.avg_execution_time_ms = total_time / metrics.total_executions as f64;

        metrics.last_execution = Some(result.timestamp);
        metrics.last_updated = Utc::now();

        Ok(())
    }

    /// 检查失效条件
    async fn check_failure_conditions(&self, strategy_id: &str) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let metrics_opt = {
            let metrics_map = self.performance_metrics.read().await;
            metrics_map.get(strategy_id).cloned()
        };

        if let Some(metrics) = metrics_opt {
            let mut failure_reason = None;

            // 检查连续亏损
            if metrics.consecutive_losses >= config.consecutive_loss_threshold {
                failure_reason = Some(FailureReason::ConsecutiveLosses {
                    count: metrics.consecutive_losses,
                    threshold: config.consecutive_loss_threshold,
                });
            }

            // 检查最大回撤
            if metrics.current_drawdown > config.max_drawdown_threshold {
                failure_reason = Some(FailureReason::MaxDrawdownExceeded {
                    current_drawdown: metrics.current_drawdown,
                    threshold: config.max_drawdown_threshold,
                });
            }

            // 检查夏普比率
            if metrics.total_executions > 20 && metrics.sharpe_ratio < config.sharpe_ratio_threshold {
                failure_reason = Some(FailureReason::LowSharpeRatio {
                    current_ratio: metrics.sharpe_ratio,
                    threshold: config.sharpe_ratio_threshold,
                });
            }

            if let Some(reason) = failure_reason {
                self.trigger_failure_detection(strategy_id, reason, metrics).await?;
            }
        }

        Ok(())
    }

    /// 触发失效检测
    async fn trigger_failure_detection(
        &self,
        strategy_id: &str,
        reason: FailureReason,
        metrics: StrategyPerformanceMetrics,
    ) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        
        // 更新策略状态
        {
            let mut status_map = self.strategy_status.write().await;
            status_map.insert(strategy_id.to_string(), StrategyStatus::Failed);
        }

        // 创建失效记录
        let failure_record = FailureDetectionRecord {
            record_id: uuid::Uuid::new_v4().to_string(),
            strategy_id: strategy_id.to_string(),
            detection_timestamp: Utc::now(),
            failure_reason: reason,
            performance_snapshot: metrics,
            auto_recovery_enabled: config.auto_recovery.enabled,
            manual_review_required: config.manual_review.required,
            status: if config.manual_review.required {
                FailureDetectionStatus::UnderReview
            } else {
                FailureDetectionStatus::Detected
            },
        };

        {
            let mut records = self.failure_records.write().await;
            records.push(failure_record.clone());
        }

        tracing::warn!(
            strategy_id = %strategy_id,
            record_id = %failure_record.record_id,
            reason = ?failure_record.failure_reason,
            "Strategy failure detected"
        );

        Ok(())
    }

    /// 检查是否满足恢复条件
    async fn meets_recovery_criteria(
        &self,
        metrics: &StrategyPerformanceMetrics,
        recovery_config: &AutoRecoveryConfig,
    ) -> bool {
        metrics.win_rate >= recovery_config.success_rate_threshold
            && metrics.consecutive_wins >= recovery_config.consecutive_success_required
            && metrics.current_drawdown < 0.05 // 5%以下回撤
    }

    /// 应用自动恢复
    async fn apply_auto_recovery(&self, strategy_id: &str) -> Result<(), StrategyError> {
        {
            let mut status_map = self.strategy_status.write().await;
            status_map.insert(strategy_id.to_string(), StrategyStatus::Active);
        }

        // 更新失效记录状态
        {
            let mut records = self.failure_records.write().await;
            if let Some(record) = records.iter_mut()
                .filter(|r| r.strategy_id == strategy_id)
                .last() {
                record.status = FailureDetectionStatus::AutoRecovered;
            }
        }

        tracing::info!(
            strategy_id = %strategy_id,
            "Strategy auto-recovered"
        );

        Ok(())
    }

    /// 计算当前回撤
    async fn calculate_current_drawdown(&self, strategy_id: &str) -> f64 {
        let history = self.execution_history.read().await;
        if let Some(executions) = history.get(strategy_id) {
            if executions.is_empty() {
                return 0.0;
            }

            let mut peak = 0.0;
            let mut current_value = 0.0;
            let mut max_drawdown: f64 = 0.0;

            for execution in executions {
                current_value += execution.profit_realized;
                if current_value > peak {
                    peak = current_value;
                }
                let drawdown = (peak - current_value) / peak.max(1.0);
                max_drawdown = max_drawdown.max(drawdown);
            }

            max_drawdown
        } else {
            0.0
        }
    }

    /// 计算夏普比率
    async fn calculate_sharpe_ratio(&self, strategy_id: &str) -> f64 {
        let history = self.execution_history.read().await;
        if let Some(executions) = history.get(strategy_id) {
            if executions.len() < 10 {
                return 0.0;
            }

            let returns: Vec<f64> = executions.iter().map(|e| e.profit_realized).collect();
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
            
            if returns.len() < 2 {
                return 0.0;
            }

            let variance = returns.iter()
                .map(|r| (r - mean_return).powi(2))
                .sum::<f64>() / (returns.len() - 1) as f64;
            
            let std_dev = variance.sqrt();
            
            if std_dev > 0.0 {
                mean_return / std_dev
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}


impl Default for StrategyFailureDetector {
    fn default() -> Self {
        Self::new(None)
    }
}





