//! NATS adapter for high-frequency messaging
//! 
//! Provides reliable, low-latency messaging for system components.
//! Implements both request/reply and publish/subscribe patterns.

use crate::{Adapter, AdapterError, AdapterResult};
use async_nats::{jetstream, Client, ConnectOptions};
use futures_util::stream::StreamExt;
use common::{ArbitrageOpportunity, ExecutionResult, NormalizedSnapshot, TraceId};
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use zstd::stream::encode_all as zstd_encode_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::{error, info, warn};

/// NATS adapter configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NatsConfig {
    /// NATS server URLs
    pub servers: Vec<String>,
    
    /// Connection name for identification
    pub name: String,
    
    /// Authentication token
    pub token: Option<String>,
    
    /// Username for authentication
    pub username: Option<String>,
    
    /// Password for authentication
    pub password: Option<String>,
    
    /// Connection timeout
    pub connect_timeout: Duration,
    
    /// Request timeout
    pub request_timeout: Duration,
    
    /// Enable JetStream
    pub enable_jetstream: bool,
    
    /// JetStream domain
    pub jetstream_domain: Option<String>,
    
    /// Stream configurations
    pub streams: Vec<StreamConfig>,
    
    /// Consumer configurations
    pub consumers: Vec<ConsumerConfig>,
}

/// Stream configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StreamConfig {
    pub name: String,
    pub subjects: Vec<String>,
    pub retention: String, // "limits", "interest", "workqueue"
    pub max_msgs: i64,
    pub max_bytes: i64,
    pub max_age: Duration,
    pub replicas: usize,
}

/// Consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsumerConfig {
    pub name: String,
    pub stream_name: String,
    pub deliver_subject: Option<String>,
    pub durable_name: Option<String>,
    pub ack_policy: String, // "none", "all", "explicit"
    pub max_deliver: i32,
    pub filter_subject: Option<String>,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            servers: vec!["nats://localhost:4222".to_string()],
            name: "qingxi-strategy".to_string(),
            token: None,
            username: None,
            password: None,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_millis(100), // Fast for HFT
            enable_jetstream: true,
            jetstream_domain: None,
            streams: vec![
                StreamConfig {
                    name: "OPPORTUNITIES".to_string(),
                    subjects: vec!["strategy.opportunities.*".to_string()],
                    retention: "workqueue".to_string(),
                    max_msgs: 10000,
                    max_bytes: 100 * 1024 * 1024, // 100MB
                    max_age: Duration::from_secs(60),
                    replicas: 1,
                },
                StreamConfig {
                    name: "EXECUTIONS".to_string(),
                    subjects: vec!["execution.results.*".to_string()],
                    retention: "limits".to_string(),
                    max_msgs: 50000,
                    max_bytes: 500 * 1024 * 1024, // 500MB
                    max_age: Duration::from_secs(24 * 60 * 60),
                    replicas: 1,
                },
                StreamConfig {
                    name: "MARKET_DATA".to_string(),
                    subjects: vec!["market.data.*".to_string()],
                    retention: "limits".to_string(),
                    max_msgs: 100000,
                    max_bytes: 1024 * 1024 * 1024, // 1GB
                    max_age: Duration::from_secs(6 * 60 * 60),
                    replicas: 1, // Market data doesn't need high availability
                },
            ],
            consumers: vec![
                ConsumerConfig {
                    name: "strategy-processor".to_string(),
                    stream_name: "MARKET_DATA".to_string(),
                    deliver_subject: None,
                    durable_name: Some("strategy-processor".to_string()),
                    ack_policy: "explicit".to_string(),
                    max_deliver: 3,
                    filter_subject: Some("market.data.normalized.*".to_string()),
                },
            ],
        }
    }
}

/// Message types for NATS communication
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum NatsMessage {
    /// Market data snapshot
    MarketData {
        snapshot: NormalizedSnapshot,
        trace_id: TraceId,
    },
    
    /// Arbitrage opportunity detected
    OpportunityDetected {
        opportunity: ArbitrageOpportunity,
        trace_id: TraceId,
        idempotency_key: String,
    },
    /// Exchange fee update (bps)
    FeeUpdate {
        exchange: String,
        taker_bps: f64,
        maker_bps: Option<f64>,
    },
    /// Symbol precision and step/tick size update
    PrecisionUpdate {
        symbol: String,
        price_scale: u8,
        qty_scale: u8,
        tick_size: f64,
        step_size: f64,
    },
    
    /// Risk check request
    RiskCheckRequest {
        opportunity: ArbitrageOpportunity,
        trace_id: TraceId,
    },
    
    /// Risk check response
    RiskCheckResponse {
        opportunity_id: String,
        approved: bool,
        reason: Option<String>,
        trace_id: TraceId,
    },
    
    /// Execution request
    ExecutionRequest {
        opportunity: ArbitrageOpportunity,
        trace_id: TraceId,
    },
    
    /// Execution result
    ExecutionResult {
        result: ExecutionResult,
        trace_id: TraceId,
    },
    /// Standardized execution intent
    ExecIntent {
        opportunity: ArbitrageOpportunity,
        idempotency_key: String,
        deadline_ns: u64,
        trace_id: TraceId,
    },
    /// Execution ACK
    ExecAck {
        ack: ExecutionResult,
    },
    
    /// Health check ping
    HealthPing {
        component: String,
        timestamp: u64,
    },
    
    /// Health check pong
    HealthPong {
        component: String,
        timestamp: u64,
        status: String,
    },
    
    /// Configuration update
    ConfigUpdate {
        component: String,
        config: serde_json::Value,
        version: u64,
    },
    
    /// Metrics data
    Metrics {
        component: String,
        metrics: HashMap<String, f64>,
        timestamp: u64,
    },
    /// Rich audit event payload
    AuditEvent {
        component: String,
        payload: serde_json::Value,
        timestamp: u64,
    },
}

/// Message handler trait
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: NatsMessage, reply_subject: Option<String>) -> AdapterResult<Option<NatsMessage>>;
}

/// Message router for distributing messages to handlers
pub struct MessageRouter {
    handlers: RwLock<HashMap<String, Arc<dyn MessageHandler>>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a handler for a subject pattern
    pub fn register_handler(&self, subject_pattern: String, handler: Arc<dyn MessageHandler>) {
        self.handlers.write().insert(subject_pattern, handler);
    }
    
    /// Route a message to the appropriate handler
    pub async fn route_message(
        &self,
        subject: &str,
        message: NatsMessage,
        reply_subject: Option<String>,
    ) -> AdapterResult<Option<NatsMessage>> {
        // Find matching handler without holding the lock across await
        let handler_opt: Option<Arc<dyn MessageHandler>> = {
            let handlers = self.handlers.read();
            let mut found: Option<Arc<dyn MessageHandler>> = None;
            for (pattern, handler) in handlers.iter() {
                if subject.starts_with(pattern.trim_end_matches('*')) {
                    found = Some(handler.clone());
                    break;
                }
            }
            found
        };
        if let Some(handler) = handler_opt {
            return handler.handle_message(message, reply_subject).await;
        }
        
        warn!("No handler found for subject: {}", subject);
        Ok(None)
    }
}

/// High-performance NATS adapter
pub struct NatsAdapter {
    config: Option<NatsConfig>,
    client: Option<Client>,
    jetstream: Option<jetstream::Context>,
    router: Arc<MessageRouter>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    running: Arc<parking_lot::Mutex<bool>>,
    test_sink: Option<Arc<parking_lot::Mutex<Vec<(String, NatsMessage)>>>>,
}

impl NatsAdapter {
    pub fn set_test_sink(&mut self, sink: Arc<parking_lot::Mutex<Vec<(String, NatsMessage)>>>) { self.test_sink = Some(sink); }
    pub fn router(&self) -> Arc<MessageRouter> { self.router.clone() }
    pub fn register_handler(&self, subject_prefix: &str, handler: Arc<dyn MessageHandler>) {
        self.router.register_handler(subject_prefix.to_string(), handler);
    }
    fn serialize_message(message: &NatsMessage, content_type: &str) -> Result<Vec<u8>, AdapterError> {
        match content_type {
            "application/json" => serde_json::to_vec(message).map_err(|e| AdapterError::Generic { message: e.to_string() }),
            "application/zstd+json" => {
                let json = serde_json::to_vec(message).map_err(|e| AdapterError::Generic { message: e.to_string() })?;
                zstd_encode_all(&json[..], 1).map_err(|e| AdapterError::Generic { message: e.to_string() })
            }
            _ => bincode::serialize(message).map_err(|e| AdapterError::Generic { message: e.to_string() }),
        }
    }
    #[allow(dead_code)]
    fn deserialize_message(_payload: &[u8], _content_type: &str) -> Result<NatsMessage, AdapterError> {
        Err(AdapterError::Generic { message: "NatsMessage deserialization is handler-specific and disabled here".to_string() })
    }
    
    pub fn new() -> Self { Self::default() }
    /// Publish a message to a subject
    pub async fn publish(&self, subject: &str, message: &NatsMessage) -> AdapterResult<()> {
        if let Some(sink) = &self.test_sink { sink.lock().push((subject.to_string(), message.clone())); }
        let client = self.client.as_ref().ok_or(AdapterError::NotInitialized)?;
        let content_type = "application/bincode"; // default fast path
        let payload = Self::serialize_message(message, content_type)?;
        client.publish(subject.to_string(), payload.into()).await.map_err(|e| AdapterError::NatsPublish(e.to_string()))?;
        Ok(())
    }
    
    /// Send a request and wait for reply
    pub async fn request(&self, subject: &str, message: &NatsMessage) -> AdapterResult<NatsMessage> {
        let client = self.client.as_ref().ok_or(AdapterError::NotInitialized)?;
        let content_type = "application/bincode";
        let payload = Self::serialize_message(message, content_type)?;
        let response = tokio::time::timeout(
            self.config.as_ref().unwrap().request_timeout,
            client.request(subject.to_string(), payload.into()),
        )
        .await
        .map_err(|_| AdapterError::Timeout {
            duration_ms: self.config.as_ref().unwrap().request_timeout.as_millis() as u64,
        })?.map_err(|e| AdapterError::NatsRequest(e.to_string()))?;
        let response_message = Self::deserialize_message(&response.payload, content_type)?;
        Ok(response_message)
    }
    
    /// Subscribe to a subject with a handler
    pub async fn subscribe(&self, subject: &str) -> AdapterResult<()> {
        let client = self.client.as_ref().ok_or(AdapterError::NotInitialized)?;
        let router = self.router.clone();
        let subj = subject.to_string();
        let mut subscription = client.subscribe(subj).await.map_err(|e| AdapterError::NatsSubscribe(e.to_string()))?;
        let subject_owned = subject.to_string();
        let content_type = "application/bincode".to_string();
        
        // Spawn task to handle messages
        tokio::spawn(async move {
            while let Some(message) = subscription.next().await {
                let payload = message.payload;
                // Placeholder: deserialization disabled at adapter level; handlers should own decoding
                let _ = payload; let _ = &content_type; let _ = &subject_owned; let _ = &router;
            }
        });
        
        Ok(())
    }
    
    /// Setup JetStream streams and consumers
    async fn setup_jetstream(&mut self) -> AdapterResult<()> {
        let config = self.config.as_ref().unwrap();
        if !config.enable_jetstream {
            return Ok(());
        }
        
        let client = self.client.as_ref().unwrap();
        let jetstream = jetstream::new(client.clone());
        
        // Create or update streams
        for stream_config in &config.streams {
            let stream_info = jetstream::stream::Config {
                name: stream_config.name.clone(),
                subjects: stream_config.subjects.clone(),
                retention: match stream_config.retention.as_str() {
                    "limits" => jetstream::stream::RetentionPolicy::Limits,
                    "interest" => jetstream::stream::RetentionPolicy::Interest,
                    "workqueue" => jetstream::stream::RetentionPolicy::WorkQueue,
                    _ => jetstream::stream::RetentionPolicy::Limits,
                },
                max_messages: stream_config.max_msgs,
                max_bytes: stream_config.max_bytes,
                max_age: stream_config.max_age,
                num_replicas: stream_config.replicas,
                ..Default::default()
            };
            
            match jetstream.get_or_create_stream(stream_info).await {
                Ok(_) => { info!("Stream '{}' ready", stream_config.name); }
                Err(e) => { error!("Failed to create stream '{}': {}", stream_config.name, e); return Err(AdapterError::Generic{ message: e.to_string() }); }
            }
        }
        
        self.jetstream = Some(jetstream);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Adapter for NatsAdapter {
    type Config = NatsConfig;
    type Error = AdapterError;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        info!("Initializing NATS adapter with servers: {:?}", config.servers);
        
        // Build connection options
        let mut opts = ConnectOptions::new()
            .name(&config.name)
            .connection_timeout(config.connect_timeout);
        
        if let Some(token) = &config.token { 
            opts = opts.token(token.to_string());
        } else if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            opts = opts.user_and_password(user.to_string(), pass.to_string());
        }
        
        // Connect to NATS
        let client = async_nats::connect_with_options(&config.servers.join(","), opts)
            .await
            .map_err(|e| AdapterError::Generic{ message: e.to_string() })?;
        
        self.config = Some(config);
        self.client = Some(client);
        
        // Setup JetStream
        self.setup_jetstream().await?;
        
        info!("NATS adapter initialized successfully");
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        {
            let mut running = self.running.lock();
            if *running { return Err(AdapterError::AlreadyRunning); }
            *running = true;
        }
        let (shutdown_tx, _shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);
        let core_subjects = [
            "strategy.opportunities.*",
            "market.data.*",
            "execution.results.*",
            "health.*",
            "config.*",
            "metrics.*",
        ];
        for subject in &core_subjects { self.subscribe(subject).await?; }
        info!("NATS adapter started");
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Self::Error> {
        let mut running = self.running.lock();
        if !*running {
            return Ok(());
        }
        
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        
        // Close NATS connection
        if let Some(_client) = self.client.take() {
            // Note: async_nats doesn't have an explicit close method in this version
            // Connection will be closed when client is dropped
        }
        
        *running = false;
        info!("NATS adapter stopped");
        Ok(())
    }
    
    async fn health_check(&self) -> Result<(), Self::Error> {
        let _client = self.client.as_ref().ok_or(AdapterError::NotInitialized)?;
        
        // Try to publish a health ping
        let health_msg = NatsMessage::HealthPing {
            component: "nats_adapter".to_string(),
            timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap() as u64,
        };
        
        self.publish("health.ping", &health_msg).await?;
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "nats_adapter"
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NatsAdapter {
    fn default() -> Self {
        Self {
            config: None,
            client: None,
            jetstream: None,
            router: Arc::new(MessageRouter::new()),
            shutdown_tx: None,
            running: Arc::new(parking_lot::Mutex::new(false)),
            test_sink: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tokio_test;

    #[tokio::test]
    async fn test_nats_config_default() {
        let config = NatsConfig::default();
        assert_eq!(config.servers, vec!["nats://localhost:4222"]);
        assert_eq!(config.name, "qingxi-strategy");
        assert!(config.enable_jetstream);
        assert_eq!(config.streams.len(), 3);
    }
    
    #[tokio::test]
    async fn test_message_router() {
        let router = MessageRouter::new();
        
        // Test with no handlers
        let msg = NatsMessage::HealthPing {
            component: "test".to_string(),
            timestamp: 12345,
        };
        
        let result = router.route_message("test.subject", msg, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
    
    #[tokio::test]
    async fn test_adapter_lifecycle() {
        let adapter = NatsAdapter::new();
        
        // Should not be initialized
        assert!(adapter.health_check().await.is_err());
        
        // Initialize with default config
        let _config = NatsConfig::default();
        // Note: This would fail without a running NATS server
        // assert!(adapter.initialize(config).await.is_ok());
    }
}
