//! 可视化管理器 - 集成所有可视化组件的高级API
//!
//! 提供统一的接口管理Sankey图、盈利曲线和多维分析可视化

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, error, instrument};

use super::{
    sankey_diagram::{SankeyDiagram, SankeyConfig, SankeyNode, SankeyLink},
    fund_flow_tracker::{FundFlowTracker, FundFlowConfig, FundFlowEvent},
    profit_curve_visualizer::{ProfitCurveVisualizer, ProfitDataPoint, RiskMetricPoint, StrategyPerformance, TimeWindow},
    web_dashboard::{WebDashboard, DashboardState},
    VisualizationConfig, VisualizationResponse, ChartType,
};

/// 可视化管理器 - 统一管理所有可视化组件
pub struct VisualizationManager {
    /// 配置
    config: VisualizationConfig,
    /// Sankey资金流向图
    sankey_diagram: Arc<SankeyDiagram>,
    /// 资金流动追踪器
    fund_flow_tracker: Arc<FundFlowTracker>,
    /// 盈利曲线可视化器
    profit_visualizer: Arc<ProfitCurveVisualizer>,
    /// Web仪表板
    web_dashboard: Option<Arc<WebDashboard>>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 统一事件广播器
    event_sender: broadcast::Sender<VisualizationEvent>,
}

/// 统一可视化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VisualizationEvent {
    /// 系统启动
    SystemStarted {
        timestamp: DateTime<Utc>,
        components: Vec<String>,
    },
    /// 系统停止
    SystemStopped {
        timestamp: DateTime<Utc>,
    },
    /// 数据更新
    DataUpdated {
        component: String,
        data_type: String,
        timestamp: DateTime<Utc>,
    },
    /// 异常检测
    AnomalyDetected {
        component: String,
        anomaly_type: String,
        severity: String,
        description: String,
        timestamp: DateTime<Utc>,
    },
    /// 性能告警
    PerformanceAlert {
        component: String,
        metric: String,
        value: f64,
        threshold: f64,
        timestamp: DateTime<Utc>,
    },
}

/// 可视化管理器配置
#[derive(Debug, Clone)]
pub struct VisualizationManagerConfig {
    /// 基础可视化配置
    pub visualization_config: VisualizationConfig,
    /// Sankey图配置
    pub sankey_config: SankeyConfig,
    /// 资金流追踪配置
    pub fund_flow_config: FundFlowConfig,
    /// 启用Web仪表板
    pub enable_web_dashboard: bool,
    /// 自动启动所有组件
    pub auto_start_components: bool,
}

impl Default for VisualizationManagerConfig {
    fn default() -> Self {
        Self {
            visualization_config: VisualizationConfig::default(),
            sankey_config: SankeyConfig::default(),
            fund_flow_config: FundFlowConfig::default(),
            enable_web_dashboard: true,
            auto_start_components: true,
        }
    }
}

/// 可视化数据查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationQuery {
    /// 策略ID筛选
    pub strategy_ids: Option<Vec<String>>,
    /// 时间范围筛选
    pub time_range: Option<TimeRange>,
    /// 时间窗口
    pub time_window: Option<TimeWindow>,
    /// 数据类型筛选
    pub data_types: Option<Vec<String>>,
    /// 限制返回条数
    pub limit: Option<usize>,
}

/// 时间范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// 综合仪表板数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// Sankey图数据
    pub sankey_data: serde_json::Value,
    /// 盈利曲线数据
    pub profit_curves: HashMap<String, VisualizationResponse>,
    /// 风险指标数据
    pub risk_metrics: HashMap<String, VisualizationResponse>,
    /// 策略对比数据
    pub strategy_comparison: VisualizationResponse,
    /// 异常事件
    pub anomalies: Vec<serde_json::Value>,
    /// 系统统计
    pub system_stats: SystemStats,
    /// 生成时间
    pub timestamp: DateTime<Utc>,
}

/// 系统统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    /// 活跃策略数量
    pub active_strategies: u32,
    /// 总数据点数量
    pub total_data_points: u64,
    /// 检测到的异常数量
    pub total_anomalies: u64,
    /// 监控的交易所数量
    pub tracked_exchanges: u32,
    /// 系统运行时间
    pub uptime_seconds: u64,
    /// 内存使用情况
    pub memory_usage_mb: f64,
}

impl VisualizationManager {
    /// 创建新的可视化管理器
    pub fn new(config: VisualizationManagerConfig) -> Self {
        // 创建核心组件
        let sankey_diagram = Arc::new(SankeyDiagram::new(config.sankey_config));
        let fund_flow_tracker = Arc::new(FundFlowTracker::new(
            config.fund_flow_config,
            Arc::clone(&sankey_diagram),
        ));
        let profit_visualizer = Arc::new(ProfitCurveVisualizer::new(config.visualization_config.clone()));

        // 创建Web仪表板（如果启用）
        let web_dashboard = if config.enable_web_dashboard {
            Some(Arc::new(WebDashboard::new(
                Arc::clone(&sankey_diagram),
                Arc::clone(&fund_flow_tracker),
                config.visualization_config.clone(),
            )))
        } else {
            None
        };

        let (event_sender, _) = broadcast::channel(1000);

        Self {
            config: config.visualization_config,
            sankey_diagram,
            fund_flow_tracker,
            profit_visualizer,
            web_dashboard,
            running: Arc::new(RwLock::new(false)),
            event_sender,
        }
    }

    /// 启动可视化管理器
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;

        info!("Starting visualization manager");

        let mut started_components = Vec::new();

        // 启动盈利曲线可视化器
        if let Err(e) = self.profit_visualizer.start().await {
            error!("Failed to start profit visualizer: {}", e);
        } else {
            started_components.push("ProfitCurveVisualizer".to_string());
        }

        // 启动资金流追踪器
        if let Err(e) = self.fund_flow_tracker.start().await {
            error!("Failed to start fund flow tracker: {}", e);
        } else {
            started_components.push("FundFlowTracker".to_string());
        }

        // 启动Web仪表板（如果启用）
        if let Some(dashboard) = &self.web_dashboard {
            tokio::spawn({
                let dashboard = Arc::clone(dashboard);
                async move {
                    if let Err(e) = dashboard.start_server().await {
                        error!("Failed to start web dashboard: {}", e);
                    }
                }
            });
            started_components.push("WebDashboard".to_string());
        }

        // 发送启动事件
        let _ = self.event_sender.send(VisualizationEvent::SystemStarted {
            timestamp: Utc::now(),
            components: started_components,
        });

        info!("Visualization manager started successfully");
        Ok(())
    }

    /// 停止可视化管理器
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;

        info!("Stopping visualization manager");

        // 停止盈利曲线可视化器
        if let Err(e) = self.profit_visualizer.stop().await {
            warn!("Failed to stop profit visualizer: {}", e);
        }

        // 发送停止事件
        let _ = self.event_sender.send(VisualizationEvent::SystemStopped {
            timestamp: Utc::now(),
        });

        info!("Visualization manager stopped");
        Ok(())
    }

    /// 添加交易所节点到Sankey图
    #[instrument(skip(self))]
    pub async fn add_exchange(&self, exchange: &str, symbol: &str, initial_balance: f64) -> Result<()> {
        let node = SankeyNode {
            id: format!("{}:{}", exchange, symbol),
            name: format!("{} ({})", exchange, symbol),
            balance: initial_balance,
            historical_balances: vec![(Utc::now(), initial_balance)],
            position: crate::visualization::sankey_diagram::NodePosition {
                x: 0.0, y: 0.0, width: 150.0, height: 60.0
            },
            color: self.get_exchange_color(exchange),
            status: crate::visualization::sankey_diagram::NodeStatus::Active,
            metadata: HashMap::new(),
        };

        self.sankey_diagram.add_node(node).await?;
        
        info!("Added exchange {} with symbol {} to Sankey diagram", exchange, symbol);
        Ok(())
    }

    /// 记录资金流动
    #[instrument(skip(self))]
    pub async fn record_fund_flow(
        &self,
        source_exchange: &str,
        target_exchange: &str,
        symbol: &str,
        amount: f64,
        flow_type: crate::visualization::sankey_diagram::FlowType,
        transaction_id: Option<String>,
        fee: Option<f64>,
    ) -> Result<String> {
        let flow_id = self.fund_flow_tracker.record_flow(
            source_exchange,
            target_exchange,
            symbol,
            amount,
            flow_type,
            transaction_id,
            fee,
        ).await?;

        info!("Recorded fund flow: {} -> {} ({}: {})", 
              source_exchange, target_exchange, symbol, amount);

        Ok(flow_id)
    }

    /// 更新交易所余额
    #[instrument(skip(self))]
    pub async fn update_exchange_balance(
        &self,
        exchange: &str,
        symbol: &str,
        balance: f64,
        frozen: Option<f64>,
        reason: crate::visualization::fund_flow_tracker::BalanceChangeReason,
    ) -> Result<()> {
        self.fund_flow_tracker.update_exchange_balance(
            exchange, symbol, balance, frozen, reason
        ).await?;

        info!("Updated balance for {}:{} to {}", exchange, symbol, balance);
        Ok(())
    }

    /// 添加策略盈利数据
    #[instrument(skip(self))]
    pub async fn add_strategy_profit(&self, strategy_id: &str, profit_data: ProfitDataPoint) -> Result<()> {
        self.profit_visualizer.add_profit_data(strategy_id, profit_data).await?;
        
        info!("Added profit data for strategy: {}", strategy_id);
        Ok(())
    }

    /// 添加策略风险指标
    #[instrument(skip(self))]
    pub async fn add_strategy_risk_metrics(&self, strategy_id: &str, risk_metrics: RiskMetricPoint) -> Result<()> {
        self.profit_visualizer.add_risk_metrics(strategy_id, risk_metrics).await?;
        
        info!("Added risk metrics for strategy: {}", strategy_id);
        Ok(())
    }

    /// 更新策略性能对比数据
    #[instrument(skip(self))]
    pub async fn update_strategy_performance(&self, performance: StrategyPerformance) -> Result<()> {
        let strategy_id = performance.strategy_id.clone();
        self.profit_visualizer.update_strategy_performance(performance).await?;
        
        info!("Updated performance data for strategy: {}", strategy_id);
        Ok(())
    }

    /// 生成综合仪表板数据
    #[instrument(skip(self))]
    pub async fn generate_dashboard_data(&self, query: Option<VisualizationQuery>) -> Result<DashboardData> {
        let timestamp = Utc::now();

        // 获取Sankey图数据
        let sankey_data = serde_json::to_value(self.sankey_diagram.get_current_data().await)
            .context("Failed to serialize Sankey data")?;

        // 获取策略列表
        let strategy_ids = if let Some(ref q) = query {
            q.strategy_ids.clone().unwrap_or_else(|| vec!["default".to_string()])
        } else {
            vec!["default".to_string()]
        };

        let time_window = query.as_ref()
            .and_then(|q| q.time_window)
            .unwrap_or(TimeWindow::Day1);

        // 生成盈利曲线数据
        let mut profit_curves = HashMap::new();
        for strategy_id in &strategy_ids {
            if let Ok(chart) = self.profit_visualizer.generate_profit_curve_chart(strategy_id, time_window).await {
                profit_curves.insert(strategy_id.clone(), chart);
            }
        }

        // 生成风险指标数据
        let mut risk_metrics = HashMap::new();
        for strategy_id in &strategy_ids {
            if let Ok(chart) = self.profit_visualizer.generate_risk_metrics_chart(strategy_id, time_window).await {
                risk_metrics.insert(strategy_id.clone(), chart);
            }
        }

        // 生成策略对比数据
        let strategy_comparison = self.profit_visualizer.generate_strategy_comparison_chart().await
            .unwrap_or_else(|_| VisualizationResponse {
                chart_type: ChartType::Bar,
                title: "策略对比 (无数据)".to_string(),
                series: vec![],
                x_axis_label: "策略".to_string(),
                y_axis_label: "指标值".to_string(),
                timestamp,
            });

        // 获取异常事件
        let anomalies = self.profit_visualizer.get_anomalies(Some(20)).await
            .unwrap_or_default()
            .into_iter()
            .map(|anomaly| serde_json::to_value(anomaly).unwrap_or_default())
            .collect();

        // 获取系统统计
        let profit_stats = self.profit_visualizer.get_stats().await;
        let fund_stats = self.fund_flow_tracker.get_stats().await;
        
        let system_stats = SystemStats {
            active_strategies: profit_stats.active_strategies,
            total_data_points: profit_stats.total_data_points,
            total_anomalies: profit_stats.anomalies_detected,
            tracked_exchanges: fund_stats.exchanges_tracked as u32,
            uptime_seconds: 0, // TODO: 计算实际运行时间
            memory_usage_mb: profit_stats.memory_usage_mb,
        };

        Ok(DashboardData {
            sankey_data,
            profit_curves,
            risk_metrics,
            strategy_comparison,
            anomalies,
            system_stats,
            timestamp,
        })
    }

    /// 获取指定策略的盈利曲线图表
    pub async fn get_profit_curve_chart(&self, strategy_id: &str, time_window: TimeWindow) -> Result<VisualizationResponse> {
        self.profit_visualizer.generate_profit_curve_chart(strategy_id, time_window).await
    }

    /// 获取指定策略的风险指标图表
    pub async fn get_risk_metrics_chart(&self, strategy_id: &str, time_window: TimeWindow) -> Result<VisualizationResponse> {
        self.profit_visualizer.generate_risk_metrics_chart(strategy_id, time_window).await
    }

    /// 获取策略对比图表
    pub async fn get_strategy_comparison_chart(&self) -> Result<VisualizationResponse> {
        self.profit_visualizer.generate_strategy_comparison_chart().await
    }

    /// 获取Sankey图数据
    pub async fn get_sankey_data(&self) -> crate::visualization::sankey_diagram::SankeyVisualizationData {
        self.sankey_diagram.get_current_data().await
    }

    /// 获取所有异常事件
    pub async fn get_anomalies(&self, limit: Option<usize>) -> Result<Vec<serde_json::Value>> {
        let anomalies = self.profit_visualizer.get_anomalies(limit).await?;
        Ok(anomalies.into_iter()
            .map(|anomaly| serde_json::to_value(anomaly).unwrap_or_default())
            .collect())
    }

    /// 订阅可视化事件
    pub fn subscribe(&self) -> broadcast::Receiver<VisualizationEvent> {
        self.event_sender.subscribe()
    }

    /// 检查系统运行状态
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// 获取系统健康状态
    pub async fn get_health_status(&self) -> SystemHealthStatus {
        let running = self.is_running().await;
        let profit_stats = self.profit_visualizer.get_stats().await;
        let fund_stats = self.fund_flow_tracker.get_stats().await;

        SystemHealthStatus {
            overall_status: if running { HealthStatus::Healthy } else { HealthStatus::Stopped },
            components: vec![
                ComponentHealth {
                    name: "ProfitVisualizer".to_string(),
                    status: HealthStatus::Healthy,
                    last_update: profit_stats.last_update,
                    metrics: serde_json::json!({
                        "active_strategies": profit_stats.active_strategies,
                        "total_data_points": profit_stats.total_data_points
                    }),
                },
                ComponentHealth {
                    name: "FundFlowTracker".to_string(),
                    status: HealthStatus::Healthy,
                    last_update: fund_stats.last_update,
                    metrics: serde_json::json!({
                        "total_flows": fund_stats.total_flows,
                        "exchanges_tracked": fund_stats.exchanges_tracked
                    }),
                },
            ],
            timestamp: Utc::now(),
        }
    }

    /// 获取交易所颜色
    fn get_exchange_color(&self, exchange: &str) -> String {
        match exchange.to_lowercase().as_str() {
            "binance" => "#f3ba2f".to_string(),
            "okx" => "#0052ff".to_string(),
            "huobi" => "#2ebd85".to_string(),
            "bybit" => "#f7a600".to_string(),
            "kucoin" => "#24ae8f".to_string(),
            _ => "#6c757d".to_string(),
        }
    }
}

/// 系统健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthStatus {
    pub overall_status: HealthStatus,
    pub components: Vec<ComponentHealth>,
    pub timestamp: DateTime<Utc>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub last_update: Option<DateTime<Utc>>,
    pub metrics: serde_json::Value,
}

/// 健康状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Error,
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::fund_flow_tracker::BalanceChangeReason;

    #[tokio::test]
    async fn test_visualization_manager_lifecycle() {
        let config = VisualizationManagerConfig {
            enable_web_dashboard: false, // 禁用Web仪表板以避免端口冲突
            auto_start_components: true,
            ..Default::default()
        };
        
        let manager = VisualizationManager::new(config);

        // 测试启动
        manager.start().await.unwrap();
        assert!(manager.is_running().await);

        // 测试添加交易所
        manager.add_exchange("binance", "BTC", 10.0).await.unwrap();

        // 测试更新余额
        manager.update_exchange_balance(
            "binance",
            "BTC",
            15.0,
            Some(1.0),
            BalanceChangeReason::Trade,
        ).await.unwrap();

        // 测试记录资金流动
        let _flow_id = manager.record_fund_flow(
            "binance",
            "okx",
            "BTC",
            2.0,
            crate::visualization::sankey_diagram::FlowType::ArbitrageOutflow,
            Some("tx_123".to_string()),
            Some(0.001),
        ).await.unwrap();

        // 测试添加策略盈利数据
        let profit_data = ProfitDataPoint {
            timestamp: Utc::now(),
            cumulative_profit: 1000.0,
            daily_profit: 50.0,
            unrealized_pnl: 20.0,
            realized_pnl: 30.0,
            total_volume: 50000.0,
            trade_count: 10,
            win_rate: 0.7,
            profit_factor: 1.5,
            total_fees: 10.0,
            net_profit: 40.0,
            roi_percent: 5.0,
        };

        manager.add_strategy_profit("test_strategy", profit_data).await.unwrap();

        // 测试生成仪表板数据
        let dashboard_data = manager.generate_dashboard_data(None).await.unwrap();
        assert!(!dashboard_data.profit_curves.is_empty());

        // 测试健康检查
        let health = manager.get_health_status().await;
        assert_eq!(health.components.len(), 2);

        // 测试停止
        manager.stop().await.unwrap();
        assert!(!manager.is_running().await);
    }

    #[tokio::test]
    async fn test_comprehensive_visualization_flow() {
        let config = VisualizationManagerConfig {
            enable_web_dashboard: false,
            ..Default::default()
        };
        
        let manager = VisualizationManager::new(config);
        manager.start().await.unwrap();

        // 设置多个交易所
        let exchanges = vec![
            ("binance", "BTC", 10.0),
            ("okx", "BTC", 8.0),
            ("huobi", "BTC", 5.0),
        ];

        for (exchange, symbol, balance) in exchanges {
            manager.add_exchange(exchange, symbol, balance).await.unwrap();
        }

        // 模拟资金流动
        let _flow1 = manager.record_fund_flow(
            "binance", "okx", "BTC", 2.0,
            crate::visualization::sankey_diagram::FlowType::ArbitrageOutflow,
            None, Some(0.001)
        ).await.unwrap();

        let _flow2 = manager.record_fund_flow(
            "okx", "huobi", "BTC", 1.5,
            crate::visualization::sankey_diagram::FlowType::ArbitrageInflow,
            None, Some(0.0008)
        ).await.unwrap();

        // 添加策略性能数据
        for i in 0..5 {
            let profit_data = ProfitDataPoint {
                timestamp: Utc::now(),
                cumulative_profit: 1000.0 + i as f64 * 100.0,
                daily_profit: 20.0 + i as f64 * 5.0,
                unrealized_pnl: 10.0,
                realized_pnl: 20.0,
                total_volume: 50000.0,
                trade_count: 10 + i,
                win_rate: 0.7,
                profit_factor: 1.5,
                total_fees: 10.0,
                net_profit: 30.0,
                roi_percent: 3.0 + i as f64,
            };

            manager.add_strategy_profit("strategy_1", profit_data).await.unwrap();
        }

        // 生成完整的仪表板数据
        let dashboard_data = manager.generate_dashboard_data(Some(VisualizationQuery {
            strategy_ids: Some(vec!["strategy_1".to_string()]),
            time_window: Some(TimeWindow::Hour1),
            ..Default::default()
        })).await.unwrap();

        // 验证数据完整性
        assert!(!dashboard_data.sankey_data.is_null());
        assert!(dashboard_data.profit_curves.contains_key("strategy_1"));
        assert!(dashboard_data.system_stats.active_strategies > 0);

        manager.stop().await.unwrap();
    }
}

impl Default for VisualizationQuery {
    fn default() -> Self {
        Self {
            strategy_ids: None,
            time_range: None,
            time_window: Some(TimeWindow::Day1),
            data_types: None,
            limit: Some(1000),
        }
    }
}