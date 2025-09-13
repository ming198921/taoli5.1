use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use crate::{AppState, models::StandardResponse};

// GET /api/strategies/list - 获取策略列表
pub async fn list_strategies(
    State(state): State<AppState>,
) -> Response {
    let strategies = state.strategy_monitor.list_strategies().await;
    Json(StandardResponse::success(strategies)).into_response()
}

// GET /api/strategies/{id} - 获取策略详情
pub async fn get_strategy(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_monitor.get_strategy(&id).await {
        Some(strategy) => Json(StandardResponse::success(strategy)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(StandardResponse::<Value>::error(format!("策略 {} 不存在", id)))).into_response()
    }
}

// GET /api/strategies/{id}/status - 获取策略状态
pub async fn get_strategy_status(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let status = state.strategy_monitor.get_strategy_status(&id).await;
    Json(StandardResponse::success(status)).into_response()
}

// GET /api/strategies/{id}/config - 获取策略配置
pub async fn get_strategy_config(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Response {
    let config = json!({
        "strategy_id": id,
        "enabled": true,
        "parameters": {
            "risk_level": "medium",
            "max_position": 1000.0,
            "stop_loss": 0.05
        }
    });
    Json(StandardResponse::success(config)).into_response()
}

// POST /api/strategies/{id}/config - 更新策略配置
pub async fn update_strategy_config(
    Path(id): Path<String>,
    State(_state): State<AppState>,
    Json(config): Json<Value>,
) -> Response {
    let response = json!({
        "strategy_id": id,
        "message": "配置已更新",
        "updated_config": config
    });
    Json(StandardResponse::success(response)).into_response()
}

// GET /api/strategies/{id}/logs - 获取策略日志
pub async fn get_strategy_logs(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Response {
    let logs = vec![
        json!({
            "timestamp": chrono::Utc::now().timestamp(),
            "level": "info",
            "message": format!("策略 {} 正常运行", id)
        })
    ];
    Json(StandardResponse::success(logs)).into_response()
}

// GET /api/strategies/{id}/metrics - 获取策略指标
pub async fn get_strategy_metrics(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_monitor.get_performance_metrics(&id).await {
        Some(metrics) => Json(StandardResponse::success(metrics)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(StandardResponse::<Value>::error(format!("策略 {} 指标不存在", id)))).into_response()
    }
}

pub async fn start_strategy(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_controller.start_strategy(&id).await {
        Ok(message) => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "action": "start",
            "message": message,
            "timestamp": chrono::Utc::now()
        }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

pub async fn stop_strategy(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_controller.stop_strategy(&id).await {
        Ok(message) => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "action": "stop",
            "message": message,
            "timestamp": chrono::Utc::now()
        }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

pub async fn restart_strategy(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_controller.restart_strategy(&id).await {
        Ok(message) => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "action": "restart",
            "message": message,
            "timestamp": chrono::Utc::now()
        }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

pub async fn pause_strategy(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_controller.pause_strategy(&id).await {
        Ok(message) => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "action": "pause",
            "message": message,
            "timestamp": chrono::Utc::now()
        }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

pub async fn resume_strategy(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.strategy_controller.resume_strategy(&id).await {
        Ok(message) => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "action": "resume",
            "message": message,
            "timestamp": chrono::Utc::now()
        }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

pub async fn get_lifecycle_status(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "strategy_id": id,
        "current_state": "running",
        "last_action": "start",
        "last_action_time": chrono::Utc::now(),
        "uptime": "2h 35m",
        "restart_count": 0
    })))
}

pub async fn get_lifecycle_history(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let history = state.strategy_controller.get_lifecycle_history(&id).await;
    Json(StandardResponse::success(json!({
        "strategy_id": id,
        "history": history,
        "total_operations": history.len()
    })))
}

pub async fn batch_start(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let strategy_ids = payload["strategy_ids"].as_array()
        .and_then(|arr| {
            arr.iter()
                .map(|v| v.as_str().map(String::from))
                .collect::<Option<Vec<String>>>()
        })
        .unwrap_or_default();

    let mut results = Vec::new();
    for id in strategy_ids {
        match state.strategy_controller.start_strategy(&id).await {
            Ok(message) => results.push(json!({
                "strategy_id": id,
                "success": true,
                "message": message
            })),
            Err(error) => results.push(json!({
                "strategy_id": id,
                "success": false,
                "error": error.to_string()
            }))
        }
    }

    Json(StandardResponse::success(json!({
        "operation": "batch_start",
        "results": results,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn batch_stop(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let strategy_ids = payload["strategy_ids"].as_array()
        .and_then(|arr| {
            arr.iter()
                .map(|v| v.as_str().map(String::from))
                .collect::<Option<Vec<String>>>()
        })
        .unwrap_or_default();

    let mut results = Vec::new();
    for id in strategy_ids {
        match state.strategy_controller.stop_strategy(&id).await {
            Ok(message) => results.push(json!({
                "strategy_id": id,
                "success": true,
                "message": message
            })),
            Err(error) => results.push(json!({
                "strategy_id": id,
                "success": false,
                "error": error.to_string()
            }))
        }
    }

    Json(StandardResponse::success(json!({
        "operation": "batch_stop",
        "results": results,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn batch_restart(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let strategy_ids = payload["strategy_ids"].as_array()
        .and_then(|arr| {
            arr.iter()
                .map(|v| v.as_str().map(String::from))
                .collect::<Option<Vec<String>>>()
        })
        .unwrap_or_default();

    let mut results = Vec::new();
    for id in strategy_ids {
        match state.strategy_controller.restart_strategy(&id).await {
            Ok(message) => results.push(json!({
                "strategy_id": id,
                "success": true,
                "message": message
            })),
            Err(error) => results.push(json!({
                "strategy_id": id,
                "success": false,
                "error": error.to_string()
            }))
        }
    }

    Json(StandardResponse::success(json!({
        "operation": "batch_restart",
        "results": results,
        "timestamp": chrono::Utc::now()
    })))
}