use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::{AppState, models::StandardResponse};

pub async fn get_realtime_status(State(state): State<AppState>) -> impl IntoResponse {
    let status = state.strategy_monitor.get_realtime_status().await;
    Json(StandardResponse::success(status))
}

pub async fn list_strategies(State(state): State<AppState>) -> impl IntoResponse {
    let strategies = state.strategy_monitor.list_strategies().await;
    Json(StandardResponse::success(strategies))
}

pub async fn get_strategy_status(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let status = state.strategy_monitor.get_strategy_status(&id).await;
    Json(StandardResponse::success(status)).into_response()
}

// GET /api/monitoring/health - 获取系统健康状态
pub async fn get_system_health(
    State(state): State<AppState>,
) -> Response {
    let realtime_status = state.strategy_monitor.get_realtime_status().await;
    let health = json!({
        "system_health": realtime_status.system_health,
        "active_strategies": realtime_status.active_count,
        "total_strategies": realtime_status.total_count,
        "timestamp": chrono::Utc::now().timestamp()
    });
    Json(StandardResponse::success(health)).into_response()
}

// GET /api/monitoring/performance - 获取性能概览
pub async fn get_performance_overview(
    State(state): State<AppState>,
) -> Response {
    let strategies = state.strategy_monitor.list_strategies().await;
    let total_cpu = strategies.iter().map(|s| s.performance.cpu_usage).sum::<f64>();
    let total_memory = strategies.iter().map(|s| s.performance.memory_usage).sum::<f64>();
    let avg_response_time = strategies.iter().map(|s| s.performance.response_time).sum::<f64>() / strategies.len() as f64;
    
    let overview = json!({
        "total_cpu_usage": total_cpu,
        "total_memory_usage": total_memory,
        "average_response_time": avg_response_time,
        "strategy_count": strategies.len()
    });
    Json(StandardResponse::success(overview)).into_response()
}

// GET /api/monitoring/alerts - 获取活跃告警
pub async fn get_active_alerts(
    State(_state): State<AppState>,
) -> Response {
    let alerts = vec![
        json!({
            "id": "alert_001",
            "level": "warning",
            "message": "CPU使用率过高",
            "strategy_id": "strategy_1",
            "timestamp": chrono::Utc::now().timestamp()
        })
    ];
    Json(StandardResponse::success(alerts)).into_response()
}

// GET /api/monitoring/metrics/cpu - 获取CPU指标
pub async fn get_cpu_metrics(
    State(state): State<AppState>,
) -> Response {
    let strategies = state.strategy_monitor.list_strategies().await;
    let cpu_metrics: Vec<_> = strategies.iter().map(|s| json!({
        "strategy_id": s.id,
        "cpu_usage": s.performance.cpu_usage
    })).collect();
    Json(StandardResponse::success(cpu_metrics)).into_response()
}

// GET /api/monitoring/metrics/memory - 获取内存指标
pub async fn get_memory_metrics(
    State(state): State<AppState>,
) -> Response {
    let strategies = state.strategy_monitor.list_strategies().await;
    let memory_metrics: Vec<_> = strategies.iter().map(|s| json!({
        "strategy_id": s.id,
        "memory_usage": s.performance.memory_usage
    })).collect();
    Json(StandardResponse::success(memory_metrics)).into_response()
}

// GET /api/monitoring/metrics/network - 获取网络指标
pub async fn get_network_metrics(
    State(state): State<AppState>,
) -> Response {
    let strategies = state.strategy_monitor.list_strategies().await;
    let network_metrics: Vec<_> = strategies.iter().map(|s| json!({
        "strategy_id": s.id,
        "network_usage": s.performance.network_usage
    })).collect();
    Json(StandardResponse::success(network_metrics)).into_response()
}

// GET /api/monitoring/metrics/history - 获取历史指标
pub async fn get_metrics_history(
    State(_state): State<AppState>,
) -> Response {
    let history = vec![
        json!({
            "timestamp": chrono::Utc::now().timestamp() - 3600,
            "cpu_usage": 45.2,
            "memory_usage": 512.0,
            "network_usage": 1024.0
        })
    ];
    Json(StandardResponse::success(history)).into_response()
}

pub async fn get_strategy_logs(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Response {
    let logs = vec![
        json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "level": "info",
            "message": format!("策略 {} 日志", id),
            "strategy_id": id
        })
    ];
    Json(StandardResponse::success(logs)).into_response()
}

pub async fn get_performance_metrics(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_monitor.get_performance_metrics(&id).await {
        Some(metrics) => Json(StandardResponse::success(metrics)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(StandardResponse::<Value>::error("Metrics not found".to_string()))).into_response()
    }
}

pub async fn check_strategy_health(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let status = state.strategy_monitor.get_strategy_status(&id).await;
    Json(StandardResponse::success(json!({
        "strategy_id": id,
        "health": status.health,
        "status": status.status,
        "last_update": status.last_update,
        "performance": status.performance
    }))).into_response()
}

pub async fn get_strategy_errors(
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|s| s.parse::<usize>().ok()).unwrap_or(50);
    
    let mock_errors = (0..std::cmp::min(limit, 5)).map(|i| {
        json!({
            "timestamp": chrono::Utc::now(),
            "error_code": format!("ERR_{:03}", 500 + i),
            "message": format!("Mock error {} for strategy {}", i, id),
            "severity": if i % 2 == 0 { "HIGH" } else { "MEDIUM" },
            "resolved": i % 3 == 0
        })
    }).collect::<Vec<_>>();

    Json(StandardResponse::success(json!({
        "strategy_id": id,
        "errors": mock_errors,
        "total": mock_errors.len()
    })))
}

pub async fn get_alerts(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let severity = params.get("severity").unwrap_or(&"all".to_string()).clone();
    
    let mock_alerts = vec![
        json!({
            "id": "alert_001",
            "strategy_id": "strategy_1",
            "severity": "HIGH",
            "message": "CPU usage exceeded 90%",
            "timestamp": chrono::Utc::now(),
            "acknowledged": false
        }),
        json!({
            "id": "alert_002", 
            "strategy_id": "strategy_2",
            "severity": "MEDIUM",
            "message": "Memory usage above 80%",
            "timestamp": chrono::Utc::now(),
            "acknowledged": true
        })
    ];

    let filtered_alerts = if severity == "all" {
        mock_alerts
    } else {
        mock_alerts.into_iter()
            .filter(|alert| alert["severity"].as_str() == Some(&severity))
            .collect()
    };

    Json(StandardResponse::success(json!({
        "alerts": filtered_alerts,
        "total": filtered_alerts.len(),
        "filter": severity
    })))
}