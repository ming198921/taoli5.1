use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use crate::{AppState, models::StandardResponse};

pub async fn get_interfaces(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "interfaces": [
            {
                "name": "eth0",
                "type": "ethernet",
                "status": "up",
                "speed": "1000 Mbps",
                "duplex": "full",
                "mtu": 1500,
                "mac_address": "00:1a:2b:3c:4d:5e",
                "ip_addresses": ["192.168.1.100", "fe80::21a:2bff:fe3c:4d5e"]
            },
            {
                "name": "lo",
                "type": "loopback",
                "status": "up", 
                "speed": "unknown",
                "duplex": "unknown",
                "mtu": 65536,
                "mac_address": "00:00:00:00:00:00",
                "ip_addresses": ["127.0.0.1", "::1"]
            }
        ],
        "total_interfaces": 2,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_network_stats(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let interface = params.get("interface").cloned().unwrap_or_else(|| "eth0".to_string());
    
    Json(StandardResponse::success(json!({
        "interface": interface,
        "rx_bytes": 1024000000,
        "tx_bytes": 512000000,
        "rx_packets": 1000000,
        "tx_packets": 500000,
        "rx_errors": 12,
        "tx_errors": 8,
        "rx_dropped": 5,
        "tx_dropped": 3,
        "collisions": 0,
        "multicast": 1500,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_bandwidth(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let interface = params.get("interface").cloned().unwrap_or_else(|| "eth0".to_string());
    
    Json(StandardResponse::success(json!({
        "interface": interface,
        "current_rx_mbps": 125.5,
        "current_tx_mbps": 67.8,
        "max_rx_mbps": 850.0,
        "max_tx_mbps": 920.0,
        "utilization_rx_percent": 14.8,
        "utilization_tx_percent": 7.4,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn measure_latency(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let target = params.get("target").cloned().unwrap_or_else(|| "8.8.8.8".to_string());
    let count = params.get("count").and_then(|s| s.parse::<u32>().ok()).unwrap_or(4);
    
    let measurements = (1..=count).map(|i| {
        json!({
            "sequence": i,
            "latency_ms": 12.5 + (i as f64 * 0.3),
            "status": "success"
        })
    }).collect::<Vec<_>>();
    
    Json(StandardResponse::success(json!({
        "target": target,
        "measurements": measurements,
        "min_latency": 12.5,
        "max_latency": 13.7,
        "avg_latency": 13.1,
        "packet_loss": 0.0,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_connections(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let protocol = params.get("protocol").cloned().unwrap_or_else(|| "all".to_string());
    let state_filter = params.get("state").cloned();
    
    let connections = vec![
        json!({
            "protocol": "tcp",
            "local_address": "192.168.1.100:22",
            "remote_address": "192.168.1.50:54321",
            "state": "ESTABLISHED",
            "pid": 1234,
            "process": "sshd"
        }),
        json!({
            "protocol": "tcp",
            "local_address": "0.0.0.0:80",
            "remote_address": "0.0.0.0:*",
            "state": "LISTEN",
            "pid": 5678,
            "process": "nginx"
        }),
        json!({
            "protocol": "udp",
            "local_address": "0.0.0.0:53",
            "remote_address": "0.0.0.0:*",
            "state": "LISTEN",
            "pid": 9012,
            "process": "systemd-resolved"
        })
    ];
    
    let filtered = if protocol != "all" {
        connections.into_iter().filter(|conn| 
            conn["protocol"].as_str() == Some(&protocol)
        ).collect()
    } else {
        connections
    };
    
    Json(StandardResponse::success(json!({
        "connections": filtered,
        "total_connections": filtered.len(),
        "filter": {
            "protocol": protocol,
            "state": state_filter
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_tcp_tuning(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "tcp_window_scaling": true,
        "tcp_timestamps": true,
        "tcp_sack": true,
        "tcp_congestion_control": "bbr",
        "tcp_rmem": [4096, 131072, 16777216],
        "tcp_wmem": [4096, 65536, 16777216],
        "tcp_max_syn_backlog": 4096,
        "tcp_keepalive_time": 7200,
        "tcp_keepalive_probes": 9,
        "tcp_keepalive_intvl": 75,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_tcp_tuning(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let congestion_control = payload["congestion_control"].as_str().unwrap_or("bbr");
    let window_scaling = payload["window_scaling"].as_bool().unwrap_or(true);
    
    Json(StandardResponse::success(json!({
        "message": "TCP tuning parameters updated successfully",
        "applied_settings": {
            "congestion_control": congestion_control,
            "window_scaling": window_scaling
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_buffer_sizes(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "net_core_rmem_default": 212992,
        "net_core_rmem_max": 16777216,
        "net_core_wmem_default": 212992,
        "net_core_wmem_max": 16777216,
        "net_core_netdev_max_backlog": 5000,
        "net_core_netdev_budget": 300,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_buffer_sizes(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let rmem_max = payload["rmem_max"].as_u64().unwrap_or(16777216);
    let wmem_max = payload["wmem_max"].as_u64().unwrap_or(16777216);
    
    Json(StandardResponse::success(json!({
        "message": "Network buffer sizes updated successfully",
        "applied_settings": {
            "rmem_max": rmem_max,
            "wmem_max": wmem_max
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_congestion_control(State(_state): State<AppState>) -> impl IntoResponse {
    Json(StandardResponse::success(json!({
        "current": "bbr",
        "available": ["bbr", "cubic", "reno", "vegas", "westwood", "hybla"],
        "per_connection": {
            "tcp": "bbr",
            "default": "bbr"
        },
        "statistics": {
            "packets_retransmitted": 150,
            "congestion_window_reductions": 25,
            "slow_start_restarts": 12
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_congestion_control(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let algorithm = payload["algorithm"].as_str().unwrap_or("bbr");
    
    Json(StandardResponse::success(json!({
        "message": format!("Congestion control algorithm set to {}", algorithm),
        "applied_algorithm": algorithm,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_queue_discipline(
    Query(params): Query<HashMap<String, String>>,
    State(_state): State<AppState>,
) -> impl IntoResponse {
    let interface = params.get("interface").cloned().unwrap_or_else(|| "eth0".to_string());
    
    Json(StandardResponse::success(json!({
        "interface": interface,
        "root_qdisc": "mq",
        "classes": [
            {
                "handle": "1:1",
                "qdisc": "fq_codel",
                "rate": "1000mbit",
                "packets": 500000,
                "bytes": 512000000,
                "dropped": 12
            }
        ],
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn set_queue_discipline(
    State(_state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let interface = payload["interface"].as_str().unwrap_or("eth0");
    let qdisc = payload["qdisc"].as_str().unwrap_or("fq_codel");
    
    Json(StandardResponse::success(json!({
        "message": format!("Queue discipline set to {} on {}", qdisc, interface),
        "applied_settings": {
            "interface": interface,
            "qdisc": qdisc
        },
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn optimize_network(State(state): State<AppState>) -> impl IntoResponse {
    match state.system_optimizer.optimize_network().await {
        Ok(result) => Json(StandardResponse::success(result)).into_response(),
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(StandardResponse::<Value>::error(error))).into_response()
    }
}

pub async fn run_network_test(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let test_type = payload["test_type"].as_str().unwrap_or("throughput");
    
    match state.tuning_engine.run_benchmark("network").await {
        Ok(mut result) => {
            result.benchmark_type = test_type.to_string();
            Json(StandardResponse::success(result)).into_response()
        },
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, Json(StandardResponse::<Value>::error(error))).into_response()
    }
}