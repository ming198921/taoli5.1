#![allow(dead_code)]
//! # Bybit 交易所适配器
//!
//! 基于 Bybit V5 WebSocket API 的高性能市场数据适配器

use super::ExchangeAdapter;
use crate::types::*;
use crate::errors::MarketDataError;
use crate::{MarketDataMessage, OrderedFloat};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info, warn};
use tokio_tungstenite::tungstenite::Message;

/// Bybit 适配器实现
pub struct BybitAdapter {
    websocket_url: String,
    rest_api_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BybitOrderBookResponse {
    #[allow(dead_code)]
    topic: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    msg_type: String,
    #[allow(dead_code)]
    ts: u64,
    data: BybitOrderBookData,
}

#[derive(Debug, Deserialize)]
struct BybitOrderBookData {
    s: String,        // symbol
    b: Vec<[String; 2]>, // bids [price, size]
    a: Vec<[String; 2]>, // asks [price, size]
    #[allow(dead_code)]
    u: u64,           // update_id
    seq: u64,         // sequence
}

#[derive(Debug, Deserialize)]
struct BybitTradeResponse {
    #[allow(dead_code)]
    topic: String,
    #[serde(rename = "type")]
    #[allow(dead_code)]
    msg_type: String,
    #[allow(dead_code)]
    ts: u64,
    data: Vec<BybitTradeData>,
}

#[derive(Debug, Deserialize)]
struct BybitTradeData {
    #[serde(rename = "T")]
    timestamp: u64,
    s: String,        // symbol
    #[serde(rename = "S")]
    side: String,     // Buy/Sell
    v: String,        // volume
    p: String,        // price
    #[serde(rename = "L", default)]
    trade_id: Option<String>,  // 可选的交易ID字段
    #[serde(rename = "i", default)]
    alt_trade_id: Option<String>,  // 备用交易ID字段
    // 添加其他可能的字段以防解析错误
    #[serde(flatten)]
    other: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for BybitAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BybitAdapter {
    /// 创建新的 Bybit 适配器
    pub fn new() -> Self {
        // 使用配置化的默认值
        use crate::settings::Settings;
        if let Ok(settings) = Settings::load() {
            if let Some(bybit_config) = settings.sources.iter().find(|s| s.exchange_id == "bybit") {
                return Self::new_with_config(bybit_config).unwrap_or_else(|_| {
                    // 回退到硬编码默认值（仅在配置加载失败时使用）
                    Self {
                        websocket_url: "wss://stream.bybit.com/v5/public/spot".to_string(),
                        rest_api_url: Some("https://api.bybit.com".to_string()),
                    }
                });
            }
        }
        
        // 回退到硬编码默认值（仅在配置加载失败时使用）
        Self {
            websocket_url: "wss://stream.bybit.com/v5/public/spot".to_string(),
            rest_api_url: Some("https://api.bybit.com".to_string()),
        }
    }

    /// 使用自定义配置创建 Bybit 适配器
    pub fn new_with_config(config: &MarketSourceConfig) -> Result<Self, anyhow::Error> {
        // 检查API凭证是否为占位符
        if let Some(api_key) = &config.api_key {
            if api_key.contains("PLACEHOLDER") || api_key.contains("YOUR_") {
                warn!("⚠️ WARNING: API Key for Bybit appears to be a placeholder: {}. Operating in limited functionality mode.", api_key);
            }
        }
        
        if let Some(secret_key) = &config.api_secret {
            if secret_key.contains("PLACEHOLDER") || secret_key.contains("YOUR_") {
                warn!("⚠️ WARNING: Secret Key for Bybit appears to be a placeholder. Operating in limited functionality mode.");
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

    /// 将 Bybit 符号转换为标准符号
    fn parse_symbol(&self, bybit_symbol: &str) -> Option<Symbol> {
        // Bybit 使用 BTCUSDT 格式，需要转换为 BTC/USDT
        if let Some(base) = bybit_symbol.strip_suffix("USDT") {
            Some(Symbol::new(base, "USDT"))
        } else if let Some(base) = bybit_symbol.strip_suffix("USD") {
            Some(Symbol::new(base, "USD"))
        } else {
            warn!("Unknown Bybit symbol format: {}", bybit_symbol);
            None
        }
    }

    /// 将标准符号转换为 Bybit 符号
    fn format_symbol(&self, symbol: &Symbol) -> String {
        format!("{}{}", symbol.base, symbol.quote)
    }

    /// 解析订单簿数据
    fn parse_orderbook(&self, data: &BybitOrderBookData) -> Result<OrderBook, MarketDataError> {
        let symbol = self.parse_symbol(&data.s)
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid symbol: {}", data.s),
            })?;

        let mut bids = Vec::new();
        for bid in &data.b {
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

        let mut asks = Vec::new();
        for ask in &data.a {
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

        // 确保价格排序正确
        bids.sort_by(|a, b| b.price.cmp(&a.price)); // 降序
        asks.sort_by(|a, b| a.price.cmp(&b.price)); // 升序

        Ok(OrderBook {
            symbol,
            source: "bybit".to_string(),
            bids,
            asks,
            timestamp: crate::high_precision_time::Nanos::now(),
            sequence_id: Some(data.seq),
            checksum: None,
        })
    }

    /// 解析交易数据
    fn parse_trade(&self, trade_data: &BybitTradeData) -> Result<TradeUpdate, MarketDataError> {
        let symbol = self.parse_symbol(&trade_data.s)
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid symbol: {}", trade_data.s),
            })?;

        let price = trade_data.p.parse::<f64>()
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid trade price: {}", e),
            })?;
        let quantity = trade_data.v.parse::<f64>()
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid trade quantity: {}", e),
            })?;

        let side = match trade_data.side.as_str() {
            "Buy" => TradeSide::Buy,
            "Sell" => TradeSide::Sell,
            _ => return Err(MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid trade side: {}", trade_data.side),
            }),
        };

        // 尝试获取交易ID，使用多个可能的字段
        let trade_id = trade_data.trade_id
            .clone()
            .or_else(|| trade_data.alt_trade_id.clone())
            .or_else(|| {
                // 尝试从其他字段中获取ID
                trade_data.other.get("i")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });

        Ok(TradeUpdate {
            symbol,
            source: "bybit".to_string(),
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
            side,
            timestamp: crate::high_precision_time::Nanos::from_millis(trade_data.timestamp as i64),
            trade_id,
        })
    }
}

#[async_trait]
impl ExchangeAdapter for BybitAdapter {
    fn exchange_id(&self) -> &str {
        "bybit"
    }

    fn build_subscription_messages(
        &self,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Vec<Message>, MarketDataError> {
        let mut topics = Vec::new();
        
        for sub in subscriptions {
            let bybit_symbol = self.format_symbol(&sub.symbol);
            // 订阅50档订单簿和交易数据 - 修复深度问题
            topics.push(format!("orderbook.50.{}", bybit_symbol));
            topics.push(format!("publicTrade.{}", bybit_symbol));
        }

        let subscribe_msg = json!({
            "op": "subscribe",
            "args": topics
        });

        Ok(vec![Message::Text(subscribe_msg.to_string())])
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

        let value: Value = serde_json::from_str(text)
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("JSON parse error: {}", e),
            })?;

        // 检查是否为心跳或确认消息
        if let Some(op) = value.get("op").and_then(|v| v.as_str()) {
            match op {
                "pong" => {
                    debug!("Received pong from Bybit");
                    return Ok(Some(MarketDataMessage::Heartbeat {
                        source: "bybit".to_string(),
                        timestamp: crate::high_precision_time::Nanos::now(),
                    }));
                }
                "subscribe" => {
                    if let Some(success) = value.get("success").and_then(|v| v.as_bool()) {
                        if success {
                            info!("Bybit subscription successful");
                        } else {
                            warn!("Bybit subscription failed: {:?}", value);
                        }
                    }
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }

        // 检查是否为数据消息
        if let Some(topic) = value.get("topic").and_then(|v| v.as_str()) {
            if topic.starts_with("orderbook.") {
                // 解析订单簿数据
                let response: BybitOrderBookResponse = serde_json::from_value(value)
                    .map_err(|e| MarketDataError::Parse {
                        exchange: self.exchange_id().to_string(),
                        details: format!("Failed to parse orderbook: {}", e),
                    })?;

                let orderbook = self.parse_orderbook(&response.data)?;
                return Ok(Some(MarketDataMessage::OrderBookSnapshot(orderbook)));
            } else if topic.starts_with("publicTrade.") {
                // 解析交易数据 - 只返回第一个交易
                let response: BybitTradeResponse = serde_json::from_value(value)
                    .map_err(|e| MarketDataError::Parse {
                        exchange: self.exchange_id().to_string(),
                        details: format!("Failed to parse trade: {}", e),
                    })?;

                if let Some(trade_data) = response.data.first() {
                    let trade = self.parse_trade(trade_data)?;
                    return Ok(Some(MarketDataMessage::Trade(trade)));
                }
            }
        }

        debug!("Ignoring unknown Bybit message: {}", text);
        Ok(None)
    }

    fn get_heartbeat_request(&self) -> Option<Message> {
        Some(Message::Text(json!({"op": "ping"}).to_string()))
    }

    fn is_heartbeat(&self, message: &Message) -> bool {
        if let Message::Text(text) = message {
            if let Ok(value) = serde_json::from_str::<Value>(text) {
                if let Some(op) = value.get("op").and_then(|v| v.as_str()) {
                    return op == "pong" || op == "ping";
                }
            }
        }
        false
    }

    async fn get_initial_snapshot(
        &self,
        subscription: &SubscriptionDetail,
        rest_api_url: &str,
    ) -> Result<MarketDataMessage, MarketDataError> {
        let client = reqwest::Client::new();
        let symbol = self.format_symbol(&subscription.symbol);
        
        // 使用传入的REST API URL或配置中的URL
        let base_url = if rest_api_url.is_empty() {
            self.rest_api_url().unwrap_or("https://api.bybit.com")
        } else {
            rest_api_url
        };
        
        let url = format!("{}/v5/market/orderbook?category=spot&symbol={}&limit=200", base_url, symbol);

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Value>().await {
                        Ok(data) => {
                            if let Some(result) = data.get("result") {
                                let empty_vec = vec![];
                                let bids_raw = result.get("b").and_then(|b| b.as_array()).unwrap_or(&empty_vec);
                                let asks_raw = result.get("a").and_then(|a| a.as_array()).unwrap_or(&empty_vec);

                                let mut bids = Vec::new();
                                for bid in bids_raw {
                                    if let Some(arr) = bid.as_array() {
                                        if arr.len() >= 2 {
                                            if let (Some(price_str), Some(qty_str)) = (arr[0].as_str(), arr[1].as_str()) {
                                                if let (Ok(price), Ok(qty)) = (price_str.parse::<f64>(), qty_str.parse::<f64>()) {
                                                    bids.push(OrderBookEntry {
                                                        price: OrderedFloat(price),
                                                        quantity: OrderedFloat(qty),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }

                                let mut asks = Vec::new();
                                for ask in asks_raw {
                                    if let Some(arr) = ask.as_array() {
                                        if arr.len() >= 2 {
                                            if let (Some(price_str), Some(qty_str)) = (arr[0].as_str(), arr[1].as_str()) {
                                                if let (Ok(price), Ok(qty)) = (price_str.parse::<f64>(), qty_str.parse::<f64>()) {
                                                    asks.push(OrderBookEntry {
                                                        price: OrderedFloat(price),
                                                        quantity: OrderedFloat(qty),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }

                                return Ok(MarketDataMessage::OrderBookSnapshot(OrderBook {
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
                        Err(e) => {
                            warn!("Failed to parse Bybit snapshot response: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Failed to fetch Bybit snapshot: {}", e);
            }
        }

        // 返回空的快照作为fallback
        Ok(MarketDataMessage::OrderBookSnapshot(OrderBook {
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

impl std::fmt::Display for BybitAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BybitAdapter(bybit)")
    }
}
