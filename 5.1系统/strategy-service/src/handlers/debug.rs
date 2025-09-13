use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use crate::{AppState, models::{StandardResponse, Breakpoint}};

// GET /api/debug/sessions - 获取调试会话列表
pub async fn list_debug_sessions(
    State(state): State<AppState>,
) -> Response {
    let sessions = state.debug_manager.list_sessions().await;
    Json(StandardResponse::success(sessions)).into_response()
}

// POST /api/debug/sessions - 创建调试会话
pub async fn create_debug_session(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let strategy_id = payload.get("strategy_id")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    
    let session = state.debug_manager.create_session(strategy_id).await;
    Json(StandardResponse::success(session)).into_response()
}

// GET /api/debug/sessions/{id} - 获取调试会话详情
pub async fn get_debug_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.debug_manager.get_session(&id).await {
        Some(session) => Json(StandardResponse::success(session)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(StandardResponse::<Value>::error(format!("调试会话 {} 不存在", id)))).into_response()
    }
}

// DELETE /api/debug/sessions/{id} - 删除调试会话
pub async fn delete_debug_session(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.debug_manager.delete_session(&id).await {
        Ok(message) => Json(StandardResponse::success(message)).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

// GET /api/debug/breakpoints/{strategy_id} - 获取断点列表
pub async fn list_breakpoints(
    Path(strategy_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let breakpoints = state.debug_manager.list_breakpoints(&strategy_id).await;
    Json(StandardResponse::success(breakpoints)).into_response()
}

// POST /api/debug/breakpoints/{strategy_id} - 添加断点
pub async fn add_breakpoint(
    Path(strategy_id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Response {
    let line = payload.get("line").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
    let file = payload.get("file").and_then(|v| v.as_str()).unwrap_or("main.rs").to_string();
    let condition = payload.get("condition").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    let breakpoint = Breakpoint {
        id: uuid::Uuid::new_v4().to_string(),
        line,
        file,
        condition,
        enabled: true,
    };
    
    state.debug_manager.add_breakpoint(strategy_id.clone(), breakpoint).await;
    
    let response = json!({
        "strategy_id": strategy_id,
        "message": "断点已添加",
        "line": line
    });
    Json(StandardResponse::success(response)).into_response()
}

// DELETE /api/debug/breakpoints/{strategy_id}/{bp_id} - 移除断点
pub async fn remove_breakpoint(
    Path((strategy_id, bp_id)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Response {
    state.debug_manager.remove_breakpoint(&strategy_id, &bp_id).await;
    let response = json!({
        "strategy_id": strategy_id,
        "breakpoint_id": bp_id,
        "message": "断点已移除"
    });
    Json(StandardResponse::success(response)).into_response()
}

// GET /api/debug/variables/{strategy_id} - 获取变量
pub async fn get_variables(
    Path(strategy_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let variables = state.debug_manager.get_variables(&strategy_id).await;
    Json(StandardResponse::success(variables)).into_response()
}

// GET /api/debug/stack/{strategy_id} - 获取调用栈
pub async fn get_stack_trace(
    Path(strategy_id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    let stack_trace = state.debug_manager.get_stack_trace(&strategy_id).await;
    Json(StandardResponse::success(stack_trace)).into_response()
}

pub async fn enable_debug(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.debug_manager.enable_debug(&id).await {
        Ok(message) => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "message": message
        }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}

pub async fn disable_debug(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Response {
    match state.debug_manager.disable_debug(&id).await {
        Ok(message) => Json(StandardResponse::success(json!({
            "strategy_id": id,
            "message": message
        }))).into_response(),
        Err(error) => (StatusCode::BAD_REQUEST, Json(StandardResponse::<Value>::error(error.to_string()))).into_response()
    }
}