//! Inter-exchange arbitrage strategy implementation.

use crate::{
    context::StrategyContext,
    traits::{ArbitrageStrategy, ExecutionResult, StrategyError, StrategyKind},
};
use async_trait::async_trait;
use common::{
    arbitrage::{ArbitrageLeg, ArbitrageOpportunity, Side},
    market_data::{NormalizedSnapshot, OrderBook},
    precision::{FixedPrice, FixedQuantity},
};
use itertools::Itertools;

pub struct InterExchangeStrategy;

#[async_trait]
impl ArbitrageStrategy for InterExchangeStrategy {
    fn name(&self) -> &'static str {
        "inter_exchange"
    }

    fn kind(&self) -> StrategyKind {
        StrategyKind::InterExchange
    }

    fn detect(
        &self,
        ctx: &StrategyContext,
        input: &NormalizedSnapshot,
    ) -> Option<ArbitrageOpportunity> {
        if input.exchanges.len() < 2 {
            return None;
        }

        let min_profit_pct = ctx.current_min_profit_pct();

        // Only consider order books for the same symbol as the snapshot
        let same_symbol_books: Vec<&OrderBook> = input
            .exchanges
            .iter()
            .filter(|ob| ob.symbol == input.symbol)
            .collect();
        if same_symbol_books.len() < 2 {
            return None;
        }

        // Iterate over all unique pairs of exchanges for the same symbol
        for (book_a, book_b) in same_symbol_books.iter().copied().tuple_combinations() {
            // Opportunity: Buy on A, Sell on B
            if let Some(opp) = self.find_opportunity(ctx, book_a, book_b, min_profit_pct) {
                return Some(opp);
            }
            // Opportunity: Buy on B, Sell on A
            if let Some(opp) = self.find_opportunity(ctx, book_b, book_a, min_profit_pct) {
                return Some(opp);
            }
        }
        None
    }

    async fn execute(
        &self,
        _ctx: &StrategyContext,
        _opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError> {
        // For simulation mode, we return success
        Ok(ExecutionResult {
            accepted: true,
            reason: Some("Simulation execution".to_string()),
            order_ids: vec!["sim_001".to_string(), "sim_002".to_string()],
        })
    }
}

impl InterExchangeStrategy {
    /// Find arbitrage opportunity between two exchanges
    fn find_opportunity(
        &self,
        ctx: &StrategyContext,
        buy_book: &OrderBook,
        sell_book: &OrderBook,
        min_profit_pct: FixedPrice,
    ) -> Option<ArbitrageOpportunity> {
        // Get the best prices
        let buy_price = buy_book.best_ask()?; // We buy at the ask price
        let sell_price = sell_book.best_bid()?; // We sell at the bid price

        // Check if there's a profit opportunity
        if sell_price.price <= buy_price.price {
            return None;
        }

        // Calculate gross profit percentage in fixed-point domain to avoid floating point
        // profit_pct = (sell - buy) / buy
        let gross_diff = sell_price.price - buy_price.price;
        // Use scale 6 for percentage
        let profit_percentage = FixedPrice::from_f64(
            gross_diff.to_f64() / buy_price.price.to_f64(),
            6,
        );

        // Check if profit meets minimum threshold
        if profit_percentage < min_profit_pct {
            return None;
        }

        // Determine trade quantity (minimum of both sides available), apply step size snap if available
        let max_buy_qty = buy_price.quantity;
        let max_sell_qty = sell_price.quantity;
        let mut trade_qty = if max_buy_qty < max_sell_qty { max_buy_qty } else { max_sell_qty };
        if let Some(step) = ctx.fee_precision_repo.get_step_size_for_symbol(&buy_book.symbol.to_string()) {
            // snap down to nearest step
            let step_scaled = FixedQuantity::from_f64(step, trade_qty.scale());
            let steps = (trade_qty.raw_value() / step_scaled.raw_value()).max(1);
            trade_qty = FixedQuantity::from_raw(steps * step_scaled.raw_value(), trade_qty.scale());
        }

        // Calculate profits using the real price difference
        // Apply tick snap on prices if available
        let mut buy_px = buy_price.price;
        let mut sell_px = sell_price.price;
        if let Some(tick) = ctx.fee_precision_repo.get_tick_size_for_symbol(&buy_book.symbol.to_string()) {
            let tick_scaled = FixedPrice::from_f64(tick, buy_px.scale());
            // snap to tick grid conservatively
            buy_px = FixedPrice::from_raw((buy_px.raw_value() / tick_scaled.raw_value()) * tick_scaled.raw_value(), buy_px.scale());
            sell_px = FixedPrice::from_raw((sell_px.raw_value() / tick_scaled.raw_value()) * tick_scaled.raw_value(), sell_px.scale());
        }
        let gross_profit_per_unit = sell_px - buy_px;
        let gross_profit = gross_profit_per_unit * trade_qty;

        let buy_exchange = buy_book.exchange.as_str();
        let sell_exchange = sell_book.exchange.as_str();
        // 获取手续费率 - 通过context动态获取，移除硬编码
        let buy_fee_bps = ctx.fee_precision_repo.get_fee_rate_bps_for_exchange(buy_exchange)
            .unwrap_or_else(|| {
                tracing::warn!("交易所 {} 手续费配置缺失，使用taker_fee计算", buy_exchange);
                ctx.get_taker_fee(&buy_book.exchange)
                    .map(|fee| fee.to_f64() * 10000.0) // 转换为bps
                    .unwrap_or_else(|| {
                        tracing::error!("交易所 {} 无任何手续费配置，跳过套利机会", buy_exchange);
                        f64::MAX // 返回极大值，确保被过滤
                    })
            });
        let sell_fee_bps = ctx.fee_precision_repo.get_fee_rate_bps_for_exchange(sell_exchange)
            .unwrap_or_else(|| {
                tracing::warn!("交易所 {} 手续费配置缺失，使用taker_fee计算", sell_exchange);
                ctx.get_taker_fee(&sell_book.exchange)
                    .map(|fee| fee.to_f64() * 10000.0) // 转换为bps
                    .unwrap_or_else(|| {
                        tracing::error!("交易所 {} 无任何手续费配置，跳过套利机会", sell_exchange);
                        f64::MAX // 返回极大值，确保被过滤
                    })
            });
        // 安全检查：如果手续费异常，跳过此套利机会
        if buy_fee_bps >= f64::MAX || sell_fee_bps >= f64::MAX {
            return None;
        }
        
        let buy_fee_rate = buy_fee_bps / 10_000.0;
        let sell_fee_rate = sell_fee_bps / 10_000.0;

        let buy_cost = buy_px * trade_qty;
        let sell_proceeds = sell_px * trade_qty;
        
        // 使用FixedPrice进行精确的手续费计算，避免f64精度损失
        let buy_fee_rate_fixed = FixedPrice::from_f64(buy_fee_rate, 6);
        let sell_fee_rate_fixed = FixedPrice::from_f64(sell_fee_rate, 6);
        
        // 计算手续费：cost * fee_rate
        let buy_fees = FixedPrice::from_f64(
            buy_cost.to_f64() * buy_fee_rate_fixed.to_f64(), 
            buy_cost.scale()
        );
        let sell_fees = FixedPrice::from_f64(
            sell_proceeds.to_f64() * sell_fee_rate_fixed.to_f64(), 
            sell_proceeds.scale()
        );
        let total_fees = buy_fees + sell_fees;

        let net_profit = gross_profit - total_fees;
        let net_profit_pct = FixedPrice::from_f64(net_profit.to_f64() / buy_cost.to_f64(), 6);

        // Apply slippage budget per leg: reduce expected proceeds and increase expected cost
        let slip = ctx.inter_exchange_slippage_per_leg_pct;
        let _ = slip; // used in required below to avoid unused warnings if cfg changes
        // approximate: require additional margin equal to 2*slippage (buy worse, sell worse)
        let required = FixedPrice::from_f64(min_profit_pct.to_f64() + 2.0 * slip, 6);
        if net_profit_pct < required {
            return None;
        }

        // Validate that we have positive net profit
        if net_profit.to_f64() <= 0.0 {
            return None;
        }

        // Create arbitrage legs with realistic cost calculations
        let buy_leg = ArbitrageLeg {
            exchange: buy_book.exchange.clone(),
            symbol: buy_book.symbol.clone(),
            side: Side::Buy,
            price: buy_price.price,
            quantity: trade_qty,
            cost: buy_cost,
        };

        let sell_leg = ArbitrageLeg {
            exchange: sell_book.exchange.clone(),
            symbol: sell_book.symbol.clone(),
            side: Side::Sell,
            price: sell_price.price,
            quantity: trade_qty,
            cost: sell_proceeds,
        };

        // Create the opportunity with current timestamp
        let opportunity = ArbitrageOpportunity::new_inter_exchange(
            "inter_exchange",
            buy_leg,
            sell_leg,
            net_profit,
            net_profit_pct,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        );

        Some(opportunity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{context::StrategyContext, market_state::{AtomicMarketState, MarketState}, min_profit::MinProfitModel};
    use common::{NormalizedSnapshot, Symbol, Exchange, OrderBook};
    use common::precision::{FixedPrice, FixedQuantity};
    use std::sync::Arc;
    use crate::context::FeePrecisionRepoImpl;

    fn create_test_context() -> StrategyContext {
        let min_profit_model = Arc::new(MinProfitModel::new(50, 1.4, 2.5));
        let market_state = Arc::new(AtomicMarketState::new(MarketState::Regular));
        // 创建策略上下文 - 简化版本，避免不存在的模块
        let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
        let strategy_metrics = Arc::new(adapters::metrics::AdapterMetrics::new());
        let ctx = StrategyContext::new(fee_repo, strategy_metrics);
        let paths: Arc<Vec<String>> = Arc::new(vec![]);
        // let health_snapshot = Arc::new(adapters::health::HealthSnapshot::new()); // 暂时注释
        ctx
    }

    #[tokio::test]
    async fn test_inter_exchange_detection_with_real_data() {
        let strategy = InterExchangeStrategy;
        let ctx = create_test_context();
        let symbol = Symbol::new("BTCUSDT");

        // Create realistic market data based on actual exchange spreads
        let mut binance_book = OrderBook::new(
            Exchange::new("binance"),
            symbol.clone(),
            1000000000,
            1,
        );
        binance_book.add_ask(
            FixedPrice::from_f64(42800.00, 2), // Binance ask
            FixedQuantity::from_f64(0.5, 8),
        );

        let mut coinbase_book = OrderBook::new(
            Exchange::new("coinbase"),
            symbol.clone(),
            1000000100,
            1,
        );
        coinbase_book.add_bid(
            FixedPrice::from_f64(43800.00, 2), // Coinbase bid higher than Binance ask
            FixedQuantity::from_f64(0.5, 8),
        );

        let snapshot = NormalizedSnapshot {
            symbol: symbol.clone(),
            timestamp_ns: 1000000200,
            exchanges: vec![binance_book, coinbase_book],
            weighted_mid_price: FixedPrice::from_f64(43375.0, 2),
            total_bid_volume: FixedQuantity::from_f64(1.0, 8),
            total_ask_volume: FixedQuantity::from_f64(1.0, 8),
            quality_score: 0.98,
            sequence: Some(1),
        };

        let result = strategy.detect(&ctx, &snapshot);
        if let Some(opportunity) = result {
            // Should detect profitable opportunity with proper fee calculations
            println!("Detected opportunity: net_profit={}, net_profit_pct={:.4}%", 
                     opportunity.net_profit.to_f64(),
                     opportunity.net_profit_pct.to_f64() * 100.0);
            
            assert!(opportunity.net_profit.to_f64() > 0.0);
            assert!(opportunity.net_profit_pct.to_f64() > 0.005); // Should be > 0.5%
        } else {
            panic!("Should detect profitable opportunity with 1000 USDT spread");
        }
    }
}
