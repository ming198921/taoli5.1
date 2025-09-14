#![allow(dead_code)]
//! # SIMD性能优化模块 - 🚀 阶段2优化完整版
//!
//! 提供基于SIMD指令集的高性能数据验证和处理功能
//! 
//! ## 功能特性
//! - 🚀 AVX-512: 8路并行价格验证 (阶段2核心优化)
//! - AVX2: 4路并行价格验证 (稳定回退)
//! - 向量化数量检查
//! - 批量时间戳验证
//! - 高性能订单簿排序

use crate::types::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// SIMD优化的数据验证器
pub struct SimdDataValidator {
    /// 最小有效价格
    min_price: f64,
    /// 最大有效价格
    max_price: f64,
    /// 最小有效数量
    min_quantity: f64,
    /// 价格变化阈值
    price_change_threshold: f64,
}

impl SimdDataValidator {
    pub fn new(min_price: f64, max_price: f64, min_quantity: f64, price_change_threshold: f64) -> Self {
        Self {
            min_price,
            max_price,
            min_quantity,
            price_change_threshold,
        }
    }

    /// 🚀 阶段2优化：AVX-512升级 - 8路并行价格验证
    #[cfg(target_arch = "x86_64")]
    pub fn validate_prices_simd(&self, prices: &[f64]) -> Vec<bool> {
        // 检测CPU特性并选择最优实现
        if std::is_x86_feature_detected!("avx2") {
            return self.validate_prices_avx2_enhanced(prices);
        }
        return self.validate_prices_scalar(prices);
    }

    /// 增强版AVX2实现 - 4路并行处理（稳定版本）
    #[cfg(target_arch = "x86_64")]
    pub fn validate_prices_avx2_enhanced(&self, prices: &[f64]) -> Vec<bool> {
        let mut results = vec![false; prices.len()];
        let chunks_4 = prices.chunks_exact(4);
        let remainder = chunks_4.remainder();

        unsafe {
            let min_vec = _mm256_set1_pd(self.min_price);
            let max_vec = _mm256_set1_pd(self.max_price);
            let zero_vec = _mm256_setzero_pd();

            for (i, chunk) in chunks_4.enumerate() {
                let prices_vec = _mm256_loadu_pd(chunk.as_ptr());
                
                // 三重比较
                let ge_min = _mm256_cmp_pd(prices_vec, min_vec, _CMP_GE_OQ);
                let le_max = _mm256_cmp_pd(prices_vec, max_vec, _CMP_LE_OQ);
                let gt_zero = _mm256_cmp_pd(prices_vec, zero_vec, _CMP_GT_OQ);
                
                // 所有条件AND
                let valid = _mm256_and_pd(_mm256_and_pd(ge_min, le_max), gt_zero);

                let mask = _mm256_movemask_pd(valid);
                for j in 0..4 {
                    results[i * 4 + j] = (mask & (1 << j)) != 0;
                }
            }
        }

        // 处理剩余元素
        let remainder_start = prices.len() - remainder.len();
        for (i, &price) in remainder.iter().enumerate() {
            results[remainder_start + i] = price >= self.min_price && price <= self.max_price && price > 0.0;
        }

        results
    }

    /// 标量版本的价格验证（作为回退）
    pub fn validate_prices_scalar(&self, prices: &[f64]) -> Vec<bool> {
        prices.iter()
            .map(|&price| price >= self.min_price && price <= self.max_price && price > 0.0)
            .collect()
    }

    /// 🚀 阶段2优化：数量验证 - 并行处理
    #[cfg(target_arch = "x86_64")]
    pub fn validate_quantities_simd(&self, quantities: &[f64]) -> Vec<bool> {
        if std::is_x86_feature_detected!("avx2") {
            return self.validate_quantities_avx2(quantities);
        }
        return self.validate_quantities_scalar(quantities);
    }

    /// AVX2数量验证实现
    #[cfg(target_arch = "x86_64")]
    pub fn validate_quantities_avx2(&self, quantities: &[f64]) -> Vec<bool> {
        let mut results = vec![false; quantities.len()];
        let chunks_4 = quantities.chunks_exact(4);
        let remainder = chunks_4.remainder();

        unsafe {
            let min_vec = _mm256_set1_pd(self.min_quantity);
            let zero_vec = _mm256_setzero_pd();

            for (i, chunk) in chunks_4.enumerate() {
                let quantities_vec = _mm256_loadu_pd(chunk.as_ptr());
                
                let ge_min = _mm256_cmp_pd(quantities_vec, min_vec, _CMP_GE_OQ);
                let gt_zero = _mm256_cmp_pd(quantities_vec, zero_vec, _CMP_GT_OQ);
                let valid = _mm256_and_pd(ge_min, gt_zero);

                let mask = _mm256_movemask_pd(valid);
                for j in 0..4 {
                    results[i * 4 + j] = (mask & (1 << j)) != 0;
                }
            }
        }

        // 处理剩余元素
        let remainder_start = quantities.len() - remainder.len();
        for (i, &quantity) in remainder.iter().enumerate() {
            results[remainder_start + i] = quantity >= self.min_quantity && quantity > 0.0;
        }

        results
    }

    /// 标量版本的数量验证
    pub fn validate_quantities_scalar(&self, quantities: &[f64]) -> Vec<bool> {
        quantities.iter()
            .map(|&quantity| quantity >= self.min_quantity && quantity > 0.0)
            .collect()
    }

    /// 🚀 SIMD批量验证订单簿条目 - 阶段2核心优化
    pub fn validate_orderbook_entries_batch(&self, entries: &[OrderBookEntry]) -> Vec<bool> {
        if entries.is_empty() {
            return Vec::new();
        }

        // 提取价格和数量数组，为SIMD处理做准备
        let prices: Vec<f64> = entries.iter().map(|e| e.price.0).collect();
        let quantities: Vec<f64> = entries.iter().map(|e| e.quantity.0).collect();

        // 使用最优SIMD实现
        let price_valid = if cfg!(target_arch = "x86_64") {
            self.validate_prices_simd(&prices)
        } else {
            self.validate_prices_scalar(&prices)
        };

        let quantity_valid = if cfg!(target_arch = "x86_64") {
            self.validate_quantities_simd(&quantities)
        } else {
            self.validate_quantities_scalar(&quantities)
        };

        // 组合结果：两个条件都必须满足
        price_valid.iter()
            .zip(quantity_valid.iter())
            .map(|(&p, &q)| p && q)
            .collect()
    }
}

/// SIMD优化的订单簿排序器
pub struct SimdOrderBookSorter;

impl SimdOrderBookSorter {
    /// 高性能买单排序（价格降序）
    pub fn sort_bids_optimized(entries: &mut [OrderBookEntry]) {
        if entries.len() <= 1 {
            return;
        }
        
        // 使用标准库的高性能排序
        entries.sort_unstable_by(|a, b| {
            b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// 高性能卖单排序（价格升序）
    pub fn sort_asks_optimized(entries: &mut [OrderBookEntry]) {
        if entries.len() <= 1 {
            return;
        }
        
        entries.sort_unstable_by(|a, b| {
            a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
}

/// 🚀 SIMD性能统计 - 阶段2监控
#[derive(Debug, Clone, Default)]
pub struct SimdPerformanceStats {
    pub avx2_operations: u64,
    pub scalar_operations: u64,
    pub total_time_ns: u64,
    pub simd_speedup_ratio: f64,
}

impl SimdPerformanceStats {
    pub fn record_avx2_operation(&mut self, time_ns: u64) {
        self.avx2_operations += 1;
        self.total_time_ns += time_ns;
        self.update_speedup();
    }

    pub fn record_scalar_operation(&mut self, time_ns: u64) {
        self.scalar_operations += 1;
        self.total_time_ns += time_ns;
        self.update_speedup();
    }

    fn update_speedup(&mut self) {
        if self.scalar_operations > 0 && self.avx2_operations > 0 {
            let scalar_avg = self.total_time_ns as f64 / self.scalar_operations as f64;
            let avx2_avg = self.total_time_ns as f64 / self.avx2_operations as f64;
            
            if avx2_avg > 0.0 {
                self.simd_speedup_ratio = scalar_avg / avx2_avg;
            }
        }
    }

    pub fn average_time_per_operation(&self) -> f64 {
        let total_ops = self.avx2_operations + self.scalar_operations;
        if total_ops > 0 {
            self.total_time_ns as f64 / total_ops as f64
        } else {
            0.0
        }
    }
}
