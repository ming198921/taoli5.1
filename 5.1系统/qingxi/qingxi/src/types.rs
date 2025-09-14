#![allow(dead_code)]
//! # 核心类型定义 - 系统唯一权威来源
//!
//! 本模块定义了整个qingxi系统中使用的所有核心数据类型。
//! 这是系统类型系统的权威定义，所有其他模块必须导入并使用这些类型。

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 类型别名，用于更好的语义表达
pub type Price = OrderedFloat<f64>;
pub type Quantity = OrderedFloat<f64>;

/// 权威交易对定义
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
}

impl Symbol {
    pub fn new(base: &str, quote: &str) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    /// V3.0 零分配优化：空符号常量
    pub const EMPTY: Self = Self {
        base: String::new(),
        quote: String::new(),
    };

    pub fn as_pair(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }

    pub fn as_combined(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }

    pub fn from_pair(pair: &str) -> Option<Self> {
        if let Some(separator_pos) = pair.find('/') {
            let base = pair[..separator_pos].to_uppercase();
            let quote = pair[separator_pos + 1..].to_uppercase();
            Some(Symbol { base, quote })
        } else {
            None
        }
    }

    pub fn from_string(s: &str) -> Result<Self, String> {
        // 首先尝试解析 'BASE/QUOTE' 格式
        if let Some(separator_pos) = s.find('/') {
            let base = s[..separator_pos].trim().to_uppercase();
            let quote = s[separator_pos + 1..].trim().to_uppercase();
            if base.is_empty() || quote.is_empty() {
                return Err(format!("Invalid symbol format: base or quote part is empty in '{}'", s));
            }
            return Ok(Symbol { base, quote });
        }

        // 然后尝试解析 'BASE-QUOTE' 格式 (例如 OKX, Coinbase)
        if let Some(separator_pos) = s.find('-') {
            let base = s[..separator_pos].trim().to_uppercase();
            let quote = s[separator_pos + 1..].trim().to_uppercase();
            if base.is_empty() || quote.is_empty() {
                return Err(format!("Invalid symbol format: base or quote part is empty in '{}'", s));
            }
            return Ok(Symbol { base, quote });
        }
        
        // 最后尝试解析无分隔符的格式，但要更智能
        // 假设 USDT, USDC, BTC, ETH, BUSD 是常见的 quote 货币
        const COMMON_QUOTES: &[&str] = &["USDT", "USDC", "BTC", "ETH", "BUSD"];
        for quote in COMMON_QUOTES {
            if s.to_uppercase().ends_with(quote) && s.len() > quote.len() {
                let base = s[..s.len() - quote.len()].to_uppercase();
                return Ok(Symbol { base, quote: quote.to_string() });
            }
        }

        // 如果所有尝试都失败，则返回错误，不再猜测
        Err(format!("Could not determine a valid symbol pair from '{}'. Please use 'BASE/QUOTE' or 'BASE-QUOTE' format.", s))
    }

    pub fn to_string(&self) -> String {
        self.as_combined()
    }
}

/// 权威订单簿条目定义 - 结构体形式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct OrderBookEntry {
    pub price: Price,
    pub quantity: Quantity,
}

impl OrderBookEntry {
    pub fn new(price: f64, quantity: f64) -> Self {
        Self {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
        }
    }
    
    /// V3.0 零分配优化：空订单簿条目常量
    pub const EMPTY: Self = Self {
        price: OrderedFloat(0.0),
        quantity: OrderedFloat(0.0),
    };
}

/// 权威订单簿定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBook {
    pub symbol: Symbol,
    pub source: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub timestamp: crate::high_precision_time::Nanos,
    pub sequence_id: Option<u64>,
    pub checksum: Option<String>,
}

impl OrderBook {
    pub fn new(symbol: Symbol, source: String) -> Self {
        Self {
            symbol,
            source,
            bids: Vec::new(),
            asks: Vec::new(),
            timestamp: crate::high_precision_time::Nanos::now(),
            sequence_id: None,
            checksum: None,
        }
    }

    /// 获取最佳买价
    pub fn best_bid(&self) -> Option<&OrderBookEntry> {
        self.bids.first()
    }

    /// 获取最佳卖价
    pub fn best_ask(&self) -> Option<&OrderBookEntry> {
        self.asks.first()
    }
}

/// 权威交易方向定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 权威交易更新定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeUpdate {
    pub symbol: Symbol,
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub side: TradeSide,
    pub timestamp: crate::high_precision_time::Nanos,
    pub source: String,
    pub trade_id: Option<String>,
}

/// 权威市场数据快照定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketDataSnapshot {
    pub orderbook: Option<OrderBook>,
    pub trades: Vec<TradeUpdate>,
    pub timestamp: crate::high_precision_time::Nanos,
    pub source: String,
}

/// 权威订阅详情定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionDetail {
    pub symbol: Symbol,
    pub channel: String,
}

impl SubscriptionDetail {
    pub fn new(symbol: Symbol, channel: &str) -> Self {
        Self {
            symbol,
            channel: channel.to_string(),
        }
    }
}

/// 权威心跳配置定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeartbeatConfig {
    pub interval_sec: u64,
    pub message: Option<String>,
}

fn default_true() -> bool {
    true
}

/// 权威市场数据源配置定义 - 与 config.toml 完全匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSourceConfig {
    // --- 直接来自 config.toml 的字段 ---
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub exchange_id: String,
    pub adapter_type: String, // 匹配 config.toml 中的 adapter_type
    #[serde(rename = "ws_endpoint")]
    pub websocket_url: String, // 我们将 toml 的 ws_endpoint 映射为内部的 websocket_url
    #[serde(rename = "rest_url")]
    pub rest_api_url: Option<String>, // 我们将 toml 的 rest_url 映射为内部的 rest_api_url
    pub symbols: Vec<String>, // 注意：config.toml 中是字符串数组，不是 Symbol 数组
    pub channel: String, // 在 config.toml 中这是必需字段

    // --- 可以在 toml 中可选的字段 ---
    #[serde(default)]
    pub rate_limit: Option<u32>,
    #[serde(default)]
    pub connection_timeout_ms: Option<u32>,
    #[serde(default)]
    pub heartbeat_interval_ms: Option<u32>,
    #[serde(default)]
    pub reconnect_interval_sec: Option<u64>,
    #[serde(default)]
    pub max_reconnect_attempts: Option<u32>,

    // --- 为未来扩展保留 API 密钥字段 (保持默认) ---
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_secret: Option<String>,
    #[serde(default)]
    pub api_passphrase: Option<String>,
}

impl MarketSourceConfig {
    /// 获取WebSocket URL
    pub fn get_websocket_url(&self) -> &str {
        &self.websocket_url
    }

    /// 获取REST API URL
    pub fn get_rest_api_url(&self) -> Option<&String> {
        self.rest_api_url.as_ref()
    }

    /// 将字符串符号转换为 Symbol 结构体
    pub fn get_symbols(&self) -> Result<Vec<crate::types::Symbol>, String> {
        self.symbols.iter()
            .map(|s| crate::types::Symbol::from_string(s))
            .collect()
    }

    /// 获取兼容的 ws_endpoint（向后兼容）
    pub fn get_ws_endpoint(&self) -> &str {
        &self.websocket_url
    }

    /// 获取兼容的 rest_endpoint（向后兼容）
    pub fn get_rest_endpoint(&self) -> Option<&String> {
        self.rest_api_url.as_ref()
    }

    /// 获取 channel 作为 Option（向后兼容）
    pub fn get_channel(&self) -> Option<String> {
        Some(self.channel.clone())
    }

    /// 检查API密钥是否有效（非空且非占位符）
    pub fn has_valid_api_key(&self) -> bool {
        self.api_key.as_ref().map_or(false, |key| {
            !key.is_empty() && !key.starts_with("YOUR_") && !key.contains("_API_KEY")
        })
    }

    /// 检查API密钥密码是否有效（非空且非占位符）
    pub fn has_valid_api_secret(&self) -> bool {
        self.api_secret.as_ref().map_or(false, |secret| {
            !secret.is_empty() && !secret.starts_with("YOUR_") && !secret.contains("_SECRET")
        })
    }

    /// 检查OKX API密码短语是否有效（非空且非占位符）
    pub fn has_valid_api_passphrase(&self) -> bool {
        self.api_passphrase.as_ref().map_or(false, |passphrase| {
            !passphrase.is_empty() && !passphrase.starts_with("YOUR_") && !passphrase.contains("_PASSPHRASE")
        })
    }

    /// 检查是否具有完整的API凭据
    pub fn has_complete_api_credentials(&self) -> bool {
        match self.exchange_id.as_str() {
            "okx" => self.has_valid_api_key() && self.has_valid_api_secret() && self.has_valid_api_passphrase(),
            "binance" | "huobi" => self.has_valid_api_key() && self.has_valid_api_secret(),
            _ => true, // 对于未知交易所，假设不需要API凭据
        }
    }
}

/// 权威一致性阈值定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsistencyThresholds {
    pub price_diff_percentage: f64,
    pub timestamp_diff_ms: u64,
    pub sequence_gap_threshold: u64,
    pub spread_threshold_percentage: f64,
    pub critical_spread_threshold_percentage: f64,
    pub max_time_diff_ms: f64,
    pub volume_consistency_threshold: f64,
}

/// 权威异常类型定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalyType {
    PriceGap,
    TimestampGap,
    SequenceGap,
    AbnormalSpread,
    LowLiquidity,
    ConnectionLoss,
    DataCorruption,
    Other(String),
}

impl fmt::Display for AnomalyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnomalyType::PriceGap => write!(f, "PriceGap"),
            AnomalyType::TimestampGap => write!(f, "TimestampGap"),
            AnomalyType::SequenceGap => write!(f, "SequenceGap"),
            AnomalyType::AbnormalSpread => write!(f, "AbnormalSpread"),
            AnomalyType::LowLiquidity => write!(f, "LowLiquidity"),
            AnomalyType::ConnectionLoss => write!(f, "ConnectionLoss"),
            AnomalyType::DataCorruption => write!(f, "DataCorruption"),
            AnomalyType::Other(msg) => write!(f, "Other(\"{msg}\")"),
        }
    }
}

/// 权威异常严重程度定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnomalySeverity {
    Info,
    Warning,
    Critical,
    Fatal,
}

/// 权威异常检测结果定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnomalyDetectionResult {
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub details: String,
    pub description: String,
    pub symbol: Symbol,
    pub timestamp: crate::high_precision_time::Nanos,
    pub source: String,
    pub recovery_suggestion: Option<String>,
}

/// 权威适配器事件定义 - 统一的适配器输出接口
#[derive(Debug, Clone)]
pub enum AdapterEvent {
    MarketData(crate::orderbook::local_orderbook::MarketDataMessage),
    Error(String),
    Connected,
    Disconnected,
    Subscribed(SubscriptionDetail),
    Unsubscribed(SubscriptionDetail),
}

/// 权威订阅定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub symbol: Symbol,
    pub channel: String,
}

impl Subscription {
    pub fn new(symbol: Symbol, channel: &str) -> Self {
        Self {
            symbol,
            channel: channel.to_string(),
        }
    }
}
