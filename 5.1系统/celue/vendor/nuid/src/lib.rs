pub fn new() -> String { next() }

pub fn next() -> String {
    // Simple timestamp-based unique id with counter fallback
    use std::sync::atomic::{AtomicU64, Ordering};
    static CNT: AtomicU64 = AtomicU64::new(0);
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let c = CNT.fetch_add(1, Ordering::Relaxed);
    format!("NUID{}{:016x}", ts, c)
} 