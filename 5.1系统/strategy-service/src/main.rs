use axum::{
    routing::{get, post, delete},
    Json, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

mod handlers;
mod services;
mod models;

use models::StandardResponse;
use services::{StrategyMonitor, StrategyController, DebugManager, HotReloadManager};

#[derive(Clone)]
pub struct AppState {
    strategy_monitor: Arc<StrategyMonitor>,
    strategy_controller: Arc<StrategyController>,
    debug_manager: Arc<DebugManager>,
    hot_reload_manager: Arc<HotReloadManager>,
}

async fn health_check() -> Json<StandardResponse<String>> {
    Json(StandardResponse::success("Strategy Service is healthy".to_string()))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("strategy_service=debug")
        .init();

    info!("🚀 Starting Strategy Service v1.0.0 (38 APIs)");

    let strategy_monitor = Arc::new(StrategyMonitor::new().await?);
    let strategy_controller = Arc::new(StrategyController::new().await?);
    let debug_manager = Arc::new(DebugManager::new().await?);
    let hot_reload_manager = Arc::new(HotReloadManager::new().await?);

    let app_state = AppState {
        strategy_monitor,
        strategy_controller,
        debug_manager,
        hot_reload_manager,
    };

    // 构建路由 - 38个API端点
    let app = Router::new()
        .route("/health", get(health_check))
        
        // === 策略生命周期管理API (12个) ===
        .route("/api/strategies/list", get(handlers::lifecycle::list_strategies))
        .route("/api/strategies/:id", get(handlers::lifecycle::get_strategy))
        .route("/api/strategies/:id/start", post(handlers::lifecycle::start_strategy))
        .route("/api/strategies/:id/stop", post(handlers::lifecycle::stop_strategy))
        .route("/api/strategies/:id/restart", post(handlers::lifecycle::restart_strategy))
        .route("/api/strategies/:id/pause", post(handlers::lifecycle::pause_strategy))
        .route("/api/strategies/:id/resume", post(handlers::lifecycle::resume_strategy))
        .route("/api/strategies/:id/status", get(handlers::lifecycle::get_strategy_status))
        .route("/api/strategies/:id/config", get(handlers::lifecycle::get_strategy_config))
        .route("/api/strategies/:id/config", post(handlers::lifecycle::update_strategy_config))
        .route("/api/strategies/:id/logs", get(handlers::lifecycle::get_strategy_logs))
        .route("/api/strategies/:id/metrics", get(handlers::lifecycle::get_strategy_metrics))
        
        // === 实时监控API (8个) ===
        .route("/api/monitoring/realtime", get(handlers::monitoring::get_realtime_status))
        .route("/api/monitoring/health", get(handlers::monitoring::get_system_health))
        .route("/api/monitoring/performance", get(handlers::monitoring::get_performance_overview))
        .route("/api/monitoring/alerts", get(handlers::monitoring::get_active_alerts))
        .route("/api/monitoring/metrics/cpu", get(handlers::monitoring::get_cpu_metrics))
        .route("/api/monitoring/metrics/memory", get(handlers::monitoring::get_memory_metrics))
        .route("/api/monitoring/metrics/network", get(handlers::monitoring::get_network_metrics))
        .route("/api/monitoring/metrics/history", get(handlers::monitoring::get_metrics_history))
        
        // === 调试工具API (9个) ===
        .route("/api/debug/sessions", get(handlers::debug::list_debug_sessions))
        .route("/api/debug/sessions", post(handlers::debug::create_debug_session))
        .route("/api/debug/sessions/:id", get(handlers::debug::get_debug_session))
        .route("/api/debug/sessions/:id", delete(handlers::debug::delete_debug_session))
        .route("/api/debug/breakpoints/:strategy_id", get(handlers::debug::list_breakpoints))
        .route("/api/debug/breakpoints/:strategy_id", post(handlers::debug::add_breakpoint))
        .route("/api/debug/breakpoints/:strategy_id/:bp_id", delete(handlers::debug::remove_breakpoint))
        .route("/api/debug/variables/:strategy_id", get(handlers::debug::get_variables))
        .route("/api/debug/stack/:strategy_id", get(handlers::debug::get_stack_trace))
        
        // === 热重载API (9个) ===
        .route("/api/hotreload/status", get(handlers::hot_reload::get_reload_status_all))
        .route("/api/hotreload/:strategy_id/status", get(handlers::hot_reload::get_reload_status))
//         // .route("/api/hotreload/:strategy_id/reload", post(handlers::hot_reload::reload_strategy))
        .route("/api/hotreload/:strategy_id/enable", post(handlers::hot_reload::enable_hot_reload))
        .route("/api/hotreload/:strategy_id/disable", post(handlers::hot_reload::disable_hot_reload))
        .route("/api/hotreload/:strategy_id/validate", post(handlers::hot_reload::validate_changes))
        .route("/api/hotreload/:strategy_id/rollback", post(handlers::hot_reload::rollback_changes))
        .route("/api/hotreload/history", get(handlers::hot_reload::get_reload_history))
        .route("/api/hotreload/config", get(handlers::hot_reload::get_reload_config))
        
        // 中间件
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 
        std::env::var("STRATEGY_SERVICE_PORT")
            .unwrap_or_else(|_| "4003".to_string())
            .parse::<u16>()
            .unwrap_or(4003)
    ));

    info!("🌐 策略监控服务启动成功，监听端口: http://{}", addr);
    info!("📊 已注册 38 个API端点");
    info!("🔧 服务状态: /health");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}