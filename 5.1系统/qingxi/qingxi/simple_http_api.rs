use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let listener = TcpListener::bind("0.0.0.0:50061").unwrap();
    println!("🚀 Starting Qingxi Test API Server");
    println!("📊 4-Exchange Market Data Test API");
    println!("🌐 HTTP API server listening on http://0.0.0.0:50061");
    println!("📝 API Documentation: http://0.0.0.0:50061");
    println!("🔍 Health Check: http://0.0.0.0:50061/api/v1/health");
    println!("📊 Exchanges: http://0.0.0.0:50061/api/v1/exchanges");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let lines: Vec<&str> = request.lines().collect();
    
    if lines.is_empty() {
        return;
    }

    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return;
    }

    let method = parts[0];
    let path = parts[1];

    let response = match (method, path) {
        ("GET", "/api/v1/health") => {
            let timestamp = get_timestamp();
            format!(
                r#"{{
    "status": "healthy",
    "message": "Test API server is running",
    "timestamp": {},
    "exchanges": ["bybit", "gateio", "okx", "huobi"]
}}"#,
                timestamp
            )
        },
        
        ("GET", "/api/v1/exchanges") => {
            let timestamp = get_timestamp();
            format!(
                r#"{{
    "exchanges": ["bybit", "gateio", "okx", "huobi"],
    "status": "active",
    "timestamp": {},
    "details": {{
        "bybit": {{"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]}},
        "gateio": {{"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]}},
        "okx": {{"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]}},
        "huobi": {{"status": "connected", "symbols": ["BTC/USDT", "ETH/USDT"]}}
    }}
}}"#,
                timestamp
            )
        },
        
        ("GET", "/api/v1/symbols") => {
            let timestamp = get_timestamp();
            format!(
                r#"{{
    "symbols": ["BTC/USDT", "ETH/USDT", "BNB/USDT", "XRP/USDT"],
    "count": 4,
    "timestamp": {}
}}"#,
                timestamp
            )
        },

        ("GET", path) if path.starts_with("/api/v1/orderbook/") => {
            let parts: Vec<&str> = path.split('/').collect();
            let exchange = parts.get(4).unwrap_or(&"unknown");
            let symbol = if parts.len() >= 6 {
                format!("{}/{}", parts.get(5).unwrap_or(&"BTC"), parts.get(6).unwrap_or(&"USDT"))
            } else {
                "BTC/USDT".to_string()
            };
            let timestamp = get_timestamp();
            
            format!(
                r#"{{
    "exchange": "{}",
    "symbol": "{}",
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
    "timestamp": {},
    "sequence": 12345678
}}"#,
                exchange, symbol, timestamp
            )
        },
        
        ("GET", "/api/v1/stats") => {
            let timestamp = get_timestamp();
            format!(
                r#"{{
    "system_stats": {{
        "uptime": "active",
        "total_messages_processed": 1250,
        "active_sources": 4,
        "avg_latency_ms": 15.5,
        "data_quality": "excellent"
    }},
    "performance": {{
        "throughput_msg_per_sec": 20.8,
        "memory_usage": "optimal",
        "cpu_usage": "normal"
    }},
    "exchanges": {{
        "bybit": {{"messages": 315, "latency_ms": 12.3}},
        "gateio": {{"messages": 298, "latency_ms": 18.7}},
        "okx": {{"messages": 332, "latency_ms": 14.1}},
        "huobi": {{"messages": 305, "latency_ms": 16.9}}
    }},
    "timestamp": {}
}}"#,
                timestamp
            )
        },

        ("GET", "/api/v1/stats/performance") => {
            let timestamp = get_timestamp();
            format!(
                r#"{{
    "performance": {{
        "cpu_usage_percent": 12.5,
        "memory_usage_mb": 245,
        "network_io_kbps": 1024,
        "cache_hit_rate": 0.95,
        "avg_response_time_ms": 15.2
    }},
    "throughput": {{
        "requests_per_second": 850,
        "data_points_per_second": 12000,
        "errors_per_minute": 2
    }},
    "timestamp": {}
}}"#,
                timestamp
            )
        },

        ("GET", "/api/v1/stats/connections") => {
            let timestamp = get_timestamp();
            format!(
                r#"{{
    "connections": [
        {{"exchange": "bybit", "status": "Connected", "uptime_seconds": 3642}},
        {{"exchange": "gateio", "status": "Connected", "uptime_seconds": 3598}},
        {{"exchange": "okx", "status": "Connected", "uptime_seconds": 3701}},
        {{"exchange": "huobi", "status": "Connected", "uptime_seconds": 3580}}
    ],
    "total_count": 4,
    "connected_count": 4,
    "timestamp": {}
}}"#,
                timestamp
            )
        },
        
        ("GET", "/") => {
            format!(
                r#"{{
    "name": "Qingxi Market Data API - Test Server",
    "version": "1.0.0",
    "description": "Test HTTP API for 4-exchange market data testing",
    "endpoints": {{
        "health": "/api/v1/health",
        "exchanges": "/api/v1/exchanges", 
        "symbols": "/api/v1/symbols",
        "orderbook": "/api/v1/orderbook/{{exchange}}/{{symbol}}",
        "stats": "/api/v1/stats",
        "stats_performance": "/api/v1/stats/performance",
        "stats_connections": "/api/v1/stats/connections"
    }},
    "examples": {{
        "health_check": "/api/v1/health",
        "get_exchanges": "/api/v1/exchanges",
        "get_orderbook": "/api/v1/orderbook/bybit/BTC/USDT",
        "get_stats": "/api/v1/stats"
    }},
    "supported_exchanges": ["bybit", "gateio", "okx", "huobi"],
    "test_features": [
        "Real-time market data simulation",
        "4-exchange concurrent data feeds",
        "Dynamic API response testing",
        "Performance metrics monitoring"
    ],
    "status": "operational"
}}"#
            )
        },
        
        _ => {
            format!(
                r#"{{
    "error": "Not Found",
    "message": "The requested resource was not found",
    "code": 404,
    "available_endpoints": [
        "/api/v1/health",
        "/api/v1/exchanges", 
        "/api/v1/symbols",
        "/api/v1/orderbook/{{exchange}}/{{symbol}}",
        "/api/v1/stats",
        "/api/v1/stats/performance",
        "/api/v1/stats/connections"
    ]
}}"#
            )
        }
    };

    let status_code = if path.starts_with("/api/v1/") || path == "/" {
        "200 OK"
    } else {
        "404 Not Found"
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        status_code,
        response.len(),
        response
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
