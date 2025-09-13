//! 市场模拟器模块

use crate::config::{MarketSimulationConfig, VolatilityModel};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{debug, info};

/// 市场条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketCondition {
    Bull,      // 牛市
    Bear,      // 熊市
    Sideways,  // 震荡
    Volatile,  // 高波动
}

/// 价格模拟数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceSimulation {
    pub symbol: String,
    pub initial_price: f64,
    pub target_price: Option<f64>,
    pub volatility: f64,
    pub drift: f64,
    pub jump_intensity: f64,
}

/// 市场数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataPoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
    pub bid: f64,
    pub ask: f64,
    pub volatility: f64,
}

/// 市场模拟器
pub struct MarketSimulator {
    config: MarketSimulationConfig,
    current_prices: Arc<RwLock<HashMap<String, f64>>>,
    price_history: Arc<RwLock<HashMap<String, Vec<MarketDataPoint>>>>,
    market_condition: Arc<RwLock<MarketCondition>>,
    price_simulations: Arc<RwLock<HashMap<String, PriceSimulation>>>,
    running: Arc<RwLock<bool>>,
    last_update: Arc<RwLock<Instant>>,
}

impl MarketSimulator {
    pub fn new(config: MarketSimulationConfig) -> Result<Self> {
        let mut initial_prices = HashMap::new();
        
        // 设置初始价格
        initial_prices.insert("BTC/USDT".to_string(), 45000.0);
        initial_prices.insert("ETH/USDT".to_string(), 3000.0);
        initial_prices.insert("BNB/USDT".to_string(), 300.0);
        initial_prices.insert("ADA/USDT".to_string(), 0.5);
        initial_prices.insert("SOL/USDT".to_string(), 100.0);

        Ok(Self {
            config,
            current_prices: Arc::new(RwLock::new(initial_prices)),
            price_history: Arc::new(RwLock::new(HashMap::new())),
            market_condition: Arc::new(RwLock::new(MarketCondition::Sideways)),
            price_simulations: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
            last_update: Arc::new(RwLock::new(Instant::now())),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        self.start_price_simulation().await;
        info!("Market simulator started");
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Market simulator stopped");
        Ok(())
    }

    pub async fn get_current_price(&self, symbol: &str) -> Result<f64> {
        let prices = self.current_prices.read().await;
        prices.get(symbol)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Symbol {} not found", symbol))
    }

    pub async fn is_symbol_supported(&self, symbol: &str) -> bool {
        let prices = self.current_prices.read().await;
        prices.contains_key(symbol)
    }

    pub async fn set_market_condition(&self, condition: MarketCondition) -> Result<()> {
        let mut current_condition = self.market_condition.write().await;
        *current_condition = condition;
        Ok(())
    }

    pub async fn add_price_simulation(&self, symbol: String, simulation: PriceSimulation) -> Result<()> {
        let mut simulations = self.price_simulations.write().await;
        simulations.insert(symbol, simulation);
        Ok(())
    }

    async fn start_price_simulation(&self) {
        let simulator = Arc::new(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_millis(simulator.config.price_update_interval_ms)
            );

            loop {
                interval.tick().await;
                
                let running = *simulator.running.read().await;
                if !running {
                    break;
                }

                if let Err(e) = simulator.update_prices().await {
                    debug!(error = %e, "Failed to update prices");
                }
            }
        });
    }

    async fn update_prices(&self) -> Result<()> {
        let now = Utc::now();
        let symbols: Vec<String> = {
            let prices = self.current_prices.read().await;
            prices.keys().cloned().collect()
        };

        for symbol in symbols {
            let new_price = self.simulate_price_movement(&symbol).await?;
            
            // 更新当前价格
            {
                let mut prices = self.current_prices.write().await;
                prices.insert(symbol.clone(), new_price);
            }

            // 添加到历史数据
            let data_point = MarketDataPoint {
                timestamp: now,
                price: new_price,
                volume: rand::thread_rng().gen_range(1000.0..10000.0),
                bid: new_price * 0.999,
                ask: new_price * 1.001,
                volatility: self.config.base_volatility,
            };

            let mut history = self.price_history.write().await;
            history.entry(symbol)
                .or_insert_with(Vec::new)
                .push(data_point);

            // 限制历史数据大小
            if let Some(symbol_history) = history.get_mut(&symbol) {
                if symbol_history.len() > 10000 {
                    symbol_history.remove(0);
                }
            }
        }

        *self.last_update.write().await = Instant::now();
        Ok(())
    }

    async fn simulate_price_movement(&self, symbol: &str) -> Result<f64> {
        let current_price = self.get_current_price(symbol).await?;
        let condition = self.market_condition.read().await;
        
        let mut rng = rand::thread_rng();
        let dt = self.config.price_update_interval_ms as f64 / 1000.0 / 3600.0; // 转换为小时

        // 基础波动率
        let mut volatility = self.config.base_volatility;
        
        // 根据市场条件调整参数
        let (drift, vol_multiplier) = match *condition {
            MarketCondition::Bull => (0.1, 0.8),    // 上涨趋势，低波动
            MarketCondition::Bear => (-0.1, 1.2),   // 下跌趋势，高波动
            MarketCondition::Sideways => (0.0, 1.0), // 无趋势，正常波动
            MarketCondition::Volatile => (0.0, 2.0), // 无趋势，高波动
        };

        volatility *= vol_multiplier;

        // 几何布朗运动
        let random_shock = rng.gen_range(-3.0..3.0) * (dt.sqrt());
        let price_change = drift * dt + volatility * random_shock;
        
        let mut new_price = current_price * (1.0 + price_change);

        // 跳跃模拟
        if self.config.enable_jump_simulation && rng.gen::<f64>() < self.config.jump_probability {
            let jump_direction = if rng.gen::<bool>() { 1.0 } else { -1.0 };
            let jump_size = rng.gen_range(0.5..2.0) * self.config.average_jump_size;
            new_price *= 1.0 + jump_direction * jump_size;
        }

        // 确保价格为正
        new_price = new_price.max(0.01);

        Ok(new_price)
    }

    pub async fn get_price_history(&self, symbol: &str, limit: Option<usize>) -> Vec<MarketDataPoint> {
        let history = self.price_history.read().await;
        if let Some(symbol_history) = history.get(symbol) {
            if let Some(limit) = limit {
                symbol_history.iter().rev().take(limit).cloned().collect()
            } else {
                symbol_history.clone()
            }
        } else {
            Vec::new()
        }
    }

    pub async fn get_market_data(&self, symbol: &str) -> Result<MarketDataPoint> {
        let price = self.get_current_price(symbol).await?;
        let spread_pct = 0.001; // 0.1% spread
        
        Ok(MarketDataPoint {
            timestamp: Utc::now(),
            price,
            volume: rand::thread_rng().gen_range(1000.0..10000.0),
            bid: price * (1.0 - spread_pct),
            ask: price * (1.0 + spread_pct),
            volatility: self.config.base_volatility,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_market_simulator_creation() {
        let config = MarketSimulationConfig::default();
        let simulator = MarketSimulator::new(config);
        assert!(simulator.is_ok());
    }

    #[tokio::test]
    async fn test_price_retrieval() {
        let config = MarketSimulationConfig::default();
        let simulator = MarketSimulator::new(config).unwrap();
        
        let price = simulator.get_current_price("BTC/USDT").await;
        assert!(price.is_ok());
        assert!(price.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_market_condition() {
        let config = MarketSimulationConfig::default();
        let simulator = MarketSimulator::new(config).unwrap();
        
        let result = simulator.set_market_condition(MarketCondition::Bull).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_simulator_lifecycle() {
        let config = MarketSimulationConfig::default();
        let simulator = MarketSimulator::new(config).unwrap();
        
        let start_result = simulator.start().await;
        assert!(start_result.is_ok());
        
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let stop_result = simulator.stop().await;
        assert!(stop_result.is_ok());
    }
}