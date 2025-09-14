//! QingXi 5.1 增强错误处理模块
//! 提供生产级安全的错误处理和配置管理

pub mod safe_wrapper;
pub mod safe_config;

pub use safe_wrapper::SafeWrapper;
pub use safe_config::{SafeConfigManager, QingxiSafeConfig};

// 重新导出宏
pub use crate::{safe_config, safe_execute};
