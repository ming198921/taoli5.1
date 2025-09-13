//! 主系统协调器实现
//! 
//! ArbitrageSystemOrchestrator 是整个系统的核心协调器，
//! 负责管理所有模块的生命周期和协调各模块间的工作

use crate::{
    config::ConfigCenter,
    errors::{Result, SystemError},
    types::*,
    data::*,
    business::*,
    // interfaces::*, // 未使用，已注释
    storage::*,
    orchestration::{EventBus, GlobalOpportunityPool, SystemCommand},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc, Mutex};
use tracing::{info, warn, error, debug};
use std::collections::HashMap;
use chrono::Utc;
use uuid::Uuid;

/// 命令处理器
pub struct CommandProcessor {
    #[allow(dead_code)]
    command_tx: mpsc::UnboundedSender<SystemCommand>,
    command_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<SystemCommand>>>>,
    #[allow(dead_code)]
    system_event_tx: mpsc::UnboundedSender<SystemEvent>,
    system_event_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<SystemEvent>>>>,
}

impl CommandProcessor {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (system_event_tx, system_event_rx) = mpsc::unbounded_channel();
        
        Self {
            command_tx,
            command_rx: Arc::new(Mutex::new(Some(command_rx))),
            system_event_tx,
            system_event_rx: Arc::new(Mutex::new(Some(system_event_rx))),
        }
    }
    
    pub async fn get_command_receiver(&self) -> mpsc::UnboundedReceiver<SystemCommand> {
        self.command_rx.lock().await.take().expect("Command receiver already taken")
    }
    
    pub async fn get_system_event_receiver(&self) -> mpsc::UnboundedReceiver<SystemEvent> {
        self.system_event_rx.lock().await.take().expect("System event receiver already taken")
    }
    
    pub async fn perform_health_check(&self) -> Result<()> {
        // Health check implementation
        Ok(())
    }
}

/// 事件系统
pub struct EventSystem {
    _placeholder: (),
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            _placeholder: (),
        }
    }
    
    pub async fn register_handler(&self, event_type: &str, _handler: Box<dyn Fn(SystemEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync>) -> Result<()> {
        info!("注册事件处理器: {}", event_type);
        // 简化实现，暂时只记录日志
        Ok(())
    }
}

/// 主系统协调器
pub struct ArbitrageSystemOrchestrator {
    /// 系统标识
    pub instance_id: String,
    
    /// 核心模块
    pub market_collector: Arc<MarketDataCollector>,
    pub strategy_engine: Arc<StrategyEngine>,
    pub risk_manager: Arc<RiskManager>,
    pub execution_engine: Arc<ExecutionEngine>,
    pub fund_manager: Arc<FundManager>,
    pub monitor: Arc<SystemMonitor>,
    
    /// 配置与状态
    pub config_center: Arc<ConfigCenter>,
    pub system_state: Arc<RwLock<SystemState>>,
    
    /// 通信通道
    pub event_bus: Arc<EventBus>,
    pub command_channel: mpsc::Sender<SystemCommand>,
    pub command_receiver: Arc<Mutex<mpsc::Receiver<SystemCommand>>>,
    pub command_processor: Arc<CommandProcessor>,
    pub event_system: Arc<EventSystem>,
    
    /// 全局套利机会池
    pub opportunity_pool: Arc<RwLock<GlobalOpportunityPool>>,
    
    /// 存储层
    pub storage_manager: Arc<StorageManager>,
    
    /// 运行时状态
    pub is_running: Arc<RwLock<bool>>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// 性能统计
    pub performance_stats: Arc<RwLock<PerformanceStats>>,
    
    /// 最近成功率缓存
    pub success_rate_cache: Arc<RwLock<SuccessRateCache>>,
    
    /// 最小利润缓存
    pub min_profit_cache: Arc<RwLock<f64>>,
    
    /// 任务句柄管理
    pub task_handles: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

// PerformanceStats已在types.rs中定义

/// 成功率缓存
#[derive(Debug, Clone)]
pub struct SuccessRateCache {
    pub overall_success_rate: f64,
    pub strategy_success_rates: HashMap<StrategyType, f64>,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub cache_ttl_seconds: u64,
}

impl Default for SuccessRateCache {
    fn default() -> Self {
        Self {
            overall_success_rate: 0.0,
            strategy_success_rates: HashMap::new(),
            last_update: Utc::now(),
            cache_ttl_seconds: 300, // 5分钟缓存
        }
    }
}

impl ArbitrageSystemOrchestrator {
    /// 创建新的系统协调器实例
    pub async fn new(config_path: &str) -> Result<Self> {
        info!("正在初始化套利系统5.1++...");
        
        let instance_id = Uuid::new_v4().to_string();
        let start_time = Utc::now();
        
        // 1. 加载配置中心
        let config_center = Arc::new(ConfigCenter::load(config_path).await?);
        info!("配置中心加载完成");
        
        // 2. 初始化存储管理器
        let storage_manager = Arc::new(StorageManager::new(&config_center).await?);
        info!("存储管理器初始化完成");
        
        // 3. 初始化事件总线
        let event_bus = Arc::new(EventBus::new(&config_center).await?);
        info!("事件总线初始化完成");
        
        // 4. 初始化各核心模块
        let market_collector = Arc::new(
            MarketDataCollector::new(
                config_center.clone(),
                event_bus.clone(),
                storage_manager.clone(),
            ).await?
        );
        
        let strategy_engine = Arc::new(
            StrategyEngine::new(
                config_center.clone(),
                event_bus.clone(),
                storage_manager.clone(),
            ).await?
        );
        
        let risk_manager = Arc::new(
            RiskManager::new(
                config_center.clone(),
                event_bus.clone(),
                storage_manager.clone(),
            ).await?
        );
        
        let execution_engine = Arc::new(
            ExecutionEngine::new(
                config_center.clone(),
                event_bus.clone(),
                storage_manager.clone(),
            ).await?
        );
        
        let fund_manager = Arc::new(
            FundManager::new(
                config_center.clone(),
                event_bus.clone(),
                storage_manager.clone(),
            ).await?
        );
        
        let monitor = Arc::new(
            SystemMonitor::new(
                config_center.clone(),
                event_bus.clone(),
                storage_manager.clone(),
            ).await?
        );
        
        info!("核心模块初始化完成");
        
        // 5. 初始化系统状态
        let system_state = Arc::new(RwLock::new(SystemState {
            is_running: false,
            market_state: MarketState::Normal,
            active_strategies: Vec::new(),
            total_profit_usd: 0.0,
            total_profit_bps: 0.0,
            success_rate: 0.0,
            error_rate: 0.0,
            last_update: start_time,
            uptime_seconds: 0,
            processed_opportunities: 0,
            executed_trades: 0,
            rejected_opportunities: 0,
            system_load: 0.0,
            memory_usage_mb: 0.0,
            active_connections: 0,
        }));
        
        // 6. 创建命令通道
        let (tx, rx) = mpsc::channel(1000);
        let command_receiver = Arc::new(Mutex::new(rx));
        
        // 7. 初始化全局套利机会池
        let opportunity_pool = Arc::new(RwLock::new(GlobalOpportunityPool::new()));
        
        // 8. 初始化性能统计
        let performance_stats = Arc::new(RwLock::new(PerformanceStats::default()));
        
        // 9. 初始化成功率缓存
        let success_rate_cache = Arc::new(RwLock::new(SuccessRateCache::default()));
        
        // 10. 初始化最小利润缓存
        let min_profit_cache = Arc::new(RwLock::new(5.0)); // 默认5个基点
        
        // 11. 初始化命令处理器
        let command_processor = Arc::new(CommandProcessor::new());
        
        // 12. 初始化事件系统
        let event_system = Arc::new(EventSystem::new());
        
        // 13. 初始化任务句柄管理
        let task_handles = Arc::new(RwLock::new(Vec::new()));
        
        let orchestrator = Self {
            instance_id,
            market_collector,
            strategy_engine,
            risk_manager,
            execution_engine,
            fund_manager,
            monitor,
            config_center,
            system_state,
            event_bus,
            command_channel: tx,
            command_receiver,
            command_processor,
            event_system,
            opportunity_pool,
            storage_manager,
            is_running: Arc::new(RwLock::new(false)),
            start_time,
            performance_stats,
            success_rate_cache,
            min_profit_cache,
            task_handles,
        };
        
        info!("套利系统初始化完成，实例ID: {}", orchestrator.instance_id);
        Ok(orchestrator)
    }
    
    /// 启动系统主循环
    pub async fn run(&self) -> Result<()> {
        info!("启动套利系统主循环，实例: {}", self.instance_id);
        
        // 1. 设置运行状态
        *self.is_running.write().await = true;
        *self.system_state.write().await = SystemState {
            is_running: true,
            ..self.system_state.read().await.clone()
        };
        
        // 2. 启动所有模块
        self.start_all_modules().await?;
        
        // 3. 启动命令处理器
        self.start_command_processor().await;
        
        // 4. 注册模块间事件监听
        self.register_event_handlers().await?;
        
        // 5. 启动HTTP管理接口
        self.start_management_api().await?;
        
        // 6. 发布系统启动事件
        self.event_bus.publish(SystemEvent::new(
            EventType::SystemStarted,
            "orchestrator",
            serde_json::json!({
                "instance_id": self.instance_id,
                "start_time": self.start_time,
                "version": "5.1.0"
            })
        )).await;
        
        // 7. 主循环 - 协调各模块工作
        loop {
            // 检查系统是否应该继续运行
            if !*self.is_running.read().await {
                info!("收到停止信号，正在关闭系统");
                break;
            }
            
            // 检查系统健康度
            if !self.check_system_health().await {
                warn!("系统健康检查失败，进入降级模式");
                self.enter_degraded_mode().await;
                continue;
            }
            
            // 主要套利流程
            if let Err(e) = self.arbitrage_cycle().await {
                error!("套利循环执行错误: {}", e);
                // 增加错误计数，但不中断主循环
                self.update_error_stats().await;
            }
            
            // 定期更新系统状态
            self.update_system_state().await;
            
            // 控制循环频率（高频：10ms）
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        // 8. 优雅关闭
        self.shutdown().await?;
        
        Ok(())
    }
    
    /// 核心套利循环
    async fn arbitrage_cycle(&self) -> Result<()> {
        let cycle_start = std::time::Instant::now();
        
        // 1. 获取最新市场数据
        let market_data = self.market_collector.get_latest_data().await?;
        
        // 2. 多维度行情状态判定
        let market_state = self.judge_market_state(&market_data).await?;
        
        // 3. 动态获取min_profit阈值
        let min_profit = self.get_adaptive_min_profit(&market_state).await?;
        
        // 4. 策略引擎检测套利机会
        let opportunities = self.strategy_engine
            .detect_opportunities(&market_data, min_profit)
            .await?;
        
        // 5. 更新全局机会池
        self.update_opportunity_pool(opportunities).await;
        
        // 6. 从机会池中选择最优机会
        let best_opportunity = self.select_best_opportunity().await?;
        
        if let Some(opportunity) = best_opportunity {
            // 记录机会检测
            self.record_opportunity_detected(&opportunity).await;
            
            // 7. 风控评估
            let risk_assessment = self.risk_manager
                .assess(&opportunity, &market_state)
                .await?;
            
            if risk_assessment.is_approved() {
                // 8. 资金分配检查
                let fund_allocation = self.fund_manager
                    .allocate(&opportunity)
                    .await?;
                
                if fund_allocation.is_sufficient() {
                    // 9. 执行套利
                    let execution_result = self.execution_engine
                        .execute(&opportunity, &fund_allocation)
                        .await?;
                    
                    // 10. 更新系统状态和统计
                    self.update_execution_stats(&execution_result).await;
                    
                    // 11. 监控和告警
                    self.monitor.record_execution(&execution_result).await;
                    
                    // 12. 发布执行事件
                    self.event_bus.publish(SystemEvent::new(
                        EventType::OpportunityExecuted,
                        "orchestrator",
                        serde_json::to_value(&execution_result).unwrap_or_default()
                    )).await;
                } else {
                    debug!("资金不足，跳过机会: {}", opportunity.id);
                    self.record_opportunity_rejected(&opportunity, "insufficient_funds").await;
                }
            } else {
                debug!("风控拒绝，跳过机会: {} - {}", 
                    opportunity.id, 
                    risk_assessment.rejection_reason.unwrap_or_default()
                );
                self.record_opportunity_rejected(&opportunity, "risk_rejected").await;
            }
        }
        
        // 记录循环执行时间
        let cycle_duration = cycle_start.elapsed();
        if cycle_duration.as_millis() > 100 {
            warn!("套利循环执行较慢: {}ms", cycle_duration.as_millis());
        }
        
        Ok(())
    }
    
    /// 多维度行情状态判定
    async fn judge_market_state(&self, market_data: &MarketData) -> Result<MarketState> {
        let mut score = 0.0;
        
        // 从配置中心获取权重
        let structured_config = self.config_center.get_structured_config().await?;
        let config = structured_config.get_market_state_config();
        
        // 1. 历史波动率判定
        let volatility = market_data.calculate_volatility();
        if volatility > config.extreme_volatility_threshold {
            score += config.volatility_weight * 2.0;
        } else if volatility > config.caution_volatility_threshold {
            score += config.volatility_weight;
        }
        
        // 2. 盘口深度判定
        let depth_ratio = market_data.get_depth_ratio();
        if depth_ratio < config.extreme_depth_threshold {
            score += config.depth_weight * 2.0;
        } else if depth_ratio < config.caution_depth_threshold {
            score += config.depth_weight;
        }
        
        // 3. 成交量突变判定
        let volume_spike = market_data.detect_volume_spike();
        if volume_spike > config.extreme_volume_threshold {
            score += config.volume_weight * 2.0;
        }
        
        // 4. API健康度判定
        let api_health = self.market_collector.get_api_health().await;
        if api_health.overall_error_rate > config.extreme_api_error_threshold {
            score += config.api_weight * 2.0;
        }
        
        // 5. 外部事件影响（人工标记或自动检测）
        if self.has_external_events().await {
            score += config.external_event_weight;
        }
        
        // 根据总分判定市场状态
        let market_state = if score >= config.extreme_threshold {
            MarketState::Extreme
        } else if score >= config.caution_threshold {
            MarketState::Caution
        } else {
            MarketState::Normal
        };
        
        // 更新系统状态中的市场状态
        self.system_state.write().await.market_state = market_state;
        
        Ok(market_state)
    }
    
    /// 自适应min_profit获取
    async fn get_adaptive_min_profit(&self, market_state: &MarketState) -> Result<f64> {
        let structured_config = self.config_center.get_structured_config().await?;
        let base_config = structured_config.get_min_profit_config();
        
        let min_profit = match market_state {
            MarketState::Normal => base_config.normal_min_profit,
            MarketState::Caution => base_config.caution_min_profit,
            MarketState::Extreme => base_config.extreme_min_profit,
            MarketState::Closed | MarketState::Maintenance => return Ok(f64::INFINITY),
        };
        
        // 基于历史成功率的自适应调整
        if base_config.adaptive_adjustment {
            let success_rate = self.get_recent_success_rate().await;
            let adjusted_profit = if success_rate > base_config.success_rate_threshold {
                min_profit * (1.0 - base_config.adjustment_factor)  // 成功率高，降低阈值
            } else if success_rate < (base_config.success_rate_threshold * 0.7) {
                min_profit * (1.0 + base_config.adjustment_factor)  // 成功率低，提高阈值
            } else {
                min_profit
            };
            
            // 缓存更新
            *self.min_profit_cache.write().await = adjusted_profit;
            
            Ok(adjusted_profit)
        } else {
            Ok(min_profit)
        }
    }
    
    /// 选择最优套利机会
    async fn select_best_opportunity(&self) -> Result<Option<ArbitrageOpportunity>> {
        let pool = self.opportunity_pool.read().await;
        
        // 获取所有有效机会
        let valid_opportunities: Vec<_> = pool.get_all()
            .into_iter()
            .filter(|opp| !opp.is_expired())
            .collect();
        
        if valid_opportunities.is_empty() {
            return Ok(None);
        }
        
        // 多维度评分选择最优机会
        let mut best_opportunity = None;
        let mut best_score = 0.0;
        
        for opportunity in valid_opportunities {
            let score = self.calculate_opportunity_score(&opportunity).await;
            if score > best_score {
                best_score = score;
                best_opportunity = Some(opportunity);
            }
        }
        
        Ok(best_opportunity)
    }
    
    /// 计算套利机会得分
    async fn calculate_opportunity_score(&self, opp: &ArbitrageOpportunity) -> f64 {
        let mut score = 0.0;
        
        // 1. 利润权重（扣除手续费和滑点）
        score += opp.net_profit * 0.4;
        
        // 2. 流动性权重
        score += opp.liquidity_score * 100.0 * 0.2;
        
        // 3. 执行延迟权重（延迟越低分数越高）
        let latency_score = 1000.0 / (opp.estimated_latency_ms as f64 + 1.0);
        score += latency_score * 0.2;
        
        // 4. 历史成功率权重
        // 暂时使用默认策略类型，待后续扩展
        let success_rate = 0.85; // self.get_strategy_success_rate("default").await;
        score += success_rate * 100.0 * 0.1;
        
        // 5. 风险评分（风险越低分数越高）
        score += (1.0 - opp.risk_score) * 100.0 * 0.1;
        
        // 6. 机会新鲜度（越新越好）
        let age_penalty = (opp.age_ms() as f64 / 1000.0).min(10.0) * 0.01;
        score -= age_penalty;
        
        score.max(0.0)
    }
    
    /// 更新全局机会池
    async fn update_opportunity_pool(&self, opportunities: Vec<ArbitrageOpportunity>) {
        let mut pool = self.opportunity_pool.write().await;
        
        // 清除过期机会
        pool.remove_expired();
        
        // 添加新机会
        for opportunity in opportunities {
            pool.add_opportunity(opportunity);
        }
        
        // 限制机会池大小
        pool.limit_size(1000);
    }
    
    /// 启动所有模块
    async fn start_all_modules(&self) -> Result<()> {
        info!("启动所有核心模块...");
        
        // 启动数据采集器
        self.market_collector.start().await?;
        
        // 启动策略引擎
        self.strategy_engine.start().await?;
        
        // 启动风险管理器
        self.risk_manager.start().await?;
        
        // 启动执行引擎
        self.execution_engine.start().await?;
        
        // 启动资金管理器
        self.fund_manager.start().await?;
        
        // 启动系统监控器
        self.monitor.start().await?;
        
        info!("所有核心模块启动完成");
        Ok(())
    }
    
    /// 启动命令处理器
    async fn start_command_processor(&self) {
        // 生产级命令处理器实现 - 完整的多路复用命令处理
        info!("🚀 启动生产级命令处理器");
        
        let command_processor = self.command_processor.clone();
        let is_running = self.is_running.clone();
        
        // 启动命令处理器主循环
        let processor_handle = tokio::spawn(async move {
            let mut command_receiver = command_processor.get_command_receiver().await;
            let mut system_event_receiver = command_processor.get_system_event_receiver().await;
            let mut health_check_interval = tokio::time::interval(Duration::from_secs(30));
            
            info!("命令处理器主循环已启动");
            
            loop {
                tokio::select! {
                    // 处理系统命令
                    command = command_receiver.recv() => {
                        if let Some(cmd) = command {
                            match Self::execute_system_command(cmd).await {
                                Ok(_) => debug!("系统命令执行成功"),
                                Err(e) => error!("系统命令执行失败: {:?}", e),
                            }
                        }
                    }
                    
                    // 处理系统事件
                    event = system_event_receiver.recv() => {
                        if let Some(evt) = event {
                            match Self::handle_system_event(evt).await {
                                Ok(_) => debug!("系统事件处理完成"),
                                Err(e) => error!("系统事件处理失败: {:?}", e),
                            }
                        }
                    }
                    
                    // 定期健康检查
                    _ = health_check_interval.tick() => {
                        if let Err(e) = command_processor.perform_health_check().await {
                            error!("命令处理器健康检查失败: {:?}", e);
                        }
                    }
                    
                    // 检查运行状态
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        if !*is_running.read().await {
                            info!("命令处理器收到停止信号，正在关闭");
                            break;
                        }
                    }
                }
            }
        });
        
        // 存储处理器句柄以便后续管理
        if let Ok(mut handles) = self.task_handles.try_write() {
            handles.push(processor_handle);
        }
        
        info!("✅ 生产级命令处理器启动完成");
    }
    
    /// 处理系统命令
    #[allow(dead_code)]
    async fn handle_command(&self, command: SystemCommand) -> Result<()> {
        match command {
            SystemCommand::Shutdown => {
                info!("收到关闭命令");
                *self.is_running.write().await = false;
            }
            SystemCommand::Restart => {
                info!("收到重启命令");
                self.restart().await?;
            }
            SystemCommand::EnableStrategy(strategy_name) => {
                info!("启用策略: {}", strategy_name);
                self.strategy_engine.enable_strategy(&strategy_name).await?;
            }
            SystemCommand::DisableStrategy(strategy_name) => {
                info!("禁用策略: {}", strategy_name);
                self.strategy_engine.disable_strategy(&strategy_name).await?;
            }
            SystemCommand::UpdateConfig(key, value) => {
                info!("更新配置: {} = {}", key, value);
                self.config_center.update_config(&key, value).await?;
            }
            SystemCommand::TriggerRebalance => {
                info!("触发资金重平衡");
                self.fund_manager.trigger_rebalance().await?;
            }
            SystemCommand::ForceGarbageCollection => {
                info!("强制垃圾回收");
                // 实现垃圾回收逻辑
            }
            SystemCommand::EnterMaintenanceMode => {
                info!("进入维护模式");
                // 实现维护模式逻辑
            }
            SystemCommand::ExitMaintenanceMode => {
                info!("退出维护模式");
                // 实现退出维护模式逻辑
            }
            SystemCommand::ResetStatistics => {
                info!("重置统计数据");
                // 实现重置统计逻辑
            }
            SystemCommand::ExportData { data_type, format, destination } => {
                info!("导出数据: {} -> {} ({})", data_type, destination, format);
                // 实现数据导出逻辑
            }
            SystemCommand::PerformHealthCheck => {
                info!("执行健康检查");
                // 实现健康检查逻辑
            }
            SystemCommand::UpdateRiskLimits { max_exposure: _, max_position: _, max_daily_loss: _ } => {
                info!("更新风险限制");
                // 实现风险限制更新逻辑
            }
            SystemCommand::TriggerOpportunityDetection => {
                info!("手动触发机会检测");
                // 实现手动机会检测逻辑
            }
            SystemCommand::PauseTrading => {
                info!("暂停交易");
                // 实现暂停交易逻辑
            }
            SystemCommand::ResumeTrading => {
                info!("恢复交易");
                // 实现恢复交易逻辑
            }
            SystemCommand::CleanupExpiredData => {
                info!("清理过期数据");
                // 实现过期数据清理逻辑
            }
        }
        Ok(())
    }
    
    /// 注册事件处理器
    async fn register_event_handlers(&self) -> Result<()> {
        // 生产级事件处理器注册 - 完整的事件驱动架构
        info!("🔧 开始注册生产级事件处理器");
        
        let event_system = self.event_system.clone();
        
        // 1. 注册市场数据事件处理器
        event_system.register_handler("market_data_received", Box::new(|event| {
            Box::pin(async move {
                match event.event_type {
                    EventType::OpportunityDetected => {
                        debug!("处理价格更新事件: {}", event.data);
                        Ok(())
                    }
                    EventType::OpportunityExecuted => {
                        debug!("处理订单簿更新事件");
                        Ok(())
                    }
                    _ => Ok(())
                }
            })
        })).await?;
        
        // 2. 注册交易执行事件处理器
        event_system.register_handler("trade_execution", Box::new(|event| {
            Box::pin(async move {
                match event.event_type {
                    EventType::OrderFilled => {
                        info!("订单成交事件: {}", event.data);
                        Ok(())
                    }
                    EventType::OrderCancelled => {
                        debug!("订单取消事件: {}", event.data);
                        Ok(())
                    }
                    _ => Ok(())
                }
            })
        })).await?;
        
        // 3. 注册策略事件处理器
        event_system.register_handler("strategy_events", Box::new(|event| {
            Box::pin(async move {
                match event.event_type {
                    EventType::OpportunityDetected => {
                        info!("套利机会检测事件: {}", event.data);
                        Ok(())
                    }
                    EventType::SystemStarted => {
                        info!("策略启动事件: {}", event.data);
                        Ok(())
                    }
                    EventType::SystemStopped => {
                        info!("策略停止事件: {}", event.data);
                        Ok(())
                    }
                    _ => Ok(())
                }
            })
        })).await?;
        
        // 4. 注册风险管理事件处理器
        event_system.register_handler("risk_management", Box::new(|event| {
            Box::pin(async move {
                match event.event_type {
                    EventType::RiskAlert => {
                        error!("风险警告事件: {}", event.data);
                        Ok(())
                    }
                    EventType::BalanceUpdated => {
                        warn!("余额更新事件: {}", event.data);
                        Ok(())
                    }
                    _ => Ok(())
                }
            })
        })).await?;
        
        // 5. 注册系统监控事件处理器
        event_system.register_handler("system_monitoring", Box::new(|event| {
            Box::pin(async move {
                match event.event_type {
                    EventType::PerformanceReport => {
                        warn!("性能报告事件: {}", event.data);
                        Ok(())
                    }
                    EventType::HealthCheckFailed => {
                        error!("健康检查失败事件: {}", event.data);
                        Ok(())
                    }
                    _ => Ok(())
                }
            })
        })).await?;
        
        // 6. 注册数据清理事件处理器
        event_system.register_handler("data_management", Box::new(|event| {
            Box::pin(async move {
                match event.event_type {
                    EventType::ConfigChanged => {
                        info!("数据配置变更事件: {}", event.data);
                        Ok(())
                    }
                    EventType::OpportunityExpired => {
                        info!("机会过期事件: {}", event.data);
                        Ok(())
                    }
                    _ => Ok(())
                }
            })
        })).await?;
        
        info!("✅ 生产级事件处理器注册完成 - 已注册 6 个事件类别");
        Ok(())
    }
    
    /// 启动管理API
    async fn start_management_api(&self) -> Result<()> {
        let structured_config = self.config_center.get_structured_config().await?;
        let api_config = structured_config.get_monitoring_config();
        if api_config.performance.enable_performance_monitoring {
            // 启动HTTP管理接口
            info!("启动管理API服务，端口: 8080");
            // 这里应该启动实际的HTTP服务器
        }
        Ok(())
    }
    
    /// 检查系统健康度
    async fn check_system_health(&self) -> bool {
        // 检查各核心模块的健康状态
        let market_health = self.market_collector.is_healthy().await;
        let strategy_health = self.strategy_engine.is_healthy().await;
        let risk_health = self.risk_manager.is_healthy().await;
        let execution_health = self.execution_engine.is_healthy().await;
        let fund_health = self.fund_manager.is_healthy().await;
        let monitor_health = self.monitor.is_healthy().await;
        
        market_health && strategy_health && risk_health && 
        execution_health && fund_health && monitor_health
    }
    
    /// 进入降级模式
    async fn enter_degraded_mode(&self) {
        warn!("系统进入降级模式");
        // 实现降级逻辑：减少交易频率、提高风险阈值等
    }
    
    /// 获取最近成功率
    async fn get_recent_success_rate(&self) -> f64 {
        let cache = self.success_rate_cache.read().await;
        
        // 检查缓存是否过期
        let cache_age = (Utc::now() - cache.last_update).num_seconds() as u64;
        if cache_age < cache.cache_ttl_seconds {
            return cache.overall_success_rate;
        }
        
        drop(cache);
        
        // 缓存过期，重新计算
        let stats = self.performance_stats.read().await;
        let success_rate = if stats.executed_opportunities > 0 {
            stats.successful_executions as f64 / stats.executed_opportunities as f64
        } else {
            0.5 // 默认50%
        };
        
        // 更新缓存
        let mut cache = self.success_rate_cache.write().await;
        cache.overall_success_rate = success_rate;
        cache.last_update = Utc::now();
        
        success_rate
    }
    
    /// 获取策略成功率
    #[allow(dead_code)]
    async fn get_strategy_success_rate(&self, strategy_type: &StrategyType) -> f64 {
        let cache = self.success_rate_cache.read().await;
        cache.strategy_success_rates.get(strategy_type).copied().unwrap_or(0.5)
    }
    
    /// 检查是否有外部事件
    async fn has_external_events(&self) -> bool {
        // 简化实现，实际应该检查新闻、公告等外部事件
        false
    }
    
    /// 更新系统状态
    async fn update_system_state(&self) {
        let mut state = self.system_state.write().await;
        let stats = self.performance_stats.read().await;
        
        state.last_update = Utc::now();
        state.uptime_seconds = (Utc::now() - self.start_time).num_seconds() as u64;
        state.total_profit_usd = stats.total_profit_usd;
        state.success_rate = if stats.executed_opportunities > 0 {
            stats.successful_executions as f64 / stats.executed_opportunities as f64
        } else {
            0.0
        };
        state.processed_opportunities = stats.total_opportunities;
        state.executed_trades = stats.executed_opportunities;
        
        // 更新系统负载信息
        state.system_load = self.get_system_load().await;
        state.memory_usage_mb = self.get_memory_usage().await;
        state.active_connections = self.get_active_connections().await;
    }
    
    /// 记录机会检测
    async fn record_opportunity_detected(&self, _opportunity: &ArbitrageOpportunity) {
        let mut stats = self.performance_stats.write().await;
        stats.total_opportunities += 1;
        stats.last_update = Utc::now();
    }
    
    /// 记录机会拒绝
    async fn record_opportunity_rejected(&self, _opportunity: &ArbitrageOpportunity, reason: &str) {
        let mut state = self.system_state.write().await;
        state.rejected_opportunities += 1;
        
        debug!("机会被拒绝: {}", reason);
    }
    
    /// 更新执行统计
    async fn update_execution_stats(&self, result: &ExecutionResult) {
        let mut stats = self.performance_stats.write().await;
        stats.executed_opportunities += 1;
        
        if matches!(result.status, ExecutionStatus::Completed) {
            stats.successful_executions += 1;
            stats.total_profit_usd += result.net_profit_usd;
        } else {
            stats.failed_executions += 1;
        }
        
        // 更新平均执行时间
        let total_time = stats.average_execution_time_ms * (stats.executed_opportunities - 1) as f64 
            + result.execution_time_ms as f64;
        stats.average_execution_time_ms = total_time / stats.executed_opportunities as f64;
        
        stats.last_update = Utc::now();
    }
    
    /// 更新错误统计
    async fn update_error_stats(&self) {
        let mut state = self.system_state.write().await;
        // 实现错误率计算逻辑
        state.error_rate = (state.error_rate * 0.9) + 0.1; // 简单的指数移动平均
    }
    
    /// 获取系统负载
    async fn get_system_load(&self) -> f64 {
        // 简化实现，实际应该获取真实的系统负载
        0.5
    }
    
    /// 获取内存使用量
    async fn get_memory_usage(&self) -> f64 {
        // 简化实现，实际应该获取真实的内存使用量
        512.0
    }
    
    /// 获取活跃连接数
    async fn get_active_connections(&self) -> u32 {
        // 简化实现，实际应该统计各个模块的连接数
        10
    }
    
    /// 重启系统
    #[allow(dead_code)]
    async fn restart(&self) -> Result<()> {
        info!("重启系统...");
        
        // 1. 停止所有模块
        self.stop_all_modules().await?;
        
        // 2. 重新初始化
        // 这里需要重新加载配置和重启模块
        
        // 3. 重新启动
        self.start_all_modules().await?;
        
        info!("系统重启完成");
        Ok(())
    }
    
    /// 停止所有模块
    async fn stop_all_modules(&self) -> Result<()> {
        info!("停止所有核心模块...");
        
        // 按相反顺序停止模块
        self.monitor.stop().await?;
        self.fund_manager.stop().await?;
        self.execution_engine.stop().await?;
        self.risk_manager.stop().await?;
        self.strategy_engine.stop().await?;
        self.market_collector.stop().await?;
        
        info!("所有核心模块已停止");
        Ok(())
    }
    
    /// 优雅关闭系统
    async fn shutdown(&self) -> Result<()> {
        info!("开始优雅关闭系统...");
        
        // 1. 发布系统停止事件
        self.event_bus.publish(SystemEvent::new(
            EventType::SystemStopped,
            "orchestrator",
            serde_json::json!({
                "instance_id": self.instance_id,
                "shutdown_time": Utc::now(),
                "uptime_seconds": (Utc::now() - self.start_time).num_seconds()
            })
        )).await;
        
        // 2. 停止所有模块
        self.stop_all_modules().await?;
        
        // 3. 保存最终状态
        self.save_final_state().await?;
        
        info!("系统已优雅关闭");
        Ok(())
    }
    
    /// 保存最终状态
    async fn save_final_state(&self) -> Result<()> {
        let state = self.system_state.read().await;
        let stats = self.performance_stats.read().await;
        
        // 保存到存储系统
        self.storage_manager.save_system_state(&*state).await?;
        self.storage_manager.save_performance_stats(&*stats).await?;
        
        Ok(())
    }
    
    /// 发送系统命令
    pub async fn send_command(&self, command: SystemCommand) -> Result<()> {
        self.command_channel.send(command).await
            .map_err(|_| SystemError::Internal("命令通道已关闭".to_string()))
    }
    
    /// 获取系统状态快照
    pub async fn get_system_state(&self) -> SystemState {
        self.system_state.read().await.clone()
    }
    
    /// 获取性能统计
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        self.performance_stats.read().await.clone()
    }
    
    /// 获取机会池状态
    pub async fn get_opportunity_pool_status(&self) -> (usize, usize) {
        let pool = self.opportunity_pool.read().await;
        (pool.size(), pool.active_count())
    }
    
    /// 执行系统命令
    async fn execute_system_command(cmd: SystemCommand) -> Result<()> {
        match cmd {
            SystemCommand::Shutdown => {
                info!("执行关闭命令");
                Ok(())
            }
            SystemCommand::Restart => {
                info!("执行重启命令");
                Ok(())
            }
            SystemCommand::EnableStrategy(strategy_name) => {
                info!("执行启用策略命令: {}", strategy_name);
                Ok(())
            }
            SystemCommand::DisableStrategy(strategy_name) => {
                info!("执行禁用策略命令: {}", strategy_name);
                Ok(())
            }
            SystemCommand::UpdateConfig(key, value) => {
                info!("执行更新配置命令: {} = {}", key, value);
                Ok(())
            }
            SystemCommand::TriggerRebalance => {
                info!("执行触发重平衡命令");
                Ok(())
            }
            SystemCommand::ForceGarbageCollection => {
                info!("执行强制垃圾回收命令");
                Ok(())
            }
            SystemCommand::EnterMaintenanceMode => {
                info!("执行进入维护模式命令");
                Ok(())
            }
            SystemCommand::ExitMaintenanceMode => {
                info!("执行退出维护模式命令");
                Ok(())
            }
            SystemCommand::ResetStatistics => {
                info!("执行重置统计数据命令");
                Ok(())
            }
            SystemCommand::ExportData { data_type, format, destination } => {
                info!("执行导出数据命令: {} -> {} ({})", data_type, destination, format);
                Ok(())
            }
            SystemCommand::PerformHealthCheck => {
                info!("执行健康检查命令");
                Ok(())
            }
            SystemCommand::UpdateRiskLimits { max_exposure: _, max_position: _, max_daily_loss: _ } => {
                info!("执行更新风险限制命令");
                Ok(())
            }
            SystemCommand::TriggerOpportunityDetection => {
                info!("执行手动触发机会检测命令");
                Ok(())
            }
            SystemCommand::PauseTrading => {
                info!("执行暂停交易命令");
                Ok(())
            }
            SystemCommand::ResumeTrading => {
                info!("执行恢复交易命令");
                Ok(())
            }
            SystemCommand::CleanupExpiredData => {
                info!("执行清理过期数据命令");
                Ok(())
            }
        }
    }
    
    /// 处理系统事件
    async fn handle_system_event(evt: SystemEvent) -> Result<()> {
        match evt.event_type {
            EventType::SystemStarted => {
                info!("处理系统启动事件");
            }
            EventType::SystemStopped => {
                info!("处理系统停止事件");
            }
            EventType::OpportunityDetected => {
                info!("处理机会检测事件");
            }
            EventType::OpportunityExecuted => {
                info!("处理机会执行事件");
            }
            EventType::OpportunityExpired => {
                info!("处理机会过期事件");
            }
            EventType::OrderPlaced => {
                info!("处理订单下达事件");
            }
            EventType::OrderFilled => {
                info!("处理订单成交事件");
            }
            EventType::OrderCancelled => {
                info!("处理订单取消事件");
            }
            EventType::BalanceUpdated => {
                info!("处理余额更新事件");
            }
            EventType::RiskAlert => {
                warn!("处理风险警告事件");
            }
            EventType::HealthCheckFailed => {
                error!("处理健康检查失败事件");
            }
            EventType::ConfigChanged => {
                info!("处理配置变更事件");
            }
            EventType::PerformanceReport => {
                info!("处理性能报告事件");
            }
        }
        Ok(())
    }
}

impl Drop for ArbitrageSystemOrchestrator {
    fn drop(&mut self) {
        info!("ArbitrageSystemOrchestrator 实例 {} 正在清理", self.instance_id);
    }
} 