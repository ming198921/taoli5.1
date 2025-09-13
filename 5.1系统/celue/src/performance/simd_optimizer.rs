use std::arch::x86_64::*;
use std::mem;
use anyhow::Result;
use packed_simd_2::*;

/// SIMD-optimized operations for high-frequency trading
pub struct SimdOptimizer {
    use_avx2: bool,
    use_avx512: bool,
}

impl SimdOptimizer {
    pub fn new() -> Self {
        Self {
            use_avx2: is_x86_feature_detected!("avx2"),
            use_avx512: is_x86_feature_detected!("avx512f"),
        }
    }

    /// Vectorized price difference calculation
    #[inline(always)]
    pub fn calculate_price_differences(&self, bids: &[f64], asks: &[f64]) -> Vec<f64> {
        if self.use_avx512 {
            unsafe { self.calculate_price_differences_avx512(bids, asks) }
        } else if self.use_avx2 {
            unsafe { self.calculate_price_differences_avx2(bids, asks) }
        } else {
            self.calculate_price_differences_scalar(bids, asks)
        }
    }

    #[target_feature(enable = "avx512f")]
    unsafe fn calculate_price_differences_avx512(&self, bids: &[f64], asks: &[f64]) -> Vec<f64> {
        let len = bids.len().min(asks.len());
        let mut result = vec![0.0; len];
        
        let chunks = len / 8;
        let remainder = len % 8;
        
        for i in 0..chunks {
            let idx = i * 8;
            let bid_vec = _mm512_loadu_pd(&bids[idx]);
            let ask_vec = _mm512_loadu_pd(&asks[idx]);
            let diff = _mm512_sub_pd(ask_vec, bid_vec);
            _mm512_storeu_pd(&mut result[idx], diff);
        }
        
        for i in (chunks * 8)..len {
            result[i] = asks[i] - bids[i];
        }
        
        result
    }

    #[target_feature(enable = "avx2")]
    unsafe fn calculate_price_differences_avx2(&self, bids: &[f64], asks: &[f64]) -> Vec<f64> {
        let len = bids.len().min(asks.len());
        let mut result = vec![0.0; len];
        
        let chunks = len / 4;
        let remainder = len % 4;
        
        for i in 0..chunks {
            let idx = i * 4;
            let bid_vec = _mm256_loadu_pd(&bids[idx]);
            let ask_vec = _mm256_loadu_pd(&asks[idx]);
            let diff = _mm256_sub_pd(ask_vec, bid_vec);
            _mm256_storeu_pd(&mut result[idx], diff);
        }
        
        for i in (chunks * 4)..len {
            result[i] = asks[i] - bids[i];
        }
        
        result
    }

    fn calculate_price_differences_scalar(&self, bids: &[f64], asks: &[f64]) -> Vec<f64> {
        bids.iter()
            .zip(asks.iter())
            .map(|(bid, ask)| ask - bid)
            .collect()
    }

    /// Vectorized profit calculation with fees
    #[inline(always)]
    pub fn calculate_profits_with_fees(
        &self,
        prices: &[f64],
        volumes: &[f64],
        fees: &[f64],
    ) -> Vec<f64> {
        if self.use_avx512 {
            unsafe { self.calculate_profits_avx512(prices, volumes, fees) }
        } else if self.use_avx2 {
            unsafe { self.calculate_profits_avx2(prices, volumes, fees) }
        } else {
            self.calculate_profits_scalar(prices, volumes, fees)
        }
    }

    #[target_feature(enable = "avx512f")]
    unsafe fn calculate_profits_avx512(
        &self,
        prices: &[f64],
        volumes: &[f64],
        fees: &[f64],
    ) -> Vec<f64> {
        let len = prices.len().min(volumes.len()).min(fees.len());
        let mut result = vec![0.0; len];
        
        let chunks = len / 8;
        
        for i in 0..chunks {
            let idx = i * 8;
            let price_vec = _mm512_loadu_pd(&prices[idx]);
            let volume_vec = _mm512_loadu_pd(&volumes[idx]);
            let fee_vec = _mm512_loadu_pd(&fees[idx]);
            
            let gross = _mm512_mul_pd(price_vec, volume_vec);
            let fee_amount = _mm512_mul_pd(gross, fee_vec);
            let net = _mm512_sub_pd(gross, fee_amount);
            
            _mm512_storeu_pd(&mut result[idx], net);
        }
        
        for i in (chunks * 8)..len {
            result[i] = prices[i] * volumes[i] * (1.0 - fees[i]);
        }
        
        result
    }

    #[target_feature(enable = "avx2")]
    unsafe fn calculate_profits_avx2(
        &self,
        prices: &[f64],
        volumes: &[f64],
        fees: &[f64],
    ) -> Vec<f64> {
        let len = prices.len().min(volumes.len()).min(fees.len());
        let mut result = vec![0.0; len];
        
        let chunks = len / 4;
        
        for i in 0..chunks {
            let idx = i * 4;
            let price_vec = _mm256_loadu_pd(&prices[idx]);
            let volume_vec = _mm256_loadu_pd(&volumes[idx]);
            let fee_vec = _mm256_loadu_pd(&fees[idx]);
            
            let gross = _mm256_mul_pd(price_vec, volume_vec);
            let fee_amount = _mm256_mul_pd(gross, fee_vec);
            let net = _mm256_sub_pd(gross, fee_amount);
            
            _mm256_storeu_pd(&mut result[idx], net);
        }
        
        for i in (chunks * 4)..len {
            result[i] = prices[i] * volumes[i] * (1.0 - fees[i]);
        }
        
        result
    }

    fn calculate_profits_scalar(
        &self,
        prices: &[f64],
        volumes: &[f64],
        fees: &[f64],
    ) -> Vec<f64> {
        prices.iter()
            .zip(volumes.iter())
            .zip(fees.iter())
            .map(|((p, v), f)| p * v * (1.0 - f))
            .collect()
    }

    /// Vectorized moving average calculation
    pub fn moving_average(&self, data: &[f64], window: usize) -> Vec<f64> {
        if data.len() < window {
            return vec![];
        }

        let mut result = vec![0.0; data.len() - window + 1];
        
        if self.use_avx2 {
            unsafe { self.moving_average_simd(data, window, &mut result) }
        } else {
            self.moving_average_scalar(data, window, &mut result)
        }
        
        result
    }

    #[target_feature(enable = "avx2")]
    unsafe fn moving_average_simd(&self, data: &[f64], window: usize, result: &mut [f64]) {
        let mut sum = 0.0;
        for i in 0..window {
            sum += data[i];
        }
        result[0] = sum / window as f64;
        
        for i in 1..result.len() {
            sum = sum - data[i - 1] + data[i + window - 1];
            result[i] = sum / window as f64;
        }
    }

    fn moving_average_scalar(&self, data: &[f64], window: usize, result: &mut [f64]) {
        for i in 0..result.len() {
            let sum: f64 = data[i..i + window].iter().sum();
            result[i] = sum / window as f64;
        }
    }

    /// Vectorized standard deviation
    pub fn std_deviation(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mean = self.mean(data);
        let variance = if self.use_avx2 {
            unsafe { self.variance_simd(data, mean) }
        } else {
            self.variance_scalar(data, mean)
        };
        
        variance.sqrt()
    }

    pub fn mean(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        if self.use_avx2 {
            unsafe { self.mean_simd(data) }
        } else {
            data.iter().sum::<f64>() / data.len() as f64
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn mean_simd(&self, data: &[f64]) -> f64 {
        let len = data.len();
        let chunks = len / 4;
        let remainder = len % 4;
        
        let mut sum_vec = _mm256_setzero_pd();
        
        for i in 0..chunks {
            let vec = _mm256_loadu_pd(&data[i * 4]);
            sum_vec = _mm256_add_pd(sum_vec, vec);
        }
        
        let mut sum_array = [0.0; 4];
        _mm256_storeu_pd(sum_array.as_mut_ptr(), sum_vec);
        let mut sum = sum_array.iter().sum::<f64>();
        
        for i in (chunks * 4)..len {
            sum += data[i];
        }
        
        sum / len as f64
    }

    #[target_feature(enable = "avx2")]
    unsafe fn variance_simd(&self, data: &[f64], mean: f64) -> f64 {
        let len = data.len();
        let chunks = len / 4;
        
        let mean_vec = _mm256_set1_pd(mean);
        let mut sum_sq_vec = _mm256_setzero_pd();
        
        for i in 0..chunks {
            let vec = _mm256_loadu_pd(&data[i * 4]);
            let diff = _mm256_sub_pd(vec, mean_vec);
            let sq = _mm256_mul_pd(diff, diff);
            sum_sq_vec = _mm256_add_pd(sum_sq_vec, sq);
        }
        
        let mut sum_sq_array = [0.0; 4];
        _mm256_storeu_pd(sum_sq_array.as_mut_ptr(), sum_sq_vec);
        let mut sum_sq = sum_sq_array.iter().sum::<f64>();
        
        for i in (chunks * 4)..len {
            let diff = data[i] - mean;
            sum_sq += diff * diff;
        }
        
        sum_sq / len as f64
    }

    fn variance_scalar(&self, data: &[f64], mean: f64) -> f64 {
        data.iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<f64>() / data.len() as f64
    }

    /// Vectorized correlation calculation
    pub fn correlation(&self, x: &[f64], y: &[f64]) -> f64 {
        let n = x.len().min(y.len());
        if n == 0 {
            return 0.0;
        }

        let mean_x = self.mean(&x[..n]);
        let mean_y = self.mean(&y[..n]);

        if self.use_avx2 {
            unsafe { self.correlation_simd(&x[..n], &y[..n], mean_x, mean_y) }
        } else {
            self.correlation_scalar(&x[..n], &y[..n], mean_x, mean_y)
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn correlation_simd(&self, x: &[f64], y: &[f64], mean_x: f64, mean_y: f64) -> f64 {
        let len = x.len();
        let chunks = len / 4;
        
        let mean_x_vec = _mm256_set1_pd(mean_x);
        let mean_y_vec = _mm256_set1_pd(mean_y);
        
        let mut cov_vec = _mm256_setzero_pd();
        let mut var_x_vec = _mm256_setzero_pd();
        let mut var_y_vec = _mm256_setzero_pd();
        
        for i in 0..chunks {
            let idx = i * 4;
            let x_vec = _mm256_loadu_pd(&x[idx]);
            let y_vec = _mm256_loadu_pd(&y[idx]);
            
            let dx = _mm256_sub_pd(x_vec, mean_x_vec);
            let dy = _mm256_sub_pd(y_vec, mean_y_vec);
            
            cov_vec = _mm256_add_pd(cov_vec, _mm256_mul_pd(dx, dy));
            var_x_vec = _mm256_add_pd(var_x_vec, _mm256_mul_pd(dx, dx));
            var_y_vec = _mm256_add_pd(var_y_vec, _mm256_mul_pd(dy, dy));
        }
        
        let mut cov_array = [0.0; 4];
        let mut var_x_array = [0.0; 4];
        let mut var_y_array = [0.0; 4];
        
        _mm256_storeu_pd(cov_array.as_mut_ptr(), cov_vec);
        _mm256_storeu_pd(var_x_array.as_mut_ptr(), var_x_vec);
        _mm256_storeu_pd(var_y_array.as_mut_ptr(), var_y_vec);
        
        let mut cov = cov_array.iter().sum::<f64>();
        let mut var_x = var_x_array.iter().sum::<f64>();
        let mut var_y = var_y_array.iter().sum::<f64>();
        
        for i in (chunks * 4)..len {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            cov += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
        }
        
        if var_x > 0.0 && var_y > 0.0 {
            cov / (var_x * var_y).sqrt()
        } else {
            0.0
        }
    }

    fn correlation_scalar(&self, x: &[f64], y: &[f64], mean_x: f64, mean_y: f64) -> f64 {
        let mut cov = 0.0;
        let mut var_x = 0.0;
        let mut var_y = 0.0;
        
        for i in 0..x.len() {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            cov += dx * dy;
            var_x += dx * dx;
            var_y += dy * dy;
        }
        
        if var_x > 0.0 && var_y > 0.0 {
            cov / (var_x * var_y).sqrt()
        } else {
            0.0
        }
    }

    /// Vectorized matrix multiplication for arbitrage calculations
    pub fn matrix_multiply(&self, a: &[f64], b: &[f64], m: usize, n: usize, k: usize) -> Vec<f64> {
        let mut c = vec![0.0; m * n];
        
        if self.use_avx2 {
            unsafe { self.matrix_multiply_simd(a, b, &mut c, m, n, k) }
        } else {
            self.matrix_multiply_scalar(a, b, &mut c, m, n, k)
        }
        
        c
    }

    #[target_feature(enable = "avx2")]
    unsafe fn matrix_multiply_simd(
        &self,
        a: &[f64],
        b: &[f64],
        c: &mut [f64],
        m: usize,
        n: usize,
        k: usize,
    ) {
        for i in 0..m {
            for j in 0..n {
                let mut sum_vec = _mm256_setzero_pd();
                let chunks = k / 4;
                
                for l in 0..chunks {
                    let idx = l * 4;
                    let a_vals = [
                        a[i * k + idx],
                        a[i * k + idx + 1],
                        a[i * k + idx + 2],
                        a[i * k + idx + 3],
                    ];
                    let b_vals = [
                        b[idx * n + j],
                        b[(idx + 1) * n + j],
                        b[(idx + 2) * n + j],
                        b[(idx + 3) * n + j],
                    ];
                    
                    let a_vec = _mm256_loadu_pd(a_vals.as_ptr());
                    let b_vec = _mm256_loadu_pd(b_vals.as_ptr());
                    let prod = _mm256_mul_pd(a_vec, b_vec);
                    sum_vec = _mm256_add_pd(sum_vec, prod);
                }
                
                let mut sum_array = [0.0; 4];
                _mm256_storeu_pd(sum_array.as_mut_ptr(), sum_vec);
                let mut sum = sum_array.iter().sum::<f64>();
                
                for l in (chunks * 4)..k {
                    sum += a[i * k + l] * b[l * n + j];
                }
                
                c[i * n + j] = sum;
            }
        }
    }

    fn matrix_multiply_scalar(
        &self,
        a: &[f64],
        b: &[f64],
        c: &mut [f64],
        m: usize,
        n: usize,
        k: usize,
    ) {
        for i in 0..m {
            for j in 0..n {
                let mut sum = 0.0;
                for l in 0..k {
                    sum += a[i * k + l] * b[l * n + j];
                }
                c[i * n + j] = sum;
            }
        }
    }

    /// Fast parallel sorting using SIMD
    pub fn fast_sort(&self, data: &mut [f64]) {
        if data.len() <= 32 {
            data.sort_by(|a, b| a.partial_cmp(b).unwrap());
            return;
        }

        if self.use_avx2 {
            self.bitonic_sort_simd(data);
        } else {
            data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        }
    }

    fn bitonic_sort_simd(&self, data: &mut [f64]) {
        let n = data.len();
        let power_of_2 = n.next_power_of_two();
        
        if n != power_of_2 {
            let mut padded = vec![f64::MAX; power_of_2];
            padded[..n].copy_from_slice(data);
            self.bitonic_sort_recursive(&mut padded, 0, power_of_2, true);
            data.copy_from_slice(&padded[..n]);
        } else {
            self.bitonic_sort_recursive(data, 0, n, true);
        }
    }

    fn bitonic_sort_recursive(&self, data: &mut [f64], low: usize, cnt: usize, dir: bool) {
        if cnt > 1 {
            let k = cnt / 2;
            self.bitonic_sort_recursive(data, low, k, true);
            self.bitonic_sort_recursive(data, low + k, k, false);
            self.bitonic_merge(data, low, cnt, dir);
        }
    }

    fn bitonic_merge(&self, data: &mut [f64], low: usize, cnt: usize, dir: bool) {
        if cnt > 1 {
            let k = cnt / 2;
            for i in low..(low + k) {
                self.compare_and_swap(data, i, i + k, dir);
            }
            self.bitonic_merge(data, low, k, dir);
            self.bitonic_merge(data, low + k, k, dir);
        }
    }

    fn compare_and_swap(&self, data: &mut [f64], i: usize, j: usize, dir: bool) {
        if dir == (data[i] > data[j]) {
            data.swap(i, j);
        }
    }
}

/// SIMD-optimized batch operations
pub struct BatchSimdProcessor {
    optimizer: SimdOptimizer,
}

impl BatchSimdProcessor {
    pub fn new() -> Self {
        Self {
            optimizer: SimdOptimizer::new(),
        }
    }

    /// Process multiple arbitrage opportunities in parallel
    pub fn process_opportunities_batch(
        &self,
        buy_prices: &[Vec<f64>],
        sell_prices: &[Vec<f64>],
        volumes: &[Vec<f64>],
        fees: &[Vec<f64>],
    ) -> Vec<Vec<f64>> {
        buy_prices
            .iter()
            .zip(sell_prices.iter())
            .zip(volumes.iter())
            .zip(fees.iter())
            .map(|(((buy, sell), vol), fee)| {
                let spreads = self.optimizer.calculate_price_differences(buy, sell);
                let profits = self.optimizer.calculate_profits_with_fees(&spreads, vol, fee);
                profits
            })
            .collect()
    }

    /// Fast correlation matrix computation
    pub fn correlation_matrix(&self, data: &[Vec<f64>]) -> Vec<Vec<f64>> {
        let n = data.len();
        let mut matrix = vec![vec![0.0; n]; n];
        
        for i in 0..n {
            matrix[i][i] = 1.0;
            for j in (i + 1)..n {
                let corr = self.optimizer.correlation(&data[i], &data[j]);
                matrix[i][j] = corr;
                matrix[j][i] = corr;
            }
        }
        
        matrix
    }

    /// Parallel risk calculation
    pub fn calculate_risk_metrics(
        &self,
        returns: &[Vec<f64>],
    ) -> Vec<(f64, f64, f64)> {
        returns
            .iter()
            .map(|ret| {
                let mean = self.optimizer.mean(ret);
                let std = self.optimizer.std_deviation(ret);
                let sharpe = if std > 0.0 {
                    mean / std * (252.0_f64).sqrt()
                } else {
                    0.0
                };
                (mean, std, sharpe)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_differences() {
        let optimizer = SimdOptimizer::new();
        let bids = vec![100.0, 101.0, 102.0, 103.0];
        let asks = vec![100.5, 101.5, 102.5, 103.5];
        
        let diffs = optimizer.calculate_price_differences(&bids, &asks);
        assert_eq!(diffs.len(), 4);
        for diff in diffs {
            assert!((diff - 0.5).abs() < 1e-10);
        }
    }

    #[test]
    fn test_profit_calculation() {
        let optimizer = SimdOptimizer::new();
        let prices = vec![100.0, 200.0, 300.0, 400.0];
        let volumes = vec![10.0, 20.0, 30.0, 40.0];
        let fees = vec![0.001, 0.001, 0.001, 0.001];
        
        let profits = optimizer.calculate_profits_with_fees(&prices, &volumes, &fees);
        assert_eq!(profits.len(), 4);
        assert!((profits[0] - 999.0).abs() < 1e-6);
    }

    #[test]
    fn test_correlation() {
        let optimizer = SimdOptimizer::new();
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let corr = optimizer.correlation(&x, &y);
        assert!((corr - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_moving_average() {
        let optimizer = SimdOptimizer::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let ma = optimizer.moving_average(&data, 3);
        
        assert_eq!(ma.len(), 4);
        assert!((ma[0] - 2.0).abs() < 1e-10);
        assert!((ma[1] - 3.0).abs() < 1e-10);
    }
}