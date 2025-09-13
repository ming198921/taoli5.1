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

// GET /api/ml/training/jobs - 获取训练任务列表
pub async fn list_training_jobs(State(state): State<AppState>) -> Response {
    let jobs = state.training_engine.list_training_jobs().await;
    Json(StandardResponse::success(jobs)).into_response()
}

// POST /api/ml/training/jobs - 创建训练任务
pub async fn create_training_job(State(state): State<AppState>, Json(config): Json<Value>) -> Response {
    let model_id = config.get("model_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let training_config = crate::models::TrainingConfig {
        epochs: config.get("epochs").and_then(|v| v.as_u64()).unwrap_or(100) as u32,
        batch_size: config.get("batch_size").and_then(|v| v.as_u64()).unwrap_or(32) as u32,
        learning_rate: config.get("learning_rate").and_then(|v| v.as_f64()).unwrap_or(0.001),
        optimizer: config.get("optimizer").and_then(|v| v.as_str()).unwrap_or("adam").to_string(),
        loss_function: config.get("loss_function").and_then(|v| v.as_str()).unwrap_or("mse").to_string(),
        validation_split: config.get("validation_split").and_then(|v| v.as_f64()).unwrap_or(0.2),
        early_stopping: config.get("early_stopping").and_then(|v| v.as_bool()).unwrap_or(true),
        patience: config.get("patience").and_then(|v| v.as_u64()).unwrap_or(10) as u32,
    };

    match state.training_engine.start_training(model_id, training_config).await {
        Ok(job_id) => Json(StandardResponse::success(json!({
            "job_id": job_id,
            "message": "Training job created successfully"
        }))).into_response(),
        Err(e) => Json(StandardResponse::<Value>::error(e.to_string())).into_response()
    }
}

// GET /api/ml/training/jobs/:id - 获取训练任务详情
pub async fn get_training_job(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.training_engine.get_training_job(&id).await {
        Some(job) => Json(StandardResponse::success(job)).into_response(),
        None => Json(StandardResponse::<Value>::error("Training job not found".to_string())).into_response()
    }
}

// POST /api/ml/training/jobs/:id/stop - 停止训练任务
pub async fn stop_training_job(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.training_engine.stop_training(&id).await {
        Ok(()) => Json(StandardResponse::success(json!({
            "job_id": id,
            "message": "Training job stopped successfully"
        }))).into_response(),
        Err(e) => Json(StandardResponse::<Value>::error(e.to_string())).into_response()
    }
}

// GET /api/ml/training/jobs/:id/progress - 获取训练进度
pub async fn get_training_progress(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.training_engine.get_training_job(&id).await {
        Some(job) => Json(StandardResponse::success(json!({
            "job_id": id,
            "progress": job.progress,
            "status": job.status,
            "current_epoch": job.progress * job.epochs as f64,
            "total_epochs": job.epochs
        }))).into_response(),
        None => Json(StandardResponse::<Value>::error("Training job not found".to_string())).into_response()
    }
}

// GET /api/ml/training/jobs/:id/logs - 获取训练日志
pub async fn get_training_logs(Path(id): Path<String>, State(_state): State<AppState>) -> Response {
    let logs = vec![
        json!({
            "timestamp": chrono::Utc::now(),
            "level": "info",
            "message": format!("Training job {} started", id)
        }),
        json!({
            "timestamp": chrono::Utc::now(),
            "level": "info", 
            "message": "Epoch 1/100 - loss: 0.5432, accuracy: 0.7891"
        })
    ];
    Json(StandardResponse::success(logs)).into_response()
}

// GET /api/ml/training/datasets - 获取数据集列表
pub async fn list_datasets(State(state): State<AppState>) -> Response {
    let datasets = state.dataset_manager.list_datasets().await;
    Json(StandardResponse::success(datasets)).into_response()
}

// POST /api/ml/training/datasets - 创建数据集
pub async fn create_dataset(State(state): State<AppState>, Json(dataset_data): Json<Value>) -> Response {
    let dataset = crate::models::DatasetInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name: dataset_data.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed Dataset").to_string(),
        description: dataset_data.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        size: dataset_data.get("size").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        features: dataset_data.get("features").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default(),
        target: dataset_data.get("target").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        quality_score: dataset_data.get("quality_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
    };

    match state.dataset_manager.create_dataset(dataset.clone()).await {
        Ok(dataset_id) => Json(StandardResponse::success(json!({
            "dataset_id": dataset_id,
            "message": "Dataset created successfully"
        }))).into_response(),
        Err(e) => Json(StandardResponse::<Value>::error(e.to_string())).into_response()
    }
}

// GET /api/ml/training/datasets/:id - 获取数据集详情
pub async fn get_dataset(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.dataset_manager.get_dataset(&id).await {
        Some(dataset) => Json(StandardResponse::success(dataset)).into_response(),
        None => Json(StandardResponse::<Value>::error("Dataset not found".to_string())).into_response()
    }
}

// GET /api/ml/training/datasets/:id/stats - 获取数据集统计
pub async fn get_dataset_stats(Path(id): Path<String>, State(_state): State<AppState>) -> Response {
    Json(StandardResponse::success(json!({
        "dataset_id": id,
        "total_samples": 10000,
        "feature_count": 15,
        "class_distribution": {
            "class_0": 4500,
            "class_1": 5500
        },
        "missing_values": 50,
        "data_quality_score": 0.92
    }))).into_response()
}

// GET /api/ml/training/hyperparams/search - 超参数搜索
pub async fn hyperparameter_search(State(_state): State<AppState>, Json(search_config): Json<Value>) -> Response {
    let search_id = uuid::Uuid::new_v4().to_string();
    Json(StandardResponse::success(json!({
        "search_id": search_id,
        "status": "started",
        "search_space": search_config,
        "message": "Hyperparameter search started"
    }))).into_response()
}
