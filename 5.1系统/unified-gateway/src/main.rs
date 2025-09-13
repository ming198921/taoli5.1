use axum::{
    extract::Path,
    http::{Method, StatusCode},
    response::Json,
    routing::{any, get, post},
    Router,
};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

mod system_control;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("unified_gateway=debug,tower_http=debug")
        .init();

    info!("🚀 Starting Unified Gateway v1.0.0");
    info!("📊 Managing 7 microservices with 402 total APIs");

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(gateway_status))
        
        // === 系统控制API ===
        .route("/api/system/status", get(system_status))
        .route("/api/system/start", post(system_control::system_start))
        .route("/api/system/stop", post(system_control::system_stop))
        .route("/api/system/restart", post(system_control::system_restart))
        .route("/api/system/emergency-stop", post(system_control::emergency_stop))
        .route("/api/system/maintenance/enable", post(system_control::enable_maintenance_mode))
        .route("/api/system/maintenance/disable", post(system_control::disable_maintenance_mode))
        .route("/api/system/backup/create", post(system_control::create_system_backup))
        .route("/api/system/backup/restore", post(system_control::restore_system_backup))
        .route("/api/system/diagnostics/run", post(system_control::run_system_diagnostics))
        .route("/api/system/health/deep-check", get(system_control::deep_health_check))
        
        // WebSocket升级端点
        .route("/ws/system/monitor", get(system_control::websocket_system_monitor))
        .route("/ws/system/logs", get(system_control::websocket_system_logs))
        
        // 通用代理路由
        .route("/api/*path", any(proxy_handler))
        .layer(CorsLayer::permissive());

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    info!("⚙️ Unified Gateway listening on http://0.0.0.0:3000");
    info!("✅ Gateway initialized - routing to 7 microservices");

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "unified-gateway",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now(),
        "microservices": 7,
        "total_apis": 402
    }))
}

async fn system_status() -> Json<Value> {
    let services = vec![
        ("logging-service", "4001", 45),
        ("cleaning-service", "4002", 52), 
        ("strategy-service", "4003", 38),
        ("performance-service", "4004", 67),
        ("trading-service", "4005", 56),
        ("ai-model-service", "4006", 48),
        ("config-service", "4007", 96),
    ];

    let mut healthy_services = 0;
    let mut service_status = Vec::new();
    
    for (name, port, apis) in services.iter() {
        let health_url = format!("http://localhost:{}/health", port);
        let is_healthy = match reqwest::get(&health_url).await {
            Ok(resp) if resp.status().is_success() => {
                healthy_services += 1;
                true
            }
            _ => false,
        };
        
        service_status.push(json!({
            "name": name,
            "port": port,
            "apis": apis,
            "status": if is_healthy { "running" } else { "stopped" },
            "health": if is_healthy { "healthy" } else { "unhealthy" },
            "health_check_url": health_url
        }));
    }

    Json(json!({
        "success": true,
        "data": {
            "uptime": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            "isRunning": healthy_services > 0,
            "services_healthy": healthy_services,
            "services_total": services.len(),
            "services": service_status,
            "system_health": if healthy_services == services.len() { "healthy" } else { "degraded" }
        },
        "timestamp": chrono::Utc::now()
    }))
}

async fn gateway_status() -> Json<Value> {
    let services = vec![
        ("logging-service", "4001", 45),
        ("cleaning-service", "4002", 52), 
        ("strategy-service", "4003", 38),
        ("performance-service", "4004", 67),
        ("trading-service", "4005", 56),
        ("ai-model-service", "4006", 48),
        ("config-service", "4007", 96),
    ];

    let mut service_status = Vec::new();
    for (name, port, apis) in services {
        let health_url = format!("http://localhost:{}/health", port);
        let status = match reqwest::get(&health_url).await {
            Ok(resp) if resp.status().is_success() => "healthy",
            _ => "unhealthy",
        };
        
        service_status.push(json!({
            "name": name,
            "port": port,
            "apis": apis,
            "status": status,
            "health_check_url": health_url
        }));
    }

    Json(json!({
        "gateway": "unified-gateway",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now(),
        "services": service_status,
        "total_apis": 402,
        "total_services": 7
    }))
}

async fn proxy_handler(
    method: Method,
    Path(path): Path<String>,
    body: String,
) -> Result<Json<Value>, StatusCode> {
    info!("🔄 Proxying request: {} /api/{}", method, path);
    
    // 根据路径路由到对应的微服务
    let (service_port, service_name) = match path.as_str() {
        // 日志服务路由
        p if p.starts_with("logs") || p.starts_with("logs/") => ("4001", "logging-service"),
        
        // 清洗服务路由  
        p if p.starts_with("cleaning") || p.starts_with("cleaning/") => ("4002", "cleaning-service"),
        
        // 策略服务路由
        p if p.starts_with("strategy") || p.starts_with("strategy/") ||
             p.starts_with("strategies") || p.starts_with("strategies/") ||
             p.starts_with("hotreload") || p.starts_with("hotreload/") => ("4003", "strategy-service"),
        
        // 性能服务路由
        p if p.starts_with("performance") || p.starts_with("performance/") => ("4004", "performance-service"),
        
        // 交易服务路由 - 支持多个路径前缀
        p if p.starts_with("orders") || p.starts_with("orders/") || 
             p.starts_with("positions") || p.starts_with("positions/") ||
             p.starts_with("funds") || p.starts_with("funds/") ||
             p.starts_with("risk") || p.starts_with("risk/") ||
             p.starts_with("fees") || p.starts_with("fees/") ||
             p.starts_with("exchange-api") || p.starts_with("exchange-api/") => ("4005", "trading-service"),
        
        // AI模型服务路由
        p if p.starts_with("models") || p.starts_with("models/") ||
             p.starts_with("training") || p.starts_with("training/") ||
             p.starts_with("inference") || p.starts_with("inference/") => ("4006", "ai-model-service"),
        
        // 配置服务路由
        p if p.starts_with("config") || p.starts_with("config/") => ("4007", "config-service"),
        
        _ => {
            warn!("❌ No route found for path: {}", path);
            return Err(StatusCode::NOT_FOUND);
        }
    };

    let target_url = format!("http://localhost:{}/api/{}", service_port, path);
    info!("🎯 Routing to: {} -> {}", service_name, target_url);
    
    // 转发请求到目标微服务
    let client = reqwest::Client::new();
    let request = match method {
        Method::GET => client.get(&target_url),
        Method::POST => client.post(&target_url)
            .header("Content-Type", "application/json")
            .body(body),
        Method::PUT => client.put(&target_url)
            .header("Content-Type", "application/json")
            .body(body),
        Method::DELETE => client.delete(&target_url),
        _ => return Err(StatusCode::METHOD_NOT_ALLOWED),
    };

    match request.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                match response.json::<Value>().await {
                    Ok(json_body) => {
                        info!("✅ Successfully proxied to {}", service_name);
                        Ok(Json(json_body))
                    },
                    Err(_) => {
                        warn!("❌ Failed to parse JSON response from {}", service_name);
                        Err(StatusCode::BAD_GATEWAY)
                    }
                }
            } else {
                warn!("❌ Service {} returned status: {}", service_name, status);
                Err(StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::BAD_GATEWAY))
            }
        }
        Err(e) => {
            warn!("❌ Failed to proxy request to {}: {}", service_name, e);
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}
