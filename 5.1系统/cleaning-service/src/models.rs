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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CleaningRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: String,
    pub conditions: Vec<String>,
    pub actions: Vec<String>,
    pub priority: i32,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleaningConfig {
    pub batch_size: usize,
    pub parallel_threads: usize,
    pub memory_limit_mb: usize,
    pub timeout_seconds: u64,
    pub enable_simd: bool,
}

impl Default for CleaningConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            parallel_threads: 8,
            memory_limit_mb: 512,
            timeout_seconds: 30,
            enable_simd: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct PerformanceMetrics {
    pub throughput_records_per_sec: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub error_rate: f64,
    pub average_processing_time_ms: f64,
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
                    service: "cleaning-service".to_string(),
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
                    service: "cleaning-service".to_string(),
                    version: "1.0.0".to_string(),
                    instance_id: "instance-1".to_string(),
                },
            },
        }
    }
} 