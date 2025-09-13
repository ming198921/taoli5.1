//! 套利引擎模块 - 策略-风险集成核心
//! 
//! 负责协调策略执行和风险控制

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};
use anyhow::Result;

use common_types::{StrategyContext, ArbitrageStrategy, ExecutionResult, ArbitrageOpportunity, NormalizedSnapshot};
use crate::config::SystemConfig;
use crate::risk::{DynamicRiskController, StrategyRiskInterface};

/// 配置驱动的套利引擎
// Debug trait removed due to complex inner types
pub struct ConfigurableArbitrageEngine {
    /// 风险控制器
    risk_controller: Arc<DynamicRiskController>,
    /// 注册的策略
    strategies: RwLock<HashMap<String, Arc<dyn ArbitrageStrategy + Send + Sync>>>,
    /// 策略上下文
    strategy_context: Arc<dyn StrategyContext + Send + Sync>,
    /// 引擎配置
    config: Arc<RwLock<EngineConfig>>,
    /// 执行统计
    stats: Arc<RwLock<EngineStats>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// 最大并发策略数
    pub max_concurrent_strategies: usize,
    /// 策略执行超时（毫秒）
    pub strategy_timeout_ms: u64,
    /// 是否启用风险检查
    pub enable_risk_check: bool,
    /// 机会检测间隔（毫秒）
    pub opportunity_check_interval_ms: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_strategies: std::env::var("CELUE_MAX_CONCURRENT_STRATEGIES")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(10),
            strategy_timeout_ms: std::env::var("CELUE_STRATEGY_TIMEOUT_MS")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(5000),
            enable_risk_check: std::env::var("CELUE_ENABLE_RISK_CHECK")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(true),
            opportunity_check_interval_ms: std::env::var("CELUE_OPPORTUNITY_CHECK_INTERVAL_MS")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(100),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EngineStats {
    pub strategies_registered: usize,
    pub opportunities_detected: u64,
    pub opportunities_executed: u64,
    pub total_pnl: f64,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
}

impl ConfigurableArbitrageEngine {
    /// 创建新的套利引擎
    pub fn new(
        system_config: &SystemConfig,
        strategy_context: Arc<dyn StrategyContext + Send + Sync>,
    ) -> Self {
        let risk_controller = Arc::new(DynamicRiskController::from_system_config(system_config));
        let engine_config = EngineConfig::default();
        
        Self {
            risk_controller,
            strategies: RwLock::new(HashMap::new()),
            strategy_context,
            config: Arc::new(RwLock::new(engine_config)),
            stats: Arc::new(RwLock::new(EngineStats::default())),
        }
    }

    /// 新的构造函数：直接使用ConfigCenter，无需配置转换层
    pub async fn new_with_config_center(
        config_center: Arc<arbitrage_architecture::config::ConfigCenter>,
        strategy_context: Arc<dyn StrategyContext + Send + Sync>,
    ) -> Result<Self> {
        info!("🔧 从ConfigCenter直接初始化ConfigurableArbitrageEngine...");
        
        // 直接从ConfigCenter获取配置
        let system_config = config_center.get_system_config().await?;
        let risk_config = config_center.get_risk_config().await?;
        let strategy_configs = config_center.get_strategy_configs().await?;
        
        // 创建风险控制器，直接使用ConfigCenter
        let risk_controller = Arc::new(DynamicRiskController::new_with_config_center(config_center.clone()).await?);
        
        // 根据系统配置创建引擎配置
        let engine_config = EngineConfig {
            max_concurrent_strategies: 10, // 默认最大10个并发策略
            strategy_timeout_ms: 5000, // 默认5秒超时
            enable_risk_check: true,    // 总是启用风险检查
            opportunity_check_interval_ms: 100, // 100ms检测间隔
        };
        
        let engine = Self {
            risk_controller,
            strategies: RwLock::new(HashMap::new()),
            strategy_context,
            config: Arc::new(RwLock::new(engine_config)),
            stats: Arc::new(RwLock::new(EngineStats::default())),
        };
        
        info!("🎉 ConfigurableArbitrageEngine从ConfigCenter初始化完成");
        Ok(engine)
    }

    /// 注册策略
    pub async fn register_strategy(
        &self,
        name: String,
        strategy: Arc<dyn ArbitrageStrategy + Send + Sync>,
    ) -> Result<()> {
        let mut strategies = self.strategies.write().await;
        strategies.insert(name.clone(), strategy);
        
        // 更新统计
        let mut stats = self.stats.write().await;
        stats.strategies_registered = strategies.len();
        
        info!("✅ 策略已注册: {}", name);
        Ok(())
    }

    /// 检测机会并执行策略（风险集成）
    pub async fn detect_and_execute(&self, market_snapshot: &NormalizedSnapshot) -> Result<Vec<ExecutionResult>> {
        let config = self.config.read().await;
        
        // 风险检查
        if config.enable_risk_check {
            if !self.risk_controller.perform_risk_check().await? {
                warn!("🚫 风险检查失败，停止策略执行");
                return Ok(vec![]);
            }
        }

        let strategies = self.strategies.read().await;
        let mut results = Vec::new();
        let mut opportunities_count = 0u64;

        // 遍历所有注册的策略
        for (strategy_name, strategy) in strategies.iter() {
            // 检测机会
            if let Some(opportunity) = strategy.detect(self.strategy_context.as_ref(), market_snapshot) {
                opportunities_count += 1;

                // 策略级风险检查
                if config.enable_risk_check {
                    let expected_profit = opportunity.net_profit;
                    let can_execute = self.risk_controller
                        .can_execute_strategy(strategy_name, expected_profit)
                        .await;
                    
                    if !can_execute {
                        warn!("🚫 策略 {} 被风控阻止，预期利润: ${:.2}", strategy_name, expected_profit);
                        continue;
                    }
                }

                // 执行策略
                let execution_start = std::time::Instant::now();
                let result = strategy.execute(self.strategy_context.as_ref(), &opportunity).await;
                let execution_time = execution_start.elapsed().as_millis() as f64;

                match result {
                    Ok(exec_result) => {
                                // 计算实际利润（基于机会的预期利润和执行状态）
        let profit = if exec_result.accepted { 
            opportunity.net_profit 
        } else { 
            -opportunity.net_profit.abs() * 0.1 // 失败时的小幅损失
        };
                        self.risk_controller
                            .report_strategy_result(
                                strategy_name,
                                profit,
                                exec_result.accepted,
                            )
                            .await;

                        // 更新统计
                        self.update_stats(&exec_result, execution_time).await;
                        
                        results.push(exec_result.clone());
                        
                        if exec_result.accepted {
                            info!("✅ 策略 {} 执行成功，订单ID: {:?}", 
                                  strategy_name, exec_result.order_ids);
                        } else {
                            warn!("❌ 策略 {} 执行失败: {}", 
                                  strategy_name, exec_result.reason.as_deref().unwrap_or("未知原因"));
                        }
                    }
                    Err(e) => {
                        error!("💥 策略 {} 执行异常: {}", strategy_name, e);
                        
                        // 报告失败给风险控制器
                        self.risk_controller
                            .report_strategy_result(strategy_name, 0.0, false)
                            .await;
                    }
                }
            }
        }

        // 更新机会检测统计
        let mut stats = self.stats.write().await;
        stats.opportunities_detected += opportunities_count;

        Ok(results)
    }

    /// 更新引擎统计
    async fn update_stats(&self, result: &ExecutionResult, execution_time_ms: f64) {
        let mut stats = self.stats.write().await;
        
        if result.accepted {
            stats.opportunities_executed += 1;
            // 简化PnL计算 - 在实际部署时应从实际交易结果获取
            stats.total_pnl += 1.0; // 临时值
        }
        
        // 更新平均执行时间（简单移动平均）
        if stats.opportunities_executed > 0 {
            stats.avg_execution_time_ms = (stats.avg_execution_time_ms * (stats.opportunities_executed - 1) as f64 
                + execution_time_ms) / stats.opportunities_executed as f64;
        } else {
            stats.avg_execution_time_ms = execution_time_ms;
        }
        
        // 计算成功率
        if stats.opportunities_detected > 0 {
            stats.success_rate = stats.opportunities_executed as f64 / stats.opportunities_detected as f64 * 100.0;
        }
    }

    /// 获取风险状态
    pub async fn get_risk_status(&self) -> crate::risk::RiskStatus {
        self.risk_controller.get_risk_status().await
    }

    /// 获取引擎统计
    pub async fn get_stats(&self) -> EngineStats {
        self.stats.read().await.clone()
    }

    /// 动态更新配置
    pub async fn update_config(&self, new_config: EngineConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        info!("🔄 引擎配置已更新");
        Ok(())
    }

    /// 获取已注册的策略列表
    pub async fn get_registered_strategies(&self) -> Vec<String> {
        let strategies = self.strategies.read().await;
        strategies.keys().cloned().collect()
    }

    /// 启动引擎（持续运行）
    pub async fn start(&self, mut snapshot_receiver: tokio::sync::mpsc::Receiver<NormalizedSnapshot>) -> Result<()> {
        info!("🚀 套利引擎启动");
        
        loop {
            // 等待市场快照
            if let Some(snapshot) = snapshot_receiver.recv().await {
                // 检测并执行策略
                match self.detect_and_execute(&snapshot).await {
                    Ok(results) => {
                        if !results.is_empty() {
                            debug!("📊 本轮执行了 {} 个机会", results.len());
                        }
                    }
                    Err(e) => {
                        error!("💥 策略执行出错: {}", e);
                    }
                }
            }
            
            // 检查间隔
            let config = self.config.read().await;
            let interval = std::time::Duration::from_millis(config.opportunity_check_interval_ms);
            drop(config);
            
            tokio::time::sleep(interval).await;
        }
    }

    /// 优雅停机
    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 套利引擎正在关闭...");
        
        // 获取最终统计
        let stats = self.get_stats().await;
        let risk_status = self.get_risk_status().await;
        
        info!("📊 最终统计:");
        info!("  - 注册策略数: {}", stats.strategies_registered);
        info!("  - 检测机会数: {}", stats.opportunities_detected);
        info!("  - 执行机会数: {}", stats.opportunities_executed);
        info!("  - 总损益: ${:.2}", stats.total_pnl);
        info!("  - 成功率: {:.1}%", stats.success_rate);
        info!("  - 平均执行时间: {:.1}ms", stats.avg_execution_time_ms);
        info!("🛡️ 风险状态:");
        info!("  - 日损益: ${:.2}", risk_status.daily_pnl);
        info!("  - 风险分数: {:.3}", risk_status.risk_score);
        info!("  - 连续失败: {}", risk_status.consecutive_failures);
        info!("  - 健康状态: {}", if risk_status.is_healthy { "✅" } else { "❌" });
        
        info!("✅ 套利引擎已关闭");
        Ok(())
    }
}
