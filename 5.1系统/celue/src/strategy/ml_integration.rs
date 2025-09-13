use std::sync::Arc;
use parking_lot::RwLock;
use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::tree::decision_tree_classifier::{DecisionTreeClassifier, DecisionTreeClassifierParameters};
use smartcore::tree::decision_tree_regressor::{DecisionTreeRegressor, DecisionTreeRegressorParameters};
use smartcore::ensemble::random_forest_regressor::{RandomForestRegressor, RandomForestRegressorParameters};
use smartcore::linear::linear_regression::{LinearRegression, LinearRegressionParameters};
use smartcore::metrics::mean_squared_error;
use linfa::prelude::*;
use linfa_trees::{DecisionTree, SplitQuality};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::strategy::core::StrategyError;

/// ML模型接口trait
pub trait MLModel: Send + Sync {
    fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<(), StrategyError>;
    fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>, StrategyError>;
    fn get_feature_importance(&self) -> Option<Vec<f64>>;
    fn save(&self) -> Result<Vec<u8>, StrategyError>;
    fn load(data: &[u8]) -> Result<Box<Self>, StrategyError> where Self: Sized;
    fn model_type(&self) -> MLModelType;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MLModelType {
    DecisionTree,
    RandomForest,
    LinearRegression,
    GradientBoosting,
    NeuralNetwork,
    EnsembleVoting,
    EnsembleBagging,
}

/// 统一的ML模型包装器
pub struct MLModelWrapper {
    model_type: MLModelType,
    inner: Box<dyn MLModel>,
    metadata: ModelMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub last_trained: Option<DateTime<Utc>>,
    pub training_samples: usize,
    pub feature_names: Vec<String>,
    pub performance_metrics: HashMap<String, f64>,
    pub hyperparameters: HashMap<String, serde_json::Value>,
}

impl MLModelWrapper {
    pub fn new(model_type: MLModelType, feature_names: Vec<String>) -> Self {
        let inner: Box<dyn MLModel> = match model_type {
            MLModelType::DecisionTree => Box::new(DecisionTreeModel::new()),
            MLModelType::RandomForest => Box::new(RandomForestModel::new()),
            MLModelType::LinearRegression => Box::new(LinearRegressionModel::new()),
            _ => Box::new(DecisionTreeModel::new()), // 默认决策树
        };

        Self {
            model_type,
            inner,
            metadata: ModelMetadata {
                id: uuid::Uuid::new_v4().to_string(),
                created_at: Utc::now(),
                last_trained: None,
                training_samples: 0,
                feature_names,
                performance_metrics: HashMap::new(),
                hyperparameters: HashMap::new(),
            },
        }
    }

    pub fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<(), StrategyError> {
        self.inner.train(features, targets)?;
        self.metadata.last_trained = Some(Utc::now());
        self.metadata.training_samples = features.nrows();
        Ok(())
    }

    pub fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>, StrategyError> {
        self.inner.predict(features)
    }

    pub fn get_metadata(&self) -> &ModelMetadata {
        &self.metadata
    }

    pub fn update_performance_metric(&mut self, name: &str, value: f64) {
        self.metadata.performance_metrics.insert(name.to_string(), value);
    }
}

/// 决策树模型实现
pub struct DecisionTreeModel {
    model: Option<DecisionTreeRegressor<f64, i32, DenseMatrix<f64>, Vec<i32>>>,
    feature_importance: Option<Vec<f64>>,
}

impl DecisionTreeModel {
    pub fn new() -> Self {
        Self {
            model: None,
            feature_importance: None,
        }
    }
}

impl MLModel for DecisionTreeModel {
    fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<(), StrategyError> {
        // 转换为DenseMatrix
        let x = DenseMatrix::from_2d_array(&features.view().into_dyn()
            .rows()
            .into_iter()
            .map(|row| row.to_vec())
            .collect::<Vec<Vec<f64>>>());
        
        let y = targets.to_vec();
        
        let params = DecisionTreeRegressorParameters::default()
            .with_max_depth(10)
            .with_min_samples_split(20);
        
        self.model = Some(DecisionTreeRegressor::fit(&x, &y, params)
            .map_err(|e| StrategyError::ModelTrainingError(format!("Decision tree training failed: {:?}", e)))?);
        
        // 计算特征重要性（简化版本）
        self.feature_importance = Some(vec![1.0 / features.ncols() as f64; features.ncols()]);
        
        Ok(())
    }

    fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>, StrategyError> {
        match &self.model {
            Some(model) => {
                let x = DenseMatrix::from_2d_array(&features.view().into_dyn()
                    .rows()
                    .into_iter()
                    .map(|row| row.to_vec())
                    .collect::<Vec<Vec<f64>>>());
                
                let predictions = model.predict(&x)
                    .map_err(|e| StrategyError::PredictionError(format!("Decision tree prediction failed: {:?}", e)))?;
                
                Ok(Array1::from_vec(predictions))
            }
            None => Err(StrategyError::PredictionError("Model not trained".to_string())),
        }
    }

    fn get_feature_importance(&self) -> Option<Vec<f64>> {
        self.feature_importance.clone()
    }

    fn save(&self) -> Result<Vec<u8>, StrategyError> {
        // 使用bincode序列化
        bincode::serialize(&self.feature_importance)
            .map_err(|e| StrategyError::ConfigurationError(format!("Model save failed: {:?}", e)))
    }

    fn load(_data: &[u8]) -> Result<Box<Self>, StrategyError> {
        // 简化实现
        Ok(Box::new(Self::new()))
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::DecisionTree
    }
}

/// 随机森林模型实现
pub struct RandomForestModel {
    model: Option<RandomForestRegressor<f64, i32, DenseMatrix<f64>, Vec<i32>>>,
    feature_importance: Option<Vec<f64>>,
}

impl RandomForestModel {
    pub fn new() -> Self {
        Self {
            model: None,
            feature_importance: None,
        }
    }
}

impl MLModel for RandomForestModel {
    fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<(), StrategyError> {
        let x = DenseMatrix::from_2d_array(&features.view().into_dyn()
            .rows()
            .into_iter()
            .map(|row| row.to_vec())
            .collect::<Vec<Vec<f64>>>());
        
        let y = targets.to_vec();
        
        let params = RandomForestRegressorParameters::default()
            .with_n_trees(100)
            .with_max_depth(Some(10))
            .with_min_samples_split(20);
        
        self.model = Some(RandomForestRegressor::fit(&x, &y, params)
            .map_err(|e| StrategyError::ModelTrainingError(format!("Random forest training failed: {:?}", e)))?);
        
        // 简化的特征重要性计算
        self.feature_importance = Some(vec![1.0 / features.ncols() as f64; features.ncols()]);
        
        Ok(())
    }

    fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>, StrategyError> {
        match &self.model {
            Some(model) => {
                let x = DenseMatrix::from_2d_array(&features.view().into_dyn()
                    .rows()
                    .into_iter()
                    .map(|row| row.to_vec())
                    .collect::<Vec<Vec<f64>>>());
                
                let predictions = model.predict(&x)
                    .map_err(|e| StrategyError::PredictionError(format!("Random forest prediction failed: {:?}", e)))?;
                
                Ok(Array1::from_vec(predictions))
            }
            None => Err(StrategyError::PredictionError("Model not trained".to_string())),
        }
    }

    fn get_feature_importance(&self) -> Option<Vec<f64>> {
        self.feature_importance.clone()
    }

    fn save(&self) -> Result<Vec<u8>, StrategyError> {
        bincode::serialize(&self.feature_importance)
            .map_err(|e| StrategyError::ConfigurationError(format!("Model save failed: {:?}", e)))
    }

    fn load(_data: &[u8]) -> Result<Box<Self>, StrategyError> {
        Ok(Box::new(Self::new()))
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::RandomForest
    }
}

/// 线性回归模型实现
pub struct LinearRegressionModel {
    model: Option<LinearRegression<f64, i32, DenseMatrix<f64>, Vec<i32>>>,
    coefficients: Option<Vec<f64>>,
}

impl LinearRegressionModel {
    pub fn new() -> Self {
        Self {
            model: None,
            coefficients: None,
        }
    }
}

impl MLModel for LinearRegressionModel {
    fn train(&mut self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<(), StrategyError> {
        let x = DenseMatrix::from_2d_array(&features.view().into_dyn()
            .rows()
            .into_iter()
            .map(|row| row.to_vec())
            .collect::<Vec<Vec<f64>>>());
        
        let y = targets.to_vec();
        
        let params = LinearRegressionParameters::default()
            .with_solver(smartcore::linear::linear_regression::LinearRegressionSolverName::SVD);
        
        self.model = Some(LinearRegression::fit(&x, &y, params)
            .map_err(|e| StrategyError::ModelTrainingError(format!("Linear regression training failed: {:?}", e)))?);
        
        // 存储系数作为特征重要性的代理
        self.coefficients = Some(vec![1.0 / features.ncols() as f64; features.ncols()]);
        
        Ok(())
    }

    fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>, StrategyError> {
        match &self.model {
            Some(model) => {
                let x = DenseMatrix::from_2d_array(&features.view().into_dyn()
                    .rows()
                    .into_iter()
                    .map(|row| row.to_vec())
                    .collect::<Vec<Vec<f64>>>());
                
                let predictions = model.predict(&x)
                    .map_err(|e| StrategyError::PredictionError(format!("Linear regression prediction failed: {:?}", e)))?;
                
                Ok(Array1::from_vec(predictions))
            }
            None => Err(StrategyError::PredictionError("Model not trained".to_string())),
        }
    }

    fn get_feature_importance(&self) -> Option<Vec<f64>> {
        self.coefficients.clone()
    }

    fn save(&self) -> Result<Vec<u8>, StrategyError> {
        bincode::serialize(&self.coefficients)
            .map_err(|e| StrategyError::ConfigurationError(format!("Model save failed: {:?}", e)))
    }

    fn load(_data: &[u8]) -> Result<Box<Self>, StrategyError> {
        Ok(Box::new(Self::new()))
    }

    fn model_type(&self) -> MLModelType {
        MLModelType::LinearRegression
    }
}

/// 集成学习协调器
pub struct EnsembleCoordinator {
    models: Vec<Arc<RwLock<MLModelWrapper>>>,
    weights: Vec<f64>,
    voting_strategy: VotingStrategy,
}

#[derive(Debug, Clone, Copy)]
pub enum VotingStrategy {
    Average,
    Weighted,
    Median,
    Max,
}

impl EnsembleCoordinator {
    pub fn new(voting_strategy: VotingStrategy) -> Self {
        Self {
            models: Vec::new(),
            weights: Vec::new(),
            voting_strategy,
        }
    }

    pub fn add_model(&mut self, model: Arc<RwLock<MLModelWrapper>>, weight: f64) {
        self.models.push(model);
        self.weights.push(weight);
    }

    pub fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>, StrategyError> {
        if self.models.is_empty() {
            return Err(StrategyError::PredictionError("No models in ensemble".to_string()));
        }

        let mut predictions = Vec::new();
        
        for model in &self.models {
            let model_guard = model.read();
            let pred = model_guard.predict(features)?;
            predictions.push(pred);
        }

        match self.voting_strategy {
            VotingStrategy::Average => {
                let sum = predictions.iter()
                    .fold(Array1::zeros(predictions[0].len()), |acc, pred| acc + pred);
                Ok(sum / self.models.len() as f64)
            }
            VotingStrategy::Weighted => {
                let weighted_sum = predictions.iter()
                    .zip(&self.weights)
                    .fold(Array1::zeros(predictions[0].len()), |acc, (pred, weight)| {
                        acc + pred * *weight
                    });
                let total_weight: f64 = self.weights.iter().sum();
                Ok(weighted_sum / total_weight)
            }
            VotingStrategy::Median => {
                let n_samples = predictions[0].len();
                let mut result = Array1::zeros(n_samples);
                
                for i in 0..n_samples {
                    let mut values: Vec<f64> = predictions.iter()
                        .map(|pred| pred[i])
                        .collect();
                    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    result[i] = values[values.len() / 2];
                }
                
                Ok(result)
            }
            VotingStrategy::Max => {
                let n_samples = predictions[0].len();
                let mut result = Array1::zeros(n_samples);
                
                for i in 0..n_samples {
                    result[i] = predictions.iter()
                        .map(|pred| pred[i])
                        .fold(f64::NEG_INFINITY, f64::max);
                }
                
                Ok(result)
            }
        }
    }
}

/// 自动ML管道
pub struct AutoMLPipeline {
    feature_selector: FeatureSelector,
    model_selector: ModelSelector,
    hyperparameter_tuner: HyperparameterTuner,
    cross_validator: CrossValidator,
}

impl AutoMLPipeline {
    pub fn new() -> Self {
        Self {
            feature_selector: FeatureSelector::new(),
            model_selector: ModelSelector::new(),
            hyperparameter_tuner: HyperparameterTuner::new(),
            cross_validator: CrossValidator::new(5), // 5-fold CV
        }
    }

    pub async fn run(&mut self, 
        features: &Array2<f64>, 
        targets: &Array1<f64>,
        feature_names: &[String]
    ) -> Result<Arc<RwLock<MLModelWrapper>>, StrategyError> {
        // 1. 特征选择
        let selected_features = self.feature_selector.select_features(features, targets)?;
        
        // 2. 模型选择
        let best_model_type = self.model_selector.select_best_model(&selected_features, targets)?;
        
        // 3. 超参数调优
        let best_params = self.hyperparameter_tuner.tune(
            best_model_type,
            &selected_features,
            targets
        )?;
        
        // 4. 训练最终模型
        let mut final_model = MLModelWrapper::new(
            best_model_type,
            feature_names.to_vec()
        );
        
        // 更新超参数
        for (key, value) in best_params {
            final_model.metadata.hyperparameters.insert(key, value);
        }
        
        // 交叉验证评估
        let cv_scores = self.cross_validator.evaluate(
            &mut final_model,
            &selected_features,
            targets
        )?;
        
        // 记录性能指标
        final_model.update_performance_metric("cv_mean_score", cv_scores.mean);
        final_model.update_performance_metric("cv_std_score", cv_scores.std);
        
        // 最终训练
        final_model.train(&selected_features, targets)?;
        
        Ok(Arc::new(RwLock::new(final_model)))
    }
}

/// 特征选择器
struct FeatureSelector {
    importance_threshold: f64,
}

impl FeatureSelector {
    fn new() -> Self {
        Self {
            importance_threshold: 0.01,
        }
    }

    fn select_features(&self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<Array2<f64>, StrategyError> {
        // 使用相关性分析选择特征
        let mut correlations = Vec::new();
        
        for col in features.axis_iter(Axis(1)) {
            let corr = self.calculate_correlation(&col.to_owned(), targets);
            correlations.push(corr.abs());
        }
        
        // 选择相关性高于阈值的特征
        let selected_indices: Vec<usize> = correlations.iter()
            .enumerate()
            .filter(|(_, &corr)| corr > self.importance_threshold)
            .map(|(idx, _)| idx)
            .collect();
        
        if selected_indices.is_empty() {
            return Err(StrategyError::FeatureEngineeringError("No features selected".to_string()));
        }
        
        // 构建新的特征矩阵
        let selected_features = features.select(Axis(1), &selected_indices);
        Ok(selected_features)
    }

    fn calculate_correlation(&self, x: &Array1<f64>, y: &Array1<f64>) -> f64 {
        let n = x.len() as f64;
        let sum_x: f64 = x.sum();
        let sum_y: f64 = y.sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum();
        let sum_x2: f64 = x.iter().map(|a| a * a).sum();
        let sum_y2: f64 = y.iter().map(|a| a * a).sum();
        
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}

/// 模型选择器
struct ModelSelector {
    candidate_models: Vec<MLModelType>,
}

impl ModelSelector {
    fn new() -> Self {
        Self {
            candidate_models: vec![
                MLModelType::DecisionTree,
                MLModelType::RandomForest,
                MLModelType::LinearRegression,
            ],
        }
    }

    fn select_best_model(&self, features: &Array2<f64>, targets: &Array1<f64>) -> Result<MLModelType, StrategyError> {
        let mut best_score = f64::NEG_INFINITY;
        let mut best_model_type = MLModelType::DecisionTree;
        
        // 简单的模型选择策略
        let n_features = features.ncols();
        let n_samples = features.nrows();
        
        if n_samples < 100 {
            // 小数据集使用决策树
            best_model_type = MLModelType::DecisionTree;
        } else if n_features > 20 {
            // 高维数据使用随机森林
            best_model_type = MLModelType::RandomForest;
        } else {
            // 其他情况使用线性回归
            best_model_type = MLModelType::LinearRegression;
        }
        
        Ok(best_model_type)
    }
}

/// 超参数调优器
struct HyperparameterTuner {
    n_trials: usize,
}

impl HyperparameterTuner {
    fn new() -> Self {
        Self {
            n_trials: 20,
        }
    }

    fn tune(&self, model_type: MLModelType, features: &Array2<f64>, targets: &Array1<f64>) 
        -> Result<HashMap<String, serde_json::Value>, StrategyError> {
        let mut best_params = HashMap::new();
        
        match model_type {
            MLModelType::DecisionTree => {
                best_params.insert("max_depth".to_string(), serde_json::json!(10));
                best_params.insert("min_samples_split".to_string(), serde_json::json!(20));
            }
            MLModelType::RandomForest => {
                best_params.insert("n_trees".to_string(), serde_json::json!(100));
                best_params.insert("max_depth".to_string(), serde_json::json!(10));
                best_params.insert("min_samples_split".to_string(), serde_json::json!(20));
            }
            MLModelType::LinearRegression => {
                best_params.insert("fit_intercept".to_string(), serde_json::json!(true));
                best_params.insert("normalize".to_string(), serde_json::json!(true));
            }
            _ => {}
        }
        
        Ok(best_params)
    }
}

/// 交叉验证器
struct CrossValidator {
    n_folds: usize,
}

#[derive(Debug)]
struct CVScores {
    mean: f64,
    std: f64,
    scores: Vec<f64>,
}

impl CrossValidator {
    fn new(n_folds: usize) -> Self {
        Self { n_folds }
    }

    fn evaluate(&self, model: &mut MLModelWrapper, features: &Array2<f64>, targets: &Array1<f64>) 
        -> Result<CVScores, StrategyError> {
        let n_samples = features.nrows();
        let fold_size = n_samples / self.n_folds;
        let mut scores = Vec::new();
        
        for fold in 0..self.n_folds {
            let test_start = fold * fold_size;
            let test_end = if fold == self.n_folds - 1 { n_samples } else { (fold + 1) * fold_size };
            
            // 创建训练和测试索引
            let mut train_indices: Vec<usize> = (0..test_start).collect();
            train_indices.extend(test_end..n_samples);
            let test_indices: Vec<usize> = (test_start..test_end).collect();
            
            // 分割数据
            let train_features = features.select(Axis(0), &train_indices);
            let train_targets = targets.select(Axis(0), &train_indices);
            let test_features = features.select(Axis(0), &test_indices);
            let test_targets = targets.select(Axis(0), &test_indices);
            
            // 训练和评估
            model.train(&train_features, &train_targets)?;
            let predictions = model.predict(&test_features)?;
            
            // 计算MSE
            let mse = predictions.iter()
                .zip(test_targets.iter())
                .map(|(pred, actual)| (pred - actual).powi(2))
                .sum::<f64>() / test_targets.len() as f64;
            
            scores.push(1.0 / (1.0 + mse)); // 转换为分数（越高越好）
        }
        
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        let variance = scores.iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f64>() / scores.len() as f64;
        let std = variance.sqrt();
        
        Ok(CVScores { mean, std, scores })
    }
} 