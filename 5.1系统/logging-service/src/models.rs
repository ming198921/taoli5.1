use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StandardResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ErrorInfo>,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: String,
    pub timestamp: i64,
    pub execution_time_ms: u64,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub service: String,
    pub version: String,
    pub instance_id: String,
}

// 添加缺失的类型定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub timestamp: i64,
    pub level: String,
    pub service: String,
    pub message: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct LogStreamQuery {
    pub level: Option<String>,
    pub service: Option<String>,
    pub module: Option<String>,
    pub search: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub modules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogAnalysis {
    pub anomalies: Vec<String>,
    pub patterns: Vec<String>,
    pub insights: Vec<String>,
}

impl<T> StandardResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: ResponseMetadata {
                request_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                execution_time_ms: 0,
                server_info: ServerInfo {
                    service: "logging-service".to_string(),
                    version: "1.0.0".to_string(),
                    instance_id: "instance-1".to_string(),
                },
            },
        }
    }

    pub fn error(code: String, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code,
                message,
                details: None,
            }),
            metadata: ResponseMetadata {
                request_id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().timestamp(),
                execution_time_ms: 0,
                server_info: ServerInfo {
                    service: "logging-service".to_string(),
                    version: "1.0.0".to_string(),
                    instance_id: "instance-1".to_string(),
                },
            },
        }
    }
} 