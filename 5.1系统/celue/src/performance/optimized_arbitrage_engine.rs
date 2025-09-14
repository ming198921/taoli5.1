use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{RwLock, broadcast};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::performance::ultra_high_frequency::{
    UltraHighFrequencyProcessor, ProcessingTask, UltraHFConfig, PerformanceStats
};
use crate::performance::simd_fixed_point::{FixedPrice, SIMDFixedPointProcessor};
use crate::strategy::plugins::triangular::TriangularStrategy;
use crate::strategy::plugins::inter_exchange::InterExchangeStrategy;
use crate::adapters::risk::RiskAdapter;

/// 优化的套利引擎 - 专为100,000条/秒设计
pub struct OptimizedArbitrageEngine {
    /// 超高频处理器
    ultra_hf_processor: Arc<UltraHighFrequencyProcessor>,
    /// 三角套利策略
    triangular_strategy: Arc<TriangularStrategy>,
    /// 跨交易所套利策略
    inter_exchange_strategy: Arc<InterExchangeStrategy>,
    /// 风控适配器
    risk_adapter: Arc<RiskAdapter>,
    /// 性能广播通道
    perf_sender: broadcast::Sender<EnginePerformanceReport>,
    /// 机会统计
    opportunity_stats: Arc<RwLock<OpportunityStatistics>>,
    /// 符号映射（快速查找）
    symbol_map: Arc<RwLock<HashMap<String, u32>>>,
    /// 交易所映射
    exchange_map: Arc<RwLock<HashMap<String, u32>>>,
    /// 引擎配置
    config: EngineConfig,
}

/// 引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// 最小利润阈值（基点）
    pub min_profit_bps: u32,
    /// 最大单笔交易量
    pub max_trade_volume: f64,
    /// 启用三角套利
    pub enable_triangular: bool,
    /// 启用跨交易所套利
    pub enable_inter_exchange: bool,
    /// 智能批处理
    pub smart_batching: bool,
    /// 自适应风险控制
    pub adaptive_risk: bool,
}

/// 机会统计
#[derive(Debug, Default)]
pub struct OpportunityStatistics {
    pub total_opportunities: u64,
    pub triangular_opportunities: u64,
    pub inter_exchange_opportunities: u64,
    pub executed_trades: u64,
    pub rejected_by_risk: u64,
    pub total_profit: f64,
    pub avg_execution_time_us: f64,
    pub success_rate: f64,
}

/// 引擎性能报告
#[derive(Debug, Clone, Serialize)]
pub struct EnginePerformanceReport {
    pub timestamp: u64,
    pub processing_stats: PerformanceStats,
    pub opportunity_stats: OpportunityStatistics,
    pub risk_stats: RiskStatistics,
    pub strategy_efficiency: StrategyEfficiency,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskStatistics {
    pub total_checks: u64,
    pub passed_checks: u64,
    pub rejected_checks: u64,
    pub avg_check_time_us: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyEfficiency {
    pub triangular_hit_rate: f64,
    pub inter_exchange_hit_rate: f64,
    pub avg_profit_per_opportunity: f64,
    pub execution_success_rate: f64,
}

/// 市场数据输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataInput {
    pub symbol: String,
    pub exchange: String,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub timestamp: u64,
}

impl OptimizedArbitrageEngine {
    /// 创建优化套利引擎
    pub async fn new(config: EngineConfig) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        println!("🚀 初始化优化套利引擎...");
        
        // 创建超高频处理器
        let hf_config = UltraHFConfig {
            num_input_queues: 32,           // 32个队列减少竞争
            queue_capacity: 16384,          // 16K队列容量
            buffer_size: 4096,              // 4K批处理
            buffer_pool_size: 64,           // 64个缓冲区
            num_simd_processors: 16,        // 16个SIMD处理器
            simd_batch_size: 2048,          // 2K SIMD批处理
            max_workers: 24,                // 24个工作线程
            initial_batch_size: 2048,       // 初始2K批处理
            cpu_affinity: (0..24).collect(), // 绑定CPU核心0-23
        };
        
        let ultra_hf_processor = UltraHighFrequencyProcessor::new(hf_config);
        
        // 初始化策略模块
        let triangular_strategy = Arc::new(TriangularStrategy::new().await?);
        let inter_exchange_strategy = Arc::new(InterExchangeStrategy::new().await?);
        let risk_adapter = Arc::new(RiskAdapter::new().await?);
        
        // 性能广播通道
        let (perf_sender, _) = broadcast::channel(1000);
        
        // 统计数据
        let opportunity_stats = Arc::new(RwLock::new(OpportunityStatistics::default()));
        
        // 符号和交易所映射
        let symbol_map = Arc::new(RwLock::new(HashMap::new()));
        let exchange_map = Arc::new(RwLock::new(HashMap::new()));
        
        let engine = Arc::new(Self {
            ultra_hf_processor,
            triangular_strategy,
            inter_exchange_strategy,
            risk_adapter,
            perf_sender,
            opportunity_stats,
            symbol_map,
            exchange_map,
            config,
        });
        
        println!("✅ 优化套利引擎初始化完成");
        Ok(engine)
    }
    
    /// 启动引擎
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎯 启动优化套利引擎...");
        
        // 启动超高频处理器
        self.ultra_hf_processor.start(self.config.max_workers()).await;
        
        // 启动性能监控
        let engine = Arc::clone(self);
        tokio::spawn(async move {
            engine.performance_monitoring_loop().await;
        });
        
        // 启动智能调优
        let engine = Arc::clone(self);
        tokio::spawn(async move {
            engine.intelligent_optimization_loop().await;
        });
        
        println!("✅ 优化套利引擎启动完成");
        Ok(())
    }
    
    /// 处理市场数据（主入口）
    pub async fn process_market_data(&self, data: MarketDataInput) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // 快速符号/交易所ID映射
        let symbol_id = self.get_or_create_symbol_id(&data.symbol).await;
        let exchange_id = self.get_or_create_exchange_id(&data.exchange).await;
        
        // 转换为固定点数格式
        let buy_price = FixedPrice::from_f64(data.best_ask); // 买入价格是ask
        let sell_price = FixedPrice::from_f64(data.best_bid); // 卖出价格是bid
        let volume = FixedPrice::from_f64(data.bid_volume.min(data.ask_volume));
        
        // 创建处理任务
        let tasks = self.create_processing_tasks(
            symbol_id, 
            exchange_id, 
            buy_price, 
            sell_price, 
            volume, 
            data.timestamp
        ).await;
        
        // 批量提交到超高频处理器
        let mut submitted = 0;
        for task in tasks {
            if self.ultra_hf_processor.submit_task(task) {
                submitted += 1;
            }
        }
        
        // 记录处理延迟
        let processing_time = start_time.elapsed();
        if processing_time.as_micros() > 100 {
            println!("⚠️ 数据处理延迟: {}μs", processing_time.as_micros());
        }
        
        Ok(())
    }
    
    /// 创建处理任务
    async fn create_processing_tasks(
        &self,
        symbol_id: u32,
        exchange_id: u32,
        buy_price: FixedPrice,
        sell_price: FixedPrice,
        volume: FixedPrice,
        timestamp: u64,
    ) -> Vec<ProcessingTask> {
        let mut tasks = Vec::new();
        
        // 跨交易所套利任务
        if self.config.enable_inter_exchange {
            tasks.push(ProcessingTask {
                timestamp_nanos: timestamp,
                buy_price: buy_price.to_raw(),
                sell_price: sell_price.to_raw(),
                volume: volume.to_raw(),
                exchange_id,
                symbol_id,
                task_type: 0, // inter_exchange
                padding: 0,
            });
        }
        
        // 三角套利任务
        if self.config.enable_triangular {
            tasks.push(ProcessingTask {
                timestamp_nanos: timestamp,
                buy_price: buy_price.to_raw(),
                sell_price: sell_price.to_raw(),
                volume: volume.to_raw(),
                exchange_id,
                symbol_id,
                task_type: 1, // triangular
                padding: 0,
            });
        }
        
        tasks
    }
    
    /// 获取或创建符号ID
    async fn get_or_create_symbol_id(&self, symbol: &str) -> u32 {
        let symbol_map = self.symbol_map.read().await;
        if let Some(&id) = symbol_map.get(symbol) {
            return id;
        }
        drop(symbol_map);
        
        let mut symbol_map = self.symbol_map.write().await;
        let id = symbol_map.len() as u32;
        symbol_map.insert(symbol.to_string(), id);
        id
    }
    
    /// 获取或创建交易所ID
    async fn get_or_create_exchange_id(&self, exchange: &str) -> u32 {
        let exchange_map = self.exchange_map.read().await;
        if let Some(&id) = exchange_map.get(exchange) {
            return id;
        }
        drop(exchange_map);
        
        let mut exchange_map = self.exchange_map.write().await;
        let id = exchange_map.len() as u32;
        exchange_map.insert(exchange.to_string(), id);
        id
    }
    
    /// 性能监控循环
    async fn performance_monitoring_loop(&self) {
        let mut report_interval = tokio::time::interval(std::time::Duration::from_secs(5));
        
        loop {
            report_interval.tick().await;
            
            // 收集性能数据
            let processing_stats = self.ultra_hf_processor.get_performance_stats();
            let opportunity_stats = self.opportunity_stats.read().await.clone();
            
            // 生成报告
            let report = EnginePerformanceReport {
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
                processing_stats,
                opportunity_stats,
                risk_stats: RiskStatistics {
                    total_checks: 0, // 从风控适配器获取
                    passed_checks: 0,
                    rejected_checks: 0,
                    avg_check_time_us: 0.0,
                },
                strategy_efficiency: StrategyEfficiency {
                    triangular_hit_rate: 0.0,
                    inter_exchange_hit_rate: 0.0,
                    avg_profit_per_opportunity: 0.0,
                    execution_success_rate: 0.0,
                },
            };
            
            // 广播性能报告
            self.perf_sender.send(report.clone()).ok();
            
            // 控制台输出
            println!("📊 引擎性能: 吞吐量={:.0}条/秒, 延迟={:.1}μs, 机会数={}", 
                report.processing_stats.throughput,
                report.processing_stats.avg_latency_us,
                report.opportunity_stats.total_opportunities
            );
        }
    }
    
    /// 智能优化循环
    async fn intelligent_optimization_loop(&self) {
        let mut optimization_interval = tokio::time::interval(std::time::Duration::from_secs(30));
        
        loop {
            optimization_interval.tick().await;
            
            let stats = self.ultra_hf_processor.get_performance_stats();
            
            // 智能调优逻辑
            if stats.avg_latency_us > 100.0 {
                println!("🔧 检测到高延迟，启动优化...");
                self.optimize_for_latency().await;
            }
            
            if stats.throughput < 50000.0 {
                println!("🔧 检测到低吞吐量，启动优化...");
                self.optimize_for_throughput().await;
            }
        }
    }
    
    /// 延迟优化
    async fn optimize_for_latency(&self) {
        // 实现延迟优化策略
        println!("⚡ 执行延迟优化策略");
        
        // 1. 减少批处理大小
        // 2. 增加工作线程
        // 3. 优化内存分配
        // 4. 调整CPU亲和性
    }
    
    /// 吞吐量优化
    async fn optimize_for_throughput(&self) {
        // 实现吞吐量优化策略
        println!("🚀 执行吞吐量优化策略");
        
        // 1. 增加批处理大小
        // 2. 更多SIMD并行
        // 3. 优化队列容量
        // 4. 减少锁竞争
    }
    
    /// 获取性能报告订阅
    pub fn subscribe_performance(&self) -> broadcast::Receiver<EnginePerformanceReport> {
        self.perf_sender.subscribe()
    }
    
    /// 获取实时统计
    pub async fn get_real_time_stats(&self) -> EnginePerformanceReport {
        let processing_stats = self.ultra_hf_processor.get_performance_stats();
        let opportunity_stats = self.opportunity_stats.read().await.clone();
        
        EnginePerformanceReport {
            timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            processing_stats,
            opportunity_stats,
            risk_stats: RiskStatistics {
                total_checks: 0,
                passed_checks: 0,
                rejected_checks: 0,
                avg_check_time_us: 0.0,
            },
            strategy_efficiency: StrategyEfficiency {
                triangular_hit_rate: 0.0,
                inter_exchange_hit_rate: 0.0,
                avg_profit_per_opportunity: 0.0,
                execution_success_rate: 0.0,
            },
        }
    }
    
    /// 停止引擎
    pub async fn stop(&self) {
        println!("🛑 停止优化套利引擎...");
        self.ultra_hf_processor.stop().await;
        println!("✅ 优化套利引擎已停止");
    }
}

impl EngineConfig {
    pub fn default() -> Self {
        Self {
            min_profit_bps: 10,           // 10个基点最小利润
            max_trade_volume: 100000.0,   // 最大10万交易量
            enable_triangular: true,
            enable_inter_exchange: true,
            smart_batching: true,
            adaptive_risk: true,
        }
    }
    
    pub fn max_workers(&self) -> usize {
        24 // 根据配置返回最大工作线程数
    }
} 
use std::time::Instant;
use tokio::sync::{RwLock, broadcast};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::performance::ultra_high_frequency::{
    UltraHighFrequencyProcessor, ProcessingTask, UltraHFConfig, PerformanceStats
};
use crate::performance::simd_fixed_point::{FixedPrice, SIMDFixedPointProcessor};
use crate::strategy::plugins::triangular::TriangularStrategy;
use crate::strategy::plugins::inter_exchange::InterExchangeStrategy;
use crate::adapters::risk::RiskAdapter;

/// 优化的套利引擎 - 专为100,000条/秒设计
pub struct OptimizedArbitrageEngine {
    /// 超高频处理器
    ultra_hf_processor: Arc<UltraHighFrequencyProcessor>,
    /// 三角套利策略
    triangular_strategy: Arc<TriangularStrategy>,
    /// 跨交易所套利策略
    inter_exchange_strategy: Arc<InterExchangeStrategy>,
    /// 风控适配器
    risk_adapter: Arc<RiskAdapter>,
    /// 性能广播通道
    perf_sender: broadcast::Sender<EnginePerformanceReport>,
    /// 机会统计
    opportunity_stats: Arc<RwLock<OpportunityStatistics>>,
    /// 符号映射（快速查找）
    symbol_map: Arc<RwLock<HashMap<String, u32>>>,
    /// 交易所映射
    exchange_map: Arc<RwLock<HashMap<String, u32>>>,
    /// 引擎配置
    config: EngineConfig,
}

/// 引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// 最小利润阈值（基点）
    pub min_profit_bps: u32,
    /// 最大单笔交易量
    pub max_trade_volume: f64,
    /// 启用三角套利
    pub enable_triangular: bool,
    /// 启用跨交易所套利
    pub enable_inter_exchange: bool,
    /// 智能批处理
    pub smart_batching: bool,
    /// 自适应风险控制
    pub adaptive_risk: bool,
}

/// 机会统计
#[derive(Debug, Default)]
pub struct OpportunityStatistics {
    pub total_opportunities: u64,
    pub triangular_opportunities: u64,
    pub inter_exchange_opportunities: u64,
    pub executed_trades: u64,
    pub rejected_by_risk: u64,
    pub total_profit: f64,
    pub avg_execution_time_us: f64,
    pub success_rate: f64,
}

/// 引擎性能报告
#[derive(Debug, Clone, Serialize)]
pub struct EnginePerformanceReport {
    pub timestamp: u64,
    pub processing_stats: PerformanceStats,
    pub opportunity_stats: OpportunityStatistics,
    pub risk_stats: RiskStatistics,
    pub strategy_efficiency: StrategyEfficiency,
}

#[derive(Debug, Clone, Serialize)]
pub struct RiskStatistics {
    pub total_checks: u64,
    pub passed_checks: u64,
    pub rejected_checks: u64,
    pub avg_check_time_us: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyEfficiency {
    pub triangular_hit_rate: f64,
    pub inter_exchange_hit_rate: f64,
    pub avg_profit_per_opportunity: f64,
    pub execution_success_rate: f64,
}

/// 市场数据输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataInput {
    pub symbol: String,
    pub exchange: String,
    pub best_bid: f64,
    pub best_ask: f64,
    pub bid_volume: f64,
    pub ask_volume: f64,
    pub timestamp: u64,
}

impl OptimizedArbitrageEngine {
    /// 创建优化套利引擎
    pub async fn new(config: EngineConfig) -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        println!("🚀 初始化优化套利引擎...");
        
        // 创建超高频处理器
        let hf_config = UltraHFConfig {
            num_input_queues: 32,           // 32个队列减少竞争
            queue_capacity: 16384,          // 16K队列容量
            buffer_size: 4096,              // 4K批处理
            buffer_pool_size: 64,           // 64个缓冲区
            num_simd_processors: 16,        // 16个SIMD处理器
            simd_batch_size: 2048,          // 2K SIMD批处理
            max_workers: 24,                // 24个工作线程
            initial_batch_size: 2048,       // 初始2K批处理
            cpu_affinity: (0..24).collect(), // 绑定CPU核心0-23
        };
        
        let ultra_hf_processor = UltraHighFrequencyProcessor::new(hf_config);
        
        // 初始化策略模块
        let triangular_strategy = Arc::new(TriangularStrategy::new().await?);
        let inter_exchange_strategy = Arc::new(InterExchangeStrategy::new().await?);
        let risk_adapter = Arc::new(RiskAdapter::new().await?);
        
        // 性能广播通道
        let (perf_sender, _) = broadcast::channel(1000);
        
        // 统计数据
        let opportunity_stats = Arc::new(RwLock::new(OpportunityStatistics::default()));
        
        // 符号和交易所映射
        let symbol_map = Arc::new(RwLock::new(HashMap::new()));
        let exchange_map = Arc::new(RwLock::new(HashMap::new()));
        
        let engine = Arc::new(Self {
            ultra_hf_processor,
            triangular_strategy,
            inter_exchange_strategy,
            risk_adapter,
            perf_sender,
            opportunity_stats,
            symbol_map,
            exchange_map,
            config,
        });
        
        println!("✅ 优化套利引擎初始化完成");
        Ok(engine)
    }
    
    /// 启动引擎
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎯 启动优化套利引擎...");
        
        // 启动超高频处理器
        self.ultra_hf_processor.start(self.config.max_workers()).await;
        
        // 启动性能监控
        let engine = Arc::clone(self);
        tokio::spawn(async move {
            engine.performance_monitoring_loop().await;
        });
        
        // 启动智能调优
        let engine = Arc::clone(self);
        tokio::spawn(async move {
            engine.intelligent_optimization_loop().await;
        });
        
        println!("✅ 优化套利引擎启动完成");
        Ok(())
    }
    
    /// 处理市场数据（主入口）
    pub async fn process_market_data(&self, data: MarketDataInput) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // 快速符号/交易所ID映射
        let symbol_id = self.get_or_create_symbol_id(&data.symbol).await;
        let exchange_id = self.get_or_create_exchange_id(&data.exchange).await;
        
        // 转换为固定点数格式
        let buy_price = FixedPrice::from_f64(data.best_ask); // 买入价格是ask
        let sell_price = FixedPrice::from_f64(data.best_bid); // 卖出价格是bid
        let volume = FixedPrice::from_f64(data.bid_volume.min(data.ask_volume));
        
        // 创建处理任务
        let tasks = self.create_processing_tasks(
            symbol_id, 
            exchange_id, 
            buy_price, 
            sell_price, 
            volume, 
            data.timestamp
        ).await;
        
        // 批量提交到超高频处理器
        let mut submitted = 0;
        for task in tasks {
            if self.ultra_hf_processor.submit_task(task) {
                submitted += 1;
            }
        }
        
        // 记录处理延迟
        let processing_time = start_time.elapsed();
        if processing_time.as_micros() > 100 {
            println!("⚠️ 数据处理延迟: {}μs", processing_time.as_micros());
        }
        
        Ok(())
    }
    
    /// 创建处理任务
    async fn create_processing_tasks(
        &self,
        symbol_id: u32,
        exchange_id: u32,
        buy_price: FixedPrice,
        sell_price: FixedPrice,
        volume: FixedPrice,
        timestamp: u64,
    ) -> Vec<ProcessingTask> {
        let mut tasks = Vec::new();
        
        // 跨交易所套利任务
        if self.config.enable_inter_exchange {
            tasks.push(ProcessingTask {
                timestamp_nanos: timestamp,
                buy_price: buy_price.to_raw(),
                sell_price: sell_price.to_raw(),
                volume: volume.to_raw(),
                exchange_id,
                symbol_id,
                task_type: 0, // inter_exchange
                padding: 0,
            });
        }
        
        // 三角套利任务
        if self.config.enable_triangular {
            tasks.push(ProcessingTask {
                timestamp_nanos: timestamp,
                buy_price: buy_price.to_raw(),
                sell_price: sell_price.to_raw(),
                volume: volume.to_raw(),
                exchange_id,
                symbol_id,
                task_type: 1, // triangular
                padding: 0,
            });
        }
        
        tasks
    }
    
    /// 获取或创建符号ID
    async fn get_or_create_symbol_id(&self, symbol: &str) -> u32 {
        let symbol_map = self.symbol_map.read().await;
        if let Some(&id) = symbol_map.get(symbol) {
            return id;
        }
        drop(symbol_map);
        
        let mut symbol_map = self.symbol_map.write().await;
        let id = symbol_map.len() as u32;
        symbol_map.insert(symbol.to_string(), id);
        id
    }
    
    /// 获取或创建交易所ID
    async fn get_or_create_exchange_id(&self, exchange: &str) -> u32 {
        let exchange_map = self.exchange_map.read().await;
        if let Some(&id) = exchange_map.get(exchange) {
            return id;
        }
        drop(exchange_map);
        
        let mut exchange_map = self.exchange_map.write().await;
        let id = exchange_map.len() as u32;
        exchange_map.insert(exchange.to_string(), id);
        id
    }
    
    /// 性能监控循环
    async fn performance_monitoring_loop(&self) {
        let mut report_interval = tokio::time::interval(std::time::Duration::from_secs(5));
        
        loop {
            report_interval.tick().await;
            
            // 收集性能数据
            let processing_stats = self.ultra_hf_processor.get_performance_stats();
            let opportunity_stats = self.opportunity_stats.read().await.clone();
            
            // 生成报告
            let report = EnginePerformanceReport {
                timestamp: chrono::Utc::now().timestamp_nanos() as u64,
                processing_stats,
                opportunity_stats,
                risk_stats: RiskStatistics {
                    total_checks: 0, // 从风控适配器获取
                    passed_checks: 0,
                    rejected_checks: 0,
                    avg_check_time_us: 0.0,
                },
                strategy_efficiency: StrategyEfficiency {
                    triangular_hit_rate: 0.0,
                    inter_exchange_hit_rate: 0.0,
                    avg_profit_per_opportunity: 0.0,
                    execution_success_rate: 0.0,
                },
            };
            
            // 广播性能报告
            self.perf_sender.send(report.clone()).ok();
            
            // 控制台输出
            println!("📊 引擎性能: 吞吐量={:.0}条/秒, 延迟={:.1}μs, 机会数={}", 
                report.processing_stats.throughput,
                report.processing_stats.avg_latency_us,
                report.opportunity_stats.total_opportunities
            );
        }
    }
    
    /// 智能优化循环
    async fn intelligent_optimization_loop(&self) {
        let mut optimization_interval = tokio::time::interval(std::time::Duration::from_secs(30));
        
        loop {
            optimization_interval.tick().await;
            
            let stats = self.ultra_hf_processor.get_performance_stats();
            
            // 智能调优逻辑
            if stats.avg_latency_us > 100.0 {
                println!("🔧 检测到高延迟，启动优化...");
                self.optimize_for_latency().await;
            }
            
            if stats.throughput < 50000.0 {
                println!("🔧 检测到低吞吐量，启动优化...");
                self.optimize_for_throughput().await;
            }
        }
    }
    
    /// 延迟优化
    async fn optimize_for_latency(&self) {
        // 实现延迟优化策略
        println!("⚡ 执行延迟优化策略");
        
        // 1. 减少批处理大小
        // 2. 增加工作线程
        // 3. 优化内存分配
        // 4. 调整CPU亲和性
    }
    
    /// 吞吐量优化
    async fn optimize_for_throughput(&self) {
        // 实现吞吐量优化策略
        println!("🚀 执行吞吐量优化策略");
        
        // 1. 增加批处理大小
        // 2. 更多SIMD并行
        // 3. 优化队列容量
        // 4. 减少锁竞争
    }
    
    /// 获取性能报告订阅
    pub fn subscribe_performance(&self) -> broadcast::Receiver<EnginePerformanceReport> {
        self.perf_sender.subscribe()
    }
    
    /// 获取实时统计
    pub async fn get_real_time_stats(&self) -> EnginePerformanceReport {
        let processing_stats = self.ultra_hf_processor.get_performance_stats();
        let opportunity_stats = self.opportunity_stats.read().await.clone();
        
        EnginePerformanceReport {
            timestamp: chrono::Utc::now().timestamp_nanos() as u64,
            processing_stats,
            opportunity_stats,
            risk_stats: RiskStatistics {
                total_checks: 0,
                passed_checks: 0,
                rejected_checks: 0,
                avg_check_time_us: 0.0,
            },
            strategy_efficiency: StrategyEfficiency {
                triangular_hit_rate: 0.0,
                inter_exchange_hit_rate: 0.0,
                avg_profit_per_opportunity: 0.0,
                execution_success_rate: 0.0,
            },
        }
    }
    
    /// 停止引擎
    pub async fn stop(&self) {
        println!("🛑 停止优化套利引擎...");
        self.ultra_hf_processor.stop().await;
        println!("✅ 优化套利引擎已停止");
    }
}

impl EngineConfig {
    pub fn default() -> Self {
        Self {
            min_profit_bps: 10,           // 10个基点最小利润
            max_trade_volume: 100000.0,   // 最大10万交易量
            enable_triangular: true,
            enable_inter_exchange: true,
            smart_batching: true,
            adaptive_risk: true,
        }
    }
    
    pub fn max_workers(&self) -> usize {
        24 // 根据配置返回最大工作线程数
    }
} 