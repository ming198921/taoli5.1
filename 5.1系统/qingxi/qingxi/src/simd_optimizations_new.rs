#![allow(dead_code)]
//! # SIMD性能优化模块
//!
//! 提供基于SIMD指令集的高性能数据验证和处理功能
//! 
//! ## 功能特性
//! - 🚀 AVX-512: 8路并行价格验证 (阶段2新增)
//! - AVX2: 4路并行价格验证 (回退)
//! - 向量化数量检查
//! - 批量时间戳验证
//! - 高性能订单簿排序

// 🚀 阶段2优化：启用内联汇编特性
#![allow(incomplete_features)]

use crate::types::*;
use tracing::instrument;
use ordered_float::OrderedFloat;

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
        // 优先使用AVX-512，回退到AVX2
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512dq") {
            return self.validate_prices_avx512(prices);
        }
        if is_x86_feature_detected!("avx2") {
            return self.validate_prices_avx2_enhanced(prices);
        }
        return self.validate_prices_scalar(prices);
    }

    /// 🚀 AVX-512实现 - 8路并行处理（使用内联汇编）
    #[cfg(target_arch = "x86_64")]
    pub fn validate_prices_avx512(&self, prices: &[f64]) -> Vec<bool> {
        let mut results = vec![false; prices.len()];
        let mut chunks_iter = prices.chunks_exact(8);

        // 使用安全的汇编包装器
        for (i, chunk) in chunks_iter.by_ref().enumerate() {
            let valid_mask = self.avx512_compare_chunk(chunk);
            
            // 提取8位结果
            for j in 0..8 {
                results[i * 8 + j] = (valid_mask & (1u8 << j)) != 0;
            }
        }

        // 处理剩余元素
        let remainder = chunks_iter.remainder();
        for (i, &price) in remainder.iter().enumerate() {
            results[prices.len() - remainder.len() + i] = price >= self.min_price && price <= self.max_price;
        }

        results
    }

    /// AVX-512内联汇编包装器
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f,avx512dq")]
    unsafe fn avx512_compare_chunk(&self, chunk: &[f64]) -> u8 {
        let mut result: u8;
        let min_val = self.min_price;
        let max_val = self.max_price;
        
        std::arch::asm!(
            // 加载数据到ZMM寄存器
            "vmovupd zmm0, [{data}]",           // 加载8个价格到zmm0
            "vbroadcastsd zmm1, {min}",         // 广播最小值到zmm1
            "vbroadcastsd zmm2, {max}",         // 广播最大值到zmm2
            
            // 比较操作
            "vcmppd k1, zmm0, zmm1, 0x1d",     // 价格 >= 最小值，结果存储到k1
            "vcmppd k2, zmm0, zmm2, 0x12",     // 价格 <= 最大值，结果存储到k2
            "kandw k0, k1, k2",                // k0 = k1 AND k2
            "kmovw {result:e}, k0",            // 移动掩码到结果
            
            data = in(reg) chunk.as_ptr(),
            min = in(xmm_reg) min_val,
            max = in(xmm_reg) max_val,
            result = out(reg) result,
            out("zmm0") _,
            out("zmm1") _,
            out("zmm2") _,
            out("k0") _,
            out("k1") _,
            out("k2") _,
        );
        
        result
    }

    /// 增强版AVX2实现 - 4路并行处理，优化版本
    #[cfg(target_arch = "x86_64")]
    pub fn validate_prices_avx2_enhanced(&self, prices: &[f64]) -> Vec<bool> {
        let mut results = vec![false; prices.len()];
        let mut chunks_iter = prices.chunks_exact(4);

        unsafe {
            let min_vec = _mm256_set1_pd(self.min_price);
            let max_vec = _mm256_set1_pd(self.max_price);

            for (i, chunk) in chunks_iter.by_ref().enumerate() {
                let prices_vec = _mm256_loadu_pd(chunk.as_ptr());
                
                let ge_min = _mm256_cmp_pd(prices_vec, min_vec, _CMP_GE_OQ);
                let le_max = _mm256_cmp_pd(prices_vec, max_vec, _CMP_LE_OQ);
                let valid = _mm256_and_pd(ge_min, le_max);

                let mask = _mm256_movemask_pd(valid);
                for j in 0..4 {
                    results[i * 4 + j] = (mask & (1 << j)) != 0;
                }
            }
        }

        // 处理剩余元素
        let remainder = chunks_iter.remainder();
        for (i, &price) in remainder.iter().enumerate() {
            results[prices.len() - remainder.len() + i] = price >= self.min_price && price <= self.max_price;
        }

        results
    }

    /// 标量版本的价格验证（作为回退）
    pub fn validate_prices_scalar(&self, prices: &[f64]) -> Vec<bool> {
        prices.iter()
            .map(|&price| price >= self.min_price && price <= self.max_price && price > 0.0)
            .collect()
    }

    /// 🚀 阶段2优化：AVX-512数量验证 - 8路并行处理
    #[cfg(target_arch = "x86_64")]
    pub fn validate_quantities_simd(&self, quantities: &[f64]) -> Vec<bool> {
        // 优先使用AVX-512
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512dq") {
            return self.validate_quantities_avx512(quantities);
        }
        // 回退到AVX2
        if is_x86_feature_detected!("avx2") {
            return self.validate_quantities_avx2(quantities);
        }
        return self.validate_quantities_scalar(quantities);
    }

    /// AVX-512数量验证实现
    #[cfg(target_arch = "x86_64")]
    pub fn validate_quantities_avx512(&self, quantities: &[f64]) -> Vec<bool> {
        let mut results = vec![false; quantities.len()];
        let mut chunks_iter = quantities.chunks_exact(8);

        for (i, chunk) in chunks_iter.by_ref().enumerate() {
            let valid_mask = self.avx512_validate_quantities_chunk(chunk);
            
            for j in 0..8 {
                results[i * 8 + j] = (valid_mask & (1u8 << j)) != 0;
            }
        }

        // 处理剩余元素
        let remainder = chunks_iter.remainder();
        for (i, &qty) in remainder.iter().enumerate() {
            results[quantities.len() - remainder.len() + i] = qty >= self.min_quantity && qty > 0.0;
        }

        results
    }

    /// AVX-512数量验证内联汇编
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx512f,avx512dq")]
    unsafe fn avx512_validate_quantities_chunk(&self, chunk: &[f64]) -> u8 {
        let mut result: u8;
        let min_val = self.min_quantity;
        let zero_val = 0.0f64;
        
        std::arch::asm!(
            // 加载数据
            "vmovupd zmm0, [{data}]",           // 加载8个数量
            "vbroadcastsd zmm1, {min}",         // 广播最小值
            "vbroadcastsd zmm2, {zero}",        // 广播零值
            
            // 比较操作
            "vcmppd k1, zmm0, zmm1, 0x1d",     // 数量 >= 最小值
            "vcmppd k2, zmm0, zmm2, 0x1e",     // 数量 > 0
            "kandw k0, k1, k2",                // 两个条件都满足
            "kmovw {result:e}, k0",
            
            data = in(reg) chunk.as_ptr(),
            min = in(xmm_reg) min_val,
            zero = in(xmm_reg) zero_val,
            result = out(reg) result,
            out("zmm0") _,
            out("zmm1") _,
            out("zmm2") _,
            out("k0") _,
            out("k1") _,
            out("k2") _,
        );
        
        result
    }

    /// AVX2数量验证实现（回退版本）
    #[cfg(target_arch = "x86_64")]
    pub fn validate_quantities_avx2(&self, quantities: &[f64]) -> Vec<bool> {
        let mut results = vec![false; quantities.len()];
        let mut chunks_iter = quantities.chunks_exact(4);

        unsafe {
            let min_vec = _mm256_set1_pd(self.min_quantity);
            let zero_vec = _mm256_setzero_pd();

            for (i, chunk) in chunks_iter.by_ref().enumerate() {
                let quantities_vec = _mm256_loadu_pd(chunk.as_ptr());
                
                // 检查数量范围：quantity >= min_quantity && quantity > 0
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
        let remainder = chunks_iter.remainder();
        for (i, &quantity) in remainder.iter().enumerate() {
            results[quantities.len() - remainder.len() + i] = quantity >= self.min_quantity && quantity > 0.0;
        }

        results
    }

    /// 标量版本的数量验证
    pub fn validate_quantities_scalar(&self, quantities: &[f64]) -> Vec<bool> {
        quantities.iter()
            .map(|&quantity| quantity >= self.min_quantity && quantity > 0.0)
            .collect()
    }

    /// SIMD批量验证订单簿条目
    pub fn validate_orderbook_entries_batch(&self, entries: &[OrderBookEntry]) -> Vec<bool> {
        let prices: Vec<f64> = entries.iter().map(|e| e.price.0).collect();
        let quantities: Vec<f64> = entries.iter().map(|e| e.quantity.0).collect();

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

        price_valid.iter()
            .zip(quantity_valid.iter())
            .map(|(&p, &q)| p && q)
            .collect()
    }

    /// 高性能价格连续性检查
    #[cfg(target_arch = "x86_64")]
    pub fn check_price_continuity_simd(&self, prices: &[f64]) -> bool {
        if prices.len() < 2 || !is_x86_feature_detected!("avx2") {
            return self.check_price_continuity_scalar(prices);
        }

        let threshold = self.price_change_threshold;
        let chunks = prices.windows(2).collect::<Vec<_>>();
        let chunk_size = 4;

        for chunk_group in chunks.chunks(chunk_size) {
            unsafe {
                let threshold_vec = _mm256_set1_pd(threshold);
                
                // 准备当前价格和下一价格的向量
                let mut curr_prices = [0.0f64; 4];
                let mut next_prices = [0.0f64; 4];
                
                for (i, window) in chunk_group.iter().enumerate() {
                    if i < 4 {
                        curr_prices[i] = window[0];
                        next_prices[i] = window[1];
                    }
                }

                let curr_vec = _mm256_loadu_pd(curr_prices.as_ptr());
                let next_vec = _mm256_loadu_pd(next_prices.as_ptr());

                // 计算价格变化率：|next - curr| / curr
                let diff = _mm256_sub_pd(next_vec, curr_vec);
                let abs_diff = _mm256_andnot_pd(_mm256_set1_pd(-0.0), diff); // 绝对值
                let ratio = _mm256_div_pd(abs_diff, curr_vec);

                // 检查是否超过阈值
                let exceeds = _mm256_cmp_pd(ratio, threshold_vec, _CMP_GT_OQ);
                let mask = _mm256_movemask_pd(exceeds);

                if mask != 0 {
                    return false; // 发现异常跳跃
                }
            }
        }

        true
    }

    /// 标量版本的价格连续性检查
    pub fn check_price_continuity_scalar(&self, prices: &[f64]) -> bool {
        for window in prices.windows(2) {
            let change_ratio = (window[1] - window[0]).abs() / window[0];
            if change_ratio > self.price_change_threshold {
                return false;
            }
        }
        true
    }
}

/// SIMD优化的订单簿排序器
pub struct SimdOrderBookSorter;

impl SimdOrderBookSorter {
    /// 高性能订单簿排序
    pub fn sort_bids_optimized(entries: &mut [OrderBookEntry]) {
        // 使用高性能排序算法，针对订单簿特点优化
        entries.sort_unstable_by(|a, b| {
            b.price.partial_cmp(&a.price).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn sort_asks_optimized(entries: &mut [OrderBookEntry]) {
        entries.sort_unstable_by(|a, b| {
            a.price.partial_cmp(&b.price).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// 向量化价格级别合并
    pub fn merge_price_levels(entries: &mut Vec<OrderBookEntry>) {
        if entries.is_empty() {
            return;
        }

        let mut write_index = 0;
        let mut current_price = entries[0].price;
        let mut current_quantity = entries[0].quantity;

        for read_index in 1..entries.len() {
            if entries[read_index].price == current_price {
                // 相同价格，合并数量
                current_quantity.0 += entries[read_index].quantity.0;
            } else {
                // 不同价格，写入当前累积结果
                entries[write_index] = OrderBookEntry {
                    price: current_price,
                    quantity: current_quantity,
                };
                write_index += 1;
                current_price = entries[read_index].price;
                current_quantity = entries[read_index].quantity;
            }
        }

        // 写入最后一个价格级别
        entries[write_index] = OrderBookEntry {
            price: current_price,
            quantity: current_quantity,
        };
        write_index += 1;

        // 截断向量到实际大小
        entries.truncate(write_index);
    }
}

/// SIMD性能统计
#[derive(Debug, Clone, Default)]
pub struct SimdPerformanceStats {
    pub simd_operations: u64,
    pub scalar_operations: u64,
    pub simd_time_ns: u64,
    pub scalar_time_ns: u64,
    pub simd_speedup_ratio: f64,
}

impl SimdPerformanceStats {
    pub fn update_simd_time(&mut self, time_ns: u64) {
        self.simd_operations += 1;
        self.simd_time_ns += time_ns;
        self.update_speedup_ratio();
    }

    pub fn update_scalar_time(&mut self, time_ns: u64) {
        self.scalar_operations += 1;
        self.scalar_time_ns += time_ns;
        self.update_speedup_ratio();
    }

    fn update_speedup_ratio(&mut self) {
        if self.scalar_operations > 0 && self.simd_operations > 0 {
            let scalar_avg = self.scalar_time_ns as f64 / self.scalar_operations as f64;
            let simd_avg = self.simd_time_ns as f64 / self.simd_operations as f64;
            
            if simd_avg > 0.0 {
                self.simd_speedup_ratio = scalar_avg / simd_avg;
            }
        }
    }

    pub fn average_simd_time(&self) -> f64 {
        if self.simd_operations > 0 {
            self.simd_time_ns as f64 / self.simd_operations as f64
        } else {
            0.0
        }
    }

    pub fn average_scalar_time(&self) -> f64 {
        if self.scalar_operations > 0 {
            self.scalar_time_ns as f64 / self.scalar_operations as f64
        } else {
            0.0
        }
    }
}
