#![allow(dead_code)]
//! # qingxi 高性能行情数据采集与一致性模块 - 系统权威入口
//!
//! Production-grade, high-performance, multi-source cryptocurrency market data
//! collection, cleaning, and consistency verification system.
//!
//! 本库提供了基于权威类型系统的完整市场数据采集与处理解决方案。

// 模块声明 - 基于权威架构
pub mod adapters;
pub mod api_server;
pub mod batch;
// 🚀 阶段2优化：添加桶排序订单簿模块
pub mod bucket_orderbook;
pub mod cache;
pub mod central_manager;
pub mod circuit_breaker;
pub mod cleaner;
pub mod performance_config;
pub mod performance_optimization;
// 🚀 阶段2优化：添加性能基准测试模块
pub mod performance_benchmark;
pub mod collector;
pub mod consistency;
pub mod errors;
pub mod events;
pub mod event_bus;
pub mod health;
pub mod high_precision_time;
pub mod http_api;
pub mod lockfree;
// 🚀 V3.0高级内存管理模块
pub mod memory;
pub mod object_pool;
pub mod observability;
pub mod orderbook;
pub mod pipeline;
pub mod reasoner_client;
pub mod settings;
pub mod simd_utils;
pub mod types;

// 新增性能优化模块
pub mod simd_optimizations;
pub mod btreemap_orderbook;
pub mod dynamic_threading;
pub mod advanced_caching;

// V3.0 极限优化模块
pub mod zero_allocation_arch;      // 零分配内存架构
pub mod intel_cpu_optimizer;       // 英特尔 CPU 硬件优化
pub mod o1_sort_revolution;        // O(1) 排序算法革命
pub mod realtime_performance_monitor_simple; // 实时性能监控与自调优
pub mod v3_ultra_performance_cleaner; // V3.0 终极清洗器集成

// 综合性能管理和集成测试模块
pub mod comprehensive_performance_manager;
pub mod integration_tests;
pub mod performance_integration;

// 异常检测模块 - 使用单一文件形式，移除目录冲突
#[path = "anomaly.rs"]
pub mod anomaly;

// 权威公共API重导出 - 基于第一、二阶段的权威定义
pub use central_manager::{CentralManager, CentralManagerApi, CentralManagerHandle};
pub use errors::{MarketDataApiError, MarketDataError};
pub use types::{
    // 事件类型
    AdapterEvent,
    // 异常检测类型
    AnomalyDetectionResult,
    AnomalySeverity,
    AnomalyType,
    ConsistencyThresholds,
    HeartbeatConfig,
    MarketDataSnapshot,
    MarketSourceConfig,
    OrderBook,
    OrderBookEntry,
    Subscription,
    // 订阅和配置类型
    SubscriptionDetail,
    // 核心数据类型
    Symbol,
    TradeSide,
    TradeUpdate,
};

// 从orderbook模块导出消息类型
pub use orderbook::local_orderbook::{MarketDataMessage, OrderBookUpdate};

// 适配器接口重导出
pub use adapters::{AdapterRegistry, ExchangeAdapter};

// HTTP API 重导出
pub use http_api::{HttpApiServer, serve_http_api};

// 健康监控重导出
pub use health::{ApiHealthMonitor, HealthStatus, HealthSummary};

// 第三方类型重导出
pub use ordered_float::OrderedFloat;

// 管道系统重导出
pub use pipeline::DataPipeline;

/// 系统版本号
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 系统名称
pub const SYSTEM_NAME: &str = "qingxi-market-data";

/// 构建信息
pub fn build_info() -> String {
    format!("qingxi v{VERSION} - Production-grade market data collection system")
}
