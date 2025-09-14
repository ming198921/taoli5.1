#![allow(dead_code)]
// src/collector/websocket_collector.rs
use super::super::{
    adapters::ExchangeAdapter, circuit_breaker::CircuitBreaker, errors::MarketDataError,
    health::ApiHealthMonitor, high_precision_time::Nanos,
    orderbook::local_orderbook::MarketDataMessage, types::MarketSourceConfig,
    settings::WebSocketNetworkSettings,
};
use futures_util::{stream::StreamExt, SinkExt};
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::protocol::WebSocketConfig};
use tracing::{debug, info, instrument, warn};

pub struct WebsocketCollector {
    config: MarketSourceConfig,
    adapter: Arc<dyn ExchangeAdapter>,
    data_tx: flume::Sender<MarketDataMessage>,
    #[allow(dead_code)]
    circuit_breaker: Arc<CircuitBreaker>,
    health_monitor: Arc<ApiHealthMonitor>,
    network_settings: WebSocketNetworkSettings,
    reconnect_attempts: u32,
}

impl WebsocketCollector {
    pub fn new(
        config: MarketSourceConfig,
        adapter: Arc<dyn ExchangeAdapter>,
        data_tx: flume::Sender<MarketDataMessage>,
        health_monitor: Arc<ApiHealthMonitor>,
        network_settings: WebSocketNetworkSettings,
    ) -> Self {
        Self {
            config,
            adapter,
            data_tx,
            circuit_breaker: Arc::new(CircuitBreaker::new(3, Duration::from_secs(30))),
            health_monitor,
            network_settings,
            reconnect_attempts: 0,
        }
    }

    #[instrument(name="collector_run", skip(self, shutdown_rx), fields(exchange = %self.config.exchange_id.to_string()))]
    pub async fn run(mut self, mut shutdown_rx: broadcast::Receiver<()>) {
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Shutdown signal received, stopping WebSocket collector");
                    break;
                }
                result = self.connect_and_stream() => {
                    match result {
                        Ok(_) => {
                            info!("WebSocket connection ended normally");
                            self.reconnect_attempts = 0; // 重置重连计数
                        }
                        Err(e) => {
                            self.reconnect_attempts += 1;
                            
                            if self.reconnect_attempts > self.network_settings.max_reconnect_attempts {
                                warn!(
                                    "Maximum reconnection attempts ({}) exceeded for exchange {}, stopping collector",
                                    self.network_settings.max_reconnect_attempts,
                                    self.config.exchange_id
                                );
                                break;
                            }
                            
                            let delay = self.network_settings.get_exponential_backoff_delay(self.reconnect_attempts - 1);
                            warn!(
                                "WebSocket connection failed: {e}, retrying in {:?} (attempt {}/{})",
                                delay,
                                self.reconnect_attempts,
                                self.network_settings.max_reconnect_attempts
                            );
                            
                            // 在重试期间也监听关闭信号
                            tokio::select! {
                                _ = shutdown_rx.recv() => {
                                    info!("Shutdown signal received during retry delay");
                                    break;
                                }
                                _ = tokio::time::sleep(delay) => {
                                    // 继续重试
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
        info!("WebSocket collector stopped gracefully");
    }

    async fn connect_and_stream(&self) -> Result<(), MarketDataError> {
        info!("Attempting to connect with advanced network settings...");
        
        // 创建WebSocket配置
        let _ws_config = WebSocketConfig {
            max_frame_size: Some(self.network_settings.max_frame_size),
            max_message_size: Some(self.network_settings.max_frame_size),
            // 注意：max_send_queue已废弃，使用默认配置
            ..WebSocketConfig::default()
        };
        
        // 使用连接超时的异步连接
        let connection_result = tokio::time::timeout(
            self.network_settings.get_connection_timeout(),
            connect_async(&self.config.websocket_url)
        ).await;
        
        let (ws_stream, _) = match connection_result {
            Ok(Ok((stream, response))) => {
                info!("Connection established successfully");
                (stream, response)
            }
            Ok(Err(e)) => {
                return Err(MarketDataError::Connection {
                    exchange: self.config.exchange_id.clone(),
                    details: format!("WebSocket connection failed: {}", e),
                });
            }
            Err(_) => {
                return Err(MarketDataError::Connection {
                    exchange: self.config.exchange_id.clone(),
                    details: format!("Connection timeout after {:?}", self.network_settings.get_connection_timeout()),
                });
            }
        };
        
        info!("Connection established. Subscribing...");

        // 更新连接状态
        let source_id = self.config.exchange_id.to_string();
        self.health_monitor
            .update_connection_status(&source_id, true);

        let (mut write, mut read) = ws_stream.split();

        // 获取初始快照 (如果需要)
        for symbol_str in &self.config.symbols {
            // 将字符串转换为 Symbol
            let symbol = match crate::types::Symbol::from_string(symbol_str) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Failed to parse symbol '{}': {}", symbol_str, e);
                    continue;
                }
            };
            
            let sub = crate::types::SubscriptionDetail {
                symbol,
                channel: self.config.channel.clone(),
            };
            if let Some(url) = &self.config.rest_api_url {
                match self.adapter.get_initial_snapshot(&sub, url).await {
                    Ok(snapshot) => {
                        // 直接使用从适配器返回的MarketDataMessage，无需转换
                        let local_msg = snapshot;
                        if self.data_tx.send_async(local_msg).await.is_err() {
                            return Err(MarketDataError::InternalError(
                                "Data channel closed".into(),
                            ));
                        }
                    }
                    Err(MarketDataError::UnsupportedOperation(_)) => { /* Not all adapters support this */
                    }
                    Err(e) => warn!(error = %e, "Failed to get initial snapshot"),
                }
            }
        }

        // 发送订阅消息
        let subscriptions: Vec<_> = self
            .config
            .symbols
            .iter()
            .map(|symbol_str| {
                // 将字符串转换为 Symbol
                let symbol = match crate::types::Symbol::from_string(symbol_str) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Failed to parse symbol '{}': {}", symbol_str, e);
                        return None;
                    }
                };
                
                Some(crate::types::SubscriptionDetail {
                    symbol,
                    channel: self.config.channel.clone(),
                })
            })
            .filter_map(|x| x) // 过滤掉 None 值
            .collect();

        let sub_msgs = self.adapter.build_subscription_messages(&subscriptions)?;
        for msg in sub_msgs {
            write
                .send(msg)
                .await
                .map_err(|e| MarketDataError::Communication {
                    exchange: self.config.exchange_id.clone(),
                    details: e.to_string(),
                })?;
        }
        info!("Subscription messages sent. Starting message loop.");

        // 使用配置的心跳间隔
        let mut heartbeat_interval = tokio::time::interval(self.network_settings.get_heartbeat_interval());

        // 核心消息循环 - 添加读取和写入超时
        loop {
            tokio::select! {
                msg_result = tokio::time::timeout(
                    self.network_settings.get_read_timeout(),
                    read.next()
                ) => {
                    match msg_result {
                        Ok(Some(msg_result)) => {
                            let msg = msg_result.map_err(|e| MarketDataError::Communication { 
                                exchange: self.config.exchange_id.clone(), 
                                details: e.to_string() 
                            })?;

                            // 计算延迟并更新健康状态
                            let _receive_time = Nanos::now();
                            let source_id = self.config.exchange_id.to_string();

                            if self.adapter.is_heartbeat(&msg) {
                                debug!("Heartbeat received/handled.");
                                self.health_monitor.update_message_received(&source_id, 0); // 心跳延迟设为0
                                if let Some(response) = self.adapter.get_heartbeat_response(&msg) {
                                    // 添加写入超时
                                    let write_result = tokio::time::timeout(
                                        self.network_settings.get_write_timeout(),
                                        write.send(response)
                                    ).await;
                                    
                                    match write_result {
                                        Ok(Ok(_)) => {},
                                        Ok(Err(e)) => {
                                            return Err(MarketDataError::Communication { 
                                                exchange: self.config.exchange_id.clone(), 
                                                details: format!("Failed to send heartbeat response: {}", e) 
                                            });
                                        }
                                        Err(_) => {
                                            return Err(MarketDataError::Communication { 
                                                exchange: self.config.exchange_id.clone(), 
                                                details: format!("Heartbeat response write timeout after {:?}", self.network_settings.get_write_timeout()) 
                                            });
                                        }
                                    }
                                }
                                continue;
                            }

                            if let Some(market_message) = self.adapter.parse_message(&msg, &subscriptions)? {
                                // 估算延迟（简化实现，实际应用中可能需要从消息中提取服务器时间戳）
                                let estimated_latency_us = 1000; // 1ms作为估算值
                                self.health_monitor.update_message_received(&source_id, estimated_latency_us);

                                // 直接使用适配器返回的MarketDataMessage
                                if self.data_tx.send_async(market_message).await.is_err() {
                                    return Err(MarketDataError::InternalError("Data channel closed".into()));
                                }
                            }
                        },
                        Ok(None) => {
                            warn!("WebSocket stream ended.");
                            // 更新连接状态为断开
                            let source_id = self.config.exchange_id.to_string();
                            self.health_monitor.update_connection_status(&source_id, false);
                            break;
                        },
                        Err(_) => {
                            return Err(MarketDataError::Communication {
                                exchange: self.config.exchange_id.clone(),
                                details: format!("Read timeout after {:?}", self.network_settings.get_read_timeout()),
                            });
                        }
                    }
                },
                _ = heartbeat_interval.tick() => {
                    if let Some(hb_msg) = self.adapter.get_heartbeat_request() {
                        debug!("Sending heartbeat request.");
                        // 添加写入超时到心跳请求
                        let write_result = tokio::time::timeout(
                            self.network_settings.get_write_timeout(),
                            write.send(hb_msg)
                        ).await;
                        
                        match write_result {
                            Ok(Ok(_)) => {},
                            Ok(Err(e)) => {
                                return Err(MarketDataError::Communication { 
                                    exchange: self.config.exchange_id.clone(), 
                                    details: format!("Failed to send heartbeat: {}", e) 
                                });
                            }
                            Err(_) => {
                                return Err(MarketDataError::Communication { 
                                    exchange: self.config.exchange_id.clone(), 
                                    details: format!("Heartbeat write timeout after {:?}", self.network_settings.get_write_timeout()) 
                                });
                            }
                        }
                    }
                },
                else => {
                    warn!("WebSocket stream ended unexpectedly.");
                    // 更新连接状态为断开
                    let source_id = self.config.exchange_id.to_string();
                    self.health_monitor.update_connection_status(&source_id, false);
                    break;
                }
            }
        }
        Ok(())
    }
}
