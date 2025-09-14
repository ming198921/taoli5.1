#![allow(dead_code)]
//! # Gate.io 交易所适配器
//!
//! 基于 Gate.io WebSocket API v4 的高性能市场数据适配器

use super::ExchangeAdapter;
use crate::types::*;
use crate::errors::MarketDataError;
use crate::{MarketDataMessage, OrderedFloat};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info, warn};
use tokio_tungstenite::tungstenite::Message;
use reqwest;
use chrono;

/// Gate.io 适配器实现
pub struct GateioAdapter {
    websocket_url: String,
    rest_api_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GateioMessage {
    #[allow(dead_code)]
    time: u64,
    channel: String,
    event: String,
    result: Option<GateioResult>,
}

#[derive(Debug, Deserialize)]
struct GateioResult {
    #[serde(rename = "t")]
    timestamp: Option<u64>,
    #[serde(rename = "s")]
    #[allow(dead_code)]
    symbol: Option<String>,
    #[serde(rename = "c")]
    #[allow(dead_code)]
    change: Option<String>,
    #[serde(rename = "v")]
    #[allow(dead_code)]
    volume: Option<String>,
    #[serde(rename = "h")]
    #[allow(dead_code)]
    high: Option<String>,
    #[serde(rename = "l")]
    #[allow(dead_code)]
    low: Option<String>,
    #[serde(rename = "p")]
    #[allow(dead_code)]
    price: Option<String>,
    bids: Option<Vec<[String; 2]>>,
    asks: Option<Vec<[String; 2]>>,
    trades: Option<Vec<GateioTradeData>>,
}

#[derive(Debug, Deserialize)]
struct GateioTradeData {
    id: u64,
    time: u64,
    price: String,
    amount: String,
    side: String, // "buy" or "sell"
}

impl Default for GateioAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl GateioAdapter {
    /// 创建新的 Gate.io 适配器
    pub fn new() -> Self {
        // 使用配置化的默认值
        use crate::settings::Settings;
        if let Ok(settings) = Settings::load() {
            if let Some(gateio_config) = settings.sources.iter().find(|s| s.exchange_id == "gateio") {
                return Self::new_with_config(gateio_config).unwrap_or_else(|_| {
                    // 回退到硬编码默认值（仅在配置加载失败时使用）
                    Self {
                        websocket_url: "wss://api.gateio.ws/ws/v4/".to_string(),
                        rest_api_url: Some("https://api.gateio.ws".to_string()),
                    }
                });
            }
        }
        
        // 回退到硬编码默认值（仅在配置加载失败时使用）
        Self {
            websocket_url: "wss://api.gateio.ws/ws/v4/".to_string(),
            rest_api_url: Some("https://api.gateio.ws".to_string()),
        }
    }

    /// 使用自定义配置创建 Gate.io 适配器
    pub fn new_with_config(config: &MarketSourceConfig) -> Result<Self, anyhow::Error> {
        // 检查API凭证是否为占位符
        if let Some(api_key) = &config.api_key {
            if api_key.contains("PLACEHOLDER") || api_key.contains("YOUR_") {
                warn!("⚠️ WARNING: API Key for Gate.io appears to be a placeholder: {}. Operating in limited functionality mode.", api_key);
            }
        }
        
        if let Some(secret_key) = &config.api_secret {
            if secret_key.contains("PLACEHOLDER") || secret_key.contains("YOUR_") {
                warn!("⚠️ WARNING: Secret Key for Gate.io appears to be a placeholder. Operating in limited functionality mode.");
            }
        }

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

    /// 将 Gate.io 符号转换为标准符号
    fn parse_symbol(&self, gateio_symbol: &str) -> Option<Symbol> {
        // Gate.io 使用 BTC_USDT 格式，需要转换为 BTC/USDT
        let parts: Vec<&str> = gateio_symbol.split('_').collect();
        if parts.len() == 2 {
            Some(Symbol::new(parts[0], parts[1]))
        } else {
            warn!("Unknown Gate.io symbol format: {}", gateio_symbol);
            None
        }
    }

    /// 将标准符号转换为 Gate.io 符号
    fn format_symbol(&self, symbol: &Symbol) -> String {
        format!("{}_{}", symbol.base, symbol.quote)
    }

    /// 解析订单簿数据
    fn parse_orderbook(&self, symbol_str: &str, result: &GateioResult) -> Result<OrderBook, MarketDataError> {
        let symbol = self.parse_symbol(symbol_str)
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid symbol: {}", symbol_str),
            })?;

        let mut bids = Vec::new();
        if let Some(bid_data) = &result.bids {
            for bid in bid_data {
                let price = bid[0].parse::<f64>()
                    .map_err(|e| MarketDataError::Parse {
                        exchange: self.exchange_id().to_string(),
                        details: format!("Invalid bid price: {}", e),
                    })?;
                let quantity = bid[1].parse::<f64>()
                    .map_err(|e| MarketDataError::Parse {
                        exchange: self.exchange_id().to_string(),
                        details: format!("Invalid bid quantity: {}", e),
                    })?;
                
                bids.push(OrderBookEntry {
                    price: OrderedFloat(price),
                    quantity: OrderedFloat(quantity),
                });
            }
        }

        let mut asks = Vec::new();
        if let Some(ask_data) = &result.asks {
            for ask in ask_data {
                let price = ask[0].parse::<f64>()
                    .map_err(|e| MarketDataError::Parse {
                        exchange: self.exchange_id().to_string(),
                        details: format!("Invalid ask price: {}", e),
                    })?;
                let quantity = ask[1].parse::<f64>()
                    .map_err(|e| MarketDataError::Parse {
                        exchange: self.exchange_id().to_string(),
                        details: format!("Invalid ask quantity: {}", e),
                    })?;
                
                asks.push(OrderBookEntry {
                    price: OrderedFloat(price),
                    quantity: OrderedFloat(quantity),
                });
            }
        }

        // 确保价格排序正确
        bids.sort_by(|a, b| b.price.cmp(&a.price)); // 降序
        asks.sort_by(|a, b| a.price.cmp(&b.price)); // 升序

        Ok(OrderBook {
            symbol,
            source: "gateio".to_string(),
            bids,
            asks,
            timestamp: crate::high_precision_time::Nanos::now(),
            sequence_id: result.timestamp,
            checksum: None,
        })
    }

    /// 解析交易数据
    fn parse_trade(&self, symbol_str: &str, trade_data: &GateioTradeData) -> Result<TradeUpdate, MarketDataError> {
        let symbol = self.parse_symbol(symbol_str)
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid symbol: {}", symbol_str),
            })?;

        let price = trade_data.price.parse::<f64>()
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid trade price: {}", e),
            })?;
        let quantity = trade_data.amount.parse::<f64>()
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid trade quantity: {}", e),
            })?;

        let side = match trade_data.side.as_str() {
            "buy" => TradeSide::Buy,
            "sell" => TradeSide::Sell,
            _ => return Err(MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid trade side: {}", trade_data.side),
            }),
        };

        Ok(TradeUpdate {
            symbol,
            source: "gateio".to_string(),
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
            side,
            timestamp: crate::high_precision_time::Nanos::from_millis((trade_data.time * 1000) as i64), // Gate.io 使用秒
            trade_id: Some(trade_data.id.to_string()),
        })
    }
}

#[async_trait]
impl ExchangeAdapter for GateioAdapter {
    fn exchange_id(&self) -> &str {
        "gateio"
    }

    fn build_subscription_messages(
        &self,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Vec<Message>, MarketDataError> {
        let mut messages = Vec::new();
        
        for sub in subscriptions {
            let gateio_symbol = self.format_symbol(&sub.symbol);
            
            // Gate.io 需要分别订阅每个通道
            let orderbook_msg = json!({
                "time": chrono::Utc::now().timestamp(),
                "channel": format!("spot.order_book.{}", gateio_symbol),
                "event": "subscribe",
                "payload": [gateio_symbol.clone()]
            });
            
            messages.push(Message::Text(orderbook_msg.to_string()));
            
            // 可选：同时订阅交易数据
            let trades_msg = json!({
                "time": chrono::Utc::now().timestamp(), 
                "channel": format!("spot.trades.{}", gateio_symbol),
                "event": "subscribe",
                "payload": [gateio_symbol]
            });
            
            messages.push(Message::Text(trades_msg.to_string()));
        }

        Ok(messages)
    }

    fn parse_message(
        &self,
        message: &Message,
        _subscriptions: &[SubscriptionDetail],
    ) -> Result<Option<MarketDataMessage>, MarketDataError> {
        let text = match message {
            Message::Text(text) => text,
            Message::Binary(_) => return Ok(None),
            _ => return Ok(None),
        };

        let msg: GateioMessage = serde_json::from_str(text)
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("JSON parse error: {}", e),
            })?;

        match msg.event.as_str() {
            "subscribe" => {
                info!("Gate.io subscription successful for channel: {}", msg.channel);
                Ok(None)
            }
            "unsubscribe" => {
                info!("Gate.io unsubscription successful for channel: {}", msg.channel);
                Ok(None)
            }
            "update" => {
                if let Some(result) = msg.result {
                    if msg.channel.starts_with("spot.order_book.") {
                        // 解析订单簿数据
                        let symbol_str = msg.channel.strip_prefix("spot.order_book.")
                            .ok_or_else(|| MarketDataError::Parse {
                                exchange: self.exchange_id().to_string(),
                                details: "Invalid orderbook channel".to_string(),
                            })?;
                        
                        let orderbook = self.parse_orderbook(symbol_str, &result)?;
                        Ok(Some(MarketDataMessage::OrderBookSnapshot(orderbook)))
                    } else if msg.channel.starts_with("spot.trades.") {
                        // 解析交易数据 - 只返回第一个交易
                        let symbol_str = msg.channel.strip_prefix("spot.trades.")
                            .ok_or_else(|| MarketDataError::Parse {
                                exchange: self.exchange_id().to_string(),
                                details: "Invalid trades channel".to_string(),
                            })?;
                        
                        if let Some(trades) = result.trades {
                            if let Some(trade_data) = trades.first() {
                                let trade = self.parse_trade(symbol_str, trade_data)?;
                                return Ok(Some(MarketDataMessage::Trade(trade)));
                            }
                        }
                        Ok(None)
                    } else {
                        debug!("Ignoring unknown Gate.io channel: {}", msg.channel);
                        Ok(None)
                    }
                } else {
                    debug!("Gate.io message without result data");
                    Ok(None)
                }
            }
            "ping" => {
                debug!("Received ping from Gate.io");
                Ok(Some(MarketDataMessage::Heartbeat {
                    source: "gateio".to_string(),
                    timestamp: crate::high_precision_time::Nanos::now(),
                }))
            }
            _ => {
                debug!("Unknown Gate.io event: {}", msg.event);
                Ok(None)
            }
        }
    }

    fn get_heartbeat_request(&self) -> Option<Message> {
        Some(Message::Text(json!({
            "time": chrono::Utc::now().timestamp(),
            "channel": "spot.ping",
            "event": "ping"
        }).to_string()))
    }

    fn is_heartbeat(&self, message: &Message) -> bool {
        if let Message::Text(text) = message {
            if let Ok(msg) = serde_json::from_str::<GateioMessage>(text) {
                return msg.event == "ping" || msg.event == "pong";
            }
        }
        false
    }

    async fn get_initial_snapshot(
        &self,
        subscription: &SubscriptionDetail,
        rest_api_url: &str,
    ) -> Result<MarketDataMessage, MarketDataError> {
        let gateio_symbol = self.format_symbol(&subscription.symbol);
        let url = format!("{}/spot/order_book?currency_pair={}&limit=100", rest_api_url, gateio_symbol);
        
        // 使用 reqwest 获取初始快照
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .header("User-Agent", "qingxi-market-data/1.0")
            .send()
            .await
            .map_err(|e| MarketDataError::Communication {
                exchange: "gateio".to_string(),
                details: format!("REST API request failed: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(MarketDataError::Communication {
                exchange: "gateio".to_string(),
                details: format!("REST API returned status: {}", response.status()),
            });
        }

        let json: Value = response.json().await
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Failed to parse REST response: {}", e),
            })?;

        // 解析 REST API 响应到订单簿格式
        let result = GateioResult {
            timestamp: json.get("current").and_then(|v| v.as_u64()),
            symbol: Some(gateio_symbol.clone()),
            change: None,
            volume: None,
            high: None,
            low: None,
            price: None,
            bids: json.get("bids").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter().filter_map(|item| {
                        if let [price, size] = item.as_array()?.as_slice() {
                            Some([price.as_str()?.to_string(), size.as_str()?.to_string()])
                        } else {
                            None
                        }
                    }).collect()
                })
            }),
            asks: json.get("asks").and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter().filter_map(|item| {
                        if let [price, size] = item.as_array()?.as_slice() {
                            Some([price.as_str()?.to_string(), size.as_str()?.to_string()])
                        } else {
                            None
                        }
                    }).collect()
                })
            }),
            trades: None,
        };

        let orderbook = self.parse_orderbook(&gateio_symbol, &result)?;
        Ok(MarketDataMessage::OrderBookSnapshot(orderbook))
    }
}

impl std::fmt::Display for GateioAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GateioAdapter(gateio)")
    }
}
