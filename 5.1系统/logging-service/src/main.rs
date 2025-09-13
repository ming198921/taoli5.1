use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put, delete},
    Json, Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use tokio::sync::broadcast;

mod handlers;
mod services;
mod models;

use models::{StandardResponse, LogEntry, LogStreamQuery, LogConfig, LogAnalysis};
use services::{LogCollector, LogAnalyzer, LogStorage};

#[derive(Clone)]
pub struct AppState {
    log_collector: Arc<LogCollector>,
    log_analyzer: Arc<LogAnalyzer>,
    log_storage: Arc<LogStorage>,
    log_broadcaster: Arc<broadcast::Sender<LogEntry>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("logging_service=debug")
        .init();

    info!("🚀 Starting Logging Service v1.0.0 (45 APIs)");

    // 初始化服务
    let log_storage = Arc::new(LogStorage::new().await?);
    let log_collector = Arc::new(LogCollector::new(log_storage.clone()).await?);
    let log_analyzer = Arc::new(LogAnalyzer::new(log_storage.clone()).await?);
    let (log_broadcaster, _) = broadcast::channel(10000);

    let app_state = AppState {
        log_collector,
        log_analyzer,
        log_storage,
        log_broadcaster: Arc::new(log_broadcaster),
    };

    // 构建路由 - 45个API端点
    let app = Router::new()
        // 健康检查
        .route("/health", get(health_check))
        
        // === 实时日志流API (15个) ===
        .route("/api/logs/stream/realtime", get(handlers::stream::get_realtime_logs))
        .route("/api/logs/stream/by-service/:service", get(handlers::stream::get_service_logs))
        .route("/api/logs/stream/by-level/:level", get(handlers::stream::get_level_logs))
        .route("/api/logs/stream/by-module/:module", get(handlers::stream::get_module_logs))
        .route("/api/logs/stream/search", post(handlers::stream::search_logs))
        .route("/api/logs/stream/tail", get(handlers::stream::tail_logs))
        .route("/api/logs/stream/follow", get(handlers::stream::follow_logs))
        .route("/api/logs/stream/buffer", get(handlers::stream::get_buffer_logs))
        .route("/api/logs/stream/history", get(handlers::stream::get_history_logs))
        .route("/api/logs/stream/export", post(handlers::stream::export_logs))
        .route("/ws/logs/realtime", get(handlers::stream::websocket_logs))
        .route("/ws/logs/filtered", get(handlers::stream::websocket_filtered_logs))
        .route("/api/logs/stream/stats", get(handlers::stream::get_stream_stats))
        .route("/api/logs/stream/pause", post(handlers::stream::pause_stream))
        .route("/api/logs/stream/resume", post(handlers::stream::resume_stream))
        
        // === 日志配置API (18个) ===
        .route("/api/logs/config/levels", get(handlers::config::get_log_levels))
        .route("/api/logs/config/levels", put(handlers::config::set_log_levels))
        .route("/api/logs/config/levels/:service", get(handlers::config::get_service_log_level))
        .route("/api/logs/config/levels/:service", put(handlers::config::set_service_log_level))
        .route("/api/logs/config/filters", get(handlers::config::get_log_filters))
        .route("/api/logs/config/filters", post(handlers::config::add_log_filter))
        .route("/api/logs/config/filters/:id", delete(handlers::config::delete_log_filter))
        .route("/api/logs/config/retention", get(handlers::config::get_retention_policy))
        .route("/api/logs/config/retention", put(handlers::config::set_retention_policy))
        .route("/api/logs/config/rotation", get(handlers::config::get_rotation_config))
        .route("/api/logs/config/rotation", put(handlers::config::set_rotation_config))
        .route("/api/logs/config/storage", get(handlers::config::get_storage_config))
        .route("/api/logs/config/storage", put(handlers::config::set_storage_config))
        .route("/api/logs/config/format", get(handlers::config::get_log_format))
        .route("/api/logs/config/format", put(handlers::config::set_log_format))
        .route("/api/logs/config/sampling", get(handlers::config::get_sampling_config))
        .route("/api/logs/config/sampling", put(handlers::config::set_sampling_config))
        .route("/api/logs/config/export", post(handlers::config::export_config))
        
        // === 日志分析API (12个) ===
        .route("/api/logs/analysis/stats", get(handlers::analysis::get_log_stats))
        .route("/api/logs/analysis/trends", get(handlers::analysis::get_log_trends))
        .route("/api/logs/analysis/anomaly", post(handlers::analysis::detect_anomalies))
        .route("/api/logs/analysis/patterns", post(handlers::analysis::find_patterns))
        .route("/api/logs/analysis/errors", get(handlers::analysis::analyze_errors))
        .route("/api/logs/analysis/performance", get(handlers::analysis::analyze_performance))
        .route("/api/logs/analysis/frequency", get(handlers::analysis::get_frequency_analysis))
        .route("/api/logs/analysis/correlation", post(handlers::analysis::correlate_logs))
        .route("/api/logs/analysis/summary", get(handlers::analysis::get_log_summary))
        .route("/api/logs/analysis/alerts", get(handlers::analysis::get_alerts))
        .route("/api/logs/analysis/alerts", post(handlers::analysis::create_alert))
        .route("/api/logs/analysis/report", post(handlers::analysis::generate_report))
        
        // 中间件
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], 4001));
    info!("📝 Logging Service listening on http://{}", addr);
    info!("✅ All 45 APIs initialized successfully");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(StandardResponse::success(serde_json::json!({
        "status": "healthy",
        "service": "logging-service",
        "version": "1.0.0",
        "apis_count": 45,
        "timestamp": chrono::Utc::now().timestamp(),
    })))
}