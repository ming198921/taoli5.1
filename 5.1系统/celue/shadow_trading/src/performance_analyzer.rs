//! 性能分析模块

use crate::config::PerformanceAnalysisConfig;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 交易指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingMetrics {
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub profit_factor: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub consecutive_wins: u64,
    pub consecutive_losses: u64,
    pub max_consecutive_wins: u64,
    pub max_consecutive_losses: u64,
}

/// 投资组合统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioStats {
    pub initial_value: f64,
    pub current_value: f64,
    pub total_return: f64,
    pub annualized_return: f64,
    pub volatility: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub var_95: f64,
    pub var_99: f64,
    pub beta: f64,
    pub alpha: f64,
    pub treynor_ratio: f64,
    pub information_ratio: f64,
    pub calmar_ratio: f64,
}

/// 性能数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    pub timestamp: DateTime<Utc>,
    pub portfolio_value: f64,
    pub daily_return: f64,
    pub cumulative_return: f64,
    pub drawdown: f64,
    pub volatility: f64,
}

/// 性能分析器
pub struct PerformanceAnalyzer {
    config: PerformanceAnalysisConfig,
    account_stats: Arc<RwLock<HashMap<String, PortfolioStats>>>,
    account_metrics: Arc<RwLock<HashMap<String, TradingMetrics>>>,
    performance_history: Arc<RwLock<HashMap<String, VecDeque<PerformanceDataPoint>>>>,
    running: Arc<RwLock<bool>>,
}

impl PerformanceAnalyzer {
    pub fn new(config: PerformanceAnalysisConfig) -> Result<Self> {
        Ok(Self {
            config,
            account_stats: Arc::new(RwLock::new(HashMap::new())),
            account_metrics: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        
        self.start_performance_calculation().await;
        info!("Performance analyzer started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Performance analyzer stopped");
        Ok(())
    }

    pub async fn get_account_stats(&self, account_id: &str) -> Result<PortfolioStats> {
        let stats = self.account_stats.read().await;
        stats.get(account_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))
    }

    pub async fn get_trading_metrics(&self, account_id: &str) -> Result<TradingMetrics> {
        let metrics = self.account_metrics.read().await;
        metrics.get(account_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Account {} not found", account_id))
    }

    pub async fn update_stats(&self, account_id: &str) -> Result<()> {
        // 这里应该从实际的账户数据更新统计信息
        // 为了测试，我们创建一些模拟数据
        self.update_portfolio_stats(account_id, 100000.0, 110000.0).await?;
        self.update_trading_metrics(account_id, 50, 30, 20, 2000.0, -1500.0).await?;
        Ok(())
    }

    pub async fn reset_account_stats(&self, account_id: &str) -> Result<()> {
        {
            let mut stats = self.account_stats.write().await;
            stats.remove(account_id);
        }
        
        {
            let mut metrics = self.account_metrics.write().await;
            metrics.remove(account_id);
        }
        
        {
            let mut history = self.performance_history.write().await;
            history.remove(account_id);
        }
        
        info!(account_id = %account_id, "Account performance stats reset");
        Ok(())
    }

    async fn update_portfolio_stats(
        &self,
        account_id: &str,
        initial_value: f64,
        current_value: f64,
    ) -> Result<()> {
        let total_return = (current_value - initial_value) / initial_value;
        let days_elapsed = 30.0; // 假设30天
        let annualized_return = (1.0 + total_return).powf(365.0 / days_elapsed) - 1.0;
        
        // 计算波动率和其他指标
        let returns = self.get_daily_returns(account_id).await;
        let volatility = self.calculate_volatility(&returns);
        let sharpe_ratio = (annualized_return - self.config.risk_free_rate) / (volatility * (252.0_f64).sqrt());
        let max_drawdown = self.calculate_max_drawdown(&returns);
        
        let stats = PortfolioStats {
            initial_value,
            current_value,
            total_return,
            annualized_return,
            volatility,
            sharpe_ratio,
            max_drawdown,
            current_drawdown: 0.0, // 需要实际计算
            var_95: self.calculate_var(&returns, 0.95),
            var_99: self.calculate_var(&returns, 0.99),
            beta: 1.0, // 需要基准数据
            alpha: annualized_return - self.config.benchmark_return,
            treynor_ratio: (annualized_return - self.config.risk_free_rate) / 1.0, // 假设beta=1
            information_ratio: (annualized_return - self.config.benchmark_return) / volatility,
            calmar_ratio: if max_drawdown != 0.0 { annualized_return / max_drawdown.abs() } else { 0.0 },
        };

        let mut account_stats = self.account_stats.write().await;
        account_stats.insert(account_id.to_string(), stats);

        Ok(())
    }

    async fn update_trading_metrics(
        &self,
        account_id: &str,
        total_trades: u64,
        winning_trades: u64,
        losing_trades: u64,
        total_profit: f64,
        total_loss: f64,
    ) -> Result<()> {
        let win_rate = if total_trades > 0 { winning_trades as f64 / total_trades as f64 } else { 0.0 };
        let average_win = if winning_trades > 0 { total_profit / winning_trades as f64 } else { 0.0 };
        let average_loss = if losing_trades > 0 { total_loss.abs() / losing_trades as f64 } else { 0.0 };
        let profit_factor = if average_loss != 0.0 { average_win / average_loss } else { 0.0 };

        let metrics = TradingMetrics {
            total_trades,
            winning_trades,
            losing_trades,
            win_rate,
            average_win,
            average_loss,
            profit_factor,
            largest_win: total_profit * 0.3, // 假设最大盈利是总盈利的30%
            largest_loss: total_loss * 0.4,  // 假设最大亏损是总亏损的40%
            consecutive_wins: 0,
            consecutive_losses: 0,
            max_consecutive_wins: 5,
            max_consecutive_losses: 3,
        };

        let mut account_metrics = self.account_metrics.write().await;
        account_metrics.insert(account_id.to_string(), metrics);

        Ok(())
    }

    async fn get_daily_returns(&self, account_id: &str) -> Vec<f64> {
        // 从性能历史中获取日收益率
        let history = self.performance_history.read().await;
        if let Some(account_history) = history.get(account_id) {
            account_history.iter().map(|point| point.daily_return).collect()
        } else {
            // 生成一些模拟收益率数据
            vec![0.01, -0.005, 0.02, -0.01, 0.015, 0.008, -0.012, 0.025, -0.018, 0.007]
        }
    }

    fn calculate_volatility(&self, returns: &[f64]) -> f64 {
        if returns.len() < 2 {
            return 0.0;
        }

        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|&r| (r - mean).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;
        
        variance.sqrt()
    }

    fn calculate_max_drawdown(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mut cumulative_return = 1.0;
        let mut peak = 1.0;
        let mut max_drawdown = 0.0;

        for &daily_return in returns {
            cumulative_return *= 1.0 + daily_return;
            peak = peak.max(cumulative_return);
            let drawdown = (peak - cumulative_return) / peak;
            max_drawdown = max_drawdown.max(drawdown);
        }

        max_drawdown
    }

    fn calculate_var(&self, returns: &[f64], confidence_level: f64) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }

        let mut sorted_returns = returns.to_vec();
        sorted_returns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = ((1.0 - confidence_level) * sorted_returns.len() as f64) as usize;
        sorted_returns[index.min(sorted_returns.len() - 1)]
    }

    async fn start_performance_calculation(&self) {
        let analyzer = Arc::new(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(analyzer.config.calculation_interval_seconds)
            );

            loop {
                interval.tick().await;
                
                let running = *analyzer.running.read().await;
                if !running {
                    break;
                }

                if let Err(e) = analyzer.calculate_performance_metrics().await {
                    debug!(error = %e, "Failed to calculate performance metrics");
                }
            }
        });
    }

    async fn calculate_performance_metrics(&self) -> Result<()> {
        let account_ids: Vec<String> = {
            let stats = self.account_stats.read().await;
            stats.keys().cloned().collect()
        };

        for account_id in account_ids {
            if let Err(e) = self.update_performance_history(&account_id).await {
                debug!(account_id = %account_id, error = %e, "Failed to update performance history");
            }
        }

        Ok(())
    }

    async fn update_performance_history(&self, account_id: &str) -> Result<()> {
        let stats = {
            let stats_map = self.account_stats.read().await;
            stats_map.get(account_id).cloned()
        };

        if let Some(stats) = stats {
            let data_point = PerformanceDataPoint {
                timestamp: Utc::now(),
                portfolio_value: stats.current_value,
                daily_return: 0.005, // 模拟日收益率
                cumulative_return: stats.total_return,
                drawdown: stats.current_drawdown,
                volatility: stats.volatility,
            };

            let mut history = self.performance_history.write().await;
            let account_history = history.entry(account_id.to_string()).or_insert_with(VecDeque::new);
            account_history.push_back(data_point);

            // 限制历史数据大小
            let max_history_points = self.config.lookback_window_days as usize * 24; // 假设每小时一个数据点
            while account_history.len() > max_history_points {
                account_history.pop_front();
            }
        }

        Ok(())
    }

    pub async fn get_performance_history(
        &self,
        account_id: &str,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Vec<PerformanceDataPoint> {
        let history = self.performance_history.read().await;
        
        if let Some(account_history) = history.get(account_id) {
            account_history.iter()
                .filter(|point| {
                    if let Some(from_time) = from {
                        if point.timestamp < from_time {
                            return false;
                        }
                    }
                    if let Some(to_time) = to {
                        if point.timestamp > to_time {
                            return false;
                        }
                    }
                    true
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub async fn calculate_correlation(&self, account1: &str, account2: &str) -> Result<f64> {
        let history1 = self.get_performance_history(account1, None, None).await;
        let history2 = self.get_performance_history(account2, None, None).await;

        if history1.len() != history2.len() || history1.is_empty() {
            return Ok(0.0);
        }

        let returns1: Vec<f64> = history1.iter().map(|p| p.daily_return).collect();
        let returns2: Vec<f64> = history2.iter().map(|p| p.daily_return).collect();

        let correlation = self.calculate_correlation_coefficient(&returns1, &returns2);
        Ok(correlation)
    }

    fn calculate_correlation_coefficient(&self, x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        let n = x.len() as f64;
        let mean_x = x.iter().sum::<f64>() / n;
        let mean_y = y.iter().sum::<f64>() / n;

        let numerator: f64 = x.iter().zip(y.iter())
            .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
            .sum();

        let sum_sq_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
        let sum_sq_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();

        let denominator = (sum_sq_x * sum_sq_y).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    pub async fn calculate_beta(&self, account_id: &str, benchmark_returns: &[f64]) -> Result<f64> {
        let history = self.get_performance_history(account_id, None, None).await;
        let account_returns: Vec<f64> = history.iter().map(|p| p.daily_return).collect();

        if account_returns.len() != benchmark_returns.len() || account_returns.is_empty() {
            return Ok(1.0); // 默认beta为1
        }

        let correlation = self.calculate_correlation_coefficient(&account_returns, benchmark_returns);
        let account_volatility = self.calculate_volatility(&account_returns);
        let benchmark_volatility = self.calculate_volatility(benchmark_returns);

        let beta = if benchmark_volatility != 0.0 {
            correlation * (account_volatility / benchmark_volatility)
        } else {
            1.0
        };

        Ok(beta)
    }
}

impl Default for TradingMetrics {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate: 0.0,
            average_win: 0.0,
            average_loss: 0.0,
            profit_factor: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
            consecutive_wins: 0,
            consecutive_losses: 0,
            max_consecutive_wins: 0,
            max_consecutive_losses: 0,
        }
    }
}

impl Default for PortfolioStats {
    fn default() -> Self {
        Self {
            initial_value: 0.0,
            current_value: 0.0,
            total_return: 0.0,
            annualized_return: 0.0,
            volatility: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
            var_95: 0.0,
            var_99: 0.0,
            beta: 1.0,
            alpha: 0.0,
            treynor_ratio: 0.0,
            information_ratio: 0.0,
            calmar_ratio: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_analyzer_creation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config);
        assert!(analyzer.is_ok());
    }

    #[tokio::test]
    async fn test_volatility_calculation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config).unwrap();
        
        let returns = vec![0.01, -0.005, 0.02, -0.01, 0.015];
        let volatility = analyzer.calculate_volatility(&returns);
        assert!(volatility > 0.0);
    }

    #[tokio::test]
    async fn test_max_drawdown_calculation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config).unwrap();
        
        let returns = vec![0.1, -0.2, 0.05, -0.1, 0.15];
        let max_drawdown = analyzer.calculate_max_drawdown(&returns);
        assert!(max_drawdown >= 0.0);
    }

    #[tokio::test]
    async fn test_var_calculation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config).unwrap();
        
        let returns = vec![-0.05, -0.03, -0.01, 0.01, 0.02, 0.03, 0.05];
        let var_95 = analyzer.calculate_var(&returns, 0.95);
        assert!(var_95 < 0.0); // VaR should be negative
    }

    #[tokio::test]
    async fn test_correlation_calculation() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config).unwrap();
        
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // 完全相关
        
        let correlation = analyzer.calculate_correlation_coefficient(&x, &y);
        assert!((correlation - 1.0).abs() < 1e-10); // 应该接近1
    }

    #[tokio::test]
    async fn test_performance_update() {
        let config = PerformanceAnalysisConfig::default();
        let analyzer = PerformanceAnalyzer::new(config).unwrap();
        
        let result = analyzer.update_portfolio_stats("test_account", 100000.0, 110000.0).await;
        assert!(result.is_ok());
        
        let stats = analyzer.get_account_stats("test_account").await;
        assert!(stats.is_ok());
        
        let stats = stats.unwrap();
        assert_eq!(stats.total_return, 0.1); // 10% return
    }
}