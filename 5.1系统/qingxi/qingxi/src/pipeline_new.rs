#![allow(dead_code)]
use crate::{anomaly::AnomalyDetector, orderbook::local_orderbook::{LocalOrderBook, MarketDataMessage}, types::*, settings::Settings};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct DataPipeline {
    local_books: Arc<DashMap<(String, Symbol), Arc<Mutex<LocalOrderBook>>>>,
    anomaly_detector: AnomalyDetector,
}

impl DataPipeline {
    pub fn new(settings: &Settings) -> Self {
        Self {
            local_books: Arc::new(DashMap::new()),
            anomaly_detector: AnomalyDetector::new(settings),
        }
    }

    pub async fn process(&self, msg: MarketDataMessage) -> Option<MarketDataSnapshot> {
        let key = (msg.source().to_string(), msg.symbol().clone());
        match msg {
            MarketDataMessage::OrderBookSnapshot(snapshot) => {
                info!(source = %snapshot.source, symbol = %snapshot.symbol.as_pair(), "Initializing local book from snapshot.");
                let local_book = Arc::new(Mutex::new(LocalOrderBook::new(snapshot)));
                self.local_books.insert(key, local_book);
            },
            MarketDataMessage::OrderBookUpdate(update) => {
                if let Some(book_ref) = self.local_books.get(&key) {
                    let mut book = book_ref.lock().await;
                    if book.apply_update(update) {
                        // 成功应用更新，可以生成一个快照用于后续处理
                        return Some(MarketDataSnapshot {
                            orderbook: Some(book.snapshot(50)),
                            trades: vec![],
                            timestamp: chrono::Utc::now().timestamp_millis(),
                            source: key.0.clone(),
                        });
                    } else {
                        // 序列号不匹配，需要触发重同步逻辑 (暂未实现)
                        warn!(source = %key.0, symbol = %key.1.as_pair(), "Sequence gap detected, resync needed.");
                    }
                } else {
                    warn!(source = %key.0, symbol = %key.1.as_pair(), "Received update for uninitialized book.");
                }
            },
            MarketDataMessage::Trade(_trade) => {
                // 可以处理交易数据
            },
        }
        None
    }
}
