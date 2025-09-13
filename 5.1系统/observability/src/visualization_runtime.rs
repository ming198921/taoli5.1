//! 可视化管理器运行时间显示
//!
//! 提供系统运行时间的可视化展示和统计功能

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// 运行时间可视化管理器
pub struct RuntimeVisualizationManager {
    /// 启动时间
    start_time: Instant,
    /// 组件运行时间记录
    component_runtimes: Arc<RwLock<HashMap<String, ComponentRuntime>>>,
    /// 运行时间统计
    runtime_stats: Arc<RwLock<RuntimeStatistics>>,
}

/// 组件运行时间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRuntime {
    pub component_name: String,
    pub start_time: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub status: RuntimeStatus,
    pub restart_count: u32,
    pub last_restart: Option<DateTime<Utc>>,
}

/// 运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeStatus {
    Running,
    Stopped,
    Restarting,
    Error,
}

/// 运行时间统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatistics {
    pub total_uptime_seconds: u64,
    pub average_uptime_per_component: f64,
    pub longest_running_component: Option<String>,
    pub most_restarted_component: Option<String>,
    pub system_availability_percent: f64,
}

/// 运行时间展示格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDisplay {
    pub formatted_uptime: String,
    pub uptime_days: u64,
    pub uptime_hours: u64,
    pub uptime_minutes: u64,
    pub uptime_seconds: u64,
    pub start_timestamp: DateTime<Utc>,
    pub current_timestamp: DateTime<Utc>,
}

impl RuntimeVisualizationManager {
    /// 创建新的运行时间可视化管理器
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            component_runtimes: Arc::new(RwLock::new(HashMap::new())),
            runtime_stats: Arc::new(RwLock::new(RuntimeStatistics {
                total_uptime_seconds: 0,
                average_uptime_per_component: 0.0,
                longest_running_component: None,
                most_restarted_component: None,
                system_availability_percent: 100.0,
            })),
        }
    }

    /// 注册组件
    pub async fn register_component(&self, component_name: &str) -> Result<()> {
        let mut runtimes = self.component_runtimes.write().await;
        runtimes.insert(
            component_name.to_string(),
            ComponentRuntime {
                component_name: component_name.to_string(),
                start_time: Utc::now(),
                uptime_seconds: 0,
                status: RuntimeStatus::Running,
                restart_count: 0,
                last_restart: None,
            },
        );
        Ok(())
    }

    /// 更新组件运行时间
    pub async fn update_component_runtime(&self, component_name: &str) -> Result<()> {
        let mut runtimes = self.component_runtimes.write().await;
        if let Some(runtime) = runtimes.get_mut(component_name) {
            let elapsed = Utc::now() - runtime.start_time;
            runtime.uptime_seconds = elapsed.num_seconds() as u64;
        }
        Ok(())
    }

    /// 记录组件重启
    pub async fn record_component_restart(&self, component_name: &str) -> Result<()> {
        let mut runtimes = self.component_runtimes.write().await;
        if let Some(runtime) = runtimes.get_mut(component_name) {
            runtime.restart_count += 1;
            runtime.last_restart = Some(Utc::now());
            runtime.start_time = Utc::now();
            runtime.uptime_seconds = 0;
            runtime.status = RuntimeStatus::Restarting;
        }
        Ok(())
    }

    /// 获取系统运行时间显示
    pub async fn get_runtime_display(&self) -> RuntimeDisplay {
        let elapsed = self.start_time.elapsed();
        let total_seconds = elapsed.as_secs();
        
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;
        
        let formatted = format!(
            "{}d {}h {}m {}s",
            days, hours, minutes, seconds
        );
        
        RuntimeDisplay {
            formatted_uptime: formatted,
            uptime_days: days,
            uptime_hours: hours,
            uptime_minutes: minutes,
            uptime_seconds: seconds,
            start_timestamp: Utc::now() - Duration::seconds(total_seconds as i64),
            current_timestamp: Utc::now(),
        }
    }

    /// 获取组件运行时间统计
    pub async fn get_runtime_statistics(&self) -> Result<RuntimeStatistics> {
        let runtimes = self.component_runtimes.read().await;
        
        let total_uptime: u64 = runtimes.values().map(|r| r.uptime_seconds).sum();
        let component_count = runtimes.len() as f64;
        let average_uptime = if component_count > 0.0 {
            total_uptime as f64 / component_count
        } else {
            0.0
        };
        
        let longest_running = runtimes
            .values()
            .max_by_key(|r| r.uptime_seconds)
            .map(|r| r.component_name.clone());
        
        let most_restarted = runtimes
            .values()
            .max_by_key(|r| r.restart_count)
            .map(|r| r.component_name.clone());
        
        let running_count = runtimes
            .values()
            .filter(|r| r.status == RuntimeStatus::Running)
            .count() as f64;
        
        let availability = if component_count > 0.0 {
            (running_count / component_count) * 100.0
        } else {
            100.0
        };
        
        Ok(RuntimeStatistics {
            total_uptime_seconds: total_uptime,
            average_uptime_per_component: average_uptime,
            longest_running_component: longest_running,
            most_restarted_component: most_restarted,
            system_availability_percent: availability,
        })
    }

    /// 获取所有组件运行时间
    pub async fn get_all_component_runtimes(&self) -> HashMap<String, ComponentRuntime> {
        let runtimes = self.component_runtimes.read().await;
        runtimes.clone()
    }

    /// 生成运行时间报告
    pub async fn generate_runtime_report(&self) -> Result<RuntimeReport> {
        let display = self.get_runtime_display().await;
        let stats = self.get_runtime_statistics().await?;
        let components = self.get_all_component_runtimes().await;
        
        Ok(RuntimeReport {
            system_runtime: display,
            statistics: stats,
            component_details: components,
            generated_at: Utc::now(),
        })
    }
}

/// 运行时间报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReport {
    pub system_runtime: RuntimeDisplay,
    pub statistics: RuntimeStatistics,
    pub component_details: HashMap<String, ComponentRuntime>,
    pub generated_at: DateTime<Utc>,
}

/// 格式化运行时间为人类可读格式
pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, secs)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_visualization_manager() {
        let manager = RuntimeVisualizationManager::new();
        
        // 注册组件
        manager.register_component("component1").await.unwrap();
        manager.register_component("component2").await.unwrap();
        
        // 更新运行时间
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        manager.update_component_runtime("component1").await.unwrap();
        
        // 获取显示
        let display = manager.get_runtime_display().await;
        assert!(!display.formatted_uptime.is_empty());
        
        // 获取统计
        let stats = manager.get_runtime_statistics().await.unwrap();
        assert_eq!(stats.system_availability_percent, 100.0);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(59), "59s");
        assert_eq!(format_duration(60), "1m 0s");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(86461), "1d 0h 1m 1s");
    }
}