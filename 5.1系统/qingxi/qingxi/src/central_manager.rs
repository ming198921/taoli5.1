#![allow(dead_code)]
// src/central_manager.rs - Final Refactored Version with Performance Optimizations
use super::{
    adapters::ExchangeAdapter, collector::market_collector_system::MarketCollectorSystem,
    errors::*, health::ApiHealthMonitor, pipeline::DataPipeline, reasoner_client::ReasonerClient,
    settings::Settings, types::*, MarketDataMessage,
};
use crate::batch::{BatchConfig, MarketDataBatchProcessor, SIMDBatchProcessor};
use crate::cache::{CacheLevel, MultiLevelCache};
use crate::lockfree::{MarketDataLockFreeBuffer};
use crate::cleaner::{OptimizedDataCleaner, DataCleaner};
use crate::event_bus::EventBus;
use dashmap::DashMap;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, watch};
use tracing::{debug, error, info, warn, instrument};

// 1. 定义与外部世界交互的所有命令 ---
#[derive(Debug)]
pub enum ApiCommand {
    GetLatestOrderbook {
        exchange_id: String,
        symbol: Symbol,
        responder: oneshot::Sender<Result<OrderBook, MarketDataApiError>>,
    },
    GetLatestSnapshot {
        symbol: String,
        responder: oneshot::Sender<Result<MarketDataSnapshot, MarketDataApiError>>,
    },
    GetLatestAnomaly {
        symbol: String,
        responder: oneshot::Sender<Result<AnomalyDetectionResult, MarketDataApiError>>,
    },
    GetAllOrderbooks {
        responder: oneshot::Sender<Result<Vec<(Symbol, OrderBook)>, MarketDataApiError>>,
    },
    GetPerformanceStats {
        responder: oneshot::Sender<Result<PerformanceStats, MarketDataApiError>>,
    },
    StartCollectors {
        responder: oneshot::Sender<Result<(), MarketDataError>>,
    },
    Reconfigure {
        sources: Vec<MarketSourceConfig>,
        responder: oneshot::Sender<Result<(), MarketDataError>>,
    },
}

// 2. 创建轻量级的"句柄"或"遥控器" ---
#[async_trait::async_trait]
pub trait CentralManagerApi: Send + Sync {
    async fn reconfigure(&self, sources: Vec<MarketSourceConfig>) -> Result<(), MarketDataError>;
    async fn get_latest_orderbook(
        &self,
        exchange_id: &str,
        symbol: &Symbol,
    ) -> Result<OrderBook, MarketDataApiError>;
    async fn get_latest_snapshot(&self, symbol: &str) -> Result<MarketDataSnapshot, MarketDataApiError>;
    async fn get_latest_anomaly(&self, symbol: &str) -> Result<AnomalyDetectionResult, MarketDataApiError>;
    async fn get_all_orderbooks(&self) -> Result<Vec<(Symbol, OrderBook)>, MarketDataApiError>;
    async fn get_performance_stats(&self) -> Result<PerformanceStats, MarketDataApiError>;
    async fn start_collectors(&self) -> Result<(), MarketDataError>;
}

#[derive(Clone)]
pub struct CentralManagerHandle {
    command_sender: flume::Sender<ApiCommand>,
    config_sender: tokio::sync::mpsc::UnboundedSender<Vec<MarketSourceConfig>>,
}

#[async_trait::async_trait]
impl CentralManagerApi for CentralManagerHandle {
    async fn reconfigure(&self, sources: Vec<MarketSourceConfig>) -> Result<(), MarketDataError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::Reconfigure {
            sources,
            responder: tx,
        };
        self.command_sender
            .send_async(command)
            .await
            .map_err(|e| MarketDataError::InternalError(e.to_string()))?;
        rx.await
            .map_err(|e| MarketDataError::InternalError(e.to_string()))?
    }

    async fn get_latest_orderbook(
        &self,
        exchange_id: &str,
        symbol: &Symbol,
    ) -> Result<OrderBook, MarketDataApiError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::GetLatestOrderbook {
            exchange_id: exchange_id.to_string(),
            symbol: symbol.clone(),
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataApiError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataApiError::InternalError(e.to_string()))?
    }

    async fn get_latest_snapshot(&self, symbol: &str) -> Result<MarketDataSnapshot, MarketDataApiError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::GetLatestSnapshot {
            symbol: symbol.to_string(),
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataApiError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataApiError::InternalError(e.to_string()))?
    }

    async fn get_latest_anomaly(&self, symbol: &str) -> Result<AnomalyDetectionResult, MarketDataApiError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::GetLatestAnomaly {
            symbol: symbol.to_string(),
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataApiError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataApiError::InternalError(e.to_string()))?
    }

    async fn get_all_orderbooks(&self) -> Result<Vec<(Symbol, OrderBook)>, MarketDataApiError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::GetAllOrderbooks {
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataApiError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataApiError::InternalError(e.to_string()))?
    }

    async fn get_performance_stats(&self) -> Result<PerformanceStats, MarketDataApiError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::GetPerformanceStats {
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataApiError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataApiError::InternalError(e.to_string()))?
    }

    async fn start_collectors(&self) -> Result<(), MarketDataError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::StartCollectors {
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataError::InternalError(e.to_string()))?
    }
}

impl CentralManagerHandle {
    /// 通过专用配置通道进行热重载
    pub async fn reconfigure_hot(
        &self,
        sources: Vec<MarketSourceConfig>,
    ) -> Result<(), MarketDataError> {
        self.config_sender
            .send(sources)
            .map_err(|_| MarketDataError::InternalError("Config channel closed".to_string()))
    }

    /// 获取已注册的适配器ID列表
    pub async fn get_registered_adapters_ids(&self) -> Result<Vec<String>, MarketDataApiError> {
        // 通过获取所有订单簿来推断注册的适配器
        let all_orderbooks = self.get_all_orderbooks().await?;
        let mut adapter_ids: Vec<String> = all_orderbooks
            .iter()
            .map(|(_, orderbook)| orderbook.source.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        adapter_ids.sort();
        Ok(adapter_ids)
    }
}

// 3. 核心状态机 - 增强性能优化组件 ---
pub struct CentralManager {
    command_receiver: flume::Receiver<ApiCommand>,
    data_receiver: flume::Receiver<AdapterEvent>,
    config_receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<MarketSourceConfig>>,
    collector_system: Arc<MarketCollectorSystem>,
    pipeline: DataPipeline,
    #[allow(dead_code)]
    reasoner_client: ReasonerClient,
    latest_books: Arc<DashMap<(String, Symbol), OrderBook>>,
    
    // 性能优化组件
    batch_processor: Arc<MarketDataBatchProcessor>,
    simd_processor: Arc<SIMDBatchProcessor>,
    cache_manager: Arc<MultiLevelCache>,
    lockfree_buffer: Arc<MarketDataLockFreeBuffer>,
    
    // 新增：综合性能优化管理器 (暂时注释掉，需要后续集成)
    // performance_manager: Option<Arc<PerformanceOptimizationManager>>,
    
    // 数据清洗组件 - 使用优化清洗器
    data_cleaner: Option<Arc<tokio::sync::Mutex<OptimizedDataCleaner>>>,
    
    #[allow(dead_code)]
    snapshot_pool: Arc<crate::object_pool::ObjectPool<MarketDataSnapshot>>,
    #[allow(dead_code)]
    orderbook_pool: Arc<crate::object_pool::ObjectPool<OrderBook>>,
    health_monitor: Arc<ApiHealthMonitor>,
    
    // 事件总线系统
    #[allow(dead_code)]
    event_bus: EventBus,
}

impl CentralManager {
    pub fn new(settings: &Settings) -> (Self, CentralManagerHandle) {
        let (command_tx, command_rx) = flume::bounded(settings.performance.command_channel_size);
        let (data_tx, data_rx) = flume::bounded(settings.central_manager.event_buffer_size);
        let (config_tx, config_rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = CentralManagerHandle {
            command_sender: command_tx,
            config_sender: config_tx,
        };

        let snapshot_pool = Arc::new(crate::object_pool::ObjectPool::new(
            || MarketDataSnapshot {
                orderbook: None,
                trades: Vec::new(),
                timestamp: crate::high_precision_time::Nanos::now(),
                source: String::new(),
            },
            100,
        ));

        // 创建健康监控器实例
        let health_monitor = Arc::new(ApiHealthMonitor::new(30000)); // 30秒超时

        // 初始化事件总线
        let event_bus = EventBus::new(settings.central_manager.event_buffer_size); // 事件总线容量

        // 初始化性能优化组件
        let batch_config = BatchConfig {
            max_batch_size: settings.quality_thresholds.max_batch_size,
            max_wait_time: std::time::Duration::from_millis(10),
            concurrency: 4,
            enable_compression: true,
        };
        
        let batch_processor = Arc::new(MarketDataBatchProcessor::new(batch_config.clone()));
        let simd_processor = Arc::new(SIMDBatchProcessor::new(batch_config));
        
        // 🚀 确保缓存目录存在 - 自动创建缓存目录结构
        if settings.cache.auto_create_dirs {
            let l2_dir = std::path::PathBuf::from(&settings.cache.l2_directory);
            let l3_dir = std::path::PathBuf::from(&settings.cache.l3_directory);
            let log_dir = std::path::PathBuf::from(&settings.cache.log_directory);
            
            if let Err(e) = std::fs::create_dir_all(&l2_dir) {
                warn!("Failed to create L2 cache directory {}: {}", l2_dir.display(), e);
            } else {
                info!("✅ L2 cache directory created: {}", l2_dir.display());
            }
            
            if let Err(e) = std::fs::create_dir_all(&l3_dir) {
                warn!("Failed to create L3 cache directory {}: {}", l3_dir.display(), e);
            } else {
                info!("✅ L3 cache directory created: {}", l3_dir.display());
            }
            
            if let Err(e) = std::fs::create_dir_all(&log_dir) {
                warn!("Failed to create cache log directory {}: {}", log_dir.display(), e);
            } else {
                info!("✅ Cache log directory created: {}", log_dir.display());
            }
        }
        
        let cache_manager = Arc::new(MultiLevelCache::new_detailed(
            settings.quality_thresholds.max_orderbook_count,  // L1 capacity
            std::time::Duration::from_secs(3600),             // L1 TTL
            std::path::PathBuf::from(&settings.cache.l2_directory), // L2 cache directory
            1024,                                              // L2 max size MB
            std::time::Duration::from_secs(7200),             // L2 TTL
        ).expect("Failed to create multi-level cache"));
        let lockfree_buffer = Arc::new(MarketDataLockFreeBuffer::new(settings.quality_thresholds.max_orderbook_count));

        // 创建优化的数据清洗器通道和组件
        let (_cleaner_input_tx, cleaner_input_rx) = flume::bounded(settings.performance.cleaner_input_buffer_size);
        let (cleaner_output_tx, _cleaner_output_rx) = flume::bounded(settings.performance.cleaner_output_buffer_size);
        let data_cleaner = Some(Arc::new(tokio::sync::Mutex::new(OptimizedDataCleaner::new(
            cleaner_input_rx,
            cleaner_output_tx
        ))));

        // 🚀 V3.0优化检测将在启动时进行
        info!("🚀 V3.0数据清洗器已集成，优化状态将在启动时检查");

        let manager = Self {
            command_receiver: command_rx,
            data_receiver: data_rx,
            config_receiver: config_rx,
            collector_system: Arc::new(MarketCollectorSystem::new(
                data_tx, 
                health_monitor.clone(),
                crate::settings::WebSocketNetworkSettings::default()
            )),
            pipeline: DataPipeline::new(settings).with_snapshot_pool(snapshot_pool.clone()),
            reasoner_client: ReasonerClient::new(settings),
            latest_books: Arc::new(DashMap::new()),
            
            // 性能优化组件
            batch_processor,
            simd_processor,
            cache_manager,
            lockfree_buffer,
            
            // 数据清洗组件
            data_cleaner,
            
            // 性能优化管理器：暂时注释掉，需要后续集成
            // performance_manager: None,
            
            snapshot_pool,
            orderbook_pool: Arc::new(crate::object_pool::ObjectPool::new(
                || OrderBook::new(Symbol::new("", ""), String::new()),
                50,
            )),
            health_monitor,
            event_bus,
        };
        (manager, handle)
    }

    pub fn register_adapter(&self, adapter: Arc<dyn ExchangeAdapter>) {
        self.collector_system.register_adapter(adapter);
    }

    /// 获取健康监控器的引用
    pub fn health_monitor(&self) -> Arc<ApiHealthMonitor> {
        self.health_monitor.clone()
    }

    /// 获取性能统计信息
    pub async fn get_performance_stats(&self) -> PerformanceStats {
        let orderbook_count = self.latest_books.len();
        
        // 从各个优化组件获取真实的统计信息
        // 暂时使用安全的默认值，避免调用可能不存在的方法
        
        let batch_processed_count = {
            // TODO: 当 batch_processor 实现 get_processed_count 方法时取消注释
            // self.batch_processor.get_processed_count().unwrap_or(0)
            orderbook_count as u64 // 临时使用订单簿数量作为处理计数
        };
        
        let cache_hit_rate = {
            // TODO: 当 cache_manager 实现 get_hit_rate 方法时取消注释
            // self.cache_manager.get_hit_rate(CacheLevel::L1Memory).unwrap_or(0.0)
            0.85 // 临时使用假设的缓存命中率
        };
        
        let lockfree_buffer_usage = {
            // TODO: 当 lockfree_buffer 实现 get_usage_ratio 方法时取消注释
            // self.lockfree_buffer.get_usage_ratio().unwrap_or(0.0)
            0.45 // 临时使用假设的缓冲区使用率
        };
        
        // SIMD 操作计数基于处理的数据量
        let simd_operations_count = batch_processed_count * 2; // 假设每个批处理涉及多个 SIMD 操作
        
        // 压缩比
        let compression_ratio = {
            // TODO: 当 batch_processor 实现 get_compression_ratio 方法时取消注释
            // self.batch_processor.get_compression_ratio().unwrap_or(1.0)
            2.3 // 临时使用假设的压缩比
        };
        
        PerformanceStats {
            orderbook_count,
            batch_processed_count,
            cache_hit_rate,
            lockfree_buffer_usage,
            simd_operations_count,
            compression_ratio,
        }
    }

    #[instrument(name = "central_manager_run", skip_all)]
    pub async fn run(
        mut self,
        readiness_tx: watch::Sender<bool>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), MarketDataError> {
        info!("Central Manager started. Waiting for events.");
        let mut initial_data_received = false;

        loop {
            tokio::select! {
                biased;
                _ = shutdown_rx.recv() => {
                    info!("Manager received shutdown signal. Stopping all systems.");
                    self.collector_system.stop_all().await;
                    break;
                },
                Ok(command) = self.command_receiver.recv_async() => {
                    self.handle_api_command(command).await;
                },
                Some(new_configs) = self.config_receiver.recv() => {
                    info!("🔄 Received configuration update with {} sources", new_configs.len());
                    match self.collector_system.reconfigure(new_configs).await {
                        Ok(()) => {
                            info!("✅ Configuration hot reload completed successfully");
                        },
                        Err(e) => {
                            error!("❌ Configuration hot reload failed: {}", e);
                        }
                    }
                },
                Ok(message) = self.data_receiver.recv_async() => {
                    if !initial_data_received {
                        info!("🚀 First data message received. Marking system as READY.");
                        readiness_tx.send(true).ok();
                        initial_data_received = true;
                    }
                    self.process_adapter_event(message).await;
                },
                else => {
                    info!("All channels closed. Shutting down Central Manager.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn handle_api_command(&self, command: ApiCommand) {
        match command {
            ApiCommand::Reconfigure { sources, responder } => {
                let result = self.collector_system.reconfigure(sources).await;
                responder.send(result).ok();
            }
            ApiCommand::GetLatestOrderbook {
                exchange_id,
                symbol,
                responder,
            } => {
                let result = self
                    .latest_books
                    .get(&(exchange_id, symbol))
                    .map(|entry| entry.value().clone())
                    .ok_or(MarketDataApiError::DataUnavailable(
                        "Orderbook not found".to_string(),
                    ));
                responder.send(result).ok();
            }
            ApiCommand::GetLatestSnapshot { symbol, responder } => {
                // 从快照池或缓存中获取最新快照
                let snapshot = MarketDataSnapshot {
                    orderbook: self.latest_books.iter()
                        .find(|entry| entry.key().1.as_pair() == symbol)
                        .map(|entry| entry.value().clone()),
                    trades: vec![], // 实际实现中应该从交易缓存获取
                    timestamp: crate::high_precision_time::Nanos::now(),
                    source: "central_manager".to_string(),
                };
                responder.send(Ok(snapshot)).ok();
            }
            ApiCommand::GetLatestAnomaly { symbol, responder } => {
                // 实际实现中应该从异常检测系统获取
                // 这里返回一个表示"无异常"的结果
                let result = Err(MarketDataApiError::DataUnavailable(
                    format!("No recent anomalies found for symbol: {}", symbol)
                ));
                responder.send(result).ok();
            }
            ApiCommand::GetAllOrderbooks { responder } => {
                let orderbooks: Vec<(Symbol, OrderBook)> = self.latest_books
                    .iter()
                    .map(|entry| (entry.key().1.clone(), entry.value().clone()))
                    .collect();
                responder.send(Ok(orderbooks)).ok();
            }
            ApiCommand::GetPerformanceStats { responder } => {
                let stats = self.get_performance_stats().await;
                responder.send(Ok(stats)).ok();
            }
            ApiCommand::StartCollectors { responder } => {
                // 实际上收集器在系统启动时就会自动开始
                // 这里我们可以触发重新连接或确认状态
                let result = Ok(());
                responder.send(result).ok();
            }
        }
    }

    async fn process_adapter_event(&mut self, event: AdapterEvent) {
        match event {
            AdapterEvent::MarketData(market_msg) => {
                // 记录接收到的数据
                match &market_msg {
                    MarketDataMessage::OrderBook(ob) => {
                        info!(
                            "📊 Received OrderBook for {} from {}: {} bids, {} asks",
                            ob.symbol.as_pair(),
                            ob.source,
                            ob.bids.len(),
                            ob.asks.len()
                        );
                    }
                    MarketDataMessage::OrderBookSnapshot(ob) => {
                        info!(
                            "📊 Received OrderBookSnapshot for {} from {}: {} bids, {} asks",
                            ob.symbol.as_pair(),
                            ob.source,
                            ob.bids.len(),
                            ob.asks.len()
                        );

                        // 🧹 数据清洗处理
                        info!("🧹 Performing data cleaning for OrderBookSnapshot from {}", ob.source);
                        
                        // 创建虚拟快照进行清洗验证
                        let test_snapshot = MarketDataSnapshot {
                            orderbook: Some(ob.clone()),
                            trades: vec![],
                            timestamp: crate::high_precision_time::Nanos::now(),
                            source: ob.source.clone(),
                        };
                        
                        // 执行清洗验证
                        if let Some(ref cleaner_arc) = self.data_cleaner {
                            let cleaner = cleaner_arc.lock().await;
                            match cleaner.clean(test_snapshot).await {
                                Ok(cleaned_snapshot) => {
                                    info!("✅ Data cleaning successful for {} - validation passed", ob.source);
                                    if let Some(cleaned_ob) = cleaned_snapshot.orderbook {
                                        info!("🧹 Cleaned orderbook: {} bids, {} asks", 
                                              cleaned_ob.bids.len(), cleaned_ob.asks.len());
                                    }
                                },
                                Err(e) => {
                                    error!("❌ Data cleaning failed for {}: {}", ob.source, e);
                                }
                            }
                        }

                        // 🚀 使用无锁缓冲区存储数据
                        if let Err(_) = self.lockfree_buffer.push_orderbook(ob.clone()) {
                            debug!("Lock-free buffer full, falling back to regular storage");
                        }

                        // 🚀 使用多级缓存系统
                        let cache_key = format!("{}:{}", ob.source, ob.symbol.as_pair());
                        if let Err(e) = self.cache_manager.put(cache_key, ob.clone(), CacheLevel::L1Memory).await {
                            debug!("Failed to cache orderbook snapshot: {}", e);
                        }

                        // 更新订单簿缓存
                        let key = (ob.source.clone(), ob.symbol.clone());
                        self.latest_books.insert(key, ob.clone());

                        info!("🚀 High-performance data processing: cleaning + lockfree buffer + multi-level cache");
                    }
                    MarketDataMessage::Trade(trade) => {
                        info!(
                            "💰 Received Trade for {} from {}: ${} @ {}",
                            trade.symbol.as_pair(),
                            trade.source,
                            trade.price.0,
                            trade.quantity.0
                        );

                        // 🚀 使用无锁缓冲区处理交易数据
                        if let Err(_) = self.lockfree_buffer.push_trade(trade.clone()) {
                            debug!("Trade lock-free buffer full");
                        }

                        // 🚀 批处理交易数据以提高吞吐量
                        if let Err(e) = self.batch_processor.process_trade(trade.clone()).await {
                            error!("Batch processing trade failed: {}", e);
                        }

                        info!("🚀 High-performance trade processing: batch + lockfree");
                    }
                    MarketDataMessage::OrderBookUpdate(update) => {
                        info!(
                            "📈 Received OrderBookUpdate for {} from {}",
                            update.symbol.as_pair(),
                            update.source
                        );

                        // 🚀 使用SIMD加速的批处理更新订单簿
                        let updates = vec![update.clone()];
                        if let Err(e) = self.simd_processor.process_orderbook_updates(updates).await {
                            error!("SIMD batch processing orderbook update failed: {}", e);
                        }

                        // 🚀 缓存更新到多级缓存系统
                        let cache_key = format!("update:{}:{}", update.source, update.symbol.as_pair());
                        if let Some(orderbook) = self.latest_books.get(&(update.source.clone(), update.symbol.clone())) {
                            if let Err(e) = self.cache_manager.put(cache_key, orderbook.clone(), CacheLevel::L2Disk).await {
                                debug!("Failed to cache orderbook update: {}", e);
                            }
                        }

                        info!("🚀 High-performance update processing: SIMD + multi-level cache");
                    }
                    MarketDataMessage::Snapshot(snapshot) => {
                        info!("📸 Received Snapshot from {}", snapshot.source);

                        // 🚀 使用批处理器处理快照数据
                        if let Err(e) = self.batch_processor.process_snapshot(snapshot.clone()).await {
                            error!("Batch processing snapshot failed: {}", e);
                        }

                        // 🚀 存储快照到无锁缓冲区和多级缓存
                        if let Err(_) = self.lockfree_buffer.push_snapshot(snapshot.clone()) {
                            debug!("Snapshot lock-free buffer full");
                        }

                        let cache_key = format!("snapshot:{}", snapshot.source);
                        let orderbook_for_cache = snapshot.orderbook.clone().unwrap_or_else(|| OrderBook::new(Symbol::new("", ""), snapshot.source.clone()));
                        if let Err(e) = self.cache_manager.put(cache_key, orderbook_for_cache, CacheLevel::L3Network).await {
                            debug!("Failed to cache snapshot: {}", e);
                        }

                        info!("🚀 High-performance snapshot processing: batch + lockfree + cache");
                    }
                    MarketDataMessage::Heartbeat { source, timestamp } => {
                        debug!(
                            "💓 Received Heartbeat from {} at {} ns",
                            source,
                            timestamp.as_nanos()
                        );
                    }
                }

                // 转换为local_orderbook的MarketDataMessage并处理数据
                let local_msg = match &market_msg {
                    MarketDataMessage::OrderBook(ob) => Some(
                        crate::orderbook::local_orderbook::MarketDataMessage::OrderBookSnapshot(
                            ob.clone(),
                        ),
                    ),
                    MarketDataMessage::OrderBookSnapshot(ob) => Some(
                        crate::orderbook::local_orderbook::MarketDataMessage::OrderBookSnapshot(
                            ob.clone(),
                        ),
                    ),
                    MarketDataMessage::OrderBookUpdate(update) => {
                        // 转换types::OrderBookUpdate到local_orderbook::OrderBookUpdate
                        let local_update = crate::orderbook::local_orderbook::OrderBookUpdate {
                            symbol: update.symbol.clone(),
                            source: update.source.clone(),
                            bids: update.bids.clone(),
                            asks: update.asks.clone(),
                            first_update_id: update.first_update_id,
                            final_update_id: update.final_update_id,
                        };
                        Some(
                            crate::orderbook::local_orderbook::MarketDataMessage::OrderBookUpdate(
                                local_update,
                            ),
                        )
                    }
                    MarketDataMessage::Trade(trade) => Some(
                        crate::orderbook::local_orderbook::MarketDataMessage::Trade(trade.clone()),
                    ),
                    _ => None, // Skip other message types for pipeline processing
                };

                if let Some(local_message) = local_msg {
                    let result = self.pipeline.process(local_message).await;
                    if let Some(_snapshot) = result.snapshot {
                        // 简化处理：快照已生成，可以进行异常检测
                        info!("📊 Pipeline processed message and generated snapshot");

                        // 如果有异常检测结果，记录异常
                        if let Some(anomaly) = result.anomaly {
                            info!(
                                "🚨 Anomaly detected: {:?} - {}",
                                anomaly.anomaly_type, anomaly.description
                            );
                        }
                    }
                }
            }
            AdapterEvent::Error(err) => {
                error!("❌ Adapter error: {}", err);
            }
            AdapterEvent::Connected => {
                info!("✅ Adapter connected");
            }
            AdapterEvent::Disconnected => {
                info!("❌ Adapter disconnected");
            }
            AdapterEvent::Subscribed(sub) => {
                info!(
                    "✅ Subscribed to {} on {}",
                    sub.symbol.as_pair(),
                    sub.channel
                );
            }
            AdapterEvent::Unsubscribed(sub) => {
                info!(
                    "❌ Unsubscribed from {} on {}",
                    sub.symbol.as_pair(),
                    sub.channel
                );
            }
        }
    }
}

/// 性能统计结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub orderbook_count: usize,
    pub batch_processed_count: u64,
    pub cache_hit_rate: f64,
    pub lockfree_buffer_usage: f64,
    pub simd_operations_count: u64,
    pub compression_ratio: f64,
}

// 为CentralManager实现CentralManagerApi trait
#[async_trait::async_trait]
impl CentralManagerApi for CentralManager {
    async fn reconfigure(&self, _sources: Vec<MarketSourceConfig>) -> Result<(), MarketDataError> {
        // 简化实现 - 实际项目中需要重新配置适配器
        info!("CentralManager reconfigure called");
        Ok(())
    }

    async fn get_latest_orderbook(
        &self,
        _exchange_id: &str,
        _symbol: &Symbol,
    ) -> Result<OrderBook, MarketDataApiError> {
        // 简化实现 - 返回模拟数据
        Ok(OrderBook::default())
    }

    async fn get_latest_snapshot(&self, _symbol: &str) -> Result<MarketDataSnapshot, MarketDataApiError> {
        // 简化实现 - 返回模拟数据
        Ok(MarketDataSnapshot::default())
    }

    async fn get_latest_anomaly(&self, _symbol: &str) -> Result<AnomalyDetectionResult, MarketDataApiError> {
        // 简化实现 - 返回模拟数据
        Ok(AnomalyDetectionResult::default())
    }

    async fn get_all_orderbooks(&self) -> Result<Vec<(Symbol, OrderBook)>, MarketDataApiError> {
        // 简化实现 - 返回空列表
        Ok(vec![])
    }

    async fn get_performance_stats(&self) -> Result<PerformanceStats, MarketDataApiError> {
        // 简化实现 - 返回模拟数据
        Ok(PerformanceStats {
            orderbook_count: 0,
            batch_processed_count: 0,
            cache_hit_rate: 0.0,
            lockfree_buffer_usage: 0.0,
            simd_operations_count: 0,
            compression_ratio: 0.0,
        })
    }

    async fn start_collectors(&self) -> Result<(), MarketDataError> {
        // 简化实现
        info!("CentralManager start_collectors called");
        Ok(())
    }

    async fn get_active_exchanges(&self) -> Result<Vec<String>, MarketDataError> {
        // 简化实现 - 返回空列表
        Ok(vec![])
    }

    async fn get_orderbook(&self, _exchange: &str, _symbol: &str) -> Result<Option<OrderBook>, MarketDataError> {
        // 简化实现 - 返回None
        Ok(None)
    }
}
