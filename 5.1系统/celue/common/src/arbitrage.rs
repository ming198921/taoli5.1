//! Defines the data structures for representing arbitrage opportunities.

use crate::precision::{FixedPrice, FixedQuantity};
use crate::types::{Exchange, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents the side of an order (buy or sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

/// Represents one leg of an arbitrage trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageLeg {
    pub exchange: Exchange,
    pub symbol: Symbol,
    pub side: Side,
    pub price: FixedPrice,
    pub quantity: FixedQuantity,
    pub cost: FixedPrice, // Total cost/proceeds for this leg (price * quantity)
}

/// Represents a detected arbitrage opportunity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageOpportunity {
    pub id: Uuid,
    pub strategy_name: String,
    pub legs: Vec<ArbitrageLeg>,
    /// The raw profit calculated from buy/sell costs, before fees.
    pub gross_profit: FixedPrice,
    /// The estimated net profit after deducting fees.
    pub net_profit: FixedPrice,
    /// The estimated net profit as a percentage of the total investment.
    pub net_profit_pct: FixedPrice,
    /// The timestamp (in ns) when the opportunity was created.
    pub created_at_ns: u64,
    /// Time-to-live for this opportunity in nanoseconds.
    pub ttl_ns: u64,
    /// Arbitrary metadata for audit/tracing
    pub tags: HashMap<String, String>,
}

impl ArbitrageOpportunity {
    /// Creates a new inter-exchange arbitrage opportunity.
    pub fn new_inter_exchange(
        strategy_name: &str,
        buy_leg: ArbitrageLeg,
        sell_leg: ArbitrageLeg,
        net_profit: FixedPrice,
        net_profit_pct: FixedPrice,
        clock_time_ns: u64,
    ) -> Self {
        // Basic validation
        assert_eq!(buy_leg.side, Side::Buy);
        assert_eq!(sell_leg.side, Side::Sell);
        assert_eq!(buy_leg.symbol, sell_leg.symbol);

        let gross_profit = sell_leg.cost - buy_leg.cost;

        Self {
            id: Uuid::new_v4(),
            strategy_name: strategy_name.to_string(),
            legs: vec![buy_leg, sell_leg],
            gross_profit,
            net_profit,
            net_profit_pct,
            created_at_ns: clock_time_ns,
            ttl_ns: 150_000_000, // 150ms, as per documentation
            tags: HashMap::new(),
        }
    }

    /// Creates a new opportunity from arbitrary legs (2 or 3+), computing gross from legs cost signs.
    pub fn new_with_legs(
        strategy_name: &str,
        legs: Vec<ArbitrageLeg>,
        net_profit: FixedPrice,
        net_profit_pct: FixedPrice,
        clock_time_ns: u64,
    ) -> Self {
        assert!(legs.len() >= 2);
        // Convention: Buy legs cost are negative cash flow, Sell legs cost are positive proceeds
        let mut gross = FixedPrice::from_raw(0, legs[0].price.scale());
        for leg in &legs {
            match leg.side {
                Side::Buy => { gross = gross - leg.cost; }
                Side::Sell => { gross = gross + leg.cost; }
            }
        }
        Self {
            id: Uuid::new_v4(),
            strategy_name: strategy_name.to_string(),
            legs,
            gross_profit: gross,
            net_profit,
            net_profit_pct,
            created_at_ns: clock_time_ns,
            ttl_ns: 150_000_000,
            tags: HashMap::new(),
        }
    }
}
