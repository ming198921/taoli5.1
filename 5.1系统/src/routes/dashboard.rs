use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use common_types::ApiResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 仪表板配置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DashboardConfig {
    pub title: String,
    pub refresh_interval: u32,
    pub theme: String,
    pub widgets: Vec<Widget>,
}

/// 仪表板部件
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Widget {
    pub id: String,
    pub widget_type: String,
    pub title: String,
    pub position: Position,
    pub size: Size,
    pub config: serde_json::Value,
}

/// 部件位置
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

/// 部件尺寸
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

/// Sankey图数据
#[derive(Debug, Serialize)]
pub struct SankeyData {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub timestamp: i64,
}

/// Sankey节点
#[derive(Debug, Serialize)]
pub struct SankeyNode {
    pub id: String,
    pub name: String,
    pub category: String,
    pub value: f64,
}

/// Sankey链接
#[derive(Debug, Serialize)]
pub struct SankeyLink {
    pub source: String,
    pub target: String,
    pub value: f64,
    pub flow_type: String,
}

/// 资金流历史
#[derive(Debug, Serialize)]
pub struct FlowHistory {
    pub timeframe: String,
    pub data: Vec<FlowDataPoint>,
}

/// 流数据点
#[derive(Debug, Serialize)]
pub struct FlowDataPoint {
    pub timestamp: i64,
    pub inflow: f64,
    pub outflow: f64,
    pub net_flow: f64,
    pub volume: f64,
}

/// 仪表板路由
pub fn routes<S>(state: Arc<S>) -> Router
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/config", get(get_dashboard_config).post(update_dashboard_config))
        .route("/sankey/data", get(get_sankey_data))
        .route("/sankey/realtime", get(get_realtime_sankey))
        .route("/flows/history", get(get_flow_history))
        .route("/flows/current", get(get_current_flows))
        .route("/widgets", get(get_widgets).post(create_widget))
        .route("/widgets/:widget_id", get(get_widget).put(update_widget).delete(delete_widget))
        .route("/charts/profit-curve", get(get_profit_curve))
        .route("/charts/performance", get(get_performance_chart))
        .route("/alerts/anomalies", get(get_flow_anomalies))
        .route("/export/data", get(export_dashboard_data))
        .with_state(state)
}

/// 获取仪表板配置
async fn get_dashboard_config<S>(
    State(_state): State<Arc<S>>,
) -> impl IntoResponse {
    let config = DashboardConfig {
        title: "5.1套利系统仪表板".to_string(),
        refresh_interval: 5000, // 5秒
        theme: "dark".to_string(),
        widgets: vec![
            Widget {
                id: "sankey_main".to_string(),
                widget_type: "sankey".to_string(),
                title: "资金流向图".to_string(),
                position: Position { x: 0, y: 0 },
                size: Size { width: 12, height: 8 },
                config: serde_json::json!({"show_values": true}),
            },
            Widget {
                id: "profit_curve".to_string(),
                widget_type: "line_chart".to_string(),
                title: "收益曲线".to_string(),
                position: Position { x: 0, y: 8 },
                size: Size { width: 6, height: 4 },
                config: serde_json::json!({"timeframe": "24h"}),
            },
        ],
    };
    
    (StatusCode::OK, Json(ApiResponse::success(config)))
}

/// 更新仪表板配置
async fn update_dashboard_config<S>(
    State(_state): State<Arc<S>>,
    Json(config): Json<DashboardConfig>,
) -> impl IntoResponse {
    // TODO: 保存配置到存储
    let response = ApiResponse::success(serde_json::json!({
        "message": "Dashboard configuration updated successfully",
        "config": config
    }));
    
    (StatusCode::OK, Json(response))
}

/// 获取Sankey图数据
async fn get_sankey_data<S>(
    State(_state): State<Arc<S>>,
) -> impl IntoResponse {
    let data = SankeyData {
        nodes: vec![
            SankeyNode {
                id: "binance".to_string(),
                name: "Binance".to_string(),
                category: "exchange".to_string(),
                value: 100000.0,
            },
            SankeyNode {
                id: "okx".to_string(),
                name: "OKX".to_string(),
                category: "exchange".to_string(),
                value: 95000.0,
            },
            SankeyNode {
                id: "arbitrage_pool".to_string(),
                name: "套利池".to_string(),
                category: "strategy".to_string(),
                value: 15000.0,
            },
        ],
        links: vec![
            SankeyLink {
                source: "binance".to_string(),
                target: "arbitrage_pool".to_string(),
                value: 10000.0,
                flow_type: "arbitrage".to_string(),
            },
            SankeyLink {
                source: "arbitrage_pool".to_string(),
                target: "okx".to_string(),
                value: 5000.0,
                flow_type: "profit".to_string(),
            },
        ],
        timestamp: chrono::Utc::now().timestamp_millis(),
    };
    
    (StatusCode::OK, Json(ApiResponse::success(data)))
}

/// 获取实时Sankey数据
async fn get_realtime_sankey<S>(
    State(_state): State<Arc<S>>,
) -> impl IntoResponse {
    // TODO: 实现实时数据流
    let data = get_sankey_data(State(_state)).await;
    data
}

/// 获取资金流历史
async fn get_flow_history<S>(
    State(_state): State<Arc<S>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let timeframe = params.get("timeframe").unwrap_or(&"24h".to_string()).clone();
    
    let history = FlowHistory {
        timeframe: timeframe.clone(),
        data: vec![
            FlowDataPoint {
                timestamp: chrono::Utc::now().timestamp_millis() - 3600000, // 1小时前
                inflow: 50000.0,
                outflow: 45000.0,
                net_flow: 5000.0,
                volume: 95000.0,
            },
            FlowDataPoint {
                timestamp: chrono::Utc::now().timestamp_millis() - 1800000, // 30分钟前
                inflow: 52000.0,
                outflow: 48000.0,
                net_flow: 4000.0,
                volume: 100000.0,
            },
            FlowDataPoint {
                timestamp: chrono::Utc::now().timestamp_millis(),
                inflow: 55000.0,
                outflow: 50000.0,
                net_flow: 5000.0,
                volume: 105000.0,
            },
        ],
    };
    
    (StatusCode::OK, Json(ApiResponse::success(history)))
}

/// 获取当前资金流
async fn get_current_flows<S>(
    State(_state): State<Arc<S>>,
) -> impl IntoResponse {
    let flows = serde_json::json!({
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "total_inflow": 55000.0,
        "total_outflow": 50000.0,
        "net_flow": 5000.0,
        "active_flows": 12,
        "flows_by_exchange": {
            "binance": {"inflow": 25000.0, "outflow": 20000.0},
            "okx": {"inflow": 20000.0, "outflow": 22000.0},
            "huobi": {"inflow": 10000.0, "outflow": 8000.0}
        }
    });
    
    (StatusCode::OK, Json(ApiResponse::success(flows)))
}

/// 获取所有部件
async fn get_widgets<S>(
    State(_state): State<Arc<S>>,
) -> impl IntoResponse {
    // TODO: 从存储获取部件列表
    let widgets = vec![
        serde_json::json!({
            "id": "sankey_main",
            "type": "sankey",
            "title": "资金流向图",
            "status": "active"
        }),
        serde_json::json!({
            "id": "profit_curve",
            "type": "line_chart", 
            "title": "收益曲线",
            "status": "active"
        }),
    ];
    
    (StatusCode::OK, Json(ApiResponse::success(widgets)))
}

/// 创建部件
async fn create_widget<S>(
    State(_state): State<Arc<S>>,
    Json(widget): Json<Widget>,
) -> impl IntoResponse {
    // TODO: 保存部件到存储
    let response = ApiResponse::success(serde_json::json!({
        "message": "Widget created successfully",
        "widget_id": widget.id
    }));
    
    (StatusCode::CREATED, Json(response))
}

/// 获取特定部件
async fn get_widget<S>(
    State(_state): State<Arc<S>>,
    Path(widget_id): Path<String>,
) -> impl IntoResponse {
    // TODO: 从存储获取部件
    let widget = serde_json::json!({
        "id": widget_id,
        "type": "sankey",
        "title": "资金流向图",
        "position": {"x": 0, "y": 0},
        "size": {"width": 12, "height": 8},
        "config": {"show_values": true}
    });
    
    (StatusCode::OK, Json(ApiResponse::success(widget)))
}

/// 更新部件
async fn update_widget<S>(
    State(_state): State<Arc<S>>,
    Path(widget_id): Path<String>,
    Json(widget): Json<Widget>,
) -> impl IntoResponse {
    // TODO: 更新部件存储
    let response = ApiResponse::success(serde_json::json!({
        "message": "Widget updated successfully",
        "widget_id": widget_id
    }));
    
    (StatusCode::OK, Json(response))
}

/// 删除部件
async fn delete_widget<S>(
    State(_state): State<Arc<S>>,
    Path(widget_id): Path<String>,
) -> impl IntoResponse {
    // TODO: 从存储删除部件
    let response = ApiResponse::success(serde_json::json!({
        "message": "Widget deleted successfully",
        "widget_id": widget_id
    }));
    
    (StatusCode::OK, Json(response))
}

/// 获取收益曲线
async fn get_profit_curve<S>(
    State(_state): State<Arc<S>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let timeframe = params.get("timeframe").unwrap_or(&"24h".to_string()).clone();
    
    let curve = serde_json::json!({
        "timeframe": timeframe,
        "data": [
            {"time": 1706352000, "profit": 1000.0, "cumulative": 1000.0},
            {"time": 1706355600, "profit": 1500.0, "cumulative": 2500.0},
            {"time": 1706359200, "profit": -200.0, "cumulative": 2300.0},
            {"time": 1706362800, "profit": 800.0, "cumulative": 3100.0},
        ]
    });
    
    (StatusCode::OK, Json(ApiResponse::success(curve)))
}

/// 获取性能图表
async fn get_performance_chart<S>(
    State(_state): State<Arc<S>>,
) -> impl IntoResponse {
    let performance = serde_json::json!({
        "metrics": {
            "total_trades": 150,
            "successful_trades": 142,
            "success_rate": 0.9467,
            "average_profit": 45.5,
            "max_drawdown": 0.12,
            "sharpe_ratio": 2.34
        },
        "daily_performance": [
            {"date": "2025-01-26", "pnl": 2300.0, "trades": 45},
            {"date": "2025-01-27", "pnl": 2800.0, "trades": 52},
        ]
    });
    
    (StatusCode::OK, Json(ApiResponse::success(performance)))
}

/// 获取资金流异常
async fn get_flow_anomalies<S>(
    State(_state): State<Arc<S>>,
) -> impl IntoResponse {
    let anomalies = serde_json::json!({
        "anomalies": [
            {
                "id": "anom_001",
                "type": "unusual_volume",
                "severity": "medium",
                "exchange": "binance",
                "symbol": "BTC/USDT",
                "description": "交易量异常增加300%",
                "timestamp": chrono::Utc::now().timestamp_millis() - 300000,
                "status": "investigating"
            },
            {
                "id": "anom_002", 
                "type": "flow_interruption",
                "severity": "high",
                "exchange": "okx",
                "symbol": "ETH/USDT",
                "description": "资金流中断超过5分钟",
                "timestamp": chrono::Utc::now().timestamp_millis() - 600000,
                "status": "resolved"
            }
        ],
        "total": 2
    });
    
    (StatusCode::OK, Json(ApiResponse::success(anomalies)))
}

/// 导出仪表板数据
async fn export_dashboard_data<S>(
    State(_state): State<Arc<S>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let format = params.get("format").unwrap_or(&"json".to_string()).clone();
    let timeframe = params.get("timeframe").unwrap_or(&"24h".to_string()).clone();
    
    let export_data = serde_json::json!({
        "export_info": {
            "format": format,
            "timeframe": timeframe,
            "generated_at": chrono::Utc::now(),
            "record_count": 1000
        },
        "download_url": "/api/v1/dashboard/exports/download/latest.json",
        "expires_at": chrono::Utc::now().timestamp_millis() + 3600000 // 1小时后过期
    });
    
    (StatusCode::OK, Json(ApiResponse::success(export_data)))
}