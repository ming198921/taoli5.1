use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Json, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

mod handlers;
mod services;
mod models;

use models::{StandardResponse, OrderStatus, Position, FundStatus, RiskMetrics};
use services::{OrderMonitor, PositionTracker, FundManager, RiskController};

#[derive(Clone)]
pub struct AppState {
    order_monitor: Arc<OrderMonitor>,
    position_tracker: Arc<PositionTracker>,
    fund_manager: Arc<FundManager>,
    risk_controller: Arc<RiskController>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("trading_service=debug")
        .init();

    info!("🚀 Starting Trading Service v1.0.0 (56 APIs)");

    let order_monitor = Arc::new(OrderMonitor::new().await?);
    let position_tracker = Arc::new(PositionTracker::new().await?);
    let fund_manager = Arc::new(FundManager::new().await?);
    let risk_controller = Arc::new(RiskController::new().await?);

    let app_state = AppState {
        order_monitor,
        position_tracker,
        fund_manager,
        risk_controller,
    };

    // 构建路由 - 56个API端点 (新增15个管理API)
    let app = Router::new()
        .route("/health", get(health_check))
        
        // === 订单监控API (15个) ===
        .route("/api/orders/active", get(handlers::orders::get_active_orders))
        .route("/api/orders/history", get(handlers::orders::get_order_history))
        .route("/api/orders/:id", get(handlers::orders::get_order_details))
        .route("/api/orders/:id/status", get(handlers::orders::get_order_status))
        .route("/api/orders/:id/fills", get(handlers::orders::get_order_fills))
        .route("/api/orders/:id/cancel", post(handlers::orders::cancel_order))
        .route("/api/orders/:id/modify", put(handlers::orders::modify_order))
        .route("/api/orders/batch/cancel", post(handlers::orders::batch_cancel_orders))
        .route("/api/orders/stats", get(handlers::orders::get_order_stats))
        .route("/api/orders/execution-quality", get(handlers::orders::get_execution_quality))
        .route("/api/orders/latency", get(handlers::orders::get_order_latency))
        .route("/api/orders/slippage", get(handlers::orders::analyze_slippage))
        .route("/api/orders/rejected", get(handlers::orders::get_rejected_orders))
        .route("/api/orders/alerts", get(handlers::orders::get_order_alerts))
        .route("/api/orders/performance", get(handlers::orders::get_execution_performance))

        // === 仓位监控API (12个) ===
        .route("/api/positions/current", get(handlers::positions::get_current_positions))
        .route("/api/positions/realtime", get(handlers::positions::get_realtime_positions))
        .route("/api/positions/:symbol", get(handlers::positions::get_position_by_symbol))
        .route("/api/positions/:symbol/pnl", get(handlers::positions::get_position_pnl))
        .route("/api/positions/:symbol/history", get(handlers::positions::get_position_history))
        .route("/api/positions/total-pnl", get(handlers::positions::get_total_pnl))
        .route("/api/positions/exposure", get(handlers::positions::get_exposure_analysis))
        .route("/api/positions/concentration", get(handlers::positions::get_concentration_risk))
        .route("/api/positions/correlation", get(handlers::positions::get_position_correlation))
        .route("/api/positions/stress-test", post(handlers::positions::run_stress_test))
        .route("/api/positions/var", get(handlers::positions::calculate_var))
        .route("/api/positions/attribution", get(handlers::positions::get_pnl_attribution))

        // === 资金管理API (8个) ===
        .route("/api/funds/balance", get(handlers::funds::get_fund_balance))
        .route("/api/funds/available", get(handlers::funds::get_available_funds))
        .route("/api/funds/margin", get(handlers::funds::get_margin_status))
        .route("/api/funds/utilization", get(handlers::funds::get_fund_utilization))
        .route("/api/funds/history", get(handlers::funds::get_fund_history))
        .route("/api/funds/flows", get(handlers::funds::get_fund_flows))
        .route("/api/funds/allocation", get(handlers::funds::get_fund_allocation))
        .route("/api/funds/requirements", get(handlers::funds::get_margin_requirements))

        // === 风险控制API (6个) ===
        .route("/api/risk/metrics", get(handlers::risk::get_risk_metrics))
        .route("/api/risk/limits", get(handlers::risk::get_risk_limits))
        .route("/api/risk/limits", put(handlers::risk::update_risk_limits))
        .route("/api/risk/violations", get(handlers::risk::get_risk_violations))
        .route("/api/risk/alerts", get(handlers::risk::get_risk_alerts))
        .route("/api/risk/report", post(handlers::risk::generate_risk_report))

        // === 交易费用管理API (8个) ===
        .route("/api/fees/exchanges", get(handlers::fees::get_all_exchange_fees))
        .route("/api/fees/exchanges/:exchange", get(handlers::fees::get_exchange_fees))
        .route("/api/fees/exchanges/:exchange", put(handlers::fees::update_exchange_fees))
        .route("/api/fees/symbols/:symbol", get(handlers::fees::get_symbol_fees))
        .route("/api/fees/calculate", post(handlers::fees::calculate_trading_fees))
        .route("/api/fees/comparison", get(handlers::fees::compare_fees))
        .route("/api/fees/refresh", post(handlers::fees::refresh_all_fees))
        .route("/api/fees/arbitrage-analysis", get(handlers::fees::analyze_arbitrage_fees))

        // === 交易所API管理 (8个) ===
        .route("/api/exchange-api/credentials", get(handlers::exchange_api::list_configured_exchanges))
        .route("/api/exchange-api/:exchange/credentials", post(handlers::exchange_api::add_credentials))
        .route("/api/exchange-api/:exchange/credentials", delete(handlers::exchange_api::remove_credentials))
        .route("/api/exchange-api/:exchange/status", get(handlers::exchange_api::get_api_status))
        .route("/api/exchange-api/:exchange/test", post(handlers::exchange_api::test_api_connection))
        .route("/api/exchange-api/:exchange/account", get(handlers::exchange_api::get_account_info))
        .route("/api/exchange-api/:exchange/trading-fees", get(handlers::exchange_api::get_real_trading_fees))
        .route("/api/exchange-api/:exchange/order", post(handlers::exchange_api::place_real_order))
        .route("/api/exchange-api/ultra-fast-prices", get(handlers::exchange_api::get_ultra_fast_prices))

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 
        std::env::var("TRADING_SERVICE_PORT")
            .unwrap_or_else(|_| "4005".to_string())
            .parse::<u16>()
            .unwrap_or(4005)
    ));
    info!("💹 Trading Service listening on http://{}", addr);
    info!("✅ All 57 APIs initialized successfully (新增真实下单API)");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(StandardResponse::success(serde_json::json!({
        "status": "healthy",
        "service": "trading-service",
        "version": "1.0.0",
        "apis_count": 57,
        "modules": ["orders", "positions", "funds", "risk", "fees", "exchange_api"],
        "timestamp": chrono::Utc::now().timestamp(),
    })))
}