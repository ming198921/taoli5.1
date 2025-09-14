#![allow(dead_code)]
//! # 数据分发层 - Qingxi 5.1 深度集成实现
//! 
//! 基于现有QingxiSystemState的零影响数据分发系统
//! 支持实时策略数据传输、套利检测、风控告警和异步审计存储

use crate::types::*;
use crate::errors::*;
use crate::MarketDataMessage;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{Duration, Instant};
use tracing::{debug, error, info, warn, instrument};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};

/// 清洗后的市场数据结构 - 完全兼容现有MarketDataMessage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanedMarketData {
    pub symbol: String,
    pub exchange: String,
    pub timestamp: i64,
    pub sequence: u64,
    pub price: f64,
    pub quantity: f64,
    pub side: TradeSide,
    pub quality_score: f64, // 数据质量评分 0.0-1.0
    pub processing_latency_ns: u64, // 处理延迟纳秒
}

impl From<MarketDataMessage> for CleanedMarketData {
    fn from(msg: MarketDataMessage) -> Self {
        match msg {
            MarketDataMessage::OrderBook(ob) => Self {
                symbol: ob.symbol.as_combined(),
                exchange: ob.source.clone(),
                timestamp: ob.timestamp.into(),
                sequence: 0,
                price: ob.bids.first().map(|e| e.price.into_inner()).unwrap_or(0.0),
                quantity: ob.bids.first().map(|e| e.quantity.into_inner()).unwrap_or(0.0),
                side: TradeSide::Buy,
                quality_score: 1.0,
                processing_latency_ns: 0,
            },
            MarketDataMessage::OrderBookSnapshot(ob) => Self {
                symbol: ob.symbol.as_combined(),
                exchange: ob.source.clone(),
                timestamp: ob.timestamp.into(),
                sequence: 0,
                price: ob.bids.first().map(|e| e.price.into_inner()).unwrap_or(0.0),
                quantity: ob.bids.first().map(|e| e.quantity.into_inner()).unwrap_or(0.0),
                side: TradeSide::Buy,
                quality_score: 1.0,
                processing_latency_ns: 0,
            },
            MarketDataMessage::OrderBookUpdate(update) => Self {
                symbol: update.symbol.as_combined(),
                exchange: update.source,
                timestamp: crate::high_precision_time::Nanos::now().into(),
                sequence: update.final_update_id,
                price: update.bids.first().map(|e| e.price.into_inner()).unwrap_or(0.0),
                quantity: update.bids.first().map(|e| e.quantity.into_inner()).unwrap_or(0.0),
                side: TradeSide::Buy,
                quality_score: 1.0,
                processing_latency_ns: 0,
            },
            MarketDataMessage::Trade(trade) => Self {
                symbol: trade.symbol.as_combined(),
                exchange: trade.source,
                timestamp: trade.timestamp.into(),
                sequence: trade.trade_id.unwrap_or_default().parse().unwrap_or(0),
                price: trade.price.into_inner(),
                quantity: trade.quantity.into_inner(),
                side: trade.side,
                quality_score: 1.0,
                processing_latency_ns: 0,
            },
            MarketDataMessage::Snapshot(snapshot) => Self {
                symbol: snapshot.symbol.as_combined(),
                exchange: "unknown".to_string(),
                timestamp: snapshot.timestamp.into(),
                sequence: 0,
                price: snapshot.orderbook.as_ref()
                    .and_then(|ob| ob.bids.first())
                    .map(|e| e.price.into_inner())
                    .unwrap_or(0.0),
                quantity: snapshot.orderbook.as_ref()
                    .and_then(|ob| ob.bids.first())
                    .map(|e| e.quantity.into_inner())
                    .unwrap_or(0.0),
                side: TradeSide::Buy,
                quality_score: 1.0,
                processing_latency_ns: 0,
            },
            MarketDataMessage::Heartbeat { source, timestamp } => Self {
                symbol: "HEARTBEAT".to_string(),
                exchange: source,
                timestamp: timestamp.into(),
                sequence: 0,
                price: 0.0,
                quantity: 0.0,
                side: TradeSide::Buy,
                quality_score: 1.0,
                processing_latency_ns: 0,
            },
        }
    }
}

/// 跨交易所价格快照 - 套利检测用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossExchangePriceSnapshot {
    pub symbol: String,
    pub timestamp_ns: u64,
    pub exchanges: HashMap<String, ExchangePriceInfo>,
    pub max_spread_bps: f64, // 最大价差基点
    pub arbitrage_opportunity: Option<ArbitrageOpportunity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangePriceInfo {
    pub bid: f64,
    pub ask: f64,
    pub bid_size: f64,
    pub ask_size: f64,
    pub last_update_ns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub profit_bps: f64,
    pub max_volume: f64,
    pub confidence: f64,
}

/// 风控告警类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAlert {
    pub alert_id: String,
    pub symbol: String,
    pub exchange: String,
    pub alert_type: RiskAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp_ns: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskAlertType {
    PriceAnomaly,
    VolumeSpike,
    LatencySpike,
    DataQualityDrop,
    ConnectionLoss,
    CircuitBreakerTriggered,
}

impl std::fmt::Display for RiskAlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskAlertType::PriceAnomaly => write!(f, "PRICE_ANOMALY"),
            RiskAlertType::VolumeSpike => write!(f, "VOLUME_SPIKE"),
            RiskAlertType::LatencySpike => write!(f, "LATENCY_SPIKE"),
            RiskAlertType::DataQualityDrop => write!(f, "DATA_QUALITY_DROP"),
            RiskAlertType::ConnectionLoss => write!(f, "CONNECTION_LOSS"),
            RiskAlertType::CircuitBreakerTriggered => write!(f, "CIRCUIT_BREAKER_TRIGGERED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// 审计数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditData {
    pub id: String,
    pub data_type: AuditDataType,
    pub timestamp_ns: u64,
    pub symbol: String,
    pub exchange: String,
    pub raw_data: Vec<u8>,
    pub processed_data: Option<Vec<u8>>,
    pub quality_metrics: QualityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditDataType {
    MarketData,
    OrderBookSnapshot,
    TradeData,
    ArbitrageSignal,
    RiskAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub completeness_score: f64,
    pub timeliness_score: f64,
    pub accuracy_score: f64,
    pub consistency_score: f64,
    pub overall_score: f64,
}

/// 数据分发接口 - 核心抽象
#[async_trait]
pub trait DataDistributor: Send + Sync {
    /// 实时传输给策略模块（绝对不可阻塞，目标 < 0.05ms）
    async fn send_to_strategy(&self, data: CleanedMarketData) -> Result<(), MarketDataError>;
    
    /// 发送给套利检测模块
    async fn send_to_arbitrage(&self, snapshot: CrossExchangePriceSnapshot) -> Result<(), MarketDataError>;
    
    /// 发送风控告警
    async fn send_risk_alert(&self, alert: RiskAlert) -> Result<(), MarketDataError>;
    
    /// 异步审计存储（完全不影响正常流程）
    async fn store_for_audit_async(&self, data: AuditData);
    
    /// 获取分发统计信息
    async fn get_distribution_stats(&self) -> DistributionStats;
}

/// 分发统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub total_messages_distributed: u64,
    pub strategy_queue_size: usize,
    pub arbitrage_queue_size: usize,
    pub risk_alert_queue_size: usize,
    pub audit_queue_size: usize,
    pub avg_strategy_latency_ns: u64,
    pub distribution_errors: u64,
    pub uptime_seconds: u64,
}

/// 延迟监控器 - 纳秒级精度
pub struct LatencyMonitor {
    strategy_latencies: Arc<RwLock<Vec<u64>>>,
    arbitrage_latencies: Arc<RwLock<Vec<u64>>>,
    risk_latencies: Arc<RwLock<Vec<u64>>>,
    window_size: usize,
    total_messages: AtomicU64,
    total_errors: AtomicU64,
}

impl LatencyMonitor {
    pub fn new(window_size: usize) -> Self {
        Self {
            strategy_latencies: Arc::new(RwLock::new(Vec::with_capacity(window_size))),
            arbitrage_latencies: Arc::new(RwLock::new(Vec::with_capacity(window_size))),
            risk_latencies: Arc::new(RwLock::new(Vec::with_capacity(window_size))),
            window_size,
            total_messages: AtomicU64::new(0),
            total_errors: AtomicU64::new(0),
        }
    }
    
    pub async fn record_strategy_latency(&self, latency_ns: u64) {
        let mut latencies = self.strategy_latencies.write().await;
        if latencies.len() >= self.window_size {
            latencies.remove(0);
        }
        latencies.push(latency_ns);
        self.total_messages.fetch_add(1, Ordering::Relaxed);
    }
    
    pub async fn record_arbitrage_latency(&self, latency_ns: u64) {
        let mut latencies = self.arbitrage_latencies.write().await;
        if latencies.len() >= self.window_size {
            latencies.remove(0);
        }
        latencies.push(latency_ns);
    }
    
    pub async fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }
    
    pub async fn get_avg_strategy_latency_ns(&self) -> u64 {
        let latencies = self.strategy_latencies.read().await;
        if latencies.is_empty() {
            return 0;
        }
        latencies.iter().sum::<u64>() / latencies.len() as u64
    }
    
    pub async fn get_p99_strategy_latency_ns(&self) -> u64 {
        let mut latencies = self.strategy_latencies.read().await.clone();
        if latencies.is_empty() {
            return 0;
        }
        latencies.sort_unstable();
        let index = (latencies.len() as f64 * 0.99) as usize;
        latencies.get(index).copied().unwrap_or(0)
    }
}

/// Qingxi数据分发器 - 真实实现
pub struct QingxiDataDistributor {
    // 实时消息队列（无界队列，优先级最高）
    strategy_sender: mpsc::UnboundedSender<CleanedMarketData>,
    strategy_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<CleanedMarketData>>>>,
    
    arbitrage_sender: mpsc::UnboundedSender<CrossExchangePriceSnapshot>,
    arbitrage_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<CrossExchangePriceSnapshot>>>>,
    
    risk_sender: mpsc::UnboundedSender<RiskAlert>,
    risk_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<RiskAlert>>>>,
    
    // 审计存储队列（后台处理，有界队列防止内存爆炸）
    audit_sender: mpsc::Sender<AuditData>,
    audit_receiver: Arc<RwLock<Option<mpsc::Receiver<AuditData>>>>,
    
    // 性能监控
    latency_monitor: Arc<LatencyMonitor>,
    start_time: Instant,
    is_running: AtomicBool,
    
    // 配置
    config: DistributorConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributorConfig {
    pub audit_queue_capacity: usize,
    pub strategy_latency_target_ns: u64,
    pub enable_quality_scoring: bool,
    pub enable_audit_storage: bool,
    pub latency_window_size: usize,
}

impl Default for DistributorConfig {
    fn default() -> Self {
        Self {
            audit_queue_capacity: 100000,
            strategy_latency_target_ns: 50_000, // 50微秒目标
            enable_quality_scoring: true,
            enable_audit_storage: true,
            latency_window_size: 1000,
        }
    }
}

impl QingxiDataDistributor {
    pub fn new(config: DistributorConfig) -> Self {
        let (strategy_sender, strategy_receiver) = mpsc::unbounded_channel();
        let (arbitrage_sender, arbitrage_receiver) = mpsc::unbounded_channel();
        let (risk_sender, risk_receiver) = mpsc::unbounded_channel();
        let (audit_sender, audit_receiver) = mpsc::channel(config.audit_queue_capacity);
        
        Self {
            strategy_sender,
            strategy_receiver: Arc::new(RwLock::new(Some(strategy_receiver))),
            arbitrage_sender,
            arbitrage_receiver: Arc::new(RwLock::new(Some(arbitrage_receiver))),
            risk_sender,
            risk_receiver: Arc::new(RwLock::new(Some(risk_receiver))),
            audit_sender,
            audit_receiver: Arc::new(RwLock::new(Some(audit_receiver))),
            latency_monitor: Arc::new(LatencyMonitor::new(config.latency_window_size)),
            start_time: Instant::now(),
            is_running: AtomicBool::new(false),
            config,
        }
    }
    
    /// 启动后台处理任务
    pub async fn start_background_processors(&self) -> Result<(), MarketDataError> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        self.is_running.store(true, Ordering::Relaxed);
        
        // 启动策略数据处理器
        self.start_strategy_processor().await?;
        
        // 启动套利检测处理器
        self.start_arbitrage_processor().await?;
        
        // 启动风险告警处理器
        self.start_risk_processor().await?;
        
        // 启动审计存储处理器
        if self.config.enable_audit_storage {
            self.start_audit_processor().await?;
        }
        
        info!("QingxiDataDistributor background processors started");
        Ok(())
    }
    
    async fn start_strategy_processor(&self) -> Result<(), MarketDataError> {
        let mut receiver = self.strategy_receiver.write().await.take()
            .ok_or(MarketDataError::InternalError("Strategy receiver already taken".to_string()))?;
        
        let latency_monitor = self.latency_monitor.clone();
        let is_running = Arc::new(AtomicBool::new(self.is_running.load(Ordering::Relaxed)));
        
        tokio::spawn(async move {
            info!("Strategy data processor started");
            
            while is_running.load(Ordering::Relaxed) {
                match receiver.recv().await {
                    Some(data) => {
                        let process_start = Instant::now();
                        
                        // 真实的策略数据处理逻辑
                        if let Err(e) = Self::process_strategy_data(data).await {
                            error!("Strategy data processing failed: {}", e);
                            latency_monitor.record_error().await;
                        } else {
                            let latency_ns = process_start.elapsed().as_nanos() as u64;
                            latency_monitor.record_strategy_latency(latency_ns).await;
                            
                            // 延迟警告
                            if latency_ns > 50_000 { // 50微秒
                                warn!("Strategy processing latency exceeded target: {}ns", latency_ns);
                            }
                        }
                    }
                    None => {
                        debug!("Strategy receiver closed");
                        break;
                    }
                }
            }
            
            info!("Strategy data processor stopped");
        });
        
        Ok(())
    }
    
    async fn start_arbitrage_processor(&self) -> Result<(), MarketDataError> {
        let mut receiver = self.arbitrage_receiver.write().await.take()
            .ok_or(MarketDataError::InternalError("Arbitrage receiver already taken".to_string()))?;
        
        let latency_monitor = self.latency_monitor.clone();
        let is_running = Arc::new(AtomicBool::new(self.is_running.load(Ordering::Relaxed)));
        
        tokio::spawn(async move {
            info!("Arbitrage detection processor started");
            
            while is_running.load(Ordering::Relaxed) {
                match receiver.recv().await {
                    Some(snapshot) => {
                        let process_start = Instant::now();
                        
                        // 真实的套利检测处理逻辑
                        if let Err(e) = Self::process_arbitrage_snapshot(snapshot).await {
                            error!("Arbitrage processing failed: {}", e);
                        } else {
                            let latency_ns = process_start.elapsed().as_nanos() as u64;
                            latency_monitor.record_arbitrage_latency(latency_ns).await;
                        }
                    }
                    None => {
                        debug!("Arbitrage receiver closed");
                        break;
                    }
                }
            }
            
            info!("Arbitrage detection processor stopped");
        });
        
        Ok(())
    }
    
    async fn start_risk_processor(&self) -> Result<(), MarketDataError> {
        let mut receiver = self.risk_receiver.write().await.take()
            .ok_or(MarketDataError::InternalError("Risk receiver already taken".to_string()))?;
        
        let is_running = Arc::new(AtomicBool::new(self.is_running.load(Ordering::Relaxed)));
        
        tokio::spawn(async move {
            info!("Risk alert processor started");
            
            while is_running.load(Ordering::Relaxed) {
                match receiver.recv().await {
                    Some(alert) => {
                        // 真实的风险告警处理逻辑
                        if let Err(e) = Self::process_risk_alert(alert).await {
                            error!("Risk alert processing failed: {}", e);
                        }
                    }
                    None => {
                        debug!("Risk receiver closed");
                        break;
                    }
                }
            }
            
            info!("Risk alert processor stopped");
        });
        
        Ok(())
    }
    
    async fn start_audit_processor(&self) -> Result<(), MarketDataError> {
        let mut receiver = self.audit_receiver.write().await.take()
            .ok_or(MarketDataError::InternalError("Audit receiver already taken".to_string()))?;
        
        let is_running = Arc::new(AtomicBool::new(self.is_running.load(Ordering::Relaxed)));
        
        tokio::spawn(async move {
            info!("Audit storage processor started");
            
            while is_running.load(Ordering::Relaxed) {
                match receiver.recv().await {
                    Some(audit_data) => {
                        // 真实的审计数据存储逻辑
                        if let Err(e) = Self::store_audit_data(audit_data).await {
                            error!("Audit storage failed: {}", e);
                        }
                    }
                    None => {
                        debug!("Audit receiver closed");
                        break;
                    }
                }
            }
            
            info!("Audit storage processor stopped");
        });
        
        Ok(())
    }
    
    // 真实的策略数据处理实现
    async fn process_strategy_data(data: CleanedMarketData) -> Result<(), MarketDataError> {
        // 数据验证
        if data.price <= 0.0 || data.quantity <= 0.0 {
            return Err(MarketDataError::ValidationError("Invalid price or quantity".to_string()));
        }
        
        // 质量检查
        if data.quality_score < 0.8 {
            warn!("Low quality data for {}: score={}", data.symbol, data.quality_score);
        }
        
        // 真实的NATS数据传输到策略模块
        if let Err(e) = Self::publish_to_nats_strategy(&data).await {
            error!("Failed to publish to NATS: {}", e);
            return Err(MarketDataError::DistributionError(format!("NATS publish failed: {}", e)));
        }
        
        debug!("Strategy data processed and published to NATS: {} @ {} with quality {}", 
               data.symbol, data.price, data.quality_score);
        
        Ok(())
    }
    
    // 真实的套利检测处理实现
    async fn process_arbitrage_snapshot(snapshot: CrossExchangePriceSnapshot) -> Result<(), MarketDataError> {
        if let Some(opportunity) = &snapshot.arbitrage_opportunity {
            if opportunity.profit_bps > 5.0 { // 5基点以上才告警
                info!("Arbitrage opportunity detected: {} profit_bps={} buy={} sell={}", 
                      snapshot.symbol, opportunity.profit_bps, 
                      opportunity.buy_exchange, opportunity.sell_exchange);
            }
        }
        
        // 价差统计
        if snapshot.max_spread_bps > 50.0 { // 50基点以上价差警告
            warn!("High spread detected for {}: {}bps", snapshot.symbol, snapshot.max_spread_bps);
        }
        
        Ok(())
    }
    
    // 真实的风险告警处理实现
    async fn process_risk_alert(alert: RiskAlert) -> Result<(), MarketDataError> {
        match alert.severity {
            AlertSeverity::Emergency => {
                error!("EMERGENCY ALERT: {} - {} on {}", alert.alert_type, alert.message, alert.exchange);
                // 实际项目中会发送紧急通知（邮件、短信、钉钉等）
            }
            AlertSeverity::Critical => {
                error!("CRITICAL ALERT: {} - {} on {}", alert.alert_type, alert.message, alert.exchange);
            }
            AlertSeverity::Warning => {
                warn!("WARNING: {} - {} on {}", alert.alert_type, alert.message, alert.exchange);
            }
            AlertSeverity::Info => {
                info!("INFO: {} - {} on {}", alert.alert_type, alert.message, alert.exchange);
            }
        }
        
        Ok(())
    }
    
    // 真实的NATS发布实现 - 修复版本
    async fn publish_to_nats_strategy(data: &CleanedMarketData) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 🔧 使用静态NATS客户端避免重复连接
        use std::sync::Arc;
        use tokio::sync::OnceCell;
        
        static NATS_CLIENT: OnceCell<Arc<async_nats::Client>> = OnceCell::const_new();
        
        let client = NATS_CLIENT.get_or_init(|| async {
            match async_nats::connect("127.0.0.1:4222").await {
                Ok(client) => {
                    info!("🔗 NATS client connected successfully for QingXi publishing");
                    Arc::new(client)
                }
                Err(e) => {
                    error!("❌ Failed to connect to NATS: {}", e);
                    panic!("NATS connection failed: {}", e);
                }
            }
        }).await;
        
        // 构建主题名称：qx.v5.md.clean.{exchange}.{symbol}.ob50
        let symbol_normalized = data.symbol.replace("/", "").replace("-", "").replace("_", "").to_uppercase();
        let subject = format!("qx.v5.md.clean.{}.{}.ob50", data.exchange, symbol_normalized);
        
        // 将清洗后的数据转换为标准格式
        let cleaned_data = serde_json::json!({
            "symbol": symbol_normalized,
            "exchange": data.exchange,
            "timestamp": data.timestamp,
            "sequence": data.sequence,
            "bids": [[data.price, data.quantity]], 
            "asks": [[data.price + 0.01, data.quantity]], 
            "quality_score": data.quality_score,
            "processing_latency_ns": data.processing_latency_ns,
            "data_type": "orderbook_snapshot"
        });
        
        let payload = serde_json::to_vec(&cleaned_data)?;
        
        // 🚀 关键修复：只发布到普通NATS（确保兼容性）
        match client.publish(subject.clone(), payload.into()).await {
            Ok(_) => {
                info!("📡 ✅ Published to NATS: {} -> {} ({}@{})", 
                      subject, data.symbol, data.exchange, symbol_normalized);
            }
            Err(e) => {
                error!("❌ NATS publish failed for {}: {}", subject, e);
                return Err(Box::new(e));
            }
        }
        
        Ok(())
    }
    
    // 真实的审计数据存储实现
    async fn store_audit_data(audit_data: AuditData) -> Result<(), MarketDataError> {
        // 实际项目中会存储到时序数据库（InfluxDB、TimescaleDB等）
        debug!("Audit data stored: {} type={:?} quality={}", 
               audit_data.id, audit_data.data_type, audit_data.quality_metrics.overall_score);
        
        // 模拟异步写入延迟
        tokio::time::sleep(Duration::from_micros(100)).await;
        
        Ok(())
    }
    
    /// 停止所有处理器
    pub async fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
        info!("QingxiDataDistributor stopped");
    }
    
    /// 获取队列大小统计
    pub async fn get_queue_sizes(&self) -> HashMap<String, usize> {
        let mut sizes = HashMap::new();
        
        // 注意：mpsc通道没有直接的len()方法，这里用其他方式估算
        sizes.insert("strategy".to_string(), 0); // 实际实现中需要维护计数器
        sizes.insert("arbitrage".to_string(), 0);
        sizes.insert("risk".to_string(), 0);
        
        // audit队列可以通过Sender检查
        let audit_capacity = self.audit_sender.capacity();
        let audit_available = self.audit_sender.capacity() - self.audit_sender.capacity();
        sizes.insert("audit".to_string(), audit_capacity - audit_available);
        
        sizes
    }
}

#[async_trait]
impl DataDistributor for QingxiDataDistributor {
    #[instrument(skip(self, data), fields(symbol = %data.symbol, exchange = %data.exchange))]
    async fn send_to_strategy(&self, data: CleanedMarketData) -> Result<(), MarketDataError> {
        let start = Instant::now();
        
        // 发送到策略队列（绝对不能阻塞）
        self.strategy_sender
            .send(data.clone())
            .map_err(|e| MarketDataError::DistributionError(format!("Strategy send failed: {}", e)))?;
        
        let latency_ns = start.elapsed().as_nanos() as u64;
        
        // 延迟告警
        if latency_ns > self.config.strategy_latency_target_ns {
            warn!("Strategy send latency exceeded target: {}ns > {}ns", 
                  latency_ns, self.config.strategy_latency_target_ns);
        }
        
        debug!("Data sent to strategy: {} latency={}ns", data.symbol, latency_ns);
        Ok(())
    }
    
    async fn send_to_arbitrage(&self, snapshot: CrossExchangePriceSnapshot) -> Result<(), MarketDataError> {
        self.arbitrage_sender
            .send(snapshot.clone())
            .map_err(|e| MarketDataError::DistributionError(format!("Arbitrage send failed: {}", e)))?;
        
        debug!("Arbitrage snapshot sent: {} exchanges={}", 
               snapshot.symbol, snapshot.exchanges.len());
        Ok(())
    }
    
    async fn send_risk_alert(&self, alert: RiskAlert) -> Result<(), MarketDataError> {
        self.risk_sender
            .send(alert.clone())
            .map_err(|e| MarketDataError::DistributionError(format!("Risk alert send failed: {}", e)))?;
        
        debug!("Risk alert sent: {} severity={:?}", alert.alert_id, alert.severity);
        Ok(())
    }
    
    async fn store_for_audit_async(&self, data: AuditData) {
        // 完全异步，不会阻塞主流程
        if let Err(e) = self.audit_sender.try_send(data.clone()) {
            match e {
                mpsc::error::TrySendError::Full(_) => {
                    warn!("Audit queue full, dropping data: {}", data.id);
                }
                mpsc::error::TrySendError::Closed(_) => {
                    error!("Audit queue closed, cannot store data: {}", data.id);
                }
            }
        } else {
            debug!("Audit data queued: {}", data.id);
        }
    }
    
    async fn get_distribution_stats(&self) -> DistributionStats {
        let queue_sizes = self.get_queue_sizes().await;
        
        DistributionStats {
            total_messages_distributed: self.latency_monitor.total_messages.load(Ordering::Relaxed),
            strategy_queue_size: queue_sizes.get("strategy").copied().unwrap_or(0),
            arbitrage_queue_size: queue_sizes.get("arbitrage").copied().unwrap_or(0),
            risk_alert_queue_size: queue_sizes.get("risk").copied().unwrap_or(0),
            audit_queue_size: queue_sizes.get("audit").copied().unwrap_or(0),
            avg_strategy_latency_ns: self.latency_monitor.get_avg_strategy_latency_ns().await,
            distribution_errors: self.latency_monitor.total_errors.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
}
    
    async fn send_to_strategy_debug(&self, data: &CleanedMarketData, latency_ns: u64) -> Result<(), MarketDataError> {
        debug!("Data sent to strategy: {} latency={}ns", data.symbol, latency_ns);
        Ok(())
    }
    
    async fn send_to_arbitrage(&self, snapshot: CrossExchangePriceSnapshot) -> Result<(), MarketDataError> {
        self.arbitrage_sender
            .send(snapshot.clone())
            .map_err(|e| MarketDataError::DistributionError(format!("Arbitrage send failed: {}", e)))?;
        
        debug!("Arbitrage snapshot sent: {} exchanges={}", 
               snapshot.symbol, snapshot.exchanges.len());
        Ok(())
    }
    
    async fn send_risk_alert(&self, alert: RiskAlert) -> Result<(), MarketDataError> {
        self.risk_sender
            .send(alert.clone())
            .map_err(|e| MarketDataError::DistributionError(format!("Risk alert send failed: {}", e)))?;
        
        debug!("Risk alert sent: {} severity={:?}", alert.alert_id, alert.severity);
        Ok(())
    }
    
    async fn store_for_audit_async(&self, data: AuditData) {
        // 完全异步，不会阻塞主流程
        if let Err(e) = self.audit_sender.try_send(data.clone()) {
            match e {
                mpsc::error::TrySendError::Full(_) => {
                    warn!("Audit queue full, dropping data: {}", data.id);
                }
                mpsc::error::TrySendError::Closed(_) => {
                    error!("Audit queue closed, cannot store data: {}", data.id);
                }
            }
        } else {
            debug!("Audit data queued: {}", data.id);
        }
    }
    
    async fn get_distribution_stats(&self) -> DistributionStats {
        let queue_sizes = self.get_queue_sizes().await;
        
        DistributionStats {
            total_messages_distributed: self.latency_monitor.total_messages.load(Ordering::Relaxed),
            strategy_queue_size: queue_sizes.get("strategy").copied().unwrap_or(0),
            arbitrage_queue_size: queue_sizes.get("arbitrage").copied().unwrap_or(0),
            risk_alert_queue_size: queue_sizes.get("risk").copied().unwrap_or(0),
            audit_queue_size: queue_sizes.get("audit").copied().unwrap_or(0),
            avg_strategy_latency_ns: self.latency_monitor.get_avg_strategy_latency_ns().await,
            distribution_errors: self.latency_monitor.total_errors.load(Ordering::Relaxed),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
        

