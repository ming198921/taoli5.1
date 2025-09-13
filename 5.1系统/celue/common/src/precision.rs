//! Fixed-point arithmetic for high precision financial calculations
//! As specified in section 6.2 of the design document

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub, Mul, Div};

/// Fixed-point price representation using i64 with scale
/// Avoids floating-point precision issues in financial calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FixedPrice {
    /// Raw value in the smallest unit
    raw: i64,
    /// Scale factor (number of decimal places)
    scale: u8,
}

impl FixedPrice {
    /// Create a new fixed-point price from raw value and scale
    pub fn from_raw(raw: i64, scale: u8) -> Self {
        Self { raw, scale }
    }
    
    /// Create from floating point with specified scale
    pub fn from_f64(value: f64, scale: u8) -> Self {
        let multiplier = 10_i64.pow(scale as u32);
        let raw = (value * multiplier as f64).round() as i64;
        Self { raw, scale }
    }
    
    /// Convert to f64 for display/output
    pub fn to_f64(&self) -> f64 {
        let divisor = 10_f64.powi(self.scale as i32);
        self.raw as f64 / divisor
    }
    
    /// Get raw value
    pub fn raw_value(&self) -> i64 {
        self.raw
    }
    
    /// Get raw value (for internal calculations only)
    pub fn raw(&self) -> i64 {
        self.raw
    }
    
    /// Get scale
    pub fn scale(&self) -> u8 {
        self.scale
    }
    
    /// Check if price is positive
    pub fn is_positive(&self) -> bool {
        self.raw > 0
    }
    
    /// Check if price is zero
    pub fn is_zero(&self) -> bool {
        self.raw == 0
    }
    
    /// Normalize two prices to the same scale for arithmetic operations
    fn normalize_scale(&self, other: FixedPrice) -> (FixedPrice, FixedPrice) {
        let max_scale = self.scale.max(other.scale);
        let self_normalized = if self.scale < max_scale {
            let multiplier = 10_i64.pow((max_scale - self.scale) as u32);
            Self { raw: self.raw * multiplier, scale: max_scale }
        } else {
            *self
        };
        let other_normalized = if other.scale < max_scale {
            let multiplier = 10_i64.pow((max_scale - other.scale) as u32);
            Self { raw: other.raw * multiplier, scale: max_scale }
        } else {
            other
        };
        (self_normalized, other_normalized)
    }
}

impl Add for FixedPrice {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        let (a, b) = self.normalize_scale(other);
        Self {
            raw: a.raw.saturating_add(b.raw),
            scale: a.scale,
        }
    }
}

impl Sub for FixedPrice {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        let (a, b) = self.normalize_scale(other);
        Self {
            raw: a.raw.saturating_sub(b.raw),
            scale: a.scale,
        }
    }
}

impl Mul for FixedPrice {
    type Output = Self;
    
    fn mul(self, other: Self) -> Self {
        // Use i128 to prevent overflow during multiplication
        let result = (self.raw as i128) * (other.raw as i128);
        let _scale_sum = self.scale + other.scale;
        
        // Divide by one scale factor to maintain reasonable scale
        let divisor = 10_i128.pow(other.scale as u32);
        let normalized_result = result / divisor;
        
        Self {
            raw: normalized_result.clamp(i64::MIN as i128, i64::MAX as i128) as i64,
            scale: self.scale,
        }
    }
}

impl Div for FixedPrice {
    type Output = Self;
    
    fn div(self, other: Self) -> Self {
        if other.raw == 0 {
            panic!("Division by zero in FixedPrice");
        }
        
        // Use i128 for precision
        let multiplier = 10_i128.pow(other.scale as u32);
        let numerator = (self.raw as i128) * multiplier;
        let result = numerator / (other.raw as i128);
        
        Self {
            raw: result.clamp(i64::MIN as i128, i64::MAX as i128) as i64,
            scale: self.scale,
        }
    }
}

impl fmt::Display for FixedPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1$}", self.to_f64(), self.scale as usize)
    }
}

/// Fixed-point quantity representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FixedQuantity {
    raw: i64,
    scale: u8,
}

impl FixedQuantity {
    /// Create a new fixed-point quantity from raw value and scale
    pub fn from_raw(raw: i64, scale: u8) -> Self {
        Self { raw, scale }
    }
    
    /// Create from floating point with specified scale
    pub fn from_f64(value: f64, scale: u8) -> Self {
        let multiplier = 10_i64.pow(scale as u32);
        let raw = (value * multiplier as f64).round() as i64;
        Self { raw, scale }
    }
    
    /// Convert to f64 for display/output
    pub fn to_f64(&self) -> f64 {
        let divisor = 10_f64.powi(self.scale as i32);
        self.raw as f64 / divisor
    }
    
    /// Get raw value
    pub fn raw_value(&self) -> i64 {
        self.raw
    }
    
    /// Get raw value (for internal calculations only)
    pub fn raw(&self) -> i64 {
        self.raw
    }
    
    /// Get scale
    pub fn scale(&self) -> u8 {
        self.scale
    }
    
    /// Check if quantity is positive
    pub fn is_positive(&self) -> bool {
        self.raw > 0
    }
    
    /// Check if quantity is zero
    pub fn is_zero(&self) -> bool {
        self.raw == 0
    }
}

impl fmt::Display for FixedQuantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1$}", self.to_f64(), self.scale as usize)
    }
}

// Arithmetic operations for FixedQuantity
impl Add for FixedQuantity {
    type Output = Self;
    
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            raw: self.raw + rhs.raw,
            scale: self.scale.max(rhs.scale),
        }
    }
}

impl Sub for FixedQuantity {
    type Output = Self;
    
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            raw: self.raw - rhs.raw,
            scale: self.scale.max(rhs.scale),
        }
    }
}

impl Mul for FixedQuantity {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            raw: self.raw * rhs.raw / (10_i64.pow(rhs.scale as u32)),
            scale: self.scale,
        }
    }
}

impl Div for FixedQuantity {
    type Output = Self;
    
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            raw: self.raw * (10_i64.pow(rhs.scale as u32)) / rhs.raw,
            scale: self.scale,
        }
    }
}

// Cross-type operations between FixedPrice and FixedQuantity
impl Mul<FixedPrice> for FixedQuantity {
    type Output = FixedPrice;
    
    fn mul(self, rhs: FixedPrice) -> Self::Output {
        FixedPrice {
            raw: (self.raw * rhs.raw) / (10_i64.pow(self.scale as u32)),
            scale: rhs.scale,
        }
    }
}

impl Mul<FixedQuantity> for FixedPrice {
    type Output = FixedPrice;
    
    fn mul(self, rhs: FixedQuantity) -> Self::Output {
        FixedPrice {
            raw: (self.raw * rhs.raw) / (10_i64.pow(rhs.scale as u32)),
            scale: self.scale,
        }
    }
}

// Division operations for FixedQuantity with FixedPrice
impl Div<FixedPrice> for FixedQuantity {
    type Output = FixedQuantity;
    
    fn div(self, rhs: FixedPrice) -> Self::Output {
        FixedQuantity {
            raw: (self.raw * (10_i64.pow(rhs.scale as u32))) / rhs.raw,
            scale: self.scale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_price_creation() {
        let price = FixedPrice::from_f64(100.25, 2);
        assert_eq!(price.to_f64(), 100.25);
        assert_eq!(price.raw_value(), 10025);
        assert_eq!(price.scale(), 2);
    }

    #[test]
    fn test_fixed_price_arithmetic() {
        let p1 = FixedPrice::from_f64(100.0, 2);
        let p2 = FixedPrice::from_f64(50.0, 2);
        
        let sum = p1 + p2;
        assert_eq!(sum.to_f64(), 150.0);
        
        let diff = p1 - p2;
        assert_eq!(diff.to_f64(), 50.0);
    }

    #[test]
    fn test_scale_normalization() {
        let p1 = FixedPrice::from_f64(100.0, 2); // 100.00
        let p2 = FixedPrice::from_f64(1.5, 1);   // 1.5
        
        let sum = p1 + p2;
        assert_eq!(sum.to_f64(), 101.5);
    }
    
    #[test]
    fn test_cross_type_multiplication() {
        let price = FixedPrice::from_f64(10.0, 2);
        let quantity = FixedQuantity::from_f64(5.0, 3);
        
        let total = price * quantity;
        assert_eq!(total.to_f64(), 50.0);
        
        let total2 = quantity * price;
        assert_eq!(total2.to_f64(), 50.0);
    }
}
