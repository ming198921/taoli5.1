//! Shadow Trading System
//! 
//! 影子模式交易系统，提供完整的模拟交易环境
//! 包含虚拟账户管理、交易执行模拟和性能分析

pub mod config;
pub mod virtual_account;
pub mod execution_engine;
pub mod market_simulator;
pub mod performance_analyzer;
pub mod risk_manager;
pub mod order_matching;
pub mod metrics;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug, instrument};

pub use config::ShadowTradingConfig;
pub use virtual_account::{VirtualAccount, VirtualPosition, VirtualBalance};
pub use execution_engine::{ShadowExecutionEngine, ShadowOrder, OrderStatus};
pub use market_simulator::{MarketSimulator, MarketCondition, PriceSimulation};
pub use performance_analyzer::{PerformanceAnalyzer, TradingMetrics, PortfolioStats};
pub use risk_manager::{ShadowRiskManager, RiskLimits, RiskViolation};
pub use order_matching::{OrderMatchingEngine, MatchingResult, TradeExecution};
pub use metrics::ShadowTradingMetrics;

/// 影子交易系统主入口
pub struct ShadowTradingSystem {
    /// 配置
    config: ShadowTradingConfig,
    /// 虚拟账户管理器
    account_manager: Arc<RwLock<HashMap<String, VirtualAccount>>>,
    /// 执行引擎
    execution_engine: Arc<ShadowExecutionEngine>,
    /// 市场模拟器
    market_simulator: Arc<MarketSimulator>,
    /// 性能分析器
    performance_analyzer: Arc<PerformanceAnalyzer>,
    /// 风险管理器
    risk_manager: Arc<ShadowRiskManager>,
    /// 订单匹配引擎
    order_matching: Arc<OrderMatchingEngine>,
    /// 指标收集器
    metrics: Arc<ShadowTradingMetrics>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
}

impl ShadowTradingSystem {
    /// 创建新的影子交易系统
    pub async fn new(config: ShadowTradingConfig) -> Result<Self> {
        let metrics = Arc::new(ShadowTradingMetrics::new()?);
        let market_simulator = Arc::new(MarketSimulator::new(config.market_simulation.clone())?);
        let order_matching = Arc::new(OrderMatchingEngine::new(config.matching.clone())?);
        let risk_manager = Arc::new(ShadowRiskManager::new(config.risk_limits.clone())?);
        let performance_analyzer = Arc::new(PerformanceAnalyzer::new(config.performance.clone())?);
        let execution_engine = Arc::new(ShadowExecutionEngine::new(
            config.execution.clone(),
            Arc::clone(&market_simulator),
            Arc::clone(&order_matching),
            Arc::clone(&metrics),
        )?);

        let system = Self {
            config,
            account_manager: Arc::new(RwLock::new(HashMap::new())),
            execution_engine,
            market_simulator,
            performance_analyzer,
            risk_manager,
            order_matching,
            metrics,
            running: Arc::new(RwLock::new(false)),
        };

        info!("Shadow trading system initialized successfully");
        Ok(system)
    }

    /// 启动影子交易系统
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                warn!("Shadow trading system is already running");
                return Ok(());
            }
            *running = true;
        }

        // 启动市场模拟器
        self.market_simulator.start().await?;
        
        // 启动订单匹配引擎
        self.order_matching.start().await?;
        
        // 启动执行引擎
        self.execution_engine.start().await?;
        
        // 启动性能分析器
        self.performance_analyzer.start().await?;

        // 启动后台任务
        self.start_background_tasks().await;

        info!("Shadow trading system started successfully");
        Ok(())
    }

    /// 停止影子交易系统
    pub async fn stop(&self) -> Result<()> {
        {
            let mut running = self.running.write().await;
            if !*running {
                warn!("Shadow trading system is not running");
                return Ok(());
            }
            *running = false;
        }

        // 停止各个组件
        self.execution_engine.stop().await?;
        self.order_matching.stop().await?;
        self.market_simulator.stop().await?;
        self.performance_analyzer.stop().await?;

        info!("Shadow trading system stopped successfully");
        Ok(())
    }

    /// 创建虚拟账户
    pub async fn create_virtual_account(
        &self,
        account_id: String,
        initial_balance: HashMap<String, f64>,
    ) -> Result<()> {
        let mut accounts = self.account_manager.write().await;
        
        if accounts.contains_key(&account_id) {
            return Err(anyhow::anyhow!("Account {} already exists", account_id));
        }

        let account = VirtualAccount::new(
            account_id.clone(),
            initial_balance,
            self.config.account.clone(),
        )?;

        accounts.insert(account_id.clone(), account);
        
        self.metrics.account_created(&account_id).await;
        info!(account_id = %account_id, "Created virtual account");

        Ok(())
    }

    /// 删除虚拟账户
    pub async fn delete_virtual_account(&self, account_id: &str) -> Result<()> {
        let mut accounts = self.account_manager.write().await;
        
        if accounts.remove(account_id).is_none() {
            return Err(anyhow::anyhow!("Account {} not found", account_id));
        }

        self.metrics.account_deleted(account_id).await;
        info!(account_id = %account_id, "Deleted virtual account");

        Ok(())
    }

    /// 获取虚拟账户信息
    pub async fn get_virtual_account(&self, account_id: &str) -> Result<VirtualAccount> {
        let accounts = self.account_manager.read().await;
        
        accounts.get(account_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))
    }

    /// 提交影子订单
    #[instrument(skip(self, order), fields(account_id = %order.account_id, symbol = %order.symbol))]
    pub async fn submit_shadow_order(&self, mut order: ShadowOrder) -> Result<String> {
        // 检查账户是否存在
        {
            let accounts = self.account_manager.read().await;
            if !accounts.contains_key(&order.account_id) {
                return Err(anyhow::anyhow!("Account {} not found", order.account_id));
            }
        }

        // 风险检查
        self.risk_manager.validate_order(&order).await?;

        // 设置订单时间戳
        order.created_at = Utc::now();
        order.updated_at = Utc::now();

        // 提交到执行引擎
        let order_id = self.execution_engine.submit_order(order).await?;

        self.metrics.order_submitted(&order_id).await;
        debug!(order_id = %order_id, "Shadow order submitted");

        Ok(order_id)
    }

    /// 取消影子订单
    pub async fn cancel_shadow_order(&self, order_id: &str) -> Result<()> {
        self.execution_engine.cancel_order(order_id).await?;
        self.metrics.order_cancelled(order_id).await;
        debug!(order_id = %order_id, "Shadow order cancelled");
        Ok(())
    }

    /// 获取订单状态
    pub async fn get_order_status(&self, order_id: &str) -> Result<ShadowOrder> {
        self.execution_engine.get_order(order_id).await
    }

    /// 获取账户订单历史
    pub async fn get_account_orders(&self, account_id: &str) -> Result<Vec<ShadowOrder>> {
        self.execution_engine.get_account_orders(account_id).await
    }

    /// 获取交易历史
    pub async fn get_trade_history(
        &self,
        account_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<TradeExecution>> {
        self.execution_engine.get_trade_history(account_id, from, to).await
    }

    /// 获取账户性能统计
    pub async fn get_performance_stats(&self, account_id: &str) -> Result<PortfolioStats> {
        self.performance_analyzer.get_account_stats(account_id).await
    }

    /// 获取实时交易指标
    pub async fn get_trading_metrics(&self, account_id: &str) -> Result<TradingMetrics> {
        self.performance_analyzer.get_trading_metrics(account_id).await
    }

    /// 重置账户状态
    pub async fn reset_account(&self, account_id: &str) -> Result<()> {
        // 取消所有未完成订单
        let orders = self.execution_engine.get_account_orders(account_id).await?;
        for order in orders {
            if matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
                self.execution_engine.cancel_order(&order.id).await?;
            }
        }

        // 重置账户余额和持仓
        {
            let mut accounts = self.account_manager.write().await;
            if let Some(account) = accounts.get_mut(account_id) {
                account.reset_to_initial_state()?;
            } else {
                return Err(anyhow::anyhow!("Account {} not found", account_id));
            }
        }

        // 重置性能统计
        self.performance_analyzer.reset_account_stats(account_id).await?;

        self.metrics.account_reset(account_id).await;
        info!(account_id = %account_id, "Account reset to initial state");

        Ok(())
    }

    /// 设置市场条件
    pub async fn set_market_condition(&self, condition: MarketCondition) -> Result<()> {
        self.market_simulator.set_market_condition(condition).await?;
        info!(condition = ?condition, "Market condition updated");
        Ok(())
    }

    /// 获取模拟市场价格
    pub async fn get_simulated_price(&self, symbol: &str) -> Result<f64> {
        self.market_simulator.get_current_price(symbol).await
    }

    /// 添加价格模拟数据
    pub async fn add_price_simulation(&self, symbol: String, simulation: PriceSimulation) -> Result<()> {
        self.market_simulator.add_price_simulation(symbol, simulation).await?;
        debug!(symbol = %symbol, "Price simulation added");
        Ok(())
    }

    /// 启动后台任务
    async fn start_background_tasks(&self) {
        let system = Arc::new(self);
        
        // 账户状态更新任务
        {
            let system_clone = Arc::clone(&system);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    if let Err(e) = system_clone.update_account_states().await {
                        error!(error = %e, "Failed to update account states");
                    }
                }
            });
        }

        // 性能统计更新任务
        {
            let system_clone = Arc::clone(&system);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    if let Err(e) = system_clone.update_performance_stats().await {
                        error!(error = %e, "Failed to update performance stats");
                    }
                }
            });
        }

        // 风险监控任务
        {
            let system_clone = Arc::clone(&system);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
                loop {
                    interval.tick().await;
                    if let Err(e) = system_clone.monitor_risk_limits().await {
                        error!(error = %e, "Failed to monitor risk limits");
                    }
                }
            });
        }

        debug!("Background tasks started");
    }

    /// 更新账户状态
    async fn update_account_states(&self) -> Result<()> {
        let running = *self.running.read().await;
        if !running {
            return Ok(());
        }

        let mut accounts = self.account_manager.write().await;
        for account in accounts.values_mut() {
            account.update_positions(&self.market_simulator).await?;
            account.calculate_unrealized_pnl(&self.market_simulator).await?;
        }

        Ok(())
    }

    /// 更新性能统计
    async fn update_performance_stats(&self) -> Result<()> {
        let running = *self.running.read().await;
        if !running {
            return Ok(());
        }

        let accounts = self.account_manager.read().await;
        for account_id in accounts.keys() {
            if let Err(e) = self.performance_analyzer.update_stats(account_id).await {
                warn!(account_id = %account_id, error = %e, "Failed to update performance stats");
            }
        }

        Ok(())
    }

    /// 监控风险限制
    async fn monitor_risk_limits(&self) -> Result<()> {
        let running = *self.running.read().await;
        if !running {
            return Ok(());
        }

        let accounts = self.account_manager.read().await;
        for account in accounts.values() {
            if let Err(violations) = self.risk_manager.check_account_risk(account).await {
                for violation in violations {
                    warn!(
                        account_id = %account.id,
                        violation_type = ?violation.violation_type,
                        "Risk limit violation detected"
                    );
                    
                    // 记录风险违规事件
                    self.metrics.risk_violation_detected(&account.id, &violation).await;
                    
                    // 根据配置采取行动（如停止交易、发送警报等）
                    if self.config.risk_limits.auto_stop_on_violation {
                        self.stop_account_trading(&account.id).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 停止账户交易
    async fn stop_account_trading(&self, account_id: &str) -> Result<()> {
        // 取消所有未完成订单
        let orders = self.execution_engine.get_account_orders(account_id).await?;
        for order in orders {
            if matches!(order.status, OrderStatus::Pending | OrderStatus::PartiallyFilled) {
                self.execution_engine.cancel_order(&order.id).await?;
            }
        }

        warn!(account_id = %account_id, "Stopped trading for account due to risk violation");
        Ok(())
    }

    /// 导出交易报告
    pub async fn export_trading_report(
        &self,
        account_id: &str,
        format: ReportFormat,
    ) -> Result<String> {
        let stats = self.get_performance_stats(account_id).await?;
        let metrics = self.get_trading_metrics(account_id).await?;
        let trades = self.get_trade_history(account_id, None, None).await?;

        let report = TradingReport {
            account_id: account_id.to_string(),
            generated_at: Utc::now(),
            portfolio_stats: stats,
            trading_metrics: metrics,
            trade_history: trades,
        };

        match format {
            ReportFormat::Json => Ok(serde_json::to_string_pretty(&report)?),
            ReportFormat::Csv => self.export_csv_report(&report),
            ReportFormat::Html => self.export_html_report(&report),
        }
    }

    fn export_csv_report(&self, report: &TradingReport) -> Result<String> {
        let mut csv = String::new();
        
        // 头部信息
        csv.push_str(&format!("Account ID,{}\n", report.account_id));
        csv.push_str(&format!("Generated At,{}\n", report.generated_at));
        csv.push_str("\n");
        
        // 性能统计
        csv.push_str("Performance Statistics\n");
        csv.push_str(&format!("Total Return,{:.4}\n", report.portfolio_stats.total_return));
        csv.push_str(&format!("Sharpe Ratio,{:.4}\n", report.portfolio_stats.sharpe_ratio));
        csv.push_str(&format!("Max Drawdown,{:.4}\n", report.portfolio_stats.max_drawdown));
        csv.push_str("\n");
        
        // 交易历史
        csv.push_str("Trade History\n");
        csv.push_str("Timestamp,Symbol,Side,Quantity,Price,Value\n");
        for trade in &report.trade_history {
            csv.push_str(&format!(
                "{},{},{:?},{},{},{}\n",
                trade.executed_at,
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.price,
                trade.value
            ));
        }
        
        Ok(csv)
    }

    fn export_html_report(&self, report: &TradingReport) -> Result<String> {
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>Shadow Trading Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .stats {{ margin: 20px 0; }}
    </style>
</head>
<body>
    <h1>Shadow Trading Report</h1>
    <p><strong>Account ID:</strong> {}</p>
    <p><strong>Generated At:</strong> {}</p>
    
    <div class="stats">
        <h2>Performance Statistics</h2>
        <table>
            <tr><th>Metric</th><th>Value</th></tr>
            <tr><td>Total Return</td><td>{:.4}</td></tr>
            <tr><td>Sharpe Ratio</td><td>{:.4}</td></tr>
            <tr><td>Max Drawdown</td><td>{:.4}</td></tr>
            <tr><td>Win Rate</td><td>{:.2}%</td></tr>
        </table>
    </div>
    
    <div class="trades">
        <h2>Recent Trades</h2>
        <table>
            <tr><th>Time</th><th>Symbol</th><th>Side</th><th>Quantity</th><th>Price</th><th>Value</th></tr>
        "#,
            report.account_id,
            report.account_id,
            report.generated_at,
            report.portfolio_stats.total_return,
            report.portfolio_stats.sharpe_ratio,
            report.portfolio_stats.max_drawdown,
            report.trading_metrics.win_rate * 100.0
        );

        let mut html = html;
        for trade in report.trade_history.iter().take(50) {
            html.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{:?}</td><td>{}</td><td>{:.4}</td><td>{:.2}</td></tr>\n",
                trade.executed_at.format("%Y-%m-%d %H:%M:%S"),
                trade.symbol,
                trade.side,
                trade.quantity,
                trade.price,
                trade.value
            ));
        }

        html.push_str(
            r#"
        </table>
    </div>
</body>
</html>
            "#
        );

        Ok(html)
    }

    /// 获取系统状态
    pub async fn get_system_status(&self) -> SystemStatus {
        let running = *self.running.read().await;
        let account_count = self.account_manager.read().await.len();
        
        SystemStatus {
            running,
            account_count,
            uptime: self.metrics.get_uptime().await,
            total_orders: self.metrics.get_total_orders().await,
            total_trades: self.metrics.get_total_trades().await,
            last_update: Utc::now(),
        }
    }
}

/// 交易报告格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Csv,
    Html,
}

/// 交易报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingReport {
    pub account_id: String,
    pub generated_at: DateTime<Utc>,
    pub portfolio_stats: PortfolioStats,
    pub trading_metrics: TradingMetrics,
    pub trade_history: Vec<TradeExecution>,
}

/// 系统状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub running: bool,
    pub account_count: usize,
    pub uptime: std::time::Duration,
    pub total_orders: u64,
    pub total_trades: u64,
    pub last_update: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_system_initialization() {
        let config = ShadowTradingConfig::default();
        let system = ShadowTradingSystem::new(config).await;
        assert!(system.is_ok());
    }

    #[tokio::test]
    async fn test_virtual_account_management() {
        let config = ShadowTradingConfig::default();
        let system = ShadowTradingSystem::new(config).await.unwrap();
        
        let mut initial_balance = HashMap::new();
        initial_balance.insert("USDT".to_string(), 10000.0);
        
        // 创建账户
        let result = system.create_virtual_account("test_account".to_string(), initial_balance).await;
        assert!(result.is_ok());
        
        // 获取账户信息
        let account = system.get_virtual_account("test_account").await;
        assert!(account.is_ok());
        
        let account = account.unwrap();
        assert_eq!(account.id, "test_account");
        assert_eq!(*account.balances.get("USDT").unwrap(), 10000.0);
        
        // 删除账户
        let result = system.delete_virtual_account("test_account").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shadow_order_lifecycle() {
        let config = ShadowTradingConfig::default();
        let system = ShadowTradingSystem::new(config).await.unwrap();
        
        let mut initial_balance = HashMap::new();
        initial_balance.insert("USDT".to_string(), 10000.0);
        system.create_virtual_account("test_account".to_string(), initial_balance).await.unwrap();
        
        let order = ShadowOrder {
            id: String::new(),
            account_id: "test_account".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: crate::execution_engine::OrderSide::Buy,
            quantity: 1.0,
            price: Some(50000.0),
            order_type: crate::execution_engine::OrderType::Limit,
            status: OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            filled_quantity: 0.0,
            average_price: None,
            fees: 0.0,
            metadata: HashMap::new(),
        };
        
        // 提交订单
        let order_id = system.submit_shadow_order(order).await;
        assert!(order_id.is_ok());
        
        let order_id = order_id.unwrap();
        assert!(!order_id.is_empty());
        
        // 获取订单状态
        let order_status = system.get_order_status(&order_id).await;
        assert!(order_status.is_ok());
        
        // 取消订单
        let result = system.cancel_shadow_order(&order_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_system_lifecycle() {
        let config = ShadowTradingConfig::default();
        let system = ShadowTradingSystem::new(config).await.unwrap();
        
        // 启动系统
        let result = system.start().await;
        assert!(result.is_ok());
        
        let status = system.get_system_status().await;
        assert!(status.running);
        
        // 停止系统
        let result = system.stop().await;
        assert!(result.is_ok());
        
        let status = system.get_system_status().await;
        assert!(!status.running);
    }
}