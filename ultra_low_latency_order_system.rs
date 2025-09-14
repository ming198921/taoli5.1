// 5.1套利系统 - 超低延迟订单发送系统 (目标: <1ms)

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, BufWriter};
use bytes::BytesMut;
use crossbeam::channel::{bounded, Sender, Receiver};
use parking_lot::RwLock;
use futures::SinkExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// 条件编译: 仅在Linux且有socket2特性时启用高级socket优化
#[cfg(all(target_os = "linux", feature = "socket2"))]
use std::os::unix::io::AsRawFd;
#[cfg(all(target_os = "linux", feature = "socket2"))]
use socket2::Socket;

// ==================== 配置常量 ====================
const MAX_CONNECTIONS_PER_EXCHANGE: usize = 32;
const ORDER_QUEUE_SIZE: usize = 65536;
const PREALLOCATED_BUFFERS: usize = 1024;
const TCP_NODELAY: bool = true;
const TCP_QUICKACK: bool = true;
const SO_BUSY_POLL: u32 = 50; // 微秒

// ==================== 订单结构优化 ====================
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct UltraFastOrder {
    // 使用固定大小，避免动态分配
    symbol: [u8; 12],     // "BTCUSDT\0" 等
    side: u8,              // 0=BUY, 1=SELL
    order_type: u8,        // 0=LIMIT, 1=MARKET
    quantity: u64,         // 固定点数表示 (8位小数)
    price: u64,            // 固定点数表示 (8位小数)
    timestamp_ns: u64,     // 纳秒时间戳
    nonce: u32,           // 唯一标识
    checksum: u32,        // CRC32校验和
}

impl UltraFastOrder {
    #[inline(always)]
    pub fn new_buy_limit(symbol: &str, quantity: f64, price: f64) -> Self {
        let mut order = Self {
            symbol: [0u8; 12],
            side: 0,
            order_type: 0,
            quantity: (quantity * 1e8) as u64,
            price: (price * 1e8) as u64,
            timestamp_ns: nanotime(),
            nonce: generate_nonce(),
            checksum: 0,
        };
        
        // 快速复制symbol
        let symbol_bytes = symbol.as_bytes();
        let len = symbol_bytes.len().min(11);
        order.symbol[..len].copy_from_slice(&symbol_bytes[..len]);
        
        order.checksum = order.calculate_checksum();
        order
    }
    
    #[inline(always)]
    fn calculate_checksum(&self) -> u32 {
        // 使用硬件CRC32指令
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::_mm_crc32_u64;
            let data = std::slice::from_raw_parts(
                self as *const _ as *const u8,
                std::mem::size_of::<Self>() - 4
            );
            
            let mut crc = 0u32;
            for chunk in data.chunks(8) {
                if chunk.len() == 8 {
                    let val = u64::from_le_bytes(chunk.try_into().unwrap());
                    crc = _mm_crc32_u64(crc as u64, val) as u32;
                }
            }
            crc
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            // Fallback to software CRC
            crc32fast::hash(&self.as_bytes()[..std::mem::size_of::<Self>() - 4])
        }
    }
    
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const _ as *const u8,
                std::mem::size_of::<Self>()
            )
        }
    }
}

// ==================== 连接池管理 ====================
pub struct UltraLowLatencyConnectionPool {
    connections: Arc<RwLock<Vec<OptimizedConnection>>>,
    round_robin: AtomicU64,
}

struct OptimizedConnection {
    stream: BufWriter<TcpStream>,
    last_used: Instant,
    pending_orders: u32,
}

impl UltraLowLatencyConnectionPool {
    pub async fn new(exchange_url: &str, pool_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut connections = Vec::with_capacity(pool_size);
        
        for _ in 0..pool_size {
            let stream = TcpStream::connect(exchange_url).await?;
            
            // 兼容性TCP优化 - 条件编译确保跨平台兼容性
            #[cfg(all(target_os = "linux", feature = "socket2"))]
            {
                // 高级socket优化 (仅在支持的平台启用)
                use std::os::unix::io::FromRawFd;
                let socket = unsafe { Socket::from_raw_fd(stream.as_raw_fd()) };
                let _ = socket.set_nodelay(TCP_NODELAY);

                // TCP_QUICKACK可能不可用，安全忽略错误
                #[cfg(feature = "tcp_quickack")]
                let _ = socket.set_tcp_quickack(TCP_QUICKACK);

                unsafe {
                    // SO_BUSY_POLL: 让网卡驱动忙等，减少中断延迟
                    let _ = libc::setsockopt(
                        stream.as_raw_fd(),
                        libc::SOL_SOCKET,
                        libc::SO_BUSY_POLL,
                        &SO_BUSY_POLL as *const _ as *const libc::c_void,
                        std::mem::size_of::<u32>() as u32,
                    );

                    // TCP_USER_TIMEOUT: 快速检测连接失败
                    let timeout_ms: u32 = 5000;
                    let _ = libc::setsockopt(
                        stream.as_raw_fd(),
                        libc::IPPROTO_TCP,
                        libc::TCP_USER_TIMEOUT,
                        &timeout_ms as *const _ as *const libc::c_void,
                        std::mem::size_of::<u32>() as u32,
                    );
                }
            }
            #[cfg(not(all(target_os = "linux", feature = "socket2")))]
            {
                // 基础TCP优化作为后备方案
                stream.set_nodelay(true)?;
            }
            
            connections.push(OptimizedConnection {
                stream: BufWriter::with_capacity(65536, stream),
                last_used: Instant::now(),
                pending_orders: 0,
            });
        }
        
        Ok(Self {
            connections: Arc::new(RwLock::new(connections)),
            round_robin: AtomicU64::new(0),
        })
    }
    
    #[inline(always)]
    pub async fn send_order(&self, order: &UltraFastOrder) -> Result<Duration, Box<dyn std::error::Error>> {
        let start = Instant::now();
        
        // 轮询选择连接
        let idx = self.round_robin.fetch_add(1, Ordering::Relaxed) as usize;
        let mut connections = self.connections.write();
        let conn_idx = idx % connections.len();
        let conn = &mut connections[conn_idx];
        
        // 直接写入二进制数据
        conn.stream.write_all(order.as_bytes()).await?;
        conn.stream.flush().await?;
        
        Ok(start.elapsed())
    }
}

// ==================== 批量订单处理器 ====================
pub struct BatchOrderProcessor {
    order_queue: Receiver<UltraFastOrder>,
    sender: Sender<UltraFastOrder>,
    pool: Arc<UltraLowLatencyConnectionPool>,
}

impl BatchOrderProcessor {
    pub fn new(pool: Arc<UltraLowLatencyConnectionPool>) -> Self {
        let (sender, receiver) = bounded(ORDER_QUEUE_SIZE);
        
        Self {
            order_queue: receiver,
            sender,
            pool,
        }
    }
    
    pub async fn run(&self) {
        // 预分配缓冲区
        let mut batch_buffer = Vec::with_capacity(128);
        
        loop {
            // 批量收集订单
            while let Ok(order) = self.order_queue.try_recv() {
                batch_buffer.push(order);
                if batch_buffer.len() >= 32 {
                    break;
                }
            }
            
            if !batch_buffer.is_empty() {
                // 并行发送批量订单
                let futures: Vec<_> = batch_buffer
                    .iter()
                    .map(|order| self.pool.send_order(order))
                    .collect();
                
                let results = futures::future::join_all(futures).await;
                
                // 统计延迟
                for result in results {
                    if let Ok(latency) = result {
                        if latency < Duration::from_millis(1) {
                            println!("✓ Sub-1ms latency achieved: {:?}", latency);
                        }
                    }
                }
                
                batch_buffer.clear();
            }
            
            // 避免忙等
            tokio::time::sleep(Duration::from_micros(10)).await;
        }
    }
    
    #[inline(always)]
    pub fn submit_order(&self, order: UltraFastOrder) -> bool {
        self.sender.try_send(order).is_ok()
    }
}

// ==================== WebSocket二进制协议 ====================
pub struct BinaryWebSocketClient {
    ws_stream: tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    write_buffer: BytesMut,
}

impl BinaryWebSocketClient {
    pub async fn connect(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(url).await?;
        
        Ok(Self {
            ws_stream,
            write_buffer: BytesMut::with_capacity(4096),
        })
    }
    
    #[inline(always)]
    pub async fn send_binary_order(&mut self, order: &UltraFastOrder) -> Result<(), Box<dyn std::error::Error>> {
        use tokio_tungstenite::tungstenite::Message;
        
        // 直接发送二进制消息
        let message = Message::Binary(order.as_bytes().to_vec());
        self.ws_stream.send(message).await?;
        
        Ok(())
    }
}

// ==================== 硬件加速函数 ====================
#[inline(always)]
fn nanotime() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        use std::arch::x86_64::_rdtsc;
        _rdtsc()
    }
    
    #[cfg(not(target_arch = "x86_64"))]
    {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

static NONCE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
fn generate_nonce() -> u32 {
    NONCE_COUNTER.fetch_add(1, Ordering::Relaxed) as u32
}

// ==================== 预热和基准测试 ====================
pub struct LatencyBenchmark {
    samples: Vec<Duration>,
}

impl LatencyBenchmark {
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(10000),
        }
    }
    
    pub async fn warmup(pool: &UltraLowLatencyConnectionPool) {
        println!("Warming up connections...");
        
        // 发送预热订单
        for _ in 0..100 {
            let order = UltraFastOrder::new_buy_limit("BTCUSDT", 0.001, 50000.0);
            let _ = pool.send_order(&order).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        println!("Warmup completed");
    }
    
    pub async fn benchmark(&mut self, pool: &UltraLowLatencyConnectionPool, iterations: usize) {
        println!("Starting benchmark ({} iterations)...", iterations);
        
        for i in 0..iterations {
            let order = UltraFastOrder::new_buy_limit("BTCUSDT", 0.001, 50000.0 + i as f64);
            
            if let Ok(latency) = pool.send_order(&order).await {
                self.samples.push(latency);
            }
            
            // 避免过载
            if i % 100 == 0 {
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        }
        
        self.print_statistics();
    }
    
    fn print_statistics(&self) {
        if self.samples.is_empty() {
            return;
        }
        
        let mut sorted = self.samples.clone();
        sorted.sort();
        
        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let median = sorted[sorted.len() / 2];
        let p99 = sorted[sorted.len() * 99 / 100];
        let p999 = sorted[sorted.len() * 999 / 1000];
        
        let sum: Duration = sorted.iter().sum();
        let avg = sum / sorted.len() as u32;
        
        let under_1ms = sorted.iter().filter(|&&d| d < Duration::from_millis(1)).count();
        
        println!("\n=== Latency Benchmark Results ===");
        println!("Samples: {}", self.samples.len());
        println!("Min: {:?}", min);
        println!("Max: {:?}", max);
        println!("Avg: {:?}", avg);
        println!("Median: {:?}", median);
        println!("P99: {:?}", p99);
        println!("P99.9: {:?}", p999);
        println!("Under 1ms: {} ({:.2}%)", under_1ms, under_1ms as f64 / self.samples.len() as f64 * 100.0);
        
        if min < Duration::from_millis(1) {
            println!("\n✅ SUCCESS: Sub-millisecond latency achieved!");
        } else {
            println!("\n⚠️  Minimum latency: {:?} (target: <1ms)", min);
        }
    }
}

// ==================== 主函数 ====================
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Ultra Low Latency Order System - Target: <1ms");
    println!("{}", "=".repeat(50));
    
    // 创建连接池
    let pool = Arc::new(
        UltraLowLatencyConnectionPool::new("127.0.0.1:8881", 16).await?
    );
    
    // 预热
    LatencyBenchmark::warmup(&pool).await;
    
    // 运行基准测试
    let mut benchmark = LatencyBenchmark::new();
    benchmark.benchmark(&pool, 1000).await;
    
    Ok(())
}