//! 系统协调层模块
//! 
//! 负责系统的统一协调管理，包括：
//! - ArbitrageSystemOrchestrator: 主系统协调器
//! - EventBus: 事件总线
//! - GlobalOpportunityPool: 全局机会池
//! - SystemCommand: 系统命令处理

pub mod orchestrator;
pub mod event_bus;
pub mod opportunity_pool;
pub mod command;

pub use orchestrator::*;
pub use event_bus::*;
pub use opportunity_pool::*;
pub use command::*; 