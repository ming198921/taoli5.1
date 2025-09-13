//! Market state management for dynamic profit threshold adjustment

use std::sync::atomic::{AtomicU8, Ordering};

/// Market state representing current market conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MarketState {
    /// Normal market conditions
    Regular = 0,
    /// Increased volatility or reduced liquidity  
    Cautious = 1,
    /// Extreme conditions requiring maximum caution
    Extreme = 2,
}

impl From<u8> for MarketState {
    fn from(value: u8) -> Self {
        match value {
            0 => MarketState::Regular,
            1 => MarketState::Cautious,
            2 => MarketState::Extreme,
            _ => MarketState::Regular,
        }
    }
}

/// Thread-safe atomic market state
pub struct AtomicMarketState {
    state: AtomicU8,
}

impl AtomicMarketState {
    /// Create a new atomic market state
    pub fn new(initial: MarketState) -> Self {
        Self {
            state: AtomicU8::new(initial as u8),
        }
    }

    /// Get current market state
    pub fn get(&self) -> MarketState {
        MarketState::from(self.state.load(Ordering::Relaxed))
    }

    /// Set market state
    pub fn set(&self, state: MarketState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }

    /// Transition to a new state with validation
    pub fn transition(&self, new_state: MarketState) -> bool {
        let current = self.get();
        
        // Validate transition rules (example: can't jump from Regular to Extreme)
        let valid_transition = match (current, new_state) {
            (MarketState::Regular, MarketState::Cautious) => true,
            (MarketState::Cautious, MarketState::Regular) => true,
            (MarketState::Cautious, MarketState::Extreme) => true,
            (MarketState::Extreme, MarketState::Cautious) => true,
            (a, b) if a == b => true,
            _ => false,
        };

        if valid_transition {
            self.set(new_state);
        }
        
        valid_transition
    }
}

impl Default for AtomicMarketState {
    fn default() -> Self {
        Self::new(MarketState::Regular)
    }
}
