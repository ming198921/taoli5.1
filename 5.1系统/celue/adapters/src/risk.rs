//! Risk management adapter

use crate::{Adapter, AdapterError, AdapterResult};
use common::ArbitrageOpportunity;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RiskConfig {
    pub max_position_size: f64,
    pub max_daily_loss: f64,
    pub enabled_strategies: Vec<String>,
    /// 单日最大交易次数
    pub max_daily_trades: u32,
    /// 单笔最大亏损比例
    pub max_single_loss_pct: f64,
    /// 最大资金使用率
    pub max_fund_utilization: f64,
    /// 异常价格偏差阈值（超过此值触发风控）
    pub abnormal_price_deviation_pct: f64,
    /// 连续失败次数限制
    pub max_consecutive_failures: u32,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_size: 10000.0, // USD
            max_daily_loss: 1000.0, // USD
            enabled_strategies: vec!["inter_exchange".to_string(), "triangular".to_string()],
            max_daily_trades: 100,
            max_single_loss_pct: 5.0, // 5%
            max_fund_utilization: 80.0, // 80%
            abnormal_price_deviation_pct: 20.0, // 20%
            max_consecutive_failures: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RiskDecision {
    pub approved: bool,
    pub reason: Option<String>,
    pub max_quantity: Option<f64>,
    /// 风控等级 (1-5, 5最严格)
    pub risk_level: u8,
    /// 建议等待时间（秒）
    pub suggested_wait_time: Option<u64>,
}

/// 风控统计数据
#[derive(Debug, Clone)]
pub struct RiskStats {
    pub daily_trades: u32,
    pub daily_pnl: f64,
    pub consecutive_failures: u32,
    pub last_failure_time: Option<DateTime<Utc>>,
    pub current_exposure: f64,
    pub last_reset_date: DateTime<Utc>,
}

impl Default for RiskStats {
    fn default() -> Self {
        Self {
            daily_trades: 0,
            daily_pnl: 0.0,
            consecutive_failures: 0,
            last_failure_time: None,
            current_exposure: 0.0,
            last_reset_date: Utc::now(),
        }
    }
}

#[async_trait::async_trait]
pub trait RiskCheck: Send + Sync {
    async fn check_opportunity(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<RiskDecision>;
}

pub struct RiskAdapter {
    config: Arc<RwLock<RiskConfig>>,
    running: Arc<parking_lot::Mutex<bool>>,
    /// 实时风控统计
    stats: Arc<RwLock<RiskStats>>,
    /// 交易所风控状态
    exchange_risk_states: Arc<RwLock<HashMap<String, ExchangeRiskState>>>,
}

#[derive(Debug, Clone)]
struct ExchangeRiskState {
    /// 是否被风控暂停
    is_suspended: bool,
    /// 暂停原因
    suspension_reason: Option<String>,
    /// 暂停到期时间
    suspension_until: Option<DateTime<Utc>>,
    /// 近期错误率
    recent_error_rate: f64,
    /// 最后健康检查时间
    last_health_check: DateTime<Utc>,
}

impl Default for ExchangeRiskState {
    fn default() -> Self {
        Self {
            is_suspended: false,
            suspension_reason: None,
            suspension_until: None,
            recent_error_rate: 0.0,
            last_health_check: Utc::now(),
        }
    }
}

impl RiskAdapter {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(RiskConfig::default())),
            running: Arc::new(parking_lot::Mutex::new(false)),
            stats: Arc::new(RwLock::new(RiskStats::default())),
            exchange_risk_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn with_config(config: RiskConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            running: Arc::new(parking_lot::Mutex::new(false)),
            stats: Arc::new(RwLock::new(RiskStats::default())),
            exchange_risk_states: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 实时风控检查 - 策略联动的核心
    pub async fn check_risk(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<RiskDecision> {
        // 1. 基础风控检查
        if let Some(basic_rejection) = self.basic_risk_checks(opportunity).await? {
            return Ok(basic_rejection);
        }
        
        // 2. 交易所状态检查
        if let Some(exchange_rejection) = self.check_exchange_risk_state(opportunity).await? {
            return Ok(exchange_rejection);
        }
        
        // 3. 实时风控统计检查
        if let Some(stats_rejection) = self.check_risk_statistics(opportunity).await? {
            return Ok(stats_rejection);
        }
        
        // 4. 异常价格检查
        if let Some(price_rejection) = self.check_price_anomaly(opportunity).await? {
            return Ok(price_rejection);
        }
        
        // 5. 动态风控等级计算
        let risk_level = self.calculate_dynamic_risk_level(opportunity).await?;
        
        // 6. 所有检查通过，批准交易
        Ok(RiskDecision {
            approved: true,
            reason: Some("All risk checks passed".to_string()),
            max_quantity: self.calculate_max_safe_quantity(opportunity).await?,
            risk_level,
            suggested_wait_time: None,
        })
    }
    
    /// 基础风控检查
    async fn basic_risk_checks(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<Option<RiskDecision>> {
        let _config = self.config.read().await;
        
        // 检查利润是否过低（可能是数据错误）
        if opportunity.net_profit_pct.to_f64() < 0.001 {
            return Ok(Some(RiskDecision {
                approved: false,
                reason: Some("Profit too low, possible data error".to_string()),
                max_quantity: None,
                risk_level: 1,
                suggested_wait_time: None,
            }));
        }
        
        // 检查利润是否异常高（可能是价格错误）
        if opportunity.net_profit_pct.to_f64() > 0.1 { // 10%
            return Ok(Some(RiskDecision {
                approved: false,
                reason: Some("Profit suspiciously high, possible price error".to_string()),
                max_quantity: None,
                risk_level: 5,
                suggested_wait_time: Some(30), // 等待30秒
            }));
        }
        
        Ok(None)
    }
    
    /// 检查交易所风控状态
    async fn check_exchange_risk_state(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<Option<RiskDecision>> {
        let exchange_states = self.exchange_risk_states.read().await;
        
        // 检查相关交易所是否被暂停
        for leg in &opportunity.legs {
            if let Some(state) = exchange_states.get(&leg.exchange.to_string()) {
                if state.is_suspended {
                    if let Some(until) = state.suspension_until {
                        if Utc::now() < until {
                            return Ok(Some(RiskDecision {
                                approved: false,
                                reason: Some(format!(
                                    "Exchange {} suspended until {}: {}", 
                                    leg.exchange.to_string(), 
                                    until.format("%H:%M:%S"),
                                    state.suspension_reason.as_deref().unwrap_or("Unknown")
                                )),
                                max_quantity: None,
                                risk_level: 4,
                                suggested_wait_time: Some((until - Utc::now()).num_seconds() as u64),
                            }));
                        }
                    }
                }
                
                // 检查错误率是否过高
                if state.recent_error_rate > 0.1 { // 10%错误率
                    return Ok(Some(RiskDecision {
                        approved: false,
                        reason: Some(format!("Exchange {} error rate too high: {:.1}%", leg.exchange.to_string(), state.recent_error_rate * 100.0)),
                        max_quantity: None,
                        risk_level: 3,
                        suggested_wait_time: Some(60),
                    }));
                }
            }
        }
        
        Ok(None)
    }
    
    /// 检查风控统计
    async fn check_risk_statistics(&self, _opportunity: &ArbitrageOpportunity) -> AdapterResult<Option<RiskDecision>> {
        let mut stats = self.stats.write().await;
        let config = self.config.read().await;
        
        // 重置日统计（如果是新的一天）
        let now = Utc::now();
        if now.date_naive() != stats.last_reset_date.date_naive() {
            stats.daily_trades = 0;
            stats.daily_pnl = 0.0;
            stats.last_reset_date = now;
        }
        
        // 检查日交易次数限制
        if stats.daily_trades >= config.max_daily_trades {
            return Ok(Some(RiskDecision {
                approved: false,
                reason: Some(format!("Daily trade limit reached: {}/{}", stats.daily_trades, config.max_daily_trades)),
                max_quantity: None,
                risk_level: 2,
                suggested_wait_time: Some(3600), // 1小时后重试
            }));
        }
        
        // 检查日亏损限制
        if stats.daily_pnl < -config.max_daily_loss {
            return Ok(Some(RiskDecision {
                approved: false,
                reason: Some(format!("Daily loss limit exceeded: ${:.2}", stats.daily_pnl.abs())),
                max_quantity: None,
                risk_level: 4,
                suggested_wait_time: Some(7200), // 2小时后重试
            }));
        }
        
        // 检查连续失败次数
        if stats.consecutive_failures >= config.max_consecutive_failures {
            return Ok(Some(RiskDecision {
                approved: false,
                reason: Some(format!("Too many consecutive failures: {}", stats.consecutive_failures)),
                max_quantity: None,
                risk_level: 3,
                suggested_wait_time: Some(1800), // 30分钟后重试
            }));
        }
        
        Ok(None)
    }
    
    /// 检查价格异常
    async fn check_price_anomaly(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<Option<RiskDecision>> {
        let config = self.config.read().await;
        
        // 检查价格偏差是否过大
        if opportunity.net_profit_pct.to_f64() * 100.0 > config.abnormal_price_deviation_pct {
            return Ok(Some(RiskDecision {
                approved: false,
                reason: Some(format!(
                    "Price deviation too large: {:.2}% > {:.1}%", 
                    opportunity.net_profit_pct.to_f64() * 100.0,
                    config.abnormal_price_deviation_pct
                )),
                max_quantity: None,
                risk_level: 5,
                suggested_wait_time: Some(120), // 2分钟等待
            }));
        }
        
        Ok(None)
    }
    
    /// 计算动态风控等级
    async fn calculate_dynamic_risk_level(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<u8> {
        let stats = self.stats.read().await;
        let mut risk_level = 1u8;
        
        // 基于近期表现调整风控等级
        if stats.consecutive_failures > 2 {
            risk_level += 1;
        }
        
        if stats.daily_pnl < 0.0 {
            risk_level += 1;
        }
        
        // 基于利润大小调整
        let profit_pct = opportunity.net_profit_pct.to_f64() * 100.0;
        if profit_pct > 5.0 {
            risk_level += 1; // 高利润需要更谨慎
        }
        
        Ok(risk_level.min(5))
    }
    
    /// 计算最大安全数量
    async fn calculate_max_safe_quantity(&self, opportunity: &ArbitrageOpportunity) -> AdapterResult<Option<f64>> {
        let config = self.config.read().await;
        let stats = self.stats.read().await;
        
        // 基于配置的最大仓位大小
        let mut max_qty = config.max_position_size;
        
        // 基于当前资金使用率调整
        let utilization_factor = (100.0 - config.max_fund_utilization) / 100.0;
        max_qty *= utilization_factor;
        
        // 基于近期表现调整
        if stats.consecutive_failures > 0 {
            max_qty *= 0.5; // 减半
        }
        
        // 基于机会本身的大小调整
        if let Some(first_leg) = opportunity.legs.first() {
            let opportunity_size = first_leg.cost.to_f64();
            max_qty = max_qty.min(opportunity_size);
        }
        
        Ok(Some(max_qty))
    }
    
    /// 交易执行结果反馈 - 用于动态调整风控策略
    pub async fn report_execution_result(&self, opportunity: &ArbitrageOpportunity, success: bool, actual_pnl: f64) -> AdapterResult<()> {
        let mut stats = self.stats.write().await;
        
        // 更新统计数据
        stats.daily_trades += 1;
        stats.daily_pnl += actual_pnl;
        
        if success {
            stats.consecutive_failures = 0;
        } else {
            stats.consecutive_failures += 1;
            stats.last_failure_time = Some(Utc::now());
        }
        
        // 更新交易所风控状态
        self.update_exchange_risk_states(opportunity, success).await?;
        
        Ok(())
    }
    
    /// 更新交易所风控状态
    async fn update_exchange_risk_states(&self, opportunity: &ArbitrageOpportunity, success: bool) -> AdapterResult<()> {
        let mut exchange_states = self.exchange_risk_states.write().await;
        
        for leg in &opportunity.legs {
            let state = exchange_states.entry(leg.exchange.to_string()).or_insert_with(ExchangeRiskState::default);
            
            state.last_health_check = Utc::now();
            
            if !success {
                // 增加错误率
                state.recent_error_rate = (state.recent_error_rate * 0.9 + 0.1).min(1.0);
                
                // 如果错误率过高，暂停交易所
                if state.recent_error_rate > 0.2 && !state.is_suspended {
                    state.is_suspended = true;
                    state.suspension_reason = Some("High error rate detected".to_string());
                    state.suspension_until = Some(Utc::now() + Duration::minutes(30));
                }
            } else {
                // 降低错误率
                state.recent_error_rate = (state.recent_error_rate * 0.95).max(0.0);
                
                // 如果错误率恢复正常，解除暂停
                if state.recent_error_rate < 0.05 && state.is_suspended {
                    if let Some(until) = state.suspension_until {
                        if Utc::now() > until {
                            state.is_suspended = false;
                            state.suspension_reason = None;
                            state.suspension_until = None;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// 获取风控统计信息
    pub async fn get_risk_statistics(&self) -> RiskStats {
        self.stats.read().await.clone()
    }
    
    /// 手动调整风控配置
    pub async fn update_risk_config(&self, new_config: RiskConfig) -> AdapterResult<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Adapter for RiskAdapter {
    type Config = RiskConfig;
    type Error = AdapterError;
    
    async fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        *self.running.lock() = true;
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Self::Error> {
        *self.running.lock() = false;
        Ok(())
    }
    
    async fn health_check(&self) -> Result<(), Self::Error> {
        if !*self.running.lock() {
            return Err(AdapterError::NotInitialized);
        }
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "risk_adapter"
    }
}

impl Default for RiskAdapter {
    fn default() -> Self {
        Self::new()
    }
}
