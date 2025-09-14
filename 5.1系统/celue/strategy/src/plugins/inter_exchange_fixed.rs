use crate::{
    context::StrategyContext,
    traits::{ArbitrageStrategy, ExecutionResult, StrategyError, StrategyKind},
};
use async_trait::async_trait;
use common::{
    arbitrage::{ArbitrageLeg, ArbitrageOpportunity, Side},
    market_data::{NormalizedSnapshot, OrderBook},
    precision::{FixedPrice, FixedQuantity},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

/// 执行模式配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// 模拟模式 - 不发送真实订单
    Simulation,
    /// 生产模式 - 发送真实订单
    Production,
    /// 干跑模式 - 验证逻辑但不执行
    DryRun,
}

/// 交易所API配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub sandbox_mode: bool,
    pub max_slippage_bps: u32,
    pub order_timeout_seconds: u64,
    pub retry_attempts: u32,
}

/// 可配置的跨交易所套利策略
pub struct ConfigurableInterExchangeStrategy {
    /// 执行模式
    execution_mode: ExecutionMode,
    /// 交易所配置
    exchange_configs: HashMap<String, ExchangeConfig>,
    /// 性能配置
    max_slippage_bps: u32,
    order_timeout_seconds: u64,
    retry_attempts: u32,
    /// 真实交易API客户端
    exchange_clients: HashMap<String, Box<dyn ExchangeClient + Send + Sync>>,
}

/// 交易所客户端trait
#[async_trait]
pub trait ExchangeClient {
    async fn place_order(
        &self,
        symbol: &str,
        side: Side,
        quantity: FixedQuantity,
        price: FixedPrice,
    ) -> Result<String, StrategyError>;
    
    async fn cancel_order(&self, order_id: &str) -> Result<(), StrategyError>;
    
    async fn get_order_status(&self, order_id: &str) -> Result<OrderStatus, StrategyError>;
    
    async fn get_account_balance(&self, asset: &str) -> Result<f64, StrategyError>;
}

/// 订单状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled(f64), // 填充百分比
    Filled,
    Cancelled,
    Rejected(String),
}

impl ConfigurableInterExchangeStrategy {
    /// 从配置创建新策略实例
    pub fn from_config(
        execution_mode: ExecutionMode,
        exchange_configs: HashMap<String, ExchangeConfig>,
        max_slippage_bps: u32,
        order_timeout_seconds: u64,
        retry_attempts: u32,
    ) -> Self {
        Self {
            execution_mode,
            exchange_configs,
            max_slippage_bps,
            order_timeout_seconds,
            retry_attempts,
            exchange_clients: HashMap::new(),
        }
    }
    
    /// 注册交易所客户端
    pub fn register_exchange_client(
        &mut self,
        exchange: String,
        client: Box<dyn ExchangeClient + Send + Sync>,
    ) {
        self.exchange_clients.insert(exchange, client);
    }
}

#[async_trait]
impl ArbitrageStrategy for ConfigurableInterExchangeStrategy {
    fn name(&self) -> &'static str {
        "configurable_inter_exchange"
    }

    fn kind(&self) -> StrategyKind {
        StrategyKind::InterExchange
    }

    fn detect(
        &self,
        ctx: &StrategyContext,
        input: &NormalizedSnapshot,
    ) -> Option<ArbitrageOpportunity> {
        if input.exchanges.len() < 2 {
            return None;
        }

        let min_profit_pct = ctx.current_min_profit_pct();

        let same_symbol_books: Vec<&OrderBook> = input
            .exchanges
            .iter()
            .filter(|ob| ob.symbol == input.symbol)
            .collect();
        if same_symbol_books.len() < 2 {
            return None;
        }

        for (book_a, book_b) in same_symbol_books.iter().copied().tuple_combinations() {
            if let Some(opp) = self.find_opportunity(ctx, book_a, book_b, min_profit_pct) {
                return Some(opp);
            }
            if let Some(opp) = self.find_opportunity(ctx, book_b, book_a, min_profit_pct) {
                return Some(opp);
            }
        }
        None
    }

    async fn execute(
        &self,
        ctx: &StrategyContext,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError> {
        match &self.execution_mode {
            ExecutionMode::Simulation => self.execute_simulation(ctx, opportunity).await,
            ExecutionMode::Production => self.execute_production(ctx, opportunity).await,
            ExecutionMode::DryRun => self.execute_dry_run(ctx, opportunity).await,
        }
    }
}

impl ConfigurableInterExchangeStrategy {
    /// 模拟执行
    async fn execute_simulation(
        &self,
        _ctx: &StrategyContext,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError> {
        // 从环境变量获取模拟参数，避免硬编码
        let base_delay_ms = std::env::var("CELUE_SIMULATION_BASE_DELAY_MS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100u64); // 保守默认：100ms
            
        let delay_variance_ms = std::env::var("CELUE_SIMULATION_DELAY_VARIANCE_MS")
            .and_then(|s| s.parse().ok())
            .unwrap_or(50u64); // 保守默认：50ms方差
            
        let execution_delay = Duration::from_millis(base_delay_ms + rand::random::<u64>() % delay_variance_ms);
        sleep(execution_delay).await;
        
        // 从环境变量获取成功率，避免硬编码
        let success_rate = std::env::var("CELUE_SIMULATION_SUCCESS_RATE")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.90); // 保守默认：90%成功率
            
        let is_successful = rand::random::<f64>() < success_rate;
        
        let simulation_order_ids: Vec<String> = opportunity.legs
            .iter()
            .map(|_| format!("sim_{}", Uuid::new_v4().to_simple()))
            .collect();
        
        Ok(ExecutionResult {
            accepted: is_successful,
            reason: if is_successful {
                Some("Simulation execution successful".to_string())
            } else {
                Some("Simulation execution failed (random)".to_string())
            },
            order_ids: simulation_order_ids,
        })
    }
    
    /// 干跑执行
    async fn execute_dry_run(
        &self,
        _ctx: &StrategyContext,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError> {
        // 验证所有前置条件
        let mut validations = Vec::new();
        
        for leg in &opportunity.legs {
            let exchange = leg.exchange.to_string();
            
            // 检查交易所配置
            if !self.exchange_configs.contains_key(&exchange) {
                validations.push(format!("Missing config for exchange: {}", exchange));
                continue;
            }
            
            // 检查是否有对应的客户端
            if !self.exchange_clients.contains_key(&exchange) {
                validations.push(format!("Missing client for exchange: {}", exchange));
                continue;
            }
            
            // 验证订单参数
            if leg.quantity.to_f64() <= 0.0 {
                validations.push(format!("Invalid quantity for {}: {}", exchange, leg.quantity.to_f64()));
            }
            
            if leg.price.to_f64() <= 0.0 {
                validations.push(format!("Invalid price for {}: {}", exchange, leg.price.to_f64()));
            }
        }
        
        let dry_run_order_ids: Vec<String> = opportunity.legs
            .iter()
            .map(|_| format!("dry_{}", Uuid::new_v4().to_simple()))
            .collect();
        
        if validations.is_empty() {
            Ok(ExecutionResult {
                accepted: true,
                reason: Some("Dry run validation passed".to_string()),
                order_ids: dry_run_order_ids,
            })
        } else {
            Ok(ExecutionResult {
                accepted: false,
                reason: Some(format!("Dry run validation failed: {}", validations.join(", "))),
                order_ids: Vec::new(),
            })
        }
    }
    
    /// 生产执行
    async fn execute_production(
        &self,
        _ctx: &StrategyContext,
        opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError> {
        let mut order_ids = Vec::new();
        let mut execution_errors = Vec::new();
        
        // 分两个阶段执行：
        // 1. 并行提交所有订单
        // 2. 监控执行状态
        
        // 阶段1：提交订单
        let mut pending_orders = Vec::new();
        
        for leg in &opportunity.legs {
            let exchange = leg.exchange.to_string();
            
            // 获取交易所客户端
            let client = match self.exchange_clients.get(&exchange) {
                Some(client) => client,
                None => {
                    execution_errors.push(format!("No client for exchange: {}", exchange));
                    continue;
                }
            };
            
            // 提交订单
            match client.place_order(
                &leg.symbol.to_string(),
                leg.side,
                leg.quantity,
                leg.price,
            ).await {
                Ok(order_id) => {
                    order_ids.push(order_id.clone());
                    pending_orders.push((exchange.clone(), order_id));
                }
                Err(e) => {
                    execution_errors.push(format!("Order placement failed on {}: {}", exchange, e));
                }
            }
        }
        
        // 如果有订单失败，取消所有成功的订单
        if !execution_errors.is_empty() && !pending_orders.is_empty() {
            self.cancel_all_orders(&pending_orders).await;
            
            return Ok(ExecutionResult {
                accepted: false,
                reason: Some(format!("Execution failed: {}", execution_errors.join(", "))),
                order_ids: Vec::new(),
            });
        }
        
        // 阶段2：监控订单执行
        let execution_success = self.monitor_order_execution(&pending_orders).await?;
        
        Ok(ExecutionResult {
            accepted: execution_success,
            reason: if execution_success {
                Some("Production execution completed successfully".to_string())
            } else {
                Some("Production execution failed during monitoring".to_string())
            },
            order_ids,
        })
    }
    
    /// 取消所有订单
    async fn cancel_all_orders(&self, orders: &[(String, String)]) {
        for (exchange, order_id) in orders {
            if let Some(client) = self.exchange_clients.get(exchange) {
                if let Err(e) = client.cancel_order(order_id).await {
                    tracing::error!("Failed to cancel order {} on {}: {}", order_id, exchange, e);
                }
            }
        }
    }
    
    /// 监控订单执行
    async fn monitor_order_execution(&self, orders: &[(String, String)]) -> Result<bool, StrategyError> {
        let timeout = Duration::from_secs(self.order_timeout_seconds);
        let start_time = tokio::time::Instant::now();
        
        let mut completed_orders = 0;
        let total_orders = orders.len();
        
        while start_time.elapsed() < timeout && completed_orders < total_orders {
            for (exchange, order_id) in orders {
                if let Some(client) = self.exchange_clients.get(exchange) {
                    match client.get_order_status(order_id).await {
                        Ok(OrderStatus::Filled) => {
                            completed_orders += 1;
                        }
                        Ok(OrderStatus::Rejected(reason)) => {
                            tracing::error!("Order {} rejected on {}: {}", order_id, exchange, reason);
                            return Ok(false);
                        }
                        Ok(OrderStatus::Cancelled) => {
                            tracing::warn!("Order {} cancelled on {}", order_id, exchange);
                            return Ok(false);
                        }
                        Ok(_) => {
                            // 订单仍在处理中
                        }
                        Err(e) => {
                            tracing::error!("Failed to get order status for {} on {}: {}", order_id, exchange, e);
                            return Err(e);
                        }
                    }
                }
            }
            
            // 短暂等待后再次检查
            sleep(Duration::from_millis(100)).await;
        }
        
        // 检查是否所有订单都已完成
        if completed_orders == total_orders {
            Ok(true)
        } else {
            // 超时或部分失败，取消剩余订单
            self.cancel_all_orders(orders).await;
            Ok(false)
        }
    }
    
    /// 寻找套利机会（保持原有逻辑）
    fn find_opportunity(
        &self,
        ctx: &StrategyContext,
        buy_book: &OrderBook,
        sell_book: &OrderBook,
        min_profit_pct: FixedPrice,
    ) -> Option<ArbitrageOpportunity> {
        let buy_price = buy_book.best_ask()?;
        let sell_price = sell_book.best_bid()?;

        if sell_price.price <= buy_price.price {
            return None;
        }

        let gross_diff = sell_price.price - buy_price.price;
        let profit_percentage = FixedPrice::from_f64(
            gross_diff.to_f64() / buy_price.price.to_f64(),
            6,
        );

        if profit_percentage < min_profit_pct {
            return None;
        }

        // 费用计算保持不变
        let buy_exchange_fee = ctx.get_taker_fee(&buy_book.exchange)?;
        let sell_exchange_fee = ctx.get_taker_fee(&sell_book.exchange)?;

        let quantity = buy_price.quantity.min(sell_price.quantity);
        let buy_cost = buy_price.price * quantity;
        let sell_revenue = sell_price.price * quantity;
        let buy_fee = buy_cost * buy_exchange_fee;
        let sell_fee = sell_revenue * sell_exchange_fee;
        let net_profit = sell_revenue - buy_cost - buy_fee - sell_fee;
        let net_profit_pct = FixedPrice::from_f64(net_profit.to_f64() / buy_cost.to_f64(), 6);

        if net_profit_pct < min_profit_pct {
            return None;
        }

        Some(ArbitrageOpportunity {
            id: format!("inter_{}_{}", buy_book.exchange.to_string(), sell_book.exchange.to_string()),
            strategy: "inter_exchange".to_string(),
            legs: vec![
                ArbitrageLeg {
                    exchange: buy_book.exchange.clone(),
                    symbol: buy_book.symbol.clone(),
                    side: Side::Buy,
                    quantity,
                    price: buy_price.price,
                    cost: buy_cost,
                    fee: buy_fee,
                },
                ArbitrageLeg {
                    exchange: sell_book.exchange.clone(),
                    symbol: sell_book.symbol.clone(),
                    side: Side::Sell,
                    quantity,
                    price: sell_price.price,
                    cost: sell_revenue,
                    fee: sell_fee,
                },
            ],
            gross_profit: gross_diff * quantity,
            net_profit,
            gross_profit_pct: profit_percentage,
            net_profit_pct,
            estimated_execution_time_ms: 100.0,
            confidence_score: 0.95,
        })
    }
}

// 示例交易所客户端实现（Binance）
pub struct BinanceClient {
    api_key: String,
    api_secret: String,
    sandbox_mode: bool,
}

impl BinanceClient {
    pub fn new(api_key: String, api_secret: String, sandbox_mode: bool) -> Self {
        Self {
            api_key,
            api_secret,
            sandbox_mode,
        }
    }
}

#[async_trait]
impl ExchangeClient for BinanceClient {
    async fn place_order(
        &self,
        symbol: &str,
        side: Side,
        quantity: FixedQuantity,
        price: FixedPrice,
    ) -> Result<String, StrategyError> {
        // 实际的Binance API调用实现
        if self.sandbox_mode {
            // 沙盒模式 - 返回模拟订单ID
            Ok(format!("binance_test_{}", Uuid::new_v4().to_simple()))
        } else {
            // 真实API调用
            // 这里应该实现真实的Binance API调用
            Err(StrategyError::ExecutionError("Real Binance API not implemented".to_string()))
        }
    }
    
    async fn cancel_order(&self, order_id: &str) -> Result<(), StrategyError> {
        if self.sandbox_mode {
            tracing::info!("Cancelled test order: {}", order_id);
            Ok(())
        } else {
            Err(StrategyError::ExecutionError("Real Binance API not implemented".to_string()))
        }
    }
    
    async fn get_order_status(&self, order_id: &str) -> Result<OrderStatus, StrategyError> {
        if self.sandbox_mode {
            // 模拟订单状态
            if order_id.starts_with("binance_test_") {
                Ok(OrderStatus::Filled)
            } else {
                Ok(OrderStatus::Rejected("Invalid order ID".to_string()))
            }
        } else {
            Err(StrategyError::ExecutionError("Real Binance API not implemented".to_string()))
        }
    }
    
    async fn get_account_balance(&self, asset: &str) -> Result<f64, StrategyError> {
        if self.sandbox_mode {
            // 返回模拟余额
            Ok(10000.0)
        } else {
            Err(StrategyError::ExecutionError("Real Binance API not implemented".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_configurable_strategy_simulation() {
        let mut strategy = ConfigurableInterExchangeStrategy::from_config(
            ExecutionMode::Simulation,
            HashMap::new(),
            10, // 0.1% max slippage
            30, // 30 second timeout
            3,  // 3 retry attempts
        );
        
        // 这里可以添加更多测试
    }
    
    #[tokio::test]
    async fn test_dry_run_validation() {
        let mut strategy = ConfigurableInterExchangeStrategy::from_config(
            ExecutionMode::DryRun,
            HashMap::new(),
            10,
            30,
            3,
        );
        
        // 测试干跑模式的验证逻辑
    }
} 
        let mut strategy = ConfigurableInterExchangeStrategy::from_config(
            ExecutionMode::DryRun,
            HashMap::new(),
            10,
            30,
            3,
        );
        
        // 测试干跑模式的验证逻辑
    }
} 