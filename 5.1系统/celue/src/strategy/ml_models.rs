 
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use ndarray::{Array1, Array2};
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::strategy::adaptive_profit::{RealMLModel, MLModelType, ModelValidationResult, ModelHyperparameters};
use crate::strategy::core::StrategyError;

/// 模型版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub version_id: String,
    pub model_type: MLModelType,
    pub created_at: DateTime<Utc>,
    pub hyperparameters: ModelHyperparameters,
    pub validation_result: ModelValidationResult,
    pub model_size_bytes: usize,
    pub training_time_seconds: f64,
    pub is_production: bool,
    pub is_champion: bool,
    pub deployment_stage: DeploymentStage,
}

/// 部署阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStage {
    Development,
    Testing,
    Staging,
    Production,
    Deprecated,
}

/// A/B测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    pub test_id: String,
    pub champion_model_id: String,
    pub challenger_model_id: String,
    pub traffic_split: f64, // 0.0-1.0, challenger的流量比例
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_samples: usize,
    pub significance_level: f64,
    pub early_stopping_enabled: bool,
    pub metrics_to_track: Vec<String>,
}

/// A/B测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    pub test_id: String,
    pub champion_performance: TestPerformance,
    pub challenger_performance: TestPerformance,
    pub statistical_significance: f64,
    pub confidence_interval: (f64, f64),
    pub recommendation: TestRecommendation,
    pub completed_at: DateTime<Utc>,
}

/// 测试性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPerformance {
    pub model_id: String,
    pub sample_size: usize,
    pub mean_accuracy: f64,
    pub std_accuracy: f64,
    pub mean_profit: f64,
    pub success_rate: f64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
}

/// 测试建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestRecommendation {
    PromoteChallenger,
    KeepChampion,
    ExtendTest,
    RequireManualReview,
}

/// 模型仓库管理器
pub struct ModelRegistry {
    /// 所有模型版本
    model_versions: Arc<RwLock<HashMap<String, ModelVersion>>>,
    /// 活跃模型实例
    active_models: Arc<RwLock<HashMap<String, Arc<RealMLModel>>>>,
    /// 当前冠军模型ID
    champion_model_id: Arc<RwLock<Option<String>>>,
    /// A/B测试配置
    ab_tests: Arc<RwLock<HashMap<String, ABTestConfig>>>,
    /// A/B测试结果
    ab_test_results: Arc<RwLock<HashMap<String, ABTestResult>>>,
    /// 模型性能历史
    performance_history: Arc<RwLock<HashMap<String, Vec<ModelPerformanceRecord>>>>,
    /// 自动部署配置
    auto_deployment_config: Arc<RwLock<AutoDeploymentConfig>>,
}

/// 模型性能记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceRecord {
    pub timestamp: DateTime<Utc>,
    pub model_id: String,
    pub accuracy: f64,
    pub latency_ms: f64,
    pub memory_usage_mb: f64,
    pub prediction_count: u64,
    pub error_count: u64,
}

/// 自动部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDeploymentConfig {
    pub enabled: bool,
    pub min_improvement_threshold: f64,
    pub min_test_duration_hours: u64,
    pub min_samples_required: usize,
    pub auto_rollback_enabled: bool,
    pub rollback_threshold: f64,
    pub champion_protection_period_hours: u64,
}

impl Default for AutoDeploymentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_improvement_threshold: 0.05, // 5%
            min_test_duration_hours: 24,
            min_samples_required: 1000,
            auto_rollback_enabled: true,
            rollback_threshold: 0.02, // 2%
            champion_protection_period_hours: 72,
        }
    }
}

impl ModelRegistry {
    pub fn new(auto_deployment_config: Option<AutoDeploymentConfig>) -> Self {
        Self {
            model_versions: Arc::new(RwLock::new(HashMap::new())),
            active_models: Arc::new(RwLock::new(HashMap::new())),
            champion_model_id: Arc::new(RwLock::new(None)),
            ab_tests: Arc::new(RwLock::new(HashMap::new())),
            ab_test_results: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            auto_deployment_config: Arc::new(RwLock::new(
                auto_deployment_config.unwrap_or_default()
            )),
        }
    }
    
    /// 注册新模型版本
    pub async fn register_model(
        &self,
        model: Arc<RealMLModel>,
        model_type: MLModelType,
        hyperparameters: ModelHyperparameters,
        validation_result: ModelValidationResult,
        training_time_seconds: f64,
    ) -> Result<String, StrategyError> {
        let version_id = Uuid::new_v4().to_string();
        
        let model_version = ModelVersion {
            version_id: version_id.clone(),
            model_type: model_type.clone(),
            created_at: Utc::now(),
            hyperparameters,
            validation_result: validation_result.clone(),
            model_size_bytes: 0, // 需要实际计算
            training_time_seconds,
            is_production: false,
            is_champion: false,
            deployment_stage: DeploymentStage::Development,
        };
        
        // 注册模型版本
        self.model_versions.write().await.insert(version_id.clone(), model_version);
        
        // 添加到活跃模型
        self.active_models.write().await.insert(version_id.clone(), model);
        
        tracing::info!(
            version_id = %version_id,
            model_type = ?model_type,
            validation_r2 = %validation_result.val_r2,
            "New model version registered"
        );
        
        Ok(version_id)
    }
    
    /// 启动A/B测试
    pub async fn start_ab_test(
        &self,
        challenger_model_id: String,
        traffic_split: f64,
        duration_hours: u64,
    ) -> Result<String, StrategyError> {
        let champion_id = self.champion_model_id.read().await.clone()
            .ok_or_else(|| StrategyError::ValidationError("No champion model available".to_string()))?;
        
        let test_id = Uuid::new_v4().to_string();
        let start_time = Utc::now();
        let end_time = start_time + Duration::hours(duration_hours as i64);
        
        let ab_test_config = ABTestConfig {
            test_id: test_id.clone(),
            champion_model_id: champion_id,
            challenger_model_id,
            traffic_split,
            start_time,
            end_time,
            min_samples: 1000,
            significance_level: 0.05,
            early_stopping_enabled: true,
            metrics_to_track: vec![
                "accuracy".to_string(),
                "latency".to_string(),
                "profit".to_string(),
                "success_rate".to_string(),
            ],
        };
        
        // 验证挑战者模型存在
        if !self.active_models.read().await.contains_key(&ab_test_config.challenger_model_id) {
            return Err(StrategyError::ValidationError(
                format!("Challenger model {} not found", ab_test_config.challenger_model_id)
            ));
        }
        
        let ab_test_config_clone = ab_test_config.clone();
        self.ab_tests.write().await.insert(test_id.clone(), ab_test_config);
        
        tracing::info!(
            test_id = %test_id,
            champion_id = %ab_test_config_clone.champion_model_id,
            challenger_id = %ab_test_config_clone.challenger_model_id,
            traffic_split = %traffic_split,
            "A/B test started"
        );
        
        Ok(test_id)
    }
    
    /// 获取用于预测的模型（支持A/B测试路由）
    pub async fn get_model_for_prediction(&self, request_id: &str) -> Result<Arc<RealMLModel>, StrategyError> {
        // 检查是否有活跃的A/B测试
        let active_tests: Vec<ABTestConfig> = self.ab_tests.read().await
            .values()
            .filter(|test| {
                let now = Utc::now();
                now >= test.start_time && now <= test.end_time
            })
            .cloned()
            .collect();
        
        if let Some(test) = active_tests.first() {
            // A/B测试路由逻辑
            let use_challenger = self.should_route_to_challenger(request_id, test.traffic_split);
            
            let model_id = if use_challenger {
                &test.challenger_model_id
            } else {
                &test.champion_model_id
            };
            
            self.active_models.read().await.get(model_id)
                .ok_or_else(|| StrategyError::ModelTrainingError(format!("Model {} not found", model_id)))
                .map(|model| model.clone())
        } else {
            // 使用冠军模型
            let champion_id = self.champion_model_id.read().await.clone()
                .ok_or_else(|| StrategyError::ValidationError("No champion model available".to_string()))?;
            
            self.active_models.read().await.get(&champion_id)
                .ok_or_else(|| StrategyError::ModelTrainingError(format!("Champion model {} not found", champion_id)))
                .map(|model| model.clone())
        }
    }
    
    /// A/B测试路由决策
    fn should_route_to_challenger(&self, request_id: &str, traffic_split: f64) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        let normalized = (hash % 1000) as f64 / 1000.0;
        normalized < traffic_split
    }
    
    /// 记录模型性能
    pub async fn record_model_performance(
        &self,
        model_id: String,
        accuracy: f64,
        latency_ms: f64,
        memory_usage_mb: f64,
        prediction_count: u64,
        error_count: u64,
    ) {
        let record = ModelPerformanceRecord {
            timestamp: Utc::now(),
            model_id: model_id.clone(),
            accuracy,
            latency_ms,
            memory_usage_mb,
            prediction_count,
            error_count,
        };
        
        let mut history = self.performance_history.write().await;
        history.entry(model_id).or_insert_with(Vec::new).push(record);
    }
    
    /// 分析A/B测试结果
    pub async fn analyze_ab_test(&self, test_id: &str) -> Result<ABTestResult, StrategyError> {
        let test_config = self.ab_tests.read().await.get(test_id)
            .ok_or_else(|| StrategyError::ValidationError(format!("A/B test {} not found", test_id)))?
            .clone();
        
        let performance_history = self.performance_history.read().await;
        
        // 获取测试期间的性能数据
        let champion_performance = self.calculate_test_performance(
            &test_config.champion_model_id,
            &performance_history,
            test_config.start_time,
            Utc::now(),
        );
        
        let challenger_performance = self.calculate_test_performance(
            &test_config.challenger_model_id,
            &performance_history,
            test_config.start_time,
            Utc::now(),
        );
        
        // 统计显著性检验
        let (significance, confidence_interval) = self.statistical_test(
            &champion_performance,
            &challenger_performance,
            test_config.significance_level,
        );
        
        // 生成建议
        let recommendation = self.generate_recommendation(
            &champion_performance,
            &challenger_performance,
            significance,
            &test_config,
        ).await;
        
        let result = ABTestResult {
            test_id: test_id.to_string(),
            champion_performance,
            challenger_performance,
            statistical_significance: significance,
            confidence_interval,
            recommendation,
            completed_at: Utc::now(),
        };
        
        self.ab_test_results.write().await.insert(test_id.to_string(), result.clone());
        
        Ok(result)
    }
    
    /// 计算测试性能
    fn calculate_test_performance(
        &self,
        model_id: &str,
        performance_history: &HashMap<String, Vec<ModelPerformanceRecord>>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> TestPerformance {
        let records = performance_history.get(model_id)
            .map(|records| {
                records.iter()
                    .filter(|r| r.timestamp >= start_time && r.timestamp <= end_time)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        
        if records.is_empty() {
            return TestPerformance {
                model_id: model_id.to_string(),
                sample_size: 0,
                mean_accuracy: 0.0,
                std_accuracy: 0.0,
                mean_profit: 0.0,
                success_rate: 0.0,
                average_latency_ms: 0.0,
                error_rate: 1.0,
            };
        }
        
        let sample_size = records.len();
        let accuracies: Vec<f64> = records.iter().map(|r| r.accuracy).collect();
        let latencies: Vec<f64> = records.iter().map(|r| r.latency_ms).collect();
        
        let mean_accuracy = accuracies.iter().sum::<f64>() / sample_size as f64;
        let variance = accuracies.iter()
            .map(|&x| (x - mean_accuracy).powi(2))
            .sum::<f64>() / sample_size as f64;
        let std_accuracy = variance.sqrt();
        
        let total_predictions: u64 = records.iter().map(|r| r.prediction_count).sum();
        let total_errors: u64 = records.iter().map(|r| r.error_count).sum();
        let error_rate = if total_predictions > 0 {
            total_errors as f64 / total_predictions as f64
        } else {
            1.0
        };
        
        TestPerformance {
            model_id: model_id.to_string(),
            sample_size,
            mean_accuracy,
            std_accuracy,
            mean_profit: mean_accuracy, // 简化：使用准确率作为利润指标
            success_rate: 1.0 - error_rate,
            average_latency_ms: latencies.iter().sum::<f64>() / sample_size as f64,
            error_rate,
        }
    }
    
    /// 统计显著性检验（t检验）
    fn statistical_test(
        &self,
        champion: &TestPerformance,
        challenger: &TestPerformance,
        alpha: f64,
    ) -> (f64, (f64, f64)) {
        if champion.sample_size == 0 || challenger.sample_size == 0 {
            return (0.0, (0.0, 0.0));
        }
        
        let n1 = champion.sample_size as f64;
        let n2 = challenger.sample_size as f64;
        let mean1 = champion.mean_accuracy;
        let mean2 = challenger.mean_accuracy;
        let std1 = champion.std_accuracy;
        let std2 = challenger.std_accuracy;
        
        // Welch's t-test
        let pooled_std = ((std1.powi(2) / n1) + (std2.powi(2) / n2)).sqrt();
        if pooled_std == 0.0 {
            return (0.0, (0.0, 0.0));
        }
        
        let t_stat = (mean2 - mean1) / pooled_std;
        let df = ((std1.powi(2) / n1) + (std2.powi(2) / n2)).powi(2) /
                 ((std1.powi(2) / n1).powi(2) / (n1 - 1.0) + (std2.powi(2) / n2).powi(2) / (n2 - 1.0));
        
        // 简化的p值计算（实际应该使用t分布）
        let p_value = 2.0 * (1.0 - (t_stat.abs() / (1.0 + t_stat.abs())));
        
        // 置信区间
        let margin_error = 1.96 * pooled_std; // 95% CI
        let confidence_interval = (
            (mean2 - mean1) - margin_error,
            (mean2 - mean1) + margin_error,
        );
        
        (1.0 - p_value, confidence_interval)
    }
    
    /// 生成测试建议
    async fn generate_recommendation(
        &self,
        champion: &TestPerformance,
        challenger: &TestPerformance,
        significance: f64,
        test_config: &ABTestConfig,
    ) -> TestRecommendation {
        let config = self.auto_deployment_config.read().await;
        
        // 检查样本量
        if challenger.sample_size < config.min_samples_required {
            return TestRecommendation::ExtendTest;
        }
        
        // 检查统计显著性
        if significance < (1.0 - test_config.significance_level) {
            return TestRecommendation::ExtendTest;
        }
        
        // 检查性能改进
        let improvement = (challenger.mean_accuracy - champion.mean_accuracy) / champion.mean_accuracy;
        
        if improvement >= config.min_improvement_threshold {
            // 检查其他指标
            if challenger.average_latency_ms <= champion.average_latency_ms * 1.1 && // 延迟不能增加超过10%
               challenger.error_rate <= champion.error_rate * 1.1 { // 错误率不能增加超过10%
                TestRecommendation::PromoteChallenger
            } else {
                TestRecommendation::RequireManualReview
            }
        } else if improvement < -config.rollback_threshold {
            TestRecommendation::KeepChampion
        } else {
            TestRecommendation::RequireManualReview
        }
    }
    
    /// 自动执行A/B测试决策
    pub async fn auto_execute_ab_decision(&self, test_id: &str) -> Result<(), StrategyError> {
        let config = self.auto_deployment_config.read().await;
        if !config.enabled {
            return Ok(());
        }
        
        let result = self.analyze_ab_test(test_id).await?;
        
        match result.recommendation {
            TestRecommendation::PromoteChallenger => {
                self.promote_challenger(&result.challenger_performance.model_id).await?;
                tracing::info!(
                    test_id = %test_id,
                    new_champion = %result.challenger_performance.model_id,
                    "Challenger automatically promoted to champion"
                );
            },
            TestRecommendation::KeepChampion => {
                tracing::info!(
                    test_id = %test_id,
                    "Champion model retained after A/B test"
                );
            },
            _ => {
                tracing::warn!(
                    test_id = %test_id,
                    recommendation = ?result.recommendation,
                    "A/B test requires manual review"
                );
            }
        }
        
        Ok(())
    }
    
    /// 提升挑战者为冠军
    pub async fn promote_challenger(&self, challenger_id: &str) -> Result<(), StrategyError> {
        // 更新模型版本状态
        {
            let mut versions = self.model_versions.write().await;
            
            // 降级当前冠军
            if let Some(current_champion_id) = &*self.champion_model_id.read().await {
                if let Some(champion_version) = versions.get_mut(current_champion_id) {
                    champion_version.is_champion = false;
                    champion_version.deployment_stage = DeploymentStage::Production;
                }
            }
            
            // 提升挑战者
            if let Some(challenger_version) = versions.get_mut(challenger_id) {
                challenger_version.is_champion = true;
                challenger_version.is_production = true;
                challenger_version.deployment_stage = DeploymentStage::Production;
            } else {
                return Err(StrategyError::ModelTrainingError(format!("Challenger model {} not found", challenger_id)));
            }
        }
        
        // 更新冠军模型ID
        *self.champion_model_id.write().await = Some(challenger_id.to_string());
        
        Ok(())
    }
    
    /// 获取模型版本信息
    pub async fn get_model_version(&self, version_id: &str) -> Option<ModelVersion> {
        self.model_versions.read().await.get(version_id).cloned()
    }
    
    /// 获取所有模型版本
    pub async fn list_model_versions(&self) -> Vec<ModelVersion> {
        self.model_versions.read().await.values().cloned().collect()
    }
    
    /// 获取当前冠军模型ID
    pub async fn get_champion_model_id(&self) -> Option<String> {
        self.champion_model_id.read().await.clone()
    }
    
    /// 获取活跃的A/B测试
    pub async fn get_active_ab_tests(&self) -> Vec<ABTestConfig> {
        let now = Utc::now();
        self.ab_tests.read().await
            .values()
            .filter(|test| now >= test.start_time && now <= test.end_time)
            .cloned()
            .collect()
    }
    
    /// 停止A/B测试
    pub async fn stop_ab_test(&self, test_id: &str) -> Result<(), StrategyError> {
        let mut ab_tests = self.ab_tests.write().await;
        if let Some(mut test) = ab_tests.remove(test_id) {
            test.end_time = Utc::now();
            ab_tests.insert(test_id.to_string(), test);
            
            tracing::info!(test_id = %test_id, "A/B test manually stopped");
            Ok(())
        } else {
            Err(StrategyError::ValidationError(format!("A/B test {} not found", test_id)))
        }
    }
    
    /// 清理过期的模型版本
    pub async fn cleanup_old_versions(&self, keep_latest_n: usize) -> Result<(), StrategyError> {
        let mut versions = self.model_versions.write().await;
        let mut active_models = self.active_models.write().await;
        
        // 按创建时间排序
        let mut version_list: Vec<_> = versions.values().cloned().collect();
        version_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // 保护冠军模型和生产环境模型
        let protected_ids: std::collections::HashSet<String> = version_list.iter()
            .filter(|v| v.is_champion || v.is_production || v.deployment_stage == DeploymentStage::Production)
            .map(|v| v.version_id.clone())
            .collect();
        
        // 标记要删除的版本
        let mut to_remove = Vec::new();
        let mut kept_count = 0;
        
        for version in version_list {
            if protected_ids.contains(&version.version_id) {
                continue; // 保护的模型不删除
            }
            
            if kept_count >= keep_latest_n {
                to_remove.push(version.version_id.clone());
            } else {
                kept_count += 1;
            }
        }
        
        // 删除旧版本
        for version_id in to_remove {
            versions.remove(&version_id);
            active_models.remove(&version_id);
            
            tracing::debug!(version_id = %version_id, "Old model version cleaned up");
        }
        
        Ok(())
    }
} 
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use ndarray::{Array1, Array2};
use uuid::Uuid;
use rand::{thread_rng, Rng};

use crate::strategy::adaptive_profit::{RealMLModel, MLModelType, ModelValidationResult, ModelHyperparameters};
use crate::strategy::core::StrategyError;

/// 模型版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    pub version_id: String,
    pub model_type: MLModelType,
    pub created_at: DateTime<Utc>,
    pub hyperparameters: ModelHyperparameters,
    pub validation_result: ModelValidationResult,
    pub model_size_bytes: usize,
    pub training_time_seconds: f64,
    pub is_production: bool,
    pub is_champion: bool,
    pub deployment_stage: DeploymentStage,
}

/// 部署阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStage {
    Development,
    Testing,
    Staging,
    Production,
    Deprecated,
}

/// A/B测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestConfig {
    pub test_id: String,
    pub champion_model_id: String,
    pub challenger_model_id: String,
    pub traffic_split: f64, // 0.0-1.0, challenger的流量比例
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub min_samples: usize,
    pub significance_level: f64,
    pub early_stopping_enabled: bool,
    pub metrics_to_track: Vec<String>,
}

/// A/B测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestResult {
    pub test_id: String,
    pub champion_performance: TestPerformance,
    pub challenger_performance: TestPerformance,
    pub statistical_significance: f64,
    pub confidence_interval: (f64, f64),
    pub recommendation: TestRecommendation,
    pub completed_at: DateTime<Utc>,
}

/// 测试性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPerformance {
    pub model_id: String,
    pub sample_size: usize,
    pub mean_accuracy: f64,
    pub std_accuracy: f64,
    pub mean_profit: f64,
    pub success_rate: f64,
    pub average_latency_ms: f64,
    pub error_rate: f64,
}

/// 测试建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestRecommendation {
    PromoteChallenger,
    KeepChampion,
    ExtendTest,
    RequireManualReview,
}

/// 模型仓库管理器
pub struct ModelRegistry {
    /// 所有模型版本
    model_versions: Arc<RwLock<HashMap<String, ModelVersion>>>,
    /// 活跃模型实例
    active_models: Arc<RwLock<HashMap<String, Arc<RealMLModel>>>>,
    /// 当前冠军模型ID
    champion_model_id: Arc<RwLock<Option<String>>>,
    /// A/B测试配置
    ab_tests: Arc<RwLock<HashMap<String, ABTestConfig>>>,
    /// A/B测试结果
    ab_test_results: Arc<RwLock<HashMap<String, ABTestResult>>>,
    /// 模型性能历史
    performance_history: Arc<RwLock<HashMap<String, Vec<ModelPerformanceRecord>>>>,
    /// 自动部署配置
    auto_deployment_config: Arc<RwLock<AutoDeploymentConfig>>,
}

/// 模型性能记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformanceRecord {
    pub timestamp: DateTime<Utc>,
    pub model_id: String,
    pub accuracy: f64,
    pub latency_ms: f64,
    pub memory_usage_mb: f64,
    pub prediction_count: u64,
    pub error_count: u64,
}

/// 自动部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDeploymentConfig {
    pub enabled: bool,
    pub min_improvement_threshold: f64,
    pub min_test_duration_hours: u64,
    pub min_samples_required: usize,
    pub auto_rollback_enabled: bool,
    pub rollback_threshold: f64,
    pub champion_protection_period_hours: u64,
}

impl Default for AutoDeploymentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_improvement_threshold: 0.05, // 5%
            min_test_duration_hours: 24,
            min_samples_required: 1000,
            auto_rollback_enabled: true,
            rollback_threshold: 0.02, // 2%
            champion_protection_period_hours: 72,
        }
    }
}

impl ModelRegistry {
    pub fn new(auto_deployment_config: Option<AutoDeploymentConfig>) -> Self {
        Self {
            model_versions: Arc::new(RwLock::new(HashMap::new())),
            active_models: Arc::new(RwLock::new(HashMap::new())),
            champion_model_id: Arc::new(RwLock::new(None)),
            ab_tests: Arc::new(RwLock::new(HashMap::new())),
            ab_test_results: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            auto_deployment_config: Arc::new(RwLock::new(
                auto_deployment_config.unwrap_or_default()
            )),
        }
    }
    
    /// 注册新模型版本
    pub async fn register_model(
        &self,
        model: Arc<RealMLModel>,
        model_type: MLModelType,
        hyperparameters: ModelHyperparameters,
        validation_result: ModelValidationResult,
        training_time_seconds: f64,
    ) -> Result<String, StrategyError> {
        let version_id = Uuid::new_v4().to_string();
        
        let model_version = ModelVersion {
            version_id: version_id.clone(),
            model_type: model_type.clone(),
            created_at: Utc::now(),
            hyperparameters,
            validation_result: validation_result.clone(),
            model_size_bytes: 0, // 需要实际计算
            training_time_seconds,
            is_production: false,
            is_champion: false,
            deployment_stage: DeploymentStage::Development,
        };
        
        // 注册模型版本
        self.model_versions.write().await.insert(version_id.clone(), model_version);
        
        // 添加到活跃模型
        self.active_models.write().await.insert(version_id.clone(), model);
        
        tracing::info!(
            version_id = %version_id,
            model_type = ?model_type,
            validation_r2 = %validation_result.val_r2,
            "New model version registered"
        );
        
        Ok(version_id)
    }
    
    /// 启动A/B测试
    pub async fn start_ab_test(
        &self,
        challenger_model_id: String,
        traffic_split: f64,
        duration_hours: u64,
    ) -> Result<String, StrategyError> {
        let champion_id = self.champion_model_id.read().await.clone()
            .ok_or_else(|| StrategyError::ValidationError("No champion model available".to_string()))?;
        
        let test_id = Uuid::new_v4().to_string();
        let start_time = Utc::now();
        let end_time = start_time + Duration::hours(duration_hours as i64);
        
        let ab_test_config = ABTestConfig {
            test_id: test_id.clone(),
            champion_model_id: champion_id,
            challenger_model_id,
            traffic_split,
            start_time,
            end_time,
            min_samples: 1000,
            significance_level: 0.05,
            early_stopping_enabled: true,
            metrics_to_track: vec![
                "accuracy".to_string(),
                "latency".to_string(),
                "profit".to_string(),
                "success_rate".to_string(),
            ],
        };
        
        // 验证挑战者模型存在
        if !self.active_models.read().await.contains_key(&ab_test_config.challenger_model_id) {
            return Err(StrategyError::ValidationError(
                format!("Challenger model {} not found", ab_test_config.challenger_model_id)
            ));
        }
        
        let ab_test_config_clone = ab_test_config.clone();
        self.ab_tests.write().await.insert(test_id.clone(), ab_test_config);
        
        tracing::info!(
            test_id = %test_id,
            champion_id = %ab_test_config_clone.champion_model_id,
            challenger_id = %ab_test_config_clone.challenger_model_id,
            traffic_split = %traffic_split,
            "A/B test started"
        );
        
        Ok(test_id)
    }
    
    /// 获取用于预测的模型（支持A/B测试路由）
    pub async fn get_model_for_prediction(&self, request_id: &str) -> Result<Arc<RealMLModel>, StrategyError> {
        // 检查是否有活跃的A/B测试
        let active_tests: Vec<ABTestConfig> = self.ab_tests.read().await
            .values()
            .filter(|test| {
                let now = Utc::now();
                now >= test.start_time && now <= test.end_time
            })
            .cloned()
            .collect();
        
        if let Some(test) = active_tests.first() {
            // A/B测试路由逻辑
            let use_challenger = self.should_route_to_challenger(request_id, test.traffic_split);
            
            let model_id = if use_challenger {
                &test.challenger_model_id
            } else {
                &test.champion_model_id
            };
            
            self.active_models.read().await.get(model_id)
                .ok_or_else(|| StrategyError::ModelTrainingError(format!("Model {} not found", model_id)))
                .map(|model| model.clone())
        } else {
            // 使用冠军模型
            let champion_id = self.champion_model_id.read().await.clone()
                .ok_or_else(|| StrategyError::ValidationError("No champion model available".to_string()))?;
            
            self.active_models.read().await.get(&champion_id)
                .ok_or_else(|| StrategyError::ModelTrainingError(format!("Champion model {} not found", champion_id)))
                .map(|model| model.clone())
        }
    }
    
    /// A/B测试路由决策
    fn should_route_to_challenger(&self, request_id: &str, traffic_split: f64) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request_id.hash(&mut hasher);
        let hash = hasher.finish();
        
        let normalized = (hash % 1000) as f64 / 1000.0;
        normalized < traffic_split
    }
    
    /// 记录模型性能
    pub async fn record_model_performance(
        &self,
        model_id: String,
        accuracy: f64,
        latency_ms: f64,
        memory_usage_mb: f64,
        prediction_count: u64,
        error_count: u64,
    ) {
        let record = ModelPerformanceRecord {
            timestamp: Utc::now(),
            model_id: model_id.clone(),
            accuracy,
            latency_ms,
            memory_usage_mb,
            prediction_count,
            error_count,
        };
        
        let mut history = self.performance_history.write().await;
        history.entry(model_id).or_insert_with(Vec::new).push(record);
    }
    
    /// 分析A/B测试结果
    pub async fn analyze_ab_test(&self, test_id: &str) -> Result<ABTestResult, StrategyError> {
        let test_config = self.ab_tests.read().await.get(test_id)
            .ok_or_else(|| StrategyError::ValidationError(format!("A/B test {} not found", test_id)))?
            .clone();
        
        let performance_history = self.performance_history.read().await;
        
        // 获取测试期间的性能数据
        let champion_performance = self.calculate_test_performance(
            &test_config.champion_model_id,
            &performance_history,
            test_config.start_time,
            Utc::now(),
        );
        
        let challenger_performance = self.calculate_test_performance(
            &test_config.challenger_model_id,
            &performance_history,
            test_config.start_time,
            Utc::now(),
        );
        
        // 统计显著性检验
        let (significance, confidence_interval) = self.statistical_test(
            &champion_performance,
            &challenger_performance,
            test_config.significance_level,
        );
        
        // 生成建议
        let recommendation = self.generate_recommendation(
            &champion_performance,
            &challenger_performance,
            significance,
            &test_config,
        ).await;
        
        let result = ABTestResult {
            test_id: test_id.to_string(),
            champion_performance,
            challenger_performance,
            statistical_significance: significance,
            confidence_interval,
            recommendation,
            completed_at: Utc::now(),
        };
        
        self.ab_test_results.write().await.insert(test_id.to_string(), result.clone());
        
        Ok(result)
    }
    
    /// 计算测试性能
    fn calculate_test_performance(
        &self,
        model_id: &str,
        performance_history: &HashMap<String, Vec<ModelPerformanceRecord>>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> TestPerformance {
        let records = performance_history.get(model_id)
            .map(|records| {
                records.iter()
                    .filter(|r| r.timestamp >= start_time && r.timestamp <= end_time)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        
        if records.is_empty() {
            return TestPerformance {
                model_id: model_id.to_string(),
                sample_size: 0,
                mean_accuracy: 0.0,
                std_accuracy: 0.0,
                mean_profit: 0.0,
                success_rate: 0.0,
                average_latency_ms: 0.0,
                error_rate: 1.0,
            };
        }
        
        let sample_size = records.len();
        let accuracies: Vec<f64> = records.iter().map(|r| r.accuracy).collect();
        let latencies: Vec<f64> = records.iter().map(|r| r.latency_ms).collect();
        
        let mean_accuracy = accuracies.iter().sum::<f64>() / sample_size as f64;
        let variance = accuracies.iter()
            .map(|&x| (x - mean_accuracy).powi(2))
            .sum::<f64>() / sample_size as f64;
        let std_accuracy = variance.sqrt();
        
        let total_predictions: u64 = records.iter().map(|r| r.prediction_count).sum();
        let total_errors: u64 = records.iter().map(|r| r.error_count).sum();
        let error_rate = if total_predictions > 0 {
            total_errors as f64 / total_predictions as f64
        } else {
            1.0
        };
        
        TestPerformance {
            model_id: model_id.to_string(),
            sample_size,
            mean_accuracy,
            std_accuracy,
            mean_profit: mean_accuracy, // 简化：使用准确率作为利润指标
            success_rate: 1.0 - error_rate,
            average_latency_ms: latencies.iter().sum::<f64>() / sample_size as f64,
            error_rate,
        }
    }
    
    /// 统计显著性检验（t检验）
    fn statistical_test(
        &self,
        champion: &TestPerformance,
        challenger: &TestPerformance,
        alpha: f64,
    ) -> (f64, (f64, f64)) {
        if champion.sample_size == 0 || challenger.sample_size == 0 {
            return (0.0, (0.0, 0.0));
        }
        
        let n1 = champion.sample_size as f64;
        let n2 = challenger.sample_size as f64;
        let mean1 = champion.mean_accuracy;
        let mean2 = challenger.mean_accuracy;
        let std1 = champion.std_accuracy;
        let std2 = challenger.std_accuracy;
        
        // Welch's t-test
        let pooled_std = ((std1.powi(2) / n1) + (std2.powi(2) / n2)).sqrt();
        if pooled_std == 0.0 {
            return (0.0, (0.0, 0.0));
        }
        
        let t_stat = (mean2 - mean1) / pooled_std;
        let df = ((std1.powi(2) / n1) + (std2.powi(2) / n2)).powi(2) /
                 ((std1.powi(2) / n1).powi(2) / (n1 - 1.0) + (std2.powi(2) / n2).powi(2) / (n2 - 1.0));
        
        // 简化的p值计算（实际应该使用t分布）
        let p_value = 2.0 * (1.0 - (t_stat.abs() / (1.0 + t_stat.abs())));
        
        // 置信区间
        let margin_error = 1.96 * pooled_std; // 95% CI
        let confidence_interval = (
            (mean2 - mean1) - margin_error,
            (mean2 - mean1) + margin_error,
        );
        
        (1.0 - p_value, confidence_interval)
    }
    
    /// 生成测试建议
    async fn generate_recommendation(
        &self,
        champion: &TestPerformance,
        challenger: &TestPerformance,
        significance: f64,
        test_config: &ABTestConfig,
    ) -> TestRecommendation {
        let config = self.auto_deployment_config.read().await;
        
        // 检查样本量
        if challenger.sample_size < config.min_samples_required {
            return TestRecommendation::ExtendTest;
        }
        
        // 检查统计显著性
        if significance < (1.0 - test_config.significance_level) {
            return TestRecommendation::ExtendTest;
        }
        
        // 检查性能改进
        let improvement = (challenger.mean_accuracy - champion.mean_accuracy) / champion.mean_accuracy;
        
        if improvement >= config.min_improvement_threshold {
            // 检查其他指标
            if challenger.average_latency_ms <= champion.average_latency_ms * 1.1 && // 延迟不能增加超过10%
               challenger.error_rate <= champion.error_rate * 1.1 { // 错误率不能增加超过10%
                TestRecommendation::PromoteChallenger
            } else {
                TestRecommendation::RequireManualReview
            }
        } else if improvement < -config.rollback_threshold {
            TestRecommendation::KeepChampion
        } else {
            TestRecommendation::RequireManualReview
        }
    }
    
    /// 自动执行A/B测试决策
    pub async fn auto_execute_ab_decision(&self, test_id: &str) -> Result<(), StrategyError> {
        let config = self.auto_deployment_config.read().await;
        if !config.enabled {
            return Ok(());
        }
        
        let result = self.analyze_ab_test(test_id).await?;
        
        match result.recommendation {
            TestRecommendation::PromoteChallenger => {
                self.promote_challenger(&result.challenger_performance.model_id).await?;
                tracing::info!(
                    test_id = %test_id,
                    new_champion = %result.challenger_performance.model_id,
                    "Challenger automatically promoted to champion"
                );
            },
            TestRecommendation::KeepChampion => {
                tracing::info!(
                    test_id = %test_id,
                    "Champion model retained after A/B test"
                );
            },
            _ => {
                tracing::warn!(
                    test_id = %test_id,
                    recommendation = ?result.recommendation,
                    "A/B test requires manual review"
                );
            }
        }
        
        Ok(())
    }
    
    /// 提升挑战者为冠军
    pub async fn promote_challenger(&self, challenger_id: &str) -> Result<(), StrategyError> {
        // 更新模型版本状态
        {
            let mut versions = self.model_versions.write().await;
            
            // 降级当前冠军
            if let Some(current_champion_id) = &*self.champion_model_id.read().await {
                if let Some(champion_version) = versions.get_mut(current_champion_id) {
                    champion_version.is_champion = false;
                    champion_version.deployment_stage = DeploymentStage::Production;
                }
            }
            
            // 提升挑战者
            if let Some(challenger_version) = versions.get_mut(challenger_id) {
                challenger_version.is_champion = true;
                challenger_version.is_production = true;
                challenger_version.deployment_stage = DeploymentStage::Production;
            } else {
                return Err(StrategyError::ModelTrainingError(format!("Challenger model {} not found", challenger_id)));
            }
        }
        
        // 更新冠军模型ID
        *self.champion_model_id.write().await = Some(challenger_id.to_string());
        
        Ok(())
    }
    
    /// 获取模型版本信息
    pub async fn get_model_version(&self, version_id: &str) -> Option<ModelVersion> {
        self.model_versions.read().await.get(version_id).cloned()
    }
    
    /// 获取所有模型版本
    pub async fn list_model_versions(&self) -> Vec<ModelVersion> {
        self.model_versions.read().await.values().cloned().collect()
    }
    
    /// 获取当前冠军模型ID
    pub async fn get_champion_model_id(&self) -> Option<String> {
        self.champion_model_id.read().await.clone()
    }
    
    /// 获取活跃的A/B测试
    pub async fn get_active_ab_tests(&self) -> Vec<ABTestConfig> {
        let now = Utc::now();
        self.ab_tests.read().await
            .values()
            .filter(|test| now >= test.start_time && now <= test.end_time)
            .cloned()
            .collect()
    }
    
    /// 停止A/B测试
    pub async fn stop_ab_test(&self, test_id: &str) -> Result<(), StrategyError> {
        let mut ab_tests = self.ab_tests.write().await;
        if let Some(mut test) = ab_tests.remove(test_id) {
            test.end_time = Utc::now();
            ab_tests.insert(test_id.to_string(), test);
            
            tracing::info!(test_id = %test_id, "A/B test manually stopped");
            Ok(())
        } else {
            Err(StrategyError::ValidationError(format!("A/B test {} not found", test_id)))
        }
    }
    
    /// 清理过期的模型版本
    pub async fn cleanup_old_versions(&self, keep_latest_n: usize) -> Result<(), StrategyError> {
        let mut versions = self.model_versions.write().await;
        let mut active_models = self.active_models.write().await;
        
        // 按创建时间排序
        let mut version_list: Vec<_> = versions.values().cloned().collect();
        version_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // 保护冠军模型和生产环境模型
        let protected_ids: std::collections::HashSet<String> = version_list.iter()
            .filter(|v| v.is_champion || v.is_production || v.deployment_stage == DeploymentStage::Production)
            .map(|v| v.version_id.clone())
            .collect();
        
        // 标记要删除的版本
        let mut to_remove = Vec::new();
        let mut kept_count = 0;
        
        for version in version_list {
            if protected_ids.contains(&version.version_id) {
                continue; // 保护的模型不删除
            }
            
            if kept_count >= keep_latest_n {
                to_remove.push(version.version_id.clone());
            } else {
                kept_count += 1;
            }
        }
        
        // 删除旧版本
        for version_id in to_remove {
            versions.remove(&version_id);
            active_models.remove(&version_id);
            
            tracing::debug!(version_id = %version_id, "Old model version cleaned up");
        }
        
        Ok(())
    }
} 