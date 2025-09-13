//! 可视化适配器
//! 
//! 连接性能分析器与可视化系统的桥梁

use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use crate::performance_analyzer::{PerformanceAnalyzer, PortfolioStats, TradingMetrics, PerformanceDataPoint};

/// 可视化适配器
pub struct VisualizationAdapter {
    /// 性能分析器引用
    performance_analyzer: Arc<PerformanceAnalyzer>,
    /// 可视化器引用（使用动态引用避免循环依赖）
    visualizer_endpoints: Arc<RwLock<Vec<String>>>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 更新间隔
    update_interval_seconds: u64,
}

/// 可视化数据传输对象
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisualizationDTO {
    pub strategy_name: String,
    pub timestamp: DateTime<Utc>,
    pub profit_data: ProfitDataDTO,
    pub risk_metrics: RiskMetricsDTO,
    pub performance_summary: PerformanceSummaryDTO,
}

/// 盈利数据传输对象
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfitDataDTO {
    pub cumulative_profit: f64,
    pub daily_profit: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_volume: f64,
    pub trade_count: u64,
    pub win_rate: f64,
}

/// 风险指标传输对象
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RiskMetricsDTO {
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub volatility: f64,
    pub var_95: f64,
    pub var_99: f64,
    pub beta: f64,
    pub alpha: f64,
}

/// 性能摘要传输对象
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceSummaryDTO {
    pub total_return: f64,
    pub annualized_return: f64,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub profit_factor: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
}

impl VisualizationAdapter {
    /// 创建新的可视化适配器
    pub fn new(performance_analyzer: Arc<PerformanceAnalyzer>) -> Self {
        Self {
            performance_analyzer,
            visualizer_endpoints: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            update_interval_seconds: 30, // 默认30秒更新
        }
    }

    /// 设置更新间隔
    pub async fn set_update_interval(&self, seconds: u64) {
        self.update_interval_seconds = seconds;
    }

    /// 添加可视化端点
    pub async fn add_visualizer_endpoint(&self, endpoint: String) {
        let mut endpoints = self.visualizer_endpoints.write().await;
        endpoints.push(endpoint);
        info!(endpoint = %endpoint, "Added visualizer endpoint");
    }

    /// 启动适配器
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;

        // 启动数据同步任务
        self.start_data_synchronization().await;

        info!("Visualization adapter started");
        Ok(())
    }

    /// 停止适配器
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;

        info!("Visualization adapter stopped");
        Ok(())
    }

    /// 手动同步数据到可视化系统
    #[instrument(skip(self))]
    pub async fn sync_data_for_account(&self, account_id: &str) -> Result<()> {
        // 从性能分析器获取数据
        let portfolio_stats = self.performance_analyzer.get_account_stats(account_id).await?;
        let trading_metrics = self.performance_analyzer.get_trading_metrics(account_id).await?;
        let performance_history = self.performance_analyzer.get_performance_history(account_id, None, None).await;

        // 转换为可视化数据格式
        let visualization_data = self.convert_to_visualization_format(
            account_id,
            &portfolio_stats,
            &trading_metrics,
            &performance_history,
        ).await?;

        // 发送到可视化系统
        self.send_to_visualizers(&visualization_data).await?;

        debug!(account_id = %account_id, "Synchronized data to visualization system");
        Ok(())
    }

    /// 获取实时数据快照
    #[instrument(skip(self))]
    pub async fn get_real_time_snapshot(&self, account_id: &str) -> Result<VisualizationDTO> {
        let portfolio_stats = self.performance_analyzer.get_account_stats(account_id).await?;
        let trading_metrics = self.performance_analyzer.get_trading_metrics(account_id).await?;
        let performance_history = self.performance_analyzer.get_performance_history(
            account_id, 
            Some(Utc::now() - Duration::hours(24)), // 最近24小时
            None
        ).await;

        self.convert_to_visualization_format(
            account_id,
            &portfolio_stats,
            &trading_metrics,
            &performance_history,
        ).await
    }

    /// 启动数据同步任务
    async fn start_data_synchronization(&self) {
        let adapter = Arc::new(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(adapter.update_interval_seconds)
            );

            loop {
                interval.tick().await;
                
                let running = *adapter.running.read().await;
                if !running {
                    break;
                }

                if let Err(e) = adapter.perform_periodic_sync().await {
                    debug!(error = %e, "Failed to perform periodic visualization sync");
                }
            }
        });
    }

    /// 执行周期性同步
    async fn perform_periodic_sync(&self) -> Result<()> {
        // 这里应该获取所有活跃账户列表
        // 为了演示，我们使用固定的账户列表
        let account_ids = vec!["main_account", "test_account", "strategy_1", "strategy_2"];

        for account_id in account_ids {
            // 尝试更新每个账户的统计信息
            if let Err(e) = self.performance_analyzer.update_stats(account_id).await {
                debug!(account_id = %account_id, error = %e, "Failed to update account stats");
                continue;
            }

            // 同步到可视化系统
            if let Err(e) = self.sync_data_for_account(account_id).await {
                debug!(account_id = %account_id, error = %e, "Failed to sync account data");
            }
        }

        Ok(())
    }

    /// 转换为可视化格式
    async fn convert_to_visualization_format(
        &self,
        account_id: &str,
        portfolio_stats: &PortfolioStats,
        trading_metrics: &TradingMetrics,
        performance_history: &[PerformanceDataPoint],
    ) -> Result<VisualizationDTO> {
        // 计算当前盈亏
        let latest_performance = performance_history.last();
        let daily_profit = latest_performance.map(|p| p.daily_return * portfolio_stats.current_value).unwrap_or(0.0);
        
        // 计算交易量（简化计算）
        let total_volume = trading_metrics.total_trades as f64 * 1000.0; // 假设平均每笔1000单位

        let profit_data = ProfitDataDTO {
            cumulative_profit: portfolio_stats.current_value - portfolio_stats.initial_value,
            daily_profit,
            unrealized_pnl: 0.0, // 需要从实际持仓计算
            realized_pnl: portfolio_stats.current_value - portfolio_stats.initial_value,
            total_volume,
            trade_count: trading_metrics.total_trades,
            win_rate: trading_metrics.win_rate,
        };

        let risk_metrics = RiskMetricsDTO {
            sharpe_ratio: portfolio_stats.sharpe_ratio,
            max_drawdown: portfolio_stats.max_drawdown,
            current_drawdown: portfolio_stats.current_drawdown,
            volatility: portfolio_stats.volatility,
            var_95: portfolio_stats.var_95,
            var_99: portfolio_stats.var_99,
            beta: portfolio_stats.beta,
            alpha: portfolio_stats.alpha,
        };

        let performance_summary = PerformanceSummaryDTO {
            total_return: portfolio_stats.total_return,
            annualized_return: portfolio_stats.annualized_return,
            total_trades: trading_metrics.total_trades,
            winning_trades: trading_metrics.winning_trades,
            profit_factor: trading_metrics.profit_factor,
            largest_win: trading_metrics.largest_win,
            largest_loss: trading_metrics.largest_loss,
        };

        Ok(VisualizationDTO {
            strategy_name: account_id.to_string(),
            timestamp: Utc::now(),
            profit_data,
            risk_metrics,
            performance_summary,
        })
    }

    /// 发送数据到可视化系统
    async fn send_to_visualizers(&self, data: &VisualizationDTO) -> Result<()> {
        let endpoints = self.visualizer_endpoints.read().await;
        
        for endpoint in endpoints.iter() {
            if let Err(e) = self.send_to_single_visualizer(endpoint, data).await {
                debug!(endpoint = %endpoint, error = %e, "Failed to send data to visualizer");
            }
        }

        Ok(())
    }

    /// 发送数据到单个可视化器
    async fn send_to_single_visualizer(&self, endpoint: &str, data: &VisualizationDTO) -> Result<()> {
        // 这里实现实际的HTTP/WebSocket发送逻辑
        // 为了演示，我们只是记录日志
        debug!(
            endpoint = %endpoint,
            strategy = %data.strategy_name,
            profit = %data.profit_data.cumulative_profit,
            "Sent data to visualizer"
        );

        // 模拟HTTP POST请求
        // let client = reqwest::Client::new();
        // let response = client
        //     .post(endpoint)
        //     .json(data)
        //     .send()
        //     .await?;
        //
        // if !response.status().is_success() {
        //     return Err(anyhow::anyhow!("Visualizer endpoint returned error: {}", response.status()));
        // }

        Ok(())
    }

    /// 获取性能对比数据
    #[instrument(skip(self))]
    pub async fn get_strategy_comparison_data(&self, account_ids: &[&str]) -> Result<Vec<VisualizationDTO>> {
        let mut comparison_data = Vec::new();

        for &account_id in account_ids {
            match self.get_real_time_snapshot(account_id).await {
                Ok(data) => comparison_data.push(data),
                Err(e) => {
                    debug!(account_id = %account_id, error = %e, "Failed to get comparison data");
                }
            }
        }

        Ok(comparison_data)
    }

    /// 计算相关性矩阵
    #[instrument(skip(self))]
    pub async fn calculate_strategy_correlations(&self, account_ids: &[&str]) -> Result<Vec<Vec<f64>>> {
        let n = account_ids.len();
        let mut correlation_matrix = vec![vec![0.0; n]; n];

        for (i, &account1) in account_ids.iter().enumerate() {
            for (j, &account2) in account_ids.iter().enumerate() {
                if i == j {
                    correlation_matrix[i][j] = 1.0;
                } else {
                    match self.performance_analyzer.calculate_correlation(account1, account2).await {
                        Ok(correlation) => correlation_matrix[i][j] = correlation,
                        Err(_) => correlation_matrix[i][j] = 0.0,
                    }
                }
            }
        }

        Ok(correlation_matrix)
    }

    /// 生成性能报告
    #[instrument(skip(self))]
    pub async fn generate_performance_report(&self, account_id: &str, days: i64) -> Result<PerformanceReport> {
        let from_time = Utc::now() - Duration::days(days);
        let performance_history = self.performance_analyzer.get_performance_history(
            account_id, 
            Some(from_time), 
            None
        ).await;

        let portfolio_stats = self.performance_analyzer.get_account_stats(account_id).await?;
        let trading_metrics = self.performance_analyzer.get_trading_metrics(account_id).await?;

        let report = PerformanceReport {
            account_id: account_id.to_string(),
            period_start: from_time,
            period_end: Utc::now(),
            total_return: portfolio_stats.total_return,
            annualized_return: portfolio_stats.annualized_return,
            volatility: portfolio_stats.volatility,
            sharpe_ratio: portfolio_stats.sharpe_ratio,
            max_drawdown: portfolio_stats.max_drawdown,
            total_trades: trading_metrics.total_trades,
            win_rate: trading_metrics.win_rate,
            profit_factor: trading_metrics.profit_factor,
            data_points: performance_history.len(),
            best_day_return: performance_history.iter().map(|p| p.daily_return).fold(f64::NEG_INFINITY, f64::max),
            worst_day_return: performance_history.iter().map(|p| p.daily_return).fold(f64::INFINITY, f64::min),
            avg_daily_return: performance_history.iter().map(|p| p.daily_return).sum::<f64>() / performance_history.len() as f64,
        };

        Ok(report)
    }
}

/// 性能报告
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceReport {
    pub account_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub total_trades: u64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub data_points: usize,
    pub best_day_return: f64,
    pub worst_day_return: f64,
    pub avg_daily_return: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PerformanceAnalysisConfig;

    #[tokio::test]
    async fn test_visualization_adapter_creation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = Arc::new(PerformanceAnalyzer::new(config).unwrap());
        let adapter = VisualizationAdapter::new(analyzer);
        
        // Basic creation test
        assert_eq!(*adapter.running.read().await, false);
    }

    #[tokio::test]
    async fn test_add_visualizer_endpoint() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = Arc::new(PerformanceAnalyzer::new(config).unwrap());
        let adapter = VisualizationAdapter::new(analyzer);

        adapter.add_visualizer_endpoint("http://localhost:8080/visualization".to_string()).await;
        
        let endpoints = adapter.visualizer_endpoints.read().await;
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0], "http://localhost:8080/visualization");
    }

    #[tokio::test]
    async fn test_real_time_snapshot() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = Arc::new(PerformanceAnalyzer::new(config).unwrap());
        let adapter = VisualizationAdapter::new(analyzer.clone());

        // Update test data first
        analyzer.update_stats("test_account").await.unwrap();

        let snapshot = adapter.get_real_time_snapshot("test_account").await;
        assert!(snapshot.is_ok());
        
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.strategy_name, "test_account");
        assert!(snapshot.performance_summary.total_trades > 0);
    }

    #[tokio::test]
    async fn test_performance_report_generation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = Arc::new(PerformanceAnalyzer::new(config).unwrap());
        let adapter = VisualizationAdapter::new(analyzer.clone());

        // Update test data
        analyzer.update_stats("test_account").await.unwrap();

        let report = adapter.generate_performance_report("test_account", 30).await;
        assert!(report.is_ok());
        
        let report = report.unwrap();
        assert_eq!(report.account_id, "test_account");
        assert!(report.period_start <= report.period_end);
    }

    #[tokio::test]
    async fn test_strategy_comparison_data() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = Arc::new(PerformanceAnalyzer::new(config).unwrap());
        let adapter = VisualizationAdapter::new(analyzer.clone());

        // Update test data for multiple accounts
        let accounts = vec!["strategy_1", "strategy_2", "strategy_3"];
        for account in &accounts {
            analyzer.update_stats(account).await.unwrap();
        }

        let comparison_data = adapter.get_strategy_comparison_data(&accounts).await;
        assert!(comparison_data.is_ok());
        
        let data = comparison_data.unwrap();
        assert_eq!(data.len(), 3);
        
        for (i, datum) in data.iter().enumerate() {
            assert_eq!(datum.strategy_name, accounts[i]);
        }
    }
}