//! 执行引擎滑点集成模块
//! 
//! 在订单执行前应用滑点预测和补偿策略

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, instrument, Span};
use ordered_float::OrderedFloat;

use crate::{Adapter, AdapterError, AdapterResult};
use common_types::{ArbitrageOpportunity, ExecutionResult};
use common::market_data::OrderBook;
use crate::execution::{OrderExecutor, ExecutionConfig};

/// 滑点感知的执行器
pub struct SlippageAwareExecutor {
    /// 基础执行器
    base_executor: Arc<dyn OrderExecutor>,
    /// 滑点管理器（通过HTTP客户端连接到qingxi服务）
    slippage_client: Arc<SlippageClient>,
    /// 执行配置
    config: SlippageExecutionConfig,
    /// 执行历史记录
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
}

/// 滑点执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlippageExecutionConfig {
    /// 是否启用滑点补偿
    pub enable_slippage_compensation: bool,
    /// 最大允许的预测延迟（毫秒）
    pub max_prediction_latency_ms: u64,
    /// 滑点预测超时时间
    pub prediction_timeout: Duration,
    /// 是否启用订单分割
    pub enable_order_splitting: bool,
    /// 最小启用滑点补偿的订单价值（USD）
    pub min_order_value_for_compensation: f64,
    /// 预测置信度阈值
    pub min_prediction_confidence: f64,
}

impl Default for SlippageExecutionConfig {
    fn default() -> Self {
        Self {
            enable_slippage_compensation: true,
            max_prediction_latency_ms: 100, // 100ms最大延迟
            prediction_timeout: Duration::from_millis(500),
            enable_order_splitting: true,
            min_order_value_for_compensation: 1000.0, // $1K以上启用
            min_prediction_confidence: 0.6, // 60%以上置信度
        }
    }
}

/// 执行记录
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionRecord {
    opportunity_id: String,
    exchange: String,
    symbol: String,
    predicted_slippage_bps: Option<u32>,
    actual_slippage_bps: Option<u32>,
    compensation_applied: bool,
    order_split: bool,
    execution_time: u64,
    success: bool,
    error_message: Option<String>,
}

/// 滑点服务客户端
pub struct SlippageClient {
    /// HTTP客户端
    client: reqwest::Client,
    /// 服务基础URL
    base_url: String,
    /// API认证密钥
    api_key: Option<String>,
}

impl SlippageClient {
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(1000))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url,
            api_key,
        }
    }
    
    /// 获取滑点预测
    async fn get_slippage_prediction(&self, request: SlippagePredictionRequest) -> Result<SlippagePredictionResponse> {
        let url = format!("{}/api/v1/slippage/predict", self.base_url);
        
        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req_builder.send().await
            .context("Failed to send slippage prediction request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Slippage prediction failed: {} - {}", status, text));
        }
        
        let prediction: SlippagePredictionResponse = response.json().await
            .context("Failed to parse slippage prediction response")?;
        
        Ok(prediction)
    }
    
    /// 获取补偿建议
    async fn get_compensation_suggestion(&self, request: CompensationRequest) -> Result<CompensationResponse> {
        let url = format!("{}/api/v1/slippage/compensate", self.base_url);
        
        let mut req_builder = self.client.post(&url).json(&request);
        
        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req_builder.send().await
            .context("Failed to send compensation request")?;
        
        let compensation: CompensationResponse = response.json().await
            .context("Failed to parse compensation response")?;
        
        Ok(compensation)
    }
    
    /// 记录实际滑点
    async fn record_actual_slippage(&self, record: SlippageRecord) -> Result<()> {
        let url = format!("{}/api/v1/slippage/record", self.base_url);
        
        let mut req_builder = self.client.post(&url).json(&record);
        
        if let Some(ref api_key) = self.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = req_builder.send().await
            .context("Failed to record actual slippage")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            warn!("Failed to record slippage: {} - {}", status, text);
        }
        
        Ok(())
    }
}

/// 滑点预测请求
#[derive(Debug, Serialize, Deserialize)]
struct SlippagePredictionRequest {
    exchange: String,
    symbol: String,
    trade_size: f64,
    order_book: OrderBookData,
}

/// 简化的订单簿数据
#[derive(Debug, Serialize, Deserialize)]
struct OrderBookData {
    bids: Vec<PriceLevel>,
    asks: Vec<PriceLevel>,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct PriceLevel {
    price: f64,
    quantity: f64,
}

/// 滑点预测响应
#[derive(Debug, Serialize, Deserialize)]
struct SlippagePredictionResponse {
    expected_slippage_bps: u32,
    confidence: f64,
    market_condition: String,
    recommended_max_size: f64,
    timestamp: u64,
    validity_seconds: u32,
}

/// 补偿请求
#[derive(Debug, Serialize, Deserialize)]
struct CompensationRequest {
    exchange: String,
    symbol: String,
    order_size: f64,
    market_price: f64,
    prediction: SlippagePredictionResponse,
}

/// 补偿响应
#[derive(Debug, Serialize, Deserialize)]
struct CompensationResponse {
    price_adjustment_bps: i32,
    order_splitting: Option<OrderSplittingData>,
    expected_improvement_bps: u32,
    confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OrderSplittingData {
    num_chunks: u32,
    interval_ms: u64,
    chunk_sizes: Vec<f64>,
}

/// 滑点记录
#[derive(Debug, Serialize, Deserialize)]
struct SlippageRecord {
    exchange: String,
    symbol: String,
    predicted_slippage_bps: u32,
    actual_slippage_bps: u32,
    trade_size: f64,
    market_condition: String,
}

impl SlippageAwareExecutor {
    pub fn new(
        base_executor: Arc<dyn OrderExecutor>,
        slippage_service_url: String,
        api_key: Option<String>,
        config: Option<SlippageExecutionConfig>,
    ) -> Self {
        Self {
            base_executor,
            slippage_client: Arc::new(SlippageClient::new(slippage_service_url, api_key)),
            config: config.unwrap_or_default(),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// 执行机会（带滑点补偿）
    #[instrument(skip(self, opportunity), fields(
        exchange = %opportunity.legs.first().map(|l| l.exchange.as_str()).unwrap_or("unknown"),
        symbol = %opportunity.legs.first().map(|l| l.symbol.as_str()).unwrap_or("unknown")
    ))]
    pub async fn execute_with_slippage_compensation(
        &self,
        opportunity: &ArbitrageOpportunity,
        order_book: &OrderBook,
    ) -> AdapterResult<ExecutionResult> {
        let start_time = Instant::now();
        let execution_span = Span::current();
        
        // 检查是否启用滑点补偿
        if !self.config.enable_slippage_compensation {
            debug!("Slippage compensation disabled, using base executor");
            return self.base_executor.execute_opportunity(opportunity).await;
        }
        
        // 计算订单价值
        let order_value = self.calculate_order_value(opportunity);
        if order_value < self.config.min_order_value_for_compensation {
            debug!(
                order_value = %order_value,
                min_threshold = %self.config.min_order_value_for_compensation,
                "Order value below compensation threshold, using base executor"
            );
            return self.base_executor.execute_opportunity(opportunity).await;
        }
        
        // 获取滑点预测
        let prediction = match self.get_slippage_prediction_with_timeout(opportunity, order_book).await {
            Ok(pred) => pred,
            Err(e) => {
                warn!(error = %e, "Failed to get slippage prediction, using base executor");
                return self.base_executor.execute_opportunity(opportunity).await;
            }
        };
        
        // 检查预测置信度
        if prediction.confidence < self.config.min_prediction_confidence {
            warn!(
                confidence = %prediction.confidence,
                min_confidence = %self.config.min_prediction_confidence,
                "Prediction confidence too low, using base executor"
            );
            return self.base_executor.execute_opportunity(opportunity).await;
        }
        
        // 获取补偿建议
        let compensation = match self.get_compensation_suggestion(opportunity, &prediction).await {
            Ok(comp) => comp,
            Err(e) => {
                warn!(error = %e, "Failed to get compensation suggestion, proceeding without compensation");
                return self.base_executor.execute_opportunity(opportunity).await;
            }
        };
        
        info!(
            predicted_slippage_bps = prediction.expected_slippage_bps,
            confidence = %prediction.confidence,
            price_adjustment_bps = compensation.price_adjustment_bps,
            has_order_splitting = compensation.order_splitting.is_some(),
            "Applying slippage compensation"
        );
        
        // 应用补偿策略执行订单
        let execution_result = if self.config.enable_order_splitting && compensation.order_splitting.is_some() {
            self.execute_with_order_splitting(opportunity, &compensation).await
        } else {
            self.execute_with_price_adjustment(opportunity, &compensation).await
        };
        
        // 记录执行历史
        let execution_record = ExecutionRecord {
            opportunity_id: opportunity.id.clone().unwrap_or_else(|| "unknown".to_string()),
            exchange: opportunity.legs.first().map(|l| l.exchange.as_str().to_string()).unwrap_or_default(),
            symbol: opportunity.legs.first().map(|l| l.symbol.as_str().to_string()).unwrap_or_default(),
            predicted_slippage_bps: Some(prediction.expected_slippage_bps),
            actual_slippage_bps: None, // 需要后续计算
            compensation_applied: true,
            order_split: compensation.order_splitting.is_some(),
            execution_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            success: execution_result.is_ok(),
            error_message: execution_result.as_ref().err().map(|e| e.to_string()),
        };
        
        {
            let mut history = self.execution_history.write().await;
            history.push(execution_record);
            
            // 保持历史记录在合理范围内
            if history.len() > 10000 {
                history.drain(0..1000);
            }
        }
        
        let execution_duration = start_time.elapsed();
        execution_span.record("execution_duration_ms", execution_duration.as_millis() as u64);
        
        debug!(
            execution_duration_ms = execution_duration.as_millis(),
            success = execution_result.is_ok(),
            "Slippage-aware execution completed"
        );
        
        execution_result
    }
    
    /// 带超时的滑点预测
    async fn get_slippage_prediction_with_timeout(
        &self,
        opportunity: &ArbitrageOpportunity,
        order_book: &OrderBook,
    ) -> Result<SlippagePredictionResponse> {
        let leg = opportunity.legs.first()
            .ok_or_else(|| anyhow::anyhow!("No legs in opportunity"))?;
        
        let request = SlippagePredictionRequest {
            exchange: leg.exchange.as_str().to_string(),
            symbol: leg.symbol.as_str().to_string(),
            trade_size: leg.quantity.into_inner(),
            order_book: OrderBookData {
                bids: order_book.bids.iter().take(10).map(|level| PriceLevel {
                    price: level.price.into_inner(),
                    quantity: level.quantity.into_inner(),
                }).collect(),
                asks: order_book.asks.iter().take(10).map(|level| PriceLevel {
                    price: level.price.into_inner(),
                    quantity: level.quantity.into_inner(),
                }).collect(),
                timestamp: order_book.timestamp,
            },
        };
        
        let prediction_future = self.slippage_client.get_slippage_prediction(request);
        let prediction = tokio::time::timeout(self.config.prediction_timeout, prediction_future).await
            .context("Slippage prediction timeout")?
            .context("Slippage prediction failed")?;
        
        Ok(prediction)
    }
    
    /// 获取补偿建议
    async fn get_compensation_suggestion(
        &self,
        opportunity: &ArbitrageOpportunity,
        prediction: &SlippagePredictionResponse,
    ) -> Result<CompensationResponse> {
        let leg = opportunity.legs.first()
            .ok_or_else(|| anyhow::anyhow!("No legs in opportunity"))?;
        
        let request = CompensationRequest {
            exchange: leg.exchange.as_str().to_string(),
            symbol: leg.symbol.as_str().to_string(),
            order_size: leg.quantity.into_inner(),
            market_price: leg.price.into_inner(),
            prediction: prediction.clone(),
        };
        
        self.slippage_client.get_compensation_suggestion(request).await
    }
    
    /// 使用价格调整执行
    async fn execute_with_price_adjustment(
        &self,
        opportunity: &ArbitrageOpportunity,
        compensation: &CompensationResponse,
    ) -> AdapterResult<ExecutionResult> {
        // 创建调整后的机会
        let mut adjusted_opportunity = opportunity.clone();
        
        // 应用价格调整
        for leg in &mut adjusted_opportunity.legs {
            let adjustment_ratio = compensation.price_adjustment_bps as f64 / 10000.0;
            match leg.side {
                common::arbitrage::Side::Buy => {
                    // 买单价格上调，增加成交概率
                    let adjusted_price = leg.price.into_inner() * (1.0 + adjustment_ratio);
                    leg.price = OrderedFloat(adjusted_price);
                },
                common::arbitrage::Side::Sell => {
                    // 卖单价格下调，增加成交概率
                    let adjusted_price = leg.price.into_inner() * (1.0 - adjustment_ratio);
                    leg.price = OrderedFloat(adjusted_price);
                },
            }
        }
        
        debug!(
            price_adjustment_bps = compensation.price_adjustment_bps,
            "Executing with price adjustment"
        );
        
        self.base_executor.execute_opportunity(&adjusted_opportunity).await
    }
    
    /// 使用订单分割执行
    async fn execute_with_order_splitting(
        &self,
        opportunity: &ArbitrageOpportunity,
        compensation: &CompensationResponse,
    ) -> AdapterResult<ExecutionResult> {
        let splitting_data = compensation.order_splitting.as_ref()
            .ok_or_else(|| AdapterError::InvalidInput("No order splitting data".to_string()))?;
        
        debug!(
            num_chunks = splitting_data.num_chunks,
            interval_ms = splitting_data.interval_ms,
            "Executing with order splitting"
        );
        
        let mut all_results = Vec::new();
        let mut total_executed_qty = OrderedFloat(0.0);
        
        for (i, &chunk_ratio) in splitting_data.chunk_sizes.iter().enumerate() {
            let mut chunk_opportunity = opportunity.clone();
            
            // 调整每个子订单的数量
            for leg in &mut chunk_opportunity.legs {
                let chunk_qty = leg.quantity.into_inner() * chunk_ratio;
                leg.quantity = OrderedFloat(chunk_qty);
                
                // 同时应用价格调整
                let adjustment_ratio = compensation.price_adjustment_bps as f64 / 10000.0;
                match leg.side {
                    common::arbitrage::Side::Buy => {
                        let adjusted_price = leg.price.into_inner() * (1.0 + adjustment_ratio);
                        leg.price = OrderedFloat(adjusted_price);
                    },
                    common::arbitrage::Side::Sell => {
                        let adjusted_price = leg.price.into_inner() * (1.0 - adjustment_ratio);
                        leg.price = OrderedFloat(adjusted_price);
                    },
                }
            }
            
            debug!(chunk_index = i, chunk_ratio = %chunk_ratio, "Executing order chunk");
            
            // 执行子订单
            let chunk_result = self.base_executor.execute_opportunity(&chunk_opportunity).await;
            
            match &chunk_result {
                Ok(result) => {
                    total_executed_qty += chunk_opportunity.legs.first().map(|l| l.quantity).unwrap_or(OrderedFloat(0.0));
                    all_results.push(result.clone());
                    
                    // 在子订单之间等待
                    if i < splitting_data.num_chunks as usize - 1 {
                        tokio::time::sleep(Duration::from_millis(splitting_data.interval_ms)).await;
                    }
                },
                Err(e) => {
                    error!(
                        chunk_index = i,
                        error = %e,
                        "Failed to execute order chunk"
                    );
                    // 继续执行剩余订单，而不是立即失败
                }
            }
        }
        
        // 合并所有执行结果
        if all_results.is_empty() {
            Err(AdapterError::ExecutionFailed("All order chunks failed".to_string()))
        } else {
            let combined_result = ExecutionResult {
                accepted: all_results.iter().all(|r| r.accepted),
                reason: Some(format!("Split order execution: {}/{} chunks succeeded", 
                                   all_results.len(), splitting_data.num_chunks)),
                order_ids: all_results.into_iter()
                    .flat_map(|r| r.order_ids)
                    .collect(),
            };
            
            info!(
                total_chunks = splitting_data.num_chunks,
                successful_chunks = combined_result.order_ids.len(),
                total_executed_qty = %total_executed_qty,
                "Order splitting execution completed"
            );
            
            Ok(combined_result)
        }
    }
    
    /// 计算订单价值
    fn calculate_order_value(&self, opportunity: &ArbitrageOpportunity) -> f64 {
        opportunity.legs.iter()
            .map(|leg| leg.price.into_inner() * leg.quantity.into_inner())
            .sum()
    }
    
    /// 记录实际滑点（在订单完成后调用）
    pub async fn record_actual_slippage(
        &self,
        opportunity_id: &str,
        actual_slippage_bps: u32,
    ) -> Result<()> {
        // 从历史记录中查找对应的执行记录
        let mut history = self.execution_history.write().await;
        if let Some(record) = history.iter_mut().find(|r| r.opportunity_id == opportunity_id) {
            record.actual_slippage_bps = Some(actual_slippage_bps);
            
            // 发送给滑点服务记录
            if let Some(predicted_slippage) = record.predicted_slippage_bps {
                let slippage_record = SlippageRecord {
                    exchange: record.exchange.clone(),
                    symbol: record.symbol.clone(),
                    predicted_slippage_bps: predicted_slippage,
                    actual_slippage_bps,
                    trade_size: 0.0, // 需要从机会中获取
                    market_condition: "unknown".to_string(), // 需要保存市场状况
                };
                
                if let Err(e) = self.slippage_client.record_actual_slippage(slippage_record).await {
                    warn!(error = %e, "Failed to record actual slippage to service");
                }
            }
        }
        
        Ok(())
    }
    
    /// 获取执行统计信息
    pub async fn get_execution_statistics(&self) -> ExecutionStatistics {
        let history = self.execution_history.read().await;
        
        let total_executions = history.len();
        let successful_executions = history.iter().filter(|r| r.success).count();
        let compensated_executions = history.iter().filter(|r| r.compensation_applied).count();
        let split_executions = history.iter().filter(|r| r.order_split).count();
        
        let prediction_accuracy = if history.iter().any(|r| r.predicted_slippage_bps.is_some() && r.actual_slippage_bps.is_some()) {
            let accurate_predictions = history.iter()
                .filter_map(|r| {
                    if let (Some(predicted), Some(actual)) = (r.predicted_slippage_bps, r.actual_slippage_bps) {
                        let error = (predicted as i32 - actual as i32).abs() as u32;
                        Some(error <= 10) // 10基点以内算准确
                    } else {
                        None
                    }
                })
                .filter(|&accurate| accurate)
                .count();
                
            let total_with_predictions = history.iter()
                .filter(|r| r.predicted_slippage_bps.is_some() && r.actual_slippage_bps.is_some())
                .count();
                
            if total_with_predictions > 0 {
                accurate_predictions as f64 / total_with_predictions as f64
            } else {
                0.0
            }
        } else {
            0.0
        };
        
        ExecutionStatistics {
            total_executions,
            successful_executions,
            success_rate: if total_executions > 0 { successful_executions as f64 / total_executions as f64 } else { 0.0 },
            compensated_executions,
            compensation_rate: if total_executions > 0 { compensated_executions as f64 / total_executions as f64 } else { 0.0 },
            split_executions,
            split_rate: if total_executions > 0 { split_executions as f64 / total_executions as f64 } else { 0.0 },
            prediction_accuracy,
        }
    }
}

/// 执行统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStatistics {
    pub total_executions: usize,
    pub successful_executions: usize,
    pub success_rate: f64,
    pub compensated_executions: usize,
    pub compensation_rate: f64,
    pub split_executions: usize,
    pub split_rate: f64,
    pub prediction_accuracy: f64,
}

#[async_trait]
impl OrderExecutor for SlippageAwareExecutor {
    async fn execute_opportunity(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<ExecutionResult> {
        // 如果没有订单簿信息，使用基础执行器
        // 在实际实现中，需要从市场数据服务获取订单簿
        warn!("No order book available, using base executor without slippage compensation");
        self.base_executor.execute_opportunity(opportunity).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    struct MockOrderExecutor {
        should_succeed: bool,
    }
    
    #[async_trait]
    impl OrderExecutor for MockOrderExecutor {
        async fn execute_opportunity(&self, _opportunity: &ArbitrageOpportunity) -> AdapterResult<ExecutionResult> {
            if self.should_succeed {
                Ok(ExecutionResult {
                    accepted: true,
                    reason: Some("Mock execution".to_string()),
                    order_ids: vec!["mock_order_1".to_string()],
                })
            } else {
                Err(AdapterError::ExecutionFailed("Mock failure".to_string()))
            }
        }
    }
    
    #[test]
    async fn test_slippage_aware_executor_creation() {
        let base_executor = Arc::new(MockOrderExecutor { should_succeed: true });
        let executor = SlippageAwareExecutor::new(
            base_executor,
            "http://localhost:8080".to_string(),
            None,
            None,
        );
        
        assert!(executor.config.enable_slippage_compensation);
    }
    
    #[test]
    async fn test_order_value_calculation() {
        let base_executor = Arc::new(MockOrderExecutor { should_succeed: true });
        let executor = SlippageAwareExecutor::new(
            base_executor,
            "http://localhost:8080".to_string(),
            None,
            None,
        );
        
        let opportunity = create_mock_opportunity();
        let value = executor.calculate_order_value(&opportunity);
        
        assert!(value > 0.0);
    }
    
    #[test]
    async fn test_execution_statistics() {
        let base_executor = Arc::new(MockOrderExecutor { should_succeed: true });
        let executor = SlippageAwareExecutor::new(
            base_executor,
            "http://localhost:8080".to_string(),
            None,
            None,
        );
        
        // 添加一些模拟执行记录
        {
            let mut history = executor.execution_history.write().await;
            history.push(ExecutionRecord {
                opportunity_id: "test_1".to_string(),
                exchange: "binance".to_string(),
                symbol: "BTCUSDT".to_string(),
                predicted_slippage_bps: Some(50),
                actual_slippage_bps: Some(45),
                compensation_applied: true,
                order_split: false,
                execution_time: 0,
                success: true,
                error_message: None,
            });
        }
        
        let stats = executor.get_execution_statistics().await;
        assert_eq!(stats.total_executions, 1);
        assert_eq!(stats.successful_executions, 1);
        assert_eq!(stats.compensated_executions, 1);
    }
    
    fn create_mock_opportunity() -> ArbitrageOpportunity {
        use common::arbitrage::{ArbitrageLeg, Side};
        use common::types::{Exchange, Symbol};
        
        ArbitrageOpportunity {
            id: Some("test_opportunity".to_string()),
            strategy: "inter_exchange".to_string(),
            legs: vec![
                ArbitrageLeg {
                    exchange: Exchange::new("binance".to_string()),
                    symbol: Symbol::new("BTCUSDT".to_string()),
                    side: Side::Buy,
                    price: OrderedFloat(50000.0),
                    quantity: OrderedFloat(0.1),
                    cost: OrderedFloat(5000.0),
                },
                ArbitrageLeg {
                    exchange: Exchange::new("okx".to_string()),
                    symbol: Symbol::new("BTCUSDT".to_string()),
                    side: Side::Sell,
                    price: OrderedFloat(50100.0),
                    quantity: OrderedFloat(0.1),
                    cost: OrderedFloat(5010.0),
                },
            ],
            net_profit: OrderedFloat(10.0),
            net_profit_pct: OrderedFloat(0.002),
            timestamp_ns: 0,
        }
    }
}