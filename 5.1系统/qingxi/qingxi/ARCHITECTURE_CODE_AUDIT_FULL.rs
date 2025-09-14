//! Qingxi高性能行情数据采集与一致性模块——整体架构与核心代码审核专用文件
// 本文件用于代码审核，汇总了qingxi文件夹内所有核心高性能行情数据采集与一致性模块的主要架构与实现代码。
// 包含主入口(main.rs)、核心管道(pipeline.rs)、中心管理(central_manager.rs)、类型(types.rs)、适配器(adapters)、本地订单簿(orderbook/local_orderbook.rs)等关键模块。

// ===================== main.rs =====================
use qingxi_market_data::{
    adapters::{binance::BinanceAdapter, okx::OkxAdapter, huobi::HuobiAdapter},
    api_server,
    central_manager::{CentralManager, CentralManagerApi, CentralManagerHandle},
    errors::MarketDataError,
    observability,
    settings::Settings,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = Settings::load()?;
    observability::init_subscriber(&settings.general.log_level, "qingxi-market-data");
    let metrics_addr = format!("{}:{}", settings.api_server.host, settings.api_server.port + 1);
    if settings.general.metrics_enabled {
        observability::start_metrics_server(metrics_addr.parse()?)?;
    }
    let (readiness_tx, readiness_rx) = tokio::sync::watch::channel(false);
    let health_probe_addr = format!("{}:{}", settings.api_server.host, settings.api_server.port + 2).parse()?;
    observability::start_health_probe_server(health_probe_addr, Arc::new(readiness_rx.clone()));
    let (shutdown_tx, _) = broadcast::channel(1);
    let (manager, manager_handle) = CentralManager::new(&settings, shutdown_tx.subscribe());
    manager.register_adapter(Arc::new(BinanceAdapter::new()));
    manager.register_adapter(Arc::new(OkxAdapter::new()));
    manager.register_adapter(Arc::new(HuobiAdapter::new()));
    let mut tasks = tokio::task::JoinSet::new();
    let api_manager_handle: Arc<dyn CentralManagerApi> = Arc::new(manager_handle.clone());
    let manager_shutdown_rx = shutdown_tx.subscribe();
    tasks.spawn(async move {
        manager.run(readiness_tx, manager_shutdown_rx).await
    });
    let shutdown_signal_handle = shutdown_tx.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received.");
        let _ = shutdown_signal_handle.send(());
    });
    manager_handle.reconfigure(settings.sources.clone()).await?;
    info!("Waiting for system to become ready...");
    let mut readiness_check_rx = readiness_rx.clone();
    if tokio::time::timeout(Duration::from_secs(60), readiness_check_rx.wait_for(|ready| *ready)).await.is_err() {
        error!("System did not become ready within 60 seconds. Shutting down.");
        let _ = shutdown_tx.send(());
    } else {
        info!("System is ready. Starting gRPC API server...");
        let api_addr = settings.get_api_address().parse()?;
        tasks.spawn(async move {
            api_server::run_server(api_addr, api_manager_handle).await
        });
    }
    if let Some(res) = tasks.join_next().await {
        match res {
            Ok(Ok(_)) => info!("A critical task completed."),
            Ok(Err(e)) => error!(error = %e, "A critical task failed. Initiating shutdown."),
            Err(e) => error!(error = %e, "A critical task panicked. Initiating shutdown."),
        }
        let _ = shutdown_tx.send(());
    }
    tasks.shutdown().await;
    info!("Application has shut down gracefully.");
    Ok(())
}

// ===================== pipeline.rs =====================
// src/pipeline.rs
use crate::types::{ExchangeId, Symbol, OrderBook, ConsistencyThresholds, AnomalyType, AnomalySeverity};
use crate::orderbook::local_orderbook::{LocalOrderBook, MarketDataMessage};
use crate::errors::MarketDataError;
use crate::settings::Settings;
use crate::types::AnomalyDetectionResult;
use crate::events::SystemEvent;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use tracing::{instrument, warn, info, error};
use chrono::Utc;
use dashmap::DashMap;
use metrics;

// ===================== central_manager.rs =====================
// src/central_manager.rs
use crate::adapters::ExchangeAdapter;
use crate::anomaly::{AnomalyDetector, AnomalyThresholds};
use crate::cleaner::{BaseDataCleaner, DataCleaner};
use crate::collector::market_collector_system::MarketCollectorSystem;
use crate::errors::MarketDataApiError;
use crate::pipeline::DataPipeline;
use crate::types::*;
use crate::object_pool::ObjectPool;
use crate::settings::Settings;
use crate::orderbook::local_orderbook::MarketDataMessage;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, oneshot, Mutex, watch};
use tokio::task::JoinHandle;
use tokio::runtime::Handle;
use tracing::{error, info, instrument, warn, debug};
use std::collections::HashMap;

// ===================== types.rs =====================
// src/types.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use std::fmt;

// ===================== adapters/mod.rs =====================
//! # Exchange Adapters Module
// 提供与各个交易所API交互的适配器。
use crate::errors::MarketDataError;
use crate::orderbook::local_orderbook::MarketDataMessage;
use crate::types::SubscriptionDetail;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;
pub mod binance;
pub mod okx;
pub mod huobi;

// ===================== adapters/binance.rs =====================
//! # Binance交易所适配器
// 提供与Binance交易所API交互的适配器实现。
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::Message;
use ordered_float::OrderedFloat;
use chrono::Utc;
use simd_json;
use simd_json::prelude::*;
use crate::orderbook::local_orderbook::{MarketDataMessage, OrderBookUpdate};
use crate::adapters::ExchangeAdapter;
use crate::types::*;
use crate::errors::MarketDataError;
use crate::simd_utils::{JsonFieldLocator, JsonNumber};

// ===================== orderbook/local_orderbook.rs =====================
/// # Local Order Book
/// Manages an in-memory, sorted order book and applies incremental updates.
use crate::types::{ExchangeId, OrderBook, OrderBookEntry, Symbol, TradeUpdate};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;
use std::cmp::Reverse;
use tracing::warn;

pub struct LocalOrderBook {
    // ...existing fields...
}

impl LocalOrderBook {
    // ...existing methods...

    /// Applies a snapshot of the order book.
    pub fn apply_snapshot(&mut self, snapshot: OrderBook) {
        // ...existing code...
    }

    /// Applies an incremental update to the order book.
    pub fn apply_update(&mut self, update: OrderBookUpdate) {
        // ...existing code...
    }

    /// Gets the current best bid price.
    pub fn best_bid(&self) -> Option<OrderedFloat<f64>> {
        // ...existing code...
    }

    /// Gets the current best ask price.
    pub fn best_ask(&self) -> Option<OrderedFloat<f64>> {
        // ...existing code...
    }

    // ...other methods...
}

// 如需详细代码审核，请依次查阅各模块分区内容。
