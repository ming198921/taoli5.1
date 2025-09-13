//! 跨服务追踪传播机制
//!
//! 实现生产级的分布式追踪传播，包括：
//! - HTTP头自动注入和提取
//! - NATS消息队列追踪传播
//! - 微服务链路完整追踪
//! - 上下文自动管理

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn, Span};
use uuid::Uuid;

/// 跨服务传播管理器
pub struct CrossServicePropagator {
    /// 配置
    config: PropagationConfig,
    /// 传播上下文存储
    contexts: Arc<RwLock<HashMap<String, PropagationContext>>>,
    /// HTTP头提取器
    http_extractor: HttpHeaderExtractor,
    /// NATS消息提取器
    nats_extractor: NatsMessageExtractor,
}

/// 传播配置
#[derive(Debug, Clone)]
pub struct PropagationConfig {
    /// 启用HTTP头传播
    pub enable_http_propagation: bool,
    /// 启用NATS消息传播
    pub enable_nats_propagation: bool,
    /// 自定义HTTP头名称
    pub trace_id_header: String,
    /// 父span ID头名称
    pub parent_span_header: String,
    /// 上下文过期时间（秒）
    pub context_ttl_seconds: u64,
    /// 最大传播深度
    pub max_propagation_depth: u32,
}

impl Default for PropagationConfig {
    fn default() -> Self {
        Self {
            enable_http_propagation: true,
            enable_nats_propagation: true,
            trace_id_header: "X-Trace-ID".to_string(),
            parent_span_header: "X-Parent-Span-ID".to_string(),
            context_ttl_seconds: 300, // 5分钟
            max_propagation_depth: 20,
        }
    }
}

/// 传播上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationContext {
    /// 追踪ID
    pub trace_id: String,
    /// 父span ID
    pub parent_span_id: Option<String>,
    /// 当前span ID
    pub current_span_id: String,
    /// 服务名称
    pub service_name: String,
    /// 操作名称
    pub operation_name: String,
    /// 传播深度
    pub depth: u32,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 自定义标签
    pub tags: HashMap<String, String>,
    /// 日志条目
    pub logs: Vec<LogEntry>,
}

/// 日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// HTTP头提取器
#[derive(Debug)]
pub struct HttpHeaderExtractor {
    trace_id_header: String,
    parent_span_header: String,
}

/// NATS消息提取器
#[derive(Debug)]
pub struct NatsMessageExtractor {
    trace_key: String,
    span_key: String,
}

/// 传播结果
#[derive(Debug)]
pub struct PropagationResult {
    pub success: bool,
    pub trace_id: Option<String>,
    pub parent_span_id: Option<String>,
    pub error_message: Option<String>,
}

impl CrossServicePropagator {
    /// 创建新的跨服务传播器
    pub fn new(config: PropagationConfig) -> Self {
        let http_extractor = HttpHeaderExtractor {
            trace_id_header: config.trace_id_header.clone(),
            parent_span_header: config.parent_span_header.clone(),
        };

        let nats_extractor = NatsMessageExtractor {
            trace_key: "trace_id".to_string(),
            span_key: "parent_span_id".to_string(),
        };

        Self {
            config,
            contexts: Arc::new(RwLock::new(HashMap::new())),
            http_extractor,
            nats_extractor,
        }
    }

    /// 注入追踪信息到HTTP头 - 优化版本
    #[instrument(skip(self))]
    pub async fn inject_http_headers(
        &self,
        headers: &mut HashMap<String, String>,
        context: &PropagationContext,
    ) -> Result<()> {
        if !self.config.enable_http_propagation {
            return Ok(());
        }

        // 高性能批量注入 - 预分配容量以减少内存重分配
        let initial_capacity = headers.len() + 10;
        if headers.capacity() < initial_capacity {
            headers.reserve(10);
        }

        // 核心追踪头 - 使用静态字符串减少分配
        headers.insert(self.config.trace_id_header.clone(), context.trace_id.clone());
        headers.insert(self.config.parent_span_header.clone(), context.current_span_id.clone());
        
        // OpenTelemetry兼容头 - 遵循行业标准
        headers.insert("traceparent".to_string(), 
            format!("00-{}-{}-01", 
                &context.trace_id[..32].chars().take(32).collect::<String>(), 
                &context.current_span_id[..16].chars().take(16).collect::<String>()
            )
        );
        
        // W3C Trace Context标准头
        let tracestate = format!("arb-sys={}", context.current_span_id);
        headers.insert("tracestate".to_string(), tracestate);

        // 服务元数据 - 批量插入
        let service_headers = [
            ("X-Service-Name", context.service_name.as_str()),
            ("X-Operation-Name", context.operation_name.as_str()),
            ("X-Trace-Depth", &context.depth.to_string()),
            ("X-Trace-Timestamp", &context.created_at.timestamp_millis().to_string()),
        ];
        
        for (key, value) in service_headers {
            headers.insert(key.to_string(), value.to_string());
        }

        // 性能优化：仅在有标签时才序列化
        if !context.tags.is_empty() {
            // 使用更快的压缩JSON序列化
            match simd_json::to_string(&context.tags) {
                Ok(tags_json) => {
                    // Base64编码以防止HTTP头字符问题
                    let encoded_tags = base64::encode(tags_json);
                    headers.insert("X-Trace-Tags".to_string(), encoded_tags);
                },
                Err(e) => {
                    warn!("Failed to serialize trace tags: {}", e);
                    // 降级到简单字符串格式
                    let simple_tags: Vec<String> = context.tags
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect();
                    headers.insert("X-Trace-Tags-Simple".to_string(), simple_tags.join(","));
                }
            }
        }

        // 添加采样决策头
        headers.insert("X-Trace-Sampled".to_string(), "1".to_string());
        
        // 添加系统标识
        headers.insert("X-Arbitrage-System".to_string(), "v5.1".to_string());

        debug!(
            trace_id = %context.trace_id,
            span_id = %context.current_span_id,
            service = %context.service_name,
            depth = context.depth,
            "Injected optimized trace context to HTTP headers"
        );

        Ok(())
    }

    /// 从HTTP头提取追踪信息 - 优化版本
    #[instrument(skip(self))]
    pub async fn extract_from_http_headers(
        &self,
        headers: &HashMap<String, String>,
        service_name: &str,
        operation_name: &str,
    ) -> Result<Option<PropagationContext>> {
        if !self.config.enable_http_propagation {
            return Ok(None);
        }

        // 优化：多种头格式支持，按优先级顺序提取
        let trace_id = self.extract_trace_id_optimized(headers)?;
        let parent_span_id = self.extract_parent_span_id_optimized(headers);

        if trace_id.is_none() {
            debug!("No trace ID found in any supported format, starting new trace");
            return Ok(None);
        }

        let trace_id = trace_id.unwrap();
        
        // 高效深度提取与验证
        let depth = self.extract_and_validate_depth(headers)?;
        if depth >= self.config.max_propagation_depth {
            warn!(
                trace_id = %trace_id,
                depth = depth,
                max_depth = self.config.max_propagation_depth,
                "Trace depth exceeds maximum, stopping propagation"
            );
            return Ok(None);
        }

        // 高性能span ID生成
        let current_span_id = self.generate_optimized_span_id();

        // 优化标签提取 - 支持多种编码格式
        let mut tags = HashMap::with_capacity(8);
        self.extract_tags_optimized(headers, &mut tags).await?;
        
        // 批量提取HTTP元数据
        self.extract_http_metadata(headers, &mut tags);

        let context = PropagationContext {
            trace_id: trace_id.clone(),
            parent_span_id,
            current_span_id: current_span_id.clone(),
            service_name: service_name.to_string(),
            operation_name: operation_name.to_string(),
            depth: depth + 1,
            created_at: Utc::now(),
            tags,
            logs: Vec::new(),
        };

        // 异步存储上下文，避免阻塞
        self.store_context_async(&current_span_id, &context).await?;

        debug!(
            trace_id = %trace_id,
            parent_span_id = ?parent_span_id,
            depth = context.depth,
            service = %service_name,
            "Extracted optimized trace context from HTTP headers"
        );

        Ok(Some(context))
    }
    
    /// 优化的trace ID提取 - 支持多种标准格式
    fn extract_trace_id_optimized(&self, headers: &HashMap<String, String>) -> Result<Option<String>> {
        // 按优先级顺序检查不同格式的trace ID
        let trace_id_sources = [
            // W3C Trace Context标准 - 最优先
            ("traceparent", |v: &str| {
                let parts: Vec<&str> = v.split('-').collect();
                if parts.len() >= 2 && parts[0] == "00" {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            }),
            // 自定义配置的头
            (self.config.trace_id_header.as_str(), |v: &str| Some(v.to_string())),
            // OpenTelemetry标准
            ("x-trace-id", |v: &str| Some(v.to_string())),
            // Jaeger格式
            ("uber-trace-id", |v: &str| {
                let parts: Vec<&str> = v.split(':').collect();
                if !parts.is_empty() {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            }),
            // B3格式
            ("x-b3-traceid", |v: &str| Some(v.to_string())),
        ];
        
        for (header_name, extractor) in trace_id_sources {
            if let Some(value) = headers.get(header_name) {
                if let Some(trace_id) = extractor(value) {
                    return Ok(Some(trace_id));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 优化的parent span ID提取
    fn extract_parent_span_id_optimized(&self, headers: &HashMap<String, String>) -> Option<String> {
        // 按优先级顺序检查不同格式的span ID
        let span_id_sources = [
            // W3C Trace Context
            ("traceparent", |v: &str| {
                let parts: Vec<&str> = v.split('-').collect();
                if parts.len() >= 3 {
                    Some(parts[2].to_string())
                } else {
                    None
                }
            }),
            // 自定义配置
            (self.config.parent_span_header.as_str(), |v: &str| Some(v.to_string())),
            // OpenTelemetry
            ("x-parent-span-id", |v: &str| Some(v.to_string())),
            // B3格式
            ("x-b3-spanid", |v: &str| Some(v.to_string())),
        ];
        
        for (header_name, extractor) in span_id_sources {
            if let Some(value) = headers.get(header_name) {
                if let Some(span_id) = extractor(value) {
                    return Some(span_id);
                }
            }
        }
        
        None
    }
    
    /// 高效深度提取与验证
    fn extract_and_validate_depth(&self, headers: &HashMap<String, String>) -> Result<u32> {
        let depth = headers
            .get("X-Trace-Depth")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        // 验证深度合理性
        if depth > 1000 {
            return Err(anyhow::anyhow!("Invalid trace depth: {}", depth));
        }
        
        Ok(depth)
    }
    
    /// 生成高性能span ID
    fn generate_optimized_span_id(&self) -> String {
        // 使用更高性能的ID生成策略
        format!("{:016x}", fastrand::u64(..))
    }
    
    /// 优化标签提取
    async fn extract_tags_optimized(&self, headers: &HashMap<String, String>, tags: &mut HashMap<String, String>) -> Result<()> {
        // 尝试Base64编码的标签
        if let Some(encoded_tags) = headers.get("X-Trace-Tags") {
            match base64::decode(encoded_tags) {
                Ok(decoded_bytes) => {
                    if let Ok(decoded_str) = String::from_utf8(decoded_bytes) {
                        if let Ok(parsed_tags) = simd_json::from_str::<HashMap<String, String>>(&decoded_str) {
                            tags.extend(parsed_tags);
                            return Ok(());
                        }
                    }
                },
                Err(_) => {
                    // 尝试直接JSON解析（兼容旧格式）
                    if let Ok(parsed_tags) = simd_json::from_str::<HashMap<String, String>>(encoded_tags) {
                        tags.extend(parsed_tags);
                        return Ok(());
                    }
                }
            }
        }
        
        // 尝试简单格式的标签
        if let Some(simple_tags) = headers.get("X-Trace-Tags-Simple") {
            for tag_pair in simple_tags.split(',') {
                if let Some((key, value)) = tag_pair.split_once('=') {
                    tags.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
        
        Ok(())
    }
    
    /// 批量提取HTTP元数据
    fn extract_http_metadata(&self, headers: &HashMap<String, String>, tags: &mut HashMap<String, String>) {
        // 预定义的重要HTTP头映射
        let important_headers = [
            ("user-agent", "http.user_agent"),
            ("x-real-ip", "http.remote_addr"),
            ("x-forwarded-for", "http.x_forwarded_for"),
            ("x-forwarded-proto", "http.scheme"),
            ("content-type", "http.content_type"),
            ("authorization", "http.has_auth"), // 不记录实际值，仅记录是否存在
        ];
        
        for (header_key, tag_key) in important_headers {
            if let Some(value) = headers.get(header_key) {
                let tag_value = if header_key == "authorization" {
                    "true".to_string()
                } else {
                    value.clone()
                };
                tags.insert(tag_key.to_string(), tag_value);
            }
        }
    }
    
    /// 异步存储上下文
    async fn store_context_async(&self, span_id: &str, context: &PropagationContext) -> Result<()> {
        // 使用tokio::spawn避免阻塞主线程
        let contexts_clone = Arc::clone(&self.contexts);
        let span_id_clone = span_id.to_string();
        let context_clone = context.clone();
        
        tokio::spawn(async move {
            let mut contexts = contexts_clone.write().await;
            contexts.insert(span_id_clone, context_clone);
        });
        
        Ok(())
    }

    /// 注入追踪信息到NATS消息
    #[instrument(skip(self))]
    pub async fn inject_nats_message(
        &self,
        message_headers: &mut HashMap<String, String>,
        context: &PropagationContext,
    ) -> Result<()> {
        if !self.config.enable_nats_propagation {
            return Ok(());
        }

        // 注入基础追踪信息
        message_headers.insert(self.nats_extractor.trace_key.clone(), context.trace_id.clone());
        message_headers.insert(
            self.nats_extractor.span_key.clone(),
            context.current_span_id.clone(),
        );

        // 注入扩展信息
        message_headers.insert("service".to_string(), context.service_name.clone());
        message_headers.insert("operation".to_string(), context.operation_name.clone());
        message_headers.insert("depth".to_string(), context.depth.to_string());
        message_headers.insert("timestamp".to_string(), context.created_at.timestamp().to_string());

        // 序列化上下文（用于完整传播）
        if let Ok(context_json) = serde_json::to_string(context) {
            message_headers.insert("trace_context".to_string(), context_json);
        }

        debug!(
            "Injected trace context to NATS message: trace_id={}, span_id={}",
            context.trace_id, context.current_span_id
        );

        Ok(())
    }

    /// 从NATS消息提取追踪信息
    #[instrument(skip(self))]
    pub async fn extract_from_nats_message(
        &self,
        message_headers: &HashMap<String, String>,
        service_name: &str,
        operation_name: &str,
    ) -> Result<Option<PropagationContext>> {
        if !self.config.enable_nats_propagation {
            return Ok(None);
        }

        // 尝试提取完整上下文
        if let Some(context_json) = message_headers.get("trace_context") {
            if let Ok(mut context) = serde_json::from_str::<PropagationContext>(context_json) {
                // 更新服务信息
                context.parent_span_id = Some(context.current_span_id.clone());
                context.current_span_id = Uuid::new_v4().to_string();
                context.service_name = service_name.to_string();
                context.operation_name = operation_name.to_string();
                context.depth += 1;

                // 检查深度限制
                if context.depth >= self.config.max_propagation_depth {
                    warn!("NATS trace depth {} exceeds maximum, stopping propagation", context.depth);
                    return Ok(None);
                }

                // 存储上下文
                {
                    let mut contexts = self.contexts.write().await;
                    contexts.insert(context.current_span_id.clone(), context.clone());
                }

                debug!(
                    "Extracted full trace context from NATS message: trace_id={}, depth={}",
                    context.trace_id, context.depth
                );

                return Ok(Some(context));
            }
        }

        // 回退到基础提取
        let trace_id = match message_headers.get(&self.nats_extractor.trace_key) {
            Some(id) => id.clone(),
            None => {
                debug!("No trace ID found in NATS message headers");
                return Ok(None);
            }
        };

        let parent_span_id = message_headers.get(&self.nats_extractor.span_key).cloned();
        let depth = message_headers
            .get("depth")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        if depth >= self.config.max_propagation_depth {
            return Ok(None);
        }

        let current_span_id = Uuid::new_v4().to_string();
        let context = PropagationContext {
            trace_id,
            parent_span_id,
            current_span_id: current_span_id.clone(),
            service_name: service_name.to_string(),
            operation_name: operation_name.to_string(),
            depth: depth + 1,
            created_at: Utc::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
        };

        // 存储上下文
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(current_span_id.clone(), context.clone());
        }

        Ok(Some(context))
    }

    /// 创建新的根追踪上下文
    pub async fn create_root_context(
        &self,
        service_name: &str,
        operation_name: &str,
    ) -> PropagationContext {
        let trace_id = Uuid::new_v4().to_string();
        let span_id = Uuid::new_v4().to_string();

        let context = PropagationContext {
            trace_id,
            parent_span_id: None,
            current_span_id: span_id.clone(),
            service_name: service_name.to_string(),
            operation_name: operation_name.to_string(),
            depth: 0,
            created_at: Utc::now(),
            tags: HashMap::new(),
            logs: Vec::new(),
        };

        // 存储上下文
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(span_id, context.clone());
        }

        debug!(
            "Created root trace context: trace_id={}, service={}, operation={}",
            context.trace_id, service_name, operation_name
        );

        context
    }

    /// 创建子span上下文
    pub async fn create_child_context(
        &self,
        parent_context: &PropagationContext,
        operation_name: &str,
    ) -> PropagationContext {
        let child_span_id = Uuid::new_v4().to_string();
        let mut child_context = parent_context.clone();

        child_context.parent_span_id = Some(parent_context.current_span_id.clone());
        child_context.current_span_id = child_span_id.clone();
        child_context.operation_name = operation_name.to_string();
        child_context.depth += 1;
        child_context.created_at = Utc::now();
        child_context.logs.clear(); // 子上下文从空日志开始

        // 存储上下文
        {
            let mut contexts = self.contexts.write().await;
            contexts.insert(child_span_id, child_context.clone());
        }

        debug!(
            "Created child trace context: trace_id={}, parent_span_id={}, child_span_id={}",
            child_context.trace_id,
            child_context.parent_span_id.as_ref().unwrap(),
            child_context.current_span_id
        );

        child_context
    }

    /// 添加日志到追踪上下文
    pub async fn add_log_to_context(
        &self,
        span_id: &str,
        level: &str,
        message: &str,
        fields: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.get_mut(span_id) {
            context.logs.push(LogEntry {
                timestamp: Utc::now(),
                level: level.to_string(),
                message: message.to_string(),
                fields,
            });

            // 限制日志条目数量
            if context.logs.len() > 100 {
                context.logs.drain(0..context.logs.len() - 100);
            }
        }
        Ok(())
    }

    /// 获取追踪上下文
    pub async fn get_context(&self, span_id: &str) -> Option<PropagationContext> {
        let contexts = self.contexts.read().await;
        contexts.get(span_id).cloned()
    }

    /// 清理过期的上下文
    #[instrument(skip(self))]
    pub async fn cleanup_expired_contexts(&self) -> usize {
        let now = Utc::now();
        let ttl = chrono::Duration::seconds(self.config.context_ttl_seconds as i64);
        
        let mut contexts = self.contexts.write().await;
        let initial_count = contexts.len();
        
        contexts.retain(|_, context| {
            now.signed_duration_since(context.created_at) < ttl
        });
        
        let cleaned_count = initial_count - contexts.len();
        
        if cleaned_count > 0 {
            debug!("Cleaned up {} expired trace contexts", cleaned_count);
        }
        
        cleaned_count
    }

    /// 获取所有活跃的追踪上下文
    pub async fn get_active_contexts(&self) -> HashMap<String, PropagationContext> {
        self.contexts.read().await.clone()
    }

    /// 启动清理任务
    pub async fn start_cleanup_task(&self) {
        let contexts = Arc::clone(&self.contexts);
        let ttl_seconds = self.config.context_ttl_seconds;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(ttl_seconds.max(60)) // 最小1分钟清理一次
            );
            
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let ttl = chrono::Duration::seconds(ttl_seconds as i64);
                
                let mut contexts_guard = contexts.write().await;
                let initial_count = contexts_guard.len();
                
                contexts_guard.retain(|_, context| {
                    now.signed_duration_since(context.created_at) < ttl
                });
                
                let cleaned_count = initial_count - contexts_guard.len();
                
                if cleaned_count > 0 {
                    debug!("Background cleanup removed {} expired trace contexts", cleaned_count);
                }
            }
        });
    }
}

impl HttpHeaderExtractor {
    /// 验证HTTP头名称的有效性
    pub fn validate_header_names(&self) -> Result<()> {
        fn is_valid_header_name(name: &str) -> bool {
            !name.is_empty() 
                && name.chars().all(|c| c.is_ascii() && (c.is_alphanumeric() || c == '-' || c == '_'))
                && !name.starts_with('-')
                && !name.ends_with('-')
        }

        if !is_valid_header_name(&self.trace_id_header) {
            return Err(anyhow::anyhow!("Invalid trace ID header name: {}", self.trace_id_header));
        }

        if !is_valid_header_name(&self.parent_span_header) {
            return Err(anyhow::anyhow!("Invalid parent span header name: {}", self.parent_span_header));
        }

        Ok(())
    }
}

impl NatsMessageExtractor {
    /// 验证NATS消息键的有效性
    pub fn validate_message_keys(&self) -> Result<()> {
        if self.trace_key.is_empty() || self.span_key.is_empty() {
            return Err(anyhow::anyhow!("NATS message keys cannot be empty"));
        }
        Ok(())
    }
}

/// 中间件宏：自动创建追踪上下文
#[macro_export]
macro_rules! with_trace_context {
    ($propagator:expr, $headers:expr, $service:expr, $operation:expr, $block:block) => {
        {
            let context = match $propagator.extract_from_http_headers($headers, $service, $operation).await? {
                Some(ctx) => ctx,
                None => $propagator.create_root_context($service, $operation).await,
            };
            
            let span = tracing::info_span!(
                $operation,
                trace_id = %context.trace_id,
                span_id = %context.current_span_id,
                parent_span_id = context.parent_span_id.as_deref().unwrap_or(""),
                service = %context.service_name,
                depth = context.depth
            );
            
            async move $block.instrument(span).await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_header_propagation() {
        let config = PropagationConfig::default();
        let propagator = CrossServicePropagator::new(config);

        // 创建根上下文
        let root_context = propagator.create_root_context("service-a", "handle_request").await;

        // 注入到HTTP头
        let mut headers = HashMap::new();
        propagator.inject_http_headers(&mut headers, &root_context).await.unwrap();

        // 验证头信息
        assert!(headers.contains_key("X-Trace-ID"));
        assert!(headers.contains_key("X-Parent-Span-ID"));
        assert_eq!(headers.get("X-Service-Name"), Some(&"service-a".to_string()));

        // 提取上下文
        let extracted = propagator.extract_from_http_headers(
            &headers, 
            "service-b", 
            "process_request"
        ).await.unwrap();

        assert!(extracted.is_some());
        let extracted_context = extracted.unwrap();
        assert_eq!(extracted_context.trace_id, root_context.trace_id);
        assert_eq!(extracted_context.parent_span_id, Some(root_context.current_span_id));
        assert_eq!(extracted_context.depth, 1);
    }

    #[tokio::test]
    async fn test_nats_message_propagation() {
        let config = PropagationConfig::default();
        let propagator = CrossServicePropagator::new(config);

        let root_context = propagator.create_root_context("publisher", "send_message").await;

        // 注入到NATS消息
        let mut message_headers = HashMap::new();
        propagator.inject_nats_message(&mut message_headers, &root_context).await.unwrap();

        // 提取上下文
        let extracted = propagator.extract_from_nats_message(
            &message_headers, 
            "subscriber", 
            "receive_message"
        ).await.unwrap();

        assert!(extracted.is_some());
        let extracted_context = extracted.unwrap();
        assert_eq!(extracted_context.trace_id, root_context.trace_id);
        assert_eq!(extracted_context.depth, 1);
    }

    #[tokio::test]
    async fn test_context_cleanup() {
        let mut config = PropagationConfig::default();
        config.context_ttl_seconds = 1; // 1秒过期

        let propagator = CrossServicePropagator::new(config);

        // 创建上下文
        let context = propagator.create_root_context("test-service", "test-operation").await;
        assert!(propagator.get_context(&context.current_span_id).await.is_some());

        // 等待过期
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 清理过期上下文
        let cleaned_count = propagator.cleanup_expired_contexts().await;
        assert_eq!(cleaned_count, 1);
        assert!(propagator.get_context(&context.current_span_id).await.is_none());
    }

    #[tokio::test]
    async fn test_depth_limit() {
        let mut config = PropagationConfig::default();
        config.max_propagation_depth = 2;

        let propagator = CrossServicePropagator::new(config);

        // 创建深度为2的上下文
        let mut headers = HashMap::new();
        headers.insert("X-Trace-ID".to_string(), "test-trace".to_string());
        headers.insert("X-Parent-Span-ID".to_string(), "parent-span".to_string());
        headers.insert("X-Trace-Depth".to_string(), "2".to_string());

        // 应该因为超过深度限制而返回None
        let result = propagator.extract_from_http_headers(
            &headers, 
            "deep-service", 
            "deep-operation"
        ).await.unwrap();

        assert!(result.is_none());
    }
}