#![allow(dead_code)]
//! # Exchange Adapters Module - 系统适配器权威接口
//!
//! 本模块定义了交易所适配器的权威接口规范。
//! 所有交易所适配器必须实现这里定义的 ExchangeAdapter trait。

use crate::{errors::MarketDataError, types::*, MarketDataMessage};
use async_trait::async_trait;
use tokio_tungstenite::tungstenite::Message;

pub mod binance;
pub mod huobi;
pub mod okx;
pub mod bybit;
pub mod gateio;

/// 权威交易所适配器接口 - 所有适配器必须实现此 trait
#[async_trait]
pub trait ExchangeAdapter: Send + Sync {
    /// 返回交易所唯一标识符
    fn exchange_id(&self) -> &str;

    /// 构建订阅消息 - 基于权威 SubscriptionDetail 类型
    fn build_subscription_messages(
        &self,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Vec<Message>, MarketDataError>;

    /// 解析来自交易所的消息 - 返回权威 MarketDataMessage 类型
    fn parse_message(
        &self,
        message: &Message,
        subscriptions: &[SubscriptionDetail],
    ) -> Result<Option<MarketDataMessage>, MarketDataError>;

    /// 获取心跳请求消息（可选）
    fn get_heartbeat_request(&self) -> Option<Message> {
        None
    }

    /// 检查消息是否为心跳
    fn is_heartbeat(&self, message: &Message) -> bool;

    /// 获取心跳响应消息（可选）
    fn get_heartbeat_response(&self, _message: &Message) -> Option<Message> {
        None
    }

    /// 获取初始快照数据 - 返回权威 MarketDataMessage 类型
    async fn get_initial_snapshot(
        &self,
        subscription: &SubscriptionDetail,
        rest_api_url: &str,
    ) -> Result<MarketDataMessage, MarketDataError>;

    /// 验证连接状态
    async fn validate_connection(&self) -> Result<bool, MarketDataError> {
        Ok(true) // 默认实现，具体适配器可重写
    }

    /// 获取支持的通道列表
    fn supported_channels(&self) -> Vec<&'static str> {
        vec!["orderbook", "trades"] // 默认支持的通道
    }
}

/// 权威适配器输出事件类型 - 基于第一阶段定义的 AdapterEvent
pub use crate::types::AdapterEvent;

/// 权威适配器错误类型 - 基于第一阶段定义的 MarketDataError
pub use crate::errors::MarketDataError as AdapterError;

/// 适配器工厂函数类型定义
pub type AdapterFactory = Box<dyn Fn() -> Box<dyn ExchangeAdapter> + Send + Sync>;

/// 适配器注册表 - 用于管理已注册的适配器
pub struct AdapterRegistry {
    adapters: std::collections::HashMap<String, AdapterFactory>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: std::collections::HashMap::new(),
        }
    }

    /// 注册适配器工厂
    pub fn register<F>(&mut self, exchange_id: &str, factory: F)
    where
        F: Fn() -> Box<dyn ExchangeAdapter> + Send + Sync + 'static,
    {
        self.adapters
            .insert(exchange_id.to_string(), Box::new(factory));
    }

    /// 创建适配器实例
    pub fn create_adapter(&self, exchange_id: &str) -> Option<Box<dyn ExchangeAdapter>> {
        self.adapters.get(exchange_id).map(|factory| factory())
    }

    /// 获取所有已注册的交易所ID
    pub fn registered_exchanges(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        let mut registry = Self::new();

        // 注册内置适配器（使用默认配置）
        registry.register("binance", || Box::new(binance::BinanceAdapter::new()));
        registry.register("okx", || Box::new(okx::OkxAdapter::new()));
        registry.register("huobi", || Box::new(huobi::HuobiAdapter::new()));
        registry.register("bybit", || Box::new(bybit::BybitAdapter::new()));
        registry.register("gateio", || Box::new(gateio::GateioAdapter::new()));

        registry
    }
}

/// 权威适配器类型重导出 - 保持向后兼容性
pub use self::ExchangeAdapter as MarketAdapter;
