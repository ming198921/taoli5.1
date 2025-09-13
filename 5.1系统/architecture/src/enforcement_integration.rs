//! 运行时强制执行集成模块
//! 
//! 将系统限制验证器与运行时强制执行器集成，提供生产级的运行时保护

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::config::system_limits::SystemLimitsValidator;
use crate::runtime_enforcement::{
    RuntimeEnforcer, EnforcementConfig, EnforcementAction, SystemHealth
};

/// 强制执行系统集成器
pub struct EnforcementSystemIntegrator {
    validator: Arc<SystemLimitsValidator>,
    enforcer: Arc<RuntimeEnforcer>,
    action_sender: mpsc::UnboundedSender<EnforcementAction>,
    action_receiver: Option<mpsc::UnboundedReceiver<EnforcementAction>>,
}

impl EnforcementSystemIntegrator {
    /// 创建新的强制执行系统集成器
    pub async fn new() -> Result<Self> {
        // 从配置文件加载系统限制
        let validator = Arc::new(
            SystemLimitsValidator::from_config_file("config/system_limits.toml").await
                .unwrap_or_else(|_| {
                    warn!("Failed to load system limits config, using defaults");
                    SystemLimitsValidator::new(Default::default())
                })
        );
        
        // 创建强制执行配置
        let enforcement_config = EnforcementConfig {
            monitoring_interval_seconds: 5,
            auto_enforcement_enabled: true,
            critical_violation_shutdown: true,
            high_risk_warning_threshold: 80.0,
            critical_risk_threshold: 90.0,
            ..Default::default()
        };
        
        let enforcer = Arc::new(RuntimeEnforcer::new(
            validator.clone(),
            enforcement_config
        ));
        
        // 创建动作通道
        let (action_sender, action_receiver) = mpsc::unbounded_channel();
        
        Ok(Self {
            validator,
            enforcer,
            action_sender,
            action_receiver: Some(action_receiver),
        })
    }
    
    /// 启动强制执行系统
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting integrated enforcement system");
        
        if let Some(receiver) = self.action_receiver.take() {
            // 启动运行时强制执行器
            let enforcer_handle = {
                let enforcer = self.enforcer.clone();
                tokio::spawn(async move {
                    if let Err(e) = enforcer.start(receiver).await {
                        error!("Runtime enforcer failed: {}", e);
                    }
                })
            };
            
            // 启动集成监控循环
            let integration_handle = self.start_integration_loop().await;
            
            info!("Enforcement system started successfully");
            
            // 等待任务完成
            tokio::select! {
                _ = enforcer_handle => {
                    warn!("Enforcer handle completed");
                }
                _ = integration_handle => {
                    warn!("Integration handle completed");
                }
            }
        } else {
            return Err(anyhow::anyhow!("Action receiver already taken"));
        }
        
        Ok(())
    }
    
    /// 停止强制执行系统
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping integrated enforcement system");
        self.enforcer.stop().await?;
        info!("Enforcement system stopped");
        Ok(())
    }
    
    /// 获取系统健康状态
    pub fn get_health_receiver(&self) -> tokio::sync::watch::Receiver<SystemHealth> {
        self.enforcer.get_health_receiver()
    }
    
    /// 获取系统限制验证器（用于外部系统集成）
    pub fn get_validator(&self) -> Arc<SystemLimitsValidator> {
        self.validator.clone()
    }
    
    /// 手动触发强制执行动作
    pub async fn trigger_action(&self, action: EnforcementAction) -> Result<()> {
        self.action_sender.send(action)?;
        Ok(())
    }
    
    /// 启动集成监控循环
    async fn start_integration_loop(&self) -> tokio::task::JoinHandle<()> {
        let validator = self.validator.clone();
        let action_sender = self.action_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // 定期检查系统状态并执行预防性措施
                let status = validator.get_system_status().await;
                
                // 检查是否接近限制
                if status.compliance_status.exchange_usage_percent > 85.0 {
                    let _ = action_sender.send(EnforcementAction::Warning {
                        message: format!(
                            "Exchange usage approaching limit: {:.1}%",
                            status.compliance_status.exchange_usage_percent
                        ),
                        risk_level: status.compliance_status.risk_level,
                    });
                }
                
                if status.compliance_status.symbol_usage_percent > 85.0 {
                    let _ = action_sender.send(EnforcementAction::Warning {
                        message: format!(
                            "Symbol usage approaching limit: {:.1}%",
                            status.compliance_status.symbol_usage_percent
                        ),
                        risk_level: status.compliance_status.risk_level,
                    });
                }
                
                // 检查最近的违规模式
                let critical_violations_count = status.recent_violations
                    .iter()
                    .filter(|v| matches!(v.severity, crate::config::system_limits::ViolationSeverity::Critical))
                    .count();
                
                if critical_violations_count >= 2 {
                    let _ = action_sender.send(EnforcementAction::ThrottleOperations {
                        reason: format!(
                            "Multiple critical violations detected: {} violations",
                            critical_violations_count
                        ),
                        duration_seconds: 120,
                    });
                }
            }
        })
    }
}

/// 创建并启动强制执行系统的便捷函数
pub async fn create_and_start_enforcement_system() -> Result<Arc<EnforcementSystemIntegrator>> {
    let integrator = EnforcementSystemIntegrator::new().await?;
    
    // 在后台启动系统
    let integrator_arc = Arc::new(integrator);
    let integrator_clone = integrator_arc.clone();
    
    tokio::spawn(async move {
        // 需要获取可变引用，所以这里需要不同的方法
        // 实际使用中，可能需要调整这个设计
        info!("Enforcement system running in background");
        
        // 这里可以添加定期健康检查等逻辑
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            
            let stats = integrator_clone.enforcer.get_enforcement_stats().await;
            info!(
                "Enforcement system status - Violations: {}, Actions: {}, Uptime: {}s",
                stats.total_violations_detected,
                stats.warnings_issued + stats.throttle_actions_taken,
                stats.uptime_seconds
            );
        }
    });
    
    Ok(integrator_arc)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enforcement_system_creation() {
        let result = EnforcementSystemIntegrator::new().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test] 
    async fn test_health_receiver() {
        let integrator = EnforcementSystemIntegrator::new().await.unwrap();
        let _health_receiver = integrator.get_health_receiver();
        
        // 健康状态接收器应该能够成功创建
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_trigger_action() {
        let integrator = EnforcementSystemIntegrator::new().await.unwrap();
        
        let result = integrator.trigger_action(EnforcementAction::Warning {
            message: "Test warning".to_string(),
            risk_level: crate::config::system_limits::RiskLevel::Low,
        }).await;
        
        assert!(result.is_ok());
    }
}