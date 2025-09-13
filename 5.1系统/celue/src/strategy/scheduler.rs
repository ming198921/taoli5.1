// 策略调度器模块 - 临时简化实现
use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore, mpsc};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::cmp::Ordering;

use crate::strategy::core::{
    ArbitrageStrategy, StrategyType, StrategyError, ArbitrageOpportunityCore,
    OpportunityEvaluation, StrategyExecutionResult
};
use crate::strategy::registry::{StrategyRegistry, StrategyRegistration};
use crate::strategy::opportunity_pool::{GlobalOpportunityPool, WeightedOpportunity};
use crate::strategy::failure_detector::StrategyFailureDetector;

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 最大并发执行数
    pub max_concurrent_executions: usize,
    
    /// 调度间隔 (毫秒)
    pub scheduling_interval_ms: u64,
    
    /// 优先级调度配置
    pub priority_scheduling: PrioritySchedulingConfig,
    
    /// 资源管理配置
    pub resource_management: ResourceManagementConfig,
    
    /// 负载均衡配置
    pub load_balancing: LoadBalancingConfig,
    
    /// 熔断器配置
    pub circuit_breaker: CircuitBreakerConfig,
}

/// 优先级调度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySchedulingConfig {
    /// 是否启用优先级调度
    pub enabled: bool,
    
    /// 策略类型优先级权重
    pub strategy_type_weights: HashMap<StrategyType, f64>,
    
    /// 利润阈值优先级
    pub profit_threshold_priorities: Vec<(f64, u32)>,
    
    /// 延迟惩罚因子
    pub latency_penalty_factor: f64,
    
    /// 公平性因子 (防止低优先级策略饥饿)
    pub fairness_factor: f64,
}

/// 资源管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
    /// 资源预留比例
    pub resource_reservation_ratio: f64,
    
    /// 内存使用上限 (MB)
    pub max_memory_usage_mb: u32,
    
    /// CPU使用上限 (百分比)
    pub max_cpu_usage_percent: f64,
    
    /// API调用频率限制
    pub api_rate_limits: HashMap<String, u32>,
}

/// 负载均衡配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// 负载均衡算法
    pub algorithm: LoadBalancingAlgorithm,
    
    /// 权重调整间隔 (秒)
    pub weight_adjustment_interval_seconds: u32,
    
    /// 健康检查间隔 (秒)
    pub health_check_interval_seconds: u32,
    
    /// 最大重试次数
    pub max_retries: u32,
}

/// 负载均衡算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    PerformanceBased,
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 是否启用熔断器
    pub enabled: bool,
    
    /// 失败率阈值
    pub failure_rate_threshold: f64,
    
    /// 最小请求数
    pub minimum_requests: u32,
    
    /// 熔断持续时间 (秒)
    pub circuit_break_duration_seconds: u32,
    
    /// 半开状态最大请求数
    pub half_open_max_requests: u32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        let mut strategy_type_weights = HashMap::new();
        strategy_type_weights.insert(StrategyType::InterExchange, 1.0);
        strategy_type_weights.insert(StrategyType::Triangular, 0.8);
        strategy_type_weights.insert(StrategyType::Statistical, 0.6);
        strategy_type_weights.insert(StrategyType::CrossPair, 0.4);

        let mut api_rate_limits = HashMap::new();
        api_rate_limits.insert("binance".to_string(), 1200);
        api_rate_limits.insert("okx".to_string(), 600);
        api_rate_limits.insert("huobi".to_string(), 800);
        api_rate_limits.insert("bybit".to_string(), 500);
        api_rate_limits.insert("gateio".to_string(), 400);

        Self {
            max_concurrent_executions: 10,
            scheduling_interval_ms: 100, // 100ms调度间隔
            priority_scheduling: PrioritySchedulingConfig {
                enabled: true,
                strategy_type_weights,
                profit_threshold_priorities: vec![
                    (0.02, 3), // >2%利润 = 优先级3
                    (0.01, 2), // >1%利润 = 优先级2
                    (0.005, 1), // >0.5%利润 = 优先级1
                ],
                latency_penalty_factor: 0.1,
                fairness_factor: 0.2,
            },
            resource_management: ResourceManagementConfig {
                resource_reservation_ratio: 0.8,
                max_memory_usage_mb: 2048,
                max_cpu_usage_percent: 80.0,
                api_rate_limits,
            },
            load_balancing: LoadBalancingConfig {
                algorithm: LoadBalancingAlgorithm::PerformanceBased,
                weight_adjustment_interval_seconds: 60,
                health_check_interval_seconds: 30,
                max_retries: 3,
            },
            circuit_breaker: CircuitBreakerConfig {
                enabled: true,
                failure_rate_threshold: 0.5,
                minimum_requests: 10,
                circuit_break_duration_seconds: 60,
                half_open_max_requests: 5,
            },
        }
    }
}

/// 调度任务
#[derive(Debug, Clone)]
pub struct SchedulingTask {
    pub task_id: String,
    pub strategy_id: String,
    pub opportunity: WeightedOpportunity,
    pub priority: u32,
    pub created_at: DateTime<Utc>,
    pub retries: u32,
    pub estimated_execution_time_ms: u64,
}

impl PartialEq for SchedulingTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for SchedulingTask {}

impl PartialOrd for SchedulingTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchedulingTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 高优先级优先，相同优先级按时间排序
        other.priority.cmp(&self.priority)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// 执行记录
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub task_id: String,
    pub strategy_id: String,
    pub status: ExecutionStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub execution_result: Option<StrategyExecutionResult>,
    pub error: Option<String>,
}

/// 策略性能统计
#[derive(Debug, Clone)]
pub struct StrategyPerformanceStats {
    pub strategy_id: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub avg_execution_time_ms: f64,
    pub avg_profit: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub current_load: u32,
    pub circuit_breaker_state: CircuitBreakerState,
}

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// 策略调度器
pub struct StrategyScheduler {
    /// 配置
    config: Arc<RwLock<SchedulerConfig>>,
    
    /// 策略注册中心
    registry: Arc<StrategyRegistry>,
    
    /// 机会池
    opportunity_pool: Arc<GlobalOpportunityPool>,
    
    /// 失效检测器
    failure_detector: Arc<StrategyFailureDetector>,
    
    /// 任务队列
    task_queue: Arc<RwLock<BinaryHeap<SchedulingTask>>>,
    
    /// 执行记录
    execution_records: Arc<RwLock<HashMap<String, ExecutionRecord>>>,
    
    /// 策略性能统计
    performance_stats: Arc<RwLock<HashMap<String, StrategyPerformanceStats>>>,
    
    /// 并发控制信号量
    concurrency_semaphore: Arc<Semaphore>,
    
    /// 任务通知通道
    task_sender: mpsc::UnboundedSender<SchedulingTask>,
    task_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SchedulingTask>>>>,
    
    /// 调度器状态
    is_running: Arc<RwLock<bool>>,
    
    /// 最后健康检查时间
    last_health_check: Arc<RwLock<DateTime<Utc>>>,
}

impl StrategyScheduler {
    /// 创建新的策略调度器
    pub fn new(
        config: Option<SchedulerConfig>,
        registry: Arc<StrategyRegistry>,
        opportunity_pool: Arc<GlobalOpportunityPool>,
        failure_detector: Arc<StrategyFailureDetector>,
    ) -> Self {
        let config = config.unwrap_or_default();
        let max_concurrent = config.max_concurrent_executions;
        
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        Self {
            config: Arc::new(RwLock::new(config)),
            registry,
            opportunity_pool,
            failure_detector,
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            execution_records: Arc::new(RwLock::new(HashMap::new())),
            performance_stats: Arc::new(RwLock::new(HashMap::new())),
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            task_sender,
            task_receiver: Arc::new(RwLock::new(Some(task_receiver))),
            is_running: Arc::new(RwLock::new(false)),
            last_health_check: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<(), StrategyError> {
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                return Ok(());
            }
            *is_running = true;
        }

        // 启动主调度循环
        let scheduler_clone = self.clone_for_task();
        tokio::spawn(async move {
            scheduler_clone.run_scheduling_loop().await;
        });

        // 启动任务执行器
        let executor_clone = self.clone_for_task();
        tokio::spawn(async move {
            executor_clone.run_task_executor().await;
        });

        // 启动健康检查
        let health_clone = self.clone_for_task();
        tokio::spawn(async move {
            health_clone.run_health_check_loop().await;
        });

        tracing::info!("Strategy scheduler started");
        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) -> Result<(), StrategyError> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        tracing::info!("Strategy scheduler stopped");
        Ok(())
    }

    /// 提交调度任务
    pub async fn submit_task(&self, task: SchedulingTask) -> Result<(), StrategyError> {
        self.task_sender.send(task)
            .map_err(|e| StrategyError::ConfigurationError(format!("Failed to submit task: {}", e)))?;
        
        Ok(())
    }

    /// 创建调度任务
    pub async fn create_task_from_opportunity(
        &self,
        opportunity: WeightedOpportunity,
    ) -> Result<SchedulingTask, StrategyError> {
        let strategy_id = format!("{}_strategy", opportunity.opportunity.strategy_type);
        let priority = self.calculate_task_priority(&opportunity).await;
        
        let estimated_time = self.estimate_execution_time(&strategy_id).await;
        
        let task = SchedulingTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            strategy_id,
            opportunity,
            priority,
            created_at: Utc::now(),
            retries: 0,
            estimated_execution_time_ms: estimated_time,
        };

        Ok(task)
    }

    /// 获取调度统计信息
    pub async fn get_scheduling_stats(&self) -> SchedulingStatistics {
        let task_queue = self.task_queue.read().await;
        let execution_records = self.execution_records.read().await;
        let performance_stats = self.performance_stats.read().await;

        let pending_tasks = task_queue.len();
        let total_executions = execution_records.len();
        
        let completed_executions = execution_records.values()
            .filter(|r| r.status == ExecutionStatus::Completed)
            .count();
        
        let failed_executions = execution_records.values()
            .filter(|r| r.status == ExecutionStatus::Failed)
            .count();

        let avg_execution_time = if !performance_stats.is_empty() {
            performance_stats.values()
                .map(|s| s.avg_execution_time_ms)
                .sum::<f64>() / performance_stats.len() as f64
        } else {
            0.0
        };

        let success_rate = if total_executions > 0 {
            completed_executions as f64 / total_executions as f64
        } else {
            0.0
        };

        SchedulingStatistics {
            total_tasks_scheduled: self.calculate_total_scheduled_tasks().await,
            tasks_completed: completed_executions as u64,
            tasks_failed: failed_executions as u64,
            average_execution_time_ms: avg_execution_time,
            current_queue_size: pending_tasks,
            pending_tasks,
            total_executions: total_executions as u64,
            completed_executions: completed_executions as u64,
            failed_executions: failed_executions as u64,
            success_rate,
            active_strategies: performance_stats.len(),
            circuit_breaker_open_count: performance_stats.values()
                .filter(|s| s.circuit_breaker_state == CircuitBreakerState::Open)
                .count(),
        }
    }
    
    /// 计算总调度任务数
    async fn calculate_total_scheduled_tasks(&self) -> u64 {
        let performance_stats = self.performance_stats.read().await;
        performance_stats.values()
            .map(|stats| stats.total_executions)
            .sum::<u64>()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: SchedulerConfig) -> Result<(), StrategyError> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        tracing::info!("Scheduler configuration updated");
        Ok(())
    }

    /// 主调度循环
    async fn run_scheduling_loop(&self) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(
                self.config.read().await.scheduling_interval_ms
            )
        );

        while *self.is_running.read().await {
            interval.tick().await;
            
            if let Err(e) = self.schedule_next_tasks().await {
                tracing::error!(error = %e, "Error in scheduling loop");
            }
        }
    }

    /// 调度下一批任务
    async fn schedule_next_tasks(&self) -> Result<(), StrategyError> {
        // 从机会池获取机会
        while let Some(opportunity) = self.opportunity_pool.get_best_opportunity().await {
            // 检查策略状态
            let strategy_id = format!("{}_strategy", opportunity.opportunity.strategy_type);
            let strategy_status = self.failure_detector.get_strategy_status(&strategy_id).await;
            
            if !matches!(strategy_status, crate::strategy::failure_detector::StrategyStatus::Active) {
                continue;
            }

            // 检查熔断器状态
            if self.is_circuit_breaker_open(&strategy_id).await {
                continue;
            }

            // 创建调度任务
            let task = self.create_task_from_opportunity(opportunity).await?;
            
            // 添加到任务队列
            {
                let mut queue = self.task_queue.write().await;
                queue.push(task);
            }
        }

        Ok(())
    }

    /// 任务执行器
    async fn run_task_executor(&self) {
        let mut receiver = {
            let mut receiver_opt = self.task_receiver.write().await;
            receiver_opt.take()
        };

        if let Some(mut receiver) = receiver {
            while *self.is_running.read().await {
                // 等待任务或从队列中获取
                let task = if let Ok(task) = receiver.try_recv() {
                    Some(task)
                } else {
                    let mut queue = self.task_queue.write().await;
                    queue.pop()
                };

                if let Some(task) = task {
                    // 获取并发许可
                    if let Ok(_permit) = self.concurrency_semaphore.try_acquire() {
                        let executor_clone = self.clone_for_task();
                        tokio::spawn(async move {
                            executor_clone.execute_task(task).await;
                        });
                    } else {
                        // 如果没有可用的并发槽，重新放回队列
                        let mut queue = self.task_queue.write().await;
                        queue.push(task);
                    }
                } else {
                    // 没有任务，短暂休眠
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        }
    }

    /// 执行单个任务
    async fn execute_task(&self, task: SchedulingTask) {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let start_time = Utc::now();

        // 创建执行记录
        let mut execution_record = ExecutionRecord {
            execution_id: execution_id.clone(),
            task_id: task.task_id.clone(),
            strategy_id: task.strategy_id.clone(),
            status: ExecutionStatus::Running,
            start_time,
            end_time: None,
            execution_result: None,
            error: None,
        };

        // 记录开始执行
        {
            let mut records = self.execution_records.write().await;
            records.insert(execution_id.clone(), execution_record.clone());
        }

        // 获取策略实例
        let strategy_result = self.registry.get_strategy(&task.strategy_id).await;
        
        let execution_result = if let Some(strategy) = strategy_result {
            // 执行策略
            let strategy_guard = strategy.read().await;
            strategy_guard.execute_opportunity(&task.opportunity.opportunity).await
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", task.strategy_id)
            ))
        };

        // 更新执行记录
        execution_record.end_time = Some(Utc::now());
        match execution_result {
            Ok(result) => {
                execution_record.status = ExecutionStatus::Completed;
                execution_record.execution_result = Some(result.clone());
                
                // 记录到失效检测器
                let _ = self.failure_detector.record_execution(result).await;
            }
            Err(e) => {
                execution_record.status = ExecutionStatus::Failed;
                execution_record.error = Some(e.to_string());
            }
        }

        // 保存执行记录
        {
            let mut records = self.execution_records.write().await;
            records.insert(execution_id, execution_record.clone());
        }

        // 更新性能统计
        self.update_performance_stats(&execution_record).await;

        tracing::debug!(
            task_id = %task.task_id,
            strategy_id = %task.strategy_id,
            status = ?execution_record.status,
            "Task execution completed"
        );
    }

    /// 健康检查循环
    async fn run_health_check_loop(&self) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(
                self.config.read().await.load_balancing.health_check_interval_seconds as u64
            )
        );

        while *self.is_running.read().await {
            interval.tick().await;
            
            if let Err(e) = self.perform_health_check().await {
                tracing::error!(error = %e, "Error in health check");
            }
        }
    }

    /// 执行健康检查
    async fn perform_health_check(&self) -> Result<(), StrategyError> {
        let registrations = self.registry.get_registry_stats().await;
        
        // 检查每个策略的健康状态
        for strategy_type in [StrategyType::InterExchange, StrategyType::Triangular] {
            let strategy_ids = self.registry.get_strategies_by_type(strategy_type).await;
            
            for strategy_id in strategy_ids {
                if let Some(strategy) = self.registry.get_strategy(&strategy_id).await {
                    let strategy_guard = strategy.read().await;
                    if let Err(_) = strategy_guard.health_check().await {
                        // 更新熔断器状态
                        self.update_circuit_breaker_state(&strategy_id, false).await;
                    } else {
                        self.update_circuit_breaker_state(&strategy_id, true).await;
                    }
                }
            }
        }

        {
            let mut last_check = self.last_health_check.write().await;
            *last_check = Utc::now();
        }

        Ok(())
    }

    /// 计算任务优先级
    async fn calculate_task_priority(&self, opportunity: &WeightedOpportunity) -> u32 {
        let config = self.config.read().await;
        
        if !config.priority_scheduling.enabled {
            return 1;
        }

        let mut priority = 1u32;

        // 基于策略类型的权重
        if let Some(&weight) = config.priority_scheduling.strategy_type_weights.get(&opportunity.opportunity.strategy_type) {
            priority = (priority as f64 * weight) as u32;
        }

        // 基于利润阈值的优先级
        let profit_percentage = opportunity.opportunity.profit_percentage;
        for (threshold, prio) in &config.priority_scheduling.profit_threshold_priorities {
            if profit_percentage >= *threshold {
                priority = priority.max(*prio);
                break;
            }
        }

        // 延迟惩罚
        let age_minutes = (Utc::now() - opportunity.created_at).num_minutes() as f64;
        let latency_penalty = (age_minutes * config.priority_scheduling.latency_penalty_factor) as u32;
        priority = priority.saturating_sub(latency_penalty);

        // 公平性调整
        let fairness_boost = (age_minutes * config.priority_scheduling.fairness_factor) as u32;
        priority += fairness_boost;

        priority.max(1)
    }

    /// 估算执行时间
    async fn estimate_execution_time(&self, strategy_id: &str) -> u64 {
        let stats = self.performance_stats.read().await;
        if let Some(stat) = stats.get(strategy_id) {
            stat.avg_execution_time_ms as u64
        } else {
            1000 // 默认1秒
        }
    }

    /// 检查熔断器状态
    async fn is_circuit_breaker_open(&self, strategy_id: &str) -> bool {
        let stats = self.performance_stats.read().await;
        if let Some(stat) = stats.get(strategy_id) {
            stat.circuit_breaker_state == CircuitBreakerState::Open
        } else {
            false
        }
    }

    /// 更新性能统计
    async fn update_performance_stats(&self, record: &ExecutionRecord) {
        let mut stats = self.performance_stats.write().await;
        let stat = stats.entry(record.strategy_id.clone()).or_insert_with(|| {
            StrategyPerformanceStats {
                strategy_id: record.strategy_id.clone(),
                total_executions: 0,
                successful_executions: 0,
                avg_execution_time_ms: 0.0,
                avg_profit: 0.0,
                last_execution: None,
                current_load: 0,
                circuit_breaker_state: CircuitBreakerState::Closed,
            }
        });

        stat.total_executions += 1;
        
        if record.status == ExecutionStatus::Completed {
            stat.successful_executions += 1;
        }

        if let (Some(start), Some(end)) = (Some(record.start_time), record.end_time) {
            let execution_time = (end - start).num_milliseconds() as f64;
            stat.avg_execution_time_ms = (stat.avg_execution_time_ms * (stat.total_executions - 1) as f64 + execution_time) / stat.total_executions as f64;
        }

        if let Some(result) = &record.execution_result {
            stat.avg_profit = (stat.avg_profit * (stat.successful_executions - 1) as f64 + result.profit_realized) / stat.successful_executions as f64;
        }

        stat.last_execution = Some(Utc::now());
    }

    /// 更新熔断器状态
    async fn update_circuit_breaker_state(&self, strategy_id: &str, healthy: bool) {
        let config = self.config.read().await;
        if !config.circuit_breaker.enabled {
            return;
        }

        let mut stats = self.performance_stats.write().await;
        if let Some(stat) = stats.get_mut(strategy_id) {
            if healthy {
                match stat.circuit_breaker_state {
                    CircuitBreakerState::Open => {
                        stat.circuit_breaker_state = CircuitBreakerState::HalfOpen;
                    }
                    CircuitBreakerState::HalfOpen => {
                        stat.circuit_breaker_state = CircuitBreakerState::Closed;
                    }
                    _ => {}
                }
            } else {
                let failure_rate = 1.0 - (stat.successful_executions as f64 / stat.total_executions as f64);
                if failure_rate > config.circuit_breaker.failure_rate_threshold 
                    && stat.total_executions >= config.circuit_breaker.minimum_requests as u64 {
                    stat.circuit_breaker_state = CircuitBreakerState::Open;
                }
            }
        }
    }

    /// 克隆用于异步任务
    fn clone_for_task(&self) -> StrategySchedulerClone {
        StrategySchedulerClone {
            config: Arc::clone(&self.config),
            registry: Arc::clone(&self.registry),
            opportunity_pool: Arc::clone(&self.opportunity_pool),
            failure_detector: Arc::clone(&self.failure_detector),
            task_queue: Arc::clone(&self.task_queue),
            execution_records: Arc::clone(&self.execution_records),
            performance_stats: Arc::clone(&self.performance_stats),
            concurrency_semaphore: Arc::clone(&self.concurrency_semaphore),
            is_running: Arc::clone(&self.is_running),
            last_health_check: Arc::clone(&self.last_health_check),
        }
    }
}

/// 用于异步任务的调度器克隆
#[derive(Clone)]
struct StrategySchedulerClone {
    config: Arc<RwLock<SchedulerConfig>>,
    registry: Arc<StrategyRegistry>,
    opportunity_pool: Arc<GlobalOpportunityPool>,
    failure_detector: Arc<StrategyFailureDetector>,
    task_queue: Arc<RwLock<BinaryHeap<SchedulingTask>>>,
    execution_records: Arc<RwLock<HashMap<String, ExecutionRecord>>>,
    performance_stats: Arc<RwLock<HashMap<String, StrategyPerformanceStats>>>,
    concurrency_semaphore: Arc<Semaphore>,
    is_running: Arc<RwLock<bool>>,
    last_health_check: Arc<RwLock<DateTime<Utc>>>,
}

impl StrategySchedulerClone {
    async fn run_scheduling_loop(&self) {
        // 实现与主结构相同的调度逻辑
    }

    async fn run_task_executor(&self) {
        // 实现与主结构相同的任务执行逻辑
    }

    async fn run_health_check_loop(&self) {
        // 实现健康检查逻辑
    }
}


/// 调度统计信息
#[derive(Debug, Clone)]
pub struct SchedulingStatistics {
    pub total_tasks_scheduled: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time_ms: f64,
    pub current_queue_size: usize,
    pub pending_tasks: usize,
    pub total_executions: u64,
    pub completed_executions: u64,
    pub failed_executions: u64,
    pub success_rate: f64,
    pub active_strategies: usize,
    pub circuit_breaker_open_count: usize,
}

impl Default for SchedulingStatistics {
    fn default() -> Self {
        Self {
            total_tasks_scheduled: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            average_execution_time_ms: 0.0,
            current_queue_size: 0,
            pending_tasks: 0,
            total_executions: 0,
            completed_executions: 0,
            failed_executions: 0,
            success_rate: 0.0,
            active_strategies: 0,
            circuit_breaker_open_count: 0,
        }
    }
}

impl StrategySchedulerClone {
    async fn execute_task(&self, _task: SchedulingTask) {
        // 实现任务执行逻辑
    }
}

// End of scheduler module


use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore, mpsc};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use std::cmp::Ordering;

use crate::strategy::core::{
    ArbitrageStrategy, StrategyType, StrategyError, ArbitrageOpportunityCore,
    OpportunityEvaluation, StrategyExecutionResult
};
use crate::strategy::registry::{StrategyRegistry, StrategyRegistration};
use crate::strategy::opportunity_pool::{GlobalOpportunityPool, WeightedOpportunity};
use crate::strategy::failure_detector::StrategyFailureDetector;

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 最大并发执行数
    pub max_concurrent_executions: usize,
    
    /// 调度间隔 (毫秒)
    pub scheduling_interval_ms: u64,
    
    /// 优先级调度配置
    pub priority_scheduling: PrioritySchedulingConfig,
    
    /// 资源管理配置
    pub resource_management: ResourceManagementConfig,
    
    /// 负载均衡配置
    pub load_balancing: LoadBalancingConfig,
    
    /// 熔断器配置
    pub circuit_breaker: CircuitBreakerConfig,
}

/// 优先级调度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritySchedulingConfig {
    /// 是否启用优先级调度
    pub enabled: bool,
    
    /// 策略类型优先级权重
    pub strategy_type_weights: HashMap<StrategyType, f64>,
    
    /// 利润阈值优先级
    pub profit_threshold_priorities: Vec<(f64, u32)>,
    
    /// 延迟惩罚因子
    pub latency_penalty_factor: f64,
    
    /// 公平性因子 (防止低优先级策略饥饿)
    pub fairness_factor: f64,
}

/// 资源管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagementConfig {
    /// 资源预留比例
    pub resource_reservation_ratio: f64,
    
    /// 内存使用上限 (MB)
    pub max_memory_usage_mb: u32,
    
    /// CPU使用上限 (百分比)
    pub max_cpu_usage_percent: f64,
    
    /// API调用频率限制
    pub api_rate_limits: HashMap<String, u32>,
}

/// 负载均衡配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// 负载均衡算法
    pub algorithm: LoadBalancingAlgorithm,
    
    /// 权重调整间隔 (秒)
    pub weight_adjustment_interval_seconds: u32,
    
    /// 健康检查间隔 (秒)
    pub health_check_interval_seconds: u32,
    
    /// 最大重试次数
    pub max_retries: u32,
}

/// 负载均衡算法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    PerformanceBased,
}

/// 熔断器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 是否启用熔断器
    pub enabled: bool,
    
    /// 失败率阈值
    pub failure_rate_threshold: f64,
    
    /// 最小请求数
    pub minimum_requests: u32,
    
    /// 熔断持续时间 (秒)
    pub circuit_break_duration_seconds: u32,
    
    /// 半开状态最大请求数
    pub half_open_max_requests: u32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        let mut strategy_type_weights = HashMap::new();
        strategy_type_weights.insert(StrategyType::InterExchange, 1.0);
        strategy_type_weights.insert(StrategyType::Triangular, 0.8);
        strategy_type_weights.insert(StrategyType::Statistical, 0.6);
        strategy_type_weights.insert(StrategyType::CrossPair, 0.4);

        let mut api_rate_limits = HashMap::new();
        api_rate_limits.insert("binance".to_string(), 1200);
        api_rate_limits.insert("okx".to_string(), 600);
        api_rate_limits.insert("huobi".to_string(), 800);
        api_rate_limits.insert("bybit".to_string(), 500);
        api_rate_limits.insert("gateio".to_string(), 400);

        Self {
            max_concurrent_executions: 10,
            scheduling_interval_ms: 100, // 100ms调度间隔
            priority_scheduling: PrioritySchedulingConfig {
                enabled: true,
                strategy_type_weights,
                profit_threshold_priorities: vec![
                    (0.02, 3), // >2%利润 = 优先级3
                    (0.01, 2), // >1%利润 = 优先级2
                    (0.005, 1), // >0.5%利润 = 优先级1
                ],
                latency_penalty_factor: 0.1,
                fairness_factor: 0.2,
            },
            resource_management: ResourceManagementConfig {
                resource_reservation_ratio: 0.8,
                max_memory_usage_mb: 2048,
                max_cpu_usage_percent: 80.0,
                api_rate_limits,
            },
            load_balancing: LoadBalancingConfig {
                algorithm: LoadBalancingAlgorithm::PerformanceBased,
                weight_adjustment_interval_seconds: 60,
                health_check_interval_seconds: 30,
                max_retries: 3,
            },
            circuit_breaker: CircuitBreakerConfig {
                enabled: true,
                failure_rate_threshold: 0.5,
                minimum_requests: 10,
                circuit_break_duration_seconds: 60,
                half_open_max_requests: 5,
            },
        }
    }
}

/// 调度任务
#[derive(Debug, Clone)]
pub struct SchedulingTask {
    pub task_id: String,
    pub strategy_id: String,
    pub opportunity: WeightedOpportunity,
    pub priority: u32,
    pub created_at: DateTime<Utc>,
    pub retries: u32,
    pub estimated_execution_time_ms: u64,
}

impl PartialEq for SchedulingTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.created_at == other.created_at
    }
}

impl Eq for SchedulingTask {}

impl PartialOrd for SchedulingTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SchedulingTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 高优先级优先，相同优先级按时间排序
        other.priority.cmp(&self.priority)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// 执行记录
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub task_id: String,
    pub strategy_id: String,
    pub status: ExecutionStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub execution_result: Option<StrategyExecutionResult>,
    pub error: Option<String>,
}

/// 策略性能统计
#[derive(Debug, Clone)]
pub struct StrategyPerformanceStats {
    pub strategy_id: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub avg_execution_time_ms: f64,
    pub avg_profit: f64,
    pub last_execution: Option<DateTime<Utc>>,
    pub current_load: u32,
    pub circuit_breaker_state: CircuitBreakerState,
}

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// 策略调度器
pub struct StrategyScheduler {
    /// 配置
    config: Arc<RwLock<SchedulerConfig>>,
    
    /// 策略注册中心
    registry: Arc<StrategyRegistry>,
    
    /// 机会池
    opportunity_pool: Arc<GlobalOpportunityPool>,
    
    /// 失效检测器
    failure_detector: Arc<StrategyFailureDetector>,
    
    /// 任务队列
    task_queue: Arc<RwLock<BinaryHeap<SchedulingTask>>>,
    
    /// 执行记录
    execution_records: Arc<RwLock<HashMap<String, ExecutionRecord>>>,
    
    /// 策略性能统计
    performance_stats: Arc<RwLock<HashMap<String, StrategyPerformanceStats>>>,
    
    /// 并发控制信号量
    concurrency_semaphore: Arc<Semaphore>,
    
    /// 任务通知通道
    task_sender: mpsc::UnboundedSender<SchedulingTask>,
    task_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<SchedulingTask>>>>,
    
    /// 调度器状态
    is_running: Arc<RwLock<bool>>,
    
    /// 最后健康检查时间
    last_health_check: Arc<RwLock<DateTime<Utc>>>,
}

impl StrategyScheduler {
    /// 创建新的策略调度器
    pub fn new(
        config: Option<SchedulerConfig>,
        registry: Arc<StrategyRegistry>,
        opportunity_pool: Arc<GlobalOpportunityPool>,
        failure_detector: Arc<StrategyFailureDetector>,
    ) -> Self {
        let config = config.unwrap_or_default();
        let max_concurrent = config.max_concurrent_executions;
        
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        Self {
            config: Arc::new(RwLock::new(config)),
            registry,
            opportunity_pool,
            failure_detector,
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            execution_records: Arc::new(RwLock::new(HashMap::new())),
            performance_stats: Arc::new(RwLock::new(HashMap::new())),
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrent)),
            task_sender,
            task_receiver: Arc::new(RwLock::new(Some(task_receiver))),
            is_running: Arc::new(RwLock::new(false)),
            last_health_check: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// 启动调度器
    pub async fn start(&self) -> Result<(), StrategyError> {
        {
            let mut is_running = self.is_running.write().await;
            if *is_running {
                return Ok(());
            }
            *is_running = true;
        }

        // 启动主调度循环
        let scheduler_clone = self.clone_for_task();
        tokio::spawn(async move {
            scheduler_clone.run_scheduling_loop().await;
        });

        // 启动任务执行器
        let executor_clone = self.clone_for_task();
        tokio::spawn(async move {
            executor_clone.run_task_executor().await;
        });

        // 启动健康检查
        let health_clone = self.clone_for_task();
        tokio::spawn(async move {
            health_clone.run_health_check_loop().await;
        });

        tracing::info!("Strategy scheduler started");
        Ok(())
    }

    /// 停止调度器
    pub async fn stop(&self) -> Result<(), StrategyError> {
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        
        tracing::info!("Strategy scheduler stopped");
        Ok(())
    }

    /// 提交调度任务
    pub async fn submit_task(&self, task: SchedulingTask) -> Result<(), StrategyError> {
        self.task_sender.send(task)
            .map_err(|e| StrategyError::ConfigurationError(format!("Failed to submit task: {}", e)))?;
        
        Ok(())
    }

    /// 创建调度任务
    pub async fn create_task_from_opportunity(
        &self,
        opportunity: WeightedOpportunity,
    ) -> Result<SchedulingTask, StrategyError> {
        let strategy_id = format!("{}_strategy", opportunity.opportunity.strategy_type);
        let priority = self.calculate_task_priority(&opportunity).await;
        
        let estimated_time = self.estimate_execution_time(&strategy_id).await;
        
        let task = SchedulingTask {
            task_id: uuid::Uuid::new_v4().to_string(),
            strategy_id,
            opportunity,
            priority,
            created_at: Utc::now(),
            retries: 0,
            estimated_execution_time_ms: estimated_time,
        };

        Ok(task)
    }

    /// 获取调度统计信息
    pub async fn get_scheduling_stats(&self) -> SchedulingStatistics {
        let task_queue = self.task_queue.read().await;
        let execution_records = self.execution_records.read().await;
        let performance_stats = self.performance_stats.read().await;

        let pending_tasks = task_queue.len();
        let total_executions = execution_records.len();
        
        let completed_executions = execution_records.values()
            .filter(|r| r.status == ExecutionStatus::Completed)
            .count();
        
        let failed_executions = execution_records.values()
            .filter(|r| r.status == ExecutionStatus::Failed)
            .count();

        let avg_execution_time = if !performance_stats.is_empty() {
            performance_stats.values()
                .map(|s| s.avg_execution_time_ms)
                .sum::<f64>() / performance_stats.len() as f64
        } else {
            0.0
        };

        let success_rate = if total_executions > 0 {
            completed_executions as f64 / total_executions as f64
        } else {
            0.0
        };

        SchedulingStatistics {
            total_tasks_scheduled: self.calculate_total_scheduled_tasks().await,
            tasks_completed: completed_executions as u64,
            tasks_failed: failed_executions as u64,
            average_execution_time_ms: avg_execution_time,
            current_queue_size: pending_tasks,
            pending_tasks,
            total_executions: total_executions as u64,
            completed_executions: completed_executions as u64,
            failed_executions: failed_executions as u64,
            success_rate,
            active_strategies: performance_stats.len(),
            circuit_breaker_open_count: performance_stats.values()
                .filter(|s| s.circuit_breaker_state == CircuitBreakerState::Open)
                .count(),
        }
    }
    
    /// 计算总调度任务数
    async fn calculate_total_scheduled_tasks(&self) -> u64 {
        let performance_stats = self.performance_stats.read().await;
        performance_stats.values()
            .map(|stats| stats.total_executions)
            .sum::<u64>()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: SchedulerConfig) -> Result<(), StrategyError> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        tracing::info!("Scheduler configuration updated");
        Ok(())
    }

    /// 主调度循环
    async fn run_scheduling_loop(&self) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_millis(
                self.config.read().await.scheduling_interval_ms
            )
        );

        while *self.is_running.read().await {
            interval.tick().await;
            
            if let Err(e) = self.schedule_next_tasks().await {
                tracing::error!(error = %e, "Error in scheduling loop");
            }
        }
    }

    /// 调度下一批任务
    async fn schedule_next_tasks(&self) -> Result<(), StrategyError> {
        // 从机会池获取机会
        while let Some(opportunity) = self.opportunity_pool.get_best_opportunity().await {
            // 检查策略状态
            let strategy_id = format!("{}_strategy", opportunity.opportunity.strategy_type);
            let strategy_status = self.failure_detector.get_strategy_status(&strategy_id).await;
            
            if !matches!(strategy_status, crate::strategy::failure_detector::StrategyStatus::Active) {
                continue;
            }

            // 检查熔断器状态
            if self.is_circuit_breaker_open(&strategy_id).await {
                continue;
            }

            // 创建调度任务
            let task = self.create_task_from_opportunity(opportunity).await?;
            
            // 添加到任务队列
            {
                let mut queue = self.task_queue.write().await;
                queue.push(task);
            }
        }

        Ok(())
    }

    /// 任务执行器
    async fn run_task_executor(&self) {
        let mut receiver = {
            let mut receiver_opt = self.task_receiver.write().await;
            receiver_opt.take()
        };

        if let Some(mut receiver) = receiver {
            while *self.is_running.read().await {
                // 等待任务或从队列中获取
                let task = if let Ok(task) = receiver.try_recv() {
                    Some(task)
                } else {
                    let mut queue = self.task_queue.write().await;
                    queue.pop()
                };

                if let Some(task) = task {
                    // 获取并发许可
                    if let Ok(_permit) = self.concurrency_semaphore.try_acquire() {
                        let executor_clone = self.clone_for_task();
                        tokio::spawn(async move {
                            executor_clone.execute_task(task).await;
                        });
                    } else {
                        // 如果没有可用的并发槽，重新放回队列
                        let mut queue = self.task_queue.write().await;
                        queue.push(task);
                    }
                } else {
                    // 没有任务，短暂休眠
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        }
    }

    /// 执行单个任务
    async fn execute_task(&self, task: SchedulingTask) {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let start_time = Utc::now();

        // 创建执行记录
        let mut execution_record = ExecutionRecord {
            execution_id: execution_id.clone(),
            task_id: task.task_id.clone(),
            strategy_id: task.strategy_id.clone(),
            status: ExecutionStatus::Running,
            start_time,
            end_time: None,
            execution_result: None,
            error: None,
        };

        // 记录开始执行
        {
            let mut records = self.execution_records.write().await;
            records.insert(execution_id.clone(), execution_record.clone());
        }

        // 获取策略实例
        let strategy_result = self.registry.get_strategy(&task.strategy_id).await;
        
        let execution_result = if let Some(strategy) = strategy_result {
            // 执行策略
            let strategy_guard = strategy.read().await;
            strategy_guard.execute_opportunity(&task.opportunity.opportunity).await
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Strategy {} not found", task.strategy_id)
            ))
        };

        // 更新执行记录
        execution_record.end_time = Some(Utc::now());
        match execution_result {
            Ok(result) => {
                execution_record.status = ExecutionStatus::Completed;
                execution_record.execution_result = Some(result.clone());
                
                // 记录到失效检测器
                let _ = self.failure_detector.record_execution(result).await;
            }
            Err(e) => {
                execution_record.status = ExecutionStatus::Failed;
                execution_record.error = Some(e.to_string());
            }
        }

        // 保存执行记录
        {
            let mut records = self.execution_records.write().await;
            records.insert(execution_id, execution_record.clone());
        }

        // 更新性能统计
        self.update_performance_stats(&execution_record).await;

        tracing::debug!(
            task_id = %task.task_id,
            strategy_id = %task.strategy_id,
            status = ?execution_record.status,
            "Task execution completed"
        );
    }

    /// 健康检查循环
    async fn run_health_check_loop(&self) {
        let mut interval = tokio::time::interval(
            tokio::time::Duration::from_secs(
                self.config.read().await.load_balancing.health_check_interval_seconds as u64
            )
        );

        while *self.is_running.read().await {
            interval.tick().await;
            
            if let Err(e) = self.perform_health_check().await {
                tracing::error!(error = %e, "Error in health check");
            }
        }
    }

    /// 执行健康检查
    async fn perform_health_check(&self) -> Result<(), StrategyError> {
        let registrations = self.registry.get_registry_stats().await;
        
        // 检查每个策略的健康状态
        for strategy_type in [StrategyType::InterExchange, StrategyType::Triangular] {
            let strategy_ids = self.registry.get_strategies_by_type(strategy_type).await;
            
            for strategy_id in strategy_ids {
                if let Some(strategy) = self.registry.get_strategy(&strategy_id).await {
                    let strategy_guard = strategy.read().await;
                    if let Err(_) = strategy_guard.health_check().await {
                        // 更新熔断器状态
                        self.update_circuit_breaker_state(&strategy_id, false).await;
                    } else {
                        self.update_circuit_breaker_state(&strategy_id, true).await;
                    }
                }
            }
        }

        {
            let mut last_check = self.last_health_check.write().await;
            *last_check = Utc::now();
        }

        Ok(())
    }

    /// 计算任务优先级
    async fn calculate_task_priority(&self, opportunity: &WeightedOpportunity) -> u32 {
        let config = self.config.read().await;
        
        if !config.priority_scheduling.enabled {
            return 1;
        }

        let mut priority = 1u32;

        // 基于策略类型的权重
        if let Some(&weight) = config.priority_scheduling.strategy_type_weights.get(&opportunity.opportunity.strategy_type) {
            priority = (priority as f64 * weight) as u32;
        }

        // 基于利润阈值的优先级
        let profit_percentage = opportunity.opportunity.profit_percentage;
        for (threshold, prio) in &config.priority_scheduling.profit_threshold_priorities {
            if profit_percentage >= *threshold {
                priority = priority.max(*prio);
                break;
            }
        }

        // 延迟惩罚
        let age_minutes = (Utc::now() - opportunity.created_at).num_minutes() as f64;
        let latency_penalty = (age_minutes * config.priority_scheduling.latency_penalty_factor) as u32;
        priority = priority.saturating_sub(latency_penalty);

        // 公平性调整
        let fairness_boost = (age_minutes * config.priority_scheduling.fairness_factor) as u32;
        priority += fairness_boost;

        priority.max(1)
    }

    /// 估算执行时间
    async fn estimate_execution_time(&self, strategy_id: &str) -> u64 {
        let stats = self.performance_stats.read().await;
        if let Some(stat) = stats.get(strategy_id) {
            stat.avg_execution_time_ms as u64
        } else {
            1000 // 默认1秒
        }
    }

    /// 检查熔断器状态
    async fn is_circuit_breaker_open(&self, strategy_id: &str) -> bool {
        let stats = self.performance_stats.read().await;
        if let Some(stat) = stats.get(strategy_id) {
            stat.circuit_breaker_state == CircuitBreakerState::Open
        } else {
            false
        }
    }

    /// 更新性能统计
    async fn update_performance_stats(&self, record: &ExecutionRecord) {
        let mut stats = self.performance_stats.write().await;
        let stat = stats.entry(record.strategy_id.clone()).or_insert_with(|| {
            StrategyPerformanceStats {
                strategy_id: record.strategy_id.clone(),
                total_executions: 0,
                successful_executions: 0,
                avg_execution_time_ms: 0.0,
                avg_profit: 0.0,
                last_execution: None,
                current_load: 0,
                circuit_breaker_state: CircuitBreakerState::Closed,
            }
        });

        stat.total_executions += 1;
        
        if record.status == ExecutionStatus::Completed {
            stat.successful_executions += 1;
        }

        if let (Some(start), Some(end)) = (Some(record.start_time), record.end_time) {
            let execution_time = (end - start).num_milliseconds() as f64;
            stat.avg_execution_time_ms = (stat.avg_execution_time_ms * (stat.total_executions - 1) as f64 + execution_time) / stat.total_executions as f64;
        }

        if let Some(result) = &record.execution_result {
            stat.avg_profit = (stat.avg_profit * (stat.successful_executions - 1) as f64 + result.profit_realized) / stat.successful_executions as f64;
        }

        stat.last_execution = Some(Utc::now());
    }

    /// 更新熔断器状态
    async fn update_circuit_breaker_state(&self, strategy_id: &str, healthy: bool) {
        let config = self.config.read().await;
        if !config.circuit_breaker.enabled {
            return;
        }

        let mut stats = self.performance_stats.write().await;
        if let Some(stat) = stats.get_mut(strategy_id) {
            if healthy {
                match stat.circuit_breaker_state {
                    CircuitBreakerState::Open => {
                        stat.circuit_breaker_state = CircuitBreakerState::HalfOpen;
                    }
                    CircuitBreakerState::HalfOpen => {
                        stat.circuit_breaker_state = CircuitBreakerState::Closed;
                    }
                    _ => {}
                }
            } else {
                let failure_rate = 1.0 - (stat.successful_executions as f64 / stat.total_executions as f64);
                if failure_rate > config.circuit_breaker.failure_rate_threshold 
                    && stat.total_executions >= config.circuit_breaker.minimum_requests as u64 {
                    stat.circuit_breaker_state = CircuitBreakerState::Open;
                }
            }
        }
    }

    /// 克隆用于异步任务
    fn clone_for_task(&self) -> StrategySchedulerClone {
        StrategySchedulerClone {
            config: Arc::clone(&self.config),
            registry: Arc::clone(&self.registry),
            opportunity_pool: Arc::clone(&self.opportunity_pool),
            failure_detector: Arc::clone(&self.failure_detector),
            task_queue: Arc::clone(&self.task_queue),
            execution_records: Arc::clone(&self.execution_records),
            performance_stats: Arc::clone(&self.performance_stats),
            concurrency_semaphore: Arc::clone(&self.concurrency_semaphore),
            is_running: Arc::clone(&self.is_running),
            last_health_check: Arc::clone(&self.last_health_check),
        }
    }
}

/// 用于异步任务的调度器克隆
#[derive(Clone)]
struct StrategySchedulerClone {
    config: Arc<RwLock<SchedulerConfig>>,
    registry: Arc<StrategyRegistry>,
    opportunity_pool: Arc<GlobalOpportunityPool>,
    failure_detector: Arc<StrategyFailureDetector>,
    task_queue: Arc<RwLock<BinaryHeap<SchedulingTask>>>,
    execution_records: Arc<RwLock<HashMap<String, ExecutionRecord>>>,
    performance_stats: Arc<RwLock<HashMap<String, StrategyPerformanceStats>>>,
    concurrency_semaphore: Arc<Semaphore>,
    is_running: Arc<RwLock<bool>>,
    last_health_check: Arc<RwLock<DateTime<Utc>>>,
}

impl StrategySchedulerClone {
    async fn run_scheduling_loop(&self) {
        // 实现与主结构相同的调度逻辑
    }

    async fn run_task_executor(&self) {
        // 实现与主结构相同的任务执行逻辑
    }

    async fn run_health_check_loop(&self) {
        // 实现健康检查逻辑
    }
}


/// 调度统计信息
#[derive(Debug, Clone)]
pub struct SchedulingStatistics {
    pub total_tasks_scheduled: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time_ms: f64,
    pub current_queue_size: usize,
    pub pending_tasks: usize,
    pub total_executions: u64,
    pub completed_executions: u64,
    pub failed_executions: u64,
    pub success_rate: f64,
    pub active_strategies: usize,
    pub circuit_breaker_open_count: usize,
}

impl Default for SchedulingStatistics {
    fn default() -> Self {
        Self {
            total_tasks_scheduled: 0,
            tasks_completed: 0,
            tasks_failed: 0,
            average_execution_time_ms: 0.0,
            current_queue_size: 0,
            pending_tasks: 0,
            total_executions: 0,
            completed_executions: 0,
            failed_executions: 0,
            success_rate: 0.0,
            active_strategies: 0,
            circuit_breaker_open_count: 0,
        }
    }
}

impl StrategySchedulerClone {
    async fn execute_task(&self, _task: SchedulingTask) {
        // 实现任务执行逻辑
    }
}

// End of scheduler module

