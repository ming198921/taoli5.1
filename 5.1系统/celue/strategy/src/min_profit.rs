//! Minimum profit threshold management with market state adaptivity

use common::precision::FixedPrice;
use crate::market_state::MarketState;

/// Configuration for market state weights
#[derive(Debug, Clone)]
pub struct MarketStateWeights {
    pub regular: f64,
    pub cautious: f64,
    pub extreme: f64,
}

impl Default for MarketStateWeights {
    fn default() -> Self {
        Self {
            // 从环境变量加载权重，避免硬编码
            regular: std::env::var("CELUE_MARKET_STATE_WEIGHT_REGULAR")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(1.0), // 基础权重
            cautious: std::env::var("CELUE_MARKET_STATE_WEIGHT_CAUTIOUS")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(2.0), // 保守：更高权重
            extreme: std::env::var("CELUE_MARKET_STATE_WEIGHT_EXTREME")
                .ok().and_then(|s| s.parse().ok())
                .unwrap_or(3.0), // 保守：最高权重
        }
    }
}

/// Manages the minimum profit threshold based on market conditions.
#[derive(Debug)]
pub struct MinProfitModel {
    /// Base profit threshold in basis points (1 bps = 0.01%).
    base_min_profit_bps: u32,
    /// Weight multiplier for Cautious market state.
    cautious_weight: f64,
    /// Weight multiplier for Extreme market state.
    extreme_weight: f64,
}

impl MinProfitModel {
    pub fn new(base_min_profit_bps: u32, cautious_weight: f64, extreme_weight: f64) -> Self {
        Self {
            base_min_profit_bps,
            cautious_weight,
            extreme_weight,
        }
    }

    /// Calculates the current minimum profit threshold as a percentage.
    pub fn get_threshold_pct(&self, state: MarketState) -> FixedPrice {
        let weight = match state {
            MarketState::Regular => 1.0,
            MarketState::Cautious => self.cautious_weight,
            MarketState::Extreme => self.extreme_weight,
        };

        let dynamic_profit_bps = (self.base_min_profit_bps as f64 * weight) as u32;

        // 1 basis point = 0.0001.
        // We use a scale of 6 for high precision percentages.
        // 1 bps = 100 raw value with scale 6.
        FixedPrice::from_raw((dynamic_profit_bps * 100) as i64, 6)
    }
}
