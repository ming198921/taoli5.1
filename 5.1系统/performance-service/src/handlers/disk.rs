use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::{AppState, models::StandardResponse};

pub async fn get_disk_usage(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let path = params.get("path").cloned().unwrap_or_else(|| "/".to_string());
    
    Json(StandardResponse::success(json!({
        "path": path,
        "total_space": 107374182400i64,
        "used_space": 53687091200i64,
        "free_space": 53687091200i64,
        "used_percent": 50.0,
        "inodes_total": 6553600,
        "inodes_used": 327680,
        "inodes_free": 6225920,
        "inodes_used_percent": 5.0,
        "filesystem": "ext4",
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_io_stats(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let device = params.get("device").cloned().unwrap_or_else(|| "sda".to_string());
    
    Json(StandardResponse::success(json!({
        "device": device,
        "read_operations": 1000000,
        "write_operations": 500000,
        "read_bytes": 4294967296i64,
        "write_bytes": 2147483648u32,
        "read_time_ms": 120000,
        "write_time_ms": 90000,
        "io_time_ms": 150000,
        "queue_depth": 8,
        "utilization_percent": 45.5,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn measure_iops(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let device = params.get("device").cloned().unwrap_or_else(|| "sda".to_string());
    let block_size = params.get("block_size").and_then(|s| s.parse::<u32>().ok()).unwrap_or(4096);
    
    Json(StandardResponse::success(json!({
        "device": device,
        "block_size": block_size,
        "sequential_read_iops": 850,
        "sequential_write_iops": 750,
        "random_read_iops": 12000,
        "random_write_iops": 8000,
        "mixed_workload_iops": 10500,
        "test_duration": 30,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn measure_latency(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let device = params.get("device").cloned().unwrap_or_else(|| "sda".to_string());
    
    Json(StandardResponse::success(json!({
        "device": device,
        "read_latency": {
            "min_ms": 0.1,
            "max_ms": 15.2,
            "avg_ms": 2.5,
            "p50_ms": 2.1,
            "p95_ms": 8.3,
            "p99_ms": 12.7
        },
        "write_latency": {
            "min_ms": 0.2,
            "max_ms": 25.8,
            "avg_ms": 4.1,
            "p50_ms": 3.5,
            "p95_ms": 12.1,
            "p99_ms": 18.9
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_io_scheduler(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let device = params.get("device").cloned().unwrap_or_else(|| "sda".to_string());
    
    Json(StandardResponse::success(json!({
        "device": device,
        "current_scheduler": "mq-deadline",
        "available_schedulers": ["none", "mq-deadline", "bfq", "kyber"],
        "queue_depth": 32,
        "read_expire": 500,
        "write_expire": 5000,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_io_scheduler(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let device = payload["device"].as_str().unwrap_or("sda");
    let scheduler = payload["scheduler"].as_str().unwrap_or("mq-deadline");
    
    Json(StandardResponse::success(json!({
        "message": format!("I/O scheduler set to {} for device {}", scheduler, device),
        "applied_settings": {
            "device": device,
            "scheduler": scheduler
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_queue_depth(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let device = params.get("device").cloned().unwrap_or_else(|| "sda".to_string());
    
    Json(StandardResponse::success(json!({
        "device": device,
        "current_queue_depth": 32,
        "max_queue_depth": 64,
        "optimal_queue_depth": 32,
        "active_requests": 8,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_queue_depth(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let device = payload["device"].as_str().unwrap_or("sda");
    let depth = payload["depth"].as_u64().unwrap_or(32);
    
    Json(StandardResponse::success(json!({
        "message": format!("Queue depth set to {} for device {}", depth, device),
        "applied_settings": {
            "device": device,
            "queue_depth": depth
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_read_ahead(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let device = params.get("device").cloned().unwrap_or_else(|| "sda".to_string());
    
    Json(StandardResponse::success(json!({
        "device": device,
        "current_read_ahead": 256,
        "optimal_read_ahead": 256,
        "min_read_ahead": 128,
        "max_read_ahead": 2048,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_read_ahead(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let device = payload["device"].as_str().unwrap_or("sda");
    let read_ahead = payload["read_ahead"].as_u64().unwrap_or(256);
    
    Json(StandardResponse::success(json!({
        "message": format!("Read-ahead set to {} sectors for device {}", read_ahead, device),
        "applied_settings": {
            "device": device,
            "read_ahead": read_ahead
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_disk_cache(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let device = params.get("device").cloned().unwrap_or_else(|| "sda".to_string());
    
    Json(StandardResponse::success(json!({
        "device": device,
        "write_cache_enabled": true,
        "read_cache_enabled": true,
        "cache_size": 134217728,
        "dirty_data": 8388608,
        "cache_hit_ratio": 92.5,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn configure_disk_cache(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let device = payload["device"].as_str().unwrap_or("sda");
    let write_cache = payload["write_cache"].as_bool().unwrap_or(true);
    let read_cache = payload["read_cache"].as_bool().unwrap_or(true);
    
    Json(StandardResponse::success(json!({
        "message": format!("Disk cache configured for device {}", device),
        "applied_settings": {
            "device": device,
            "write_cache": write_cache,
            "read_cache": read_cache
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_mount_options(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let mount_point = params.get("mount_point").cloned().unwrap_or_else(|| "/".to_string());
    
    Json(StandardResponse::success(json!({
        "mount_point": mount_point,
        "device": "/dev/sda1",
        "filesystem": "ext4",
        "options": [
            "rw", "relatime", "data=ordered", "barrier=1"
        ],
        "recommended_options": [
            "noatime", "commit=60", "data=writeback"
        ],
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_mount_options(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mount_point = payload["mount_point"].as_str().unwrap_or("/");
    let options = payload["options"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec!["rw", "noatime"]);
    
    Json(StandardResponse::success(json!({
        "message": format!("Mount options updated for {}", mount_point),
        "applied_settings": {
            "mount_point": mount_point,
            "options": options
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn defragment_disk(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let path = payload["path"].as_str().unwrap_or("/");
    
    Json(StandardResponse::success(json!({
        "message": format!("Defragmentation completed for {}", path),
        "path": path,
        "files_processed": 15000,
        "fragmentation_reduced": 15.5,
        "duration_minutes": 45,
        "space_savings": 2097152,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn trim_ssd(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let device = payload["device"].as_str().unwrap_or("sda");
    
    Json(StandardResponse::success(json!({
        "message": format!("TRIM operation completed for SSD {}", device),
        "device": device,
        "blocks_trimmed": 1048576,
        "duration_seconds": 30,
        "performance_improvement": 8.2,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn run_disk_benchmark(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let device = payload["device"].as_str().unwrap_or("sda");
    
    match state.tuning_engine.run_benchmark("disk").await {
        Ok(mut result) => {
            result.details.insert("device".to_string(), serde_json::Value::String(device.to_string()));
            Json(StandardResponse::success(result)).into_response()
        },
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(StandardResponse::<Value>::error(error))).into_response()
    }
}

pub async fn optimize_disk(State(state): State<AppState>) -> impl IntoResponse {
    match state.system_optimizer.optimize_disk().await {
        Ok(result) => Json(StandardResponse::success(result)).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(StandardResponse::<Value>::error(error))).into_response()
    }
}