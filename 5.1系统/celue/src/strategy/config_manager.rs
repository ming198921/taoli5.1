use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use config::{Config, File, ConfigError};

use crate::strategy::core::StrategyError;

/// 配置版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub version_id: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: String,
    pub config_data: HashMap<String, serde_json::Value>,
    pub checksum: String,
}

/// 配置变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeRecord {
    pub change_id: String,
    pub timestamp: DateTime<Utc>,
    pub operator: String,
    pub change_type: ConfigChangeType,
    pub old_version: Option<String>,
    pub new_version: String,
    pub approval_status: ApprovalStatus,
    pub approver: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rollback_to: Option<String>,
}

/// 配置变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigChangeType {
    Create,
    Update,
    Delete,
    Rollback,
}

/// 审批状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Emergency, // 紧急情况下的自动审批
}

/// 策略配置管理器
pub struct StrategyConfigManager {
    /// 配置文件路径
    config_path: PathBuf,
    /// 当前配置
    current_config: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// 配置版本历史
    version_history: Arc<RwLock<Vec<ConfigVersion>>>,
    /// 变更记录
    change_history: Arc<RwLock<Vec<ConfigChangeRecord>>>,
    /// 文件监听器
    _watcher: Option<notify::RecommendedWatcher>,
    /// 热加载回调
    reload_callbacks: Arc<RwLock<Vec<Box<dyn Fn(&HashMap<String, serde_json::Value>) + Send + Sync>>>>,
    /// 审批配置
    approval_config: ApprovalConfig,
}

/// 审批配置
#[derive(Debug, Clone)]
pub struct ApprovalConfig {
    pub require_approval: bool,
    pub emergency_auto_approve: bool,
    pub max_rollback_versions: usize,
    pub config_backup_path: PathBuf,
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            require_approval: true,
            emergency_auto_approve: true,
            max_rollback_versions: 10,
            config_backup_path: PathBuf::from("./config_backups"),
        }
    }
}

impl StrategyConfigManager {
    /// 创建新的配置管理器
    pub async fn new<P: AsRef<Path>>(
        config_path: P,
        approval_config: Option<ApprovalConfig>,
    ) -> Result<Self, StrategyError> {
        let config_path = config_path.as_ref().to_path_buf();
        let approval_config = approval_config.unwrap_or_default();

        // 加载初始配置
        let initial_config = Self::load_config_from_file(&config_path).await?;
        
        // 创建初始版本
        let initial_version = ConfigVersion {
            version_id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            created_by: "system".to_string(),
            description: "Initial configuration".to_string(),
            config_data: initial_config.clone(),
            checksum: Self::calculate_checksum(&initial_config),
        };

        let mut manager = Self {
            config_path,
            current_config: Arc::new(RwLock::new(initial_config)),
            version_history: Arc::new(RwLock::new(vec![initial_version])),
            change_history: Arc::new(RwLock::new(Vec::new())),
            _watcher: None,
            reload_callbacks: Arc::new(RwLock::new(Vec::new())),
            approval_config,
        };

        // 设置文件监听
        manager.setup_file_watcher().await?;

        Ok(manager)
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> HashMap<String, serde_json::Value> {
        self.current_config.read().await.clone()
    }

    /// 获取特定配置项
    pub async fn get_config_value(&self, key: &str) -> Option<serde_json::Value> {
        self.current_config.read().await.get(key).cloned()
    }

    /// 更新配置（需要审批）
    pub async fn update_config(
        &self,
        new_config: HashMap<String, serde_json::Value>,
        operator: String,
        description: String,
        emergency: bool,
    ) -> Result<String, StrategyError> {
        // 验证配置
        self.validate_config(&new_config)?;

        // 创建新版本
        let new_version = ConfigVersion {
            version_id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            created_by: operator.clone(),
            description,
            config_data: new_config.clone(),
            checksum: Self::calculate_checksum(&new_config),
        };

        // 获取当前版本ID
        let current_version_id = {
            let versions = self.version_history.read().await;
            versions.last().map(|v| v.version_id.clone())
        };

        // 创建变更记录
        let change_record = ConfigChangeRecord {
            change_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operator,
            change_type: ConfigChangeType::Update,
            old_version: current_version_id,
            new_version: new_version.version_id.clone(),
            approval_status: if self.approval_config.require_approval && !emergency {
                ApprovalStatus::Pending
            } else if emergency && self.approval_config.emergency_auto_approve {
                ApprovalStatus::Emergency
            } else {
                ApprovalStatus::Approved
            },
            approver: if emergency { Some("system".to_string()) } else { None },
            approved_at: if emergency { Some(Utc::now()) } else { None },
            rollback_to: None,
        };

        // 如果不需要审批或是紧急情况，直接应用
        if matches!(change_record.approval_status, ApprovalStatus::Approved | ApprovalStatus::Emergency) {
            self.apply_config_version(&new_version).await?;
        }

        // 保存版本和变更记录
        {
            let mut versions = self.version_history.write().await;
            let mut changes = self.change_history.write().await;
            
            versions.push(new_version);
            changes.push(change_record.clone());

            // 限制版本历史数量
            if versions.len() > self.approval_config.max_rollback_versions {
                versions.remove(0);
            }
        }

        // 备份配置文件
        self.backup_config_file(&change_record.change_id).await?;

        tracing::info!(
            change_id = %change_record.change_id,
            approval_status = ?change_record.approval_status,
            "Configuration change recorded"
        );

        Ok(change_record.change_id)
    }

    /// 审批配置变更
    pub async fn approve_change(
        &self,
        change_id: &str,
        approver: String,
    ) -> Result<(), StrategyError> {
        let mut changes = self.change_history.write().await;
        
        if let Some(change) = changes.iter_mut().find(|c| c.change_id == change_id) {
            if !matches!(change.approval_status, ApprovalStatus::Pending) {
                return Err(StrategyError::ConfigurationError(
                    "Change is not in pending status".to_string()
                ));
            }

            change.approval_status = ApprovalStatus::Approved;
            change.approver = Some(approver);
            change.approved_at = Some(Utc::now());

            // 应用配置
            let versions = self.version_history.read().await;
            if let Some(version) = versions.iter().find(|v| v.version_id == change.new_version) {
                self.apply_config_version(version).await?;
            }

            tracing::info!(
                change_id = %change_id,
                approver = %change.approver.as_ref().unwrap(),
                "Configuration change approved and applied"
            );

            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Change {} not found", change_id)
            ))
        }
    }

    /// 拒绝配置变更
    pub async fn reject_change(
        &self,
        change_id: &str,
        approver: String,
    ) -> Result<(), StrategyError> {
        let mut changes = self.change_history.write().await;
        
        if let Some(change) = changes.iter_mut().find(|c| c.change_id == change_id) {
            if !matches!(change.approval_status, ApprovalStatus::Pending) {
                return Err(StrategyError::ConfigurationError(
                    "Change is not in pending status".to_string()
                ));
            }

            change.approval_status = ApprovalStatus::Rejected;
            change.approver = Some(approver);
            change.approved_at = Some(Utc::now());

            tracing::info!(
                change_id = %change_id,
                approver = %change.approver.as_ref().unwrap(),
                "Configuration change rejected"
            );

            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Change {} not found", change_id)
            ))
        }
    }

    /// 回滚到指定版本
    pub async fn rollback_to_version(
        &self,
        version_id: &str,
        operator: String,
    ) -> Result<String, StrategyError> {
        let versions = self.version_history.read().await;
        
        if let Some(target_version) = versions.iter().find(|v| v.version_id == version_id) {
            let current_version_id = versions.last().map(|v| v.version_id.clone());
            
            // 创建回滚记录
            let rollback_record = ConfigChangeRecord {
                change_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                operator,
                change_type: ConfigChangeType::Rollback,
                old_version: current_version_id,
                new_version: target_version.version_id.clone(),
                approval_status: ApprovalStatus::Approved, // 回滚通常立即生效
                approver: Some("system".to_string()),
                approved_at: Some(Utc::now()),
                rollback_to: Some(version_id.to_string()),
            };

            // 应用目标版本
            self.apply_config_version(target_version).await?;

            // 记录回滚
            {
                let mut changes = self.change_history.write().await;
                changes.push(rollback_record.clone());
            }

            tracing::info!(
                rollback_to = %version_id,
                change_id = %rollback_record.change_id,
                "Configuration rolled back"
            );

            Ok(rollback_record.change_id)
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Version {} not found", version_id)
            ))
        }
    }

    /// 注册热加载回调
    pub async fn register_reload_callback<F>(&self, callback: F)
    where
        F: Fn(&HashMap<String, serde_json::Value>) + Send + Sync + 'static,
    {
        let mut callbacks = self.reload_callbacks.write().await;
        callbacks.push(Box::new(callback));
    }

    /// 获取版本历史
    pub async fn get_version_history(&self) -> Vec<ConfigVersion> {
        self.version_history.read().await.clone()
    }

    /// 获取变更历史
    pub async fn get_change_history(&self) -> Vec<ConfigChangeRecord> {
        self.change_history.read().await.clone()
    }

    /// 获取待审批的变更
    pub async fn get_pending_changes(&self) -> Vec<ConfigChangeRecord> {
        let changes = self.change_history.read().await;
        changes
            .iter()
            .filter(|c| matches!(c.approval_status, ApprovalStatus::Pending))
            .cloned()
            .collect()
    }

    /// 从文件加载配置
    async fn load_config_from_file(
        config_path: &Path,
    ) -> Result<HashMap<String, serde_json::Value>, StrategyError> {
        let config = Config::builder()
            .add_source(File::from(config_path))
            .build()
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        let config_map = config
            .try_deserialize::<HashMap<String, serde_json::Value>>()
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        Ok(config_map)
    }

    /// 验证配置
    fn validate_config(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), StrategyError> {
        // 基本验证：检查必需的字段
        let required_fields = ["min_profit", "max_capital", "enabled_exchanges"];
        
        for field in &required_fields {
            if !config.contains_key(*field) {
                return Err(StrategyError::ConfigurationError(
                    format!("Missing required field: {}", field)
                ));
            }
        }

        // 验证数值范围
        if let Some(min_profit) = config.get("min_profit") {
            if let Some(value) = min_profit.as_f64() {
                if value < 0.0 || value > 1.0 {
                    return Err(StrategyError::ConfigurationError(
                        "min_profit must be between 0.0 and 1.0".to_string()
                    ));
                }
            }
        }

        Ok(())
    }

    /// 应用配置版本
    async fn apply_config_version(&self, version: &ConfigVersion) -> Result<(), StrategyError> {
        // 更新当前配置
        {
            let mut current_config = self.current_config.write().await;
            *current_config = version.config_data.clone();
        }

        // 调用热加载回调
        let callbacks = self.reload_callbacks.read().await;
        for callback in callbacks.iter() {
            callback(&version.config_data);
        }

        // 写入文件（如果需要）
        self.write_config_to_file(&version.config_data).await?;

        tracing::info!(
            version_id = %version.version_id,
            "Configuration version applied"
        );

        Ok(())
    }

    /// 设置文件监听器
    async fn setup_file_watcher(&mut self) -> Result<(), StrategyError> {
        let config_path = self.config_path.clone();
        let current_config = Arc::clone(&self.current_config);
        let reload_callbacks = Arc::clone(&self.reload_callbacks);

        let mut watcher = notify::recommended_watcher(move |event: Result<Event, notify::Error>| {
            if let Ok(event) = event {
                if matches!(event.kind, EventKind::Modify(_)) {
                    if let Some(path) = event.paths.first() {
                        if path == &config_path {
                            // 文件被修改，重新加载
                            tokio::spawn({
                                let config_path = config_path.clone();
                                let current_config = Arc::clone(&current_config);
                                let reload_callbacks = Arc::clone(&reload_callbacks);
                                
                                async move {
                                    if let Ok(new_config) = Self::load_config_from_file(&config_path).await {
                                        {
                                            let mut config = current_config.write().await;
                                            *config = new_config.clone();
                                        }

                                        let callbacks = reload_callbacks.read().await;
                                        for callback in callbacks.iter() {
                                            callback(&new_config);
                                        }

                                        tracing::info!("Configuration file reloaded");
                                    }
                                }
                            });
                        }
                    }
                }
            }
        })
        .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        watcher
            .watch(&self.config_path, RecursiveMode::NonRecursive)
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        self._watcher = Some(watcher);
        Ok(())
    }

    /// 写入配置到文件
    async fn write_config_to_file(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), StrategyError> {
        let toml_string = toml::to_string_pretty(config)
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        tokio::fs::write(&self.config_path, toml_string)
            .await
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        Ok(())
    }

    /// 备份配置文件
    async fn backup_config_file(&self, change_id: &str) -> Result<(), StrategyError> {
        let backup_dir = &self.approval_config.config_backup_path;
        tokio::fs::create_dir_all(backup_dir)
            .await
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        let backup_file = backup_dir.join(format!("config_backup_{}.toml", change_id));
        tokio::fs::copy(&self.config_path, backup_file)
            .await
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        Ok(())
    }

    /// 计算配置校验和
    fn calculate_checksum(config: &HashMap<String, serde_json::Value>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let serialized = serde_json::to_string(config).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}




use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use config::{Config, File, ConfigError};

use crate::strategy::core::StrategyError;

/// 配置版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub version_id: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: String,
    pub config_data: HashMap<String, serde_json::Value>,
    pub checksum: String,
}

/// 配置变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeRecord {
    pub change_id: String,
    pub timestamp: DateTime<Utc>,
    pub operator: String,
    pub change_type: ConfigChangeType,
    pub old_version: Option<String>,
    pub new_version: String,
    pub approval_status: ApprovalStatus,
    pub approver: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rollback_to: Option<String>,
}

/// 配置变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigChangeType {
    Create,
    Update,
    Delete,
    Rollback,
}

/// 审批状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Emergency, // 紧急情况下的自动审批
}

/// 策略配置管理器
pub struct StrategyConfigManager {
    /// 配置文件路径
    config_path: PathBuf,
    /// 当前配置
    current_config: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// 配置版本历史
    version_history: Arc<RwLock<Vec<ConfigVersion>>>,
    /// 变更记录
    change_history: Arc<RwLock<Vec<ConfigChangeRecord>>>,
    /// 文件监听器
    _watcher: Option<notify::RecommendedWatcher>,
    /// 热加载回调
    reload_callbacks: Arc<RwLock<Vec<Box<dyn Fn(&HashMap<String, serde_json::Value>) + Send + Sync>>>>,
    /// 审批配置
    approval_config: ApprovalConfig,
}

/// 审批配置
#[derive(Debug, Clone)]
pub struct ApprovalConfig {
    pub require_approval: bool,
    pub emergency_auto_approve: bool,
    pub max_rollback_versions: usize,
    pub config_backup_path: PathBuf,
}

impl Default for ApprovalConfig {
    fn default() -> Self {
        Self {
            require_approval: true,
            emergency_auto_approve: true,
            max_rollback_versions: 10,
            config_backup_path: PathBuf::from("./config_backups"),
        }
    }
}

impl StrategyConfigManager {
    /// 创建新的配置管理器
    pub async fn new<P: AsRef<Path>>(
        config_path: P,
        approval_config: Option<ApprovalConfig>,
    ) -> Result<Self, StrategyError> {
        let config_path = config_path.as_ref().to_path_buf();
        let approval_config = approval_config.unwrap_or_default();

        // 加载初始配置
        let initial_config = Self::load_config_from_file(&config_path).await?;
        
        // 创建初始版本
        let initial_version = ConfigVersion {
            version_id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            created_by: "system".to_string(),
            description: "Initial configuration".to_string(),
            config_data: initial_config.clone(),
            checksum: Self::calculate_checksum(&initial_config),
        };

        let mut manager = Self {
            config_path,
            current_config: Arc::new(RwLock::new(initial_config)),
            version_history: Arc::new(RwLock::new(vec![initial_version])),
            change_history: Arc::new(RwLock::new(Vec::new())),
            _watcher: None,
            reload_callbacks: Arc::new(RwLock::new(Vec::new())),
            approval_config,
        };

        // 设置文件监听
        manager.setup_file_watcher().await?;

        Ok(manager)
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> HashMap<String, serde_json::Value> {
        self.current_config.read().await.clone()
    }

    /// 获取特定配置项
    pub async fn get_config_value(&self, key: &str) -> Option<serde_json::Value> {
        self.current_config.read().await.get(key).cloned()
    }

    /// 更新配置（需要审批）
    pub async fn update_config(
        &self,
        new_config: HashMap<String, serde_json::Value>,
        operator: String,
        description: String,
        emergency: bool,
    ) -> Result<String, StrategyError> {
        // 验证配置
        self.validate_config(&new_config)?;

        // 创建新版本
        let new_version = ConfigVersion {
            version_id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            created_by: operator.clone(),
            description,
            config_data: new_config.clone(),
            checksum: Self::calculate_checksum(&new_config),
        };

        // 获取当前版本ID
        let current_version_id = {
            let versions = self.version_history.read().await;
            versions.last().map(|v| v.version_id.clone())
        };

        // 创建变更记录
        let change_record = ConfigChangeRecord {
            change_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            operator,
            change_type: ConfigChangeType::Update,
            old_version: current_version_id,
            new_version: new_version.version_id.clone(),
            approval_status: if self.approval_config.require_approval && !emergency {
                ApprovalStatus::Pending
            } else if emergency && self.approval_config.emergency_auto_approve {
                ApprovalStatus::Emergency
            } else {
                ApprovalStatus::Approved
            },
            approver: if emergency { Some("system".to_string()) } else { None },
            approved_at: if emergency { Some(Utc::now()) } else { None },
            rollback_to: None,
        };

        // 如果不需要审批或是紧急情况，直接应用
        if matches!(change_record.approval_status, ApprovalStatus::Approved | ApprovalStatus::Emergency) {
            self.apply_config_version(&new_version).await?;
        }

        // 保存版本和变更记录
        {
            let mut versions = self.version_history.write().await;
            let mut changes = self.change_history.write().await;
            
            versions.push(new_version);
            changes.push(change_record.clone());

            // 限制版本历史数量
            if versions.len() > self.approval_config.max_rollback_versions {
                versions.remove(0);
            }
        }

        // 备份配置文件
        self.backup_config_file(&change_record.change_id).await?;

        tracing::info!(
            change_id = %change_record.change_id,
            approval_status = ?change_record.approval_status,
            "Configuration change recorded"
        );

        Ok(change_record.change_id)
    }

    /// 审批配置变更
    pub async fn approve_change(
        &self,
        change_id: &str,
        approver: String,
    ) -> Result<(), StrategyError> {
        let mut changes = self.change_history.write().await;
        
        if let Some(change) = changes.iter_mut().find(|c| c.change_id == change_id) {
            if !matches!(change.approval_status, ApprovalStatus::Pending) {
                return Err(StrategyError::ConfigurationError(
                    "Change is not in pending status".to_string()
                ));
            }

            change.approval_status = ApprovalStatus::Approved;
            change.approver = Some(approver);
            change.approved_at = Some(Utc::now());

            // 应用配置
            let versions = self.version_history.read().await;
            if let Some(version) = versions.iter().find(|v| v.version_id == change.new_version) {
                self.apply_config_version(version).await?;
            }

            tracing::info!(
                change_id = %change_id,
                approver = %change.approver.as_ref().unwrap(),
                "Configuration change approved and applied"
            );

            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Change {} not found", change_id)
            ))
        }
    }

    /// 拒绝配置变更
    pub async fn reject_change(
        &self,
        change_id: &str,
        approver: String,
    ) -> Result<(), StrategyError> {
        let mut changes = self.change_history.write().await;
        
        if let Some(change) = changes.iter_mut().find(|c| c.change_id == change_id) {
            if !matches!(change.approval_status, ApprovalStatus::Pending) {
                return Err(StrategyError::ConfigurationError(
                    "Change is not in pending status".to_string()
                ));
            }

            change.approval_status = ApprovalStatus::Rejected;
            change.approver = Some(approver);
            change.approved_at = Some(Utc::now());

            tracing::info!(
                change_id = %change_id,
                approver = %change.approver.as_ref().unwrap(),
                "Configuration change rejected"
            );

            Ok(())
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Change {} not found", change_id)
            ))
        }
    }

    /// 回滚到指定版本
    pub async fn rollback_to_version(
        &self,
        version_id: &str,
        operator: String,
    ) -> Result<String, StrategyError> {
        let versions = self.version_history.read().await;
        
        if let Some(target_version) = versions.iter().find(|v| v.version_id == version_id) {
            let current_version_id = versions.last().map(|v| v.version_id.clone());
            
            // 创建回滚记录
            let rollback_record = ConfigChangeRecord {
                change_id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                operator,
                change_type: ConfigChangeType::Rollback,
                old_version: current_version_id,
                new_version: target_version.version_id.clone(),
                approval_status: ApprovalStatus::Approved, // 回滚通常立即生效
                approver: Some("system".to_string()),
                approved_at: Some(Utc::now()),
                rollback_to: Some(version_id.to_string()),
            };

            // 应用目标版本
            self.apply_config_version(target_version).await?;

            // 记录回滚
            {
                let mut changes = self.change_history.write().await;
                changes.push(rollback_record.clone());
            }

            tracing::info!(
                rollback_to = %version_id,
                change_id = %rollback_record.change_id,
                "Configuration rolled back"
            );

            Ok(rollback_record.change_id)
        } else {
            Err(StrategyError::ConfigurationError(
                format!("Version {} not found", version_id)
            ))
        }
    }

    /// 注册热加载回调
    pub async fn register_reload_callback<F>(&self, callback: F)
    where
        F: Fn(&HashMap<String, serde_json::Value>) + Send + Sync + 'static,
    {
        let mut callbacks = self.reload_callbacks.write().await;
        callbacks.push(Box::new(callback));
    }

    /// 获取版本历史
    pub async fn get_version_history(&self) -> Vec<ConfigVersion> {
        self.version_history.read().await.clone()
    }

    /// 获取变更历史
    pub async fn get_change_history(&self) -> Vec<ConfigChangeRecord> {
        self.change_history.read().await.clone()
    }

    /// 获取待审批的变更
    pub async fn get_pending_changes(&self) -> Vec<ConfigChangeRecord> {
        let changes = self.change_history.read().await;
        changes
            .iter()
            .filter(|c| matches!(c.approval_status, ApprovalStatus::Pending))
            .cloned()
            .collect()
    }

    /// 从文件加载配置
    async fn load_config_from_file(
        config_path: &Path,
    ) -> Result<HashMap<String, serde_json::Value>, StrategyError> {
        let config = Config::builder()
            .add_source(File::from(config_path))
            .build()
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        let config_map = config
            .try_deserialize::<HashMap<String, serde_json::Value>>()
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        Ok(config_map)
    }

    /// 验证配置
    fn validate_config(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), StrategyError> {
        // 基本验证：检查必需的字段
        let required_fields = ["min_profit", "max_capital", "enabled_exchanges"];
        
        for field in &required_fields {
            if !config.contains_key(*field) {
                return Err(StrategyError::ConfigurationError(
                    format!("Missing required field: {}", field)
                ));
            }
        }

        // 验证数值范围
        if let Some(min_profit) = config.get("min_profit") {
            if let Some(value) = min_profit.as_f64() {
                if value < 0.0 || value > 1.0 {
                    return Err(StrategyError::ConfigurationError(
                        "min_profit must be between 0.0 and 1.0".to_string()
                    ));
                }
            }
        }

        Ok(())
    }

    /// 应用配置版本
    async fn apply_config_version(&self, version: &ConfigVersion) -> Result<(), StrategyError> {
        // 更新当前配置
        {
            let mut current_config = self.current_config.write().await;
            *current_config = version.config_data.clone();
        }

        // 调用热加载回调
        let callbacks = self.reload_callbacks.read().await;
        for callback in callbacks.iter() {
            callback(&version.config_data);
        }

        // 写入文件（如果需要）
        self.write_config_to_file(&version.config_data).await?;

        tracing::info!(
            version_id = %version.version_id,
            "Configuration version applied"
        );

        Ok(())
    }

    /// 设置文件监听器
    async fn setup_file_watcher(&mut self) -> Result<(), StrategyError> {
        let config_path = self.config_path.clone();
        let current_config = Arc::clone(&self.current_config);
        let reload_callbacks = Arc::clone(&self.reload_callbacks);

        let mut watcher = notify::recommended_watcher(move |event: Result<Event, notify::Error>| {
            if let Ok(event) = event {
                if matches!(event.kind, EventKind::Modify(_)) {
                    if let Some(path) = event.paths.first() {
                        if path == &config_path {
                            // 文件被修改，重新加载
                            tokio::spawn({
                                let config_path = config_path.clone();
                                let current_config = Arc::clone(&current_config);
                                let reload_callbacks = Arc::clone(&reload_callbacks);
                                
                                async move {
                                    if let Ok(new_config) = Self::load_config_from_file(&config_path).await {
                                        {
                                            let mut config = current_config.write().await;
                                            *config = new_config.clone();
                                        }

                                        let callbacks = reload_callbacks.read().await;
                                        for callback in callbacks.iter() {
                                            callback(&new_config);
                                        }

                                        tracing::info!("Configuration file reloaded");
                                    }
                                }
                            });
                        }
                    }
                }
            }
        })
        .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        watcher
            .watch(&self.config_path, RecursiveMode::NonRecursive)
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        self._watcher = Some(watcher);
        Ok(())
    }

    /// 写入配置到文件
    async fn write_config_to_file(
        &self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), StrategyError> {
        let toml_string = toml::to_string_pretty(config)
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        tokio::fs::write(&self.config_path, toml_string)
            .await
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        Ok(())
    }

    /// 备份配置文件
    async fn backup_config_file(&self, change_id: &str) -> Result<(), StrategyError> {
        let backup_dir = &self.approval_config.config_backup_path;
        tokio::fs::create_dir_all(backup_dir)
            .await
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        let backup_file = backup_dir.join(format!("config_backup_{}.toml", change_id));
        tokio::fs::copy(&self.config_path, backup_file)
            .await
            .map_err(|e| StrategyError::ConfigurationError(e.to_string()))?;

        Ok(())
    }

    /// 计算配置校验和
    fn calculate_checksum(config: &HashMap<String, serde_json::Value>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let serialized = serde_json::to_string(config).unwrap_or_default();
        let mut hasher = DefaultHasher::new();
        serialized.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}



















