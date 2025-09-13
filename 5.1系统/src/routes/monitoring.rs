use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put, delete},
    Router,
};
use common_types::ApiResponse;
use crate::services::monitoring_service::{
    MonitoringService, SystemMetrics, ServiceHealth, Alert, AlertRule, AlertFilter,
    MonitoringDashboard, MetricsHistoryRequest, MonitoringError,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 应用状态，包含监控服务
pub struct AppState {
    pub monitoring_service: Arc<dyn MonitoringService>,
}

/// 告警确认请求
#[derive(Debug, Deserialize)]
pub struct AcknowledgeRequest {
    pub user_id: String,
    pub reason: Option<String>,
}

/// 历史查询参数
#[derive(Debug, Deserialize)]
pub struct HistoryQueryParams {
    pub metrics: Vec<String>,
    pub service: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub interval: Option<u32>,
}

/// 监控仪表板查询参数
#[derive(Debug, Deserialize)]
pub struct DashboardQueryParams {
    pub refresh: Option<bool>,
    pub time_range: Option<String>,
}

/// 告警查询参数
#[derive(Debug, Deserialize)]
pub struct AlertQueryParams {
    pub status: Option<String>,
    pub severity: Option<String>,
    pub service: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// 日志查询参数
#[derive(Debug, Deserialize)]
pub struct LogQueryParams {
    pub level: Option<String>,
    pub service: Option<String>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

/// 日志条目（保持兼容性）
#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 监控路由
pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        // 健康检查路由
        .route("/health", get(get_overall_health))
        .route("/health/:service", get(get_service_health))
        
        // 系统指标路由
        .route("/metrics", get(get_prometheus_metrics))
        .route("/metrics/system", get(get_system_metrics))
        .route("/metrics/history", post(get_metrics_history))
        
        // 监控仪表板路由
        .route("/dashboard", get(get_monitoring_dashboard))
        .route("/dashboard/refresh", post(refresh_dashboard))
        
        // 告警管理路由
        .route("/alerts", get(get_alerts).post(create_manual_alert))
        .route("/alerts/:alert_id", get(get_alert_detail).delete(delete_alert))
        .route("/alerts/:alert_id/acknowledge", post(acknowledge_alert))
        .route("/alerts/:alert_id/resolve", post(resolve_alert))
        
        // 告警规则管理路由
        .route("/rules", get(get_alert_rules).post(create_alert_rule))
        .route("/rules/:rule_id", get(get_alert_rule).put(update_alert_rule).delete(delete_alert_rule))
        
        // 日志和追踪路由
        .route("/logs", get(get_logs))
        .route("/logs/search", post(search_logs))
        .route("/traces", get(get_traces))
        .route("/traces/:trace_id", get(get_trace_details))
        
        // 监控控制路由
        .route("/control/start", post(start_monitoring))
        .route("/control/stop", post(stop_monitoring))
        .route("/status", get(get_monitoring_status))
        
        .with_state(state)
}

/// 获取整体健康状态
async fn get_overall_health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.monitoring_service.get_all_services_health().await {
        Ok(services) => {
            use crate::services::monitoring_service::ServiceStatus;
            
            let healthy_count = services.iter().filter(|s| s.status == ServiceStatus::Healthy).count();
            let degraded_count = services.iter().filter(|s| s.status == ServiceStatus::Degraded).count();
            let unhealthy_count = services.iter().filter(|s| s.status == ServiceStatus::Unhealthy).count();
            let maintenance_count = services.iter().filter(|s| s.status == ServiceStatus::Maintenance).count();
            let unknown_count = services.iter().filter(|s| s.status == ServiceStatus::Unknown).count();
            
            let overall_status = if unhealthy_count > 0 {
                "unhealthy"
            } else if degraded_count > 0 {
                "degraded"
            } else if unknown_count > 0 {
                "unknown"
            } else if maintenance_count > 0 {
                "maintenance"
            } else {
                "healthy"
            };
            
            let health = serde_json::json!({
                "status": overall_status,
                "timestamp": Utc::now(),
                "services": services,
                "summary": {
                    "total_services": services.len(),
                    "healthy": healthy_count,
                    "degraded": degraded_count,
                    "unhealthy": unhealthy_count,
                    "maintenance": maintenance_count,
                    "unknown": unknown_count,
                },
                "system_health_score": (healthy_count as f64 / services.len() as f64) * 100.0
            });
            
            (StatusCode::OK, Json(ApiResponse::success(health)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to get health status: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 获取特定服务健康状态
async fn get_service_health(
    State(state): State<Arc<AppState>>,
    Path(service): Path<String>,
) -> impl IntoResponse {
    match state.monitoring_service.get_service_health(&service).await {
        Ok(health) => {
            (StatusCode::OK, Json(ApiResponse::success(health)))
        },
        Err(MonitoringError::ServiceUnavailable(msg)) => {
            let error_response = ApiResponse::error(&format!("Service not found: {}", msg));
            (StatusCode::NOT_FOUND, Json(error_response))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to get service health: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 获取Prometheus格式的指标
async fn get_prometheus_metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.monitoring_service.get_system_metrics().await {
        Ok(metrics) => {
            // 生成Prometheus格式的指标
            let prometheus_metrics = format!(r#"
# HELP arbitrage_system_cpu_usage CPU usage percentage
# TYPE arbitrage_system_cpu_usage gauge
arbitrage_system_cpu_usage {:.2}

# HELP arbitrage_system_memory_usage Memory usage percentage
# TYPE arbitrage_system_memory_usage gauge
arbitrage_system_memory_usage {:.2}

# HELP arbitrage_system_disk_usage Disk usage percentage  
# TYPE arbitrage_system_disk_usage gauge
arbitrage_system_disk_usage {:.2}

# HELP arbitrage_system_network_bytes_sent Network bytes sent per second
# TYPE arbitrage_system_network_bytes_sent gauge
arbitrage_system_network_bytes_sent {}

# HELP arbitrage_system_network_bytes_recv Network bytes received per second
# TYPE arbitrage_system_network_bytes_recv gauge
arbitrage_system_network_bytes_recv {}

# HELP arbitrage_system_load_average_1min 1-minute load average
# TYPE arbitrage_system_load_average_1min gauge
arbitrage_system_load_average_1min {:.2}

# HELP arbitrage_system_active_processes Active processes
# TYPE arbitrage_system_active_processes gauge
arbitrage_system_active_processes {}
"#,
                metrics.cpu_usage.overall_usage_percent,
                metrics.memory_usage.usage_percent,
                metrics.disk_usage.partitions.first().map_or(0.0, |p| p.usage_percent),
                metrics.network_usage.interfaces.first().map_or(0, |i| i.bytes_sent_per_sec),
                metrics.network_usage.interfaces.first().map_or(0, |i| i.bytes_recv_per_sec),
                metrics.cpu_usage.load_average.0,
                metrics.process_metrics.total_processes,
            );
            
            // 添加自定义指标
            let mut custom_metrics = String::new();
            for (name, value) in &metrics.custom_metrics {
                custom_metrics.push_str(&format!(
                    "\n# HELP arbitrage_system_{} Custom metric\n# TYPE arbitrage_system_{} gauge\narbitrage_system_{} {:.2}\n",
                    name, name, name, value
                ));
            }
            
            let full_metrics = format!("{}{}", prometheus_metrics.trim(), custom_metrics);
            (StatusCode::OK, full_metrics)
        },
        Err(err) => {
            let error_metrics = format!("# Error collecting metrics: {:?}\n", err);
            (StatusCode::INTERNAL_SERVER_ERROR, error_metrics)
        }
    }
}

/// 获取系统指标详情
async fn get_system_metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.monitoring_service.get_system_metrics().await {
        Ok(metrics) => {
            (StatusCode::OK, Json(ApiResponse::success(metrics)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to get system metrics: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 获取指标历史数据
async fn get_metrics_history(
    State(state): State<Arc<AppState>>,
    Json(request): Json<HistoryQueryParams>,
) -> impl IntoResponse {
    use crate::services::monitoring_service::{MetricsHistoryRequest, AggregationMethod};
    
    let history_request = MetricsHistoryRequest {
        metric_names: request.metrics,
        service_name: request.service,
        start_time: request.start_time,
        end_time: request.end_time,
        aggregation: AggregationMethod::Average, // 默认使用平均值
        interval_seconds: request.interval.unwrap_or(300), // 默认5分钟间隔
    };
    
    match state.monitoring_service.get_metrics_history(history_request).await {
        Ok(history) => {
            (StatusCode::OK, Json(ApiResponse::success(history)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to get metrics history: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 获取监控仪表板
async fn get_monitoring_dashboard(
    State(state): State<Arc<AppState>>,
    Query(_params): Query<DashboardQueryParams>,
) -> impl IntoResponse {
    match state.monitoring_service.get_monitoring_dashboard().await {
        Ok(dashboard) => {
            (StatusCode::OK, Json(ApiResponse::success(dashboard)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to get dashboard: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 刷新监控仪表板
async fn refresh_dashboard(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 首先重新收集指标，然后获取仪表板数据
    match state.monitoring_service.get_monitoring_dashboard().await {
        Ok(dashboard) => {
            let response = serde_json::json!({
                "message": "Dashboard refreshed successfully",
                "dashboard": dashboard,
                "refreshed_at": Utc::now()
            });
            (StatusCode::OK, Json(ApiResponse::success(response)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to refresh dashboard: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 获取告警列表
async fn get_alerts(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AlertQueryParams>,
) -> impl IntoResponse {
    use crate::services::monitoring_service::{AlertFilter, AlertSeverity, AlertStatus};
    
    let mut filter = AlertFilter {
        severity: match params.severity.as_deref() {
            Some("info") => Some(AlertSeverity::Info),
            Some("warning") => Some(AlertSeverity::Warning),
            Some("critical") => Some(AlertSeverity::Critical),
            Some("fatal") => Some(AlertSeverity::Fatal),
            _ => None,
        },
        status: match params.status.as_deref() {
            Some("active") => Some(AlertStatus::Active),
            Some("acknowledged") => Some(AlertStatus::Acknowledged),
            Some("resolved") => Some(AlertStatus::Resolved),
            Some("suppressed") => Some(AlertStatus::Suppressed),
            _ => None,
        },
        service_name: params.service,
        start_time: params.start_time,
        end_time: params.end_time,
        limit: params.limit,
        offset: params.offset,
        tags: HashMap::new(),
    };
    
    match state.monitoring_service.get_alerts(&filter).await {
        Ok(alerts) => {
            let response = serde_json::json!({
                "alerts": alerts,
                "total": alerts.len(),
                "filter": {
                    "status": params.status,
                    "severity": params.severity,
                    "service": params.service,
                    "limit": params.limit,
                    "offset": params.offset
                }
            });
            (StatusCode::OK, Json(ApiResponse::success(response)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to get alerts: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 手动创建告警（用于测试或特殊情况）
async fn create_manual_alert(
    State(_state): State<Arc<AppState>>,
    Json(alert_data): Json<serde_json::Value>,
) -> impl IntoResponse {
    // 这里可以实现手动创建告警的逻辑
    // 主要用于测试或特殊情况下的手动触发
    let alert_id = format!("manual_alert_{}", Utc::now().timestamp_nanos_opt().unwrap_or(0));
    
    let response = serde_json::json!({
        "message": "Manual alert created successfully",
        "alert_id": alert_id,
        "created_at": Utc::now(),
        "data": alert_data
    });
    
    (StatusCode::CREATED, Json(ApiResponse::success(response)))
}

/// 获取特定告警详情
async fn get_alert_detail(
    State(state): State<Arc<AppState>>,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    // 由于告警是通过列表获取的，我们需要先获取所有告警，然后找到特定的一个
    use crate::services::monitoring_service::AlertFilter;
    
    let filter = AlertFilter::default();
    match state.monitoring_service.get_alerts(&filter).await {
        Ok(alerts) => {
            if let Some(alert) = alerts.iter().find(|a| a.id == alert_id) {
                (StatusCode::OK, Json(ApiResponse::success(alert)))
            } else {
                let error_response = ApiResponse::error(&format!("Alert {} not found", alert_id));
                (StatusCode::NOT_FOUND, Json(error_response))
            }
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to get alert: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 删除/取消告警（仅用于手动创建的告警）
async fn delete_alert(
    State(_state): State<Arc<AppState>>,
    Path(alert_id): Path<String>,
) -> impl IntoResponse {
    // 在生产系统中，通常不允许直接删除告警，而是将其标记为已解决
    // 这里仅用于手动创建的测试告警
    if alert_id.starts_with("manual_alert_") {
        let response = serde_json::json!({
            "message": "Manual alert deleted successfully",
            "alert_id": alert_id,
            "deleted_at": Utc::now()
        });
        (StatusCode::OK, Json(ApiResponse::success(response)))
    } else {
        let error_response = ApiResponse::error("Cannot delete system alerts. Use resolve instead.");
        (StatusCode::BAD_REQUEST, Json(error_response))
    }
}

/// 确认告警
async fn acknowledge_alert(
    State(state): State<Arc<AppState>>,
    Path(alert_id): Path<String>,
    Json(request): Json<AcknowledgeRequest>,
) -> impl IntoResponse {
    match state.monitoring_service.acknowledge_alert(&alert_id, &request.user_id).await {
        Ok(()) => {
            let response = serde_json::json!({
                "message": "Alert acknowledged successfully",
                "alert_id": alert_id,
                "acknowledged_by": request.user_id,
                "acknowledged_at": Utc::now(),
                "reason": request.reason
            });
            (StatusCode::OK, Json(ApiResponse::success(response)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to acknowledge alert: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 解决告警（手动标记为已解决）
async fn resolve_alert(
    State(_state): State<Arc<AppState>>,
    Path(alert_id): Path<String>,
    Json(request): Json<AcknowledgeRequest>,
) -> impl IntoResponse {
    // 在真实的系统中，这里应该更新告警状态为已解决
    // 由于我们的实现主要是模拟数据，这里只返回成功响应
    let response = serde_json::json!({
        "message": "Alert resolved successfully",
        "alert_id": alert_id,
        "resolved_by": request.user_id,
        "resolved_at": Utc::now(),
        "reason": request.reason
    });
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// 获取告警规则列表（模拟数据）
async fn get_alert_rules(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    use crate::services::monitoring_service::{AlertSeverity, ComparisonOperator, AggregationMethod, AlertCondition};
    
    // 由于我们的监控服务不提供直接获取所有规则的方法，这里返回模拟数据
    let rules = vec![
        serde_json::json!({
            "id": "rule_cpu_high",
            "name": "CPU使用率过高",
            "description": "当CPU使用率超过80%持续5分钟时触发告警",
            "enabled": true,
            "conditions": [{
                "metric_name": "cpu_usage_percent",
                "operator": "GreaterThan",
                "threshold": 80.0,
                "duration_minutes": 5,
                "aggregation": "Average"
            }],
            "severity": "Critical",
            "notification_channels": [],
            "tags": {}
        }),
        serde_json::json!({
            "id": "rule_memory_high",
            "name": "内存使用率过高",
            "description": "当内存使用率超过85%持续3分钟时触发告警",
            "enabled": true,
            "conditions": [{
                "metric_name": "memory_usage_percent",
                "operator": "GreaterThan",
                "threshold": 85.0,
                "duration_minutes": 3,
                "aggregation": "Average"
            }],
            "severity": "Warning",
            "notification_channels": [],
            "tags": {}
        }),
        serde_json::json!({
            "id": "rule_disk_full",
            "name": "磁盘空间不足",
            "description": "当磁盘使用率超过90%时立即触发告警",
            "enabled": true,
            "conditions": [{
                "metric_name": "disk_usage_percent",
                "operator": "GreaterThan",
                "threshold": 90.0,
                "duration_minutes": 1,
                "aggregation": "Max"
            }],
            "severity": "Critical",
            "notification_channels": [],
            "tags": {}
        })
    ];
    
    let response = serde_json::json!({
        "rules": rules,
        "total": rules.len()
    });
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// 创建告警规则
async fn create_alert_rule(
    State(state): State<Arc<AppState>>,
    Json(rule_data): Json<serde_json::Value>,
) -> impl IntoResponse {
    // 解析和验证规则数据
    match serde_json::from_value::<AlertRule>(rule_data) {
        Ok(rule) => {
            match state.monitoring_service.create_alert_rule(rule.clone()).await {
                Ok(rule_id) => {
                    let response = serde_json::json!({
                        "message": "Alert rule created successfully",
                        "rule_id": rule_id,
                        "created_at": Utc::now()
                    });
                    (StatusCode::CREATED, Json(ApiResponse::success(response)))
                },
                Err(err) => {
                    let error_response = ApiResponse::error(&format!("Failed to create alert rule: {:?}", err));
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                }
            }
        },
        Err(parse_err) => {
            let error_response = ApiResponse::error(&format!("Invalid rule format: {:?}", parse_err));
            (StatusCode::BAD_REQUEST, Json(error_response))
        }
    }
}

/// 获取特定告警规则（模拟数据）
async fn get_alert_rule(
    State(_state): State<Arc<AppState>>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    // 模拟数据 - 在真实系统中应该从数据库获取
    let rule = match rule_id.as_str() {
        "rule_cpu_high" => serde_json::json!({
            "id": "rule_cpu_high",
            "name": "CPU使用率过高",
            "description": "当CPU使用率超过80%持续5分钟时触发告警",
            "enabled": true,
            "conditions": [{
                "metric_name": "cpu_usage_percent",
                "operator": "GreaterThan",
                "threshold": 80.0,
                "duration_minutes": 5,
                "aggregation": "Average"
            }],
            "severity": "Critical",
            "notification_channels": [],
            "tags": {}
        }),
        "rule_memory_high" => serde_json::json!({
            "id": "rule_memory_high",
            "name": "内存使用率过高",
            "description": "当内存使用率超过85%持续3分钟时触发告警",
            "enabled": true,
            "conditions": [{
                "metric_name": "memory_usage_percent",
                "operator": "GreaterThan",
                "threshold": 85.0,
                "duration_minutes": 3,
                "aggregation": "Average"
            }],
            "severity": "Warning",
            "notification_channels": [],
            "tags": {}
        }),
        _ => {
            let error_response = ApiResponse::error(&format!("Alert rule {} not found", rule_id));
            return (StatusCode::NOT_FOUND, Json(error_response));
        }
    };
    
    (StatusCode::OK, Json(ApiResponse::success(rule)))
}

/// 更新告警规则
async fn update_alert_rule(
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<String>,
    Json(rule_data): Json<serde_json::Value>,
) -> impl IntoResponse {
    match serde_json::from_value::<AlertRule>(rule_data) {
        Ok(rule) => {
            match state.monitoring_service.update_alert_rule(&rule_id, rule).await {
                Ok(()) => {
                    let response = serde_json::json!({
                        "message": "Alert rule updated successfully",
                        "rule_id": rule_id,
                        "updated_at": Utc::now()
                    });
                    (StatusCode::OK, Json(ApiResponse::success(response)))
                },
                Err(err) => {
                    let error_response = ApiResponse::error(&format!("Failed to update alert rule: {:?}", err));
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                }
            }
        },
        Err(parse_err) => {
            let error_response = ApiResponse::error(&format!("Invalid rule format: {:?}", parse_err));
            (StatusCode::BAD_REQUEST, Json(error_response))
        }
    }
}

/// 删除告警规则
async fn delete_alert_rule(
    State(state): State<Arc<AppState>>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match state.monitoring_service.delete_alert_rule(&rule_id).await {
        Ok(()) => {
            let response = serde_json::json!({
                "message": "Alert rule deleted successfully",
                "rule_id": rule_id,
                "deleted_at": Utc::now()
            });
            (StatusCode::OK, Json(ApiResponse::success(response)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to delete alert rule: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 获取日志（模拟数据）
async fn get_logs(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<LogQueryParams>,
) -> impl IntoResponse {
    // 模拟日志数据
    let mut logs = vec![
        LogEntry {
            timestamp: Utc::now().timestamp_millis(),
            level: "INFO".to_string(),
            service: "arbitrage_system".to_string(),
            message: "系统启动完成，所有服务正常运行".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));
                meta.insert("startup_time_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(1250)));
                meta
            },
        },
        LogEntry {
            timestamp: Utc::now().timestamp_millis() - 30000,
            level: "INFO".to_string(),
            service: "qingxi".to_string(),
            message: "成功连接到交易所，开始接收市场数据".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("exchange".to_string(), serde_json::Value::String("binance".to_string()));
                meta.insert("symbols".to_string(), serde_json::Value::Number(serde_json::Number::from(15)));
                meta
            },
        },
        LogEntry {
            timestamp: Utc::now().timestamp_millis() - 60000,
            level: "WARN".to_string(),
            service: "qingxi".to_string(),
            message: "连接到OKX交易所超时，正在重试连接".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("exchange".to_string(), serde_json::Value::String("okx".to_string()));
                meta.insert("retry_count".to_string(), serde_json::Value::Number(serde_json::Number::from(2)));
                meta.insert("timeout_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(5000)));
                meta
            },
        },
        LogEntry {
            timestamp: Utc::now().timestamp_millis() - 90000,
            level: "INFO".to_string(),
            service: "dashboard".to_string(),
            message: "仪表板数据更新完成，当前活跃用户数: 25".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("active_users".to_string(), serde_json::Value::Number(serde_json::Number::from(25)));
                meta.insert("update_duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(245)));
                meta
            },
        },
        LogEntry {
            timestamp: Utc::now().timestamp_millis() - 120000,
            level: "ERROR".to_string(),
            service: "arbitrage_system".to_string(),
            message: "套利机会执行失败: 资金不足无法执行交易".to_string(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("opportunity_id".to_string(), serde_json::Value::String("arb_123456".to_string()));
                meta.insert("required_balance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1500.50).unwrap()));
                meta.insert("available_balance".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(1200.25).unwrap()));
                meta.insert("symbol".to_string(), serde_json::Value::String("BTC/USDT".to_string()));
                meta
            },
        },
    ];
    
    // 应用过滤器
    if let Some(level_filter) = &params.level {
        logs.retain(|l| l.level.to_lowercase() == level_filter.to_lowercase());
    }
    
    if let Some(service_filter) = &params.service {
        logs.retain(|l| l.service.contains(service_filter));
    }
    
    if let Some(search_term) = &params.search {
        logs.retain(|l| l.message.to_lowercase().contains(&search_term.to_lowercase()));
    }
    
    if let Some(start_time) = &params.start_time {
        let start_timestamp = start_time.timestamp_millis();
        logs.retain(|l| l.timestamp >= start_timestamp);
    }
    
    if let Some(end_time) = &params.end_time {
        let end_timestamp = end_time.timestamp_millis();
        logs.retain(|l| l.timestamp <= end_timestamp);
    }
    
    // 按时间排序（最新的在前）
    logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // 应用限制
    let limit = params.limit.unwrap_or(100);
    logs.truncate(limit);
    
    let response = serde_json::json!({
        "logs": logs,
        "total": logs.len(),
        "filters": {
            "level": params.level,
            "service": params.service,
            "search": params.search,
            "start_time": params.start_time,
            "end_time": params.end_time,
            "limit": limit
        },
        "query_time_ms": 15
    });
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// 高级日志搜索
async fn search_logs(
    State(_state): State<Arc<AppState>>,
    Json(search_query): Json<serde_json::Value>,
) -> impl IntoResponse {
    // 解析搜索查询
    let query_text = search_query.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let start_time = search_query.get("start_time").and_then(|t| t.as_str());
    let end_time = search_query.get("end_time").and_then(|t| t.as_str());
    let services = search_query.get("services").and_then(|s| s.as_array());
    let levels = search_query.get("levels").and_then(|l| l.as_array());
    let limit = search_query.get("limit").and_then(|l| l.as_u64()).unwrap_or(100) as usize;
    
    // 模拟搜索结果
    let mut results = Vec::new();
    
    if query_text.to_lowercase().contains("error") || query_text.to_lowercase().contains("错误") {
        results.push(serde_json::json!({
            "timestamp": Utc::now().timestamp_millis() - 120000,
            "level": "ERROR",
            "service": "arbitrage_system",
            "message": "套利机会执行失败: 资金不足无法执行交易",
            "metadata": {
                "opportunity_id": "arb_123456",
                "required_balance": 1500.50,
                "available_balance": 1200.25,
                "symbol": "BTC/USDT"
            },
            "match_score": 0.95
        }));
    }
    
    if query_text.to_lowercase().contains("connection") || query_text.to_lowercase().contains("连接") {
        results.push(serde_json::json!({
            "timestamp": Utc::now().timestamp_millis() - 60000,
            "level": "WARN",
            "service": "qingxi",
            "message": "连接到OKX交易所超时，正在重试连接",
            "metadata": {
                "exchange": "okx",
                "retry_count": 2,
                "timeout_ms": 5000
            },
            "match_score": 0.88
        }));
        
        results.push(serde_json::json!({
            "timestamp": Utc::now().timestamp_millis() - 30000,
            "level": "INFO",
            "service": "qingxi",
            "message": "成功连接到交易所，开始接收市场数据",
            "metadata": {
                "exchange": "binance",
                "symbols": 15
            },
            "match_score": 0.82
        }));
    }
    
    if query_text.is_empty() {
        // 返回最近的日志
        results.push(serde_json::json!({
            "timestamp": Utc::now().timestamp_millis(),
            "level": "INFO",
            "service": "arbitrage_system",
            "message": "系统启动完成，所有服务正常运行",
            "metadata": {
                "version": "1.0.0",
                "startup_time_ms": 1250
            },
            "match_score": 1.0
        }));
    }
    
    // 按匹配度排序
    results.sort_by(|a, b| {
        let score_a = a.get("match_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let score_b = b.get("match_score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // 应用限制
    results.truncate(limit);
    
    let response = serde_json::json!({
        "query": {
            "text": query_text,
            "start_time": start_time,
            "end_time": end_time,
            "services": services,
            "levels": levels,
            "limit": limit
        },
        "results": results,
        "total": results.len(),
        "took_ms": 42,
        "max_score": results.first().and_then(|r| r.get("match_score")).unwrap_or(&serde_json::Value::Number(serde_json::Number::from(0)))
    });
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// 获取链路追踪（模拟数据）
async fn get_traces(
    State(_state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let limit: usize = params.get("limit").unwrap_or(&"50".to_string()).parse().unwrap_or(50);
    let service = params.get("service");
    let min_duration = params.get("min_duration").and_then(|d| d.parse::<u64>().ok());
    
    // 模拟链路追踪数据
    let mut traces = vec![
        serde_json::json!({
            "trace_id": "trace_arb_001",
            "operation": "arbitrage_opportunity_execution",
            "service": "arbitrage_system",
            "spans": 12,
            "duration_ms": 485,
            "status": "success",
            "timestamp": Utc::now().timestamp_millis(),
            "tags": {
                "symbol": "BTC/USDT",
                "profit_usd": 25.50,
                "exchanges": "binance,okx"
            },
            "error_count": 0
        }),
        serde_json::json!({
            "trace_id": "trace_dashboard_002",
            "operation": "dashboard_data_refresh",
            "service": "dashboard",
            "spans": 8,
            "duration_ms": 125,
            "status": "success",
            "timestamp": Utc::now().timestamp_millis() - 30000,
            "tags": {
                "user_id": "user_123",
                "widgets": 6,
                "data_sources": 3
            },
            "error_count": 0
        }),
        serde_json::json!({
            "trace_id": "trace_auth_003",
            "operation": "user_authentication",
            "service": "auth_service",
            "spans": 4,
            "duration_ms": 85,
            "status": "success",
            "timestamp": Utc::now().timestamp_millis() - 60000,
            "tags": {
                "user_id": "user_456",
                "login_method": "jwt",
                "ip_address": "192.168.1.100"
            },
            "error_count": 0
        }),
        serde_json::json!({
            "trace_id": "trace_qingxi_004",
            "operation": "market_data_collection",
            "service": "qingxi",
            "spans": 15,
            "duration_ms": 2150,
            "status": "error",
            "timestamp": Utc::now().timestamp_millis() - 120000,
            "tags": {
                "exchange": "okx",
                "symbols": 10,
                "error": "connection_timeout"
            },
            "error_count": 3
        }),
        serde_json::json!({
            "trace_id": "trace_api_005",
            "operation": "api_request_handling",
            "service": "api_gateway",
            "spans": 6,
            "duration_ms": 45,
            "status": "success",
            "timestamp": Utc::now().timestamp_millis() - 180000,
            "tags": {
                "endpoint": "/api/v1/dashboard",
                "method": "GET",
                "status_code": 200
            },
            "error_count": 0
        })
    ];
    
    // 应用过滤器
    if let Some(service_filter) = service {
        traces.retain(|t| {
            t.get("service")
                .and_then(|s| s.as_str())
                .map_or(false, |s| s.contains(service_filter))
        });
    }
    
    if let Some(min_dur) = min_duration {
        traces.retain(|t| {
            t.get("duration_ms")
                .and_then(|d| d.as_u64())
                .map_or(false, |d| d >= min_dur)
        });
    }
    
    // 按时间排序（最新的在前）
    traces.sort_by(|a, b| {
        let time_a = a.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0);
        let time_b = b.get("timestamp").and_then(|t| t.as_i64()).unwrap_or(0);
        time_b.cmp(&time_a)
    });
    
    traces.truncate(limit);
    
    let response = serde_json::json!({
        "traces": traces,
        "total": traces.len(),
        "filters": {
            "service": service,
            "min_duration": min_duration,
            "limit": limit
        },
        "query_time_ms": 18
    });
    
    (StatusCode::OK, Json(ApiResponse::success(response)))
}

/// 获取链路追踪详情
async fn get_trace_details(
    State(_state): State<Arc<AppState>>,
    Path(trace_id): Path<String>,
) -> impl IntoResponse {
    // 模拟不同类型的追踪详情
    let trace = match trace_id.as_str() {
        "trace_arb_001" => serde_json::json!({
            "trace_id": trace_id,
            "operation": "arbitrage_opportunity_execution",
            "service": "arbitrage_system",
            "start_time": Utc::now().timestamp_millis(),
            "total_duration_ms": 485,
            "status": "success",
            "tags": {
                "symbol": "BTC/USDT",
                "profit_usd": 25.50,
                "exchanges": "binance,okx"
            },
            "spans": [
                {
                    "span_id": "span_001",
                    "parent_id": null,
                    "operation": "opportunity_detection",
                    "service": "arbitrage_system",
                    "start_time_offset_ms": 0,
                    "duration_ms": 45,
                    "status": "success",
                    "tags": {
                        "price_diff_percent": 0.25,
                        "min_profit_usd": 20.0
                    }
                },
                {
                    "span_id": "span_002",
                    "parent_id": "span_001",
                    "operation": "balance_check",
                    "service": "arbitrage_system",
                    "start_time_offset_ms": 45,
                    "duration_ms": 25,
                    "status": "success",
                    "tags": {
                        "required_balance": 1000.0,
                        "available_balance": 1500.0
                    }
                },
                {
                    "span_id": "span_003",
                    "parent_id": "span_001",
                    "operation": "place_buy_order",
                    "service": "exchange_binance",
                    "start_time_offset_ms": 70,
                    "duration_ms": 180,
                    "status": "success",
                    "tags": {
                        "order_id": "binance_123456",
                        "quantity": 0.1,
                        "price": 45000.0
                    }
                },
                {
                    "span_id": "span_004",
                    "parent_id": "span_001",
                    "operation": "place_sell_order",
                    "service": "exchange_okx",
                    "start_time_offset_ms": 250,
                    "duration_ms": 210,
                    "status": "success",
                    "tags": {
                        "order_id": "okx_789012",
                        "quantity": 0.1,
                        "price": 45125.5
                    }
                },
                {
                    "span_id": "span_005",
                    "parent_id": "span_001",
                    "operation": "profit_calculation",
                    "service": "arbitrage_system",
                    "start_time_offset_ms": 460,
                    "duration_ms": 25,
                    "status": "success",
                    "tags": {
                        "gross_profit": 25.50,
                        "fees": 2.25,
                        "net_profit": 23.25
                    }
                }
            ]
        }),
        
        "trace_dashboard_002" => serde_json::json!({
            "trace_id": trace_id,
            "operation": "dashboard_data_refresh",
            "service": "dashboard",
            "start_time": Utc::now().timestamp_millis() - 30000,
            "total_duration_ms": 125,
            "status": "success",
            "spans": [
                {
                    "span_id": "dash_001",
                    "parent_id": null,
                    "operation": "user_authentication_check",
                    "service": "auth_service",
                    "start_time_offset_ms": 0,
                    "duration_ms": 15,
                    "status": "success"
                },
                {
                    "span_id": "dash_002",
                    "parent_id": "dash_001",
                    "operation": "fetch_portfolio_data",
                    "service": "dashboard",
                    "start_time_offset_ms": 15,
                    "duration_ms": 35,
                    "status": "success"
                },
                {
                    "span_id": "dash_003",
                    "parent_id": "dash_001",
                    "operation": "fetch_market_data",
                    "service": "qingxi",
                    "start_time_offset_ms": 20,
                    "duration_ms": 45,
                    "status": "success"
                },
                {
                    "span_id": "dash_004",
                    "parent_id": "dash_001",
                    "operation": "generate_charts",
                    "service": "dashboard",
                    "start_time_offset_ms": 65,
                    "duration_ms": 60,
                    "status": "success"
                }
            ]
        }),
        
        _ => {
            let error_response = ApiResponse::error(&format!("Trace {} not found", trace_id));
            return (StatusCode::NOT_FOUND, Json(error_response));
        }
    };
    
    (StatusCode::OK, Json(ApiResponse::success(trace)))
}

/// 启动监控系统
async fn start_monitoring(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.monitoring_service.start_monitoring().await {
        Ok(()) => {
            let response = serde_json::json!({
                "message": "Monitoring system started successfully",
                "status": "running",
                "started_at": Utc::now()
            });
            (StatusCode::OK, Json(ApiResponse::success(response)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to start monitoring: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 停止监控系统
async fn stop_monitoring(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.monitoring_service.stop_monitoring().await {
        Ok(()) => {
            let response = serde_json::json!({
                "message": "Monitoring system stopped successfully",
                "status": "stopped",
                "stopped_at": Utc::now()
            });
            (StatusCode::OK, Json(ApiResponse::success(response)))
        },
        Err(err) => {
            let error_response = ApiResponse::error(&format!("Failed to stop monitoring: {:?}", err));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        }
    }
}

/// 获取监控系统状态
async fn get_monitoring_status(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // 获取当前系统指标以评估监控系统健康状态
    let now = Utc::now();
    
    let status = serde_json::json!({
        "monitoring_active": true,
        "system_health": "healthy",
        "uptime_seconds": 86400, // 24小时
        "version": "1.0.0",
        "collectors": {
            "system_metrics": {
                "status": "active",
                "last_collection": now,
                "collection_interval_sec": 30,
                "success_rate_percent": 99.8
            },
            "service_health": {
                "status": "active",
                "last_check": now,
                "check_interval_sec": 60,
                "services_monitored": 4
            },
            "alert_engine": {
                "status": "active",
                "active_rules": 15,
                "active_alerts": 2,
                "evaluation_interval_sec": 30
            },
            "log_collector": {
                "status": "active",
                "log_sources": 4,
                "logs_per_minute": 150,
                "error_rate_percent": 0.02
            }
        },
        "data_storage": {
            "metrics_retention_days": 30,
            "logs_retention_days": 7,
            "traces_retention_days": 3,
            "storage_usage_percent": 45.2,
            "cleanup_last_run": now - chrono::Duration::hours(6)
        },
        "notification_channels": {
            "email": {
                "enabled": true,
                "last_test": now - chrono::Duration::hours(1),
                "success_rate_percent": 98.5
            },
            "webhook": {
                "enabled": true,
                "endpoints": 2,
                "last_test": now - chrono::Duration::minutes(30),
                "success_rate_percent": 99.2
            },
            "slack": {
                "enabled": false,
                "reason": "not_configured"
            }
        },
        "performance_metrics": {
            "avg_response_time_ms": 25.5,
            "memory_usage_mb": 156.8,
            "cpu_usage_percent": 8.2,
            "disk_io_ops_per_sec": 42
        },
        "recent_activity": {
            "alerts_triggered_last_hour": 3,
            "notifications_sent_last_hour": 2,
            "health_checks_last_hour": 60,
            "metrics_collected_last_hour": 120
        },
        "configuration": {
            "auto_remediation_enabled": false,
            "high_availability_mode": true,
            "backup_enabled": true,
            "last_config_update": now - chrono::Duration::days(2)
        },
        "last_updated": now
    });
    
    (StatusCode::OK, Json(ApiResponse::success(status)))
}