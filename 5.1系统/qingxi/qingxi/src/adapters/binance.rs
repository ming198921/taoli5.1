#![allow(dead_code)]
//! # Binance交易所适配器
//!
//! 提供与Binance交易所API交互的适配器实现。

use super::{ExchangeAdapter, MarketDataError, SubscriptionDetail};
use crate::types::{MarketSourceConfig, OrderBook, OrderBookEntry, TradeSide, TradeUpdate};
use crate::MarketDataMessage;
use async_trait::async_trait;
use ordered_float::OrderedFloat;
use serde_json::{json, Value};
use std::str::FromStr;
use tokio_tungstenite::tungstenite::Message;

pub struct BinanceAdapter {
    websocket_url: String,
    rest_api_url: Option<String>,
}

impl Default for BinanceAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceAdapter {
    pub fn new() -> Self {
        // 使用配置化的默认值
        use crate::settings::Settings;
        if let Ok(settings) = Settings::load() {
            if let Some(binance_config) = settings.sources.iter().find(|s| s.exchange_id == "binance") {
                return Self::new_with_config(binance_config).unwrap_or_else(|_| {
                    // 回退到硬编码默认值（仅在配置加载失败时使用）
                    Self {
                        websocket_url: "wss://stream.binance.com:9443/ws".to_string(),
                        rest_api_url: Some("https://api.binance.com/api/v3".to_string()),
                    }
                });
            }
        }
        
        // 回退到硬编码默认值（仅在配置加载失败时使用）
        Self {
            websocket_url: "wss://stream.binance.com:9443/ws".to_string(),
            rest_api_url: Some("https://api.binance.com/api/v3".to_string()),
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
}

#[async_trait]
impl ExchangeAdapter for BinanceAdapter {
    fn exchange_id(&self) -> &str {
        "binance"
    }

    fn build_subscription_messages(
        &self,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Vec<Message>, MarketDataError> {
        let streams: Vec<String> = subscriptions
            .iter()
            .map(|sub| match sub.channel.as_str() {
                "orderbook" => Ok(format!(
                    "{}@depth",
                    sub.symbol.as_pair().replace("/", "").to_lowercase()
                )),
                "trades" => Ok(format!(
                    "{}@trade",
                    sub.symbol.as_pair().replace("/", "").to_lowercase()
                )),
                other => Err(MarketDataError::Configuration(format!(
                    "Unsupported channel type for Binance: {other}"
                ))),
            })
            .collect::<Result<_, _>>()?;
        let sub_msg = json!({
            "method": "SUBSCRIBE",
            "params": streams,
            "id": 1
        });
        Ok(vec![Message::Text(sub_msg.to_string())])
    }

    fn parse_message(
        &self,
        message: &Message,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Option<MarketDataMessage>, MarketDataError> {
        if let Message::Text(text) = message {
            let data: Value = serde_json::from_str(text)?;

            // Skip subscription confirmations and control messages
            if data.get("result").is_some() || data.get("id").is_some() {
                return Ok(None);
            }

            // Handle different Binance WebSocket message formats
            let stream_name: String;
            let data_obj = if let Some(stream) = data.get("stream") {
                // Multi-stream format: {"stream": "btcusdt@depth", "data": {...}}
                stream_name = stream.as_str().unwrap_or_default().to_string();
                data.get("data").ok_or_else(|| MarketDataError::Parse {
                    exchange: self.exchange_id().to_string(),
                    details: "Missing 'data' field in multi-stream message".to_string(),
                })?
            } else if data.get("e").is_some() {
                // Single stream format: {"e": "depthUpdate", "s": "BTCUSDT", ...}
                let event_type = data.get("e").and_then(Value::as_str).unwrap_or("");
                let symbol = data.get("s").and_then(Value::as_str).unwrap_or("").to_lowercase();
                stream_name = if event_type == "depthUpdate" { 
                    format!("{}@depth", symbol)
                } else if event_type == "trade" { 
                    format!("{}@trade", symbol)
                } else { 
                    format!("{}@{}", symbol, event_type)
                };
                &data
            } else {
                return Ok(None); // Unknown message format
            };

            let exchange_symbol = stream_name.split('@').next().unwrap_or_default();

            // Find matching subscription
            let sub_detail = subscriptions
                .iter()
                .find(|s| s.symbol.as_pair().replace("/", "").to_lowercase() == exchange_symbol)
                .ok_or_else(|| {
                    MarketDataError::Configuration(format!(
                        "Message for unsubscribed stream: {stream_name}"
                    ))
                })?;

            let symbol = sub_detail.symbol.clone();

            return if stream_name.contains("@depth") {
                let bids_raw: Vec<Vec<String>> = serde_json::from_value(
                    data_obj.get("b").or_else(|| data_obj.get("bids")).cloned()
                        .unwrap_or(serde_json::Value::Array(vec![]))
                )?;
                let asks_raw: Vec<Vec<String>> = serde_json::from_value(
                    data_obj.get("a").or_else(|| data_obj.get("asks")).cloned()
                        .unwrap_or(serde_json::Value::Array(vec![]))
                )?;

                let bids: Vec<OrderBookEntry> = bids_raw
                    .into_iter()
                    .filter_map(|level| {
                        if level.len() >= 2 {
                            Some(OrderBookEntry::new(
                                level[0].parse().ok()?,
                                level[1].parse().ok()?,
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();

                let asks: Vec<OrderBookEntry> = asks_raw
                    .into_iter()
                    .filter_map(|level| {
                        if level.len() >= 2 {
                            Some(OrderBookEntry::new(
                                level[0].parse().ok()?,
                                level[1].parse().ok()?,
                            ))
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(Some(MarketDataMessage::OrderBook(OrderBook {
                    symbol,
                    source: self.exchange_id().to_string(),
                    bids,
                    asks,
                    timestamp: crate::high_precision_time::Nanos::from_millis(
                        data_obj["E"].as_i64().unwrap_or(0),
                    ),
                    sequence_id: Some(data_obj["u"].as_u64().unwrap_or(0)),
                    checksum: None,
                })))
            } else if stream_name.contains("@trade") {
                Ok(Some(MarketDataMessage::Trade(TradeUpdate {
                    symbol,
                    source: self.exchange_id().to_string(),
                    price: OrderedFloat(f64::from_str(data_obj["p"].as_str().unwrap_or("0"))?),
                    quantity: OrderedFloat(f64::from_str(data_obj["q"].as_str().unwrap_or("0"))?),
                    side: if data_obj["m"].as_bool().unwrap_or(false) {
                        TradeSide::Sell
                    } else {
                        TradeSide::Buy
                    },
                    timestamp: crate::high_precision_time::Nanos::from_millis(
                        data_obj["T"].as_i64().unwrap_or(0),
                    ),
                    trade_id: data_obj["t"].as_u64().map(|id| id.to_string()),
                })))
            } else {
                Ok(None)
            };
        }
        Ok(None)
    }

    fn get_heartbeat_request(&self) -> Option<Message> {
        None
    }
    fn is_heartbeat(&self, _message: &Message) -> bool {
        false
    }
    fn get_heartbeat_response(&self, _message: &Message) -> Option<Message> {
        None
    }

    async fn get_initial_snapshot(
        &self,
        subscription: &SubscriptionDetail,
        rest_api_url: &str,
    ) -> Result<MarketDataMessage, MarketDataError> {
        let symbol_upper = subscription.symbol.as_pair().replace("/", "");
        let url = format!("{rest_api_url}/depth?symbol={symbol_upper}&limit=1000");

        let res = reqwest::get(&url)
            .await
            .map_err(|e| MarketDataError::Connection {
                exchange: self.exchange_id().to_string(),
                details: e.to_string(),
            })?;

        let snapshot_data: Value = res.json().await.map_err(|e| MarketDataError::Parse {
            exchange: self.exchange_id().to_string(),
            details: e.to_string(),
        })?;

        let bids: Vec<OrderBookEntry> = serde_json::from_value(snapshot_data["bids"].clone())?;
        let asks: Vec<OrderBookEntry> = serde_json::from_value(snapshot_data["asks"].clone())?;

        Ok(MarketDataMessage::OrderBook(OrderBook {
            symbol: subscription.symbol.clone(),
            source: self.exchange_id().to_string(),
            bids,
            asks,
            timestamp: crate::high_precision_time::Nanos::now(),
            sequence_id: snapshot_data.get("lastUpdateId").and_then(Value::as_u64),
            checksum: None,
        }))
    }
}
