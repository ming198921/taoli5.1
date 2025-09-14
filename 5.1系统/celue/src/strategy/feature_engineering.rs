use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, Timelike, Datelike};
use ndarray::{Array1, Array2, ArrayView1, Axis};
use ndarray_stats::QuantileExt;
use rand::{thread_rng, Rng};

use crate::strategy::core::StrategyError;
use crate::strategy::market_state::MarketIndicators;

/// 特征工程配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEngineeringConfig {
    /// 技术指标配置
    pub technical_indicators: TechnicalIndicatorConfig,
    /// 时间特征配置
    pub temporal_features: TemporalFeatureConfig,
    /// 统计特征配置
    pub statistical_features: StatisticalFeatureConfig,
    /// 交叉特征配置
    pub interaction_features: InteractionFeatureConfig,
    /// 特征选择配置
    pub feature_selection: FeatureSelectionConfig,
    /// 特征变换配置
    pub feature_transformation: FeatureTransformationConfig,
}

/// 技术指标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicatorConfig {
    /// 移动平均窗口
    pub ma_windows: Vec<usize>,
    /// RSI周期
    pub rsi_periods: Vec<usize>,
    /// 布林带配置
    pub bollinger_bands: BollingerBandConfig,
    /// MACD配置
    pub macd_config: MACDConfig,
    /// 随机指标配置
    pub stochastic_config: StochasticConfig,
    /// 威廉指标配置
    pub williams_r_periods: Vec<usize>,
    /// 动量指标配置
    pub momentum_periods: Vec<usize>,
}

/// 布林带配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBandConfig {
    pub periods: Vec<usize>,
    pub std_multipliers: Vec<f64>,
}

/// MACD配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACDConfig {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

/// 随机指标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticConfig {
    pub k_period: usize,
    pub d_period: usize,
    pub smooth_k: usize,
}

/// 时间特征配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalFeatureConfig {
    /// 滞后特征数量
    pub lag_features: Vec<usize>,
    /// 滑动窗口统计
    pub rolling_windows: Vec<usize>,
    /// 差分阶数
    pub difference_orders: Vec<usize>,
    /// 季节性特征
    pub seasonal_features: bool,
    /// 循环特征
    pub cyclical_features: bool,
}

/// 统计特征配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalFeatureConfig {
    /// 统计窗口大小
    pub stat_windows: Vec<usize>,
    /// 包含的统计量
    pub statistics: Vec<StatisticType>,
    /// 分位数
    pub quantiles: Vec<f64>,
    /// 矩统计
    pub moments: Vec<usize>,
}

/// 统计类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatisticType {
    Mean,
    Std,
    Var,
    Skew,
    Kurt,
    Min,
    Max,
    Median,
    Range,
    IQR,
}

/// 交叉特征配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionFeatureConfig {
    /// 是否启用交叉特征
    pub enabled: bool,
    /// 最大交叉深度
    pub max_interaction_depth: usize,
    /// 特征组合策略
    pub combination_strategy: CombinationStrategy,
    /// 最大特征数量
    pub max_features: usize,
}

/// 组合策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombinationStrategy {
    All,
    TopK(usize),
    Correlation,
    MutualInformation,
}

/// 特征选择配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSelectionConfig {
    /// 特征选择方法
    pub methods: Vec<FeatureSelectionMethod>,
    /// 选择阈值
    pub selection_threshold: f64,
    /// 最大特征数
    pub max_features: Option<usize>,
    /// 最小特征数
    pub min_features: usize,
    /// 稳定性要求
    pub stability_threshold: f64,
}

/// 特征选择方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureSelectionMethod {
    /// 相关性选择
    Correlation,
    /// 方差选择
    VarianceThreshold(f64),
    /// 单变量选择
    UnivariateSelection,
    /// 递归特征消除
    RecursiveFeatureElimination,
    /// L1正则化
    L1Regularization,
    /// 互信息
    MutualInformation,
    /// 卡方检验
    ChiSquare,
}

/// 特征变换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureTransformationConfig {
    /// 标准化方法
    pub scaling_method: ScalingMethod,
    /// 是否应用PCA
    pub apply_pca: bool,
    /// PCA组件数
    pub pca_components: Option<usize>,
    /// 是否应用多项式特征
    pub polynomial_features: bool,
    /// 多项式度数
    pub polynomial_degree: usize,
    /// 是否应用对数变换
    pub log_transform: bool,
}

/// 缩放方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingMethod {
    StandardScaler,
    MinMaxScaler,
    RobustScaler,
    Normalizer,
    None,
}

impl Default for FeatureEngineeringConfig {
    fn default() -> Self {
        Self {
            technical_indicators: TechnicalIndicatorConfig {
                ma_windows: vec![5, 10, 20, 50],
                rsi_periods: vec![14, 21],
                bollinger_bands: BollingerBandConfig {
                    periods: vec![20],
                    std_multipliers: vec![2.0],
                },
                macd_config: MACDConfig {
                    fast_period: 12,
                    slow_period: 26,
                    signal_period: 9,
                },
                stochastic_config: StochasticConfig {
                    k_period: 14,
                    d_period: 3,
                    smooth_k: 3,
                },
                williams_r_periods: vec![14],
                momentum_periods: vec![10, 20],
            },
            temporal_features: TemporalFeatureConfig {
                lag_features: vec![1, 2, 3, 5, 10],
                rolling_windows: vec![5, 10, 20],
                difference_orders: vec![1, 2],
                seasonal_features: true,
                cyclical_features: true,
            },
            statistical_features: StatisticalFeatureConfig {
                stat_windows: vec![10, 20, 50],
                statistics: vec![
                    StatisticType::Mean,
                    StatisticType::Std,
                    StatisticType::Skew,
                    StatisticType::Kurt,
                ],
                quantiles: vec![0.25, 0.5, 0.75],
                moments: vec![3, 4],
            },
            interaction_features: InteractionFeatureConfig {
                enabled: true,
                max_interaction_depth: 2,
                combination_strategy: CombinationStrategy::TopK(10),
                max_features: 50,
            },
            feature_selection: FeatureSelectionConfig {
                methods: vec![
                    FeatureSelectionMethod::Correlation,
                    FeatureSelectionMethod::VarianceThreshold(0.01),
                ],
                selection_threshold: 0.01,
                max_features: Some(100),
                min_features: 10,
                stability_threshold: 0.8,
            },
            feature_transformation: FeatureTransformationConfig {
                scaling_method: ScalingMethod::StandardScaler,
                apply_pca: false,
                pca_components: None,
                polynomial_features: false,
                polynomial_degree: 2,
                log_transform: false,
            },
        }
    }
}

/// 特征重要性记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureImportance {
    pub feature_name: String,
    pub importance_score: f64,
    pub selection_frequency: f64,
    pub stability_score: f64,
    pub correlation_with_target: f64,
}

/// 特征统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStatistics {
    pub feature_name: String,
    pub data_type: String,
    pub missing_ratio: f64,
    pub unique_values: usize,
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub outlier_ratio: f64,
}

/// 高级特征工程器
pub struct AdvancedFeatureEngineer {
    config: Arc<RwLock<FeatureEngineeringConfig>>,
    feature_names: Arc<RwLock<Vec<String>>>,
    feature_importance: Arc<RwLock<HashMap<String, FeatureImportance>>>,
    feature_statistics: Arc<RwLock<HashMap<String, FeatureStatistics>>>,
    selected_features: Arc<RwLock<Vec<usize>>>,
    transformation_params: Arc<RwLock<TransformationParams>>,
}

/// 变换参数
#[derive(Debug, Clone)]
pub struct TransformationParams {
    pub scaling_params: Option<ScalingParams>,
    pub pca_params: Option<PCAParams>,
    pub polynomial_params: Option<PolynomialParams>,
}

/// 缩放参数
#[derive(Debug, Clone)]
pub struct ScalingParams {
    pub mean: Vec<f64>,
    pub std: Vec<f64>,
    pub min: Vec<f64>,
    pub max: Vec<f64>,
}

/// PCA参数
#[derive(Debug, Clone)]
pub struct PCAParams {
    pub components: Array2<f64>,
    pub explained_variance: Vec<f64>,
    pub mean: Vec<f64>,
}

/// 多项式参数
#[derive(Debug, Clone)]
pub struct PolynomialParams {
    pub degree: usize,
    pub feature_combinations: Vec<Vec<usize>>,
}

impl AdvancedFeatureEngineer {
    pub fn new(config: FeatureEngineeringConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            feature_names: Arc::new(RwLock::new(Vec::new())),
            feature_importance: Arc::new(RwLock::new(HashMap::new())),
            feature_statistics: Arc::new(RwLock::new(HashMap::new())),
            selected_features: Arc::new(RwLock::new(Vec::new())),
            transformation_params: Arc::new(RwLock::new(TransformationParams {
                scaling_params: None,
                pca_params: None,
                polynomial_params: None,
            })),
        }
    }
    
    /// 构建完整特征集
    pub async fn engineer_features(
        &self,
        market_indicators: &[MarketIndicators],
        price_history: &[f64],
        volume_history: &[f64],
        returns_history: &[f64],
        timestamp_history: &[DateTime<Utc>],
    ) -> Result<Array2<f64>, StrategyError> {
        let config = self.config.read().await;
        let mut all_features = Vec::new();
        let mut feature_names = Vec::new();
        
        // 1. 基础市场指标特征
        if let Some(latest) = market_indicators.last() {
            let basic_features = self.extract_basic_features(latest);
            all_features.extend(basic_features.0);
            feature_names.extend(basic_features.1);
        }
        
        // 2. 技术指标特征
        let tech_features = self.create_technical_indicators(
            price_history,
            volume_history,
            &config.technical_indicators,
        )?;
        all_features.extend(tech_features.0);
        feature_names.extend(tech_features.1);
        
        // 3. 时间特征
        let temporal_features = self.create_temporal_features(
            price_history,
            returns_history,
            timestamp_history,
            &config.temporal_features,
        )?;
        all_features.extend(temporal_features.0);
        feature_names.extend(temporal_features.1);
        
        // 4. 统计特征
        let stat_features = self.create_statistical_features(
            price_history,
            volume_history,
            returns_history,
            &config.statistical_features,
        )?;
        all_features.extend(stat_features.0);
        feature_names.extend(stat_features.1);
        
        // 5. 市场微观结构特征
        let microstructure_features = self.create_microstructure_features(market_indicators)?;
        all_features.extend(microstructure_features.0);
        feature_names.extend(microstructure_features.1);
        
        // 更新特征名称
        *self.feature_names.write().await = feature_names;
        
        if all_features.is_empty() {
            return Err(StrategyError::FeatureEngineeringError("No features generated".to_string()));
        }
        
        // 转换为矩阵
        let feature_matrix = Array2::from_shape_vec((1, all_features.len()), all_features)
            .map_err(|e| StrategyError::FeatureEngineeringError(format!("Matrix creation failed: {:?}", e)))?;
        
        Ok(feature_matrix)
    }
    
    /// 提取基础特征
    fn extract_basic_features(&self, indicators: &MarketIndicators) -> (Vec<f64>, Vec<String>) {
        let mut features = Vec::new();
        let mut names = Vec::new();
        
        // 基础市场指标
        features.extend_from_slice(&[
            indicators.volatility_1h,
            indicators.volatility_4h,
            indicators.volatility_24h,
            indicators.liquidity_index,
            indicators.bid_ask_spread,
            indicators.order_book_depth,
            indicators.volume_ratio_1h,
            indicators.volume_ratio_4h,
            indicators.max_price_change_1m,
            indicators.max_price_change_5m,
            indicators.average_slippage,
            indicators.api_latency_avg,
            indicators.api_error_rate,
            indicators.api_success_rate,
            indicators.external_event_risk,
            indicators.news_sentiment_score,
        ]);
        
        names.extend_from_slice(&[
            "volatility_1h", "volatility_4h", "volatility_24h",
            "liquidity_index", "bid_ask_spread", "order_book_depth",
            "volume_ratio_1h", "volume_ratio_4h",
            "max_price_change_1m", "max_price_change_5m",
            "average_slippage", "api_latency_avg", "api_error_rate",
            "api_success_rate", "external_event_risk", "news_sentiment_score"
        ].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        
        (features, names)
    }
    
    /// 创建技术指标特征
    fn create_technical_indicators(
        &self,
        prices: &[f64],
        volumes: &[f64],
        config: &TechnicalIndicatorConfig,
    ) -> Result<(Vec<f64>, Vec<String>), StrategyError> {
        let mut features = Vec::new();
        let mut names = Vec::new();
        
        // 移动平均
        for &window in &config.ma_windows {
            if prices.len() >= window {
                let ma = self.simple_moving_average(&prices, window);
                features.push(ma);
                names.push(format!("ma_{}", window));
                
                // 价格相对于移动平均的位置
                if let Some(&current_price) = prices.last() {
                    let relative_position = (current_price - ma) / ma;
                    features.push(relative_position);
                    names.push(format!("price_ma_ratio_{}", window));
                }
            }
        }
        
        // RSI
        for &period in &config.rsi_periods {
            if prices.len() > period {
                let rsi = self.calculate_rsi(&prices, period);
                features.push(rsi);
                names.push(format!("rsi_{}", period));
            }
        }
        
        // 布林带
        for &period in &config.bollinger_bands.periods {
            for &std_mult in &config.bollinger_bands.std_multipliers {
                if prices.len() >= period {
                    let (upper, lower, width) = self.calculate_bollinger_bands(&prices, period, std_mult);
                    features.extend_from_slice(&[upper, lower, width]);
                    names.extend_from_slice(&[
                        format!("bb_upper_{}_{}", period, std_mult),
                        format!("bb_lower_{}_{}", period, std_mult),
                        format!("bb_width_{}_{}", period, std_mult),
                    ]);
                    
                    // 布林带位置
                    if let Some(&current_price) = prices.last() {
                        let bb_position = if width > 0.0 {
                            (current_price - lower) / width
                        } else {
                            0.5
                        };
                        features.push(bb_position);
                        names.push(format!("bb_position_{}_{}", period, std_mult));
                    }
                }
            }
        }
        
        // MACD
        if prices.len() > config.macd_config.slow_period {
            let (macd, signal, histogram) = self.calculate_macd(
                &prices,
                config.macd_config.fast_period,
                config.macd_config.slow_period,
                config.macd_config.signal_period,
            );
            features.extend_from_slice(&[macd, signal, histogram]);
            names.extend_from_slice(&[
                "macd".to_string(),
                "macd_signal".to_string(),
                "macd_histogram".to_string(),
            ]);
        }
        
        // 随机指标
        if prices.len() > config.stochastic_config.k_period {
            let (k, d) = self.calculate_stochastic(
                &prices,
                config.stochastic_config.k_period,
                config.stochastic_config.d_period,
            );
            features.extend_from_slice(&[k, d]);
            names.extend_from_slice(&["stoch_k".to_string(), "stoch_d".to_string()]);
        }
        
        // 威廉指标
        for &period in &config.williams_r_periods {
            if prices.len() >= period {
                let williams_r = self.calculate_williams_r(&prices, period);
                features.push(williams_r);
                names.push(format!("williams_r_{}", period));
            }
        }
        
        // 动量指标
        for &period in &config.momentum_periods {
            if prices.len() > period {
                let momentum = self.calculate_momentum(&prices, period);
                features.push(momentum);
                names.push(format!("momentum_{}", period));
                
                // 动量变化率
                if prices.len() > period + 1 {
                    let prev_momentum = self.calculate_momentum(&prices[..prices.len()-1], period);
                    let momentum_roc = (momentum - prev_momentum) / prev_momentum.abs().max(1e-8);
                    features.push(momentum_roc);
                    names.push(format!("momentum_roc_{}", period));
                }
            }
        }
        
        // 成交量技术指标
        if !volumes.is_empty() {
            // 成交量移动平均
            for &window in &config.ma_windows {
                if volumes.len() >= window {
                    let volume_ma = self.simple_moving_average(&volumes, window);
                    features.push(volume_ma);
                    names.push(format!("volume_ma_{}", window));
                    
                    // 当前成交量相对于移动平均
                    if let Some(&current_volume) = volumes.last() {
                        let volume_ratio = current_volume / volume_ma.max(1e-8);
                        features.push(volume_ratio);
                        names.push(format!("volume_ratio_{}", window));
                    }
                }
            }
            
            // OBV (On-Balance Volume)
            let obv = self.calculate_obv(&prices, &volumes);
            features.push(obv);
            names.push("obv".to_string());
        }
        
        Ok((features, names))
    }
    
    /// 创建时间特征
    fn create_temporal_features(
        &self,
        prices: &[f64],
        returns: &[f64],
        timestamps: &[DateTime<Utc>],
        config: &TemporalFeatureConfig,
    ) -> Result<(Vec<f64>, Vec<String>), StrategyError> {
        let mut features = Vec::new();
        let mut names = Vec::new();
        
        // 滞后特征
        for &lag in &config.lag_features {
            if prices.len() > lag {
                let lagged_price = prices[prices.len() - 1 - lag];
                features.push(lagged_price);
                names.push(format!("price_lag_{}", lag));
                
                // 价格变化
                if let Some(&current_price) = prices.last() {
                    let price_change = (current_price - lagged_price) / lagged_price.abs().max(1e-8);
                    features.push(price_change);
                    names.push(format!("price_change_lag_{}", lag));
                }
            }
            
            if returns.len() > lag {
                let lagged_return = returns[returns.len() - 1 - lag];
                features.push(lagged_return);
                names.push(format!("return_lag_{}", lag));
            }
        }
        
        // 滑动窗口统计
        for &window in &config.rolling_windows {
            if prices.len() >= window {
                let recent_prices = &prices[prices.len() - window..];
                
                // 滑动平均
                let rolling_mean = recent_prices.iter().sum::<f64>() / window as f64;
                features.push(rolling_mean);
                names.push(format!("rolling_mean_{}", window));
                
                // 滑动标准差
                let rolling_std = self.calculate_std(recent_prices, rolling_mean);
                features.push(rolling_std);
                names.push(format!("rolling_std_{}", window));
                
                // 滑动最大值和最小值
                let rolling_max = recent_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let rolling_min = recent_prices.iter().cloned().fold(f64::INFINITY, f64::min);
                features.extend_from_slice(&[rolling_max, rolling_min]);
                names.extend_from_slice(&[
                    format!("rolling_max_{}", window),
                    format!("rolling_min_{}", window),
                ]);
                
                // 当前价格在滑动范围中的位置
                if rolling_max != rolling_min {
                    if let Some(&current_price) = prices.last() {
                        let position = (current_price - rolling_min) / (rolling_max - rolling_min);
                        features.push(position);
                        names.push(format!("rolling_position_{}", window));
                    }
                }
            }
        }
        
        // 差分特征
        for &order in &config.difference_orders {
            if prices.len() > order {
                let diff = prices[prices.len() - 1] - prices[prices.len() - 1 - order];
                features.push(diff);
                names.push(format!("diff_{}", order));
                
                // 相对差分
                let relative_diff = diff / prices[prices.len() - 1 - order].abs().max(1e-8);
                features.push(relative_diff);
                names.push(format!("relative_diff_{}", order));
            }
        }
        
        // 季节性特征
        if config.seasonal_features && !timestamps.is_empty() {
            if let Some(latest_time) = timestamps.last() {
                let hour = latest_time.hour() as f64;
                let day_of_week = latest_time.weekday().number_from_monday() as f64;
                let day_of_month = latest_time.day() as f64;
                
                // 循环编码
                let hour_sin = (2.0 * std::f64::consts::PI * hour / 24.0).sin();
                let hour_cos = (2.0 * std::f64::consts::PI * hour / 24.0).cos();
                let dow_sin = (2.0 * std::f64::consts::PI * day_of_week / 7.0).sin();
                let dow_cos = (2.0 * std::f64::consts::PI * day_of_week / 7.0).cos();
                
                features.extend_from_slice(&[hour_sin, hour_cos, dow_sin, dow_cos]);
                names.extend_from_slice(&[
                    "hour_sin".to_string(),
                    "hour_cos".to_string(),
                    "dow_sin".to_string(),
                    "dow_cos".to_string(),
                ]);
                
                // 月份特征
                let month = latest_time.month() as f64;
                let month_sin = (2.0 * std::f64::consts::PI * month / 12.0).sin();
                let month_cos = (2.0 * std::f64::consts::PI * month / 12.0).cos();
                features.extend_from_slice(&[month_sin, month_cos]);
                names.extend_from_slice(&["month_sin".to_string(), "month_cos".to_string()]);
            }
        }
        
        Ok((features, names))
    }
    
    /// 创建统计特征
    fn create_statistical_features(
        &self,
        prices: &[f64],
        volumes: &[f64],
        returns: &[f64],
        config: &StatisticalFeatureConfig,
    ) -> Result<(Vec<f64>, Vec<String>), StrategyError> {
        let mut features = Vec::new();
        let mut names = Vec::new();
        
        for &window in &config.stat_windows {
            // 价格统计特征
            if prices.len() >= window {
                let recent_prices = &prices[prices.len() - window..];
                let stats = self.calculate_comprehensive_statistics(recent_prices, &config.statistics);
                
                for (stat_type, value) in stats {
                    features.push(value);
                    names.push(format!("price_{}_{}", stat_type, window));
                }
                
                // 分位数特征
                for &quantile in &config.quantiles {
                    let q_value = self.calculate_quantile(recent_prices, quantile);
                    features.push(q_value);
                    names.push(format!("price_q{}_{}", (quantile * 100.0) as usize, window));
                }
            }
            
            // 成交量统计特征
            if volumes.len() >= window {
                let recent_volumes = &volumes[volumes.len() - window..];
                let stats = self.calculate_comprehensive_statistics(recent_volumes, &config.statistics);
                
                for (stat_type, value) in stats {
                    features.push(value);
                    names.push(format!("volume_{}_{}", stat_type, window));
                }
            }
            
            // 收益率统计特征
            if returns.len() >= window {
                let recent_returns = &returns[returns.len() - window..];
                let stats = self.calculate_comprehensive_statistics(recent_returns, &config.statistics);
                
                for (stat_type, value) in stats {
                    features.push(value);
                    names.push(format!("return_{}_{}", stat_type, window));
                }
                
                // 夏普比率
                let mean_return = recent_returns.iter().sum::<f64>() / window as f64;
                let return_std = self.calculate_std(recent_returns, mean_return);
                let sharpe_ratio = if return_std > 0.0 { mean_return / return_std } else { 0.0 };
                features.push(sharpe_ratio);
                names.push(format!("sharpe_ratio_{}", window));
                
                // 最大回撤
                let max_drawdown = self.calculate_max_drawdown(recent_returns);
                features.push(max_drawdown);
                names.push(format!("max_drawdown_{}", window));
            }
        }
        
        Ok((features, names))
    }
    
    /// 创建市场微观结构特征
    fn create_microstructure_features(
        &self,
        market_indicators: &[MarketIndicators],
    ) -> Result<(Vec<f64>, Vec<String>), StrategyError> {
        let mut features = Vec::new();
        let mut names = Vec::new();
        
        if market_indicators.is_empty() {
            return Ok((features, names));
        }
        
        let latest = market_indicators.last().unwrap();
        
        // 订单流特征
        let order_flow_imbalance = (latest.liquidity_index - 0.5) * 2.0;
        features.push(order_flow_imbalance);
        names.push("order_flow_imbalance".to_string());
        
        // 价差特征
        let spread_volatility = latest.bid_ask_spread / latest.liquidity_index.max(0.001);
        features.push(spread_volatility);
        names.push("spread_volatility".to_string());
        
        // API质量特征
        let api_quality = latest.api_success_rate * (1.0 - latest.api_error_rate) / (latest.api_latency_avg / 1000.0).max(0.001);
        features.push(api_quality);
        names.push("api_quality_score".to_string());
        
        // 波动率比率
        let volatility_ratio_1h_4h = latest.volatility_1h / latest.volatility_4h.max(1e-8);
        let volatility_ratio_4h_24h = latest.volatility_4h / latest.volatility_24h.max(1e-8);
        features.extend_from_slice(&[volatility_ratio_1h_4h, volatility_ratio_4h_24h]);
        names.extend_from_slice(&[
            "volatility_ratio_1h_4h".to_string(),
            "volatility_ratio_4h_24h".to_string(),
        ]);
        
        // 成交量异常检测
        let volume_anomaly = (latest.volume_ratio_1h - 1.0).abs();
        features.push(volume_anomaly);
        names.push("volume_anomaly".to_string());
        
        // 如果有历史数据，计算趋势特征
        if market_indicators.len() > 1 {
            let prev = &market_indicators[market_indicators.len() - 2];
            
            // 波动率趋势
            let volatility_trend = (latest.volatility_1h - prev.volatility_1h) / prev.volatility_1h.max(1e-8);
            features.push(volatility_trend);
            names.push("volatility_trend".to_string());
            
            // 流动性趋势
            let liquidity_trend = (latest.liquidity_index - prev.liquidity_index) / prev.liquidity_index.max(1e-8);
            features.push(liquidity_trend);
            names.push("liquidity_trend".to_string());
            
            // 价差趋势
            let spread_trend = (latest.bid_ask_spread - prev.bid_ask_spread) / prev.bid_ask_spread.max(1e-8);
            features.push(spread_trend);
            names.push("spread_trend".to_string());
        }
        
        Ok((features, names))
    }
    
    /// 应用特征变换
    pub async fn transform_features(
        &self,
        features: &Array2<f64>,
        fit_transform: bool,
    ) -> Result<Array2<f64>, StrategyError> {
        let config = self.config.read().await;
        let mut transformed_features = features.clone();
        
        // 应用缩放
        transformed_features = self.apply_scaling(&transformed_features, &config.feature_transformation.scaling_method, fit_transform).await?;
        
        // 应用PCA
        if config.feature_transformation.apply_pca {
            transformed_features = self.apply_pca(&transformed_features, config.feature_transformation.pca_components, fit_transform).await?;
        }
        
        // 应用多项式特征
        if config.feature_transformation.polynomial_features {
            transformed_features = self.apply_polynomial_features(&transformed_features, config.feature_transformation.polynomial_degree, fit_transform).await?;
        }
        
        // 应用对数变换
        if config.feature_transformation.log_transform {
            transformed_features = self.apply_log_transform(&transformed_features);
        }
        
        Ok(transformed_features)
    }
    
    /// 技术指标计算方法实现
    fn simple_moving_average(&self, data: &[f64], window: usize) -> f64 {
        if data.len() < window {
            return data.iter().sum::<f64>() / data.len() as f64;
        }
        
        let recent_data = &data[data.len() - window..];
        recent_data.iter().sum::<f64>() / window as f64
    }
    
    fn calculate_rsi(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() <= period {
            return 50.0;
        }
        
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        
        for i in 1..prices.len() {
            let change = prices[i] - prices[i-1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }
        
        if gains.len() < period {
            return 50.0;
        }
        
        let avg_gain = gains[gains.len() - period..].iter().sum::<f64>() / period as f64;
        let avg_loss = losses[losses.len() - period..].iter().sum::<f64>() / period as f64;
        
        if avg_loss == 0.0 {
            return 100.0;
        }
        
        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }
    
    fn calculate_bollinger_bands(&self, prices: &[f64], period: usize, std_multiplier: f64) -> (f64, f64, f64) {
        if prices.len() < period {
            let mean = prices.iter().sum::<f64>() / prices.len() as f64;
            return (mean, mean, 0.0);
        }
        
        let recent_prices = &prices[prices.len() - period..];
        let mean = recent_prices.iter().sum::<f64>() / period as f64;
        let std_dev = self.calculate_std(recent_prices, mean);
        
        let upper = mean + std_multiplier * std_dev;
        let lower = mean - std_multiplier * std_dev;
        let width = upper - lower;
        
        (upper, lower, width)
    }
    
    fn calculate_macd(&self, prices: &[f64], fast_period: usize, slow_period: usize, signal_period: usize) -> (f64, f64, f64) {
        if prices.len() < slow_period {
            return (0.0, 0.0, 0.0);
        }
        
        // 生产级EMA计算
        let ema_fast = self.exponential_moving_average_production(prices, fast_period);
        let ema_slow = self.exponential_moving_average_production(prices, slow_period);
        let macd = ema_fast - ema_slow;
        
        // 生产级信号线：基于MACD历史的EMA
        let signal = if prices.len() >= slow_period + signal_period {
            // 构建MACD历史序列进行信号线计算
            let mut macd_history = Vec::with_capacity(prices.len() - slow_period + 1);
            
            for i in slow_period..=prices.len() {
                let window_prices = &prices[..i];
                let window_ema_fast = self.exponential_moving_average_production(window_prices, fast_period);
                let window_ema_slow = self.exponential_moving_average_production(window_prices, slow_period);
                macd_history.push(window_ema_fast - window_ema_slow);
            }
            
            // 对MACD历史计算EMA作为信号线
            self.exponential_moving_average_production(&macd_history, signal_period)
        } else {
            macd
        };
        
        let histogram = macd - signal;
        
        (macd, signal, histogram)
    }
    
    /// 生产级EMA计算 - 避免简化实现
    fn exponential_moving_average_production(&self, prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }
        
        if prices.len() < period {
            // 使用简单移动平均作为初始值
            return prices.iter().sum::<f64>() / prices.len() as f64;
        }
        
        let smoothing_factor = 2.0 / (period as f64 + 1.0);
        
        // 使用前N个值的SMA作为EMA的初始值
        let initial_sma = prices[..period].iter().sum::<f64>() / period as f64;
        let mut ema = initial_sma;
        
        // 从第N+1个值开始计算EMA
        for &price in prices.iter().skip(period) {
            ema = (price * smoothing_factor) + (ema * (1.0 - smoothing_factor));
        }
        
        ema
    }
    
    fn exponential_moving_average(&self, prices: &[f64], period: usize) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }
        
        let alpha = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];
        
        for &price in prices.iter().skip(1) {
            ema = alpha * price + (1.0 - alpha) * ema;
        }
        
        ema
    }
    
    fn calculate_stochastic(&self, prices: &[f64], k_period: usize, d_period: usize) -> (f64, f64) {
        if prices.len() < k_period {
            return (50.0, 50.0);
        }
        
        let recent_prices = &prices[prices.len() - k_period..];
        let highest = recent_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let lowest = recent_prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let current_price = prices[prices.len() - 1];
        
        let k = if highest != lowest {
            100.0 * (current_price - lowest) / (highest - lowest)
        } else {
            50.0
        };
        
        // 生产级D值计算：K值的d_period移动平均
        let d = if prices.len() >= k_period + d_period {
            // 计算历史K值
            let mut k_values = Vec::with_capacity(d_period);
            
            for i in 0..d_period {
                let end_idx = prices.len() - i;
                let start_idx = end_idx.saturating_sub(k_period);
                
                if start_idx < end_idx {
                    let window_prices = &prices[start_idx..end_idx];
                    let window_highest = window_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    let window_lowest = window_prices.iter().cloned().fold(f64::INFINITY, f64::min);
                    let window_current = window_prices[window_prices.len() - 1];
                    
                    let window_k = if window_highest != window_lowest {
                        100.0 * (window_current - window_lowest) / (window_highest - window_lowest)
                    } else {
                        50.0
                    };
                    
                    k_values.push(window_k);
                }
            }
            
            if !k_values.is_empty() {
                k_values.iter().sum::<f64>() / k_values.len() as f64
            } else {
                k
            }
        } else {
            k
        };
        
        (k, d)
    }
    
    fn calculate_williams_r(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return -50.0;
        }
        
        let recent_prices = &prices[prices.len() - period..];
        let highest = recent_prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let lowest = recent_prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let current_price = prices[prices.len() - 1];
        
        if highest != lowest {
            -100.0 * (highest - current_price) / (highest - lowest)
        } else {
            -50.0
        }
    }
    
    fn calculate_momentum(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() <= period {
            return 0.0;
        }
        
        let current_price = prices[prices.len() - 1];
        let past_price = prices[prices.len() - 1 - period];
        
        (current_price - past_price) / past_price.abs().max(1e-8)
    }
    
    fn calculate_obv(&self, prices: &[f64], volumes: &[f64]) -> f64 {
        if prices.len() != volumes.len() || prices.len() < 2 {
            return 0.0;
        }
        
        let mut obv = 0.0;
        for i in 1..prices.len() {
            if prices[i] > prices[i-1] {
                obv += volumes[i];
            } else if prices[i] < prices[i-1] {
                obv -= volumes[i];
            }
        }
        
        obv
    }
    
    fn calculate_std(&self, data: &[f64], mean: f64) -> f64 {
        if data.len() <= 1 {
            return 0.0;
        }
        
        let variance = data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / data.len() as f64;
        
        variance.sqrt()
    }
    
    fn calculate_comprehensive_statistics(&self, data: &[f64], stat_types: &[StatisticType]) -> Vec<(String, f64)> {
        let mut results = Vec::new();
        
        if data.is_empty() {
            return results;
        }
        
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        
        for stat_type in stat_types {
            let value = match stat_type {
                StatisticType::Mean => mean,
                StatisticType::Std => self.calculate_std(data, mean),
                StatisticType::Var => {
                    let std = self.calculate_std(data, mean);
                    std * std
                },
                StatisticType::Skew => self.calculate_skewness(data, mean),
                StatisticType::Kurt => self.calculate_kurtosis(data, mean),
                StatisticType::Min => data.iter().cloned().fold(f64::INFINITY, f64::min),
                StatisticType::Max => data.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                StatisticType::Median => self.calculate_quantile(data, 0.5),
                StatisticType::Range => {
                    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    max - min
                },
                StatisticType::IQR => {
                    let q75 = self.calculate_quantile(data, 0.75);
                    let q25 = self.calculate_quantile(data, 0.25);
                    q75 - q25
                },
            };
            
            results.push((format!("{:?}", stat_type).to_lowercase(), value));
        }
        
        results
    }
    
    fn calculate_skewness(&self, data: &[f64], mean: f64) -> f64 {
        if data.len() < 3 {
            return 0.0;
        }
        
        let std = self.calculate_std(data, mean);
        if std == 0.0 {
            return 0.0;
        }
        
        let n = data.len() as f64;
        let sum_cubed_deviations: f64 = data.iter()
            .map(|&x| ((x - mean) / std).powi(3))
            .sum();
        
        (n / ((n - 1.0) * (n - 2.0))) * sum_cubed_deviations
    }
    
    fn calculate_kurtosis(&self, data: &[f64], mean: f64) -> f64 {
        if data.len() < 4 {
            return 3.0;
        }
        
        let std = self.calculate_std(data, mean);
        if std == 0.0 {
            return 3.0;
        }
        
        let n = data.len() as f64;
        let sum_fourth_deviations: f64 = data.iter()
            .map(|&x| ((x - mean) / std).powi(4))
            .sum();
        
        let kurtosis = (n * (n + 1.0) / ((n - 1.0) * (n - 2.0) * (n - 3.0))) * sum_fourth_deviations;
        kurtosis - 3.0 * ((n - 1.0).powi(2) / ((n - 2.0) * (n - 3.0)))
    }
    
    fn calculate_quantile(&self, data: &[f64], quantile: f64) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (quantile * (sorted_data.len() - 1) as f64).round() as usize;
        sorted_data[index.min(sorted_data.len() - 1)]
    }
    
    fn calculate_max_drawdown(&self, returns: &[f64]) -> f64 {
        if returns.is_empty() {
            return 0.0;
        }
        
        let mut cumulative_return = 1.0;
        let mut peak = 1.0;
        let mut max_drawdown: f64 = 0.0;
        
        for &ret in returns {
            cumulative_return *= (1.0 + ret);
            if cumulative_return > peak {
                peak = cumulative_return;
            }
            let drawdown = (peak - cumulative_return) / peak;
            max_drawdown = max_drawdown.max(drawdown);
        }
        
        max_drawdown
    }
    
    // 变换方法的简化实现
    async fn apply_scaling(&self, features: &Array2<f64>, method: &ScalingMethod, fit: bool) -> Result<Array2<f64>, StrategyError> {
        match method {
            ScalingMethod::StandardScaler => {
                if fit {
                    // 计算均值和标准差
                    let means: Vec<f64> = (0..features.ncols()).map(|i| {
                        features.column(i).mean().unwrap_or(0.0)
                    }).collect();
                    
                    let stds: Vec<f64> = (0..features.ncols()).map(|i| {
                        let col = features.column(i);
                        let mean = means[i];
                        let variance = col.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / col.len() as f64;
                        variance.sqrt().max(1e-8)
                    }).collect();
                    
                    // 存储参数
                    let mut params = self.transformation_params.write().await;
                    params.scaling_params = Some(ScalingParams {
                        mean: means.clone(),
                        std: stds.clone(),
                        min: vec![0.0; features.ncols()],
                        max: vec![1.0; features.ncols()],
                    });
                    
                    // 应用标准化
                    let mut scaled = features.clone();
                    for i in 0..features.ncols() {
                        for j in 0..features.nrows() {
                            scaled[[j, i]] = (features[[j, i]] - means[i]) / stds[i];
                        }
                    }
                    Ok(scaled)
                } else {
                    // 使用已存储的参数
                    let params = self.transformation_params.read().await;
                    if let Some(ref scaling_params) = params.scaling_params {
                        let mut scaled = features.clone();
                        for i in 0..features.ncols() {
                            for j in 0..features.nrows() {
                                scaled[[j, i]] = (features[[j, i]] - scaling_params.mean[i]) / scaling_params.std[i];
                            }
                        }
                        Ok(scaled)
                    } else {
                        Err(StrategyError::FeatureEngineeringError("Scaling parameters not fitted".to_string()))
                    }
                }
            },
            ScalingMethod::None => Ok(features.clone()),
            _ => {
                // 其他缩放方法的简化实现
                Ok(features.clone())
            }
        }
    }
    
    async fn apply_pca(&self, features: &Array2<f64>, _n_components: Option<usize>, _fit: bool) -> Result<Array2<f64>, StrategyError> {
        // PCA的简化实现 - 直接返回原始特征
        Ok(features.clone())
    }
    
    async fn apply_polynomial_features(&self, features: &Array2<f64>, _degree: usize, _fit: bool) -> Result<Array2<f64>, StrategyError> {
        // 多项式特征的简化实现 - 直接返回原始特征
        Ok(features.clone())
    }
    
    fn apply_log_transform(&self, features: &Array2<f64>) -> Array2<f64> {
        let mut transformed = features.clone();
        for i in 0..features.nrows() {
            for j in 0..features.ncols() {
                if features[[i, j]] > 0.0 {
                    transformed[[i, j]] = features[[i, j]].ln();
                }
            }
        }
        transformed
    }
    
    /// 获取特征名称
    pub async fn get_feature_names(&self) -> Vec<String> {
        self.feature_names.read().await.clone()
    }
    
    /// 获取选中的特征索引
    pub async fn get_selected_features(&self) -> Vec<usize> {
        self.selected_features.read().await.clone()
    }
    
    /// 更新配置
    pub async fn update_config(&self, config: FeatureEngineeringConfig) {
        *self.config.write().await = config;
    }
} 
                    for i in 0..features.ncols() {
                        for j in 0..features.nrows() {
                            scaled[[j, i]] = (features[[j, i]] - means[i]) / stds[i];
                        }
                    }
                    Ok(scaled)
                } else {
                    // 使用已存储的参数
                    let params = self.transformation_params.read().await;
                    if let Some(ref scaling_params) = params.scaling_params {
                        let mut scaled = features.clone();
                        for i in 0..features.ncols() {
                            for j in 0..features.nrows() {
                                scaled[[j, i]] = (features[[j, i]] - scaling_params.mean[i]) / scaling_params.std[i];
                            }
                        }
                        Ok(scaled)
                    } else {
                        Err(StrategyError::FeatureEngineeringError("Scaling parameters not fitted".to_string()))
                    }
                }
            },
            ScalingMethod::None => Ok(features.clone()),
            _ => {
                // 其他缩放方法的简化实现
                Ok(features.clone())
            }
        }
    }
    
    async fn apply_pca(&self, features: &Array2<f64>, _n_components: Option<usize>, _fit: bool) -> Result<Array2<f64>, StrategyError> {
        // PCA的简化实现 - 直接返回原始特征
        Ok(features.clone())
    }
    
    async fn apply_polynomial_features(&self, features: &Array2<f64>, _degree: usize, _fit: bool) -> Result<Array2<f64>, StrategyError> {
        // 多项式特征的简化实现 - 直接返回原始特征
        Ok(features.clone())
    }
    
    fn apply_log_transform(&self, features: &Array2<f64>) -> Array2<f64> {
        let mut transformed = features.clone();
        for i in 0..features.nrows() {
            for j in 0..features.ncols() {
                if features[[i, j]] > 0.0 {
                    transformed[[i, j]] = features[[i, j]].ln();
                }
            }
        }
        transformed
    }
    
    /// 获取特征名称
    pub async fn get_feature_names(&self) -> Vec<String> {
        self.feature_names.read().await.clone()
    }
    
    /// 获取选中的特征索引
    pub async fn get_selected_features(&self) -> Vec<usize> {
        self.selected_features.read().await.clone()
    }
    
    /// 更新配置
    pub async fn update_config(&self, config: FeatureEngineeringConfig) {
        *self.config.write().await = config;
    }
} 