//! 相似度分析器
//! 
//! 基于余弦相似度和其他算法分析告警相似性

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;

use super::RawAlert;

/// 相似度分析器
pub struct SimilarityAnalyzer {
    /// 停用词集合
    stop_words: HashSet<String>,
    /// 词汇权重
    word_weights: HashMap<String, f64>,
}

impl SimilarityAnalyzer {
    pub fn new() -> Self {
        let stop_words = vec![
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", 
            "of", "with", "by", "is", "are", "was", "were", "be", "been", "have", 
            "has", "had", "do", "does", "did", "will", "would", "could", "should"
        ].into_iter().map(|s| s.to_string()).collect();
        
        let word_weights = HashMap::new();
        
        Self {
            stop_words,
            word_weights,
        }
    }
    
    /// 计算两个告警的相似度
    pub fn calculate_alert_similarity(&self, alert1: &RawAlert, alert2: &RawAlert) -> Result<f64> {
        // 计算各个维度的相似度
        let title_sim = self.calculate_text_similarity(&alert1.title, &alert2.title)?;
        let desc_sim = self.calculate_text_similarity(&alert1.description, &alert2.description)?;
        let label_sim = self.calculate_label_similarity(&alert1.labels, &alert2.labels)?;
        let metric_sim = self.calculate_metric_similarity(&alert1.metrics, &alert2.metrics)?;
        let source_sim = if alert1.source == alert2.source { 1.0 } else { 0.0 };
        let severity_sim = self.calculate_severity_similarity(alert1.severity, alert2.severity);
        
        // 加权平均
        let weighted_similarity = title_sim * 0.3 + 
                                 desc_sim * 0.25 + 
                                 label_sim * 0.2 + 
                                 metric_sim * 0.1 + 
                                 source_sim * 0.1 + 
                                 severity_sim * 0.05;
        
        debug!(
            alert1_id = %alert1.id,
            alert2_id = %alert2.id,
            title_sim = %title_sim,
            desc_sim = %desc_sim,
            label_sim = %label_sim,
            metric_sim = %metric_sim,
            source_sim = %source_sim,
            severity_sim = %severity_sim,
            weighted_sim = %weighted_similarity,
            "Calculated alert similarity"
        );
        
        Ok(weighted_similarity)
    }
    
    /// 计算文本相似度（余弦相似度）
    pub fn calculate_text_similarity(&self, text1: &str, text2: &str) -> Result<f64> {
        let tokens1 = self.tokenize_and_normalize(text1);
        let tokens2 = self.tokenize_and_normalize(text2);
        
        if tokens1.is_empty() || tokens2.is_empty() {
            return Ok(0.0);
        }
        
        let vec1 = self.create_tf_idf_vector(&tokens1, &[&tokens1, &tokens2]);
        let vec2 = self.create_tf_idf_vector(&tokens2, &[&tokens1, &tokens2]);
        
        Ok(self.cosine_similarity(&vec1, &vec2))
    }
    
    /// 分词和规范化
    fn tokenize_and_normalize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|word| {
                // 移除标点符号
                word.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
            })
            .filter(|word| !word.is_empty() && !self.stop_words.contains(word))
            .collect()
    }
    
    /// 创建TF-IDF向量
    fn create_tf_idf_vector(&self, tokens: &[String], all_documents: &[&Vec<String>]) -> HashMap<String, f64> {
        let mut vector = HashMap::new();
        let token_count = tokens.len() as f64;
        
        // 计算词频 (TF)
        let mut term_frequency = HashMap::new();
        for token in tokens {
            *term_frequency.entry(token.clone()).or_insert(0) += 1;
        }
        
        // 计算TF-IDF
        for (term, count) in term_frequency {
            let tf = count as f64 / token_count;
            let df = all_documents.iter()
                .filter(|doc| doc.contains(&term))
                .count() as f64;
            let idf = (all_documents.len() as f64 / (1.0 + df)).ln();
            
            vector.insert(term, tf * idf);
        }
        
        vector
    }
    
    /// 计算余弦相似度
    fn cosine_similarity(&self, vec1: &HashMap<String, f64>, vec2: &HashMap<String, f64>) -> f64 {
        let mut dot_product = 0.0;
        let mut norm1 = 0.0;
        let mut norm2 = 0.0;
        
        // 获取所有唯一的词汇
        let all_terms: HashSet<_> = vec1.keys().chain(vec2.keys()).collect();
        
        for term in all_terms {
            let val1 = vec1.get(term).unwrap_or(&0.0);
            let val2 = vec2.get(term).unwrap_or(&0.0);
            
            dot_product += val1 * val2;
            norm1 += val1 * val1;
            norm2 += val2 * val2;
        }
        
        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1.sqrt() * norm2.sqrt())
        }
    }
    
    /// 计算标签相似度
    fn calculate_label_similarity(
        &self,
        labels1: &HashMap<String, String>,
        labels2: &HashMap<String, String>,
    ) -> Result<f64> {
        if labels1.is_empty() && labels2.is_empty() {
            return Ok(1.0);
        }
        
        if labels1.is_empty() || labels2.is_empty() {
            return Ok(0.0);
        }
        
        let all_keys: HashSet<_> = labels1.keys().chain(labels2.keys()).collect();
        let mut matches = 0;
        let mut partial_matches = 0.0;
        
        for key in all_keys {
            match (labels1.get(key), labels2.get(key)) {
                (Some(val1), Some(val2)) => {
                    if val1 == val2 {
                        matches += 1;
                    } else {
                        // 计算值的相似度
                        let val_similarity = self.calculate_text_similarity(val1, val2)?;
                        partial_matches += val_similarity;
                    }
                },
                _ => {
                    // 一方有另一方没有，不计分
                }
            }
        }
        
        let total_possible_matches = all_keys.len() as f64;
        let similarity = (matches as f64 + partial_matches) / total_possible_matches;
        
        Ok(similarity.min(1.0))
    }
    
    /// 计算指标相似度
    fn calculate_metric_similarity(
        &self,
        metrics1: &HashMap<String, f64>,
        metrics2: &HashMap<String, f64>,
    ) -> Result<f64> {
        if metrics1.is_empty() && metrics2.is_empty() {
            return Ok(1.0);
        }
        
        if metrics1.is_empty() || metrics2.is_empty() {
            return Ok(0.0);
        }
        
        let all_keys: HashSet<_> = metrics1.keys().chain(metrics2.keys()).collect();
        let mut similarity_sum = 0.0;
        let mut count = 0;
        
        for key in all_keys {
            if let (Some(&val1), Some(&val2)) = (metrics1.get(key), metrics2.get(key)) {
                // 计算数值相似度
                let max_val = val1.abs().max(val2.abs());
                let similarity = if max_val == 0.0 {
                    1.0
                } else {
                    1.0 - (val1 - val2).abs() / max_val
                };
                
                similarity_sum += similarity;
                count += 1;
            }
        }
        
        Ok(if count > 0 { similarity_sum / count as f64 } else { 0.0 })
    }
    
    /// 计算严重性相似度
    fn calculate_severity_similarity(
        &self,
        severity1: super::AlertSeverity,
        severity2: super::AlertSeverity,
    ) -> f64 {
        let diff = (severity1 as i32 - severity2 as i32).abs();
        match diff {
            0 => 1.0,   // 完全相同
            1 => 0.8,   // 相邻等级
            2 => 0.5,   // 相差两级
            3 => 0.2,   // 相差三级
            _ => 0.0,   // 相差四级或以上
        }
    }
    
    /// 计算Jaccard相似度（用于集合比较）
    pub fn calculate_jaccard_similarity<T: std::hash::Hash + Eq>(
        &self,
        set1: &HashSet<T>,
        set2: &HashSet<T>,
    ) -> f64 {
        let intersection = set1.intersection(set2).count();
        let union = set1.union(set2).count();
        
        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }
    
    /// 计算编辑距离
    pub fn calculate_edit_distance(&self, s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let len1 = s1_chars.len();
        let len2 = s2_chars.len();
        
        let mut dp = vec![vec![0; len2 + 1]; len1 + 1];
        
        // 初始化
        for i in 0..=len1 {
            dp[i][0] = i;
        }
        for j in 0..=len2 {
            dp[0][j] = j;
        }
        
        // 动态规划填表
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                dp[i][j] = std::cmp::min(
                    std::cmp::min(dp[i - 1][j] + 1, dp[i][j - 1] + 1),
                    dp[i - 1][j - 1] + cost
                );
            }
        }
        
        dp[len1][len2]
    }
    
    /// 基于编辑距离的相似度
    pub fn calculate_edit_similarity(&self, s1: &str, s2: &str) -> f64 {
        let max_len = s1.len().max(s2.len());
        if max_len == 0 {
            return 1.0;
        }
        
        let edit_distance = self.calculate_edit_distance(s1, s2);
        1.0 - (edit_distance as f64 / max_len as f64)
    }
    
    /// 更新词汇权重（用于在线学习）
    pub fn update_word_weights(&mut self, feedback: &[(String, f64)]) {
        for (word, weight) in feedback {
            self.word_weights.insert(word.clone(), *weight);
        }
    }
    
    /// 分析告警集合的聚类
    pub fn cluster_alerts(&self, alerts: &[RawAlert], similarity_threshold: f64) -> Result<Vec<Vec<usize>>> {
        let mut clusters = Vec::new();
        let mut visited = vec![false; alerts.len()];
        
        for i in 0..alerts.len() {
            if visited[i] {
                continue;
            }
            
            let mut cluster = vec![i];
            visited[i] = true;
            
            for j in (i + 1)..alerts.len() {
                if !visited[j] {
                    let similarity = self.calculate_alert_similarity(&alerts[i], &alerts[j])?;
                    if similarity >= similarity_threshold {
                        cluster.push(j);
                        visited[j] = true;
                    }
                }
            }
            
            clusters.push(cluster);
        }
        
        Ok(clusters)
    }
}

/// 相似度分析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarityConfig {
    /// 文本相似度权重
    pub text_weight: f64,
    /// 标签相似度权重
    pub label_weight: f64,
    /// 指标相似度权重
    pub metric_weight: f64,
    /// 来源相似度权重
    pub source_weight: f64,
    /// 严重性相似度权重
    pub severity_weight: f64,
    /// 默认聚类阈值
    pub default_clustering_threshold: f64,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            text_weight: 0.5,
            label_weight: 0.2,
            metric_weight: 0.15,
            source_weight: 0.1,
            severity_weight: 0.05,
            default_clustering_threshold: 0.8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alert_aggregation::{AlertSeverity, RawAlert};
    
    #[test]
    fn test_text_similarity() {
        let analyzer = SimilarityAnalyzer::new();
        
        let similarity = analyzer.calculate_text_similarity(
            "API connection timeout error",
            "API timeout error occurred"
        ).unwrap();
        
        assert!(similarity > 0.5);
        
        let no_similarity = analyzer.calculate_text_similarity(
            "Database connection error",
            "Network configuration issue"
        ).unwrap();
        
        assert!(no_similarity < 0.3);
    }
    
    #[test]
    fn test_edit_distance() {
        let analyzer = SimilarityAnalyzer::new();
        
        let distance = analyzer.calculate_edit_distance("kitten", "sitting");
        assert_eq!(distance, 3);
        
        let similarity = analyzer.calculate_edit_similarity("kitten", "sitting");
        assert!((similarity - 0.625).abs() < 0.01); // 1 - 3/8 = 0.625
    }
    
    #[test]
    fn test_jaccard_similarity() {
        let analyzer = SimilarityAnalyzer::new();
        
        let set1: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
        let set2: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();
        
        let similarity = analyzer.calculate_jaccard_similarity(&set1, &set2);
        assert!((similarity - 0.333).abs() < 0.01); // 2/6 = 0.333
    }
    
    #[test]
    fn test_severity_similarity() {
        let analyzer = SimilarityAnalyzer::new();
        
        let same = analyzer.calculate_severity_similarity(AlertSeverity::High, AlertSeverity::High);
        assert_eq!(same, 1.0);
        
        let adjacent = analyzer.calculate_severity_similarity(AlertSeverity::High, AlertSeverity::Medium);
        assert_eq!(adjacent, 0.8);
        
        let different = analyzer.calculate_severity_similarity(AlertSeverity::Critical, AlertSeverity::Info);
        assert_eq!(different, 0.0);
    }
    
    #[test]
    fn test_alert_similarity() {
        let analyzer = SimilarityAnalyzer::new();
        
        let alert1 = RawAlert::new(
            "API Error".to_string(),
            "Connection timeout to service".to_string(),
            AlertSeverity::High,
            "api_gateway".to_string(),
        );
        
        let alert2 = RawAlert::new(
            "API Timeout".to_string(),
            "Service connection timeout".to_string(),
            AlertSeverity::High,
            "api_gateway".to_string(),
        );
        
        let similarity = analyzer.calculate_alert_similarity(&alert1, &alert2).unwrap();
        assert!(similarity > 0.7); // 应该有较高相似度
        
        let alert3 = RawAlert::new(
            "Database Error".to_string(),
            "SQL query failed".to_string(),
            AlertSeverity::Low,
            "database".to_string(),
        );
        
        let low_similarity = analyzer.calculate_alert_similarity(&alert1, &alert3).unwrap();
        assert!(low_similarity < 0.3); // 应该有较低相似度
    }
    
    #[test]
    fn test_tokenization() {
        let analyzer = SimilarityAnalyzer::new();
        
        let tokens = analyzer.tokenize_and_normalize("The API connection failed!");
        assert_eq!(tokens, vec!["api", "connection", "failed"]);
        
        let empty_tokens = analyzer.tokenize_and_normalize("The and or but");
        assert!(empty_tokens.is_empty()); // 所有词都是停用词
    }
    
    #[test]
    fn test_clustering() {
        let analyzer = SimilarityAnalyzer::new();
        
        let alerts = vec![
            RawAlert::new("API Error 1".to_string(), "Timeout".to_string(), AlertSeverity::High, "api".to_string()),
            RawAlert::new("API Error 2".to_string(), "Timeout".to_string(), AlertSeverity::High, "api".to_string()),
            RawAlert::new("DB Error".to_string(), "Query failed".to_string(), AlertSeverity::Medium, "db".to_string()),
        ];
        
        let clusters = analyzer.cluster_alerts(&alerts, 0.7).unwrap();
        
        // 应该有两个聚类：API错误一组，DB错误一组
        assert_eq!(clusters.len(), 2);
        
        // 第一个聚类应该包含前两个API错误
        let api_cluster = clusters.iter().find(|cluster| cluster.contains(&0)).unwrap();
        assert!(api_cluster.contains(&1));
    }
}