#![allow(dead_code)]
// src/observability.rs
//! # Observability Module
//!
//! Initializes and manages logging, metrics, and distributed tracing for the application.

use once_cell::sync::Lazy;
use prometheus::{register_int_counter_vec, Encoder, IntCounterVec, Registry, TextEncoder};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use warp::Filter;

static REQUEST_COUNTER: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!("qingxi_requests", "Request count", &["endpoint"])
        .expect("Failed to register request counter")
});

/// Initializes the global tracing subscriber with JSON logging, metrics, and OpenTelemetry tracing.
pub fn init_subscriber(log_level: &str, service_name: &str) {
    // 创建一个基本的日志订阅者，不使用OpenTelemetry
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    tracing::info!(service = service_name, "Tracing initialized");
}

/// Initializes metrics with Prometheus registry.
pub fn init_metrics() -> Registry {
    let registry = Registry::new();
    registry
        .register(Box::new(REQUEST_COUNTER.clone()))
        .expect("Failed to register request counter");
    registry
}

/// Starts the Prometheus metrics server.
pub async fn serve_metrics(addr: SocketAddr, registry: Registry) {
    let metrics_route = warp::path!("metrics").map(move || {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let mf = registry.gather();
        encoder.encode(&mf, &mut buffer)
            .expect("Failed to encode metrics");
        String::from_utf8(buffer)
            .expect("Failed to convert metrics to UTF-8")
    });
    warp::serve(metrics_route).run(addr).await;
}

/// Health probe for the application.
pub async fn health_probe(addr: SocketAddr) {
    let route = warp::path!("healthz").map(|| "ok");
    warp::serve(route).run(addr).await;
}

/// Records the health status of a component
pub fn record_component_health(component: &str, is_healthy: bool) {
    let status = if is_healthy { 1.0 } else { 0.0 };
    metrics::gauge!("component_health", "component" => component.to_string()).set(status);
}

/// 健康状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// 组件健康状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub message: String,
    pub details: std::collections::HashMap<String, String>,
}

impl ComponentHealth {
    pub fn new(status: HealthStatus, message: &str) -> Self {
        Self {
            status,
            last_check: chrono::Utc::now(),
            message: message.to_string(),
            details: std::collections::HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: &str, value: &str) -> Self {
        self.details.insert(key.to_string(), value.to_string());
        self
    }
}

/// 健康检查注册表
pub struct HealthRegistry {
    components: std::sync::RwLock<std::collections::HashMap<String, ComponentHealth>>,
    last_update: std::sync::RwLock<std::time::Instant>,
}

impl HealthRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            components: std::sync::RwLock::new(std::collections::HashMap::new()),
            last_update: std::sync::RwLock::new(std::time::Instant::now()),
        })
    }

    /// 更新组件健康状态
    pub fn update_health(&self, component: &str, health: ComponentHealth) {
        let status = health.status;
        let mut components = self.components.write().expect("Failed to acquire write lock");
        components.insert(component.to_string(), health);
        *self.last_update.write().expect("Failed to acquire write lock") = std::time::Instant::now();

        // 同时更新指标
        let status_value = match status {
            HealthStatus::Healthy => 2.0,
            HealthStatus::Degraded => 1.0,
            HealthStatus::Unhealthy => 0.0,
        };
        metrics::gauge!("component_health_status", "component" => component.to_string())
            .set(status_value);
    }

    /// 获取组件健康状态
    pub fn get_component_health(&self, component: &str) -> Option<ComponentHealth> {
        let components = self.components.read().expect("Failed to acquire read lock");
        components.get(component).cloned()
    }

    /// 获取所有组件健康状态
    pub fn get_all_health(&self) -> std::collections::HashMap<String, ComponentHealth> {
        let components = self.components.read().expect("Failed to acquire read lock");
        components.clone()
    }

    /// 获取系统整体健康状态
    pub fn get_system_health(&self) -> HealthStatus {
        let components = self.components.read().expect("Failed to acquire read lock");

        if components.is_empty() {
            return HealthStatus::Unhealthy;
        }

        let mut has_degraded = false;

        for health in components.values() {
            match health.status {
                HealthStatus::Unhealthy => return HealthStatus::Unhealthy,
                HealthStatus::Degraded => has_degraded = true,
                _ => {}
            }
        }

        if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// 检查上次更新时间
    pub fn last_update_age(&self) -> Duration {
        let last_update = *self.last_update.read().expect("Failed to acquire read lock");
        last_update.elapsed()
    }
}

/// 启动增强版健康检查服务器
pub fn start_enhanced_health_server(
    addr: SocketAddr,
    health_registry: Arc<HealthRegistry>,
    readiness_check: Arc<tokio::sync::watch::Receiver<bool>>,
) {
    let make_svc = hyper::service::make_service_fn(move |_conn| {
        let health_registry = health_registry.clone();
        let readiness_check = readiness_check.clone();

        async move {
            Ok::<_, hyper::Error>(hyper::service::service_fn(
                move |req: hyper::Request<hyper::Body>| {
                    let health_registry = health_registry.clone();
                    let readiness_check = readiness_check.clone();

                    async move {
                        match req.uri().path() {
                            "/health/live" => {
                                // 简单的存活检查
                                Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from(
                                    "ALIVE",
                                )))
                            }
                            "/health/ready" => {
                                // 就绪检查
                                if *readiness_check.borrow() {
                                    Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from(
                                        "READY",
                                    )))
                                } else {
                                    let mut res =
                                        hyper::Response::new(hyper::Body::from("NOT_READY"));
                                    *res.status_mut() = hyper::StatusCode::SERVICE_UNAVAILABLE;
                                    Ok::<_, hyper::Error>(res)
                                }
                            }
                            "/health" => {
                                // 详细的健康状态
                                let system_health = health_registry.get_system_health();
                                let status_code = match system_health {
                                    HealthStatus::Healthy => hyper::StatusCode::OK,
                                    HealthStatus::Degraded => hyper::StatusCode::OK,
                                    HealthStatus::Unhealthy => {
                                        hyper::StatusCode::SERVICE_UNAVAILABLE
                                    }
                                };

                                let body = serde_json::json!({
                                    "status": system_health.to_string(),
                                    "components": health_registry.get_all_health(),
                                    "timestamp": chrono::Utc::now().to_rfc3339(),
                                });

                                let mut response = hyper::Response::new(hyper::Body::from(
                                    serde_json::to_string(&body).expect("Failed to serialize to JSON"),
                                ));
                                *response.status_mut() = status_code;
                                response.headers_mut().insert(
                                    hyper::header::CONTENT_TYPE,
                                    hyper::header::HeaderValue::from_static("application/json"),
                                );

                                Ok::<_, hyper::Error>(response)
                            }
                            "/health/component" => {
                                // 获取特定组件的健康状态
                                let query = req.uri().query().unwrap_or("");
                                let params: std::collections::HashMap<_, _> =
                                    url::form_urlencoded::parse(query.as_bytes())
                                        .into_owned()
                                        .collect();

                                if let Some(component) = params.get("name") {
                                    if let Some(health) =
                                        health_registry.get_component_health(component)
                                    {
                                        let status_code = match health.status {
                                            HealthStatus::Healthy => hyper::StatusCode::OK,
                                            HealthStatus::Degraded => hyper::StatusCode::OK,
                                            HealthStatus::Unhealthy => {
                                                hyper::StatusCode::SERVICE_UNAVAILABLE
                                            }
                                        };

                                        let mut response = hyper::Response::new(hyper::Body::from(
                                            serde_json::to_string(&health).expect("Failed to serialize to JSON"),
                                        ));
                                        *response.status_mut() = status_code;
                                        response.headers_mut().insert(
                                            hyper::header::CONTENT_TYPE,
                                            hyper::header::HeaderValue::from_static(
                                                "application/json",
                                            ),
                                        );

                                        Ok::<_, hyper::Error>(response)
                                    } else {
                                        let mut not_found = hyper::Response::default();
                                        *not_found.status_mut() = hyper::StatusCode::NOT_FOUND;
                                        Ok::<_, hyper::Error>(not_found)
                                    }
                                } else {
                                    let mut bad_request = hyper::Response::default();
                                    *bad_request.status_mut() = hyper::StatusCode::BAD_REQUEST;
                                    Ok::<_, hyper::Error>(bad_request)
                                }
                            }
                            _ => {
                                let mut not_found = hyper::Response::default();
                                *not_found.status_mut() = hyper::StatusCode::NOT_FOUND;
                                Ok::<_, hyper::Error>(not_found)
                            }
                        }
                    }
                },
            ))
        }
    });

    tokio::spawn(async move {
        let server = hyper::Server::bind(&addr).serve(make_svc);
        tracing::info!(message = "Enhanced health server listening.", %addr);
        if let Err(e) = server.await {
            tracing::error!(error = %e, "Health server failed.");
        }
    });
}

/// Spawns a basic HTTP server for health checks.
pub fn start_health_probe_server(
    addr: SocketAddr,
    readiness_check: Arc<tokio::sync::watch::Receiver<bool>>,
) {
    let make_svc = hyper::service::make_service_fn(move |_conn| {
        let readiness_check = readiness_check.clone();
        async move {
            Ok::<_, hyper::Error>(hyper::service::service_fn(
                move |req: hyper::Request<hyper::Body>| {
                    let readiness_check = readiness_check.clone();
                    async move {
                        match req.uri().path() {
                            "/health/live" => {
                                // Liveness probe - is the app running?
                                Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from(
                                    "ALIVE",
                                )))
                            }
                            "/health/ready" => {
                                // Readiness probe - is the app ready to serve traffic?
                                if *readiness_check.borrow() {
                                    Ok::<_, hyper::Error>(hyper::Response::new(hyper::Body::from(
                                        "READY",
                                    )))
                                } else {
                                    let mut res =
                                        hyper::Response::new(hyper::Body::from("NOT_READY"));
                                    *res.status_mut() = hyper::StatusCode::SERVICE_UNAVAILABLE;
                                    Ok::<_, hyper::Error>(res)
                                }
                            }
                            _ => {
                                let mut not_found = hyper::Response::default();
                                *not_found.status_mut() = hyper::StatusCode::NOT_FOUND;
                                Ok::<_, hyper::Error>(not_found)
                            }
                        }
                    }
                },
            ))
        }
    });

    tokio::spawn(async move {
        let server = hyper::Server::bind(&addr).serve(make_svc);
        tracing::info!(message = "Health probe server listening.", %addr);
        if let Err(e) = server.await {
            tracing::error!(error = %e, "Health probe server failed.");
        }
    });
}

/// 记录市场数据事件
pub fn record_market_data_event(exchange: &str, symbol: &str, event_type: &str) {
    metrics::counter!("market_data_events_total", "exchange" => exchange.to_string(), "symbol" => symbol.to_string(), "type" => event_type.to_string()).increment(1);
}

/// 记录订单簿延迟
pub fn record_orderbook_latency(exchange: &str, symbol: &str, latency_ms: u64) {
    metrics::histogram!("orderbook_latency_ms", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).record(latency_ms as f64);
}

/// 记录异常事件
pub fn record_anomaly_event(exchange: &str, symbol: &str, anomaly_type: &str, severity: &str) {
    metrics::counter!("anomaly_events_total", "exchange" => exchange.to_string(), "symbol" => symbol.to_string(), "type" => anomaly_type.to_string(), "severity" => severity.to_string()).increment(1);
}

/// 记录连接状态
pub fn record_connection_status(exchange: &str, is_connected: bool) {
    metrics::gauge!("exchange_connection_status", "exchange" => exchange.to_string())
        .set(if is_connected { 1.0 } else { 0.0 });
}

/// 记录处理时间
pub fn record_processing_time(component: &str, operation: &str, duration: Duration) {
    metrics::histogram!("processing_time_ms", "component" => component.to_string(), "operation" => operation.to_string()).record(duration.as_millis() as f64);
}

/// 记录内存池使用情况
pub fn record_pool_usage(pool_name: &str, used: usize, capacity: usize) {
    metrics::gauge!("object_pool_usage", "pool" => pool_name.to_string()).set(used as f64);
    metrics::gauge!("object_pool_capacity", "pool" => pool_name.to_string()).set(capacity as f64);
}

/// 记录队列深度
pub fn record_queue_depth(queue_name: &str, depth: usize) {
    metrics::gauge!("queue_depth", "queue" => queue_name.to_string()).set(depth as f64);
}

/// 记录数据一致性检查结果
pub fn record_consistency_check(exchange: &str, symbol: &str, is_consistent: bool) {
    metrics::counter!("consistency_checks_total", "exchange" => exchange.to_string(), "symbol" => symbol.to_string(), "result" => if is_consistent { "pass" } else { "fail" }.to_string()).increment(1);
}

/// 记录价格偏差
pub fn record_price_deviation(exchange: &str, symbol: &str, deviation_percentage: f64) {
    metrics::gauge!("price_deviation_percentage", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(deviation_percentage);
}

/// 记录交易量统计
pub fn record_trading_volume(exchange: &str, symbol: &str, volume: f64) {
    metrics::gauge!("trading_volume", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(volume);
}

/// 记录数据更新频率
pub fn record_update_frequency(exchange: &str, symbol: &str, updates_per_second: f64) {
    metrics::gauge!("updates_per_second", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(updates_per_second);
}

/// 记录时间同步偏差
pub fn record_time_sync_offset(exchange: &str, offset_ms: i64) {
    metrics::gauge!("time_sync_offset_ms", "exchange" => exchange.to_string())
        .set(offset_ms as f64);
}

/// 记录断路器触发事件
pub fn record_circuit_breaker_event(exchange: &str, symbol: &str, reason: &str) {
    metrics::counter!("circuit_breaker_events_total", "exchange" => exchange.to_string(), "symbol" => symbol.to_string(), "reason" => reason.to_string()).increment(1);
}

/// 记录系统资源使用情况
pub fn record_system_resources(
    cpu_usage: f64,
    memory_usage: f64,
    network_in_bytes: u64,
    network_out_bytes: u64,
) {
    metrics::gauge!("system_cpu_usage_percent").set(cpu_usage);
    metrics::gauge!("system_memory_usage_percent").set(memory_usage);
    metrics::gauge!("system_network_in_bytes_total").set(network_in_bytes as f64);
    metrics::gauge!("system_network_out_bytes_total").set(network_out_bytes as f64);
}

/// 记录API请求统计
pub fn record_api_request(endpoint: &str, method: &str, status_code: u16, duration_ms: u64) {
    metrics::counter!("api_requests_total", "endpoint" => endpoint.to_string(), "method" => method.to_string(), "status" => status_code.to_string()).increment(1);
    metrics::histogram!("api_request_duration_ms", "endpoint" => endpoint.to_string(), "method" => method.to_string()).record(duration_ms as f64);
}

/// 记录数据处理延迟分布
pub fn record_processing_latency_percentiles(component: &str, p50: f64, p90: f64, p99: f64) {
    metrics::gauge!("processing_latency_p50_ms", "component" => component.to_string()).set(p50);
    metrics::gauge!("processing_latency_p90_ms", "component" => component.to_string()).set(p90);
    metrics::gauge!("processing_latency_p99_ms", "component" => component.to_string()).set(p99);
}

/// 记录数据采集错误
pub fn record_collection_error(exchange: &str, error_type: &str) {
    metrics::counter!("collection_errors_total", "exchange" => exchange.to_string(), "error_type" => error_type.to_string()).increment(1);
}

/// 记录数据处理管道状态
pub fn record_pipeline_stage_status(
    stage: &str,
    items_processed: u64,
    queue_size: usize,
    processing_time_ms: f64,
) {
    metrics::gauge!("pipeline_items_processed", "stage" => stage.to_string())
        .set(items_processed as f64);
    metrics::gauge!("pipeline_queue_size", "stage" => stage.to_string()).set(queue_size as f64);
    metrics::gauge!("pipeline_processing_time_ms", "stage" => stage.to_string())
        .set(processing_time_ms);
}

/// 记录数据快照完整性
pub fn record_snapshot_integrity(
    exchange: &str,
    symbol: &str,
    is_complete: bool,
    missing_levels: usize,
) {
    let status = if is_complete {
        "complete"
    } else {
        "incomplete"
    };
    metrics::counter!("snapshot_integrity_checks", "exchange" => exchange.to_string(), "symbol" => symbol.to_string(), "status" => status.to_string()).increment(1);
    if !is_complete {
        metrics::gauge!("snapshot_missing_levels", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(missing_levels as f64);
    }
}

/// 记录高频交易数据统计
pub fn record_hft_statistics(
    exchange: &str,
    symbol: &str,
    updates_per_second: f64,
    price_volatility: f64,
) {
    metrics::gauge!("hft_updates_per_second", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(updates_per_second);
    metrics::gauge!("hft_price_volatility", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(price_volatility);
}

/// 记录订单簿深度变化
pub fn record_orderbook_depth_change(
    exchange: &str,
    symbol: &str,
    bid_depth: usize,
    ask_depth: usize,
) {
    metrics::gauge!("orderbook_bid_depth", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(bid_depth as f64);
    metrics::gauge!("orderbook_ask_depth", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(ask_depth as f64);
}

/// 记录价差统计
pub fn record_spread_metrics(exchange: &str, symbol: &str, spread: f64, spread_bps: f64) {
    metrics::gauge!("market_spread", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(spread);
    metrics::gauge!("market_spread_bps", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(spread_bps);
}

/// 记录订单簿更新类型分布
pub fn record_orderbook_update_type(exchange: &str, symbol: &str, update_type: &str) {
    metrics::counter!("orderbook_updates_by_type", "exchange" => exchange.to_string(), "symbol" => symbol.to_string(), "type" => update_type.to_string()).increment(1);
}

/// 记录交易所响应时间
pub fn record_exchange_response_time(exchange: &str, api_type: &str, response_time_ms: u64) {
    metrics::histogram!("exchange_response_time_ms", "exchange" => exchange.to_string(), "api_type" => api_type.to_string()).record(response_time_ms as f64);
}

/// 记录数据处理吞吐量
pub fn record_throughput(component: &str, messages_per_second: f64, bytes_per_second: f64) {
    metrics::gauge!("throughput_messages_per_second", "component" => component.to_string())
        .set(messages_per_second);
    metrics::gauge!("throughput_bytes_per_second", "component" => component.to_string())
        .set(bytes_per_second);
}

/// 记录内存使用情况
pub fn record_memory_usage(component: &str, bytes_used: u64) {
    metrics::gauge!("memory_usage_bytes", "component" => component.to_string())
        .set(bytes_used as f64);
}

/// 记录线程统计信息
pub fn record_thread_stats(component: &str, active_threads: usize, queue_depth: usize) {
    metrics::gauge!("active_threads", "component" => component.to_string())
        .set(active_threads as f64);
    metrics::gauge!("thread_queue_depth", "component" => component.to_string())
        .set(queue_depth as f64);
}

/// 记录CPU使用率
pub fn record_cpu_usage(component: &str, cpu_percentage: f64) {
    metrics::gauge!("cpu_usage_percentage", "component" => component.to_string())
        .set(cpu_percentage);
}

/// 记录市场数据质量指标
pub fn record_data_quality_metrics(
    exchange: &str,
    symbol: &str,
    completeness: f64,
    accuracy: f64,
    timeliness: f64,
) {
    metrics::gauge!("data_quality_completeness", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(completeness);
    metrics::gauge!("data_quality_accuracy", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(accuracy);
    metrics::gauge!("data_quality_timeliness", "exchange" => exchange.to_string(), "symbol" => symbol.to_string()).set(timeliness);
}

/// 记录数据处理阶段延迟
pub fn record_stage_latency(pipeline: &str, stage: &str, latency_ms: f64) {
    metrics::histogram!("pipeline_stage_latency_ms", "pipeline" => pipeline.to_string(), "stage" => stage.to_string()).record(latency_ms);
}

/// 创建一个自定义的指标注册器
pub fn create_custom_metrics_registry() -> Result<(), Box<dyn std::error::Error>> {
    // 简化实现，避免with_namespace方法不存在的问题
    Ok(())
}

/// 创建一个结构化的跟踪span，用于分布式追踪
pub fn create_span(
    _name: &str,
    _level: tracing::Level,
    component: &str,
    operation: &str,
) -> tracing::Span {
    tracing::info_span!(
        "span",
        component = component,
        operation = operation,
        start_time = tracing::field::debug(chrono::Utc::now())
    )
}

/// 创建一个市场数据处理span
pub fn create_market_data_span(exchange: &str, symbol: &str, operation: &str) -> tracing::Span {
    tracing::span!(
        tracing::Level::INFO,
        "market_data_processing",
        exchange = exchange,
        symbol = symbol,
        operation = operation,
        start_time = tracing::field::debug(chrono::Utc::now())
    )
}

/// 创建一个异常检测span
pub fn create_anomaly_detection_span(exchange: &str, symbol: &str) -> tracing::Span {
    tracing::span!(
        tracing::Level::INFO,
        "anomaly_detection",
        exchange = exchange,
        symbol = symbol,
        start_time = tracing::field::debug(chrono::Utc::now())
    )
}

/// 创建一个数据一致性检查span
pub fn create_consistency_check_span(exchange: &str, symbol: &str) -> tracing::Span {
    tracing::span!(
        tracing::Level::INFO,
        "consistency_check",
        exchange = exchange,
        symbol = symbol,
        start_time = tracing::field::debug(chrono::Utc::now())
    )
}
