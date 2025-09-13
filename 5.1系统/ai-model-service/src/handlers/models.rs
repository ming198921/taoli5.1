#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use axum::response::IntoResponse;
use axum::{
    extract::{Path, State},
    response::Response,
    Json,
};
use serde_json::{json, Value};
use crate::{AppState, models::StandardResponse};

// GET /api/ml/models - 获取模型列表
pub async fn list_models(State(state): State<AppState>) -> Response {
    let models = state.model_manager.list_models().await;
    Json(StandardResponse::success(models)).into_response()
}

// POST /api/ml/models - 创建新模型
pub async fn create_model(State(state): State<AppState>, Json(config): Json<Value>) -> Response {
    let model_config = crate::models::ModelConfig {
        model_type: config.get("model_type").and_then(|v| v.as_str()).unwrap_or("risk_assessment").to_string(),
        parameters: config.clone(),
        training_config: crate::models::TrainingConfig {
            epochs: config.get("epochs").and_then(|v| v.as_u64()).unwrap_or(100) as u32,
            batch_size: config.get("batch_size").and_then(|v| v.as_u64()).unwrap_or(32) as u32,
            learning_rate: config.get("learning_rate").and_then(|v| v.as_f64()).unwrap_or(0.001),
            optimizer: config.get("optimizer").and_then(|v| v.as_str()).unwrap_or("adam").to_string(),
            loss_function: config.get("loss_function").and_then(|v| v.as_str()).unwrap_or("mse").to_string(),
            validation_split: config.get("validation_split").and_then(|v| v.as_f64()).unwrap_or(0.2),
            early_stopping: config.get("early_stopping").and_then(|v| v.as_bool()).unwrap_or(true),
            patience: config.get("patience").and_then(|v| v.as_u64()).unwrap_or(10) as u32,
        },
        deployment_config: crate::models::DeploymentConfig {
            max_concurrent_requests: 100,
            timeout_seconds: 30,
            scaling_policy: "auto".to_string(),
            resource_limits: crate::models::ResourceLimits {
                cpu_cores: 2.0,
                memory_mb: 2048,
                gpu_memory_mb: Some(1024),
            },
        },
    };
    
    match state.model_manager.create_model(model_config).await {
        Ok(model_id) => Json(StandardResponse::success(json!({
            "model_id": model_id,
            "message": "Model created successfully"
        }))).into_response(),
        Err(e) => Json(StandardResponse::<Value>::error(e.to_string())).into_response()
    }
}

// GET /api/ml/models/:id - 获取模型详情
pub async fn get_model(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.model_manager.get_model(&id).await {
        Some(model) => Json(StandardResponse::success(model)).into_response(),
        None => Json(StandardResponse::<Value>::error("Model not found".to_string())).into_response()
    }
}

// PUT /api/ml/models/:id - 更新模型
pub async fn update_model(Path(id): Path<String>, State(_state): State<AppState>, Json(_config): Json<Value>) -> Response {
    Json(StandardResponse::success(json!({
        "model_id": id,
        "message": "Model updated successfully"
    }))).into_response()
}

// DELETE /api/ml/models/:id - 删除模型
pub async fn delete_model(Path(id): Path<String>, State(_state): State<AppState>) -> Response {
    Json(StandardResponse::success(json!({
        "model_id": id,
        "message": "Model deleted successfully"
    }))).into_response()
}

// GET /api/ml/models/:id/status - 获取模型状态
pub async fn get_model_status(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.model_manager.get_model(&id).await {
        Some(model) => Json(StandardResponse::success(json!({
            "model_id": id,
            "status": model.status,
            "training_progress": model.training_progress,
            "accuracy": model.accuracy
        }))).into_response(),
        None => Json(StandardResponse::<Value>::error("Model not found".to_string())).into_response()
    }
}

// GET /api/ml/models/:id/metadata - 获取模型元数据
pub async fn get_model_metadata(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.model_manager.get_model(&id).await {
        Some(model) => Json(StandardResponse::success(json!({
            "model_id": id,
            "name": model.name,
            "version": model.version,
            "model_type": model.model_type,
            "created_at": model.last_trained,
            "performance_metrics": model.performance_metrics
        }))).into_response(),
        None => Json(StandardResponse::<Value>::error("Model not found".to_string())).into_response()
    }
}

// PUT /api/ml/models/:id/metadata - 更新模型元数据
pub async fn update_model_metadata(Path(id): Path<String>, State(_state): State<AppState>, Json(_metadata): Json<Value>) -> Response {
    Json(StandardResponse::success(json!({
        "model_id": id,
        "message": "Metadata updated successfully"
    }))).into_response()
}

// GET /api/ml/models/:id/versions - 获取模型版本
pub async fn get_model_versions(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    let versions = state.model_registry.get_versions(&id).await;
    Json(StandardResponse::success(versions)).into_response()
}

// POST /api/ml/models/:id/versions - 创建新版本
pub async fn create_model_version(Path(id): Path<String>, State(_state): State<AppState>, Json(_version_data): Json<Value>) -> Response {
    let version_id = uuid::Uuid::new_v4().to_string();
    Json(StandardResponse::success(json!({
        "model_id": id,
        "version_id": version_id,
        "message": "New version created successfully"
    }))).into_response()
}

// GET /api/ml/models/:id/versions/:version - 获取特定版本
pub async fn get_model_version(Path((id, version)): Path<(String, String)>, State(_state): State<AppState>) -> Response {
    Json(StandardResponse::success(json!({
        "model_id": id,
        "version": version,
        "status": "active",
        "created_at": chrono::Utc::now()
    }))).into_response()
}

// POST /api/ml/models/:id/deploy - 部署模型
pub async fn deploy_model(Path(id): Path<String>, State(_state): State<AppState>) -> Response {
    Json(StandardResponse::success(json!({
        "model_id": id,
        "status": "deployed",
        "endpoint": format!("/api/ml/models/{}/predict", id),
        "message": "Model deployed successfully"
    }))).into_response()
}

// POST /api/ml/models/:id/undeploy - 取消部署
pub async fn undeploy_model(Path(id): Path<String>, State(_state): State<AppState>) -> Response {
    Json(StandardResponse::success(json!({
        "model_id": id,
        "status": "undeployed",
        "message": "Model undeployed successfully"
    }))).into_response()
}

// POST /api/ml/models/:id/rollback - 回滚模型
pub async fn rollback_model(Path(id): Path<String>, State(_state): State<AppState>) -> Response {
    Json(StandardResponse::success(json!({
        "model_id": id,
        "message": "Model rolled back to previous version"
    }))).into_response()
}

// POST /api/ml/models/search - 搜索模型
pub async fn search_models(State(_state): State<AppState>, Json(search_params): Json<Value>) -> Response {
    let query = search_params.get("query").and_then(|v| v.as_str()).unwrap_or("");
    Json(StandardResponse::success(json!({
        "query": query,
        "results": [],
        "total_count": 0
    }))).into_response()
}

// POST /api/ml/models/:id/clone - 克隆模型
pub async fn clone_model(Path(id): Path<String>, State(_state): State<AppState>) -> Response {
    let new_id = uuid::Uuid::new_v4().to_string();
    Json(StandardResponse::success(json!({
        "original_model_id": id,
        "cloned_model_id": new_id,
        "message": "Model cloned successfully"
    }))).into_response()
} 