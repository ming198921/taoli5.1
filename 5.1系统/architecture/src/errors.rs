//! 错误类型定义模块
//! 
//! 定义系统所有可能的错误类型，支持错误链和上下文信息

use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 系统主错误类型
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SystemError {
    #[error("配置错误: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("网络连接错误: {0}")]
    Network(#[from] NetworkError),
    
    #[error("API错误: {0}")]
    Api(#[from] ApiError),
    
    #[error("数据库错误: {0}")]
    Database(#[from] DatabaseError),
    
    #[error("策略错误: {0}")]
    Strategy(#[from] StrategyError),
    
    #[error("风险管理错误: {0}")]
    Risk(#[from] RiskError),
    
    #[error("执行错误: {0}")]
    Execution(#[from] ExecutionError),
    
    #[error("资金管理错误: {0}")]
    Fund(#[from] FundError),
    
    #[error("数据处理错误: {0}")]
    Data(#[from] DataError),
    
    #[error("监控错误: {0}")]
    Monitoring(#[from] MonitoringError),
    
    #[error("不支持的交易所: {0}")]
    UnsupportedExchange(String),
    
    #[error("系统未就绪")]
    SystemNotReady,
    
    #[error("系统关闭中")]
    SystemShuttingDown,
    
    #[error("资源不足")]
    InsufficientResources,
    
    #[error("流动性不足")]
    InsufficientLiquidity,
    
    #[error("超时: {operation} 在 {timeout_ms}ms 内未完成")]
    Timeout { operation: String, timeout_ms: u64 },
    
    #[error("并发限制: {resource} 达到最大并发数 {limit}")]
    ConcurrencyLimit { resource: String, limit: usize },
    
    #[error("内部错误: {0}")]
    Internal(String),
    
    #[error("IO错误: {0}")]
    Io(String),
    
    #[error("其他错误: {0}")]
    Other(String),
    
    #[error("配置加载错误: {0}")]
    ConfigLoad(String),
    
    #[error("适配器错误: {0}")]
    Adapter(String),
}

/// 配置相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ConfigError {
    #[error("配置文件不存在: {path}")]
    FileNotFound { path: String },
    
    #[error("配置解析失败: {reason}")]
    ParseError { reason: String },
    
    #[error("配置验证失败: {field}: {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("必需的配置项缺失: {key}")]
    MissingRequiredKey { key: String },
    
    #[error("配置值无效: {key} = {value}, 原因: {reason}")]
    InvalidValue { key: String, value: String, reason: String },
    
    #[error("热重载失败: {reason}")]
    HotReloadFailed { reason: String },
}

/// 网络相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum NetworkError {
    #[error("连接失败: {endpoint}")]
    ConnectionFailed { endpoint: String },
    
    #[error("连接超时: {endpoint} 在 {timeout_ms}ms 内未响应")]
    ConnectionTimeout { endpoint: String, timeout_ms: u64 },
    
    #[error("SSL/TLS错误: {reason}")]
    TlsError { reason: String },
    
    #[error("DNS解析失败: {domain}")]
    DnsResolutionFailed { domain: String },
    
    #[error("代理错误: {proxy_url}")]
    ProxyError { proxy_url: String },
    
    #[error("网络不可达: {target}")]
    NetworkUnreachable { target: String },
}

/// API相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ApiError {
    #[error("认证失败: {exchange}")]
    AuthenticationFailed { exchange: String },
    
    #[error("权限不足: 操作 {operation} 需要权限 {required_permission}")]
    InsufficientPermissions { operation: String, required_permission: String },
    
    #[error("API限流: {exchange} 限制 {limit_type}, 重试时间 {retry_after_ms}ms")]
    RateLimited { exchange: String, limit_type: String, retry_after_ms: u64 },
    
    #[error("API配额耗尽: {exchange} {quota_type}")]
    QuotaExhausted { exchange: String, quota_type: String },
    
    #[error("请求格式错误: {reason}")]
    BadRequest { reason: String },
    
    #[error("服务器错误: {exchange} 返回 {status_code}: {message}")]
    ServerError { exchange: String, status_code: u16, message: String },
    
    #[error("API版本不兼容: {exchange} 需要版本 {required_version}")]
    VersionMismatch { exchange: String, required_version: String },
    
    #[error("WebSocket连接错误: {reason}")]
    WebSocketError { reason: String },
    
    #[error("数据格式错误: 期望 {expected}, 得到 {actual}")]
    DataFormatError { expected: String, actual: String },
}

/// 数据库相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseError {
    #[error("连接池错误: {reason}")]
    ConnectionPoolError { reason: String },
    
    #[error("查询失败: {query}: {reason}")]
    QueryFailed { query: String, reason: String },
    
    #[error("事务失败: {reason}")]
    TransactionFailed { reason: String },
    
    #[error("数据冲突: {table} 中的 {key}")]
    DataConflict { table: String, key: String },
    
    #[error("约束违反: {constraint} 在表 {table}")]
    ConstraintViolation { constraint: String, table: String },
    
    #[error("序列化失败: {data_type}: {reason}")]
    SerializationError { data_type: String, reason: String },
    
    #[error("迁移失败: 版本 {from} -> {to}: {reason}")]
    MigrationError { from: String, to: String, reason: String },
}

/// 策略相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum StrategyError {
    #[error("策略未找到: {strategy_id}")]
    StrategyNotFound { strategy_id: String },
    
    #[error("策略验证失败: {strategy_id}: {reason}")]
    ValidationFailed { strategy_id: String, reason: String },
    
    #[error("策略执行失败: {strategy_id}: {reason}")]
    ExecutionFailed { strategy_id: String, reason: String },
    
    #[error("机会检测失败: {reason}")]
    OpportunityDetectionFailed { reason: String },
    
    #[error("参数无效: {parameter} = {value}")]
    InvalidParameter { parameter: String, value: String },
    
    #[error("策略配置错误: {reason}")]
    ConfigurationError { reason: String },
    
    #[error("机器学习模型错误: {model_type}: {reason}")]
    MLModelError { model_type: String, reason: String },
}

/// 风险管理相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum RiskError {
    #[error("风险评估失败: {reason}")]
    AssessmentFailed { reason: String },
    
    #[error("风险阈值超限: {risk_type} {current} > {limit}")]
    ThresholdExceeded { risk_type: String, current: f64, limit: f64 },
    
    #[error("仓位限制: {symbol} 当前 {current} 超过限制 {limit}")]
    PositionLimit { symbol: String, current: f64, limit: f64 },
    
    #[error("集中度风险: {risk_type} 集中度 {concentration}% 超过限制 {limit}%")]
    ConcentrationRisk { risk_type: String, concentration: f64, limit: f64 },
    
    #[error("相关性风险: 相关系数 {correlation} 超过限制 {limit}")]
    CorrelationRisk { correlation: f64, limit: f64 },
    
    #[error("VaR超限: {confidence_level}% VaR {current} 超过限制 {limit}")]
    VarExceeded { confidence_level: f64, current: f64, limit: f64 },
    
    #[error("极端市场条件检测: {market_condition}")]
    ExtremeMarketCondition { market_condition: String },
}

/// 执行相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionError {
    #[error("订单创建失败: {reason}")]
    OrderCreationFailed { reason: String },
    
    #[error("订单执行失败: {order_id}: {reason}")]
    OrderExecutionFailed { order_id: String, reason: String },
    
    #[error("订单取消失败: {order_id}: {reason}")]
    OrderCancellationFailed { order_id: String, reason: String },
    
    #[error("部分成交: {order_id} 成交 {filled} / {total}")]
    PartialFill { order_id: String, filled: f64, total: f64 },
    
    #[error("滑点过大: 期望 {expected}, 实际 {actual}, 滑点 {slippage}%")]
    ExcessiveSlippage { expected: f64, actual: f64, slippage: f64 },
    
    #[error("路由失败: 无法为 {symbol} 找到最优执行路径")]
    RoutingFailed { symbol: String },
    
    #[error("执行超时: {order_id} 在 {timeout_ms}ms 内未完成")]
    ExecutionTimeout { order_id: String, timeout_ms: u64 },
    
    #[error("原子性失败: 批量订单 {batch_id} 无法保证原子执行")]
    AtomicityFailed { batch_id: String },
}

/// 资金管理相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum FundError {
    #[error("余额不足: {asset} 需要 {required}, 可用 {available}")]
    InsufficientBalance { asset: String, required: f64, available: f64 },
    
    #[error("资金冻结: {asset} 金额 {amount} 冻结失败")]
    FreezeFailed { asset: String, amount: f64 },
    
    #[error("资金解冻失败: {asset} 金额 {amount}")]
    UnfreezeFailed { asset: String, amount: f64 },
    
    #[error("转账失败: {from_exchange} -> {to_exchange}, {asset}: {amount}")]
    TransferFailed { from_exchange: String, to_exchange: String, asset: String, amount: f64 },
    
    #[error("资金分配失败: {reason}")]
    AllocationFailed { reason: String },
    
    #[error("重平衡失败: {reason}")]
    RebalanceFailed { reason: String },
    
    #[error("保证金不足: {exchange} 需要 {required}, 可用 {available}")]
    InsufficientMargin { exchange: String, required: f64, available: f64 },
}

/// 数据处理相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum DataError {
    #[error("数据获取失败: {data_source}: {reason}")]
    FetchFailed { data_source: String, reason: String },
    
    #[error("数据验证失败: {data_type}: {reason}")]
    ValidationFailed { data_type: String, reason: String },
    
    #[error("数据清洗失败: {field}: {reason}")]
    CleaningFailed { field: String, reason: String },
    
    #[error("数据格式不匹配: 期望 {expected}, 实际 {actual}")]
    FormatMismatch { expected: String, actual: String },
    
    #[error("数据过期: {data_type} 时间戳 {timestamp}, 最大延迟 {max_age_ms}ms")]
    DataExpired { data_type: String, timestamp: i64, max_age_ms: u64 },
    
    #[error("数据不一致: {source1} vs {source2}: {discrepancy}")]
    DataInconsistency { source1: String, source2: String, discrepancy: String },
    
    #[error("缓存错误: {operation}: {reason}")]
    CacheError { operation: String, reason: String },
}

/// 监控相关错误
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringError {
    #[error("指标收集失败: {metric}: {reason}")]
    MetricsCollectionFailed { metric: String, reason: String },
    
    #[error("告警发送失败: {channel}: {reason}")]
    AlertSendFailed { channel: String, reason: String },
    
    #[error("健康检查失败: {component}: {reason}")]
    HealthCheckFailed { component: String, reason: String },
    
    #[error("追踪上下文丢失: {trace_id}")]
    TracingContextLost { trace_id: String },
    
    #[error("日志写入失败: {destination}: {reason}")]
    LogWriteFailed { destination: String, reason: String },
}

/// 错误严重性级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// 信息性错误，不影响系统运行
    Info,
    /// 警告性错误，可能影响性能
    Warning,
    /// 严重错误，影响功能
    Error,
    /// 致命错误，需要立即处理
    Critical,
    /// 系统级错误，可能导致系统停机
    Fatal,
}

/// 错误上下文信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: ErrorSeverity,
    pub component: String,
    pub operation: String,
    pub trace_id: Option<String>,
    pub user_id: Option<String>,
    pub additional_info: std::collections::HashMap<String, serde_json::Value>,
}

impl ErrorContext {
    pub fn new(severity: ErrorSeverity, component: &str, operation: &str) -> Self {
        Self {
            error_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            severity,
            component: component.to_string(),
            operation: operation.to_string(),
            trace_id: None,
            user_id: None,
            additional_info: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_trace_id(mut self, trace_id: &str) -> Self {
        self.trace_id = Some(trace_id.to_string());
        self
    }
    
    pub fn with_user_id(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    pub fn with_info<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.additional_info.insert(key.to_string(), json_value);
        }
        self
    }
}

/// 系统错误扩展特征
pub trait SystemErrorExt {
    fn with_context(self, context: ErrorContext) -> SystemError;
    fn get_severity(&self) -> ErrorSeverity;
    fn is_recoverable(&self) -> bool;
    fn should_retry(&self) -> bool;
    fn get_retry_delay_ms(&self) -> Option<u64>;
}

impl SystemErrorExt for SystemError {
    fn with_context(self, _context: ErrorContext) -> SystemError {
        // 在实际实现中，这里会将上下文信息附加到错误中
        self
    }
    
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            SystemError::SystemNotReady | SystemError::SystemShuttingDown => ErrorSeverity::Warning,
            SystemError::InsufficientResources | SystemError::InsufficientLiquidity => ErrorSeverity::Error,
            SystemError::ConcurrencyLimit { .. } => ErrorSeverity::Warning,
            SystemError::Timeout { .. } => ErrorSeverity::Warning,
            SystemError::Configuration(_) => ErrorSeverity::Error,
            SystemError::Database(_) => ErrorSeverity::Critical,
            SystemError::Network(_) => ErrorSeverity::Warning,
            SystemError::Api(api_err) => match api_err {
                ApiError::RateLimited { .. } => ErrorSeverity::Warning,
                ApiError::AuthenticationFailed { .. } => ErrorSeverity::Critical,
                _ => ErrorSeverity::Error,
            },
            SystemError::Internal(_) => ErrorSeverity::Fatal,
            _ => ErrorSeverity::Error,
        }
    }
    
    fn is_recoverable(&self) -> bool {
        match self {
            SystemError::SystemShuttingDown | SystemError::Internal(_) => false,
            SystemError::Configuration(ConfigError::FileNotFound { .. }) => false,
            SystemError::Api(ApiError::AuthenticationFailed { .. }) => false,
            _ => true,
        }
    }
    
    fn should_retry(&self) -> bool {
        match self {
            SystemError::Network(_) => true,
            SystemError::Api(ApiError::RateLimited { .. }) => true,
            SystemError::Api(ApiError::ServerError { status_code, .. }) => *status_code >= 500,
            SystemError::Timeout { .. } => true,
            SystemError::Database(DatabaseError::ConnectionPoolError { .. }) => true,
            _ => false,
        }
    }
    
    fn get_retry_delay_ms(&self) -> Option<u64> {
        match self {
            SystemError::Api(ApiError::RateLimited { retry_after_ms, .. }) => Some(*retry_after_ms),
            SystemError::Network(_) => Some(1000), // 1秒
            SystemError::Timeout { .. } => Some(500), // 500毫秒
            SystemError::Database(_) => Some(2000), // 2秒
            _ => None,
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, SystemError>;

/// 异步结果类型别名
pub type AsyncResult<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>;

impl From<std::io::Error> for SystemError {
    fn from(err: std::io::Error) -> Self {
        SystemError::Io(err.to_string())
    }
}

impl From<config::ConfigError> for SystemError {
    fn from(err: config::ConfigError) -> Self {
        SystemError::ConfigLoad(err.to_string())
    }
}


impl From<anyhow::Error> for SystemError {
    fn from(err: anyhow::Error) -> Self {
        SystemError::Other(err.to_string())
    }
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "INFO"),
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
            ErrorSeverity::Fatal => write!(f, "FATAL"),
        }
    }
} 