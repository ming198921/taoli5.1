use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub timestamp: DateTime<Utc>,
}

impl<T> StandardResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "Success".to_string(),
            data: Some(data),
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
            timestamp: Utc::now(),
        }
    }
}

// 配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub id: String,
    pub key: String,
    pub value: serde_json::Value,
    pub description: String,
    pub category: String,
    pub environment: String, // "development", "staging", "production"
    pub is_encrypted: bool,
    pub is_required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation_rules: Vec<ValidationRule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub updated_by: String,
    pub version: u32,
}

// 配置版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub id: String,
    pub config_id: String,
    pub version: u32,
    pub value: serde_json::Value,
    pub change_description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub is_rollback: bool,
    pub parent_version: Option<u32>,
}

// 热重载状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotReloadStatus {
    pub config_id: String,
    pub status: String, // "pending", "in_progress", "completed", "failed"
    pub last_reload_at: Option<DateTime<Utc>>,
    pub reload_duration_ms: Option<u64>,
    pub affected_services: Vec<String>,
    pub error_message: Option<String>,
    pub reload_count: u32,
}

// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: String, // "required", "min_length", "max_length", "pattern", "range"
    pub parameters: HashMap<String, serde_json::Value>,
    pub error_message: String,
}

// 配置模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub template_data: HashMap<String, ConfigTemplateItem>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 配置模板项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplateItem {
    pub key: String,
    pub data_type: String, // "string", "number", "boolean", "array", "object"
    pub default_value: Option<serde_json::Value>,
    pub description: String,
    pub is_required: bool,
    pub validation_rules: Vec<ValidationRule>,
}

// 配置快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub id: String,
    pub name: String,
    pub description: String,
    pub environment: String,
    pub configurations: Vec<Configuration>,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub is_automatic: bool,
    pub trigger_reason: String,
}

// 配置审计日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigAuditLog {
    pub id: String,
    pub config_id: String,
    pub action: String, // "create", "update", "delete", "rollback"
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub user_id: String,
    pub user_name: String,
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub change_reason: String,
}

// 配置权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPermission {
    pub id: String,
    pub user_id: String,
    pub role: String,
    pub permissions: Vec<String>, // "read", "write", "delete", "rollback"
    pub config_categories: Vec<String>,
    pub environments: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

// 配置依赖关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDependency {
    pub id: String,
    pub config_id: String,
    pub depends_on_config_id: String,
    pub dependency_type: String, // "required", "optional", "conditional"
    pub condition: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// 配置监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetrics {
    pub config_id: String,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub update_count: u32,
    pub last_updated: DateTime<Utc>,
    pub error_count: u32,
    pub last_error: Option<DateTime<Utc>>,
    pub validation_failures: u32,
    pub performance_metrics: ConfigPerformanceMetrics,
}

// 配置性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPerformanceMetrics {
    pub average_load_time_ms: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_kb: u64,
    pub network_latency_ms: f64,
}

// 配置通知设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigNotification {
    pub id: String,
    pub config_id: String,
    pub notification_type: String, // "email", "slack", "webhook"
    pub recipients: Vec<String>,
    pub events: Vec<String>, // "update", "error", "validation_failure"
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
}

// 配置加密设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEncryption {
    pub config_id: String,
    pub encryption_algorithm: String,
    pub key_id: String,
    pub is_encrypted: bool,
    pub encrypted_at: Option<DateTime<Utc>>,
    pub encryption_version: u32,
}

// 配置备份
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBackup {
    pub id: String,
    pub backup_name: String,
    pub environment: String,
    pub backup_data: Vec<Configuration>,
    pub created_at: DateTime<Utc>,
    pub backup_size_bytes: u64,
    pub compression_type: String,
    pub is_automatic: bool,
    pub retention_days: u32,
}

// 配置同步状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSyncStatus {
    pub source_environment: String,
    pub target_environment: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_status: String, // "pending", "in_progress", "completed", "failed"
    pub synced_configs_count: u32,
    pub failed_configs_count: u32,
    pub error_details: Option<String>,
}

// 配置中心状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCenterStatus {
    pub service_name: String,
    pub version: String,
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub total_configs: u32,
    pub active_connections: u32,
    pub cache_size_mb: f64,
    pub uptime_seconds: u64,
    pub last_health_check: DateTime<Utc>,
}

// 配置变更请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeRequest {
    pub id: String,
    pub config_id: String,
    pub requested_value: serde_json::Value,
    pub change_reason: String,
    pub requested_by: String,
    pub requested_at: DateTime<Utc>,
    pub status: String, // "pending", "approved", "rejected", "applied"
    pub approved_by: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
}

// 配置环境
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEnvironment {
    pub name: String,
    pub description: String,
    pub is_production: bool,
    pub config_count: u32,
    pub last_deployment: Option<DateTime<Utc>>,
    pub health_status: String,
    pub access_restrictions: Vec<String>,
}
