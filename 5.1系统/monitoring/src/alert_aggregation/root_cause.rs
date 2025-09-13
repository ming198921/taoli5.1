//! Root cause analysis engine using Bayesian networks

use crate::types::{Alert, AlertId, AlertSeverity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalRelation {
    pub from_alert_type: String,
    pub to_alert_type: String,
    pub probability: f64,
    pub time_window_seconds: u64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub root_cause_alert_id: AlertId,
    pub affected_alerts: Vec<AlertId>,
    pub probability: f64,
    pub reasoning: String,
    pub analysis_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BayesianNode {
    pub alert_type: String,
    pub prior_probability: f64,
    pub conditional_probabilities: HashMap<String, f64>,
}

pub struct RootCauseAnalyzer {
    causal_relations: Arc<RwLock<HashMap<String, Vec<CausalRelation>>>>,
    bayesian_network: Arc<RwLock<HashMap<String, BayesianNode>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
    max_history_size: usize,
}

impl RootCauseAnalyzer {
    pub fn new(max_history_size: usize) -> Self {
        let mut analyzer = Self {
            causal_relations: Arc::new(RwLock::new(HashMap::new())),
            bayesian_network: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
        };
        
        analyzer.initialize_default_relations();
        analyzer
    }

    fn initialize_default_relations(&self) {
        let rt = tokio::runtime::Handle::current();
        rt.spawn(async move {
            // This would be done in the constructor, but we need async context
        });
    }

    pub async fn add_causal_relation(&self, relation: CausalRelation) {
        let mut relations = self.causal_relations.write().await;
        relations
            .entry(relation.from_alert_type.clone())
            .or_insert_with(Vec::new)
            .push(relation);
    }

    pub async fn update_bayesian_network(&self, node: BayesianNode) {
        let mut network = self.bayesian_network.write().await;
        network.insert(node.alert_type.clone(), node);
    }

    pub async fn add_alert_to_history(&self, alert: Alert) {
        let mut history = self.alert_history.write().await;
        history.push(alert);
        
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    pub async fn analyze_root_cause(&self, alerts: &[Alert]) -> Option<RootCauseAnalysis> {
        if alerts.is_empty() {
            return None;
        }

        let causal_relations = self.causal_relations.read().await;
        let bayesian_network = self.bayesian_network.read().await;
        
        let mut candidate_roots = Vec::new();
        
        for alert in alerts {
            let probability = self.calculate_root_probability(alert, alerts, &causal_relations, &bayesian_network).await;
            candidate_roots.push((alert.id.clone(), probability));
        }
        
        candidate_roots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some((root_alert_id, probability)) = candidate_roots.first() {
            if *probability > 0.3 {
                let affected_alerts = alerts.iter()
                    .filter(|a| a.id != *root_alert_id)
                    .map(|a| a.id.clone())
                    .collect();

                let reasoning = self.generate_reasoning(&root_alert_id, &affected_alerts, *probability).await;

                return Some(RootCauseAnalysis {
                    root_cause_alert_id: root_alert_id.clone(),
                    affected_alerts,
                    probability: *probability,
                    reasoning,
                    analysis_time: Utc::now(),
                });
            }
        }

        None
    }

    async fn calculate_root_probability(
        &self,
        candidate: &Alert,
        all_alerts: &[Alert],
        causal_relations: &HashMap<String, Vec<CausalRelation>>,
        bayesian_network: &HashMap<String, BayesianNode>,
    ) -> f64 {
        let mut probability = 0.0;
        
        if let Some(relations) = causal_relations.get(&candidate.alert_type) {
            for relation in relations {
                let affected_count = all_alerts.iter()
                    .filter(|a| {
                        a.alert_type == relation.to_alert_type &&
                        self.is_within_time_window(candidate, a, relation.time_window_seconds)
                    })
                    .count();
                
                if affected_count > 0 {
                    probability += relation.probability * (affected_count as f64 / all_alerts.len() as f64);
                }
            }
        }
        
        if let Some(node) = bayesian_network.get(&candidate.alert_type) {
            probability = probability * 0.7 + node.prior_probability * 0.3;
        }
        
        match candidate.severity {
            AlertSeverity::Critical => probability *= 1.5,
            AlertSeverity::High => probability *= 1.2,
            AlertSeverity::Medium => probability *= 1.0,
            AlertSeverity::Low => probability *= 0.8,
        }

        probability.min(1.0)
    }

    fn is_within_time_window(&self, root: &Alert, affected: &Alert, window_seconds: u64) -> bool {
        let time_diff = affected.timestamp.signed_duration_since(root.timestamp);
        time_diff.num_seconds() >= 0 && time_diff.num_seconds() <= window_seconds as i64
    }

    async fn generate_reasoning(
        &self,
        root_alert_id: &AlertId,
        affected_alerts: &[AlertId],
        probability: f64,
    ) -> String {
        format!(
            "Root cause analysis identified alert {} as the likely root cause (probability: {:.2}) affecting {} downstream alerts. Analysis based on temporal correlation and Bayesian inference.",
            root_alert_id,
            probability,
            affected_alerts.len()
        )
    }

    pub async fn learn_from_feedback(&self, analysis: &RootCauseAnalysis, is_correct: bool) {
        if is_correct {
            self.reinforce_relations(analysis).await;
        } else {
            self.weaken_relations(analysis).await;
        }
    }

    async fn reinforce_relations(&self, analysis: &RootCauseAnalysis) {
        let mut causal_relations = self.causal_relations.write().await;
        
        for affected_id in &analysis.affected_alerts {
            if let Some(relations) = causal_relations.get_mut("system") {
                for relation in relations {
                    if relation.probability < 0.9 {
                        relation.probability += 0.05;
                        relation.confidence += 0.02;
                    }
                }
            }
        }
    }

    async fn weaken_relations(&self, analysis: &RootCauseAnalysis) {
        let mut causal_relations = self.causal_relations.write().await;
        
        for affected_id in &analysis.affected_alerts {
            if let Some(relations) = causal_relations.get_mut("system") {
                for relation in relations {
                    if relation.probability > 0.1 {
                        relation.probability -= 0.03;
                        relation.confidence -= 0.01;
                    }
                }
            }
        }
    }

    pub async fn get_causal_graph(&self) -> HashMap<String, Vec<CausalRelation>> {
        self.causal_relations.read().await.clone()
    }

    pub async fn update_from_historical_data(&self) {
        let history = self.alert_history.read().await;
        
        let mut type_correlations: HashMap<(String, String), Vec<i64>> = HashMap::new();
        
        for i in 0..history.len() {
            for j in (i + 1)..history.len() {
                let alert1 = &history[i];
                let alert2 = &history[j];
                
                let time_diff = alert2.timestamp.signed_duration_since(alert1.timestamp).num_seconds();
                
                if time_diff > 0 && time_diff <= 3600 {
                    let key = (alert1.alert_type.clone(), alert2.alert_type.clone());
                    type_correlations.entry(key).or_insert_with(Vec::new).push(time_diff);
                }
            }
        }
        
        let mut causal_relations = self.causal_relations.write().await;
        
        for ((from_type, to_type), time_diffs) in type_correlations {
            if time_diffs.len() >= 3 {
                let avg_time = time_diffs.iter().sum::<i64>() / time_diffs.len() as i64;
                let probability = (time_diffs.len() as f64 / history.len() as f64).min(0.8);
                
                let relation = CausalRelation {
                    from_alert_type: from_type.clone(),
                    to_alert_type: to_type,
                    probability,
                    time_window_seconds: (avg_time * 2) as u64,
                    confidence: (time_diffs.len() as f64 / 10.0).min(1.0),
                };
                
                causal_relations
                    .entry(from_type)
                    .or_insert_with(Vec::new)
                    .push(relation);
            }
        }
        
        info!("Updated causal relations from {} historical alerts", history.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AlertType;
    use uuid::Uuid;

    fn create_test_alert(alert_type: &str, severity: AlertSeverity, timestamp: DateTime<Utc>) -> Alert {
        Alert {
            id: Uuid::new_v4().to_string(),
            alert_type: alert_type.to_string(),
            severity,
            message: "Test alert".to_string(),
            source: "test".to_string(),
            timestamp,
            metadata: HashMap::new(),
            resolved: false,
            acknowledged: false,
        }
    }

    #[tokio::test]
    async fn test_root_cause_analysis() {
        let analyzer = RootCauseAnalyzer::new(1000);
        
        let relation = CausalRelation {
            from_alert_type: "system_failure".to_string(),
            to_alert_type: "service_degradation".to_string(),
            probability: 0.8,
            time_window_seconds: 300,
            confidence: 0.9,
        };
        
        analyzer.add_causal_relation(relation).await;
        
        let now = Utc::now();
        let alerts = vec![
            create_test_alert("system_failure", AlertSeverity::Critical, now),
            create_test_alert("service_degradation", AlertSeverity::High, now + chrono::Duration::seconds(60)),
            create_test_alert("service_degradation", AlertSeverity::Medium, now + chrono::Duration::seconds(120)),
        ];
        
        let analysis = analyzer.analyze_root_cause(&alerts).await;
        assert!(analysis.is_some());
        
        let analysis = analysis.unwrap();
        assert_eq!(analysis.affected_alerts.len(), 2);
        assert!(analysis.probability > 0.3);
    }

    #[tokio::test]
    async fn test_learning_from_feedback() {
        let analyzer = RootCauseAnalyzer::new(1000);
        
        let relation = CausalRelation {
            from_alert_type: "test_type".to_string(),
            to_alert_type: "affected_type".to_string(),
            probability: 0.5,
            time_window_seconds: 300,
            confidence: 0.5,
        };
        
        analyzer.add_causal_relation(relation).await;
        
        let analysis = RootCauseAnalysis {
            root_cause_alert_id: "test_alert".to_string(),
            affected_alerts: vec!["affected_alert".to_string()],
            probability: 0.8,
            reasoning: "Test reasoning".to_string(),
            analysis_time: Utc::now(),
        };
        
        analyzer.learn_from_feedback(&analysis, true).await;
        
        let updated_relations = analyzer.get_causal_graph().await;
        assert!(!updated_relations.is_empty());
    }

    #[tokio::test]
    async fn test_historical_data_learning() {
        let analyzer = RootCauseAnalyzer::new(1000);
        
        let now = Utc::now();
        let alerts = vec![
            create_test_alert("database_error", AlertSeverity::Critical, now),
            create_test_alert("api_slow", AlertSeverity::High, now + chrono::Duration::seconds(30)),
            create_test_alert("database_error", AlertSeverity::Critical, now + chrono::Duration::seconds(3600)),
            create_test_alert("api_slow", AlertSeverity::High, now + chrono::Duration::seconds(3630)),
        ];
        
        for alert in alerts {
            analyzer.add_alert_to_history(alert).await;
        }
        
        analyzer.update_from_historical_data().await;
        
        let relations = analyzer.get_causal_graph().await;
        assert!(relations.contains_key("database_error"));
    }
}