#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

// AI模型状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub id: String,
    pub name: String,
    pub version: String,
    pub status: String, // "training", "ready", "deployed", "error"
    pub accuracy: f64,
    pub last_trained: DateTime<Utc>,
    pub training_progress: f64,
    pub model_type: String, // "risk_assessment", "price_prediction", "anomaly_detection"
    pub performance_metrics: ModelMetrics,
}

// 训练任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    pub id: String,
    pub model_id: String,
    pub status: String, // "pending", "running", "completed", "failed"
    pub progress: f64,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub dataset_size: usize,
    pub epochs: u32,
    pub learning_rate: f64,
    pub batch_size: u32,
}

// 预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    pub id: String,
    pub model_id: String,
    pub input_data: serde_json::Value,
    pub prediction: serde_json::Value,
    pub confidence: f64,
    pub timestamp: DateTime<Utc>,
    pub processing_time_ms: u64,
    pub model_version: String,
}

// 模型性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub accuracy: f64,
    pub precision: f64,
    pub recall: f64,
    pub f1_score: f64,
    pub auc_roc: f64,
    pub training_loss: f64,
    pub validation_loss: f64,
    pub inference_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_type: String,
    pub parameters: serde_json::Value,
    pub training_config: TrainingConfig,
    pub deployment_config: DeploymentConfig,
}

// 训练配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub epochs: u32,
    pub batch_size: u32,
    pub learning_rate: f64,
    pub optimizer: String,
    pub loss_function: String,
    pub validation_split: f64,
    pub early_stopping: bool,
    pub patience: u32,
}

// 部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub max_concurrent_requests: u32,
    pub timeout_seconds: u32,
    pub scaling_policy: String,
    pub resource_limits: ResourceLimits,
}

// 资源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: f64,
    pub memory_mb: u64,
    pub gpu_memory_mb: Option<u64>,
}

// 数据集信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub size: usize,
    pub features: Vec<String>,
    pub target: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub quality_score: f64,
}

// 模型版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub id: String,
    pub model_id: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub metrics: ModelMetrics,
    pub config: ModelConfig,
    pub is_active: bool,
}

// SHAP解释结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapExplanation {
    pub prediction_id: String,
    pub feature_importance: std::collections::HashMap<String, f64>,
    pub base_value: f64,
    pub expected_value: f64,
    pub shap_values: Vec<f64>,
    pub feature_names: Vec<String>,
}

// 模型监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringMetrics {
    pub model_id: String,
    pub timestamp: DateTime<Utc>,
    pub prediction_count: u64,
    pub average_confidence: f64,
    pub error_rate: f64,
    pub latency_p95_ms: f64,
    pub drift_score: f64,
    pub resource_usage: ResourceUsage,
}

// 资源使用情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f64,
    pub memory_mb: f64,
    pub gpu_memory_mb: Option<f64>,
    pub disk_io_mb: f64,
    pub network_io_mb: f64,
}
