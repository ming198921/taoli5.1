//! 🚀 审批工作流集成层
//! 
//! 功能：
//! - 将审批工作流集成到主系统
//! - 配置变更拦截
//! - 自动审批触发
//! - 审批状态通知

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::strategy::approval_workflow::{
    ApprovalWorkflowEngine, ApprovalRequest, ApprovalDecision, ApprovalLevel
};
use crate::strategy::core::StrategyError;

/// 配置变更事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    pub change_id: String,
    pub config_type: String,
    pub change_type: String,
    pub requester: String,
    pub description: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub requires_approval: bool,
}

/// 审批状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    AutoApproved,
}

/// 审批集成管理器
pub struct ApprovalIntegration {
    workflow_engine: Arc<ApprovalWorkflowEngine>,
    pending_changes: Arc<RwLock<HashMap<String, ConfigChangeEvent>>>,
    approval_status: Arc<RwLock<HashMap<String, ApprovalStatus>>>,
}

impl ApprovalIntegration {
    /// 创建新的审批集成管理器
    pub fn new(workflow_engine: Arc<ApprovalWorkflowEngine>) -> Self {
        Self {
            workflow_engine,
            pending_changes: Arc::new(RwLock::new(HashMap::new())),
            approval_status: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 拦截配置变更并启动审批流程
    pub async fn intercept_config_change(
        &self,
        event: ConfigChangeEvent,
    ) -> Result<String, StrategyError> {
        let change_id = event.change_id.clone();
        
        tracing::info!("拦截配置变更: {} ({})", change_id, event.config_type);

        // 判断是否需要审批
        if !event.requires_approval || self.is_auto_approvable(&event).await? {
            // 直接批准
            let mut status = self.approval_status.write().await;
            status.insert(change_id.clone(), ApprovalStatus::AutoApproved);
            
            tracing::info!("配置变更 {} 自动批准", change_id);
            return Ok(change_id);
        }

        // 存储待审批变更
        let mut pending = self.pending_changes.write().await;
        pending.insert(change_id.clone(), event.clone());

        // 设置初始状态
        let mut status = self.approval_status.write().await;
        status.insert(change_id.clone(), ApprovalStatus::Pending);

        // 创建审批请求
        let request_id = self.workflow_engine.create_approval_request(
            event.change_id,
            event.config_type,
            event.change_type,
            event.requester,
            event.description,
            event.old_value,
            event.new_value,
        ).await?;

        tracing::info!("创建审批请求: {} -> {}", change_id, request_id);
        Ok(request_id)
    }

    /// 处理审批结果
    pub async fn handle_approval_result(
        &self,
        request_id: &str,
        decision: ApprovalDecision,
    ) -> Result<(), StrategyError> {
        // 更新审批状态
        let status = match decision {
            ApprovalDecision::Approved => ApprovalStatus::Approved,
            ApprovalDecision::Rejected => ApprovalStatus::Rejected,
        };

        let mut approval_status = self.approval_status.write().await;
        approval_status.insert(request_id.to_string(), status.clone());

        match status {
            ApprovalStatus::Approved => {
                tracing::info!("配置变更 {} 已批准，执行变更", request_id);
                self.execute_approved_change(request_id).await?;
            }
            ApprovalStatus::Rejected => {
                tracing::info!("配置变更 {} 被拒绝", request_id);
                self.cleanup_rejected_change(request_id).await?;
            }
            _ => {}
        }

        Ok(())
    }

    /// 检查是否可以自动批准
    async fn is_auto_approvable(&self, event: &ConfigChangeEvent) -> Result<bool, StrategyError> {
        // 低风险变更可以自动批准
        match event.config_type.as_str() {
            "monitoring" | "logging" => Ok(true),
            "performance" => {
                // 性能参数的小幅调整可以自动批准
                if let Some(old_val) = &event.old_value {
                    if let (Ok(old_f64), Ok(new_f64)) = (
                        old_val.as_f64().ok_or(0.0),
                        event.new_value.as_f64().unwrap_or(0.0)
                    ) {
                        let change_percentage = ((new_f64 - old_f64) / old_f64).abs();
                        return Ok(change_percentage < 0.1); // 10%以内的变化
                    }
                }
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    /// 执行已批准的变更
    async fn execute_approved_change(&self, change_id: &str) -> Result<(), StrategyError> {
        let pending = self.pending_changes.read().await;
        if let Some(change) = pending.get(change_id) {
            tracing::info!("执行已批准的配置变更: {}", change_id);
            // 这里应该调用实际的配置更新逻辑
            // 例如：config_center.update_config(change.config_type, change.new_value)
        }
        Ok(())
    }

    /// 清理被拒绝的变更
    async fn cleanup_rejected_change(&self, change_id: &str) -> Result<(), StrategyError> {
        let mut pending = self.pending_changes.write().await;
        pending.remove(change_id);
        tracing::info!("清理被拒绝的配置变更: {}", change_id);
        Ok(())
    }

    /// 获取待审批变更列表
    pub async fn get_pending_changes(&self) -> Result<Vec<ConfigChangeEvent>, StrategyError> {
        let pending = self.pending_changes.read().await;
        Ok(pending.values().cloned().collect())
    }

    /// 获取审批状态
    pub async fn get_approval_status(&self, change_id: &str) -> Result<ApprovalStatus, StrategyError> {
        let status = self.approval_status.read().await;
        Ok(status.get(change_id)
            .cloned()
            .unwrap_or(ApprovalStatus::Pending))
    }

    /// 清理过期的审批请求
    pub async fn cleanup_expired_requests(&self) -> Result<(), StrategyError> {
        let now = Utc::now();
        let mut pending = self.pending_changes.write().await;
        let mut status = self.approval_status.write().await;

        let expired_keys: Vec<String> = pending.iter()
            .filter(|(_, event)| {
                let age = now - event.timestamp;
                age.num_hours() > 24 // 24小时过期
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            pending.remove(&key);
            status.insert(key.clone(), ApprovalStatus::Expired);
            tracing::info!("清理过期审批请求: {}", key);
        }

        Ok(())
    }
}

/// 审批事件监听器
pub struct ApprovalEventListener {
    integration: Arc<ApprovalIntegration>,
}

impl ApprovalEventListener {
    pub fn new(integration: Arc<ApprovalIntegration>) -> Self {
        Self { integration }
    }

    /// 启动事件监听
    pub async fn start_listening(&self) -> Result<(), StrategyError> {
        let integration = Arc::clone(&self.integration);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5分钟
            
            loop {
                interval.tick().await;
                
                if let Err(e) = integration.cleanup_expired_requests().await {
                    tracing::error!("清理过期审批请求失败: {}", e);
                }
            }
        });

        tracing::info!("审批事件监听器已启动");
        Ok(())
    }
}

/// 审批通知器
pub struct ApprovalNotifier {
    // 通知实现
}

impl ApprovalNotifier {
    pub fn new() -> Self {
        Self {}
    }

    /// 发送审批通知
    pub async fn notify_approval_required(
        &self,
        request: &ApprovalRequest,
        approvers: Vec<String>,
    ) -> Result<(), StrategyError> {
        for approver in approvers {
            tracing::info!("发送审批通知给 {}: 请求 {}", approver, request.request_id);
            // 实际实现应该发送邮件/短信/系统通知
        }
        Ok(())
    }

    /// 发送审批结果通知
    pub async fn notify_approval_result(
        &self,
        request_id: &str,
        decision: ApprovalDecision,
        requester: &str,
    ) -> Result<(), StrategyError> {
        tracing::info!("发送审批结果通知给 {}: 请求 {} {}", 
            requester, request_id, 
            match decision {
                ApprovalDecision::Approved => "已批准",
                ApprovalDecision::Rejected => "被拒绝",
            }
        );
        Ok(())
    }
} 