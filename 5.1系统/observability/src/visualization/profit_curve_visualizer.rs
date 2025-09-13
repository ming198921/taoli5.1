//! 实时盈利曲线多维可视化模块
//! 
//! 实现生产级多维度盈利分析可视化：
//! - 实时盈利曲线图表
//! - 风险指标动态展示  
//! - 策略性能对比分析
//! - 多时间窗口分析
//! - 异常检测与告警

use anyhow::{Result, Context};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BTreeMap};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, info, instrument, warn, error};
use uuid::Uuid;

use super::{
    VisualizationConfig, VisualizationDataPoint, VisualizationSeries, 
    VisualizationResponse, ChartType, LineType
};

/// 实时盈利曲线多维可视化器
pub struct ProfitCurveVisualizer {
    /// 配置
    config: VisualizationConfig,
    /// 实时盈利数据存储
    profit_data: Arc<RwLock<HashMap<String, VecDeque<ProfitDataPoint>>>>,
    /// 风险指标数据存储
    risk_metrics: Arc<RwLock<HashMap<String, VecDeque<RiskMetricPoint>>>>,
    /// 策略性能对比数据
    strategy_comparison: Arc<RwLock<HashMap<String, StrategyPerformance>>>,
    /// 多时间窗口数据
    time_window_data: Arc<RwLock<BTreeMap<TimeWindow, WindowData>>>,
    /// 异常检测引擎
    anomaly_detector: Arc<RwLock<AnomalyDetector>>,
    /// 事件广播器
    event_sender: broadcast::Sender<ProfitVisualizationEvent>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// 性能统计
    performance_stats: Arc<RwLock<VisualizerStats>>,
}

/// 盈利数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfitDataPoint {
    pub timestamp: DateTime<Utc>,
    pub cumulative_profit: f64,
    pub daily_profit: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_volume: f64,
    pub trade_count: u64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_fees: f64,
    pub net_profit: f64,
    pub roi_percent: f64,
}

/// 风险指标数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetricPoint {
    pub timestamp: DateTime<Utc>,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
    pub volatility: f64,
    pub var_95: f64,
    pub var_99: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,
    pub beta: f64,
    pub alpha: f64,
    pub information_ratio: f64,
    pub tracking_error: f64,
}

/// 策略性能对比数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub strategy_id: String,
    pub strategy_name: String,
    pub total_return: f64,
    pub annualized_return: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_trades: u64,
    pub avg_trade_duration: f64,
    pub best_trade: f64,
    pub worst_trade: f64,
    pub last_update: DateTime<Utc>,
    pub status: StrategyStatus,
}

/// 策略状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StrategyStatus {
    Active,
    Paused,
    Stopped,
    Error,
}

/// 时间窗口定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeWindow {
    Minutes15,
    Hour1,
    Hours4,
    Day1,
    Week1,
    Month1,
}

/// 时间窗口数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowData {
    pub window: TimeWindow,
    pub profit_points: VecDeque<ProfitDataPoint>,
    pub risk_points: VecDeque<RiskMetricPoint>,
    pub summary_stats: WindowSummary,
    pub last_update: DateTime<Utc>,
}

/// 时间窗口统计摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSummary {
    pub total_return: f64,
    pub avg_daily_return: f64,
    pub volatility: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub trade_count: u64,
    pub best_day: f64,
    pub worst_day: f64,
}

/// 异常检测器
#[derive(Debug)]
pub struct AnomalyDetector {
    thresholds: AnomalyThresholds,
    detection_history: VecDeque<AnomalyEvent>,
    statistical_models: StatisticalModels,
}

/// 异常阈值配置
#[derive(Debug, Clone)]
pub struct AnomalyThresholds {
    pub max_drawdown_alert: f64,      // 最大回撤告警阈值
    pub profit_volatility_alert: f64,  // 盈利波动告警阈值
    pub sharpe_ratio_warning: f64,     // 夏普比率警告阈值
    pub consecutive_losses_alert: u32, // 连续亏损次数告警
    pub daily_loss_limit: f64,         // 日亏损限额
}

impl Default for AnomalyThresholds {
    fn default() -> Self {
        Self {
            max_drawdown_alert: 0.1,      // 10%
            profit_volatility_alert: 0.15, // 15%
            sharpe_ratio_warning: 1.0,
            consecutive_losses_alert: 5,
            daily_loss_limit: 0.05,       // 5%
        }
    }
}

/// 统计模型
#[derive(Debug)]
struct StatisticalModels {
    moving_averages: HashMap<u32, f64>, // 移动平均
    bollinger_bands: BollingerBands,
    z_score_threshold: f64,
}

/// 布林带指标
#[derive(Debug)]
struct BollingerBands {
    period: u32,
    std_dev_multiplier: f64,
    upper_band: f64,
    lower_band: f64,
    middle_band: f64,
}

/// 异常事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyEvent {
    pub id: String,
    pub event_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub affected_strategy: Option<String>,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    ExcessiveDrawdown,
    HighVolatility,
    LowSharpeRatio,
    ConsecutiveLosses,
    DailyLossLimit,
    UnusualProfitPattern,
    RiskMetricAnomaly,
}

/// 异常严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 可视化事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProfitVisualizationEvent {
    DataUpdated {
        strategy_id: String,
        data_type: DataType,
        timestamp: DateTime<Utc>,
    },
    AnomalyDetected {
        anomaly: AnomalyEvent,
    },
    ThresholdBreached {
        strategy_id: String,
        metric_name: String,
        value: f64,
        threshold: f64,
    },
    PerformanceAlert {
        strategy_id: String,
        message: String,
        severity: AnomalySeverity,
    },
}

/// 数据类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Profit,
    Risk,
    Performance,
}

/// 可视化器统计信息
#[derive(Debug, Clone, Default)]
pub struct VisualizerStats {
    pub total_data_points: u64,
    pub active_strategies: u32,
    pub anomalies_detected: u64,
    pub updates_per_second: f64,
    pub last_update: Option<DateTime<Utc>>,
    pub memory_usage_mb: f64,
}

impl ProfitCurveVisualizer {
    /// 创建新的盈利曲线可视化器
    pub fn new(config: VisualizationConfig) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            config,
            profit_data: Arc::new(RwLock::new(HashMap::new())),
            risk_metrics: Arc::new(RwLock::new(HashMap::new())),
            strategy_comparison: Arc::new(RwLock::new(HashMap::new())),
            time_window_data: Arc::new(RwLock::new(BTreeMap::new())),
            anomaly_detector: Arc::new(RwLock::new(AnomalyDetector::new())),
            event_sender,
            running: Arc::new(RwLock::new(false)),
            performance_stats: Arc::new(RwLock::new(VisualizerStats::default())),
        }
    }

    /// 启动可视化器
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;

        info!("Starting profit curve visualizer");

        // 初始化时间窗口
        self.initialize_time_windows().await?;

        // 启动数据更新任务
        self.start_update_tasks().await;

        Ok(())
    }

    /// 停止可视化器
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;

        info!("Stopping profit curve visualizer");
        Ok(())
    }

    /// 添加盈利数据点
    #[instrument(skip(self))]
    pub async fn add_profit_data(&self, strategy_id: &str, data_point: ProfitDataPoint) -> Result<()> {
        // 添加到主数据存储
        {
            let mut profit_data = self.profit_data.write().await;
            let strategy_data = profit_data.entry(strategy_id.to_string()).or_insert_with(VecDeque::new);
            strategy_data.push_back(data_point.clone());

            // 保持数据点数量在配置限制内
            if strategy_data.len() > self.config.max_data_points {
                strategy_data.pop_front();
            }
        }

        // 更新时间窗口数据
        self.update_time_windows(strategy_id, &data_point, None).await?;

        // 异常检测
        self.detect_profit_anomalies(strategy_id, &data_point).await?;

        // 发送更新事件
        let _ = self.event_sender.send(ProfitVisualizationEvent::DataUpdated {
            strategy_id: strategy_id.to_string(),
            data_type: DataType::Profit,
            timestamp: data_point.timestamp,
        });

        // 更新统计信息
        self.update_stats().await;

        debug!("Added profit data for strategy: {}", strategy_id);
        Ok(())
    }

    /// 添加风险指标数据点
    #[instrument(skip(self))]
    pub async fn add_risk_metrics(&self, strategy_id: &str, risk_point: RiskMetricPoint) -> Result<()> {
        // 添加到主数据存储
        {
            let mut risk_metrics = self.risk_metrics.write().await;
            let strategy_data = risk_metrics.entry(strategy_id.to_string()).or_insert_with(VecDeque::new);
            strategy_data.push_back(risk_point.clone());

            // 保持数据点数量在配置限制内
            if strategy_data.len() > self.config.max_data_points {
                strategy_data.pop_front();
            }
        }

        // 更新时间窗口数据
        self.update_time_windows(strategy_id, None, Some(&risk_point)).await?;

        // 风险异常检测
        self.detect_risk_anomalies(strategy_id, &risk_point).await?;

        // 发送更新事件
        let _ = self.event_sender.send(ProfitVisualizationEvent::DataUpdated {
            strategy_id: strategy_id.to_string(),
            data_type: DataType::Risk,
            timestamp: risk_point.timestamp,
        });

        debug!("Added risk metrics for strategy: {}", strategy_id);
        Ok(())
    }

    /// 更新策略性能对比数据
    #[instrument(skip(self))]
    pub async fn update_strategy_performance(&self, performance: StrategyPerformance) -> Result<()> {
        let strategy_id = performance.strategy_id.clone();
        
        {
            let mut comparison = self.strategy_comparison.write().await;
            comparison.insert(strategy_id.clone(), performance);
        }

        // 发送更新事件
        let _ = self.event_sender.send(ProfitVisualizationEvent::DataUpdated {
            strategy_id,
            data_type: DataType::Performance,
            timestamp: Utc::now(),
        });

        Ok(())
    }

    /// 生成实时盈利曲线图表
    #[instrument(skip(self))]
    pub async fn generate_profit_curve_chart(&self, strategy_id: &str, time_window: TimeWindow) -> Result<VisualizationResponse> {
        let window_data = self.get_window_data(time_window).await?;
        let profit_data = window_data.profit_points;

        if profit_data.is_empty() {
            return Ok(VisualizationResponse {
                chart_type: ChartType::Line,
                title: format!("盈利曲线 - {} (暂无数据)", strategy_id),
                series: vec![],
                x_axis_label: "时间".to_string(),
                y_axis_label: "累计盈利".to_string(),
                timestamp: Utc::now(),
            });
        }

        // 生成累计盈利序列
        let cumulative_series = VisualizationSeries {
            name: "累计盈利".to_string(),
            data_points: profit_data.iter().map(|point| {
                VisualizationDataPoint {
                    timestamp: point.timestamp,
                    value: point.cumulative_profit,
                    label: format!("{:.2}", point.cumulative_profit),
                    metadata: serde_json::Map::new().into(),
                }
            }).collect(),
            color: "#2ecc71".to_string(),
            line_type: LineType::Solid,
        };

        // 生成日盈利序列
        let daily_series = VisualizationSeries {
            name: "日盈利".to_string(),
            data_points: profit_data.iter().map(|point| {
                VisualizationDataPoint {
                    timestamp: point.timestamp,
                    value: point.daily_profit,
                    label: format!("{:.2}", point.daily_profit),
                    metadata: serde_json::Map::new().into(),
                }
            }).collect(),
            color: "#3498db".to_string(),
            line_type: LineType::Dashed,
        };

        Ok(VisualizationResponse {
            chart_type: ChartType::Line,
            title: format!("盈利曲线 - {} ({:?})", strategy_id, time_window),
            series: vec![cumulative_series, daily_series],
            x_axis_label: "时间".to_string(),
            y_axis_label: "盈利 (USDT)".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// 生成风险指标图表
    #[instrument(skip(self))]
    pub async fn generate_risk_metrics_chart(&self, strategy_id: &str, time_window: TimeWindow) -> Result<VisualizationResponse> {
        let window_data = self.get_window_data(time_window).await?;
        let risk_data = window_data.risk_points;

        if risk_data.is_empty() {
            return Ok(VisualizationResponse {
                chart_type: ChartType::Line,
                title: format!("风险指标 - {} (暂无数据)", strategy_id),
                series: vec![],
                x_axis_label: "时间".to_string(),
                y_axis_label: "指标值".to_string(),
                timestamp: Utc::now(),
            });
        }

        // 夏普比率序列
        let sharpe_series = VisualizationSeries {
            name: "夏普比率".to_string(),
            data_points: risk_data.iter().map(|point| {
                VisualizationDataPoint {
                    timestamp: point.timestamp,
                    value: point.sharpe_ratio,
                    label: format!("{:.2}", point.sharpe_ratio),
                    metadata: serde_json::Map::new().into(),
                }
            }).collect(),
            color: "#e74c3c".to_string(),
            line_type: LineType::Solid,
        };

        // 最大回撤序列
        let drawdown_series = VisualizationSeries {
            name: "最大回撤".to_string(),
            data_points: risk_data.iter().map(|point| {
                VisualizationDataPoint {
                    timestamp: point.timestamp,
                    value: point.max_drawdown,
                    label: format!("{:.2}%", point.max_drawdown * 100.0),
                    metadata: serde_json::Map::new().into(),
                }
            }).collect(),
            color: "#f39c12".to_string(),
            line_type: LineType::Solid,
        };

        Ok(VisualizationResponse {
            chart_type: ChartType::Line,
            title: format!("风险指标 - {} ({:?})", strategy_id, time_window),
            series: vec![sharpe_series, drawdown_series],
            x_axis_label: "时间".to_string(),
            y_axis_label: "指标值".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// 生成策略对比图表
    #[instrument(skip(self))]
    pub async fn generate_strategy_comparison_chart(&self) -> Result<VisualizationResponse> {
        let comparison_data = self.strategy_comparison.read().await;

        if comparison_data.is_empty() {
            return Ok(VisualizationResponse {
                chart_type: ChartType::Bar,
                title: "策略性能对比 (暂无数据)".to_string(),
                series: vec![],
                x_axis_label: "策略".to_string(),
                y_axis_label: "收益率 (%)".to_string(),
                timestamp: Utc::now(),
            });
        }

        // 创建策略对比数据点
        let mut return_points = Vec::new();
        let mut sharpe_points = Vec::new();
        let mut drawdown_points = Vec::new();

        for (strategy_id, performance) in comparison_data.iter() {
            let timestamp = performance.last_update;

            return_points.push(VisualizationDataPoint {
                timestamp,
                value: performance.total_return * 100.0,
                label: format!("{}: {:.2}%", performance.strategy_name, performance.total_return * 100.0),
                metadata: serde_json::Map::new().into(),
            });

            sharpe_points.push(VisualizationDataPoint {
                timestamp,
                value: performance.sharpe_ratio,
                label: format!("{}: {:.2}", performance.strategy_name, performance.sharpe_ratio),
                metadata: serde_json::Map::new().into(),
            });

            drawdown_points.push(VisualizationDataPoint {
                timestamp,
                value: performance.max_drawdown * 100.0,
                label: format!("{}: {:.2}%", performance.strategy_name, performance.max_drawdown * 100.0),
                metadata: serde_json::Map::new().into(),
            });
        }

        let series = vec![
            VisualizationSeries {
                name: "总收益率".to_string(),
                data_points: return_points,
                color: "#2ecc71".to_string(),
                line_type: LineType::Solid,
            },
            VisualizationSeries {
                name: "夏普比率".to_string(),
                data_points: sharpe_points,
                color: "#3498db".to_string(),
                line_type: LineType::Solid,
            },
            VisualizationSeries {
                name: "最大回撤".to_string(),
                data_points: drawdown_points,
                color: "#e74c3c".to_string(),
                line_type: LineType::Solid,
            },
        ];

        Ok(VisualizationResponse {
            chart_type: ChartType::Bar,
            title: "策略性能对比".to_string(),
            series,
            x_axis_label: "策略".to_string(),
            y_axis_label: "指标值".to_string(),
            timestamp: Utc::now(),
        })
    }

    /// 获取异常事件列表
    pub async fn get_anomalies(&self, limit: Option<usize>) -> Result<Vec<AnomalyEvent>> {
        let detector = self.anomaly_detector.read().await;
        let events = &detector.detection_history;

        let result = match limit {
            Some(n) => events.iter().rev().take(n).cloned().collect(),
            None => events.iter().cloned().collect(),
        };

        Ok(result)
    }

    /// 订阅可视化事件
    pub fn subscribe(&self) -> broadcast::Receiver<ProfitVisualizationEvent> {
        self.event_sender.subscribe()
    }

    /// 获取可视化器统计信息
    pub async fn get_stats(&self) -> VisualizerStats {
        self.performance_stats.read().await.clone()
    }

    /// 初始化时间窗口
    async fn initialize_time_windows(&self) -> Result<()> {
        let mut window_data = self.time_window_data.write().await;
        
        let windows = [
            TimeWindow::Minutes15,
            TimeWindow::Hour1,
            TimeWindow::Hours4,
            TimeWindow::Day1,
            TimeWindow::Week1,
            TimeWindow::Month1,
        ];

        for window in windows {
            window_data.insert(window, WindowData {
                window,
                profit_points: VecDeque::new(),
                risk_points: VecDeque::new(),
                summary_stats: WindowSummary {
                    total_return: 0.0,
                    avg_daily_return: 0.0,
                    volatility: 0.0,
                    max_drawdown: 0.0,
                    win_rate: 0.0,
                    profit_factor: 0.0,
                    trade_count: 0,
                    best_day: 0.0,
                    worst_day: 0.0,
                },
                last_update: Utc::now(),
            });
        }

        info!("Initialized {} time windows", windows.len());
        Ok(())
    }

    /// 更新时间窗口数据
    async fn update_time_windows(
        &self,
        strategy_id: &str,
        profit_point: Option<&ProfitDataPoint>,
        risk_point: Option<&RiskMetricPoint>,
    ) -> Result<()> {
        let mut window_data = self.time_window_data.write().await;

        for (window, data) in window_data.iter_mut() {
            let window_duration = self.get_window_duration(*window);
            let cutoff_time = Utc::now() - window_duration;

            // 清理过期数据
            data.profit_points.retain(|point| point.timestamp > cutoff_time);
            data.risk_points.retain(|point| point.timestamp > cutoff_time);

            // 添加新数据
            if let Some(profit) = profit_point {
                if profit.timestamp > cutoff_time {
                    data.profit_points.push_back(profit.clone());
                }
            }

            if let Some(risk) = risk_point {
                if risk.timestamp > cutoff_time {
                    data.risk_points.push_back(risk.clone());
                }
            }

            // 更新统计摘要
            data.summary_stats = self.calculate_window_summary(&data.profit_points);
            data.last_update = Utc::now();
        }

        Ok(())
    }

    /// 获取窗口持续时间
    fn get_window_duration(&self, window: TimeWindow) -> Duration {
        match window {
            TimeWindow::Minutes15 => Duration::minutes(15),
            TimeWindow::Hour1 => Duration::hours(1),
            TimeWindow::Hours4 => Duration::hours(4),
            TimeWindow::Day1 => Duration::days(1),
            TimeWindow::Week1 => Duration::weeks(1),
            TimeWindow::Month1 => Duration::days(30),
        }
    }

    /// 计算时间窗口统计摘要
    fn calculate_window_summary(&self, profit_points: &VecDeque<ProfitDataPoint>) -> WindowSummary {
        if profit_points.is_empty() {
            return WindowSummary {
                total_return: 0.0,
                avg_daily_return: 0.0,
                volatility: 0.0,
                max_drawdown: 0.0,
                win_rate: 0.0,
                profit_factor: 0.0,
                trade_count: 0,
                best_day: 0.0,
                worst_day: 0.0,
            };
        }

        let first_profit = profit_points.front().unwrap().cumulative_profit;
        let last_profit = profit_points.back().unwrap().cumulative_profit;
        let total_return = if first_profit != 0.0 {
            (last_profit - first_profit) / first_profit
        } else {
            0.0
        };

        let daily_profits: Vec<f64> = profit_points.iter().map(|p| p.daily_profit).collect();
        let avg_daily_return = daily_profits.iter().sum::<f64>() / daily_profits.len() as f64;
        
        // 计算波动率
        let variance = daily_profits.iter()
            .map(|profit| (profit - avg_daily_return).powi(2))
            .sum::<f64>() / daily_profits.len() as f64;
        let volatility = variance.sqrt();

        let max_drawdown = profit_points.iter()
            .map(|p| p.max_drawdown)
            .fold(0.0, f64::max);

        let win_count = daily_profits.iter().filter(|&&p| p > 0.0).count();
        let win_rate = win_count as f64 / daily_profits.len() as f64;

        let positive_sum: f64 = daily_profits.iter().filter(|&&p| p > 0.0).sum();
        let negative_sum: f64 = daily_profits.iter().filter(|&&p| p < 0.0).sum::<f64>().abs();
        let profit_factor = if negative_sum != 0.0 {
            positive_sum / negative_sum
        } else {
            0.0
        };

        let best_day = daily_profits.iter().fold(0.0, |a, &b| a.max(b));
        let worst_day = daily_profits.iter().fold(0.0, |a, &b| a.min(b));
        let trade_count = profit_points.back().map(|p| p.trade_count).unwrap_or(0);

        WindowSummary {
            total_return,
            avg_daily_return,
            volatility,
            max_drawdown,
            win_rate,
            profit_factor,
            trade_count,
            best_day,
            worst_day,
        }
    }

    /// 获取指定时间窗口数据
    async fn get_window_data(&self, window: TimeWindow) -> Result<WindowData> {
        let window_data = self.time_window_data.read().await;
        window_data.get(&window)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Window data not found: {:?}", window))
    }

    /// 检测盈利异常
    async fn detect_profit_anomalies(&self, strategy_id: &str, data_point: &ProfitDataPoint) -> Result<()> {
        let mut detector = self.anomaly_detector.write().await;
        
        // 检测日亏损限额
        if data_point.daily_profit < -detector.thresholds.daily_loss_limit {
            let anomaly = AnomalyEvent {
                id: Uuid::new_v4().to_string(),
                event_type: AnomalyType::DailyLossLimit,
                severity: AnomalySeverity::High,
                description: format!(
                    "策略 {} 日亏损超过限额: {:.2}% (限额: {:.2}%)",
                    strategy_id,
                    data_point.daily_profit * 100.0,
                    detector.thresholds.daily_loss_limit * 100.0
                ),
                timestamp: data_point.timestamp,
                affected_strategy: Some(strategy_id.to_string()),
                metric_value: data_point.daily_profit,
                threshold_value: -detector.thresholds.daily_loss_limit,
                metadata: HashMap::new(),
            };

            detector.detection_history.push_back(anomaly.clone());
            let _ = self.event_sender.send(ProfitVisualizationEvent::AnomalyDetected { anomaly });
        }

        // 保持异常历史记录在限制范围内
        while detector.detection_history.len() > 1000 {
            detector.detection_history.pop_front();
        }

        Ok(())
    }

    /// 检测风险异常
    async fn detect_risk_anomalies(&self, strategy_id: &str, risk_point: &RiskMetricPoint) -> Result<()> {
        let mut detector = self.anomaly_detector.write().await;

        // 检测最大回撤超标
        if risk_point.max_drawdown > detector.thresholds.max_drawdown_alert {
            let anomaly = AnomalyEvent {
                id: Uuid::new_v4().to_string(),
                event_type: AnomalyType::ExcessiveDrawdown,
                severity: AnomalySeverity::Critical,
                description: format!(
                    "策略 {} 最大回撤超过告警阈值: {:.2}% (阈值: {:.2}%)",
                    strategy_id,
                    risk_point.max_drawdown * 100.0,
                    detector.thresholds.max_drawdown_alert * 100.0
                ),
                timestamp: risk_point.timestamp,
                affected_strategy: Some(strategy_id.to_string()),
                metric_value: risk_point.max_drawdown,
                threshold_value: detector.thresholds.max_drawdown_alert,
                metadata: HashMap::new(),
            };

            detector.detection_history.push_back(anomaly.clone());
            let _ = self.event_sender.send(ProfitVisualizationEvent::AnomalyDetected { anomaly });
        }

        // 检测夏普比率过低
        if risk_point.sharpe_ratio < detector.thresholds.sharpe_ratio_warning {
            let anomaly = AnomalyEvent {
                id: Uuid::new_v4().to_string(),
                event_type: AnomalyType::LowSharpeRatio,
                severity: AnomalySeverity::Medium,
                description: format!(
                    "策略 {} 夏普比率过低: {:.2} (警告阈值: {:.2})",
                    strategy_id,
                    risk_point.sharpe_ratio,
                    detector.thresholds.sharpe_ratio_warning
                ),
                timestamp: risk_point.timestamp,
                affected_strategy: Some(strategy_id.to_string()),
                metric_value: risk_point.sharpe_ratio,
                threshold_value: detector.thresholds.sharpe_ratio_warning,
                metadata: HashMap::new(),
            };

            detector.detection_history.push_back(anomaly.clone());
            let _ = self.event_sender.send(ProfitVisualizationEvent::AnomalyDetected { anomaly });
        }

        Ok(())
    }

    /// 启动后台更新任务
    async fn start_update_tasks(&self) {
        let stats = Arc::clone(&self.performance_stats);
        let running = Arc::clone(&self.running);

        // 统计信息更新任务
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if !*running.read().await {
                    break;
                }

                // 这里可以添加定期的统计信息更新逻辑
                debug!("Updating visualizer statistics");
            }
        });
    }

    /// 更新统计信息
    async fn update_stats(&self) {
        let mut stats = self.performance_stats.write().await;
        stats.last_update = Some(Utc::now());
        
        // 统计活跃策略数量
        let profit_data = self.profit_data.read().await;
        stats.active_strategies = profit_data.len() as u32;
        
        // 统计总数据点数量
        stats.total_data_points = profit_data.values()
            .map(|deque| deque.len() as u64)
            .sum();
    }
}

impl AnomalyDetector {
    fn new() -> Self {
        Self {
            thresholds: AnomalyThresholds::default(),
            detection_history: VecDeque::new(),
            statistical_models: StatisticalModels {
                moving_averages: HashMap::new(),
                bollinger_bands: BollingerBands {
                    period: 20,
                    std_dev_multiplier: 2.0,
                    upper_band: 0.0,
                    lower_band: 0.0,
                    middle_band: 0.0,
                },
                z_score_threshold: 2.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_profit_curve_visualizer_basic() {
        let config = VisualizationConfig::default();
        let visualizer = ProfitCurveVisualizer::new(config);

        // 启动可视化器
        visualizer.start().await.unwrap();

        // 添加测试数据
        let profit_point = ProfitDataPoint {
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

        visualizer.add_profit_data("test_strategy", profit_point).await.unwrap();

        // 生成图表
        let chart = visualizer.generate_profit_curve_chart("test_strategy", TimeWindow::Hour1).await.unwrap();
        assert_eq!(chart.chart_type, ChartType::Line);
        assert!(!chart.series.is_empty());

        // 停止可视化器
        visualizer.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let config = VisualizationConfig::default();
        let visualizer = ProfitCurveVisualizer::new(config);
        let mut receiver = visualizer.subscribe();

        // 启动可视化器
        visualizer.start().await.unwrap();

        // 添加触发异常的数据
        let high_loss_point = ProfitDataPoint {
            timestamp: Utc::now(),
            cumulative_profit: 1000.0,
            daily_profit: -0.08, // -8% daily loss, should trigger anomaly
            unrealized_pnl: -80.0,
            realized_pnl: 0.0,
            total_volume: 50000.0,
            trade_count: 10,
            win_rate: 0.3,
            profit_factor: 0.5,
            total_fees: 10.0,
            net_profit: -80.0,
            roi_percent: -8.0,
        };

        visualizer.add_profit_data("test_strategy", high_loss_point).await.unwrap();

        // 检查是否收到异常事件
        if let Ok(event) = receiver.try_recv() {
            match event {
                ProfitVisualizationEvent::AnomalyDetected { anomaly } => {
                    assert_eq!(anomaly.event_type, AnomalyType::DailyLossLimit);
                },
                _ => panic!("Expected anomaly detection event"),
            }
        }

        visualizer.stop().await.unwrap();
    }
}