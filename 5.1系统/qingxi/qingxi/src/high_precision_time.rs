#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use std::ops::{Add, Sub};
use std::time::{SystemTime, UNIX_EPOCH};

/// 纳秒级高精度时间戳结构体
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Nanos(pub i64);

impl Nanos {
    /// 获取当前时间的纳秒时间戳
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Self(now.as_nanos() as i64)
    }

    /// 从毫秒创建纳秒时间戳
    pub fn from_millis(millis: i64) -> Self {
        Self(millis * 1_000_000)
    }

    /// 从微秒创建纳秒时间戳
    pub fn from_micros(micros: i64) -> Self {
        Self(micros * 1_000)
    }

    /// 从u64毫秒时间戳创建
    pub fn from_u64(millis: u64) -> Self {
        Self::from_millis(millis as i64)
    }

    /// 转换为毫秒
    pub fn as_millis(&self) -> i64 {
        self.0 / 1_000_000
    }

    /// 转换为微秒
    pub fn as_micros(&self) -> i64 {
        self.0 / 1_000
    }

    /// 转换为u64毫秒时间戳
    pub fn as_u64(&self) -> u64 {
        self.as_millis() as u64
    }

    /// 获取内部纳秒值
    pub fn as_nanos(&self) -> i64 {
        self.0
    }
}

impl Add for Nanos {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for Nanos {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl From<i64> for Nanos {
    fn from(nanos: i64) -> Self {
        Self(nanos)
    }
}

impl From<Nanos> for i64 {
    fn from(nanos: Nanos) -> Self {
        nanos.0
    }
}
