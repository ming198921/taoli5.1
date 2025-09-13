//! Sankey资金流向图实现
//!
//! 提供实时资金流向可视化，支持：
//! - 多交易所资金分布展示
//! - 套利资金流动路径追踪
//! - 实时数据更新和历史回放
//! - 自适应布局算法

use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

/// Sankey资金流向图主结构
pub struct SankeyDiagram {
    /// 图数据
    data: Arc<RwLock<SankeyData>>,
    /// 配置参数
    config: SankeyConfig,
    /// 布局引擎
    layout_engine: SankeyLayoutEngine,
    /// 动画控制器
    animation_controller: AnimationController,
}

/// Sankey图数据
#[derive(Debug, Clone)]
struct SankeyData {
    /// 节点集合（交易所）
    nodes: HashMap<String, SankeyNode>,
    /// 链接集合（资金流）
    links: HashMap<String, SankeyLink>,
    /// 时间序列数据
    time_series: VecDeque<SankeySnapshot>,
    /// 最后更新时间
    last_update: DateTime<Utc>,
}

/// Sankey节点（交易所）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SankeyNode {
    /// 节点ID（交易所名称）
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 当前余额
    pub balance: f64,
    /// 历史余额
    pub historical_balances: Vec<(DateTime<Utc>, f64)>,
    /// 节点位置
    pub position: NodePosition,
    /// 节点颜色
    pub color: String,
    /// 节点状态
    pub status: NodeStatus,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Sankey链接（资金流）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SankeyLink {
    /// 链接ID
    pub id: String,
    /// 源节点ID
    pub source: String,
    /// 目标节点ID
    pub target: String,
    /// 流量值
    pub value: f64,
    /// 流动类型
    pub flow_type: FlowType,
    /// 链接颜色
    pub color: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 链接状态
    pub status: LinkStatus,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 节点位置
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// 节点状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Inactive,
    Warning,
    Error,
}

/// 流动类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FlowType {
    /// 套利流入
    ArbitrageInflow,
    /// 套利流出
    ArbitrageOutflow,
    /// 余额调整
    BalanceAdjustment,
    /// 费用支付
    FeePayment,
    /// 盈利提取
    ProfitWithdrawal,
}

/// 链接状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LinkStatus {
    Active,
    Completed,
    Pending,
    Failed,
}

/// Sankey配置
#[derive(Debug, Clone)]
pub struct SankeyConfig {
    /// 画布宽度
    pub canvas_width: f64,
    /// 画布高度
    pub canvas_height: f64,
    /// 节点间距
    pub node_spacing: f64,
    /// 链接曲率
    pub link_curvature: f64,
    /// 最小链接宽度
    pub min_link_width: f64,
    /// 最大链接宽度
    pub max_link_width: f64,
    /// 历史数据保留条数
    pub max_history_entries: usize,
    /// 动画持续时间（毫秒）
    pub animation_duration_ms: u64,
}

impl Default for SankeyConfig {
    fn default() -> Self {
        Self {
            canvas_width: 1200.0,
            canvas_height: 800.0,
            node_spacing: 200.0,
            link_curvature: 0.5,
            min_link_width: 2.0,
            max_link_width: 50.0,
            max_history_entries: 1000,
            animation_duration_ms: 1000,
        }
    }
}

/// 历史快照
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SankeySnapshot {
    timestamp: DateTime<Utc>,
    nodes: HashMap<String, SankeyNode>,
    links: HashMap<String, SankeyLink>,
}

/// 布局引擎
struct SankeyLayoutEngine {
    config: SankeyConfig,
}

/// 动画控制器
struct AnimationController {
    current_frame: u64,
    total_frames: u64,
    animation_type: AnimationType,
}

/// 动画类型
#[derive(Debug, Clone, Copy)]
enum AnimationType {
    FlowAnimation,
    NodeTransition,
    LinkTransition,
}

impl SankeyDiagram {
    /// 创建新的Sankey图
    pub fn new(config: SankeyConfig) -> Self {
        Self {
            data: Arc::new(RwLock::new(SankeyData {
                nodes: HashMap::new(),
                links: HashMap::new(),
                time_series: VecDeque::new(),
                last_update: Utc::now(),
            })),
            layout_engine: SankeyLayoutEngine::new(config.clone()),
            animation_controller: AnimationController::new(),
            config,
        }
    }

    /// 添加节点（交易所）
    #[instrument(skip(self))]
    pub async fn add_node(&self, mut node: SankeyNode) -> Result<()> {
        // 计算节点位置
        node.position = self.layout_engine.calculate_node_position(&node.id, &self.data).await?;
        
        let mut data = self.data.write().await;
        data.nodes.insert(node.id.clone(), node.clone());
        data.last_update = Utc::now();
        
        info!("Added Sankey node: {}", node.name);
        Ok(())
    }

    /// 添加链接（资金流）
    #[instrument(skip(self))]
    pub async fn add_link(&self, link: SankeyLink) -> Result<()> {
        // 验证源节点和目标节点存在
        let data_read = self.data.read().await;
        if !data_read.nodes.contains_key(&link.source) {
            return Err(anyhow::anyhow!("Source node not found: {}", link.source));
        }
        if !data_read.nodes.contains_key(&link.target) {
            return Err(anyhow::anyhow!("Target node not found: {}", link.target));
        }
        drop(data_read);

        let mut data = self.data.write().await;
        data.links.insert(link.id.clone(), link.clone());
        data.last_update = Utc::now();
        
        info!("Added Sankey link: {} -> {} (value: {})", 
              link.source, link.target, link.value);
        Ok(())
    }

    /// 更新节点余额
    #[instrument(skip(self))]
    pub async fn update_node_balance(&self, node_id: &str, new_balance: f64) -> Result<()> {
        let mut data = self.data.write().await;
        
        if let Some(node) = data.nodes.get_mut(node_id) {
            let old_balance = node.balance;
            node.balance = new_balance;
            node.historical_balances.push((Utc::now(), new_balance));
            
            // 保持历史记录在限制范围内
            if node.historical_balances.len() > self.config.max_history_entries {
                node.historical_balances.drain(0..node.historical_balances.len() - self.config.max_history_entries);
            }
            
            data.last_update = Utc::now();
            
            debug!("Updated node {} balance: {} -> {}", 
                   node_id, old_balance, new_balance);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Node not found: {}", node_id))
        }
    }

    /// 更新链接流量
    #[instrument(skip(self))]
    pub async fn update_link_value(&self, link_id: &str, new_value: f64) -> Result<()> {
        let mut data = self.data.write().await;
        
        if let Some(link) = data.links.get_mut(link_id) {
            let old_value = link.value;
            link.value = new_value;
            link.timestamp = Utc::now();
            data.last_update = Utc::now();
            
            debug!("Updated link {} value: {} -> {}", 
                   link_id, old_value, new_value);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Link not found: {}", link_id))
        }
    }

    /// 创建历史快照
    #[instrument(skip(self))]
    pub async fn create_snapshot(&self) -> Result<()> {
        let mut data = self.data.write().await;
        
        let snapshot = SankeySnapshot {
            timestamp: Utc::now(),
            nodes: data.nodes.clone(),
            links: data.links.clone(),
        };
        
        data.time_series.push_back(snapshot);
        
        // 保持历史快照在限制范围内
        if data.time_series.len() > self.config.max_history_entries {
            data.time_series.pop_front();
        }
        
        debug!("Created Sankey snapshot");
        Ok(())
    }

    /// 获取当前图数据
    pub async fn get_current_data(&self) -> SankeyVisualizationData {
        let data = self.data.read().await;
        
        SankeyVisualizationData {
            nodes: data.nodes.values().cloned().collect(),
            links: data.links.values().cloned().collect(),
            timestamp: data.last_update,
            canvas_width: self.config.canvas_width,
            canvas_height: self.config.canvas_height,
        }
    }

    /// 获取历史数据
    pub async fn get_historical_data(&self, limit: Option<usize>) -> Vec<SankeySnapshot> {
        let data = self.data.read().await;
        
        match limit {
            Some(n) => data.time_series.iter().rev().take(n).cloned().collect(),
            None => data.time_series.iter().cloned().collect(),
        }
    }

    /// 计算总流入流量
    pub async fn calculate_total_inflow(&self, node_id: &str) -> Result<f64> {
        let data = self.data.read().await;
        
        let total = data.links.values()
            .filter(|link| link.target == node_id && link.status == LinkStatus::Active)
            .map(|link| link.value)
            .sum();
        
        Ok(total)
    }

    /// 计算总流出流量
    pub async fn calculate_total_outflow(&self, node_id: &str) -> Result<f64> {
        let data = self.data.read().await;
        
        let total = data.links.values()
            .filter(|link| link.source == node_id && link.status == LinkStatus::Active)
            .map(|link| link.value)
            .sum();
        
        Ok(total)
    }

    /// 获取节点的净流量
    pub async fn get_net_flow(&self, node_id: &str) -> Result<f64> {
        let inflow = self.calculate_total_inflow(node_id).await?;
        let outflow = self.calculate_total_outflow(node_id).await?;
        Ok(inflow - outflow)
    }

    /// 检测资金流向异常
    pub async fn detect_flow_anomalies(&self) -> Result<Vec<FlowAnomaly>> {
        let data = self.data.read().await;
        let mut anomalies = Vec::new();

        for link in data.links.values() {
            // 检测异常大的流量
            if link.value > 1000000.0 { // 100万以上
                anomalies.push(FlowAnomaly {
                    anomaly_type: AnomalyType::LargeFlow,
                    link_id: link.id.clone(),
                    description: format!("Large flow detected: {:.2}", link.value),
                    severity: AnomalySeverity::High,
                    timestamp: link.timestamp,
                });
            }

            // 检测循环流动
            if link.source == link.target {
                anomalies.push(FlowAnomaly {
                    anomaly_type: AnomalyType::CircularFlow,
                    link_id: link.id.clone(),
                    description: "Circular flow detected".to_string(),
                    severity: AnomalySeverity::Medium,
                    timestamp: link.timestamp,
                });
            }
        }

        // 检测孤立节点
        for node in data.nodes.values() {
            let has_incoming = data.links.values().any(|link| link.target == node.id);
            let has_outgoing = data.links.values().any(|link| link.source == node.id);
            
            if !has_incoming && !has_outgoing {
                anomalies.push(FlowAnomaly {
                    anomaly_type: AnomalyType::IsolatedNode,
                    link_id: format!("node_{}", node.id),
                    description: format!("Isolated node: {}", node.name),
                    severity: AnomalySeverity::Low,
                    timestamp: Utc::now(),
                });
            }
        }

        Ok(anomalies)
    }

    /// 生成Web可视化数据
    pub async fn generate_web_data(&self) -> Result<String> {
        let viz_data = self.get_current_data().await;
        serde_json::to_string_pretty(&viz_data)
            .context("Failed to serialize Sankey visualization data")
    }

    /// 清理过期数据
    #[instrument(skip(self))]
    pub async fn cleanup_expired_data(&self, retention_hours: i64) -> Result<usize> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(retention_hours);
        let mut data = self.data.write().await;
        
        let initial_count = data.time_series.len();
        data.time_series.retain(|snapshot| snapshot.timestamp > cutoff_time);
        let removed_count = initial_count - data.time_series.len();
        
        if removed_count > 0 {
            info!("Cleaned up {} expired Sankey snapshots", removed_count);
        }
        
        Ok(removed_count)
    }
}

impl SankeyLayoutEngine {
    fn new(config: SankeyConfig) -> Self {
        Self { config }
    }

    async fn calculate_node_position(&self, node_id: &str, data: &Arc<RwLock<SankeyData>>) -> Result<NodePosition> {
        let data_read = data.read().await;
        let node_count = data_read.nodes.len();
        let index = node_count; // 新节点的索引
        
        // 简单的垂直布局算法
        let y = (index as f64 * (self.config.canvas_height / 10.0)) % self.config.canvas_height;
        let x = if index % 2 == 0 { 100.0 } else { self.config.canvas_width - 200.0 };
        
        Ok(NodePosition {
            x,
            y,
            width: 150.0,
            height: 60.0,
        })
    }
}

impl AnimationController {
    fn new() -> Self {
        Self {
            current_frame: 0,
            total_frames: 60,
            animation_type: AnimationType::FlowAnimation,
        }
    }
}

/// 可视化数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SankeyVisualizationData {
    pub nodes: Vec<SankeyNode>,
    pub links: Vec<SankeyLink>,
    pub timestamp: DateTime<Utc>,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

/// 流量异常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowAnomaly {
    pub anomaly_type: AnomalyType,
    pub link_id: String,
    pub description: String,
    pub severity: AnomalySeverity,
    pub timestamp: DateTime<Utc>,
}

/// 异常类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    LargeFlow,
    CircularFlow,
    IsolatedNode,
    SuddenChange,
}

/// 异常严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sankey_basic_operations() {
        let config = SankeyConfig::default();
        let diagram = SankeyDiagram::new(config);

        // 添加节点
        let node1 = SankeyNode {
            id: "binance".to_string(),
            name: "Binance".to_string(),
            balance: 10000.0,
            historical_balances: Vec::new(),
            position: NodePosition { x: 0.0, y: 0.0, width: 150.0, height: 60.0 },
            color: "#3498db".to_string(),
            status: NodeStatus::Active,
            metadata: HashMap::new(),
        };

        let node2 = SankeyNode {
            id: "okx".to_string(),
            name: "OKX".to_string(),
            balance: 8000.0,
            historical_balances: Vec::new(),
            position: NodePosition { x: 600.0, y: 0.0, width: 150.0, height: 60.0 },
            color: "#2ecc71".to_string(),
            status: NodeStatus::Active,
            metadata: HashMap::new(),
        };

        diagram.add_node(node1).await.unwrap();
        diagram.add_node(node2).await.unwrap();

        // 添加链接
        let link = SankeyLink {
            id: "flow_1".to_string(),
            source: "binance".to_string(),
            target: "okx".to_string(),
            value: 5000.0,
            flow_type: FlowType::ArbitrageOutflow,
            color: "#e74c3c".to_string(),
            timestamp: Utc::now(),
            status: LinkStatus::Active,
            metadata: HashMap::new(),
        };

        diagram.add_link(link).await.unwrap();

        // 验证数据
        let viz_data = diagram.get_current_data().await;
        assert_eq!(viz_data.nodes.len(), 2);
        assert_eq!(viz_data.links.len(), 1);

        // 计算流量
        let inflow = diagram.calculate_total_inflow("okx").await.unwrap();
        let outflow = diagram.calculate_total_outflow("binance").await.unwrap();
        assert_eq!(inflow, 5000.0);
        assert_eq!(outflow, 5000.0);
    }

    #[tokio::test]
    async fn test_anomaly_detection() {
        let config = SankeyConfig::default();
        let diagram = SankeyDiagram::new(config);

        // 添加测试数据
        let node = SankeyNode {
            id: "test".to_string(),
            name: "Test".to_string(),
            balance: 1000.0,
            historical_balances: Vec::new(),
            position: NodePosition { x: 0.0, y: 0.0, width: 150.0, height: 60.0 },
            color: "#ffffff".to_string(),
            status: NodeStatus::Active,
            metadata: HashMap::new(),
        };

        diagram.add_node(node).await.unwrap();

        // 添加异常大流量链接
        let large_flow_link = SankeyLink {
            id: "large_flow".to_string(),
            source: "test".to_string(),
            target: "test".to_string(), // 循环流动
            value: 2000000.0, // 异常大的值
            flow_type: FlowType::ArbitrageOutflow,
            color: "#ff0000".to_string(),
            timestamp: Utc::now(),
            status: LinkStatus::Active,
            metadata: HashMap::new(),
        };

        diagram.add_link(large_flow_link).await.unwrap();

        // 检测异常
        let anomalies = diagram.detect_flow_anomalies().await.unwrap();
        assert!(anomalies.len() >= 2); // 应该检测到大流量和循环流动异常
    }
}