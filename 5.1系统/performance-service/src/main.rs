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

use models::{StandardResponse, SystemMetrics, OptimizationResult};
use services::{PerformanceAnalyzer, SystemOptimizer, ResourceMonitor, TuningEngine};

#[derive(Clone)]
pub struct AppState {
    performance_analyzer: Arc<PerformanceAnalyzer>,
    system_optimizer: Arc<SystemOptimizer>,
    resource_monitor: Arc<ResourceMonitor>,
    tuning_engine: Arc<TuningEngine>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("performance_service=debug")
        .init();

    info!("🚀 Starting Performance Service v1.0.0 (67 APIs)");

    let performance_analyzer = Arc::new(PerformanceAnalyzer::new().await?);
    let system_optimizer = Arc::new(SystemOptimizer::new().await?);
    let resource_monitor = Arc::new(ResourceMonitor::new().await?);
    let tuning_engine = Arc::new(TuningEngine::new().await?);

    let app_state = AppState {
        performance_analyzer,
        system_optimizer,
        resource_monitor,
        tuning_engine,
    };

    // 构建路由 - 67个API端点
    let app = Router::new()
        .route("/health", get(health_check))
        
        // === CPU优化API (18个) ===
        .route("/api/performance/cpu/usage", get(handlers::cpu::get_cpu_usage))
        .route("/api/performance/cpu/cores", get(handlers::cpu::get_cpu_cores))
        .route("/api/performance/cpu/frequency", get(handlers::cpu::get_cpu_frequency))
        .route("/api/performance/cpu/frequency", put(handlers::cpu::set_cpu_frequency))
        .route("/api/performance/cpu/governor", get(handlers::cpu::get_governor))
        .route("/api/performance/cpu/governor", put(handlers::cpu::set_governor))
        .route("/api/performance/cpu/affinity/:process", get(handlers::cpu::get_affinity))
        .route("/api/performance/cpu/affinity/:process", put(handlers::cpu::set_affinity))
        .route("/api/performance/cpu/cache", get(handlers::cpu::get_cache_stats))
        .route("/api/performance/cpu/cache/flush", post(handlers::cpu::flush_cache))
        .route("/api/performance/cpu/temperature", get(handlers::cpu::get_temperature))
        .route("/api/performance/cpu/throttling", get(handlers::cpu::get_throttling_status))
        .route("/api/performance/cpu/topology", get(handlers::cpu::get_topology))
        .route("/api/performance/cpu/processes", get(handlers::cpu::get_process_usage))
        .route("/api/performance/cpu/optimize", post(handlers::cpu::optimize_cpu))
        .route("/api/performance/cpu/benchmark", post(handlers::cpu::run_cpu_benchmark))
        .route("/api/performance/cpu/scheduler", get(handlers::cpu::get_scheduler_info))
        .route("/api/performance/cpu/scheduler", put(handlers::cpu::tune_scheduler))

        // === 内存优化API (16个) ===
        .route("/api/performance/memory/usage", get(handlers::memory::get_memory_usage))
        .route("/api/performance/memory/swap", get(handlers::memory::get_swap_usage))
        .route("/api/performance/memory/swap", put(handlers::memory::configure_swap))
        .route("/api/performance/memory/cache", get(handlers::memory::get_memory_cache))
        .route("/api/performance/memory/cache/clear", post(handlers::memory::clear_cache))
        .route("/api/performance/memory/fragmentation", get(handlers::memory::get_fragmentation))
        .route("/api/performance/memory/compaction", post(handlers::memory::compact_memory))
        .route("/api/performance/memory/huge-pages", get(handlers::memory::get_huge_pages))
        .route("/api/performance/memory/huge-pages", put(handlers::memory::configure_huge_pages))
        .route("/api/performance/memory/numa", get(handlers::memory::get_numa_info))
        .route("/api/performance/memory/numa", put(handlers::memory::optimize_numa))
        .route("/api/performance/memory/pressure", get(handlers::memory::get_memory_pressure))
        .route("/api/performance/memory/leaks", get(handlers::memory::detect_leaks))
        .route("/api/performance/memory/gc", get(handlers::memory::get_gc_stats))
        .route("/api/performance/memory/gc", post(handlers::memory::trigger_gc))
        .route("/api/performance/memory/optimize", post(handlers::memory::optimize_memory))

        // === 网络优化API (15个) ===
        .route("/api/performance/network/interfaces", get(handlers::network::get_interfaces))
        .route("/api/performance/network/stats", get(handlers::network::get_network_stats))
        .route("/api/performance/network/bandwidth", get(handlers::network::get_bandwidth))
        .route("/api/performance/network/latency", get(handlers::network::measure_latency))
        .route("/api/performance/network/connections", get(handlers::network::get_connections))
        .route("/api/performance/network/tcp-tuning", get(handlers::network::get_tcp_tuning))
        .route("/api/performance/network/tcp-tuning", put(handlers::network::set_tcp_tuning))
        .route("/api/performance/network/buffer-sizes", get(handlers::network::get_buffer_sizes))
        .route("/api/performance/network/buffer-sizes", put(handlers::network::set_buffer_sizes))
        .route("/api/performance/network/congestion", get(handlers::network::get_congestion_control))
        .route("/api/performance/network/congestion", put(handlers::network::set_congestion_control))
        .route("/api/performance/network/queue", get(handlers::network::get_queue_discipline))
        .route("/api/performance/network/queue", put(handlers::network::set_queue_discipline))
        .route("/api/performance/network/optimize", post(handlers::network::optimize_network))
        .route("/api/performance/network/test", post(handlers::network::run_network_test))

        // === 磁盘I/O优化API (18个) ===
        .route("/api/performance/disk/usage", get(handlers::disk::get_disk_usage))
        .route("/api/performance/disk/io-stats", get(handlers::disk::get_io_stats))
        .route("/api/performance/disk/iops", get(handlers::disk::measure_iops))
        .route("/api/performance/disk/latency", get(handlers::disk::measure_latency))
        .route("/api/performance/disk/scheduler", get(handlers::disk::get_io_scheduler))
        .route("/api/performance/disk/scheduler", put(handlers::disk::set_io_scheduler))
        .route("/api/performance/disk/queue-depth", get(handlers::disk::get_queue_depth))
        .route("/api/performance/disk/queue-depth", put(handlers::disk::set_queue_depth))
        .route("/api/performance/disk/read-ahead", get(handlers::disk::get_read_ahead))
        .route("/api/performance/disk/read-ahead", put(handlers::disk::set_read_ahead))
        .route("/api/performance/disk/cache", get(handlers::disk::get_disk_cache))
        .route("/api/performance/disk/cache", put(handlers::disk::configure_disk_cache))
        .route("/api/performance/disk/mount-options", get(handlers::disk::get_mount_options))
        .route("/api/performance/disk/mount-options", put(handlers::disk::set_mount_options))
        .route("/api/performance/disk/defrag", post(handlers::disk::defragment_disk))
        .route("/api/performance/disk/trim", post(handlers::disk::trim_ssd))
        .route("/api/performance/disk/benchmark", post(handlers::disk::run_disk_benchmark))
        .route("/api/performance/disk/optimize", post(handlers::disk::optimize_disk))

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4004));
    info!("⚡ Performance Service listening on http://{}", addr);
    info!("✅ All 67 APIs initialized successfully");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(StandardResponse::success(serde_json::json!({
        "status": "healthy",
        "service": "performance-service",
        "version": "1.0.0",
        "apis_count": 67,
        "optimization_categories": ["cpu", "memory", "network", "disk"],
        "timestamp": chrono::Utc::now().timestamp(),
    })))
}