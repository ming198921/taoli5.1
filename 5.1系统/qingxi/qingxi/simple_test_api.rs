// 独立的HTTP API测试服务器 - 简化版本
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use serde_json::json;
use std::{convert::Infallible, net::SocketAddr};
use tokio::time::{Duration, sleep};

fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let method = req.method().as_str();
    let path = req.uri().path();
    
    match (method, path) {
        ("GET", "/api/v1/health") => {
            let response = json!({
                "status": "healthy",
                "message": "Test API server is running",
                "timestamp": get_timestamp(),
                "exchanges": ["bybit", "gateio", "okx", "huobi"]
            });
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(response.to_string()))
                .unwrap())
        },
        
        ("GET", "/api/v1/exchanges") => {
            let response = json!({
                "exchanges": ["bybit", "gateio", "okx", "huobi"],
                "status": "active",
                "timestamp": get_timestamp(),
                "details": {
                    "bybit": {"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]},
                    "gateio": {"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]},
                    "okx": {"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]},
                    "huobi": {"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]}
                }
            });
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(response.to_string()))
                .unwrap())
        },
        
        ("GET", "/api/v1/symbols") => {
            let response = json!({
                "symbols": ["BTC/USDT", "ETH/USDT", "BNB/USDT", "XRP/USDT"],
                "count": 4,
                "timestamp": get_timestamp()
            });
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(response.to_string()))
                .unwrap())
        },

        ("GET", path) if path.starts_with("/api/v1/orderbook/") => {
            // 解析路径: /api/v1/orderbook/{exchange}/{symbol}
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 5 {
                let exchange = parts[4];
                let symbol = parts.get(5).unwrap_or("BTC");
                let symbol = if parts.len() >= 6 { 
                    format!("{}/{}", symbol, parts.get(6).unwrap_or("USDT"))
                } else {
                    format!("{}/USDT", symbol)
                };
                
                let response = json!({
                    "exchange": exchange,
                    "symbol": symbol,
                    "bids": [
                        ["43250.50", "0.15420"],
                        ["43250.00", "0.25130"],
                        ["43249.50", "0.08750"]
                    ],
                    "asks": [
                        ["43251.00", "0.12340"],
                        ["43251.50", "0.34560"],
                        ["43252.00", "0.18920"]
                    ],
                    "timestamp": get_timestamp(),
                    "sequence": 12345678
                });
                
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(response.to_string()))
                    .unwrap())
            } else {
                not_found()
            }
        },
        
        ("GET", "/api/v1/stats") => {
            let response = json!({
                "system_stats": {
                    "uptime": "active",
                    "total_messages_processed": 1250,
                    "active_sources": 4,
                    "avg_latency_ms": 15.5,
                    "data_quality": "excellent"
                },
                "performance": {
                    "throughput_msg_per_sec": 20.8,
                    "memory_usage": "optimal",
                    "cpu_usage": "normal"
                },
                "exchanges": {
                    "bybit": {"messages": 315, "latency_ms": 12.3},
                    "gateio": {"messages": 298, "latency_ms": 18.7},
                    "okx": {"messages": 332, "latency_ms": 14.1},
                    "huobi": {"messages": 305, "latency_ms": 16.9}
                },
                "timestamp": get_timestamp()
            });
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(response.to_string()))
                .unwrap())
        },

        ("GET", "/api/v1/stats/performance") => {
            let response = json!({
                "performance": {
                    "cpu_usage_percent": 12.5,
                    "memory_usage_mb": 245,
                    "network_io_kbps": 1024,
                    "cache_hit_rate": 0.95,
                    "avg_response_time_ms": 15.2
                },
                "throughput": {
                    "requests_per_second": 850,
                    "data_points_per_second": 12000,
                    "errors_per_minute": 2
                },
                "timestamp": get_timestamp()
            });
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(response.to_string()))
                .unwrap())
        },

        ("GET", "/api/v1/stats/connections") => {
            let response = json!({
                "connections": [
                    {"exchange": "bybit", "status": "Connected", "uptime_seconds": 3642},
                    {"exchange": "gateio", "status": "Connected", "uptime_seconds": 3598},
                    {"exchange": "okx", "status": "Connected", "uptime_seconds": 3701},
                    {"exchange": "huobi", "status": "Connected", "uptime_seconds": 3580}
                ],
                "total_count": 4,
                "connected_count": 4,
                "timestamp": get_timestamp()
            });
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(response.to_string()))
                .unwrap())
        },
        
        ("GET", "/") => {
            let api_docs = json!({
                "name": "Qingxi Market Data API - Test Server",
                "version": "1.0.0",
                "description": "Test HTTP API for 4-exchange market data testing",
                "endpoints": {
                    "health": "/api/v1/health",
                    "exchanges": "/api/v1/exchanges", 
                    "symbols": "/api/v1/symbols",
                    "orderbook": "/api/v1/orderbook/{exchange}/{symbol}",
                    "stats": "/api/v1/stats",
                    "stats_performance": "/api/v1/stats/performance",
                    "stats_connections": "/api/v1/stats/connections"
                },
                "examples": {
                    "health_check": "/api/v1/health",
                    "get_exchanges": "/api/v1/exchanges",
                    "get_orderbook": "/api/v1/orderbook/bybit/BTC/USDT",
                    "get_stats": "/api/v1/stats"
                },
                "supported_exchanges": ["bybit", "gateio", "okx", "huobi"],
                "test_features": [
                    "Real-time market data simulation",
                    "4-exchange concurrent data feeds",
                    "Dynamic API response testing",
                    "Performance metrics monitoring"
                ],
                "status": "operational"
            });
            
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(api_docs.to_string()))
                .unwrap())
        },
        
        _ => not_found(),
    }
}

fn not_found() -> Result<Response<Body>, Infallible> {
    let error = json!({
        "error": "Not Found",
        "message": "The requested resource was not found",
        "code": 404,
        "available_endpoints": [
            "/api/v1/health",
            "/api/v1/exchanges", 
            "/api/v1/symbols",
            "/api/v1/orderbook/{exchange}/{symbol}",
            "/api/v1/stats",
            "/api/v1/stats/performance",
            "/api/v1/stats/connections"
        ]
    });

    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("content-type", "application/json")
        .body(Body::from(error.to_string()))
        .unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Qingxi Test API Server");
    println!("📊 4-Exchange Market Data Test API");
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 50061));
    
    let make_svc = make_service_fn(|_conn| {
        async {
            Ok::<_, Infallible>(service_fn(handle_request))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    println!("🌐 HTTP API server listening on http://{}", addr);
    println!("📝 API Documentation: http://{}", addr);
    println!("🔍 Health Check: http://{}/api/v1/health", addr);
    println!("📊 Exchanges: http://{}/api/v1/exchanges", addr);
    println!("📈 Stats: http://{}/api/v1/stats", addr);
    
    // 启动后台任务模拟数据更新
    tokio::spawn(async {
        loop {
            sleep(Duration::from_secs(30)).await;
            println!("📊 [{}] Simulating data updates from 4 exchanges...", get_timestamp());
        }
    });

    if let Err(e) = server.await {
        eprintln!("❌ HTTP API server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
