//! 资金流动追踪器
//!
//! 实时跟踪多交易所间的资金流动，为Sankey图提供数据支持

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use tracing::{debug, info, instrument, warn, error};
use uuid::Uuid;

use super::sankey_diagram::{SankeyDiagram, SankeyNode, SankeyLink, FlowType, NodeStatus, LinkStatus, NodePosition};

/// 资金流动追踪器主结构
pub struct FundFlowTracker {
    /// 交易所余额记录
    exchange_balances: Arc<RwLock<HashMap<String, ExchangeBalance>>>,
    /// 流动记录
    flow_records: Arc<RwLock<VecDeque<FlowRecord>>>,
    /// Sankey图实例
    sankey_diagram: Arc<SankeyDiagram>,
    /// 配置
    config: FundFlowConfig,
    /// 事件发送器
    event_sender: broadcast::Sender<FundFlowEvent>,
    /// 统计信息
    stats: Arc<RwLock<FlowStats>>,
}

/// 交易所余额信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeBalance {
    pub exchange: String,
    pub symbol: String,
    pub balance: f64,
    pub frozen_balance: f64,
    pub available_balance: f64,
    pub last_update: DateTime<Utc>,
    pub historical_balances: VecDeque<BalanceSnapshot>,
}

/// 余额快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub balance: f64,
    pub reason: BalanceChangeReason,
}

/// 余额变化原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BalanceChangeReason {
    Trade,
    Transfer,
    Fee,
    Arbitrage,
    Deposit,
    Withdrawal,
    Unknown,
}

/// 流动记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRecord {
    pub id: String,
    pub source_exchange: String,
    pub target_exchange: String,
    pub symbol: String,
    pub amount: f64,
    pub flow_type: FlowType,
    pub timestamp: DateTime<Utc>,
    pub status: FlowStatus,
    pub transaction_id: Option<String>,
    pub fee: Option<f64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 流动状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowStatus {
    Initiated,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// 资金流动配置
#[derive(Debug, Clone)]
pub struct FundFlowConfig {
    /// 最大流动记录数
    pub max_flow_records: usize,
    /// 最大历史余额快照数
    pub max_balance_snapshots: usize,
    /// 自动更新间隔（秒）
    pub auto_update_interval_secs: u64,
    /// 异常检测阈值
    pub anomaly_threshold: f64,
    /// 启用实时更新
    pub enable_real_time_updates: bool,
}

impl Default for FundFlowConfig {
    fn default() -> Self {
        Self {
            max_flow_records: 10000,
            max_balance_snapshots: 1000,
            auto_update_interval_secs: 30,
            anomaly_threshold: 0.1, // 10%
            enable_real_time_updates: true,
        }
    }
}

/// 资金流动事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FundFlowEvent {
    BalanceUpdated {
        exchange: String,
        symbol: String,
        old_balance: f64,
        new_balance: f64,
        timestamp: DateTime<Utc>,
    },
    FlowDetected {
        flow_record: FlowRecord,
    },
    AnomalyDetected {
        anomaly: FlowAnomaly,
    },
    LargeFlowAlert {
        flow_record: FlowRecord,
        threshold: f64,
    },
}

/// 流动异常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowAnomaly {
    pub id: String,
    pub anomaly_type: AnomalyType,
    pub exchange: String,
    pub symbol: String,
    pub description: String,
    pub severity: AnomalySeverity,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    SuddenBalanceChange,
    UnusualFlowPattern,
    HighVelocityFlow,
    NegativeBalance,
    SuspiciousTransfer,
}

/// 异常严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 流动统计信息
#[derive(Debug, Clone, Default)]
pub struct FlowStats {
    pub total_flows: u64,
    pub active_flows: u64,
    pub completed_flows: u64,
    pub failed_flows: u64,
    pub total_volume: f64,
    pub avg_flow_amount: f64,
    pub largest_flow: f64,
    pub anomaly_count: u64,
    pub exchanges_tracked: usize,
    pub last_update: Option<DateTime<Utc>>,
}

impl FundFlowTracker {
    /// 创建新的资金流动追踪器
    pub fn new(config: FundFlowConfig, sankey_diagram: Arc<SankeyDiagram>) -> Self {
        let (event_sender, _) = broadcast::channel(1000);
        
        Self {
            exchange_balances: Arc::new(RwLock::new(HashMap::new())),
            flow_records: Arc::new(RwLock::new(VecDeque::new())),
            sankey_diagram,
            config,
            event_sender,
            stats: Arc::new(RwLock::new(FlowStats::default())),
        }
    }

    /// 启动追踪器
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting fund flow tracker");

        if self.config.enable_real_time_updates {
            self.start_auto_update_task().await;
        }

        Ok(())
    }

    /// 更新交易所余额
    #[instrument(skip(self))]
    pub async fn update_exchange_balance(
        &self,
        exchange: &str,
        symbol: &str,
        balance: f64,
        frozen: Option<f64>,
        reason: BalanceChangeReason,
    ) -> Result<()> {
        let mut balances = self.exchange_balances.write().await;
        let key = format!("{}:{}", exchange, symbol);
        let now = Utc::now();
        
        let old_balance = balances.get(&key).map(|b| b.balance).unwrap_or(0.0);
        
        // 创建余额快照
        let snapshot = BalanceSnapshot {
            timestamp: now,
            balance,
            reason: reason.clone(),
        };

        let exchange_balance = balances.entry(key.clone()).or_insert_with(|| {
            ExchangeBalance {
                exchange: exchange.to_string(),
                symbol: symbol.to_string(),
                balance: 0.0,
                frozen_balance: 0.0,
                available_balance: 0.0,
                last_update: now,
                historical_balances: VecDeque::new(),
            }
        });

        // 更新余额
        exchange_balance.balance = balance;
        exchange_balance.frozen_balance = frozen.unwrap_or(0.0);
        exchange_balance.available_balance = balance - exchange_balance.frozen_balance;
        exchange_balance.last_update = now;
        
        // 添加历史快照
        exchange_balance.historical_balances.push_back(snapshot);
        if exchange_balance.historical_balances.len() > self.config.max_balance_snapshots {
            exchange_balance.historical_balances.pop_front();
        }

        // 检测异常
        if let Err(e) = self.detect_balance_anomaly(exchange, symbol, old_balance, balance).await {
            warn!("Failed to detect balance anomaly: {}", e);
        }

        // 更新Sankey图
        let node_id = format!("{}:{}", exchange, symbol);
        if let Err(e) = self.sankey_diagram.update_node_balance(&node_id, balance).await {
            // 如果节点不存在，创建新节点
            let node = SankeyNode {
                id: node_id.clone(),
                name: format!("{} ({})", exchange, symbol),
                balance,
                historical_balances: vec![(now, balance)],
                position: NodePosition { x: 0.0, y: 0.0, width: 150.0, height: 60.0 },
                color: self.get_exchange_color(exchange),
                status: NodeStatus::Active,
                metadata: HashMap::new(),
            };
            
            if let Err(e) = self.sankey_diagram.add_node(node).await {
                warn!("Failed to add node to Sankey diagram: {}", e);
            }
        }

        // 发送事件
        let _ = self.event_sender.send(FundFlowEvent::BalanceUpdated {
            exchange: exchange.to_string(),
            symbol: symbol.to_string(),
            old_balance,
            new_balance: balance,
            timestamp: now,
        });

        // 更新统计信息
        self.update_stats().await;

        debug!("Updated balance for {}:{} - {} -> {}", 
               exchange, symbol, old_balance, balance);

        Ok(())
    }

    /// 记录资金流动
    #[instrument(skip(self))]
    pub async fn record_flow(
        &self,
        source_exchange: &str,
        target_exchange: &str,
        symbol: &str,
        amount: f64,
        flow_type: FlowType,
        transaction_id: Option<String>,
        fee: Option<f64>,
    ) -> Result<String> {
        let flow_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let flow_record = FlowRecord {
            id: flow_id.clone(),
            source_exchange: source_exchange.to_string(),
            target_exchange: target_exchange.to_string(),
            symbol: symbol.to_string(),
            amount,
            flow_type,
            timestamp: now,
            status: FlowStatus::Initiated,
            transaction_id,
            fee,
            metadata: HashMap::new(),
        };

        // 添加到流动记录
        {
            let mut records = self.flow_records.write().await;
            records.push_back(flow_record.clone());
            
            // 保持记录数量在限制范围内
            if records.len() > self.config.max_flow_records {
                records.pop_front();
            }
        }

        // 检测大额流动
        if amount > 100000.0 { // 超过10万
            let _ = self.event_sender.send(FundFlowEvent::LargeFlowAlert {
                flow_record: flow_record.clone(),
                threshold: 100000.0,
            });
        }

        // 添加到Sankey图
        let link_id = format!("flow_{}", flow_id);
        let source_node_id = format!("{}:{}", source_exchange, symbol);
        let target_node_id = format!("{}:{}", target_exchange, symbol);

        let link = SankeyLink {
            id: link_id,
            source: source_node_id,
            target: target_node_id,
            value: amount,
            flow_type,
            color: self.get_flow_color(&flow_type),
            timestamp: now,
            status: LinkStatus::Active,
            metadata: HashMap::new(),
        };

        if let Err(e) = self.sankey_diagram.add_link(link).await {
            warn!("Failed to add link to Sankey diagram: {}", e);
        }

        // 发送事件
        let _ = self.event_sender.send(FundFlowEvent::FlowDetected {
            flow_record: flow_record.clone(),
        });

        // 更新统计信息
        self.update_stats().await;

        info!("Recorded flow: {} -> {} ({}: {})", 
              source_exchange, target_exchange, symbol, amount);

        Ok(flow_id)
    }

    /// 更新流动状态
    #[instrument(skip(self))]
    pub async fn update_flow_status(&self, flow_id: &str, status: FlowStatus) -> Result<()> {
        let mut records = self.flow_records.write().await;
        
        if let Some(record) = records.iter_mut().find(|r| r.id == flow_id) {
            record.status = status.clone();
            
            // 更新Sankey图链接状态
            let link_id = format!("flow_{}", flow_id);
            let sankey_status = match status {
                FlowStatus::Completed => LinkStatus::Completed,
                FlowStatus::Failed | FlowStatus::Cancelled => LinkStatus::Failed,
                FlowStatus::InProgress => LinkStatus::Active,
                _ => LinkStatus::Pending,
            };
            
            // Note: 这里需要扩展Sankey图API来更新链接状态
            debug!("Updated flow {} status to {:?}", flow_id, status);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Flow record not found: {}", flow_id))
        }
    }

    /// 获取交易所余额
    pub async fn get_exchange_balance(&self, exchange: &str, symbol: &str) -> Option<ExchangeBalance> {
        let balances = self.exchange_balances.read().await;
        let key = format!("{}:{}", exchange, symbol);
        balances.get(&key).cloned()
    }

    /// 获取所有交易所余额
    pub async fn get_all_balances(&self) -> HashMap<String, ExchangeBalance> {
        self.exchange_balances.read().await.clone()
    }

    /// 获取流动记录
    pub async fn get_flow_records(&self, limit: Option<usize>) -> Vec<FlowRecord> {
        let records = self.flow_records.read().await;
        
        match limit {
            Some(n) => records.iter().rev().take(n).cloned().collect(),
            None => records.iter().cloned().collect(),
        }
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> FlowStats {
        self.stats.read().await.clone()
    }

    /// 订阅事件
    pub fn subscribe(&self) -> broadcast::Receiver<FundFlowEvent> {
        self.event_sender.subscribe()
    }

    /// 检测余额异常
    async fn detect_balance_anomaly(
        &self,
        exchange: &str,
        symbol: &str,
        old_balance: f64,
        new_balance: f64,
    ) -> Result<()> {
        if old_balance == 0.0 {
            return Ok(()); // 初始余额设置不算异常
        }

        let change_ratio = (new_balance - old_balance).abs() / old_balance;
        
        if change_ratio > self.config.anomaly_threshold {
            let anomaly = FlowAnomaly {
                id: Uuid::new_v4().to_string(),
                anomaly_type: AnomalyType::SuddenBalanceChange,
                exchange: exchange.to_string(),
                symbol: symbol.to_string(),
                description: format!(
                    "Large balance change detected: {:.2} -> {:.2} ({:.1}% change)",
                    old_balance, new_balance, change_ratio * 100.0
                ),
                severity: if change_ratio > 0.5 { 
                    AnomalySeverity::High 
                } else { 
                    AnomalySeverity::Medium 
                },
                timestamp: Utc::now(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("old_balance".to_string(), serde_json::Value::from(old_balance));
                    meta.insert("new_balance".to_string(), serde_json::Value::from(new_balance));
                    meta.insert("change_ratio".to_string(), serde_json::Value::from(change_ratio));
                    meta
                },
            };

            // 发送异常事件
            let _ = self.event_sender.send(FundFlowEvent::AnomalyDetected { anomaly });
        }

        Ok(())
    }

    /// 更新统计信息
    async fn update_stats(&self) {
        let mut stats = self.stats.write().await;
        let records = self.flow_records.read().await;
        let balances = self.exchange_balances.read().await;

        stats.total_flows = records.len() as u64;
        stats.active_flows = records.iter().filter(|r| matches!(r.status, FlowStatus::InProgress)).count() as u64;
        stats.completed_flows = records.iter().filter(|r| matches!(r.status, FlowStatus::Completed)).count() as u64;
        stats.failed_flows = records.iter().filter(|r| matches!(r.status, FlowStatus::Failed | FlowStatus::Cancelled)).count() as u64;
        stats.exchanges_tracked = balances.len();
        
        if !records.is_empty() {
            stats.total_volume = records.iter().map(|r| r.amount).sum();
            stats.avg_flow_amount = stats.total_volume / records.len() as f64;
            stats.largest_flow = records.iter().map(|r| r.amount).fold(0.0, f64::max);
        }
        
        stats.last_update = Some(Utc::now());
    }

    /// 启动自动更新任务
    async fn start_auto_update_task(&self) {
        let sankey_diagram = Arc::clone(&self.sankey_diagram);
        let interval_secs = self.config.auto_update_interval_secs;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_secs)
            );

            loop {
                interval.tick().await;
                
                if let Err(e) = sankey_diagram.create_snapshot().await {
                    error!("Failed to create Sankey snapshot: {}", e);
                }
            }
        });
    }

    /// 获取交易所颜色
    fn get_exchange_color(&self, exchange: &str) -> String {
        match exchange.to_lowercase().as_str() {
            "binance" => "#f3ba2f".to_string(),
            "okx" => "#0052ff".to_string(),
            "huobi" => "#2ebd85".to_string(),
            "bybit" => "#f7a600".to_string(),
            "kucoin" => "#24ae8f".to_string(),
            _ => "#6c757d".to_string(), // 默认灰色
        }
    }

    /// 获取流动类型颜色
    fn get_flow_color(&self, flow_type: &FlowType) -> String {
        match flow_type {
            FlowType::ArbitrageInflow => "#28a745".to_string(),  // 绿色
            FlowType::ArbitrageOutflow => "#dc3545".to_string(), // 红色
            FlowType::BalanceAdjustment => "#007bff".to_string(), // 蓝色
            FlowType::FeePayment => "#ffc107".to_string(),       // 黄色
            FlowType::ProfitWithdrawal => "#17a2b8".to_string(), // 青色
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization::sankey_diagram::SankeyConfig;

    #[tokio::test]
    async fn test_fund_flow_tracker_basic_operations() {
        let sankey_config = SankeyConfig::default();
        let sankey_diagram = Arc::new(SankeyDiagram::new(sankey_config));
        
        let config = FundFlowConfig::default();
        let tracker = FundFlowTracker::new(config, sankey_diagram);

        // 测试余额更新
        tracker.update_exchange_balance(
            "binance",
            "BTC",
            10.0,
            Some(1.0),
            BalanceChangeReason::Trade,
        ).await.unwrap();

        let balance = tracker.get_exchange_balance("binance", "BTC").await.unwrap();
        assert_eq!(balance.balance, 10.0);
        assert_eq!(balance.available_balance, 9.0);

        // 测试流动记录
        let flow_id = tracker.record_flow(
            "binance",
            "okx",
            "BTC",
            1.0,
            FlowType::ArbitrageOutflow,
            Some("tx123".to_string()),
            Some(0.001),
        ).await.unwrap();

        // 测试状态更新
        tracker.update_flow_status(&flow_id, FlowStatus::Completed).await.unwrap();

        let records = tracker.get_flow_records(Some(10)).await;
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, flow_id);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let sankey_config = SankeyConfig::default();
        let sankey_diagram = Arc::new(SankeyDiagram::new(sankey_config));
        
        let mut config = FundFlowConfig::default();
        config.anomaly_threshold = 0.5; // 50% threshold
        
        let tracker = FundFlowTracker::new(config, sankey_diagram);
        let mut receiver = tracker.subscribe();

        // 设置初始余额
        tracker.update_exchange_balance(
            "test",
            "BTC",
            10.0,
            None,
            BalanceChangeReason::Trade,
        ).await.unwrap();

        // 创建大幅度余额变化（应该触发异常）
        tracker.update_exchange_balance(
            "test",
            "BTC",
            20.0, // 100% increase
            None,
            BalanceChangeReason::Trade,
        ).await.unwrap();

        // 检查是否收到异常事件
        if let Ok(event) = receiver.try_recv() {
            match event {
                FundFlowEvent::AnomalyDetected { anomaly } => {
                    assert_eq!(anomaly.anomaly_type, AnomalyType::SuddenBalanceChange);
                },
                _ => panic!("Expected anomaly event"),
            }
        }
    }
}