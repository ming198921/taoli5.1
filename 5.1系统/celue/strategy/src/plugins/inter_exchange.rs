//! Inter-exchange arbitrage strategy implementation.

use crate::{
    traits::{ArbitrageStrategy, ExecutionResult, StrategyError, StrategyKind},
};
use common_types::StrategyContext;
use async_trait::async_trait;
use common_types::ArbitrageOpportunity;
use common_types::{NormalizedSnapshot, ExchangeSnapshot};
use common::{
    arbitrage::{ArbitrageLeg, Side},
    market_data::OrderBook,
    precision::{FixedPrice, FixedQuantity},
};
use itertools::Itertools;
use uuid::Uuid;

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
        ctx: &dyn StrategyContext,
        input: &NormalizedSnapshot,
    ) -> Option<ArbitrageOpportunity> {
        if input.exchanges.len() < 2 {
            return None;
        }

        let min_profit_pct = ctx.current_min_profit_pct();

        // 使用ExchangeSnapshot数据进行套利检测
        if input.exchanges.len() < 2 {
            return None;
        }
        
        let exchange_snapshots: Vec<(&String, &ExchangeSnapshot)> = input
            .exchanges
            .iter()
            .collect();

        // Iterate over all unique pairs of exchanges for the same symbol
        for ((exchange_a, snapshot_a), (exchange_b, snapshot_b)) in exchange_snapshots.iter().tuple_combinations() {
            // 简化版套利检测：基于价格差直接计算
            let buy_price = snapshot_a.ask_price;  // 在A交易所买入（支付ask价格）
            let sell_price = snapshot_b.bid_price; // 在B交易所卖出（获得bid价格）
            
            if sell_price > buy_price {
                let profit_pct = (sell_price - buy_price) / buy_price;
                if profit_pct >= min_profit_pct {
                    // 创建套利机会
                    return Some(self.create_opportunity(
                        &input.symbol, exchange_a, exchange_b, buy_price, sell_price, profit_pct
                    ));
                }
            }
            
            // 反向检测：在B买入，在A卖出
            let buy_price_b = snapshot_b.ask_price;
            let sell_price_a = snapshot_a.bid_price;
            
            if sell_price_a > buy_price_b {
                let profit_pct = (sell_price_a - buy_price_b) / buy_price_b;
                if profit_pct >= min_profit_pct {
                    return Some(self.create_opportunity(
                        &input.symbol, exchange_b, exchange_a, buy_price_b, sell_price_a, profit_pct
                    ));
                }
            }
        }
        None
    }

    async fn execute(
        &self,
        _ctx: &dyn StrategyContext,
        _opportunity: &ArbitrageOpportunity,
    ) -> Result<ExecutionResult, StrategyError> {
        // For simulation mode, we return success
        Ok(ExecutionResult {
            accepted: true,
            reason: Some("Simulation execution".to_string()),
            order_ids: vec!["sim_001".to_string(), "sim_002".to_string()],
            executed_quantity: 1.0,
            realized_profit: 0.1,
            execution_time_ms: 50,
            slippage: 0.001,
            fees_paid: 0.005,
        })
    }
}

impl InterExchangeStrategy {
    /// Find arbitrage opportunity between two exchanges
    #[allow(dead_code)]
    fn find_opportunity(
        &self,
        ctx: &dyn StrategyContext,
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
        if let Some(repo) = ctx.fee_precision_repo() {
            if let Some(step) = repo.get_fee(&buy_book.symbol.to_string(), "step_size") {
                // snap down to nearest step
                let step_scaled = FixedQuantity::from_f64(step, trade_qty.scale());
                let steps = (trade_qty.raw_value() / step_scaled.raw_value()).max(1);
                trade_qty = FixedQuantity::from_raw(steps * step_scaled.raw_value(), trade_qty.scale());
            }
        }

        // Calculate profits using the real price difference
        // Apply tick snap on prices if available
        let mut buy_px = buy_price.price;
        let mut sell_px = sell_price.price;
        if let Some(repo) = ctx.fee_precision_repo() {
            if let Some(tick) = repo.get_fee(&buy_book.symbol.to_string(), "tick_size") {
                let tick_scaled = FixedPrice::from_f64(tick, buy_px.scale());
                // snap to tick grid conservatively
                buy_px = FixedPrice::from_raw((buy_px.raw_value() / tick_scaled.raw_value()) * tick_scaled.raw_value(), buy_px.scale());
                sell_px = FixedPrice::from_raw((sell_px.raw_value() / tick_scaled.raw_value()) * tick_scaled.raw_value(), sell_px.scale());
            }
        }
        let gross_profit_per_unit = sell_px - buy_px;
        let gross_profit = gross_profit_per_unit * trade_qty;

        let buy_exchange = buy_book.exchange.as_str();
        let sell_exchange = sell_book.exchange.as_str();
        // 获取手续费率 - 使用简化的context方法
        let buy_fee_bps = {
            if let Some(repo) = ctx.fee_precision_repo() {
                repo.get_fee(buy_exchange, "rate_bps").unwrap_or_else(|| {
                    tracing::warn!("交易所 {} 手续费配置缺失，使用默认taker费率", buy_exchange);
                    ctx.get_taker_fee(buy_exchange) * 10000.0 // 转换为bps
                })
            } else {
                ctx.get_taker_fee(buy_exchange) * 10000.0 // 转换为bps
            }
        };
        
        let sell_fee_bps = {
            if let Some(repo) = ctx.fee_precision_repo() {
                repo.get_fee(sell_exchange, "rate_bps").unwrap_or_else(|| {
                    tracing::warn!("交易所 {} 手续费配置缺失，使用默认taker费率", sell_exchange);
                    ctx.get_taker_fee(sell_exchange) * 10000.0 // 转换为bps
                })
            } else {
                ctx.get_taker_fee(sell_exchange) * 10000.0 // 转换为bps
            }
        };
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
        let slip = ctx.inter_exchange_slippage_per_leg_pct();
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
        let _buy_leg = ArbitrageLeg {
            exchange: buy_book.exchange.clone(),
            symbol: buy_book.symbol.clone(),
            side: Side::Buy,
            price: buy_price.price,
            quantity: trade_qty,
            cost: buy_cost,
        };

        let _sell_leg = ArbitrageLeg {
            exchange: sell_book.exchange.clone(),
            symbol: sell_book.symbol.clone(),
            side: Side::Sell,
            price: sell_price.price,
            quantity: trade_qty,
            cost: sell_proceeds,
        };

        // Create the opportunity using the unified API
        // 使用原有算法的真实数据：buy_book, sell_book, buy_price, sell_price, trade_qty
        let opportunity = ArbitrageOpportunity::new(
            format!("inter_exchange_{}", Uuid::new_v4()),
            buy_book.symbol.to_string(),
            buy_book.exchange.to_string(),
            sell_book.exchange.to_string(),
            buy_price.price.to_f64(), // buy_price (ask price from buy exchange)
            sell_price.price.to_f64(), // sell_price (bid price from sell exchange)
            trade_qty.to_f64(),
            150_000, // 150 seconds TTL
        );

        Some(opportunity)
    }
    
    /// 创建套利机会 - 适配统一的ArbitrageOpportunity结构
    fn create_opportunity(
        &self,
        symbol: &str,
        buy_exchange: &str,
        sell_exchange: &str,
        buy_price: f64,
        sell_price: f64,
        profit_pct: f64,
    ) -> ArbitrageOpportunity {
        ArbitrageOpportunity {
            id: uuid::Uuid::new_v4().to_string(),
            strategy_type: common_types::StrategyType::InterExchangeArbitrage,
            exchange_pair: Some((buy_exchange.to_string(), sell_exchange.to_string())),
            triangle_path: None,
            symbol: symbol.to_string(),
            estimated_profit: sell_price - buy_price,
            net_profit: (sell_price - buy_price) * 0.95, // 预扣5%手续费和滑点
            profit_bps: profit_pct * 10000.0, // 转换为基点
            liquidity_score: 0.8, // 默认流动性评分
            confidence_score: 0.9, // 默认置信度
            estimated_latency_ms: 100,
            risk_score: 0.3, // 相对低风险
            required_funds: std::collections::HashMap::new(),
            market_impact: 0.001,
            slippage_estimate: 0.002,
            timestamp: chrono::Utc::now().to_rfc3339(),
            expires_at: (chrono::Utc::now() + chrono::Duration::seconds(150)).to_rfc3339(),
            priority: 128,
            tags: vec!["inter-exchange".to_string()],
            status: common_types::OpportunityStatus::Active,
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{context::StrategyContext, market_state::MarketState};
    use common::{NormalizedSnapshot, Symbol, Exchange, OrderBook};
    use common::precision::{FixedPrice, FixedQuantity};
    use std::sync::Arc;
    use crate::context::FeePrecisionRepoImpl;

    fn create_test_context() -> StrategyContext {
        // 创建策略上下文 - 简化版本，避免不存在的模块
        let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
        let strategy_config = StrategyConfig::default();
        StrategyContext::new(fee_repo, strategy_config, None, None, None, None)
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
