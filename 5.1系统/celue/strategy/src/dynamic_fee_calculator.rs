//! 动态费率计算器 - 实时手续费率获取和缓存
//! 
//! 替代硬编码手续费，通过context.fee_precision_repo实时获取，
//! 并提供智能缓存和降级策略。

use common_types::StrategyContext;
use common::{
    precision::FixedPrice,
    types::Exchange,
};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::time::{Instant, Duration};
use anyhow::Result;

/// 费率类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeeType {
    Taker,
    Maker,
}

/// 缓存的费率条目
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct CachedFeeEntry {
    fee_rate: FixedPrice,
    last_updated: Instant,
    source: FeeSource,
}

/// 费率来源
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum FeeSource {
    Repo,        // 从fee_precision_repo获取
    Context,     // 从context获取
    Fallback,    // 使用降级值
    MarketData,  // 从市场数据推断（未来扩展）
}

/// 动态费率计算器
pub struct DynamicFeeCalculator {
    /// 费率缓存: (exchange, fee_type) -> CachedFeeEntry
    fee_cache: Arc<RwLock<HashMap<(String, FeeType), CachedFeeEntry>>>,
    /// 缓存TTL
    cache_ttl: Duration,
    /// 降级费率映射
    fallback_rates: HashMap<String, (FixedPrice, FixedPrice)>, // (taker, maker)
    /// 性能统计
    stats: Arc<RwLock<FeeCalculatorStats>>,
}

/// 费率计算器统计
#[derive(Debug, Default, Clone)]
pub struct FeeCalculatorStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub repo_queries: u64,
    pub fallback_used: u64,
    pub avg_query_time_us: f64,
}

impl Default for DynamicFeeCalculator {
    fn default() -> Self {
        Self::new(Duration::from_secs(30)) // 30秒缓存TTL
    }
}

impl DynamicFeeCalculator {
    pub fn new(cache_ttl: Duration) -> Self {
        // 预设主流交易所的降级费率
        let mut fallback_rates = HashMap::new();
        
        // 币安费率
        fallback_rates.insert(
            "binance".to_string(),
            (FixedPrice::from_f64(0.001, 6), FixedPrice::from_f64(0.001, 6)) // 0.1% taker/maker
        );
        
        // OKX费率
        fallback_rates.insert(
            "okx".to_string(),
            (FixedPrice::from_f64(0.0008, 6), FixedPrice::from_f64(0.0008, 6)) // 0.08%
        );
        
        // 火币费率
        fallback_rates.insert(
            "huobi".to_string(),
            (FixedPrice::from_f64(0.002, 6), FixedPrice::from_f64(0.002, 6)) // 0.2%
        );
        
        // Gate.io费率
        fallback_rates.insert(
            "gate".to_string(),
            (FixedPrice::from_f64(0.002, 6), FixedPrice::from_f64(0.002, 6)) // 0.2%
        );
        
        // 默认未知交易所费率（保守估计）
        fallback_rates.insert(
            "default".to_string(),
            (FixedPrice::from_f64(0.003, 6), FixedPrice::from_f64(0.003, 6)) // 0.3%
        );

        Self {
            fee_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
            fallback_rates,
            stats: Arc::new(RwLock::new(FeeCalculatorStats::default())),
        }
    }

    /// 获取动态费率 - 主要API
    pub fn get_fee_rate(
        &self,
        ctx: &dyn StrategyContext,
        exchange: &str,
        fee_type: FeeType,
    ) -> FixedPrice {
        let start_time = Instant::now();
        
        // 检查缓存
        let cache_key = (exchange.to_string(), fee_type);
        if let Some(cached_rate) = self.get_cached_rate(&cache_key) {
            self.update_stats_cache_hit(start_time);
            return cached_rate;
        }

        // 缓存未命中，从各种来源获取
        let fee_rate = self.fetch_fee_rate(ctx, exchange, fee_type);
        
        // 更新缓存
        let cache_entry = CachedFeeEntry {
            fee_rate,
            last_updated: Instant::now(),
            source: self.determine_source(ctx, exchange, fee_type),
        };
        
        self.fee_cache.write().insert(cache_key, cache_entry);
        self.update_stats_cache_miss(start_time);
        
        fee_rate
    }

    /// 检查缓存
    fn get_cached_rate(&self, cache_key: &(String, FeeType)) -> Option<FixedPrice> {
        let cache = self.fee_cache.read();
        if let Some(entry) = cache.get(cache_key) {
            if entry.last_updated.elapsed() < self.cache_ttl {
                return Some(entry.fee_rate);
            }
        }
        None
    }

    /// 从各种来源获取费率
    fn fetch_fee_rate(
        &self,
        ctx: &dyn StrategyContext,
        exchange: &str,
        fee_type: FeeType,
    ) -> FixedPrice {
        let exchange_obj = Exchange::new(exchange);
        
        // 优先级1: 从fee_precision_repo获取具体费率
        if let Some(rate) = self.get_rate_from_repo(ctx, &exchange_obj, fee_type) {
            tracing::debug!("从repo获取{}费率: {:.6}%", exchange, rate.to_f64() * 100.0);
            return rate;
        }

        // 优先级2: 从context获取通用费率
        if let Some(rate) = self.get_rate_from_context(ctx, &exchange_obj, fee_type) {
            tracing::debug!("从context获取{}费率: {:.6}%", exchange, rate.to_f64() * 100.0);
            return rate;
        }

        // 优先级3: 使用降级费率
        let rate = self.get_fallback_rate(exchange, fee_type);
        tracing::warn!("使用{}降级费率: {:.6}%", exchange, rate.to_f64() * 100.0);
        
        // 统计降级使用
        self.stats.write().fallback_used += 1;
        
        rate
    }

    /// 从fee_precision_repo获取费率
    fn get_rate_from_repo(
        &self,
        ctx: &dyn StrategyContext,
        exchange: &Exchange,
        fee_type: FeeType,
    ) -> Option<FixedPrice> {
        self.stats.write().repo_queries += 1;
        
        let rate = match fee_type {
            FeeType::Taker => ctx.get_taker_fee(exchange.as_str()),
            FeeType::Maker => ctx.get_maker_fee(exchange.as_str()),
        };
        
        if rate > 0.0 {
            Some(FixedPrice::from_f64(rate, 6))
        } else {
            None
        }
    }

    /// 从context获取费率
    fn get_rate_from_context(
        &self,
        ctx: &dyn StrategyContext,
        exchange: &Exchange,
        _fee_type: FeeType,
    ) -> Option<FixedPrice> {
        // 通过fee_precision_repo的bps接口获取
        let exchange_str = exchange.as_str();
        
        if let Some(repo) = ctx.fee_precision_repo() {
            if let Some(fee_bps) = repo.get_fee(exchange_str, "rate_bps") {
                let fee_rate = fee_bps / 10000.0; // bps转换为小数
                return Some(FixedPrice::from_f64(fee_rate, 6));
            }
        }
        
        None
    }

    /// 获取降级费率
    fn get_fallback_rate(&self, exchange: &str, fee_type: FeeType) -> FixedPrice {
        let exchange_lower = exchange.to_lowercase();
        
        // 查找匹配的交易所
        let rates = self.fallback_rates.get(&exchange_lower)
            .or_else(|| {
                // 模糊匹配
                self.fallback_rates.iter()
                    .find(|(key, _)| exchange_lower.contains(*key))
                    .map(|(_, rates)| rates)
            })
            .unwrap_or_else(|| self.fallback_rates.get("default").unwrap());

        match fee_type {
            FeeType::Taker => rates.0,
            FeeType::Maker => rates.1,
        }
    }

    /// 确定费率来源
    fn determine_source(
        &self,
        ctx: &dyn StrategyContext,
        exchange: &str,
        fee_type: FeeType,
    ) -> FeeSource {
        let exchange_obj = Exchange::new(exchange);
        
        if self.get_rate_from_repo(ctx, &exchange_obj, fee_type).is_some() {
            FeeSource::Repo
        } else if self.get_rate_from_context(ctx, &exchange_obj, fee_type).is_some() {
            FeeSource::Context
        } else {
            FeeSource::Fallback
        }
    }

    /// 更新缓存命中统计
    fn update_stats_cache_hit(&self, start_time: Instant) {
        let mut stats = self.stats.write();
        stats.cache_hits += 1;
        let query_time = start_time.elapsed().as_micros() as f64;
        stats.avg_query_time_us = (stats.avg_query_time_us * (stats.cache_hits - 1) as f64 + query_time) / stats.cache_hits as f64;
    }

    /// 更新缓存未命中统计
    fn update_stats_cache_miss(&self, start_time: Instant) {
        let mut stats = self.stats.write();
        stats.cache_misses += 1;
        let query_time = start_time.elapsed().as_micros() as f64;
        let total_queries = stats.cache_hits + stats.cache_misses;
        stats.avg_query_time_us = (stats.avg_query_time_us * (total_queries - 1) as f64 + query_time) / total_queries as f64;
    }

    /// 批量获取三角套利费率
    pub fn get_triangular_fee_rates(
        &self,
        ctx: &dyn StrategyContext,
        exchanges: &[&str],
        is_taker: &[bool], // 每一腿是否为taker
    ) -> Result<Vec<FixedPrice>> {
        if exchanges.len() != is_taker.len() {
            return Err(anyhow::anyhow!("交易所数量与taker标志数量不匹配"));
        }

        let mut rates = Vec::new();
        for (exchange, &is_taker_flag) in exchanges.iter().zip(is_taker.iter()) {
            let fee_type = if is_taker_flag { FeeType::Taker } else { FeeType::Maker };
            let rate = self.get_fee_rate(ctx, exchange, fee_type);
            rates.push(rate);
        }

        Ok(rates)
    }

    /// 智能费率建议 - 为三角套利选择最优交易所组合
    pub fn suggest_optimal_exchanges(
        &self,
        ctx: &dyn StrategyContext,
        available_exchanges: &[&str],
        _target_volume_usd: f64,
    ) -> Vec<(String, FixedPrice)> {
        let mut exchange_fees = Vec::new();
        
        for &exchange in available_exchanges {
            let taker_fee = self.get_fee_rate(ctx, exchange, FeeType::Taker);
            let maker_fee = self.get_fee_rate(ctx, exchange, FeeType::Maker);
            
            // 计算综合费率（假设50%taker，50%maker）
            let blended_fee = FixedPrice::from_f64(
                (taker_fee.to_f64() + maker_fee.to_f64()) / 2.0,
                6
            );
            
            exchange_fees.push((exchange.to_string(), blended_fee));
        }
        
        // 按费率排序（升序）
        exchange_fees.sort_by(|a, b| a.1.to_f64().partial_cmp(&b.1.to_f64()).unwrap());
        
        tracing::info!("费率排序结果（最优先）:");
        for (i, (exchange, fee)) in exchange_fees.iter().enumerate() {
            tracing::info!("  {}. {}: {:.6}%", i + 1, exchange, fee.to_f64() * 100.0);
        }
        
        exchange_fees
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> FeeCalculatorStats {
        self.stats.read().clone()
    }

    /// 清除缓存
    pub fn clear_cache(&self) {
        self.fee_cache.write().clear();
        tracing::info!("费率缓存已清除");
    }

    /// 预热缓存 - 预先加载常用交易所费率
    pub fn warmup_cache(&self, ctx: &dyn StrategyContext, exchanges: &[&str]) {
        tracing::info!("预热费率缓存，交易所: {:?}", exchanges);
        
        for &exchange in exchanges {
            // 预加载taker和maker费率
            let _ = self.get_fee_rate(ctx, exchange, FeeType::Taker);
            let _ = self.get_fee_rate(ctx, exchange, FeeType::Maker);
        }
        
        let cache_size = self.fee_cache.read().len();
        tracing::info!("缓存预热完成，已加载{}个费率条目", cache_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::{FeePrecisionRepoImpl, StrategyContext};

    #[test]
    fn test_fallback_rates() {
        let calculator = DynamicFeeCalculator::default();
        
        // 测试已知交易所
        let binance_rate = calculator.get_fallback_rate("binance", FeeType::Taker);
        assert_eq!(binance_rate.to_f64(), 0.001);
        
        // 测试未知交易所
        let unknown_rate = calculator.get_fallback_rate("unknown_exchange", FeeType::Taker);
        assert_eq!(unknown_rate.to_f64(), 0.003);
    }

    #[test]
    fn test_cache_behavior() {
        let calculator = DynamicFeeCalculator::new(Duration::from_millis(100));
        let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
        let strategy_config = StrategyConfig::default();
        let ctx = StrategyContext::new(fee_repo, strategy_config, None, None, None, None);
        
        // 第一次调用应该缓存未命中
        let rate1 = calculator.get_fee_rate(&ctx, "binance", FeeType::Taker);
        let stats1 = calculator.get_stats();
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.cache_hits, 0);
        
        // 第二次调用应该缓存命中
        let rate2 = calculator.get_fee_rate(&ctx, "binance", FeeType::Taker);
        let stats2 = calculator.get_stats();
        assert_eq!(stats2.cache_hits, 1);
        assert_eq!(rate1.to_f64(), rate2.to_f64());
        
        // 等待缓存过期
        std::thread::sleep(Duration::from_millis(150));
        
        // 第三次调用应该再次缓存未命中
        let _rate3 = calculator.get_fee_rate(&ctx, "binance", FeeType::Taker);
        let stats3 = calculator.get_stats();
        assert_eq!(stats3.cache_misses, 2);
    }
} 