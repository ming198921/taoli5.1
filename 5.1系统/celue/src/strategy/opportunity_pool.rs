use std::collections::{HashMap, BinaryHeap};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::cmp::Ordering;

use crate::strategy::core::{
    ArbitrageOpportunityCore, OpportunityEvaluation, StrategyType, 
    OpportunityPriority, StrategyError
};

use crate::strategy::opportunity_pool_enhanced::{
    DynamicBacktestEngine, BacktestConfig, ExecutionRecord, PathInfo, 
    ExecutionStep, StepType, ScoreWeights
};

/// 机会池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityPoolConfig {
    pub max_opportunities: usize,
    pub expiry_seconds: i64,
    pub priority_weights: PriorityWeights,
    pub evaluation_criteria: EvaluationCriteria,
    pub auto_cleanup_interval: u64,
    /// 回测频率（每处理多少个机会运行一次）
    pub backtest_frequency: Option<u64>,
}

impl Default for OpportunityPoolConfig {
    fn default() -> Self {
        Self {
            max_opportunities: 1000,
            expiry_seconds: 30,
            priority_weights: PriorityWeights::default(),
            evaluation_criteria: EvaluationCriteria::default(),
            auto_cleanup_interval: 5,
            backtest_frequency: Some(100),
        }
    }
}

/// 优先级权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityWeights {
    pub profit_weight: f64,
    pub liquidity_weight: f64,
    pub risk_weight: f64,
    pub execution_speed_weight: f64,
    pub confidence_weight: f64,
    pub strategy_priority_weight: f64,
}

impl Default for PriorityWeights {
    fn default() -> Self {
        Self {
            profit_weight: 0.3,
            liquidity_weight: 0.25,
            risk_weight: 0.2,
            execution_speed_weight: 0.1,
            confidence_weight: 0.1,
            strategy_priority_weight: 0.05,
        }
    }
}

/// 评估标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCriteria {
    pub min_profit_threshold: f64,
    pub min_liquidity_score: f64,
    pub max_risk_score: f64,
    pub max_execution_delay_ms: u64,
    pub min_confidence_score: f64,
}

impl Default for EvaluationCriteria {
    fn default() -> Self {
        Self {
            min_profit_threshold: 0.001, // 0.1%
            min_liquidity_score: 0.5,
            max_risk_score: 0.7,
            max_execution_delay_ms: 1000,
            min_confidence_score: 0.6,
        }
    }
}

/// 带权重的机会包装器，用于优先级队列
#[derive(Debug, Clone)]
pub struct WeightedOpportunity {
    pub opportunity: ArbitrageOpportunityCore,
    pub evaluation: OpportunityEvaluation,
    pub weighted_score: f64,
    pub created_at: DateTime<Utc>,
}

impl PartialEq for WeightedOpportunity {
    fn eq(&self, other: &Self) -> bool {
        self.weighted_score == other.weighted_score
    }
}

impl Eq for WeightedOpportunity {}

impl PartialOrd for WeightedOpportunity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WeightedOpportunity {
    fn cmp(&self, other: &Self) -> Ordering {
        // 反转比较以实现最大堆
        other.weighted_score.partial_cmp(&self.weighted_score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

/// 全局套利机会池
pub struct GlobalOpportunityPool {
    /// 配置
    config: Arc<RwLock<OpportunityPoolConfig>>,
    /// 优先级队列（最大堆）
    priority_queue: Arc<RwLock<BinaryHeap<WeightedOpportunity>>>,
    /// 按ID索引的机会映射
    opportunity_map: Arc<RwLock<HashMap<String, WeightedOpportunity>>>,
    /// 按策略类型分组
    type_groups: Arc<RwLock<HashMap<StrategyType, Vec<String>>>>,
    /// 按交易所分组
    exchange_groups: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// 统计信息
    stats: Arc<RwLock<PoolStatistics>>,
    /// 🚀 动态回测引擎
    backtest_engine: Arc<DynamicBacktestEngine>,
}

/// 机会池统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub total_opportunities_received: u64,
    pub total_opportunities_processed: u64,
    pub current_pool_size: usize,
    pub opportunities_by_type: HashMap<StrategyType, usize>,
    pub opportunities_by_priority: HashMap<OpportunityPriority, usize>,
    pub avg_weighted_score: f64,
    pub last_cleanup: DateTime<Utc>,
    pub cleanup_removed_count: u64,
}

impl Default for PoolStatistics {
    fn default() -> Self {
        Self {
            total_opportunities_received: 0,
            total_opportunities_processed: 0,
            current_pool_size: 0,
            opportunities_by_type: HashMap::new(),
            opportunities_by_priority: HashMap::new(),
            avg_weighted_score: 0.0,
            last_cleanup: Utc::now(),
            cleanup_removed_count: 0,
        }
    }
}

impl GlobalOpportunityPool {
    /// 创建新的机会池
    pub fn new(config: OpportunityPoolConfig) -> Self {
        let backtest_config = BacktestConfig::default();
        
        Self {
            config: Arc::new(RwLock::new(config)),
            priority_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            opportunity_map: Arc::new(RwLock::new(HashMap::new())),
            type_groups: Arc::new(RwLock::new(HashMap::new())),
            exchange_groups: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PoolStatistics::default())),
            backtest_engine: Arc::new(DynamicBacktestEngine::new(backtest_config)),
        }
    }

    /// 添加机会到池中
    pub async fn add_opportunity(
        &self,
        opportunity: ArbitrageOpportunityCore,
        evaluation: OpportunityEvaluation,
    ) -> Result<(), StrategyError> {
        // 验证机会是否符合评估标准
        self.validate_opportunity(&opportunity, &evaluation).await?;

        // 计算加权得分
        let weighted_score = self.calculate_weighted_score(&evaluation).await;

        let weighted_opportunity = WeightedOpportunity {
            opportunity: opportunity.clone(),
            evaluation,
            weighted_score,
            created_at: Utc::now(),
        };

        // 添加到各个存储结构
        {
            let mut priority_queue = self.priority_queue.write().await;
            let mut opportunity_map = self.opportunity_map.write().await;
            let mut type_groups = self.type_groups.write().await;
            let mut exchange_groups = self.exchange_groups.write().await;
            let mut stats = self.stats.write().await;

            // 检查是否超过最大容量
            let config = self.config.read().await;
            if priority_queue.len() >= config.max_opportunities {
                // 移除最低优先级的机会
                self.remove_lowest_priority_internal(&mut priority_queue, &mut opportunity_map, 
                    &mut type_groups, &mut exchange_groups).await;
            }

            // 添加新机会
            priority_queue.push(weighted_opportunity.clone());
            opportunity_map.insert(opportunity.id.clone(), weighted_opportunity.clone());

            // 更新分组索引
            type_groups
                .entry(opportunity.strategy_type)
                .or_insert_with(Vec::new)
                .push(opportunity.id.clone());

            exchange_groups
                .entry(opportunity.buy_exchange.clone())
                .or_insert_with(Vec::new)
                .push(opportunity.id.clone());

            if opportunity.buy_exchange != opportunity.sell_exchange {
                exchange_groups
                    .entry(opportunity.sell_exchange.clone())
                    .or_insert_with(Vec::new)
                    .push(opportunity.id.clone());
            }

            // 更新统计信息
            stats.total_opportunities_received += 1;
            stats.current_pool_size = priority_queue.len();
            stats.opportunities_by_type
                .entry(opportunity.strategy_type)
                .and_modify(|count| *count += 1)
                .or_insert(1);
            stats.opportunities_by_priority
                .entry(weighted_opportunity.evaluation.priority)
                .and_modify(|count| *count += 1)
                .or_insert(1);

            // 重新计算平均得分
            self.recalculate_avg_score(&priority_queue, &mut stats).await;
        }

        tracing::debug!(
            opportunity_id = %opportunity.id,
            strategy_type = %opportunity.strategy_type,
            weighted_score = %weighted_score,
            "Opportunity added to pool"
        );

        Ok(())
    }

    /// 获取最高优先级的机会
    pub async fn get_best_opportunity(&self) -> Option<WeightedOpportunity> {
        let mut priority_queue = self.priority_queue.write().await;
        let mut opportunity_map = self.opportunity_map.write().await;
        let mut type_groups = self.type_groups.write().await;
        let mut exchange_groups = self.exchange_groups.write().await;
        let mut stats = self.stats.write().await;

        // 清理过期机会
        self.cleanup_expired_internal(&mut priority_queue, &mut opportunity_map, 
            &mut type_groups, &mut exchange_groups, &mut stats).await;

        if let Some(weighted_opportunity) = priority_queue.pop() {
            // 从其他数据结构中移除
            opportunity_map.remove(&weighted_opportunity.opportunity.id);
            self.remove_from_groups(&weighted_opportunity.opportunity, 
                &mut type_groups, &mut exchange_groups).await;

            stats.total_opportunities_processed += 1;
            stats.current_pool_size = priority_queue.len();
            
            // 更新统计计数
            if let Some(count) = stats.opportunities_by_type.get_mut(&weighted_opportunity.opportunity.strategy_type) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    stats.opportunities_by_type.remove(&weighted_opportunity.opportunity.strategy_type);
                }
            }
            
            if let Some(count) = stats.opportunities_by_priority.get_mut(&weighted_opportunity.evaluation.priority) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    stats.opportunities_by_priority.remove(&weighted_opportunity.evaluation.priority);
                }
            }

            // 重新计算平均得分
            self.recalculate_avg_score(&priority_queue, &mut stats).await;

            tracing::debug!(
                opportunity_id = %weighted_opportunity.opportunity.id,
                strategy_type = %weighted_opportunity.opportunity.strategy_type,
                weighted_score = %weighted_opportunity.weighted_score,
                "Opportunity extracted from pool"
            );

            Some(weighted_opportunity)
        } else {
            None
        }
    }

    /// 根据策略类型获取最佳机会
    pub async fn get_best_opportunity_by_type(&self, strategy_type: StrategyType) -> Option<WeightedOpportunity> {
        let type_groups = self.type_groups.read().await;
        let opportunity_map = self.opportunity_map.read().await;

        if let Some(opportunity_ids) = type_groups.get(&strategy_type) {
            let mut best_opportunity: Option<WeightedOpportunity> = None;
            let mut best_score = f64::NEG_INFINITY;

            for opportunity_id in opportunity_ids {
                if let Some(weighted_opportunity) = opportunity_map.get(opportunity_id) {
                    // 检查是否过期
                    if weighted_opportunity.opportunity.is_valid() {
                        if weighted_opportunity.weighted_score > best_score {
                            best_score = weighted_opportunity.weighted_score;
                            best_opportunity = Some(weighted_opportunity.clone());
                        }
                    }
                }
            }

            best_opportunity
        } else {
            None
        }
    }

    /// 获取池统计信息
    pub async fn get_statistics(&self) -> PoolStatistics {
        self.stats.read().await.clone()
    }

    /// 清理过期机会
    pub async fn cleanup_expired(&self) -> u64 {
        let mut priority_queue = self.priority_queue.write().await;
        let mut opportunity_map = self.opportunity_map.write().await;
        let mut type_groups = self.type_groups.write().await;
        let mut exchange_groups = self.exchange_groups.write().await;
        let mut stats = self.stats.write().await;

        let removed_count = self.cleanup_expired_internal(&mut priority_queue, &mut opportunity_map, 
            &mut type_groups, &mut exchange_groups, &mut stats).await;

        stats.last_cleanup = Utc::now();
        stats.cleanup_removed_count += removed_count;

        removed_count
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: OpportunityPoolConfig) -> Result<(), StrategyError> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        tracing::info!("Opportunity pool configuration updated");
        Ok(())
    }

    /// 获取当前池大小
    pub async fn size(&self) -> usize {
        self.priority_queue.read().await.len()
    }

    /// 验证机会是否符合标准
    async fn validate_opportunity(
        &self,
        opportunity: &ArbitrageOpportunityCore,
        evaluation: &OpportunityEvaluation,
    ) -> Result<(), StrategyError> {
        let config = self.config.read().await;
        let criteria = &config.evaluation_criteria;

        if !opportunity.is_valid() {
            return Err(StrategyError::InvalidOpportunity);
        }

        if evaluation.profit_estimate < criteria.min_profit_threshold {
            return Err(StrategyError::InvalidOpportunity);
        }

        if evaluation.liquidity_score < criteria.min_liquidity_score {
            return Err(StrategyError::InsufficientLiquidity);
        }

        if evaluation.risk_exposure > criteria.max_risk_score {
            return Err(StrategyError::RiskLimitExceeded);
        }

        if evaluation.execution_delay_estimate_ms > criteria.max_execution_delay_ms {
            return Err(StrategyError::ExecutionTimeout);
        }

        if evaluation.confidence_score < criteria.min_confidence_score {
            return Err(StrategyError::InvalidOpportunity);
        }

        Ok(())
    }

    /// 计算加权得分（使用动态优化的权重）
    async fn calculate_weighted_score(&self, evaluation: &OpportunityEvaluation) -> f64 {
        // 获取动态优化的权重
        let weights = self.backtest_engine.get_optimized_weights().await;
        
        // 归一化各个指标到 0-1 范围
        let profit_score = (evaluation.profit_estimate / 0.01).min(1.0); // 1% 利润 = 满分
        let liquidity_score = evaluation.liquidity_score;
        let latency_score = 1.0 - (evaluation.execution_delay_estimate_ms as f64 / 1000.0).min(1.0);
        let success_score = evaluation.confidence_score;
        let risk_score = 1.0 - evaluation.risk_exposure;
        let freshness_score = 1.0; // 新机会得分最高

        // 使用动态权重计算总分
        let weighted_score = 
            profit_score * weights.profit_weight +
            liquidity_score * weights.liquidity_weight +
            latency_score * weights.latency_weight +
            success_score * weights.success_rate_weight +
            risk_score * weights.risk_weight +
            freshness_score * weights.freshness_weight;

        // 应用优先级乘数
        let priority_multiplier = match evaluation.priority {
            OpportunityPriority::Critical => 2.0,
            OpportunityPriority::High => 1.5,
            OpportunityPriority::Medium => 1.0,
            OpportunityPriority::Low => 0.7,
        };

        weighted_score * priority_multiplier
    }

    /// 记录执行结果（供策略模块调用）
    pub async fn record_execution_result(
        &self,
        opportunity_id: &str,
        expected_profit: f64,
        actual_profit: f64,
        execution_time_ms: u64,
        success: bool,
        failure_reason: Option<String>,
        path_info: PathInfo,
    ) -> Result<(), StrategyError> {
        // 获取机会信息
        let opportunity_info = {
            let opportunity_map = self.opportunity_map.read().await;
            opportunity_map.get(opportunity_id).map(|w| (
                w.opportunity.strategy_type.clone(),
                w.opportunity.clone()
            ))
        };

        if let Some((strategy_type, _opportunity)) = opportunity_info {
            let record = ExecutionRecord {
                opportunity_id: opportunity_id.to_string(),
                strategy_type: strategy_type.to_string(),
                timestamp: Utc::now(),
                expected_profit,
                actual_profit,
                execution_time_ms,
                success,
                failure_reason,
                path_info,
            };

            // 记录到回测引擎
            self.backtest_engine.record_execution(record).await?;

            // 定期运行回测优化
            if self.should_run_backtest().await {
                tokio::spawn({
                    let backtest_engine = Arc::clone(&self.backtest_engine);
                    async move {
                        if let Err(e) = backtest_engine.run_backtest().await {
                            tracing::error!("回测优化失败: {}", e);
                        } else {
                            tracing::info!("回测优化完成，权重已更新");
                        }
                    }
                });
            }
        }

        Ok(())
    }

    /// 检查路径是否可用
    pub async fn is_path_available(&self, path_info: &PathInfo) -> bool {
        self.backtest_engine.is_path_available(path_info).await
    }

    /// 判断是否应该运行回测
    async fn should_run_backtest(&self) -> bool {
        let stats = self.stats.read().await;
        let config = self.config.read().await;
        // 根据配置决定回测频率
        let backtest_frequency = config.backtest_frequency.unwrap_or(100);
        stats.total_opportunities_processed % backtest_frequency == 0
    }

    /// 获取回测报告
    pub async fn get_backtest_report(&self) -> Result<serde_json::Value, StrategyError> {
        let result = self.backtest_engine.run_backtest().await?;
        Ok(serde_json::to_value(result).map_err(|e| StrategyError::Serialization(e.to_string()))?)
    }

    /// 内部清理过期机会
    async fn cleanup_expired_internal(
        &self,
        priority_queue: &mut BinaryHeap<WeightedOpportunity>,
        opportunity_map: &mut HashMap<String, WeightedOpportunity>,
        type_groups: &mut HashMap<StrategyType, Vec<String>>,
        exchange_groups: &mut HashMap<String, Vec<String>>,
        stats: &mut PoolStatistics,
    ) -> u64 {
        let mut removed_count = 0;
        let now = Utc::now();

        // 重建优先级队列，过滤掉过期的机会
        let valid_opportunities: Vec<WeightedOpportunity> = priority_queue
            .drain()
            .filter(|wo| {
                if wo.opportunity.valid_until > now {
                    true
                } else {
                    // 移除过期机会
                    opportunity_map.remove(&wo.opportunity.id);
                    self.remove_from_groups_sync(&wo.opportunity, type_groups, exchange_groups);
                    removed_count += 1;
                    false
                }
            })
            .collect();

        // 重新插入有效机会
        for opportunity in valid_opportunities {
            priority_queue.push(opportunity);
        }

        stats.current_pool_size = priority_queue.len();
        removed_count
    }

    /// 移除最低优先级机会
    async fn remove_lowest_priority_internal(
        &self,
        priority_queue: &mut BinaryHeap<WeightedOpportunity>,
        opportunity_map: &mut HashMap<String, WeightedOpportunity>,
        type_groups: &mut HashMap<StrategyType, Vec<String>>,
        exchange_groups: &mut HashMap<String, Vec<String>>,
    ) {
        if let Some(lowest) = priority_queue.iter().min_by(|a, b| a.weighted_score.partial_cmp(&b.weighted_score).unwrap()) {
            let opportunity_id = lowest.opportunity.id.clone();
            let opportunity = lowest.opportunity.clone();
            
            // 重建队列，排除最低优先级的机会
            let remaining: Vec<WeightedOpportunity> = priority_queue
                .drain()
                .filter(|wo| wo.opportunity.id != opportunity_id)
                .collect();
            
            for wo in remaining {
                priority_queue.push(wo);
            }

            opportunity_map.remove(&opportunity_id);
            self.remove_from_groups_sync(&opportunity, type_groups, exchange_groups);
        }
    }

    /// 从分组中移除机会
    async fn remove_from_groups(
        &self,
        opportunity: &ArbitrageOpportunityCore,
        type_groups: &mut HashMap<StrategyType, Vec<String>>,
        exchange_groups: &mut HashMap<String, Vec<String>>,
    ) {
        self.remove_from_groups_sync(opportunity, type_groups, exchange_groups);
    }

    /// 同步版本的分组移除
    fn remove_from_groups_sync(
        &self,
        opportunity: &ArbitrageOpportunityCore,
        type_groups: &mut HashMap<StrategyType, Vec<String>>,
        exchange_groups: &mut HashMap<String, Vec<String>>,
    ) {
        // 从策略类型分组中移除
        if let Some(type_list) = type_groups.get_mut(&opportunity.strategy_type) {
            type_list.retain(|id| id != &opportunity.id);
            if type_list.is_empty() {
                type_groups.remove(&opportunity.strategy_type);
            }
        }

        // 从交易所分组中移除
        if let Some(exchange_list) = exchange_groups.get_mut(&opportunity.buy_exchange) {
            exchange_list.retain(|id| id != &opportunity.id);
            if exchange_list.is_empty() {
                exchange_groups.remove(&opportunity.buy_exchange);
            }
        }

        if opportunity.buy_exchange != opportunity.sell_exchange {
            if let Some(exchange_list) = exchange_groups.get_mut(&opportunity.sell_exchange) {
                exchange_list.retain(|id| id != &opportunity.id);
                if exchange_list.is_empty() {
                    exchange_groups.remove(&opportunity.sell_exchange);
                }
            }
        }
    }

    /// 重新计算平均得分
    async fn recalculate_avg_score(
        &self,
        priority_queue: &BinaryHeap<WeightedOpportunity>,
        stats: &mut PoolStatistics,
    ) {
        if priority_queue.is_empty() {
            stats.avg_weighted_score = 0.0;
        } else {
            let total_score: f64 = priority_queue.iter().map(|wo| wo.weighted_score).sum();
            stats.avg_weighted_score = total_score / priority_queue.len() as f64;
        }
    }
}

impl Default for GlobalOpportunityPool {
    fn default() -> Self {
        Self::new(OpportunityPoolConfig::default())
    }
}


