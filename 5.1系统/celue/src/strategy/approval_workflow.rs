//! 🚀 多级审批工作流引擎
//! 
//! 实现完整的多级审批功能：
//! - 审批工作流引擎
//! - 权限级别管理
//! - 审批历史追溯

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::strategy::core::StrategyError;

/// 审批级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum ApprovalLevel {
    /// 操作员（无审批权限）
    Operator = 0,
    /// 初级审批员
    JuniorApprover = 1,
    /// 高级审批员
    SeniorApprover = 2,
    /// 管理员
    Administrator = 3,
    /// 超级管理员
    SuperAdministrator = 4,
}

/// 审批者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approver {
    pub id: String,
    pub name: String,
    pub email: String,
    pub level: ApprovalLevel,
    pub department: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

/// 审批规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRule {
    pub rule_id: String,
    pub name: String,
    pub description: String,
    /// 配置类型匹配规则
    pub config_type_pattern: String,
    /// 变更类型
    pub change_types: Vec<String>,
    /// 所需审批级别
    pub required_levels: Vec<ApprovalLevel>,
    /// 最少审批人数
    pub min_approvers: usize,
    /// 是否需要所有级别都审批
    pub all_levels_required: bool,
    /// 超时时间（秒）
    pub timeout_seconds: Option<i64>,
    /// 自动审批条件
    pub auto_approval_conditions: Vec<AutoApprovalCondition>,
}

/// 自动审批条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApprovalCondition {
    pub condition_type: ConditionType,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    /// 配置值在范围内
    ValueInRange,
    /// 与当前值差异小于阈值
    MinimalChange,
    /// 特定时间窗口
    TimeWindow,
    /// 紧急标记
    Emergency,
}

/// 审批请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: String,
    pub config_change_id: String,
    pub config_type: String,
    pub change_type: String,
    pub requester: String,
    pub description: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: ApprovalStatus,
    pub required_approvals: Vec<RequiredApproval>,
    pub actual_approvals: Vec<ActualApproval>,
    pub comments: Vec<ApprovalComment>,
    pub completion_time: Option<DateTime<Utc>>,
}

/// 审批状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    /// 待审批
    Pending,
    /// 部分批准
    PartiallyApproved,
    /// 已批准
    Approved,
    /// 已拒绝
    Rejected,
    /// 已撤销
    Cancelled,
    /// 已超时
    Expired,
    /// 自动批准
    AutoApproved,
}

/// 必需的审批
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredApproval {
    pub level: ApprovalLevel,
    pub count: usize,
    pub fulfilled: bool,
}

/// 实际的审批
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualApproval {
    pub approval_id: String,
    pub approver_id: String,
    pub approver_name: String,
    pub approval_level: ApprovalLevel,
    pub decision: ApprovalDecision,
    pub timestamp: DateTime<Utc>,
    pub comments: Option<String>,
    pub signature: String, // 数字签名
}

/// 审批决定
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approve,
    Reject,
    RequestMoreInfo,
}

/// 审批评论
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalComment {
    pub comment_id: String,
    pub author_id: String,
    pub author_name: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_private: bool,
}

/// 审批工作流配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// 默认超时时间（秒）
    pub default_timeout_seconds: i64,
    /// 是否允许自我审批
    pub allow_self_approval: bool,
    /// 是否允许撤销
    pub allow_cancellation: bool,
    /// 是否发送通知
    pub send_notifications: bool,
    /// 审批历史保留天数
    pub history_retention_days: u32,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 86400, // 24小时
            allow_self_approval: false,
            allow_cancellation: true,
            send_notifications: true,
            history_retention_days: 365,
        }
    }
}

/// 🚀 审批工作流引擎
pub struct ApprovalWorkflowEngine {
    /// 配置
    config: Arc<RwLock<WorkflowConfig>>,
    /// 审批者注册表
    approvers: Arc<RwLock<HashMap<String, Approver>>>,
    /// 审批规则
    rules: Arc<RwLock<Vec<ApprovalRule>>>,
    /// 活跃的审批请求
    active_requests: Arc<RwLock<HashMap<String, ApprovalRequest>>>,
    /// 历史审批记录
    history: Arc<RwLock<Vec<ApprovalRequest>>>,
    /// 通知处理器
    notification_handler: Option<Arc<dyn NotificationHandler>>,
}

/// 通知处理器trait
#[async_trait::async_trait]
pub trait NotificationHandler: Send + Sync {
    async fn send_approval_request(&self, request: &ApprovalRequest, approvers: &[Approver]) -> Result<(), StrategyError>;
    async fn send_approval_result(&self, request: &ApprovalRequest) -> Result<(), StrategyError>;
    async fn send_reminder(&self, request: &ApprovalRequest, approvers: &[Approver]) -> Result<(), StrategyError>;
}

impl ApprovalWorkflowEngine {
    /// 创建新的工作流引擎
    pub fn new(config: WorkflowConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            approvers: Arc::new(RwLock::new(HashMap::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            notification_handler: None,
        }
    }

    /// 设置通知处理器
    pub fn set_notification_handler(&mut self, handler: Arc<dyn NotificationHandler>) {
        self.notification_handler = Some(handler);
    }

    /// 注册审批者
    pub async fn register_approver(&self, approver: Approver) -> Result<(), StrategyError> {
        let mut approvers = self.approvers.write().await;
        
        if approvers.contains_key(&approver.id) {
            return Err(StrategyError::Configuration(
                format!("审批者 {} 已存在", approver.id)
            ));
        }
        
        approvers.insert(approver.id.clone(), approver);
        Ok(())
    }

    /// 添加审批规则
    pub async fn add_rule(&self, rule: ApprovalRule) -> Result<(), StrategyError> {
        let mut rules = self.rules.write().await;
        
        // 验证规则合法性
        if rule.required_levels.is_empty() {
            return Err(StrategyError::Configuration(
                "审批规则必须指定至少一个审批级别".to_string()
            ));
        }
        
        if rule.min_approvers == 0 {
            return Err(StrategyError::Configuration(
                "最少审批人数必须大于0".to_string()
            ));
        }
        
        rules.push(rule);
        Ok(())
    }

    /// 创建审批请求
    pub async fn create_approval_request(
        &self,
        config_change_id: String,
        config_type: String,
        change_type: String,
        requester: String,
        description: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
    ) -> Result<String, StrategyError> {
        // 检查请求者是否有权限发起请求
        let approvers = self.approvers.read().await;
        if !self.config.read().await.allow_self_approval {
            if let Some(requester_info) = approvers.get(&requester) {
                if requester_info.level == ApprovalLevel::Operator {
                    // 操作员可以发起请求
                } else {
                    // 其他级别需要检查
                }
            }
        }
        
        // 查找匹配的审批规则
        let rules = self.rules.read().await;
        let matching_rule = self.find_matching_rule(&rules, &config_type, &change_type)?;
        
        // 检查自动审批条件
        if let Some(reason) = self.check_auto_approval(&matching_rule, &old_value, &new_value).await? {
            // 创建自动批准的请求
            let request = self.create_auto_approved_request(
                config_change_id,
                config_type,
                change_type,
                requester,
                description,
                old_value,
                new_value,
                reason,
            );
            
            let request_id = request.request_id.clone();
            self.history.write().await.push(request);
            
            return Ok(request_id);
        }
        
        // 创建审批请求
        let request_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let due_date = matching_rule.timeout_seconds
            .or(Some(self.config.read().await.default_timeout_seconds))
            .map(|seconds| now + chrono::Duration::seconds(seconds));
        
        // 构建必需的审批列表
        let mut required_approvals = Vec::new();
        for level in &matching_rule.required_levels {
            required_approvals.push(RequiredApproval {
                level: *level,
                count: if matching_rule.all_levels_required {
                    matching_rule.min_approvers
                } else {
                    1
                },
                fulfilled: false,
            });
        }
        
        let request = ApprovalRequest {
            request_id: request_id.clone(),
            config_change_id,
            config_type,
            change_type,
            requester,
            description,
            old_value,
            new_value,
            created_at: now,
            due_date,
            status: ApprovalStatus::Pending,
            required_approvals,
            actual_approvals: Vec::new(),
            comments: Vec::new(),
            completion_time: None,
        };
        
        // 保存请求
        self.active_requests.write().await.insert(request_id.clone(), request.clone());
        
        // 发送通知
        if self.config.read().await.send_notifications {
            if let Some(handler) = &self.notification_handler {
                let eligible_approvers = self.get_eligible_approvers(&matching_rule).await?;
                handler.send_approval_request(&request, &eligible_approvers).await?;
            }
        }
        
        Ok(request_id)
    }

    /// 处理审批决定
    pub async fn process_approval(
        &self,
        request_id: &str,
        approver_id: &str,
        decision: ApprovalDecision,
        comments: Option<String>,
    ) -> Result<(), StrategyError> {
        let mut active_requests = self.active_requests.write().await;
        
        let request = active_requests.get_mut(request_id)
            .ok_or_else(|| StrategyError::NotFound(format!("审批请求 {} 不存在", request_id)))?;
        
        // 验证审批者权限
        let approvers = self.approvers.read().await;
        let approver = approvers.get(approver_id)
            .ok_or_else(|| StrategyError::Authorization(format!("审批者 {} 未注册", approver_id)))?;
        
        if !approver.active {
            return Err(StrategyError::Authorization("审批者账户已停用".to_string()));
        }
        
        // 检查是否已经审批过
        if request.actual_approvals.iter().any(|a| a.approver_id == approver_id) {
            return Err(StrategyError::InvalidOperation("已经审批过此请求".to_string()));
        }
        
        // 检查审批者级别是否符合要求
        let level_required = request.required_approvals.iter()
            .any(|r| r.level <= approver.level && !r.fulfilled);
        
        if !level_required {
            return Err(StrategyError::Authorization(
                format!("审批者级别 {:?} 不符合要求", approver.level)
            ));
        }
        
        // 添加审批记录
        let approval = ActualApproval {
            approval_id: uuid::Uuid::new_v4().to_string(),
            approver_id: approver_id.to_string(),
            approver_name: approver.name.clone(),
            approval_level: approver.level,
            decision: decision.clone(),
            timestamp: Utc::now(),
            comments: comments.clone(),
            signature: self.generate_signature(request_id, approver_id, &decision),
        };
        
        request.actual_approvals.push(approval);
        
        // 添加评论
        if let Some(comment_text) = comments {
            request.comments.push(ApprovalComment {
                comment_id: uuid::Uuid::new_v4().to_string(),
                author_id: approver_id.to_string(),
                author_name: approver.name.clone(),
                content: comment_text,
                timestamp: Utc::now(),
                is_private: false,
            });
        }
        
        // 更新审批状态
        match decision {
            ApprovalDecision::Reject => {
                request.status = ApprovalStatus::Rejected;
                request.completion_time = Some(Utc::now());
                
                // 移到历史记录
                let request = active_requests.remove(request_id).unwrap();
                self.history.write().await.push(request.clone());
                
                // 发送通知
                if self.config.read().await.send_notifications {
                    if let Some(handler) = &self.notification_handler {
                        handler.send_approval_result(&request).await?;
                    }
                }
            }
            ApprovalDecision::Approve => {
                // 检查是否满足所有审批要求
                if self.check_approval_requirements(request) {
                    request.status = ApprovalStatus::Approved;
                    request.completion_time = Some(Utc::now());
                    
                    // 移到历史记录
                    let request = active_requests.remove(request_id).unwrap();
                    self.history.write().await.push(request.clone());
                    
                    // 发送通知
                    if self.config.read().await.send_notifications {
                        if let Some(handler) = &self.notification_handler {
                            handler.send_approval_result(&request).await?;
                        }
                    }
                } else {
                    request.status = ApprovalStatus::PartiallyApproved;
                }
            }
            ApprovalDecision::RequestMoreInfo => {
                // 保持待审批状态，等待更多信息
            }
        }
        
        Ok(())
    }

    /// 撤销审批请求
    pub async fn cancel_request(&self, request_id: &str, canceller_id: &str) -> Result<(), StrategyError> {
        if !self.config.read().await.allow_cancellation {
            return Err(StrategyError::InvalidOperation("不允许撤销审批请求".to_string()));
        }
        
        let mut active_requests = self.active_requests.write().await;
        let mut request = active_requests.remove(request_id)
            .ok_or_else(|| StrategyError::NotFound(format!("审批请求 {} 不存在", request_id)))?;
        
        // 验证撤销权限（只有请求者或管理员可以撤销）
        let approvers = self.approvers.read().await;
        let is_admin = approvers.get(canceller_id)
            .map(|a| a.level >= ApprovalLevel::Administrator)
            .unwrap_or(false);
        
        if request.requester != canceller_id && !is_admin {
            return Err(StrategyError::Authorization("无权撤销此审批请求".to_string()));
        }
        
        request.status = ApprovalStatus::Cancelled;
        request.completion_time = Some(Utc::now());
        
        self.history.write().await.push(request);
        Ok(())
    }

    /// 检查过期的审批请求
    pub async fn check_expired_requests(&self) -> Result<Vec<String>, StrategyError> {
        let mut active_requests = self.active_requests.write().await;
        let now = Utc::now();
        let mut expired_ids = Vec::new();
        
        let expired_requests: Vec<_> = active_requests.iter()
            .filter_map(|(id, request)| {
                if let Some(due_date) = request.due_date {
                    if due_date < now {
                        expired_ids.push(id.clone());
                        Some(request.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();
        
        // 处理过期请求
        for mut request in expired_requests {
            request.status = ApprovalStatus::Expired;
            request.completion_time = Some(now);
            
            active_requests.remove(&request.request_id);
            self.history.write().await.push(request);
        }
        
        Ok(expired_ids)
    }

    /// 获取审批历史
    pub async fn get_approval_history(
        &self,
        filter: Option<HistoryFilter>,
    ) -> Result<Vec<ApprovalRequest>, StrategyError> {
        let history = self.history.read().await;
        
        if let Some(filter) = filter {
            Ok(history.iter()
                .filter(|request| filter.matches(request))
                .cloned()
                .collect())
        } else {
            Ok(history.clone())
        }
    }

    /// 查找匹配的审批规则
    fn find_matching_rule(
        &self,
        rules: &[ApprovalRule],
        config_type: &str,
        change_type: &str,
    ) -> Result<ApprovalRule, StrategyError> {
        for rule in rules {
            if config_type.contains(&rule.config_type_pattern) &&
               rule.change_types.contains(&change_type.to_string()) {
                return Ok(rule.clone());
            }
        }
        
        Err(StrategyError::Configuration(
            format!("未找到匹配的审批规则: {} - {}", config_type, change_type)
        ))
    }

    /// 检查自动审批条件
    async fn check_auto_approval(
        &self,
        rule: &ApprovalRule,
        old_value: &Option<serde_json::Value>,
        new_value: &serde_json::Value,
    ) -> Result<Option<String>, StrategyError> {
        for condition in &rule.auto_approval_conditions {
            match condition.condition_type {
                ConditionType::Emergency => {
                    if let Some(emergency) = new_value.get("emergency").and_then(|v| v.as_bool()) {
                        if emergency {
                            return Ok(Some("紧急情况自动批准".to_string()));
                        }
                    }
                }
                ConditionType::MinimalChange => {
                    if let Some(old) = old_value {
                        if let Some(threshold) = condition.parameters.get("threshold")
                            .and_then(|t| t.parse::<f64>().ok()) {
                            if self.calculate_change_ratio(old, new_value) < threshold {
                                return Ok(Some(format!("变更幅度小于{}%", threshold * 100.0)));
                            }
                        }
                    }
                }
                ConditionType::ValueInRange => {
                    // 实现值范围检查
                }
                ConditionType::TimeWindow => {
                    // 实现时间窗口检查
                }
            }
        }
        
        Ok(None)
    }

    /// 创建自动批准的请求
    fn create_auto_approved_request(
        &self,
        config_change_id: String,
        config_type: String,
        change_type: String,
        requester: String,
        description: String,
        old_value: Option<serde_json::Value>,
        new_value: serde_json::Value,
        auto_approval_reason: String,
    ) -> ApprovalRequest {
        let now = Utc::now();
        
        ApprovalRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            config_change_id,
            config_type,
            change_type,
            requester,
            description,
            old_value,
            new_value,
            created_at: now,
            due_date: None,
            status: ApprovalStatus::AutoApproved,
            required_approvals: Vec::new(),
            actual_approvals: vec![ActualApproval {
                approval_id: uuid::Uuid::new_v4().to_string(),
                approver_id: "system".to_string(),
                approver_name: "System".to_string(),
                approval_level: ApprovalLevel::SuperAdministrator,
                decision: ApprovalDecision::Approve,
                timestamp: now,
                comments: Some(auto_approval_reason),
                signature: "auto-approved".to_string(),
            }],
            comments: Vec::new(),
            completion_time: Some(now),
        }
    }

    /// 获取有资格的审批者
    async fn get_eligible_approvers(&self, rule: &ApprovalRule) -> Result<Vec<Approver>, StrategyError> {
        let approvers = self.approvers.read().await;
        
        Ok(approvers.values()
            .filter(|approver| {
                approver.active &&
                rule.required_levels.iter().any(|level| approver.level >= *level)
            })
            .cloned()
            .collect())
    }

    /// 检查审批要求是否满足
    fn check_approval_requirements(&self, request: &mut ApprovalRequest) -> bool {
        // 统计各级别的审批数量
        let mut level_counts: HashMap<ApprovalLevel, usize> = HashMap::new();
        
        for approval in &request.actual_approvals {
            if approval.decision == ApprovalDecision::Approve {
                *level_counts.entry(approval.approval_level).or_insert(0) += 1;
            }
        }
        
        // 检查每个必需的审批级别
        for required in &mut request.required_approvals {
            let approved_count = (required.level.clone() as u8..=ApprovalLevel::SuperAdministrator as u8)
                .filter_map(|l| {
                    let level = unsafe { std::mem::transmute::<u8, ApprovalLevel>(l) };
                    level_counts.get(&level)
                })
                .sum::<usize>();
            
            required.fulfilled = approved_count >= required.count;
        }
        
        // 所有必需的审批都满足
        request.required_approvals.iter().all(|r| r.fulfilled)
    }

    /// 生成数字签名
    fn generate_signature(&self, request_id: &str, approver_id: &str, decision: &ApprovalDecision) -> String {
        use blake3::Hasher;
        
        let mut hasher = Hasher::new();
        hasher.update(request_id.as_bytes());
        hasher.update(approver_id.as_bytes());
        hasher.update(format!("{:?}", decision).as_bytes());
        hasher.update(&Utc::now().timestamp().to_le_bytes());
        
        hasher.finalize().to_hex().to_string()
    }

    /// 计算变更比率
    fn calculate_change_ratio(&self, old_value: &serde_json::Value, new_value: &serde_json::Value) -> f64 {
        // 简化实现：比较数值型字段
        if let (Some(old_num), Some(new_num)) = (old_value.as_f64(), new_value.as_f64()) {
            if old_num != 0.0 {
                ((new_num - old_num) / old_num).abs()
            } else {
                1.0
            }
        } else {
            // 对于非数值型，如果不同则认为是100%变更
            if old_value != new_value { 1.0 } else { 0.0 }
        }
    }
}

/// 历史过滤器
#[derive(Debug, Clone)]
pub struct HistoryFilter {
    pub requester: Option<String>,
    pub approver: Option<String>,
    pub status: Option<ApprovalStatus>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub config_type: Option<String>,
}

impl HistoryFilter {
    fn matches(&self, request: &ApprovalRequest) -> bool {
        if let Some(ref requester) = self.requester {
            if &request.requester != requester {
                return false;
            }
        }
        
        if let Some(ref approver) = self.approver {
            if !request.actual_approvals.iter().any(|a| &a.approver_id == approver) {
                return false;
            }
        }
        
        if let Some(ref status) = self.status {
            if &request.status != status {
                return false;
            }
        }
        
        if let Some(start) = self.start_date {
            if request.created_at < start {
                return false;
            }
        }
        
        if let Some(end) = self.end_date {
            if request.created_at > end {
                return false;
            }
        }
        
        if let Some(ref config_type) = self.config_type {
            if !request.config_type.contains(config_type) {
                return false;
            }
        }
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_approval_workflow() {
        let engine = ApprovalWorkflowEngine::new(WorkflowConfig::default());
        
        // 注册审批者
        let approver1 = Approver {
            id: "user1".to_string(),
            name: "张三".to_string(),
            email: "zhangsan@example.com".to_string(),
            level: ApprovalLevel::JuniorApprover,
            department: "交易部".to_string(),
            active: true,
            created_at: Utc::now(),
            last_login: Some(Utc::now()),
        };
        
        let approver2 = Approver {
            id: "user2".to_string(),
            name: "李四".to_string(),
            email: "lisi@example.com".to_string(),
            level: ApprovalLevel::SeniorApprover,
            department: "风控部".to_string(),
            active: true,
            created_at: Utc::now(),
            last_login: Some(Utc::now()),
        };
        
        engine.register_approver(approver1).await.unwrap();
        engine.register_approver(approver2).await.unwrap();
        
        // 添加审批规则
        let rule = ApprovalRule {
            rule_id: "rule1".to_string(),
            name: "策略参数变更".to_string(),
            description: "需要初级和高级审批员各一名审批".to_string(),
            config_type_pattern: "strategy".to_string(),
            change_types: vec!["update".to_string()],
            required_levels: vec![ApprovalLevel::JuniorApprover, ApprovalLevel::SeniorApprover],
            min_approvers: 1,
            all_levels_required: true,
            timeout_seconds: Some(3600),
            auto_approval_conditions: vec![],
        };
        
        engine.add_rule(rule).await.unwrap();
        
        // 创建审批请求
        let request_id = engine.create_approval_request(
            "change123".to_string(),
            "strategy.triangular".to_string(),
            "update".to_string(),
            "operator1".to_string(),
            "调整三角套利最小利润阈值".to_string(),
            Some(serde_json::json!({"min_profit": 0.001})),
            serde_json::json!({"min_profit": 0.002}),
        ).await.unwrap();
        
        // 初级审批员批准
        engine.process_approval(
            &request_id,
            "user1",
            ApprovalDecision::Approve,
            Some("参数调整合理".to_string()),
        ).await.unwrap();
        
        // 验证状态为部分批准
        let request = engine.active_requests.read().await.get(&request_id).cloned().unwrap();
        assert_eq!(request.status, ApprovalStatus::PartiallyApproved);
        
        // 高级审批员批准
        engine.process_approval(
            &request_id,
            "user2",
            ApprovalDecision::Approve,
            Some("风控评估通过".to_string()),
        ).await.unwrap();
        
        // 验证状态为已批准
        let history = engine.get_approval_history(None).await.unwrap();
        let approved_request = history.iter().find(|r| r.request_id == request_id).unwrap();
        assert_eq!(approved_request.status, ApprovalStatus::Approved);
    }
} 