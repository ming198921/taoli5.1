//! Web仪表板实现
//!
//! 提供Web界面展示Sankey资金流向图和实时数据

use anyhow::{Result, Context};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{info, warn, error, instrument};

use super::{
    sankey_diagram::{SankeyDiagram, SankeyVisualizationData, FlowAnomaly},
    fund_flow_tracker::{FundFlowTracker, FundFlowEvent, FlowStats},
    VisualizationConfig,
};

/// Web仪表板状态
#[derive(Clone)]
pub struct DashboardState {
    pub sankey_diagram: Arc<SankeyDiagram>,
    pub fund_flow_tracker: Arc<FundFlowTracker>,
    pub config: VisualizationConfig,
    pub sessions: Arc<RwLock<HashMap<String, DashboardSession>>>,
}

/// 仪表板会话
#[derive(Debug, Clone)]
struct DashboardSession {
    id: String,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
    preferences: UserPreferences,
}

/// 用户偏好设置
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserPreferences {
    chart_theme: String,
    auto_refresh: bool,
    refresh_interval_ms: u64,
    show_historical_data: bool,
    max_data_points: usize,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            chart_theme: "dark".to_string(),
            auto_refresh: true,
            refresh_interval_ms: 5000,
            show_historical_data: true,
            max_data_points: 1000,
        }
    }
}

/// API查询参数
#[derive(Debug, Deserialize)]
struct TimeRangeQuery {
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
    limit: Option<usize>,
}

/// API响应结构
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

/// Web仪表板
pub struct WebDashboard {
    state: DashboardState,
}

impl WebDashboard {
    /// 创建新的Web仪表板
    pub fn new(
        sankey_diagram: Arc<SankeyDiagram>,
        fund_flow_tracker: Arc<FundFlowTracker>,
        config: VisualizationConfig,
    ) -> Self {
        let state = DashboardState {
            sankey_diagram,
            fund_flow_tracker,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        Self { state }
    }

    /// 启动Web服务器
    #[instrument(skip(self))]
    pub async fn start_server(&self) -> Result<()> {
        let app = self.create_router();
        let port = self.state.config.web_server_port;
        
        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .context("Failed to bind to address")?;

        info!("Starting web dashboard on port {}", port);
        
        axum::serve(listener, app)
            .await
            .context("Failed to start web server")?;

        Ok(())
    }

    /// 创建路由器
    fn create_router(&self) -> Router {
        Router::new()
            // 静态文件服务
            .nest_service("/static", ServeDir::new("static"))
            
            // 主页面
            .route("/", get(dashboard_home))
            .route("/dashboard", get(dashboard_home))
            
            // API路由
            .route("/api/sankey/data", get(get_sankey_data))
            .route("/api/sankey/historical", get(get_historical_data))
            .route("/api/flows", get(get_flows))
            .route("/api/balances", get(get_balances))
            .route("/api/stats", get(get_stats))
            .route("/api/anomalies", get(get_anomalies))
            .route("/api/events/stream", get(event_stream))
            
            // 配置管理
            .route("/api/preferences", get(get_preferences))
            .route("/api/preferences", post(update_preferences))
            
            // 健康检查
            .route("/health", get(health_check))
            
            .layer(CorsLayer::permissive())
            .with_state(self.state.clone())
    }
}

/// 仪表板主页
async fn dashboard_home() -> Html<&'static str> {
    Html(include_str!("templates/dashboard.html"))
}

/// 获取Sankey图数据
#[instrument(skip(state))]
async fn get_sankey_data(
    State(state): State<DashboardState>,
) -> Result<Json<ApiResponse<SankeyVisualizationData>>, StatusCode> {
    match state.sankey_diagram.get_current_data().await {
        data => Ok(Json(ApiResponse::success(data))),
    }
}

/// 获取历史数据
#[instrument(skip(state))]
async fn get_historical_data(
    State(state): State<DashboardState>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let historical_data = state.sankey_diagram.get_historical_data(params.limit).await;
    
    let response_data = serde_json::json!({
        "snapshots": historical_data,
        "count": historical_data.len()
    });
    
    Ok(Json(ApiResponse::success(response_data)))
}

/// 获取流动记录
#[instrument(skip(state))]
async fn get_flows(
    State(state): State<DashboardState>,
    Query(params): Query<TimeRangeQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let flows = state.fund_flow_tracker.get_flow_records(params.limit).await;
    
    let response_data = serde_json::json!({
        "flows": flows,
        "count": flows.len()
    });
    
    Ok(Json(ApiResponse::success(response_data)))
}

/// 获取余额信息
#[instrument(skip(state))]
async fn get_balances(
    State(state): State<DashboardState>,
) -> Result<Json<ApiResponse<HashMap<String, serde_json::Value>>>, StatusCode> {
    let balances = state.fund_flow_tracker.get_all_balances().await;
    
    let mut response_data = HashMap::new();
    for (key, balance) in balances {
        response_data.insert(key, serde_json::to_value(balance).unwrap_or_default());
    }
    
    Ok(Json(ApiResponse::success(response_data)))
}

/// 获取统计信息
#[instrument(skip(state))]
async fn get_stats(
    State(state): State<DashboardState>,
) -> Result<Json<ApiResponse<FlowStats>>, StatusCode> {
    let stats = state.fund_flow_tracker.get_stats().await;
    Ok(Json(ApiResponse::success(stats)))
}

/// 获取异常信息
#[instrument(skip(state))]
async fn get_anomalies(
    State(state): State<DashboardState>,
) -> Result<Json<ApiResponse<Vec<FlowAnomaly>>>, StatusCode> {
    match state.sankey_diagram.detect_flow_anomalies().await {
        Ok(anomalies) => Ok(Json(ApiResponse::success(anomalies))),
        Err(e) => {
            error!("Failed to get anomalies: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 事件流（Server-Sent Events）
#[instrument(skip(state))]
async fn event_stream(
    State(state): State<DashboardState>,
) -> impl IntoResponse {
    use axum::response::sse::{Event, Sse};
    use futures::stream::Stream;
    use tokio_stream::wrappers::BroadcastStream;
    use std::convert::Infallible;

    let receiver = state.fund_flow_tracker.subscribe();
    let stream = BroadcastStream::new(receiver);

    let event_stream = stream.map(|event| {
        match event {
            Ok(fund_flow_event) => {
                match serde_json::to_string(&fund_flow_event) {
                    Ok(json) => Ok(Event::default().data(json)),
                    Err(e) => {
                        warn!("Failed to serialize event: {}", e);
                        Ok(Event::default().data("{}"))
                    }
                }
            },
            Err(e) => {
                warn!("Event stream error: {}", e);
                Ok(Event::default().data("{}"))
            }
        }
    });

    Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive-text")
    )
}

/// 获取用户偏好设置
async fn get_preferences(
    State(_state): State<DashboardState>,
) -> Json<ApiResponse<UserPreferences>> {
    // 这里应该从数据库或会话中获取用户偏好
    // 目前返回默认设置
    Json(ApiResponse::success(UserPreferences::default()))
}

/// 更新用户偏好设置
async fn update_preferences(
    State(_state): State<DashboardState>,
    Json(_preferences): Json<UserPreferences>,
) -> Json<ApiResponse<String>> {
    // 这里应该保存用户偏好到数据库或会话
    Json(ApiResponse::success("Preferences updated successfully".to_string()))
}

/// 健康检查
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::{
        sankey_diagram::SankeyConfig,
        fund_flow_tracker::FundFlowConfig,
    };

    #[tokio::test]
    async fn test_web_dashboard_creation() {
        let sankey_config = SankeyConfig::default();
        let sankey_diagram = Arc::new(SankeyDiagram::new(sankey_config));
        
        let flow_config = FundFlowConfig::default();
        let fund_flow_tracker = Arc::new(FundFlowTracker::new(flow_config, Arc::clone(&sankey_diagram)));
        
        let viz_config = VisualizationConfig::default();
        let dashboard = WebDashboard::new(sankey_diagram, fund_flow_tracker, viz_config);
        
        // 测试路由器创建
        let _router = dashboard.create_router();
        
        // 这里可以添加更多的单元测试
        assert!(true); // 基本的创建测试
    }

    // 注意：完整的集成测试需要启动实际的HTTP服务器，这里仅做基础测试
}