#![allow(dead_code)]
//! # 一致性检查模块
//!
//! 负责跨交易所的价格、时间、交易量一致性验证

use std::sync::Arc;
use std::collections::HashMap;

use tokio::sync::RwLock;
use tracing::{info, error, debug, instrument};
use async_trait::async_trait;

use crate::types::*;
use crate::errors::MarketDataError;
use crate::high_precision_time::Nanos;

/// 一致性检查结果
#[derive(Debug, Clone)]
pub struct ConsistencyResult {
    pub symbol: Symbol,
    pub timestamp: Nanos,
    pub check_type: ConsistencyCheckType,
    pub severity: ConsistencySeverity,
    pub message: String,
    pub exchanges_involved: Vec<String>,
    pub values: HashMap<String, f64>,
}

/// 一致性检查类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConsistencyCheckType {
    PriceSpread,
    TimeSynchronization,
    VolumeConsistency,
    OrderBookDepth,
}

/// 一致性问题严重程度
#[derive(Debug, Clone, PartialEq)]
pub enum ConsistencySeverity {
    Info,
    Warning,
    Critical,
}

/// 一致性检查器特性
#[async_trait]
pub trait ConsistencyChecker: Send + Sync {
    /// 检查一致性
    async fn check_consistency(&self, data: &[MarketDataSnapshot]) -> Vec<ConsistencyResult>;
    
    /// 启动一致性检查器
    async fn start(&mut self) -> Result<(), MarketDataError>;
    
    /// 停止一致性检查器
    async fn stop(&mut self) -> Result<(), MarketDataError>;
}

/// 跨交易所一致性检查器
pub struct CrossExchangeConsistencyChecker {
    /// 配置阈值
    thresholds: ConsistencyThresholds,
    /// 输入数据通道
    input_rx: Arc<RwLock<Option<flume::Receiver<Vec<MarketDataSnapshot>>>>>,
    /// 结果输出通道
    output_tx: flume::Sender<ConsistencyResult>,
    /// 运行状态
    is_running: Arc<RwLock<bool>>,
    /// 处理任务句柄
    task_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// 最近的数据缓存
    recent_data: Arc<RwLock<HashMap<String, MarketDataSnapshot>>>,
}

impl CrossExchangeConsistencyChecker {
    /// 创建新的一致性检查器
    pub fn new(
        thresholds: ConsistencyThresholds,
        input_rx: flume::Receiver<Vec<MarketDataSnapshot>>,
        output_tx: flume::Sender<ConsistencyResult>,
    ) -> Self {
        Self {
            thresholds,
            input_rx: Arc::new(RwLock::new(Some(input_rx))),
            output_tx,
            is_running: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(RwLock::new(None)),
            recent_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 检查价格差异
    #[instrument(skip(self, snapshots))]
    async fn check_price_spread(&self, snapshots: &[MarketDataSnapshot]) -> Vec<ConsistencyResult> {
        let mut results = Vec::new();
        
        // 按交易对分组
        let mut by_symbol: HashMap<Symbol, Vec<&MarketDataSnapshot>> = HashMap::new();
        for snapshot in snapshots {
            if let Some(ref ob) = snapshot.orderbook {
                by_symbol.entry(ob.symbol.clone()).or_default().push(snapshot);
            }
        }

        for (symbol, symbol_snapshots) in by_symbol {
            if symbol_snapshots.len() < 2 {
                continue; // 至少需要两个交易所的数据
            }

            let mut mid_prices = HashMap::new();
            
            // 计算每个交易所的中间价
            for snapshot in symbol_snapshots {
                    if let Some(ref ob) = snapshot.orderbook {
                        if let (Some(bid), Some(ask)) = (ob.best_bid(), ob.best_ask()) {
                            let mid_price = (bid.price.0 + ask.price.0) / 2.0;
                            mid_prices.insert(snapshot.source.clone(), mid_price);
                        }
                    }
            }

            if mid_prices.len() < 2 {
                continue;
            }

            // 计算价格差异
            let prices: Vec<f64> = mid_prices.values().cloned().collect();
            let max_price = prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let min_price = prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            
            let spread_percentage = ((max_price - min_price) / min_price) * 100.0;
            
            let severity = if spread_percentage > self.thresholds.critical_spread_threshold_percentage {
                ConsistencySeverity::Critical
            } else if spread_percentage > self.thresholds.spread_threshold_percentage {
                ConsistencySeverity::Warning
            } else {
                ConsistencySeverity::Info
            };

            if spread_percentage > self.thresholds.spread_threshold_percentage {
                results.push(ConsistencyResult {
                    symbol: symbol.clone(),
                    timestamp: Nanos::now(),
                    check_type: ConsistencyCheckType::PriceSpread,
                    severity,
                    message: format!(
                        "Price spread of {:.2}% detected across exchanges",
                        spread_percentage
                    ),
                    exchanges_involved: mid_prices.keys().cloned().collect(),
                    values: mid_prices,
                });
            }
        }

        results
    }

    /// 检查时间同步
    #[instrument(skip(self, snapshots))]
    async fn check_time_synchronization(&self, snapshots: &[MarketDataSnapshot]) -> Vec<ConsistencyResult> {
        let mut results = Vec::new();
        
        if snapshots.len() < 2 {
            return results;
        }

        let timestamps: Vec<i64> = snapshots.iter().map(|s| s.timestamp.as_nanos()).collect();
        let max_timestamp = timestamps.iter().max().expect("Empty collection");
        let min_timestamp = timestamps.iter().min().expect("Empty collection");
        
        let time_diff_ms = (max_timestamp - min_timestamp) as f64 / 1_000_000.0;
        
        if time_diff_ms > self.thresholds.max_time_diff_ms {
            let severity = if time_diff_ms > self.thresholds.max_time_diff_ms * 2.0 {
                ConsistencySeverity::Critical
            } else {
                ConsistencySeverity::Warning
            };

            let mut exchanges_involved = Vec::new();
            let mut values = HashMap::new();
            
            for snapshot in snapshots {
                exchanges_involved.push(snapshot.source.clone());
                values.insert(
                    snapshot.source.clone(),
                    snapshot.timestamp.as_nanos() as f64 / 1_000_000.0,
                );
            }

            results.push(ConsistencyResult {
                symbol: Symbol::new("ALL", "ALL"), // 时间同步影响所有交易对
                timestamp: Nanos::now(),
                check_type: ConsistencyCheckType::TimeSynchronization,
                severity,
                message: format!(
                    "Time synchronization issue: {:.2}ms difference detected",
                    time_diff_ms
                ),
                exchanges_involved,
                values,
            });
        }

        results
    }

    /// 检查交易量一致性
    #[instrument(skip(self, snapshots))]
    async fn check_volume_consistency(&self, snapshots: &[MarketDataSnapshot]) -> Vec<ConsistencyResult> {
        let mut results = Vec::new();
        
        // 按交易对分组
        let mut by_symbol: HashMap<Symbol, Vec<&MarketDataSnapshot>> = HashMap::new();
        for snapshot in snapshots {
            if let Some(ref ob) = snapshot.orderbook {
                by_symbol.entry(ob.symbol.clone()).or_default().push(snapshot);
            }
        }

        for (symbol, symbol_snapshots) in by_symbol {
            if symbol_snapshots.len() < 2 {
                continue;
            }

            let mut volumes = HashMap::new();
            
            // 计算每个交易所的订单簿深度
            for snapshot in symbol_snapshots {
                if let Some(ref ob) = snapshot.orderbook {
                    let bid_volume: f64 = ob.bids.iter()
                        .take(10) // 前10档
                        .map(|entry| entry.quantity.0)
                        .sum();
                    let ask_volume: f64 = ob.asks.iter()
                        .take(10) // 前10档
                        .map(|entry| entry.quantity.0)
                        .sum();
                    
                    let total_volume = bid_volume + ask_volume;
                    volumes.insert(snapshot.source.clone(), total_volume);
                }
            }

            if volumes.len() < 2 {
                continue;
            }

            // 计算交易量变异系数
            let volume_values: Vec<f64> = volumes.values().cloned().collect();
            let mean = volume_values.iter().sum::<f64>() / volume_values.len() as f64;
            let variance = volume_values.iter()
                .map(|v| (v - mean).powi(2))
                .sum::<f64>() / volume_values.len() as f64;
            let std_dev = variance.sqrt();
            let coefficient_of_variation = if mean > 0.0 { std_dev / mean } else { 0.0 };

            if coefficient_of_variation > self.thresholds.volume_consistency_threshold {
                let severity = if coefficient_of_variation > self.thresholds.volume_consistency_threshold * 2.0 {
                    ConsistencySeverity::Critical
                } else {
                    ConsistencySeverity::Warning
                };

                results.push(ConsistencyResult {
                    symbol: symbol.clone(),
                    timestamp: Nanos::now(),
                    check_type: ConsistencyCheckType::VolumeConsistency,
                    severity,
                    message: format!(
                        "Volume inconsistency detected: CV = {:.2}",
                        coefficient_of_variation
                    ),
                    exchanges_involved: volumes.keys().cloned().collect(),
                    values: volumes,
                });
            }
        }

        results
    }
}

#[async_trait]
impl ConsistencyChecker for CrossExchangeConsistencyChecker {
    #[instrument(skip(self, data))]
    async fn check_consistency(&self, data: &[MarketDataSnapshot]) -> Vec<ConsistencyResult> {
        let mut all_results = Vec::new();
        
        // 执行所有一致性检查
        let price_results = self.check_price_spread(data).await;
        let time_results = self.check_time_synchronization(data).await;
        let volume_results = self.check_volume_consistency(data).await;
        
        all_results.extend(price_results);
        all_results.extend(time_results);
        all_results.extend(volume_results);
        
        debug!("Generated {} consistency check results", all_results.len());
        all_results
    }

    async fn start(&mut self) -> Result<(), MarketDataError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Ok(());
        }

        let input_rx = {
            let mut rx_lock = self.input_rx.write().await;
            rx_lock.take().ok_or_else(|| MarketDataError::InternalError(
                "Input channel already consumed".to_string()
            ))?
        };

        let checker = Arc::new(self.clone());
        let output_tx = self.output_tx.clone();
        let running_flag = self.is_running.clone();

        let handle = tokio::spawn(async move {
            info!("Consistency checker started");
            
            while *running_flag.read().await {
                match input_rx.recv_async().await {
                    Ok(snapshots) => {
                        let results = checker.check_consistency(&snapshots).await;
                        
                        for result in results {
                            if let Err(e) = output_tx.send_async(result).await {
                                error!("Failed to send consistency result: {}", e);
                            }
                        }
                    },
                    Err(_) => {
                        info!("Input channel closed, stopping consistency checker");
                        break;
                    }
                }
            }
            
            info!("Consistency checker stopped");
        });

        {
            let mut task_handle = self.task_handle.write().await;
            *task_handle = Some(handle);
        }

        *is_running = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), MarketDataError> {
        info!("Stopping consistency checker");
        
        {
            let mut is_running = self.is_running.write().await;
            *is_running = false;
        }

        if let Some(handle) = {
            let mut task_handle = self.task_handle.write().await;
            task_handle.take()
        } {
            if let Err(e) = handle.await {
                error!("Consistency checker task failed: {:?}", e);
            }
        }

        Ok(())
    }
}

// 实现Clone以便在start方法中创建Arc
impl Clone for CrossExchangeConsistencyChecker {
    fn clone(&self) -> Self {
        Self {
            thresholds: self.thresholds.clone(),
            input_rx: self.input_rx.clone(),
            output_tx: self.output_tx.clone(),
            is_running: self.is_running.clone(),
            task_handle: self.task_handle.clone(),
            recent_data: self.recent_data.clone(),
        }
    }
}
