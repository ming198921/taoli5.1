use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tokio::fs;
use crate::strategy::core::StrategyError;

/// 策略模块完整配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyModuleConfig {
    pub system: SystemConfig,
    pub adaptive_profit: AdaptiveProfitConfig,
    pub opportunity_pool: OpportunityPoolConfig,
    pub scheduling: SchedulingConfig,
    pub risk_management: RiskManagementConfig,
    pub execution: ExecutionConfig,
    pub monitoring: MonitoringConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveProfitConfig {
    pub thresholds: ThresholdConfig,
    pub ml: MLConfig,
    pub hyperparameters: HyperparameterConfig,
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub normal_min: f64,
    pub normal_max: f64,
    pub cautious_min: f64,
    pub cautious_max: f64,
    pub extreme_min: f64,
    pub extreme_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    pub primary_model: String,
    pub ensemble_models: Vec<String>,
    pub min_training_samples: usize,
    pub retrain_interval_hours: u64,
    pub cv_folds: usize,
    pub early_stopping_patience: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperparameterConfig {
    pub max_depth: usize,
    pub n_estimators: usize,
    pub learning_rate: f64,
    pub regularization: f64,
    pub min_samples_split: usize,
    pub min_samples_leaf: usize,
    pub max_features: f64,
    pub random_state: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    pub technical_indicator_windows: Vec<usize>,
    pub lag_features: usize,
    pub rolling_stats_windows: Vec<usize>,
    pub difference_orders: Vec<usize>,
    pub include_interaction_features: bool,
    pub enable_feature_selection: bool,
    pub feature_selection_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityPoolConfig {
    pub max_opportunities: usize,
    pub expiry_seconds: i64,
    pub auto_cleanup_interval: u64,
    pub priority_weights: PriorityWeightConfig,
    pub evaluation_criteria: EvaluationCriteriaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityWeightConfig {
    pub profit_weight: f64,
    pub liquidity_weight: f64,
    pub risk_weight: f64,
    pub execution_speed_weight: f64,
    pub confidence_weight: f64,
    pub strategy_priority_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationCriteriaConfig {
    pub min_profit_threshold: f64,
    pub min_liquidity_score: f64,
    pub max_risk_score: f64,
    pub max_execution_delay_ms: u64,
    pub min_confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConfig {
    pub strategy_weights: HashMap<String, f64>,
    pub api_limits: HashMap<String, u32>,
    pub resources: ResourceConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub max_concurrent_executions: usize,
    pub scheduling_interval_ms: u64,
    pub latency_penalty_factor: f64,
    pub fairness_factor: f64,
    pub resource_reservation_ratio: f64,
    pub max_memory_usage_mb: usize,
    pub max_cpu_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_rate_threshold: f64,
    pub minimum_requests: usize,
    pub circuit_break_duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_daily_trades: u32,
    pub max_single_trade_usd: f64,
    pub max_total_exposure_usd: f64,
    pub max_consecutive_losses: u32,
    pub enable_dynamic_risk: bool,
    pub risk_score_threshold: f64,
    pub volatility_threshold: f64,
    pub correlation_threshold: f64,
    pub price_deviation_threshold: f64,
    pub volume_spike_threshold: f64,
    pub enable_ai_anomaly_detection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub mode: String,
    pub enable_real_trading: bool,
    pub dry_run_mode: bool,
    pub exchanges: ExchangeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub enabled_exchanges: Vec<String>,
    pub max_slippage_bps: u32,
    pub order_timeout_seconds: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_performance_tracking: bool,
    pub stats_collection_interval_seconds: u64,
    pub max_history_records: usize,
    pub enable_ai_monitoring: bool,
    pub prediction_cache_ttl_seconds: u64,
    pub model_performance_check_interval_hours: u64,
    pub drift_detection_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_level: String,
    pub log_predictions: bool,
    pub log_execution_details: bool,
    pub log_feature_importance: bool,
}

/// 配置加载器
pub struct StrategyConfigLoader;

impl StrategyConfigLoader {
    /// 从文件加载配置
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<StrategyModuleConfig, StrategyError> {
        let content = fs::read_to_string(path).await
            .map_err(|e| StrategyError::ConfigurationError(format!("Failed to read config file: {}", e)))?;
        
        let config: StrategyModuleConfig = toml::from_str(&content)
            .map_err(|e| StrategyError::ConfigurationError(format!("Failed to parse config: {}", e)))?;
        
        Self::validate_config(&config)?;
        
        Ok(config)
    }
    
    /// 验证配置有效性
    fn validate_config(config: &StrategyModuleConfig) -> Result<(), StrategyError> {
        // 验证阈值配置
        let thresholds = &config.adaptive_profit.thresholds;
        if thresholds.normal_min >= thresholds.normal_max {
            return Err(StrategyError::ConfigurationError(
                "normal_min must be less than normal_max".to_string()
            ));
        }
        
        if thresholds.cautious_min >= thresholds.cautious_max {
            return Err(StrategyError::ConfigurationError(
                "cautious_min must be less than cautious_max".to_string()
            ));
        }
        
        if thresholds.extreme_min >= thresholds.extreme_max {
            return Err(StrategyError::ConfigurationError(
                "extreme_min must be less than extreme_max".to_string()
            ));
        }
        
        // 验证ML配置
        let ml_config = &config.adaptive_profit.ml;
        if ml_config.min_training_samples == 0 {
            return Err(StrategyError::ConfigurationError(
                "min_training_samples must be greater than 0".to_string()
            ));
        }
        
        if ml_config.cv_folds < 2 {
            return Err(StrategyError::ConfigurationError(
                "cv_folds must be at least 2".to_string()
            ));
        }
        
        // 验证超参数
        let hyperparams = &config.adaptive_profit.hyperparameters;
        if hyperparams.learning_rate <= 0.0 || hyperparams.learning_rate > 1.0 {
            return Err(StrategyError::ConfigurationError(
                "learning_rate must be between 0 and 1".to_string()
            ));
        }
        
        if hyperparams.max_features <= 0.0 || hyperparams.max_features > 1.0 {
            return Err(StrategyError::ConfigurationError(
                "max_features must be between 0 and 1".to_string()
            ));
        }
        
        // 验证权重配置
        let weights = &config.opportunity_pool.priority_weights;
        let total_weight = weights.profit_weight + weights.liquidity_weight + 
                          weights.risk_weight + weights.execution_speed_weight + 
                          weights.confidence_weight + weights.strategy_priority_weight;
        
        if (total_weight - 1.0).abs() > 0.01 {
            return Err(StrategyError::ConfigurationError(
                format!("Priority weights must sum to 1.0, got {:.3}", total_weight)
            ));
        }
        
        // 验证风控配置
        let risk_config = &config.risk_management;
        if risk_config.max_single_trade_usd > risk_config.max_total_exposure_usd {
            return Err(StrategyError::ConfigurationError(
                "max_single_trade_usd cannot exceed max_total_exposure_usd".to_string()
            ));
        }
        
        // 验证执行配置
        if !["simulation", "production"].contains(&config.execution.mode.as_str()) {
            return Err(StrategyError::ConfigurationError(
                "execution.mode must be 'simulation' or 'production'".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// 获取默认配置
    pub fn default_config() -> StrategyModuleConfig {
        StrategyModuleConfig {
            system: SystemConfig {
                name: "celue-strategy-module".to_string(),
                version: "1.0.0".to_string(),
                environment: "development".to_string(),
            },
            adaptive_profit: AdaptiveProfitConfig {
                thresholds: ThresholdConfig {
                    normal_min: 0.005,
                    normal_max: 0.008,
                    cautious_min: 0.012,
                    cautious_max: 0.018,
                    extreme_min: 0.020,
                    extreme_max: 0.035,
                },
                ml: MLConfig {
                    primary_model: "random_forest".to_string(),
                    ensemble_models: vec![
                        "decision_tree".to_string(),
                        "random_forest".to_string(),
                        "linear_regression".to_string(),
                    ],
                    min_training_samples: 100,
                    retrain_interval_hours: 24,
                    cv_folds: 5,
                    early_stopping_patience: 10,
                },
                hyperparameters: HyperparameterConfig {
                    max_depth: 10,
                    n_estimators: 100,
                    learning_rate: 0.01,
                    regularization: 0.1,
                    min_samples_split: 2,
                    min_samples_leaf: 1,
                    max_features: 0.8,
                    random_state: 42,
                },
                features: FeatureConfig {
                    technical_indicator_windows: vec![5, 10, 20, 50],
                    lag_features: 10,
                    rolling_stats_windows: vec![5, 10, 20],
                    difference_orders: vec![1, 2],
                    include_interaction_features: true,
                    enable_feature_selection: true,
                    feature_selection_threshold: 0.01,
                },
            },
            opportunity_pool: OpportunityPoolConfig {
                max_opportunities: 1000,
                expiry_seconds: 30,
                auto_cleanup_interval: 5,
                priority_weights: PriorityWeightConfig {
                    profit_weight: 0.3,
                    liquidity_weight: 0.25,
                    risk_weight: 0.2,
                    execution_speed_weight: 0.1,
                    confidence_weight: 0.1,
                    strategy_priority_weight: 0.05,
                },
                evaluation_criteria: EvaluationCriteriaConfig {
                    min_profit_threshold: 0.001,
                    min_liquidity_score: 0.5,
                    max_risk_score: 0.7,
                    max_execution_delay_ms: 1000,
                    min_confidence_score: 0.6,
                },
            },
            scheduling: SchedulingConfig {
                strategy_weights: [
                    ("inter_exchange".to_string(), 1.0),
                    ("triangular".to_string(), 0.8),
                    ("statistical".to_string(), 0.6),
                    ("cross_pair".to_string(), 0.4),
                ].iter().cloned().collect(),
                api_limits: [
                    ("binance".to_string(), 1200),
                    ("okx".to_string(), 600),
                    ("huobi".to_string(), 800),
                    ("bybit".to_string(), 500),
                    ("gateio".to_string(), 400),
                ].iter().cloned().collect(),
                resources: ResourceConfig {
                    max_concurrent_executions: 10,
                    scheduling_interval_ms: 100,
                    latency_penalty_factor: 0.1,
                    fairness_factor: 0.2,
                    resource_reservation_ratio: 0.8,
                    max_memory_usage_mb: 2048,
                    max_cpu_usage_percent: 80.0,
                },
                circuit_breaker: CircuitBreakerConfig {
                    failure_rate_threshold: 0.5,
                    minimum_requests: 10,
                    circuit_break_duration_seconds: 60,
                },
            },
            risk_management: RiskManagementConfig {
                max_daily_trades: 1000,
                max_single_trade_usd: 50000.0,
                max_total_exposure_usd: 500000.0,
                max_consecutive_losses: 5,
                enable_dynamic_risk: true,
                risk_score_threshold: 0.7,
                volatility_threshold: 0.1,
                correlation_threshold: 0.8,
                price_deviation_threshold: 0.15,
                volume_spike_threshold: 5.0,
                enable_ai_anomaly_detection: true,
            },
            execution: ExecutionConfig {
                mode: "simulation".to_string(),
                enable_real_trading: false,
                dry_run_mode: true,
                exchanges: ExchangeConfig {
                    enabled_exchanges: vec![
                        "binance".to_string(),
                        "okx".to_string(),
                        "bybit".to_string(),
                    ],
                    max_slippage_bps: 10,
                    order_timeout_seconds: 30,
                    retry_attempts: 3,
                },
            },
            monitoring: MonitoringConfig {
                enable_performance_tracking: true,
                stats_collection_interval_seconds: 10,
                max_history_records: 10000,
                enable_ai_monitoring: true,
                prediction_cache_ttl_seconds: 300,
                model_performance_check_interval_hours: 1,
                drift_detection_threshold: 0.2,
            },
            logging: LoggingConfig {
                log_level: "info".to_string(),
                log_predictions: true,
                log_execution_details: false,
                log_feature_importance: false,
            },
        }
    }
    
    /// 将配置保存到文件
    pub async fn save_to_file<P: AsRef<Path>>(
        config: &StrategyModuleConfig, 
        path: P
    ) -> Result<(), StrategyError> {
        let content = toml::to_string_pretty(config)
            .map_err(|e| StrategyError::ConfigurationError(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(path, content).await
            .map_err(|e| StrategyError::ConfigurationError(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// 热重载配置
    pub async fn reload_config<P: AsRef<Path>>(
        path: P,
        current_config: &mut StrategyModuleConfig
    ) -> Result<bool, StrategyError> {
        let new_config = Self::load_from_file(path).await?;
        
        // 检查是否有变化
        let config_changed = !self::configs_equal(current_config, &new_config);
        
        if config_changed {
            *current_config = new_config;
            tracing::info!("Strategy configuration reloaded successfully");
        }
        
        Ok(config_changed)
    }
}

/// 比较两个配置是否相等（简化版本）
fn configs_equal(a: &StrategyModuleConfig, b: &StrategyModuleConfig) -> bool {
    // 这里可以实现更精确的配置比较逻辑
    // 为简化起见，使用序列化后的字符串比较
    toml::to_string(a).unwrap_or_default() == toml::to_string(b).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[tokio::test]
    async fn test_config_load_save() {
        let config = StrategyConfigLoader::default_config();
        
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        
        // 保存配置
        StrategyConfigLoader::save_to_file(&config, temp_path).await.unwrap();
        
        // 加载配置
        let loaded_config = StrategyConfigLoader::load_from_file(temp_path).await.unwrap();
        
        // 验证配置一致性
        assert_eq!(config.system.name, loaded_config.system.name);
        assert_eq!(config.adaptive_profit.thresholds.normal_min, loaded_config.adaptive_profit.thresholds.normal_min);
    }
    
    #[tokio::test]
    async fn test_config_validation() {
        let mut config = StrategyConfigLoader::default_config();
        
        // 测试无效阈值
        config.adaptive_profit.thresholds.normal_min = 0.1;
        config.adaptive_profit.thresholds.normal_max = 0.05;
        
        let result = StrategyConfigLoader::validate_config(&config);
        assert!(result.is_err());
    }
} 