use std::collections::HashMap;
use std::sync::Arc;
use ndarray::{Array1, Array2, Axis, s};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use tracing::{info, warn, debug};
use rayon::prelude::*;
use tokio::sync::RwLock;

/// Enhanced SHAP explainer with TreeSHAP and KernelSHAP support
#[derive(Debug)]
pub struct EnhancedShapExplainer {
    max_coalition_size: usize,
    sampling_config: SamplingConfig,
    parallel_config: ParallelConfig,
    cache: Arc<RwLock<ShapCache>>,
    algorithm: ShapAlgorithm,
}

#[derive(Debug, Clone)]
pub enum ShapAlgorithm {
    TreeShap { max_depth: usize },
    KernelShap { n_samples: usize },
    DeepShap { layers: Vec<usize> },
    LinearShap,
}

#[derive(Debug, Default)]
struct ShapCache {
    coalition_values: HashMap<Vec<usize>, f64>,
    background_predictions: HashMap<u64, f64>,
}

#[derive(Debug, Clone)]
pub struct SamplingConfig {
    pub max_samples: usize,
    pub coalition_sampling_rate: f64,
    pub use_monte_carlo: bool,
    pub monte_carlo_iterations: usize,
    pub adaptive_sampling: bool,
    pub convergence_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct ParallelConfig {
    pub worker_threads: usize,
    pub batch_size: usize,
    pub use_gpu: bool,
    pub memory_limit_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedShapValues {
    pub values: Array2<f64>,
    pub expected_value: f64,
    pub feature_names: Vec<String>,
    pub feature_importance: HashMap<String, f64>,
    pub interaction_values: Option<Array2<f64>>,
    pub confidence_intervals: Option<HashMap<String, (f64, f64)>>,
    pub computation_time_ms: u64,
}

impl EnhancedShapExplainer {
    pub fn new(algorithm: ShapAlgorithm) -> Self {
        Self {
            max_coalition_size: 15,
            sampling_config: SamplingConfig {
                max_samples: 5000,
                coalition_sampling_rate: 0.1,
                use_monte_carlo: true,
                monte_carlo_iterations: 10000,
                adaptive_sampling: true,
                convergence_threshold: 0.001,
            },
            parallel_config: ParallelConfig {
                worker_threads: num_cpus::get(),
                batch_size: 64,
                use_gpu: false,
                memory_limit_gb: 8.0,
            },
            cache: Arc::new(RwLock::new(ShapCache::default())),
            algorithm,
        }
    }

    pub async fn explain_prediction<F>(
        &self,
        predict_fn: F,
        features: &Array2<f64>,
        background_data: &Array2<f64>,
        feature_names: &[String],
    ) -> Result<EnhancedShapValues>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>> + Send + Sync + Clone + 'static,
    {
        let start_time = std::time::Instant::now();
        let n_samples = features.nrows();
        let n_features = features.ncols();
        
        info!("Starting enhanced SHAP analysis: {} samples, {} features", n_samples, n_features);
        
        let shap_values = match &self.algorithm {
            ShapAlgorithm::TreeShap { max_depth } => {
                self.compute_tree_shap(predict_fn.clone(), features, background_data, *max_depth).await?
            },
            ShapAlgorithm::KernelShap { n_samples } => {
                self.compute_kernel_shap(predict_fn.clone(), features, background_data, *n_samples).await?
            },
            ShapAlgorithm::DeepShap { layers } => {
                self.compute_deep_shap(predict_fn.clone(), features, background_data, layers).await?
            },
            ShapAlgorithm::LinearShap => {
                self.compute_linear_shap(features, background_data).await?
            },
        };
        
        let expected_value = self.calculate_expected_value(predict_fn.clone(), background_data).await?;
        
        let feature_importance = self.calculate_feature_importance(&shap_values, feature_names);
        
        let interaction_values = if n_features <= 20 && self.should_compute_interactions() {
            Some(self.compute_interaction_matrix(predict_fn.clone(), features, background_data).await?)
        } else {
            None
        };
        
        let confidence_intervals = if self.sampling_config.adaptive_sampling {
            Some(self.compute_confidence_intervals(&shap_values, feature_names).await?)
        } else {
            None
        };
        
        let computation_time_ms = start_time.elapsed().as_millis() as u64;
        
        info!("SHAP analysis completed in {}ms", computation_time_ms);
        
        Ok(EnhancedShapValues {
            values: shap_values,
            expected_value,
            feature_names: feature_names.to_vec(),
            feature_importance,
            interaction_values,
            confidence_intervals,
            computation_time_ms,
        })
    }

    async fn compute_tree_shap<F>(
        &self,
        predict_fn: F,
        features: &Array2<f64>,
        background_data: &Array2<f64>,
        max_depth: usize,
    ) -> Result<Array2<f64>>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>> + Send + Sync + Clone + 'static,
    {
        let n_samples = features.nrows();
        let n_features = features.ncols();
        let mut shap_values = Array2::zeros((n_samples, n_features));
        
        let chunks: Vec<_> = (0..n_samples)
            .collect::<Vec<_>>()
            .chunks(self.parallel_config.batch_size)
            .map(|c| c.to_vec())
            .collect();
        
        let results: Vec<_> = chunks
            .par_iter()
            .map(|chunk| {
                let mut batch_values = Array2::zeros((chunk.len(), n_features));
                for (local_idx, &global_idx) in chunk.iter().enumerate() {
                    let sample = features.row(global_idx);
                    for feature_idx in 0..n_features {
                        let value = self.tree_shap_recursive(
                            &predict_fn,
                            &sample,
                            background_data,
                            feature_idx,
                            0,
                            max_depth,
                        );
                        if let Ok(v) = value {
                            batch_values[[local_idx, feature_idx]] = v;
                        }
                    }
                }
                batch_values
            })
            .collect();
        
        for (chunk_idx, batch_values) in results.into_iter().enumerate() {
            let chunk = &chunks[chunk_idx];
            for (local_idx, &global_idx) in chunk.iter().enumerate() {
                for feature_idx in 0..n_features {
                    shap_values[[global_idx, feature_idx]] = batch_values[[local_idx, feature_idx]];
                }
            }
        }
        
        Ok(shap_values)
    }

    fn tree_shap_recursive<F>(
        &self,
        predict_fn: &F,
        sample: &ndarray::ArrayView1<f64>,
        background_data: &Array2<f64>,
        feature_idx: usize,
        depth: usize,
        max_depth: usize,
    ) -> Result<f64>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>>,
    {
        if depth >= max_depth {
            return Ok(0.0);
        }
        
        let n_features = sample.len();
        let background_mean = background_data.mean_axis(Axis(0)).unwrap();
        
        let mut with_feature = Array2::from_shape_fn((1, n_features), |(_, j)| {
            if j == feature_idx {
                sample[j]
            } else {
                background_mean[j]
            }
        });
        
        let mut without_feature = Array2::from_shape_fn((1, n_features), |(_, j)| {
            background_mean[j]
        });
        
        let pred_with = predict_fn(&with_feature)?[0];
        let pred_without = predict_fn(&without_feature)?[0];
        
        Ok(pred_with - pred_without)
    }

    async fn compute_kernel_shap<F>(
        &self,
        predict_fn: F,
        features: &Array2<f64>,
        background_data: &Array2<f64>,
        n_samples: usize,
    ) -> Result<Array2<f64>>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>> + Send + Sync + Clone,
    {
        let n_features = features.ncols();
        let n_instances = features.nrows();
        let mut shap_values = Array2::zeros((n_instances, n_features));
        
        for instance_idx in 0..n_instances {
            let instance = features.row(instance_idx);
            let background_sample = self.sample_background(background_data, n_samples);
            
            let kernel_weights = self.compute_kernel_weights(&instance, &background_sample);
            
            for feature_idx in 0..n_features {
                let mut weighted_sum = 0.0;
                let mut weight_sum = 0.0;
                
                for (sample_idx, background_instance) in background_sample.outer_iter().enumerate() {
                    let weight = kernel_weights[sample_idx];
                    
                    let mut perturbed = background_instance.to_owned();
                    perturbed[feature_idx] = instance[feature_idx];
                    
                    let pred_original = predict_fn(&background_instance.insert_axis(Axis(0)))?[0];
                    let pred_perturbed = predict_fn(&perturbed.insert_axis(Axis(0)))?[0];
                    
                    weighted_sum += weight * (pred_perturbed - pred_original);
                    weight_sum += weight;
                }
                
                if weight_sum > 0.0 {
                    shap_values[[instance_idx, feature_idx]] = weighted_sum / weight_sum;
                }
            }
        }
        
        Ok(shap_values)
    }

    async fn compute_deep_shap<F>(
        &self,
        predict_fn: F,
        features: &Array2<f64>,
        background_data: &Array2<f64>,
        layers: &[usize],
    ) -> Result<Array2<f64>>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>> + Send + Sync,
    {
        let n_samples = features.nrows();
        let n_features = features.ncols();
        let mut shap_values = Array2::zeros((n_samples, n_features));
        
        for layer_size in layers {
            let layer_contributions = self.compute_layer_contributions(
                &predict_fn,
                features,
                background_data,
                *layer_size,
            ).await?;
            
            shap_values = shap_values + layer_contributions;
        }
        
        shap_values /= layers.len() as f64;
        
        Ok(shap_values)
    }

    async fn compute_linear_shap(
        &self,
        features: &Array2<f64>,
        background_data: &Array2<f64>,
    ) -> Result<Array2<f64>> {
        let n_samples = features.nrows();
        let n_features = features.ncols();
        
        let background_mean = background_data.mean_axis(Axis(0)).unwrap();
        
        let mut shap_values = Array2::zeros((n_samples, n_features));
        
        for i in 0..n_samples {
            for j in 0..n_features {
                shap_values[[i, j]] = features[[i, j]] - background_mean[j];
            }
        }
        
        Ok(shap_values)
    }

    async fn compute_layer_contributions<F>(
        &self,
        predict_fn: &F,
        features: &Array2<f64>,
        background_data: &Array2<f64>,
        layer_size: usize,
    ) -> Result<Array2<f64>>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>>,
    {
        let n_samples = features.nrows();
        let n_features = features.ncols();
        let mut contributions = Array2::zeros((n_samples, n_features));
        
        let background_mean = background_data.mean_axis(Axis(0)).unwrap();
        
        for sample_idx in 0..n_samples.min(layer_size) {
            let sample = features.row(sample_idx);
            
            for feature_idx in 0..n_features {
                let mut with_feature = background_mean.to_owned();
                with_feature[feature_idx] = sample[feature_idx];
                
                let pred_with = predict_fn(&with_feature.insert_axis(Axis(0)))?[0];
                let pred_without = predict_fn(&background_mean.to_owned().insert_axis(Axis(0)))?[0];
                
                contributions[[sample_idx, feature_idx]] = pred_with - pred_without;
            }
        }
        
        Ok(contributions)
    }

    fn sample_background(&self, background_data: &Array2<f64>, n_samples: usize) -> Array2<f64> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        let n_background = background_data.nrows();
        let indices: Vec<usize> = (0..n_background)
            .collect::<Vec<_>>()
            .choose_multiple(&mut rng, n_samples.min(n_background))
            .cloned()
            .collect();
        
        let mut sampled = Array2::zeros((indices.len(), background_data.ncols()));
        for (i, &idx) in indices.iter().enumerate() {
            sampled.row_mut(i).assign(&background_data.row(idx));
        }
        
        sampled
    }

    fn compute_kernel_weights(
        &self,
        instance: &ndarray::ArrayView1<f64>,
        background_sample: &Array2<f64>,
    ) -> Vec<f64> {
        let n_samples = background_sample.nrows();
        let mut weights = vec![0.0; n_samples];
        
        for i in 0..n_samples {
            let background_instance = background_sample.row(i);
            let distance = instance.iter()
                .zip(background_instance.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            
            weights[i] = (-distance / 2.0).exp();
        }
        
        let sum: f64 = weights.iter().sum();
        if sum > 0.0 {
            weights.iter_mut().for_each(|w| *w /= sum);
        }
        
        weights
    }

    async fn calculate_expected_value<F>(
        &self,
        predict_fn: F,
        background_data: &Array2<f64>,
    ) -> Result<f64>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>>,
    {
        let predictions = predict_fn(background_data)?;
        Ok(predictions.mean().unwrap_or(0.0))
    }

    fn calculate_feature_importance(
        &self,
        shap_values: &Array2<f64>,
        feature_names: &[String],
    ) -> HashMap<String, f64> {
        let mut importance = HashMap::new();
        let n_features = shap_values.ncols();
        let n_samples = shap_values.nrows();
        
        for (feature_idx, name) in feature_names.iter().enumerate().take(n_features) {
            let feature_importance = shap_values.column(feature_idx)
                .iter()
                .map(|v| v.abs())
                .sum::<f64>() / n_samples as f64;
            importance.insert(name.clone(), feature_importance);
        }
        
        importance
    }

    async fn compute_interaction_matrix<F>(
        &self,
        predict_fn: F,
        features: &Array2<f64>,
        background_data: &Array2<f64>,
    ) -> Result<Array2<f64>>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>> + Send + Sync,
    {
        let n_features = features.ncols();
        let mut interaction_matrix = Array2::zeros((n_features, n_features));
        
        let background_mean = background_data.mean_axis(Axis(0)).unwrap();
        
        for i in 0..n_features {
            for j in (i + 1)..n_features {
                let interaction = self.compute_feature_interaction(
                    &predict_fn,
                    features,
                    &background_mean,
                    i,
                    j,
                ).await?;
                
                interaction_matrix[[i, j]] = interaction;
                interaction_matrix[[j, i]] = interaction;
            }
        }
        
        Ok(interaction_matrix)
    }

    async fn compute_feature_interaction<F>(
        &self,
        predict_fn: &F,
        features: &Array2<f64>,
        background_mean: &Array1<f64>,
        feature_i: usize,
        feature_j: usize,
    ) -> Result<f64>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>>,
    {
        let n_samples = features.nrows().min(100);
        let mut total_interaction = 0.0;
        
        for sample_idx in 0..n_samples {
            let sample = features.row(sample_idx);
            
            let mut baseline = background_mean.to_owned();
            let mut with_i = baseline.clone();
            let mut with_j = baseline.clone();
            let mut with_both = baseline.clone();
            
            with_i[feature_i] = sample[feature_i];
            with_j[feature_j] = sample[feature_j];
            with_both[feature_i] = sample[feature_i];
            with_both[feature_j] = sample[feature_j];
            
            let pred_baseline = predict_fn(&baseline.insert_axis(Axis(0)))?[0];
            let pred_with_i = predict_fn(&with_i.insert_axis(Axis(0)))?[0];
            let pred_with_j = predict_fn(&with_j.insert_axis(Axis(0)))?[0];
            let pred_with_both = predict_fn(&with_both.insert_axis(Axis(0)))?[0];
            
            let interaction = pred_with_both - pred_with_i - pred_with_j + pred_baseline;
            total_interaction += interaction;
        }
        
        Ok(total_interaction / n_samples as f64)
    }

    async fn compute_confidence_intervals(
        &self,
        shap_values: &Array2<f64>,
        feature_names: &[String],
    ) -> Result<HashMap<String, (f64, f64)>> {
        let mut intervals = HashMap::new();
        let n_features = shap_values.ncols();
        
        for (feature_idx, name) in feature_names.iter().enumerate().take(n_features) {
            let values = shap_values.column(feature_idx);
            let mean = values.mean().unwrap_or(0.0);
            let std = values.std(1.0);
            
            let z_score = 1.96;
            let margin = z_score * std / (values.len() as f64).sqrt();
            
            intervals.insert(name.clone(), (mean - margin, mean + margin));
        }
        
        Ok(intervals)
    }

    fn should_compute_interactions(&self) -> bool {
        true
    }
}

/// Enhanced LIME explainer with advanced perturbation strategies
#[derive(Debug)]
pub struct EnhancedLimeExplainer {
    neighborhood_size: usize,
    perturbation_strategy: PerturbationStrategy,
    regularization_alpha: f64,
    feature_selection: FeatureSelection,
    kernel_type: KernelType,
}

#[derive(Debug, Clone)]
pub enum PerturbationStrategy {
    GaussianNoise { std_dev: f64 },
    UniformNoise { range: f64 },
    FeatureMasking { mask_probability: f64 },
    Interpolation { background_samples: usize },
    MixedStrategy { strategies: Vec<(PerturbationStrategy, f64)> },
}

#[derive(Debug, Clone)]
pub enum FeatureSelection {
    None,
    ForwardSelection { max_features: usize },
    Lasso { alpha: f64 },
    Ridge { alpha: f64 },
}

#[derive(Debug, Clone)]
pub enum KernelType {
    Exponential { sigma: f64 },
    Gaussian { sigma: f64 },
    Cosine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedLimeExplanation {
    pub feature_weights: HashMap<String, f64>,
    pub model_fidelity: f64,
    pub neighborhood_size: usize,
    pub selected_features: Vec<String>,
    pub local_prediction: f64,
    pub original_prediction: f64,
}

impl EnhancedLimeExplainer {
    pub fn new() -> Self {
        Self {
            neighborhood_size: 5000,
            perturbation_strategy: PerturbationStrategy::MixedStrategy {
                strategies: vec![
                    (PerturbationStrategy::GaussianNoise { std_dev: 0.1 }, 0.5),
                    (PerturbationStrategy::FeatureMasking { mask_probability: 0.3 }, 0.5),
                ],
            },
            regularization_alpha: 0.01,
            feature_selection: FeatureSelection::Lasso { alpha: 0.01 },
            kernel_type: KernelType::Exponential { sigma: 0.75 },
        }
    }

    pub async fn explain_instance<F>(
        &self,
        predict_fn: F,
        instance: &Array1<f64>,
        feature_names: &[String],
        background_data: &Array2<f64>,
    ) -> Result<EnhancedLimeExplanation>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>> + Send + Sync,
    {
        info!("Starting enhanced LIME analysis");
        
        let original_prediction = predict_fn(&instance.to_owned().insert_axis(Axis(0)))?[0];
        
        let (neighborhood, predictions) = self.generate_neighborhood(
            &predict_fn,
            instance,
            background_data,
        ).await?;
        
        let weights = self.compute_kernel_weights(instance, &neighborhood);
        
        let (feature_weights, selected_features) = self.fit_weighted_model(
            &neighborhood,
            &predictions,
            &weights,
            feature_names,
        ).await?;
        
        let local_prediction = self.compute_local_prediction(instance, &feature_weights);
        
        let model_fidelity = self.compute_fidelity(
            original_prediction,
            local_prediction,
            &predictions,
            &weights,
        );
        
        info!("LIME analysis completed, fidelity: {:.4}", model_fidelity);
        
        Ok(EnhancedLimeExplanation {
            feature_weights,
            model_fidelity,
            neighborhood_size: self.neighborhood_size,
            selected_features,
            local_prediction,
            original_prediction,
        })
    }

    async fn generate_neighborhood<F>(
        &self,
        predict_fn: &F,
        instance: &Array1<f64>,
        background_data: &Array2<f64>,
    ) -> Result<(Array2<f64>, Array1<f64>)>
    where
        F: Fn(&Array2<f64>) -> Result<Array1<f64>>,
    {
        let n_features = instance.len();
        let mut neighborhood = Array2::zeros((self.neighborhood_size, n_features));
        
        match &self.perturbation_strategy {
            PerturbationStrategy::MixedStrategy { strategies } => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                
                for i in 0..self.neighborhood_size {
                    let (strategy, _) = strategies.choose_weighted(&mut rng, |item| item.1)
                        .context("Failed to select perturbation strategy")?;
                    
                    let perturbed = self.apply_perturbation(instance, background_data, strategy)?;
                    neighborhood.row_mut(i).assign(&perturbed);
                }
            },
            strategy => {
                for i in 0..self.neighborhood_size {
                    let perturbed = self.apply_perturbation(instance, background_data, strategy)?;
                    neighborhood.row_mut(i).assign(&perturbed);
                }
            },
        }
        
        let predictions = predict_fn(&neighborhood)?;
        
        Ok((neighborhood, predictions))
    }

    fn apply_perturbation(
        &self,
        instance: &Array1<f64>,
        background_data: &Array2<f64>,
        strategy: &PerturbationStrategy,
    ) -> Result<Array1<f64>> {
        use rand::distributions::{Distribution, Uniform};
        use rand_distr::Normal;
        
        match strategy {
            PerturbationStrategy::GaussianNoise { std_dev } => {
                let mut rng = rand::thread_rng();
                let normal = Normal::new(0.0, *std_dev)?;
                let perturbed = instance.mapv(|x| x + normal.sample(&mut rng));
                Ok(perturbed)
            },
            PerturbationStrategy::UniformNoise { range } => {
                let mut rng = rand::thread_rng();
                let uniform = Uniform::new(-range, *range);
                let perturbed = instance.mapv(|x| x + uniform.sample(&mut rng));
                Ok(perturbed)
            },
            PerturbationStrategy::FeatureMasking { mask_probability } => {
                let mut rng = rand::thread_rng();
                let background_mean = background_data.mean_axis(Axis(0)).unwrap();
                let uniform = Uniform::new(0.0, 1.0);
                let perturbed = instance.iter()
                    .zip(background_mean.iter())
                    .map(|(&inst_val, &bg_val)| {
                        if uniform.sample(&mut rng) < *mask_probability {
                            bg_val
                        } else {
                            inst_val
                        }
                    })
                    .collect();
                Ok(perturbed)
            },
            PerturbationStrategy::Interpolation { background_samples } => {
                use rand::seq::SliceRandom;
                let mut rng = rand::thread_rng();
                let idx = (0..background_data.nrows())
                    .collect::<Vec<_>>()
                    .choose(&mut rng)
                    .copied()
                    .unwrap_or(0);
                let background_instance = background_data.row(idx);
                let alpha = rng.gen::<f64>();
                let perturbed = instance * alpha + &background_instance * (1.0 - alpha);
                Ok(perturbed.to_owned())
            },
            _ => Ok(instance.to_owned()),
        }
    }

    fn compute_kernel_weights(
        &self,
        instance: &Array1<f64>,
        neighborhood: &Array2<f64>,
    ) -> Array1<f64> {
        let n_samples = neighborhood.nrows();
        let mut weights = Array1::zeros(n_samples);
        
        for i in 0..n_samples {
            let neighbor = neighborhood.row(i);
            let distance = match &self.kernel_type {
                KernelType::Exponential { sigma } => {
                    let euclidean = instance.iter()
                        .zip(neighbor.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>()
                        .sqrt();
                    (-euclidean / sigma).exp()
                },
                KernelType::Gaussian { sigma } => {
                    let euclidean = instance.iter()
                        .zip(neighbor.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>()
                        .sqrt();
                    (-euclidean.powi(2) / (2.0 * sigma.powi(2))).exp()
                },
                KernelType::Cosine => {
                    let dot_product: f64 = instance.iter()
                        .zip(neighbor.iter())
                        .map(|(a, b)| a * b)
                        .sum();
                    let norm_a = instance.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
                    let norm_b = neighbor.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
                    if norm_a > 0.0 && norm_b > 0.0 {
                        (dot_product / (norm_a * norm_b) + 1.0) / 2.0
                    } else {
                        0.0
                    }
                },
            };
            weights[i] = distance;
        }
        
        let sum = weights.sum();
        if sum > 0.0 {
            weights /= sum;
        }
        
        weights
    }

    async fn fit_weighted_model(
        &self,
        neighborhood: &Array2<f64>,
        predictions: &Array1<f64>,
        weights: &Array1<f64>,
        feature_names: &[String],
    ) -> Result<(HashMap<String, f64>, Vec<String>)> {
        let n_features = neighborhood.ncols();
        let mut feature_weights = HashMap::new();
        let mut selected_features = Vec::new();
        
        match &self.feature_selection {
            FeatureSelection::Lasso { alpha } => {
                let coefficients = self.lasso_regression(
                    neighborhood,
                    predictions,
                    weights,
                    *alpha,
                )?;
                
                for (i, &coef) in coefficients.iter().enumerate() {
                    if coef.abs() > 1e-6 && i < feature_names.len() {
                        feature_weights.insert(feature_names[i].clone(), coef);
                        selected_features.push(feature_names[i].clone());
                    }
                }
            },
            FeatureSelection::Ridge { alpha } => {
                let coefficients = self.ridge_regression(
                    neighborhood,
                    predictions,
                    weights,
                    *alpha,
                )?;
                
                for (i, &coef) in coefficients.iter().enumerate() {
                    if i < feature_names.len() {
                        feature_weights.insert(feature_names[i].clone(), coef);
                        selected_features.push(feature_names[i].clone());
                    }
                }
            },
            _ => {
                let coefficients = self.weighted_linear_regression(
                    neighborhood,
                    predictions,
                    weights,
                )?;
                
                for (i, &coef) in coefficients.iter().enumerate() {
                    if i < feature_names.len() {
                        feature_weights.insert(feature_names[i].clone(), coef);
                        selected_features.push(feature_names[i].clone());
                    }
                }
            },
        }
        
        Ok((feature_weights, selected_features))
    }

    fn lasso_regression(
        &self,
        x: &Array2<f64>,
        y: &Array1<f64>,
        weights: &Array1<f64>,
        alpha: f64,
    ) -> Result<Array1<f64>> {
        self.weighted_linear_regression_with_regularization(x, y, weights, alpha, 1.0)
    }

    fn ridge_regression(
        &self,
        x: &Array2<f64>,
        y: &Array1<f64>,
        weights: &Array1<f64>,
        alpha: f64,
    ) -> Result<Array1<f64>> {
        self.weighted_linear_regression_with_regularization(x, y, weights, alpha, 2.0)
    }

    fn weighted_linear_regression(
        &self,
        x: &Array2<f64>,
        y: &Array1<f64>,
        weights: &Array1<f64>,
    ) -> Result<Array1<f64>> {
        self.weighted_linear_regression_with_regularization(x, y, weights, 0.0, 2.0)
    }

    fn weighted_linear_regression_with_regularization(
        &self,
        x: &Array2<f64>,
        y: &Array1<f64>,
        weights: &Array1<f64>,
        alpha: f64,
        norm: f64,
    ) -> Result<Array1<f64>> {
        use ndarray_linalg::Solve;
        
        let n_features = x.ncols();
        let w_sqrt = weights.mapv(f64::sqrt);
        
        let x_weighted = x.outer_iter()
            .zip(w_sqrt.iter())
            .map(|(row, &w)| row * w)
            .collect::<Vec<_>>();
        
        let mut x_w = Array2::zeros((x.nrows(), n_features));
        for (i, row) in x_weighted.iter().enumerate() {
            x_w.row_mut(i).assign(row);
        }
        
        let y_weighted = y * &w_sqrt;
        
        let xtx = x_w.t().dot(&x_w);
        let xty = x_w.t().dot(&y_weighted);
        
        let mut regularization = Array2::eye(n_features) * alpha;
        if norm == 1.0 {
            regularization = regularization.mapv(|x| x.abs());
        }
        
        let a = xtx + regularization;
        let coefficients = a.solve(&xty)?;
        
        Ok(coefficients)
    }

    fn compute_local_prediction(
        &self,
        instance: &Array1<f64>,
        feature_weights: &HashMap<String, f64>,
    ) -> f64 {
        instance.iter()
            .enumerate()
            .map(|(i, &val)| {
                feature_weights.get(&format!("feature_{}", i))
                    .unwrap_or(&0.0) * val
            })
            .sum()
    }

    fn compute_fidelity(
        &self,
        original_prediction: f64,
        local_prediction: f64,
        predictions: &Array1<f64>,
        weights: &Array1<f64>,
    ) -> f64 {
        let weighted_mse = predictions.iter()
            .zip(weights.iter())
            .map(|(&pred, &weight)| {
                weight * (pred - local_prediction).powi(2)
            })
            .sum::<f64>();
        
        let total_weight: f64 = weights.sum();
        let rmse = (weighted_mse / total_weight).sqrt();
        
        let prediction_range = predictions.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
            - predictions.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        
        if prediction_range > 0.0 {
            1.0 - (rmse / prediction_range).min(1.0)
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enhanced_shap_explainer() {
        let explainer = EnhancedShapExplainer::new(ShapAlgorithm::KernelShap { n_samples: 100 });
        
        let features = Array2::from_shape_fn((10, 5), |(i, j)| (i * j) as f64);
        let background = Array2::from_shape_fn((20, 5), |(i, j)| (i + j) as f64);
        let feature_names = vec!["f1".to_string(), "f2".to_string(), "f3".to_string(), "f4".to_string(), "f5".to_string()];
        
        let predict_fn = |x: &Array2<f64>| -> Result<Array1<f64>> {
            Ok(x.sum_axis(Axis(1)))
        };
        
        let result = explainer.explain_prediction(
            predict_fn,
            &features,
            &background,
            &feature_names,
        ).await;
        
        assert!(result.is_ok());
        let shap_values = result.unwrap();
        assert_eq!(shap_values.values.shape(), &[10, 5]);
    }

    #[tokio::test]
    async fn test_enhanced_lime_explainer() {
        let explainer = EnhancedLimeExplainer::new();
        
        let instance = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let background = Array2::from_shape_fn((20, 5), |(i, j)| (i + j) as f64);
        let feature_names = vec!["f1".to_string(), "f2".to_string(), "f3".to_string(), "f4".to_string(), "f5".to_string()];
        
        let predict_fn = |x: &Array2<f64>| -> Result<Array1<f64>> {
            Ok(x.sum_axis(Axis(1)))
        };
        
        let result = explainer.explain_instance(
            predict_fn,
            &instance,
            &feature_names,
            &background,
        ).await;
        
        assert!(result.is_ok());
        let explanation = result.unwrap();
        assert!(explanation.model_fidelity >= 0.0 && explanation.model_fidelity <= 1.0);
    }
}