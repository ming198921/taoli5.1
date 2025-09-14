#![allow(dead_code)]
use crate::{
    anomaly::AnomalyDetector,
    orderbook::local_orderbook::{LocalOrderBook, MarketDataMessage},
    settings::Settings,
    types::*,
};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

#[derive(Debug, Default)]
pub struct ProcessResult {
    pub snapshot: Option<MarketDataSnapshot>,
    pub anomaly: Option<AnomalyDetectionResult>,
}

impl ProcessResult {
    pub fn new() -> Self {
        Self {
            snapshot: None,
            anomaly: None,
        }
    }

    pub fn with_snapshot(snapshot: MarketDataSnapshot) -> Self {
        Self {
            snapshot: Some(snapshot),
            anomaly: None,
        }
    }

    pub fn with_anomaly(mut self, anomaly: AnomalyDetectionResult) -> Self {
        self.anomaly = Some(anomaly);
        self
    }
}

// type_complexity: 定义类型别名
pub type LocalBookMap = Arc<DashMap<(String, Symbol), Arc<Mutex<LocalOrderBook>>>>;

pub struct DataPipeline {
    local_books: LocalBookMap,
    #[allow(dead_code)]
    anomaly_detector: AnomalyDetector,
    snapshot_pool: Option<Arc<crate::object_pool::ObjectPool<MarketDataSnapshot>>>,
}

impl DataPipeline {
    pub fn new(settings: &Settings) -> Self {
        Self {
            local_books: Arc::new(DashMap::new()),
            anomaly_detector: AnomalyDetector::new(settings),
            snapshot_pool: None,
        }
    }

    pub fn with_snapshot_pool(
        mut self,
        pool: Arc<crate::object_pool::ObjectPool<MarketDataSnapshot>>,
    ) -> Self {
        self.snapshot_pool = Some(pool);
        self
    }

    pub async fn process(&self, msg: MarketDataMessage) -> ProcessResult {
        let key = (msg.source().to_string(), msg.symbol().clone());
        match msg {
            MarketDataMessage::OrderBook(orderbook) => {
                info!(source = %orderbook.source, symbol = %orderbook.symbol.as_pair(), "Processing OrderBook as snapshot.");

                // 检测订单簿异常
                let anomaly = self.anomaly_detector.detect_orderbook_anomalies(&orderbook);

                let local_book = Arc::new(Mutex::new(LocalOrderBook::new(orderbook)));
                self.local_books.insert(key, local_book);

                if let Some(anomaly) = anomaly {
                    ProcessResult::new().with_anomaly(anomaly)
                } else {
                    ProcessResult::new()
                }
            }
            MarketDataMessage::OrderBookSnapshot(snapshot) => {
                info!(source = %snapshot.source, symbol = %snapshot.symbol.as_pair(), "Initializing local book from snapshot.");

                // 检测订单簿异常
                let anomaly = self.anomaly_detector.detect_orderbook_anomalies(&snapshot);

                let local_book = Arc::new(Mutex::new(LocalOrderBook::new(snapshot)));
                self.local_books.insert(key, local_book);

                if let Some(anomaly) = anomaly {
                    ProcessResult::new().with_anomaly(anomaly)
                } else {
                    ProcessResult::new()
                }
            }
            MarketDataMessage::OrderBookUpdate(update) => {
                if let Some(book_ref) = self.local_books.get(&key) {
                    let mut book = book_ref.lock().await;
                    if book.apply_update(update) {
                        let book_snapshot = book.snapshot(50);

                        // 检测订单簿异常
                        let anomaly = self
                            .anomaly_detector
                            .detect_orderbook_anomalies(&book_snapshot);

                        // 成功应用更新，使用对象池创建快照
                        let market_snapshot = if let Some(pool) = &self.snapshot_pool {
                            let mut snapshot = pool.get().await;
                            // 重置并填充快照数据
                            snapshot.orderbook = Some(book_snapshot);
                            snapshot.trades.clear();
                            snapshot.timestamp = crate::high_precision_time::Nanos::now();
                            snapshot.source = key.0.clone();
                            snapshot
                        } else {
                            // 回退到直接创建
                            MarketDataSnapshot {
                                orderbook: Some(book_snapshot),
                                trades: vec![],
                                timestamp: crate::high_precision_time::Nanos::now(),
                                source: key.0.clone(),
                            }
                        };

                        let mut result = ProcessResult::with_snapshot(market_snapshot);
                        if let Some(anomaly) = anomaly {
                            result = result.with_anomaly(anomaly);
                        }
                        result
                    } else {
                        // 序列号不匹配，需要触发重同步逻辑 (暂未实现)
                        warn!(source = %key.0, symbol = %key.1.as_pair(), "Sequence gap detected, resync needed.");
                        ProcessResult::new()
                    }
                } else {
                    warn!(source = %key.0, symbol = %key.1.as_pair(), "Received update for uninitialized book.");
                    ProcessResult::new()
                }
            }
            MarketDataMessage::Trade(_trade) => {
                // 可以处理交易数据
                ProcessResult::new()
            }
            MarketDataMessage::Snapshot(snapshot) => {
                // 处理快照消息
                if let Some(orderbook) = snapshot.orderbook {
                    info!(source = %snapshot.source, "Processing snapshot with orderbook");

                    // 检测订单簿异常
                    let anomaly = self.anomaly_detector.detect_orderbook_anomalies(&orderbook);

                    let local_book = Arc::new(Mutex::new(LocalOrderBook::new(orderbook)));
                    self.local_books.insert(key, local_book);

                    if let Some(anomaly) = anomaly {
                        ProcessResult::new().with_anomaly(anomaly)
                    } else {
                        ProcessResult::new()
                    }
                } else {
                    ProcessResult::new()
                }
            }
            MarketDataMessage::Heartbeat {
                source,
                timestamp: _,
            } => {
                // 处理心跳消息，暂时忽略
                debug!(source = %source, "Received heartbeat message");
                ProcessResult::new()
            }
        }
    }
}
