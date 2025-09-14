//! 订单簿深度分析模块 - 精确滑点和流动性计算
//! 
//! 实现真实的市场微观结构分析，替代简化的滑点模型。
//! 通过遍历订单簿层级，计算累积价格影响和实际可执行数量。

use common::{
    market_data::OrderBook,
    precision::{FixedPrice, FixedQuantity},
    arbitrage::Side,
};
use std::collections::BTreeMap;
use anyhow::Result;

/// 深度分析结果
#[derive(Debug, Clone)]
pub struct DepthAnalysisResult {
    /// 实际平均执行价格（考虑滑点）
    pub effective_price: FixedPrice,
    /// 最大可执行数量
    pub max_quantity: FixedQuantity,
    /// 累积滑点百分比
    pub cumulative_slippage_pct: f64,
    /// 价格影响曲线（数量 -> 平均价格）
    pub price_impact_curve: Vec<(FixedQuantity, FixedPrice)>,
    /// 流动性深度评分 (0-100)
    pub liquidity_score: u8,
    /// 执行风险评分 (0-100，越低越好)
    pub execution_risk_score: u8,
}

/// 深度分析器
pub struct DepthAnalyzer {
    /// 最大分析深度（层数）
    max_depth_levels: usize,
    /// 最小tick size（避免除零）
    min_tick_size: f64,
}

impl Default for DepthAnalyzer {
    fn default() -> Self {
        Self {
            max_depth_levels: 20, // 分析前20层深度
            min_tick_size: 1e-8,
        }
    }
}

impl DepthAnalyzer {
    pub fn new(max_depth_levels: usize) -> Self {
        Self {
            max_depth_levels,
            min_tick_size: 1e-8,
        }
    }

    /// 分析订单簿深度，计算指定交易数量的真实执行效果
    pub fn analyze_depth(
        &self,
        orderbook: &OrderBook,
        side: Side,
        target_quantity: FixedQuantity,
    ) -> Result<DepthAnalysisResult> {
        match side {
            Side::Buy => self.analyze_ask_depth(orderbook, target_quantity),
            Side::Sell => self.analyze_bid_depth(orderbook, target_quantity),
        }
    }

    /// 分析卖单深度（用于买入订单）
    fn analyze_ask_depth(
        &self,
        orderbook: &OrderBook,
        target_quantity: FixedQuantity,
    ) -> Result<DepthAnalysisResult> {
        if orderbook.ask_prices.is_empty() || orderbook.ask_quantities.is_empty() {
            return Err(anyhow::anyhow!("卖单深度为空"));
        }

        // 构建价格-数量映射，按价格升序排列
        let mut price_levels = BTreeMap::new();
        for (i, &price) in orderbook.ask_prices.iter().enumerate() {
            if i < orderbook.ask_quantities.len() {
                let quantity = orderbook.ask_quantities[i];
                let existing = price_levels.entry(price).or_insert(FixedQuantity::from_raw(0, quantity.scale()));
                *existing = *existing + quantity;
            }
        }

        self.calculate_cumulative_impact(price_levels, target_quantity, true)
    }

    /// 分析买单深度（用于卖出订单）
    fn analyze_bid_depth(
        &self,
        orderbook: &OrderBook,
        target_quantity: FixedQuantity,
    ) -> Result<DepthAnalysisResult> {
        if orderbook.bid_prices.is_empty() || orderbook.bid_quantities.is_empty() {
            return Err(anyhow::anyhow!("买单深度为空"));
        }

        // 构建价格-数量映射，按价格降序排列
        let mut price_levels = BTreeMap::new();
        for (i, &price) in orderbook.bid_prices.iter().enumerate() {
            if i < orderbook.bid_quantities.len() {
                let quantity = orderbook.bid_quantities[i];
                { let existing = price_levels.entry(price).or_insert(FixedQuantity::from_raw(0, quantity.scale())); *existing = *existing + quantity; }
            }
        }

        self.calculate_cumulative_impact(price_levels, target_quantity, false)
    }

    /// 计算累积价格影响
    fn calculate_cumulative_impact(
        &self,
        price_levels: BTreeMap<FixedPrice, FixedQuantity>,
        target_quantity: FixedQuantity,
        is_ascending: bool, // true=ask(升序), false=bid(降序)
    ) -> Result<DepthAnalysisResult> {
        let mut cumulative_quantity = FixedQuantity::from_raw(0, target_quantity.scale());
        let mut cumulative_cost = FixedPrice::from_raw(0, 6); // 使用scale 6表示USD价值
        let mut price_impact_curve = Vec::new();
        let mut levels_processed = 0;

        // 获取最佳价格（参考价格）
        let best_price = if is_ascending {
            price_levels.keys().next().copied()
        } else {
            price_levels.keys().next_back().copied()
        }.ok_or_else(|| anyhow::anyhow!("订单簿为空"))?;

        // 按价格顺序遍历深度
        let level_iter: Box<dyn Iterator<Item = (&FixedPrice, &FixedQuantity)>> = if is_ascending {
            Box::new(price_levels.iter())
        } else {
            Box::new(price_levels.iter().rev())
        };

        for (&price, &available_quantity) in level_iter {
            if levels_processed >= self.max_depth_levels {
                break;
            }

            let remaining_target = target_quantity - cumulative_quantity;
            if remaining_target <= FixedQuantity::from_raw(0, target_quantity.scale()) {
                break; // 已满足目标数量
            }

            // 当前层级可消耗数量
            let consumed_quantity = available_quantity.min(remaining_target);
            
            // 累积数量和成本
            cumulative_quantity = cumulative_quantity + consumed_quantity;
            let level_cost = price * consumed_quantity;
            cumulative_cost = cumulative_cost + FixedPrice::from_f64(level_cost.to_f64(), 6);

            // 计算当前平均价格
            let current_avg_price = if cumulative_quantity.to_f64() > 0.0 {
                FixedPrice::from_f64(cumulative_cost.to_f64() / cumulative_quantity.to_f64(), price.scale())
            } else {
                price
            };

            // 记录价格影响点
            price_impact_curve.push((cumulative_quantity, current_avg_price));

            levels_processed += 1;

            // 如果当前层级完全满足需求，退出
            if cumulative_quantity >= target_quantity {
                break;
            }
        }

        // 计算最终结果
        let max_quantity = cumulative_quantity;
        let effective_price = if cumulative_quantity.to_f64() > 0.0 {
            FixedPrice::from_f64(cumulative_cost.to_f64() / cumulative_quantity.to_f64(), best_price.scale())
        } else {
            best_price
        };

        // 计算滑点百分比
        let cumulative_slippage_pct = if best_price.to_f64() > 0.0 {
            let price_diff = effective_price.to_f64() - best_price.to_f64();
            (price_diff / best_price.to_f64()).abs() * 100.0
        } else {
            0.0
        };

        // 计算流动性评分
        let liquidity_score = self.calculate_liquidity_score(
            &price_impact_curve,
            target_quantity,
            max_quantity,
        );

        // 计算执行风险评分
        let execution_risk_score = self.calculate_execution_risk_score(
            cumulative_slippage_pct,
            levels_processed,
            target_quantity,
            max_quantity,
        );

        Ok(DepthAnalysisResult {
            effective_price,
            max_quantity,
            cumulative_slippage_pct,
            price_impact_curve,
            liquidity_score,
            execution_risk_score,
        })
    }

    /// 计算流动性评分 (0-100，越高越好)
    fn calculate_liquidity_score(
        &self,
        price_impact_curve: &[(FixedQuantity, FixedPrice)],
        target_quantity: FixedQuantity,
        max_quantity: FixedQuantity,
    ) -> u8 {
        let mut score = 100u8;

        // 可满足比例影响
        let fulfillment_ratio = (max_quantity.to_f64() / target_quantity.to_f64()).min(1.0);
        score = ((score as f64) * fulfillment_ratio) as u8;

        // 深度平滑度影响
        if price_impact_curve.len() < 3 {
            score = score.saturating_sub(30); // 深度太浅扣分
        } else if price_impact_curve.len() > 10 {
            score = score.saturating_add(10); // 深度充足加分
        }

        // 价格连续性影响
        let mut price_gap_penalty = 0u8;
        for window in price_impact_curve.windows(2) {
            let price_change_pct = if window[0].1.to_f64() > 0.0 {
                ((window[1].1.to_f64() - window[0].1.to_f64()) / window[0].1.to_f64()).abs() * 100.0
            } else {
                0.0
            };
            
            if price_change_pct > 1.0 { // 单层跳价超过1%
                price_gap_penalty = price_gap_penalty.saturating_add(5);
            }
        }

        score.saturating_sub(price_gap_penalty)
    }

    /// 计算执行风险评分 (0-100，越低越好)
    fn calculate_execution_risk_score(
        &self,
        cumulative_slippage_pct: f64,
        levels_processed: usize,
        target_quantity: FixedQuantity,
        max_quantity: FixedQuantity,
    ) -> u8 {
        let mut risk_score = 0u8;

        // 滑点风险
        if cumulative_slippage_pct > 2.0 {
            risk_score += 40; // 超过2%滑点高风险
        } else if cumulative_slippage_pct > 1.0 {
            risk_score += 25; // 超过1%滑点中风险
        } else if cumulative_slippage_pct > 0.5 {
            risk_score += 10; // 超过0.5%滑点低风险
        }

        // 深度风险
        if levels_processed < 3 {
            risk_score += 25; // 深度不足风险
        } else if levels_processed > 15 {
            risk_score += 5; // 过度分散小风险
        }

        // 满足度风险
        let fulfillment_ratio = max_quantity.to_f64() / target_quantity.to_f64();
        if fulfillment_ratio < 0.5 {
            risk_score += 35; // 满足度低于50%高风险
        } else if fulfillment_ratio < 0.8 {
            risk_score += 15; // 满足度低于80%中风险
        }

        risk_score.min(100)
    }

    /// 批量分析多个订单簿的深度（用于三角套利）
    pub fn batch_analyze_triangular_depth(
        &self,
        orderbooks: &[&OrderBook],
        sides: &[Side],
        quantities: &[FixedQuantity],
    ) -> Result<Vec<DepthAnalysisResult>> {
        if orderbooks.len() != sides.len() || sides.len() != quantities.len() {
            return Err(anyhow::anyhow!("批量分析参数长度不匹配"));
        }

        let mut results = Vec::new();
        for (i, &orderbook) in orderbooks.iter().enumerate() {
            let result = self.analyze_depth(orderbook, sides[i], quantities[i])?;
            results.push(result);
        }

        Ok(results)
    }

    /// 优化三角套利路径的数量分配
    pub fn optimize_triangular_quantities(
        &self,
        orderbooks: &[&OrderBook],
        sides: &[Side],
        initial_amount: FixedQuantity,
    ) -> Result<(Vec<FixedQuantity>, f64)> {
        if orderbooks.len() != 3 || sides.len() != 3 {
            return Err(anyhow::anyhow!("三角套利需要3个订单簿和交易方向"));
        }

        // 从小到大尝试不同的初始数量，找到最优分配
        let mut best_quantities = vec![initial_amount; 3];
        let mut best_efficiency = 0.0f64;

        // 尝试不同规模的交易
        for scale_factor in [0.1, 0.2, 0.5, 0.8, 1.0, 1.5, 2.0] {
            let test_amount = FixedQuantity::from_f64(initial_amount.to_f64() * scale_factor, initial_amount.scale());
            
            // 分析每一腿
            let mut leg_results = Vec::new();
            let mut all_feasible = true;
            
            for i in 0..3 {
                match self.analyze_depth(orderbooks[i], sides[i], test_amount) {
                    Ok(result) => {
                        // 检查是否可以满足至少80%的目标数量
                        if result.max_quantity.to_f64() < test_amount.to_f64() * 0.8 {
                            all_feasible = false;
                            break;
                        }
                        leg_results.push(result);
                    }
                    Err(_) => {
                        all_feasible = false;
                        break;
                    }
                }
            }

            if all_feasible && leg_results.len() == 3 {
                // 计算整体效率：考虑滑点、风险和满足度
                let avg_slippage = leg_results.iter()
                    .map(|r| r.cumulative_slippage_pct)
                    .sum::<f64>() / 3.0;
                
                let avg_risk = leg_results.iter()
                    .map(|r| r.execution_risk_score as f64)
                    .sum::<f64>() / 3.0;
                
                let min_fulfillment = leg_results.iter()
                    .map(|r| r.max_quantity.to_f64() / test_amount.to_f64())
                    .fold(1.0, f64::min);

                // 效率评分：高满足度，低滑点，低风险
                let efficiency = min_fulfillment * 100.0 - avg_slippage * 10.0 - avg_risk * 0.5;
                
                if efficiency > best_efficiency {
                    best_efficiency = efficiency;
                    best_quantities = leg_results.iter()
                        .map(|r| r.max_quantity)
                        .collect();
                }
            }
        }

        Ok((best_quantities, best_efficiency))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::types::{Exchange, Symbol};

    /// 创建测试用订单簿
    fn create_test_orderbook() -> OrderBook {
        OrderBook {
            exchange: Exchange::new("test_exchange"),
            symbol: Symbol::new("BTCUSDT"),
            bid_prices: vec![
                FixedPrice::from_f64(50000.0, 2),
                FixedPrice::from_f64(49999.0, 2),
                FixedPrice::from_f64(49998.0, 2),
            ],
            bid_quantities: vec![
                FixedQuantity::from_f64(1.0, 8),
                FixedQuantity::from_f64(2.0, 8),
                FixedQuantity::from_f64(3.0, 8),
            ],
            ask_prices: vec![
                FixedPrice::from_f64(50001.0, 2),
                FixedPrice::from_f64(50002.0, 2),
                FixedPrice::from_f64(50003.0, 2),
            ],
            ask_quantities: vec![
                FixedQuantity::from_f64(1.5, 8),
                FixedQuantity::from_f64(2.5, 8),
                FixedQuantity::from_f64(3.5, 8),
            ],
            timestamp_ns: 0,
        }
    }

    #[test]
    fn test_analyze_ask_depth() {
        let analyzer = DepthAnalyzer::default();
        let orderbook = create_test_orderbook();
        let target_quantity = FixedQuantity::from_f64(2.0, 8);

        let result = analyzer.analyze_depth(&orderbook, Side::Buy, target_quantity).unwrap();
        
        assert!(result.max_quantity.to_f64() > 0.0);
        assert!(result.effective_price.to_f64() > 50001.0); // 应该高于最佳ask价格
        assert!(result.cumulative_slippage_pct > 0.0);
        assert!(!result.price_impact_curve.is_empty());
    }

    #[test]
    fn test_analyze_bid_depth() {
        let analyzer = DepthAnalyzer::default();
        let orderbook = create_test_orderbook();
        let target_quantity = FixedQuantity::from_f64(1.5, 8);

        let result = analyzer.analyze_depth(&orderbook, Side::Sell, target_quantity).unwrap();
        
        assert!(result.max_quantity.to_f64() > 0.0);
        assert!(result.effective_price.to_f64() <= 50000.0); // 应该低于最佳bid价格
        assert!(result.cumulative_slippage_pct >= 0.0);
    }

    #[test]
    fn test_triangular_quantity_optimization() {
        let analyzer = DepthAnalyzer::default();
        let orderbook = create_test_orderbook();
        let orderbooks = vec![&orderbook, &orderbook, &orderbook];
        let sides = vec![Side::Buy, Side::Sell, Side::Buy];
        let initial_amount = FixedQuantity::from_f64(1.0, 8);

        let (quantities, efficiency) = analyzer.optimize_triangular_quantities(
            &orderbooks, &sides, initial_amount
        ).unwrap();
        
        assert_eq!(quantities.len(), 3);
        assert!(efficiency > 0.0);
        assert!(quantities.iter().all(|q| q.to_f64() > 0.0));
    }
} 