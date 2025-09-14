#![allow(dead_code)]
//! # OKX交易所适配器
//!
//! 提供与OKX交易所API交互的适配器实现。

use super::{ExchangeAdapter, MarketDataError, SubscriptionDetail};
use crate::types::{MarketSourceConfig, OrderBookEntry};
use crate::MarketDataMessage;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::str::FromStr;
use tokio_tungstenite::tungstenite::Message;

pub struct OkxAdapter {
    websocket_url: String,
    rest_api_url: Option<String>,
}

impl Default for OkxAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl OkxAdapter {
    pub fn new() -> Self {
        // 使用配置化的默认值
        use crate::settings::Settings;
        if let Ok(settings) = Settings::load() {
            if let Some(okx_config) = settings.sources.iter().find(|s| s.exchange_id == "okx") {
                return Self::new_with_config(okx_config).unwrap_or_else(|_| {
                    // 回退到硬编码默认值（仅在配置加载失败时使用）
                    Self {
                        websocket_url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
                        rest_api_url: Some("https://www.okx.com/api/v5".to_string()),
                    }
                });
            }
        }
        
        // 回退到硬编码默认值（仅在配置加载失败时使用）
        Self {
            websocket_url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            rest_api_url: Some("https://www.okx.com/api/v5".to_string()),
        }
    }

    pub fn new_with_config(config: &MarketSourceConfig) -> Result<Self, anyhow::Error> {
        Ok(Self {
            websocket_url: config.get_websocket_url().to_string(),
            rest_api_url: config.get_rest_api_url().map(|s| s.to_string()),
        })
    }

    pub fn websocket_url(&self) -> &str {
        &self.websocket_url
    }

    pub fn rest_api_url(&self) -> Option<&str> {
        self.rest_api_url.as_deref()
    }

    #[allow(dead_code)]
    fn parse_price_qty_pair(&self, v: &Value) -> Result<OrderBookEntry, MarketDataError> {
        let arr = v.as_array().ok_or_else(|| MarketDataError::Parse {
            exchange: self.exchange_id().to_string(),
            details: "Expected price/qty to be an array".to_string(),
        })?;
        let price = f64::from_str(arr.first().and_then(Value::as_str).unwrap_or("0"))?;
        let qty = f64::from_str(arr.get(1).and_then(Value::as_str).unwrap_or("0"))?;
        Ok(OrderBookEntry::new(price, qty))
    }
}

#[async_trait]
impl ExchangeAdapter for OkxAdapter {
    fn exchange_id(&self) -> &str {
        "okx"
    }

    fn build_subscription_messages(
        &self,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Vec<Message>, MarketDataError> {
        let args: Vec<_> = subscriptions
            .iter()
            .map(|sub| match sub.channel.as_str() {
                "orderbook" => Ok(json!({"channel": "books", "instId": sub.symbol.as_pair()})),
                "trades" => Ok(json!({"channel": "trades", "instId": sub.symbol.as_pair()})),
                other => Err(MarketDataError::Configuration(format!(
                    "Unsupported channel type for OKX: {other}"
                ))),
            })
            .collect::<Result<_, _>>()?;
        let sub_msg = json!({"op": "subscribe", "args": args});
        Ok(vec![Message::Text(sub_msg.to_string())])
    }

    fn parse_message(
        &self,
        message: &Message,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Option<MarketDataMessage>, MarketDataError> {
        let text = match message {
            Message::Text(s) => s,
            _ => return Ok(None),
        };
        
        let v: Value = serde_json::from_str(text).map_err(|e| MarketDataError::Parse {
            exchange: self.exchange_id().to_string(),
            details: format!("JSON解析失败: {}", e),
        })?;
        
        // 检查是否是数据推送
        if let Some(data_array) = v.get("data").and_then(|d| d.as_array()) {
            if let Some(data) = data_array.first() {
                if let Some(bids_raw) = data.get("bids").and_then(|b| b.as_array()) {
                    if let Some(asks_raw) = data.get("asks").and_then(|a| a.as_array()) {
                        // 解析订单簿数据
                        let bids: Vec<crate::types::OrderBookEntry> = bids_raw
                            .iter()
                            .filter_map(|level| {
                                if let Some(arr) = level.as_array() {
                                    if arr.len() >= 2 {
                                        let price = arr[0].as_str()?.parse().ok()?;
                                        let qty = arr[1].as_str()?.parse().ok()?;
                                        Some(crate::types::OrderBookEntry::new(price, qty))
                                    } else { None }
                                } else { None }
                            })
                            .collect();
                            
                        let asks: Vec<crate::types::OrderBookEntry> = asks_raw
                            .iter()
                            .filter_map(|level| {
                                if let Some(arr) = level.as_array() {
                                    if arr.len() >= 2 {
                                        let price = arr[0].as_str()?.parse().ok()?;
                                        let qty = arr[1].as_str()?.parse().ok()?;
                                        Some(crate::types::OrderBookEntry::new(price, qty))
                                    } else { None }
                                } else { None }
                            })
                            .collect();
                        
                        // 获取交易对信息
                        if let Some(sub_detail) = subscriptions.first() {
                            return Ok(Some(MarketDataMessage::OrderBook(crate::types::OrderBook {
                                symbol: sub_detail.symbol.clone(),
                                source: self.exchange_id().to_string(),
                                bids,
                                asks,
                                timestamp: crate::high_precision_time::Nanos::now(),
                                sequence_id: data.get("seqId").and_then(|s| s.as_u64()),
                                checksum: data.get("checksum").and_then(|c| c.as_str().map(|s| s.to_string())),
                            })));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }

    fn get_heartbeat_request(&self) -> Option<Message> {
        Some(Message::Text("ping".to_string()))
    }
    
    fn is_heartbeat(&self, message: &Message) -> bool {
        if let Message::Text(text) = message {
            text == "ping" || text == "pong" || text.contains("\"event\":\"ping\"")
        } else {
            false
        }
    }
    
    fn get_heartbeat_response(&self, message: &Message) -> Option<Message> {
        if let Message::Text(text) = message {
            if text == "ping" || text.contains("\"event\":\"ping\"") {
                Some(Message::Text("pong".to_string()))
            } else {
                None
            }
        } else {
            None
        }
    }

    async fn get_initial_snapshot(
        &self,
        subscription: &SubscriptionDetail,
        rest_api_url: &str,
    ) -> Result<MarketDataMessage, MarketDataError> {
        let client = reqwest::Client::new();
        let symbol = subscription.symbol.as_pair();
        let url = format!("{}/market/books?instId={}&sz=20", rest_api_url, symbol);
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            if let Some(data_array) = data.get("data").and_then(|d| d.as_array()) {
                                if let Some(book_data) = data_array.first() {
                                    let empty_bids = vec![];
                                    let empty_asks = vec![];
                                    let bids_raw = book_data.get("bids").and_then(|b| b.as_array()).unwrap_or(&empty_bids);
                                    let asks_raw = book_data.get("asks").and_then(|a| a.as_array()).unwrap_or(&empty_asks);
                                    
                                    let bids: Vec<OrderBookEntry> = bids_raw
                                        .iter()
                                        .filter_map(|level| {
                                            if let Some(arr) = level.as_array() {
                                                if arr.len() >= 2 {
                                                    let price = arr[0].as_str()?.parse().ok()?;
                                                    let qty = arr[1].as_str()?.parse().ok()?;
                                                    Some(OrderBookEntry::new(price, qty))
                                                } else { None }
                                            } else { None }
                                        })
                                        .collect();
                                    
                                    let asks: Vec<OrderBookEntry> = asks_raw
                                        .iter()
                                        .filter_map(|level| {
                                            if let Some(arr) = level.as_array() {
                                                if arr.len() >= 2 {
                                                    let price = arr[0].as_str()?.parse().ok()?;
                                                    let qty = arr[1].as_str()?.parse().ok()?;
                                                    Some(OrderBookEntry::new(price, qty))
                                                } else { None }
                                            } else { None }
                                        })
                                        .collect();
                                    
                                    return Ok(MarketDataMessage::OrderBookSnapshot(crate::types::OrderBook {
                                        symbol: subscription.symbol.clone(),
                                        source: self.exchange_id().to_string(),
                                        bids,
                                        asks,
                                        timestamp: crate::high_precision_time::Nanos::now(),
                                        checksum: None,
                                        sequence_id: None,
                                    }));
                                }
                            }
                        }
                        Err(e) => {
                            return Err(MarketDataError::Parse {
                                exchange: self.exchange_id().to_string(),
                                details: format!("Failed to parse OKX snapshot response: {}", e),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                return Err(MarketDataError::Connection {
                    exchange: self.exchange_id().to_string(),
                    details: format!("Failed to fetch OKX snapshot: {}", e),
                });
            }
        }
        
        // 返回空的快照作为fallback
        Ok(MarketDataMessage::OrderBookSnapshot(crate::types::OrderBook {
            symbol: subscription.symbol.clone(),
            source: self.exchange_id().to_string(),
            bids: vec![],
            asks: vec![],
            timestamp: crate::high_precision_time::Nanos::now(),
            checksum: None,
            sequence_id: None,
        }))
    }
}
