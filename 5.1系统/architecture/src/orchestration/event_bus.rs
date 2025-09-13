//! 事件总线实现
//! 
//! 提供系统内部的事件发布和订阅机制

use crate::{
    config::ConfigCenter,
    errors::Result,
    types::{SystemEvent, EventType},
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use tracing::{debug, error};

/// 事件处理器特征
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: SystemEvent) -> Result<()>;
}

/// 事件总线
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<EventType, Vec<Arc<dyn EventHandler>>>>>,
    #[allow(dead_code)]
    config: Arc<ConfigCenter>,
}

impl EventBus {
    /// 创建新的事件总线
    pub async fn new(config: &Arc<ConfigCenter>) -> Result<Self> {
        Ok(Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
        })
    }
    
    /// 发布事件
    pub async fn publish(&self, event: SystemEvent) {
        debug!("发布事件: {:?}", event.event_type);
        
        let subscribers = self.subscribers.read().await;
        if let Some(handlers) = subscribers.get(&event.event_type) {
            for handler in handlers {
                let handler_clone = handler.clone();
                let event_clone = event.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = handler_clone.handle(event_clone).await {
                        error!("事件处理器执行失败: {}", e);
                    }
                });
            }
        }
    }
    
    /// 订阅事件
    pub async fn subscribe(&self, event_type: EventType, handler: Arc<dyn EventHandler>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.entry(event_type).or_insert_with(Vec::new).push(handler);
    }
    
    /// 取消订阅
    pub async fn unsubscribe(&self, event_type: EventType) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.remove(&event_type);
    }
    
    /// 获取订阅数量
    pub async fn get_subscriber_count(&self, event_type: EventType) -> usize {
        let subscribers = self.subscribers.read().await;
        subscribers.get(&event_type).map(|v| v.len()).unwrap_or(0)
    }
} 