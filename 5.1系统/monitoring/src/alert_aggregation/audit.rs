//! Audit logging and compliance tracking for alert aggregation

use crate::types::{Alert, AlertId, AlertSeverity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: AuditEventType,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub source_ip: Option<String>,
    pub alert_id: Option<AlertId>,
    pub details: serde_json::Value,
    pub severity: AuditSeverity,
    pub compliance_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    AlertCreated,
    AlertAggregated,
    AlertSuppressed,
    AlertEscalated,
    AlertAcknowledged,
    AlertResolved,
    RuleCreated,
    RuleUpdated,
    RuleDeleted,
    ConfigurationChanged,
    UserAuthenticated,
    AccessDenied,
    DataExported,
    SystemMaintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum AuditSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub rule_id: String,
    pub rule_name: String,
    pub regulation: String,
    pub description: String,
    pub required_fields: Vec<String>,
    pub retention_days: u32,
    pub encryption_required: bool,
    pub access_controls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub policy_id: String,
    pub event_types: Vec<AuditEventType>,
    pub retention_days: u32,
    pub archive_after_days: u32,
    pub purge_after_days: u32,
    pub compression_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditQuery {
    pub event_types: Option<Vec<AuditEventType>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub alert_id: Option<String>,
    pub severity: Option<AuditSeverity>,
    pub compliance_tags: Option<Vec<String>>,
    pub limit: Option<usize>,
}

pub struct AuditLogger {
    audit_file_path: String,
    compliance_rules: Arc<RwLock<HashMap<String, ComplianceRule>>>,
    retention_policies: Arc<RwLock<HashMap<String, RetentionPolicy>>>,
    audit_buffer: Arc<Mutex<Vec<AuditEvent>>>,
    buffer_size: usize,
    file_writer: Arc<Mutex<Option<BufWriter<tokio::fs::File>>>>,
    encryption_enabled: bool,
    audit_trail: Arc<RwLock<Vec<AuditEvent>>>,
    max_memory_events: usize,
}

impl AuditLogger {
    pub async fn new(
        audit_file_path: String,
        buffer_size: usize,
        encryption_enabled: bool,
        max_memory_events: usize,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut logger = Self {
            audit_file_path: audit_file_path.clone(),
            compliance_rules: Arc::new(RwLock::new(HashMap::new())),
            retention_policies: Arc::new(RwLock::new(HashMap::new())),
            audit_buffer: Arc::new(Mutex::new(Vec::new())),
            buffer_size,
            file_writer: Arc::new(Mutex::new(None)),
            encryption_enabled,
            audit_trail: Arc::new(RwLock::new(Vec::new())),
            max_memory_events,
        };

        logger.initialize_file_writer().await?;
        logger.setup_default_policies().await;
        
        Ok(logger)
    }

    async fn initialize_file_writer(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.audit_file_path)
            .await?;
        
        let writer = BufWriter::new(file);
        let mut file_writer = self.file_writer.lock().await;
        *file_writer = Some(writer);
        
        Ok(())
    }

    async fn setup_default_policies(&self) {
        let mut policies = self.retention_policies.write().await;
        
        let default_policy = RetentionPolicy {
            policy_id: "default".to_string(),
            event_types: vec![
                AuditEventType::AlertCreated,
                AuditEventType::AlertAggregated,
                AuditEventType::AlertSuppressed,
            ],
            retention_days: 365,
            archive_after_days: 90,
            purge_after_days: 2555, // 7 years for compliance
            compression_enabled: true,
        };
        
        policies.insert("default".to_string(), default_policy);

        let security_policy = RetentionPolicy {
            policy_id: "security".to_string(),
            event_types: vec![
                AuditEventType::UserAuthenticated,
                AuditEventType::AccessDenied,
                AuditEventType::ConfigurationChanged,
            ],
            retention_days: 2555,
            archive_after_days: 365,
            purge_after_days: 3650, // 10 years for security events
            compression_enabled: true,
        };
        
        policies.insert("security".to_string(), security_policy);

        let mut rules = self.compliance_rules.write().await;
        
        let gdpr_rule = ComplianceRule {
            rule_id: "gdpr".to_string(),
            rule_name: "GDPR Compliance".to_string(),
            regulation: "General Data Protection Regulation".to_string(),
            description: "EU data protection regulation compliance".to_string(),
            required_fields: vec![
                "event_id".to_string(),
                "timestamp".to_string(),
                "event_type".to_string(),
            ],
            retention_days: 2555,
            encryption_required: true,
            access_controls: vec!["admin".to_string(), "compliance_officer".to_string()],
        };
        
        rules.insert("gdpr".to_string(), gdpr_rule);
    }

    pub async fn log_event(&self, mut event: AuditEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event.event_id.is_empty() {
            event.event_id = Uuid::new_v4().to_string();
        }
        
        self.validate_compliance(&event).await?;
        
        let mut buffer = self.audit_buffer.lock().await;
        buffer.push(event.clone());
        
        let mut trail = self.audit_trail.write().await;
        trail.push(event);
        
        if trail.len() > self.max_memory_events {
            trail.remove(0);
        }
        
        if buffer.len() >= self.buffer_size {
            self.flush_buffer_internal(&mut buffer).await?;
        }
        
        Ok(())
    }

    async fn validate_compliance(&self, event: &AuditEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rules = self.compliance_rules.read().await;
        
        for rule in rules.values() {
            if !event.compliance_tags.iter().any(|tag| tag == &rule.rule_id) {
                continue;
            }
            
            let event_json = serde_json::to_value(event)?;
            
            for required_field in &rule.required_fields {
                if event_json.get(required_field).is_none() {
                    return Err(format!("Missing required field '{}' for compliance rule '{}'", 
                                     required_field, rule.rule_name).into());
                }
            }
            
            if rule.encryption_required && !self.encryption_enabled {
                warn!("Compliance rule '{}' requires encryption but it's not enabled", rule.rule_name);
            }
        }
        
        Ok(())
    }

    pub async fn flush_buffer(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = self.audit_buffer.lock().await;
        self.flush_buffer_internal(&mut buffer).await
    }

    async fn flush_buffer_internal(&self, buffer: &mut Vec<AuditEvent>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if buffer.is_empty() {
            return Ok(());
        }
        
        let mut file_writer = self.file_writer.lock().await;
        if let Some(writer) = file_writer.as_mut() {
            for event in buffer.iter() {
                let json_line = serde_json::to_string(event)? + "\n";
                writer.write_all(json_line.as_bytes()).await?;
            }
            writer.flush().await?;
        }
        
        buffer.clear();
        Ok(())
    }

    pub async fn query_events(&self, query: AuditQuery) -> Vec<AuditEvent> {
        let trail = self.audit_trail.read().await;
        
        let filtered_events: Vec<AuditEvent> = trail
            .iter()
            .filter(|event| {
                if let Some(ref event_types) = query.event_types {
                    if !event_types.contains(&event.event_type) {
                        return false;
                    }
                }
                
                if let Some(start_time) = query.start_time {
                    if event.timestamp < start_time {
                        return false;
                    }
                }
                
                if let Some(end_time) = query.end_time {
                    if event.timestamp > end_time {
                        return false;
                    }
                }
                
                if let Some(ref user_id) = query.user_id {
                    if event.user_id.as_ref() != Some(user_id) {
                        return false;
                    }
                }
                
                if let Some(ref alert_id) = query.alert_id {
                    if event.alert_id.as_ref() != Some(alert_id) {
                        return false;
                    }
                }
                
                if let Some(severity) = &query.severity {
                    if &event.severity != severity {
                        return false;
                    }
                }
                
                if let Some(ref compliance_tags) = query.compliance_tags {
                    if !compliance_tags.iter().any(|tag| event.compliance_tags.contains(tag)) {
                        return false;
                    }
                }
                
                true
            })
            .cloned()
            .collect();
        
        if let Some(limit) = query.limit {
            filtered_events.into_iter().take(limit).collect()
        } else {
            filtered_events
        }
    }

    pub async fn log_alert_created(&self, alert: &Alert, user_id: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: AuditEventType::AlertCreated,
            timestamp: Utc::now(),
            user_id,
            session_id: None,
            source_ip: None,
            alert_id: Some(alert.id.clone()),
            details: serde_json::json!({
                "alert_type": alert.alert_type,
                "severity": alert.severity,
                "source": alert.source,
                "message": alert.message
            }),
            severity: self.map_alert_severity_to_audit(&alert.severity),
            compliance_tags: vec!["gdpr".to_string()],
        };
        
        self.log_event(event).await
    }

    pub async fn log_alert_aggregated(&self, alert_ids: &[AlertId], aggregation_rule: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: AuditEventType::AlertAggregated,
            timestamp: Utc::now(),
            user_id: None,
            session_id: None,
            source_ip: None,
            alert_id: alert_ids.first().cloned(),
            details: serde_json::json!({
                "aggregated_alerts": alert_ids,
                "aggregation_rule": aggregation_rule,
                "count": alert_ids.len()
            }),
            severity: AuditSeverity::Medium,
            compliance_tags: vec!["gdpr".to_string()],
        };
        
        self.log_event(event).await
    }

    pub async fn log_alert_suppressed(&self, alert_id: &AlertId, reason: &str, rule_id: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: AuditEventType::AlertSuppressed,
            timestamp: Utc::now(),
            user_id: None,
            session_id: None,
            source_ip: None,
            alert_id: Some(alert_id.clone()),
            details: serde_json::json!({
                "suppression_reason": reason,
                "suppression_rule_id": rule_id
            }),
            severity: AuditSeverity::Low,
            compliance_tags: vec!["gdpr".to_string()],
        };
        
        self.log_event(event).await
    }

    pub async fn log_configuration_changed(&self, config_type: &str, changes: serde_json::Value, user_id: Option<String>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: AuditEventType::ConfigurationChanged,
            timestamp: Utc::now(),
            user_id,
            session_id: None,
            source_ip: None,
            alert_id: None,
            details: serde_json::json!({
                "configuration_type": config_type,
                "changes": changes
            }),
            severity: AuditSeverity::High,
            compliance_tags: vec!["gdpr".to_string()],
        };
        
        self.log_event(event).await
    }

    fn map_alert_severity_to_audit(&self, severity: &AlertSeverity) -> AuditSeverity {
        match severity {
            AlertSeverity::Critical => AuditSeverity::Critical,
            AlertSeverity::High => AuditSeverity::High,
            AlertSeverity::Medium => AuditSeverity::Medium,
            AlertSeverity::Low => AuditSeverity::Low,
        }
    }

    pub async fn add_compliance_rule(&self, rule: ComplianceRule) {
        let mut rules = self.compliance_rules.write().await;
        rules.insert(rule.rule_id.clone(), rule);
    }

    pub async fn add_retention_policy(&self, policy: RetentionPolicy) {
        let mut policies = self.retention_policies.write().await;
        policies.insert(policy.policy_id.clone(), policy);
    }

    pub async fn cleanup_expired_events(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let cutoff_time = Utc::now() - chrono::Duration::days(365);
        let mut trail = self.audit_trail.write().await;
        let original_len = trail.len();
        
        trail.retain(|event| event.timestamp > cutoff_time);
        
        let cleaned_count = original_len - trail.len();
        if cleaned_count > 0 {
            info!("Cleaned up {} expired audit events", cleaned_count);
        }
        
        Ok(cleaned_count)
    }

    pub async fn get_audit_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let trail = self.audit_trail.read().await;
        let buffer = self.audit_buffer.lock().await;
        
        stats.insert("total_events".to_string(), serde_json::Value::Number(trail.len().into()));
        stats.insert("buffered_events".to_string(), serde_json::Value::Number(buffer.len().into()));
        
        let mut event_type_counts: HashMap<String, usize> = HashMap::new();
        for event in trail.iter() {
            let type_str = format!("{:?}", event.event_type);
            *event_type_counts.entry(type_str).or_insert(0) += 1;
        }
        
        stats.insert("event_type_distribution".to_string(), serde_json::to_value(event_type_counts).unwrap());
        
        let compliance_rules = self.compliance_rules.read().await;
        stats.insert("compliance_rules_count".to_string(), serde_json::Value::Number(compliance_rules.len().into()));
        
        stats
    }

    pub async fn export_audit_trail(&self, format: &str, query: Option<AuditQuery>) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let events = if let Some(query) = query {
            self.query_events(query).await
        } else {
            let trail = self.audit_trail.read().await;
            trail.clone()
        };
        
        self.log_event(AuditEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: AuditEventType::DataExported,
            timestamp: Utc::now(),
            user_id: None,
            session_id: None,
            source_ip: None,
            alert_id: None,
            details: serde_json::json!({
                "export_format": format,
                "exported_events_count": events.len()
            }),
            severity: AuditSeverity::Medium,
            compliance_tags: vec!["gdpr".to_string()],
        }).await?;
        
        match format.to_lowercase().as_str() {
            "json" => Ok(serde_json::to_string_pretty(&events)?),
            "csv" => self.export_to_csv(&events),
            _ => Err("Unsupported export format".into()),
        }
    }

    fn export_to_csv(&self, events: &[AuditEvent]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut csv_data = String::new();
        csv_data.push_str("event_id,event_type,timestamp,user_id,alert_id,severity,details\n");
        
        for event in events {
            csv_data.push_str(&format!(
                "{},{:?},{},{},{},{:?},\"{}\"\n",
                event.event_id,
                event.event_type,
                event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                event.user_id.as_deref().unwrap_or(""),
                event.alert_id.as_deref().unwrap_or(""),
                event.severity,
                event.details.to_string().replace('"', "'")
            ));
        }
        
        Ok(csv_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::collections::HashMap;

    fn create_test_alert() -> Alert {
        Alert {
            id: Uuid::new_v4().to_string(),
            alert_type: "test_alert".to_string(),
            severity: AlertSeverity::High,
            message: "Test alert message".to_string(),
            source: "test_source".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            resolved: false,
            acknowledged: false,
        }
    }

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let temp_dir = tempdir().unwrap();
        let audit_file = temp_dir.path().join("audit.log");
        
        let logger = AuditLogger::new(
            audit_file.to_str().unwrap().to_string(),
            10,
            false,
            1000,
        ).await;
        
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_log_alert_events() {
        let temp_dir = tempdir().unwrap();
        let audit_file = temp_dir.path().join("audit.log");
        
        let logger = AuditLogger::new(
            audit_file.to_str().unwrap().to_string(),
            10,
            false,
            1000,
        ).await.unwrap();
        
        let alert = create_test_alert();
        
        logger.log_alert_created(&alert, Some("user123".to_string())).await.unwrap();
        logger.log_alert_suppressed(&alert.id, "High frequency", Some("rule1".to_string())).await.unwrap();
        
        let stats = logger.get_audit_stats().await;
        assert_eq!(stats.get("total_events").and_then(|v| v.as_u64()), Some(2));
    }

    #[tokio::test]
    async fn test_audit_query() {
        let temp_dir = tempdir().unwrap();
        let audit_file = temp_dir.path().join("audit.log");
        
        let logger = AuditLogger::new(
            audit_file.to_str().unwrap().to_string(),
            10,
            false,
            1000,
        ).await.unwrap();
        
        let alert = create_test_alert();
        logger.log_alert_created(&alert, Some("user123".to_string())).await.unwrap();
        
        let query = AuditQuery {
            event_types: Some(vec![AuditEventType::AlertCreated]),
            start_time: Some(Utc::now() - chrono::Duration::hours(1)),
            end_time: Some(Utc::now() + chrono::Duration::hours(1)),
            user_id: Some("user123".to_string()),
            alert_id: None,
            severity: None,
            compliance_tags: None,
            limit: None,
        };
        
        let events = logger.query_events(query).await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, AuditEventType::AlertCreated);
    }

    #[tokio::test]
    async fn test_compliance_validation() {
        let temp_dir = tempdir().unwrap();
        let audit_file = temp_dir.path().join("audit.log");
        
        let logger = AuditLogger::new(
            audit_file.to_str().unwrap().to_string(),
            10,
            false,
            1000,
        ).await.unwrap();
        
        let event = AuditEvent {
            event_id: "".to_string(),
            event_type: AuditEventType::AlertCreated,
            timestamp: Utc::now(),
            user_id: Some("user123".to_string()),
            session_id: None,
            source_ip: None,
            alert_id: Some("alert123".to_string()),
            details: serde_json::json!({}),
            severity: AuditSeverity::High,
            compliance_tags: vec!["gdpr".to_string()],
        };
        
        let result = logger.log_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_functionality() {
        let temp_dir = tempdir().unwrap();
        let audit_file = temp_dir.path().join("audit.log");
        
        let logger = AuditLogger::new(
            audit_file.to_str().unwrap().to_string(),
            10,
            false,
            1000,
        ).await.unwrap();
        
        let alert = create_test_alert();
        logger.log_alert_created(&alert, Some("user123".to_string())).await.unwrap();
        
        let json_export = logger.export_audit_trail("json", None).await.unwrap();
        assert!(json_export.contains("AlertCreated"));
        
        let csv_export = logger.export_audit_trail("csv", None).await.unwrap();
        assert!(csv_export.contains("event_id,event_type"));
    }
}