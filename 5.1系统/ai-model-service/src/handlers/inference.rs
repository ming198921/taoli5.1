#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use axum::response::IntoResponse;
use axum::{extract::{Path, State}, response::Response, Json};
use serde_json::{json, Value};
use crate::{AppState, models::StandardResponse};

pub async fn predict(Path(id): Path<String>, State(state): State<AppState>, Json(input_data): Json<Value>) -> Response {
    match state.inference_engine.predict(id.clone(), input_data.clone()).await {
        Ok(result) => Json(StandardResponse::success(result)).into_response(),
        Err(e) => Json(StandardResponse::<Value>::error(e.to_string())).into_response()
    }
}

pub async fn batch_predict(State(_state): State<AppState>, Json(_batch_data): Json<Value>) -> Response {
    Json(StandardResponse::success(json!({"predictions": [], "batch_id": uuid::Uuid::new_v4().to_string()}))).into_response()
}

pub async fn get_prediction_history(State(_state): State<AppState>) -> Response {
    Json(StandardResponse::success(vec![json!({"id": "pred_001", "timestamp": chrono::Utc::now()})])).into_response()
}

pub async fn explain_prediction(Path(id): Path<String>, State(state): State<AppState>) -> Response {
    match state.shap_explainer.explain_prediction(id.clone()).await {
        Ok(explanation) => Json(StandardResponse::success(explanation)).into_response(),
        Err(e) => Json(StandardResponse::<Value>::error(e.to_string())).into_response()
    }
}
