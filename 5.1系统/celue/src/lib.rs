//! Celue高频套利交易系统主库
pub mod performance {
    pub mod simd_fixed_point;
}

// 重新导出子模块
pub use orchestrator;
pub use adapters;
pub use common;
pub use strategy;
