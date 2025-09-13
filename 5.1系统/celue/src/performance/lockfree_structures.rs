//! 无锁数据结构
//! 
//! 基于原子操作的无锁并发数据结构，目标延迟 ≤ 1微秒

use crossbeam::queue::ArrayQueue;
use atomic::{Atomic, Ordering};
use crate::performance::simd_fixed_point::{FixedPrice, FixedQuantity};

/// 市场状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MarketState {
    Normal = 0,
    Cautious = 1,
    Extreme = 2,
}

/// 使用统一的ArbitrageOpportunity，避免重复定义
pub use common_types::ArbitrageOpportunity;

/// 策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrategyType {
    InterExchange = 0,
    Triangular = 1,
}

/// 无锁套利机会池
pub struct LockFreeOpportunityPool {
    // 固定大小的无锁队列，避免动态分配
    opportunities: ArrayQueue<ArbitrageOpportunity>,
    // 原子序列号生成器
    sequence: Atomic<u64>,
    // 原子市场状态
    market_state: Atomic<u8>,
    // 原子统计计数器
    total_opportunities: Atomic<u64>,
    inter_exchange_count: Atomic<u64>,
    triangular_count: Atomic<u64>,
    // 原子最大利润记录
    max_profit_raw: Atomic<i64>,
}

impl LockFreeOpportunityPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            opportunities: ArrayQueue::new(capacity),
            sequence: Atomic::new(0),
            market_state: Atomic::new(MarketState::Normal as u8),
            total_opportunities: Atomic::new(0),
            inter_exchange_count: Atomic::new(0),
            triangular_count: Atomic::new(0),
            max_profit_raw: Atomic::new(0),
        }
    }
    
    /// 原子方式推送套利机会
    #[inline(always)]
    pub fn try_push_opportunity(&self, mut opportunity: ArbitrageOpportunity) -> Result<(), ArbitrageOpportunity> {
        // 分配唯一序列号
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        opportunity.id = seq;
        
        // 更新统计计数器
        self.total_opportunities.fetch_add(1, Ordering::Relaxed);
        match opportunity.strategy_type {
            StrategyType::InterExchange => {
                self.inter_exchange_count.fetch_add(1, Ordering::Relaxed);
            }
            StrategyType::Triangular => {
                self.triangular_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        // 更新最大利润记录
        let current_max = self.max_profit_raw.load(Ordering::Relaxed);
        let new_profit = opportunity.profit.raw();
        if new_profit > current_max {
            let _ = self.max_profit_raw.compare_exchange_weak(
                current_max,
                new_profit,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
        
        // 推送到队列
        self.opportunities.push(opportunity)
    }
    
    /// 原子方式弹出最优套利机会
    #[inline(always)]
    pub fn try_pop_best(&self) -> Option<ArbitrageOpportunity> {
        self.opportunities.pop()
    }
    
    /// 获取当前市场状态
    #[inline(always)]
    pub fn get_market_state(&self) -> MarketState {
        let state_val = self.market_state.load(Ordering::Relaxed);
        match state_val {
            0 => MarketState::Normal,
            1 => MarketState::Cautious,
            2 => MarketState::Extreme,
            _ => MarketState::Normal, // 默认回退
        }
    }
    
    /// 原子方式设置市场状态
    #[inline(always)]
    pub fn set_market_state(&self, state: MarketState) {
        self.market_state.store(state as u8, Ordering::Relaxed);
    }
    
    /// 原子方式获取统计信息
    #[inline(always)]
    pub fn get_statistics(&self) -> OpportunityStatistics {
        OpportunityStatistics {
            total_opportunities: self.total_opportunities.load(Ordering::Relaxed),
            inter_exchange_count: self.inter_exchange_count.load(Ordering::Relaxed),
            triangular_count: self.triangular_count.load(Ordering::Relaxed),
            max_profit: FixedPrice::from_raw(self.max_profit_raw.load(Ordering::Relaxed)),
            queue_size: self.opportunities.len(),
            queue_capacity: self.opportunities.capacity(),
        }
    }
    
    /// 清空队列和重置统计
    pub fn clear(&self) {
        // 清空队列
        while self.opportunities.pop().is_some() {}
        
        // 重置计数器
        self.total_opportunities.store(0, Ordering::Relaxed);
        self.inter_exchange_count.store(0, Ordering::Relaxed);
        self.triangular_count.store(0, Ordering::Relaxed);
        self.max_profit_raw.store(0, Ordering::Relaxed);
    }
}

/// 统计信息结构
#[derive(Debug, Clone)]
pub struct OpportunityStatistics {
    pub total_opportunities: u64,
    pub inter_exchange_count: u64,
    pub triangular_count: u64,
    pub max_profit: FixedPrice,
    pub queue_size: usize,
    pub queue_capacity: usize,
}

/// 无锁配置缓存
pub struct LockFreeConfigCache {
    // 不同市场状态下的最小利润阈值
    min_profit_normal: Atomic<i64>,
    min_profit_cautious: Atomic<i64>,
    min_profit_extreme: Atomic<i64>,
    
    // 流动性检查阈值
    min_order_book_depth: Atomic<i64>,
    
    // 滑点限制
    max_slippage: Atomic<i64>,
    
    // 配置版本号
    config_version: Atomic<u64>,
}

impl LockFreeConfigCache {
    pub fn new() -> Self {
        Self {
            min_profit_normal: Atomic::new(FixedPrice::from_f64(0.005).raw()),     // 0.5%
            min_profit_cautious: Atomic::new(FixedPrice::from_f64(0.012).raw()),   // 1.2%
            min_profit_extreme: Atomic::new(FixedPrice::from_f64(0.020).raw()),    // 2.0%
            min_order_book_depth: Atomic::new(FixedQuantity::from_f64(1000.0).raw()),
            max_slippage: Atomic::new(FixedPrice::from_f64(0.003).raw()),          // 0.3%
            config_version: Atomic::new(1),
        }
    }
    
    /// 获取指定市场状态的最小利润阈值
    #[inline(always)]
    pub fn get_min_profit(&self, market_state: MarketState) -> FixedPrice {
        let raw_value = match market_state {
            MarketState::Normal => self.min_profit_normal.load(Ordering::Relaxed),
            MarketState::Cautious => self.min_profit_cautious.load(Ordering::Relaxed),
            MarketState::Extreme => self.min_profit_extreme.load(Ordering::Relaxed),
        };
        FixedPrice::from_raw(raw_value)
    }
    
    /// 原子方式更新最小利润阈值
    #[inline(always)]
    pub fn update_min_profit(&self, market_state: MarketState, new_profit: FixedPrice) {
        let atomic_ref = match market_state {
            MarketState::Normal => &self.min_profit_normal,
            MarketState::Cautious => &self.min_profit_cautious,
            MarketState::Extreme => &self.min_profit_extreme,
        };
        
        atomic_ref.store(new_profit.raw(), Ordering::Relaxed);
        self.config_version.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取最小订单簿深度
    #[inline(always)]
    pub fn get_min_order_book_depth(&self) -> FixedQuantity {
        FixedQuantity::from_raw(self.min_order_book_depth.load(Ordering::Relaxed))
    }
    
    /// 获取最大滑点
    #[inline(always)]
    pub fn get_max_slippage(&self) -> FixedPrice {
        FixedPrice::from_raw(self.max_slippage.load(Ordering::Relaxed))
    }
    
    /// 获取配置版本号
    #[inline(always)]
    pub fn get_version(&self) -> u64 {
        self.config_version.load(Ordering::Relaxed)
    }
    
    /// 批量更新配置
    pub fn batch_update(&self, updates: &ConfigUpdates) {
        if let Some(normal) = updates.min_profit_normal {
            self.min_profit_normal.store(normal.raw(), Ordering::Relaxed);
        }
        if let Some(cautious) = updates.min_profit_cautious {
            self.min_profit_cautious.store(cautious.raw(), Ordering::Relaxed);
        }
        if let Some(extreme) = updates.min_profit_extreme {
            self.min_profit_extreme.store(extreme.raw(), Ordering::Relaxed);
        }
        if let Some(depth) = updates.min_order_book_depth {
            self.min_order_book_depth.store(depth.raw(), Ordering::Relaxed);
        }
        if let Some(slippage) = updates.max_slippage {
            self.max_slippage.store(slippage.raw(), Ordering::Relaxed);
        }
        
        self.config_version.fetch_add(1, Ordering::Relaxed);
    }
}

/// 配置更新结构
#[derive(Debug, Default)]
pub struct ConfigUpdates {
    pub min_profit_normal: Option<FixedPrice>,
    pub min_profit_cautious: Option<FixedPrice>,
    pub min_profit_extreme: Option<FixedPrice>,
    pub min_order_book_depth: Option<FixedQuantity>,
    pub max_slippage: Option<FixedPrice>,
}

/// 无锁性能计数器
pub struct LockFreePerformanceCounters {
    // 处理延迟统计
    total_detection_time_ns: Atomic<u64>,
    detection_count: Atomic<u64>,
    
    // SIMD操作计数
    simd_operations: Atomic<u64>,
    
    // 缓存命中统计
    cache_hits: Atomic<u64>,
    cache_misses: Atomic<u64>,
    
    // 错误计数
    error_count: Atomic<u64>,
    
    // 最后更新时间戳
    last_update_ns: Atomic<u64>,
}

impl LockFreePerformanceCounters {
    pub fn new() -> Self {
        Self {
            total_detection_time_ns: Atomic::new(0),
            detection_count: Atomic::new(0),
            simd_operations: Atomic::new(0),
            cache_hits: Atomic::new(0),
            cache_misses: Atomic::new(0),
            error_count: Atomic::new(0),
            last_update_ns: Atomic::new(0),
        }
    }
    
    /// 记录检测延迟
    #[inline(always)]
    pub fn record_detection_latency(&self, latency_ns: u64) {
        self.total_detection_time_ns.fetch_add(latency_ns, Ordering::Relaxed);
        self.detection_count.fetch_add(1, Ordering::Relaxed);
        self.last_update_ns.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            Ordering::Relaxed,
        );
    }
    
    /// 增加SIMD操作计数
    #[inline(always)]
    pub fn increment_simd_operations(&self, count: u64) {
        self.simd_operations.fetch_add(count, Ordering::Relaxed);
    }
    
    /// 记录缓存命中
    #[inline(always)]
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录缓存未命中
    #[inline(always)]
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 增加错误计数
    #[inline(always)]
    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取平均检测延迟
    #[inline(always)]
    pub fn get_average_detection_latency_ns(&self) -> u64 {
        let total_time = self.total_detection_time_ns.load(Ordering::Relaxed);
        let count = self.detection_count.load(Ordering::Relaxed);
        if count > 0 {
            total_time / count
        } else {
            0
        }
    }
    
    /// 获取缓存命中率
    #[inline(always)]
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let misses = self.cache_misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
    
    /// 获取完整性能指标
    pub fn get_metrics(&self) -> crate::performance::PerformanceMetrics {
        crate::performance::PerformanceMetrics {
            detection_latency_ns: self.get_average_detection_latency_ns(),
            simd_operations_count: self.simd_operations.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            opportunities_processed: self.detection_count.load(Ordering::Relaxed),
        }
    }
    
    /// 重置所有计数器
    pub fn reset(&self) {
        self.total_detection_time_ns.store(0, Ordering::Relaxed);
        self.detection_count.store(0, Ordering::Relaxed);
        self.simd_operations.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lockfree_opportunity_pool() {
        let pool = LockFreeOpportunityPool::new(100);
        
        let opportunity = ArbitrageOpportunity {
            id: 0, // 会被自动分配
            strategy_type: StrategyType::InterExchange,
            symbol: "BTCUSDT".to_string(),
            exchange_buy: "binance".to_string(),
            exchange_sell: "okx".to_string(),
            buy_price: FixedPrice::from_f64(50000.0),
            sell_price: FixedPrice::from_f64(50100.0),
            profit: FixedPrice::from_f64(100.0),
            volume: FixedQuantity::from_f64(1.0),
            timestamp_ns: 1234567890,
            confidence_score: 0.95,
        };
        
        // 测试推送和弹出
        assert!(pool.try_push_opportunity(opportunity.clone()).is_ok());
        let popped = pool.try_pop_best().unwrap();
        assert_eq!(popped.symbol, "BTCUSDT");
        assert_eq!(popped.id, 0); // 第一个分配的ID
        
        // 测试统计
        let stats = pool.get_statistics();
        assert_eq!(stats.total_opportunities, 1);
        assert_eq!(stats.inter_exchange_count, 1);
        assert_eq!(stats.triangular_count, 0);
    }
    
    #[test]
    fn test_lockfree_config_cache() {
        let cache = LockFreeConfigCache::new();
        
        // 测试默认值
        let normal_profit = cache.get_min_profit(MarketState::Normal);
        assert!((normal_profit.to_f64() - 0.005).abs() < 0.000001);
        
        // 测试更新
        let new_profit = FixedPrice::from_f64(0.008);
        cache.update_min_profit(MarketState::Normal, new_profit);
        
        let updated_profit = cache.get_min_profit(MarketState::Normal);
        assert!((updated_profit.to_f64() - 0.008).abs() < 0.000001);
        
        // 版本号应该增加
        assert!(cache.get_version() > 1);
    }
    
    #[test]
    fn test_performance_counters() {
        let counters = LockFreePerformanceCounters::new();
        
        // 记录一些延迟
        counters.record_detection_latency(1000); // 1μs
        counters.record_detection_latency(2000); // 2μs
        
        let avg_latency = counters.get_average_detection_latency_ns();
        assert_eq!(avg_latency, 1500); // 平均1.5μs
        
        // 测试缓存命中率
        counters.record_cache_hit();
        counters.record_cache_hit();
        counters.record_cache_miss();
        
        let hit_rate = counters.get_cache_hit_rate();
        assert!((hit_rate - 0.6666666666666666).abs() < 0.0001); // 2/3 ≈ 0.667
    }
    
    #[test]
    fn test_market_state_transitions() {
        let pool = LockFreeOpportunityPool::new(10);
        
        // 测试状态转换
        assert_eq!(pool.get_market_state(), MarketState::Normal);
        
        pool.set_market_state(MarketState::Extreme);
        assert_eq!(pool.get_market_state(), MarketState::Extreme);
        
        pool.set_market_state(MarketState::Cautious);
        assert_eq!(pool.get_market_state(), MarketState::Cautious);
    }
} 
//! 
//! 基于原子操作的无锁并发数据结构，目标延迟 ≤ 1微秒

use crossbeam::queue::ArrayQueue;
use atomic::{Atomic, Ordering};
use crate::performance::simd_fixed_point::{FixedPrice, FixedQuantity};

/// 市场状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MarketState {
    Normal = 0,
    Cautious = 1,
    Extreme = 2,
}

/// 使用统一的ArbitrageOpportunity，避免重复定义
pub use common_types::ArbitrageOpportunity;

/// 策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StrategyType {
    InterExchange = 0,
    Triangular = 1,
}

/// 无锁套利机会池
pub struct LockFreeOpportunityPool {
    // 固定大小的无锁队列，避免动态分配
    opportunities: ArrayQueue<ArbitrageOpportunity>,
    // 原子序列号生成器
    sequence: Atomic<u64>,
    // 原子市场状态
    market_state: Atomic<u8>,
    // 原子统计计数器
    total_opportunities: Atomic<u64>,
    inter_exchange_count: Atomic<u64>,
    triangular_count: Atomic<u64>,
    // 原子最大利润记录
    max_profit_raw: Atomic<i64>,
}

impl LockFreeOpportunityPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            opportunities: ArrayQueue::new(capacity),
            sequence: Atomic::new(0),
            market_state: Atomic::new(MarketState::Normal as u8),
            total_opportunities: Atomic::new(0),
            inter_exchange_count: Atomic::new(0),
            triangular_count: Atomic::new(0),
            max_profit_raw: Atomic::new(0),
        }
    }
    
    /// 原子方式推送套利机会
    #[inline(always)]
    pub fn try_push_opportunity(&self, mut opportunity: ArbitrageOpportunity) -> Result<(), ArbitrageOpportunity> {
        // 分配唯一序列号
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        opportunity.id = seq;
        
        // 更新统计计数器
        self.total_opportunities.fetch_add(1, Ordering::Relaxed);
        match opportunity.strategy_type {
            StrategyType::InterExchange => {
                self.inter_exchange_count.fetch_add(1, Ordering::Relaxed);
            }
            StrategyType::Triangular => {
                self.triangular_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        
        // 更新最大利润记录
        let current_max = self.max_profit_raw.load(Ordering::Relaxed);
        let new_profit = opportunity.profit.raw();
        if new_profit > current_max {
            let _ = self.max_profit_raw.compare_exchange_weak(
                current_max,
                new_profit,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
        
        // 推送到队列
        self.opportunities.push(opportunity)
    }
    
    /// 原子方式弹出最优套利机会
    #[inline(always)]
    pub fn try_pop_best(&self) -> Option<ArbitrageOpportunity> {
        self.opportunities.pop()
    }
    
    /// 获取当前市场状态
    #[inline(always)]
    pub fn get_market_state(&self) -> MarketState {
        let state_val = self.market_state.load(Ordering::Relaxed);
        match state_val {
            0 => MarketState::Normal,
            1 => MarketState::Cautious,
            2 => MarketState::Extreme,
            _ => MarketState::Normal, // 默认回退
        }
    }
    
    /// 原子方式设置市场状态
    #[inline(always)]
    pub fn set_market_state(&self, state: MarketState) {
        self.market_state.store(state as u8, Ordering::Relaxed);
    }
    
    /// 原子方式获取统计信息
    #[inline(always)]
    pub fn get_statistics(&self) -> OpportunityStatistics {
        OpportunityStatistics {
            total_opportunities: self.total_opportunities.load(Ordering::Relaxed),
            inter_exchange_count: self.inter_exchange_count.load(Ordering::Relaxed),
            triangular_count: self.triangular_count.load(Ordering::Relaxed),
            max_profit: FixedPrice::from_raw(self.max_profit_raw.load(Ordering::Relaxed)),
            queue_size: self.opportunities.len(),
            queue_capacity: self.opportunities.capacity(),
        }
    }
    
    /// 清空队列和重置统计
    pub fn clear(&self) {
        // 清空队列
        while self.opportunities.pop().is_some() {}
        
        // 重置计数器
        self.total_opportunities.store(0, Ordering::Relaxed);
        self.inter_exchange_count.store(0, Ordering::Relaxed);
        self.triangular_count.store(0, Ordering::Relaxed);
        self.max_profit_raw.store(0, Ordering::Relaxed);
    }
}

/// 统计信息结构
#[derive(Debug, Clone)]
pub struct OpportunityStatistics {
    pub total_opportunities: u64,
    pub inter_exchange_count: u64,
    pub triangular_count: u64,
    pub max_profit: FixedPrice,
    pub queue_size: usize,
    pub queue_capacity: usize,
}

/// 无锁配置缓存
pub struct LockFreeConfigCache {
    // 不同市场状态下的最小利润阈值
    min_profit_normal: Atomic<i64>,
    min_profit_cautious: Atomic<i64>,
    min_profit_extreme: Atomic<i64>,
    
    // 流动性检查阈值
    min_order_book_depth: Atomic<i64>,
    
    // 滑点限制
    max_slippage: Atomic<i64>,
    
    // 配置版本号
    config_version: Atomic<u64>,
}

impl LockFreeConfigCache {
    pub fn new() -> Self {
        Self {
            min_profit_normal: Atomic::new(FixedPrice::from_f64(0.005).raw()),     // 0.5%
            min_profit_cautious: Atomic::new(FixedPrice::from_f64(0.012).raw()),   // 1.2%
            min_profit_extreme: Atomic::new(FixedPrice::from_f64(0.020).raw()),    // 2.0%
            min_order_book_depth: Atomic::new(FixedQuantity::from_f64(1000.0).raw()),
            max_slippage: Atomic::new(FixedPrice::from_f64(0.003).raw()),          // 0.3%
            config_version: Atomic::new(1),
        }
    }
    
    /// 获取指定市场状态的最小利润阈值
    #[inline(always)]
    pub fn get_min_profit(&self, market_state: MarketState) -> FixedPrice {
        let raw_value = match market_state {
            MarketState::Normal => self.min_profit_normal.load(Ordering::Relaxed),
            MarketState::Cautious => self.min_profit_cautious.load(Ordering::Relaxed),
            MarketState::Extreme => self.min_profit_extreme.load(Ordering::Relaxed),
        };
        FixedPrice::from_raw(raw_value)
    }
    
    /// 原子方式更新最小利润阈值
    #[inline(always)]
    pub fn update_min_profit(&self, market_state: MarketState, new_profit: FixedPrice) {
        let atomic_ref = match market_state {
            MarketState::Normal => &self.min_profit_normal,
            MarketState::Cautious => &self.min_profit_cautious,
            MarketState::Extreme => &self.min_profit_extreme,
        };
        
        atomic_ref.store(new_profit.raw(), Ordering::Relaxed);
        self.config_version.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取最小订单簿深度
    #[inline(always)]
    pub fn get_min_order_book_depth(&self) -> FixedQuantity {
        FixedQuantity::from_raw(self.min_order_book_depth.load(Ordering::Relaxed))
    }
    
    /// 获取最大滑点
    #[inline(always)]
    pub fn get_max_slippage(&self) -> FixedPrice {
        FixedPrice::from_raw(self.max_slippage.load(Ordering::Relaxed))
    }
    
    /// 获取配置版本号
    #[inline(always)]
    pub fn get_version(&self) -> u64 {
        self.config_version.load(Ordering::Relaxed)
    }
    
    /// 批量更新配置
    pub fn batch_update(&self, updates: &ConfigUpdates) {
        if let Some(normal) = updates.min_profit_normal {
            self.min_profit_normal.store(normal.raw(), Ordering::Relaxed);
        }
        if let Some(cautious) = updates.min_profit_cautious {
            self.min_profit_cautious.store(cautious.raw(), Ordering::Relaxed);
        }
        if let Some(extreme) = updates.min_profit_extreme {
            self.min_profit_extreme.store(extreme.raw(), Ordering::Relaxed);
        }
        if let Some(depth) = updates.min_order_book_depth {
            self.min_order_book_depth.store(depth.raw(), Ordering::Relaxed);
        }
        if let Some(slippage) = updates.max_slippage {
            self.max_slippage.store(slippage.raw(), Ordering::Relaxed);
        }
        
        self.config_version.fetch_add(1, Ordering::Relaxed);
    }
}

/// 配置更新结构
#[derive(Debug, Default)]
pub struct ConfigUpdates {
    pub min_profit_normal: Option<FixedPrice>,
    pub min_profit_cautious: Option<FixedPrice>,
    pub min_profit_extreme: Option<FixedPrice>,
    pub min_order_book_depth: Option<FixedQuantity>,
    pub max_slippage: Option<FixedPrice>,
}

/// 无锁性能计数器
pub struct LockFreePerformanceCounters {
    // 处理延迟统计
    total_detection_time_ns: Atomic<u64>,
    detection_count: Atomic<u64>,
    
    // SIMD操作计数
    simd_operations: Atomic<u64>,
    
    // 缓存命中统计
    cache_hits: Atomic<u64>,
    cache_misses: Atomic<u64>,
    
    // 错误计数
    error_count: Atomic<u64>,
    
    // 最后更新时间戳
    last_update_ns: Atomic<u64>,
}

impl LockFreePerformanceCounters {
    pub fn new() -> Self {
        Self {
            total_detection_time_ns: Atomic::new(0),
            detection_count: Atomic::new(0),
            simd_operations: Atomic::new(0),
            cache_hits: Atomic::new(0),
            cache_misses: Atomic::new(0),
            error_count: Atomic::new(0),
            last_update_ns: Atomic::new(0),
        }
    }
    
    /// 记录检测延迟
    #[inline(always)]
    pub fn record_detection_latency(&self, latency_ns: u64) {
        self.total_detection_time_ns.fetch_add(latency_ns, Ordering::Relaxed);
        self.detection_count.fetch_add(1, Ordering::Relaxed);
        self.last_update_ns.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
            Ordering::Relaxed,
        );
    }
    
    /// 增加SIMD操作计数
    #[inline(always)]
    pub fn increment_simd_operations(&self, count: u64) {
        self.simd_operations.fetch_add(count, Ordering::Relaxed);
    }
    
    /// 记录缓存命中
    #[inline(always)]
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录缓存未命中
    #[inline(always)]
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 增加错误计数
    #[inline(always)]
    pub fn increment_error_count(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 获取平均检测延迟
    #[inline(always)]
    pub fn get_average_detection_latency_ns(&self) -> u64 {
        let total_time = self.total_detection_time_ns.load(Ordering::Relaxed);
        let count = self.detection_count.load(Ordering::Relaxed);
        if count > 0 {
            total_time / count
        } else {
            0
        }
    }
    
    /// 获取缓存命中率
    #[inline(always)]
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let misses = self.cache_misses.load(Ordering::Relaxed) as f64;
        let total = hits + misses;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }
    
    /// 获取完整性能指标
    pub fn get_metrics(&self) -> crate::performance::PerformanceMetrics {
        crate::performance::PerformanceMetrics {
            detection_latency_ns: self.get_average_detection_latency_ns(),
            simd_operations_count: self.simd_operations.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            opportunities_processed: self.detection_count.load(Ordering::Relaxed),
        }
    }
    
    /// 重置所有计数器
    pub fn reset(&self) {
        self.total_detection_time_ns.store(0, Ordering::Relaxed);
        self.detection_count.store(0, Ordering::Relaxed);
        self.simd_operations.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
        self.error_count.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lockfree_opportunity_pool() {
        let pool = LockFreeOpportunityPool::new(100);
        
        let opportunity = ArbitrageOpportunity {
            id: 0, // 会被自动分配
            strategy_type: StrategyType::InterExchange,
            symbol: "BTCUSDT".to_string(),
            exchange_buy: "binance".to_string(),
            exchange_sell: "okx".to_string(),
            buy_price: FixedPrice::from_f64(50000.0),
            sell_price: FixedPrice::from_f64(50100.0),
            profit: FixedPrice::from_f64(100.0),
            volume: FixedQuantity::from_f64(1.0),
            timestamp_ns: 1234567890,
            confidence_score: 0.95,
        };
        
        // 测试推送和弹出
        assert!(pool.try_push_opportunity(opportunity.clone()).is_ok());
        let popped = pool.try_pop_best().unwrap();
        assert_eq!(popped.symbol, "BTCUSDT");
        assert_eq!(popped.id, 0); // 第一个分配的ID
        
        // 测试统计
        let stats = pool.get_statistics();
        assert_eq!(stats.total_opportunities, 1);
        assert_eq!(stats.inter_exchange_count, 1);
        assert_eq!(stats.triangular_count, 0);
    }
    
    #[test]
    fn test_lockfree_config_cache() {
        let cache = LockFreeConfigCache::new();
        
        // 测试默认值
        let normal_profit = cache.get_min_profit(MarketState::Normal);
        assert!((normal_profit.to_f64() - 0.005).abs() < 0.000001);
        
        // 测试更新
        let new_profit = FixedPrice::from_f64(0.008);
        cache.update_min_profit(MarketState::Normal, new_profit);
        
        let updated_profit = cache.get_min_profit(MarketState::Normal);
        assert!((updated_profit.to_f64() - 0.008).abs() < 0.000001);
        
        // 版本号应该增加
        assert!(cache.get_version() > 1);
    }
    
    #[test]
    fn test_performance_counters() {
        let counters = LockFreePerformanceCounters::new();
        
        // 记录一些延迟
        counters.record_detection_latency(1000); // 1μs
        counters.record_detection_latency(2000); // 2μs
        
        let avg_latency = counters.get_average_detection_latency_ns();
        assert_eq!(avg_latency, 1500); // 平均1.5μs
        
        // 测试缓存命中率
        counters.record_cache_hit();
        counters.record_cache_hit();
        counters.record_cache_miss();
        
        let hit_rate = counters.get_cache_hit_rate();
        assert!((hit_rate - 0.6666666666666666).abs() < 0.0001); // 2/3 ≈ 0.667
    }
    
    #[test]
    fn test_market_state_transitions() {
        let pool = LockFreeOpportunityPool::new(10);
        
        // 测试状态转换
        assert_eq!(pool.get_market_state(), MarketState::Normal);
        
        pool.set_market_state(MarketState::Extreme);
        assert_eq!(pool.get_market_state(), MarketState::Extreme);
        
        pool.set_market_state(MarketState::Cautious);
        assert_eq!(pool.get_market_state(), MarketState::Cautious);
    }
} 