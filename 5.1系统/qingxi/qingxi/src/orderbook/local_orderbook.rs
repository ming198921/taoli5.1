#![allow(dead_code)]
//! # Local Order Book
//! Manages an in-memory, sorted order book and applies incremental updates.

use crate::types::{OrderBook, OrderBookEntry, Symbol};
use ordered_float::OrderedFloat;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct OrderBookUpdate {
    pub symbol: Symbol,
    pub source: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub first_update_id: u64,
    pub final_update_id: u64,
}

#[derive(Debug, Clone)]
pub enum MarketDataMessage {
    OrderBook(OrderBook),
    OrderBookSnapshot(OrderBook),
    OrderBookUpdate(OrderBookUpdate),
    Trade(crate::types::TradeUpdate),
    Snapshot(crate::types::MarketDataSnapshot),
    Heartbeat {
        source: String,
        timestamp: crate::high_precision_time::Nanos,
    },
}

impl MarketDataMessage {
    pub fn symbol(&self) -> &Symbol {
        match self {
            Self::OrderBook(d) => &d.symbol,
            Self::OrderBookSnapshot(d) => &d.symbol,
            Self::OrderBookUpdate(d) => &d.symbol,
            Self::Trade(d) => &d.symbol,
            Self::Snapshot(snapshot) => {
                // 对于快照，如果有订单簿就返回订单簿的symbol，否则返回第一个交易的symbol
                if let Some(ref ob) = snapshot.orderbook {
                    &ob.symbol
                } else if let Some(trade) = snapshot.trades.first() {
                    &trade.symbol
                } else {
                    // 默认返回BTC/USDT
                    use std::sync::LazyLock;
                    static DEFAULT_SYMBOL: LazyLock<Symbol> =
                        LazyLock::new(|| Symbol::new("BTC", "USDT"));
                    &DEFAULT_SYMBOL
                }
            }
            Self::Heartbeat { .. } => {
                // 心跳消息没有symbol，返回默认值
                use std::sync::LazyLock;
                static DEFAULT_SYMBOL: LazyLock<Symbol> =
                    LazyLock::new(|| Symbol::new("UNKNOWN", "UNKNOWN"));
                &DEFAULT_SYMBOL
            }
        }
    }
    pub fn source(&self) -> &str {
        match self {
            Self::OrderBook(d) => &d.source,
            Self::OrderBookSnapshot(d) => &d.source,
            Self::OrderBookUpdate(d) => &d.source,
            Self::Trade(d) => &d.source,
            Self::Snapshot(snapshot) => &snapshot.source,
            Self::Heartbeat { source, .. } => source,
        }
    }
}

#[repr(C, align(64))]
#[derive(Debug, Clone)]
pub struct LocalOrderBook {
    pub symbol: Symbol,
    pub source: String,
    bids: BTreeMap<Reverse<OrderedFloat<f64>>, OrderedFloat<f64>>,
    asks: BTreeMap<OrderedFloat<f64>, OrderedFloat<f64>>,
    last_update_id: u64,
}

impl LocalOrderBook {
    pub fn new(snapshot: OrderBook) -> Self {
        let bids = snapshot
            .bids
            .into_iter()
            .map(
                |OrderBookEntry {
                     price: p,
                     quantity: q,
                 }| (Reverse(p), q),
            )
            .collect();
        let asks = snapshot
            .asks
            .into_iter()
            .map(
                |OrderBookEntry {
                     price: p,
                     quantity: q,
                 }| (p, q),
            )
            .collect();
        Self {
            symbol: snapshot.symbol,
            source: snapshot.source,
            bids,
            asks,
            last_update_id: snapshot.sequence_id.unwrap_or(0),
        }
    }

    pub fn apply_update(&mut self, update: OrderBookUpdate) -> bool {
        if self.last_update_id > 0 && update.first_update_id > self.last_update_id + 1 {
            warn!(
                "OrderBook update gap: {} -> {}",
                self.last_update_id, update.first_update_id
            );
            return false;
        }
        for OrderBookEntry { price, quantity } in update.bids {
            if quantity.into_inner() == 0.0 {
                self.bids.remove(&Reverse(price));
            } else {
                self.bids.insert(Reverse(price), quantity);
            }
        }
        for OrderBookEntry { price, quantity } in update.asks {
            if quantity.into_inner() == 0.0 {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, quantity);
            }
        }
        self.last_update_id = update.final_update_id;
        true
    }

    pub fn snapshot(&self, depth: usize) -> OrderBook {
        let bids = self
            .bids
            .iter()
            .take(depth)
            .map(|(p, q)| OrderBookEntry::new(p.0.into_inner(), q.into_inner()))
            .collect();
        let asks = self
            .asks
            .iter()
            .take(depth)
            .map(|(p, q)| OrderBookEntry::new(p.into_inner(), q.into_inner()))
            .collect();
        OrderBook {
            symbol: self.symbol.clone(),
            source: self.source.clone(),
            bids,
            asks,
            timestamp: crate::high_precision_time::Nanos::now(),
            sequence_id: Some(self.last_update_id),
            checksum: None,
        }
    }

    pub fn best_bid(&self) -> Option<OrderBookEntry> {
        self.bids
            .iter()
            .next()
            .map(|(p, q)| OrderBookEntry::new(p.0.into_inner(), q.into_inner()))
    }
    pub fn best_ask(&self) -> Option<OrderBookEntry> {
        self.asks
            .iter()
            .next()
            .map(|(p, q)| OrderBookEntry::new(p.into_inner(), q.into_inner()))
    }

    pub fn spread(&self) -> Option<f64> {
        if let (Some(bid), Some(ask)) = (self.best_bid(), self.best_ask()) {
            Some(ask.price.into_inner() - bid.price.into_inner())
        } else {
            None
        }
    }
}

use crate::types::TradeUpdate;

impl From<OrderBook> for MarketDataMessage {
    fn from(book: OrderBook) -> Self {
        MarketDataMessage::OrderBookSnapshot(book)
    }
}

impl From<OrderBookUpdate> for MarketDataMessage {
    fn from(update: OrderBookUpdate) -> Self {
        MarketDataMessage::OrderBookUpdate(update)
    }
}

impl From<TradeUpdate> for MarketDataMessage {
    fn from(trade: TradeUpdate) -> Self {
        MarketDataMessage::Trade(trade)
    }
}
