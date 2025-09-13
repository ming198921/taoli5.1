//! 智能告警聚合系统
//! 
//! 实现告警聚合、根因分析和智能抑制功能
//! 包含基于相似度的告警分组和贝叶斯网络根因推断

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{RwLock, Notify};
use tracing::{debug, error, info, warn, instrument};
use uuid::Uuid;

pub mod similarity;
pub mod root_cause;
pub mod suppression;
pub mod audit;

use similarity::SimilarityAnalyzer;
use root_cause::RootCauseAnalyzer;
use suppression::AlertSuppressor;
use audit::AuditLogger;

/// 告警严重性等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum AlertSeverity {
    /// 信息性告警
    Info = 0,
    /// 需要关注，但不紧急
    Low = 1,
    /// 重要问题，可能影响收益
    Medium = 2,
    /// 需要立即处理，影响交易
    High = 3,
    /// 严重问题，系统可能不可用
    Critical = 4,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Info => "info",
            AlertSeverity::Low => "low",
            AlertSeverity::Medium => "medium",
            AlertSeverity::High => "high",
            AlertSeverity::Critical => "critical",
        }
    }
    
    pub fn escalation_threshold(&self) -> Duration {
        match self {
            AlertSeverity::Critical => Duration::from_secs(60),    // 1分钟
            AlertSeverity::High => Duration::from_secs(300),       // 5分钟
            AlertSeverity::Medium => Duration::from_secs(900),     // 15分钟
            AlertSeverity::Low => Duration::from_secs(3600),       // 1小时
            AlertSeverity::Info => Duration::from_secs(86400),     // 24小时
        }
    }
}

/// 原始告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAlert {
    /// 告警ID
    pub id: String,
    /// 告警标题
    pub title: String,
    /// 告警描述
    pub description: String,
    /// 严重性
    pub severity: AlertSeverity,
    /// 来源系统
    pub source: String,
    /// 告警标签
    pub labels: HashMap<String, String>,
    /// 告警指标
    pub metrics: HashMap<String, f64>,
    /// 告警时间
    pub timestamp: DateTime<Utc>,
    /// 告警指纹（用于去重）
    pub fingerprint: String,
    /// 原始数据
    pub raw_data: Option<serde_json::Value>,
}

impl RawAlert {
    pub fn new(
        title: String,
        description: String,
        severity: AlertSeverity,
        source: String,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let fingerprint = format!("{}-{}-{}", source, title, severity.as_str());
        
        Self {
            id,
            title,
            description,
            severity,
            source,
            labels: HashMap::new(),
            metrics: HashMap::new(),
            timestamp: Utc::now(),
            fingerprint,
            raw_data: None,
        }
    }
    
    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }
    
    pub fn with_metric(mut self, key: String, value: f64) -> Self {
        self.metrics.insert(key, value);
        self
    }
    
    /// 计算告警的疲劳度评分
    pub fn calculate_fatigue_score(&self, recent_count: u32, time_span: Duration) -> f64 {
        let base_score = match self.severity {
            AlertSeverity::Critical => 0.1,
            AlertSeverity::High => 0.3,
            AlertSeverity::Medium => 0.5,
            AlertSeverity::Low => 0.7,
            AlertSeverity::Info => 0.9,
        };
        
        let frequency_factor = (recent_count as f64 / 10.0).min(1.0);
        let time_factor = (time_span.as_secs() as f64 / 3600.0).min(1.0);
        
        base_score + frequency_factor * 0.4 + time_factor * 0.1
    }
}

/// 聚合告警
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedAlert {
    /// 聚合告警ID
    pub id: String,
    /// 聚合的原始告警
    pub raw_alerts: Vec<RawAlert>,
    /// 聚合标题
    pub title: String,
    /// 聚合描述
    pub description: String,
    /// 最高严重性
    pub max_severity: AlertSeverity,
    /// 聚合规则ID
    pub rule_id: String,
    /// 聚合时间窗口
    pub time_window: Duration,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 状态
    pub status: AlertStatus,
    /// 根因分析结果
    pub root_cause_analysis: Option<RootCauseResult>,
    /// 抑制信息
    pub suppression_info: Option<SuppressionInfo>,
}

/// 告警状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// 活跃状态
    Active,
    /// 已确认
    Acknowledged,
    /// 已解决
    Resolved,
    /// 已抑制
    Suppressed,
    /// 已升级
    Escalated,
}

/// 根因分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseResult {
    /// 可能的根因
    pub probable_causes: Vec<CauseProbability>,
    /// 分析置信度
    pub confidence: f64,
    /// 建议的修复措施
    pub suggested_actions: Vec<String>,
    /// 分析时间
    pub analyzed_at: DateTime<Utc>,
}

/// 原因概率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CauseProbability {
    /// 原因描述
    pub cause: String,
    /// 概率值 [0.0, 1.0]
    pub probability: f64,
    /// 支持证据
    pub evidence: Vec<String>,
}

/// 抑制信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionInfo {
    /// 抑制原因
    pub reason: String,
    /// 抑制规则
    pub rule: String,
    /// 抑制开始时间
    pub suppressed_at: DateTime<Utc>,
    /// 抑制结束时间
    pub suppressed_until: Option<DateTime<Utc>>,
    /// 自动抑制标记
    pub automatic: bool,
}

/// 聚合规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationRule {
    /// 规则ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 匹配条件
    pub conditions: Vec<AlertCondition>,
    /// 时间窗口（秒）
    pub window_seconds: u64,
    /// 最大聚合数量
    pub max_count: u32,
    /// 执行动作
    pub action: AggregationAction,
    /// 是否启用
    pub enabled: bool,
}

/// 告警条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// 字段名
    pub field: String,
    /// 操作类型
    pub operator: ConditionOperator,
    /// 期望值
    pub value: serde_json::Value,
    /// 相似度阈值（用于模糊匹配）
    pub similarity_threshold: Option<f64>,
}

/// 条件操作符
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    Similar,
    GreaterThan,
    LessThan,
}

/// 聚合动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationAction {
    /// 聚合并发出新告警
    AggregateAndAlert,
    /// 聚合并升级
    AggregateAndEscalate,
    /// 聚合并抑制
    AggregateAndSuppress,
    /// 仅记录不发送
    LogOnly,
}

/// 告警聚合引擎
pub struct AlertAggregationEngine {
    /// 聚合规则
    rules: Arc<RwLock<HashMap<String, AggregationRule>>>,
    /// 活跃告警窗口
    active_alerts: Arc<RwLock<VecDeque<RawAlert>>>,
    /// 聚合告警缓存
    aggregated_alerts: Arc<RwLock<HashMap<String, AggregatedAlert>>>,
    /// 相似度分析器
    similarity_analyzer: Arc<SimilarityAnalyzer>,
    /// 根因分析器
    root_cause_analyzer: Arc<RootCauseAnalyzer>,
    /// 告警抑制器
    alert_suppressor: Arc<AlertSuppressor>,
    /// 审计日志记录器
    audit_logger: Arc<AuditLogger>,
    /// 配置
    config: AlertAggregationConfig,
    /// 通知器
    notify: Arc<Notify>,
}

/// 聚合引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertAggregationConfig {
    /// 最大活跃告警数量
    pub max_active_alerts: usize,
    /// 告警TTL（秒）
    pub alert_ttl_seconds: u64,
    /// 聚合处理间隔（秒）
    pub aggregation_interval_seconds: u64,
    /// 是否启用根因分析
    pub enable_root_cause_analysis: bool,
    /// 是否启用智能抑制
    pub enable_smart_suppression: bool,
    /// 相似度阈值
    pub default_similarity_threshold: f64,
}

impl Default for AlertAggregationConfig {
    fn default() -> Self {
        Self {
            max_active_alerts: 10000,
            alert_ttl_seconds: 3600, // 1小时
            aggregation_interval_seconds: 30, // 30秒
            enable_root_cause_analysis: true,
            enable_smart_suppression: true,
            default_similarity_threshold: 0.8,
        }
    }
}

impl AlertAggregationEngine {
    /// 创建新的告警聚合引擎
    pub async fn new(config: Option<AlertAggregationConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        
        let engine = Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(VecDeque::new())),
            aggregated_alerts: Arc::new(RwLock::new(HashMap::new())),
            similarity_analyzer: Arc::new(SimilarityAnalyzer::new()),
            root_cause_analyzer: Arc::new(RootCauseAnalyzer::new().await?),
            alert_suppressor: Arc::new(AlertSuppressor::new()),
            audit_logger: Arc::new(AuditLogger::new().await?),
            config,
            notify: Arc::new(Notify::new()),
        };
        
        // 加载默认聚合规则
        engine.load_default_rules().await?;
        
        // 启动后台处理任务
        engine.start_background_processing().await;
        
        info!("Alert aggregation engine initialized successfully");
        Ok(engine)
    }
    
    /// 处理新告警
    #[instrument(skip(self, alert), fields(alert_id = %alert.id, severity = ?alert.severity))]
    pub async fn process_alert(&self, alert: RawAlert) -> Result<AlertProcessingResult> {
        // 检查告警去重
        if self.is_duplicate_alert(&alert).await? {
            debug!(alert_id = %alert.id, "Duplicate alert detected, skipping");
            return Ok(AlertProcessingResult::Deduplicated);
        }
        
        // 检查抑制规则
        if let Some(suppression) = self.alert_suppressor.check_suppression(&alert).await? {
            info!(
                alert_id = %alert.id,
                reason = %suppression.reason,
                "Alert suppressed"
            );
            
            self.audit_logger.log_suppression(&alert, &suppression).await?;
            return Ok(AlertProcessingResult::Suppressed(suppression));
        }
        
        // 添加到活跃告警队列
        {
            let mut active = self.active_alerts.write().await;
            active.push_back(alert.clone());
            
            // 限制队列大小
            while active.len() > self.config.max_active_alerts {
                if let Some(old_alert) = active.pop_front() {
                    warn!(old_alert_id = %old_alert.id, "Dropping old alert due to queue limit");
                }
            }
        }
        
        // 查找匹配的聚合规则
        let matching_rules = self.find_matching_rules(&alert).await?;
        
        if matching_rules.is_empty() {
            // 没有匹配规则，作为单独告警处理
            debug!(alert_id = %alert.id, "No aggregation rules matched, processing as individual alert");
            return Ok(AlertProcessingResult::Individual(alert));
        }
        
        // 应用聚合规则
        let mut aggregation_results = Vec::new();
        for rule in matching_rules {
            let result = self.apply_aggregation_rule(&alert, &rule).await?;
            aggregation_results.push(result);
        }
        
        // 通知后台处理器
        self.notify.notify_one();
        
        Ok(AlertProcessingResult::Aggregated(aggregation_results))
    }
    
    /// 检查告警是否重复
    async fn is_duplicate_alert(&self, alert: &RawAlert) -> Result<bool> {
        let active = self.active_alerts.read().await;
        
        // 检查相同指纹的告警
        let duplicate_count = active.iter()
            .filter(|existing| existing.fingerprint == alert.fingerprint)
            .count();
        
        // 如果在短时间内有多个相同告警，认为是重复
        Ok(duplicate_count > 0)
    }
    
    /// 查找匹配的聚合规则
    async fn find_matching_rules(&self, alert: &RawAlert) -> Result<Vec<AggregationRule>> {
        let rules = self.rules.read().await;
        let mut matching_rules = Vec::new();
        
        for rule in rules.values() {
            if !rule.enabled {
                continue;
            }
            
            if self.alert_matches_rule(alert, rule).await? {
                matching_rules.push(rule.clone());
            }
        }
        
        Ok(matching_rules)
    }
    
    /// 检查告警是否匹配规则
    async fn alert_matches_rule(&self, alert: &RawAlert, rule: &AggregationRule) -> Result<bool> {
        for condition in &rule.conditions {
            if !self.evaluate_condition(alert, condition).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    /// 评估单个条件
    async fn evaluate_condition(&self, alert: &RawAlert, condition: &AlertCondition) -> Result<bool> {
        let alert_value = self.get_alert_field_value(alert, &condition.field)?;
        
        match condition.operator {
            ConditionOperator::Equal => {
                Ok(alert_value == condition.value)
            },
            ConditionOperator::NotEqual => {
                Ok(alert_value != condition.value)
            },
            ConditionOperator::Contains => {
                if let (serde_json::Value::String(alert_str), serde_json::Value::String(condition_str)) = 
                    (&alert_value, &condition.value) {
                    Ok(alert_str.contains(condition_str))
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::Similar => {
                if let Some(threshold) = condition.similarity_threshold {
                    let similarity = self.similarity_analyzer.calculate_text_similarity(
                        &alert_value.to_string(),
                        &condition.value.to_string(),
                    )?;
                    Ok(similarity >= threshold)
                } else {
                    Ok(false)
                }
            },
            ConditionOperator::Regex => {
                if let (serde_json::Value::String(alert_str), serde_json::Value::String(pattern)) = 
                    (&alert_value, &condition.value) {
                    let regex = regex::Regex::new(pattern)?;
                    Ok(regex.is_match(alert_str))
                } else {
                    Ok(false)
                }
            },
            _ => Ok(false), // 其他操作符的实现
        }
    }
    
    /// 获取告警字段值
    fn get_alert_field_value(&self, alert: &RawAlert, field: &str) -> Result<serde_json::Value> {
        match field {
            "title" => Ok(serde_json::Value::String(alert.title.clone())),
            "description" => Ok(serde_json::Value::String(alert.description.clone())),
            "severity" => Ok(serde_json::Value::String(alert.severity.as_str().to_string())),
            "source" => Ok(serde_json::Value::String(alert.source.clone())),
            _ => {
                // 尝试从标签中获取
                if let Some(value) = alert.labels.get(field) {
                    Ok(serde_json::Value::String(value.clone()))
                } else if let Some(value) = alert.metrics.get(field) {
                    Ok(serde_json::Value::Number(serde_json::Number::from_f64(*value).unwrap()))
                } else {
                    Ok(serde_json::Value::Null)
                }
            }
        }
    }
    
    /// 应用聚合规则
    async fn apply_aggregation_rule(
        &self,
        alert: &RawAlert,
        rule: &AggregationRule,
    ) -> Result<AggregationResult> {
        let window_start = Utc::now() - chrono::Duration::seconds(rule.window_seconds as i64);
        
        // 查找时间窗口内的相关告警
        let related_alerts = self.find_related_alerts(alert, rule, window_start).await?;
        
        if related_alerts.len() + 1 < rule.max_count as usize {
            // 尚未达到聚合阈值
            return Ok(AggregationResult::Pending);
        }
        
        // 创建聚合告警
        let aggregated = self.create_aggregated_alert(
            alert.clone(),
            related_alerts,
            rule,
        ).await?;
        
        // 根据规则执行动作
        match rule.action {
            AggregationAction::AggregateAndAlert => {
                self.emit_aggregated_alert(aggregated.clone()).await?;
                Ok(AggregationResult::Emitted(aggregated))
            },
            AggregationAction::AggregateAndEscalate => {
                self.escalate_aggregated_alert(aggregated.clone()).await?;
                Ok(AggregationResult::Escalated(aggregated))
            },
            AggregationAction::AggregateAndSuppress => {
                self.suppress_related_alerts(&aggregated).await?;
                Ok(AggregationResult::Suppressed(aggregated))
            },
            AggregationAction::LogOnly => {
                self.audit_logger.log_aggregation(&aggregated).await?;
                Ok(AggregationResult::Logged(aggregated))
            },
        }
    }
    
    /// 查找相关告警
    async fn find_related_alerts(
        &self,
        alert: &RawAlert,
        rule: &AggregationRule,
        since: DateTime<Utc>,
    ) -> Result<Vec<RawAlert>> {
        let active = self.active_alerts.read().await;
        let mut related = Vec::new();
        
        for existing_alert in active.iter() {
            if existing_alert.timestamp >= since &&
               existing_alert.id != alert.id &&
               self.alert_matches_rule(existing_alert, rule).await? {
                related.push(existing_alert.clone());
            }
        }
        
        Ok(related)
    }
    
    /// 创建聚合告警
    async fn create_aggregated_alert(
        &self,
        trigger_alert: RawAlert,
        related_alerts: Vec<RawAlert>,
        rule: &AggregationRule,
    ) -> Result<AggregatedAlert> {
        let mut all_alerts = vec![trigger_alert];
        all_alerts.extend(related_alerts);
        
        let max_severity = all_alerts.iter()
            .map(|a| a.severity)
            .max()
            .unwrap_or(AlertSeverity::Info);
        
        let title = format!("Aggregated Alert: {} ({} alerts)", rule.name, all_alerts.len());
        let description = format!(
            "Aggregated {} alerts matching rule '{}' in the last {} seconds",
            all_alerts.len(),
            rule.name,
            rule.window_seconds
        );
        
        let mut aggregated = AggregatedAlert {
            id: Uuid::new_v4().to_string(),
            raw_alerts: all_alerts,
            title,
            description,
            max_severity,
            rule_id: rule.id.clone(),
            time_window: Duration::from_secs(rule.window_seconds),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            status: AlertStatus::Active,
            root_cause_analysis: None,
            suppression_info: None,
        };
        
        // 执行根因分析
        if self.config.enable_root_cause_analysis {
            aggregated.root_cause_analysis = self.root_cause_analyzer
                .analyze(&aggregated).await?;
        }
        
        // 缓存聚合告警
        {
            let mut aggregated_cache = self.aggregated_alerts.write().await;
            aggregated_cache.insert(aggregated.id.clone(), aggregated.clone());
        }
        
        info!(
            aggregated_id = %aggregated.id,
            rule_id = %rule.id,
            alert_count = aggregated.raw_alerts.len(),
            max_severity = ?max_severity,
            "Created aggregated alert"
        );
        
        Ok(aggregated)
    }
    
    /// 发出聚合告警
    async fn emit_aggregated_alert(&self, alert: AggregatedAlert) -> Result<()> {
        // 这里应该发送到告警系统（如 PagerDuty, Slack 等）
        info!(
            alert_id = %alert.id,
            severity = ?alert.max_severity,
            alert_count = alert.raw_alerts.len(),
            "Emitted aggregated alert"
        );
        
        self.audit_logger.log_emission(&alert).await?;
        Ok(())
    }
    
    /// 升级聚合告警
    async fn escalate_aggregated_alert(&self, mut alert: AggregatedAlert) -> Result<()> {
        alert.status = AlertStatus::Escalated;
        alert.updated_at = Utc::now();
        
        // 这里应该执行升级逻辑
        info!(
            alert_id = %alert.id,
            "Escalated aggregated alert"
        );
        
        self.audit_logger.log_escalation(&alert).await?;
        Ok(())
    }
    
    /// 抑制相关告警
    async fn suppress_related_alerts(&self, aggregated: &AggregatedAlert) -> Result<()> {
        for raw_alert in &aggregated.raw_alerts {
            let suppression = SuppressionInfo {
                reason: format!("Suppressed by aggregation rule: {}", aggregated.rule_id),
                rule: aggregated.rule_id.clone(),
                suppressed_at: Utc::now(),
                suppressed_until: Some(Utc::now() + chrono::Duration::hours(1)),
                automatic: true,
            };
            
            self.audit_logger.log_suppression(raw_alert, &suppression).await?;
        }
        
        Ok(())
    }
    
    /// 加载默认聚合规则
    async fn load_default_rules(&self) -> Result<()> {
        let default_rules = vec![
            AggregationRule {
                id: "api_errors".to_string(),
                name: "API Error Aggregation".to_string(),
                conditions: vec![
                    AlertCondition {
                        field: "source".to_string(),
                        operator: ConditionOperator::Contains,
                        value: serde_json::Value::String("api".to_string()),
                        similarity_threshold: None,
                    },
                    AlertCondition {
                        field: "title".to_string(),
                        operator: ConditionOperator::Similar,
                        value: serde_json::Value::String("error".to_string()),
                        similarity_threshold: Some(0.8),
                    },
                ],
                window_seconds: 300,
                max_count: 10,
                action: AggregationAction::AggregateAndAlert,
                enabled: true,
            },
            AggregationRule {
                id: "market_anomaly".to_string(),
                name: "Market Anomaly Correlation".to_string(),
                conditions: vec![
                    AlertCondition {
                        field: "labels.type".to_string(),
                        operator: ConditionOperator::Equal,
                        value: serde_json::Value::String("market_anomaly".to_string()),
                        similarity_threshold: None,
                    },
                ],
                window_seconds: 600,
                max_count: 3,
                action: AggregationAction::AggregateAndEscalate,
                enabled: true,
            },
        ];
        
        let mut rules = self.rules.write().await;
        for rule in default_rules {
            rules.insert(rule.id.clone(), rule);
        }
        
        info!("Loaded {} default aggregation rules", rules.len());
        Ok(())
    }
    
    /// 启动后台处理任务
    async fn start_background_processing(&self) {
        let notify = Arc::clone(&self.notify);
        let engine = Arc::new(self);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(engine.config.aggregation_interval_seconds)
            );
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = engine.process_aggregation_cycle().await {
                            error!(error = %e, "Aggregation processing cycle failed");
                        }
                    }
                    _ = notify.notified() => {
                        if let Err(e) = engine.process_aggregation_cycle().await {
                            error!(error = %e, "Triggered aggregation processing failed");
                        }
                    }
                }
            }
        });
        
        info!("Background aggregation processing started");
    }
    
    /// 处理聚合周期
    async fn process_aggregation_cycle(&self) -> Result<()> {
        // 清理过期告警
        self.cleanup_expired_alerts().await?;
        
        // 更新聚合告警状态
        self.update_aggregated_alert_status().await?;
        
        // 执行智能抑制检查
        if self.config.enable_smart_suppression {
            self.run_smart_suppression().await?;
        }
        
        Ok(())
    }
    
    /// 清理过期告警
    async fn cleanup_expired_alerts(&self) -> Result<()> {
        let cutoff_time = Utc::now() - chrono::Duration::seconds(self.config.alert_ttl_seconds as i64);
        let mut active = self.active_alerts.write().await;
        
        let initial_count = active.len();
        active.retain(|alert| alert.timestamp > cutoff_time);
        let removed_count = initial_count - active.len();
        
        if removed_count > 0 {
            debug!(removed_count = removed_count, "Cleaned up expired alerts");
        }
        
        Ok(())
    }
    
    /// 更新聚合告警状态
    async fn update_aggregated_alert_status(&self) -> Result<()> {
        let mut aggregated = self.aggregated_alerts.write().await;
        let now = Utc::now();
        
        for alert in aggregated.values_mut() {
            // 检查是否需要自动解决
            let all_raw_alerts_old = alert.raw_alerts.iter()
                .all(|raw| now - raw.timestamp > chrono::Duration::minutes(30));
            
            if all_raw_alerts_old && alert.status == AlertStatus::Active {
                alert.status = AlertStatus::Resolved;
                alert.updated_at = now;
                info!(alert_id = %alert.id, "Auto-resolved aggregated alert");
            }
        }
        
        Ok(())
    }
    
    /// 运行智能抑制
    async fn run_smart_suppression(&self) -> Result<()> {
        let active = self.active_alerts.read().await;
        
        // 分析告警模式
        for alert in active.iter() {
            if let Some(suppression) = self.alert_suppressor
                .analyze_for_smart_suppression(alert, &*active).await? {
                info!(
                    alert_id = %alert.id,
                    reason = %suppression.reason,
                    "Applied smart suppression"
                );
                
                self.audit_logger.log_suppression(alert, &suppression).await?;
            }
        }
        
        Ok(())
    }
}

/// 告警处理结果
#[derive(Debug, Clone)]
pub enum AlertProcessingResult {
    /// 去重处理
    Deduplicated,
    /// 被抑制
    Suppressed(SuppressionInfo),
    /// 单独告警
    Individual(RawAlert),
    /// 已聚合
    Aggregated(Vec<AggregationResult>),
}

/// 聚合结果
#[derive(Debug, Clone)]
pub enum AggregationResult {
    /// 等待更多告警
    Pending,
    /// 已发出告警
    Emitted(AggregatedAlert),
    /// 已升级
    Escalated(AggregatedAlert),
    /// 已抑制
    Suppressed(AggregatedAlert),
    /// 仅记录
    Logged(AggregatedAlert),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_engine_initialization() {
        let engine = AlertAggregationEngine::new(None).await;
        assert!(engine.is_ok());
        
        let engine = engine.unwrap();
        let rules = engine.rules.read().await;
        assert!(!rules.is_empty());
    }
    
    #[test]
    async fn test_alert_creation() {
        let alert = RawAlert::new(
            "Test Alert".to_string(),
            "Test Description".to_string(),
            AlertSeverity::High,
            "test_source".to_string(),
        );
        
        assert_eq!(alert.severity, AlertSeverity::High);
        assert_eq!(alert.source, "test_source");
        assert!(!alert.id.is_empty());
    }
    
    #[test]
    async fn test_alert_processing() {
        let engine = AlertAggregationEngine::new(None).await.unwrap();
        
        let alert = RawAlert::new(
            "API Error".to_string(),
            "Connection timeout".to_string(),
            AlertSeverity::Medium,
            "api_gateway".to_string(),
        );
        
        let result = engine.process_alert(alert).await;
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Critical > AlertSeverity::High);
        assert!(AlertSeverity::High > AlertSeverity::Medium);
        assert!(AlertSeverity::Medium > AlertSeverity::Low);
        assert!(AlertSeverity::Low > AlertSeverity::Info);
    }
    
    #[test]
    fn test_fatigue_score_calculation() {
        let alert = RawAlert::new(
            "Test".to_string(),
            "Test".to_string(),
            AlertSeverity::Medium,
            "test".to_string(),
        );
        
        let score = alert.calculate_fatigue_score(5, Duration::from_secs(1800));
        assert!(score >= 0.0 && score <= 1.0);
        
        // Critical alerts should have lower base fatigue scores
        let critical_alert = RawAlert::new(
            "Critical Test".to_string(),
            "Critical Test".to_string(),
            AlertSeverity::Critical,
            "test".to_string(),
        );
        
        let critical_score = critical_alert.calculate_fatigue_score(5, Duration::from_secs(1800));
        assert!(critical_score < score);
    }
    
    #[test]
    async fn test_duplicate_detection() {
        let engine = AlertAggregationEngine::new(None).await.unwrap();
        
        let alert1 = RawAlert::new(
            "Test Alert".to_string(),
            "Description".to_string(),
            AlertSeverity::Low,
            "test_source".to_string(),
        );
        
        let alert2 = RawAlert::new(
            "Test Alert".to_string(),
            "Description".to_string(),
            AlertSeverity::Low,
            "test_source".to_string(),
        );
        
        // First alert should be processed
        let result1 = engine.process_alert(alert1).await.unwrap();
        assert!(matches!(result1, AlertProcessingResult::Individual(_)));
        
        // Second alert should be detected as duplicate
        let result2 = engine.process_alert(alert2).await.unwrap();
        assert!(matches!(result2, AlertProcessingResult::Deduplicated));
    }
}