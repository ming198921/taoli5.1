//! 前端TypeScript类型的Rust镜像定义
//! 用于精确验证前后端数据结构兼容性

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 前端ApiResponse类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub message: Option<String>,
    pub timestamp: Option<i64>, // 注意：前端使用number类型
}

// 使用统一的ArbitrageOpportunity定义 - 与前端完全兼容
pub use common_types::ArbitrageOpportunity;

/// 前端SystemStatus类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendSystemStatus {
    #[serde(rename = "isRunning")]
    pub is_running: bool,
    pub qingxi: String, // 'running' | 'stopped' | 'error' | 'warning'
    pub celue: String,
    pub architecture: String,
    pub observability: String,
    pub uptime: f64,
    #[serde(rename = "lastUpdate")]
    pub last_update: String,
}

// 使用统一的MarketData定义 - 与前端兼容
pub use common_types::{MarketData as FrontendMarketData};

/// 前端RiskAlert类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendRiskAlert {
    pub id: String,
    #[serde(rename = "type")]
    pub alert_type: String, // 'position_limit' | 'loss_limit' | etc.
    pub severity: String, // 'low' | 'medium' | 'high' | 'critical'
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub created_at: String,
    pub resolved_at: Option<String>,
    pub status: String, // 'active' | 'resolved' | 'ignored'
}

/// 前端StrategyConfig类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendStrategyConfig {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub strategy_type: String, // 'cross_exchange' | 'triangular' | etc.
    pub enabled: bool,
    pub priority: i32,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub risk_limits: FrontendRiskLimits,
    pub exchanges: Vec<String>,
    pub symbols: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub performance_metrics: Option<FrontendStrategyPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendRiskLimits {
    pub max_position_size_usd: f64,
    pub max_daily_loss_usd: f64,
    pub max_exposure_per_exchange: f64,
    pub max_correlation_risk: f64,
    pub stop_loss_percentage: f64,
    pub max_drawdown_percentage: f64,
    pub position_concentration_limit: f64,
    pub leverage_limit: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendStrategyPerformance {
    pub total_pnl_usd: f64,
    pub daily_pnl_usd: f64,
    pub win_rate_percent: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown_percent: f64,
    pub total_trades: i32,
    pub successful_trades: i32,
    pub average_trade_duration_minutes: f64,
    pub return_on_capital_percent: f64,
}

/// 前端SystemResourceUsage类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendSystemResourceUsage {
    pub timestamp: String,
    pub cpu: FrontendCpuUsage,
    pub memory: FrontendMemoryUsage,
    pub disk: FrontendDiskUsage,
    pub network: FrontendNetworkUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendCpuUsage {
    pub usage_percent: f64,
    pub load_average: [f64; 3],
    pub core_count: i32,
    pub temperature_celsius: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendMemoryUsage {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub cached_gb: f64,
    pub swap_used_gb: f64,
    pub swap_total_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendDiskUsage {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub io_read_bps: f64,
    pub io_write_bps: f64,
    pub iops_read: f64,
    pub iops_write: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendNetworkUsage {
    pub interfaces: Vec<FrontendNetworkInterface>,
    pub total_bytes_sent: f64,
    pub total_bytes_received: f64,
    pub connections_active: i32,
    pub connections_established: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendNetworkInterface {
    pub name: String,
    pub bytes_sent: f64,
    pub bytes_received: f64,
    pub packets_sent: f64,
    pub packets_received: f64,
    pub errors_in: f64,
    pub errors_out: f64,
    pub dropped_in: f64,
    pub dropped_out: f64,
}

/// 前端WebSocket消息格式镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendWebSocketMessage<T> {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: T,
    pub timestamp: String,
    pub id: Option<String>,
}

/// 前端TimeSeriesData类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendTimeSeriesData {
    pub timestamp: String,
    pub value: f64,
    pub label: Option<String>,
}

/// 前端User类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendUser {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String, // 'admin' | 'trader' | 'viewer' | 'auditor'
    pub permissions: Vec<String>,
    #[serde(rename = "lastLogin")]
    pub last_login: String,
}

/// 前端AuthToken类型镜像
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendAuthToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
    pub token_type: String,
}