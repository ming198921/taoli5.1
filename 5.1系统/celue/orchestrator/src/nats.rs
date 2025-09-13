use async_nats::{Client, Message, Subscriber};
use std::sync::Arc;
use parking_lot::RwLock;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;

pub struct NatsManager {
    client: Client,
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
}

impl NatsManager {
    pub async fn new(servers: Vec<String>) -> Result<Self> {
        let client = async_nats::connect(servers.join(",")).await?;
        Ok(Self {
            client,
            subscribers: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    pub async fn publish<T: Serialize>(&self, subject: &str, message: &T) -> Result<()> {
        let payload = serde_json::to_vec(message)?;
        self.client.publish(subject.to_string(), payload.into()).await?;
        Ok(())
    }
    
    pub async fn subscribe(&self, subject: &str) -> Result<Subscriber> {
        let subscriber = self.client.subscribe(subject.to_string()).await?;
        // 不需要clone，直接存储subscriber
        // self.subscribers.write().push(subscriber.clone());
        Ok(subscriber)
    }
    
    pub async fn request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self, 
        subject: &str, 
        message: &T,
        timeout: std::time::Duration
    ) -> Result<R> {
        let payload = serde_json::to_vec(message)?;
        let response = tokio::time::timeout(
            timeout,
            self.client.request(subject.to_string(), payload.into())
        ).await??;
        
        let result: R = serde_json::from_slice(&response.payload)?;
        Ok(result)
    }
    
    pub fn get_client(&self) -> &Client {
        &self.client
    }
    
    pub async fn close(self) -> Result<()> {
        // 关闭所有订阅
        let mut subscribers = self.subscribers.write();
        for mut sub in subscribers.drain(..) {
            sub.unsubscribe().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsMessage<T> {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub data: T,
}

impl<T> NatsMessage<T> {
    pub fn new(source: String, data: T) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            source,
            data,
        }
    }
}

pub struct NatsSubscriptionHandler {
    receiver: mpsc::UnboundedReceiver<Message>,
}

impl NatsSubscriptionHandler {
    pub fn new(receiver: mpsc::UnboundedReceiver<Message>) -> Self {
        Self { receiver }
    }
    
    pub async fn next(&mut self) -> Option<Message> {
        self.receiver.recv().await
    }
} 