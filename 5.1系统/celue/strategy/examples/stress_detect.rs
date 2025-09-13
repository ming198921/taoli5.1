use std::{sync::Arc, time::{Duration, Instant}};
use strategy::{plugins::inter_exchange::InterExchangeStrategy, traits::ArbitrageStrategy, context::{StrategyContext, FeePrecisionRepoImpl}, min_profit::MinProfitModel, market_state::{AtomicMarketState, MarketState}};
use common::{NormalizedSnapshot, OrderBook, Symbol, Exchange, precision::{FixedPrice, FixedQuantity}};
use hdrhistogram::Histogram;
use crossbeam::channel;

fn build_snapshot() -> NormalizedSnapshot {
    let symbol = Symbol::new("BTCUSDT");
    let mut a = OrderBook::new(Exchange::new("a"), symbol.clone(), 0, 1);
    a.add_ask(FixedPrice::from_f64(100.0, 2), FixedQuantity::from_f64(1.0, 8));
    let mut b = OrderBook::new(Exchange::new("b"), symbol.clone(), 0, 1);
    b.add_bid(FixedPrice::from_f64(100.3, 2), FixedQuantity::from_f64(1.0, 8));
    NormalizedSnapshot { symbol, timestamp_ns: 0, exchanges: vec![a, b], weighted_mid_price: FixedPrice::from_f64(100.1, 2), total_bid_volume: FixedQuantity::from_f64(1.0, 8), total_ask_volume: FixedQuantity::from_f64(1.0, 8), quality_score: 1.0, sequence: Some(1) }
}

fn main() {
    let min_profit = Arc::new(MinProfitModel::new(20, 1.4, 2.5));
    let state = Arc::new(AtomicMarketState::new(MarketState::Regular));
    let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
    let paths = Arc::new(vec![]);
    let health_snapshot = Arc::new(adapters::health::HealthSnapshot::new());
    let ctx = StrategyContext::new(
        min_profit, 
        state, 
        None, 
        None, 
        None, 
        health_snapshot,
        None,
        fee_repo, 
        paths, 
        0.0005, 
        0.001, 
        100.0
    );
    let snap = build_snapshot();

    let secs: u64 = std::env::var("STRESS_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(3600);
    let threads: usize = std::env::var("STRESS_THREADS").ok().and_then(|s| s.parse().ok()).unwrap_or_else(|| num_cpus::get().max(2));
    let duration = Duration::from_secs(secs);
    let start = Instant::now();

    let (tx, rx) = channel::unbounded::<u64>();

    crossbeam::scope(|scope| {
        for _ in 0..threads {
            let tx = tx.clone();
            let ctx = ctx.clone();
            let strat = InterExchangeStrategy;
            let snap = snap.clone();
            scope.spawn(move |_| {
                let mut hist = Histogram::<u64>::new(3).unwrap();
                let mut last_log = Instant::now();
                let mut iters: u64 = 0;
                let mut last_iters: u64 = 0;
                while start.elapsed() < duration {
                    let t0 = Instant::now();
                    let _ = strat.detect(&ctx, &snap);
                    let ns = t0.elapsed().as_nanos() as u64; // record in ns
                    let _ = hist.record(ns);
                    iters += 1;
                    if last_log.elapsed() >= Duration::from_secs(10) {
                        let p50 = hist.value_at_quantile(0.50);
                        let p95 = hist.value_at_quantile(0.95);
                        let p99 = hist.value_at_quantile(0.99);
                        let elapsed = last_log.elapsed().as_secs_f64();
                        let delta = iters - last_iters;
                        let ops_per_sec = (delta as f64) / elapsed;
                        println!("[progress] elapsed={}s p50={}ns p95={}ns p99={}ns ops/s={:.0}", start.elapsed().as_secs(), p50, p95, p99, ops_per_sec);
                        last_iters = iters;
                        last_log = Instant::now();
                    }
                }
                // send summary values: p50/p95/p99 in ns
                let p50 = hist.value_at_quantile(0.50);
                let p95 = hist.value_at_quantile(0.95);
                let p99 = hist.value_at_quantile(0.99);
                let _ = tx.send(p50);
                let _ = tx.send(p95);
                let _ = tx.send(p99);
            });
        }
    }).unwrap();

    drop(tx);
    // Aggregate results across threads
    let mut p50s = Vec::new();
    let mut p95s = Vec::new();
    let mut p99s = Vec::new();
    let mut idx = 0;
    while let Ok(v) = rx.recv() {
        match idx % 3 {
            0 => p50s.push(v),
            1 => p95s.push(v),
            _ => p99s.push(v),
        }
        idx += 1;
    }
    let avg = |xs: &Vec<u64>| if xs.is_empty() { 0 } else { xs.iter().sum::<u64>() / xs.len() as u64 };
    let p50 = avg(&p50s);
    let p95 = avg(&p95s);
    let p99 = avg(&p99s);
    println!("STRESS inter_exchange_detect ns: p50={}, p95={}, p99={}", p50, p95, p99);
} 