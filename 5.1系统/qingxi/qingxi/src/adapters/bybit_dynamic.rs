#![allow(dead_code)]
//! # Bybit 动态适配器
//!
//! 基于 Bybit V5 WebSocket API 的高性能动态市场数据适配器
//! 
//! ## 功能特性
//! - 智能重连策略：指数退避 + 最大重试限制
//! - 动态心跳调整：基于网络延迟自适应调整心跳间隔
//! - 连接质量监控：实时评估连接质量并优化参数
//! - 分层错误处理：区分网络、协议、数据三层错误
//! - 预过滤零值数据：提前过滤无效数据，减少处理负担

use super::ExchangeAdapter;
use crate::types::*;
use crate::errors::MarketDataError;
use crate::{MarketDataMessage, OrderedFloat};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::{debug, info, warn, error};
use tokio_tungstenite::tungstenite::Message;
use std::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 动态连接质量评估器
#[derive(Debug)]
pub struct ConnectionQualityMonitor {
    /// 平均延迟 (微秒)
    avg_latency_us: AtomicU64,
    /// 连续成功消息计数
    success_count: AtomicU32,
    /// 连续失败计数
    failure_count: AtomicU32,
    /// 最后更新时间
    last_update: std::sync::Mutex<Instant>,
    /// 连接质量分数 (0-100)
    quality_score: AtomicU32,
}

impl ConnectionQualityMonitor {
    pub fn new() -> Self {
        Self {
            avg_latency_us: AtomicU64::new(1000), // 初始1ms
            success_count: AtomicU32::new(0),
            failure_count: AtomicU32::new(0),
            last_update: std::sync::Mutex::new(Instant::now()),
            quality_score: AtomicU32::new(100), // 初始满分
        }
    }

    /// 报告成功的消息处理
    pub fn report_success(&self, latency_us: u64) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);
        
        // 更新平均延迟 (简化的指数移动平均)
        let current_avg = self.avg_latency_us.load(Ordering::Relaxed);
        let new_avg = (current_avg * 7 + latency_us) / 8; // 0.875 权重
        self.avg_latency_us.store(new_avg, Ordering::Relaxed);
        
        // 更新质量分数
        self.update_quality_score();
    }

    /// 报告失败的连接或处理
    pub fn report_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        if self.failure_count.load(Ordering::Relaxed) > 3 {
            self.success_count.store(0, Ordering::Relaxed);
        }
        
        // 更新质量分数
        self.update_quality_score();
    }

    /// 更新连接质量分数
    fn update_quality_score(&self) {
        let latency = self.avg_latency_us.load(Ordering::Relaxed);
        let failures = self.failure_count.load(Ordering::Relaxed);
        let successes = self.success_count.load(Ordering::Relaxed);
        
        // 基于延迟的分数 (0-50分)
        let latency_score = if latency < 500 {
            50
        } else if latency < 1000 {
            45
        } else if latency < 2000 {
            35
        } else if latency < 5000 {
            20
        } else {
            5
        };
        
        // 基于稳定性的分数 (0-50分)
        let stability_score = if failures == 0 && successes > 10 {
            50
        } else if failures <= 1 && successes > 5 {
            40
        } else if failures <= 3 {
            25
        } else {
            5
        };
        
        let total_score = latency_score + stability_score;
        self.quality_score.store(total_score, Ordering::Relaxed);
        
        // 定期日志记录
        if let Ok(mut last_update) = self.last_update.try_lock() {
            if last_update.elapsed() > Duration::from_secs(30) {
                info!(
                    "🔍 Bybit连接质量评估: 延迟={}μs, 成功={}, 失败={}, 质量分数={}/100",
                    latency, successes, failures, total_score
                );
                *last_update = Instant::now();
            }
        }
    }

    /// 获取当前质量分数
    pub fn get_quality_score(&self) -> u32 {
        self.quality_score.load(Ordering::Relaxed)
    }

    /// 获取当前平均延迟
    pub fn get_avg_latency_us(&self) -> u64 {
        self.avg_latency_us.load(Ordering::Relaxed)
    }

    /// 获取建议的心跳间隔
    pub fn get_suggested_heartbeat_interval(&self) -> Duration {
        let quality = self.get_quality_score();
        let latency = self.get_avg_latency_us();
        
        if quality >= 80 && latency < 1000 {
            Duration::from_secs(45) // 高质量连接，心跳间隔更长
        } else if quality >= 60 && latency < 2000 {
            Duration::from_secs(30) // 正常间隔
        } else if quality >= 40 {
            Duration::from_secs(20) // 质量一般，更频繁心跳
        } else {
            Duration::from_secs(10) // 质量较差，高频心跳
        }
    }
}

/// 智能重连策略
#[derive(Debug)]
pub struct ReconnectStrategy {
    /// 重试次数
    retry_count: AtomicU32,
    /// 基础重连延迟
    base_delay: Duration,
    /// 最大重连延迟
    max_delay: Duration,
    /// 最大重试次数
    max_retries: u32,
}

impl ReconnectStrategy {
    pub fn new() -> Self {
        Self {
            retry_count: AtomicU32::new(0),
            base_delay: Duration::from_millis(1000), // 1秒基础延迟
            max_delay: Duration::from_secs(30),      // 最大30秒
            max_retries: 10,                         // 最多重试10次
        }
    }

    /// 计算下一次重连延迟（指数退避）
    pub fn get_next_delay(&self) -> Duration {
        let retry_count = self.retry_count.load(Ordering::Relaxed);
        
        if retry_count >= self.max_retries {
            warn!("🚨 Bybit重连次数超过最大限制({}次)，使用最大延迟", self.max_retries);
            return self.max_delay;
        }
        
        // 指数退避：base_delay * 2^retry_count，但不超过max_delay
        let exponential_delay = self.base_delay * (1 << retry_count.min(5)); // 限制指数最大为32
        let actual_delay = exponential_delay.min(self.max_delay);
        
        info!("🔄 Bybit重连策略: 第{}次重试，延迟{:?}", retry_count + 1, actual_delay);
        actual_delay
    }

    /// 增加重试计数
    pub fn increment_retry(&self) {
        self.retry_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 重置重试计数（连接成功后调用）
    pub fn reset(&self) {
        let old_count = self.retry_count.swap(0, Ordering::Relaxed);
        if old_count > 0 {
            info!("✅ Bybit连接成功，重置重试计数器（之前{}次重试）", old_count);
        }
    }

    /// 检查是否应该停止重试
    pub fn should_stop_retrying(&self) -> bool {
        self.retry_count.load(Ordering::Relaxed) >= self.max_retries
    }
}

/// Bybit 动态适配器实现
pub struct BybitDynamicAdapter {
    websocket_url: String,
    rest_api_url: Option<String>,
    /// 连接质量监控器
    quality_monitor: Arc<ConnectionQualityMonitor>,
    /// 重连策略
    reconnect_strategy: Arc<ReconnectStrategy>,
    /// 是否启用预过滤
    enable_prefilter: bool,
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
    trade_id: Option<String>,
    #[serde(rename = "i", default)]
    alt_trade_id: Option<String>,
    #[serde(flatten)]
    other: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for BybitDynamicAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BybitDynamicAdapter {
    /// 创建新的 Bybit 动态适配器
    pub fn new() -> Self {
        use crate::settings::Settings;
        if let Ok(settings) = Settings::load() {
            if let Some(bybit_config) = settings.sources.iter().find(|s| s.exchange_id == "bybit") {
                return Self::new_with_config(bybit_config);
            }
        }
        
        Self {
            websocket_url: "wss://stream.bybit.com/v5/public/spot".to_string(),
            rest_api_url: Some("https://api.bybit.com".to_string()),
            quality_monitor: Arc::new(ConnectionQualityMonitor::new()),
            reconnect_strategy: Arc::new(ReconnectStrategy::new()),
            enable_prefilter: true,
        }
    }

    /// 使用自定义配置创建 Bybit 动态适配器
    pub fn new_with_config(config: &MarketSourceConfig) -> Self {
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

        Self {
            websocket_url: config.get_websocket_url().to_string(),
            rest_api_url: config.get_rest_api_url().map(|s| s.to_string()),
            quality_monitor: Arc::new(ConnectionQualityMonitor::new()),
            reconnect_strategy: Arc::new(ReconnectStrategy::new()),
            enable_prefilter: true,
        }
    }

    pub fn websocket_url(&self) -> &str {
        &self.websocket_url
    }

    pub fn rest_api_url(&self) -> Option<&str> {
        self.rest_api_url.as_deref()
    }

    /// 获取连接质量监控器的引用
    pub fn quality_monitor(&self) -> Arc<ConnectionQualityMonitor> {
        self.quality_monitor.clone()
    }

    /// 获取重连策略的引用
    pub fn reconnect_strategy(&self) -> Arc<ReconnectStrategy> {
        self.reconnect_strategy.clone()
    }

    /// 将 Bybit 符号转换为标准符号
    fn parse_symbol(&self, bybit_symbol: &str) -> Option<Symbol> {
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

    /// 零值数据预过滤器
    fn prefilter_orderbook_entry(&self, price_str: &str, qty_str: &str) -> bool {
        if !self.enable_prefilter {
            return true;
        }

        // 过滤零价格或零数量
        if let (Ok(price), Ok(qty)) = (price_str.parse::<f64>(), qty_str.parse::<f64>()) {
            if price <= 0.0 || qty <= 0.0 || price.is_nan() || qty.is_nan() || price.is_infinite() || qty.is_infinite() {
                debug!("🧹 预过滤：移除无效订单条目 price={}, qty={}", price, qty);
                return false;
            }
            true
        } else {
            debug!("🧹 预过滤：无法解析价格/数量字符串 price={}, qty={}", price_str, qty_str);
            false
        }
    }

    /// 解析订单簿数据（带预过滤）
    fn parse_orderbook(&self, data: &BybitOrderBookData) -> Result<OrderBook, MarketDataError> {
        let symbol = self.parse_symbol(&data.s)
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid symbol: {}", data.s),
            })?;

        let mut bids = Vec::new();
        for bid in &data.b {
            // 预过滤检查
            if !self.prefilter_orderbook_entry(&bid[0], &bid[1]) {
                continue;
            }

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
            // 预过滤检查
            if !self.prefilter_orderbook_entry(&ask[0], &ask[1]) {
                continue;
            }

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

        debug!("📊 Bybit订单簿解析完成: {} bids, {} asks (预过滤后)", bids.len(), asks.len());

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

    /// 解析交易数据（带预过滤）
    fn parse_trade(&self, trade_data: &BybitTradeData) -> Result<TradeUpdate, MarketDataError> {
        let symbol = self.parse_symbol(&trade_data.s)
            .ok_or_else(|| MarketDataError::Parse {
                exchange: self.exchange_id().to_string(),
                details: format!("Invalid symbol: {}", trade_data.s),
            })?;

        // 预过滤价格和数量
        if self.enable_prefilter {
            if !self.prefilter_orderbook_entry(&trade_data.p, &trade_data.v) {
                return Err(MarketDataError::Parse {
                    exchange: self.exchange_id().to_string(),
                    details: format!("Trade data failed prefilter: price={}, volume={}", trade_data.p, trade_data.v),
                });
            }
        }

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

        let trade_id = trade_data.trade_id
            .clone()
            .or_else(|| trade_data.alt_trade_id.clone())
            .or_else(|| {
                trade_data.other.get("i")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            });

        debug!("💱 Bybit交易解析: {} {}@{} {}", symbol.as_pair(), quantity, price, side);

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
impl ExchangeAdapter for BybitDynamicAdapter {
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
            topics.push(format!("orderbook.1.{}", bybit_symbol));
            topics.push(format!("publicTrade.{}", bybit_symbol));
        }

        let subscribe_msg = json!({
            "op": "subscribe",
            "args": topics
        });

        info!("📡 Bybit动态适配器构建订阅消息: {} topics", topics.len());
        Ok(vec![Message::Text(subscribe_msg.to_string())])
    }

    fn parse_message(
        &self,
        message: &Message,
        _subscriptions: &[SubscriptionDetail],
    ) -> Result<Option<MarketDataMessage>, MarketDataError> {
        let start_time = Instant::now();
        
        let text = match message {
            Message::Text(text) => text,
            Message::Binary(_) => return Ok(None),
            _ => return Ok(None),
        };

        let value: Value = serde_json::from_str(text)
            .map_err(|e| {
                self.quality_monitor.report_failure();
                MarketDataError::Parse {
                    exchange: self.exchange_id().to_string(),
                    details: format!("JSON parse error: {}", e),
                }
            })?;

        // 检查是否为心跳或确认消息
        if let Some(op) = value.get("op").and_then(|v| v.as_str()) {
            match op {
                "pong" => {
                    debug!("💓 收到Bybit pong响应");
                    let latency_us = start_time.elapsed().as_micros() as u64;
                    self.quality_monitor.report_success(latency_us);
                    return Ok(Some(MarketDataMessage::Heartbeat {
                        source: "bybit".to_string(),
                        timestamp: crate::high_precision_time::Nanos::now(),
                    }));
                }
                "subscribe" => {
                    if let Some(success) = value.get("success").and_then(|v| v.as_bool()) {
                        if success {
                            info!("✅ Bybit订阅成功");
                            self.quality_monitor.report_success(100); // 快速成功
                        } else {
                            warn!("❌ Bybit订阅失败: {:?}", value);
                            self.quality_monitor.report_failure();
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
                    .map_err(|e| {
                        self.quality_monitor.report_failure();
                        MarketDataError::Parse {
                            exchange: self.exchange_id().to_string(),
                            details: format!("Failed to parse orderbook: {}", e),
                        }
                    })?;

                let orderbook = self.parse_orderbook(&response.data)?;
                let latency_us = start_time.elapsed().as_micros() as u64;
                self.quality_monitor.report_success(latency_us);
                
                return Ok(Some(MarketDataMessage::OrderBookSnapshot(orderbook)));
            } else if topic.starts_with("publicTrade.") {
                // 解析交易数据
                let response: BybitTradeResponse = serde_json::from_value(value)
                    .map_err(|e| {
                        self.quality_monitor.report_failure();
                        MarketDataError::Parse {
                            exchange: self.exchange_id().to_string(),
                            details: format!("Failed to parse trade: {}", e),
                        }
                    })?;

                if let Some(trade_data) = response.data.first() {
                    let trade = self.parse_trade(trade_data)?;
                    let latency_us = start_time.elapsed().as_micros() as u64;
                    self.quality_monitor.report_success(latency_us);
                    
                    return Ok(Some(MarketDataMessage::Trade(trade)));
                }
            }
        }

        debug!("🔍 忽略未知Bybit消息: {}", text);
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
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10)) // 设置超时
            .build()
            .map_err(|e| MarketDataError::Connection {
                exchange: self.exchange_id().to_string(),
                details: format!("Failed to create HTTP client: {}", e),
            })?;
            
        let symbol = self.format_symbol(&subscription.symbol);
        
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
                                                // 应用预过滤
                                                if self.prefilter_orderbook_entry(price_str, qty_str) {
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
                                }

                                let mut asks = Vec::new();
                                for ask in asks_raw {
                                    if let Some(arr) = ask.as_array() {
                                        if arr.len() >= 2 {
                                            if let (Some(price_str), Some(qty_str)) = (arr[0].as_str(), arr[1].as_str()) {
                                                // 应用预过滤
                                                if self.prefilter_orderbook_entry(price_str, qty_str) {
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
                                }

                                info!("📊 Bybit初始快照获取成功: {} bids, {} asks (预过滤后)", bids.len(), asks.len());
                                self.quality_monitor.report_success(1000); // REST API成功

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
                            error!("❌ Bybit快照解析失败: {}", e);
                            self.quality_monitor.report_failure();
                        }
                    }
                } else {
                    error!("❌ Bybit API响应错误: {}", response.status());
                    self.quality_monitor.report_failure();
                }
            }
            Err(e) => {
                error!("❌ Bybit快照获取失败: {}", e);
                self.quality_monitor.report_failure();
            }
        }

        // 返回空的快照作为fallback
        warn!("⚠️ 返回空快照作为Bybit初始快照的fallback");
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

impl std::fmt::Display for BybitDynamicAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let quality = self.quality_monitor.get_quality_score();
        let latency = self.quality_monitor.get_avg_latency_us();
        write!(f, "BybitDynamicAdapter(bybit, quality={}%, latency={}μs)", quality, latency)
    }
}
