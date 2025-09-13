use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use parking_lot::Mutex;
use std::future::Future;
use std::pin::Pin;

/// 重新设计的MessageProcessor trait - 支持动态分发
pub trait MessageProcessor: Send + Sync {
    type Input: Send + Sync + Clone;
    type Output: Send + Sync;
    
    /// 使用Box<dyn Future>来支持动态分发
    fn process(&self, input: Self::Input) -> Pin<Box<dyn Future<Output = Result<Self::Output>> + Send + '_>>;
    fn name(&self) -> &str;
}

/// 类型擦除的处理器包装器
pub struct DynMessageProcessor<T, U> {
    inner: Arc<dyn MessageProcessor<Input = T, Output = U>>,
}

impl<T, U> DynMessageProcessor<T, U> 
where 
    T: Send + Sync + Clone + 'static,
    U: Send + Sync + 'static,
{
    pub fn new<P>(processor: P) -> Self 
    where 
        P: MessageProcessor<Input = T, Output = U> + 'static,
    {
        Self {
            inner: Arc::new(processor),
        }
    }

    pub async fn process(&self, input: T) -> Result<U> {
        self.inner.process(input).await
    }

    pub fn name(&self) -> &str {
        self.inner.name()
    }
}

pub struct ProcessorPipeline<T, U> {
    processors: Vec<DynMessageProcessor<T, U>>,
    metrics: Arc<Mutex<ProcessorMetrics>>,
}

impl<T: Send + Sync + Clone + 'static, U: Send + Sync + 'static> ProcessorPipeline<T, U> {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            metrics: Arc::new(Mutex::new(ProcessorMetrics::new())),
        }
    }
    
    pub fn add_processor<P>(&mut self, processor: P) 
    where 
        P: MessageProcessor<Input = T, Output = U> + 'static,
    {
        self.processors.push(DynMessageProcessor::new(processor));
    }
    
    pub async fn process(&self, input: T) -> Result<Vec<U>> {
        let mut results = Vec::new();
        
        for processor in &self.processors {
            let start = std::time::Instant::now();
            match processor.process(input.clone()).await {
                Ok(output) => {
                    let elapsed = start.elapsed();
                    self.metrics.lock().record_success(processor.name(), elapsed);
                    results.push(output);
                }
                Err(e) => {
                    self.metrics.lock().record_failure(processor.name());
                    tracing::error!("Processor {} failed: {}", processor.name(), e);
                }
            }
        }
        
        Ok(results)
    }
    
    pub fn get_metrics(&self) -> ProcessorMetrics {
        self.metrics.lock().clone()
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorMetrics {
    success_count: HashMap<String, u64>,
    failure_count: HashMap<String, u64>,
    avg_latency_ms: HashMap<String, f64>,
    total_processed: u64,
}

impl ProcessorMetrics {
    fn new() -> Self {
        Self {
            success_count: HashMap::new(),
            failure_count: HashMap::new(),
            avg_latency_ms: HashMap::new(),
            total_processed: 0,
        }
    }
    
    fn record_success(&mut self, processor_name: &str, latency: std::time::Duration) {
        let count = self.success_count.entry(processor_name.to_string()).or_insert(0);
        *count += 1;
        
        let avg = self.avg_latency_ms.entry(processor_name.to_string()).or_insert(0.0);
        let new_latency = latency.as_secs_f64() * 1000.0;
        *avg = (*avg * (*count - 1) as f64 + new_latency) / *count as f64;
        
        self.total_processed += 1;
    }
    
    fn record_failure(&mut self, processor_name: &str) {
        *self.failure_count.entry(processor_name.to_string()).or_insert(0) += 1;
        self.total_processed += 1;
    }

    pub fn get_success_count(&self, processor_name: &str) -> u64 {
        self.success_count.get(processor_name).copied().unwrap_or(0)
    }

    pub fn get_failure_count(&self, processor_name: &str) -> u64 {
        self.failure_count.get(processor_name).copied().unwrap_or(0)
    }

    pub fn get_avg_latency_ms(&self, processor_name: &str) -> f64 {
        self.avg_latency_ms.get(processor_name).copied().unwrap_or(0.0)
    }

    pub fn get_total_processed(&self) -> u64 {
        self.total_processed
    }
}

// 具体的处理器实现
pub struct MarketDataProcessor {
    name: String,
}

impl MarketDataProcessor {
    pub fn new() -> Self {
        Self {
            name: "MarketDataProcessor".to_string(),
        }
    }
}

impl MessageProcessor for MarketDataProcessor {
    type Input = RawMarketData;
    type Output = ProcessedMarketData;
    
    fn process(&self, input: Self::Input) -> Pin<Box<dyn Future<Output = Result<Self::Output>> + Send + '_>> {
        Box::pin(async move {
            // 处理市场数据 - 使用统一的MarketData结构
            Ok(ProcessedMarketData {
                market_data: input, // RawMarketData现在就是MarketData类型
                processed_at: chrono::Utc::now(),
            })
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct ArbitrageDetectionProcessor {
    name: String,
    min_profit_threshold: f64,
}

impl ArbitrageDetectionProcessor {
    pub fn new(min_profit_threshold: f64) -> Self {
        Self {
            name: "ArbitrageDetectionProcessor".to_string(),
            min_profit_threshold,
        }
    }
}

impl MessageProcessor for ArbitrageDetectionProcessor {
    type Input = Vec<ProcessedMarketData>;
    type Output = ArbitrageOpportunity;
    
    fn process(&self, input: Self::Input) -> Pin<Box<dyn Future<Output = Result<Self::Output>> + Send + '_>> {
        let min_profit_threshold = self.min_profit_threshold;
        Box::pin(async move {
            // 简单的套利检测逻辑
            if input.len() < 2 {
                return Err(anyhow::anyhow!("Need at least 2 markets for arbitrage"));
            }
            
            let mut best_buy = &input[0];
            let mut best_sell = &input[0];
            
            for market in &input {
                if market.market_data.ask_price < best_buy.market_data.ask_price {
                    best_buy = market;
                }
                if market.market_data.bid_price > best_sell.market_data.bid_price {
                    best_sell = market;
                }
            }
            
            let profit = best_sell.market_data.bid_price - best_buy.market_data.ask_price;
            let profit_pct = (profit / best_buy.market_data.ask_price) * 100.0;
            
            // 使用统一的ArbitrageOpportunity::new方法
            let volume = best_buy.market_data.bid_volume.min(best_sell.market_data.ask_volume);
            let mut opportunity = ArbitrageOpportunity::new(
                uuid::Uuid::new_v4().to_string(),
                best_buy.market_data.symbol.clone(),
                best_buy.market_data.exchange.clone(),
                best_sell.market_data.exchange.clone(),
                best_buy.market_data.ask_price,
                best_sell.market_data.bid_price,
                volume,
                150_000, // 150秒TTL
            );
            
            // 添加额外的元数据
            opportunity.metadata.insert("detection_threshold".to_string(), serde_json::json!(min_profit_threshold));
            opportunity.metadata.insert("is_viable".to_string(), serde_json::json!(profit_pct >= min_profit_threshold));
            
            Ok(opportunity)
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

// 使用统一的MarketData定义替代重复的数据结构
pub use common_types::{MarketData as RawMarketData};

// ProcessedMarketData 保留为专门的处理结果结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMarketData {
    // 基础数据使用统一的MarketData
    pub market_data: common_types::MarketData,
    // 处理相关的特殊字段
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

// 使用统一的ArbitrageOpportunity定义
pub use common_types::ArbitrageOpportunity;

// 批处理器
pub struct BatchProcessor<T> {
    batch_size: usize,
    timeout: std::time::Duration,
    processor: Arc<dyn Fn(Vec<T>) -> Result<()> + Send + Sync>,
    buffer: Arc<RwLock<Vec<T>>>,
}

impl<T: Send + Sync + 'static> BatchProcessor<T> {
    pub fn new(
        batch_size: usize,
        timeout: std::time::Duration,
        processor: Arc<dyn Fn(Vec<T>) -> Result<()> + Send + Sync>,
    ) -> Self {
        Self {
            batch_size,
            timeout,
            processor,
            buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn add(&self, item: T) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        buffer.push(item);
        
        if buffer.len() >= self.batch_size {
            let batch: Vec<T> = buffer.drain(..).collect();
            drop(buffer);
            (self.processor)(batch)?;
        }
        
        Ok(())
    }
    
    pub async fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        if !buffer.is_empty() {
            let batch: Vec<T> = buffer.drain(..).collect();
            drop(buffer);
            (self.processor)(batch)?;
        }
        Ok(())
    }
    
    pub async fn start_auto_flush(self: Arc<Self>) {
        let timeout = self.timeout;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(timeout).await;
                if let Err(e) = self.flush().await {
                    tracing::error!("Batch processor flush error: {}", e);
                }
            }
        });
    }
} 