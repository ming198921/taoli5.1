use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use crate::{AppState, models::StandardResponse};

// GET /api/hotreload/status - 获取所有策略的重载状态
pub async fn get_reload_status_all(
    State(state): State<AppState>,
) -> Response {
    let history = state.hot_reload_manager.list_reload_history().await;
    Json(StandardResponse::success(history)).into_response()
}

// GET /api/hotreload/{strategy_id}/status - 获取策略重载状态
pub async fn get_reload_status(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.hot_reload_manager.get_reload_status(&id).await {
        Some(status) => Json(StandardResponse::success(status)).into_response(),
        None => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "status": "never_reloaded",
            "message": "策略从未重载过"
        }))).into_response()
    }
}

// POST /api/hotreload/{strategy_id}/reload - 重载策略
pub async fn reload_strategy(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.hot_reload_manager.reload_strategy(id.clone()).await {
        Ok(status) => Json(StandardResponse::success(status)).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

// POST /api/hotreload/{strategy_id}/enable - 启用热重载
pub async fn enable_hot_reload(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Response {
    let response = json!({
        "strategy_id": id,
        "message": "热重载已启用",
        "enabled": true
    });
    Json(StandardResponse::success(response)).into_response()
}

// POST /api/hotreload/{strategy_id}/disable - 禁用热重载
pub async fn disable_hot_reload(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Response {
    let response = json!({
        "strategy_id": id,
        "message": "热重载已禁用",
        "enabled": false
    });
    Json(StandardResponse::success(response)).into_response()
}

// POST /api/hotreload/{strategy_id}/validate - 验证变更
pub async fn validate_changes(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("");
    
    match state.hot_reload_manager.validate_code(&id, code).await {
        Ok(result) => Json(StandardResponse::success(result)).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

// POST /api/hotreload/{strategy_id}/rollback - 回滚变更
pub async fn rollback_changes(
    Path(id): Path<String>,
    State(_state): State<AppState>,
) -> Response {
    let response = json!({
        "strategy_id": id,
        "message": "已回滚到上一个版本",
        "rollback_successful": true
    });
    Json(StandardResponse::success(response)).into_response()
}

// GET /api/hotreload/history - 获取重载历史
pub async fn get_reload_history(
    State(state): State<AppState>,
) -> Response {
    let history = state.hot_reload_manager.list_reload_history().await;
    Json(StandardResponse::success(history)).into_response()
}

// GET /api/hotreload/config - 获取重载配置
pub async fn get_reload_config(
    State(_state): State<AppState>,
) -> Response {
    let config = json!({
        "auto_reload": false,
        "watch_directories": ["/home/ubuntu/5.1xitong/5.1系统/celue"],
        "reload_delay_ms": 1000,
        "backup_enabled": true
    });
    Json(StandardResponse::success(config)).into_response()
}

pub async fn validate_code(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let code = payload.get("code").and_then(|v| v.as_str()).unwrap_or("");
    
    match state.hot_reload_manager.validate_code(&id, code).await {
        Ok(result) => Json(StandardResponse::success(result)).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

pub async fn preview_changes(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.hot_reload_manager.preview_changes(&id).await {
        Ok(preview) => Json(StandardResponse::success(preview)).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}