// Systemd服务控制器
// 提供通过HTTP API控制systemd服务的能力

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub service: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogsQuery {
    pub service: String,
    #[serde(default = "default_lines")]
    pub lines: usize,
}

fn default_lines() -> usize {
    100
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ControlResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub health: String,
    pub active_since: Option<String>,
    pub memory: Option<String>,
    pub cpu: Option<String>,
    pub pid: Option<u32>,
}

/// Systemd控制器
pub struct SystemdController {
    allowed_services: Vec<String>,
}

impl SystemdController {
    pub fn new() -> Self {
        Self {
            // 白名单：只允许控制这些服务
            allowed_services: vec![
                "arbitrage-system.service".to_string(),
                "arbitrage-qingxi.service".to_string(),
                "arbitrage-celue.service".to_string(),
                "arbitrage-risk.service".to_string(),
                "arbitrage-monitor.service".to_string(),
            ],
        }
    }

    /// 验证服务名是否在白名单中
    fn validate_service(&self, service: &str) -> Result<(), String> {
        if !self.allowed_services.contains(&service.to_string()) {
            return Err(format!("Service {} is not allowed", service));
        }
        Ok(())
    }

    /// 启动服务
    pub async fn start_service(&self, service: &str) -> ControlResponse {
        if let Err(e) = self.validate_service(service) {
            return ControlResponse {
                success: false,
                message: e,
                data: None,
            };
        }

        match Command::new("sudo")
            .args(["systemctl", "start", service])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    ControlResponse {
                        success: true,
                        message: format!("Service {} started successfully", service),
                        data: Some(serde_json::json!({
                            "service": service,
                            "action": "start",
                            "status": "success"
                        })),
                    }
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    ControlResponse {
                        success: false,
                        message: format!("Failed to start service: {}", error),
                        data: None,
                    }
                }
            }
            Err(e) => ControlResponse {
                success: false,
                message: format!("Failed to execute systemctl: {}", e),
                data: None,
            },
        }
    }

    /// 停止服务
    pub async fn stop_service(&self, service: &str) -> ControlResponse {
        if let Err(e) = self.validate_service(service) {
            return ControlResponse {
                success: false,
                message: e,
                data: None,
            };
        }

        match Command::new("sudo")
            .args(["systemctl", "stop", service])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    ControlResponse {
                        success: true,
                        message: format!("Service {} stopped successfully", service),
                        data: Some(serde_json::json!({
                            "service": service,
                            "action": "stop",
                            "status": "success"
                        })),
                    }
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    ControlResponse {
                        success: false,
                        message: format!("Failed to stop service: {}", error),
                        data: None,
                    }
                }
            }
            Err(e) => ControlResponse {
                success: false,
                message: format!("Failed to execute systemctl: {}", e),
                data: None,
            },
        }
    }

    /// 重启服务
    pub async fn restart_service(&self, service: &str) -> ControlResponse {
        if let Err(e) = self.validate_service(service) {
            return ControlResponse {
                success: false,
                message: e,
                data: None,
            };
        }

        match Command::new("sudo")
            .args(["systemctl", "restart", service])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    ControlResponse {
                        success: true,
                        message: format!("Service {} restarted successfully", service),
                        data: Some(serde_json::json!({
                            "service": service,
                            "action": "restart",
                            "status": "success"
                        })),
                    }
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    ControlResponse {
                        success: false,
                        message: format!("Failed to restart service: {}", error),
                        data: None,
                    }
                }
            }
            Err(e) => ControlResponse {
                success: false,
                message: format!("Failed to execute systemctl: {}", e),
                data: None,
            },
        }
    }

    /// 获取服务状态
    pub async fn get_service_status(&self, service: &str) -> Result<ServiceStatus, String> {
        if let Err(e) = self.validate_service(service) {
            return Err(e);
        }

        // 获取服务状态
        let status_output = Command::new("sudo")
            .args(["systemctl", "is-active", service])
            .output()
            .map_err(|e| format!("Failed to check service status: {}", e))?;

        let status = String::from_utf8_lossy(&status_output.stdout)
            .trim()
            .to_string();

        // 获取详细信息
        let show_output = Command::new("sudo")
            .args(["systemctl", "show", service, "--no-pager"])
            .output()
            .map_err(|e| format!("Failed to get service details: {}", e))?;

        let show_data = String::from_utf8_lossy(&show_output.stdout);
        
        // 解析输出
        let mut active_since = None;
        let mut memory = None;
        let mut cpu = None;
        let mut pid = None;

        for line in show_data.lines() {
            if let Some((key, value)) = line.split_once('=') {
                match key {
                    "ActiveEnterTimestamp" => {
                        active_since = Some(value.to_string());
                    }
                    "MemoryCurrent" => {
                        if value != "[not set]" {
                            memory = Some(value.to_string());
                        }
                    }
                    "CPUUsageNSec" => {
                        if value != "[not set]" {
                            cpu = Some(value.to_string());
                        }
                    }
                    "MainPID" => {
                        if let Ok(p) = value.parse::<u32>() {
                            if p > 0 {
                                pid = Some(p);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let health = match status.as_str() {
            "active" => "healthy",
            "inactive" => "stopped",
            "failed" => "unhealthy",
            _ => "unknown",
        };

        Ok(ServiceStatus {
            name: service.to_string(),
            status: status.clone(),
            health: health.to_string(),
            active_since,
            memory,
            cpu,
            pid,
        })
    }

    /// 获取服务日志
    pub async fn get_service_logs(&self, service: &str, lines: usize) -> Result<Vec<String>, String> {
        if let Err(e) = self.validate_service(service) {
            return Err(e);
        }

        let output = Command::new("sudo")
            .args([
                "journalctl",
                "-u",
                service,
                "-n",
                &lines.to_string(),
                "--no-pager",
                "--output",
                "short-iso",
            ])
            .output()
            .map_err(|e| format!("Failed to get logs: {}", e))?;

        if output.status.success() {
            let logs = String::from_utf8_lossy(&output.stdout);
            Ok(logs.lines().map(|s| s.to_string()).collect())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to get logs: {}", error))
        }
    }

    /// 重新加载systemd配置
    pub async fn reload_daemon(&self) -> ControlResponse {
        match Command::new("sudo")
            .args(["systemctl", "daemon-reload"])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    ControlResponse {
                        success: true,
                        message: "Systemd daemon reloaded successfully".to_string(),
                        data: None,
                    }
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    ControlResponse {
                        success: false,
                        message: format!("Failed to reload daemon: {}", error),
                        data: None,
                    }
                }
            }
            Err(e) => ControlResponse {
                success: false,
                message: format!("Failed to execute systemctl: {}", e),
                data: None,
            },
        }
    }
}

// HTTP处理函数
pub async fn start_handler(
    State(controller): State<Arc<SystemdController>>,
    Json(req): Json<ServiceRequest>,
) -> Json<ControlResponse> {
    Json(controller.start_service(&req.service).await)
}

pub async fn stop_handler(
    State(controller): State<Arc<SystemdController>>,
    Json(req): Json<ServiceRequest>,
) -> Json<ControlResponse> {
    Json(controller.stop_service(&req.service).await)
}

pub async fn restart_handler(
    State(controller): State<Arc<SystemdController>>,
    Json(req): Json<ServiceRequest>,
) -> Json<ControlResponse> {
    Json(controller.restart_service(&req.service).await)
}

pub async fn status_handler(
    State(controller): State<Arc<SystemdController>>,
    Query(req): Query<ServiceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match controller.get_service_status(&req.service).await {
        Ok(status) => Ok(Json(serde_json::json!({
            "success": true,
            "data": status
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": e
        }))),
    }
}

pub async fn logs_handler(
    State(controller): State<Arc<SystemdController>>,
    Query(query): Query<LogsQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match controller.get_service_logs(&query.service, query.lines).await {
        Ok(logs) => Ok(Json(serde_json::json!({
            "success": true,
            "logs": logs
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "message": e
        }))),
    }
}

pub async fn reload_handler(
    State(controller): State<Arc<SystemdController>>,
) -> Json<ControlResponse> {
    Json(controller.reload_daemon().await)
}

/// 创建systemd控制路由
pub fn create_systemd_routes() -> Router {
    let controller = Arc::new(SystemdController::new());
    
    Router::new()
        .route("/api/control/systemd/start", post(start_handler))
        .route("/api/control/systemd/stop", post(stop_handler))
        .route("/api/control/systemd/restart", post(restart_handler))
        .route("/api/control/systemd/status", get(status_handler))
        .route("/api/control/systemd/logs", get(logs_handler))
        .route("/api/control/systemd/reload", post(reload_handler))
        .with_state(controller)
}