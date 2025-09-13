//! 系统限制验证器 - 强制执行系统架构限制
//! 
//! 提供运行时动态检查，确保系统在预定义的限制范围内运行：
//! - 20交易所限制
//! - 50币种限制
//! - 优雅的错误处理和用户提示

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

/// 系统限制验证器
pub struct SystemLimitsValidator {
    /// 系统限制配置
    limits: SystemLimits,
    /// 当前活跃的交易所
    active_exchanges: Arc<RwLock<HashSet<String>>>,
    /// 当前活跃的交易对
    active_symbols: Arc<RwLock<HashMap<String, HashSet<String>>>>, // exchange -> symbols
    /// 运行时统计
    runtime_stats: Arc<RwLock<RuntimeStats>>,
    /// 违规记录
    violation_history: Arc<RwLock<Vec<LimitViolation>>>,
}

/// 系统限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLimits {
    /// 最大支持交易所数量
    pub max_supported_exchanges: usize,
    /// 最大支持交易对数量（全局）
    pub max_supported_symbols: usize,
    /// 单个交易所最大交易对数量
    pub max_symbols_per_exchange: usize,
    /// 最大并发套利机会
    pub max_concurrent_opportunities: usize,
    /// 最大订单批次大小
    pub max_order_batch_size: usize,
    /// 性能限制
    pub performance_limits: PerformanceLimits,
    /// 监控配置
    pub monitoring_config: MonitoringConfig,
}

/// 性能限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceLimits {
    /// 目标延迟（微秒）
    pub target_latency_microseconds: u64,
    /// 目标吞吐量（操作/秒）
    pub target_throughput_ops_per_sec: u64,
    /// 目标成功率（百分比）
    pub target_success_rate_percent: f64,
    /// 内存使用限制（MB）
    pub max_memory_usage_mb: u64,
    /// CPU使用率限制（百分比）
    pub max_cpu_usage_percent: f64,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 健康检查间隔（秒）
    pub health_check_interval_seconds: u64,
    /// API健康检查间隔（秒）
    pub api_health_check_interval_seconds: u64,
    /// 指标收集间隔（秒）
    pub metrics_collection_interval_seconds: u64,
    /// 违规报告间隔（秒）
    pub violation_report_interval_seconds: u64,
}

/// 运行时统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeStats {
    /// 当前交易所数量
    pub current_exchange_count: usize,
    /// 当前总交易对数量
    pub current_total_symbol_count: usize,
    /// 当前并发套利机会
    pub current_concurrent_opportunities: usize,
    /// 当前订单批次大小
    pub current_order_batch_size: usize,
    /// 系统启动时间
    pub system_start_time: u64,
    /// 最后更新时间
    pub last_updated: u64,
    /// 违规计数
    pub violation_count: u64,
}

/// 限制违规记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitViolation {
    /// 违规ID
    pub violation_id: String,
    /// 违规类型
    pub violation_type: ViolationType,
    /// 违规时间
    pub timestamp: u64,
    /// 当前值
    pub current_value: usize,
    /// 限制值
    pub limit_value: usize,
    /// 违规详情
    pub details: String,
    /// 影响级别
    pub severity: ViolationSeverity,
    /// 建议操作
    pub recommended_action: String,
}

/// 违规类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    ExchangeCountExceeded,
    SymbolCountExceeded,
    SymbolPerExchangeExceeded,
    ConcurrentOpportunitiesExceeded,
    OrderBatchSizeExceeded,
    PerformanceLimitBreached,
    ResourceLimitExceeded,
}

/// 违规严重性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,  // 系统必须停止
    High,      // 需要立即处理
    Medium,    // 需要关注
    Low,       // 监控即可
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub violations: Vec<LimitViolation>,
    pub recommendations: Vec<String>,
}

impl Default for SystemLimits {
    fn default() -> Self {
        Self {
            max_supported_exchanges: 20,
            max_supported_symbols: 50,
            max_symbols_per_exchange: 10,
            max_concurrent_opportunities: 1000,
            max_order_batch_size: 50,
            performance_limits: PerformanceLimits {
                target_latency_microseconds: 500,
                target_throughput_ops_per_sec: 10000,
                target_success_rate_percent: 99.9,
                max_memory_usage_mb: 8192,
                max_cpu_usage_percent: 80.0,
            },
            monitoring_config: MonitoringConfig {
                health_check_interval_seconds: 30,
                api_health_check_interval_seconds: 10,
                metrics_collection_interval_seconds: 5,
                violation_report_interval_seconds: 60,
            },
        }
    }
}

impl SystemLimitsValidator {
    /// 创建新的系统限制验证器
    #[instrument(skip(limits))]
    pub fn new(limits: SystemLimits) -> Self {
        info!("Initializing system limits validator with max {} exchanges, {} symbols", 
               limits.max_supported_exchanges, limits.max_supported_symbols);
        
        let validator = Self {
            limits,
            active_exchanges: Arc::new(RwLock::new(HashSet::new())),
            active_symbols: Arc::new(RwLock::new(HashMap::new())),
            runtime_stats: Arc::new(RwLock::new(RuntimeStats {
                system_start_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                ..Default::default()
            })),
            violation_history: Arc::new(RwLock::new(Vec::new())),
        };
        
        // 启动监控任务
        validator.start_monitoring_task();
        
        validator
    }
    
    /// 从配置文件加载限制
    pub async fn from_config_file(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await
            .with_context(|| format!("Failed to read system limits config from {}", path))?;
        let limits: SystemLimits = toml::from_str(&content)
            .with_context(|| "Failed to parse system limits config")?;
        Ok(Self::new(limits))
    }
    
    /// 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&self, exchange: &str) -> Result<ValidationResult> {
        let mut active_exchanges = self.active_exchanges.write().await;
        
        // 检查是否超过交易所限制
        if active_exchanges.len() >= self.limits.max_supported_exchanges {
            let violation = self.create_violation(
                ViolationType::ExchangeCountExceeded,
                active_exchanges.len() + 1,
                self.limits.max_supported_exchanges,
                format!("Attempting to register exchange '{}' would exceed the maximum limit", exchange),
                ViolationSeverity::Critical,
                "Remove unused exchanges or increase system capacity".to_string(),
            ).await;
            
            return Ok(ValidationResult {
                is_valid: false,
                violations: vec![violation],
                recommendations: vec![
                    "Remove unused exchanges before adding new ones".to_string(),
                    "Consider upgrading to a higher capacity system".to_string(),
                    "Optimize exchange selection based on trading volume".to_string(),
                ],
            });
        }
        
        // 注册交易所
        active_exchanges.insert(exchange.to_string());
        
        // 初始化交易对集合
        let mut active_symbols = self.active_symbols.write().await;
        active_symbols.entry(exchange.to_string()).or_insert_with(HashSet::new);
        
        // 更新统计信息
        self.update_runtime_stats().await;
        
        info!("Successfully registered exchange: {}", exchange);
        Ok(ValidationResult {
            is_valid: true,
            violations: Vec::new(),
            recommendations: Vec::new(),
        })
    }
    
    /// 注销交易所
    #[instrument(skip(self))]
    pub async fn unregister_exchange(&self, exchange: &str) -> Result<()> {
        let mut active_exchanges = self.active_exchanges.write().await;
        active_exchanges.remove(exchange);
        
        let mut active_symbols = self.active_symbols.write().await;
        active_symbols.remove(exchange);
        
        self.update_runtime_stats().await;
        info!("Successfully unregistered exchange: {}", exchange);
        Ok(())
    }
    
    /// 注册交易对
    #[instrument(skip(self))]
    pub async fn register_symbol(&self, exchange: &str, symbol: &str) -> Result<ValidationResult> {
        let mut violations = Vec::new();
        let mut recommendations = Vec::new();
        
        // 首先检查全局限制，避免借用冲突
        let total_symbols: usize = {
            let active_symbols_guard = self.active_symbols.read().await;
            active_symbols_guard.values().map(|s| s.len()).sum()
        };
        
        if total_symbols >= self.limits.max_supported_symbols {
            let violation = self.create_violation(
                ViolationType::SymbolCountExceeded,
                total_symbols + 1,
                self.limits.max_supported_symbols,
                format!("Adding symbol '{}' to exchange '{}' would exceed global symbol limit", symbol, exchange),
                ViolationSeverity::Critical,
                "Remove unused symbols from other exchanges".to_string(),
            ).await;
            violations.push(violation);
            recommendations.push("Implement dynamic symbol management".to_string());
        }
        
        // 然后获取可变引用检查和修改
        let mut active_symbols = self.active_symbols.write().await;
        let exchange_symbols = active_symbols.entry(exchange.to_string())
            .or_insert_with(HashSet::new);
        
        // 检查单个交易所的交易对限制
        let exchange_symbol_count = exchange_symbols.len();
        if exchange_symbol_count >= self.limits.max_symbols_per_exchange {
            let violation = self.create_violation(
                ViolationType::SymbolPerExchangeExceeded,
                exchange_symbol_count + 1,
                self.limits.max_symbols_per_exchange,
                format!("Exchange '{}' would exceed per-exchange symbol limit with '{}'", exchange, symbol),
                ViolationSeverity::High,
                format!("Remove unused symbols from exchange '{}'", exchange),
            ).await;
            violations.push(violation);
            recommendations.push(format!("Optimize symbol selection for exchange '{}'", exchange));
        }
        
        if !violations.is_empty() {
            return Ok(ValidationResult {
                is_valid: false,
                violations,
                recommendations,
            });
        }
        
        // 注册交易对
        exchange_symbols.insert(symbol.to_string());
        
        // 更新统计信息
        self.update_runtime_stats().await;
        
        debug!("Successfully registered symbol: {}:{}", exchange, symbol);
        Ok(ValidationResult {
            is_valid: true,
            violations: Vec::new(),
            recommendations: Vec::new(),
        })
    }
    
    /// 注销交易对
    #[instrument(skip(self))]
    pub async fn unregister_symbol(&self, exchange: &str, symbol: &str) -> Result<()> {
        let mut active_symbols = self.active_symbols.write().await;
        if let Some(exchange_symbols) = active_symbols.get_mut(exchange) {
            exchange_symbols.remove(symbol);
        }
        
        self.update_runtime_stats().await;
        debug!("Successfully unregistered symbol: {}:{}", exchange, symbol);
        Ok(())
    }
    
    /// 验证并发套利机会数量
    #[instrument(skip(self))]
    pub async fn validate_concurrent_opportunities(&self, count: usize) -> Result<ValidationResult> {
        if count > self.limits.max_concurrent_opportunities {
            let violation = self.create_violation(
                ViolationType::ConcurrentOpportunitiesExceeded,
                count,
                self.limits.max_concurrent_opportunities,
                format!("Concurrent opportunities count ({}) exceeds limit", count),
                ViolationSeverity::High,
                "Implement opportunity prioritization and throttling".to_string(),
            ).await;
            
            return Ok(ValidationResult {
                is_valid: false,
                violations: vec![violation],
                recommendations: vec![
                    "Implement opportunity queue management".to_string(),
                    "Prioritize high-profit opportunities".to_string(),
                    "Consider increasing system capacity".to_string(),
                ],
            });
        }
        
        // 更新统计信息
        {
            let mut stats = self.runtime_stats.write().await;
            stats.current_concurrent_opportunities = count;
            stats.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        Ok(ValidationResult {
            is_valid: true,
            violations: Vec::new(),
            recommendations: Vec::new(),
        })
    }
    
    /// 验证订单批次大小
    #[instrument(skip(self))]
    pub async fn validate_order_batch_size(&self, batch_size: usize) -> Result<ValidationResult> {
        if batch_size > self.limits.max_order_batch_size {
            let violation = self.create_violation(
                ViolationType::OrderBatchSizeExceeded,
                batch_size,
                self.limits.max_order_batch_size,
                format!("Order batch size ({}) exceeds limit", batch_size),
                ViolationSeverity::Medium,
                "Split large order batches into smaller chunks".to_string(),
            ).await;
            
            return Ok(ValidationResult {
                is_valid: false,
                violations: vec![violation],
                recommendations: vec![
                    "Implement order batching strategy".to_string(),
                    "Consider async order processing".to_string(),
                ],
            });
        }
        
        // 更新统计信息
        {
            let mut stats = self.runtime_stats.write().await;
            stats.current_order_batch_size = batch_size;
            stats.last_updated = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        Ok(ValidationResult {
            is_valid: true,
            violations: Vec::new(),
            recommendations: Vec::new(),
        })
    }
    
    /// 获取当前系统状态
    pub async fn get_system_status(&self) -> SystemStatus {
        let active_exchanges = self.active_exchanges.read().await;
        let active_symbols = self.active_symbols.read().await;
        let stats = self.runtime_stats.read().await;
        let violations = self.violation_history.read().await;
        
        let total_symbols: usize = active_symbols.values().map(|s| s.len()).sum();
        
        SystemStatus {
            current_exchanges: active_exchanges.iter().cloned().collect(),
            current_exchange_count: active_exchanges.len(),
            current_symbol_count: total_symbols,
            current_symbols_per_exchange: active_symbols
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
            limits: self.limits.clone(),
            runtime_stats: stats.clone(),
            recent_violations: violations
                .iter()
                .rev()
                .take(10)
                .cloned()
                .collect(),
            compliance_status: self.calculate_compliance_status(&*stats).await,
        }
    }
    
    /// 获取违规历史
    pub async fn get_violation_history(&self, limit: Option<usize>) -> Vec<LimitViolation> {
        let violations = self.violation_history.read().await;
        match limit {
            Some(n) => violations.iter().rev().take(n).cloned().collect(),
            None => violations.clone(),
        }
    }
    
    /// 清空违规历史
    pub async fn clear_violation_history(&self) {
        let mut violations = self.violation_history.write().await;
        violations.clear();
        info!("Cleared violation history");
    }
    
    /// 更新运行时统计信息
    async fn update_runtime_stats(&self) {
        let active_exchanges = self.active_exchanges.read().await;
        let active_symbols = self.active_symbols.read().await;
        let total_symbols: usize = active_symbols.values().map(|s| s.len()).sum();
        
        let mut stats = self.runtime_stats.write().await;
        stats.current_exchange_count = active_exchanges.len();
        stats.current_total_symbol_count = total_symbols;
        stats.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
    
    /// 创建违规记录
    async fn create_violation(
        &self,
        violation_type: ViolationType,
        current_value: usize,
        limit_value: usize,
        details: String,
        severity: ViolationSeverity,
        recommended_action: String,
    ) -> LimitViolation {
        let violation = LimitViolation {
            violation_id: uuid::Uuid::new_v4().to_string(),
            violation_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            current_value,
            limit_value,
            details,
            severity,
            recommended_action,
        };
        
        // 记录违规
        {
            let mut violations = self.violation_history.write().await;
            violations.push(violation.clone());
            
            // 保持历史记录在合理范围内
            if violations.len() > 1000 {
                violations.drain(0..100);
            }
        }
        
        // 更新违规计数
        {
            let mut stats = self.runtime_stats.write().await;
            stats.violation_count += 1;
        }
        
        // 记录日志
        match violation.severity {
            ViolationSeverity::Critical => {
                error!("CRITICAL LIMIT VIOLATION: {}", violation.details);
            }
            ViolationSeverity::High => {
                warn!("HIGH SEVERITY LIMIT VIOLATION: {}", violation.details);
            }
            ViolationSeverity::Medium => {
                warn!("MEDIUM SEVERITY LIMIT VIOLATION: {}", violation.details);
            }
            ViolationSeverity::Low => {
                debug!("LOW SEVERITY LIMIT VIOLATION: {}", violation.details);
            }
        }
        
        violation
    }
    
    /// 计算合规状态
    async fn calculate_compliance_status(&self, stats: &RuntimeStats) -> ComplianceStatus {
        let exchange_compliance = (stats.current_exchange_count as f64 / self.limits.max_supported_exchanges as f64) * 100.0;
        let symbol_compliance = (stats.current_total_symbol_count as f64 / self.limits.max_supported_symbols as f64) * 100.0;
        
        let overall_compliance = (exchange_compliance + symbol_compliance) / 2.0;
        
        ComplianceStatus {
            overall_compliance_percent: overall_compliance,
            exchange_usage_percent: exchange_compliance,
            symbol_usage_percent: symbol_compliance,
            is_compliant: exchange_compliance <= 100.0 && symbol_compliance <= 100.0,
            risk_level: match overall_compliance {
                p if p <= 70.0 => RiskLevel::Low,
                p if p <= 85.0 => RiskLevel::Medium,
                p if p <= 95.0 => RiskLevel::High,
                _ => RiskLevel::Critical,
            },
        }
    }
    
    /// 启动监控任务
    fn start_monitoring_task(&self) {
        let runtime_stats = self.runtime_stats.clone();
        let violation_history = self.violation_history.clone();
        let limits = self.limits.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(
                limits.monitoring_config.violation_report_interval_seconds
            ));
            
            loop {
                interval.tick().await;
                
                // 定期报告系统状态
                let stats = runtime_stats.read().await;
                let violations = violation_history.read().await;
                
                debug!(
                    "System status - Exchanges: {}/{}, Symbols: {}/{}, Violations: {}",
                    stats.current_exchange_count,
                    limits.max_supported_exchanges,
                    stats.current_total_symbol_count,
                    limits.max_supported_symbols,
                    violations.len()
                );
                
                // 检查是否有关键违规需要关注
                let critical_violations = violations
                    .iter()
                    .filter(|v| v.severity == ViolationSeverity::Critical)
                    .count();
                
                if critical_violations > 0 {
                    error!("System has {} critical limit violations", critical_violations);
                }
            }
        });
    }
}

/// 系统状态
#[derive(Debug, Clone, Serialize)]
pub struct SystemStatus {
    pub current_exchanges: Vec<String>,
    pub current_exchange_count: usize,
    pub current_symbol_count: usize,
    pub current_symbols_per_exchange: HashMap<String, usize>,
    pub limits: SystemLimits,
    pub runtime_stats: RuntimeStats,
    pub recent_violations: Vec<LimitViolation>,
    pub compliance_status: ComplianceStatus,
}

/// 合规状态
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceStatus {
    pub overall_compliance_percent: f64,
    pub exchange_usage_percent: f64,
    pub symbol_usage_percent: f64,
    pub is_compliant: bool,
    pub risk_level: RiskLevel,
}

/// 风险级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_exchange_limit_enforcement() {
        let limits = SystemLimits {
            max_supported_exchanges: 2,
            max_supported_symbols: 10,
            ..Default::default()
        };
        
        let validator = SystemLimitsValidator::new(limits);
        
        // 注册第一个交易所
        let result = validator.register_exchange("binance").await.unwrap();
        assert!(result.is_valid);
        
        // 注册第二个交易所
        let result = validator.register_exchange("okx").await.unwrap();
        assert!(result.is_valid);
        
        // 尝试注册第三个交易所，应该失败
        let result = validator.register_exchange("huobi").await.unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.violations.len(), 1);
        assert!(matches!(
            result.violations[0].violation_type,
            ViolationType::ExchangeCountExceeded
        ));
    }
    
    #[tokio::test]
    async fn test_symbol_limit_enforcement() {
        let limits = SystemLimits {
            max_supported_exchanges: 5,
            max_supported_symbols: 3,
            max_symbols_per_exchange: 2,
            ..Default::default()
        };
        
        let validator = SystemLimitsValidator::new(limits);
        
        // 注册交易所
        validator.register_exchange("binance").await.unwrap();
        
        // 注册交易对
        let result = validator.register_symbol("binance", "BTC/USDT").await.unwrap();
        assert!(result.is_valid);
        
        let result = validator.register_symbol("binance", "ETH/USDT").await.unwrap();
        assert!(result.is_valid);
        
        // 尝试注册第三个交易对到同一交易所，应该失败（per-exchange limit）
        let result = validator.register_symbol("binance", "ADA/USDT").await.unwrap();
        assert!(!result.is_valid);
        
        // 注册另一个交易所并添加交易对
        validator.register_exchange("okx").await.unwrap();
        let result = validator.register_symbol("okx", "BTC/USDT").await.unwrap();
        assert!(result.is_valid);
        
        // 现在应该达到全局符号限制
        let result = validator.register_symbol("okx", "ETH/USDT").await.unwrap();
        assert!(!result.is_valid); // 超过全局限制
    }
    
    #[tokio::test]
    async fn test_system_status_reporting() {
        let limits = SystemLimits::default();
        let validator = SystemLimitsValidator::new(limits);
        
        // 注册一些交易所和交易对
        validator.register_exchange("binance").await.unwrap();
        validator.register_symbol("binance", "BTC/USDT").await.unwrap();
        
        let status = validator.get_system_status().await;
        
        assert_eq!(status.current_exchange_count, 1);
        assert_eq!(status.current_symbol_count, 1);
        assert!(status.compliance_status.is_compliant);
        assert_eq!(status.compliance_status.risk_level, RiskLevel::Low);
    }
    
    #[tokio::test]
    async fn test_violation_history() {
        let limits = SystemLimits {
            max_supported_exchanges: 1,
            ..Default::default()
        };
        
        let validator = SystemLimitsValidator::new(limits);
        
        // 注册第一个交易所
        validator.register_exchange("binance").await.unwrap();
        
        // 尝试注册第二个交易所（应该失败）
        let result = validator.register_exchange("okx").await.unwrap();
        assert!(!result.is_valid);
        
        // 检查违规历史
        let violations = validator.get_violation_history(None).await;
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation_type, ViolationType::ExchangeCountExceeded);
        
        // 清空历史
        validator.clear_violation_history().await;
        let violations = validator.get_violation_history(None).await;
        assert_eq!(violations.len(), 0);
    }
}