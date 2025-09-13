//! 微秒级性能优化模块
//! 
//! 基于QingXi 5.1架构的SIMD、无锁、CPU亲和性优化
//! 目标：套利检测延迟 ≤ 10微秒

pub mod simd_fixed_point;
pub mod lockfree_structures;
pub mod cpu_affinity;
pub mod triangular_arbitrage;
pub mod market_analysis;

pub use simd_fixed_point::*;
pub use lockfree_structures::*;
pub use cpu_affinity::*;
pub use triangular_arbitrage::*;
pub use market_analysis::*;

/// 性能监控指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub detection_latency_ns: u64,
    pub simd_operations_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub opportunities_processed: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            detection_latency_ns: 0,
            simd_operations_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            opportunities_processed: 0,
        }
    }
} 
 
 
//! 
//! 基于QingXi 5.1架构的SIMD、无锁、CPU亲和性优化
//! 目标：套利检测延迟 ≤ 10微秒

pub mod simd_fixed_point;
pub mod lockfree_structures;
pub mod cpu_affinity;
pub mod triangular_arbitrage;
pub mod market_analysis;

pub use simd_fixed_point::*;
pub use lockfree_structures::*;
pub use cpu_affinity::*;
pub use triangular_arbitrage::*;
pub use market_analysis::*;

/// 性能监控指标
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub detection_latency_ns: u64,
    pub simd_operations_count: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub opportunities_processed: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            detection_latency_ns: 0,
            simd_operations_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            opportunities_processed: 0,
        }
    }
} 
 
 