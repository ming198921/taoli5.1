#![allow(dead_code)]
// anomaly.rs
// 仅迁移核心 AnomalyDetector 及相关类型，供 pipeline.rs 使用

use crate::settings::Settings;
use crate::types::{
    AnomalyDetectionResult, AnomalySeverity, AnomalyType, MarketDataSnapshot, OrderBook,
};
use tracing::{info, warn};

pub struct AnomalyDetector {
    spread_threshold: f64,
    volume_threshold: f64,
    #[allow(dead_code)]
    price_change_threshold: f64,
    spread_threshold_percentage: f64,
}

impl AnomalyDetector {
    pub fn new(settings: &Settings) -> Self {
        Self {
            spread_threshold: settings.anomaly_detection.spread_threshold,
            volume_threshold: settings.anomaly_detection.volume_threshold,
            price_change_threshold: settings.anomaly_detection.price_change_threshold,
            spread_threshold_percentage: settings.anomaly_detection.spread_threshold_percentage,
        }
    }

    /// 检测订单簿异常
    pub fn detect_orderbook_anomalies(
        &self,
        orderbook: &OrderBook,
    ) -> Option<AnomalyDetectionResult> {
        // 检查盘口倒挂 - best_bid.price >= best_ask.price
        if let (Some(best_bid), Some(best_ask)) = (orderbook.bids.first(), orderbook.asks.first()) {
            if best_bid.price >= best_ask.price {
                warn!(
                    "🚨 Crossed spread detected: Best bid ({:.4}) >= Best ask ({:.4})",
                    best_bid.price.into_inner(),
                    best_ask.price.into_inner()
                );

                return Some(AnomalyDetectionResult {
                    anomaly_type: AnomalyType::Other("CrossedSpread".to_string()),
                    severity: AnomalySeverity::Critical,
                    description: "Crossed spread detected - best bid >= best ask".to_string(),
                    details: format!(
                        "Best bid: {:.4}, Best ask: {:.4}",
                        best_bid.price.into_inner(),
                        best_ask.price.into_inner()
                    ),
                    timestamp: orderbook.timestamp,
                    symbol: orderbook.symbol.clone(),
                    source: orderbook.source.clone(),
                    recovery_suggestion: Some(
                        "Check market data feed integrity or exchange connectivity".to_string(),
                    ),
                });
            }

            // 检查价差百分比是否超过阈值
            let spread = best_ask.price.into_inner() - best_bid.price.into_inner();
            let mid_price = (best_ask.price.into_inner() + best_bid.price.into_inner()) / 2.0;
            let spread_percentage = (spread / mid_price) * 100.0;

            if spread_percentage > self.spread_threshold_percentage {
                warn!(
                    "🚨 Excessive spread detected: {:.4}% (threshold: {:.4}%)",
                    spread_percentage, self.spread_threshold_percentage
                );

                return Some(AnomalyDetectionResult {
                    anomaly_type: AnomalyType::PriceGap,
                    severity: if spread_percentage > self.spread_threshold_percentage * 2.0 {
                        AnomalySeverity::Critical
                    } else {
                        AnomalySeverity::Warning
                    },
                    description: format!("Excessive spread detected: {spread_percentage:.4}%"),
                    details: format!(
                        "Spread: ${:.4}, Mid-price: ${:.4}, Threshold: {:.4}%",
                        spread, mid_price, self.spread_threshold_percentage
                    ),
                    timestamp: orderbook.timestamp,
                    symbol: orderbook.symbol.clone(),
                    source: orderbook.source.clone(),
                    recovery_suggestion: Some(
                        "Monitor for market volatility or adjust spread threshold".to_string(),
                    ),
                });
            }
        }

        None
    }

    pub fn detect(&self, snapshot: &MarketDataSnapshot) -> Option<AnomalyDetectionResult> {
        if let Some(ref orderbook) = snapshot.orderbook {
            // 首先检查新的订单簿异常（盘口倒挂和价差过大）
            if let Some(anomaly) = self.detect_orderbook_anomalies(orderbook) {
                return Some(anomaly);
            }

            // 原有的检测逻辑
            if let (Some(best_bid), Some(best_ask)) =
                (orderbook.bids.first(), orderbook.asks.first())
            {
                let spread = best_ask.price.0 - best_bid.price.0;
                let mid_price = (best_ask.price.0 + best_bid.price.0) / 2.0;
                let spread_pct = (spread / mid_price) * 100.0;

                if spread_pct > self.spread_threshold {
                    warn!(
                        "🚨 Abnormal spread detected: {:.4}% (threshold: {:.4}%)",
                        spread_pct, self.spread_threshold
                    );

                    return Some(AnomalyDetectionResult {
                        anomaly_type: AnomalyType::AbnormalSpread,
                        severity: if spread_pct > self.spread_threshold * 2.0 {
                            AnomalySeverity::Critical
                        } else {
                            AnomalySeverity::Warning
                        },
                        description: format!("Abnormal spread detected: {spread_pct:.4}%"),
                        details: format!(
                            "Spread: ${:.4}, Mid-price: ${:.4}, Threshold: {:.4}%",
                            spread, mid_price, self.spread_threshold
                        ),
                        timestamp: snapshot.timestamp,
                        symbol: orderbook.symbol.clone(),
                        source: snapshot.source.clone(),
                        recovery_suggestion: Some(
                            "Consider checking market data feed or increasing spread threshold"
                                .to_string(),
                        ),
                    });
                }

                // 检测深度异常
                let total_bid_volume: f64 =
                    orderbook.bids.iter().map(|entry| entry.quantity.0).sum();
                let total_ask_volume: f64 =
                    orderbook.asks.iter().map(|entry| entry.quantity.0).sum();

                if total_bid_volume < self.volume_threshold
                    || total_ask_volume < self.volume_threshold
                {
                    warn!(
                        "🚨 Low liquidity detected: Bid volume: {:.4}, Ask volume: {:.4}",
                        total_bid_volume, total_ask_volume
                    );

                    return Some(AnomalyDetectionResult {
                        anomaly_type: AnomalyType::LowLiquidity,
                        severity: AnomalySeverity::Warning,
                        description: "Low liquidity detected".to_string(),
                        details: format!("Bid volume: {:.4}, Ask volume: {:.4}, Threshold: {:.4}", 
                                       total_bid_volume, total_ask_volume, self.volume_threshold),
                        timestamp: snapshot.timestamp,
                        symbol: orderbook.symbol.clone(),
                        source: snapshot.source.clone(),
                        recovery_suggestion: Some("Consider switching to a more liquid exchange or adjusting volume thresholds".to_string()),
                    });
                }

                info!(
                    "✅ Orderbook health check passed: Spread: {:.4}%, Liquidity: B:{:.2}/A:{:.2}",
                    spread_pct, total_bid_volume, total_ask_volume
                );
            }
        }

        // 检测交易量异常
        if !snapshot.trades.is_empty() {
            let total_trade_volume: f64 = snapshot.trades.iter().map(|t| t.quantity.0).sum();
            if total_trade_volume > self.volume_threshold * 10.0 {
                warn!("🚨 High trading volume detected: {:.4}", total_trade_volume);

                return Some(AnomalyDetectionResult {
                    anomaly_type: AnomalyType::Other("HighTradingVolume".to_string()),
                    severity: AnomalySeverity::Warning,
                    description: "High trading volume detected".to_string(),
                    details: format!(
                        "Total volume: {:.4}, Threshold: {:.4}",
                        total_trade_volume,
                        self.volume_threshold * 10.0
                    ),
                    timestamp: snapshot.timestamp,
                    symbol: if let Some(trade) = snapshot.trades.first() {
                        trade.symbol.clone()
                    } else {
                        crate::types::Symbol::new("UNKNOWN", "UNKNOWN")
                    },
                    source: snapshot.source.clone(),
                    recovery_suggestion: Some(
                        "Monitor for market manipulation or adjust high volume thresholds"
                            .to_string(),
                    ),
                });
            }
        }

        None
    }
}
