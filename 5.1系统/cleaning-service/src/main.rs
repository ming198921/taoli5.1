#![feature(portable_simd)]

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

use models::{StandardResponse, CleaningRule, CleaningConfig, PerformanceMetrics};
use services::{CleaningEngine, RuleValidator, PerformanceMonitor};

#[derive(Clone)]
pub struct AppState {
    cleaning_engine: Arc<CleaningEngine>,
    rule_validator: Arc<RuleValidator>,
    performance_monitor: Arc<PerformanceMonitor>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("cleaning_service=debug")
        .init();

    info!("🚀 Starting Cleaning Service v1.0.0 (52 APIs)");

    let cleaning_engine = Arc::new(CleaningEngine::new().await?);
    let rule_validator = Arc::new(RuleValidator::new().await?);
    let performance_monitor = Arc::new(PerformanceMonitor::new().await?);

    let app_state = AppState {
        cleaning_engine,
        rule_validator,
        performance_monitor,
    };

    // 构建路由 - 52个API端点
    let app = Router::new()
        .route("/health", get(health_check))
        
        // === 清洗规则管理API (20个) ===
        .route("/api/cleaning/rules/list", get(handlers::rules::list_rules))
        .route("/api/cleaning/rules/create", post(handlers::rules::create_rule))
        .route("/api/cleaning/rules/:id", get(handlers::rules::get_rule))
        .route("/api/cleaning/rules/:id", put(handlers::rules::update_rule))
        .route("/api/cleaning/rules/:id", delete(handlers::rules::delete_rule))
        .route("/api/cleaning/rules/:id/enable", post(handlers::rules::enable_rule))
        .route("/api/cleaning/rules/:id/disable", post(handlers::rules::disable_rule))
        .route("/api/cleaning/rules/test", post(handlers::rules::test_rule))
        .route("/api/cleaning/rules/validate", post(handlers::rules::validate_rule))
        .route("/api/cleaning/rules/export", get(handlers::rules::export_rules))
        .route("/api/cleaning/rules/import", post(handlers::rules::import_rules))
        .route("/api/cleaning/rules/templates", get(handlers::rules::list_templates))
        .route("/api/cleaning/rules/templates/:template", post(handlers::rules::create_from_template))
        .route("/api/cleaning/rules/search", post(handlers::rules::search_rules))
        .route("/api/cleaning/rules/batch/enable", post(handlers::rules::batch_enable))
        .route("/api/cleaning/rules/batch/disable", post(handlers::rules::batch_disable))
        .route("/api/cleaning/rules/batch/delete", post(handlers::rules::batch_delete))
        .route("/api/cleaning/rules/history/:id", get(handlers::rules::get_rule_history))
        .route("/api/cleaning/rules/stats", get(handlers::rules::get_rules_stats))
        .route("/api/cleaning/rules/dependencies/:id", get(handlers::rules::get_dependencies))

        // === 交易所配置API (16个) ===
        .route("/api/cleaning/exchanges", get(handlers::exchanges::list_exchanges))
        .route("/api/cleaning/exchanges/:id/config", get(handlers::exchanges::get_config))
        .route("/api/cleaning/exchanges/:id/config", put(handlers::exchanges::update_config))
        .route("/api/cleaning/exchanges/:id/rules", get(handlers::exchanges::get_exchange_rules))
        .route("/api/cleaning/exchanges/:id/rules", post(handlers::exchanges::add_exchange_rule))
        .route("/api/cleaning/exchanges/:id/symbols", get(handlers::exchanges::get_symbols_config))
        .route("/api/cleaning/exchanges/:id/symbols", put(handlers::exchanges::update_symbols_config))
        .route("/api/cleaning/exchanges/:id/timeframes", get(handlers::exchanges::get_timeframes))
        .route("/api/cleaning/exchanges/:id/timeframes", put(handlers::exchanges::update_timeframes))
        .route("/api/cleaning/exchanges/:id/filters", get(handlers::exchanges::get_data_filters))
        .route("/api/cleaning/exchanges/:id/filters", put(handlers::exchanges::update_data_filters))
        .route("/api/cleaning/exchanges/:id/validation", get(handlers::exchanges::get_validation_rules))
        .route("/api/cleaning/exchanges/:id/validation", put(handlers::exchanges::update_validation_rules))
        .route("/api/cleaning/exchanges/:id/test", post(handlers::exchanges::test_exchange_config))
        .route("/api/cleaning/exchanges/:id/reset", post(handlers::exchanges::reset_exchange_config))
        .route("/api/cleaning/exchanges/:id/clone", post(handlers::exchanges::clone_exchange_config))

        // === SIMD性能优化API (16个) ===
        .route("/api/cleaning/simd/status", get(handlers::simd::get_simd_status))
        .route("/api/cleaning/simd/capabilities", get(handlers::simd::get_capabilities))
        .route("/api/cleaning/simd/config", get(handlers::simd::get_simd_config))
        .route("/api/cleaning/simd/config", put(handlers::simd::update_simd_config))
        .route("/api/cleaning/simd/benchmark", post(handlers::simd::run_benchmark))
        .route("/api/cleaning/simd/optimize", post(handlers::simd::optimize_rules))
        .route("/api/cleaning/simd/performance", get(handlers::simd::get_performance_metrics))
        .route("/api/cleaning/simd/vectorize", post(handlers::simd::vectorize_operations))
        .route("/api/cleaning/simd/parallel", get(handlers::simd::get_parallel_config))
        .route("/api/cleaning/simd/parallel", put(handlers::simd::update_parallel_config))
        .route("/api/cleaning/simd/threads", get(handlers::simd::get_thread_config))
        .route("/api/cleaning/simd/threads", put(handlers::simd::update_thread_config))
        .route("/api/cleaning/simd/memory", get(handlers::simd::get_memory_usage))
        .route("/api/cleaning/simd/cache", get(handlers::simd::get_cache_stats))
        .route("/api/cleaning/simd/profile", post(handlers::simd::profile_performance))
        .route("/api/cleaning/simd/report", get(handlers::simd::generate_performance_report))

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4002));
    info!("🧹 Cleaning Service listening on http://{}", addr);
    info!("✅ All 52 APIs initialized successfully");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(StandardResponse::success(serde_json::json!({
        "status": "healthy",
        "service": "cleaning-service",
        "version": "1.0.0",
        "apis_count": 52,
        "simd_enabled": true,
        "timestamp": chrono::Utc::now().timestamp(),
    })))
}