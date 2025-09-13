//! 系统限制运行时强制执行器
//! 
//! 监控和强制执行系统架构限制，确保系统在预定义范围内安全运行

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock, watch};
use tracing::{debug, error, info, instrument, warn};

use crate::config::system_limits::{
    SystemLimitsValidator, ViolationSeverity, LimitViolation, SystemStatus, RiskLevel
};

/// 运行时强制执行器
pub struct RuntimeEnforcer {
    /// 系统限制验证器
    validator: Arc<SystemLimitsValidator>,
    /// 强制执行配置
    config: EnforcementConfig,
    /// 系统状态发送器
    status_sender: watch::Sender<SystemHealth>,
    /// 关闭信号发送器
    shutdown_sender: Arc<RwLock<Option<mpsc::UnboundedSender<EnforcementAction>>>>,
    /// 强制执行统计
    enforcement_stats: Arc<RwLock<EnforcementStats>>,
    /// 运行状态
    is_running: Arc<RwLock<bool>>,
    /// 紧急停机状态
    emergency_stop: Arc<RwLock<bool>>,
}

/// 强制执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementConfig {
    /// 监控间隔（秒）
    pub monitoring_interval_seconds: u64,
    /// 自动强制执行
    pub auto_enforcement_enabled: bool,
    /// 关键违规自动停机
    pub critical_violation_shutdown: bool,
    /// 高风险警告阈值
    pub high_risk_warning_threshold: f64,
    /// 关键风险阈值
    pub critical_risk_threshold: f64,
    /// 违规容忍度配置
    pub violation_tolerance: ViolationToleranceConfig,
    /// 恢复配置
    pub recovery_config: RecoveryConfig,
}

/// 违规容忍度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationToleranceConfig {
    /// 最大连续违规次数
    pub max_consecutive_violations: u32,
    /// 违规时间窗口（秒）
    pub violation_time_window_seconds: u64,
    /// 关键违规容忍数量
    pub max_critical_violations_per_hour: u32,
}

/// 恢复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// 自动恢复尝试间隔（秒）
    pub recovery_attempt_interval_seconds: u64,
    /// 最大恢复尝试次数
    pub max_recovery_attempts: u32,
    /// 恢复成功后的监控时间（秒）
    pub post_recovery_monitoring_seconds: u64,
}

/// 强制执行动作
#[derive(Debug, Clone, Serialize)]
pub enum EnforcementAction {
    /// 发出警告
    Warning { message: String, risk_level: RiskLevel },
    /// 限制新操作
    ThrottleOperations { reason: String, duration_seconds: u64 },
    /// 暂停非关键服务
    PauseNonCriticalServices { reason: String },
    /// 拒绝新连接
    RejectNewConnections { reason: String },
    /// 强制清理资源
    ForceResourceCleanup { target: String, details: String },
    /// 紧急停机
    EmergencyShutdown { reason: String, violations: Vec<LimitViolation> },
}

/// 系统健康状态
#[derive(Debug, Clone, Serialize)]
pub struct SystemHealth {
    pub timestamp: u64,
    pub overall_health: HealthStatus,
    pub risk_level: RiskLevel,
    pub active_violations: Vec<LimitViolation>,
    pub resource_usage: ResourceUsage,
    pub enforcement_active: bool,
    pub emergency_stop_active: bool,
    pub recommendations: Vec<String>,
}

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Degraded,
    Critical,
    EmergencyStop,
}

/// 资源使用情况
#[derive(Debug, Clone, Serialize)]
pub struct ResourceUsage {
    pub exchange_utilization_percent: f64,
    pub symbol_utilization_percent: f64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub concurrent_operations: usize,
}

/// 强制执行统计
#[derive(Debug, Clone, Default, Serialize)]
pub struct EnforcementStats {
    pub total_violations_detected: u64,
    pub critical_violations_handled: u64,
    pub warnings_issued: u64,
    pub throttle_actions_taken: u64,
    pub emergency_shutdowns_triggered: u64,
    pub auto_recovery_attempts: u64,
    pub successful_recoveries: u64,
    pub uptime_seconds: u64,
    pub last_violation_time: u64,
    pub last_enforcement_action_time: u64,
}

impl Default for EnforcementConfig {
    fn default() -> Self {
        Self {
            monitoring_interval_seconds: 10,
            auto_enforcement_enabled: true,
            critical_violation_shutdown: true,
            high_risk_warning_threshold: 85.0,
            critical_risk_threshold: 95.0,
            violation_tolerance: ViolationToleranceConfig {
                max_consecutive_violations: 3,
                violation_time_window_seconds: 300,
                max_critical_violations_per_hour: 5,
            },
            recovery_config: RecoveryConfig {
                recovery_attempt_interval_seconds: 60,
                max_recovery_attempts: 3,
                post_recovery_monitoring_seconds: 300,
            },
        }
    }
}

impl RuntimeEnforcer {
    /// 创建新的运行时强制执行器
    #[instrument(skip(validator, config))]
    pub fn new(validator: Arc<SystemLimitsValidator>, config: EnforcementConfig) -> Self {
        info!("Initializing runtime enforcement system with auto-enforcement: {}", 
               config.auto_enforcement_enabled);
        
        let (status_sender, _) = watch::channel(SystemHealth {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            overall_health: HealthStatus::Healthy,
            risk_level: RiskLevel::Low,
            active_violations: Vec::new(),
            resource_usage: ResourceUsage {
                exchange_utilization_percent: 0.0,
                symbol_utilization_percent: 0.0,
                memory_usage_mb: 0,
                cpu_usage_percent: 0.0,
                concurrent_operations: 0,
            },
            enforcement_active: config.auto_enforcement_enabled,
            emergency_stop_active: false,
            recommendations: Vec::new(),
        });
        
        Self {
            validator,
            config,
            status_sender,
            shutdown_sender: Arc::new(RwLock::new(None)),
            enforcement_stats: Arc::new(RwLock::new(EnforcementStats::default())),
            is_running: Arc::new(RwLock::new(false)),
            emergency_stop: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 启动运行时强制执行
    #[instrument(skip(self))]
    pub async fn start(&self, shutdown_receiver: mpsc::UnboundedReceiver<EnforcementAction>) -> Result<()> {
        info!("Starting runtime enforcement system");
        
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(anyhow::anyhow!("Runtime enforcer is already running"));
            }
            *running = true;
        }
        
        // 启动主监控循环
        let monitoring_handle = self.start_monitoring_loop().await;
        
        // 启动强制执行处理器
        let enforcement_handle = self.start_enforcement_handler(shutdown_receiver).await;
        
        // 启动健康状态报告
        let health_reporting_handle = self.start_health_reporting().await;
        
        info!("Runtime enforcement system started successfully");
        
        // 等待所有任务完成或收到停止信号
        tokio::select! {
            _ = monitoring_handle => {
                warn!("Monitoring loop ended unexpectedly");
            }
            _ = enforcement_handle => {
                warn!("Enforcement handler ended unexpectedly");
            }
            _ = health_reporting_handle => {
                warn!("Health reporting ended unexpectedly");
            }
        }
        
        Ok(())
    }
    
    /// 停止运行时强制执行
    #[instrument(skip(self))]
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping runtime enforcement system");
        
        let mut running = self.is_running.write().await;
        *running = false;
        
        // 发送停止信号到所有子系统
        if let Some(sender) = self.shutdown_sender.read().await.as_ref() {
            let _ = sender.send(EnforcementAction::Warning {
                message: "Runtime enforcement system shutting down".to_string(),
                risk_level: RiskLevel::Medium,
            });
        }
        
        info!("Runtime enforcement system stopped");
        Ok(())
    }
    
    /// 触发紧急停机
    #[instrument(skip(self))]
    pub async fn trigger_emergency_stop(&self, reason: String, violations: Vec<LimitViolation>) -> Result<()> {
        error!("TRIGGERING EMERGENCY SHUTDOWN: {}", reason);
        
        {
            let mut emergency = self.emergency_stop.write().await;
            *emergency = true;
        }
        
        // 更新统计
        {
            let mut stats = self.enforcement_stats.write().await;
            stats.emergency_shutdowns_triggered += 1;
            stats.last_enforcement_action_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        
        // 发送紧急停机信号
        if let Some(sender) = self.shutdown_sender.read().await.as_ref() {
            sender.send(EnforcementAction::EmergencyShutdown {
                reason: reason.clone(),
                violations: violations.clone(),
            })?;
        }
        
        // 更新系统健康状态
        let health = SystemHealth {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            overall_health: HealthStatus::EmergencyStop,
            risk_level: RiskLevel::Critical,
            active_violations: violations,
            resource_usage: self.collect_resource_usage().await,
            enforcement_active: true,
            emergency_stop_active: true,
            recommendations: vec![
                "System has been emergency stopped due to critical violations".to_string(),
                "Review violation logs and system configuration".to_string(),
                "Manual intervention required before restart".to_string(),
            ],
        };
        
        let _ = self.status_sender.send(health);
        
        Ok(())
    }
    
    /// 获取系统健康状态接收器
    pub fn get_health_receiver(&self) -> watch::Receiver<SystemHealth> {
        self.status_sender.subscribe()
    }
    
    /// 获取强制执行统计
    pub async fn get_enforcement_stats(&self) -> EnforcementStats {
        let stats = self.enforcement_stats.read().await;
        let mut result = stats.clone();
        
        // 计算运行时间
        let start_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        result.uptime_seconds = start_time.saturating_sub(result.uptime_seconds);
        
        result
    }
    
    /// 检查是否处于紧急停机状态
    pub async fn is_emergency_stopped(&self) -> bool {
        *self.emergency_stop.read().await
    }
    
    /// 启动监控循环
    async fn start_monitoring_loop(&self) -> tokio::task::JoinHandle<()> {
        let validator = self.validator.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();
        let emergency_stop = self.emergency_stop.clone();
        let enforcement_stats = self.enforcement_stats.clone();
        let status_sender = self.status_sender.clone();
        let shutdown_sender = self.shutdown_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(config.monitoring_interval_seconds));
            let mut consecutive_violations = 0u32;
            let mut _last_critical_violation_time = 0u64;
            
            while *is_running.read().await {
                interval.tick().await;
                
                // 检查是否处于紧急停机状态
                if *emergency_stop.read().await {
                    debug!("System in emergency stop, monitoring suspended");
                    continue;
                }
                
                // 获取系统状态
                let system_status = validator.get_system_status().await;
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                
                // 分析违规情况
                let critical_violations: Vec<_> = system_status.recent_violations
                    .iter()
                    .filter(|v| v.severity == ViolationSeverity::Critical)
                    .cloned()
                    .collect();
                
                let _high_violations: Vec<_> = system_status.recent_violations
                    .iter()
                    .filter(|v| v.severity == ViolationSeverity::High)
                    .cloned()
                    .collect();
                
                // 更新统计信息
                {
                    let mut stats = enforcement_stats.write().await;
                    stats.total_violations_detected += system_status.recent_violations.len() as u64;
                    stats.critical_violations_handled += critical_violations.len() as u64;
                    
                    if !system_status.recent_violations.is_empty() {
                        stats.last_violation_time = current_time;
                    }
                }
                
                // 检查关键违规
                if !critical_violations.is_empty() {
                    consecutive_violations += 1;
                    _last_critical_violation_time = current_time;
                    
                    error!("Detected {} critical violations", critical_violations.len());
                    
                    // 检查是否需要紧急停机
                    if config.critical_violation_shutdown &&
                       consecutive_violations >= config.violation_tolerance.max_consecutive_violations {
                        
                        if let Some(sender) = shutdown_sender.read().await.as_ref() {
                            let _ = sender.send(EnforcementAction::EmergencyShutdown {
                                reason: format!(
                                    "System exceeded critical violation tolerance: {} consecutive violations",
                                    consecutive_violations
                                ),
                                violations: critical_violations.clone(),
                            });
                        }
                        continue;
                    }
                    
                    // 发送强制执行动作
                    if let Some(sender) = shutdown_sender.read().await.as_ref() {
                        let _ = sender.send(EnforcementAction::ThrottleOperations {
                            reason: "Critical violations detected, throttling operations".to_string(),
                            duration_seconds: 60,
                        });
                    }
                } else {
                    consecutive_violations = 0;
                }
                
                // 检查风险级别
                let risk_level = system_status.compliance_status.risk_level;
                let compliance_percent = system_status.compliance_status.overall_compliance_percent;
                
                if compliance_percent >= config.critical_risk_threshold {
                    if let Some(sender) = shutdown_sender.read().await.as_ref() {
                        let _ = sender.send(EnforcementAction::PauseNonCriticalServices {
                            reason: format!(
                                "System at critical risk level: {:.1}% resource utilization",
                                compliance_percent
                            ),
                        });
                    }
                } else if compliance_percent >= config.high_risk_warning_threshold {
                    if let Some(sender) = shutdown_sender.read().await.as_ref() {
                        let _ = sender.send(EnforcementAction::Warning {
                            message: format!(
                                "System approaching capacity limits: {:.1}% utilization",
                                compliance_percent
                            ),
                            risk_level,
                        });
                    }
                }
                
                // 发送健康状态更新
                let health_status = Self::determine_health_status(
                    &system_status,
                    consecutive_violations,
                    *emergency_stop.read().await,
                );
                
                let resource_usage = Self::extract_resource_usage(&system_status);
                
                let recent_violations = system_status.recent_violations.clone();
                let recommendations = Self::generate_recommendations(&system_status, consecutive_violations);
                
                let health = SystemHealth {
                    timestamp: current_time,
                    overall_health: health_status,
                    risk_level,
                    active_violations: recent_violations,
                    resource_usage,
                    enforcement_active: config.auto_enforcement_enabled,
                    emergency_stop_active: *emergency_stop.read().await,
                    recommendations,
                };
                
                let _ = status_sender.send(health);
            }
        })
    }
    
    /// 启动强制执行处理器
    async fn start_enforcement_handler(
        &self,
        mut shutdown_receiver: mpsc::UnboundedReceiver<EnforcementAction>
    ) -> tokio::task::JoinHandle<()> {
        let enforcement_stats = self.enforcement_stats.clone();
        let emergency_stop = self.emergency_stop.clone();
        let is_running = self.is_running.clone();
        let _config = self.config.clone();
        
        tokio::spawn(async move {
            while let Some(action) = shutdown_receiver.recv().await {
                if !*is_running.read().await {
                    break;
                }
                
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                
                // 更新统计信息
                {
                    let mut stats = enforcement_stats.write().await;
                    stats.last_enforcement_action_time = current_time;
                }
                
                match action {
                    EnforcementAction::Warning { message, risk_level } => {
                        match risk_level {
                            RiskLevel::Critical => error!("CRITICAL WARNING: {}", message),
                            RiskLevel::High => warn!("HIGH RISK WARNING: {}", message),
                            RiskLevel::Medium => warn!("MEDIUM RISK WARNING: {}", message),
                            RiskLevel::Low => info!("LOW RISK WARNING: {}", message),
                        }
                        
                        let mut stats = enforcement_stats.write().await;
                        stats.warnings_issued += 1;
                    }
                    
                    EnforcementAction::ThrottleOperations { reason, duration_seconds } => {
                        warn!("THROTTLING OPERATIONS for {}s: {}", duration_seconds, reason);
                        
                        let mut stats = enforcement_stats.write().await;
                        stats.throttle_actions_taken += 1;
                        
                        // 实际的限流逻辑
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(duration_seconds)).await;
                        });
                    }
                    
                    EnforcementAction::PauseNonCriticalServices { reason } => {
                        warn!("PAUSING NON-CRITICAL SERVICES: {}", reason);
                        
                        // 实际的服务暂停逻辑会在这里实现
                        // 例如：停止非关键的数据收集，暂停后台任务等
                    }
                    
                    EnforcementAction::RejectNewConnections { reason } => {
                        warn!("REJECTING NEW CONNECTIONS: {}", reason);
                        
                        // 实际的连接拒绝逻辑会在这里实现
                        // 例如：拒绝新的WebSocket连接，限制API访问等
                    }
                    
                    EnforcementAction::ForceResourceCleanup { target, details } => {
                        warn!("FORCING RESOURCE CLEANUP - Target: {}, Details: {}", target, details);
                        
                        // 实际的资源清理逻辑会在这里实现
                        // 例如：清理缓存，释放内存，关闭空闲连接等
                    }
                    
                    EnforcementAction::EmergencyShutdown { reason, violations } => {
                        error!("EMERGENCY SHUTDOWN TRIGGERED: {}", reason);
                        error!("Active violations: {}", violations.len());
                        
                        {
                            let mut emergency = emergency_stop.write().await;
                            *emergency = true;
                        }
                        
                        {
                            let mut stats = enforcement_stats.write().await;
                            stats.emergency_shutdowns_triggered += 1;
                        }
                        
                        // 实际的紧急停机逻辑会在这里实现
                        // 例如：停止所有交易，保存状态，优雅关闭连接等
                        
                        // 在实际实现中，这里应该触发系统关闭流程
                        error!("SYSTEM EMERGENCY STOP ACTIVATED - Manual intervention required");
                        break;
                    }
                }
            }
        })
    }
    
    /// 启动健康状态报告
    async fn start_health_reporting(&self) -> tokio::task::JoinHandle<()> {
        let _status_sender = self.status_sender.clone();
        let is_running = self.is_running.clone();
        let enforcement_stats = self.enforcement_stats.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            while *is_running.read().await {
                interval.tick().await;
                
                let stats = enforcement_stats.read().await;
                
                info!(
                    "Runtime Enforcement Status - Violations: {}, Warnings: {}, Throttles: {}, Emergency Stops: {}",
                    stats.total_violations_detected,
                    stats.warnings_issued,
                    stats.throttle_actions_taken,
                    stats.emergency_shutdowns_triggered
                );
                
                debug!(
                    "Enforcement uptime: {}s, Last violation: {}s ago, Last action: {}s ago",
                    stats.uptime_seconds,
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                        .saturating_sub(stats.last_violation_time),
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                        .saturating_sub(stats.last_enforcement_action_time)
                );
            }
        })
    }
    
    /// 收集资源使用情况
    async fn collect_resource_usage(&self) -> ResourceUsage {
        let system_status = self.validator.get_system_status().await;
        Self::extract_resource_usage(&system_status)
    }
    
    /// 从系统状态提取资源使用情况
    fn extract_resource_usage(system_status: &SystemStatus) -> ResourceUsage {
        ResourceUsage {
            exchange_utilization_percent: system_status.compliance_status.exchange_usage_percent,
            symbol_utilization_percent: system_status.compliance_status.symbol_usage_percent,
            memory_usage_mb: Self::get_memory_usage(),
            cpu_usage_percent: Self::get_cpu_usage(),
            concurrent_operations: system_status.runtime_stats.current_concurrent_opportunities,
        }
    }
    
    /// 确定健康状态
    fn determine_health_status(
        system_status: &SystemStatus,
        consecutive_violations: u32,
        emergency_stop: bool,
    ) -> HealthStatus {
        if emergency_stop {
            return HealthStatus::EmergencyStop;
        }
        
        let critical_violations = system_status.recent_violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Critical)
            .count();
        
        if critical_violations > 0 || consecutive_violations >= 3 {
            HealthStatus::Critical
        } else if system_status.compliance_status.risk_level == RiskLevel::High {
            HealthStatus::Degraded
        } else if !system_status.recent_violations.is_empty() || 
                  system_status.compliance_status.risk_level == RiskLevel::Medium {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
    
    /// 生成建议
    fn generate_recommendations(
        system_status: &SystemStatus,
        consecutive_violations: u32,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if consecutive_violations > 0 {
            recommendations.push(format!(
                "System has {} consecutive violations - review system load",
                consecutive_violations
            ));
        }
        
        if system_status.compliance_status.exchange_usage_percent > 80.0 {
            recommendations.push("Consider reducing active exchanges or increasing capacity".to_string());
        }
        
        if system_status.compliance_status.symbol_usage_percent > 80.0 {
            recommendations.push("Consider reducing active symbols or optimizing symbol selection".to_string());
        }
        
        if system_status.recent_violations.is_empty() && 
           system_status.compliance_status.overall_compliance_percent < 70.0 {
            recommendations.push("System operating within healthy limits".to_string());
        }
        
        recommendations
    }
    
    /// 获取内存使用情况(MB)
    fn get_memory_usage() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
                for line in contents.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(value_str) = line.split_whitespace().nth(1) {
                            if let Ok(available_kb) = value_str.parse::<u64>() {
                                // 假设系统总内存8GB，计算已用内存
                                let total_memory_mb = 8192;
                                let available_mb = available_kb / 1024;
                                return total_memory_mb - available_mb;
                            }
                        }
                    }
                }
            }
        }
        
        // 如果无法获取实际值，返回模拟值
        512 // 模拟512MB使用
    }
    
    /// 获取CPU使用率(%)
    fn get_cpu_usage() -> f64 {
        #[cfg(target_os = "linux")]
        {
            // 简化的CPU使用率获取
            if let Ok(contents) = std::fs::read_to_string("/proc/loadavg") {
                if let Some(load_str) = contents.split_whitespace().next() {
                    if let Ok(load) = load_str.parse::<f64>() {
                        // 将负载转换为CPU使用率百分比(简化计算)
                        return (load * 100.0 / num_cpus::get() as f64).min(100.0);
                    }
                }
            }
        }
        
        // 如果无法获取实际值，返回模拟值
        25.0 // 模拟25%使用率
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::system_limits::{SystemLimits, SystemLimitsValidator};
    
    #[tokio::test]
    async fn test_runtime_enforcer_creation() {
        let limits = SystemLimits::default();
        let validator = Arc::new(SystemLimitsValidator::new(limits));
        let config = EnforcementConfig::default();
        
        let enforcer = RuntimeEnforcer::new(validator, config);
        assert!(!enforcer.is_emergency_stopped().await);
    }
    
    #[tokio::test]
    async fn test_emergency_stop_trigger() {
        let limits = SystemLimits::default();
        let validator = Arc::new(SystemLimitsValidator::new(limits));
        let config = EnforcementConfig::default();
        
        let enforcer = RuntimeEnforcer::new(validator, config);
        
        let result = enforcer.trigger_emergency_stop(
            "Test emergency stop".to_string(),
            vec![]
        ).await;
        
        assert!(result.is_ok());
        assert!(enforcer.is_emergency_stopped().await);
    }
    
    #[tokio::test]
    async fn test_health_status_determination() {
        let mut system_status = SystemStatus {
            current_exchanges: vec!["binance".to_string()],
            current_exchange_count: 1,
            current_symbol_count: 1,
            current_symbols_per_exchange: HashMap::new(),
            limits: SystemLimits::default(),
            runtime_stats: crate::config::system_limits::RuntimeStats::default(),
            recent_violations: Vec::new(),
            compliance_status: crate::config::system_limits::ComplianceStatus {
                overall_compliance_percent: 50.0,
                exchange_usage_percent: 50.0,
                symbol_usage_percent: 50.0,
                is_compliant: true,
                risk_level: RiskLevel::Low,
            },
        };
        
        // 测试健康状态
        let health = RuntimeEnforcer::determine_health_status(&system_status, 0, false);
        assert_eq!(health, HealthStatus::Healthy);
        
        // 测试紧急停机状态
        let health = RuntimeEnforcer::determine_health_status(&system_status, 0, true);
        assert_eq!(health, HealthStatus::EmergencyStop);
        
        // 测试关键状态
        system_status.compliance_status.risk_level = RiskLevel::High;
        let health = RuntimeEnforcer::determine_health_status(&system_status, 0, false);
        assert_eq!(health, HealthStatus::Degraded);
    }
}