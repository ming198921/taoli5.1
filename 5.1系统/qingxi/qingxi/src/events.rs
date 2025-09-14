#![allow(dead_code)]
// src/events.rs
use crate::types::{Symbol, AnomalyDetectionResult};
use crate::orderbook::local_orderbook::MarketDataMessage;
use crate::health::HealthStatus;

// 临时定义缺失的类型
pub type ExchangeId = String;

#[derive(Debug, Clone)]
pub struct ConsistencyCheckResult {
    pub is_consistent: bool,
    pub symbol: Symbol,
    pub timestamp: crate::high_precision_time::Nanos,
    pub details: String,
}
/// 系统事件枚举，用于组件间通信
#[derive(Debug, Clone)]
pub enum SystemEvent {
    /// 接收到市场数据
    MarketDataReceived(MarketDataMessage),
    
    /// 检测到异常
    AnomalyDetected(AnomalyDetectionResult),
    
    /// 一致性检查完成
    ConsistencyCheckCompleted(ConsistencyCheckResult),
    
    /// 组件状态变更
    ComponentStatusChanged {
        component: String,
        status: HealthStatus,
        message: String,
    },
    
    /// 需要重新同步
    ResyncNeeded {
        exchange: ExchangeId,
        symbol: Symbol,
    },
}
