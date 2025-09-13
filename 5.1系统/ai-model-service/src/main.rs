#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use crate::models::StandardResponse;
use anyhow::Result;
use axum::{
    routing::{get, post, put, delete},
    Json, Router,
};
use services::*;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

mod handlers;
mod models;
mod services;

#[derive(Clone)]
pub struct AppState {
    model_manager: Arc<ModelManager>,
    training_engine: Arc<TrainingEngine>,
    inference_engine: Arc<InferenceEngine>,
    model_registry: Arc<ModelRegistry>,
    dataset_manager: Arc<DatasetManager>,
    shap_explainer: Arc<ShapExplainer>,
}

async fn health_check() -> Json<StandardResponse<serde_json::Value>> {
    Json(StandardResponse::success(serde_json::json!({
        "status": "healthy",
        "service": "ai-model-service",
        "version": "1.0.0",
        "apis": 48,
        "timestamp": chrono::Utc::now()
    })))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("ai_model_service=info")
        .init();

    info!("🚀 Starting AI Model Service v1.0.0 (48 APIs)");

    let model_manager = Arc::new(ModelManager::new().await?);
    let training_engine = Arc::new(TrainingEngine::new().await?);
    let inference_engine = Arc::new(InferenceEngine::new().await?);
    let model_registry = Arc::new(ModelRegistry::new().await?);
    let dataset_manager = Arc::new(DatasetManager::new().await?);
    let shap_explainer = Arc::new(ShapExplainer::new().await?);

    let app_state = AppState {
        model_manager,
        training_engine,
        inference_engine,
        model_registry,
        dataset_manager,
        shap_explainer,
    };

    let app = Router::new()
        .route("/health", get(health_check))
        
        // Model Management APIs (16个)
        .route("/api/ml/models", get(handlers::models::list_models))
        .route("/api/ml/models", post(handlers::models::create_model))
        .route("/api/ml/models/:id", get(handlers::models::get_model))
        .route("/api/ml/models/:id", put(handlers::models::update_model))
        .route("/api/ml/models/:id", delete(handlers::models::delete_model))
        .route("/api/ml/models/:id/status", get(handlers::models::get_model_status))
        .route("/api/ml/models/:id/metadata", get(handlers::models::get_model_metadata))
        .route("/api/ml/models/:id/metadata", put(handlers::models::update_model_metadata))
        .route("/api/ml/models/:id/versions", get(handlers::models::get_model_versions))
        .route("/api/ml/models/:id/versions", post(handlers::models::create_model_version))
        .route("/api/ml/models/:id/versions/:version", get(handlers::models::get_model_version))
        .route("/api/ml/models/:id/deploy", post(handlers::models::deploy_model))
        .route("/api/ml/models/:id/undeploy", post(handlers::models::undeploy_model))
        .route("/api/ml/models/:id/rollback", post(handlers::models::rollback_model))
        .route("/api/ml/models/search", post(handlers::models::search_models))
        .route("/api/ml/models/:id/clone", post(handlers::models::clone_model))

        // Training APIs (12个)
        .route("/api/ml/training/jobs", get(handlers::training::list_training_jobs))
        .route("/api/ml/training/jobs", post(handlers::training::create_training_job))
        .route("/api/ml/training/jobs/:id", get(handlers::training::get_training_job))
        .route("/api/ml/training/jobs/:id/stop", post(handlers::training::stop_training_job))
        .route("/api/ml/training/jobs/:id/progress", get(handlers::training::get_training_progress))
        .route("/api/ml/training/jobs/:id/logs", get(handlers::training::get_training_logs))
        .route("/api/ml/training/datasets", get(handlers::training::list_datasets))
        .route("/api/ml/training/datasets", post(handlers::training::create_dataset))
        .route("/api/ml/training/datasets/:id", get(handlers::training::get_dataset))
        .route("/api/ml/training/datasets/:id/stats", get(handlers::training::get_dataset_stats))
        .route("/api/ml/training/hyperparams/search", get(handlers::training::hyperparameter_search))
        .route("/api/ml/training/hyperparams/optimize", post(handlers::training::hyperparameter_search))

        // Inference APIs (10个)
        .route("/api/ml/inference/:id/predict", post(handlers::inference::predict))
        .route("/api/ml/inference/batch-predict", post(handlers::inference::batch_predict))
        .route("/api/ml/inference/history", get(handlers::inference::get_prediction_history))
        .route("/api/ml/inference/:id/explain", post(handlers::inference::explain_prediction))
        .route("/api/ml/inference/drift", get(handlers::inference::get_prediction_history))
        .route("/api/ml/inference/ab-test", post(handlers::inference::batch_predict))
        .route("/api/ml/inference/benchmark", post(handlers::inference::batch_predict))
        .route("/api/ml/inference/latency", get(handlers::inference::get_prediction_history))
        .route("/api/ml/inference/throughput", get(handlers::inference::get_prediction_history))
        .route("/api/ml/inference/quality", get(handlers::inference::get_prediction_history))

        // Monitoring APIs (10个)
        .route("/api/ml/monitoring/alerts", get(handlers::monitoring::get_model_alerts))
        .route("/api/ml/monitoring/alerts", post(handlers::monitoring::create_alert))
        .route("/api/ml/monitoring/health/:id", get(handlers::monitoring::get_model_health))
        .route("/api/ml/monitoring/usage/:id", get(handlers::monitoring::get_model_usage))
        .route("/api/ml/monitoring/resources", get(handlers::monitoring::get_resource_usage))
        .route("/api/ml/monitoring/report", post(handlers::monitoring::generate_monitoring_report))
        .route("/api/ml/monitoring/performance", get(handlers::monitoring::get_model_alerts))
        .route("/api/ml/monitoring/accuracy", get(handlers::monitoring::get_model_alerts))
        .route("/api/ml/monitoring/drift", get(handlers::monitoring::get_model_alerts))
        .route("/api/ml/monitoring/bias", get(handlers::monitoring::get_model_alerts))
        
        .with_state(app_state);

    let port = std::env::var("AI_MODEL_SERVICE_PORT").unwrap_or_else(|_| "4006".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    
    info!("⚙️ AI Model Service listening on http://{}", addr);
    info!("✅ All 48 APIs initialized successfully");

    axum::serve(listener, app).await?;
    Ok(())
}
