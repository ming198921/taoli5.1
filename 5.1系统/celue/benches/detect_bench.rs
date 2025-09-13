use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use std::sync::Arc;
use strategy::{plugins::inter_exchange::InterExchangeStrategy, traits::ArbitrageStrategy, context::{StrategyContext, FeePrecisionRepo}, min_profit::{MinProfitModel, AtomicMarketState, MarketState}};
use common::{market_data::{OrderBook, NormalizedSnapshot}, precision::{FixedPrice, FixedQuantity}, types::{Exchange, Symbol}};

fn build_snapshot() -> NormalizedSnapshot {
    let symbol = Symbol::new("BTCUSDT");
    let mut a = OrderBook::new(Exchange::new("a"), symbol.clone(), 0, 1);
    a.add_ask(FixedPrice::from_f64(100.0, 2), FixedQuantity::from_f64(1.0, 8));
    let mut b = OrderBook::new(Exchange::new("b"), symbol.clone(), 0, 1);
    b.add_bid(FixedPrice::from_f64(100.3, 2), FixedQuantity::from_f64(1.0, 8));
    NormalizedSnapshot { symbol, timestamp_ns: 0, exchanges: vec![a, b], weighted_mid_price: FixedPrice::from_f64(100.1, 2), total_bid_volume: FixedQuantity::from_f64(1.0, 8), total_ask_volume: FixedQuantity::from_f64(1.0, 8), quality_score: 1.0, sequence: Some(1) }
}

fn bench_detect(c: &mut Criterion) {
    let min_profit = Arc::new(MinProfitModel::new(20, 1.4, 2.5));
    let state = Arc::new(AtomicMarketState::new(MarketState::Regular));
    let fee_repo = Arc::new(FeePrecisionRepo::default());
    let paths = Arc::new(vec![]);
    let ctx = StrategyContext::new(min_profit, state, None, None, None, fee_repo, paths, 0.0005, 0.001, 100.0);
    let strat = InterExchangeStrategy;

    c.bench_function("inter_exchange_detect", |b| {
        b.iter_batched(
            || build_snapshot(),
            |snap| { let _ = strat.detect(&ctx, &snap); },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench_detect);
criterion_main!(benches); 