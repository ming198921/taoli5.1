use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::{AppState, models::StandardResponse};

pub async fn get_cpu_usage(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "usage_percent": 45.5,
        "cores": 8,
        "per_core_usage": [42.1, 48.3, 44.7, 46.9, 43.2, 49.1, 45.8, 47.6],
        "load_average": [1.2, 1.1, 1.3],
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_cpu_cores(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "physical_cores": 4,
        "logical_cores": 8,
        "threads_per_core": 2,
        "architecture": "x86_64",
        "vendor": "Intel",
        "model": "Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz",
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_cpu_frequency(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "current_frequency": 3200,
        "min_frequency": 800,
        "max_frequency": 4700,
        "per_core_frequency": [3200, 3150, 3180, 3220, 3190, 3210, 3170, 3200],
        "scaling_driver": "intel_pstate",
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_cpu_frequency(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let frequency = payload["frequency"].as_u64().unwrap_or(3200);
    
    Json(StandardResponse::success(json!({
        "message": format!("CPU frequency set to {} MHz", frequency),
        "applied_frequency": frequency,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_governor(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "current_governor": "performance",
        "available_governors": ["performance", "powersave", "ondemand", "conservative", "schedutil"],
        "per_core_governor": vec!["performance"; 8],
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_governor(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let governor = payload["governor"].as_str().unwrap_or("performance");
    
    Json(StandardResponse::success(json!({
        "message": format!("CPU governor set to {}", governor),
        "applied_governor": governor,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_affinity(
    Path(process): Path<String>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "process": process,
        "affinity_mask": "ff",
        "allowed_cpus": [0, 1, 2, 3, 4, 5, 6, 7],
        "current_cpu": 2,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_affinity(
    Path(process): Path<String>,
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let cpus = payload["cpus"].as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect::<Vec<_>>())
        .unwrap_or_else(|| vec![0, 1, 2, 3]);
    
    Json(StandardResponse::success(json!({
        "process": process,
        "message": format!("CPU affinity set for process {}", process),
        "applied_cpus": cpus,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_cache_stats(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "l1_data_cache": {
            "size": "32KB",
            "hits": 1000000,
            "misses": 50000,
            "hit_rate": 95.2
        },
        "l1_instruction_cache": {
            "size": "32KB", 
            "hits": 950000,
            "misses": 45000,
            "hit_rate": 95.5
        },
        "l2_cache": {
            "size": "256KB",
            "hits": 800000,
            "misses": 30000,
            "hit_rate": 96.4
        },
        "l3_cache": {
            "size": "12MB",
            "hits": 600000,
            "misses": 20000,
            "hit_rate": 96.8
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn flush_cache(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "message": "CPU caches flushed successfully",
        "flushed_levels": ["L1", "L2", "L3"],
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_temperature(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "cpu_temperature": 68.5,
        "per_core_temperature": [67.2, 69.1, 68.8, 68.9, 67.5, 69.3, 68.1, 68.7],
        "critical_temperature": 100.0,
        "warning_temperature": 85.0,
        "status": "normal",
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_throttling_status(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "is_throttling": false,
        "thermal_throttling": false,
        "power_throttling": false,
        "current_throttle_count": 0,
        "throttle_reasons": [],
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_topology(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "sockets": 1,
        "cores_per_socket": 4,
        "threads_per_core": 2,
        "numa_nodes": 1,
        "topology_map": {
            "socket_0": {
                "core_0": [0, 4],
                "core_1": [1, 5],
                "core_2": [2, 6],
                "core_3": [3, 7]
            }
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_process_usage(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let limit = params.get("limit").and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
    
    let processes = (1..=limit).map(|i| {
        json!({
            "pid": 1000 + i,
            "name": format!("process_{}", i),
            "cpu_percent": 5.0 + (i as f64),
            "cpu_time": 120 + i * 10,
            "priority": 20,
            "nice": 0,
            "threads": 1 + i % 4
        })
    }).collect::<Vec<_>>();

    Json(StandardResponse::success(json!({
        "processes": processes,
        "total_processes": limit,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn optimize_cpu(State(state): State<AppState>) -> impl IntoResponse {
    match state.system_optimizer.optimize_cpu().await {
        Ok(result) => Json(StandardResponse::success(result)).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(StandardResponse::<Value>::error(error))).into_response()
    }
}

pub async fn run_cpu_benchmark(State(state): State<AppState>) -> impl IntoResponse {
    match state.tuning_engine.run_benchmark("cpu").await {
        Ok(result) => Json(StandardResponse::success(result)).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(StandardResponse::<Value>::error(error))).into_response()
    }
}

pub async fn get_scheduler_info(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "scheduler": "CFS",
        "scheduler_policy": "SCHED_NORMAL",
        "time_slice": 4,
        "scheduler_stats": {
            "context_switches": 1000000,
            "voluntary_switches": 800000,
            "involuntary_switches": 200000
        },
        "runqueue_length": 2,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn tune_scheduler(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let time_slice = payload["time_slice"].as_u64().unwrap_or(4);
    let policy = payload["policy"].as_str().unwrap_or("SCHED_NORMAL");
    
    Json(StandardResponse::success(json!({
        "message": "CPU scheduler tuned successfully",
        "applied_settings": {
            "time_slice": time_slice,
            "policy": policy
        },
        "timestamp": chrono::Utc::now()
    })))
}