use axum::{
    extract::{State, Query},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::{AppState, models::StandardResponse};

// GET /api/risk/metrics - 获取风险指标
pub async fn get_risk_metrics(
    State(state): State<AppState>,
) -> Response {
    let metrics = state.risk_controller.get_risk_metrics("main_account").await
        .unwrap_or_else(|| {
            // 返回默认风险指标
            crate::models::RiskMetrics {
                account_id: "main_account".to_string(),
                total_exposure: 50000.0,
                max_drawdown: 0.05,
                current_drawdown: 0.02,
                var_95: 2500.0,
                sharpe_ratio: 1.5,
                sortino_ratio: 2.0,
                calmar_ratio: 1.2,
                win_rate: 0.65,
                profit_factor: 1.8,
                max_consecutive_losses: 3,
                current_consecutive_losses: 0,
                risk_score: 75.0,
                leverage_utilization: 0.3,
                concentration_risk: HashMap::new(),
            }
        });
    Json(StandardResponse::success(metrics)).into_response()
}

// GET /api/risk/limits - 获取风险限制
pub async fn get_risk_limits(
    State(state): State<AppState>,
) -> Response {
    let limits = state.risk_controller.get_risk_limits().await;
    Json(StandardResponse::success(limits)).into_response()
}

// PUT /api/risk/limits - 更新风险限制
pub async fn update_risk_limits(
    State(state): State<AppState>,
    Json(new_limits): Json<HashMap<String, f64>>,
) -> Response {
    match state.risk_controller.update_risk_limits(new_limits.clone()).await {
        Ok(()) => Json(StandardResponse::success(json!({
            "message": "Risk limits updated successfully",
            "updated_limits": new_limits
        }))).into_response(),
        Err(error) => Json(StandardResponse::<Value>::error(error.to_string())).into_response()
    }
}

// GET /api/risk/violations - 获取风险违规
pub async fn get_risk_violations(
    State(state): State<AppState>,
) -> Response {
    let violations = state.risk_controller.check_risk_violations().await;
    let violation_details: Vec<_> = violations.iter().map(|v| json!({
        "type": v,
        "severity": "high",
        "timestamp": chrono::Utc::now(),
        "recommended_action": "Reduce position size"
    })).collect();
    
    Json(StandardResponse::success(json!({
        "violations": violation_details,
        "total_violations": violations.len()
    }))).into_response()
}

// GET /api/risk/alerts - 获取风险告警
pub async fn get_risk_alerts(
    Query(_params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> Response {
    let alerts = vec![
        json!({
            "id": "risk_alert_001",
            "type": "high_leverage",
            "message": "Leverage exceeds recommended limit",
            "severity": "warning",
            "timestamp": chrono::Utc::now(),
            "acknowledged": false
        }),
        json!({
            "id": "risk_alert_002", 
            "type": "concentration_risk",
            "message": "High concentration in BTC positions",
            "severity": "medium",
            "timestamp": chrono::Utc::now(),
            "acknowledged": false
        })
    ];
    Json(StandardResponse::success(alerts)).into_response()
}

// POST /api/risk/report - 生成风险报告
pub async fn generate_risk_report(
    State(state): State<AppState>,
    Json(_params): Json<Value>,
) -> Response {
    let metrics = state.risk_controller.get_risk_metrics("main_account").await;
    let limits = state.risk_controller.get_risk_limits().await;
    let violations = state.risk_controller.check_risk_violations().await;
    
    let report = json!({
        "report_id": uuid::Uuid::new_v4().to_string(),
        "generated_at": chrono::Utc::now(),
        "risk_metrics": metrics,
        "risk_limits": limits,
        "violations": violations,
        "overall_risk_score": 75.0,
        "recommendations": [
            "Consider reducing leverage on high-risk positions",
            "Diversify portfolio to reduce concentration risk",
            "Review stop-loss levels for better risk management"
        ]
    });
    
    Json(StandardResponse::success(report)).into_response()
}