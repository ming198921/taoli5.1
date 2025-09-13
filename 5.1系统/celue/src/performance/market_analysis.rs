//! 市场状态分析模块
//! 
//! 多维度市场状态判断和动态阈值调整，目标延迟 ≤ 3微秒

use crate::performance::simd_fixed_point::{FixedPrice, FixedQuantity};
use crate::performance::lockfree_structures::{MarketState, LockFreeConfigCache, ConfigUpdates};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// 市场分析指标
#[derive(Debug, Clone)]
pub struct MarketMetrics {
    pub volatility_1h: f64,
    pub volatility_4h: f64, 
    pub volatility_24h: f64,
    pub volume_ratio: f64,
    pub liquidity_index: f64,
    pub api_latency_ms: f64,
    pub spread_average: f64,
    pub price_deviation: f64,
    pub timestamp_ns: u64,
}

/// 价格数据点
#[derive(Debug, Clone, Copy)]
pub struct PricePoint {
    pub price: FixedPrice,
    pub volume: FixedQuantity,
    pub timestamp_ns: u64,
}

/// 市场数据滑动窗口
#[derive(Debug)]
pub struct SlidingWindow {
    data: VecDeque<PricePoint>,
    capacity: usize,
    sum_price: f64,
    sum_volume: f64,
}

impl SlidingWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
            sum_price: 0.0,
            sum_volume: 0.0,
        }
    }
    
    pub fn add_point(&mut self, point: PricePoint) {
        if self.data.len() >= self.capacity {
            if let Some(old_point) = self.data.pop_front() {
                self.sum_price -= old_point.price.to_f64();
                self.sum_volume -= old_point.volume.to_f64();
            }
        }
        
        self.sum_price += point.price.to_f64();
        self.sum_volume += point.volume.to_f64();
        self.data.push_back(point);
    }
    
    pub fn get_average_price(&self) -> f64 {
        if self.data.is_empty() {
            0.0
        } else {
            self.sum_price / self.data.len() as f64
        }
    }
    
    pub fn get_volatility(&self) -> f64 {
        if self.data.len() < 2 {
            return 0.0;
        }
        
        let avg = self.get_average_price();
        let variance: f64 = self.data
            .iter()
            .map(|p| {
                let diff = p.price.to_f64() - avg;
                diff * diff
            })
            .sum::<f64>() / self.data.len() as f64;
        
        variance.sqrt()
    }
    
    pub fn get_volume_ratio(&self) -> f64 {
        if self.data.len() < 2 {
            return 1.0;
        }
        
        let recent_volume: f64 = self.data
            .iter()
            .rev()
            .take(self.data.len() / 4)
            .map(|p| p.volume.to_f64())
            .sum();
        
        let avg_volume = self.sum_volume / self.data.len() as f64;
        
        if avg_volume > 0.0 {
            recent_volume / (avg_volume * 0.25) // 比较最近25%的数据与平均值
        } else {
            1.0
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// 市场状态分析器
pub struct MarketAnalyzer {
    /// 1小时价格窗口 (3600个数据点，每秒一个)
    window_1h: Arc<RwLock<SlidingWindow>>,
    /// 4小时价格窗口 (900个数据点，每16秒一个)
    window_4h: Arc<RwLock<SlidingWindow>>,
    /// 24小时价格窗口 (360个数据点，每4分钟一个)
    window_24h: Arc<RwLock<SlidingWindow>>,
    
    /// API延迟监控
    api_latency_samples: Arc<RwLock<VecDeque<f64>>>,
    
    /// 流动性指标缓存
    liquidity_cache: Arc<RwLock<HashMap<String, f64>>>,
    
    /// 分析统计
    analysis_count: AtomicU64,
    state_changes: AtomicU64,
    
    /// 配置缓存
    config_cache: Arc<LockFreeConfigCache>,
    
    /// 当前市场状态
    current_state: Arc<RwLock<MarketState>>,
}

impl MarketAnalyzer {
    pub fn new(config_cache: Arc<LockFreeConfigCache>) -> Self {
        Self {
            window_1h: Arc::new(RwLock::new(SlidingWindow::new(3600))),
            window_4h: Arc::new(RwLock::new(SlidingWindow::new(900))),
            window_24h: Arc::new(RwLock::new(SlidingWindow::new(360))),
            api_latency_samples: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            liquidity_cache: Arc::new(RwLock::new(HashMap::new())),
            analysis_count: AtomicU64::new(0),
            state_changes: AtomicU64::new(0),
            config_cache,
            current_state: Arc::new(RwLock::new(MarketState::Normal)),
        }
    }
    
    /// 添加价格数据点
    pub fn add_price_data(&self, symbol: &str, price: FixedPrice, volume: FixedQuantity) {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        let point = PricePoint {
            price,
            volume,
            timestamp_ns,
        };
        
        // 添加到1小时窗口 (每次都添加)
        if let Ok(mut window) = self.window_1h.write() {
            window.add_point(point);
        }
        
        // 添加到4小时窗口 (每16秒采样一次)
        if timestamp_ns % (16 * 1_000_000_000) < 1_000_000_000 {
            if let Ok(mut window) = self.window_4h.write() {
                window.add_point(point);
            }
        }
        
        // 添加到24小时窗口 (每4分钟采样一次)
        if timestamp_ns % (240 * 1_000_000_000) < 1_000_000_000 {
            if let Ok(mut window) = self.window_24h.write() {
                window.add_point(point);
            }
        }
        
        // 更新流动性缓存
        self.update_liquidity_index(symbol, volume);
    }
    
    /// 记录API延迟
    pub fn record_api_latency(&self, latency_ms: f64) {
        if let Ok(mut samples) = self.api_latency_samples.write() {
            if samples.len() >= 100 {
                samples.pop_front();
            }
            samples.push_back(latency_ms);
        }
    }
    
    /// 更新流动性指标
    fn update_liquidity_index(&self, symbol: &str, volume: FixedQuantity) {
        if let Ok(mut cache) = self.liquidity_cache.write() {
            let current = cache.get(symbol).copied().unwrap_or(0.0);
            let new_volume = volume.to_f64();
            // 使用指数移动平均
            let liquidity = current * 0.9 + new_volume * 0.1;
            cache.insert(symbol.to_string(), liquidity);
        }
    }
    
    /// 计算市场指标
    pub fn calculate_metrics(&self) -> MarketMetrics {
        self.analysis_count.fetch_add(1, Ordering::Relaxed);
        
        let volatility_1h = if let Ok(window) = self.window_1h.read() {
            window.get_volatility()
        } else {
            0.0
        };
        
        let volatility_4h = if let Ok(window) = self.window_4h.read() {
            window.get_volatility()
        } else {
            0.0
        };
        
        let volatility_24h = if let Ok(window) = self.window_24h.read() {
            window.get_volatility()
        } else {
            0.0
        };
        
        let volume_ratio = if let Ok(window) = self.window_1h.read() {
            window.get_volume_ratio()
        } else {
            1.0
        };
        
        let liquidity_index = if let Ok(cache) = self.liquidity_cache.read() {
            cache.values().sum::<f64>() / cache.len().max(1) as f64
        } else {
            0.0
        };
        
        let api_latency_ms = if let Ok(samples) = self.api_latency_samples.read() {
            if samples.is_empty() {
                0.0
            } else {
                samples.iter().sum::<f64>() / samples.len() as f64
            }
        } else {
            0.0
        };
        
        // 计算价差平均值
        let spread_average = self.calculate_spread_average();
        
        // 计算价格偏差
        let price_deviation = self.calculate_price_deviation();
        
        MarketMetrics {
            volatility_1h,
            volatility_4h,
            volatility_24h,
            volume_ratio,
            liquidity_index,
            api_latency_ms,
            spread_average,
            price_deviation,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }
    }
    
    /// 判断市场状态
    pub fn analyze_market_state(&self) -> MarketState {
        let metrics = self.calculate_metrics();
        let new_state = self.determine_market_state(&metrics);
        
        // 检查状态是否发生变化
        let state_changed = if let Ok(mut current) = self.current_state.write() {
            if *current != new_state {
                *current = new_state;
                self.state_changes.fetch_add(1, Ordering::Relaxed);
                true
            } else {
                false
            }
        } else {
            false
        };
        
        // 如果状态发生变化，更新配置
        if state_changed {
            self.update_profit_thresholds(new_state);
            
            tracing::info!(
                "📊 市场状态变更为 {:?} - 波动率1h: {:.4}, 4h: {:.4}, 24h: {:.4}, 流动性: {:.2}, API延迟: {:.1}ms",
                new_state,
                metrics.volatility_1h,
                metrics.volatility_4h,
                metrics.volatility_24h,
                metrics.liquidity_index,
                metrics.api_latency_ms
            );
        }
        
        new_state
    }
    
    /// 确定市场状态
    fn determine_market_state(&self, metrics: &MarketMetrics) -> MarketState {
        let mut risk_score = 0.0;
        
        // 波动率评分 (0-40分)
        let volatility_avg = (metrics.volatility_1h + metrics.volatility_4h + metrics.volatility_24h) / 3.0;
        if volatility_avg > 0.05 { // 5%以上波动
            risk_score += 20.0;
        } else if volatility_avg > 0.02 { // 2-5%波动
            risk_score += 10.0;
        }
        
        // 流动性评分 (0-25分)
        if metrics.liquidity_index < 100.0 { // 低流动性
            risk_score += 15.0;
        } else if metrics.liquidity_index < 500.0 { // 中等流动性
            risk_score += 8.0;
        }
        
        // API延迟评分 (0-20分)
        if metrics.api_latency_ms > 500.0 { // 延迟超过500ms
            risk_score += 15.0;
        } else if metrics.api_latency_ms > 200.0 { // 延迟200-500ms
            risk_score += 8.0;
        }
        
        // 价差评分 (0-15分)
        if metrics.spread_average > 0.02 { // 价差超过2%
            risk_score += 10.0;
        } else if metrics.spread_average > 0.01 { // 价差1-2%
            risk_score += 5.0;
        }
        
        // 根据风险评分确定状态
        if risk_score >= 50.0 {
            MarketState::Extreme
        } else if risk_score >= 25.0 {
            MarketState::Cautious
        } else {
            MarketState::Normal
        }
    }
    
    /// 更新利润阈值
    fn update_profit_thresholds(&self, state: MarketState) {
        let updates = match state {
            MarketState::Normal => ConfigUpdates {
                min_profit_normal: Some(FixedPrice::from_f64(0.005)), // 0.5%
                min_profit_cautious: Some(FixedPrice::from_f64(0.012)), // 1.2%
                min_profit_extreme: Some(FixedPrice::from_f64(0.025)), // 2.5%
                max_slippage: Some(FixedPrice::from_f64(0.003)), // 0.3%
                ..Default::default()
            },
            MarketState::Cautious => ConfigUpdates {
                min_profit_normal: Some(FixedPrice::from_f64(0.008)), // 0.8%
                min_profit_cautious: Some(FixedPrice::from_f64(0.015)), // 1.5%
                min_profit_extreme: Some(FixedPrice::from_f64(0.030)), // 3.0%
                max_slippage: Some(FixedPrice::from_f64(0.005)), // 0.5%
                ..Default::default()
            },
            MarketState::Extreme => ConfigUpdates {
                min_profit_normal: Some(FixedPrice::from_f64(0.015)), // 1.5%
                min_profit_cautious: Some(FixedPrice::from_f64(0.025)), // 2.5%
                min_profit_extreme: Some(FixedPrice::from_f64(0.050)), // 5.0%
                max_slippage: Some(FixedPrice::from_f64(0.010)), // 1.0%
                ..Default::default()
            },
        };
        
        self.config_cache.batch_update(&updates);
        
        tracing::debug!(
            "⚙️  更新利润阈值 - 状态: {:?}, 正常: {:.1}%, 谨慎: {:.1}%, 极端: {:.1}%",
            state,
            updates.min_profit_normal.unwrap_or(FixedPrice::zero()).to_f64() * 100.0,
            updates.min_profit_cautious.unwrap_or(FixedPrice::zero()).to_f64() * 100.0,
            updates.min_profit_extreme.unwrap_or(FixedPrice::zero()).to_f64() * 100.0
        );
    }
    
    /// 计算平均价差
    fn calculate_spread_average(&self) -> f64 {
        // 这里应该从市场数据中计算实际价差
        // 暂时返回模拟值
        0.005 // 0.5%
    }
    
    /// 计算价格偏差
    fn calculate_price_deviation(&self) -> f64 {
        if let Ok(window) = self.window_1h.read() {
            if window.len() > 10 {
                // 计算最近价格与移动平均的偏差
                let avg = window.get_average_price();
                if let Some(latest) = window.data.back() {
                    let deviation = (latest.price.to_f64() - avg).abs() / avg;
                    return deviation;
                }
            }
        }
        0.0
    }
    
    /// 获取当前市场状态
    pub fn get_current_state(&self) -> MarketState {
        if let Ok(state) = self.current_state.read() {
            *state
        } else {
            MarketState::Normal
        }
    }
    
    /// 获取分析统计信息
    pub fn get_statistics(&self) -> AnalysisStatistics {
        AnalysisStatistics {
            analysis_count: self.analysis_count.load(Ordering::Relaxed),
            state_changes: self.state_changes.load(Ordering::Relaxed),
            data_points_1h: if let Ok(w) = self.window_1h.read() { w.len() } else { 0 },
            data_points_4h: if let Ok(w) = self.window_4h.read() { w.len() } else { 0 },
            data_points_24h: if let Ok(w) = self.window_24h.read() { w.len() } else { 0 },
            liquidity_symbols: if let Ok(l) = self.liquidity_cache.read() { l.len() } else { 0 },
            api_latency_samples: if let Ok(a) = self.api_latency_samples.read() { a.len() } else { 0 },
        }
    }
}

/// 分析统计信息
#[derive(Debug, Clone)]
pub struct AnalysisStatistics {
    pub analysis_count: u64,
    pub state_changes: u64,
    pub data_points_1h: usize,
    pub data_points_4h: usize,
    pub data_points_24h: usize,
    pub liquidity_symbols: usize,
    pub api_latency_samples: usize,
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sliding_window() {
        let mut window = SlidingWindow::new(3);
        
        let point1 = PricePoint {
            price: FixedPrice::from_f64(100.0),
            volume: FixedQuantity::from_f64(10.0),
            timestamp_ns: 1000,
        };
        
        let point2 = PricePoint {
            price: FixedPrice::from_f64(110.0),
            volume: FixedQuantity::from_f64(20.0),
            timestamp_ns: 2000,
        };
        
        window.add_point(point1);
        window.add_point(point2);
        
        assert_eq!(window.len(), 2);
        assert!((window.get_average_price() - 105.0).abs() < 0.01);
    }
    
    #[test]
    fn test_market_analyzer() {
        let config = Arc::new(LockFreeConfigCache::new());
        let analyzer = MarketAnalyzer::new(config);
        
        // 添加一些测试数据
        analyzer.add_price_data("BTCUSDT", FixedPrice::from_f64(50000.0), FixedQuantity::from_f64(1.0));
        analyzer.record_api_latency(100.0);
        
        let metrics = analyzer.calculate_metrics();
        assert!(metrics.api_latency_ms > 0.0);
        
        let state = analyzer.analyze_market_state();
        assert_eq!(state, MarketState::Normal);
    }
    
    #[test]
    fn test_market_state_determination() {
        let config = Arc::new(LockFreeConfigCache::new());
        let analyzer = MarketAnalyzer::new(config);
        
        let metrics = MarketMetrics {
            volatility_1h: 0.1, // 高波动率
            volatility_4h: 0.08,
            volatility_24h: 0.06,
            volume_ratio: 0.5,
            liquidity_index: 50.0, // 低流动性
            api_latency_ms: 600.0, // 高延迟
            spread_average: 0.03, // 高价差
            price_deviation: 0.02,
            timestamp_ns: 123456789,
        };
        
        let state = analyzer.determine_market_state(&metrics);
        assert_eq!(state, MarketState::Extreme);
    }
} 
 
//! 
//! 多维度市场状态判断和动态阈值调整，目标延迟 ≤ 3微秒

use crate::performance::simd_fixed_point::{FixedPrice, FixedQuantity};
use crate::performance::lockfree_structures::{MarketState, LockFreeConfigCache, ConfigUpdates};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// 市场分析指标
#[derive(Debug, Clone)]
pub struct MarketMetrics {
    pub volatility_1h: f64,
    pub volatility_4h: f64, 
    pub volatility_24h: f64,
    pub volume_ratio: f64,
    pub liquidity_index: f64,
    pub api_latency_ms: f64,
    pub spread_average: f64,
    pub price_deviation: f64,
    pub timestamp_ns: u64,
}

/// 价格数据点
#[derive(Debug, Clone, Copy)]
pub struct PricePoint {
    pub price: FixedPrice,
    pub volume: FixedQuantity,
    pub timestamp_ns: u64,
}

/// 市场数据滑动窗口
#[derive(Debug)]
pub struct SlidingWindow {
    data: VecDeque<PricePoint>,
    capacity: usize,
    sum_price: f64,
    sum_volume: f64,
}

impl SlidingWindow {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
            sum_price: 0.0,
            sum_volume: 0.0,
        }
    }
    
    pub fn add_point(&mut self, point: PricePoint) {
        if self.data.len() >= self.capacity {
            if let Some(old_point) = self.data.pop_front() {
                self.sum_price -= old_point.price.to_f64();
                self.sum_volume -= old_point.volume.to_f64();
            }
        }
        
        self.sum_price += point.price.to_f64();
        self.sum_volume += point.volume.to_f64();
        self.data.push_back(point);
    }
    
    pub fn get_average_price(&self) -> f64 {
        if self.data.is_empty() {
            0.0
        } else {
            self.sum_price / self.data.len() as f64
        }
    }
    
    pub fn get_volatility(&self) -> f64 {
        if self.data.len() < 2 {
            return 0.0;
        }
        
        let avg = self.get_average_price();
        let variance: f64 = self.data
            .iter()
            .map(|p| {
                let diff = p.price.to_f64() - avg;
                diff * diff
            })
            .sum::<f64>() / self.data.len() as f64;
        
        variance.sqrt()
    }
    
    pub fn get_volume_ratio(&self) -> f64 {
        if self.data.len() < 2 {
            return 1.0;
        }
        
        let recent_volume: f64 = self.data
            .iter()
            .rev()
            .take(self.data.len() / 4)
            .map(|p| p.volume.to_f64())
            .sum();
        
        let avg_volume = self.sum_volume / self.data.len() as f64;
        
        if avg_volume > 0.0 {
            recent_volume / (avg_volume * 0.25) // 比较最近25%的数据与平均值
        } else {
            1.0
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// 市场状态分析器
pub struct MarketAnalyzer {
    /// 1小时价格窗口 (3600个数据点，每秒一个)
    window_1h: Arc<RwLock<SlidingWindow>>,
    /// 4小时价格窗口 (900个数据点，每16秒一个)
    window_4h: Arc<RwLock<SlidingWindow>>,
    /// 24小时价格窗口 (360个数据点，每4分钟一个)
    window_24h: Arc<RwLock<SlidingWindow>>,
    
    /// API延迟监控
    api_latency_samples: Arc<RwLock<VecDeque<f64>>>,
    
    /// 流动性指标缓存
    liquidity_cache: Arc<RwLock<HashMap<String, f64>>>,
    
    /// 分析统计
    analysis_count: AtomicU64,
    state_changes: AtomicU64,
    
    /// 配置缓存
    config_cache: Arc<LockFreeConfigCache>,
    
    /// 当前市场状态
    current_state: Arc<RwLock<MarketState>>,
}

impl MarketAnalyzer {
    pub fn new(config_cache: Arc<LockFreeConfigCache>) -> Self {
        Self {
            window_1h: Arc::new(RwLock::new(SlidingWindow::new(3600))),
            window_4h: Arc::new(RwLock::new(SlidingWindow::new(900))),
            window_24h: Arc::new(RwLock::new(SlidingWindow::new(360))),
            api_latency_samples: Arc::new(RwLock::new(VecDeque::with_capacity(100))),
            liquidity_cache: Arc::new(RwLock::new(HashMap::new())),
            analysis_count: AtomicU64::new(0),
            state_changes: AtomicU64::new(0),
            config_cache,
            current_state: Arc::new(RwLock::new(MarketState::Normal)),
        }
    }
    
    /// 添加价格数据点
    pub fn add_price_data(&self, symbol: &str, price: FixedPrice, volume: FixedQuantity) {
        let timestamp_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        let point = PricePoint {
            price,
            volume,
            timestamp_ns,
        };
        
        // 添加到1小时窗口 (每次都添加)
        if let Ok(mut window) = self.window_1h.write() {
            window.add_point(point);
        }
        
        // 添加到4小时窗口 (每16秒采样一次)
        if timestamp_ns % (16 * 1_000_000_000) < 1_000_000_000 {
            if let Ok(mut window) = self.window_4h.write() {
                window.add_point(point);
            }
        }
        
        // 添加到24小时窗口 (每4分钟采样一次)
        if timestamp_ns % (240 * 1_000_000_000) < 1_000_000_000 {
            if let Ok(mut window) = self.window_24h.write() {
                window.add_point(point);
            }
        }
        
        // 更新流动性缓存
        self.update_liquidity_index(symbol, volume);
    }
    
    /// 记录API延迟
    pub fn record_api_latency(&self, latency_ms: f64) {
        if let Ok(mut samples) = self.api_latency_samples.write() {
            if samples.len() >= 100 {
                samples.pop_front();
            }
            samples.push_back(latency_ms);
        }
    }
    
    /// 更新流动性指标
    fn update_liquidity_index(&self, symbol: &str, volume: FixedQuantity) {
        if let Ok(mut cache) = self.liquidity_cache.write() {
            let current = cache.get(symbol).copied().unwrap_or(0.0);
            let new_volume = volume.to_f64();
            // 使用指数移动平均
            let liquidity = current * 0.9 + new_volume * 0.1;
            cache.insert(symbol.to_string(), liquidity);
        }
    }
    
    /// 计算市场指标
    pub fn calculate_metrics(&self) -> MarketMetrics {
        self.analysis_count.fetch_add(1, Ordering::Relaxed);
        
        let volatility_1h = if let Ok(window) = self.window_1h.read() {
            window.get_volatility()
        } else {
            0.0
        };
        
        let volatility_4h = if let Ok(window) = self.window_4h.read() {
            window.get_volatility()
        } else {
            0.0
        };
        
        let volatility_24h = if let Ok(window) = self.window_24h.read() {
            window.get_volatility()
        } else {
            0.0
        };
        
        let volume_ratio = if let Ok(window) = self.window_1h.read() {
            window.get_volume_ratio()
        } else {
            1.0
        };
        
        let liquidity_index = if let Ok(cache) = self.liquidity_cache.read() {
            cache.values().sum::<f64>() / cache.len().max(1) as f64
        } else {
            0.0
        };
        
        let api_latency_ms = if let Ok(samples) = self.api_latency_samples.read() {
            if samples.is_empty() {
                0.0
            } else {
                samples.iter().sum::<f64>() / samples.len() as f64
            }
        } else {
            0.0
        };
        
        // 计算价差平均值
        let spread_average = self.calculate_spread_average();
        
        // 计算价格偏差
        let price_deviation = self.calculate_price_deviation();
        
        MarketMetrics {
            volatility_1h,
            volatility_4h,
            volatility_24h,
            volume_ratio,
            liquidity_index,
            api_latency_ms,
            spread_average,
            price_deviation,
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        }
    }
    
    /// 判断市场状态
    pub fn analyze_market_state(&self) -> MarketState {
        let metrics = self.calculate_metrics();
        let new_state = self.determine_market_state(&metrics);
        
        // 检查状态是否发生变化
        let state_changed = if let Ok(mut current) = self.current_state.write() {
            if *current != new_state {
                *current = new_state;
                self.state_changes.fetch_add(1, Ordering::Relaxed);
                true
            } else {
                false
            }
        } else {
            false
        };
        
        // 如果状态发生变化，更新配置
        if state_changed {
            self.update_profit_thresholds(new_state);
            
            tracing::info!(
                "📊 市场状态变更为 {:?} - 波动率1h: {:.4}, 4h: {:.4}, 24h: {:.4}, 流动性: {:.2}, API延迟: {:.1}ms",
                new_state,
                metrics.volatility_1h,
                metrics.volatility_4h,
                metrics.volatility_24h,
                metrics.liquidity_index,
                metrics.api_latency_ms
            );
        }
        
        new_state
    }
    
    /// 确定市场状态
    fn determine_market_state(&self, metrics: &MarketMetrics) -> MarketState {
        let mut risk_score = 0.0;
        
        // 波动率评分 (0-40分)
        let volatility_avg = (metrics.volatility_1h + metrics.volatility_4h + metrics.volatility_24h) / 3.0;
        if volatility_avg > 0.05 { // 5%以上波动
            risk_score += 20.0;
        } else if volatility_avg > 0.02 { // 2-5%波动
            risk_score += 10.0;
        }
        
        // 流动性评分 (0-25分)
        if metrics.liquidity_index < 100.0 { // 低流动性
            risk_score += 15.0;
        } else if metrics.liquidity_index < 500.0 { // 中等流动性
            risk_score += 8.0;
        }
        
        // API延迟评分 (0-20分)
        if metrics.api_latency_ms > 500.0 { // 延迟超过500ms
            risk_score += 15.0;
        } else if metrics.api_latency_ms > 200.0 { // 延迟200-500ms
            risk_score += 8.0;
        }
        
        // 价差评分 (0-15分)
        if metrics.spread_average > 0.02 { // 价差超过2%
            risk_score += 10.0;
        } else if metrics.spread_average > 0.01 { // 价差1-2%
            risk_score += 5.0;
        }
        
        // 根据风险评分确定状态
        if risk_score >= 50.0 {
            MarketState::Extreme
        } else if risk_score >= 25.0 {
            MarketState::Cautious
        } else {
            MarketState::Normal
        }
    }
    
    /// 更新利润阈值
    fn update_profit_thresholds(&self, state: MarketState) {
        let updates = match state {
            MarketState::Normal => ConfigUpdates {
                min_profit_normal: Some(FixedPrice::from_f64(0.005)), // 0.5%
                min_profit_cautious: Some(FixedPrice::from_f64(0.012)), // 1.2%
                min_profit_extreme: Some(FixedPrice::from_f64(0.025)), // 2.5%
                max_slippage: Some(FixedPrice::from_f64(0.003)), // 0.3%
                ..Default::default()
            },
            MarketState::Cautious => ConfigUpdates {
                min_profit_normal: Some(FixedPrice::from_f64(0.008)), // 0.8%
                min_profit_cautious: Some(FixedPrice::from_f64(0.015)), // 1.5%
                min_profit_extreme: Some(FixedPrice::from_f64(0.030)), // 3.0%
                max_slippage: Some(FixedPrice::from_f64(0.005)), // 0.5%
                ..Default::default()
            },
            MarketState::Extreme => ConfigUpdates {
                min_profit_normal: Some(FixedPrice::from_f64(0.015)), // 1.5%
                min_profit_cautious: Some(FixedPrice::from_f64(0.025)), // 2.5%
                min_profit_extreme: Some(FixedPrice::from_f64(0.050)), // 5.0%
                max_slippage: Some(FixedPrice::from_f64(0.010)), // 1.0%
                ..Default::default()
            },
        };
        
        self.config_cache.batch_update(&updates);
        
        tracing::debug!(
            "⚙️  更新利润阈值 - 状态: {:?}, 正常: {:.1}%, 谨慎: {:.1}%, 极端: {:.1}%",
            state,
            updates.min_profit_normal.unwrap_or(FixedPrice::zero()).to_f64() * 100.0,
            updates.min_profit_cautious.unwrap_or(FixedPrice::zero()).to_f64() * 100.0,
            updates.min_profit_extreme.unwrap_or(FixedPrice::zero()).to_f64() * 100.0
        );
    }
    
    /// 计算平均价差
    fn calculate_spread_average(&self) -> f64 {
        // 这里应该从市场数据中计算实际价差
        // 暂时返回模拟值
        0.005 // 0.5%
    }
    
    /// 计算价格偏差
    fn calculate_price_deviation(&self) -> f64 {
        if let Ok(window) = self.window_1h.read() {
            if window.len() > 10 {
                // 计算最近价格与移动平均的偏差
                let avg = window.get_average_price();
                if let Some(latest) = window.data.back() {
                    let deviation = (latest.price.to_f64() - avg).abs() / avg;
                    return deviation;
                }
            }
        }
        0.0
    }
    
    /// 获取当前市场状态
    pub fn get_current_state(&self) -> MarketState {
        if let Ok(state) = self.current_state.read() {
            *state
        } else {
            MarketState::Normal
        }
    }
    
    /// 获取分析统计信息
    pub fn get_statistics(&self) -> AnalysisStatistics {
        AnalysisStatistics {
            analysis_count: self.analysis_count.load(Ordering::Relaxed),
            state_changes: self.state_changes.load(Ordering::Relaxed),
            data_points_1h: if let Ok(w) = self.window_1h.read() { w.len() } else { 0 },
            data_points_4h: if let Ok(w) = self.window_4h.read() { w.len() } else { 0 },
            data_points_24h: if let Ok(w) = self.window_24h.read() { w.len() } else { 0 },
            liquidity_symbols: if let Ok(l) = self.liquidity_cache.read() { l.len() } else { 0 },
            api_latency_samples: if let Ok(a) = self.api_latency_samples.read() { a.len() } else { 0 },
        }
    }
}

/// 分析统计信息
#[derive(Debug, Clone)]
pub struct AnalysisStatistics {
    pub analysis_count: u64,
    pub state_changes: u64,
    pub data_points_1h: usize,
    pub data_points_4h: usize,
    pub data_points_24h: usize,
    pub liquidity_symbols: usize,
    pub api_latency_samples: usize,
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sliding_window() {
        let mut window = SlidingWindow::new(3);
        
        let point1 = PricePoint {
            price: FixedPrice::from_f64(100.0),
            volume: FixedQuantity::from_f64(10.0),
            timestamp_ns: 1000,
        };
        
        let point2 = PricePoint {
            price: FixedPrice::from_f64(110.0),
            volume: FixedQuantity::from_f64(20.0),
            timestamp_ns: 2000,
        };
        
        window.add_point(point1);
        window.add_point(point2);
        
        assert_eq!(window.len(), 2);
        assert!((window.get_average_price() - 105.0).abs() < 0.01);
    }
    
    #[test]
    fn test_market_analyzer() {
        let config = Arc::new(LockFreeConfigCache::new());
        let analyzer = MarketAnalyzer::new(config);
        
        // 添加一些测试数据
        analyzer.add_price_data("BTCUSDT", FixedPrice::from_f64(50000.0), FixedQuantity::from_f64(1.0));
        analyzer.record_api_latency(100.0);
        
        let metrics = analyzer.calculate_metrics();
        assert!(metrics.api_latency_ms > 0.0);
        
        let state = analyzer.analyze_market_state();
        assert_eq!(state, MarketState::Normal);
    }
    
    #[test]
    fn test_market_state_determination() {
        let config = Arc::new(LockFreeConfigCache::new());
        let analyzer = MarketAnalyzer::new(config);
        
        let metrics = MarketMetrics {
            volatility_1h: 0.1, // 高波动率
            volatility_4h: 0.08,
            volatility_24h: 0.06,
            volume_ratio: 0.5,
            liquidity_index: 50.0, // 低流动性
            api_latency_ms: 600.0, // 高延迟
            spread_average: 0.03, // 高价差
            price_deviation: 0.02,
            timestamp_ns: 123456789,
        };
        
        let state = analyzer.determine_market_state(&metrics);
        assert_eq!(state, MarketState::Extreme);
    }
} 
 