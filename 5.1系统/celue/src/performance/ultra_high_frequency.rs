use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use crossbeam::queue::ArrayQueue;
use parking_lot::{RwLock, Mutex};
use tokio::sync::Semaphore;
use aligned_vec::{AVec, ConstAlign};
use bytemuck::{Pod, Zeroable};
use std::time::Instant;
use crate::performance::simd_fixed_point::{SIMDFixedPointProcessor, FixedPrice};

/// 超高频数据处理器 - 针对100,000条/秒优化
pub struct UltraHighFrequencyProcessor {
    /// 无锁输入队列 - 使用多个队列减少竞争
    input_queues: Vec<Arc<ArrayQueue<ProcessingTask>>>,
    /// 批处理缓冲区池
    buffer_pool: Arc<BufferPool>,
    /// SIMD处理器池
    simd_processors: Vec<Arc<SIMDFixedPointProcessor>>,
    /// 工作线程控制
    worker_control: Arc<WorkerControl>,
    /// 性能监控
    perf_monitor: Arc<PerformanceMonitor>,
    /// 队列轮询索引
    queue_index: AtomicUsize,
    /// 批处理大小 - 动态调整
    batch_size: AtomicUsize,
}

/// 处理任务
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ProcessingTask {
    pub timestamp_nanos: u64,
    pub buy_price: u64,    // FixedPrice格式
    pub sell_price: u64,   // FixedPrice格式
    pub volume: u64,       // FixedPrice格式
    pub exchange_id: u32,
    pub symbol_id: u32,
    pub task_type: u32,    // 0=inter_exchange, 1=triangular
    pub padding: u32,      // 保持64字节对齐
}

/// 缓冲区池 - 避免内存分配
pub struct BufferPool {
    /// 对齐的缓冲区池
    buffers: ArrayQueue<AVec<ProcessingTask, ConstAlign<64>>>,
    /// 缓冲区大小
    buffer_size: usize,
    /// 池大小
    pool_size: usize,
}

/// 工作线程控制
pub struct WorkerControl {
    /// 活跃线程数
    active_workers: AtomicUsize,
    /// 停止标志
    should_stop: AtomicBool,
    /// 线程并发控制
    worker_semaphore: Semaphore,
    /// CPU亲和性配置
    cpu_affinity: Vec<usize>,
}

/// 性能监控
pub struct PerformanceMonitor {
    /// 处理的任务数
    processed_tasks: AtomicUsize,
    /// 总延迟（纳秒）
    total_latency_ns: AtomicUsize,
    /// 最大延迟（纳秒）
    max_latency_ns: AtomicUsize,
    /// 当前批处理大小
    current_batch_size: AtomicUsize,
    /// 开始时间
    start_time: Instant,
}

impl UltraHighFrequencyProcessor {
    /// 创建超高频处理器
    pub fn new(config: UltraHFConfig) -> Arc<Self> {
        let num_queues = config.num_input_queues;
        let queue_capacity = config.queue_capacity;
        
        // 创建多个输入队列减少锁竞争
        let input_queues: Vec<_> = (0..num_queues)
            .map(|_| Arc::new(ArrayQueue::new(queue_capacity)))
            .collect();
        
        // 创建缓冲区池
        let buffer_pool = Arc::new(BufferPool::new(
            config.buffer_size, 
            config.buffer_pool_size
        ));
        
        // 创建SIMD处理器池
        let simd_processors: Vec<_> = (0..config.num_simd_processors)
            .map(|_| Arc::new(SIMDFixedPointProcessor::new(config.simd_batch_size)))
            .collect();
        
        // 工作线程控制
        let worker_control = Arc::new(WorkerControl {
            active_workers: AtomicUsize::new(0),
            should_stop: AtomicBool::new(false),
            worker_semaphore: Semaphore::new(config.max_workers),
            cpu_affinity: config.cpu_affinity,
        });
        
        // 性能监控
        let perf_monitor = Arc::new(PerformanceMonitor {
            processed_tasks: AtomicUsize::new(0),
            total_latency_ns: AtomicUsize::new(0),
            max_latency_ns: AtomicUsize::new(0),
            current_batch_size: AtomicUsize::new(config.initial_batch_size),
            start_time: Instant::now(),
        });
        
        Arc::new(Self {
            input_queues,
            buffer_pool,
            simd_processors,
            worker_control,
            perf_monitor,
            queue_index: AtomicUsize::new(0),
            batch_size: AtomicUsize::new(config.initial_batch_size),
        })
    }
    
    /// 启动超高频处理器
    pub async fn start(&self, num_workers: usize) {
        println!("🚀 启动超高频处理器，工作线程数: {}", num_workers);
        
        // 启动处理工作线程
        for worker_id in 0..num_workers {
            let processor = Arc::clone(self);
            let cpu_core = if worker_id < self.worker_control.cpu_affinity.len() {
                Some(self.worker_control.cpu_affinity[worker_id])
            } else {
                None
            };
            
            tokio::spawn(async move {
                processor.worker_loop(worker_id, cpu_core).await;
            });
        }
        
        // 启动性能监控线程
        let monitor = Arc::clone(&self.perf_monitor);
        tokio::spawn(async move {
            Self::performance_monitor_loop(monitor).await;
        });
        
        // 启动动态批处理调整
        let processor = Arc::clone(self);
        tokio::spawn(async move {
            processor.dynamic_batch_adjustment().await;
        });
    }
    
    /// 无锁提交任务
    pub fn submit_task(&self, task: ProcessingTask) -> bool {
        // 轮询选择队列减少竞争
        let queue_idx = self.queue_index.fetch_add(1, Ordering::Relaxed) % self.input_queues.len();
        
        // 尝试提交到选定队列
        if self.input_queues[queue_idx].push(task).is_ok() {
            return true;
        }
        
        // 如果失败，尝试其他队列
        for i in 0..self.input_queues.len() {
            let idx = (queue_idx + i + 1) % self.input_queues.len();
            if self.input_queues[idx].push(task).is_ok() {
                return true;
            }
        }
        
        false // 所有队列都满了
    }
    
    /// 工作线程主循环
    async fn worker_loop(&self, worker_id: usize, cpu_core: Option<usize>) {
        // 设置CPU亲和性
        if let Some(core) = cpu_core {
            self.set_cpu_affinity(core);
        }
        
        self.worker_control.active_workers.fetch_add(1, Ordering::SeqCst);
        
        println!("⚡ 工作线程 {} 启动，CPU核心: {:?}", worker_id, cpu_core);
        
        // 获取专用缓冲区
        let mut buffer = self.buffer_pool.get_buffer().await;
        let current_batch_size = self.batch_size.load(Ordering::Relaxed);
        let simd_processor_idx = worker_id % self.simd_processors.len();
        let simd_processor = &self.simd_processors[simd_processor_idx];
        
        while !self.worker_control.should_stop.load(Ordering::Relaxed) {
            // 批量收集任务
            let collected = self.collect_batch_tasks(&mut buffer, current_batch_size).await;
            
            if collected > 0 {
                // 批量处理
                let start_time = Instant::now();
                self.process_batch_simd(&buffer[..collected], simd_processor).await;
                let processing_time = start_time.elapsed();
                
                // 更新性能指标
                self.update_performance_metrics(collected, processing_time);
                
                buffer.clear();
            } else {
                // 没有任务时短暂休眠
                tokio::task::yield_now().await;
            }
        }
        
        // 归还缓冲区
        self.buffer_pool.return_buffer(buffer).await;
        self.worker_control.active_workers.fetch_sub(1, Ordering::SeqCst);
        
        println!("🛑 工作线程 {} 停止", worker_id);
    }
    
    /// 批量收集任务
    async fn collect_batch_tasks(
        &self, 
        buffer: &mut AVec<ProcessingTask, ConstAlign<64>>, 
        target_size: usize
    ) -> usize {
        let mut collected = 0;
        let start_queue = self.queue_index.load(Ordering::Relaxed) % self.input_queues.len();
        
        // 从多个队列收集任务，避免饥饿
        for round in 0..target_size {
            let queue_idx = (start_queue + round) % self.input_queues.len();
            
            if let Some(task) = self.input_queues[queue_idx].pop() {
                buffer.push(task);
                collected += 1;
                
                if collected >= target_size {
                    break;
                }
            }
            
            // 如果单轮收集不足，继续下一轮
            if round == self.input_queues.len() - 1 && collected < target_size / 4 {
                break; // 避免空转
            }
        }
        
        collected
    }
    
    /// SIMD批量处理
    async fn process_batch_simd(
        &self,
        tasks: &[ProcessingTask],
        simd_processor: &SIMDFixedPointProcessor,
    ) {
        if tasks.is_empty() {
            return;
        }
        
        // 准备SIMD输入数据
        let buy_prices: Vec<FixedPrice> = tasks.iter()
            .map(|t| FixedPrice::from_raw(t.buy_price))
            .collect();
        
        let sell_prices: Vec<FixedPrice> = tasks.iter()
            .map(|t| FixedPrice::from_raw(t.sell_price))
            .collect();
        
        let volumes: Vec<FixedPrice> = tasks.iter()
            .map(|t| FixedPrice::from_raw(t.volume))
            .collect();
        
        // AVX-512并行计算利润
        match simd_processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &volumes) {
            Ok(profits) => {
                // 处理有利可图的机会
                for (i, profit) in profits.iter().enumerate() {
                    if profit.to_f64() > 0.001 { // 最小利润阈值
                        self.handle_profitable_opportunity(&tasks[i], *profit).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ SIMD处理错误: {}", e);
            }
        }
    }
    
    /// 处理盈利机会
    async fn handle_profitable_opportunity(&self, task: &ProcessingTask, profit: FixedPrice) {
        // 这里集成实际的交易策略逻辑
        // 包括风险控制、订单执行等
        
        // 快速风险检查
        if self.quick_risk_check(task, profit) {
            // 记录机会（异步）
            tokio::spawn(async move {
                // 实际的交易执行逻辑
                println!("💰 发现盈利机会: 交易对={}-{}, 利润={:.6}", 
                    task.symbol_id, task.exchange_id, profit.to_f64());
            });
        }
    }
    
    /// 快速风险检查
    fn quick_risk_check(&self, task: &ProcessingTask, profit: FixedPrice) -> bool {
        // 实现快速风险检查逻辑
        let profit_pct = profit.to_f64() / FixedPrice::from_raw(task.buy_price).to_f64();
        
        // 基本检查：利润率合理性
        profit_pct > 0.001 && profit_pct < 0.1 // 0.1% - 10%
    }
    
    /// 更新性能指标
    fn update_performance_metrics(&self, processed_count: usize, processing_time: std::time::Duration) {
        let latency_ns = processing_time.as_nanos() as usize;
        
        self.perf_monitor.processed_tasks.fetch_add(processed_count, Ordering::Relaxed);
        self.perf_monitor.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
        
        // 更新最大延迟
        let current_max = self.perf_monitor.max_latency_ns.load(Ordering::Relaxed);
        if latency_ns > current_max {
            self.perf_monitor.max_latency_ns.compare_exchange_weak(
                current_max, latency_ns, Ordering::Relaxed, Ordering::Relaxed
            ).ok();
        }
    }
    
    /// 动态批处理大小调整
    async fn dynamic_batch_adjustment(&self) {
        let mut adjustment_interval = tokio::time::interval(std::time::Duration::from_millis(100));
        
        loop {
            adjustment_interval.tick().await;
            
            let processed = self.perf_monitor.processed_tasks.load(Ordering::Relaxed);
            let total_latency = self.perf_monitor.total_latency_ns.load(Ordering::Relaxed);
            
            if processed > 0 {
                let avg_latency_ns = total_latency / processed;
                let current_batch = self.batch_size.load(Ordering::Relaxed);
                
                // 动态调整策略
                let new_batch_size = if avg_latency_ns > 100_000 { // 超过100微秒
                    (current_batch * 8 / 10).max(256) // 减少20%，最小256
                } else if avg_latency_ns < 50_000 { // 小于50微秒
                    (current_batch * 12 / 10).min(4096) // 增加20%，最大4096
                } else {
                    current_batch // 保持不变
                };
                
                if new_batch_size != current_batch {
                    self.batch_size.store(new_batch_size, Ordering::Relaxed);
                    println!("📊 动态调整批处理大小: {} -> {}, 平均延迟: {}ns", 
                        current_batch, new_batch_size, avg_latency_ns);
                }
            }
        }
    }
    
    /// 性能监控循环
    async fn performance_monitor_loop(monitor: Arc<PerformanceMonitor>) {
        let mut report_interval = tokio::time::interval(std::time::Duration::from_secs(5));
        
        loop {
            report_interval.tick().await;
            
            let processed = monitor.processed_tasks.load(Ordering::Relaxed);
            let total_latency = monitor.total_latency_ns.load(Ordering::Relaxed);
            let max_latency = monitor.max_latency_ns.load(Ordering::Relaxed);
            let elapsed = monitor.start_time.elapsed().as_secs_f64();
            
            if processed > 0 && elapsed > 0.0 {
                let throughput = processed as f64 / elapsed;
                let avg_latency_us = (total_latency / processed) as f64 / 1000.0;
                let max_latency_us = max_latency as f64 / 1000.0;
                
                println!("⚡ 性能报告: 吞吐量={:.0}条/秒, 平均延迟={:.1}μs, 最大延迟={:.1}μs", 
                    throughput, avg_latency_us, max_latency_us);
                
                // 重置统计
                monitor.processed_tasks.store(0, Ordering::Relaxed);
                monitor.total_latency_ns.store(0, Ordering::Relaxed);
                monitor.max_latency_ns.store(0, Ordering::Relaxed);
            }
        }
    }
    
    /// 设置CPU亲和性
    fn set_cpu_affinity(&self, core: usize) {
        #[cfg(target_os = "linux")]
        {
            use std::mem;
            use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
            
            unsafe {
                let mut cpu_set: cpu_set_t = mem::zeroed();
                CPU_ZERO(&mut cpu_set);
                CPU_SET(core, &mut cpu_set);
                
                let result = sched_setaffinity(
                    0, // 当前线程
                    mem::size_of::<cpu_set_t>(),
                    &cpu_set,
                );
                
                if result == 0 {
                    println!("✅ 线程绑定到CPU核心 {}", core);
                } else {
                    eprintln!("⚠️ 设置CPU亲和性失败: 核心 {}", core);
                }
            }
        }
    }
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let processed = self.perf_monitor.processed_tasks.load(Ordering::Relaxed);
        let total_latency = self.perf_monitor.total_latency_ns.load(Ordering::Relaxed);
        let max_latency = self.perf_monitor.max_latency_ns.load(Ordering::Relaxed);
        let elapsed = self.perf_monitor.start_time.elapsed().as_secs_f64();
        
        PerformanceStats {
            throughput: if elapsed > 0.0 { processed as f64 / elapsed } else { 0.0 },
            avg_latency_us: if processed > 0 { (total_latency / processed) as f64 / 1000.0 } else { 0.0 },
            max_latency_us: max_latency as f64 / 1000.0,
            processed_tasks: processed,
            active_workers: self.worker_control.active_workers.load(Ordering::Relaxed),
            current_batch_size: self.batch_size.load(Ordering::Relaxed),
        }
    }
    
    /// 停止处理器
    pub async fn stop(&self) {
        println!("🛑 停止超高频处理器...");
        self.worker_control.should_stop.store(true, Ordering::SeqCst);
        
        // 等待所有工作线程停止
        while self.worker_control.active_workers.load(Ordering::SeqCst) > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        println!("✅ 超高频处理器已停止");
    }
}

impl BufferPool {
    fn new(buffer_size: usize, pool_size: usize) -> Self {
        let buffers = ArrayQueue::new(pool_size);
        
        // 预分配对齐缓冲区
        for _ in 0..pool_size {
            let mut buffer = AVec::with_capacity_aligned(buffer_size);
            buffer.resize(0, ProcessingTask::zeroed());
            buffers.push(buffer).ok();
        }
        
        Self {
            buffers,
            buffer_size,
            pool_size,
        }
    }
    
    async fn get_buffer(&self) -> AVec<ProcessingTask, ConstAlign<64>> {
        // 尝试从池中获取
        if let Some(buffer) = self.buffers.pop() {
            return buffer;
        }
        
        // 池为空时创建新缓冲区
        let mut buffer = AVec::with_capacity_aligned(self.buffer_size);
        buffer.resize(0, ProcessingTask::zeroed());
        buffer
    }
    
    async fn return_buffer(&self, mut buffer: AVec<ProcessingTask, ConstAlign<64>>) {
        buffer.clear();
        self.buffers.push(buffer).ok(); // 如果池满了就丢弃
    }
}

/// 超高频配置
pub struct UltraHFConfig {
    pub num_input_queues: usize,
    pub queue_capacity: usize,
    pub buffer_size: usize,
    pub buffer_pool_size: usize,
    pub num_simd_processors: usize,
    pub simd_batch_size: usize,
    pub max_workers: usize,
    pub initial_batch_size: usize,
    pub cpu_affinity: Vec<usize>,
}

impl Default for UltraHFConfig {
    fn default() -> Self {
        Self {
            num_input_queues: 16,        // 16个输入队列
            queue_capacity: 8192,        // 每队列8K容量
            buffer_size: 4096,           // 4K批处理缓冲区
            buffer_pool_size: 32,        // 32个缓冲区池
            num_simd_processors: 8,      // 8个SIMD处理器
            simd_batch_size: 2048,       // SIMD批处理大小
            max_workers: 16,             // 最大16个工作线程
            initial_batch_size: 2048,    // 初始批处理大小
            cpu_affinity: (0..16).collect(), // CPU核心0-15
        }
    }
}

/// 性能统计
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub throughput: f64,
    pub avg_latency_us: f64,
    pub max_latency_us: f64,
    pub processed_tasks: usize,
    pub active_workers: usize,
    pub current_batch_size: usize,
} 
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use crossbeam::queue::ArrayQueue;
use parking_lot::{RwLock, Mutex};
use tokio::sync::Semaphore;
use aligned_vec::{AVec, ConstAlign};
use bytemuck::{Pod, Zeroable};
use std::time::Instant;
use crate::performance::simd_fixed_point::{SIMDFixedPointProcessor, FixedPrice};

/// 超高频数据处理器 - 针对100,000条/秒优化
pub struct UltraHighFrequencyProcessor {
    /// 无锁输入队列 - 使用多个队列减少竞争
    input_queues: Vec<Arc<ArrayQueue<ProcessingTask>>>,
    /// 批处理缓冲区池
    buffer_pool: Arc<BufferPool>,
    /// SIMD处理器池
    simd_processors: Vec<Arc<SIMDFixedPointProcessor>>,
    /// 工作线程控制
    worker_control: Arc<WorkerControl>,
    /// 性能监控
    perf_monitor: Arc<PerformanceMonitor>,
    /// 队列轮询索引
    queue_index: AtomicUsize,
    /// 批处理大小 - 动态调整
    batch_size: AtomicUsize,
}

/// 处理任务
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ProcessingTask {
    pub timestamp_nanos: u64,
    pub buy_price: u64,    // FixedPrice格式
    pub sell_price: u64,   // FixedPrice格式
    pub volume: u64,       // FixedPrice格式
    pub exchange_id: u32,
    pub symbol_id: u32,
    pub task_type: u32,    // 0=inter_exchange, 1=triangular
    pub padding: u32,      // 保持64字节对齐
}

/// 缓冲区池 - 避免内存分配
pub struct BufferPool {
    /// 对齐的缓冲区池
    buffers: ArrayQueue<AVec<ProcessingTask, ConstAlign<64>>>,
    /// 缓冲区大小
    buffer_size: usize,
    /// 池大小
    pool_size: usize,
}

/// 工作线程控制
pub struct WorkerControl {
    /// 活跃线程数
    active_workers: AtomicUsize,
    /// 停止标志
    should_stop: AtomicBool,
    /// 线程并发控制
    worker_semaphore: Semaphore,
    /// CPU亲和性配置
    cpu_affinity: Vec<usize>,
}

/// 性能监控
pub struct PerformanceMonitor {
    /// 处理的任务数
    processed_tasks: AtomicUsize,
    /// 总延迟（纳秒）
    total_latency_ns: AtomicUsize,
    /// 最大延迟（纳秒）
    max_latency_ns: AtomicUsize,
    /// 当前批处理大小
    current_batch_size: AtomicUsize,
    /// 开始时间
    start_time: Instant,
}

impl UltraHighFrequencyProcessor {
    /// 创建超高频处理器
    pub fn new(config: UltraHFConfig) -> Arc<Self> {
        let num_queues = config.num_input_queues;
        let queue_capacity = config.queue_capacity;
        
        // 创建多个输入队列减少锁竞争
        let input_queues: Vec<_> = (0..num_queues)
            .map(|_| Arc::new(ArrayQueue::new(queue_capacity)))
            .collect();
        
        // 创建缓冲区池
        let buffer_pool = Arc::new(BufferPool::new(
            config.buffer_size, 
            config.buffer_pool_size
        ));
        
        // 创建SIMD处理器池
        let simd_processors: Vec<_> = (0..config.num_simd_processors)
            .map(|_| Arc::new(SIMDFixedPointProcessor::new(config.simd_batch_size)))
            .collect();
        
        // 工作线程控制
        let worker_control = Arc::new(WorkerControl {
            active_workers: AtomicUsize::new(0),
            should_stop: AtomicBool::new(false),
            worker_semaphore: Semaphore::new(config.max_workers),
            cpu_affinity: config.cpu_affinity,
        });
        
        // 性能监控
        let perf_monitor = Arc::new(PerformanceMonitor {
            processed_tasks: AtomicUsize::new(0),
            total_latency_ns: AtomicUsize::new(0),
            max_latency_ns: AtomicUsize::new(0),
            current_batch_size: AtomicUsize::new(config.initial_batch_size),
            start_time: Instant::now(),
        });
        
        Arc::new(Self {
            input_queues,
            buffer_pool,
            simd_processors,
            worker_control,
            perf_monitor,
            queue_index: AtomicUsize::new(0),
            batch_size: AtomicUsize::new(config.initial_batch_size),
        })
    }
    
    /// 启动超高频处理器
    pub async fn start(&self, num_workers: usize) {
        println!("🚀 启动超高频处理器，工作线程数: {}", num_workers);
        
        // 启动处理工作线程
        for worker_id in 0..num_workers {
            let processor = Arc::clone(self);
            let cpu_core = if worker_id < self.worker_control.cpu_affinity.len() {
                Some(self.worker_control.cpu_affinity[worker_id])
            } else {
                None
            };
            
            tokio::spawn(async move {
                processor.worker_loop(worker_id, cpu_core).await;
            });
        }
        
        // 启动性能监控线程
        let monitor = Arc::clone(&self.perf_monitor);
        tokio::spawn(async move {
            Self::performance_monitor_loop(monitor).await;
        });
        
        // 启动动态批处理调整
        let processor = Arc::clone(self);
        tokio::spawn(async move {
            processor.dynamic_batch_adjustment().await;
        });
    }
    
    /// 无锁提交任务
    pub fn submit_task(&self, task: ProcessingTask) -> bool {
        // 轮询选择队列减少竞争
        let queue_idx = self.queue_index.fetch_add(1, Ordering::Relaxed) % self.input_queues.len();
        
        // 尝试提交到选定队列
        if self.input_queues[queue_idx].push(task).is_ok() {
            return true;
        }
        
        // 如果失败，尝试其他队列
        for i in 0..self.input_queues.len() {
            let idx = (queue_idx + i + 1) % self.input_queues.len();
            if self.input_queues[idx].push(task).is_ok() {
                return true;
            }
        }
        
        false // 所有队列都满了
    }
    
    /// 工作线程主循环
    async fn worker_loop(&self, worker_id: usize, cpu_core: Option<usize>) {
        // 设置CPU亲和性
        if let Some(core) = cpu_core {
            self.set_cpu_affinity(core);
        }
        
        self.worker_control.active_workers.fetch_add(1, Ordering::SeqCst);
        
        println!("⚡ 工作线程 {} 启动，CPU核心: {:?}", worker_id, cpu_core);
        
        // 获取专用缓冲区
        let mut buffer = self.buffer_pool.get_buffer().await;
        let current_batch_size = self.batch_size.load(Ordering::Relaxed);
        let simd_processor_idx = worker_id % self.simd_processors.len();
        let simd_processor = &self.simd_processors[simd_processor_idx];
        
        while !self.worker_control.should_stop.load(Ordering::Relaxed) {
            // 批量收集任务
            let collected = self.collect_batch_tasks(&mut buffer, current_batch_size).await;
            
            if collected > 0 {
                // 批量处理
                let start_time = Instant::now();
                self.process_batch_simd(&buffer[..collected], simd_processor).await;
                let processing_time = start_time.elapsed();
                
                // 更新性能指标
                self.update_performance_metrics(collected, processing_time);
                
                buffer.clear();
            } else {
                // 没有任务时短暂休眠
                tokio::task::yield_now().await;
            }
        }
        
        // 归还缓冲区
        self.buffer_pool.return_buffer(buffer).await;
        self.worker_control.active_workers.fetch_sub(1, Ordering::SeqCst);
        
        println!("🛑 工作线程 {} 停止", worker_id);
    }
    
    /// 批量收集任务
    async fn collect_batch_tasks(
        &self, 
        buffer: &mut AVec<ProcessingTask, ConstAlign<64>>, 
        target_size: usize
    ) -> usize {
        let mut collected = 0;
        let start_queue = self.queue_index.load(Ordering::Relaxed) % self.input_queues.len();
        
        // 从多个队列收集任务，避免饥饿
        for round in 0..target_size {
            let queue_idx = (start_queue + round) % self.input_queues.len();
            
            if let Some(task) = self.input_queues[queue_idx].pop() {
                buffer.push(task);
                collected += 1;
                
                if collected >= target_size {
                    break;
                }
            }
            
            // 如果单轮收集不足，继续下一轮
            if round == self.input_queues.len() - 1 && collected < target_size / 4 {
                break; // 避免空转
            }
        }
        
        collected
    }
    
    /// SIMD批量处理
    async fn process_batch_simd(
        &self,
        tasks: &[ProcessingTask],
        simd_processor: &SIMDFixedPointProcessor,
    ) {
        if tasks.is_empty() {
            return;
        }
        
        // 准备SIMD输入数据
        let buy_prices: Vec<FixedPrice> = tasks.iter()
            .map(|t| FixedPrice::from_raw(t.buy_price))
            .collect();
        
        let sell_prices: Vec<FixedPrice> = tasks.iter()
            .map(|t| FixedPrice::from_raw(t.sell_price))
            .collect();
        
        let volumes: Vec<FixedPrice> = tasks.iter()
            .map(|t| FixedPrice::from_raw(t.volume))
            .collect();
        
        // AVX-512并行计算利润
        match simd_processor.calculate_profit_batch_optimal(&buy_prices, &sell_prices, &volumes) {
            Ok(profits) => {
                // 处理有利可图的机会
                for (i, profit) in profits.iter().enumerate() {
                    if profit.to_f64() > 0.001 { // 最小利润阈值
                        self.handle_profitable_opportunity(&tasks[i], *profit).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️ SIMD处理错误: {}", e);
            }
        }
    }
    
    /// 处理盈利机会
    async fn handle_profitable_opportunity(&self, task: &ProcessingTask, profit: FixedPrice) {
        // 这里集成实际的交易策略逻辑
        // 包括风险控制、订单执行等
        
        // 快速风险检查
        if self.quick_risk_check(task, profit) {
            // 记录机会（异步）
            tokio::spawn(async move {
                // 实际的交易执行逻辑
                println!("💰 发现盈利机会: 交易对={}-{}, 利润={:.6}", 
                    task.symbol_id, task.exchange_id, profit.to_f64());
            });
        }
    }
    
    /// 快速风险检查
    fn quick_risk_check(&self, task: &ProcessingTask, profit: FixedPrice) -> bool {
        // 实现快速风险检查逻辑
        let profit_pct = profit.to_f64() / FixedPrice::from_raw(task.buy_price).to_f64();
        
        // 基本检查：利润率合理性
        profit_pct > 0.001 && profit_pct < 0.1 // 0.1% - 10%
    }
    
    /// 更新性能指标
    fn update_performance_metrics(&self, processed_count: usize, processing_time: std::time::Duration) {
        let latency_ns = processing_time.as_nanos() as usize;
        
        self.perf_monitor.processed_tasks.fetch_add(processed_count, Ordering::Relaxed);
        self.perf_monitor.total_latency_ns.fetch_add(latency_ns, Ordering::Relaxed);
        
        // 更新最大延迟
        let current_max = self.perf_monitor.max_latency_ns.load(Ordering::Relaxed);
        if latency_ns > current_max {
            self.perf_monitor.max_latency_ns.compare_exchange_weak(
                current_max, latency_ns, Ordering::Relaxed, Ordering::Relaxed
            ).ok();
        }
    }
    
    /// 动态批处理大小调整
    async fn dynamic_batch_adjustment(&self) {
        let mut adjustment_interval = tokio::time::interval(std::time::Duration::from_millis(100));
        
        loop {
            adjustment_interval.tick().await;
            
            let processed = self.perf_monitor.processed_tasks.load(Ordering::Relaxed);
            let total_latency = self.perf_monitor.total_latency_ns.load(Ordering::Relaxed);
            
            if processed > 0 {
                let avg_latency_ns = total_latency / processed;
                let current_batch = self.batch_size.load(Ordering::Relaxed);
                
                // 动态调整策略
                let new_batch_size = if avg_latency_ns > 100_000 { // 超过100微秒
                    (current_batch * 8 / 10).max(256) // 减少20%，最小256
                } else if avg_latency_ns < 50_000 { // 小于50微秒
                    (current_batch * 12 / 10).min(4096) // 增加20%，最大4096
                } else {
                    current_batch // 保持不变
                };
                
                if new_batch_size != current_batch {
                    self.batch_size.store(new_batch_size, Ordering::Relaxed);
                    println!("📊 动态调整批处理大小: {} -> {}, 平均延迟: {}ns", 
                        current_batch, new_batch_size, avg_latency_ns);
                }
            }
        }
    }
    
    /// 性能监控循环
    async fn performance_monitor_loop(monitor: Arc<PerformanceMonitor>) {
        let mut report_interval = tokio::time::interval(std::time::Duration::from_secs(5));
        
        loop {
            report_interval.tick().await;
            
            let processed = monitor.processed_tasks.load(Ordering::Relaxed);
            let total_latency = monitor.total_latency_ns.load(Ordering::Relaxed);
            let max_latency = monitor.max_latency_ns.load(Ordering::Relaxed);
            let elapsed = monitor.start_time.elapsed().as_secs_f64();
            
            if processed > 0 && elapsed > 0.0 {
                let throughput = processed as f64 / elapsed;
                let avg_latency_us = (total_latency / processed) as f64 / 1000.0;
                let max_latency_us = max_latency as f64 / 1000.0;
                
                println!("⚡ 性能报告: 吞吐量={:.0}条/秒, 平均延迟={:.1}μs, 最大延迟={:.1}μs", 
                    throughput, avg_latency_us, max_latency_us);
                
                // 重置统计
                monitor.processed_tasks.store(0, Ordering::Relaxed);
                monitor.total_latency_ns.store(0, Ordering::Relaxed);
                monitor.max_latency_ns.store(0, Ordering::Relaxed);
            }
        }
    }
    
    /// 设置CPU亲和性
    fn set_cpu_affinity(&self, core: usize) {
        #[cfg(target_os = "linux")]
        {
            use std::mem;
            use libc::{cpu_set_t, sched_setaffinity, CPU_SET, CPU_ZERO};
            
            unsafe {
                let mut cpu_set: cpu_set_t = mem::zeroed();
                CPU_ZERO(&mut cpu_set);
                CPU_SET(core, &mut cpu_set);
                
                let result = sched_setaffinity(
                    0, // 当前线程
                    mem::size_of::<cpu_set_t>(),
                    &cpu_set,
                );
                
                if result == 0 {
                    println!("✅ 线程绑定到CPU核心 {}", core);
                } else {
                    eprintln!("⚠️ 设置CPU亲和性失败: 核心 {}", core);
                }
            }
        }
    }
    
    /// 获取性能统计
    pub fn get_performance_stats(&self) -> PerformanceStats {
        let processed = self.perf_monitor.processed_tasks.load(Ordering::Relaxed);
        let total_latency = self.perf_monitor.total_latency_ns.load(Ordering::Relaxed);
        let max_latency = self.perf_monitor.max_latency_ns.load(Ordering::Relaxed);
        let elapsed = self.perf_monitor.start_time.elapsed().as_secs_f64();
        
        PerformanceStats {
            throughput: if elapsed > 0.0 { processed as f64 / elapsed } else { 0.0 },
            avg_latency_us: if processed > 0 { (total_latency / processed) as f64 / 1000.0 } else { 0.0 },
            max_latency_us: max_latency as f64 / 1000.0,
            processed_tasks: processed,
            active_workers: self.worker_control.active_workers.load(Ordering::Relaxed),
            current_batch_size: self.batch_size.load(Ordering::Relaxed),
        }
    }
    
    /// 停止处理器
    pub async fn stop(&self) {
        println!("🛑 停止超高频处理器...");
        self.worker_control.should_stop.store(true, Ordering::SeqCst);
        
        // 等待所有工作线程停止
        while self.worker_control.active_workers.load(Ordering::SeqCst) > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        println!("✅ 超高频处理器已停止");
    }
}

impl BufferPool {
    fn new(buffer_size: usize, pool_size: usize) -> Self {
        let buffers = ArrayQueue::new(pool_size);
        
        // 预分配对齐缓冲区
        for _ in 0..pool_size {
            let mut buffer = AVec::with_capacity_aligned(buffer_size);
            buffer.resize(0, ProcessingTask::zeroed());
            buffers.push(buffer).ok();
        }
        
        Self {
            buffers,
            buffer_size,
            pool_size,
        }
    }
    
    async fn get_buffer(&self) -> AVec<ProcessingTask, ConstAlign<64>> {
        // 尝试从池中获取
        if let Some(buffer) = self.buffers.pop() {
            return buffer;
        }
        
        // 池为空时创建新缓冲区
        let mut buffer = AVec::with_capacity_aligned(self.buffer_size);
        buffer.resize(0, ProcessingTask::zeroed());
        buffer
    }
    
    async fn return_buffer(&self, mut buffer: AVec<ProcessingTask, ConstAlign<64>>) {
        buffer.clear();
        self.buffers.push(buffer).ok(); // 如果池满了就丢弃
    }
}

/// 超高频配置
pub struct UltraHFConfig {
    pub num_input_queues: usize,
    pub queue_capacity: usize,
    pub buffer_size: usize,
    pub buffer_pool_size: usize,
    pub num_simd_processors: usize,
    pub simd_batch_size: usize,
    pub max_workers: usize,
    pub initial_batch_size: usize,
    pub cpu_affinity: Vec<usize>,
}

impl Default for UltraHFConfig {
    fn default() -> Self {
        Self {
            num_input_queues: 16,        // 16个输入队列
            queue_capacity: 8192,        // 每队列8K容量
            buffer_size: 4096,           // 4K批处理缓冲区
            buffer_pool_size: 32,        // 32个缓冲区池
            num_simd_processors: 8,      // 8个SIMD处理器
            simd_batch_size: 2048,       // SIMD批处理大小
            max_workers: 16,             // 最大16个工作线程
            initial_batch_size: 2048,    // 初始批处理大小
            cpu_affinity: (0..16).collect(), // CPU核心0-15
        }
    }
}

/// 性能统计
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub throughput: f64,
    pub avg_latency_us: f64,
    pub max_latency_us: f64,
    pub processed_tasks: usize,
    pub active_workers: usize,
    pub current_batch_size: usize,
} 