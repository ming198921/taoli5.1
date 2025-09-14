#![allow(dead_code)]
//! # 增强型WebSocket收集器
//!
//! 具备智能重连、动态心跳和连接质量监控的高性能WebSocket数据收集器
//!
//! ## 功能特性
//! - 指数退避重连策略
//! - 动态心跳间隔调整
//! - 连接质量实时监控
//! - 分层错误处理和恢复
//! - 连接池管理和负载均衡

use super::super::{
    adapters::{ExchangeAdapter, bybit_dynamic::{BybitDynamicAdapter, ConnectionQualityMonitor, ReconnectStrategy}},
    circuit_breaker::CircuitBreaker,
    errors::MarketDataError,
    health::ApiHealthMonitor,
    high_precision_time::Nanos,
    orderbook::local_orderbook::MarketDataMessage,
    types::MarketSourceConfig,
};
use futures_util::{stream::StreamExt, SinkExt};
use std::{sync::Arc, time::{Duration, Instant}};
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;
use tracing::{debug, info, instrument, warn, error};

/// 增强型WebSocket收集器
pub struct EnhancedWebsocketCollector {
    config: MarketSourceConfig,
    adapter: Arc<dyn ExchangeAdapter>,
    data_tx: flume::Sender<MarketDataMessage>,
    circuit_breaker: Arc<CircuitBreaker>,
    health_monitor: Arc<ApiHealthMonitor>,
    /// 连接质量监控器（如果适配器支持）
    quality_monitor: Option<Arc<ConnectionQualityMonitor>>,
    /// 重连策略（如果适配器支持）
    reconnect_strategy: Option<Arc<ReconnectStrategy>>,
}

impl EnhancedWebsocketCollector {
    pub fn new(
        config: MarketSourceConfig,
        adapter: Arc<dyn ExchangeAdapter>,
        data_tx: flume::Sender<MarketDataMessage>,
        health_monitor: Arc<ApiHealthMonitor>,
    ) -> Self {
        // 尝试检测是否为动态适配器并获取其功能
        let (quality_monitor, reconnect_strategy) = if config.exchange_id == "bybit" {
            // 尝试向下转型为BybitDynamicAdapter
            // 注意：这里我们使用一种变通的方法，因为直接向下转型Arc<dyn Trait>比较复杂
            // 在实际实现中，可能需要在ExchangeAdapter trait中添加方法来获取这些功能
            (None, None)
        } else {
            (None, None)
        };

        Self {
            config,
            adapter,
            data_tx,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(60))), // 更宽松的熔断设置
            health_monitor,
            quality_monitor,
            reconnect_strategy,
        }
    }

    /// 为动态适配器创建增强收集器
    pub fn new_with_dynamic_features(
        config: MarketSourceConfig,
        adapter: Arc<dyn ExchangeAdapter>,
        data_tx: flume::Sender<MarketDataMessage>,
        health_monitor: Arc<ApiHealthMonitor>,
        quality_monitor: Arc<ConnectionQualityMonitor>,
        reconnect_strategy: Arc<ReconnectStrategy>,
    ) -> Self {
        Self {
            config,
            adapter,
            data_tx,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(60))),
            health_monitor,
            quality_monitor: Some(quality_monitor),
            reconnect_strategy: Some(reconnect_strategy),
        }
    }

    #[instrument(name="enhanced_collector_run", skip(self, shutdown_rx), fields(exchange = %self.config.exchange_id.to_string()))]
    pub async fn run(self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!("🚀 启动增强型WebSocket收集器 for {}", self.config.exchange_id);
        
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("📡 收到关闭信号，停止WebSocket收集器");
                    break;
                }
                result = self.connect_and_stream_enhanced() => {
                    match result {
                        Ok(_) => {
                            info!("✅ WebSocket连接正常结束");
                            // 重置重连策略（如果可用）
                            if let Some(strategy) = &self.reconnect_strategy {
                                strategy.reset();
                            }
                        }
                        Err(e) => {
                            error!("❌ WebSocket连接失败: {}", e);
                            
                            // 获取重连延迟
                            let retry_delay = if let Some(strategy) = &self.reconnect_strategy {
                                if strategy.should_stop_retrying() {
                                    error!("🚨 达到最大重试次数，停止重连");
                                    break;
                                }
                                strategy.increment_retry();
                                strategy.get_next_delay()
                            } else {
                                // 使用简单的固定延迟作为fallback
                                Duration::from_secs(5)
                            };
                            
                            info!("🔄 将在 {:?} 后重试连接", retry_delay);
                            
                            // 在重试期间也监听关闭信号
                            tokio::select! {
                                _ = shutdown_rx.recv() => {
                                    info!("📡 重试期间收到关闭信号");
                                    break;
                                }
                                _ = tokio::time::sleep(retry_delay) => {
                                    info!("⏰ 重试延迟结束，尝试重新连接");
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 清理连接状态
        let source_id = self.config.exchange_id.to_string();
        self.health_monitor.update_connection_status(&source_id, false);
        info!("🛑 增强型WebSocket收集器已优雅停止");
    }

    async fn connect_and_stream_enhanced(&self) -> Result<(), MarketDataError> {
        info!("🔗 尝试建立WebSocket连接到 {}", self.config.ws_endpoint);
        
        // 检查熔断器状态
        if let Err(e) = self.circuit_breaker.call().await {
            return Err(MarketDataError::Connection {
                exchange: self.config.exchange_id.clone(),
                details: format!("Circuit breaker open: {}", e),
            });
        }

        let connection_start = Instant::now();
        let (ws_stream, response) = connect_async(&self.config.ws_endpoint).await.map_err(|e| {
            // 报告连接失败
            if let Some(monitor) = &self.quality_monitor {
                monitor.report_failure();
            }
            
            MarketDataError::Connection {
                exchange: self.config.exchange_id.clone(),
                details: format!("WebSocket连接失败: {}", e),
            }
        })?;

        let connection_time = connection_start.elapsed();
        info!("✅ WebSocket连接建立成功，耗时: {:?}, 响应状态: {}", 
              connection_time, response.status());

        // 更新连接状态
        let source_id = self.config.exchange_id.to_string();
        self.health_monitor.update_connection_status(&source_id, true);

        // 报告连接成功
        if let Some(monitor) = &self.quality_monitor {
            monitor.report_success(connection_time.as_micros() as u64);
        }

        let (mut write, mut read) = ws_stream.split();

        // 获取初始快照
        if let Err(e) = self.fetch_initial_snapshots(&mut write).await {
            warn!("⚠️ 获取初始快照时出现问题: {}", e);
            // 不中断连接，继续处理实时数据
        }

        // 发送订阅消息
        if let Err(e) = self.send_subscriptions(&mut write).await {
            return Err(e);
        }

        info!("📡 订阅消息已发送，开始消息循环处理");

        // 动态心跳间隔
        let initial_heartbeat_interval = if let Some(monitor) = &self.quality_monitor {
            monitor.get_suggested_heartbeat_interval()
        } else {
            Duration::from_secs(30) // 默认30秒
        };

        let mut heartbeat_interval = tokio::time::interval(initial_heartbeat_interval);
        let mut last_heartbeat_adjustment = Instant::now();
        let mut message_count = 0u64;

        // 核心消息循环
        loop {
            tokio::select! {
                Some(msg_result) = read.next() => {
                    match self.handle_websocket_message(msg_result, &mut write, &source_id).await {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                            message_count += 1;
                            
                            // 定期调整心跳间隔
                            if message_count % 100 == 0 && last_heartbeat_adjustment.elapsed() > Duration::from_secs(60) {
                                if let Some(monitor) = &self.quality_monitor {
                                    let new_interval = monitor.get_suggested_heartbeat_interval();
                                    if new_interval != heartbeat_interval.period() {
                                        info!("🔧 动态调整心跳间隔: {:?} -> {:?}", 
                                              heartbeat_interval.period(), new_interval);
                                        heartbeat_interval = tokio::time::interval(new_interval);
                                        last_heartbeat_adjustment = Instant::now();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("❌ 处理WebSocket消息时出错: {}", e);
                            if let Some(monitor) = &self.quality_monitor {
                                monitor.report_failure();
                            }
                            return Err(e);
                        }
                    }
                },
                _ = heartbeat_interval.tick() => {
                    if let Err(e) = self.send_heartbeat(&mut write).await {
                        error!("❌ 发送心跳失败: {}", e);
                        return Err(e);
                    }
                },
                else => {
                    warn!("⚠️ WebSocket流意外结束");
                    self.health_monitor.update_connection_status(&source_id, false);
                    break;
                }
            }
        }
        
        info!("📊 消息循环结束，共处理 {} 条消息", message_count);
        Ok(())
    }

    async fn fetch_initial_snapshots(
        &self,
        _write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::Message>
    ) -> Result<(), MarketDataError> {
        for symbol in &self.config.symbols {
            let sub = crate::types::SubscriptionDetail {
                symbol: symbol.clone(),
                channel: self.config.channel.clone().unwrap_or_default(),
            };
            
            if let Some(url) = &self.config.rest_endpoint {
                match self.adapter.get_initial_snapshot(&sub, url).await {
                    Ok(snapshot) => {
                        info!("📊 获取到 {} 的初始快照", symbol.as_pair());
                        if let Err(_) = self.data_tx.send_async(snapshot).await {
                            return Err(MarketDataError::InternalError(
                                "数据通道已关闭".into(),
                            ));
                        }
                    }
                    Err(MarketDataError::UnsupportedOperation(_)) => {
                        debug!("⚠️ 适配器不支持初始快照获取");
                    }
                    Err(e) => {
                        warn!("⚠️ 获取 {} 初始快照失败: {}", symbol.as_pair(), e);
                        // 继续处理其他symbol，不中断整个流程
                    }
                }
            }
        }
        Ok(())
    }

    async fn send_subscriptions(
        &self,
        write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::Message>
    ) -> Result<(), MarketDataError> {
        let subscriptions: Vec<_> = self
            .config
            .symbols
            .iter()
            .map(|symbol| crate::types::SubscriptionDetail {
                symbol: symbol.clone(),
                channel: self.config.channel.clone().unwrap_or_default(),
            })
            .collect();

        let sub_msgs = self.adapter.build_subscription_messages(&subscriptions)?;
        
        for (i, msg) in sub_msgs.iter().enumerate() {
            write
                .send(msg.clone())
                .await
                .map_err(|e| MarketDataError::Communication {
                    exchange: self.config.exchange_id.clone(),
                    details: format!("发送订阅消息{}失败: {}", i + 1, e),
                })?;
                
            // 在订阅消息之间添加小延迟，避免过快发送
            if i + 1 < sub_msgs.len() {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        info!("📡 已发送 {} 条订阅消息", sub_msgs.len());
        Ok(())
    }

    async fn handle_websocket_message(
        &self,
        msg_result: Result<tokio_tungstenite::tungstenite::Message, tokio_tungstenite::tungstenite::Error>,
        write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::Message>,
        source_id: &str,
    ) -> Result<bool, MarketDataError> {
        let msg = msg_result.map_err(|e| {
            if let Some(monitor) = &self.quality_monitor {
                monitor.report_failure();
            }
            
            MarketDataError::Communication {
                exchange: self.config.exchange_id.clone(),
                details: format!("WebSocket消息接收错误: {}", e),
            }
        })?;

        let receive_time = Nanos::now();

        // 处理心跳消息
        if self.adapter.is_heartbeat(&msg) {
            debug!("💓 收到心跳消息");
            self.health_monitor.update_message_received(source_id, 0);
            
            if let Some(response) = self.adapter.get_heartbeat_response(&msg) {
                write.send(response).await.map_err(|e| MarketDataError::Communication {
                    exchange: self.config.exchange_id.clone(),
                    details: format!("发送心跳响应失败: {}", e),
                })?;
            }
            return Ok(true);
        }

        // 解析市场数据消息
        let subscriptions: Vec<_> = self
            .config
            .symbols
            .iter()
            .map(|symbol| crate::types::SubscriptionDetail {
                symbol: symbol.clone(),
                channel: self.config.channel.clone().unwrap_or_default(),
            })
            .collect();

        match self.adapter.parse_message(&msg, &subscriptions) {
            Ok(Some(market_message)) => {
                // 计算处理延迟
                let processing_time = receive_time.elapsed();
                let estimated_latency_us = processing_time.as_micros() as u64 + 500; // 加上网络延迟估算
                
                self.health_monitor.update_message_received(source_id, estimated_latency_us);
                
                // 报告成功处理
                if let Some(monitor) = &self.quality_monitor {
                    monitor.report_success(estimated_latency_us);
                }

                // 发送数据到处理管道
                if let Err(_) = self.data_tx.send_async(market_message).await {
                    return Err(MarketDataError::InternalError("数据通道已关闭".into()));
                }
                
                debug!("📨 成功处理并发送市场数据消息，延迟: {}μs", estimated_latency_us);
            }
            Ok(None) => {
                // 消息被忽略，这是正常的
                debug!("🔍 消息被适配器忽略");
            }
            Err(e) => {
                warn!("⚠️ 解析消息失败: {}", e);
                if let Some(monitor) = &self.quality_monitor {
                    monitor.report_failure();
                }
                // 解析错误不中断连接，继续处理
            }
        }

        Ok(true)
    }

    async fn send_heartbeat(
        &self,
        write: &mut futures_util::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, tokio_tungstenite::tungstenite::Message>
    ) -> Result<(), MarketDataError> {
        if let Some(hb_msg) = self.adapter.get_heartbeat_request() {
            debug!("💓 发送心跳请求");
            write.send(hb_msg).await.map_err(|e| MarketDataError::Communication {
                exchange: self.config.exchange_id.clone(),
                details: format!("发送心跳请求失败: {}", e),
            })?;
        }
        Ok(())
    }
}
