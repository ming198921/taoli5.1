#![allow(dead_code)]
//! # 事件总线系统
//!
//! 提供系统组件间的事件通信

use crate::events::SystemEvent;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info};

/// 事件总线，用于系统组件间的事件通信
pub struct EventBus {
    sender: broadcast::Sender<SystemEvent>,
    subscribers: Arc<RwLock<Vec<String>>>,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        
        Self {
            sender,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 发布事件
    pub async fn publish(&self, event: SystemEvent) {
        match self.sender.send(event.clone()) {
            Ok(subscriber_count) => {
                debug!("📢 事件已发布给 {} 个订阅者: {:?}", subscriber_count, event);
            }
            Err(_) => {
                debug!("📢 事件发布但无订阅者: {:?}", event);
            }
        }
    }
    
    /// 订阅事件
    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.sender.subscribe()
    }
    
    /// 注册订阅者（用于统计）
    pub async fn register_subscriber(&self, name: String) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(name.clone());
        info!("📋 新订阅者已注册: {}", name);
    }
    
    /// 获取订阅者数量
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}
