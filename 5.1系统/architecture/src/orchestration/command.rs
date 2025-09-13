//! 系统命令处理模块
//! 
//! 定义系统可以处理的各种命令

use serde::{Deserialize, Serialize};

/// 系统命令枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemCommand {
    /// 关闭系统
    Shutdown,
    
    /// 重启系统
    Restart,
    
    /// 启用策略
    EnableStrategy(String),
    
    /// 禁用策略
    DisableStrategy(String),
    
    /// 更新配置
    UpdateConfig(String, serde_json::Value),
    
    /// 触发资金重平衡
    TriggerRebalance,
    
    /// 强制垃圾回收
    ForceGarbageCollection,
    
    /// 切换到维护模式
    EnterMaintenanceMode,
    
    /// 退出维护模式
    ExitMaintenanceMode,
    
    /// 重置统计数据
    ResetStatistics,
    
    /// 导出数据
    ExportData {
        data_type: String,
        format: String,
        destination: String,
    },
    
    /// 执行健康检查
    PerformHealthCheck,
    
    /// 更新风险限制
    UpdateRiskLimits {
        max_exposure: Option<f64>,
        max_position: Option<f64>,
        max_daily_loss: Option<f64>,
    },
    
    /// 手动触发机会检测
    TriggerOpportunityDetection,
    
    /// 暂停交易
    PauseTrading,
    
    /// 恢复交易
    ResumeTrading,
    
    /// 清理过期数据
    CleanupExpiredData,
}

impl SystemCommand {
    /// 获取命令名称
    pub fn name(&self) -> &'static str {
        match self {
            SystemCommand::Shutdown => "shutdown",
            SystemCommand::Restart => "restart",
            SystemCommand::EnableStrategy(_) => "enable_strategy",
            SystemCommand::DisableStrategy(_) => "disable_strategy",
            SystemCommand::UpdateConfig(_, _) => "update_config",
            SystemCommand::TriggerRebalance => "trigger_rebalance",
            SystemCommand::ForceGarbageCollection => "force_gc",
            SystemCommand::EnterMaintenanceMode => "enter_maintenance",
            SystemCommand::ExitMaintenanceMode => "exit_maintenance",
            SystemCommand::ResetStatistics => "reset_statistics",
            SystemCommand::ExportData { .. } => "export_data",
            SystemCommand::PerformHealthCheck => "health_check",
            SystemCommand::UpdateRiskLimits { .. } => "update_risk_limits",
            SystemCommand::TriggerOpportunityDetection => "trigger_detection",
            SystemCommand::PauseTrading => "pause_trading",
            SystemCommand::ResumeTrading => "resume_trading",
            SystemCommand::CleanupExpiredData => "cleanup_expired",
        }
    }
    
    /// 检查命令是否需要管理员权限
    pub fn requires_admin(&self) -> bool {
        matches!(self,
            SystemCommand::Shutdown |
            SystemCommand::Restart |
            SystemCommand::UpdateConfig(_, _) |
            SystemCommand::UpdateRiskLimits { .. } |
            SystemCommand::EnterMaintenanceMode |
            SystemCommand::ExitMaintenanceMode
        )
    }
    
    /// 检查命令是否是破坏性操作
    pub fn is_destructive(&self) -> bool {
        matches!(self,
            SystemCommand::Shutdown |
            SystemCommand::Restart |
            SystemCommand::ResetStatistics |
            SystemCommand::CleanupExpiredData
        )
    }
    
    /// 获取命令描述
    pub fn description(&self) -> String {
        match self {
            SystemCommand::Shutdown => "关闭系统".to_string(),
            SystemCommand::Restart => "重启系统".to_string(),
            SystemCommand::EnableStrategy(name) => format!("启用策略: {}", name),
            SystemCommand::DisableStrategy(name) => format!("禁用策略: {}", name),
            SystemCommand::UpdateConfig(key, _) => format!("更新配置项: {}", key),
            SystemCommand::TriggerRebalance => "触发资金重平衡".to_string(),
            SystemCommand::ForceGarbageCollection => "强制垃圾回收".to_string(),
            SystemCommand::EnterMaintenanceMode => "进入维护模式".to_string(),
            SystemCommand::ExitMaintenanceMode => "退出维护模式".to_string(),
            SystemCommand::ResetStatistics => "重置统计数据".to_string(),
            SystemCommand::ExportData { data_type, .. } => format!("导出数据: {}", data_type),
            SystemCommand::PerformHealthCheck => "执行健康检查".to_string(),
            SystemCommand::UpdateRiskLimits { .. } => "更新风险限制".to_string(),
            SystemCommand::TriggerOpportunityDetection => "手动触发机会检测".to_string(),
            SystemCommand::PauseTrading => "暂停交易".to_string(),
            SystemCommand::ResumeTrading => "恢复交易".to_string(),
            SystemCommand::CleanupExpiredData => "清理过期数据".to_string(),
        }
    }
}

/// 命令执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub execution_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CommandResult {
    /// 创建成功结果
    pub fn success(message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: None,
            execution_time_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// 创建成功结果（带数据）
    pub fn success_with_data(message: &str, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
            execution_time_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// 创建失败结果
    pub fn failure(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
            execution_time_ms: 0,
            timestamp: chrono::Utc::now(),
        }
    }
    
    /// 设置执行时间
    pub fn with_execution_time(mut self, execution_time_ms: u64) -> Self {
        self.execution_time_ms = execution_time_ms;
        self
    }
} 