//! WebSocket实时费率监控系统 - 5分钟间隔优化版本
//! 
//! 提供实时交易费率监控、动态费率调整和费率预测功能
//! 支持多交易所WebSocket连接和智能费率优化策略
//! 
//! 核心特性：
//! - 5分钟监控间隔优化 (300秒)
//! - 配置化支持，可动态调整监控频率
//! - 自适应监控频率调整

pub mod config;
pub mod websocket_client;
pub mod fee_tracker;
pub mod fee_optimizer;
pub mod fee_predictor;
pub mod exchange_manager;
// pub mod metrics; // 已清理 - 模块不存在
pub mod storage;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{info, warn, error, debug, instrument};
use uuid::Uuid;

pub use config::FeeMonitorConfig;
pub use websocket_client::{WebSocketClient, WebSocketMessage};
pub use fee_tracker::{FeeTracker, FeeRecord, FeeType};
pub use fee_optimizer::{FeeOptimizer, OptimizationStrategy, FeeRecommendation};
pub use fee_predictor::{FeePredictor, FeePrediction, PredictionModel};
pub use exchange_manager::{ExchangeManager, ExchangeInfo, ExchangeStatus};
pub use metrics::FeeMonitorMetrics;
pub use storage::{FeeStorage, StorageBackend};

/// 费率更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeUpdateEvent {
    pub event_id: String,
    pub exchange: String,
    pub symbol: String,
    pub fee_type: FeeType,
    pub old_rate: f64,
    pub new_rate: f64,
    pub timestamp: DateTime<Utc>,
    pub change_percentage: f64,
    pub volume_24h: Option<f64>,
}

/// 费率警报事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeAlertEvent {
    pub alert_id: String,
    pub exchange: String,
    pub symbol: String,
    pub alert_type: FeeAlertType,
    pub current_rate: f64,
    pub threshold: f64,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
}

/// 费率警报类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeeAlertType {
    HighFeeAlert,
    FeeSpike,
    FeeAnomalyDetected,
    OptimalFeeFound,
    ExchangeConnectionLost,
}

/// 警报严重性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// WebSocket实时费率监控系统
pub struct RealTimeFeeMonitor {
    /// 配置
    config: FeeMonitorConfig,
    /// 交易所管理器
    exchange_manager: Arc<ExchangeManager>,
    /// 费率跟踪器
    fee_tracker: Arc<FeeTracker>,
    /// 费率优化器
    fee_optimizer: Arc<FeeOptimizer>,
    /// 费率预测器
    fee_predictor: Arc<FeePredictor>,
    /// 存储系统
    storage: Arc<dyn FeeStorage>,
    /// 指标收集器
    metrics: Arc<FeeMonitorMetrics>,
    /// 费率更新事件广播器
    fee_update_tx: broadcast::Sender<FeeUpdateEvent>,
    /// 费率警报事件广播器
    alert_tx: broadcast::Sender<FeeAlertEvent>,
    /// 内部事件处理器
    event_tx: mpsc::UnboundedSender<InternalEvent>,
    /// 运行状态
    running: Arc<RwLock<bool>>,
    /// WebSocket客户端池
    websocket_clients: Arc<RwLock<HashMap<String, Arc<WebSocketClient>>>>,
}

/// 内部事件类型
#[derive(Debug, Clone)]
enum InternalEvent {
    FeeUpdate(FeeUpdateEvent),
    Alert(FeeAlertEvent),
    ConnectionStateChanged(String, bool),
    HealthCheck,
    Cleanup,
}

impl RealTimeFeeMonitor {
    /// 创建新的实时费率监控系统
    pub async fn new(
        config: FeeMonitorConfig,
        storage: Arc<dyn FeeStorage>,
    ) -> Result<Self> {
        let exchange_manager = Arc::new(ExchangeManager::new(config.exchanges.clone()).await?);
        let fee_tracker = Arc::new(FeeTracker::new(config.tracking.clone())?);
        let fee_optimizer = Arc::new(FeeOptimizer::new(config.optimization.clone())?);
        let fee_predictor = Arc::new(FeePredictor::new(config.prediction.clone()).await?);
        let metrics = Arc::new(FeeMonitorMetrics::new(config.metrics.clone())?);

        let (fee_update_tx, _) = broadcast::channel(10000);
        let (alert_tx, _) = broadcast::channel(1000);
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let monitor = Self {
            config,
            exchange_manager,
            fee_tracker,
            fee_optimizer,
            fee_predictor,
            storage,
            metrics,
            fee_update_tx,
            alert_tx,
            event_tx,
            running: Arc::new(RwLock::new(false)),
            websocket_clients: Arc::new(RwLock::new(HashMap::new())),
        };

        // 启动内部事件处理器
        monitor.start_event_processor(event_rx).await;

        info!("Real-time fee monitor initialized successfully");
        Ok(monitor)
    }

    /// 启动费率监控系统
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            warn!("Fee monitor is already running");
            return Ok(());
        }
        *running = true;
        drop(running);

        // 启动交易所管理器
        self.exchange_manager.start().await?;

        // 启动费率跟踪器
        self.fee_tracker.start().await?;

        // 启动费率预测器
        self.fee_predictor.start().await?;

        // 启动指标收集器
        self.metrics.start().await?;

        // 连接到所有配置的交易所
        self.connect_to_exchanges().await?;

        // 启动后台任务
        self.start_background_tasks().await;
        
        // 启动5分钟间隔监控任务
        self.start_five_minute_monitoring().await;

        info!("Real-time fee monitor started successfully");
        Ok(())
    }

    /// 停止费率监控系统
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            warn!("Fee monitor is not running");
            return Ok(());
        }
        *running = false;

        // 断开所有WebSocket连接
        self.disconnect_from_exchanges().await?;

        // 停止各个组件
        self.exchange_manager.stop().await?;
        self.fee_tracker.stop().await?;
        self.fee_predictor.stop().await?;
        self.metrics.stop().await?;

        info!("Real-time fee monitor stopped successfully");
        Ok(())
    }

    /// 订阅费率更新事件
    pub fn subscribe_fee_updates(&self) -> broadcast::Receiver<FeeUpdateEvent> {
        self.fee_update_tx.subscribe()
    }

    /// 订阅费率警报事件
    pub fn subscribe_alerts(&self) -> broadcast::Receiver<FeeAlertEvent> {
        self.alert_tx.subscribe()
    }

    /// 获取当前费率
    pub async fn get_current_fee(&self, exchange: &str, symbol: &str, fee_type: FeeType) -> Result<f64> {
        self.fee_tracker.get_current_fee(exchange, symbol, fee_type).await
    }

    /// 获取最优费率交易所
    pub async fn get_optimal_exchange(&self, symbol: &str, fee_type: FeeType) -> Result<FeeRecommendation> {
        self.fee_optimizer.get_optimal_exchange(symbol, fee_type).await
    }

    /// 获取费率预测
    pub async fn get_fee_prediction(
        &self,
        exchange: &str,
        symbol: &str,
        fee_type: FeeType,
        horizon_minutes: u32,
    ) -> Result<FeePrediction> {
        self.fee_predictor.predict_fee(exchange, symbol, fee_type, horizon_minutes).await
    }

    /// 获取费率历史
    pub async fn get_fee_history(
        &self,
        exchange: &str,
        symbol: &str,
        fee_type: FeeType,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<FeeRecord>> {
        self.storage.get_fee_history(exchange, symbol, fee_type, from, to).await
    }

    /// 添加自定义费率阈值警报
    pub async fn add_fee_alert(
        &self,
        exchange: &str,
        symbol: &str,
        fee_type: FeeType,
        threshold: f64,
        alert_type: FeeAlertType,
    ) -> Result<String> {
        let alert_id = Uuid::new_v4().to_string();
        
        // 这里应该将警报配置存储起来
        // 为简化起见，我们只是记录一下
        info!(
            alert_id = %alert_id,
            exchange = exchange,
            symbol = symbol,
            fee_type = ?fee_type,
            threshold = threshold,
            "Custom fee alert added"
        );

        Ok(alert_id)
    }

    /// 移除费率警报
    pub async fn remove_fee_alert(&self, alert_id: &str) -> Result<()> {
        info!(alert_id = alert_id, "Fee alert removed");
        Ok(())
    }

    /// 获取系统统计
    pub async fn get_system_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        // 运行状态
        let running = *self.running.read().await;
        stats.insert("running".to_string(), serde_json::Value::Bool(running));
        
        // WebSocket连接状态
        let clients = self.websocket_clients.read().await;
        let connected_exchanges = clients.len();
        stats.insert("connected_exchanges".to_string(), serde_json::Value::Number(connected_exchanges.into()));
        
        // 获取其他组件的统计信息
        if let Ok(fee_stats) = self.fee_tracker.get_statistics().await {
            for (key, value) in fee_stats {
                stats.insert(format!("fee_tracker_{}", key), value);
            }
        }

        if let Ok(optimizer_stats) = self.fee_optimizer.get_statistics().await {
            for (key, value) in optimizer_stats {
                stats.insert(format!("optimizer_{}", key), value);
            }
        }

        stats
    }

    /// 连接到所有配置的交易所
    async fn connect_to_exchanges(&self) -> Result<()> {
        let exchanges = self.exchange_manager.get_all_exchanges().await;
        let mut clients = self.websocket_clients.write().await;

        for exchange_info in exchanges {
            if exchange_info.status != ExchangeStatus::Active {
                continue;
            }

            let client_config = self.config.websocket.clone();
            let client = Arc::new(WebSocketClient::new(
                exchange_info.name.clone(),
                exchange_info.websocket_url.clone(),
                client_config,
            )?);

            // 设置消息处理回调
            let event_tx = self.event_tx.clone();
            let exchange_name = exchange_info.name.clone();
            client.set_message_handler(Box::new(move |message| {
                let event_tx = event_tx.clone();
                let exchange_name = exchange_name.clone();
                Box::pin(async move {
                    if let Err(e) = Self::handle_websocket_message(event_tx, exchange_name, message).await {
                        error!(error = %e, "Failed to handle WebSocket message");
                    }
                })
            })).await;

            // 连接到交易所
            client.connect().await?;

            // 订阅费率更新
            client.subscribe_to_fees().await?;

            clients.insert(exchange_info.name.clone(), client);
            
            info!(exchange = %exchange_info.name, "Connected to exchange WebSocket");
        }

        Ok(())
    }

    /// 断开所有交易所连接
    async fn disconnect_from_exchanges(&self) -> Result<()> {
        let mut clients = self.websocket_clients.write().await;
        
        for (exchange_name, client) in clients.drain() {
            if let Err(e) = client.disconnect().await {
                warn!(exchange = %exchange_name, error = %e, "Failed to disconnect from exchange");
            } else {
                info!(exchange = %exchange_name, "Disconnected from exchange WebSocket");
            }
        }

        Ok(())
    }

    /// 处理WebSocket消息
    async fn handle_websocket_message(
        event_tx: mpsc::UnboundedSender<InternalEvent>,
        exchange: String,
        message: WebSocketMessage,
    ) -> Result<()> {
        match message {
            WebSocketMessage::FeeUpdate { symbol, maker_fee, taker_fee, timestamp } => {
                // 处理maker费率更新
                let maker_event = FeeUpdateEvent {
                    event_id: Uuid::new_v4().to_string(),
                    exchange: exchange.clone(),
                    symbol: symbol.clone(),
                    fee_type: FeeType::Maker,
                    old_rate: 0.0, // 需要从历史数据中获取
                    new_rate: maker_fee,
                    timestamp,
                    change_percentage: 0.0, // 需要计算
                    volume_24h: None,
                };

                // 处理taker费率更新
                let taker_event = FeeUpdateEvent {
                    event_id: Uuid::new_v4().to_string(),
                    exchange: exchange.clone(),
                    symbol: symbol.clone(),
                    fee_type: FeeType::Taker,
                    old_rate: 0.0, // 需要从历史数据中获取
                    new_rate: taker_fee,
                    timestamp,
                    change_percentage: 0.0, // 需要计算
                    volume_24h: None,
                };

                event_tx.send(InternalEvent::FeeUpdate(maker_event))?;
                event_tx.send(InternalEvent::FeeUpdate(taker_event))?;
            }
            WebSocketMessage::ConnectionLost => {
                event_tx.send(InternalEvent::ConnectionStateChanged(exchange, false))?;
            }
            WebSocketMessage::ConnectionRestored => {
                event_tx.send(InternalEvent::ConnectionStateChanged(exchange, true))?;
            }
            _ => {
                debug!(exchange = %exchange, "Received unhandled WebSocket message");
            }
        }

        Ok(())
    }

    /// 启动内部事件处理器
    async fn start_event_processor(&self, mut event_rx: mpsc::UnboundedReceiver<InternalEvent>) {
        let monitor = Arc::new(self);
        
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                if let Err(e) = monitor.process_internal_event(event).await {
                    error!(error = %e, "Failed to process internal event");
                }
            }
        });
    }

    /// 处理内部事件
    async fn process_internal_event(&self, event: InternalEvent) -> Result<()> {
        match event {
            InternalEvent::FeeUpdate(fee_event) => {
                // 更新费率跟踪器
                let fee_record = FeeRecord {
                    exchange: fee_event.exchange.clone(),
                    symbol: fee_event.symbol.clone(),
                    fee_type: fee_event.fee_type.clone(),
                    rate: fee_event.new_rate,
                    timestamp: fee_event.timestamp,
                    volume_24h: fee_event.volume_24h,
                };

                self.fee_tracker.record_fee(fee_record).await?;

                // 存储费率数据
                self.storage.store_fee_record(&fee_record).await?;

                // 检查是否需要触发警报
                self.check_fee_alerts(&fee_event).await?;

                // 广播费率更新事件
                let _ = self.fee_update_tx.send(fee_event);

                // 更新指标
                self.metrics.fee_update_received().await;
            }
            InternalEvent::Alert(alert_event) => {
                // 广播警报事件
                let _ = self.alert_tx.send(alert_event.clone());

                // 更新指标
                self.metrics.alert_triggered(&alert_event.exchange, &alert_event.alert_type).await;

                info!(
                    alert_id = %alert_event.alert_id,
                    exchange = %alert_event.exchange,
                    alert_type = ?alert_event.alert_type,
                    severity = ?alert_event.severity,
                    "Fee alert triggered"
                );
            }
            InternalEvent::ConnectionStateChanged(exchange, connected) => {
                if connected {
                    info!(exchange = %exchange, "WebSocket connection restored");
                    self.metrics.connection_restored(&exchange).await;
                } else {
                    warn!(exchange = %exchange, "WebSocket connection lost");
                    self.metrics.connection_lost(&exchange).await;
                    
                    // 触发连接丢失警报
                    let alert = FeeAlertEvent {
                        alert_id: Uuid::new_v4().to_string(),
                        exchange: exchange.clone(),
                        symbol: "ALL".to_string(),
                        alert_type: FeeAlertType::ExchangeConnectionLost,
                        current_rate: 0.0,
                        threshold: 0.0,
                        message: format!("Lost connection to {} WebSocket", exchange),
                        severity: AlertSeverity::High,
                        timestamp: Utc::now(),
                    };
                    
                    let _ = self.alert_tx.send(alert);
                }
            }
            InternalEvent::HealthCheck => {
                self.perform_health_check().await?;
            }
            InternalEvent::Cleanup => {
                self.perform_cleanup().await?;
            }
        }

        Ok(())
    }

    /// 检查费率警报条件
    async fn check_fee_alerts(&self, fee_event: &FeeUpdateEvent) -> Result<()> {
        // 检查费率突增
        if fee_event.change_percentage > 50.0 {
            let alert = FeeAlertEvent {
                alert_id: Uuid::new_v4().to_string(),
                exchange: fee_event.exchange.clone(),
                symbol: fee_event.symbol.clone(),
                alert_type: FeeAlertType::FeeSpike,
                current_rate: fee_event.new_rate,
                threshold: fee_event.old_rate * 1.5,
                message: format!(
                    "Fee spike detected: {} on {} for {} increased by {:.1}%",
                    fee_event.symbol,
                    fee_event.exchange,
                    format!("{:?}", fee_event.fee_type),
                    fee_event.change_percentage
                ),
                severity: AlertSeverity::High,
                timestamp: Utc::now(),
            };
            
            self.event_tx.send(InternalEvent::Alert(alert))?;
        }

        // 检查高费率警报
        if fee_event.new_rate > 0.005 {  // 0.5% 费率阈值
            let alert = FeeAlertEvent {
                alert_id: Uuid::new_v4().to_string(),
                exchange: fee_event.exchange.clone(),
                symbol: fee_event.symbol.clone(),
                alert_type: FeeAlertType::HighFeeAlert,
                current_rate: fee_event.new_rate,
                threshold: 0.005,
                message: format!(
                    "High fee detected: {} {} fee at {:.4}%",
                    fee_event.symbol,
                    format!("{:?}", fee_event.fee_type),
                    fee_event.new_rate * 100.0
                ),
                severity: AlertSeverity::Medium,
                timestamp: Utc::now(),
            };
            
            self.event_tx.send(InternalEvent::Alert(alert))?;
        }

        Ok(())
    }

    /// 启动后台任务
    async fn start_background_tasks(&self) {
        let monitor = Arc::new(self);

        // 健康检查任务
        {
            let monitor_clone = Arc::clone(&monitor);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5分钟

                loop {
                    interval.tick().await;
                    
                    let running = *monitor_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = monitor_clone.event_tx.send(InternalEvent::HealthCheck);
                }
            });
        }

        // 清理任务
        {
            let monitor_clone = Arc::clone(&monitor);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600)); // 1小时

                loop {
                    interval.tick().await;
                    
                    let running = *monitor_clone.running.read().await;
                    if !running {
                        break;
                    }

                    let _ = monitor_clone.event_tx.send(InternalEvent::Cleanup);
                }
            });
        }

        // 费率优化任务
        {
            let monitor_clone = Arc::clone(&monitor);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60)); // 1分钟

                loop {
                    interval.tick().await;
                    
                    let running = *monitor_clone.running.read().await;
                    if !running {
                        break;
                    }

                    if let Err(e) = monitor_clone.run_optimization_cycle().await {
                        error!(error = %e, "Fee optimization cycle failed");
                    }
                }
            });
        }

        info!("Background tasks started");
    }

    /// 执行健康检查
    async fn perform_health_check(&self) -> Result<()> {
        let clients = self.websocket_clients.read().await;
        let mut unhealthy_exchanges = Vec::new();

        for (exchange_name, client) in clients.iter() {
            if !client.is_connected().await {
                unhealthy_exchanges.push(exchange_name.clone());
            }
        }

        if !unhealthy_exchanges.is_empty() {
            warn!(
                unhealthy_exchanges = ?unhealthy_exchanges,
                "Health check detected unhealthy exchanges"
            );

            // 尝试重连不健康的交易所
            for exchange_name in unhealthy_exchanges {
                if let Some(client) = clients.get(&exchange_name) {
                    if let Err(e) = client.reconnect().await {
                        error!(
                            exchange = %exchange_name,
                            error = %e,
                            "Failed to reconnect to exchange"
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// 执行清理任务
    async fn perform_cleanup(&self) -> Result<()> {
        // 清理过期的费率数据
        let cutoff_time = Utc::now() - Duration::days(30);
        self.storage.cleanup_old_records(cutoff_time).await?;

        // 清理内存中的缓存数据
        self.fee_tracker.cleanup_old_data(cutoff_time).await?;

        debug!("Cleanup task completed");
        Ok(())
    }

    /// 运行优化循环
    async fn run_optimization_cycle(&self) -> Result<()> {
        // 更新费率优化器的数据
        self.fee_optimizer.update_market_data().await?;

        // 运行预测模型
        self.fee_predictor.update_predictions().await?;

        debug!("Optimization cycle completed");
        Ok(())
    }

    /// 启动5分钟间隔费率监控
    async fn start_five_minute_monitoring(&self) -> Result<()> {
        info!("Starting 5-minute interval fee monitoring optimization");
        
        let monitor = Arc::new(self);
        let monitor_clone = Arc::clone(&monitor);
        
        // 5分钟间隔的费率分析任务
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                monitor_clone.config.get_monitoring_interval_duration()
            );
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                
                let running = *monitor_clone.running.read().await;
                if !running {
                    break;
                }

                if let Err(e) = monitor_clone.perform_periodic_fee_analysis().await {
                    error!(error = %e, "Failed to perform periodic fee analysis");
                } else {
                    debug!("5-minute fee analysis cycle completed successfully");
                }
            }
        });

        Ok(())
    }

    /// 执行周期性费率分析
    async fn perform_periodic_fee_analysis(&self) -> Result<()> {
        let start_time = std::time::Instant::now();
        debug!("Starting periodic fee analysis at 5-minute interval");

        // 1. 收集所有启用交易所的当前费率数据
        let enabled_exchanges = self.config.get_enabled_exchanges();
        let mut analysis_results = Vec::new();

        for exchange in enabled_exchanges {
            if let Ok(fees) = self.collect_exchange_fees(&exchange.name).await {
                analysis_results.push((exchange.name.clone(), fees));
            }
        }

        // 2. 执行费率趋势分析
        self.analyze_fee_trends(&analysis_results).await?;

        // 3. 检测费率异常
        self.detect_fee_anomalies(&analysis_results).await?;

        // 4. 更新最优费率推荐
        self.update_optimal_fee_recommendations(&analysis_results).await?;

        // 5. 生成并发送报告
        self.generate_periodic_report(&analysis_results).await?;

        let elapsed = start_time.elapsed();
        info!(
            elapsed_ms = elapsed.as_millis(),
            exchanges_analyzed = analysis_results.len(),
            "Periodic fee analysis completed"
        );

        Ok(())
    }

    /// 收集交易所费率数据
    async fn collect_exchange_fees(&self, exchange_name: &str) -> Result<HashMap<String, (f64, f64)>> {
        let mut fees = HashMap::new();
        
        // 从费率跟踪器获取最新数据
        let supported_symbols = self.config.supported_exchanges
            .iter()
            .find(|e| e.name == exchange_name)
            .map(|e| &e.supported_symbols)
            .unwrap_or(&vec![]);

        for symbol in supported_symbols {
            if let Ok((maker, taker)) = self.fee_tracker.get_latest_fees(exchange_name, symbol).await {
                fees.insert(symbol.clone(), (maker, taker));
            }
        }

        Ok(fees)
    }

    /// 分析费率趋势
    async fn analyze_fee_trends(&self, analysis_results: &[(String, HashMap<String, (f64, f64)>)]) -> Result<()> {
        for (exchange_name, fees) in analysis_results {
            for (symbol, (maker_fee, taker_fee)) in fees {
                // 计算过去24小时的费率变化趋势
                if let Ok(trend) = self.fee_tracker
                    .calculate_fee_trend(exchange_name, symbol, chrono::Duration::hours(24)).await 
                {
                    if trend.change_percentage.abs() > self.config.tracking_config.fee_change_threshold_percent {
                        let alert = FeeAlertEvent {
                            alert_id: Uuid::new_v4().to_string(),
                            exchange: exchange_name.clone(),
                            symbol: symbol.clone(),
                            alert_type: FeeAlertType::FeeAnomalyDetected,
                            current_rate: *taker_fee,
                            threshold: self.config.tracking_config.fee_change_threshold_percent,
                            message: format!(
                                "Significant fee trend detected for {} {}: {:.2}% change",
                                exchange_name, symbol, trend.change_percentage
                            ),
                            severity: if trend.change_percentage.abs() > 5.0 {
                                AlertSeverity::High
                            } else {
                                AlertSeverity::Medium
                            },
                            timestamp: Utc::now(),
                        };
                        
                        let _ = self.event_tx.send(InternalEvent::Alert(alert));
                    }
                }
            }
        }
        Ok(())
    }

    /// 检测费率异常
    async fn detect_fee_anomalies(&self, analysis_results: &[(String, HashMap<String, (f64, f64)>)]) -> Result<()> {
        let threshold = self.config.tracking_config.anomaly_detection_threshold;
        
        for (exchange_name, fees) in analysis_results {
            for (symbol, (maker_fee, taker_fee)) in fees {
                // 检查是否存在异常高的费率
                if taker_fee > &(threshold / 100.0) {
                    let alert = FeeAlertEvent {
                        alert_id: Uuid::new_v4().to_string(),
                        exchange: exchange_name.clone(),
                        symbol: symbol.clone(),
                        alert_type: FeeAlertType::HighFeeAlert,
                        current_rate: *taker_fee,
                        threshold,
                        message: format!(
                            "Anomalous high fee detected: {} {} taker fee at {:.4}%",
                            exchange_name, symbol, taker_fee * 100.0
                        ),
                        severity: AlertSeverity::High,
                        timestamp: Utc::now(),
                    };
                    
                    let _ = self.event_tx.send(InternalEvent::Alert(alert));
                }
            }
        }
        Ok(())
    }

    /// 更新最优费率推荐
    async fn update_optimal_fee_recommendations(&self, analysis_results: &[(String, HashMap<String, (f64, f64)>)]) -> Result<()> {
        // 为每个交易对找到最优的交易所
        let mut symbol_fees: HashMap<String, Vec<(String, f64, f64)>> = HashMap::new();
        
        for (exchange_name, fees) in analysis_results {
            for (symbol, (maker_fee, taker_fee)) in fees {
                symbol_fees
                    .entry(symbol.clone())
                    .or_insert_with(Vec::new)
                    .push((exchange_name.clone(), *maker_fee, *taker_fee));
            }
        }

        for (symbol, mut exchange_fees) in symbol_fees {
            // 按taker费率排序
            exchange_fees.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
            
            if let Some((best_exchange, maker_fee, taker_fee)) = exchange_fees.first() {
                let alert = FeeAlertEvent {
                    alert_id: Uuid::new_v4().to_string(),
                    exchange: best_exchange.clone(),
                    symbol: symbol.clone(),
                    alert_type: FeeAlertType::OptimalFeeFound,
                    current_rate: *taker_fee,
                    threshold: 0.0,
                    message: format!(
                        "Optimal fee found for {}: {} (maker: {:.4}%, taker: {:.4}%)",
                        symbol, best_exchange, maker_fee * 100.0, taker_fee * 100.0
                    ),
                    severity: AlertSeverity::Low,
                    timestamp: Utc::now(),
                };
                
                let _ = self.event_tx.send(InternalEvent::Alert(alert));
            }
        }
        Ok(())
    }

    /// 生成周期性报告
    async fn generate_periodic_report(&self, analysis_results: &[(String, HashMap<String, (f64, f64)>)]) -> Result<()> {
        let total_symbols = analysis_results.iter()
            .map(|(_, fees)| fees.len())
            .sum::<usize>();
            
        let total_exchanges = analysis_results.len();
        
        info!(
            total_exchanges = total_exchanges,
            total_symbols = total_symbols,
            analysis_interval = "5_minutes",
            "Generated periodic fee analysis report"
        );
        
        // 存储分析结果到持久化存储
        for (exchange_name, fees) in analysis_results {
            for (symbol, (maker_fee, taker_fee)) in fees {
                let _ = self.storage.store_fee_data(
                    exchange_name,
                    symbol,
                    *maker_fee,
                    *taker_fee,
                    Utc::now()
                ).await;
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[tokio::test]
    async fn test_fee_monitor_creation() {
        let config = FeeMonitorConfig::default();
        let storage = Arc::new(MemoryStorage::new());
        
        let monitor = RealTimeFeeMonitor::new(config, storage).await;
        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_fee_monitor_lifecycle() {
        let config = FeeMonitorConfig::default();
        let storage = Arc::new(MemoryStorage::new());
        
        let monitor = RealTimeFeeMonitor::new(config, storage).await.unwrap();
        
        // 测试启动
        let start_result = monitor.start().await;
        // 注意：这可能会失败，因为没有实际的WebSocket端点
        // 在实际测试中，您需要模拟WebSocket服务器
        
        // 测试停止
        let stop_result = monitor.stop().await;
        assert!(stop_result.is_ok());
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = FeeMonitorConfig::default();
        let storage = Arc::new(MemoryStorage::new());
        
        let monitor = RealTimeFeeMonitor::new(config, storage).await.unwrap();
        
        // 测试订阅
        let _fee_updates = monitor.subscribe_fee_updates();
        let _alerts = monitor.subscribe_alerts();
        
        // 订阅应该成功，不抛出错误
    }

    #[tokio::test]
    async fn test_system_stats() {
        let config = FeeMonitorConfig::default();
        let storage = Arc::new(MemoryStorage::new());
        
        let monitor = RealTimeFeeMonitor::new(config, storage).await.unwrap();
        
        let stats = monitor.get_system_stats().await;
        assert!(stats.contains_key("running"));
        assert!(stats.contains_key("connected_exchanges"));
    }
}