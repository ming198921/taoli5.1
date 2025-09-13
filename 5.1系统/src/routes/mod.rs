/// 路由模块 - 集中管理所有API路由
pub mod auth;
pub mod system;
pub mod data;
pub mod dashboard;
pub mod monitoring;

use axum::Router;
use std::sync::Arc;

/// 注册所有路由
pub fn register_all_routes<S>(state: Arc<S>) -> Router 
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .merge(auth::routes(state.clone()))
        .merge(system::routes(state.clone()))
        .merge(data::routes(state.clone()))
        .merge(dashboard::routes(state.clone()))
        .merge(monitoring::routes(state))
}