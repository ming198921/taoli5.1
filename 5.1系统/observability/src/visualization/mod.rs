//! 可视化模块
//! 
//! 实现实时盈利曲线、Sankey资金流向图和多维性能分析可视化

pub mod sankey_diagram;
pub mod profit_curve_visualizer;
pub mod multi_dimensional_analyzer;
pub mod strategy_performance_chart;
pub mod fund_flow_tracker;
pub mod web_dashboard;
pub mod visualization_manager;

// Re-export main types for convenience
pub use sankey_diagram::{
    SankeyDiagram, SankeyNode, SankeyLink, SankeyConfig, SankeyVisualizationData,
    FlowType, NodeStatus, LinkStatus, FlowAnomaly, AnomalyType, AnomalySeverity
};
pub use fund_flow_tracker::{
    FundFlowTracker, FundFlowConfig, ExchangeBalance, FlowRecord, FundFlowEvent,
    FlowStats, BalanceChangeReason, FlowStatus
};
pub use web_dashboard::{WebDashboard, DashboardState};
pub use visualization_manager::{
    VisualizationManager, VisualizationManagerConfig, VisualizationEvent,
    VisualizationQuery, DashboardData, SystemStats, SystemHealthStatus
};
pub use profit_curve_visualizer::*;
pub use multi_dimensional_analyzer::*;
pub use strategy_performance_chart::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 可视化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Web服务器端口
    pub web_server_port: u16,
    /// 启用实时更新
    pub enable_real_time_updates: bool,
    /// 更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 最大数据点数量
    pub max_data_points: usize,
    /// 历史数据保留天数
    pub history_retention_days: u32,
    /// 图表主题
    pub chart_theme: ChartTheme,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            web_server_port: 8080,
            enable_real_time_updates: true,
            update_interval_ms: 1000,
            max_data_points: 10000,
            history_retention_days: 30,
            chart_theme: ChartTheme::Dark,
        }
    }
}

/// 图表主题
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChartTheme {
    Light,
    Dark,
    HighContrast,
}

/// 可视化数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub label: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 可视化系列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationSeries {
    pub name: String,
    pub data_points: Vec<VisualizationDataPoint>,
    pub color: String,
    pub line_type: LineType,
}

/// 线条类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LineType {
    Solid,
    Dashed,
    Dotted,
    DashDot,
}

/// 图表类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChartType {
    Line,
    Bar,
    Area,
    Scatter,
    Heatmap,
    Sankey,
    Candlestick,
}

/// 可视化响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationResponse {
    pub chart_type: ChartType,
    pub title: String,
    pub series: Vec<VisualizationSeries>,
    pub x_axis_label: String,
    pub y_axis_label: String,
    pub timestamp: DateTime<Utc>,
}