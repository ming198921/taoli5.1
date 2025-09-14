#![allow(dead_code)]
//! # 性能优化配置
//!
//! 包含所有性能优化相关的配置参数

use serde::{Deserialize, Serialize};

/// 性能优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 数据清洗配置
    pub cleaning: CleaningConfig,
    /// SIMD配置
    pub simd: SIMDConfig,
    /// 内存池配置
    pub memory_pool: MemoryPoolConfig,
    /// 零拷贝配置
    pub zero_copy: ZeroCopyConfig,
    /// 基准测试配置
    pub benchmark: BenchmarkConfigSerde,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            cleaning: CleaningConfig::default(),
            simd: SIMDConfig::default(),
            memory_pool: MemoryPoolConfig::default(),
            zero_copy: ZeroCopyConfig::default(),
            benchmark: BenchmarkConfigSerde::default(),
        }
    }
}

/// 数据清洗配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningConfig {
    /// 是否启用优化清洗器
    pub enable_optimized_cleaner: bool,
    /// 是否启用SIMD优化
    pub enable_simd: bool,
    /// 是否启用内存池
    pub enable_memory_pool: bool,
    /// 是否启用零拷贝
    pub enable_zero_copy: bool,
    /// 清洗超时时间（毫秒）
    pub cleaning_timeout_ms: u64,
    /// 性能报告间隔（秒）
    pub performance_report_interval_secs: u64,
}

impl Default for CleaningConfig {
    fn default() -> Self {
        let cleaning_timeout_ms = if let Ok(settings) = crate::settings::Settings::load() {
            settings.cleaner.target_latency_ns / 1_000_000  // 将纳秒转换为毫秒
        } else {
            100u64
        };
        
        Self {
            enable_optimized_cleaner: true,
            enable_simd: true,
            enable_memory_pool: true,
            enable_zero_copy: true,
            cleaning_timeout_ms,
            performance_report_interval_secs: 30,
        }
    }
}

/// SIMD配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SIMDConfig {
    /// 是否启用AVX2指令集
    pub enable_avx2: bool,
    /// 是否启用SSE4.2指令集
    pub enable_sse42: bool,
    /// SIMD批处理大小
    pub batch_size: usize,
    /// 价格缓冲区大小
    pub price_buffer_size: usize,
    /// 数量缓冲区大小
    pub quantity_buffer_size: usize,
}

impl Default for SIMDConfig {
    fn default() -> Self {
        let (batch_size, price_buffer_size, quantity_buffer_size) = 
            if let Ok(settings) = crate::settings::Settings::load() {
                (
                    settings.batch.max_batch_size / 4,  // SIMD批处理通常是主批处理的1/4
                    settings.memory_allocator.size_threshold,
                    settings.memory_allocator.size_threshold
                )
            } else {
                (256, 1024, 1024)
            };
        
        Self {
            enable_avx2: true,
            enable_sse42: true,
            batch_size,
            price_buffer_size,
            quantity_buffer_size,
        }
    }
}

/// 内存池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolConfig {
    /// 订单簿条目池大小
    pub orderbook_entry_pool_size: usize,
    /// 交易更新池大小
    pub trade_update_pool_size: usize,
    /// 快照池大小
    pub snapshot_pool_size: usize,
    /// f64缓冲区池大小
    pub f64_buffer_pool_size: usize,
    /// usize缓冲区池大小
    pub usize_buffer_pool_size: usize,
    /// 默认向量容量
    pub default_vec_capacity: usize,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        // 从设置加载，如果失败则使用默认值
        if let Ok(settings) = crate::settings::Settings::load() {
            Self {
                orderbook_entry_pool_size: settings.memory_pools.orderbook_entry_pool_size,
                trade_update_pool_size: settings.memory_pools.trade_update_pool_size,
                snapshot_pool_size: settings.memory_pools.snapshot_pool_size,
                f64_buffer_pool_size: settings.memory_pools.cleaner_buffer_size,
                usize_buffer_pool_size: settings.memory_pools.cleaner_buffer_size,
                default_vec_capacity: settings.memory_pools.default_vec_capacity,
            }
        } else {
            Self {
                orderbook_entry_pool_size: 1000,
                trade_update_pool_size: 1000,
                snapshot_pool_size: 500,
                f64_buffer_pool_size: 1000,
                usize_buffer_pool_size: 1000,
                default_vec_capacity: 100,
            }
        }
    }
}

/// 零拷贝配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroCopyConfig {
    /// 是否启用原地排序
    pub enable_in_place_sorting: bool,
    /// 是否启用原地过滤
    pub enable_in_place_filtering: bool,
    /// 临时索引缓冲区大小
    pub temp_indices_capacity: usize,
    /// 比较缓冲区大小
    pub comparison_buffer_capacity: usize,
}

impl Default for ZeroCopyConfig {
    fn default() -> Self {
        // 从设置加载，如果失败则使用默认值
        if let Ok(settings) = crate::settings::Settings::load() {
            Self {
                enable_in_place_sorting: true,
                enable_in_place_filtering: true,
                temp_indices_capacity: settings.memory_pools.cleaner_buffer_size,
                comparison_buffer_capacity: settings.memory_pools.cleaner_buffer_size,
            }
        } else {
            Self {
                enable_in_place_sorting: true,
                enable_in_place_filtering: true,
                temp_indices_capacity: 1000,
                comparison_buffer_capacity: 1000,
            }
        }
    }
}

/// 基准测试配置（可序列化版本）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfigSerde {
    /// 迭代次数
    pub iterations: usize,
    /// 订单簿深度
    pub orderbook_depth: usize,
    /// 交易数量
    pub trade_count: usize,
    /// 预热迭代次数
    pub warmup_iterations: usize,
    /// 是否启用详细输出
    pub verbose_output: bool,
}

impl Default for BenchmarkConfigSerde {
    fn default() -> Self {
        let (iterations, orderbook_depth, trade_count, warmup_iterations, verbose_output) = 
            if let Ok(settings) = crate::settings::Settings::load() {
                (
                    settings.benchmark.iterations,
                    settings.benchmark.orderbook_depth,
                    settings.benchmark.trade_count,
                    settings.benchmark.warmup_iterations,
                    settings.benchmark.verbose_output
                )
            } else {
                (1000, 20, 50, 100, false)
            };
            
        Self {
            iterations,
            orderbook_depth,
            trade_count,
            warmup_iterations,
            verbose_output,
        }
    }
}

/// 性能优化策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    /// 最大吞吐量
    MaxThroughput,
    /// 最低延迟
    LowLatency,
    /// 平衡性能
    Balanced,
    /// 内存优化
    MemoryOptimized,
    /// 自定义
    Custom(CustomOptimization),
}

/// 自定义优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomOptimization {
    /// SIMD权重 (0.0-1.0)
    pub simd_weight: f64,
    /// 内存池权重 (0.0-1.0)
    pub memory_pool_weight: f64,
    /// 零拷贝权重 (0.0-1.0)
    pub zero_copy_weight: f64,
    /// 批处理权重 (0.0-1.0)
    pub batch_weight: f64,
}

impl PerformanceConfig {
    /// 根据策略调整配置
    pub fn apply_strategy(&mut self, strategy: OptimizationStrategy) {
        // 获取配置化的基准值
        let base_pool_size = if let Ok(settings) = crate::settings::Settings::load() {
            settings.memory_pools.orderbook_entry_pool_size
        } else {
            1000
        };
        
        let base_buffer_size = if let Ok(settings) = crate::settings::Settings::load() {
            settings.memory_pools.cleaner_buffer_size
        } else {
            1000
        };

        match strategy {
            OptimizationStrategy::MaxThroughput => {
                self.cleaning.enable_simd = true;
                self.cleaning.enable_memory_pool = true;
                self.cleaning.enable_zero_copy = true;
                self.simd.batch_size = base_buffer_size / 2; // 动态计算
                self.memory_pool.orderbook_entry_pool_size = base_pool_size * 2;
            },
            OptimizationStrategy::LowLatency => {
                self.cleaning.enable_zero_copy = true;
                self.cleaning.cleaning_timeout_ms = 10;
                self.zero_copy.enable_in_place_sorting = true;
                self.zero_copy.enable_in_place_filtering = true;
                self.simd.batch_size = base_buffer_size / 8; // 小批次低延迟
            },
            OptimizationStrategy::Balanced => {
                // 使用默认配置
            },
            OptimizationStrategy::MemoryOptimized => {
                self.cleaning.enable_memory_pool = true;
                self.memory_pool.orderbook_entry_pool_size = base_pool_size * 5;
                self.memory_pool.trade_update_pool_size = base_pool_size * 5;
                self.memory_pool.default_vec_capacity = base_buffer_size / 5; // 动态计算容量
            },
            OptimizationStrategy::Custom(custom) => {
                self.cleaning.enable_simd = custom.simd_weight > 0.5;
                self.cleaning.enable_memory_pool = custom.memory_pool_weight > 0.5;
                self.cleaning.enable_zero_copy = custom.zero_copy_weight > 0.5;
                
                if custom.simd_weight > 0.8 {
                    self.simd.batch_size = 512;
                } else if custom.simd_weight > 0.5 {
                    self.simd.batch_size = 256;
                } else {
                    self.simd.batch_size = 128;
                }
            },
        }
    }
    
    /// 验证配置有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.cleaning.cleaning_timeout_ms == 0 {
            return Err("清洗超时时间不能为0".to_string());
        }
        
        if self.simd.batch_size == 0 {
            return Err("SIMD批处理大小不能为0".to_string());
        }
        
        if self.memory_pool.orderbook_entry_pool_size == 0 {
            return Err("订单簿条目池大小不能为0".to_string());
        }
        
        Ok(())
    }
    
    /// 从文件加载配置
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: PerformanceConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    /// 保存配置到文件
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        self.validate()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
