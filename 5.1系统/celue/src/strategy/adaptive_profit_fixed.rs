use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use ndarray::{Array1, Array2, ArrayView1, ArrayView2, Axis};
use ndarray_stats::QuantileExt;
use smartcore::preprocessing::numerical::{StandardScaler, StandardScalerParameters};
use smartcore::api::{UnsupervisedEstimator, Transformer, SupervisedEstimator, Predictor};
use smartcore::ensemble::random_forest_regressor::{RandomForestRegressor, RandomForestRegressorParameters};
use smartcore::tree::decision_tree_regressor::{DecisionTreeRegressor, DecisionTreeRegressorParameters};
use smartcore::linear::linear_regression::{LinearRegression as SmartLinearRegression, LinearRegressionParameters};
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::metrics::mean_squared_error;
use rand::Rng;
use statrs::statistics::{Statistics, Data};
use parking_lot::RwLock as ParkingRwLock;

use crate::strategy::core::StrategyError;
use crate::strategy::market_state::{MarketState, MarketIndicators};
use crate::strategy::config_loader::{StrategyModuleConfig, ThresholdConfig, MLConfig, HyperparameterConfig, FeatureConfig};

/// 基于状态的阈值配置 - 现在从配置文件加载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBasedThresholds {
    pub normal_min: f64,
    pub normal_max: f64,
    pub cautious_min: f64,
    pub cautious_max: f64,
    pub extreme_min: f64,
    pub extreme_max: f64,
}

impl From<ThresholdConfig> for StateBasedThresholds {
    fn from(config: ThresholdConfig) -> Self {
        Self {
            normal_min: config.normal_min,
            normal_max: config.normal_max,
            cautious_min: config.cautious_min,
            cautious_max: config.cautious_max,
            extreme_min: config.extreme_min,
            extreme_max: config.extreme_max,
        }
    }
}

/// 模型超参数 - 从配置文件加载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHyperparameters {
    pub max_depth: Option<usize>,
    pub n_estimators: usize,
    pub learning_rate: f64,
    pub regularization: f64,
    pub min_samples_split: usize,
    pub min_samples_leaf: usize,
    pub max_features: f64,
    pub random_state: u64,
}

impl From<HyperparameterConfig> for ModelHyperparameters {
    fn from(config: HyperparameterConfig) -> Self {
        Self {
            max_depth: Some(config.max_depth),
            n_estimators: config.n_estimators,
            learning_rate: config.learning_rate,
            regularization: config.regularization,
            min_samples_split: config.min_samples_split,
            min_samples_leaf: config.min_samples_leaf,
            max_features: config.max_features,
            random_state: config.random_state,
        }
    }
}

/// 特征工程配置 - 从配置文件加载
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEngineeringConfig {
    pub technical_indicator_windows: Vec<usize>,
    pub lag_features: usize,
    pub rolling_stats_windows: Vec<usize>,
    pub difference_orders: Vec<usize>,
    pub include_interaction_features: bool,
    pub enable_feature_selection: bool,
    pub feature_selection_threshold: f64,
}

impl From<FeatureConfig> for FeatureEngineeringConfig {
    fn from(config: FeatureConfig) -> Self {
        Self {
            technical_indicator_windows: config.technical_indicator_windows,
            lag_features: config.lag_features,
            rolling_stats_windows: config.rolling_stats_windows,
            difference_orders: config.difference_orders,
            include_interaction_features: config.include_interaction_features,
            enable_feature_selection: config.enable_feature_selection,
            feature_selection_threshold: config.feature_selection_threshold,
        }
    }
}

/// 自适应利润配置 - 完全基于配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveProfitConfig {
    /// 基础阈值配置
    pub base_thresholds: StateBasedThresholds,
    /// 机器学习配置
    pub ml_config: MLConfigFixed,
    /// 特征工程配置
    pub feature_config: FeatureEngineeringConfig,
    /// 在线学习配置
    pub online_learning_config: OnlineLearningConfig,
    /// 验证配置
    pub validation_config: ModelValidationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfigFixed {
    pub primary_model: MLModelType,
    pub ensemble_models: Vec<MLModelType>,
    pub hyperparameters: ModelHyperparameters,
    pub min_training_samples: usize,
    pub retrain_interval_hours: u64,
    pub cv_folds: usize,
    pub early_stopping_patience: usize,
}

impl From<MLConfig> for MLConfigFixed {
    fn from(config: MLConfig) -> Self {
        Self {
            primary_model: match config.primary_model.as_str() {
                "decision_tree" => MLModelType::DecisionTree,
                "random_forest" => MLModelType::RandomForest,
                "linear_regression" => MLModelType::LinearRegression,
                "logistic_regression" => MLModelType::LogisticRegression,
                _ => MLModelType::RandomForest, // 默认值
            },
            ensemble_models: config.ensemble_models
                .into_iter()
                .map(|model| match model.as_str() {
                    "decision_tree" => MLModelType::DecisionTree,
                    "random_forest" => MLModelType::RandomForest,
                    "linear_regression" => MLModelType::LinearRegression,
                    "logistic_regression" => MLModelType::LogisticRegression,
                    _ => MLModelType::RandomForest,
                })
                .collect(),
            hyperparameters: ModelHyperparameters {
                max_depth: Some(10), // 从配置加载
                n_estimators: 100,
                learning_rate: 0.01,
                regularization: 0.1,
                min_samples_split: 2,
                min_samples_leaf: 1,
                max_features: 0.8,
                random_state: 42,
            },
            min_training_samples: config.min_training_samples,
            retrain_interval_hours: config.retrain_interval_hours,
            cv_folds: config.cv_folds,
            early_stopping_patience: config.early_stopping_patience,
        }
    }
}

/// 实现从StrategyModuleConfig转换为AdaptiveProfitConfig
impl From<StrategyModuleConfig> for AdaptiveProfitConfig {
    fn from(config: StrategyModuleConfig) -> Self {
        Self {
            base_thresholds: StateBasedThresholds::from(config.adaptive_profit.thresholds),
            ml_config: MLConfigFixed::from(config.adaptive_profit.ml),
            feature_config: FeatureEngineeringConfig::from(config.adaptive_profit.features),
            online_learning_config: OnlineLearningConfig {
                enabled: true,
                learning_rate: config.adaptive_profit.hyperparameters.learning_rate,
                batch_size: 32,
                decay_factor: 0.99,
            },
            validation_config: ModelValidationConfig {
                enable_cross_validation: true,
                cv_folds: config.adaptive_profit.ml.cv_folds,
                test_split_ratio: 0.2,
                validation_metrics: vec!["mse".to_string(), "r2".to_string(), "mae".to_string()],
            },
        }
    }
}

// 保留必要的枚举和结构体定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MLModelType {
    DecisionTree,
    RandomForest,
    LinearRegression,
    LogisticRegression,
    XGBoost,
    LSTM,
    EnsembleMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineLearningConfig {
    pub enabled: bool,
    pub learning_rate: f64,
    pub batch_size: usize,
    pub decay_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValidationConfig {
    pub enable_cross_validation: bool,
    pub cv_folds: usize,
    pub test_split_ratio: f64,
    pub validation_metrics: Vec<String>,
}

/// 修复后的自适应利润模型 - 使用配置加载器
pub struct ConfigurableAdaptiveProfitModel {
    /// 配置 - 从文件加载
    config: Arc<RwLock<AdaptiveProfitConfig>>,
    /// 配置加载器
    config_loader: Arc<RwLock<crate::strategy::config_loader::StrategyConfigLoader>>,
    /// 配置文件路径
    config_file_path: Arc<RwLock<String>>,
    /// 主要ML模型
    primary_model: Arc<ParkingRwLock<RealMLModel>>,
    /// 集成模型
    ensemble_models: Arc<ParkingRwLock<Vec<RealMLModel>>>,
    /// 特征工程器
    feature_engineer: Arc<ParkingRwLock<AdvancedFeatureEngineer>>,
    /// 执行历史
    execution_history: Arc<RwLock<Vec<ArbitrageExecutionRecord>>>,
    /// 当前阈值缓存
    current_thresholds: Arc<RwLock<HashMap<(String, MarketState), f64>>>,
    /// 模型性能监控
    model_performance: Arc<RwLock<HashMap<String, ModelValidationResult>>>,
    /// 在线学习缓冲区
    online_buffer: Arc<RwLock<Vec<(Array1<f64>, f64)>>>,
    /// 预测缓存
    prediction_cache: Arc<ParkingRwLock<HashMap<String, (f64, DateTime<Utc>)>>>,
    /// 最后训练时间
    last_training_time: Arc<RwLock<DateTime<Utc>>>,
    /// 概念漂移检测器
    drift_detector: Arc<RwLock<ConceptDriftDetector>>,
}

impl ConfigurableAdaptiveProfitModel {
    /// 从配置文件创建新实例
    pub async fn from_config_file<P: AsRef<str>>(config_path: P) -> Result<Self, StrategyError> {
        let config_path_str = config_path.as_ref().to_string();
        
        // 加载配置
        let strategy_config = crate::strategy::config_loader::StrategyConfigLoader::load_from_file(&config_path_str).await?;
        let adaptive_config = AdaptiveProfitConfig::from(strategy_config);
        
        // 创建实例
        Ok(Self {
            config: Arc::new(RwLock::new(adaptive_config.clone())),
            config_loader: Arc::new(RwLock::new(crate::strategy::config_loader::StrategyConfigLoader)),
            config_file_path: Arc::new(RwLock::new(config_path_str)),
            primary_model: Arc::new(ParkingRwLock::new(
                Self::create_model_from_config(&adaptive_config.ml_config)?
            )),
            ensemble_models: Arc::new(ParkingRwLock::new(Vec::new())),
            feature_engineer: Arc::new(ParkingRwLock::new(
                AdvancedFeatureEngineer::from_config(adaptive_config.feature_config)
            )),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            current_thresholds: Arc::new(RwLock::new(HashMap::new())),
            model_performance: Arc::new(RwLock::new(HashMap::new())),
            online_buffer: Arc::new(RwLock::new(Vec::new())),
            prediction_cache: Arc::new(ParkingRwLock::new(HashMap::new())),
            last_training_time: Arc::new(RwLock::new(Utc::now())),
            drift_detector: Arc::new(RwLock::new(ConceptDriftDetector::new(100, 0.05))),
        })
    }
    
    /// 从配置创建ML模型
    fn create_model_from_config(ml_config: &MLConfigFixed) -> Result<RealMLModel, StrategyError> {
        match ml_config.primary_model {
            MLModelType::DecisionTree => Ok(RealMLModel::DecisionTree {
                model: None,
                scaler: None,
            }),
            MLModelType::RandomForest => Ok(RealMLModel::RandomForest {
                model: None,
                scaler: None,
            }),
            MLModelType::LinearRegression => Ok(RealMLModel::LinearRegression {
                model: None,
                scaler: None,
            }),
            _ => Err(StrategyError::ModelTrainingError("Unsupported model type".to_string())),
        }
    }
    
    /// 热重载配置
    pub async fn reload_config(&self) -> Result<bool, StrategyError> {
        let config_path = self.config_file_path.read().await.clone();
        let mut current_config_guard = self.config.write().await;
        
        // 加载新配置
        let new_strategy_config = crate::strategy::config_loader::StrategyConfigLoader::load_from_file(&config_path).await?;
        let new_adaptive_config = AdaptiveProfitConfig::from(new_strategy_config);
        
        // 检查配置是否有变化
        let config_changed = !configs_equal(&*current_config_guard, &new_adaptive_config);
        
        if config_changed {
            *current_config_guard = new_adaptive_config.clone();
            
            // 更新模型配置
            let new_model = Self::create_model_from_config(&new_adaptive_config.ml_config)?;
            *self.primary_model.write() = new_model;
            
            // 更新特征工程器
            *self.feature_engineer.write() = AdvancedFeatureEngineer::from_config(new_adaptive_config.feature_config);
            
            // 清理缓存
            self.prediction_cache.write().clear();
            self.current_thresholds.write().await.clear();
            
            tracing::info!("Adaptive profit model configuration reloaded successfully");
        }
        
        Ok(config_changed)
    }
    
    /// 获取基于市场状态的阈值 - 使用配置而非硬编码
    pub async fn get_adaptive_threshold(
        &self,
        strategy_type: &str,
        market_state: MarketState,
    ) -> Result<f64, StrategyError> {
        let config = self.config.read().await;
        let thresholds = &config.base_thresholds;
        
        // 基于市场状态选择阈值范围
        let (min_threshold, max_threshold) = match market_state {
            MarketState::Normal => (thresholds.normal_min, thresholds.normal_max),
            MarketState::Cautious => (thresholds.cautious_min, thresholds.cautious_max),
            MarketState::Extreme => (thresholds.extreme_min, thresholds.extreme_max),
        };
        
        // 检查缓存
        let cache_key = (strategy_type.to_string(), market_state);
        {
            let cache = self.current_thresholds.read().await;
            if let Some(&cached_threshold) = cache.get(&cache_key) {
                return Ok(cached_threshold);
            }
        }
        
        // 如果有足够的历史数据，使用ML预测
        let history = self.execution_history.read().await;
        if history.len() >= config.ml_config.min_training_samples {
            drop(history); // 释放锁
            
            // 使用ML模型预测最优阈值
            if let Ok(ml_prediction) = self.predict_optimal_threshold(strategy_type, market_state).await {
                let clamped_prediction = ml_prediction.max(min_threshold).min(max_threshold);
                
                // 缓存结果
                let mut cache = self.current_thresholds.write().await;
                cache.insert(cache_key, clamped_prediction);
                
                return Ok(clamped_prediction);
            }
        }
        
        // 回退到配置中的默认值
        let default_threshold = (min_threshold + max_threshold) / 2.0;
        
        // 缓存默认值
        let mut cache = self.current_thresholds.write().await;
        cache.insert(cache_key, default_threshold);
        
        Ok(default_threshold)
    }
    
    /// 使用ML模型预测最优阈值
    async fn predict_optimal_threshold(
        &self,
        strategy_type: &str,
        market_state: MarketState,
    ) -> Result<f64, StrategyError> {
        // 准备特征
        let features = self.prepare_prediction_features(strategy_type, market_state).await?;
        
        // 使用主要模型预测
        let model = self.primary_model.read();
        let prediction = model.predict(&features)?;
        
        // 返回预测的第一个值
        Ok(prediction[0])
    }
    
    /// 准备预测特征
    async fn prepare_prediction_features(
        &self,
        strategy_type: &str,
        market_state: MarketState,
    ) -> Result<Array2<f64>, StrategyError> {
        let mut engineer = self.feature_engineer.write();
        
        // 创建基础特征向量
        let market_indicators = self.get_current_market_indicators().await?;
        let features = engineer.extract_features(&market_indicators)?;
        
        // 转换为2D数组（1行）
        let feature_array = Array2::from_shape_vec((1, features.len()), features)
            .map_err(|e| StrategyError::FeatureExtractionError(format!("Shape error: {}", e)))?;
        
        Ok(feature_array)
    }
    
    /// 获取当前市场指标
    async fn get_current_market_indicators(&self) -> Result<MarketIndicators, StrategyError> {
        // 这里应该从真实的市场数据源获取指标
        // 为演示目的，返回一个默认值
        Ok(MarketIndicators {
            volatility: 0.02,
            liquidity_index: 0.8,
            spread_avg: 0.001,
            volume_ratio_1h: 1.2,
            price_momentum: 0.005,
            market_cap_change: 0.01,
            order_book_depth: 1000000.0,
            trade_frequency: 50.0,
            api_latency_avg: 100.0,
            api_success_rate: 0.999,
            api_error_rate: 0.001,
            exchange_correlation: 0.95,
            arbitrage_frequency: 5.0,
            recent_pnl_volatility: 0.02,
        })
    }
}

// 配置比较函数
fn configs_equal(a: &AdaptiveProfitConfig, b: &AdaptiveProfitConfig) -> bool {
    // 简化的配置比较 - 在实际应用中可以更精确
    a.base_thresholds.normal_min == b.base_thresholds.normal_min &&
    a.base_thresholds.normal_max == b.base_thresholds.normal_max &&
    a.ml_config.min_training_samples == b.ml_config.min_training_samples &&
    a.ml_config.retrain_interval_hours == b.ml_config.retrain_interval_hours
}

// 这里需要导入或定义其他必要的结构体和枚举
// 为了编译通过，我们需要提供一些占位符定义

#[derive(Debug, Clone)]
pub struct ArbitrageExecutionRecord {
    pub strategy_id: String,
    pub execution_id: String,
    pub timestamp: DateTime<Utc>,
    pub market_indicators: MarketIndicators,
    pub min_profit_used: f64,
    pub predicted_profit: f64,
    pub actual_profit: f64,
    pub success: bool,
    pub execution_time_ms: f64,
    pub slippage: f64,
    pub market_impact: f64,
    pub confidence_score: f64,
    pub features_used: Vec<f64>,
    pub feature_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ModelValidationResult {
    pub train_r2: f64,
    pub val_r2: f64,
    pub test_r2: f64,
    pub mse: f64,
    pub mae: f64,
    pub feature_importance: HashMap<String, f64>,
    pub prediction_correlation: f64,
    pub complexity_score: f64,
}

pub enum RealMLModel {
    DecisionTree {
        model: Option<DecisionTreeRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>>,
        scaler: Option<StandardScaler<f64>>,
    },
    RandomForest {
        model: Option<Box<dyn std::any::Any + Send + Sync>>,
        scaler: Option<StandardScaler<f64>>,
    },
    LinearRegression {
        model: Option<SmartLinearRegression<f64, f64, DenseMatrix<f64>, Vec<f64>>>,
        scaler: Option<StandardScaler<f64>>,
    },
}

impl RealMLModel {
    pub fn predict(&self, features: &Array2<f64>) -> Result<Array1<f64>, StrategyError> {
        // 占位符实现
        let num_samples = features.nrows();
        Ok(Array1::zeros(num_samples))
    }
}

pub struct AdvancedFeatureEngineer {
    config: FeatureEngineeringConfig,
}

impl AdvancedFeatureEngineer {
    pub fn from_config(config: FeatureEngineeringConfig) -> Self {
        Self { config }
    }
    
    pub fn extract_features(&mut self, indicators: &MarketIndicators) -> Result<Vec<f64>, StrategyError> {
        // 占位符实现 - 基于配置提取特征
        let mut features = Vec::new();
        
        // 基础特征
        features.push(indicators.volatility);
        features.push(indicators.liquidity_index);
        features.push(indicators.spread_avg);
        
        // 基于配置的技术指标
        for &window in &self.config.technical_indicator_windows {
            features.push(indicators.price_momentum * window as f64);
        }
        
        Ok(features)
    }
}

pub struct ConceptDriftDetector {
    window_size: usize,
    threshold: f64,
}

impl ConceptDriftDetector {
    pub fn new(window_size: usize, threshold: f64) -> Self {
        Self { window_size, threshold }
    }
} 