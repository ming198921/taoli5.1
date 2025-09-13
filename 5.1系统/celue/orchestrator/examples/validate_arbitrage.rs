use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::time;

use common::{
    market_data::{NormalizedSnapshot, OrderBook},
    precision::{FixedPrice, FixedQuantity},
    types::{Exchange, Symbol},
};
use strategy::{
    context::{FeePrecisionRepoImpl, StrategyContext, TrianglePathCfg},
    market_state::{AtomicMarketState, MarketState},
    min_profit::MinProfitModel,
    traits::ArbitrageStrategy,
};
use strategy::plugins::inter_exchange::InterExchangeStrategy;
use strategy::plugins::triangular::TriangularStrategy;
use adapters::config::DynamicConfig;
use adapters::health::HealthSnapshot;
use parking_lot::RwLock;
use rand::Rng;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    println!("=== 策略模块套利识别验证测试 ===");
    println!("测试时长: 15分钟");
    println!("数据压力: 目标每秒≈10万条");
    println!("策略类型: 跨交易所套利 + 三角套利\n");

    // 1) 构建最小运行上下文（不依赖外部 NATS/metrics/risk/funds）
    let min_profit_model = Arc::new(MinProfitModel::new(
        2,   // 基础阈值=2bps=0.02%
        1.0, // cautious 权重
        1.0, // extreme 权重
    ));
    let market_state = Arc::new(AtomicMarketState::new(MarketState::Regular));

    // Fee/precision 仓库（本示例仅用于检测验证，生产中由外部动态注入）
    let mut fee_map = std::collections::HashMap::new();
    fee_map.insert("binance".to_string(), 0.0);
    fee_map.insert("okx".to_string(), 0.0);
    fee_map.insert("huobi".to_string(), 0.0);

    let mut tick_map = std::collections::HashMap::new();
    tick_map.insert("BTCUSDT".to_string(), 0.01);
    tick_map.insert("ETHUSDT".to_string(), 0.01);
    tick_map.insert("BTCETH".to_string(), 0.0001);

    let mut step_map = std::collections::HashMap::new();
    step_map.insert("BTCUSDT".to_string(), 0.001);
    step_map.insert("ETHUSDT".to_string(), 0.001);
    step_map.insert("BTCETH".to_string(), 0.000001);

    let fee_repo = Arc::new(FeePrecisionRepoImpl {
        dynamic: Arc::new(RwLock::new(Some(DynamicConfig {
            version: 1,
            data: serde_json::json!({}),
            fee_bps_per_exchange: Some(fee_map),
            price_scale_per_symbol: None,
            qty_scale_per_symbol: None,
            tick_size_per_symbol: Some(tick_map),
            step_size_per_symbol: Some(step_map),
        })) ),
    });

    // Triangular 路径：BTC->ETH->USDT 于 binance
    let triangular_paths = Arc::new(vec![TrianglePathCfg {
        base: "BTC".to_string(),
        intermediate: "ETH".to_string(),
        quote: "USDT".to_string(),
        exchange: "binance".to_string(),
        min_liquidity_usd: Some(10.0),
        max_slippage_per_leg: Some(0.0),
    }]);

    let context = Arc::new(StrategyContext::new(
        min_profit_model,
        market_state,
        None,                 // nats
        None,                 // metrics
        None,                 // risk
        Arc::new(HealthSnapshot::new()),
        None,                 // funds
        fee_repo,
        triangular_paths,
        0.0, // inter-ex slippage per leg (test only)
        0.0, // triangular slippage per leg (test only)
        0.0, // min liquidity usd already provided per-path
    ));

    // 2) 策略集合
    let strategies: Vec<Box<dyn ArbitrageStrategy>> = vec![
        Box::new(InterExchangeStrategy),
        Box::new(TriangularStrategy),
    ];

    // 3) 统计
    let mut total_snapshots: u64 = 0;
    let mut inter_exchange_opportunities: u64 = 0;
    let mut triangular_opportunities: u64 = 0;
    let mut total_profit_bps_sum: f64 = 0.0;

    let start_time = Instant::now();
    let test_duration = Duration::from_secs(900); // 15分钟

    let mut rng = rand::thread_rng();
    let mut interval = time::interval(Duration::from_millis(10)); // 100Hz
    let mut last_report = Instant::now();
    let mut sequence: u64 = 0;

    println!("开始测试...\n");

    while start_time.elapsed() < test_duration {
        interval.tick().await;
        // 每个tick生成1000个快照 => ≈100k/s
        for _ in 0..1000 {
            sequence += 1;
            let now_ns = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos() as u64;

            // 目标利润 0.02% ~ 0.5%
            let target_bps = rng.gen_range(2.0..=50.0);
            let target_frac = target_bps / 10_000.0;

            // --------- 跨所快照（同一 symbol 在两个交易所）---------
            let symbol = Symbol::new("BTCUSDT");
            let mut binance = OrderBook::new(Exchange::new("binance"), symbol.clone(), now_ns, sequence);
            let mut okx = OrderBook::new(Exchange::new("okx"), symbol.clone(), now_ns, sequence);

            let p = 50_000.0 + rng.gen_range(-50.0..50.0);
            let q = 0.5 + rng.gen_range(0.0..0.5);
            // 买在 binance 的 ask = P
            binance.add_ask(FixedPrice::from_f64(p, 2), FixedQuantity::from_f64(q, 8));
            // 卖在 okx 的 bid = P*(1+target)
            okx.add_bid(FixedPrice::from_f64(p * (1.0 + target_frac), 2), FixedQuantity::from_f64(q, 8));

            // 其他围绕 mid 的档位（不压住 best 档）
            for i in 1..5 {
                let d = i as f64 * 0.5;
                binance.add_bid(FixedPrice::from_f64(p - d - 1.0, 2), FixedQuantity::from_f64(q, 8));
                binance.add_ask(FixedPrice::from_f64(p + d + 1.0, 2), FixedQuantity::from_f64(q, 8));
                okx.add_bid(FixedPrice::from_f64(p * (1.0 + target_frac) - d - 1.0, 2), FixedQuantity::from_f64(q, 8));
                okx.add_ask(FixedPrice::from_f64(p * (1.0 + target_frac) + d + 1.0, 2), FixedQuantity::from_f64(q, 8));
            }

            let snap_inter = NormalizedSnapshot {
                symbol: symbol.clone(),
                timestamp_ns: now_ns,
                exchanges: vec![binance, okx],
                weighted_mid_price: FixedPrice::from_f64(p, 2),
                total_bid_volume: FixedQuantity::from_f64(q, 8),
                total_ask_volume: FixedQuantity::from_f64(q, 8),
                quality_score: 0.99,
                sequence: Some(sequence),
            };

            // --------- 三角快照（单一交易所三符号）---------
            // 设定：sell_base=BTCUSDT 的 best_bid = P，buy_inter=BTCETH 的 best_ask = X，buy_base=ETHUSDT 的 best_ask = Y
            // 条件：Y / X = 1 + target_frac，可保证 base_back > qty
            let ex = Exchange::new("binance");
            let s_base_quote = Symbol::new("BTCUSDT");
            let s_base_inter = Symbol::new("BTCETH");
            let s_inter_quote = Symbol::new("ETHUSDT");

            let p_btcusdt = 50_000.0 + rng.gen_range(-50.0..50.0);
            let x_btceth = 15.0 + rng.gen_range(-0.02..0.02); // ETH per BTC
            // Ensure profit: btc_back = qty * p / (y * x) > qty => y*x < p
            // Target net ≈ target_frac => set y = p / (x * (1 + target_frac))
            let y_ethusdt = p_btcusdt / (x_btceth * (1.0 + target_frac));
            let qty = 0.2 + rng.gen_range(0.0..0.2);

            let mut ob_a = OrderBook::new(ex.clone(), s_base_quote.clone(), now_ns, sequence);
            let mut ob_b = OrderBook::new(ex.clone(), s_base_inter.clone(), now_ns, sequence);
            let mut ob_c = OrderBook::new(ex.clone(), s_inter_quote.clone(), now_ns, sequence);

            // BTCUSDT: best_bid = P（卖 base 得到 quote）
            ob_a.add_bid(FixedPrice::from_f64(p_btcusdt, 2), FixedQuantity::from_f64(qty, 8));
            // 不生成会覆盖 best 的更优档
            for i in 1..5 {
                let d = i as f64 * 0.5;
                ob_a.add_bid(FixedPrice::from_f64(p_btcusdt - d - 1.0, 2), FixedQuantity::from_f64(qty, 8));
                ob_a.add_ask(FixedPrice::from_f64(p_btcusdt + d + 1.0, 2), FixedQuantity::from_f64(qty, 8));
            }
            // BTCETH: 只注入我们希望的 best_ask = X
            ob_b.add_ask(FixedPrice::from_f64(x_btceth, 6), FixedQuantity::from_f64(qty, 8));
            // ETHUSDT: 只注入我们希望的 best_ask = Y
            ob_c.add_ask(FixedPrice::from_f64(y_ethusdt, 4), FixedQuantity::from_f64(qty, 8));

            let snap_tri = NormalizedSnapshot {
                symbol: s_base_quote.clone(),
                timestamp_ns: now_ns,
                exchanges: vec![ob_a, ob_b, ob_c],
                weighted_mid_price: FixedPrice::from_f64(p_btcusdt, 2),
                total_bid_volume: FixedQuantity::from_f64(qty, 8),
                total_ask_volume: FixedQuantity::from_f64(qty, 8),
                quality_score: 0.99,
                sequence: Some(sequence + 1),
            };

            // 跑策略
            for snap in [&snap_inter, &snap_tri] {
                for s in strategies.iter() {
                    if let Some(opp) = s.detect(&context, snap) {
                        match s.name() {
                            "inter_exchange" => inter_exchange_opportunities += 1,
                            "triangular" => triangular_opportunities += 1,
                            _ => {}
                        }
                        total_profit_bps_sum += opp.net_profit_pct.to_f64() * 10_000.0;
                    }
                }
                total_snapshots += 1;
            }
        }

        if last_report.elapsed() >= Duration::from_secs(10) {
            let elapsed = start_time.elapsed().as_secs_f64();
            let rate = (total_snapshots as f64 / elapsed).round();
            let total_opp = inter_exchange_opportunities + triangular_opportunities;
            let avg_bps = if total_opp > 0 { total_profit_bps_sum / total_opp as f64 } else { 0.0 };
            println!(
                "[{:>4.0}s] snaps={:>10} rate≈{:>7}/s inter_ex={:>8} tri={:>8} avg_profit≈{:>5.2}bps",
                start_time.elapsed().as_secs_f64(),
                total_snapshots,
                rate,
                inter_exchange_opportunities,
                triangular_opportunities,
                avg_bps,
            );
            last_report = Instant::now();
        }
    }

    // 汇总
    let elapsed = start_time.elapsed().as_secs_f64();
    let rate = (total_snapshots as f64 / elapsed).round();
    let total_opp = inter_exchange_opportunities + triangular_opportunities;
    let avg_bps = if total_opp > 0 { total_profit_bps_sum / total_opp as f64 } else { 0.0 };

    println!("\n=== 测试完成 ===");
    println!("总处理快照数: {}", total_snapshots);
    println!("平均处理速率: {} /s", rate);
    println!("跨交易所套利: {} 次 | 三角套利: {} 次 | 合计: {}", inter_exchange_opportunities, triangular_opportunities, total_opp);
    println!("机会检出率: {:.4}%", (total_opp as f64) * 100.0 / (total_snapshots as f64));
    if total_opp > 0 { println!("平均利润: {:.2} bps", avg_bps); }
} 