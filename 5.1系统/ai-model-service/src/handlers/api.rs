#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use axum::response::IntoResponse;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use crate::{AppState, models::StandardResponse};

pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({"status": "healthy"})))
}
