//! QingXi 5.1 安全包装器模块
//! 提供生产级安全的unwrap()替代方案

use crate::types::*;
use anyhow::Result;
use tracing::{error, warn};

/// 安全包装器 - 替代unsafe unwrap()调用
pub struct SafeWrapper;

impl SafeWrapper {
    /// 安全获取Option值，提供默认值和日志
    pub fn safe_unwrap_option<T: Clone>(
        option: Option<T>,
        default: T,
        context: &str,
    ) -> T {
        match option {
            Some(value) => value,
            None => {
                warn!("⚠️ SafeWrapper: {} - 使用默认值", context);
                default
            }
        }
    }

    /// 安全获取Result值，转换错误并记录
    pub fn safe_unwrap_result<T, E: std::fmt::Display>(
        result: Result<T, E>,
        context: &str,
    ) -> Result<T> {
        result.map_err(|e| {
            error!("❌ SafeWrapper: {} - 错误: {}", context, e);
            anyhow::anyhow!("{}: {}", context, e)
        })
    }

    /// 安全的浮点数比较
    pub fn safe_f64_compare(a: f64, b: f64, precision: f64) -> std::cmp::Ordering {
        let diff = a - b;
        if diff.abs() < precision {
            std::cmp::Ordering::Equal
        } else if diff > 0.0 {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    }

    /// 安全的价格格式化
    pub fn safe_format_price(price: f64, precision: u32) -> Result<f64> {
        if !price.is_finite() || price < 0.0 {
            return Err(anyhow::anyhow!("Invalid price: {}", price));
        }
        
        let factor = 10_f64.powi(precision as i32);
        Ok((price * factor).round() / factor)
    }

    /// 安全的订单簿访问
    pub fn safe_best_bid(orderbook: &OrderBook) -> Option<f64> {
        orderbook.bids.first().map(|entry| entry.price.into_inner())
    }

    pub fn safe_best_ask(orderbook: &OrderBook) -> Option<f64> {
        orderbook.asks.first().map(|entry| entry.price.into_inner())
    }
}

/// 宏：安全获取配置值
#[macro_export]
macro_rules! safe_config {
    ($config:expr, $field:expr, $default:expr) => {
        $crate::error_handling::safe_wrapper::SafeWrapper::safe_unwrap_option(
            $config.$field,
            $default,
            stringify!($field)
        )
    };
}

/// 宏：安全执行可能失败的操作
#[macro_export]
macro_rules! safe_execute {
    ($operation:expr, $context:expr) => {
        $crate::error_handling::safe_wrapper::SafeWrapper::safe_unwrap_result(
            $operation,
            $context
        )
    };
}

