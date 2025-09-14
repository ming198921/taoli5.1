#![allow(dead_code)]
//! # Circuit Breaker Module
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    error_count: Arc<Mutex<u32>>,
    last_failure: Arc<Mutex<Option<Instant>>>,
    threshold: u32,
    open_duration: Duration,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, open_duration: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            error_count: Arc::new(Mutex::new(0)),
            last_failure: Arc::new(Mutex::new(None)),
            threshold,
            open_duration,
        }
    }

    pub fn call<F, T>(&self, mut f: F) -> Result<T, String>
    where
        F: FnMut() -> Result<T, String>,
    {
        let mut state = self.state.lock().expect("Failed to acquire mutex lock");
        if *state == CircuitState::Open {
            let last = *self.last_failure.lock().expect("Failed to acquire mutex lock");
            if let Some(last) = last {
                if last.elapsed() > self.open_duration {
                    *state = CircuitState::HalfOpen;
                } else {
                    return Err("Circuit open".to_string());
                }
            } else {
                return Err("Circuit open".to_string());
            }
        }
        drop(state);
        match f() {
            Ok(val) => {
                *self.error_count.lock().expect("Failed to acquire mutex lock") = 0;
                *self.state.lock().expect("Failed to acquire mutex lock") = CircuitState::Closed;
                Ok(val)
            }
            Err(e) => {
                let mut err_cnt = self.error_count.lock().expect("Failed to acquire mutex lock");
                *err_cnt += 1;
                if *err_cnt >= self.threshold {
                    *self.state.lock().expect("Failed to acquire mutex lock") = CircuitState::Open;
                    *self.last_failure.lock().expect("Failed to acquire mutex lock") = Some(Instant::now());
                }
                Err(e)
            }
        }
    }
}
