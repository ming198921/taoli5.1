#![allow(dead_code)]
//! # Huobi Adapter
use super::{ExchangeAdapter, MarketDataError, SubscriptionDetail};
use crate::{
    simd_utils,
    types::{MarketSourceConfig, OrderBook, OrderBookEntry},
    MarketDataMessage,
};
use async_trait::async_trait;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde_json::{json, Value};
use std::io::{Read, Write};
use tokio_tungstenite::tungstenite::Message;

pub struct HuobiAdapter {
    websocket_url: String,
    rest_api_url: Option<String>,
}

impl Default for HuobiAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl HuobiAdapter {
    pub fn new() -> Self {
        // 使用配置化的默认值
        use crate::settings::Settings;
        if let Ok(settings) = Settings::load() {
            if let Some(huobi_config) = settings.sources.iter().find(|s| s.exchange_id == "huobi") {
                return Self::new_with_config(huobi_config).unwrap_or_else(|_| {
                    // 回退到硬编码默认值（仅在配置加载失败时使用）
                    Self {
                        websocket_url: "wss://api.huobi.pro/ws".to_string(),
                        rest_api_url: Some("https://api.huobi.pro".to_string()),
                    }
                });
            }
        }
        
        // 回退到硬编码默认值（仅在配置加载失败时使用）
        Self {
            websocket_url: "wss://api.huobi.pro/ws".to_string(),
            rest_api_url: Some("https://api.huobi.pro".to_string()),
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
impl ExchangeAdapter for HuobiAdapter {
    fn exchange_id(&self) -> &str {
        "huobi"
    }

    fn build_subscription_messages(
        &self,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Vec<Message>, MarketDataError> {
        subscriptions
            .iter()
            .map(|sub| {
                // Huobi 使用不带分隔符的小写符号格式
                let symbol = format!("{}{}", 
                    sub.symbol.base.to_lowercase(), 
                    sub.symbol.quote.to_lowercase()
                );
                let topic = match sub.channel.as_str() {
                    "orderbook" => format!("market.{}.depth.step0", symbol),
                    "trades" => format!("market.{}.trade.detail", symbol),
                    other => {
                        return Err(MarketDataError::Configuration(format!(
                            "Unsupported channel type for Huobi: {other}"
                        )))
                    }
                };
                let msg = json!({ 
                    "sub": topic, 
                    "id": format!("qingxi_{}", symbol)
                });
                tracing::info!("Huobi subscription message: {}", msg.to_string());
                Ok(Message::Text(msg.to_string()))
            })
            .collect()
    }

    fn parse_message(
        &self,
        message: &Message,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Option<MarketDataMessage>, MarketDataError> {
        let data = match message {
            Message::Binary(data) => data,
            Message::Text(text) => {
                // 处理可能的文本消息（连接确认等）
                if let Ok(json_data) = serde_json::from_str::<Value>(text) {
                    if json_data.get("subbed").is_some() {
                        tracing::info!("Huobi subscription confirmed: {}", text);
                        return Ok(None);
                    }
                }
                return Ok(None);
            },
            _ => return Ok(None),
        };

        // 解压缩消息
        let mut decoder = GzDecoder::new(&data[..]);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Gzip解压失败: {e}"),
            })?;

        // 检查ping消息
        if decompressed.starts_with(b"{\"ping\"") {
            tracing::debug!("Huobi ping message received, will respond with pong");
            return Ok(None);
        }

        let json_data: Value = simd_utils::parse_json(self.exchange_id(), &mut decompressed)?;
        
        // 处理ping消息
        if let Some(ping_value) = json_data.get("ping") {
            tracing::debug!("Huobi ping ID: {:?}", ping_value);
            return Ok(None); // ping消息交由心跳处理
        }
        
        // 处理订阅确认消息
        if json_data.get("subbed").is_some() {
            tracing::info!("Huobi subscription confirmed: {:?}", json_data);
            return Ok(None);
        }
        
        // 检查是否有tick数据
        let tick = match json_data.get("tick") {
            Some(t) => t,
            None => {
                tracing::debug!("Huobi message without tick data: {:?}", json_data);
                return Ok(None);
            }
        };

        // 解析订单簿数据
        let bids_raw =
            tick.get("bids")
                .and_then(|v| v.as_array())
                .ok_or_else(|| MarketDataError::Parse {
                    exchange: self.exchange_id().to_string(),
                    details: "缺少bids字段".to_string(),
                })?;
        let asks_raw =
            tick.get("asks")
                .and_then(|v| v.as_array())
                .ok_or_else(|| MarketDataError::Parse {
                    exchange: self.exchange_id().to_string(),
                    details: "缺少asks字段".to_string(),
                })?;
        let bids: Vec<OrderBookEntry> = bids_raw
            .iter()
            .filter_map(|level| {
                let price = level.get(0)?.as_f64()?;
                let qty = level.get(1)?.as_f64()?;
                Some(OrderBookEntry::new(price, qty))
            })
            .collect();
        let asks: Vec<OrderBookEntry> = asks_raw
            .iter()
            .filter_map(|level| {
                let price = level.get(0)?.as_f64()?;
                let qty = level.get(1)?.as_f64()?;
                Some(OrderBookEntry::new(price, qty))
            })
            .collect();
        let sub_detail = subscriptions
            .first()
            .ok_or_else(|| MarketDataError::Configuration("Huobi解析缺少订阅信息".to_string()))?;
        Ok(Some(MarketDataMessage::OrderBook(OrderBook {
            symbol: sub_detail.symbol.clone(),
            source: self.exchange_id().to_string(),
            timestamp: crate::high_precision_time::Nanos::from_millis(
                json_data.get("ts").and_then(Value::as_i64).unwrap_or(0),
            ),
            sequence_id: tick.get("version").and_then(Value::as_u64),
            bids,
            asks,
            checksum: None,
        })))
    }

    fn get_heartbeat_request(&self) -> Option<Message> {
        None // Huobi通常由服务器发送ping，客户端响应pong
    }
    
    fn is_heartbeat(&self, message: &Message) -> bool {
        if let Message::Binary(data) = message {
            // Huobi的ping消息是压缩的JSON
            let mut decoder = GzDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_ok() {
                if let Ok(text) = std::str::from_utf8(&decompressed) {
                    // 检查是否包含ping字段
                    return text.contains("\"ping\":") || text.starts_with("{\"ping\"");
                }
            }
        }
        false
    }
    
    fn get_heartbeat_response(&self, message: &Message) -> Option<Message> {
        if let Message::Binary(data) = message {
            let mut decoder = GzDecoder::new(&data[..]);
            let mut decompressed = Vec::new();
            if decoder.read_to_end(&mut decompressed).is_ok() {
                if let Ok(text) = std::str::from_utf8(&decompressed) {
                    if let Ok(v) = serde_json::from_str::<Value>(text) {
                        if let Some(ping_id) = v.get("ping") {
                            let pong_msg = json!({"pong": ping_id});
                            // 压缩响应
                            let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::default());
                            if encoder.write_all(pong_msg.to_string().as_bytes()).is_ok() {
                                if let Ok(compressed) = encoder.finish() {
                                    return Some(Message::Binary(compressed));
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    async fn get_initial_snapshot(
        &self,
        subscription: &SubscriptionDetail,
        rest_api_url: &str,
    ) -> Result<MarketDataMessage, MarketDataError> {
        // 实现Huobi的REST API调用获取初始快照
        use reqwest;
        
        // Huobi 使用不带分隔符的小写符号格式
        let symbol_pair = format!("{}{}", 
            subscription.symbol.base.to_lowercase(), 
            subscription.symbol.quote.to_lowercase()
        );
        
        // 使用传入的 rest_api_url 而不是硬编码
        let base_url = if rest_api_url.is_empty() {
            self.rest_api_url.as_deref().unwrap_or("https://api.huobi.pro")
        } else {
            rest_api_url
        };
        let url = format!("{}/market/depth?symbol={}&type=step0", base_url, symbol_pair);
        
        tracing::info!("Huobi snapshot request URL: {}", url);
        
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| MarketDataError::Connection {
                exchange: self.exchange_id().to_string(),
                details: format!("HTTP请求失败: {}", e),
            })?;
            
        let response_text = response.text()
            .await
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("响应读取失败: {}", e),
            })?;
            
        tracing::info!("Huobi snapshot response: {}", response_text);
            
        let json_data: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("JSON解析失败: {} - 响应内容: {}", e, response_text),
            })?;
            
        // 检查 Huobi API 响应状态
        if let Some(status) = json_data.get("status").and_then(|s| s.as_str()) {
            if status != "ok" {
                let error_msg = json_data.get("err-msg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("未知错误");
                return Err(MarketDataError::Parse {
                    exchange: self.exchange_id().to_string(),
                    details: format!("Huobi API错误: {}", error_msg),
                });
            }
        }
            
        let tick = json_data.get("tick")
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("响应中缺少tick字段 - 完整响应: {}", response_text),
            })?;
            
        let bids_raw = tick.get("bids")
            .and_then(|v| v.as_array())
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: "缺少bids字段".to_string(),
            })?;
            
        let asks_raw = tick.get("asks")
            .and_then(|v| v.as_array())
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: "缺少asks字段".to_string(),
            })?;
            
        let bids: Vec<OrderBookEntry> = bids_raw
            .iter()
            .filter_map(|level| {
                let price = level.get(0)?.as_f64()?;
                let qty = level.get(1)?.as_f64()?;
                Some(OrderBookEntry::new(price, qty))
            })
            .collect();
            
        let asks: Vec<OrderBookEntry> = asks_raw
            .iter()
            .filter_map(|level| {
                let price = level.get(0)?.as_f64()?;
                let qty = level.get(1)?.as_f64()?;
                Some(OrderBookEntry::new(price, qty))
            })
            .collect();
            
        Ok(MarketDataMessage::OrderBook(OrderBook {
            symbol: subscription.symbol.clone(),
            source: self.exchange_id().to_string(),
            timestamp: crate::high_precision_time::Nanos::now(),
            sequence_id: tick.get("version").and_then(|v| v.as_u64()),
            bids,
            asks,
            checksum: None,
        }))
    }
}
