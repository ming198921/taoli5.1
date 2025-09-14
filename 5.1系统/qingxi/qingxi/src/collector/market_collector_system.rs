#![allow(dead_code)]
// src/collector/market_collector_system.rs
use crate::collector::websocket_collector::WebsocketCollector;
use crate::health::ApiHealthMonitor;
use crate::types::{MarketSourceConfig, Symbol};
use crate::{adapters::ExchangeAdapter, errors::MarketDataError};
use crate::settings::WebSocketNetworkSettings;
use dashmap::DashMap;
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{info, instrument, warn};

/// 市场数据采集系统
pub struct MarketCollectorSystem {
    /// 数据输出通道
    data_tx: flume::Sender<crate::types::AdapterEvent>,
    /// 适配器列表
    adapters: Arc<DashMap<String, Arc<dyn ExchangeAdapter>>>,
    /// 活跃的采集任务
    active_tasks: Arc<DashMap<(String, Symbol), JoinHandle<()>>>,
    /// 健康监控器
    health_monitor: Arc<ApiHealthMonitor>,
    /// 关闭信号发送器
    shutdown_tx: broadcast::Sender<()>,
    /// WebSocket网络设置
    network_settings: WebSocketNetworkSettings,
}

impl MarketCollectorSystem {
    /// 创建新的市场数据采集系统
    pub fn new(
        data_tx: flume::Sender<crate::types::AdapterEvent>,
        health_monitor: Arc<ApiHealthMonitor>,
        network_settings: WebSocketNetworkSettings,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            data_tx,
            adapters: Arc::new(DashMap::new()),
            active_tasks: Arc::new(DashMap::new()),
            health_monitor,
            shutdown_tx,
            network_settings,
        }
    }

    /// 注册交易所适配器
    pub fn register_adapter(&self, adapter: Arc<dyn ExchangeAdapter>) {
        info!(exchange = %adapter.exchange_id(), "Registering adapter");
        self.adapters
            .insert(adapter.exchange_id().to_string(), adapter);
    }

    #[instrument(skip_all, fields(num_configs = configs.len()))]
    /// 智能配置更新 - 实现差量更新逻辑
    pub async fn reconfigure(
        &self,
        configs: Vec<MarketSourceConfig>,
    ) -> Result<(), MarketDataError> {
        info!("🔄 Starting intelligent configuration reconfigure");

        // 1. 构建新的活跃订阅集合
        let new_active_subs: HashMap<(String, Symbol), MarketSourceConfig> = configs
            .into_iter()
            .filter(|c| !c.channel.is_empty()) // channel 现在是 String，不是 Option
            .flat_map(|c| {
                let exchange = c.exchange_id.clone();
                let config = c.clone();
                // 需要将字符串符号转换为 Symbol 结构体
                c.get_symbols().unwrap_or_default().into_iter().map(move |symbol| {
                    let key = (exchange.clone(), symbol);
                    (key, config.clone())
                })
            })
            .collect();

        // 2. 获取当前活跃的订阅列表
        let current_subs: Vec<(String, Symbol)> = self
            .active_tasks
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        info!(
            "📊 Current subscriptions: {}, New subscriptions: {}",
            current_subs.len(),
            new_active_subs.len()
        );

        // 3. 对比新旧配置，找出需要停止的订阅
        let mut stopped_count = 0;
        for current_key in &current_subs {
            if !new_active_subs.contains_key(current_key) {
                if let Some((_, handle)) = self.active_tasks.remove(current_key) {
                    info!(
                        "🛑 Stopping collector for {}-{} (no longer in config)",
                        current_key.0,
                        current_key.1.as_pair()
                    );
                    handle.abort();
                    stopped_count += 1;

                    // 更新健康监控状态
                    let source_id = format!("{}-{}", current_key.0, current_key.1.as_pair());
                    self.health_monitor
                        .update_connection_status(&source_id, false);
                }
            }
        }

        // 4. 为新出现的订阅启动采集器
        let mut started_count = 0;
        let mut unchanged_count = 0;

        for (new_key, config) in new_active_subs {
            if current_subs.contains(&new_key) {
                // 订阅已存在且配置未改变，不做任何操作
                unchanged_count += 1;
                continue;
            }

            // 检查是否有对应的适配器
            if let Some(adapter) = self.adapters.get(&new_key.0) {
                let _source_id = format!("{}-{}", new_key.0, new_key.1.as_pair());
                let health_monitor = self.health_monitor.clone();
                let data_tx = self.data_tx.clone();
                let adapter_clone = adapter.clone();
                let symbol = new_key.1.clone();
                let shutdown_rx = self.shutdown_tx.subscribe();
                let _network_settings = self.network_settings.clone();

                // 使用传入的配置而不是硬编码的URL
                let market_config = MarketSourceConfig {
                    id: format!("{}_{}", new_key.0, symbol.as_combined()),
                    exchange_id: new_key.0.clone(),
                    adapter_type: config.adapter_type.clone(),
                    enabled: true,
                    symbols: vec![symbol.to_string()], // 转换为字符串
                    websocket_url: config.get_websocket_url().to_string(),
                    rest_api_url: config.get_rest_api_url().map(|s| s.to_string()),
                    channel: config.channel.clone(), // 现在是必需的 String
                    
                    // API 凭据
                    api_key: config.api_key.clone(),
                    api_secret: config.api_secret.clone(),
                    api_passphrase: config.api_passphrase.clone(),
                    
                    // 可选字段
                    rate_limit: config.rate_limit,
                    connection_timeout_ms: config.connection_timeout_ms,
                    heartbeat_interval_ms: config.heartbeat_interval_ms,
                    reconnect_interval_sec: config.reconnect_interval_sec.or(Some(5)),
                    max_reconnect_attempts: config.max_reconnect_attempts.or(Some(3)),
                };

                // 在spawn之前克隆网络设置
                let network_settings_clone = self.network_settings.clone();

                // 启动真实的WebSocket采集器任务
                let handle = tokio::spawn(async move {
                    info!(
                        "🚀 Starting real WebSocket collector for {}-{}",
                        adapter_clone.exchange_id(),
                        symbol.as_pair()
                    );

                    // 创建一个内部通道来处理类型转换 
                    // 从配置中读取通道大小，支持环境变量覆盖
                    let channel_size = if let Ok(settings) = crate::settings::Settings::load() {
                        settings.performance.internal_channel_size
                    } else {
                        std::env::var("QINGXI_PERFORMANCE__INTERNAL_CHANNEL_SIZE")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1000)
                    };
                    let (internal_tx, internal_rx) = flume::bounded(channel_size);
                    
                    // 启动类型转换任务
                    let data_tx_clone = data_tx.clone();
                    let conversion_task = tokio::spawn(async move {
                        while let Ok(market_msg) = internal_rx.recv_async().await {
                            let adapter_event = crate::types::AdapterEvent::MarketData(market_msg);
                            if data_tx_clone.send_async(adapter_event).await.is_err() {
                                break;
                            }
                        }
                    });

                    // 创建WebSocket采集器并运行
                    let collector = WebsocketCollector::new(
                        market_config,
                        adapter_clone,
                        internal_tx,
                        health_monitor,
                        network_settings_clone,
                    );

                    // 并行运行采集器和转换任务
                    tokio::select! {
                        _ = collector.run(shutdown_rx) => {},
                        _ = conversion_task => {},
                    }
                });

                info!(
                    "✅ Started collector for {}-{} (new subscription)",
                    new_key.0,
                    new_key.1.as_pair()
                );
                self.active_tasks.insert(new_key, handle);
                started_count += 1;
            } else {
                warn!(
                    "⚠️ No adapter registered for exchange '{}', cannot start collector",
                    new_key.0
                );
            }
        }

        info!(
            "🎯 Configuration update summary: {} stopped, {} started, {} unchanged",
            stopped_count, started_count, unchanged_count
        );

        Ok(())
    }

    // DashMap 没有 drain 方法，需手动遍历并移除所有条目
    /// 停止所有采集器 - 使用优雅关闭
    pub async fn stop_all(&self) {
        info!("🛑 Initiating graceful shutdown of all collectors");
        
        // 发送关闭信号给所有采集器
        if let Err(e) = self.shutdown_tx.send(()) {
            warn!("Failed to send shutdown signal: {}", e);
        }
        
        // 等待所有任务完成，带超时
        let mut handles = Vec::new();
        let keys: Vec<_> = self
            .active_tasks
            .iter()
            .map(|entry| entry.key().clone())
            .collect();
            
        for key in keys {
            if let Some((_, handle)) = self.active_tasks.remove(&key) {
                handles.push(handle);
            }
        }
        
        // 等待所有任务完成，最多等待30秒
        let timeout_duration = std::time::Duration::from_secs(30);
        match tokio::time::timeout(timeout_duration, async {
            for handle in handles {
                let _ = handle.await;
            }
        }).await {
            Ok(_) => info!("✅ All collectors stopped gracefully"),
            Err(_) => warn!("⚠️ Some collectors did not stop within timeout, forcing shutdown"),
        }
    }
}
