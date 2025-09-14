#![allow(dead_code)]
//! # 渐进式数据清洗器
//!
//! 实现智能的多阶段数据清洗，根据数据质量动态调整清洗策略

#[allow(dead_code)]
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug, instrument};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::types::*;
use crate::errors::MarketDataError;

/// 渐进式清洗配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProgressiveCleaningConfig {
    /// 基础清洗阈值
    pub basic_threshold: f64,
    /// 深度清洗阈值
    pub deep_threshold: f64,
    /// 激进清洗阈值
    pub aggressive_threshold: f64,
    /// 最大处理时间（毫秒）
    pub max_processing_time_ms: u64,
    /// 启用自适应调整
    pub enable_adaptive_adjustment: bool,
    /// 质量评估窗口大小
    pub quality_window_size: usize,
    /// 错误率阈值
    pub error_rate_threshold: f64,
}

impl Default for ProgressiveCleaningConfig {
    fn default() -> Self {
        Self {
            basic_threshold: 0.8,
            deep_threshold: 0.6,
            aggressive_threshold: 0.4,
            max_processing_time_ms: 100,
            enable_adaptive_adjustment: true,
            quality_window_size: 100,
            error_rate_threshold: 0.05,
        }
    }
}

/// 渐进式清洗统计
#[derive(Debug, Clone, Serialize)]
pub struct ProgressiveCleaningStats {
    /// 总处理数量
    pub total_processed: u64,
    /// 成功率
    pub success_rate: f64,
    /// 平均处理时间（纳秒）
    pub avg_processing_time_ns: u64,
    /// 各阶段使用次数
    pub basic_cleanings: u64,
    pub deep_cleanings: u64,
    pub aggressive_cleanings: u64,
    /// 自适应调整次数
    pub adaptive_adjustments: u64,
    /// 当前质量评分
    pub current_quality_score: f64,
}

impl Default for ProgressiveCleaningStats {
    fn default() -> Self {
        Self {
            total_processed: 0,
            success_rate: 1.0,
            avg_processing_time_ns: 0,
            basic_cleanings: 0,
            deep_cleanings: 0,
            aggressive_cleanings: 0,
            adaptive_adjustments: 0,
            current_quality_score: 1.0,
        }
    }
}

/// 清洗阶段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CleaningStage {
    Basic,
    Deep,
    Aggressive,
}

/// 质量评估结果
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct QualityAssessment {
    score: f64,
    issues: Vec<String>,
    recommended_stage: CleaningStage,
}

/// 渐进式数据清洗器
#[allow(dead_code)]
pub struct ProgressiveDataCleaner {
    config: Arc<RwLock<ProgressiveCleaningConfig>>,
    stats: Arc<RwLock<ProgressiveCleaningStats>>,
    quality_history: Arc<RwLock<Vec<f64>>>,
    start_time: Instant,
}

impl ProgressiveDataCleaner {
    /// 创建新的渐进式清洗器
    pub fn new(config: ProgressiveCleaningConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            stats: Arc::new(RwLock::new(ProgressiveCleaningStats::default())),
            quality_history: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
        }
    }

    /// 渐进式清洗数据
    #[instrument(skip(self, data))]
    pub async fn clean_progressive(&self, data: MarketDataSnapshot) -> Result<MarketDataSnapshot, MarketDataError> {
        let start_time = Instant::now();
        
        // 1. 评估数据质量
        let quality_assessment = self.assess_data_quality(&data).await;
        
        // 2. 根据质量选择清洗策略
        let cleaning_result = match quality_assessment.recommended_stage {
            CleaningStage::Basic => self.basic_cleaning(data).await,
            CleaningStage::Deep => self.deep_cleaning(data).await,
            CleaningStage::Aggressive => self.aggressive_cleaning(data).await,
        };

        // 3. 更新统计信息
        self.update_stats(quality_assessment.recommended_stage, start_time.elapsed()).await;

        // 4. 记录质量历史
        self.record_quality_score(quality_assessment.score).await;

        // 5. 自适应调整（如果启用）
        if self.config.read().await.enable_adaptive_adjustment {
            self.adaptive_adjustment().await;
        }

        cleaning_result
    }

    /// 评估数据质量
    async fn assess_data_quality(&self, data: &MarketDataSnapshot) -> QualityAssessment {
        let mut score = 1.0;
        let mut issues = Vec::new();

        // 检查订单簿质量
        if let Some(ref orderbook) = data.orderbook {
            let orderbook_score = self.assess_orderbook_quality(orderbook);
            score *= orderbook_score;
            
            if orderbook_score < 0.8 {
                issues.push("订单簿数据质量较低".to_string());
            }
        }

        // 检查交易数据质量
        let trades_score = self.assess_trades_quality(&data.trades);
        score *= trades_score;
        
        if trades_score < 0.8 {
            issues.push("交易数据质量较低".to_string());
        }

        // 检查时间戳
        let timestamp_score = self.assess_timestamp_quality(data.timestamp);
        score *= timestamp_score;
        
        if timestamp_score < 0.8 {
            issues.push("时间戳质量较低".to_string());
        }

        // 推荐清洗阶段
        let config = self.config.read().await;
        let recommended_stage = if score >= config.basic_threshold {
            CleaningStage::Basic
        } else if score >= config.deep_threshold {
            CleaningStage::Deep
        } else {
            CleaningStage::Aggressive
        };

        QualityAssessment {
            score,
            issues,
            recommended_stage,
        }
    }

    /// 评估订单簿质量
    fn assess_orderbook_quality(&self, orderbook: &OrderBook) -> f64 {
        let mut score = 1.0;

        // 检查买卖单数量
        if orderbook.bids.is_empty() || orderbook.asks.is_empty() {
            score *= 0.5; // 单边订单簿降低评分
        }

        // 检查价格合理性
        if !orderbook.bids.is_empty() && !orderbook.asks.is_empty() {
            let best_bid = orderbook.bids[0].price.0;
            let best_ask = orderbook.asks[0].price.0;
            
            if best_bid >= best_ask {
                score *= 0.3; // 价格倒挂严重降低评分
            }
            
            let spread_ratio = (best_ask - best_bid) / best_ask;
            if spread_ratio > 0.1 {
                score *= 0.7; // 过大价差降低评分
            }
        }

        // 检查数量合理性
        let total_bids = orderbook.bids.len();
        let total_asks = orderbook.asks.len();
        
        if total_bids < 5 || total_asks < 5 {
            score *= 0.8; // 深度不足降低评分
        }

        score
    }

    /// 评估交易数据质量
    fn assess_trades_quality(&self, trades: &[TradeUpdate]) -> f64 {
        if trades.is_empty() {
            return 1.0; // 没有交易数据不算质量问题
        }

        let mut score = 1.0;

        // 检查价格和数量的合理性
        for trade in trades {
            if trade.price.0 <= 0.0 || trade.quantity.0 <= 0.0 {
                score *= 0.5; // 无效数据严重降低评分
                break;
            }
        }

        // 检查时间顺序
        if trades.len() > 1 {
            let mut time_ordered = true;
            for i in 1..trades.len() {
                if trades[i].timestamp < trades[i-1].timestamp {
                    time_ordered = false;
                    break;
                }
            }
            
            if !time_ordered {
                score *= 0.7; // 时间顺序错误降低评分
            }
        }

        score
    }

    /// 评估时间戳质量
    fn assess_timestamp_quality(&self, timestamp: crate::high_precision_time::Nanos) -> f64 {
        let now = crate::high_precision_time::Nanos::now();
        let age_nanos = now.0.saturating_sub(timestamp.0);
        let age_secs = age_nanos / 1_000_000_000; // 纳秒转换为秒
        
        // 数据越新质量越高
        if age_secs < 1 {
            1.0
        } else if age_secs < 5 {
            0.9
        } else if age_secs < 30 {
            0.7
        } else {
            0.5
        }
    }

    /// 基础清洗
    async fn basic_cleaning(&self, mut data: MarketDataSnapshot) -> Result<MarketDataSnapshot, MarketDataError> {
        debug!("执行基础清洗");

        // 基础订单簿清洗
        if let Some(ref mut orderbook) = data.orderbook {
            // 移除零数量和零价格条目
            orderbook.bids.retain(|entry| entry.quantity.0 > 0.0 && entry.price.0 > 0.0);
            orderbook.asks.retain(|entry| entry.quantity.0 > 0.0 && entry.price.0 > 0.0);
            
            // 基础排序
            orderbook.bids.sort_by(|a, b| b.price.cmp(&a.price));
            orderbook.asks.sort_by(|a, b| a.price.cmp(&b.price));
        }

        // 基础交易数据清洗
        data.trades.retain(|trade| trade.price.0 > 0.0 && trade.quantity.0 > 0.0);
        data.trades.sort_by_key(|trade| trade.timestamp);

        Ok(data)
    }

    /// 深度清洗
    async fn deep_cleaning(&self, mut data: MarketDataSnapshot) -> Result<MarketDataSnapshot, MarketDataError> {
        debug!("执行深度清洗");

        // 先执行基础清洗
        data = self.basic_cleaning(data).await?;

        // 深度订单簿清洗
        if let Some(ref mut orderbook) = data.orderbook {
            // 移除异常价格条目
            if !orderbook.bids.is_empty() && !orderbook.asks.is_empty() {
                let best_bid = orderbook.bids[0].price.0;
                let best_ask = orderbook.asks[0].price.0;
                
                if best_bid > 0.0 && best_ask > 0.0 {
                    let mid_price = (best_bid + best_ask) / 2.0;
                    let tolerance = mid_price * 0.1; // 10%容忍度
                    
                    orderbook.bids.retain(|entry| {
                        (entry.price.0 - best_bid).abs() <= tolerance
                    });
                    
                    orderbook.asks.retain(|entry| {
                        (entry.price.0 - best_ask).abs() <= tolerance
                    });
                }
            }
            
            // 限制深度
            orderbook.bids.truncate(50);
            orderbook.asks.truncate(50);
        }

        // 深度交易数据清洗
        if !data.trades.is_empty() {
            // 移除明显异常的交易
            let prices: Vec<f64> = data.trades.iter().map(|t| t.price.0).collect();
            if let (Some(&min_price), Some(&max_price)) = (
                prices.iter().min_by(|a, b| a.partial_cmp(b).expect("Failed to compare prices")), 
                prices.iter().max_by(|a, b| a.partial_cmp(b).expect("Failed to compare prices"))
            ) {
                let price_range = max_price - min_price;
                let tolerance = price_range * 0.2; // 20%容忍度
                
                data.trades.retain(|trade| {
                    trade.price.0 >= min_price - tolerance && 
                    trade.price.0 <= max_price + tolerance
                });
            }
        }

        Ok(data)
    }

    /// 激进清洗
    async fn aggressive_cleaning(&self, mut data: MarketDataSnapshot) -> Result<MarketDataSnapshot, MarketDataError> {
        debug!("执行激进清洗");

        // 先执行深度清洗
        data = self.deep_cleaning(data).await?;

        // 激进订单簿清洗
        if let Some(ref mut orderbook) = data.orderbook {
            // 严格的价格一致性检查
            if !orderbook.bids.is_empty() && !orderbook.asks.is_empty() {
                let best_bid = orderbook.bids[0].price.0;
                let best_ask = orderbook.asks[0].price.0;
                
                // 如果价格倒挂，重建订单簿
                if best_bid >= best_ask {
                    warn!("检测到价格倒挂，重建订单簿");
                    orderbook.bids.clear();
                    orderbook.asks.clear();
                }
            }
            
            // 严格限制深度
            orderbook.bids.truncate(20);
            orderbook.asks.truncate(20);
            
            // 移除微小数量的订单
            let min_quantity = 0.001;
            orderbook.bids.retain(|entry| entry.quantity.0 >= min_quantity);
            orderbook.asks.retain(|entry| entry.quantity.0 >= min_quantity);
        }

        // 激进交易数据清洗
        data.trades.truncate(100); // 只保留最近的100笔交易

        Ok(data)
    }

    /// 更新统计信息
    async fn update_stats(&self, stage: CleaningStage, duration: Duration) {
        let mut stats = self.stats.write().await;
        
        stats.total_processed += 1;
        
        match stage {
            CleaningStage::Basic => stats.basic_cleanings += 1,
            CleaningStage::Deep => stats.deep_cleanings += 1,
            CleaningStage::Aggressive => stats.aggressive_cleanings += 1,
        }
        
        // 更新平均处理时间
        let new_time_ns = duration.as_nanos() as u64;
        if stats.total_processed == 1 {
            stats.avg_processing_time_ns = new_time_ns;
        } else {
            stats.avg_processing_time_ns = 
                (stats.avg_processing_time_ns * (stats.total_processed - 1) + new_time_ns) / stats.total_processed;
        }
    }

    /// 记录质量评分
    async fn record_quality_score(&self, score: f64) {
        let mut history = self.quality_history.write().await;
        let config = self.config.read().await;
        
        history.push(score);
        
        // 保持历史记录在窗口大小内
        if history.len() > config.quality_window_size {
            history.remove(0);
        }
        
        // 更新当前质量评分
        let mut stats = self.stats.write().await;
        stats.current_quality_score = score;
    }

    /// 自适应调整
    async fn adaptive_adjustment(&self) {
        let history = self.quality_history.read().await;
        
        if history.len() < 10 {
            return; // 数据不足，不进行调整
        }
        
        let recent_avg = history.iter().rev().take(10).sum::<f64>() / 10.0;
        let overall_avg = history.iter().sum::<f64>() / history.len() as f64;
        
        let mut config = self.config.write().await;
        let mut stats = self.stats.write().await;
        
        // 如果最近质量下降，降低阈值（更激进清洗）
        if recent_avg < overall_avg * 0.9 {
            config.basic_threshold = (config.basic_threshold * 0.95).max(0.5);
            config.deep_threshold = (config.deep_threshold * 0.95).max(0.3);
            config.aggressive_threshold = (config.aggressive_threshold * 0.95).max(0.1);
            
            stats.adaptive_adjustments += 1;
            info!("自适应调整：降低清洗阈值以提高数据质量");
        }
        // 如果质量持续良好，可以适当提高阈值
        else if recent_avg > overall_avg * 1.1 {
            config.basic_threshold = (config.basic_threshold * 1.02).min(0.95);
            config.deep_threshold = (config.deep_threshold * 1.02).min(0.8);
            config.aggressive_threshold = (config.aggressive_threshold * 1.02).min(0.6);
            
            stats.adaptive_adjustments += 1;
            info!("自适应调整：提高清洗阈值以优化性能");
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ProgressiveCleaningStats {
        let stats = self.stats.read().await;
        let mut result = stats.clone();
        
        // 计算成功率（简化实现）
        if result.total_processed > 0 {
            result.success_rate = 1.0 - (result.aggressive_cleanings as f64 / result.total_processed as f64 * 0.1);
        }
        
        result
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: ProgressiveCleaningConfig) {
        let mut config = self.config.write().await;
        *config = new_config;
        info!("渐进式清洗器配置已更新");
    }

    /// 重置统计信息
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = ProgressiveCleaningStats::default();
        
        let mut history = self.quality_history.write().await;
        history.clear();
        
        info!("渐进式清洗器统计信息已重置");
    }
}
