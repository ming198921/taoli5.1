#![allow(dead_code)]
//! # 英特尔 CPU 硬件级优化模块
//! 
//! 针对英特尔云服务器 CPU 的深度硬件优化，包括：
//! - CPU 亲和性绑定
//! - SIMD 指令集检测和优化
//! - 缓存行优化
//! - 分支预测优化
//! - 指令级并行(ILP)最大化

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Once;
use std::collections::HashMap;
use ordered_float::OrderedFloat;
use crate::types::OrderBookEntry;

/// CPU 特性检测结果
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub has_avx512f: bool,
    pub has_avx512bw: bool,
    pub has_avx512cd: bool,
    pub has_avx512dq: bool,
    pub has_avx512vl: bool,
    pub has_avx2: bool,
    pub has_fma: bool,
    pub has_bmi1: bool,
    pub has_bmi2: bool,
    pub cache_line_size: usize,
    pub l1_cache_size: usize,
    pub l2_cache_size: usize,
    pub l3_cache_size: usize,
    pub physical_cores: usize,
    pub logical_cores: usize,
}

/// CPU 性能配置
#[derive(Debug, Clone)]
pub struct CpuPerformanceConfig {
    pub enable_cpu_affinity: bool,
    pub dedicated_cores: Vec<usize>,
    pub disable_hyperthreading: bool,
    pub enable_performance_governor: bool,
    pub disable_frequency_scaling: bool,
    pub enable_turbo_boost: bool,
}

impl Default for CpuPerformanceConfig {
    fn default() -> Self {
        Self {
            enable_cpu_affinity: true,
            dedicated_cores: vec![0, 2, 4, 6], // 默认绑定到物理核心
            disable_hyperthreading: true,
            enable_performance_governor: true,
            disable_frequency_scaling: true,
            enable_turbo_boost: true,
        }
    }
}

/// 英特尔 CPU 硬件优化器
pub struct IntelCpuOptimizer {
    features: CpuFeatures,
    config: CpuPerformanceConfig,
    initialized: AtomicBool,
    performance_counters: PerformanceCounters,
}

/// 性能计数器
#[derive(Debug, Default)]
struct PerformanceCounters {
    simd_operations: AtomicU32,
    cache_misses: AtomicU32,
    branch_mispredictions: AtomicU32,
    instructions_retired: AtomicU32,
}

impl IntelCpuOptimizer {
    /// 创建新的 CPU 优化器实例
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let features = Self::detect_cpu_features()?;
        log::info!("Detected CPU features: {:?}", features);

        Ok(Self {
            features,
            config: CpuPerformanceConfig::default(),
            initialized: AtomicBool::new(false),
            performance_counters: PerformanceCounters::default(),
        })
    }

    /// 检测 CPU 特性
    fn detect_cpu_features() -> Result<CpuFeatures, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::__cpuid;
            
            // 检测基本 CPU 信息
            let cpuid_1 = unsafe { __cpuid(1) };
            let cpuid_7 = unsafe { __cpuid(7) };
            
            // AVX-512 特性检测
            let has_avx512f = (cpuid_7.ebx & (1 << 16)) != 0;
            let has_avx512bw = (cpuid_7.ebx & (1 << 30)) != 0;
            let has_avx512cd = (cpuid_7.ebx & (1 << 28)) != 0;
            let has_avx512dq = (cpuid_7.ebx & (1 << 17)) != 0;
            let has_avx512vl = (cpuid_7.ebx & (1 << 31)) != 0;
            
            // AVX2 和其他特性
            let has_avx2 = (cpuid_7.ebx & (1 << 5)) != 0;
            let has_fma = (cpuid_1.ecx & (1 << 12)) != 0;
            let has_bmi1 = (cpuid_7.ebx & (1 << 3)) != 0;
            let has_bmi2 = (cpuid_7.ebx & (1 << 8)) != 0;

            // 检测缓存信息
            let cache_info = Self::detect_cache_info();
            let core_info = Self::detect_core_info()?;

            Ok(CpuFeatures {
                has_avx512f,
                has_avx512bw,
                has_avx512cd,
                has_avx512dq,
                has_avx512vl,
                has_avx2,
                has_fma,
                has_bmi1,
                has_bmi2,
                cache_line_size: cache_info.0,
                l1_cache_size: cache_info.1,
                l2_cache_size: cache_info.2,
                l3_cache_size: cache_info.3,
                physical_cores: core_info.0,
                logical_cores: core_info.1,
            })
        }
        
        #[cfg(not(target_arch = "x86_64"))]
        {
            // 非 x86_64 架构的 fallback
            Ok(CpuFeatures {
                has_avx512f: false,
                has_avx512bw: false,
                has_avx512cd: false,
                has_avx512dq: false,
                has_avx512vl: false,
                has_avx2: false,
                has_fma: false,
                has_bmi1: false,
                has_bmi2: false,
                cache_line_size: 64,
                l1_cache_size: 32 * 1024,
                l2_cache_size: 256 * 1024,
                l3_cache_size: 8 * 1024 * 1024,
                physical_cores: 4,
                logical_cores: 8,
            })
        }
    }

    /// 检测缓存信息 (cache_line_size, l1_size, l2_size, l3_size)
    fn detect_cache_info() -> (usize, usize, usize, usize) {
        // 从 /proc/cpuinfo 或 sysfs 读取缓存信息
        let cache_line_size = Self::read_cache_line_size().unwrap_or(64);
        let l1_size = Self::read_cache_size("/sys/devices/system/cpu/cpu0/cache/index0/size").unwrap_or(32 * 1024);
        let l2_size = Self::read_cache_size("/sys/devices/system/cpu/cpu0/cache/index2/size").unwrap_or(256 * 1024);
        let l3_size = Self::read_cache_size("/sys/devices/system/cpu/cpu0/cache/index3/size").unwrap_or(8 * 1024 * 1024);
        
        (cache_line_size, l1_size, l2_size, l3_size)
    }

    /// 读取缓存行大小
    fn read_cache_line_size() -> Option<usize> {
        std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index0/coherency_line_size")
            .ok()?
            .trim()
            .parse()
            .ok()
    }

    /// 读取缓存大小
    fn read_cache_size(path: &str) -> Option<usize> {
        let content = std::fs::read_to_string(path).ok()?;
        let size_str = content.trim();
        
        // 解析类似 "32K", "256K", "8192K" 的格式
        if size_str.ends_with('K') {
            size_str[..size_str.len()-1].parse::<usize>().ok().map(|x| x * 1024)
        } else if size_str.ends_with('M') {
            size_str[..size_str.len()-1].parse::<usize>().ok().map(|x| x * 1024 * 1024)
        } else {
            size_str.parse().ok()
        }
    }

    /// 检测核心信息 (physical_cores, logical_cores)
    fn detect_core_info() -> Result<(usize, usize), Box<dyn std::error::Error + Send + Sync>> {
        let cpuinfo = std::fs::read_to_string("/proc/cpuinfo")?;
        
        let logical_cores;
        let mut core_ids = std::collections::HashSet::new();
        
        logical_cores = cpuinfo.lines().filter(|line| line.starts_with("processor")).count();
        
        for line in cpuinfo.lines() {
            if line.starts_with("core id") {
                if let Some(core_id) = line.split(':').nth(1) {
                    if let Ok(id) = core_id.trim().parse::<usize>() {
                        core_ids.insert(id);
                    }
                }
            }
        }
        
        let physical_cores = core_ids.len().max(1); // Ensure at least 1 core
        let physical_cores = if physical_cores == 0 { logical_cores } else { physical_cores };
        
        Ok((physical_cores, logical_cores))
    }

    /// 初始化 CPU 优化设置
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.initialized.load(Ordering::Relaxed) {
            return Ok(());
        }

        log::info!("Initializing Intel CPU optimizations...");

        // 设置 CPU 亲和性
        if self.config.enable_cpu_affinity {
            self.set_cpu_affinity()?;
        }

        // 设置性能调节器
        if self.config.enable_performance_governor {
            self.set_performance_governor()?;
        }

        // 禁用频率缩放
        if self.config.disable_frequency_scaling {
            self.disable_frequency_scaling()?;
        }

        // 启用 Turbo Boost
        if self.config.enable_turbo_boost {
            self.enable_turbo_boost()?;
        }

        self.initialized.store(true, Ordering::Relaxed);
        log::info!("Intel CPU optimizations initialized successfully");

        Ok(())
    }

    /// 设置 CPU 亲和性
    fn set_cpu_affinity(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            
            let current_pid = std::process::id();
            let core_mask = self.calculate_core_mask();
            
            log::info!("Setting CPU affinity for PID {} to cores: {:?}", current_pid, self.config.dedicated_cores);
            
            let output = Command::new("taskset")
                .arg("-p")
                .arg(format!("{:x}", core_mask))
                .arg(current_pid.to_string())
                .output();

            match output {
                Ok(result) if result.status.success() => {
                    log::info!("CPU affinity set successfully");
                },
                Ok(result) => {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    log::warn!("Failed to set CPU affinity: {}", stderr);
                },
                Err(e) => {
                    log::warn!("taskset command not available: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 计算 CPU 核心掩码
    fn calculate_core_mask(&self) -> u64 {
        let mut mask = 0u64;
        for &core in &self.config.dedicated_cores {
            if core < 64 { // 最多支持 64 个核心
                mask |= 1u64 << core;
            }
        }
        mask
    }

    /// 设置性能调节器
    fn set_performance_governor(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            
            for core in 0..self.features.logical_cores {
                let governor_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_governor", core);
                
                if let Err(e) = fs::write(&governor_path, "performance") {
                    log::warn!("Failed to set performance governor for CPU {}: {}", core, e);
                } else {
                    log::debug!("Set performance governor for CPU {}", core);
                }
            }
        }

        Ok(())
    }

    /// 禁用频率缩放
    fn disable_frequency_scaling(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            
            // 设置最小和最大频率相同
            for core in 0..self.features.logical_cores {
                let max_freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_max_freq", core);
                let min_freq_path = format!("/sys/devices/system/cpu/cpu{}/cpufreq/scaling_min_freq", core);
                
                if let Ok(max_freq) = fs::read_to_string(&max_freq_path) {
                    let max_freq = max_freq.trim();
                    if let Err(e) = fs::write(&min_freq_path, max_freq) {
                        log::warn!("Failed to set minimum frequency for CPU {}: {}", core, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// 启用 Turbo Boost
    fn enable_turbo_boost(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            
            // Intel Turbo Boost 控制
            if let Err(e) = fs::write("/sys/devices/system/cpu/intel_pstate/no_turbo", "0") {
                log::warn!("Failed to enable Turbo Boost (intel_pstate): {}", e);
            } else {
                log::info!("Intel Turbo Boost enabled");
            }

            // 通用 Turbo 控制
            if let Err(e) = fs::write("/sys/devices/system/cpu/cpufreq/boost", "1") {
                log::warn!("Failed to enable CPU boost: {}", e);
            }
        }

        Ok(())
    }

    /// 获取 CPU 特性
    pub fn get_features(&self) -> &CpuFeatures {
        &self.features
    }

    /// 检查是否支持 AVX-512
    pub fn supports_avx512(&self) -> bool {
        self.features.has_avx512f && 
        self.features.has_avx512bw && 
        self.features.has_avx512dq && 
        self.features.has_avx512vl
    }

    /// 检查是否支持 AVX2
    pub fn supports_avx2(&self) -> bool {
        self.features.has_avx2
    }

    /// 获取推荐的并行度
    pub fn get_recommended_parallelism(&self) -> usize {
        if self.config.disable_hyperthreading {
            self.features.physical_cores
        } else {
            self.features.logical_cores
        }
    }

    /// 优化的内存预取函数
    #[inline(always)]
    pub fn prefetch_data<T>(data: *const T, hint: PrefetchHint) {
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::{_mm_prefetch, _MM_HINT_T0, _MM_HINT_T1, _MM_HINT_T2, _MM_HINT_NTA};
            
            unsafe {
                match hint {
                    PrefetchHint::Temporal0 => _mm_prefetch(data as *const i8, _MM_HINT_T0),
                    PrefetchHint::Temporal1 => _mm_prefetch(data as *const i8, _MM_HINT_T1),
                    PrefetchHint::Temporal2 => _mm_prefetch(data as *const i8, _MM_HINT_T2),
                    PrefetchHint::NonTemporal => _mm_prefetch(data as *const i8, _MM_HINT_NTA),
                }
            }
        }
    }

    /// 缓存友好的数据拷贝
    #[inline(always)]
    pub fn cache_friendly_copy<T: Copy>(&self, dst: &mut [T], src: &[T]) {
        debug_assert_eq!(dst.len(), src.len());
        
        let cache_line_items = self.features.cache_line_size / std::mem::size_of::<T>();
        
        // 按缓存行大小批量处理
        for chunk in dst.chunks_mut(cache_line_items).zip(src.chunks(cache_line_items)) {
            let (dst_chunk, src_chunk) = chunk;
            
            // 预取下一个缓存行
            if src_chunk.len() == cache_line_items {
                let next_src = unsafe { src_chunk.as_ptr().add(cache_line_items) };
                Self::prefetch_data(next_src, PrefetchHint::Temporal1);
            }
            
            // 执行拷贝
            dst_chunk.copy_from_slice(src_chunk);
        }
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> HashMap<String, u32> {
        let mut stats = HashMap::new();
        stats.insert("simd_operations".to_string(), 
                    self.performance_counters.simd_operations.load(Ordering::Relaxed));
        stats.insert("cache_misses".to_string(), 
                    self.performance_counters.cache_misses.load(Ordering::Relaxed));
        stats.insert("branch_mispredictions".to_string(), 
                    self.performance_counters.branch_mispredictions.load(Ordering::Relaxed));
        stats.insert("instructions_retired".to_string(), 
                    self.performance_counters.instructions_retired.load(Ordering::Relaxed));
        stats
    }

    /// 获取CPU周期数
    pub fn get_cpu_cycles(&self) -> u64 {
        // 使用RDTSC指令获取CPU周期数
        #[cfg(target_arch = "x86_64")]
        unsafe {
            std::arch::x86_64::_rdtsc()
        }
        #[cfg(not(target_arch = "x86_64"))]
        0
    }
    
    /// 应用CPU亲和性配置
    pub fn apply_cpu_affinity(&self, _config: &CpuAffinityConfig) {
        // 在实际实现中，这里会设置CPU亲和性
        log::info!("🚀 应用CPU亲和性配置");
    }
    
    /// 预热优化组件
    pub async fn warmup_optimizations(&self) {
        log::info!("🚀 预热英特尔CPU优化组件");
        // 执行一些预热操作
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}

/// 预取提示类型
#[derive(Debug, Clone, Copy)]
pub enum PrefetchHint {
    Temporal0,      // 预取到 L1 缓存
    Temporal1,      // 预取到 L2 缓存
    Temporal2,      // 预取到 L3 缓存
    NonTemporal,    // 不污染缓存层次
}

/// 全局 CPU 优化器实例
static mut GLOBAL_CPU_OPTIMIZER: Option<IntelCpuOptimizer> = None;
static CPU_OPTIMIZER_INIT: Once = Once::new();

/// 获取全局 CPU 优化器
pub fn get_cpu_optimizer() -> &'static IntelCpuOptimizer {
    unsafe {
        CPU_OPTIMIZER_INIT.call_once(|| {
            match IntelCpuOptimizer::new() {
                Ok(optimizer) => {
                    if let Err(e) = optimizer.initialize() {
                        log::warn!("Failed to initialize CPU optimizer: {}", e);
                    }
                    GLOBAL_CPU_OPTIMIZER = Some(optimizer);
                    log::info!("Global Intel CPU optimizer initialized");
                },
                Err(e) => {
                    log::error!("Failed to create CPU optimizer: {}", e);
                    panic!("Critical: Cannot initialize CPU optimizer");
                }
            }
        });
        
        GLOBAL_CPU_OPTIMIZER.as_ref().expect("Global instance not initialized")
    }
}

/// 指令级并行优化的数据处理函数
pub mod ilp_optimized {
    use super::*;

    /// 展开循环的验证函数，最大化指令级并行
    #[inline(always)]
    pub unsafe fn validate_entries_unrolled(entries: *const OrderBookEntry, count: usize) -> bool {
        let mut i = 0;
        
        // 一次处理 8 个条目，展开循环
        while i + 8 <= count {
            let e0 = *entries.add(i);
            let e1 = *entries.add(i + 1);
            let e2 = *entries.add(i + 2);
            let e3 = *entries.add(i + 3);
            let e4 = *entries.add(i + 4);
            let e5 = *entries.add(i + 5);
            let e6 = *entries.add(i + 6);
            let e7 = *entries.add(i + 7);
            
            // 并行验证 8 个条目，使用位运算避免分支
            let valid = (e0.price > OrderedFloat(0.0)) as u8 & 
                       (e1.price > OrderedFloat(0.0)) as u8 & 
                       (e2.price > OrderedFloat(0.0)) as u8 & 
                       (e3.price > OrderedFloat(0.0)) as u8 &
                       (e4.price > OrderedFloat(0.0)) as u8 & 
                       (e5.price > OrderedFloat(0.0)) as u8 & 
                       (e6.price > OrderedFloat(0.0)) as u8 & 
                       (e7.price > OrderedFloat(0.0)) as u8;
            
            if valid == 0 { 
                return false; 
            }
            
            i += 8;
        }
        
        // 处理剩余条目
        while i < count {
            let entry = *entries.add(i);
            if entry.price <= OrderedFloat(0.0) {
                return false;
            }
            i += 1;
        }
        
        true
    }

    /// 内联汇编优化的比较函数
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    pub unsafe fn ultra_fast_comparison(a: f64, b: f64) -> bool {
        let result: u8;
        std::arch::asm!(
            "vucomisd {}, {}",
            "setbe {}",
            in(xmm_reg) a,
            in(xmm_reg) b,
            out(reg_byte) result,
            options(pure, nomem, nostack)
        );
        result != 0
    }

    /// 非 x86_64 架构的 fallback
    #[cfg(not(target_arch = "x86_64"))]
    #[inline(always)]
    pub fn ultra_fast_comparison(a: f64, b: f64) -> bool {
        a <= b
    }
}

/// CPU 亲和性配置
#[derive(Debug, Clone)]
pub struct CpuAffinityConfig {
    pub enable_affinity: bool,
    pub dedicated_cores: Vec<usize>,
    pub isolation_cores: Vec<usize>,
}

impl CpuAffinityConfig {
    /// 为英特尔云服务器创建优化配置
    pub fn for_intel_cloud_server(cpu_count: usize) -> Self {
        let dedicated_cores: Vec<usize> = (0..cpu_count.min(8)).step_by(2).collect();
        let isolation_cores: Vec<usize> = (1..cpu_count.min(8)).step_by(2).collect();
        
        Self {
            enable_affinity: true,
            dedicated_cores,
            isolation_cores,
        }
    }
}

impl CpuFeatures {
    /// 检测 CPU 特性
    pub fn detect() -> Self {
        Self {
            has_avx512f: is_x86_feature_detected!("avx512f"),
            has_avx512bw: is_x86_feature_detected!("avx512bw"),
            has_avx512cd: is_x86_feature_detected!("avx512cd"),
            has_avx512dq: is_x86_feature_detected!("avx512dq"),
            has_avx512vl: is_x86_feature_detected!("avx512vl"),
            has_avx2: is_x86_feature_detected!("avx2"),
            has_fma: is_x86_feature_detected!("fma"),
            has_bmi1: is_x86_feature_detected!("bmi1"),
            has_bmi2: is_x86_feature_detected!("bmi2"),
            cache_line_size: 64,
            l1_cache_size: 32768,
            l2_cache_size: 262144,
            l3_cache_size: 8388608,  
            physical_cores: num_cpus::get_physical(),
            logical_cores: num_cpus::get(),
        }
    }
}
