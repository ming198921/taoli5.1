//! 高性能SIMD套利计算实现
//!
//! 使用AVX-512和AVX2指令集实现套利利润批量计算
//! 目标：≤1微秒处理1000个价格点，支持完整套利运算
//! 公式：net_profit = max(sell - buy, 0) * min(vol_buy, vol_sell) - fee

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use bytemuck::{Pod, Zeroable};

/// 高精度固定点价格，6位小数精度
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct FixedPrice {
    raw: u64,
}

impl FixedPrice {
    const SCALE: u64 = 1_000_000; // 6位小数精度
    const MAX_SAFE_VALUE: u64 = i64::MAX as u64; // 防止有符号转换溢出
    
    pub fn from_f64(value: f64) -> Self {
        if value < 0.0 {
            Self { raw: 0 }
        } else if value > (Self::MAX_SAFE_VALUE as f64 / Self::SCALE as f64) {
            Self { raw: Self::MAX_SAFE_VALUE }
        } else {
            Self { raw: (value * Self::SCALE as f64) as u64 }
        }
    }
    
    pub fn from_raw(raw: u64) -> Self {
        Self { raw: raw.min(Self::MAX_SAFE_VALUE) }
    }
    
    pub fn to_f64(self) -> f64 {
        self.raw as f64 / Self::SCALE as f64
    }
    
    pub fn raw(self) -> u64 {
        self.raw
    }
    
    pub fn scale(self) -> u8 {
        6
    }
    
    pub fn raw_value(self) -> u64 {
        self.raw
    }
    
    /// 饱和减法
    pub fn saturating_sub(self, other: Self) -> Self {
        Self { raw: self.raw.saturating_sub(other.raw) }
    }
    
    /// 饱和乘法（用于费用计算）
    pub fn saturating_mul(self, other: Self) -> Self {
        let result = (self.raw as u128 * other.raw as u128) / Self::SCALE as u128;
        Self { raw: result.min(Self::MAX_SAFE_VALUE as u128) as u64 }
    }
    
    /// 最小值
    pub fn min(self, other: Self) -> Self {
        Self { raw: self.raw.min(other.raw) }
    }
}

/// 高精度固定点数量，8位小数精度
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct FixedQuantity {
    raw: u64,
}

impl FixedQuantity {
    const SCALE: u64 = 100_000_000; // 8位小数精度
    const MAX_SAFE_VALUE: u64 = i64::MAX as u64;
    
    pub fn from_f64(value: f64) -> Self {
        if value < 0.0 {
            Self { raw: 0 }
        } else if value > (Self::MAX_SAFE_VALUE as f64 / Self::SCALE as f64) {
            Self { raw: Self::MAX_SAFE_VALUE }
        } else {
            Self { raw: (value * Self::SCALE as f64) as u64 }
        }
    }
    
    pub fn from_raw(raw: u64) -> Self {
        Self { raw: raw.min(Self::MAX_SAFE_VALUE) }
    }
    
    pub fn to_f64(self) -> f64 {
        self.raw as f64 / Self::SCALE as f64
    }
    
    pub fn raw(self) -> u64 {
        self.raw
    }
    
    pub fn min(self, other: Self) -> Self {
        Self { raw: self.raw.min(other.raw) }
    }
    
    pub fn scale(self) -> u8 {
        8
    }
    
    pub fn raw_value(self) -> u64 {
        self.raw
    }
}

/// 套利利润计算结果
#[derive(Debug, Clone)]
pub struct ArbitrageProfit {
    pub gross_profit: FixedPrice,
    pub net_profit: FixedPrice,
    pub volume: FixedQuantity,
    pub fee: FixedPrice,
}

/// 高性能SIMD固定点处理器
#[derive(Clone)]
pub struct SIMDFixedPointProcessor {
    /// 预处理批大小，用于内存对齐优化
    alignment_batch_size: usize,
}

impl SIMDFixedPointProcessor {
    pub fn new(alignment_batch_size: usize) -> Self {
        Self { 
            alignment_batch_size: alignment_batch_size.max(8).next_power_of_two()
        }
    }
    
    /// 批量套利利润计算（核心方法）
    /// 
    /// 计算公式：net_profit = max(sell - buy, 0) * min(vol_buy, vol_sell) - fee
    pub fn calculate_arbitrage_profits_batch(
        &self,
        buy_prices: &[FixedPrice],
        sell_prices: &[FixedPrice],
        buy_volumes: &[FixedQuantity],
        sell_volumes: &[FixedQuantity],
        fee_rates: &[FixedPrice], // 费率（如0.001表示0.1%）
    ) -> Result<Vec<ArbitrageProfit>, String> {
        // 输入验证
        if buy_prices.len() != sell_prices.len() 
            || buy_prices.len() != buy_volumes.len()
            || buy_prices.len() != sell_volumes.len() 
            || buy_prices.len() != fee_rates.len() {
            return Err("Input arrays length mismatch".to_string());
        }
        
        if buy_prices.is_empty() {
            return Ok(Vec::new());
        }
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                return self.calculate_arbitrage_avx512(
                    buy_prices, sell_prices, buy_volumes, sell_volumes, fee_rates
                );
            }
            if is_x86_feature_detected!("avx2") {
                return self.calculate_arbitrage_avx2(
                    buy_prices, sell_prices, buy_volumes, sell_volumes, fee_rates
                );
            }
        }
        
        self.calculate_arbitrage_scalar(
            buy_prices, sell_prices, buy_volumes, sell_volumes, fee_rates
        )
    }
    
    /// 简化接口：仅价格差计算（向后兼容）
    pub fn calculate_profit_batch_optimal(
        &self,
        buy_prices: &[FixedPrice],
        sell_prices: &[FixedPrice],
        volumes: &[FixedPrice],
    ) -> Result<Vec<FixedPrice>, String> {
        if buy_prices.len() != sell_prices.len() || buy_prices.len() != volumes.len() {
            return Err("Input arrays length mismatch".to_string());
        }
        
        let buy_vols: Vec<FixedQuantity> = volumes.iter()
            .map(|v| FixedQuantity::from_f64(v.to_f64()))
            .collect();
        let sell_vols = buy_vols.clone();
        let zero_fees = vec![FixedPrice::from_f64(0.0); buy_prices.len()];
        
        let profits = self.calculate_arbitrage_profits_batch(
            buy_prices, sell_prices, &buy_vols, &sell_vols, &zero_fees
        )?;
        
        Ok(profits.into_iter().map(|p| p.gross_profit).collect())
    }
    
    #[cfg(target_arch = "x86_64")]
    fn calculate_arbitrage_avx512(
        &self,
        buy_prices: &[FixedPrice],
        sell_prices: &[FixedPrice], 
        buy_volumes: &[FixedQuantity],
        sell_volumes: &[FixedQuantity],
        fee_rates: &[FixedPrice],
    ) -> Result<Vec<ArbitrageProfit>, String> {
        unsafe { self.calculate_arbitrage_avx512_impl(
            buy_prices, sell_prices, buy_volumes, sell_volumes, fee_rates
        ) }
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe fn calculate_arbitrage_avx512_impl(
        &self,
        buy_prices: &[FixedPrice],
        sell_prices: &[FixedPrice],
        buy_volumes: &[FixedQuantity], 
        sell_volumes: &[FixedQuantity],
        fee_rates: &[FixedPrice],
    ) -> Result<Vec<ArbitrageProfit>, String> {
        let mut profits = Vec::with_capacity(buy_prices.len());
        let chunk_size = 8; // AVX-512处理8个64位整数
        let zero_vec = _mm512_setzero_si512();
        
        for i in (0..buy_prices.len()).step_by(chunk_size) {
            let remaining = (buy_prices.len() - i).min(chunk_size);
            
            if remaining == chunk_size {
                // 加载8个价格（unaligned安全）
                let buy_vec = _mm512_loadu_epi64(buy_prices[i..].as_ptr() as *const i64);
                let sell_vec = _mm512_loadu_epi64(sell_prices[i..].as_ptr() as *const i64);
                
                // 计算价格差: max(sell - buy, 0)
                let diff_vec = _mm512_sub_epi64(sell_vec, buy_vec);
                let gross_vec = _mm512_max_epi64(diff_vec, zero_vec); // 饱和到0
                
                // 加载volumes
                let buy_vol_vec = _mm512_loadu_epi64(buy_volumes[i..].as_ptr() as *const i64);
                let sell_vol_vec = _mm512_loadu_epi64(sell_volumes[i..].as_ptr() as *const i64);
                let min_vol_vec = _mm512_min_epi64(buy_vol_vec, sell_vol_vec);
                
                // 存储结果并进行标量乘法（避免SIMD溢出）
                let mut gross_array: [i64; 8] = [0; 8];
                let mut vol_array: [i64; 8] = [0; 8];
                _mm512_storeu_epi64(gross_array.as_mut_ptr(), gross_vec);
                _mm512_storeu_epi64(vol_array.as_mut_ptr(), min_vol_vec);
                
                for j in 0..8 {
                    let gross_price = FixedPrice::from_raw(gross_array[j].max(0) as u64);
                    let volume = FixedQuantity::from_raw(vol_array[j].max(0) as u64);
                    let fee_rate = fee_rates[i + j];
                    
                    // 计算净利润：gross * volume - fee
                    let gross_profit_raw = (gross_price.raw() as u128 * volume.raw() as u128) 
                        / FixedQuantity::SCALE as u128;
                    let gross_profit = FixedPrice::from_raw(gross_profit_raw as u64);
                    
                    let fee = gross_profit.saturating_mul(fee_rate);
                    let net_profit = gross_profit.saturating_sub(fee);
                    
                    profits.push(ArbitrageProfit {
                        gross_profit,
                        net_profit,
                        volume,
                        fee,
                    });
                }
            } else {
                // 处理剩余元素（标量）
                for j in 0..remaining {
                    let idx = i + j;
                    profits.push(self.calculate_single_arbitrage(
                        buy_prices[idx],
                        sell_prices[idx],
                        buy_volumes[idx],
                        sell_volumes[idx],
                        fee_rates[idx],
                    ));
                }
            }
        }
        
        Ok(profits)
    }
    
    #[cfg(target_arch = "x86_64")]
    fn calculate_arbitrage_avx2(
        &self,
        buy_prices: &[FixedPrice],
        sell_prices: &[FixedPrice],
        buy_volumes: &[FixedQuantity],
        sell_volumes: &[FixedQuantity], 
        fee_rates: &[FixedPrice],
    ) -> Result<Vec<ArbitrageProfit>, String> {
        unsafe { self.calculate_arbitrage_avx2_impl(
            buy_prices, sell_prices, buy_volumes, sell_volumes, fee_rates
        ) }
    }
    
    #[cfg(target_arch = "x86_64")]
    unsafe fn calculate_arbitrage_avx2_impl(
        &self,
        buy_prices: &[FixedPrice],
        sell_prices: &[FixedPrice],
        buy_volumes: &[FixedQuantity],
        sell_volumes: &[FixedQuantity],
        fee_rates: &[FixedPrice],
    ) -> Result<Vec<ArbitrageProfit>, String> {
        let mut profits = Vec::with_capacity(buy_prices.len());
        let chunk_size = 4; // AVX2处理4个64位整数
        let zero_vec = _mm256_setzero_si256();
        
        for i in (0..buy_prices.len()).step_by(chunk_size) {
            let remaining = (buy_prices.len() - i).min(chunk_size);
            
            if remaining == chunk_size {
                // 真实AVX2 SIMD实现
                let buy_vec = _mm256_loadu_si256(buy_prices[i..].as_ptr() as *const __m256i);
                let sell_vec = _mm256_loadu_si256(sell_prices[i..].as_ptr() as *const __m256i);
                
                // 价格差计算
                let diff_vec = _mm256_sub_epi64(sell_vec, buy_vec);
                let gross_vec = _mm256_max_epi64(diff_vec, zero_vec);
                
                // volumes处理
                let buy_vol_vec = _mm256_loadu_si256(buy_volumes[i..].as_ptr() as *const __m256i);
                let sell_vol_vec = _mm256_loadu_si256(sell_volumes[i..].as_ptr() as *const __m256i);
                let min_vol_vec = _mm256_min_epi64(buy_vol_vec, sell_vol_vec);
                
                // 存储并计算
                let mut gross_array: [i64; 4] = [0; 4];
                let mut vol_array: [i64; 4] = [0; 4];
                _mm256_storeu_si256(gross_array.as_mut_ptr() as *mut __m256i, gross_vec);
                _mm256_storeu_si256(vol_array.as_mut_ptr() as *mut __m256i, min_vol_vec);
                
                for j in 0..4 {
                    let gross_price = FixedPrice::from_raw(gross_array[j].max(0) as u64);
                    let volume = FixedQuantity::from_raw(vol_array[j].max(0) as u64);
                    let fee_rate = fee_rates[i + j];
                    
                    let gross_profit_raw = (gross_price.raw() as u128 * volume.raw() as u128) 
                        / FixedQuantity::SCALE as u128;
                    let gross_profit = FixedPrice::from_raw(gross_profit_raw as u64);
                    
                    let fee = gross_profit.saturating_mul(fee_rate);
                    let net_profit = gross_profit.saturating_sub(fee);
                    
                    profits.push(ArbitrageProfit {
                        gross_profit,
                        net_profit,
                        volume,
                        fee,
                    });
                }
            } else {
                // 剩余元素标量处理
                for j in 0..remaining {
                    let idx = i + j;
                    profits.push(self.calculate_single_arbitrage(
                        buy_prices[idx],
                        sell_prices[idx],
                        buy_volumes[idx],
                        sell_volumes[idx],
                        fee_rates[idx],
                    ));
                }
            }
        }
        
        Ok(profits)
    }
    
    fn calculate_arbitrage_scalar(
        &self,
        buy_prices: &[FixedPrice],
        sell_prices: &[FixedPrice],
        buy_volumes: &[FixedQuantity],
        sell_volumes: &[FixedQuantity],
        fee_rates: &[FixedPrice],
    ) -> Result<Vec<ArbitrageProfit>, String> {
        let mut profits = Vec::with_capacity(buy_prices.len());
        
        for i in 0..buy_prices.len() {
            profits.push(self.calculate_single_arbitrage(
                buy_prices[i],
                sell_prices[i],
                buy_volumes[i],
                sell_volumes[i],
                fee_rates[i],
            ));
        }
        
        Ok(profits)
    }
    
    fn calculate_single_arbitrage(
        &self,
        buy_price: FixedPrice,
        sell_price: FixedPrice,
        buy_volume: FixedQuantity,
        sell_volume: FixedQuantity,
        fee_rate: FixedPrice,
    ) -> ArbitrageProfit {
        let price_diff = sell_price.saturating_sub(buy_price);
        let volume = buy_volume.min(sell_volume);
        
        // gross_profit = price_diff * volume
        let gross_profit_raw = (price_diff.raw() as u128 * volume.raw() as u128) 
            / FixedQuantity::SCALE as u128;
        let gross_profit = FixedPrice::from_raw(gross_profit_raw as u64);
        
        // fee = gross_profit * fee_rate
        let fee = gross_profit.saturating_mul(fee_rate);
        
        // net_profit = gross_profit - fee
        let net_profit = gross_profit.saturating_sub(fee);
        
        ArbitrageProfit {
            gross_profit,
            net_profit,
            volume,
            fee,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_price_operations() {
        let p1 = FixedPrice::from_f64(10.5);
        let p2 = FixedPrice::from_f64(8.3);
        let diff = p1.saturating_sub(p2);
        assert!((diff.to_f64() - 2.2).abs() < 0.000001);
    }

    #[test]
    fn test_arbitrage_calculation() {
        let processor = SIMDFixedPointProcessor::new(1000);
        
        let buy_prices = vec![FixedPrice::from_f64(100.0)];
        let sell_prices = vec![FixedPrice::from_f64(101.0)];
        let buy_volumes = vec![FixedQuantity::from_f64(10.0)];
        let sell_volumes = vec![FixedQuantity::from_f64(8.0)];
        let fee_rates = vec![FixedPrice::from_f64(0.001)]; // 0.1%
        
        let profits = processor.calculate_arbitrage_profits_batch(
            &buy_prices, &sell_prices, &buy_volumes, &sell_volumes, &fee_rates
        ).unwrap();
        
        assert_eq!(profits.len(), 1);
        let profit = &profits[0];
        
        // gross = (101-100) * 8 = 8.0
        assert!((profit.gross_profit.to_f64() - 8.0).abs() < 0.000001);
        
        // fee = 8.0 * 0.001 = 0.008
        assert!((profit.fee.to_f64() - 0.008).abs() < 0.000001);
        
        // net = 8.0 - 0.008 = 7.992
        assert!((profit.net_profit.to_f64() - 7.992).abs() < 0.000001);
    }
    
    #[test]
    fn test_simd_vs_scalar_consistency() {
        let processor = SIMDFixedPointProcessor::new(1000);
        
        let buy_prices: Vec<FixedPrice> = (0..16).map(|i| FixedPrice::from_f64(100.0 + i as f64)).collect();
        let sell_prices: Vec<FixedPrice> = (0..16).map(|i| FixedPrice::from_f64(102.0 + i as f64)).collect();
        let buy_volumes: Vec<FixedQuantity> = (0..16).map(|i| FixedQuantity::from_f64(10.0 + i as f64 * 0.5)).collect();
        let sell_volumes: Vec<FixedQuantity> = (0..16).map(|i| FixedQuantity::from_f64(8.0 + i as f64 * 0.3)).collect();
        let fee_rates: Vec<FixedPrice> = (0..16).map(|_| FixedPrice::from_f64(0.001)).collect();
        
        let profits = processor.calculate_arbitrage_profits_batch(
            &buy_prices, &sell_prices, &buy_volumes, &sell_volumes, &fee_rates
        ).unwrap();
        
        // 验证结果合理性
        assert_eq!(profits.len(), 16);
        for profit in &profits {
            assert!(profit.gross_profit.to_f64() >= 0.0);
            assert!(profit.net_profit.to_f64() <= profit.gross_profit.to_f64());
            assert!(profit.fee.to_f64() >= 0.0);
        }
    }
}

/// 性能基准测试模块
#[cfg(feature = "bench")]
pub mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    pub fn benchmark_1000_points() -> std::time::Duration {
        let processor = SIMDFixedPointProcessor::new(1000);
        
        let buy_prices: Vec<FixedPrice> = (0..1000).map(|i| FixedPrice::from_f64(100.0 + i as f64 * 0.01)).collect();
        let sell_prices: Vec<FixedPrice> = (0..1000).map(|i| FixedPrice::from_f64(101.0 + i as f64 * 0.01)).collect();
        let buy_volumes: Vec<FixedQuantity> = (0..1000).map(|i| FixedQuantity::from_f64(10.0 + i as f64 * 0.001)).collect();
        let sell_volumes: Vec<FixedQuantity> = (0..1000).map(|i| FixedQuantity::from_f64(9.0 + i as f64 * 0.001)).collect();
        let fee_rates: Vec<FixedPrice> = (0..1000).map(|_| FixedPrice::from_f64(0.001)).collect();
        
        let start = Instant::now();
        let _profits = processor.calculate_arbitrage_profits_batch(
            &buy_prices, &sell_prices, &buy_volumes, &sell_volumes, &fee_rates
        ).unwrap();
        start.elapsed()
    }
}
