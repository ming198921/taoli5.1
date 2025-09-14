use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use parking_lot::Mutex;

#[async_trait]
pub trait MessageProcessor: Send + Sync {
    type Input: Send + Sync;
    type Output: Send + Sync;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output>;
    fn name(&self) -> &str;
}

pub struct ProcessorPipeline<T, U> {
    processors: Vec<Arc<dyn MessageProcessor<Input = T, Output = U>>>,
    metrics: Arc<Mutex<ProcessorMetrics>>,
}

impl<T: Send + Sync + Clone, U: Send + Sync> ProcessorPipeline<T, U> {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            metrics: Arc::new(Mutex::new(ProcessorMetrics::new())),
        }
    }
    
    pub fn add_processor(&mut self, processor: Arc<dyn MessageProcessor<Input = T, Output = U>>) {
        self.processors.push(processor);
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

#[async_trait]
impl MessageProcessor for MarketDataProcessor {
    type Input = RawMarketData;
    type Output = ProcessedMarketData;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        // 处理市场数据
        Ok(ProcessedMarketData {
            symbol: input.symbol,
            exchange: input.exchange,
            timestamp: input.timestamp,
            bid_price: input.bids.first().map(|b| b.0).unwrap_or(0.0),
            ask_price: input.asks.first().map(|a| a.0).unwrap_or(0.0),
            volume: input.volume,
            processed_at: chrono::Utc::now(),
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

#[async_trait]
impl MessageProcessor for ArbitrageDetectionProcessor {
    type Input = Vec<ProcessedMarketData>;
    type Output = ArbitrageOpportunity;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        // 简单的套利检测逻辑
        if input.len() < 2 {
            return Err(anyhow::anyhow!("Need at least 2 markets for arbitrage"));
        }
        
        let mut best_buy = &input[0];
        let mut best_sell = &input[0];
        
        for market in &input {
            if market.ask_price < best_buy.ask_price {
                best_buy = market;
            }
            if market.bid_price > best_sell.bid_price {
                best_sell = market;
            }
        }
        
        let profit = best_sell.bid_price - best_buy.ask_price;
        let profit_pct = (profit / best_buy.ask_price) * 100.0;
        
        Ok(ArbitrageOpportunity {
            id: uuid::Uuid::new_v4().to_string(),
            buy_exchange: best_buy.exchange.clone(),
            sell_exchange: best_sell.exchange.clone(),
            symbol: best_buy.symbol.clone(),
            buy_price: best_buy.ask_price,
            sell_price: best_sell.bid_price,
            profit,
            profit_pct,
            volume: best_buy.volume.min(best_sell.volume),
            timestamp: chrono::Utc::now(),
            is_viable: profit_pct >= self.min_profit_threshold,
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

// 数据结构定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMarketData {
    pub symbol: String,
    pub exchange: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub bids: Vec<(f64, f64)>, // (price, quantity)
    pub asks: Vec<(f64, f64)>,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedMarketData {
    pub symbol: String,
    pub exchange: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub bid_price: f64,
    pub ask_price: f64,
    pub volume: f64,
    pub processed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: String,
    pub buy_exchange: String,
    pub sell_exchange: String,
    pub symbol: String,
    pub buy_price: f64,
    pub sell_price: f64,
    pub profit: f64,
    pub profit_pct: f64,
    pub volume: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub is_viable: bool,
}

// 批处理器
pub struct BatchProcessor<T: Send + Sync> {
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