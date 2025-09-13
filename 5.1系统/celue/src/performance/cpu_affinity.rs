 
//! 
//! 绑定关键线程到专用CPU核心，避免上下文切换，目标延迟 ≤ 1微秒

use core_affinity::{CoreId, get_core_ids, set_for_current};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// CPU核心类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreType {
    /// 套利检测专用核心
    ArbitrageDetection,
    /// 市场数据处理专用核心
    MarketDataProcessing,
    /// NATS通信专用核心
    Communication,
    /// 系统监控专用核心
    Monitoring,
    /// 通用核心
    General,
}

/// CPU亲和性管理器
pub struct CpuAffinityManager {
    core_assignments: Arc<Mutex<HashMap<CoreType, Vec<CoreId>>>>,
    next_core_index: Arc<Mutex<HashMap<CoreType, AtomicUsize>>>,
    available_cores: Vec<CoreId>,
    is_initialized: AtomicBool,
}

impl CpuAffinityManager {
    pub fn new() -> Self {
        let available_cores = get_core_ids().unwrap_or_default();
        
        Self {
            core_assignments: Arc::new(Mutex::new(HashMap::new())),
            next_core_index: Arc::new(Mutex::new(HashMap::new())),
            available_cores,
            is_initialized: AtomicBool::new(false),
        }
    }
    
    /// 初始化CPU核心分配策略
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_initialized.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        let total_cores = self.available_cores.len();
        if total_cores < 4 {
            return Err("需要至少4个CPU核心来进行性能优化".into());
        }
        
        let mut assignments = self.core_assignments.lock().unwrap();
        let mut indices = self.next_core_index.lock().unwrap();
        
        // 核心分配策略：
        // - 前25%核心用于套利检测（最关键）
        // - 接下来25%用于市场数据处理
        // - 接下来25%用于通信
        // - 剩余25%用于监控和通用任务
        
        let detection_cores = total_cores / 4;
        let data_cores = total_cores / 4;
        let comm_cores = total_cores / 4;
        let monitor_cores = total_cores - detection_cores - data_cores - comm_cores;
        
        assignments.insert(
            CoreType::ArbitrageDetection,
            self.available_cores[0..detection_cores].to_vec(),
        );
        
        assignments.insert(
            CoreType::MarketDataProcessing,
            self.available_cores[detection_cores..detection_cores + data_cores].to_vec(),
        );
        
        assignments.insert(
            CoreType::Communication,
            self.available_cores[detection_cores + data_cores..detection_cores + data_cores + comm_cores].to_vec(),
        );
        
        assignments.insert(
            CoreType::Monitoring,
            self.available_cores[detection_cores + data_cores + comm_cores..].to_vec(),
        );
        
        assignments.insert(
            CoreType::General,
            self.available_cores.clone(), // 通用任务可以使用所有核心
        );
        
        // 初始化索引计数器
        for core_type in [
            CoreType::ArbitrageDetection,
            CoreType::MarketDataProcessing,
            CoreType::Communication,
            CoreType::Monitoring,
            CoreType::General,
        ] {
            indices.insert(core_type, AtomicUsize::new(0));
        }
        
        self.is_initialized.store(true, Ordering::Relaxed);
        
        tracing::info!(
            "🎯 CPU亲和性管理器初始化完成 - 总核心数: {}, 检测核心: {}, 数据核心: {}, 通信核心: {}, 监控核心: {}",
            total_cores,
            detection_cores,
            data_cores,
            comm_cores,
            monitor_cores
        );
        
        Ok(())
    }
    
    /// 为当前线程设置CPU亲和性
    pub fn bind_current_thread(&self, core_type: CoreType) -> Result<CoreId, Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_initialized.load(Ordering::Relaxed) {
            self.initialize()?;
        }
        
        let core_id = self.get_next_core(core_type)?;
        
        if !set_for_current(core_id) {
            return Err(format!("无法绑定线程到核心 {:?}", core_id).into());
        }
        
        tracing::debug!(
            "🔗 线程绑定到CPU核心 {:?} (类型: {:?})",
            core_id,
            core_type
        );
        
        Ok(core_id)
    }
    
    /// 获取指定类型的下一个可用核心
    fn get_next_core(&self, core_type: CoreType) -> Result<CoreId, Box<dyn std::error::Error + Send + Sync>> {
        let assignments = self.core_assignments.lock().unwrap();
        let indices = self.next_core_index.lock().unwrap();
        
        let cores = assignments.get(&core_type)
            .ok_or_else(|| format!("未找到核心类型 {:?} 的分配", core_type))?;
        
        if cores.is_empty() {
            return Err(format!("核心类型 {:?} 没有可用核心", core_type).into());
        }
        
        let index_counter = indices.get(&core_type)
            .ok_or_else(|| format!("未找到核心类型 {:?} 的索引计数器", core_type))?;
        
        let current_index = index_counter.fetch_add(1, Ordering::Relaxed);
        let core_index = current_index % cores.len();
        
        Ok(cores[core_index])
    }
    
    /// 获取核心分配统计信息
    pub fn get_assignment_stats(&self) -> HashMap<CoreType, usize> {
        let assignments = self.core_assignments.lock().unwrap();
        let mut stats = HashMap::new();
        
        for (core_type, cores) in assignments.iter() {
            stats.insert(*core_type, cores.len());
        }
        
        stats
    }
    
    /// 设置高优先级实时线程属性
    pub fn set_high_priority(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 在Linux上设置SCHED_FIFO调度策略
        #[cfg(target_os = "linux")]
        {
            use libc::{sched_param, sched_setscheduler, SCHED_FIFO};
            
            let param = sched_param {
                sched_priority: 99, // 最高优先级
            };
            
            let result = unsafe {
                sched_setscheduler(0, SCHED_FIFO, &param)
            };
            
            if result != 0 {
                return Err("无法设置SCHED_FIFO调度策略，可能需要sudo权限".into());
            }
            
            tracing::info!("⚡ 线程设置为高优先级实时调度 (SCHED_FIFO)");
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            tracing::warn!("⚠️  当前平台不支持SCHED_FIFO调度策略");
        }
        
        Ok(())
    }
    
    /// 创建绑定到特定核心类型的线程
    pub fn spawn_bound_thread<F, T>(
        &self,
        core_type: CoreType,
        name: String,
        f: F,
    ) -> Result<thread::JoinHandle<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let manager = Arc::new(self.clone());
        
        let handle = thread::Builder::new()
            .name(name.clone())
            .spawn(move || {
                // 绑定到指定核心类型
                if let Err(e) = manager.bind_current_thread(core_type) {
                    tracing::error!("❌ 无法绑定线程 {} 到核心类型 {:?}: {}", name, core_type, e);
                }
                
                // 尝试设置高优先级
                if matches!(core_type, CoreType::ArbitrageDetection | CoreType::MarketDataProcessing) {
                    if let Err(e) = manager.set_high_priority() {
                        tracing::warn!("⚠️  无法设置线程 {} 为高优先级: {}", name, e);
                    }
                }
                
                f()
            })?;
        
        Ok(handle)
    }
}

impl Clone for CpuAffinityManager {
    fn clone(&self) -> Self {
        Self {
            core_assignments: Arc::clone(&self.core_assignments),
            next_core_index: Arc::clone(&self.next_core_index),
            available_cores: self.available_cores.clone(),
            is_initialized: AtomicBool::new(self.is_initialized.load(Ordering::Relaxed)),
        }
    }
}

/// 专用的套利检测线程管理器
pub struct ArbitrageDetectionThread {
    manager: CpuAffinityManager,
    is_running: Arc<AtomicBool>,
}

impl ArbitrageDetectionThread {
    pub fn new() -> Self {
        Self {
            manager: CpuAffinityManager::new(),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// 启动套利检测线程
    pub fn start<F>(&self, detection_fn: F) -> Result<thread::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + 'static,
    {
        if self.is_running.load(Ordering::Relaxed) {
            return Err("套利检测线程已经在运行".into());
        }
        
        self.manager.initialize()?;
        let running = Arc::clone(&self.is_running);
        
        let handle = self.manager.spawn_bound_thread(
            CoreType::ArbitrageDetection,
            "arbitrage-detection".to_string(),
            move || {
                running.store(true, Ordering::Relaxed);
                tracing::info!("🎯 套利检测线程启动，绑定到专用CPU核心");
                
                while running.load(Ordering::Relaxed) {
                    if let Err(e) = detection_fn() {
                        tracing::error!("❌ 套利检测错误: {}", e);
                        // 短暂休眠避免忙循环
                        std::thread::sleep(std::time::Duration::from_nanos(100));
                    }
                }
                
                tracing::info!("🛑 套利检测线程结束");
            },
        )?;
        
        Ok(handle)
    }
    
    /// 停止套利检测线程
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

/// 获取CPU特性信息
pub fn get_cpu_features() -> CpuFeatures {
    CpuFeatures {
        avx512f: std::is_x86_feature_detected!("avx512f"),
        avx512dq: std::is_x86_feature_detected!("avx512dq"),
        avx512bw: std::is_x86_feature_detected!("avx512bw"),
        avx512vl: std::is_x86_feature_detected!("avx512vl"),
        avx2: std::is_x86_feature_detected!("avx2"),
        fma: std::is_x86_feature_detected!("fma"),
        total_cores: num_cpus::get(),
        physical_cores: num_cpus::get_physical(),
    }
}

/// CPU特性信息
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub avx512f: bool,
    pub avx512dq: bool,
    pub avx512bw: bool,
    pub avx512vl: bool,
    pub avx2: bool,
    pub fma: bool,
    pub total_cores: usize,
    pub physical_cores: usize,
}

impl CpuFeatures {
    /// 检查是否支持完整的AVX-512
    pub fn supports_full_avx512(&self) -> bool {
        self.avx512f && self.avx512dq && self.avx512bw && self.avx512vl
    }
    
    /// 获取推荐的向量化策略
    pub fn get_recommended_simd_strategy(&self) -> &'static str {
        if self.supports_full_avx512() {
            "AVX-512 (8x64bit parallel)"
        } else if self.avx2 {
            "AVX2 (4x64bit parallel)"
        } else {
            "Scalar (single 64bit)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_affinity_manager() {
        let manager = CpuAffinityManager::new();
        
        // 测试初始化
        assert!(manager.initialize().is_ok());
        
        // 测试获取统计信息
        let stats = manager.get_assignment_stats();
        assert!(stats.contains_key(&CoreType::ArbitrageDetection));
        assert!(stats.contains_key(&CoreType::MarketDataProcessing));
    }
    
    #[test]
    fn test_cpu_features() {
        let features = get_cpu_features();
        
        // 基本检查
        assert!(features.total_cores > 0);
        assert!(features.physical_cores > 0);
        
        println!("CPU特性: {:?}", features);
        println!("推荐SIMD策略: {}", features.get_recommended_simd_strategy());
    }
    
    #[test]
    fn test_arbitrage_detection_thread() {
        let detection_thread = ArbitrageDetectionThread::new();
        
        let counter = Arc::new(AtomicUsize::new(0));
        let test_counter = Arc::clone(&counter);
        
        let handle = detection_thread.start(move || {
            let count = test_counter.fetch_add(1, Ordering::Relaxed);
            if count >= 3 {
                return Ok(()); // 停止测试
            }
            
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(())
        });
        
        if let Ok(handle) = handle {
            std::thread::sleep(std::time::Duration::from_millis(50));
            detection_thread.stop();
            let _ = handle.join();
            
            let final_count = counter.load(Ordering::Relaxed);
            assert!(final_count > 0);
        }
    }
} 
 
//! 
//! 绑定关键线程到专用CPU核心，避免上下文切换，目标延迟 ≤ 1微秒

use core_affinity::{CoreId, get_core_ids, set_for_current};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// CPU核心类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoreType {
    /// 套利检测专用核心
    ArbitrageDetection,
    /// 市场数据处理专用核心
    MarketDataProcessing,
    /// NATS通信专用核心
    Communication,
    /// 系统监控专用核心
    Monitoring,
    /// 通用核心
    General,
}

/// CPU亲和性管理器
pub struct CpuAffinityManager {
    core_assignments: Arc<Mutex<HashMap<CoreType, Vec<CoreId>>>>,
    next_core_index: Arc<Mutex<HashMap<CoreType, AtomicUsize>>>,
    available_cores: Vec<CoreId>,
    is_initialized: AtomicBool,
}

impl CpuAffinityManager {
    pub fn new() -> Self {
        let available_cores = get_core_ids().unwrap_or_default();
        
        Self {
            core_assignments: Arc::new(Mutex::new(HashMap::new())),
            next_core_index: Arc::new(Mutex::new(HashMap::new())),
            available_cores,
            is_initialized: AtomicBool::new(false),
        }
    }
    
    /// 初始化CPU核心分配策略
    pub fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_initialized.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        let total_cores = self.available_cores.len();
        if total_cores < 4 {
            return Err("需要至少4个CPU核心来进行性能优化".into());
        }
        
        let mut assignments = self.core_assignments.lock().unwrap();
        let mut indices = self.next_core_index.lock().unwrap();
        
        // 核心分配策略：
        // - 前25%核心用于套利检测（最关键）
        // - 接下来25%用于市场数据处理
        // - 接下来25%用于通信
        // - 剩余25%用于监控和通用任务
        
        let detection_cores = total_cores / 4;
        let data_cores = total_cores / 4;
        let comm_cores = total_cores / 4;
        let monitor_cores = total_cores - detection_cores - data_cores - comm_cores;
        
        assignments.insert(
            CoreType::ArbitrageDetection,
            self.available_cores[0..detection_cores].to_vec(),
        );
        
        assignments.insert(
            CoreType::MarketDataProcessing,
            self.available_cores[detection_cores..detection_cores + data_cores].to_vec(),
        );
        
        assignments.insert(
            CoreType::Communication,
            self.available_cores[detection_cores + data_cores..detection_cores + data_cores + comm_cores].to_vec(),
        );
        
        assignments.insert(
            CoreType::Monitoring,
            self.available_cores[detection_cores + data_cores + comm_cores..].to_vec(),
        );
        
        assignments.insert(
            CoreType::General,
            self.available_cores.clone(), // 通用任务可以使用所有核心
        );
        
        // 初始化索引计数器
        for core_type in [
            CoreType::ArbitrageDetection,
            CoreType::MarketDataProcessing,
            CoreType::Communication,
            CoreType::Monitoring,
            CoreType::General,
        ] {
            indices.insert(core_type, AtomicUsize::new(0));
        }
        
        self.is_initialized.store(true, Ordering::Relaxed);
        
        tracing::info!(
            "🎯 CPU亲和性管理器初始化完成 - 总核心数: {}, 检测核心: {}, 数据核心: {}, 通信核心: {}, 监控核心: {}",
            total_cores,
            detection_cores,
            data_cores,
            comm_cores,
            monitor_cores
        );
        
        Ok(())
    }
    
    /// 为当前线程设置CPU亲和性
    pub fn bind_current_thread(&self, core_type: CoreType) -> Result<CoreId, Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_initialized.load(Ordering::Relaxed) {
            self.initialize()?;
        }
        
        let core_id = self.get_next_core(core_type)?;
        
        if !set_for_current(core_id) {
            return Err(format!("无法绑定线程到核心 {:?}", core_id).into());
        }
        
        tracing::debug!(
            "🔗 线程绑定到CPU核心 {:?} (类型: {:?})",
            core_id,
            core_type
        );
        
        Ok(core_id)
    }
    
    /// 获取指定类型的下一个可用核心
    fn get_next_core(&self, core_type: CoreType) -> Result<CoreId, Box<dyn std::error::Error + Send + Sync>> {
        let assignments = self.core_assignments.lock().unwrap();
        let indices = self.next_core_index.lock().unwrap();
        
        let cores = assignments.get(&core_type)
            .ok_or_else(|| format!("未找到核心类型 {:?} 的分配", core_type))?;
        
        if cores.is_empty() {
            return Err(format!("核心类型 {:?} 没有可用核心", core_type).into());
        }
        
        let index_counter = indices.get(&core_type)
            .ok_or_else(|| format!("未找到核心类型 {:?} 的索引计数器", core_type))?;
        
        let current_index = index_counter.fetch_add(1, Ordering::Relaxed);
        let core_index = current_index % cores.len();
        
        Ok(cores[core_index])
    }
    
    /// 获取核心分配统计信息
    pub fn get_assignment_stats(&self) -> HashMap<CoreType, usize> {
        let assignments = self.core_assignments.lock().unwrap();
        let mut stats = HashMap::new();
        
        for (core_type, cores) in assignments.iter() {
            stats.insert(*core_type, cores.len());
        }
        
        stats
    }
    
    /// 设置高优先级实时线程属性
    pub fn set_high_priority(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 在Linux上设置SCHED_FIFO调度策略
        #[cfg(target_os = "linux")]
        {
            use libc::{sched_param, sched_setscheduler, SCHED_FIFO};
            
            let param = sched_param {
                sched_priority: 99, // 最高优先级
            };
            
            let result = unsafe {
                sched_setscheduler(0, SCHED_FIFO, &param)
            };
            
            if result != 0 {
                return Err("无法设置SCHED_FIFO调度策略，可能需要sudo权限".into());
            }
            
            tracing::info!("⚡ 线程设置为高优先级实时调度 (SCHED_FIFO)");
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            tracing::warn!("⚠️  当前平台不支持SCHED_FIFO调度策略");
        }
        
        Ok(())
    }
    
    /// 创建绑定到特定核心类型的线程
    pub fn spawn_bound_thread<F, T>(
        &self,
        core_type: CoreType,
        name: String,
        f: F,
    ) -> Result<thread::JoinHandle<T>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let manager = Arc::new(self.clone());
        
        let handle = thread::Builder::new()
            .name(name.clone())
            .spawn(move || {
                // 绑定到指定核心类型
                if let Err(e) = manager.bind_current_thread(core_type) {
                    tracing::error!("❌ 无法绑定线程 {} 到核心类型 {:?}: {}", name, core_type, e);
                }
                
                // 尝试设置高优先级
                if matches!(core_type, CoreType::ArbitrageDetection | CoreType::MarketDataProcessing) {
                    if let Err(e) = manager.set_high_priority() {
                        tracing::warn!("⚠️  无法设置线程 {} 为高优先级: {}", name, e);
                    }
                }
                
                f()
            })?;
        
        Ok(handle)
    }
}

impl Clone for CpuAffinityManager {
    fn clone(&self) -> Self {
        Self {
            core_assignments: Arc::clone(&self.core_assignments),
            next_core_index: Arc::clone(&self.next_core_index),
            available_cores: self.available_cores.clone(),
            is_initialized: AtomicBool::new(self.is_initialized.load(Ordering::Relaxed)),
        }
    }
}

/// 专用的套利检测线程管理器
pub struct ArbitrageDetectionThread {
    manager: CpuAffinityManager,
    is_running: Arc<AtomicBool>,
}

impl ArbitrageDetectionThread {
    pub fn new() -> Self {
        Self {
            manager: CpuAffinityManager::new(),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// 启动套利检测线程
    pub fn start<F>(&self, detection_fn: F) -> Result<thread::JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + 'static,
    {
        if self.is_running.load(Ordering::Relaxed) {
            return Err("套利检测线程已经在运行".into());
        }
        
        self.manager.initialize()?;
        let running = Arc::clone(&self.is_running);
        
        let handle = self.manager.spawn_bound_thread(
            CoreType::ArbitrageDetection,
            "arbitrage-detection".to_string(),
            move || {
                running.store(true, Ordering::Relaxed);
                tracing::info!("🎯 套利检测线程启动，绑定到专用CPU核心");
                
                while running.load(Ordering::Relaxed) {
                    if let Err(e) = detection_fn() {
                        tracing::error!("❌ 套利检测错误: {}", e);
                        // 短暂休眠避免忙循环
                        std::thread::sleep(std::time::Duration::from_nanos(100));
                    }
                }
                
                tracing::info!("🛑 套利检测线程结束");
            },
        )?;
        
        Ok(handle)
    }
    
    /// 停止套利检测线程
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

/// 获取CPU特性信息
pub fn get_cpu_features() -> CpuFeatures {
    CpuFeatures {
        avx512f: std::is_x86_feature_detected!("avx512f"),
        avx512dq: std::is_x86_feature_detected!("avx512dq"),
        avx512bw: std::is_x86_feature_detected!("avx512bw"),
        avx512vl: std::is_x86_feature_detected!("avx512vl"),
        avx2: std::is_x86_feature_detected!("avx2"),
        fma: std::is_x86_feature_detected!("fma"),
        total_cores: num_cpus::get(),
        physical_cores: num_cpus::get_physical(),
    }
}

/// CPU特性信息
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    pub avx512f: bool,
    pub avx512dq: bool,
    pub avx512bw: bool,
    pub avx512vl: bool,
    pub avx2: bool,
    pub fma: bool,
    pub total_cores: usize,
    pub physical_cores: usize,
}

impl CpuFeatures {
    /// 检查是否支持完整的AVX-512
    pub fn supports_full_avx512(&self) -> bool {
        self.avx512f && self.avx512dq && self.avx512bw && self.avx512vl
    }
    
    /// 获取推荐的向量化策略
    pub fn get_recommended_simd_strategy(&self) -> &'static str {
        if self.supports_full_avx512() {
            "AVX-512 (8x64bit parallel)"
        } else if self.avx2 {
            "AVX2 (4x64bit parallel)"
        } else {
            "Scalar (single 64bit)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cpu_affinity_manager() {
        let manager = CpuAffinityManager::new();
        
        // 测试初始化
        assert!(manager.initialize().is_ok());
        
        // 测试获取统计信息
        let stats = manager.get_assignment_stats();
        assert!(stats.contains_key(&CoreType::ArbitrageDetection));
        assert!(stats.contains_key(&CoreType::MarketDataProcessing));
    }
    
    #[test]
    fn test_cpu_features() {
        let features = get_cpu_features();
        
        // 基本检查
        assert!(features.total_cores > 0);
        assert!(features.physical_cores > 0);
        
        println!("CPU特性: {:?}", features);
        println!("推荐SIMD策略: {}", features.get_recommended_simd_strategy());
    }
    
    #[test]
    fn test_arbitrage_detection_thread() {
        let detection_thread = ArbitrageDetectionThread::new();
        
        let counter = Arc::new(AtomicUsize::new(0));
        let test_counter = Arc::clone(&counter);
        
        let handle = detection_thread.start(move || {
            let count = test_counter.fetch_add(1, Ordering::Relaxed);
            if count >= 3 {
                return Ok(()); // 停止测试
            }
            
            std::thread::sleep(std::time::Duration::from_millis(10));
            Ok(())
        });
        
        if let Ok(handle) = handle {
            std::thread::sleep(std::time::Duration::from_millis(50));
            detection_thread.stop();
            let _ = handle.join();
            
            let final_count = counter.load(Ordering::Relaxed);
            assert!(final_count > 0);
        }
    }
} 
 