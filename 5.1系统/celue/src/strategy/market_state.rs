 
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use crate::strategy::core::StrategyError;

/// 市场状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketState {
    Normal,   // 常规市场
    Cautious, // 谨慎模式
    Extreme,  // 极端行情
}

/// 市场指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIndicators {
    /// 历史波动率 (1h, 4h, 24h)
    pub volatility_1h: f64,
    pub volatility_4h: f64,
    pub volatility_24h: f64,
    
    /// 盘口深度与流动性
    pub liquidity_index: f64,
    pub bid_ask_spread: f64,
    pub order_book_depth: f64,
    
    /// 成交量突变指标
    pub volume_ratio_1h: f64,
    pub volume_ratio_4h: f64,
    pub volume_spike_detected: bool,
    
    /// 价格跳变/滑点
    pub max_price_change_1m: f64,
    pub max_price_change_5m: f64,
    pub average_slippage: f64,
    
    /// API健康度
    pub api_latency_avg: f64,
    pub api_error_rate: f64,
    pub api_success_rate: f64,
    
    /// 外部事件标记
    pub external_event_risk: f64,
    pub news_sentiment_score: f64,
    
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl Default for MarketIndicators {
    fn default() -> Self {
        Self {
            volatility_1h: 0.0,
            volatility_4h: 0.0,
            volatility_24h: 0.0,
            liquidity_index: 1.0,
            bid_ask_spread: 0.001,
            order_book_depth: 1.0,
            volume_ratio_1h: 1.0,
            volume_ratio_4h: 1.0,
            volume_spike_detected: false,
            max_price_change_1m: 0.0,
            max_price_change_5m: 0.0,
            average_slippage: 0.001,
            api_latency_avg: 100.0,
            api_error_rate: 0.0,
            api_success_rate: 1.0,
            external_event_risk: 0.0,
            news_sentiment_score: 0.5,
            timestamp: Utc::now(),
        }
    }
}

/// 市场状态判定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStateConfig {
    /// 波动率阈值 (倍数)
    pub volatility_normal_threshold: f64,
    pub volatility_extreme_threshold: f64,
    
    /// 流动性阈值
    pub liquidity_normal_threshold: f64,
    pub liquidity_extreme_threshold: f64,
    
    /// 成交量突变阈值
    pub volume_spike_threshold: f64,
    pub volume_extreme_threshold: f64,
    
    /// 价格跳变阈值
    pub price_change_normal_threshold: f64,
    pub price_change_extreme_threshold: f64,
    
    /// API健康度阈值
    pub api_latency_normal_threshold: f64,
    pub api_latency_extreme_threshold: f64,
    pub api_error_rate_threshold: f64,
    
    /// 外部事件风险阈值
    pub external_risk_threshold: f64,
    
    /// 状态切换持续性要求 (分钟)
    pub state_change_persistence_minutes: i64,
    
    /// 多指标共振要求 (需要多少个指标同时异常)
    pub indicator_consensus_count: usize,
    
    /// 权重配置
    pub weights: IndicatorWeights,
}

/// 指标权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorWeights {
    pub volatility_weight: f64,
    pub liquidity_weight: f64,
    pub volume_weight: f64,
    pub price_change_weight: f64,
    pub api_health_weight: f64,
    pub external_event_weight: f64,
}

impl Default for MarketStateConfig {
    fn default() -> Self {
        Self {
            volatility_normal_threshold: 1.5,
            volatility_extreme_threshold: 2.5,
            liquidity_normal_threshold: 0.8,
            liquidity_extreme_threshold: 0.3,
            volume_spike_threshold: 2.0,
            volume_extreme_threshold: 3.0,
            price_change_normal_threshold: 0.02, // 2%
            price_change_extreme_threshold: 0.05, // 5%
            api_latency_normal_threshold: 500.0, // ms
            api_latency_extreme_threshold: 2000.0, // ms
            api_error_rate_threshold: 0.05, // 5%
            external_risk_threshold: 0.7,
            state_change_persistence_minutes: 5,
            indicator_consensus_count: 3,
            weights: IndicatorWeights {
                volatility_weight: 0.25,
                liquidity_weight: 0.25,
                volume_weight: 0.2,
                price_change_weight: 0.15,
                api_health_weight: 0.1,
                external_event_weight: 0.05,
            },
        }
    }
}

/// 状态变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeRecord {
    pub change_id: String,
    pub timestamp: DateTime<Utc>,
    pub old_state: MarketState,
    pub new_state: MarketState,
    pub risk_score: f64,
    pub triggered_indicators: Vec<String>,
    pub persistence_met: bool,
    pub consensus_met: bool,
    pub manual_override: bool,
}

/// 历史数据点
#[derive(Debug, Clone)]
struct HistoricalDataPoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
    pub api_latency: f64,
}

/// 市场状态判定器
pub struct MarketStateJudge {
    /// 配置
    config: Arc<RwLock<MarketStateConfig>>,
    
    /// 当前市场状态
    current_state: Arc<RwLock<MarketState>>,
    
    /// 历史指标数据
    historical_indicators: Arc<RwLock<Vec<MarketIndicators>>>,
    
    /// 状态变更历史
    state_change_history: Arc<RwLock<Vec<StateChangeRecord>>>,
    
    /// 历史价格数据 (用于波动率计算)
    price_history: Arc<RwLock<HashMap<String, Vec<HistoricalDataPoint>>>>,
    
    /// 当前状态开始时间 (用于持续性判断)
    current_state_start: Arc<RwLock<DateTime<Utc>>>,
    
    /// 异常指标计数 (用于共振判断)
    abnormal_indicators: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl MarketStateJudge {
    /// 创建新的市场状态判定器
    pub fn new(config: Option<MarketStateConfig>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config.unwrap_or_default())),
            current_state: Arc::new(RwLock::new(MarketState::Normal)),
            historical_indicators: Arc::new(RwLock::new(Vec::new())),
            state_change_history: Arc::new(RwLock::new(Vec::new())),
            price_history: Arc::new(RwLock::new(HashMap::new())),
            current_state_start: Arc::new(RwLock::new(Utc::now())),
            abnormal_indicators: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 更新价格数据
    pub async fn update_price_data(&self, symbol: &str, price: f64, volume: f64, api_latency: f64) {
        let data_point = HistoricalDataPoint {
            timestamp: Utc::now(),
            price,
            volume,
            api_latency,
        };

        let mut price_history = self.price_history.write().await;
        let symbol_history = price_history.entry(symbol.to_string()).or_insert_with(Vec::new);
        symbol_history.push(data_point);

        // 保留最近24小时的数据
        let cutoff_time = Utc::now() - Duration::hours(24);
        symbol_history.retain(|point| point.timestamp > cutoff_time);
    }

    /// 计算市场指标
    pub async fn calculate_market_indicators(&self, symbol: &str) -> Result<MarketIndicators, StrategyError> {
        let price_history = self.price_history.read().await;
        
        if let Some(symbol_history) = price_history.get(symbol) {
            if symbol_history.is_empty() {
                return Ok(MarketIndicators::default());
            }

            let now = Utc::now();
            
            // 计算波动率
            let volatility_1h = self.calculate_volatility(symbol_history, Duration::hours(1), now);
            let volatility_4h = self.calculate_volatility(symbol_history, Duration::hours(4), now);
            let volatility_24h = self.calculate_volatility(symbol_history, Duration::hours(24), now);
            
            // 计算成交量比率
            let volume_ratio_1h = self.calculate_volume_ratio(symbol_history, Duration::hours(1), now);
            let volume_ratio_4h = self.calculate_volume_ratio(symbol_history, Duration::hours(4), now);
            
            // 计算价格跳变
            let max_price_change_1m = self.calculate_max_price_change(symbol_history, Duration::minutes(1), now);
            let max_price_change_5m = self.calculate_max_price_change(symbol_history, Duration::minutes(5), now);
            
            // 计算API健康度
            let api_latency_avg = self.calculate_average_api_latency(symbol_history, Duration::hours(1), now);
            
            // 检测成交量突变
            let volume_spike_detected = volume_ratio_1h > 2.0 || volume_ratio_4h > 1.8;
            
            Ok(MarketIndicators {
                volatility_1h,
                volatility_4h,
                volatility_24h,
                liquidity_index: 1.0, // 需要从orderbook数据计算
                bid_ask_spread: 0.001, // 需要从orderbook数据计算
                order_book_depth: 1.0, // 需要从orderbook数据计算
                volume_ratio_1h,
                volume_ratio_4h,
                volume_spike_detected,
                max_price_change_1m,
                max_price_change_5m,
                average_slippage: 0.001, // 需要从交易数据计算
                api_latency_avg,
                api_error_rate: 0.0, // 需要从API监控数据计算
                api_success_rate: 1.0, // 需要从API监控数据计算
                external_event_risk: 0.0, // 需要外部数据源
                news_sentiment_score: 0.5, // 需要新闻情感分析
                timestamp: now,
            })
        } else {
            Ok(MarketIndicators::default())
        }
    }

    /// 判定市场状态
    pub async fn judge_market_state(&self, indicators: &MarketIndicators) -> Result<MarketState, StrategyError> {
        let config = self.config.read().await;
        
        // 计算风险得分
        let risk_score = self.calculate_risk_score(indicators, &config);
        
        // 确定目标状态
        let target_state = if risk_score < 0.3 {
            MarketState::Normal
        } else if risk_score < 0.7 {
            MarketState::Cautious
        } else {
            MarketState::Extreme
        };

        // 检查指标异常情况
        let abnormal_indicators = self.identify_abnormal_indicators(indicators, &config).await;
        
        // 检查共振条件
        let consensus_met = abnormal_indicators.len() >= config.indicator_consensus_count;
        
        // 检查持续性条件
        let persistence_met = self.check_persistence(&target_state, &config).await;
        
        let current_state = *self.current_state.read().await;
        
        // 决定是否切换状态
        let should_change_state = target_state != current_state && (consensus_met || persistence_met);
        
        if should_change_state {
            self.change_market_state(target_state, risk_score, abnormal_indicators.clone(), persistence_met, consensus_met).await?;
        }

        // 更新异常指标记录
        self.update_abnormal_indicators(&abnormal_indicators).await;
        
        // 记录历史指标
        {
            let mut historical = self.historical_indicators.write().await;
            historical.push(indicators.clone());
            
            // 保留最近1000条记录
            if historical.len() > 1000 {
                historical.remove(0);
            }
        }

        Ok(*self.current_state.read().await)
    }

    /// 手动设置市场状态
    pub async fn set_market_state_manual(&self, state: MarketState, operator: String) -> Result<(), StrategyError> {
        let current_state = *self.current_state.read().await;
        
        if state != current_state {
            let change_record = StateChangeRecord {
                change_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                old_state: current_state,
                new_state: state,
                risk_score: 0.0,
                triggered_indicators: vec!["manual_override".to_string()],
                persistence_met: false,
                consensus_met: false,
                manual_override: true,
            };

            {
                let mut current = self.current_state.write().await;
                let mut start_time = self.current_state_start.write().await;
                let mut history = self.state_change_history.write().await;
                
                *current = state;
                *start_time = Utc::now();
                history.push(change_record);
            }

            tracing::warn!(
                old_state = ?current_state,
                new_state = ?state,
                operator = %operator,
                "Market state manually overridden"
            );
        }

        Ok(())
    }

    /// 获取当前市场状态
    pub async fn get_current_state(&self) -> MarketState {
        *self.current_state.read().await
    }

    /// 获取状态变更历史
    pub async fn get_state_change_history(&self) -> Vec<StateChangeRecord> {
        self.state_change_history.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: MarketStateConfig) -> Result<(), StrategyError> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        tracing::info!("Market state judge configuration updated");
        Ok(())
    }

    /// 计算波动率
    fn calculate_volatility(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let relevant_data: Vec<f64> = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.price)
            .collect();

        if relevant_data.len() < 2 {
            return 0.0;
        }

        let mean = relevant_data.iter().sum::<f64>() / relevant_data.len() as f64;
        let variance = relevant_data
            .iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / relevant_data.len() as f64;
        
        variance.sqrt() / mean // 相对波动率
    }

    /// 计算成交量比率
    fn calculate_volume_ratio(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let recent_volume: f64 = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.volume)
            .sum();

        let historical_cutoff = cutoff_time - duration;
        let historical_volume: f64 = history
            .iter()
            .filter(|point| point.timestamp > historical_cutoff && point.timestamp <= cutoff_time)
            .map(|point| point.volume)
            .sum();

        if historical_volume > 0.0 {
            recent_volume / historical_volume
        } else {
            1.0
        }
    }

    /// 计算最大价格变化
    fn calculate_max_price_change(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let relevant_data: Vec<f64> = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.price)
            .collect();

        if relevant_data.len() < 2 {
            return 0.0;
        }

        let min_price = relevant_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_price = relevant_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        (max_price - min_price) / min_price // 相对变化
    }

    /// 计算平均API延迟
    fn calculate_average_api_latency(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let relevant_latencies: Vec<f64> = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.api_latency)
            .collect();

        if relevant_latencies.is_empty() {
            100.0 // 默认延迟
        } else {
            relevant_latencies.iter().sum::<f64>() / relevant_latencies.len() as f64
        }
    }

    /// 计算风险得分
    fn calculate_risk_score(&self, indicators: &MarketIndicators, config: &MarketStateConfig) -> f64 {
        let weights = &config.weights;
        
        // 波动率得分
        let volatility_score = {
            let max_volatility = indicators.volatility_1h.max(indicators.volatility_4h).max(indicators.volatility_24h);
            if max_volatility > config.volatility_extreme_threshold {
                1.0
            } else if max_volatility > config.volatility_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 流动性得分
        let liquidity_score = {
            if indicators.liquidity_index < config.liquidity_extreme_threshold {
                1.0
            } else if indicators.liquidity_index < config.liquidity_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 成交量得分
        let volume_score = {
            let max_ratio = indicators.volume_ratio_1h.max(indicators.volume_ratio_4h);
            if max_ratio > config.volume_extreme_threshold {
                1.0
            } else if max_ratio > config.volume_spike_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 价格跳变得分
        let price_change_score = {
            let max_change = indicators.max_price_change_1m.max(indicators.max_price_change_5m);
            if max_change > config.price_change_extreme_threshold {
                1.0
            } else if max_change > config.price_change_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // API健康度得分
        let api_health_score = {
            if indicators.api_latency_avg > config.api_latency_extreme_threshold 
                || indicators.api_error_rate > config.api_error_rate_threshold {
                1.0
            } else if indicators.api_latency_avg > config.api_latency_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 外部事件得分
        let external_score = if indicators.external_event_risk > config.external_risk_threshold {
            1.0
        } else {
            indicators.external_event_risk
        };

        // 加权总分
        weights.volatility_weight * volatility_score
            + weights.liquidity_weight * liquidity_score
            + weights.volume_weight * volume_score
            + weights.price_change_weight * price_change_score
            + weights.api_health_weight * api_health_score
            + weights.external_event_weight * external_score
    }

    /// 识别异常指标
    async fn identify_abnormal_indicators(&self, indicators: &MarketIndicators, config: &MarketStateConfig) -> Vec<String> {
        let mut abnormal = Vec::new();

        // 检查各项指标
        if indicators.volatility_1h > config.volatility_normal_threshold 
            || indicators.volatility_4h > config.volatility_normal_threshold 
            || indicators.volatility_24h > config.volatility_normal_threshold {
            abnormal.push("volatility".to_string());
        }

        if indicators.liquidity_index < config.liquidity_normal_threshold {
            abnormal.push("liquidity".to_string());
        }

        if indicators.volume_ratio_1h > config.volume_spike_threshold 
            || indicators.volume_ratio_4h > config.volume_spike_threshold {
            abnormal.push("volume".to_string());
        }

        if indicators.max_price_change_1m > config.price_change_normal_threshold 
            || indicators.max_price_change_5m > config.price_change_normal_threshold {
            abnormal.push("price_change".to_string());
        }

        if indicators.api_latency_avg > config.api_latency_normal_threshold 
            || indicators.api_error_rate > config.api_error_rate_threshold {
            abnormal.push("api_health".to_string());
        }

        if indicators.external_event_risk > config.external_risk_threshold {
            abnormal.push("external_events".to_string());
        }

        abnormal
    }

    /// 检查持续性条件
    async fn check_persistence(&self, target_state: &MarketState, config: &MarketStateConfig) -> bool {
        let current_state = *self.current_state.read().await;
        
        if *target_state == current_state {
            return true; // 状态未变化，总是满足持续性
        }

        let state_start = *self.current_state_start.read().await;
        let duration_in_current_state = Utc::now() - state_start;
        
        duration_in_current_state.num_minutes() >= config.state_change_persistence_minutes
    }

    /// 改变市场状态
    async fn change_market_state(
        &self,
        new_state: MarketState,
        risk_score: f64,
        triggered_indicators: Vec<String>,
        persistence_met: bool,
        consensus_met: bool,
    ) -> Result<(), StrategyError> {
        let old_state = *self.current_state.read().await;
        
        let change_record = StateChangeRecord {
            change_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            old_state,
            new_state,
            risk_score,
            triggered_indicators,
            persistence_met,
            consensus_met,
            manual_override: false,
        };

        {
            let mut current = self.current_state.write().await;
            let mut start_time = self.current_state_start.write().await;
            let mut history = self.state_change_history.write().await;
            
            *current = new_state;
            *start_time = Utc::now();
            history.push(change_record);
        }

        tracing::info!(
            old_state = ?old_state,
            new_state = ?new_state,
            risk_score = %risk_score,
            "Market state changed"
        );

        Ok(())
    }

    /// 更新异常指标记录
    async fn update_abnormal_indicators(&self, current_abnormal: &[String]) {
        let mut abnormal_indicators = self.abnormal_indicators.write().await;
        let now = Utc::now();

        // 添加新的异常指标
        for indicator in current_abnormal {
            abnormal_indicators.insert(indicator.clone(), now);
        }

        // 清理过期的异常指标 (超过10分钟)
        let cutoff_time = now - Duration::minutes(10);
        abnormal_indicators.retain(|_, &mut timestamp| timestamp > cutoff_time);
    }
}


 
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

use crate::strategy::core::StrategyError;

/// 市场状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarketState {
    Normal,   // 常规市场
    Cautious, // 谨慎模式
    Extreme,  // 极端行情
}

/// 市场指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketIndicators {
    /// 历史波动率 (1h, 4h, 24h)
    pub volatility_1h: f64,
    pub volatility_4h: f64,
    pub volatility_24h: f64,
    
    /// 盘口深度与流动性
    pub liquidity_index: f64,
    pub bid_ask_spread: f64,
    pub order_book_depth: f64,
    
    /// 成交量突变指标
    pub volume_ratio_1h: f64,
    pub volume_ratio_4h: f64,
    pub volume_spike_detected: bool,
    
    /// 价格跳变/滑点
    pub max_price_change_1m: f64,
    pub max_price_change_5m: f64,
    pub average_slippage: f64,
    
    /// API健康度
    pub api_latency_avg: f64,
    pub api_error_rate: f64,
    pub api_success_rate: f64,
    
    /// 外部事件标记
    pub external_event_risk: f64,
    pub news_sentiment_score: f64,
    
    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl Default for MarketIndicators {
    fn default() -> Self {
        Self {
            volatility_1h: 0.0,
            volatility_4h: 0.0,
            volatility_24h: 0.0,
            liquidity_index: 1.0,
            bid_ask_spread: 0.001,
            order_book_depth: 1.0,
            volume_ratio_1h: 1.0,
            volume_ratio_4h: 1.0,
            volume_spike_detected: false,
            max_price_change_1m: 0.0,
            max_price_change_5m: 0.0,
            average_slippage: 0.001,
            api_latency_avg: 100.0,
            api_error_rate: 0.0,
            api_success_rate: 1.0,
            external_event_risk: 0.0,
            news_sentiment_score: 0.5,
            timestamp: Utc::now(),
        }
    }
}

/// 市场状态判定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStateConfig {
    /// 波动率阈值 (倍数)
    pub volatility_normal_threshold: f64,
    pub volatility_extreme_threshold: f64,
    
    /// 流动性阈值
    pub liquidity_normal_threshold: f64,
    pub liquidity_extreme_threshold: f64,
    
    /// 成交量突变阈值
    pub volume_spike_threshold: f64,
    pub volume_extreme_threshold: f64,
    
    /// 价格跳变阈值
    pub price_change_normal_threshold: f64,
    pub price_change_extreme_threshold: f64,
    
    /// API健康度阈值
    pub api_latency_normal_threshold: f64,
    pub api_latency_extreme_threshold: f64,
    pub api_error_rate_threshold: f64,
    
    /// 外部事件风险阈值
    pub external_risk_threshold: f64,
    
    /// 状态切换持续性要求 (分钟)
    pub state_change_persistence_minutes: i64,
    
    /// 多指标共振要求 (需要多少个指标同时异常)
    pub indicator_consensus_count: usize,
    
    /// 权重配置
    pub weights: IndicatorWeights,
}

/// 指标权重配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorWeights {
    pub volatility_weight: f64,
    pub liquidity_weight: f64,
    pub volume_weight: f64,
    pub price_change_weight: f64,
    pub api_health_weight: f64,
    pub external_event_weight: f64,
}

impl Default for MarketStateConfig {
    fn default() -> Self {
        Self {
            volatility_normal_threshold: 1.5,
            volatility_extreme_threshold: 2.5,
            liquidity_normal_threshold: 0.8,
            liquidity_extreme_threshold: 0.3,
            volume_spike_threshold: 2.0,
            volume_extreme_threshold: 3.0,
            price_change_normal_threshold: 0.02, // 2%
            price_change_extreme_threshold: 0.05, // 5%
            api_latency_normal_threshold: 500.0, // ms
            api_latency_extreme_threshold: 2000.0, // ms
            api_error_rate_threshold: 0.05, // 5%
            external_risk_threshold: 0.7,
            state_change_persistence_minutes: 5,
            indicator_consensus_count: 3,
            weights: IndicatorWeights {
                volatility_weight: 0.25,
                liquidity_weight: 0.25,
                volume_weight: 0.2,
                price_change_weight: 0.15,
                api_health_weight: 0.1,
                external_event_weight: 0.05,
            },
        }
    }
}

/// 状态变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangeRecord {
    pub change_id: String,
    pub timestamp: DateTime<Utc>,
    pub old_state: MarketState,
    pub new_state: MarketState,
    pub risk_score: f64,
    pub triggered_indicators: Vec<String>,
    pub persistence_met: bool,
    pub consensus_met: bool,
    pub manual_override: bool,
}

/// 历史数据点
#[derive(Debug, Clone)]
struct HistoricalDataPoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
    pub api_latency: f64,
}

/// 市场状态判定器
pub struct MarketStateJudge {
    /// 配置
    config: Arc<RwLock<MarketStateConfig>>,
    
    /// 当前市场状态
    current_state: Arc<RwLock<MarketState>>,
    
    /// 历史指标数据
    historical_indicators: Arc<RwLock<Vec<MarketIndicators>>>,
    
    /// 状态变更历史
    state_change_history: Arc<RwLock<Vec<StateChangeRecord>>>,
    
    /// 历史价格数据 (用于波动率计算)
    price_history: Arc<RwLock<HashMap<String, Vec<HistoricalDataPoint>>>>,
    
    /// 当前状态开始时间 (用于持续性判断)
    current_state_start: Arc<RwLock<DateTime<Utc>>>,
    
    /// 异常指标计数 (用于共振判断)
    abnormal_indicators: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl MarketStateJudge {
    /// 创建新的市场状态判定器
    pub fn new(config: Option<MarketStateConfig>) -> Self {
        Self {
            config: Arc::new(RwLock::new(config.unwrap_or_default())),
            current_state: Arc::new(RwLock::new(MarketState::Normal)),
            historical_indicators: Arc::new(RwLock::new(Vec::new())),
            state_change_history: Arc::new(RwLock::new(Vec::new())),
            price_history: Arc::new(RwLock::new(HashMap::new())),
            current_state_start: Arc::new(RwLock::new(Utc::now())),
            abnormal_indicators: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 更新价格数据
    pub async fn update_price_data(&self, symbol: &str, price: f64, volume: f64, api_latency: f64) {
        let data_point = HistoricalDataPoint {
            timestamp: Utc::now(),
            price,
            volume,
            api_latency,
        };

        let mut price_history = self.price_history.write().await;
        let symbol_history = price_history.entry(symbol.to_string()).or_insert_with(Vec::new);
        symbol_history.push(data_point);

        // 保留最近24小时的数据
        let cutoff_time = Utc::now() - Duration::hours(24);
        symbol_history.retain(|point| point.timestamp > cutoff_time);
    }

    /// 计算市场指标
    pub async fn calculate_market_indicators(&self, symbol: &str) -> Result<MarketIndicators, StrategyError> {
        let price_history = self.price_history.read().await;
        
        if let Some(symbol_history) = price_history.get(symbol) {
            if symbol_history.is_empty() {
                return Ok(MarketIndicators::default());
            }

            let now = Utc::now();
            
            // 计算波动率
            let volatility_1h = self.calculate_volatility(symbol_history, Duration::hours(1), now);
            let volatility_4h = self.calculate_volatility(symbol_history, Duration::hours(4), now);
            let volatility_24h = self.calculate_volatility(symbol_history, Duration::hours(24), now);
            
            // 计算成交量比率
            let volume_ratio_1h = self.calculate_volume_ratio(symbol_history, Duration::hours(1), now);
            let volume_ratio_4h = self.calculate_volume_ratio(symbol_history, Duration::hours(4), now);
            
            // 计算价格跳变
            let max_price_change_1m = self.calculate_max_price_change(symbol_history, Duration::minutes(1), now);
            let max_price_change_5m = self.calculate_max_price_change(symbol_history, Duration::minutes(5), now);
            
            // 计算API健康度
            let api_latency_avg = self.calculate_average_api_latency(symbol_history, Duration::hours(1), now);
            
            // 检测成交量突变
            let volume_spike_detected = volume_ratio_1h > 2.0 || volume_ratio_4h > 1.8;
            
            Ok(MarketIndicators {
                volatility_1h,
                volatility_4h,
                volatility_24h,
                liquidity_index: 1.0, // 需要从orderbook数据计算
                bid_ask_spread: 0.001, // 需要从orderbook数据计算
                order_book_depth: 1.0, // 需要从orderbook数据计算
                volume_ratio_1h,
                volume_ratio_4h,
                volume_spike_detected,
                max_price_change_1m,
                max_price_change_5m,
                average_slippage: 0.001, // 需要从交易数据计算
                api_latency_avg,
                api_error_rate: 0.0, // 需要从API监控数据计算
                api_success_rate: 1.0, // 需要从API监控数据计算
                external_event_risk: 0.0, // 需要外部数据源
                news_sentiment_score: 0.5, // 需要新闻情感分析
                timestamp: now,
            })
        } else {
            Ok(MarketIndicators::default())
        }
    }

    /// 判定市场状态
    pub async fn judge_market_state(&self, indicators: &MarketIndicators) -> Result<MarketState, StrategyError> {
        let config = self.config.read().await;
        
        // 计算风险得分
        let risk_score = self.calculate_risk_score(indicators, &config);
        
        // 确定目标状态
        let target_state = if risk_score < 0.3 {
            MarketState::Normal
        } else if risk_score < 0.7 {
            MarketState::Cautious
        } else {
            MarketState::Extreme
        };

        // 检查指标异常情况
        let abnormal_indicators = self.identify_abnormal_indicators(indicators, &config).await;
        
        // 检查共振条件
        let consensus_met = abnormal_indicators.len() >= config.indicator_consensus_count;
        
        // 检查持续性条件
        let persistence_met = self.check_persistence(&target_state, &config).await;
        
        let current_state = *self.current_state.read().await;
        
        // 决定是否切换状态
        let should_change_state = target_state != current_state && (consensus_met || persistence_met);
        
        if should_change_state {
            self.change_market_state(target_state, risk_score, abnormal_indicators.clone(), persistence_met, consensus_met).await?;
        }

        // 更新异常指标记录
        self.update_abnormal_indicators(&abnormal_indicators).await;
        
        // 记录历史指标
        {
            let mut historical = self.historical_indicators.write().await;
            historical.push(indicators.clone());
            
            // 保留最近1000条记录
            if historical.len() > 1000 {
                historical.remove(0);
            }
        }

        Ok(*self.current_state.read().await)
    }

    /// 手动设置市场状态
    pub async fn set_market_state_manual(&self, state: MarketState, operator: String) -> Result<(), StrategyError> {
        let current_state = *self.current_state.read().await;
        
        if state != current_state {
            let change_record = StateChangeRecord {
                change_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                old_state: current_state,
                new_state: state,
                risk_score: 0.0,
                triggered_indicators: vec!["manual_override".to_string()],
                persistence_met: false,
                consensus_met: false,
                manual_override: true,
            };

            {
                let mut current = self.current_state.write().await;
                let mut start_time = self.current_state_start.write().await;
                let mut history = self.state_change_history.write().await;
                
                *current = state;
                *start_time = Utc::now();
                history.push(change_record);
            }

            tracing::warn!(
                old_state = ?current_state,
                new_state = ?state,
                operator = %operator,
                "Market state manually overridden"
            );
        }

        Ok(())
    }

    /// 获取当前市场状态
    pub async fn get_current_state(&self) -> MarketState {
        *self.current_state.read().await
    }

    /// 获取状态变更历史
    pub async fn get_state_change_history(&self) -> Vec<StateChangeRecord> {
        self.state_change_history.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, new_config: MarketStateConfig) -> Result<(), StrategyError> {
        let mut config = self.config.write().await;
        *config = new_config;
        
        tracing::info!("Market state judge configuration updated");
        Ok(())
    }

    /// 计算波动率
    fn calculate_volatility(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let relevant_data: Vec<f64> = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.price)
            .collect();

        if relevant_data.len() < 2 {
            return 0.0;
        }

        let mean = relevant_data.iter().sum::<f64>() / relevant_data.len() as f64;
        let variance = relevant_data
            .iter()
            .map(|price| (price - mean).powi(2))
            .sum::<f64>() / relevant_data.len() as f64;
        
        variance.sqrt() / mean // 相对波动率
    }

    /// 计算成交量比率
    fn calculate_volume_ratio(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let recent_volume: f64 = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.volume)
            .sum();

        let historical_cutoff = cutoff_time - duration;
        let historical_volume: f64 = history
            .iter()
            .filter(|point| point.timestamp > historical_cutoff && point.timestamp <= cutoff_time)
            .map(|point| point.volume)
            .sum();

        if historical_volume > 0.0 {
            recent_volume / historical_volume
        } else {
            1.0
        }
    }

    /// 计算最大价格变化
    fn calculate_max_price_change(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let relevant_data: Vec<f64> = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.price)
            .collect();

        if relevant_data.len() < 2 {
            return 0.0;
        }

        let min_price = relevant_data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_price = relevant_data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        (max_price - min_price) / min_price // 相对变化
    }

    /// 计算平均API延迟
    fn calculate_average_api_latency(&self, history: &[HistoricalDataPoint], duration: Duration, now: DateTime<Utc>) -> f64 {
        let cutoff_time = now - duration;
        let relevant_latencies: Vec<f64> = history
            .iter()
            .filter(|point| point.timestamp > cutoff_time)
            .map(|point| point.api_latency)
            .collect();

        if relevant_latencies.is_empty() {
            100.0 // 默认延迟
        } else {
            relevant_latencies.iter().sum::<f64>() / relevant_latencies.len() as f64
        }
    }

    /// 计算风险得分
    fn calculate_risk_score(&self, indicators: &MarketIndicators, config: &MarketStateConfig) -> f64 {
        let weights = &config.weights;
        
        // 波动率得分
        let volatility_score = {
            let max_volatility = indicators.volatility_1h.max(indicators.volatility_4h).max(indicators.volatility_24h);
            if max_volatility > config.volatility_extreme_threshold {
                1.0
            } else if max_volatility > config.volatility_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 流动性得分
        let liquidity_score = {
            if indicators.liquidity_index < config.liquidity_extreme_threshold {
                1.0
            } else if indicators.liquidity_index < config.liquidity_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 成交量得分
        let volume_score = {
            let max_ratio = indicators.volume_ratio_1h.max(indicators.volume_ratio_4h);
            if max_ratio > config.volume_extreme_threshold {
                1.0
            } else if max_ratio > config.volume_spike_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 价格跳变得分
        let price_change_score = {
            let max_change = indicators.max_price_change_1m.max(indicators.max_price_change_5m);
            if max_change > config.price_change_extreme_threshold {
                1.0
            } else if max_change > config.price_change_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // API健康度得分
        let api_health_score = {
            if indicators.api_latency_avg > config.api_latency_extreme_threshold 
                || indicators.api_error_rate > config.api_error_rate_threshold {
                1.0
            } else if indicators.api_latency_avg > config.api_latency_normal_threshold {
                0.5
            } else {
                0.0
            }
        };

        // 外部事件得分
        let external_score = if indicators.external_event_risk > config.external_risk_threshold {
            1.0
        } else {
            indicators.external_event_risk
        };

        // 加权总分
        weights.volatility_weight * volatility_score
            + weights.liquidity_weight * liquidity_score
            + weights.volume_weight * volume_score
            + weights.price_change_weight * price_change_score
            + weights.api_health_weight * api_health_score
            + weights.external_event_weight * external_score
    }

    /// 识别异常指标
    async fn identify_abnormal_indicators(&self, indicators: &MarketIndicators, config: &MarketStateConfig) -> Vec<String> {
        let mut abnormal = Vec::new();

        // 检查各项指标
        if indicators.volatility_1h > config.volatility_normal_threshold 
            || indicators.volatility_4h > config.volatility_normal_threshold 
            || indicators.volatility_24h > config.volatility_normal_threshold {
            abnormal.push("volatility".to_string());
        }

        if indicators.liquidity_index < config.liquidity_normal_threshold {
            abnormal.push("liquidity".to_string());
        }

        if indicators.volume_ratio_1h > config.volume_spike_threshold 
            || indicators.volume_ratio_4h > config.volume_spike_threshold {
            abnormal.push("volume".to_string());
        }

        if indicators.max_price_change_1m > config.price_change_normal_threshold 
            || indicators.max_price_change_5m > config.price_change_normal_threshold {
            abnormal.push("price_change".to_string());
        }

        if indicators.api_latency_avg > config.api_latency_normal_threshold 
            || indicators.api_error_rate > config.api_error_rate_threshold {
            abnormal.push("api_health".to_string());
        }

        if indicators.external_event_risk > config.external_risk_threshold {
            abnormal.push("external_events".to_string());
        }

        abnormal
    }

    /// 检查持续性条件
    async fn check_persistence(&self, target_state: &MarketState, config: &MarketStateConfig) -> bool {
        let current_state = *self.current_state.read().await;
        
        if *target_state == current_state {
            return true; // 状态未变化，总是满足持续性
        }

        let state_start = *self.current_state_start.read().await;
        let duration_in_current_state = Utc::now() - state_start;
        
        duration_in_current_state.num_minutes() >= config.state_change_persistence_minutes
    }

    /// 改变市场状态
    async fn change_market_state(
        &self,
        new_state: MarketState,
        risk_score: f64,
        triggered_indicators: Vec<String>,
        persistence_met: bool,
        consensus_met: bool,
    ) -> Result<(), StrategyError> {
        let old_state = *self.current_state.read().await;
        
        let change_record = StateChangeRecord {
            change_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            old_state,
            new_state,
            risk_score,
            triggered_indicators,
            persistence_met,
            consensus_met,
            manual_override: false,
        };

        {
            let mut current = self.current_state.write().await;
            let mut start_time = self.current_state_start.write().await;
            let mut history = self.state_change_history.write().await;
            
            *current = new_state;
            *start_time = Utc::now();
            history.push(change_record);
        }

        tracing::info!(
            old_state = ?old_state,
            new_state = ?new_state,
            risk_score = %risk_score,
            "Market state changed"
        );

        Ok(())
    }

    /// 更新异常指标记录
    async fn update_abnormal_indicators(&self, current_abnormal: &[String]) {
        let mut abnormal_indicators = self.abnormal_indicators.write().await;
        let now = Utc::now();

        // 添加新的异常指标
        for indicator in current_abnormal {
            abnormal_indicators.insert(indicator.clone(), now);
        }

        // 清理过期的异常指标 (超过10分钟)
        let cutoff_time = now - Duration::minutes(10);
        abnormal_indicators.retain(|_, &mut timestamp| timestamp > cutoff_time);
    }
}


 




