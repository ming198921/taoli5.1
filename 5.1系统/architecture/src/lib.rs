//! # 高频虚拟货币套利系统5.1++ 核心架构
//! 
//! 本模块实现了一个完整的分布式高频套利系统架构，包含：
//! - 系统协调层：事件总线、配置中心、全局机会池
//! - 数据采集层：多源数据采集、清洗、质量检查、分发
//! - 核心业务层：策略引擎、风险管理、执行引擎、资金管理
//! - 外部接口层：交易所API、管理接口
//! - 存储持久化层：Redis、PostgreSQL、ClickHouse

pub mod errors;
pub mod types;
pub mod config;
pub mod orchestration;
pub mod data;
pub mod business;
pub mod interfaces;
pub mod storage;
pub mod runtime_enforcement;
pub mod enforcement_integration;

// 重新导出核心类型
pub use errors::*;
pub use types::*;
pub use orchestration::*;
pub use data::*;
pub use business::*;
pub use interfaces::*;
pub use storage::*;
pub use runtime_enforcement::*;

// 主系统入口
pub use orchestration::ArbitrageSystemOrchestrator;

/// 系统版本信息
pub const VERSION: &str = "5.1.0";
pub const BUILD_TIMESTAMP: &str = "2024-08-25T14:30:00Z";
pub const GIT_SHA: &str = "development";

/// 系统配置文件路径常量
pub const DEFAULT_CONFIG_PATH: &str = "./config/system.toml";
pub const DEFAULT_SECRETS_PATH: &str = "./config/secrets.toml";
pub const DEFAULT_EXCHANGES_PATH: &str = "./config/exchanges.toml";

/// 系统限制常量
pub const MAX_SUPPORTED_EXCHANGES: usize = 20;
pub const MAX_SUPPORTED_SYMBOLS: usize = 50;
pub const MAX_CONCURRENT_OPPORTUNITIES: usize = 1000;
pub const MAX_ORDER_BATCH_SIZE: usize = 50;

/// 性能基准常量
pub const TARGET_LATENCY_MICROSECONDS: u64 = 500;
pub const TARGET_THROUGHPUT_OPS_PER_SEC: u64 = 10000;
pub const TARGET_SUCCESS_RATE_PERCENT: f64 = 99.9;

/// 系统健康检查间隔
pub const HEALTH_CHECK_INTERVAL_SECONDS: u64 = 30;
pub const API_HEALTH_CHECK_INTERVAL_SECONDS: u64 = 10;
pub const METRICS_COLLECTION_INTERVAL_SECONDS: u64 = 5; 