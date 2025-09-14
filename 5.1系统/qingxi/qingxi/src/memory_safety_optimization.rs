//! # 内存安全优化模块 (Memory Safety Optimization Module)
//!
//! PHASE 3: 高性能系统的内存安全架构
//! 提供线程安全的全局状态管理，消除unsafe static mut模式

use crate::production_error_handling::{QingxiResult, QingxiError};
use std::sync::{Arc, RwLock, OnceLock, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{info, warn, debug};

/// 线程安全的全局内存池管理器
pub struct SafeMemoryPoolManager {
    pool: Arc<RwLock<Option<Box<dyn MemoryPoolInterface + Send + Sync>>>>,
    is_initialized: AtomicBool,
}

/// 内存池接口，支持不同的实现策略
pub trait MemoryPoolInterface {
    fn allocate_order_block(&self, size: usize) -> QingxiResult<*mut u8>;
    fn deallocate_order_block(&self, ptr: *mut u8);
    fn get_allocation_stats(&self) -> MemoryPoolStats;
    fn reset_pool(&self) -> QingxiResult<()>;
}

/// 内存池统计信息
#[derive(Debug, Clone, Default)]
pub struct MemoryPoolStats {
    pub total_allocations: u64,
    pub active_allocations: u64,
    pub total_bytes_allocated: u64,
    pub pool_utilization: f64,
    pub fragmentation_ratio: f64,
}

/// 线程安全的CPU优化器管理器
pub struct SafeCpuOptimizerManager {
    optimizer: Arc<RwLock<Option<Box<dyn CpuOptimizerInterface + Send + Sync>>>>,
    is_initialized: AtomicBool,
}

/// CPU优化器接口
pub trait CpuOptimizerInterface {
    fn optimize_cache_layout(&self) -> QingxiResult<()>;
    fn get_performance_metrics(&self) -> CpuPerformanceMetrics;
    fn adjust_thread_affinity(&self, core_mask: u64) -> QingxiResult<()>;
}

/// CPU性能指标
#[derive(Debug, Clone, Default)]
pub struct CpuPerformanceMetrics {
    pub cache_hit_ratio: f64,
    pub instruction_throughput: u64,
    pub memory_bandwidth_utilization: f64,
    pub thread_efficiency: f64,
}

/// 线程安全的排序引擎管理器
pub struct SafeSortEngineManager {
    engine: Arc<Mutex<Option<Box<dyn SortEngineInterface + Send>>>>,
    is_initialized: AtomicBool,
}

/// 排序引擎接口
pub trait SortEngineInterface {
    fn sort_order_book(&mut self, data: &mut [u8]) -> QingxiResult<()>;
    fn get_sort_performance(&self) -> SortPerformanceMetrics;
    fn reset_engine(&mut self) -> QingxiResult<()>;
}

/// 排序性能指标
#[derive(Debug, Clone, Default)]
pub struct SortPerformanceMetrics {
    pub sorts_per_second: u64,
    pub average_sort_time_ns: u64,
    pub memory_efficiency: f64,
}

/// 线程安全的数据清理器管理器
pub struct SafeCleanerManager {
    cleaner: Arc<RwLock<Option<Box<dyn CleanerInterface + Send + Sync>>>>,
    is_initialized: AtomicBool,
}

/// 数据清理器接口
pub trait CleanerInterface {
    fn clean_market_data(&self, data: &mut [u8]) -> QingxiResult<usize>;
    fn get_cleaning_stats(&self) -> CleaningStats;
    fn optimize_cleaning_pattern(&self) -> QingxiResult<()>;
}

/// 清理统计信息
#[derive(Debug, Clone, Default)]
pub struct CleaningStats {
    pub data_processed_bytes: u64,
    pub cleaning_efficiency: f64,
    pub error_correction_rate: f64,
    pub processing_speed_mbps: f64,
}

/// 全局安全管理器实例
static SAFE_MEMORY_POOL: OnceLock<SafeMemoryPoolManager> = OnceLock::new();
static SAFE_CPU_OPTIMIZER: OnceLock<SafeCpuOptimizerManager> = OnceLock::new();
static SAFE_SORT_ENGINE: OnceLock<SafeSortEngineManager> = OnceLock::new();
static SAFE_CLEANER: OnceLock<SafeCleanerManager> = OnceLock::new();

impl SafeMemoryPoolManager {
    /// 创建新的内存池管理器
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(None)),
            is_initialized: AtomicBool::new(false),
        }
    }
    
    /// 安全初始化内存池
    pub fn initialize<T>(&self, pool_impl: T) -> QingxiResult<()>
    where
        T: MemoryPoolInterface + Send + Sync + 'static,
    {
        let mut pool_guard = self.pool.write()
            .map_err(|e| QingxiError::memory_pool(format!("Failed to acquire write lock: {}", e)))?;
        
        if pool_guard.is_some() {
            warn!("⚠️ Memory pool already initialized, skipping");
            return Ok(());
        }
        
        *pool_guard = Some(Box::new(pool_impl));
        self.is_initialized.store(true, Ordering::Release);
        
        info!("✅ Memory pool initialized successfully");
        Ok(())
    }
    
    /// 安全获取内存池引用
    pub fn with_pool<R, F>(&self, f: F) -> QingxiResult<R>
    where
        F: FnOnce(&dyn MemoryPoolInterface) -> QingxiResult<R>,
    {
        if !self.is_initialized.load(Ordering::Acquire) {
            return Err(QingxiError::memory_pool("Memory pool not initialized"));
        }
        
        let pool_guard = self.pool.read()
            .map_err(|e| QingxiError::memory_pool(format!("Failed to acquire read lock: {}", e)))?;
        
        match pool_guard.as_ref() {
            Some(pool) => f(pool.as_ref()),
            None => Err(QingxiError::memory_pool("Memory pool is None")),
        }
    }
    
    /// 获取全局实例
    pub fn global() -> &'static SafeMemoryPoolManager {
        SAFE_MEMORY_POOL.get_or_init(|| {
            debug!("🔧 Initializing global safe memory pool manager");
            SafeMemoryPoolManager::new()
        })
    }
}

impl SafeCpuOptimizerManager {
    /// 创建新的CPU优化器管理器
    pub fn new() -> Self {
        Self {
            optimizer: Arc::new(RwLock::new(None)),
            is_initialized: AtomicBool::new(false),
        }
    }
    
    /// 安全初始化CPU优化器
    pub fn initialize<T>(&self, optimizer_impl: T) -> QingxiResult<()>
    where
        T: CpuOptimizerInterface + Send + Sync + 'static,
    {
        let mut optimizer_guard = self.optimizer.write()
            .map_err(|e| QingxiError::system_init(format!("Failed to acquire write lock: {}", e)))?;
        
        if optimizer_guard.is_some() {
            warn!("⚠️ CPU optimizer already initialized, skipping");
            return Ok(());
        }
        
        *optimizer_guard = Some(Box::new(optimizer_impl));
        self.is_initialized.store(true, Ordering::Release);
        
        info!("✅ CPU optimizer initialized successfully");
        Ok(())
    }
    
    /// 安全获取CPU优化器引用
    pub fn with_optimizer<R, F>(&self, f: F) -> QingxiResult<R>
    where
        F: FnOnce(&dyn CpuOptimizerInterface) -> QingxiResult<R>,
    {
        if !self.is_initialized.load(Ordering::Acquire) {
            return Err(QingxiError::system_init("CPU optimizer not initialized"));
        }
        
        let optimizer_guard = self.optimizer.read()
            .map_err(|e| QingxiError::system_init(format!("Failed to acquire read lock: {}", e)))?;
        
        match optimizer_guard.as_ref() {
            Some(optimizer) => f(optimizer.as_ref()),
            None => Err(QingxiError::system_init("CPU optimizer is None")),
        }
    }
    
    /// 获取全局实例
    pub fn global() -> &'static SafeCpuOptimizerManager {
        SAFE_CPU_OPTIMIZER.get_or_init(|| {
            debug!("🔧 Initializing global safe CPU optimizer manager");
            SafeCpuOptimizerManager::new()
        })
    }
}

impl SafeSortEngineManager {
    /// 创建新的排序引擎管理器
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(None)),
            is_initialized: AtomicBool::new(false),
        }
    }
    
    /// 安全初始化排序引擎
    pub fn initialize<T>(&self, engine_impl: T) -> QingxiResult<()>
    where
        T: SortEngineInterface + Send + 'static,
    {
        let mut engine_guard = self.engine.lock()
            .map_err(|e| QingxiError::system_init(format!("Failed to acquire mutex: {}", e)))?;
        
        if engine_guard.is_some() {
            warn!("⚠️ Sort engine already initialized, skipping");
            return Ok(());
        }
        
        *engine_guard = Some(Box::new(engine_impl));
        self.is_initialized.store(true, Ordering::Release);
        
        info!("✅ Sort engine initialized successfully");
        Ok(())
    }
    
    /// 安全获取排序引擎可变引用
    pub fn with_engine_mut<R, F>(&self, f: F) -> QingxiResult<R>
    where
        F: FnOnce(&mut dyn SortEngineInterface) -> QingxiResult<R>,
    {
        if !self.is_initialized.load(Ordering::Acquire) {
            return Err(QingxiError::system_init("Sort engine not initialized"));
        }
        
        let mut engine_guard = self.engine.lock()
            .map_err(|e| QingxiError::system_init(format!("Failed to acquire mutex: {}", e)))?;
        
        match engine_guard.as_mut() {
            Some(engine) => f(engine.as_mut()),
            None => Err(QingxiError::system_init("Sort engine is None")),
        }
    }
    
    /// 获取全局实例
    pub fn global() -> &'static SafeSortEngineManager {
        SAFE_SORT_ENGINE.get_or_init(|| {
            debug!("🔧 Initializing global safe sort engine manager");
            SafeSortEngineManager::new()
        })
    }
}

impl SafeCleanerManager {
    /// 创建新的清理器管理器
    pub fn new() -> Self {
        Self {
            cleaner: Arc::new(RwLock::new(None)),
            is_initialized: AtomicBool::new(false),
        }
    }
    
    /// 安全初始化清理器
    pub fn initialize<T>(&self, cleaner_impl: T) -> QingxiResult<()>
    where
        T: CleanerInterface + Send + Sync + 'static,
    {
        let mut cleaner_guard = self.cleaner.write()
            .map_err(|e| QingxiError::system_init(format!("Failed to acquire write lock: {}", e)))?;
        
        if cleaner_guard.is_some() {
            warn!("⚠️ Cleaner already initialized, skipping");
            return Ok(());
        }
        
        *cleaner_guard = Some(Box::new(cleaner_impl));
        self.is_initialized.store(true, Ordering::Release);
        
        info!("✅ Cleaner initialized successfully");
        Ok(())
    }
    
    /// 安全获取清理器引用
    pub fn with_cleaner<R, F>(&self, f: F) -> QingxiResult<R>
    where
        F: FnOnce(&dyn CleanerInterface) -> QingxiResult<R>,
    {
        if !self.is_initialized.load(Ordering::Acquire) {
            return Err(QingxiError::system_init("Cleaner not initialized"));
        }
        
        let cleaner_guard = self.cleaner.read()
            .map_err(|e| QingxiError::system_init(format!("Failed to acquire read lock: {}", e)))?;
        
        match cleaner_guard.as_ref() {
            Some(cleaner) => f(cleaner.as_ref()),
            None => Err(QingxiError::system_init("Cleaner is None")),
        }
    }
    
    /// 获取全局实例
    pub fn global() -> &'static SafeCleanerManager {
        SAFE_CLEANER.get_or_init(|| {
            debug!("🔧 Initializing global safe cleaner manager");
            SafeCleanerManager::new()
        })
    }
}

/// 系统级内存安全初始化器
pub struct MemorySafetyInitializer;

impl MemorySafetyInitializer {
    /// 初始化所有安全管理器
    pub fn initialize_all_systems() -> QingxiResult<()> {
        info!("🛡️ Starting memory safety initialization for all systems");
        
        // 确保所有全局管理器都被创建
        let _memory_pool = SafeMemoryPoolManager::global();
        let _cpu_optimizer = SafeCpuOptimizerManager::global();
        let _sort_engine = SafeSortEngineManager::global();
        let _cleaner = SafeCleanerManager::global();
        
        info!("✅ All memory safety managers initialized successfully");
        Ok(())
    }
    
    /// 获取系统状态报告
    pub fn get_safety_status_report() -> MemorySafetyReport {
        MemorySafetyReport {
            memory_pool_initialized: SafeMemoryPoolManager::global().is_initialized.load(Ordering::Acquire),
            cpu_optimizer_initialized: SafeCpuOptimizerManager::global().is_initialized.load(Ordering::Acquire),
            sort_engine_initialized: SafeSortEngineManager::global().is_initialized.load(Ordering::Acquire),
            cleaner_initialized: SafeCleanerManager::global().is_initialized.load(Ordering::Acquire),
        }
    }
}

/// 内存安全状态报告
#[derive(Debug, Clone)]
pub struct MemorySafetyReport {
    pub memory_pool_initialized: bool,
    pub cpu_optimizer_initialized: bool,
    pub sort_engine_initialized: bool,
    pub cleaner_initialized: bool,
}

impl MemorySafetyReport {
    /// 检查所有系统是否已初始化
    pub fn all_systems_ready(&self) -> bool {
        self.memory_pool_initialized && 
        self.cpu_optimizer_initialized && 
        self.sort_engine_initialized && 
        self.cleaner_initialized
    }
    
    /// 获取未初始化的系统列表
    pub fn get_uninitialized_systems(&self) -> Vec<&'static str> {
        let mut uninitialized = Vec::new();
        
        if !self.memory_pool_initialized {
            uninitialized.push("memory_pool");
        }
        if !self.cpu_optimizer_initialized {
            uninitialized.push("cpu_optimizer");
        }
        if !self.sort_engine_initialized {
            uninitialized.push("sort_engine");
        }
        if !self.cleaner_initialized {
            uninitialized.push("cleaner");
        }
        
        uninitialized
    }
}

