use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;
use crate::{AppState, models::StandardResponse};

pub async fn get_portfolio(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({"portfolio": {"balance": 10000, "positions": []}})))
}

pub async fn get_balance(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({"balance": 10000})))
}

pub async fn get_positions(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({"positions": []})))
}