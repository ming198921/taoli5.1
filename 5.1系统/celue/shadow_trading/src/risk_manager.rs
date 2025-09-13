//! 风险管理模块

use crate::config::RiskLimitsConfig;
use crate::virtual_account::VirtualAccount;
use crate::execution_engine::ShadowOrder;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 风险违规类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskViolationType {
    DailyLossExceeded,
    TotalLossExceeded,
    PositionConcentrationExceeded,
    OrderValueExceeded,
    VarLimitExceeded,
    LeverageExceeded,
    MarginCallRequired,
    InsufficientBalance,
}

/// 风险违规记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskViolation {
    pub violation_type: RiskViolationType,
    pub current_value: f64,
    pub limit_value: f64,
    pub description: String,
    pub severity: RiskSeverity,
    pub timestamp: DateTime<Utc>,
}

/// 风险严重性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 风险限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub max_daily_loss: f64,
    pub max_total_loss: f64,
    pub max_position_size: f64,
    pub max_order_value: f64,
    pub max_leverage: f64,
    pub var_limit: f64,
    pub margin_requirement: f64,
}

/// 风险管理器
pub struct ShadowRiskManager {
    config: RiskLimitsConfig,
    risk_limits: RiskLimits,
}

impl ShadowRiskManager {
    pub fn new(config: RiskLimitsConfig) -> Result<Self> {
        let risk_limits = RiskLimits {
            max_daily_loss: config.max_daily_loss_pct,
            max_total_loss: config.max_total_loss_pct,
            max_position_size: config.max_position_concentration_pct,
            max_order_value: config.max_order_value,
            max_leverage: 10.0, // 默认最大杠杆
            var_limit: config.max_var_limit,
            margin_requirement: 0.1, // 10% 保证金要求
        };

        Ok(Self {
            config,
            risk_limits,
        })
    }

    /// 验证订单是否符合风险限制
    pub async fn validate_order(&self, order: &ShadowOrder) -> Result<()> {
        let order_value = order.quantity * order.price.unwrap_or(50000.0); // 使用默认价格

        // 检查订单价值限制
        if order_value > self.config.max_order_value {
            return Err(anyhow::anyhow!(
                "Order value {} exceeds maximum limit {}",
                order_value,
                self.config.max_order_value
            ));
        }

        // 其他订单验证...
        Ok(())
    }

    /// 检查账户风险
    pub async fn check_account_risk(&self, account: &VirtualAccount) -> Result<Vec<RiskViolation>, Vec<RiskViolation>> {
        let mut violations = Vec::new();

        // 计算当前账户净值
        let current_net_value = self.calculate_account_net_value(account);
        let initial_net_value = self.calculate_initial_net_value(account);

        // 检查日损失限制
        let daily_loss_pct = self.calculate_daily_loss_percentage(account, current_net_value);
        if daily_loss_pct > self.config.max_daily_loss_pct {
            violations.push(RiskViolation {
                violation_type: RiskViolationType::DailyLossExceeded,
                current_value: daily_loss_pct,
                limit_value: self.config.max_daily_loss_pct,
                description: format!(
                    "Daily loss {:.2}% exceeds limit {:.2}%",
                    daily_loss_pct * 100.0,
                    self.config.max_daily_loss_pct * 100.0
                ),
                severity: RiskSeverity::High,
                timestamp: Utc::now(),
            });
        }

        // 检查总损失限制
        let total_loss_pct = (initial_net_value - current_net_value) / initial_net_value;
        if total_loss_pct > self.config.max_total_loss_pct {
            violations.push(RiskViolation {
                violation_type: RiskViolationType::TotalLossExceeded,
                current_value: total_loss_pct,
                limit_value: self.config.max_total_loss_pct,
                description: format!(
                    "Total loss {:.2}% exceeds limit {:.2}%",
                    total_loss_pct * 100.0,
                    self.config.max_total_loss_pct * 100.0
                ),
                severity: RiskSeverity::Critical,
                timestamp: Utc::now(),
            });
        }

        // 检查持仓集中度
        let max_position_concentration = self.calculate_max_position_concentration(account, current_net_value);
        if max_position_concentration > self.config.max_position_concentration_pct {
            violations.push(RiskViolation {
                violation_type: RiskViolationType::PositionConcentrationExceeded,
                current_value: max_position_concentration,
                limit_value: self.config.max_position_concentration_pct,
                description: format!(
                    "Position concentration {:.2}% exceeds limit {:.2}%",
                    max_position_concentration * 100.0,
                    self.config.max_position_concentration_pct * 100.0
                ),
                severity: RiskSeverity::Medium,
                timestamp: Utc::now(),
            });
        }

        // 检查保证金要求
        let (used_margin, free_margin) = account.get_margin_usage();
        let margin_ratio = if used_margin + free_margin > 0.0 {
            used_margin / (used_margin + free_margin)
        } else {
            0.0
        };

        if margin_ratio > 0.8 { // 80% 保证金使用率警告
            violations.push(RiskViolation {
                violation_type: RiskViolationType::MarginCallRequired,
                current_value: margin_ratio,
                limit_value: 0.8,
                description: format!(
                    "Margin usage {:.2}% approaching limit",
                    margin_ratio * 100.0
                ),
                severity: if margin_ratio > 0.95 { RiskSeverity::Critical } else { RiskSeverity::High },
                timestamp: Utc::now(),
            });
        }

        // 检查余额充足性
        for (currency, balance) in &account.balances {
            if balance.available < 0.0 {
                violations.push(RiskViolation {
                    violation_type: RiskViolationType::InsufficientBalance,
                    current_value: balance.available,
                    limit_value: 0.0,
                    description: format!(
                        "Insufficient {} balance: {}",
                        currency, balance.available
                    ),
                    severity: RiskSeverity::Critical,
                    timestamp: Utc::now(),
                });
            }
        }

        if violations.is_empty() {
            Ok(violations)
        } else {
            Err(violations)
        }
    }

    /// 计算VaR
    pub async fn calculate_var(
        &self,
        account: &VirtualAccount,
        confidence_level: f64,
        time_horizon_days: u32,
    ) -> Result<f64> {
        // 简化的VaR计算，实际应该基于历史模拟或参数模型
        let portfolio_value = self.calculate_account_net_value(account);
        let daily_volatility = 0.02; // 假设2%的日波动率
        
        // 正态分布假设下的VaR
        let z_score = match confidence_level {
            0.90 => 1.28,
            0.95 => 1.645,
            0.99 => 2.33,
            _ => 1.645,
        };
        
        let time_adjusted_volatility = daily_volatility * (time_horizon_days as f64).sqrt();
        let var = portfolio_value * z_score * time_adjusted_volatility;
        
        Ok(var)
    }

    /// 计算预期亏损（Expected Shortfall）
    pub async fn calculate_expected_shortfall(
        &self,
        account: &VirtualAccount,
        confidence_level: f64,
    ) -> Result<f64> {
        let var = self.calculate_var(account, confidence_level, 1).await?;
        // 简化计算：ES ≈ VaR * 1.3 （对于正态分布）
        Ok(var * 1.3)
    }

    /// 检查订单是否会导致风险违规
    pub async fn check_order_risk_impact(
        &self,
        account: &VirtualAccount,
        order: &ShadowOrder,
    ) -> Result<Vec<RiskViolation>> {
        let mut potential_violations = Vec::new();
        
        // 模拟订单执行后的账户状态
        let order_value = order.quantity * order.price.unwrap_or(50000.0);
        
        // 检查订单价值限制
        if order_value > self.config.max_order_value {
            potential_violations.push(RiskViolation {
                violation_type: RiskViolationType::OrderValueExceeded,
                current_value: order_value,
                limit_value: self.config.max_order_value,
                description: format!(
                    "Order value {} exceeds maximum limit {}",
                    order_value, self.config.max_order_value
                ),
                severity: RiskSeverity::High,
                timestamp: Utc::now(),
            });
        }

        // 检查执行后的持仓集中度
        let current_net_value = self.calculate_account_net_value(account);
        let new_position_value = self.calculate_new_position_value(account, order);
        let new_concentration = new_position_value / current_net_value;
        
        if new_concentration > self.config.max_position_concentration_pct {
            potential_violations.push(RiskViolation {
                violation_type: RiskViolationType::PositionConcentrationExceeded,
                current_value: new_concentration,
                limit_value: self.config.max_position_concentration_pct,
                description: format!(
                    "New position would create concentration of {:.2}%",
                    new_concentration * 100.0
                ),
                severity: RiskSeverity::Medium,
                timestamp: Utc::now(),
            });
        }

        Ok(potential_violations)
    }

    /// 获取风险建议
    pub async fn get_risk_recommendations(&self, account: &VirtualAccount) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        let current_net_value = self.calculate_account_net_value(account);
        let max_position_concentration = self.calculate_max_position_concentration(account, current_net_value);
        
        if max_position_concentration > 0.5 {
            recommendations.push(
                "考虑分散持仓以降低集中度风险".to_string()
            );
        }

        let (used_margin, free_margin) = account.get_margin_usage();
        let margin_ratio = used_margin / (used_margin + free_margin);
        
        if margin_ratio > 0.7 {
            recommendations.push(
                "保证金使用率较高，建议减少杠杆或增加保证金".to_string()
            );
        }

        if account.positions.len() > 20 {
            recommendations.push(
                "持仓数量较多，建议合并相似策略或减少持仓".to_string()
            );
        }

        if recommendations.is_empty() {
            recommendations.push("当前风险状况良好".to_string());
        }

        recommendations
    }

    // 私有辅助方法

    fn calculate_account_net_value(&self, account: &VirtualAccount) -> f64 {
        let mut total_value = 0.0;
        
        // 计算现金余额
        for balance in account.balances.values() {
            total_value += balance.total();
        }
        
        // 计算持仓价值（包括未实现盈亏）
        for position in account.positions.values() {
            total_value += position.unrealized_pnl;
        }
        
        total_value
    }

    fn calculate_initial_net_value(&self, account: &VirtualAccount) -> f64 {
        account.initial_balances.values().sum()
    }

    fn calculate_daily_loss_percentage(&self, account: &VirtualAccount, current_value: f64) -> f64 {
        // 简化计算，实际应该跟踪每日开始时的净值
        let initial_value = self.calculate_initial_net_value(account);
        let assumed_daily_start_value = initial_value * 0.98; // 假设从98%开始
        
        if assumed_daily_start_value > 0.0 {
            (assumed_daily_start_value - current_value) / assumed_daily_start_value
        } else {
            0.0
        }
    }

    fn calculate_max_position_concentration(&self, account: &VirtualAccount, net_value: f64) -> f64 {
        if net_value <= 0.0 {
            return 0.0;
        }

        let mut max_concentration = 0.0;
        
        for position in account.positions.values() {
            let position_value = position.quantity.abs() * position.average_price;
            let concentration = position_value / net_value;
            max_concentration = max_concentration.max(concentration);
        }
        
        max_concentration
    }

    fn calculate_new_position_value(&self, account: &VirtualAccount, order: &ShadowOrder) -> f64 {
        // 计算如果执行该订单后新持仓的价值
        if let Some(existing_position) = account.positions.get(&order.symbol) {
            let new_quantity = existing_position.quantity + match order.side {
                crate::execution_engine::OrderSide::Buy => order.quantity,
                crate::execution_engine::OrderSide::Sell => -order.quantity,
            };
            new_quantity.abs() * order.price.unwrap_or(existing_position.average_price)
        } else {
            order.quantity * order.price.unwrap_or(50000.0)
        }
    }

    /// 更新风险限制
    pub fn update_risk_limits(&mut self, new_limits: RiskLimits) {
        self.risk_limits = new_limits;
    }

    /// 获取当前风险限制
    pub fn get_risk_limits(&self) -> &RiskLimits {
        &self.risk_limits
    }

    /// 计算风险调整收益率
    pub fn calculate_risk_adjusted_return(&self, return_rate: f64, volatility: f64, risk_free_rate: f64) -> f64 {
        if volatility != 0.0 {
            (return_rate - risk_free_rate) / volatility
        } else {
            0.0
        }
    }

    /// 计算最优仓位大小（Kelly公式）
    pub fn calculate_kelly_position_size(&self, win_rate: f64, avg_win: f64, avg_loss: f64) -> f64 {
        if avg_loss == 0.0 {
            return 0.0;
        }
        
        let win_probability = win_rate;
        let loss_probability = 1.0 - win_rate;
        let win_loss_ratio = avg_win / avg_loss;
        
        // Kelly公式：f = (bp - q) / b
        // 其中 f = 最优仓位比例，b = 盈亏比，p = 胜率，q = 败率
        let kelly_fraction = (win_loss_ratio * win_probability - loss_probability) / win_loss_ratio;
        
        // 限制最大仓位为25%（Kelly的1/4）
        kelly_fraction.max(0.0).min(0.25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VirtualAccountConfig;
    use crate::virtual_account::VirtualAccount;
    use std::collections::HashMap;

    fn create_test_account() -> VirtualAccount {
        let mut initial_balances = HashMap::new();
        initial_balances.insert("USDT".to_string(), 100000.0);
        
        let config = VirtualAccountConfig::default();
        VirtualAccount::new("test_account".to_string(), initial_balances, config).unwrap()
    }

    #[tokio::test]
    async fn test_risk_manager_creation() {
        let config = RiskLimitsConfig::default();
        let risk_manager = ShadowRiskManager::new(config);
        assert!(risk_manager.is_ok());
    }

    #[tokio::test]
    async fn test_account_risk_check() {
        let config = RiskLimitsConfig::default();
        let risk_manager = ShadowRiskManager::new(config).unwrap();
        let account = create_test_account();
        
        let result = risk_manager.check_account_risk(&account).await;
        // 新账户应该没有风险违规
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_var_calculation() {
        let config = RiskLimitsConfig::default();
        let risk_manager = ShadowRiskManager::new(config).unwrap();
        let account = create_test_account();
        
        let var = risk_manager.calculate_var(&account, 0.95, 1).await;
        assert!(var.is_ok());
        assert!(var.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_kelly_position_calculation() {
        let config = RiskLimitsConfig::default();
        let risk_manager = ShadowRiskManager::new(config).unwrap();
        
        // 测试有利的参数
        let position_size = risk_manager.calculate_kelly_position_size(0.6, 100.0, 50.0);
        assert!(position_size > 0.0);
        assert!(position_size <= 0.25); // 应该不超过25%
        
        // 测试不利的参数
        let position_size = risk_manager.calculate_kelly_position_size(0.3, 50.0, 100.0);
        assert_eq!(position_size, 0.0); // 期望值为负，应该返回0
    }

    #[test]
    fn test_risk_adjusted_return() {
        let config = RiskLimitsConfig::default();
        let risk_manager = ShadowRiskManager::new(config).unwrap();
        
        let sharpe_ratio = risk_manager.calculate_risk_adjusted_return(0.1, 0.15, 0.02);
        assert!((sharpe_ratio - 0.533333).abs() < 0.001);
    }
}