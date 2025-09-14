//! # 生产级错误处理和结果类型
//!
//! 为Qingxi 5.1增强版提供统一的错误处理机制，消除硬编码和不安全的错误处理模式

use std::fmt;
use thiserror::Error;

/// 统一的结果类型，用于所有可能失败的操作
pub type QingxiResult<T> = Result<T, QingxiError>;

/// 统一的错误类型，涵盖所有可能的错误情况
#[derive(Error, Debug)]
pub enum QingxiError {
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Network error: {message}")]
    Network { message: String },
    
    #[error("Exchange adapter error for {exchange}: {message}")]
    ExchangeAdapter { exchange: String, message: String },
    
    #[error("Memory pool error: {message}")]
    MemoryPool { message: String },
    
    #[error("Cache operation failed: {message}")]
    Cache { message: String },
    
    #[error("API operation failed: {message}")]
    Api { message: String },
    
    #[error("System initialization error: {message}")]
    SystemInit { message: String },
    
    #[error("Environment variable error: {var_name} - {message}")]
    Environment { var_name: String, message: String },
    
    #[error("File operation error: {path} - {message}")]
    FileOperation { path: String, message: String },
    
    #[error("Serialization/Deserialization error: {message}")]
    Serialization { message: String },
    
    #[error("Resource not found: {resource_type} - {message}")]
    NotFound { resource_type: String, message: String },
    
    #[error("Invalid operation: {message}")]
    InvalidOperation { message: String },
    
    #[error("IO operation error: {message}")]
    Io { message: String },
    
    #[error("Critical system error: {message}")]
    Critical { message: String },
}

impl QingxiError {
    /// 创建配置错误
    pub fn config<S: AsRef<str>>(message: S) -> Self {
        Self::Config { message: message.as_ref().to_string() }
    }
    
    /// 创建网络错误
    pub fn network<S: AsRef<str>>(message: S) -> Self {
        Self::Network { message: message.as_ref().to_string() }
    }
    
    /// 创建交易所适配器错误
    pub fn exchange_adapter<S1: AsRef<str>, S2: AsRef<str>>(exchange: S1, message: S2) -> Self {
        Self::ExchangeAdapter { 
            exchange: exchange.as_ref().to_string(),
            message: message.as_ref().to_string() 
        }
    }
    
    /// 创建内存池错误
    pub fn memory_pool<S: AsRef<str>>(message: S) -> Self {
        Self::MemoryPool { message: message.as_ref().to_string() }
    }
    
    /// 创建缓存错误
    pub fn cache<S: AsRef<str>>(message: S) -> Self {
        Self::Cache { message: message.as_ref().to_string() }
    }
    
    /// 创建API错误
    pub fn api<S: AsRef<str>>(message: S) -> Self {
        Self::Api { message: message.as_ref().to_string() }
    }
    
    /// 创建系统初始化错误
    pub fn system_init<S: AsRef<str>>(message: S) -> Self {
        Self::SystemInit { message: message.as_ref().to_string() }
    }
    
    /// 创建环境变量错误
    pub fn environment<S1: AsRef<str>, S2: AsRef<str>>(var_name: S1, message: S2) -> Self {
        Self::Environment { 
            var_name: var_name.as_ref().to_string(),
            message: message.as_ref().to_string() 
        }
    }
    
    /// 创建IO错误
    pub fn io<S: AsRef<str>>(message: S) -> Self {
        Self::Io { message: message.as_ref().to_string() }
    }
    
    /// 创建内部系统错误
    pub fn internal<S: AsRef<str>>(message: S) -> Self {
        Self::Critical { message: message.as_ref().to_string() }
    }
    
    /// 创建文件操作错误
    pub fn file_operation<S1: AsRef<str>, S2: AsRef<str>>(path: S1, message: S2) -> Self {
        Self::FileOperation { 
            path: path.as_ref().to_string(),
            message: message.as_ref().to_string() 
        }
    }
    
    /// 创建序列化错误
    pub fn serialization<S: AsRef<str>>(message: S) -> Self {
        Self::Serialization { message: message.as_ref().to_string() }
    }
    
    /// 创建资源未找到错误
    pub fn not_found<S1: AsRef<str>, S2: AsRef<str>>(resource_type: S1, message: S2) -> Self {
        Self::NotFound { 
            resource_type: resource_type.as_ref().to_string(),
            message: message.as_ref().to_string() 
        }
    }
    
    /// 创建无效操作错误
    pub fn invalid_operation<S: AsRef<str>>(message: S) -> Self {
        Self::InvalidOperation { message: message.as_ref().to_string() }
    }
    
    /// 创建关键系统错误
    pub fn critical<S: AsRef<str>>(message: S) -> Self {
        Self::Critical { message: message.as_ref().to_string() }
    }
    
    /// 检查是否为关键错误（需要立即处理）
    pub fn is_critical(&self) -> bool {
        matches!(self, QingxiError::Critical { .. } | QingxiError::MemoryPool { .. })
    }
    
    /// 检查是否为可恢复错误
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            QingxiError::Network { .. } | 
            QingxiError::ExchangeAdapter { .. } |
            QingxiError::Cache { .. }
        )
    }
}

/// 扩展Result类型的便捷方法
pub trait ResultExt<T> {
    /// 将错误转换为QingxiError::Config
    fn config_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T>;
    
    /// 将错误转换为QingxiError::Network
    fn network_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T>;
    
    /// 将错误转换为QingxiError::Api
    fn api_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T>;
    
    /// 安全的unwrap替代方案，提供更好的错误信息
    fn safe_unwrap<S: AsRef<str>>(self, context: S) -> QingxiResult<T>;
}

impl<T, E: fmt::Display> ResultExt<T> for Result<T, E> {
    fn config_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T> {
        self.map_err(|e| QingxiError::config(format!("{}: {}", message.as_ref(), e)))
    }
    
    fn network_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T> {
        self.map_err(|e| QingxiError::network(format!("{}: {}", message.as_ref(), e)))
    }
    
    fn api_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T> {
        self.map_err(|e| QingxiError::api(format!("{}: {}", message.as_ref(), e)))
    }
    
    fn safe_unwrap<S: AsRef<str>>(self, context: S) -> QingxiResult<T> {
        self.map_err(|e| QingxiError::critical(format!("{}: {}", context.as_ref(), e)))
    }
}

impl<T> ResultExt<T> for Option<T> {
    fn config_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T> {
        self.ok_or_else(|| QingxiError::config(message))
    }
    
    fn network_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T> {
        self.ok_or_else(|| QingxiError::network(message))
    }
    
    fn api_error<S: AsRef<str>>(self, message: S) -> QingxiResult<T> {
        self.ok_or_else(|| QingxiError::api(message))
    }
    
    fn safe_unwrap<S: AsRef<str>>(self, context: S) -> QingxiResult<T> {
        self.ok_or_else(|| QingxiError::critical(context))
    }
}

/// 环境变量安全访问工具
pub struct EnvVar;

impl EnvVar {
    /// 安全获取环境变量，提供默认值
    pub fn get_or_default<S: AsRef<str>>(var_name: S, default: S) -> String {
        std::env::var(var_name.as_ref())
            .unwrap_or_else(|_| default.as_ref().to_string())
    }
    
    /// 安全获取环境变量，返回Result
    pub fn get<S: AsRef<str>>(var_name: S) -> QingxiResult<String> {
        std::env::var(var_name.as_ref())
            .map_err(|e| QingxiError::environment(var_name.as_ref(), e.to_string()))
    }
    
    /// 安全获取字符串环境变量，提供默认值（向后兼容方法）
    pub fn get_string<S: AsRef<str>>(var_name: S) -> QingxiResult<String> {
        Self::get(var_name)
    }
    
    /// 安全获取布尔环境变量，提供默认值（向后兼容方法）
    pub fn get_bool<S: AsRef<str>>(var_name: S) -> QingxiResult<bool> {
        Self::get_parsed(var_name)
    }
    
    /// 安全获取并解析环境变量
    pub fn get_parsed<T, S: AsRef<str>>(var_name: S) -> QingxiResult<T> 
    where 
        T: std::str::FromStr,
        T::Err: fmt::Display,
    {
        let value = Self::get(&var_name)?;
        value.parse()
            .map_err(|e| QingxiError::environment(
                var_name.as_ref(), 
                format!("Failed to parse '{}': {}", value, e)
            ))
    }
    
    /// 安全获取并解析环境变量，提供默认值
    pub fn get_parsed_or_default<T, S: AsRef<str>>(var_name: S, default: T) -> T 
    where 
        T: std::str::FromStr + Clone,
        T::Err: fmt::Display,
    {
        Self::get_parsed(var_name).unwrap_or(default)
    }
}

/// 配置验证工具
pub struct ConfigValidator;

impl ConfigValidator {
    /// 验证URL格式
    pub fn validate_url<S: AsRef<str>>(url: S) -> QingxiResult<()> {
        let url_str = url.as_ref();
        if url_str.is_empty() {
            return Err(QingxiError::config("URL cannot be empty"));
        }
        
        if !url_str.starts_with("http://") && !url_str.starts_with("https://") && !url_str.starts_with("ws://") && !url_str.starts_with("wss://") {
            return Err(QingxiError::config(format!("Invalid URL format: {}", url_str)));
        }
        
        if url_str.contains("PLACEHOLDER") || url_str.contains("YOUR_") || url_str.contains("localhost") {
            tracing::warn!("⚠️ URL contains placeholder or localhost: {}", url_str);
        }
        
        Ok(())
    }
    
    /// 验证API密钥
    pub fn validate_api_key<S: AsRef<str>>(key: S, key_type: &str) -> QingxiResult<()> {
        let key_str = key.as_ref();
        if key_str.is_empty() {
            return Err(QingxiError::config(format!("{} cannot be empty", key_type)));
        }
        
        if key_str.contains("PLACEHOLDER") || key_str.contains("YOUR_") {
            return Err(QingxiError::config(format!("{} contains placeholder: {}", key_type, key_str)));
        }
        
        if key_str.len() < 10 {
            return Err(QingxiError::config(format!("{} appears too short: {}", key_type, key_str.len())));
        }
        
        Ok(())
    }
    
    /// 验证端口号
    pub fn validate_port(port: u16) -> QingxiResult<()> {
        if port == 0 {
            return Err(QingxiError::config("Port cannot be 0"));
        }
        
        if port < 1024 && port != 80 && port != 443 {
            tracing::warn!("⚠️ Using privileged port: {}", port);
        }
        
        Ok(())
    }
}

