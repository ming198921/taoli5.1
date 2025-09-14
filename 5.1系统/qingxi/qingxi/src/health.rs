#![allow(dead_code)]
// src/health.rs
//! # 健康监控模块
//!
//! 负责监控各个数据源的健康状态，包括连接状态、延迟和最后接收消息时间

use crate::high_precision_time::Nanos;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// 数据源健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// 数据源标识符（如 "binance-BTC/USDT"）
    pub source_id: String,
    /// 最后接收消息时间
    pub last_message_at: Nanos,
    /// 延迟（微秒）
    pub latency_us: u64,
    /// 消息计数
    pub message_count: u64,
    /// 连接状态
    pub is_connected: bool,
    /// 最后错误信息
    pub last_error: Option<String>,
    /// 最后错误时间
    pub last_error_at: Option<Nanos>,
}

impl HealthStatus {
    pub fn new(source_id: String) -> Self {
        Self {
            source_id,
            last_message_at: Nanos::now(),
            latency_us: 0,
            message_count: 0,
            is_connected: false,
            last_error: None,
            last_error_at: None,
        }
    }

    /// 检查数据源是否健康（在指定超时时间内有消息）
    pub fn is_healthy(&self, timeout_ms: u64) -> bool {
        let timeout_nanos = Nanos::from_millis(timeout_ms as i64);
        let now = Nanos::now();

        self.is_connected && (now - self.last_message_at) < timeout_nanos
    }

    /// 更新消息接收状态
    pub fn update_message_received(&mut self, latency_us: u64) {
        self.last_message_at = Nanos::now();
        self.latency_us = latency_us;
        self.message_count += 1;
        self.is_connected = true;

        debug!(
            "Updated health status for {}: latency={}μs, count={}",
            self.source_id, latency_us, self.message_count
        );
    }

    /// 设置错误状态
    pub fn set_error(&mut self, error: String) {
        self.last_error = Some(error.clone());
        self.last_error_at = Some(Nanos::now());
        self.is_connected = false;

        warn!("Health error for {}: {}", self.source_id, error);
    }

    /// 设置连接状态
    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;

        if connected {
            info!("Data source {} connected", self.source_id);
        } else {
            warn!("Data source {} disconnected", self.source_id);
        }
    }
}

/// API健康监控器
#[derive(Debug)]
pub struct ApiHealthMonitor {
    /// 数据源健康状态映射表
    health_statuses: Arc<DashMap<String, HealthStatus>>,
    /// 健康检查超时时间（毫秒）
    health_timeout_ms: u64,
}

impl ApiHealthMonitor {
    /// 创建新的健康监控器
    pub fn new(health_timeout_ms: u64) -> Self {
        Self {
            health_statuses: Arc::new(DashMap::new()),
            health_timeout_ms,
        }
    }

    /// 获取或创建数据源的健康状态
    fn get_or_create_status(
        &self,
        source_id: &str,
    ) -> dashmap::mapref::one::RefMut<'_, String, HealthStatus> {
        self.health_statuses
            .entry(source_id.to_string())
            .or_insert_with(|| {
                info!("Creating health status for new source: {}", source_id);
                HealthStatus::new(source_id.to_string())
            })
    }

    /// 更新数据源状态 - 消息接收
    pub fn update_message_received(&self, source_id: &str, latency_us: u64) {
        let mut status = self.get_or_create_status(source_id);
        status.update_message_received(latency_us);
    }

    /// 更新数据源状态 - 连接状态
    pub fn update_connection_status(&self, source_id: &str, connected: bool) {
        let mut status = self.get_or_create_status(source_id);
        status.set_connected(connected);
    }

    /// 更新数据源状态 - 错误状态
    pub fn update_error_status(&self, source_id: &str, error: String) {
        let mut status = self.get_or_create_status(source_id);
        status.set_error(error);
    }

    /// 获取特定数据源的健康状态
    pub fn get_health_status(&self, source_id: &str) -> Option<HealthStatus> {
        self.health_statuses
            .get(source_id)
            .map(|status| status.clone())
    }

    /// 获取所有数据源的健康状态
    pub fn get_all_health_statuses(&self) -> Vec<HealthStatus> {
        self.health_statuses
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// 获取健康的数据源列表
    pub fn get_healthy_sources(&self) -> Vec<String> {
        self.health_statuses
            .iter()
            .filter(|entry| entry.value().is_healthy(self.health_timeout_ms))
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// 获取不健康的数据源列表
    pub fn get_unhealthy_sources(&self) -> Vec<String> {
        self.health_statuses
            .iter()
            .filter(|entry| !entry.value().is_healthy(self.health_timeout_ms))
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// 清理过期的数据源状态
    pub fn cleanup_stale_sources(&self, stale_timeout_ms: u64) {
        let stale_timeout = Nanos::from_millis(stale_timeout_ms as i64);
        let now = Nanos::now();

        let stale_sources: Vec<String> = self
            .health_statuses
            .iter()
            .filter(|entry| (now - entry.value().last_message_at) > stale_timeout)
            .map(|entry| entry.key().clone())
            .collect();

        for source_id in stale_sources {
            info!("Removing stale health status for source: {}", source_id);
            self.health_statuses.remove(&source_id);
        }
    }

    /// 生成健康报告摘要
    pub fn get_health_summary(&self) -> HealthSummary {
        let all_statuses = self.get_all_health_statuses();
        let total_sources = all_statuses.len();
        let healthy_count = all_statuses
            .iter()
            .filter(|status| status.is_healthy(self.health_timeout_ms))
            .count();

        let average_latency = if !all_statuses.is_empty() {
            all_statuses
                .iter()
                .map(|status| status.latency_us)
                .sum::<u64>()
                / all_statuses.len() as u64
        } else {
            0
        };

        let total_messages = all_statuses.iter().map(|status| status.message_count).sum();

        HealthSummary {
            total_sources,
            healthy_sources: healthy_count,
            unhealthy_sources: total_sources - healthy_count,
            average_latency_us: average_latency,
            total_messages,
            timestamp: Nanos::now(),
        }
    }
}

/// 健康状态摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub total_sources: usize,
    pub healthy_sources: usize,
    pub unhealthy_sources: usize,
    pub average_latency_us: u64,
    pub total_messages: u64,
    pub timestamp: Nanos,
}
