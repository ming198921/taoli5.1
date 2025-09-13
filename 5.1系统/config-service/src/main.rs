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

use models::{StandardResponse, Configuration, ConfigVersion, HotReloadStatus};
use services::{ConfigManager, VersionController, HotReloadEngine, ValidationEngine};

#[derive(Clone)]
pub struct AppState {
    config_manager: Arc<ConfigManager>,
    version_controller: Arc<VersionController>,
    hot_reload_engine: Arc<HotReloadEngine>,
    validation_engine: Arc<ValidationEngine>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("config_service=debug")
        .init();

    info!("🚀 Starting Config Service v1.0.0 (96 APIs)");

    let config_manager = Arc::new(ConfigManager::new().await?);
    let version_controller = Arc::new(VersionController::new().await?);
    let hot_reload_engine = Arc::new(HotReloadEngine::new().await?);
    let validation_engine = Arc::new(ValidationEngine::new().await?);

    let app_state = AppState {
        config_manager,
        version_controller,
        hot_reload_engine,
        validation_engine,
    };

    // 构建路由 - 96个API端点
    let app = Router::new()
        .route("/health", get(health_check))
        
        // === 基础配置管理API (24个) ===
        .route("/api/config/list", get(handlers::basic::list_configs))
        .route("/api/config/:key", get(handlers::basic::get_config))
        .route("/api/config/:key", put(handlers::basic::set_config))
        .route("/api/config/:key", delete(handlers::basic::delete_config))
        .route("/api/config/:key/metadata", get(handlers::basic::get_config_metadata))
        .route("/api/config/:key/history", get(handlers::basic::get_config_history))
        .route("/api/config/batch/get", post(handlers::basic::batch_get_configs))
        .route("/api/config/batch/set", post(handlers::basic::batch_set_configs))
        .route("/api/config/batch/delete", post(handlers::basic::batch_delete_configs))
        .route("/api/config/search", post(handlers::basic::search_configs))
        .route("/api/config/tree", get(handlers::basic::get_config_tree))
        .route("/api/config/tree/:path", get(handlers::basic::get_config_subtree))
        .route("/api/config/export", post(handlers::basic::export_configs))
        .route("/api/config/import", post(handlers::basic::import_configs))
        .route("/api/config/validate", post(handlers::basic::validate_config))
        .route("/api/config/schema", get(handlers::basic::get_config_schema))
        .route("/api/config/schema", post(handlers::basic::update_config_schema))
        .route("/api/config/defaults", get(handlers::basic::get_default_configs))
        .route("/api/config/defaults", post(handlers::basic::set_default_configs))
        .route("/api/config/diff", post(handlers::basic::diff_configs))
        .route("/api/config/merge", post(handlers::basic::merge_configs))
        .route("/api/config/backup", post(handlers::basic::backup_configs))
        .route("/api/config/restore", post(handlers::basic::restore_configs))
        .route("/api/config/stats", get(handlers::basic::get_config_stats))

        // === 版本控制API (24个) ===
        .route("/api/config/versions", get(handlers::versions::list_versions))
        .route("/api/config/versions", post(handlers::versions::create_version))
        .route("/api/config/versions/:version", get(handlers::versions::get_version))
        .route("/api/config/versions/:version", delete(handlers::versions::delete_version))
        .route("/api/config/versions/:version/deploy", post(handlers::versions::deploy_version))
        .route("/api/config/versions/:version/rollback", post(handlers::versions::rollback_version))
        .route("/api/config/versions/:version/compare/:other", get(handlers::versions::compare_versions))
        .route("/api/config/versions/:version/changes", get(handlers::versions::get_version_changes))
        .route("/api/config/versions/current", get(handlers::versions::get_current_version))
        .route("/api/config/versions/latest", get(handlers::versions::get_latest_version))
        .route("/api/config/versions/:version/validate", post(handlers::versions::validate_version))
        .route("/api/config/versions/:version/conflicts", get(handlers::versions::check_conflicts))
        .route("/api/config/versions/branch", post(handlers::versions::create_branch))
        .route("/api/config/versions/merge", post(handlers::versions::merge_versions))
        .route("/api/config/versions/tag", post(handlers::versions::tag_version))
        .route("/api/config/versions/tags", get(handlers::versions::list_tags))
        .route("/api/config/versions/tags/:tag", get(handlers::versions::get_tagged_version))
        .route("/api/config/versions/:version/lock", post(handlers::versions::lock_version))
        .route("/api/config/versions/:version/unlock", post(handlers::versions::unlock_version))
        .route("/api/config/versions/:version/clone", post(handlers::versions::clone_version))
        .route("/api/config/versions/gc", post(handlers::versions::garbage_collect_versions))
        .route("/api/config/versions/audit", get(handlers::versions::get_version_audit))
        .route("/api/config/versions/permissions", get(handlers::versions::get_version_permissions))
        .route("/api/config/versions/permissions", put(handlers::versions::set_version_permissions))

        // === 热重载API (18个) ===
        .route("/api/config/hot-reload/status", get(handlers::hot_reload::get_reload_status))
        .route("/api/config/hot-reload/enable", post(handlers::hot_reload::enable_hot_reload))
        .route("/api/config/hot-reload/disable", post(handlers::hot_reload::disable_hot_reload))
        .route("/api/config/hot-reload/trigger", post(handlers::hot_reload::trigger_reload))
        .route("/api/config/hot-reload/validate", post(handlers::hot_reload::validate_reload))
        .route("/api/config/hot-reload/preview", post(handlers::hot_reload::preview_reload))
        .route("/api/config/hot-reload/rollback", post(handlers::hot_reload::rollback_reload))
        .route("/api/config/hot-reload/history", get(handlers::hot_reload::get_reload_history))
        .route("/api/config/hot-reload/services", get(handlers::hot_reload::list_reload_services))
        .route("/api/config/hot-reload/services/:service", get(handlers::hot_reload::get_service_reload_status))
        .route("/api/config/hot-reload/services/:service/trigger", post(handlers::hot_reload::trigger_service_reload))
        .route("/api/config/hot-reload/batch", post(handlers::hot_reload::batch_reload))
        .route("/api/config/hot-reload/schedule", post(handlers::hot_reload::schedule_reload))
        .route("/api/config/hot-reload/schedule/:id", get(handlers::hot_reload::get_scheduled_reload))
        .route("/api/config/hot-reload/schedule/:id", delete(handlers::hot_reload::cancel_scheduled_reload))
        .route("/api/config/hot-reload/hooks", get(handlers::hot_reload::list_reload_hooks))
        .route("/api/config/hot-reload/hooks", post(handlers::hot_reload::add_reload_hook))
        .route("/api/config/hot-reload/hooks/:id", delete(handlers::hot_reload::remove_reload_hook))

        // === 环境管理API (15个) ===
        .route("/api/config/environments", get(handlers::environments::list_environments))
        .route("/api/config/environments", post(handlers::environments::create_environment))
        .route("/api/config/environments/:env", get(handlers::environments::get_environment))
        .route("/api/config/environments/:env", put(handlers::environments::update_environment))
        .route("/api/config/environments/:env", delete(handlers::environments::delete_environment))
        .route("/api/config/environments/:env/configs", get(handlers::environments::get_env_configs))
        .route("/api/config/environments/:env/configs", put(handlers::environments::set_env_configs))
        .route("/api/config/environments/:env/deploy", post(handlers::environments::deploy_to_environment))
        .route("/api/config/environments/:env/promote", post(handlers::environments::promote_environment))
        .route("/api/config/environments/:env/clone", post(handlers::environments::clone_environment))
        .route("/api/config/environments/:env/diff/:other", get(handlers::environments::diff_environments))
        .route("/api/config/environments/:env/validate", post(handlers::environments::validate_environment))
        .route("/api/config/environments/:env/status", get(handlers::environments::get_environment_status))
        .route("/api/config/environments/:env/variables", get(handlers::environments::get_environment_variables))
        .route("/api/config/environments/:env/variables", put(handlers::environments::set_environment_variables))

        // === 权限与安全API (15个) ===
        .route("/api/config/permissions", get(handlers::security::get_permissions))
        .route("/api/config/permissions", put(handlers::security::set_permissions))
        .route("/api/config/permissions/:key", get(handlers::security::get_config_permissions))
        .route("/api/config/permissions/:key", put(handlers::security::set_config_permissions))
        .route("/api/config/access-logs", get(handlers::security::get_access_logs))
        .route("/api/config/audit", get(handlers::security::get_audit_logs))
        .route("/api/config/encrypt", post(handlers::security::encrypt_config))
        .route("/api/config/decrypt", post(handlers::security::decrypt_config))
        .route("/api/config/secrets", get(handlers::security::list_secrets))
        .route("/api/config/secrets/:key", get(handlers::security::get_secret))
        .route("/api/config/secrets/:key", put(handlers::security::set_secret))
        .route("/api/config/secrets/:key", delete(handlers::security::delete_secret))
        .route("/api/config/tokens", get(handlers::security::list_access_tokens))
        .route("/api/config/tokens", post(handlers::security::create_access_token))
        .route("/api/config/tokens/:token", delete(handlers::security::revoke_access_token))

        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4007));
    info!("⚙️ Config Service listening on http://{}", addr);
    info!("✅ All 96 APIs initialized successfully");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(StandardResponse::success(serde_json::json!({
        "status": "healthy",
        "service": "config-service", 
        "version": "1.0.0",
        "apis_count": 96,
        "features": ["basic-config", "versioning", "hot-reload", "environments", "security"],
        "timestamp": chrono::Utc::now().timestamp(),
    })))
}