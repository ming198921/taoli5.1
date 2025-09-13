//! 模型持久化和序列化模块
//! 支持：模型保存/加载、版本管理、热重载、压缩存储

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use uuid::Uuid;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};

use crate::adaptive_profit::{RealMLModel, MLModelType, ModelHyperparameters, ModelValidationResult};
use crate::core::StrategyError;

/// 模型序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    Bincode,
    Json,
    MessagePack,
    CompressedBincode,
}

/// 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub model_type: MLModelType,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub training_data_hash: String,
    pub hyperparameters: ModelHyperparameters,
    pub validation_metrics: ModelValidationResult,
    pub feature_names: Vec<String>,
    pub model_size_bytes: usize,
    pub compression_ratio: Option<f64>,
    pub checksum: String,
    pub tags: HashMap<String, String>,
}

/// 序列化的模型数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedModel {
    pub metadata: ModelMetadata,
    pub model_data: Vec<u8>,
    pub format: SerializationFormat,
    pub schema_version: String,
}

/// 模型存储管理器
pub struct ModelPersistenceManager {
    /// 存储根路径
    storage_root: PathBuf,
    /// 模型索引
    model_index: Arc<RwLock<HashMap<String, ModelMetadata>>>,
    /// 活跃模型缓存
    model_cache: Arc<RwLock<HashMap<String, Arc<RealMLModel>>>>,
    /// 默认序列化格式
    default_format: SerializationFormat,
    /// 最大缓存大小
    max_cache_size: usize,
    /// 启用压缩
    enable_compression: bool,
}

impl ModelPersistenceManager {
    pub fn new(storage_root: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&storage_root)
            .context("Failed to create storage directory")?;

        // 创建子目录结构
        std::fs::create_dir_all(storage_root.join("models"))?;
        std::fs::create_dir_all(storage_root.join("metadata"))?;
        std::fs::create_dir_all(storage_root.join("backups"))?;

        Ok(Self {
            storage_root,
            model_index: Arc::new(RwLock::new(HashMap::new())),
            model_cache: Arc::new(RwLock::new(HashMap::new())),
            default_format: SerializationFormat::CompressedBincode,
            max_cache_size: std::env::var("CELUE_MODEL_CACHE_SIZE")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(10),
            enable_compression: std::env::var("CELUE_ENABLE_MODEL_COMPRESSION")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(true),
        })
    }

    /// 保存模型
    pub async fn save_model(
        &self,
        model: &RealMLModel,
        metadata: ModelMetadata,
        format: Option<SerializationFormat>,
    ) -> Result<String> {
        let format = format.unwrap_or_else(|| self.default_format.clone());
        let model_id = metadata.model_id.clone();

        // 序列化模型
        let serialized = self.serialize_model(model, metadata.clone(), format.clone()).await?;

        // 保存到磁盘
        let model_path = self.get_model_path(&model_id);
        let metadata_path = self.get_metadata_path(&model_id);

        // 保存模型数据
        match format {
            SerializationFormat::CompressedBincode => {
                self.save_compressed(&model_path, &serialized.model_data).await?;
            },
            _ => {
                tokio::fs::write(&model_path, &serialized.model_data).await
                    .context("Failed to write model file")?;
            }
        }

        // 保存元数据
        let metadata_json = serde_json::to_string_pretty(&serialized.metadata)?;
        tokio::fs::write(&metadata_path, metadata_json).await
            .context("Failed to write metadata file")?;

        // 更新索引
        {
            let mut index = self.model_index.write().await;
            index.insert(model_id.clone(), serialized.metadata);
        }

        // 创建备份
        if std::env::var("CELUE_ENABLE_MODEL_BACKUP").unwrap_or_default() == "true" {
            self.create_backup(&model_id).await?;
        }

        tracing::info!("Model saved: {} (format: {:?})", model_id, format);
        Ok(model_id)
    }

    /// 加载模型
    pub async fn load_model(&self, model_id: &str) -> Result<Arc<RealMLModel>> {
        // 检查缓存
        {
            let cache = self.model_cache.read().await;
            if let Some(cached_model) = cache.get(model_id) {
                tracing::debug!("Loading model from cache: {}", model_id);
                return Ok(cached_model.clone());
            }
        }

        // 从磁盘加载
        let model = self.load_model_from_disk(model_id).await?;
        let model_arc = Arc::new(model);

        // 更新缓存
        {
            let mut cache = self.model_cache.write().await;
            if cache.len() >= self.max_cache_size {
                // LRU淘汰（简化实现）
                if let Some(first_key) = cache.keys().next().cloned() {
                    cache.remove(&first_key);
                }
            }
            cache.insert(model_id.to_string(), model_arc.clone());
        }

        tracing::info!("Model loaded from disk: {}", model_id);
        Ok(model_arc)
    }

    /// 热重载模型
    pub async fn hot_reload_model(&self, model_id: &str) -> Result<Arc<RealMLModel>> {
        tracing::info!("Hot reloading model: {}", model_id);

        // 强制从磁盘重新加载
        let model = self.load_model_from_disk(model_id).await?;
        let model_arc = Arc::new(model);

        // 更新缓存
        {
            let mut cache = self.model_cache.write().await;
            cache.insert(model_id.to_string(), model_arc.clone());
        }

        Ok(model_arc)
    }

    /// 列出所有模型
    pub async fn list_models(&self) -> Result<Vec<ModelMetadata>> {
        let index = self.model_index.read().await;
        Ok(index.values().cloned().collect())
    }

    /// 删除模型
    pub async fn delete_model(&self, model_id: &str) -> Result<()> {
        let model_path = self.get_model_path(model_id);
        let metadata_path = self.get_metadata_path(model_id);

        // 删除文件
        if model_path.exists() {
            tokio::fs::remove_file(&model_path).await
                .context("Failed to remove model file")?;
        }
        if metadata_path.exists() {
            tokio::fs::remove_file(&metadata_path).await
                .context("Failed to remove metadata file")?;
        }

        // 从索引中移除
        {
            let mut index = self.model_index.write().await;
            index.remove(model_id);
        }

        // 从缓存中移除
        {
            let mut cache = self.model_cache.write().await;
            cache.remove(model_id);
        }

        tracing::info!("Model deleted: {}", model_id);
        Ok(())
    }

    /// 模型版本管理
    pub async fn create_model_version(&self, base_model_id: &str, new_model: &RealMLModel) -> Result<String> {
        let base_metadata = {
            let index = self.model_index.read().await;
            index.get(base_model_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Base model not found: {}", base_model_id))?
        };

        // 创建新版本ID
        let new_version = self.increment_version(&base_metadata.version)?;
        let new_model_id = format!("{}_{}", base_model_id, new_version);

        // 创建新的元数据
        let new_metadata = ModelMetadata {
            model_id: new_model_id.clone(),
            version: new_version,
            updated_at: Utc::now(),
            ..base_metadata
        };

        // 保存新版本
        self.save_model(new_model, new_metadata, None).await?;

        Ok(new_model_id)
    }

    /// 模型比较
    pub async fn compare_models(&self, model_id_a: &str, model_id_b: &str) -> Result<ModelComparison> {
        let metadata_a = self.get_model_metadata(model_id_a).await?;
        let metadata_b = self.get_model_metadata(model_id_b).await?;

        let comparison = ModelComparison {
            model_a: metadata_a.clone(),
            model_b: metadata_b.clone(),
            performance_diff: self.calculate_performance_diff(&metadata_a, &metadata_b),
            size_diff: metadata_b.model_size_bytes as i64 - metadata_a.model_size_bytes as i64,
            created_diff: metadata_b.created_at - metadata_a.created_at,
            feature_diff: self.calculate_feature_diff(&metadata_a.feature_names, &metadata_b.feature_names),
        };

        Ok(comparison)
    }

    /// 导出模型
    pub async fn export_model(&self, model_id: &str, export_path: &Path, format: SerializationFormat) -> Result<()> {
        let model = self.load_model(model_id).await?;
        let metadata = self.get_model_metadata(model_id).await?;

        let serialized = self.serialize_model(&model, metadata, format.clone()).await?;

        // 创建导出包
        let export_package = ExportPackage {
            serialized_model: serialized,
            export_timestamp: Utc::now(),
            celue_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let package_data = match format {
            SerializationFormat::Json => serde_json::to_vec_pretty(&export_package)?,
            _ => bincode::serialize(&export_package)?,
        };

        tokio::fs::write(export_path, package_data).await
            .context("Failed to write export file")?;

        tracing::info!("Model exported: {} -> {:?}", model_id, export_path);
        Ok(())
    }

    /// 导入模型
    pub async fn import_model(&self, import_path: &Path, new_model_id: Option<String>) -> Result<String> {
        let package_data = tokio::fs::read(import_path).await
            .context("Failed to read import file")?;

        // 尝试不同格式解析
        let export_package: ExportPackage = if let Ok(pkg) = serde_json::from_slice(&package_data) {
            pkg
        } else {
            bincode::deserialize(&package_data)
                .context("Failed to deserialize import package")?
        };

        let mut metadata = export_package.serialized_model.metadata;
        
        // 更新ID和时间戳
        if let Some(new_id) = new_model_id {
            metadata.model_id = new_id;
        }
        metadata.created_at = Utc::now();
        metadata.updated_at = Utc::now();

        // 反序列化模型
        let model = self.deserialize_model(&export_package.serialized_model).await?;

        // 保存导入的模型
        let model_id = self.save_model(&model, metadata, Some(export_package.serialized_model.format)).await?;

        tracing::info!("Model imported: {} from {:?}", model_id, import_path);
        Ok(model_id)
    }

    /// 模型健康检查
    pub async fn health_check(&self, model_id: &str) -> Result<ModelHealthStatus> {
        let metadata = self.get_model_metadata(model_id).await?;
        let model_path = self.get_model_path(model_id);

        let mut status = ModelHealthStatus {
            model_id: model_id.to_string(),
            is_healthy: true,
            issues: Vec::new(),
            last_check: Utc::now(),
        };

        // 检查文件存在性
        if !model_path.exists() {
            status.is_healthy = false;
            status.issues.push("Model file not found".to_string());
        }

        // 检查文件完整性
        if let Ok(file_data) = tokio::fs::read(&model_path).await {
            let actual_checksum = self.calculate_checksum(&file_data);
            if actual_checksum != metadata.checksum {
                status.is_healthy = false;
                status.issues.push("Checksum mismatch - file may be corrupted".to_string());
            }
        }

        // 检查模型是否可加载
        match self.load_model(model_id).await {
            Ok(_) => {},
            Err(e) => {
                status.is_healthy = false;
                status.issues.push(format!("Model loading failed: {:?}", e));
            }
        }

        Ok(status)
    }

    // 私有辅助方法

    async fn serialize_model(&self, model: &RealMLModel, mut metadata: ModelMetadata, format: SerializationFormat) -> Result<SerializedModel> {
        let model_data = match format {
            SerializationFormat::Bincode | SerializationFormat::CompressedBincode => {
                bincode::serialize(model).context("Bincode serialization failed")?
            },
            SerializationFormat::Json => {
                serde_json::to_vec(model).context("JSON serialization failed")?
            },
            SerializationFormat::MessagePack => {
                rmp_serde::to_vec(model).context("MessagePack serialization failed")?
            },
        };

        // 计算压缩比
        let original_size = model_data.len();
        let final_data = if matches!(format, SerializationFormat::CompressedBincode) {
            let compressed = self.compress_data(&model_data)?;
            metadata.compression_ratio = Some(original_size as f64 / compressed.len() as f64);
            compressed
        } else {
            model_data
        };

        metadata.model_size_bytes = final_data.len();
        metadata.checksum = self.calculate_checksum(&final_data);

        Ok(SerializedModel {
            metadata,
            model_data: final_data,
            format,
            schema_version: "1.0".to_string(),
        })
    }

    async fn deserialize_model(&self, serialized: &SerializedModel) -> Result<RealMLModel> {
        let model_data = match serialized.format {
            SerializationFormat::CompressedBincode => {
                self.decompress_data(&serialized.model_data)?
            },
            _ => serialized.model_data.clone(),
        };

        let model = match serialized.format {
            SerializationFormat::Bincode | SerializationFormat::CompressedBincode => {
                bincode::deserialize(&model_data).context("Bincode deserialization failed")?
            },
            SerializationFormat::Json => {
                serde_json::from_slice(&model_data).context("JSON deserialization failed")?
            },
            SerializationFormat::MessagePack => {
                rmp_serde::from_slice(&model_data).context("MessagePack deserialization failed")?
            },
        };

        Ok(model)
    }

    async fn load_model_from_disk(&self, model_id: &str) -> Result<RealMLModel> {
        let metadata = self.get_model_metadata(model_id).await?;
        let model_path = self.get_model_path(model_id);

        let model_data = if matches!(metadata.format, SerializationFormat::CompressedBincode) {
            self.load_compressed(&model_path).await?
        } else {
            tokio::fs::read(&model_path).await
                .context("Failed to read model file")?
        };

        // 验证校验和
        let actual_checksum = self.calculate_checksum(&model_data);
        if actual_checksum != metadata.checksum {
            return Err(anyhow::anyhow!("Model file corrupted: checksum mismatch"));
        }

        let serialized = SerializedModel {
            metadata,
            model_data,
            format: self.default_format.clone(),
            schema_version: "1.0".to_string(),
        };

        self.deserialize_model(&serialized).await
    }

    async fn get_model_metadata(&self, model_id: &str) -> Result<ModelMetadata> {
        let index = self.model_index.read().await;
        index.get(model_id).cloned()
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))
    }

    fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.storage_root.join("models").join(format!("{}.model", model_id))
    }

    fn get_metadata_path(&self, model_id: &str) -> PathBuf {
        self.storage_root.join("metadata").join(format!("{}.json", model_id))
    }

    async fn save_compressed(&self, path: &Path, data: &[u8]) -> Result<()> {
        let compressed = self.compress_data(data)?;
        tokio::fs::write(path, compressed).await
            .context("Failed to write compressed file")
    }

    async fn load_compressed(&self, path: &Path) -> Result<Vec<u8>> {
        let compressed = tokio::fs::read(path).await
            .context("Failed to read compressed file")?;
        self.decompress_data(&compressed)
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_data(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn increment_version(&self, current_version: &str) -> Result<String> {
        // 简单的版本递增 (1.0 -> 1.1)
        let parts: Vec<&str> = current_version.split('.').collect();
        if parts.len() >= 2 {
            let major: u32 = parts[0].parse().unwrap_or(1);
            let minor: u32 = parts[1].parse().unwrap_or(0);
            Ok(format!("{}.{}", major, minor + 1))
        } else {
            Ok("1.1".to_string())
        }
    }

    async fn create_backup(&self, model_id: &str) -> Result<()> {
        let model_path = self.get_model_path(model_id);
        let metadata_path = self.get_metadata_path(model_id);
        
        let backup_dir = self.storage_root.join("backups").join(model_id);
        std::fs::create_dir_all(&backup_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_model_path = backup_dir.join(format!("{}.model", timestamp));
        let backup_metadata_path = backup_dir.join(format!("{}.json", timestamp));

        tokio::fs::copy(&model_path, &backup_model_path).await?;
        tokio::fs::copy(&metadata_path, &backup_metadata_path).await?;

        Ok(())
    }

    fn calculate_performance_diff(&self, metadata_a: &ModelMetadata, metadata_b: &ModelMetadata) -> f64 {
        // 简化的性能差异计算
        metadata_b.validation_metrics.mse - metadata_a.validation_metrics.mse
    }

    fn calculate_feature_diff(&self, features_a: &[String], features_b: &[String]) -> FeatureDiff {
        let set_a: std::collections::HashSet<_> = features_a.iter().collect();
        let set_b: std::collections::HashSet<_> = features_b.iter().collect();

        FeatureDiff {
            added: set_b.difference(&set_a).map(|s| s.to_string()).collect(),
            removed: set_a.difference(&set_b).map(|s| s.to_string()).collect(),
            common: set_a.intersection(&set_b).map(|s| s.to_string()).collect(),
        }
    }
}

// 辅助结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    pub model_a: ModelMetadata,
    pub model_b: ModelMetadata,
    pub performance_diff: f64,
    pub size_diff: i64,
    pub created_diff: chrono::Duration,
    pub feature_diff: FeatureDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub common: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPackage {
    pub serialized_model: SerializedModel,
    pub export_timestamp: DateTime<Utc>,
    pub celue_version: String,
}

#[derive(Debug, Clone)]
pub struct ModelHealthStatus {
    pub model_id: String,
    pub is_healthy: bool,
    pub issues: Vec<String>,
    pub last_check: DateTime<Utc>,
}

/// 模型存储格式元数据（需要添加到adaptive_profit.rs）
impl ModelMetadata {
    pub fn format(&self) -> SerializationFormat {
        // 从tags中推断格式，默认为CompressedBincode
        if let Some(format_str) = self.tags.get("format") {
            match format_str.as_str() {
                "json" => SerializationFormat::Json,
                "msgpack" => SerializationFormat::MessagePack,
                "bincode" => SerializationFormat::Bincode,
                _ => SerializationFormat::CompressedBincode,
            }
        } else {
            SerializationFormat::CompressedBincode
        }
    }
} 
//! 支持：模型保存/加载、版本管理、热重载、压缩存储

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use uuid::Uuid;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};

use crate::adaptive_profit::{RealMLModel, MLModelType, ModelHyperparameters, ModelValidationResult};
use crate::core::StrategyError;

/// 模型序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    Bincode,
    Json,
    MessagePack,
    CompressedBincode,
}

/// 模型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub model_id: String,
    pub model_type: MLModelType,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub training_data_hash: String,
    pub hyperparameters: ModelHyperparameters,
    pub validation_metrics: ModelValidationResult,
    pub feature_names: Vec<String>,
    pub model_size_bytes: usize,
    pub compression_ratio: Option<f64>,
    pub checksum: String,
    pub tags: HashMap<String, String>,
}

/// 序列化的模型数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedModel {
    pub metadata: ModelMetadata,
    pub model_data: Vec<u8>,
    pub format: SerializationFormat,
    pub schema_version: String,
}

/// 模型存储管理器
pub struct ModelPersistenceManager {
    /// 存储根路径
    storage_root: PathBuf,
    /// 模型索引
    model_index: Arc<RwLock<HashMap<String, ModelMetadata>>>,
    /// 活跃模型缓存
    model_cache: Arc<RwLock<HashMap<String, Arc<RealMLModel>>>>,
    /// 默认序列化格式
    default_format: SerializationFormat,
    /// 最大缓存大小
    max_cache_size: usize,
    /// 启用压缩
    enable_compression: bool,
}

impl ModelPersistenceManager {
    pub fn new(storage_root: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&storage_root)
            .context("Failed to create storage directory")?;

        // 创建子目录结构
        std::fs::create_dir_all(storage_root.join("models"))?;
        std::fs::create_dir_all(storage_root.join("metadata"))?;
        std::fs::create_dir_all(storage_root.join("backups"))?;

        Ok(Self {
            storage_root,
            model_index: Arc::new(RwLock::new(HashMap::new())),
            model_cache: Arc::new(RwLock::new(HashMap::new())),
            default_format: SerializationFormat::CompressedBincode,
            max_cache_size: std::env::var("CELUE_MODEL_CACHE_SIZE")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(10),
            enable_compression: std::env::var("CELUE_ENABLE_MODEL_COMPRESSION")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(true),
        })
    }

    /// 保存模型
    pub async fn save_model(
        &self,
        model: &RealMLModel,
        metadata: ModelMetadata,
        format: Option<SerializationFormat>,
    ) -> Result<String> {
        let format = format.unwrap_or_else(|| self.default_format.clone());
        let model_id = metadata.model_id.clone();

        // 序列化模型
        let serialized = self.serialize_model(model, metadata.clone(), format.clone()).await?;

        // 保存到磁盘
        let model_path = self.get_model_path(&model_id);
        let metadata_path = self.get_metadata_path(&model_id);

        // 保存模型数据
        match format {
            SerializationFormat::CompressedBincode => {
                self.save_compressed(&model_path, &serialized.model_data).await?;
            },
            _ => {
                tokio::fs::write(&model_path, &serialized.model_data).await
                    .context("Failed to write model file")?;
            }
        }

        // 保存元数据
        let metadata_json = serde_json::to_string_pretty(&serialized.metadata)?;
        tokio::fs::write(&metadata_path, metadata_json).await
            .context("Failed to write metadata file")?;

        // 更新索引
        {
            let mut index = self.model_index.write().await;
            index.insert(model_id.clone(), serialized.metadata);
        }

        // 创建备份
        if std::env::var("CELUE_ENABLE_MODEL_BACKUP").unwrap_or_default() == "true" {
            self.create_backup(&model_id).await?;
        }

        tracing::info!("Model saved: {} (format: {:?})", model_id, format);
        Ok(model_id)
    }

    /// 加载模型
    pub async fn load_model(&self, model_id: &str) -> Result<Arc<RealMLModel>> {
        // 检查缓存
        {
            let cache = self.model_cache.read().await;
            if let Some(cached_model) = cache.get(model_id) {
                tracing::debug!("Loading model from cache: {}", model_id);
                return Ok(cached_model.clone());
            }
        }

        // 从磁盘加载
        let model = self.load_model_from_disk(model_id).await?;
        let model_arc = Arc::new(model);

        // 更新缓存
        {
            let mut cache = self.model_cache.write().await;
            if cache.len() >= self.max_cache_size {
                // LRU淘汰（简化实现）
                if let Some(first_key) = cache.keys().next().cloned() {
                    cache.remove(&first_key);
                }
            }
            cache.insert(model_id.to_string(), model_arc.clone());
        }

        tracing::info!("Model loaded from disk: {}", model_id);
        Ok(model_arc)
    }

    /// 热重载模型
    pub async fn hot_reload_model(&self, model_id: &str) -> Result<Arc<RealMLModel>> {
        tracing::info!("Hot reloading model: {}", model_id);

        // 强制从磁盘重新加载
        let model = self.load_model_from_disk(model_id).await?;
        let model_arc = Arc::new(model);

        // 更新缓存
        {
            let mut cache = self.model_cache.write().await;
            cache.insert(model_id.to_string(), model_arc.clone());
        }

        Ok(model_arc)
    }

    /// 列出所有模型
    pub async fn list_models(&self) -> Result<Vec<ModelMetadata>> {
        let index = self.model_index.read().await;
        Ok(index.values().cloned().collect())
    }

    /// 删除模型
    pub async fn delete_model(&self, model_id: &str) -> Result<()> {
        let model_path = self.get_model_path(model_id);
        let metadata_path = self.get_metadata_path(model_id);

        // 删除文件
        if model_path.exists() {
            tokio::fs::remove_file(&model_path).await
                .context("Failed to remove model file")?;
        }
        if metadata_path.exists() {
            tokio::fs::remove_file(&metadata_path).await
                .context("Failed to remove metadata file")?;
        }

        // 从索引中移除
        {
            let mut index = self.model_index.write().await;
            index.remove(model_id);
        }

        // 从缓存中移除
        {
            let mut cache = self.model_cache.write().await;
            cache.remove(model_id);
        }

        tracing::info!("Model deleted: {}", model_id);
        Ok(())
    }

    /// 模型版本管理
    pub async fn create_model_version(&self, base_model_id: &str, new_model: &RealMLModel) -> Result<String> {
        let base_metadata = {
            let index = self.model_index.read().await;
            index.get(base_model_id).cloned()
                .ok_or_else(|| anyhow::anyhow!("Base model not found: {}", base_model_id))?
        };

        // 创建新版本ID
        let new_version = self.increment_version(&base_metadata.version)?;
        let new_model_id = format!("{}_{}", base_model_id, new_version);

        // 创建新的元数据
        let new_metadata = ModelMetadata {
            model_id: new_model_id.clone(),
            version: new_version,
            updated_at: Utc::now(),
            ..base_metadata
        };

        // 保存新版本
        self.save_model(new_model, new_metadata, None).await?;

        Ok(new_model_id)
    }

    /// 模型比较
    pub async fn compare_models(&self, model_id_a: &str, model_id_b: &str) -> Result<ModelComparison> {
        let metadata_a = self.get_model_metadata(model_id_a).await?;
        let metadata_b = self.get_model_metadata(model_id_b).await?;

        let comparison = ModelComparison {
            model_a: metadata_a.clone(),
            model_b: metadata_b.clone(),
            performance_diff: self.calculate_performance_diff(&metadata_a, &metadata_b),
            size_diff: metadata_b.model_size_bytes as i64 - metadata_a.model_size_bytes as i64,
            created_diff: metadata_b.created_at - metadata_a.created_at,
            feature_diff: self.calculate_feature_diff(&metadata_a.feature_names, &metadata_b.feature_names),
        };

        Ok(comparison)
    }

    /// 导出模型
    pub async fn export_model(&self, model_id: &str, export_path: &Path, format: SerializationFormat) -> Result<()> {
        let model = self.load_model(model_id).await?;
        let metadata = self.get_model_metadata(model_id).await?;

        let serialized = self.serialize_model(&model, metadata, format.clone()).await?;

        // 创建导出包
        let export_package = ExportPackage {
            serialized_model: serialized,
            export_timestamp: Utc::now(),
            celue_version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let package_data = match format {
            SerializationFormat::Json => serde_json::to_vec_pretty(&export_package)?,
            _ => bincode::serialize(&export_package)?,
        };

        tokio::fs::write(export_path, package_data).await
            .context("Failed to write export file")?;

        tracing::info!("Model exported: {} -> {:?}", model_id, export_path);
        Ok(())
    }

    /// 导入模型
    pub async fn import_model(&self, import_path: &Path, new_model_id: Option<String>) -> Result<String> {
        let package_data = tokio::fs::read(import_path).await
            .context("Failed to read import file")?;

        // 尝试不同格式解析
        let export_package: ExportPackage = if let Ok(pkg) = serde_json::from_slice(&package_data) {
            pkg
        } else {
            bincode::deserialize(&package_data)
                .context("Failed to deserialize import package")?
        };

        let mut metadata = export_package.serialized_model.metadata;
        
        // 更新ID和时间戳
        if let Some(new_id) = new_model_id {
            metadata.model_id = new_id;
        }
        metadata.created_at = Utc::now();
        metadata.updated_at = Utc::now();

        // 反序列化模型
        let model = self.deserialize_model(&export_package.serialized_model).await?;

        // 保存导入的模型
        let model_id = self.save_model(&model, metadata, Some(export_package.serialized_model.format)).await?;

        tracing::info!("Model imported: {} from {:?}", model_id, import_path);
        Ok(model_id)
    }

    /// 模型健康检查
    pub async fn health_check(&self, model_id: &str) -> Result<ModelHealthStatus> {
        let metadata = self.get_model_metadata(model_id).await?;
        let model_path = self.get_model_path(model_id);

        let mut status = ModelHealthStatus {
            model_id: model_id.to_string(),
            is_healthy: true,
            issues: Vec::new(),
            last_check: Utc::now(),
        };

        // 检查文件存在性
        if !model_path.exists() {
            status.is_healthy = false;
            status.issues.push("Model file not found".to_string());
        }

        // 检查文件完整性
        if let Ok(file_data) = tokio::fs::read(&model_path).await {
            let actual_checksum = self.calculate_checksum(&file_data);
            if actual_checksum != metadata.checksum {
                status.is_healthy = false;
                status.issues.push("Checksum mismatch - file may be corrupted".to_string());
            }
        }

        // 检查模型是否可加载
        match self.load_model(model_id).await {
            Ok(_) => {},
            Err(e) => {
                status.is_healthy = false;
                status.issues.push(format!("Model loading failed: {:?}", e));
            }
        }

        Ok(status)
    }

    // 私有辅助方法

    async fn serialize_model(&self, model: &RealMLModel, mut metadata: ModelMetadata, format: SerializationFormat) -> Result<SerializedModel> {
        let model_data = match format {
            SerializationFormat::Bincode | SerializationFormat::CompressedBincode => {
                bincode::serialize(model).context("Bincode serialization failed")?
            },
            SerializationFormat::Json => {
                serde_json::to_vec(model).context("JSON serialization failed")?
            },
            SerializationFormat::MessagePack => {
                rmp_serde::to_vec(model).context("MessagePack serialization failed")?
            },
        };

        // 计算压缩比
        let original_size = model_data.len();
        let final_data = if matches!(format, SerializationFormat::CompressedBincode) {
            let compressed = self.compress_data(&model_data)?;
            metadata.compression_ratio = Some(original_size as f64 / compressed.len() as f64);
            compressed
        } else {
            model_data
        };

        metadata.model_size_bytes = final_data.len();
        metadata.checksum = self.calculate_checksum(&final_data);

        Ok(SerializedModel {
            metadata,
            model_data: final_data,
            format,
            schema_version: "1.0".to_string(),
        })
    }

    async fn deserialize_model(&self, serialized: &SerializedModel) -> Result<RealMLModel> {
        let model_data = match serialized.format {
            SerializationFormat::CompressedBincode => {
                self.decompress_data(&serialized.model_data)?
            },
            _ => serialized.model_data.clone(),
        };

        let model = match serialized.format {
            SerializationFormat::Bincode | SerializationFormat::CompressedBincode => {
                bincode::deserialize(&model_data).context("Bincode deserialization failed")?
            },
            SerializationFormat::Json => {
                serde_json::from_slice(&model_data).context("JSON deserialization failed")?
            },
            SerializationFormat::MessagePack => {
                rmp_serde::from_slice(&model_data).context("MessagePack deserialization failed")?
            },
        };

        Ok(model)
    }

    async fn load_model_from_disk(&self, model_id: &str) -> Result<RealMLModel> {
        let metadata = self.get_model_metadata(model_id).await?;
        let model_path = self.get_model_path(model_id);

        let model_data = if matches!(metadata.format, SerializationFormat::CompressedBincode) {
            self.load_compressed(&model_path).await?
        } else {
            tokio::fs::read(&model_path).await
                .context("Failed to read model file")?
        };

        // 验证校验和
        let actual_checksum = self.calculate_checksum(&model_data);
        if actual_checksum != metadata.checksum {
            return Err(anyhow::anyhow!("Model file corrupted: checksum mismatch"));
        }

        let serialized = SerializedModel {
            metadata,
            model_data,
            format: self.default_format.clone(),
            schema_version: "1.0".to_string(),
        };

        self.deserialize_model(&serialized).await
    }

    async fn get_model_metadata(&self, model_id: &str) -> Result<ModelMetadata> {
        let index = self.model_index.read().await;
        index.get(model_id).cloned()
            .ok_or_else(|| anyhow::anyhow!("Model not found: {}", model_id))
    }

    fn get_model_path(&self, model_id: &str) -> PathBuf {
        self.storage_root.join("models").join(format!("{}.model", model_id))
    }

    fn get_metadata_path(&self, model_id: &str) -> PathBuf {
        self.storage_root.join("metadata").join(format!("{}.json", model_id))
    }

    async fn save_compressed(&self, path: &Path, data: &[u8]) -> Result<()> {
        let compressed = self.compress_data(data)?;
        tokio::fs::write(path, compressed).await
            .context("Failed to write compressed file")
    }

    async fn load_compressed(&self, path: &Path) -> Result<Vec<u8>> {
        let compressed = tokio::fs::read(path).await
            .context("Failed to read compressed file")?;
        self.decompress_data(&compressed)
    }

    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_data(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    fn increment_version(&self, current_version: &str) -> Result<String> {
        // 简单的版本递增 (1.0 -> 1.1)
        let parts: Vec<&str> = current_version.split('.').collect();
        if parts.len() >= 2 {
            let major: u32 = parts[0].parse().unwrap_or(1);
            let minor: u32 = parts[1].parse().unwrap_or(0);
            Ok(format!("{}.{}", major, minor + 1))
        } else {
            Ok("1.1".to_string())
        }
    }

    async fn create_backup(&self, model_id: &str) -> Result<()> {
        let model_path = self.get_model_path(model_id);
        let metadata_path = self.get_metadata_path(model_id);
        
        let backup_dir = self.storage_root.join("backups").join(model_id);
        std::fs::create_dir_all(&backup_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_model_path = backup_dir.join(format!("{}.model", timestamp));
        let backup_metadata_path = backup_dir.join(format!("{}.json", timestamp));

        tokio::fs::copy(&model_path, &backup_model_path).await?;
        tokio::fs::copy(&metadata_path, &backup_metadata_path).await?;

        Ok(())
    }

    fn calculate_performance_diff(&self, metadata_a: &ModelMetadata, metadata_b: &ModelMetadata) -> f64 {
        // 简化的性能差异计算
        metadata_b.validation_metrics.mse - metadata_a.validation_metrics.mse
    }

    fn calculate_feature_diff(&self, features_a: &[String], features_b: &[String]) -> FeatureDiff {
        let set_a: std::collections::HashSet<_> = features_a.iter().collect();
        let set_b: std::collections::HashSet<_> = features_b.iter().collect();

        FeatureDiff {
            added: set_b.difference(&set_a).map(|s| s.to_string()).collect(),
            removed: set_a.difference(&set_b).map(|s| s.to_string()).collect(),
            common: set_a.intersection(&set_b).map(|s| s.to_string()).collect(),
        }
    }
}

// 辅助结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    pub model_a: ModelMetadata,
    pub model_b: ModelMetadata,
    pub performance_diff: f64,
    pub size_diff: i64,
    pub created_diff: chrono::Duration,
    pub feature_diff: FeatureDiff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub common: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPackage {
    pub serialized_model: SerializedModel,
    pub export_timestamp: DateTime<Utc>,
    pub celue_version: String,
}

#[derive(Debug, Clone)]
pub struct ModelHealthStatus {
    pub model_id: String,
    pub is_healthy: bool,
    pub issues: Vec<String>,
    pub last_check: DateTime<Utc>,
}

/// 模型存储格式元数据（需要添加到adaptive_profit.rs）
impl ModelMetadata {
    pub fn format(&self) -> SerializationFormat {
        // 从tags中推断格式，默认为CompressedBincode
        if let Some(format_str) = self.tags.get("format") {
            match format_str.as_str() {
                "json" => SerializationFormat::Json,
                "msgpack" => SerializationFormat::MessagePack,
                "bincode" => SerializationFormat::Bincode,
                _ => SerializationFormat::CompressedBincode,
            }
        } else {
            SerializationFormat::CompressedBincode
        }
    }
} 