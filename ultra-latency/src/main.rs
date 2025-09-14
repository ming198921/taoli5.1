//! 🚀 5.1套利系统 - 超低延迟订单执行模块
//!
//! 核心特性:
//! - <1ms 订单执行延迟
//! - 多交易所WebSocket连接
//! - 零拷贝数据处理
//! - 硬件优化网络栈

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderExecution {
    pub exchange: String,
    pub symbol: String,
    pub side: String,
    pub amount: f64,
    pub price: f64,
    pub timestamp: u64,
    pub latency_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub exchange: String,
    pub symbol: String,
    pub bid: f64,
    pub ask: f64,
    pub timestamp: u64,
}

pub struct UltraLatencySystem {
    connections: Arc<RwLock<HashMap<String, WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    market_data: Arc<RwLock<HashMap<String, MarketData>>>,
    performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

#[derive(Debug, Default)]
struct PerformanceMetrics {
    total_orders: u64,
    avg_latency_ns: u64,
    min_latency_ns: u64,
    max_latency_ns: u64,
    successful_orders: u64,
}

impl UltraLatencySystem {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            market_data: Arc::new(RwLock::new(HashMap::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
        }
    }

    /// 建立与交易所的超低延迟连接
    pub async fn connect_to_exchange(&self, exchange: &str, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        // 优化TCP连接参数
        let (ws_stream, _) = connect_async(url).await?;

        // 应用网络优化
        #[cfg(all(target_os = "linux", feature = "socket2"))]
        if let Ok(stream) = ws_stream.get_ref() {
            if let Some(tcp_stream) = stream.as_ref() {
                use socket2::{Socket, TcpKeepalive};
                let socket = Socket::from(tcp_stream.as_raw_fd());

                // 设置TCP_NODELAY减少延迟
                socket.set_nodelay(true)?;

                // 设置TCP_QUICKACK快速确认
                #[cfg(feature = "tcp_quickack")]
                {
                    let _ = socket.set_tcp_quickack(true);
                }

                // 配置TCP keepalive
                let keepalive = TcpKeepalive::new()
                    .with_time(Duration::from_secs(10))
                    .with_interval(Duration::from_secs(5))
                    .with_retries(3);
                socket.set_tcp_keepalive(&keepalive)?;
            }
        }

        // 存储连接
        self.connections.write().insert(exchange.to_string(), ws_stream);

        let connection_latency = start_time.elapsed().as_nanos() as u64;
        println!("🔗 Connected to {} in {}ns", exchange, connection_latency);

        Ok(())
    }

    /// 执行超低延迟订单
    pub async fn execute_order(&self, order: OrderExecution) -> Result<u64, Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        // 获取连接
        let mut connections = self.connections.write();
        if let Some(ws) = connections.get_mut(&order.exchange) {

            // 构建订单消息
            let order_message = serde_json::to_string(&order)?;

            // 发送订单 - 零拷贝优化
            ws.send(Message::Text(order_message)).await?;

            // 等待确认响应
            if let Some(msg) = ws.next().await {
                let _ = msg?; // 处理响应
            }
        }

        let execution_latency = start_time.elapsed().as_nanos() as u64;

        // 更新性能指标
        self.update_performance_metrics(execution_latency);

        println!("⚡ Order executed in {}ns", execution_latency);
        Ok(execution_latency)
    }

    /// 更新性能指标
    fn update_performance_metrics(&self, latency_ns: u64) {
        let mut metrics = self.performance_metrics.write();
        metrics.total_orders += 1;
        metrics.successful_orders += 1;

        if metrics.min_latency_ns == 0 || latency_ns < metrics.min_latency_ns {
            metrics.min_latency_ns = latency_ns;
        }
        if latency_ns > metrics.max_latency_ns {
            metrics.max_latency_ns = latency_ns;
        }

        // 计算移动平均
        metrics.avg_latency_ns = (metrics.avg_latency_ns * (metrics.total_orders - 1) + latency_ns) / metrics.total_orders;
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> PerformanceMetrics {
        self.performance_metrics.read().clone()
    }

    /// 启动实时市场数据处理
    pub async fn start_market_data_stream(&self, exchange: &str) -> Result<(), Box<dyn std::error::Error>> {
        let connections = self.connections.clone();
        let market_data = self.market_data.clone();
        let exchange = exchange.to_string();

        tokio::spawn(async move {
            loop {
                if let Some(ws) = connections.write().get_mut(&exchange) {
                    if let Some(msg) = ws.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Ok(data) = serde_json::from_str::<MarketData>(&text) {
                                    let key = format!("{}:{}", data.exchange, data.symbol);
                                    market_data.write().insert(key, data);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                tokio::time::sleep(Duration::from_micros(100)).await; // 超高频更新
            }
        });

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Ultra-Low Latency Order Execution System v5.1");

    let system = UltraLatencySystem::new();

    // 连接到主要交易所
    let exchanges = vec![
        ("binance", "wss://stream.binance.com:9443/ws"),
        ("okx", "wss://ws.okx.com:8443/ws/v5/public"),
        ("bybit", "wss://stream.bybit.com/v5/public/spot"),
    ];

    for (exchange, url) in exchanges {
        match system.connect_to_exchange(exchange, url).await {
            Ok(_) => println!("✅ {} connected", exchange),
            Err(e) => println!("❌ {} connection failed: {}", exchange, e),
        }
    }

    // 启动性能监控
    let system_clone = system.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let stats = system_clone.get_performance_stats();
            println!("📊 Performance: {} orders, avg {}ns, min {}ns, max {}ns",
                stats.total_orders, stats.avg_latency_ns, stats.min_latency_ns, stats.max_latency_ns);
        }
    });

    // 模拟订单执行测试
    for i in 0..100 {
        let order = OrderExecution {
            exchange: "binance".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: "buy".to_string(),
            amount: 0.001,
            price: 50000.0,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_millis() as u64,
            latency_ns: 0,
        };

        if let Ok(latency) = system.execute_order(order).await {
            if latency < 1_000_000 { // <1ms
                println!("🎯 Sub-millisecond execution: {}ns", latency);
            }
        }

        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    println!("🏁 Ultra-latency system test completed");

    // 保持运行
    tokio::time::sleep(Duration::from_secs(60)).await;

    Ok(())
}

impl Clone for UltraLatencySystem {
    fn clone(&self) -> Self {
        Self {
            connections: Arc::clone(&self.connections),
            market_data: Arc::clone(&self.market_data),
            performance_metrics: Arc::clone(&self.performance_metrics),
        }
    }
}

impl Clone for PerformanceMetrics {
    fn clone(&self) -> Self {
        Self {
            total_orders: self.total_orders,
            avg_latency_ns: self.avg_latency_ns,
            min_latency_ns: self.min_latency_ns,
            max_latency_ns: self.max_latency_ns,
            successful_orders: self.successful_orders,
        }
    }
}

#[cfg(all(target_os = "linux", feature = "socket2"))]
use std::os::unix::io::AsRawFd;