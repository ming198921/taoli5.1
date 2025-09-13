//! Market data structures and processing
//! As specified in section 6.1 of the design document

use crate::{precision::{FixedPrice, FixedQuantity}, types::{Exchange, Symbol}};
use serde::{Deserialize, Serialize};

/// Order book entry with fixed-point precision
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: FixedPrice,
    pub quantity: FixedQuantity,
}

impl OrderBookEntry {
    pub fn new(price: FixedPrice, quantity: FixedQuantity) -> Self {
        Self { price, quantity }
    }
    
    pub fn from_f64(price: f64, quantity: f64, price_scale: u8, qty_scale: u8) -> Self {
        Self {
            price: FixedPrice::from_f64(price, price_scale),
            quantity: FixedQuantity::from_f64(quantity, qty_scale),
        }
    }
}

/// Structure-of-Arrays orderbook layout for cache efficiency
/// Each level stored as separate arrays for better SIMD access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub timestamp_ns: u64,
    pub sequence: u64,
    
    // SOA layout - prices and quantities in separate arrays
    pub bid_prices: Vec<FixedPrice>,
    pub bid_quantities: Vec<FixedQuantity>,
    pub ask_prices: Vec<FixedPrice>,
    pub ask_quantities: Vec<FixedQuantity>,
    
    pub quality_score: f64,
    pub processing_latency_ns: u64,
}

/// Normalized snapshot combining multiple exchange orderbooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSnapshot {
    pub symbol: Symbol,
    pub timestamp_ns: u64,
    pub exchanges: Vec<OrderBook>,
    pub weighted_mid_price: FixedPrice,
    pub total_bid_volume: FixedQuantity,
    pub total_ask_volume: FixedQuantity,
    pub quality_score: f64,
    pub sequence: Option<u64>,
}

impl OrderBook {
    /// Create a new empty order book
    pub fn new(exchange: Exchange, symbol: Symbol, timestamp_ns: u64, sequence: u64) -> Self {
        Self {
            exchange,
            symbol,
            timestamp_ns,
            sequence,
            bid_prices: Vec::new(),
            bid_quantities: Vec::new(),
            ask_prices: Vec::new(),
            ask_quantities: Vec::new(),
            quality_score: 0.0,
            processing_latency_ns: 0,
        }
    }
    
    /// Add bid to order book
    pub fn add_bid(&mut self, price: FixedPrice, quantity: FixedQuantity) {
        // Insert in descending order (highest price first)
        let insert_pos = self.bid_prices.binary_search_by(|p| price.partial_cmp(p).unwrap()).unwrap_or_else(|pos| pos);
        self.bid_prices.insert(insert_pos, price);
        self.bid_quantities.insert(insert_pos, quantity);
    }

    /// Add ask to order book
    pub fn add_ask(&mut self, price: FixedPrice, quantity: FixedQuantity) {
        // Insert in ascending order (lowest price first)
        let insert_pos = self.ask_prices.binary_search_by(|p| p.partial_cmp(&price).unwrap()).unwrap_or_else(|pos| pos);
        self.ask_prices.insert(insert_pos, price);
        self.ask_quantities.insert(insert_pos, quantity);
    }

    /// Get best bid entry (highest buy price) - returns None if no bids
    pub fn best_bid_entry(&self) -> Option<OrderBookEntry> {
        if self.bid_prices.is_empty() {
            return None;
        }
        Some(OrderBookEntry::new(
            self.bid_prices[0], 
            self.bid_quantities[0]
        ))
    }
    
    /// Get best ask entry (lowest sell price) - returns None if no asks
    pub fn best_ask_entry(&self) -> Option<OrderBookEntry> {
        if self.ask_prices.is_empty() {
            return None;
        }
        Some(OrderBookEntry::new(
            self.ask_prices[0], 
            self.ask_quantities[0]
        ))
    }
    
    /// Get best bid (backward compatibility)
    pub fn best_bid(&self) -> Option<OrderBookEntry> {
        self.best_bid_entry()
    }

    /// Get best ask (backward compatibility)
    pub fn best_ask(&self) -> Option<OrderBookEntry> {
        self.best_ask_entry()
    }

    /// Calculate spread between best bid and ask
    pub fn spread(&self) -> Option<FixedPrice> {
        match (self.best_ask_entry(), self.best_bid_entry()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Exchange, Symbol};

    #[test]
    fn test_order_book_entry() {
        let price = FixedPrice::from_f64(100.5, 2);
        let qty = FixedQuantity::from_f64(10.0, 3);
        let entry = OrderBookEntry::new(price, qty);
        
        assert_eq!(entry.price.to_f64(), 100.5);
        assert_eq!(entry.quantity.to_f64(), 10.0);
    }

    #[test]
    fn test_order_book_best_bid() {
        let exchange = Exchange::new("binance");
        let symbol = Symbol::new("BTCUSDT");
        let mut ob = OrderBook::new(exchange, symbol, 1000000000, 1);
        
        let price = FixedPrice::from_f64(50000.0, 2);
        let qty = FixedQuantity::from_f64(1.0, 8);
        
        ob.add_bid(price, qty);
        
        assert_eq!(ob.best_bid().unwrap().price, price);
        assert_eq!(ob.best_bid().unwrap().quantity, qty);
    }

    #[test]
    fn test_spread_calculation() {
        let exchange = Exchange::new("binance");
        let symbol = Symbol::new("BTCUSDT");
        let mut ob = OrderBook::new(exchange, symbol, 1000000000, 1);
        
        let bid_price = FixedPrice::from_f64(50000.0, 2);
        let ask_price = FixedPrice::from_f64(50001.0, 2);
        let qty = FixedQuantity::from_f64(1.0, 8);
        
        ob.add_bid(bid_price, qty);
        ob.add_ask(ask_price, qty);
        
        let spread = ob.spread().unwrap();
        assert_eq!(spread.to_f64(), 1.0);
    }
}
