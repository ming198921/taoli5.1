//! 指标收集和监控模块

use crate::config::MetricsConfig;
use crate::order_matching::TradeExecution;
use crate::risk_manager::RiskViolation;
use anyhow::Result;
use chrono::{DateTime, Utc};
use prometheus::{
    Counter, CounterVec, Histogram, HistogramVec, IntGauge, IntGaugeVec, 
    Opts, Registry, TextEncoder, Encoder
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 影子交易指标收集器
pub struct ShadowTradingMetrics {
    config: MetricsConfig,
    registry: Registry,
    
    // 订单指标
    orders_submitted_total: CounterVec,
    orders_cancelled_total: Counter,
    orders_expired_total: Counter,
    orders_filled_total: CounterVec,
    order_processing_duration: HistogramVec,
    
    // 交易指标
    trades_executed_total: CounterVec,
    trade_volume_total: CounterVec,
    trade_value_total: CounterVec,
    trade_fees_total: CounterVec,
    
    // 账户指标
    accounts_active_total: IntGauge,
    account_balance_total: IntGaugeVec,
    account_pnl_total: IntGaugeVec,
    account_positions_total: IntGaugeVec,
    
    // 风险指标
    risk_violations_total: CounterVec,
    account_var_current: IntGaugeVec,
    margin_usage_ratio: IntGaugeVec,
    
    // 系统指标
    system_uptime_seconds: IntGauge,
    market_data_updates_total: CounterVec,
    order_matching_latency: Histogram,
    
    // 内部状态
    start_time: Instant,
    last_push: Arc<RwLock<Instant>>,
    cached_metrics: Arc<RwLock<HashMap<String, f64>>>,
}

impl ShadowTradingMetrics {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        let config = MetricsConfig::default();
        
        // 初始化订单指标
        let orders_submitted_total = CounterVec::new(
            Opts::new("shadow_orders_submitted_total", "Total number of submitted orders")
                .namespace("shadow_trading"),
            &["account_id", "symbol", "side", "order_type"]
        )?;
        registry.register(Box::new(orders_submitted_total.clone()))?;

        let orders_cancelled_total = Counter::new(
            "shadow_orders_cancelled_total",
            "Total number of cancelled orders"
        )?;
        registry.register(Box::new(orders_cancelled_total.clone()))?;

        let orders_expired_total = Counter::new(
            "shadow_orders_expired_total",
            "Total number of expired orders"
        )?;
        registry.register(Box::new(orders_expired_total.clone()))?;

        let orders_filled_total = CounterVec::new(
            Opts::new("shadow_orders_filled_total", "Total number of filled orders")
                .namespace("shadow_trading"),
            &["account_id", "symbol", "side"]
        )?;
        registry.register(Box::new(orders_filled_total.clone()))?;

        let order_processing_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "shadow_order_processing_duration_seconds",
                "Time spent processing orders"
            ).namespace("shadow_trading"),
            &["order_type", "outcome"]
        )?;
        registry.register(Box::new(order_processing_duration.clone()))?;

        // 初始化交易指标
        let trades_executed_total = CounterVec::new(
            Opts::new("shadow_trades_executed_total", "Total number of executed trades")
                .namespace("shadow_trading"),
            &["account_id", "symbol", "side"]
        )?;
        registry.register(Box::new(trades_executed_total.clone()))?;

        let trade_volume_total = CounterVec::new(
            Opts::new("shadow_trade_volume_total", "Total trading volume")
                .namespace("shadow_trading"),
            &["symbol", "side"]
        )?;
        registry.register(Box::new(trade_volume_total.clone()))?;

        let trade_value_total = CounterVec::new(
            Opts::new("shadow_trade_value_total", "Total trading value")
                .namespace("shadow_trading"),
            &["symbol", "currency"]
        )?;
        registry.register(Box::new(trade_value_total.clone()))?;

        let trade_fees_total = CounterVec::new(
            Opts::new("shadow_trade_fees_total", "Total trading fees")
                .namespace("shadow_trading"),
            &["account_id", "symbol"]
        )?;
        registry.register(Box::new(trade_fees_total.clone()))?;

        // 初始化账户指标
        let accounts_active_total = IntGauge::new(
            "shadow_accounts_active_total",
            "Number of active trading accounts"
        )?;
        registry.register(Box::new(accounts_active_total.clone()))?;

        let account_balance_total = IntGaugeVec::new(
            Opts::new("shadow_account_balance_total", "Account balance by currency")
                .namespace("shadow_trading"),
            &["account_id", "currency"]
        )?;
        registry.register(Box::new(account_balance_total.clone()))?;

        let account_pnl_total = IntGaugeVec::new(
            Opts::new("shadow_account_pnl_total", "Account PnL")
                .namespace("shadow_trading"),
            &["account_id", "type"]
        )?;
        registry.register(Box::new(account_pnl_total.clone()))?;

        let account_positions_total = IntGaugeVec::new(
            Opts::new("shadow_account_positions_total", "Number of open positions")
                .namespace("shadow_trading"),
            &["account_id", "symbol"]
        )?;
        registry.register(Box::new(account_positions_total.clone()))?;

        // 初始化风险指标
        let risk_violations_total = CounterVec::new(
            Opts::new("shadow_risk_violations_total", "Total risk violations")
                .namespace("shadow_trading"),
            &["account_id", "violation_type", "severity"]
        )?;
        registry.register(Box::new(risk_violations_total.clone()))?;

        let account_var_current = IntGaugeVec::new(
            Opts::new("shadow_account_var_current", "Current Value at Risk")
                .namespace("shadow_trading"),
            &["account_id", "confidence_level"]
        )?;
        registry.register(Box::new(account_var_current.clone()))?;

        let margin_usage_ratio = IntGaugeVec::new(
            Opts::new("shadow_margin_usage_ratio", "Margin usage ratio")
                .namespace("shadow_trading"),
            &["account_id"]
        )?;
        registry.register(Box::new(margin_usage_ratio.clone()))?;

        // 初始化系统指标
        let system_uptime_seconds = IntGauge::new(
            "shadow_system_uptime_seconds",
            "System uptime in seconds"
        )?;
        registry.register(Box::new(system_uptime_seconds.clone()))?;

        let market_data_updates_total = CounterVec::new(
            Opts::new("shadow_market_data_updates_total", "Market data updates")
                .namespace("shadow_trading"),
            &["symbol", "data_type"]
        )?;
        registry.register(Box::new(market_data_updates_total.clone()))?;

        let order_matching_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "shadow_order_matching_latency_seconds",
                "Order matching latency"
            ).namespace("shadow_trading")
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0])
        )?;
        registry.register(Box::new(order_matching_latency.clone()))?;

        Ok(Self {
            config,
            registry,
            orders_submitted_total,
            orders_cancelled_total,
            orders_expired_total,
            orders_filled_total,
            order_processing_duration,
            trades_executed_total,
            trade_volume_total,
            trade_value_total,
            trade_fees_total,
            accounts_active_total,
            account_balance_total,
            account_pnl_total,
            account_positions_total,
            risk_violations_total,
            account_var_current,
            margin_usage_ratio,
            system_uptime_seconds,
            market_data_updates_total,
            order_matching_latency,
            start_time: Instant::now(),
            last_push: Arc::new(RwLock::new(Instant::now())),
            cached_metrics: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    // 订单指标更新方法
    pub async fn order_submitted(&self, order_id: &str) {
        // 这里需要从实际订单中获取标签值
        self.orders_submitted_total
            .with_label_values(&["unknown", "unknown", "unknown", "unknown"])
            .inc();
        
        debug!(order_id = %order_id, "Order submitted metric updated");
    }

    pub async fn order_cancelled(&self, order_id: &str) {
        self.orders_cancelled_total.inc();
        debug!(order_id = %order_id, "Order cancelled metric updated");
    }

    pub async fn order_expired(&self, order_id: &str) {
        self.orders_expired_total.inc();
        debug!(order_id = %order_id, "Order expired metric updated");
    }

    pub async fn order_filled(&self, account_id: &str, symbol: &str, side: &str) {
        self.orders_filled_total
            .with_label_values(&[account_id, symbol, side])
            .inc();
    }

    // 交易指标更新方法
    pub async fn trade_executed(&self, trade: &TradeExecution) {
        let side_str = format!("{:?}", trade.side);
        
        self.trades_executed_total
            .with_label_values(&["unknown", &trade.symbol, &side_str])
            .inc();

        self.trade_volume_total
            .with_label_values(&[&trade.symbol, &side_str])
            .inc_by(trade.quantity);

        self.trade_value_total
            .with_label_values(&[&trade.symbol, "USDT"])
            .inc_by(trade.value);

        self.trade_fees_total
            .with_label_values(&["unknown", &trade.symbol])
            .inc_by(trade.fees);

        debug!(trade_id = %trade.trade_id, "Trade execution metrics updated");
    }

    // 账户指标更新方法
    pub async fn account_created(&self, account_id: &str) {
        self.accounts_active_total.inc();
        info!(account_id = %account_id, "Account created metric updated");
    }

    pub async fn account_deleted(&self, account_id: &str) {
        self.accounts_active_total.dec();
        info!(account_id = %account_id, "Account deleted metric updated");
    }

    pub async fn account_reset(&self, account_id: &str) {
        // 重置账户相关指标
        debug!(account_id = %account_id, "Account reset metrics updated");
    }

    pub async fn update_account_balance(&self, account_id: &str, currency: &str, balance: f64) {
        self.account_balance_total
            .with_label_values(&[account_id, currency])
            .set(balance as i64);
    }

    pub async fn update_account_pnl(&self, account_id: &str, realized_pnl: f64, unrealized_pnl: f64) {
        self.account_pnl_total
            .with_label_values(&[account_id, "realized"])
            .set(realized_pnl as i64);
        
        self.account_pnl_total
            .with_label_values(&[account_id, "unrealized"])
            .set(unrealized_pnl as i64);
    }

    // 风险指标更新方法
    pub async fn risk_violation_detected(&self, account_id: &str, violation: &RiskViolation) {
        let violation_type = format!("{:?}", violation.violation_type);
        let severity = format!("{:?}", violation.severity);
        
        self.risk_violations_total
            .with_label_values(&[account_id, &violation_type, &severity])
            .inc();
            
        warn!(
            account_id = %account_id,
            violation_type = %violation_type,
            severity = %severity,
            "Risk violation metric updated"
        );
    }

    pub async fn update_account_var(&self, account_id: &str, var_95: f64, var_99: f64) {
        self.account_var_current
            .with_label_values(&[account_id, "95"])
            .set(var_95 as i64);
        
        self.account_var_current
            .with_label_values(&[account_id, "99"])
            .set(var_99 as i64);
    }

    pub async fn update_margin_usage(&self, account_id: &str, usage_ratio: f64) {
        self.margin_usage_ratio
            .with_label_values(&[account_id])
            .set((usage_ratio * 100.0) as i64); // 转换为百分比
    }

    // 系统指标更新方法
    pub async fn update_system_uptime(&self) {
        let uptime = self.start_time.elapsed().as_secs();
        self.system_uptime_seconds.set(uptime as i64);
    }

    pub async fn market_data_update(&self, symbol: &str, data_type: &str) {
        self.market_data_updates_total
            .with_label_values(&[symbol, data_type])
            .inc();
    }

    pub async fn record_order_matching_latency(&self, latency: Duration) {
        self.order_matching_latency.observe(latency.as_secs_f64());
    }

    // 导出和查询方法
    pub async fn export_metrics(&self) -> Result<String> {
        self.update_system_uptime().await;
        
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        
        Ok(String::from_utf8(buffer)?)
    }

    pub async fn get_cached_metric(&self, key: &str) -> Option<f64> {
        let cache = self.cached_metrics.read().await;
        cache.get(key).copied()
    }

    pub async fn set_cached_metric(&self, key: String, value: f64) {
        let mut cache = self.cached_metrics.write().await;
        cache.insert(key, value);
    }

    // 统计查询方法
    pub async fn get_total_orders(&self) -> u64 {
        // 从Prometheus指标中获取总订单数
        // 这是一个简化的实现，实际应该从指标中查询
        if let Some(cached) = self.get_cached_metric("total_orders").await {
            cached as u64
        } else {
            0
        }
    }

    pub async fn get_total_trades(&self) -> u64 {
        if let Some(cached) = self.get_cached_metric("total_trades").await {
            cached as u64
        } else {
            0
        }
    }

    pub async fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    // 推送到外部系统
    pub async fn push_to_gateway(&self) -> Result<()> {
        if let Some(gateway_url) = &self.config.prometheus_pushgateway_url {
            let metrics_text = self.export_metrics().await?;
            
            let client = reqwest::Client::new();
            let url = format!(
                "{}/metrics/job/{}/instance/{}",
                gateway_url,
                self.config.job_name,
                self.config.instance_id
            );
            
            let response = client
                .post(&url)
                .header("Content-Type", "text/plain")
                .body(metrics_text)
                .send()
                .await?;
                
            if response.status().is_success() {
                *self.last_push.write().await = Instant::now();
                debug!("Successfully pushed metrics to gateway");
            } else {
                warn!("Failed to push metrics to gateway: {}", response.status());
            }
        }
        
        Ok(())
    }

    // 启动定期推送任务
    pub async fn start_push_task(&self) {
        if !self.config.enabled {
            return;
        }

        let metrics = Arc::new(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                Duration::from_secs(metrics.config.push_interval_seconds)
            );

            loop {
                interval.tick().await;
                
                if let Err(e) = metrics.push_to_gateway().await {
                    warn!(error = %e, "Failed to push metrics");
                }
            }
        });

        info!("Metrics push task started");
    }

    // 清理过期的缓存指标
    pub async fn cleanup_cache(&self) {
        let mut cache = self.cached_metrics.write().await;
        
        // 简单的清理策略：清空所有缓存
        // 实际实现中可能需要更复杂的过期策略
        if cache.len() > 10000 {
            cache.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_creation() {
        let metrics = ShadowTradingMetrics::new();
        assert!(metrics.is_ok());
    }

    #[tokio::test]
    async fn test_order_metrics() {
        let metrics = ShadowTradingMetrics::new().unwrap();
        
        metrics.order_submitted("test_order_1").await;
        metrics.order_cancelled("test_order_1").await;
        
        // 验证指标是否正确更新
        let exported = metrics.export_metrics().await.unwrap();
        assert!(exported.contains("shadow_orders_submitted_total"));
        assert!(exported.contains("shadow_orders_cancelled_total"));
    }

    #[tokio::test]
    async fn test_trade_metrics() {
        let metrics = ShadowTradingMetrics::new().unwrap();
        
        let trade = TradeExecution {
            trade_id: "test_trade".to_string(),
            order_id: "test_order".to_string(),
            symbol: "BTC/USDT".to_string(),
            side: crate::execution_engine::OrderSide::Buy,
            quantity: 1.0,
            price: 50000.0,
            value: 50000.0,
            fees: 50.0,
            executed_at: Utc::now(),
            metadata: HashMap::new(),
        };
        
        metrics.trade_executed(&trade).await;
        
        let exported = metrics.export_metrics().await.unwrap();
        assert!(exported.contains("shadow_trades_executed_total"));
        assert!(exported.contains("shadow_trade_volume_total"));
    }

    #[tokio::test]
    async fn test_account_metrics() {
        let metrics = ShadowTradingMetrics::new().unwrap();
        
        metrics.account_created("test_account").await;
        metrics.update_account_balance("test_account", "USDT", 10000.0).await;
        metrics.update_account_pnl("test_account", 500.0, -200.0).await;
        
        let exported = metrics.export_metrics().await.unwrap();
        assert!(exported.contains("shadow_accounts_active_total"));
        assert!(exported.contains("shadow_account_balance_total"));
        assert!(exported.contains("shadow_account_pnl_total"));
    }

    #[tokio::test]
    async fn test_system_metrics() {
        let metrics = ShadowTradingMetrics::new().unwrap();
        
        metrics.update_system_uptime().await;
        metrics.market_data_update("BTC/USDT", "price").await;
        
        let exported = metrics.export_metrics().await.unwrap();
        assert!(exported.contains("shadow_system_uptime_seconds"));
        assert!(exported.contains("shadow_market_data_updates_total"));
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let metrics = ShadowTradingMetrics::new().unwrap();
        
        let exported = metrics.export_metrics().await;
        assert!(exported.is_ok());
        
        let metrics_text = exported.unwrap();
        assert!(!metrics_text.is_empty());
        assert!(metrics_text.starts_with("# HELP"));
    }
}