//! 虚拟账户管理模块

use crate::config::VirtualAccountConfig;
use crate::market_simulator::MarketSimulator;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 虚拟账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualAccount {
    /// 账户ID
    pub id: String,
    /// 账户余额
    pub balances: HashMap<String, VirtualBalance>,
    /// 持仓
    pub positions: HashMap<String, VirtualPosition>,
    /// 账户配置
    pub config: VirtualAccountConfig,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 初始余额（用于重置）
    pub initial_balances: HashMap<String, f64>,
    /// 账户状态
    pub status: AccountStatus,
    /// 统计信息
    pub stats: AccountStats,
}

/// 虚拟余额
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualBalance {
    /// 货币符号
    pub currency: String,
    /// 可用余额
    pub available: f64,
    /// 冻结余额
    pub frozen: f64,
    /// 借贷余额（保证金交易）
    pub borrowed: f64,
    /// 利息累积
    pub interest_accrued: f64,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl VirtualBalance {
    pub fn new(currency: String, amount: f64) -> Self {
        Self {
            currency,
            available: amount,
            frozen: 0.0,
            borrowed: 0.0,
            interest_accrued: 0.0,
            updated_at: Utc::now(),
        }
    }

    /// 获取总余额
    pub fn total(&self) -> f64 {
        self.available + self.frozen - self.borrowed
    }

    /// 冻结资金
    pub fn freeze(&mut self, amount: f64) -> Result<()> {
        if self.available < amount {
            return Err(anyhow::anyhow!("Insufficient available balance"));
        }
        self.available -= amount;
        self.frozen += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 解冻资金
    pub fn unfreeze(&mut self, amount: f64) -> Result<()> {
        if self.frozen < amount {
            return Err(anyhow::anyhow!("Insufficient frozen balance"));
        }
        self.frozen -= amount;
        self.available += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 扣除资金
    pub fn deduct(&mut self, amount: f64) -> Result<()> {
        if self.frozen < amount {
            return Err(anyhow::anyhow!("Insufficient frozen balance to deduct"));
        }
        self.frozen -= amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 增加资金
    pub fn credit(&mut self, amount: f64) {
        self.available += amount;
        self.updated_at = Utc::now();
    }

    /// 借贷资金（保证金交易）
    pub fn borrow(&mut self, amount: f64) -> Result<()> {
        self.borrowed += amount;
        self.available += amount;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// 还贷
    pub fn repay(&mut self, amount: f64) -> Result<()> {
        if self.available < amount {
            return Err(anyhow::anyhow!("Insufficient available balance to repay"));
        }
        let repay_amount = amount.min(self.borrowed);
        self.available -= repay_amount;
        self.borrowed -= repay_amount;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// 虚拟持仓
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualPosition {
    /// 交易对
    pub symbol: String,
    /// 持仓数量（正数为多头，负数为空头）
    pub quantity: f64,
    /// 平均成本价
    pub average_price: f64,
    /// 未实现盈亏
    pub unrealized_pnl: f64,
    /// 已实现盈亏
    pub realized_pnl: f64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 持仓方向
    pub side: PositionSide,
    /// 杠杆倍数
    pub leverage: f64,
    /// 保证金
    pub margin: f64,
    /// 强平价格
    pub liquidation_price: Option<f64>,
}

impl VirtualPosition {
    pub fn new(symbol: String, quantity: f64, price: f64, leverage: f64) -> Self {
        let side = if quantity > 0.0 { PositionSide::Long } else { PositionSide::Short };
        let margin = (quantity.abs() * price) / leverage;
        
        Self {
            symbol,
            quantity,
            average_price: price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            side,
            leverage,
            margin,
            liquidation_price: None,
        }
    }

    /// 更新持仓
    pub fn update(&mut self, quantity_change: f64, price: f64) {
        if self.quantity == 0.0 {
            // 新建持仓
            self.quantity = quantity_change;
            self.average_price = price;
            self.side = if quantity_change > 0.0 { PositionSide::Long } else { PositionSide::Short };
        } else if (self.quantity > 0.0 && quantity_change > 0.0) || 
                  (self.quantity < 0.0 && quantity_change < 0.0) {
            // 增加持仓
            let total_value = self.quantity * self.average_price + quantity_change * price;
            self.quantity += quantity_change;
            if self.quantity != 0.0 {
                self.average_price = total_value / self.quantity;
            }
        } else {
            // 减少持仓或反向
            let close_quantity = quantity_change.abs().min(self.quantity.abs());
            let close_pnl = if self.quantity > 0.0 {
                close_quantity * (price - self.average_price)
            } else {
                close_quantity * (self.average_price - price)
            };
            
            self.realized_pnl += close_pnl;
            self.quantity += quantity_change;
            
            if self.quantity.abs() < 1e-8 {
                self.quantity = 0.0;
            }
        }
        
        self.updated_at = Utc::now();
    }

    /// 计算未实现盈亏
    pub fn calculate_unrealized_pnl(&mut self, current_price: f64) {
        if self.quantity != 0.0 {
            self.unrealized_pnl = if self.quantity > 0.0 {
                self.quantity * (current_price - self.average_price)
            } else {
                self.quantity.abs() * (self.average_price - current_price)
            };
        } else {
            self.unrealized_pnl = 0.0;
        }
        self.updated_at = Utc::now();
    }

    /// 计算强平价格
    pub fn calculate_liquidation_price(&mut self, account_balance: f64, maintenance_margin_rate: f64) {
        if self.quantity == 0.0 || self.leverage == 1.0 {
            self.liquidation_price = None;
            return;
        }

        let position_value = self.quantity.abs() * self.average_price;
        let initial_margin = position_value / self.leverage;
        let maintenance_margin = position_value * maintenance_margin_rate;
        
        let available_loss = account_balance + self.realized_pnl - maintenance_margin;
        
        if self.quantity > 0.0 {
            // 多头持仓
            self.liquidation_price = Some((self.average_price * self.quantity - available_loss) / self.quantity);
        } else {
            // 空头持仓
            self.liquidation_price = Some((self.average_price * self.quantity.abs() + available_loss) / self.quantity.abs());
        }
    }
}

/// 持仓方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionSide {
    Long,
    Short,
}

/// 账户状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    Active,
    Suspended,
    Liquidating,
    Closed,
}

/// 账户统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStats {
    /// 总交易次数
    pub total_trades: u64,
    /// 盈利交易次数
    pub winning_trades: u64,
    /// 亏损交易次数
    pub losing_trades: u64,
    /// 总盈亏
    pub total_pnl: f64,
    /// 最大盈利
    pub max_profit: f64,
    /// 最大亏损
    pub max_loss: f64,
    /// 最大回撤
    pub max_drawdown: f64,
    /// 总手续费
    pub total_fees: f64,
    /// 最后交易时间
    pub last_trade_at: Option<DateTime<Utc>>,
}

impl Default for AccountStats {
    fn default() -> Self {
        Self {
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            total_pnl: 0.0,
            max_profit: 0.0,
            max_loss: 0.0,
            max_drawdown: 0.0,
            total_fees: 0.0,
            last_trade_at: None,
        }
    }
}

impl AccountStats {
    /// 更新交易统计
    pub fn update_trade_stats(&mut self, pnl: f64, fees: f64) {
        self.total_trades += 1;
        self.total_pnl += pnl;
        self.total_fees += fees;
        
        if pnl > 0.0 {
            self.winning_trades += 1;
            if pnl > self.max_profit {
                self.max_profit = pnl;
            }
        } else if pnl < 0.0 {
            self.losing_trades += 1;
            if pnl < self.max_loss {
                self.max_loss = pnl;
            }
        }
        
        self.last_trade_at = Some(Utc::now());
    }

    /// 获取胜率
    pub fn win_rate(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            self.winning_trades as f64 / self.total_trades as f64
        }
    }

    /// 获取平均盈亏
    pub fn average_pnl(&self) -> f64 {
        if self.total_trades == 0 {
            0.0
        } else {
            self.total_pnl / self.total_trades as f64
        }
    }

    /// 获取盈亏比
    pub fn profit_factor(&self) -> f64 {
        if self.losing_trades == 0 || self.max_loss == 0.0 {
            f64::INFINITY
        } else {
            let avg_profit = if self.winning_trades > 0 {
                self.max_profit / self.winning_trades as f64
            } else {
                0.0
            };
            let avg_loss = self.max_loss.abs() / self.losing_trades as f64;
            avg_profit / avg_loss
        }
    }
}

impl VirtualAccount {
    /// 创建新的虚拟账户
    pub fn new(
        id: String,
        initial_balances: HashMap<String, f64>,
        config: VirtualAccountConfig,
    ) -> Result<Self> {
        let mut balances = HashMap::new();
        
        for (currency, amount) in &initial_balances {
            balances.insert(currency.clone(), VirtualBalance::new(currency.clone(), *amount));
        }

        Ok(Self {
            id,
            balances,
            positions: HashMap::new(),
            config,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            initial_balances,
            status: AccountStatus::Active,
            stats: AccountStats::default(),
        })
    }

    /// 获取账户总价值（以基础货币计算）
    pub async fn get_total_value(&self, market_simulator: &MarketSimulator) -> Result<f64> {
        let mut total_value = 0.0;

        // 计算余额价值
        for (currency, balance) in &self.balances {
            if currency == &self.config.base_currency {
                total_value += balance.total();
            } else {
                let symbol = format!("{}/{}", currency, self.config.base_currency);
                if let Ok(price) = market_simulator.get_current_price(&symbol).await {
                    total_value += balance.total() * price;
                } else {
                    // 如果无法获取价格，尝试反向交易对
                    let reverse_symbol = format!("{}/{}", self.config.base_currency, currency);
                    if let Ok(price) = market_simulator.get_current_price(&reverse_symbol).await {
                        total_value += balance.total() / price;
                    }
                    // 如果仍无法获取价格，则忽略该货币
                }
            }
        }

        // 计算持仓价值
        for position in self.positions.values() {
            let current_price = market_simulator.get_current_price(&position.symbol).await?;
            let position_value = position.quantity * current_price;
            total_value += position_value;
        }

        Ok(total_value)
    }

    /// 获取可用余额
    pub fn get_available_balance(&self, currency: &str) -> f64 {
        self.balances
            .get(currency)
            .map(|b| b.available)
            .unwrap_or(0.0)
    }

    /// 获取总余额
    pub fn get_total_balance(&self, currency: &str) -> f64 {
        self.balances
            .get(currency)
            .map(|b| b.total())
            .unwrap_or(0.0)
    }

    /// 冻结资金
    pub fn freeze_balance(&mut self, currency: &str, amount: f64) -> Result<()> {
        let balance = self.balances
            .get_mut(currency)
            .ok_or_else(|| anyhow::anyhow!("Currency {} not found", currency))?;
        
        balance.freeze(amount)?;
        self.updated_at = Utc::now();
        
        debug!(
            account_id = %self.id,
            currency = currency,
            amount = amount,
            "Frozen balance"
        );
        
        Ok(())
    }

    /// 解冻资金
    pub fn unfreeze_balance(&mut self, currency: &str, amount: f64) -> Result<()> {
        let balance = self.balances
            .get_mut(currency)
            .ok_or_else(|| anyhow::anyhow!("Currency {} not found", currency))?;
        
        balance.unfreeze(amount)?;
        self.updated_at = Utc::now();
        
        debug!(
            account_id = %self.id,
            currency = currency,
            amount = amount,
            "Unfrozen balance"
        );
        
        Ok(())
    }

    /// 执行交易
    pub fn execute_trade(
        &mut self,
        symbol: &str,
        quantity: f64,
        price: f64,
        fees: f64,
    ) -> Result<()> {
        let (base_currency, quote_currency) = self.parse_symbol(symbol)?;
        
        // 计算交易金额
        let trade_value = quantity.abs() * price;
        
        if quantity > 0.0 {
            // 买入：使用计价货币买入基础货币
            let total_cost = trade_value + fees;
            
            // 检查并扣除计价货币
            {
                let quote_balance = self.balances
                    .get_mut(&quote_currency)
                    .ok_or_else(|| anyhow::anyhow!("Insufficient {} balance", quote_currency))?;
                quote_balance.deduct(total_cost)?;
            }
            
            // 增加基础货币
            self.balances
                .entry(base_currency.clone())
                .or_insert_with(|| VirtualBalance::new(base_currency.clone(), 0.0))
                .credit(quantity);
        } else {
            // 卖出：使用基础货币换取计价货币
            let sell_quantity = quantity.abs();
            
            // 检查并扣除基础货币
            {
                let base_balance = self.balances
                    .get_mut(&base_currency)
                    .ok_or_else(|| anyhow::anyhow!("Insufficient {} balance", base_currency))?;
                base_balance.deduct(sell_quantity)?;
            }
            
            // 增加计价货币（扣除手续费）
            let received_amount = trade_value - fees;
            self.balances
                .entry(quote_currency.clone())
                .or_insert_with(|| VirtualBalance::new(quote_currency.clone(), 0.0))
                .credit(received_amount);
        }

        // 更新或创建持仓
        self.positions
            .entry(symbol.to_string())
            .or_insert_with(|| VirtualPosition::new(symbol.to_string(), 0.0, price, 1.0))
            .update(quantity, price);

        // 更新统计信息
        let pnl = if let Some(position) = self.positions.get(symbol) {
            position.realized_pnl
        } else {
            0.0
        };
        self.stats.update_trade_stats(pnl, fees);
        
        self.updated_at = Utc::now();
        
        info!(
            account_id = %self.id,
            symbol = symbol,
            quantity = quantity,
            price = price,
            fees = fees,
            "Executed trade"
        );

        Ok(())
    }

    /// 更新所有持仓的未实现盈亏
    pub async fn update_positions(&mut self, market_simulator: &Arc<MarketSimulator>) -> Result<()> {
        for position in self.positions.values_mut() {
            if position.quantity != 0.0 {
                let current_price = market_simulator.get_current_price(&position.symbol).await?;
                position.calculate_unrealized_pnl(current_price);
            }
        }
        Ok(())
    }

    /// 计算账户的总未实现盈亏
    pub async fn calculate_unrealized_pnl(&mut self, market_simulator: &Arc<MarketSimulator>) -> Result<f64> {
        let mut total_unrealized_pnl = 0.0;
        
        for position in self.positions.values_mut() {
            if position.quantity != 0.0 {
                let current_price = market_simulator.get_current_price(&position.symbol).await?;
                position.calculate_unrealized_pnl(current_price);
                total_unrealized_pnl += position.unrealized_pnl;
            }
        }
        
        Ok(total_unrealized_pnl)
    }

    /// 检查是否需要强制平仓
    pub async fn check_liquidation(&self, market_simulator: &Arc<MarketSimulator>) -> Result<Vec<String>> {
        let mut liquidation_positions = Vec::new();
        
        for position in self.positions.values() {
            if let Some(liquidation_price) = position.liquidation_price {
                let current_price = market_simulator.get_current_price(&position.symbol).await?;
                
                let should_liquidate = if position.quantity > 0.0 {
                    current_price <= liquidation_price
                } else {
                    current_price >= liquidation_price
                };
                
                if should_liquidate {
                    liquidation_positions.push(position.symbol.clone());
                }
            }
        }
        
        Ok(liquidation_positions)
    }

    /// 重置账户到初始状态
    pub fn reset_to_initial_state(&mut self) -> Result<()> {
        // 重置余额
        self.balances.clear();
        for (currency, amount) in &self.initial_balances {
            self.balances.insert(
                currency.clone(),
                VirtualBalance::new(currency.clone(), *amount)
            );
        }
        
        // 清空持仓
        self.positions.clear();
        
        // 重置统计信息
        self.stats = AccountStats::default();
        
        // 重置状态
        self.status = AccountStatus::Active;
        self.updated_at = Utc::now();
        
        info!(account_id = %self.id, "Account reset to initial state");
        Ok(())
    }

    /// 获取保证金使用情况
    pub fn get_margin_usage(&self) -> (f64, f64) {
        let mut used_margin = 0.0;
        let mut free_margin = 0.0;
        
        for position in self.positions.values() {
            used_margin += position.margin;
        }
        
        if let Some(base_balance) = self.balances.get(&self.config.base_currency) {
            free_margin = base_balance.available;
        }
        
        (used_margin, free_margin)
    }

    /// 解析交易对符号
    fn parse_symbol(&self, symbol: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = symbol.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid symbol format: {}", symbol));
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// 检查订单是否可以执行
    pub fn can_execute_order(&self, symbol: &str, quantity: f64, price: f64) -> Result<bool> {
        let (base_currency, quote_currency) = self.parse_symbol(symbol)?;
        
        if quantity > 0.0 {
            // 买入订单：检查计价货币余额
            let required_amount = quantity * price * 1.001; // 包含手续费缓冲
            let available = self.get_available_balance(&quote_currency);
            Ok(available >= required_amount)
        } else {
            // 卖出订单：检查基础货币余额
            let required_amount = quantity.abs();
            let available = self.get_available_balance(&base_currency);
            Ok(available >= required_amount)
        }
    }

    /// 获取账户净值
    pub async fn get_net_worth(&self, market_simulator: &Arc<MarketSimulator>) -> Result<f64> {
        let mut net_worth = 0.0;
        
        // 计算余额净值
        for (currency, balance) in &self.balances {
            if currency == &self.config.base_currency {
                net_worth += balance.total();
            } else {
                // 尝试获取汇率
                let symbols = [
                    format!("{}/{}", currency, self.config.base_currency),
                    format!("{}/{}", self.config.base_currency, currency),
                ];
                
                for symbol in &symbols {
                    if let Ok(price) = market_simulator.get_current_price(symbol).await {
                        if symbol.starts_with(currency) {
                            net_worth += balance.total() * price;
                        } else {
                            net_worth += balance.total() / price;
                        }
                        break;
                    }
                }
            }
        }
        
        // 计算持仓净值
        for position in self.positions.values() {
            net_worth += position.unrealized_pnl;
        }
        
        Ok(net_worth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VirtualAccountConfig;

    #[test]
    fn test_virtual_balance_operations() {
        let mut balance = VirtualBalance::new("USDT".to_string(), 1000.0);
        
        assert_eq!(balance.available, 1000.0);
        assert_eq!(balance.total(), 1000.0);
        
        // 测试冻结
        assert!(balance.freeze(500.0).is_ok());
        assert_eq!(balance.available, 500.0);
        assert_eq!(balance.frozen, 500.0);
        assert_eq!(balance.total(), 1000.0);
        
        // 测试解冻
        assert!(balance.unfreeze(200.0).is_ok());
        assert_eq!(balance.available, 700.0);
        assert_eq!(balance.frozen, 300.0);
        
        // 测试扣除
        assert!(balance.deduct(300.0).is_ok());
        assert_eq!(balance.frozen, 0.0);
        assert_eq!(balance.total(), 700.0);
        
        // 测试增加
        balance.credit(300.0);
        assert_eq!(balance.available, 1000.0);
    }

    #[test]
    fn test_virtual_position() {
        let mut position = VirtualPosition::new("BTC/USDT".to_string(), 1.0, 50000.0, 1.0);
        
        assert_eq!(position.quantity, 1.0);
        assert_eq!(position.average_price, 50000.0);
        assert_eq!(position.side, PositionSide::Long);
        
        // 测试增加持仓
        position.update(0.5, 60000.0);
        assert_eq!(position.quantity, 1.5);
        assert!((position.average_price - 53333.33).abs() < 0.01);
        
        // 测试未实现盈亏
        position.calculate_unrealized_pnl(55000.0);
        assert!(position.unrealized_pnl > 0.0);
    }

    #[test]
    fn test_virtual_account_creation() {
        let mut initial_balances = HashMap::new();
        initial_balances.insert("USDT".to_string(), 10000.0);
        initial_balances.insert("BTC".to_string(), 0.1);
        
        let config = VirtualAccountConfig::default();
        let account = VirtualAccount::new("test_account".to_string(), initial_balances, config);
        
        assert!(account.is_ok());
        let account = account.unwrap();
        
        assert_eq!(account.id, "test_account");
        assert_eq!(account.get_available_balance("USDT"), 10000.0);
        assert_eq!(account.get_available_balance("BTC"), 0.1);
        assert_eq!(account.status, AccountStatus::Active);
    }

    #[test]
    fn test_account_stats() {
        let mut stats = AccountStats::default();
        
        // 测试盈利交易
        stats.update_trade_stats(100.0, 5.0);
        assert_eq!(stats.winning_trades, 1);
        assert_eq!(stats.total_trades, 1);
        assert_eq!(stats.win_rate(), 1.0);
        
        // 测试亏损交易
        stats.update_trade_stats(-50.0, 5.0);
        assert_eq!(stats.losing_trades, 1);
        assert_eq!(stats.total_trades, 2);
        assert_eq!(stats.win_rate(), 0.5);
        
        // 测试平均盈亏
        assert_eq!(stats.average_pnl(), 25.0); // (100 - 50) / 2
    }

    #[test]
    fn test_balance_freeze_unfreeze() {
        let mut initial_balances = HashMap::new();
        initial_balances.insert("USDT".to_string(), 1000.0);
        
        let config = VirtualAccountConfig::default();
        let mut account = VirtualAccount::new("test".to_string(), initial_balances, config).unwrap();
        
        // 测试冻结
        assert!(account.freeze_balance("USDT", 500.0).is_ok());
        assert_eq!(account.get_available_balance("USDT"), 500.0);
        
        // 测试解冻
        assert!(account.unfreeze_balance("USDT", 200.0).is_ok());
        assert_eq!(account.get_available_balance("USDT"), 700.0);
        
        // 测试冻结超额
        assert!(account.freeze_balance("USDT", 1000.0).is_err());
    }

    #[test]
    fn test_symbol_parsing() {
        let config = VirtualAccountConfig::default();
        let account = VirtualAccount::new("test".to_string(), HashMap::new(), config).unwrap();
        
        let (base, quote) = account.parse_symbol("BTC/USDT").unwrap();
        assert_eq!(base, "BTC");
        assert_eq!(quote, "USDT");
        
        assert!(account.parse_symbol("INVALID").is_err());
    }
}