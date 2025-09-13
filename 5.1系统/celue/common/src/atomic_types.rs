//! Atomic types for lock-free hot path operations
//! Section 6.1 prohibits locks in hot path - use atomic operations instead

use atomic::{Atomic, Ordering};
use std::sync::Arc;
use crate::MarketState;

/// Atomic market state for lock-free updates
#[derive(Debug)]
pub struct AtomicMarketState {
    state: Atomic<u8>,
}

impl AtomicMarketState {
    pub fn new(initial_state: MarketState) -> Self {
        Self {
            state: Atomic::new(initial_state as u8),
        }
    }
    
    pub fn load(&self) -> MarketState {
        let value = self.state.load(Ordering::Relaxed);
        match value {
            0 => MarketState::Regular,
            1 => MarketState::Cautious,
            2 => MarketState::Extreme,
            _ => MarketState::Regular, // Default fallback
        }
    }
    
    pub fn store(&self, state: MarketState) {
        self.state.store(state as u8, Ordering::Relaxed);
    }
    
    pub fn compare_exchange(&self, current: MarketState, new: MarketState) -> Result<MarketState, MarketState> {
        match self.state.compare_exchange(current as u8, new as u8, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => Ok(new),
            Err(actual) => Err(match actual {
                0 => MarketState::Regular,
                1 => MarketState::Cautious,
                2 => MarketState::Extreme,
                _ => MarketState::Regular,
            }),
        }
    }
}

/// Atomic counter for metrics (hot path safe)
#[derive(Debug)]
pub struct AtomicCounter {
    value: Atomic<u64>,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            value: Atomic::new(0),
        }
    }
    
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::Relaxed)
    }
    
    pub fn add(&self, delta: u64) -> u64 {
        self.value.fetch_add(delta, Ordering::Relaxed)
    }
    
    pub fn load(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
    
    pub fn reset(&self) -> u64 {
        self.value.swap(0, Ordering::Relaxed)
    }
}

impl Default for AtomicCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Atomic boolean for feature flags
#[derive(Debug)]
pub struct AtomicFlag {
    flag: Atomic<bool>,
}

impl AtomicFlag {
    pub fn new(initial: bool) -> Self {
        Self {
            flag: Atomic::new(initial),
        }
    }
    
    pub fn is_set(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
    
    pub fn set(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
    
    pub fn clear(&self) {
        self.flag.store(false, Ordering::Relaxed);
    }
    
    pub fn toggle(&self) -> bool {
        let old = self.flag.load(Ordering::Relaxed);
        self.flag.store(!old, Ordering::Relaxed);
        !old
    }
}

/// Shared reference to atomic market state
pub type SharedMarketState = Arc<AtomicMarketState>;

/// Create a new shared market state
pub fn new_shared_market_state(initial: MarketState) -> SharedMarketState {
    Arc::new(AtomicMarketState::new(initial))
}
