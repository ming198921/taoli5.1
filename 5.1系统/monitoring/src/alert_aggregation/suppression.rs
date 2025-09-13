//! Alert suppression system with intelligent noise reduction

use crate::types::{Alert, AlertId, AlertSeverity};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionRule {
    pub id: String,
    pub name: String,
    pub alert_type_pattern: String,
    pub source_pattern: Option<String>,
    pub message_pattern: Option<String>,
    pub max_occurrences: u32,
    pub time_window_seconds: u64,
    pub severity_threshold: AlertSeverity,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionState {
    pub rule_id: String,
    pub suppressed_count: u32,
    pub first_occurrence: DateTime<Utc>,
    pub last_occurrence: DateTime<Utc>,
    pub suppressed_alert_ids: VecDeque<AlertId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionDecision {
    pub alert_id: AlertId,
    pub should_suppress: bool,
    pub rule_id: Option<String>,
    pub reason: String,
    pub suppressed_count: u32,
    pub next_alert_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveThreshold {
    pub alert_type: String,
    pub base_threshold: u32,
    pub current_threshold: u32,
    pub adjustment_factor: f64,
    pub last_adjustment: DateTime<Utc>,
    pub noise_score: f64,
}

pub struct AlertSuppressor {
    suppression_rules: Arc<RwLock<HashMap<String, SuppressionRule>>>,
    suppression_states: Arc<RwLock<HashMap<String, SuppressionState>>>,
    adaptive_thresholds: Arc<RwLock<HashMap<String, AdaptiveThreshold>>>,
    alert_frequency_tracker: Arc<RwLock<HashMap<String, VecDeque<DateTime<Utc>>>>>,
    flapping_detection: Arc<RwLock<HashMap<String, FlappingDetection>>>,
    max_state_retention: Duration,
}

#[derive(Debug, Clone)]
struct FlappingDetection {
    state_changes: VecDeque<(DateTime<Utc>, bool)>,
    is_flapping: bool,
    suppression_start: Option<DateTime<Utc>>,
}

impl AlertSuppressor {
    pub fn new(max_state_retention: Duration) -> Self {
        Self {
            suppression_rules: Arc::new(RwLock::new(HashMap::new())),
            suppression_states: Arc::new(RwLock::new(HashMap::new())),
            adaptive_thresholds: Arc::new(RwLock::new(HashMap::new())),
            alert_frequency_tracker: Arc::new(RwLock::new(HashMap::new())),
            flapping_detection: Arc::new(RwLock::new(HashMap::new())),
            max_state_retention,
        }
    }

    pub async fn add_suppression_rule(&self, rule: SuppressionRule) {
        let mut rules = self.suppression_rules.write().await;
        rules.insert(rule.id.clone(), rule);
        info!("Added suppression rule: {}", rules.len());
    }

    pub async fn remove_suppression_rule(&self, rule_id: &str) {
        let mut rules = self.suppression_rules.write().await;
        rules.remove(rule_id);
        
        let mut states = self.suppression_states.write().await;
        states.retain(|_, state| state.rule_id != rule_id);
    }

    pub async fn update_suppression_rule(&self, rule: SuppressionRule) {
        let mut rules = self.suppression_rules.write().await;
        rules.insert(rule.id.clone(), rule);
    }

    pub async fn should_suppress_alert(&self, alert: &Alert) -> SuppressionDecision {
        self.update_alert_frequency(alert).await;
        
        if let Some(flapping_decision) = self.check_flapping_suppression(alert).await {
            return flapping_decision;
        }
        
        if let Some(frequency_decision) = self.check_frequency_suppression(alert).await {
            return frequency_decision;
        }
        
        if let Some(rule_decision) = self.check_rule_based_suppression(alert).await {
            return rule_decision;
        }
        
        SuppressionDecision {
            alert_id: alert.id.clone(),
            should_suppress: false,
            rule_id: None,
            reason: "No suppression rules matched".to_string(),
            suppressed_count: 0,
            next_alert_time: None,
        }
    }

    async fn update_alert_frequency(&self, alert: &Alert) {
        let mut tracker = self.alert_frequency_tracker.write().await;
        let key = format!("{}:{}", alert.alert_type, alert.source);
        
        let timestamps = tracker.entry(key).or_insert_with(VecDeque::new);
        timestamps.push_back(alert.timestamp);
        
        let cutoff_time = Utc::now() - Duration::hours(1);
        while let Some(&front_time) = timestamps.front() {
            if front_time < cutoff_time {
                timestamps.pop_front();
            } else {
                break;
            }
        }
    }

    async fn check_flapping_suppression(&self, alert: &Alert) -> Option<SuppressionDecision> {
        let mut flapping = self.flapping_detection.write().await;
        let key = format!("{}:{}", alert.alert_type, alert.source);
        
        let detection = flapping.entry(key.clone()).or_insert_with(|| FlappingDetection {
            state_changes: VecDeque::new(),
            is_flapping: false,
            suppression_start: None,
        });
        
        detection.state_changes.push_back((alert.timestamp, !alert.resolved));
        
        let cutoff_time = Utc::now() - Duration::minutes(15);
        while let Some(&(time, _)) = detection.state_changes.front() {
            if time < cutoff_time {
                detection.state_changes.pop_front();
            } else {
                break;
            }
        }
        
        if detection.state_changes.len() >= 6 {
            let state_changes_count = detection.state_changes.len();
            let time_span = detection.state_changes.back().unwrap().0
                .signed_duration_since(detection.state_changes.front().unwrap().0)
                .num_minutes();
            
            let flapping_rate = state_changes_count as f64 / time_span.max(1) as f64;
            
            if flapping_rate > 0.4 && !detection.is_flapping {
                detection.is_flapping = true;
                detection.suppression_start = Some(Utc::now());
                
                return Some(SuppressionDecision {
                    alert_id: alert.id.clone(),
                    should_suppress: true,
                    rule_id: Some("flapping_detection".to_string()),
                    reason: format!("Alert is flapping (rate: {:.2}/min)", flapping_rate),
                    suppressed_count: 1,
                    next_alert_time: Some(Utc::now() + Duration::minutes(30)),
                });
            }
            
            if detection.is_flapping {
                if let Some(suppression_start) = detection.suppression_start {
                    if Utc::now().signed_duration_since(suppression_start) > Duration::hours(1) {
                        detection.is_flapping = false;
                        detection.suppression_start = None;
                    } else {
                        return Some(SuppressionDecision {
                            alert_id: alert.id.clone(),
                            should_suppress: true,
                            rule_id: Some("flapping_detection".to_string()),
                            reason: "Alert still in flapping suppression period".to_string(),
                            suppressed_count: 0,
                            next_alert_time: Some(suppression_start + Duration::hours(1)),
                        });
                    }
                }
            }
        }
        
        None
    }

    async fn check_frequency_suppression(&self, alert: &Alert) -> Option<SuppressionDecision> {
        let tracker = self.alert_frequency_tracker.read().await;
        let key = format!("{}:{}", alert.alert_type, alert.source);
        
        if let Some(timestamps) = tracker.get(&key) {
            let recent_count = timestamps.len();
            
            let thresholds = self.adaptive_thresholds.read().await;
            if let Some(threshold) = thresholds.get(&alert.alert_type) {
                if recent_count > threshold.current_threshold as usize {
                    let suppression_window = self.calculate_suppression_window(recent_count);
                    
                    return Some(SuppressionDecision {
                        alert_id: alert.id.clone(),
                        should_suppress: true,
                        rule_id: Some("frequency_based".to_string()),
                        reason: format!("High frequency: {} alerts in last hour", recent_count),
                        suppressed_count: recent_count as u32,
                        next_alert_time: Some(Utc::now() + suppression_window),
                    });
                }
            } else if recent_count > 20 {
                return Some(SuppressionDecision {
                    alert_id: alert.id.clone(),
                    should_suppress: true,
                    rule_id: Some("frequency_based".to_string()),
                    reason: format!("High frequency: {} alerts in last hour", recent_count),
                    suppressed_count: recent_count as u32,
                    next_alert_time: Some(Utc::now() + Duration::minutes(30)),
                });
            }
        }
        
        None
    }

    async fn check_rule_based_suppression(&self, alert: &Alert) -> Option<SuppressionDecision> {
        let rules = self.suppression_rules.read().await;
        
        for rule in rules.values() {
            if !rule.enabled || !self.rule_matches_alert(rule, alert) {
                continue;
            }
            
            if alert.severity < rule.severity_threshold {
                continue;
            }
            
            let mut states = self.suppression_states.write().await;
            let state = states.entry(rule.id.clone()).or_insert_with(|| SuppressionState {
                rule_id: rule.id.clone(),
                suppressed_count: 0,
                first_occurrence: alert.timestamp,
                last_occurrence: alert.timestamp,
                suppressed_alert_ids: VecDeque::new(),
            });
            
            let time_since_first = alert.timestamp.signed_duration_since(state.first_occurrence);
            
            if time_since_first.num_seconds() > rule.time_window_seconds as i64 {
                state.suppressed_count = 1;
                state.first_occurrence = alert.timestamp;
                state.suppressed_alert_ids.clear();
            } else if state.suppressed_count >= rule.max_occurrences {
                state.suppressed_count += 1;
                state.last_occurrence = alert.timestamp;
                state.suppressed_alert_ids.push_back(alert.id.clone());
                
                if state.suppressed_alert_ids.len() > 100 {
                    state.suppressed_alert_ids.pop_front();
                }
                
                return Some(SuppressionDecision {
                    alert_id: alert.id.clone(),
                    should_suppress: true,
                    rule_id: Some(rule.id.clone()),
                    reason: format!("Rule '{}': {} occurrences in {}s", rule.name, state.suppressed_count, rule.time_window_seconds),
                    suppressed_count: state.suppressed_count,
                    next_alert_time: Some(state.first_occurrence + Duration::seconds(rule.time_window_seconds as i64)),
                });
            } else {
                state.suppressed_count += 1;
                state.last_occurrence = alert.timestamp;
            }
        }
        
        None
    }

    fn rule_matches_alert(&self, rule: &SuppressionRule, alert: &Alert) -> bool {
        if !self.pattern_matches(&rule.alert_type_pattern, &alert.alert_type) {
            return false;
        }
        
        if let Some(source_pattern) = &rule.source_pattern {
            if !self.pattern_matches(source_pattern, &alert.source) {
                return false;
            }
        }
        
        if let Some(message_pattern) = &rule.message_pattern {
            if !self.pattern_matches(message_pattern, &alert.message) {
                return false;
            }
        }
        
        true
    }

    fn pattern_matches(&self, pattern: &str, text: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            let mut text_pos = 0;
            
            for (i, part) in pattern_parts.iter().enumerate() {
                if part.is_empty() {
                    continue;
                }
                
                if let Some(found_pos) = text[text_pos..].find(part) {
                    text_pos += found_pos + part.len();
                } else {
                    return false;
                }
            }
            
            true
        } else {
            pattern == text
        }
    }

    fn calculate_suppression_window(&self, frequency: usize) -> Duration {
        match frequency {
            0..=10 => Duration::minutes(5),
            11..=25 => Duration::minutes(15),
            26..=50 => Duration::minutes(30),
            51..=100 => Duration::hours(1),
            _ => Duration::hours(2),
        }
    }

    pub async fn update_adaptive_thresholds(&self) {
        let mut thresholds = self.adaptive_thresholds.write().await;
        let tracker = self.alert_frequency_tracker.read().await;
        
        for (key, timestamps) in tracker.iter() {
            let alert_type = key.split(':').next().unwrap_or(key).to_string();
            let current_frequency = timestamps.len();
            
            let threshold = thresholds.entry(alert_type.clone()).or_insert_with(|| AdaptiveThreshold {
                alert_type: alert_type.clone(),
                base_threshold: 10,
                current_threshold: 10,
                adjustment_factor: 1.0,
                last_adjustment: Utc::now(),
                noise_score: 0.0,
            });
            
            let time_since_adjustment = Utc::now().signed_duration_since(threshold.last_adjustment);
            
            if time_since_adjustment > Duration::minutes(30) {
                let noise_score = self.calculate_noise_score(current_frequency, threshold.base_threshold);
                threshold.noise_score = noise_score;
                
                if noise_score > 0.7 {
                    threshold.adjustment_factor *= 1.1;
                } else if noise_score < 0.3 {
                    threshold.adjustment_factor *= 0.95;
                }
                
                threshold.adjustment_factor = threshold.adjustment_factor.clamp(0.5, 2.0);
                threshold.current_threshold = ((threshold.base_threshold as f64) * threshold.adjustment_factor) as u32;
                threshold.last_adjustment = Utc::now();
                
                debug!("Updated adaptive threshold for {}: {} (noise: {:.2})", 
                       alert_type, threshold.current_threshold, noise_score);
            }
        }
    }

    fn calculate_noise_score(&self, current_frequency: usize, base_threshold: u32) -> f64 {
        if current_frequency == 0 {
            return 0.0;
        }
        
        let ratio = current_frequency as f64 / base_threshold as f64;
        
        if ratio <= 1.0 {
            ratio * 0.5
        } else {
            0.5 + (ratio - 1.0).ln().max(0.0) * 0.2
        }
    }

    pub async fn cleanup_expired_states(&self) {
        let cutoff_time = Utc::now() - self.max_state_retention;
        
        let mut states = self.suppression_states.write().await;
        states.retain(|_, state| state.last_occurrence > cutoff_time);
        
        let mut flapping = self.flapping_detection.write().await;
        flapping.retain(|_, detection| {
            if let Some(&(last_change_time, _)) = detection.state_changes.back() {
                last_change_time > cutoff_time
            } else {
                false
            }
        });
        
        info!("Cleaned up expired suppression states");
    }

    pub async fn get_suppression_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        let states = self.suppression_states.read().await;
        let active_suppressions = states.len();
        let total_suppressed = states.values().map(|s| s.suppressed_count).sum::<u32>();
        
        let flapping = self.flapping_detection.read().await;
        let flapping_alerts = flapping.values().filter(|d| d.is_flapping).count();
        
        let thresholds = self.adaptive_thresholds.read().await;
        let avg_noise_score = if !thresholds.is_empty() {
            thresholds.values().map(|t| t.noise_score).sum::<f64>() / thresholds.len() as f64
        } else {
            0.0
        };
        
        stats.insert("active_suppressions".to_string(), serde_json::Value::Number(active_suppressions.into()));
        stats.insert("total_suppressed_alerts".to_string(), serde_json::Value::Number(total_suppressed.into()));
        stats.insert("flapping_alerts".to_string(), serde_json::Value::Number(flapping_alerts.into()));
        stats.insert("avg_noise_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(avg_noise_score).unwrap_or(0.into())));
        
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AlertType;
    use uuid::Uuid;

    fn create_test_alert(alert_type: &str, source: &str, severity: AlertSeverity) -> Alert {
        Alert {
            id: Uuid::new_v4().to_string(),
            alert_type: alert_type.to_string(),
            severity,
            message: "Test alert".to_string(),
            source: source.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            resolved: false,
            acknowledged: false,
        }
    }

    #[tokio::test]
    async fn test_rule_based_suppression() {
        let suppressor = AlertSuppressor::new(Duration::hours(24));
        
        let rule = SuppressionRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            alert_type_pattern: "test_*".to_string(),
            source_pattern: None,
            message_pattern: None,
            max_occurrences: 3,
            time_window_seconds: 300,
            severity_threshold: AlertSeverity::Medium,
            enabled: true,
            created_at: Utc::now(),
            last_updated: Utc::now(),
        };
        
        suppressor.add_suppression_rule(rule).await;
        
        let alert = create_test_alert("test_error", "system1", AlertSeverity::High);
        
        for i in 1..=5 {
            let decision = suppressor.should_suppress_alert(&alert).await;
            if i <= 3 {
                assert!(!decision.should_suppress);
            } else {
                assert!(decision.should_suppress);
                assert!(decision.rule_id.is_some());
            }
        }
    }

    #[tokio::test]
    async fn test_flapping_detection() {
        let suppressor = AlertSuppressor::new(Duration::hours(24));
        
        let mut alert = create_test_alert("flapping_test", "system1", AlertSeverity::High);
        
        for i in 0..8 {
            alert.resolved = i % 2 == 1;
            alert.timestamp = Utc::now() - Duration::minutes((8 - i) * 2);
            
            let decision = suppressor.should_suppress_alert(&alert).await;
            if i >= 6 {
                assert!(decision.should_suppress);
                assert_eq!(decision.rule_id, Some("flapping_detection".to_string()));
            }
        }
    }

    #[tokio::test]
    async fn test_frequency_suppression() {
        let suppressor = AlertSuppressor::new(Duration::hours(24));
        
        let alert = create_test_alert("frequent_error", "system1", AlertSeverity::Medium);
        
        for i in 0..25 {
            let decision = suppressor.should_suppress_alert(&alert).await;
            if i > 20 {
                assert!(decision.should_suppress);
                assert_eq!(decision.rule_id, Some("frequency_based".to_string()));
            }
        }
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        let suppressor = AlertSuppressor::new(Duration::hours(24));
        
        assert!(suppressor.pattern_matches("*", "anything"));
        assert!(suppressor.pattern_matches("test_*", "test_error"));
        assert!(suppressor.pattern_matches("*_error", "database_error"));
        assert!(suppressor.pattern_matches("test_*_error", "test_connection_error"));
        assert!(!suppressor.pattern_matches("test_*", "production_error"));
    }

    #[tokio::test]
    async fn test_adaptive_thresholds() {
        let suppressor = AlertSuppressor::new(Duration::hours(24));
        
        let alert = create_test_alert("adaptive_test", "system1", AlertSeverity::Medium);
        
        for _ in 0..15 {
            suppressor.should_suppress_alert(&alert).await;
        }
        
        suppressor.update_adaptive_thresholds().await;
        
        let stats = suppressor.get_suppression_stats().await;
        assert!(stats.contains_key("avg_noise_score"));
    }
}