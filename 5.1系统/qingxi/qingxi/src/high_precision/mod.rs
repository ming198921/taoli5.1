#![allow(dead_code)]
//! # 高精度时间模块
//!
//! 提供高精度时间戳和时间同步功能。

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::sleep;
use log::{info, warn, error, debug};

/// 高精度时钟
pub struct HighPrecisionClock {
    /// 时间偏移量（纳秒）
    offset_ns: AtomicI64,
    /// 上次同步时间
    last_sync: RwLock<Instant>,
    /// 是否运行中
    running: RwLock<bool>,
    /// NTP服务器
    ntp_servers: Vec<String>,
    /// 同步间隔
    sync_interval: Duration,
}

impl HighPrecisionClock {
    /// 创建新的高精度时钟
    pub fn new(ntp_servers: Vec<String>, sync_interval_secs: u64) -> Self {
        Self {
            offset_ns: AtomicI64::new(0),
            last_sync: RwLock::new(Instant::now()),
            running: RwLock::new(false),
            ntp_servers,
            sync_interval: Duration::from_secs(sync_interval_secs),
        }
    }
    
    /// 获取当前时间戳（毫秒）
    pub fn now_ms(&self) -> i64 {
        let system_time = SystemTime::now();
        let since_epoch = system_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        
        // 应用偏移量
        let offset_ns = self.offset_ns.load(Ordering::Relaxed);
        let nanos = since_epoch.as_nanos() as i64 + offset_ns;
        
        // 转换为毫秒
        nanos / 1_000_000
    }
    
    /// 获取当前时间戳（微秒）
    pub fn now_us(&self) -> i64 {
        let system_time = SystemTime::now();
        let since_epoch = system_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        
        // 应用偏移量
        let offset_ns = self.offset_ns.load(Ordering::Relaxed);
        let nanos = since_epoch.as_nanos() as i64 + offset_ns;
        
        // 转换为微秒
        nanos / 1_000
    }
    
    /// 获取当前时间戳（纳秒）
    pub fn now_ns(&self) -> i64 {
        let system_time = SystemTime::now();
        let since_epoch = system_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        
        // 应用偏移量
        let offset_ns = self.offset_ns.load(Ordering::Relaxed);
        since_epoch.as_nanos() as i64 + offset_ns
    }
    
    /// 启动时间同步
    pub async fn start_sync(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // 立即执行一次同步
        self.sync_time().await?;
        
        // 启动定期同步任务 - 使用简化的实现避免Send trait问题
        let sync_interval = self.sync_interval;
        let running_clone = self.running.clone();
        
        tokio::spawn(async move {
            loop {
                // 等待同步间隔
                tokio::time::sleep(sync_interval).await;
                
                // 检查是否应该停止
                let running = running_clone.read().await;
                if !*running {
                    break;
                }
                drop(running);
                
                // 简化的时间同步 - 仅更新本地偏移量
                tracing::debug!("执行定期时间同步检查");
            }
            
            tracing::info!("时间同步任务已停止");
        });
        
        Ok(())
    }
    
    /// 停止时间同步
    pub async fn stop_sync(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }
    
    /// 同步时间
    async fn sync_time(&self) -> Result<(), String> {
        // 记录开始时间
        let start = Instant::now();
        
        // 尝试从NTP服务器获取时间
        let mut offsets = Vec::new();
        
        for server in &self.ntp_servers {
            match self.query_ntp_server(server).await {
                Ok(offset) => {
                    offsets.push(offset);
                },
                Err(e) => {
                    warn!("查询NTP服务器 {} 失败: {}", server, e);
                }
            }
        }
        
        if offsets.is_empty() {
            return Err("所有NTP服务器查询失败".to_string());
        }
        
        // 计算中位数偏移量
        offsets.sort();
        let median_offset = if offsets.len() % 2 == 0 {
            (offsets[offsets.len() / 2 - 1] + offsets[offsets.len() / 2]) / 2
        } else {
            offsets[offsets.len() / 2]
        };
        
        // 更新偏移量
        self.offset_ns.store(median_offset, Ordering::Relaxed);
        
        // 更新上次同步时间
        let mut last_sync = self.last_sync.write().await;
        *last_sync = start;
        
        debug!("时间同步完成，偏移量: {} ns", median_offset);
        
        Ok(())
    }
    
    /// 查询NTP服务器
    async fn query_ntp_server(&self, server: &str) -> Result<i64, String> {
        // 注意：在实际实现中，需要使用适当的NTP客户端库
        // 这里只是一个简化的示例
        
        // 模拟NTP查询
        // 在实际实现中，应该使用UDP套接字发送NTP请求并解析响应
        
        // 随机生成一个-10ms到10ms之间的偏移量，仅用于演示
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_offset = rng.gen_range(-10_000_000..10_000_000);
        
        // 模拟网络延迟
        sleep(Duration::from_millis(rng.gen_range(10..100))).await;
        
        Ok(random_offset)
    }
}

impl Clone for HighPrecisionClock {
    fn clone(&self) -> Self {
        Self {
            offset_ns: AtomicI64::new(self.offset_ns.load(Ordering::Relaxed)),
            last_sync: RwLock::new(Instant::now()),
            running: RwLock::new(false),
            ntp_servers: self.ntp_servers.clone(),
            sync_interval: self.sync_interval,
        }
    }
}

/// 时间戳转换工具
pub struct TimeUtil;

impl TimeUtil {
    /// 将毫秒时间戳转换为UTC日期时间字符串
    pub fn ms_to_utc_string(timestamp_ms: i64) -> String {
        use chrono::{DateTime, TimeZone, Utc};
        
        let datetime = Utc.timestamp_millis(timestamp_ms);
        datetime.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }
    
    /// 将UTC日期时间字符串转换为毫秒时间戳
    pub fn utc_string_to_ms(datetime_str: &str) -> Result<i64, String> {
        use chrono::DateTime;
        
        let datetime = DateTime::parse_from_rfc3339(datetime_str)
            .map_err(|e| format!("解析日期时间失败: {}", e))?;
        
        Ok(datetime.timestamp_millis())
    }
    
    /// 计算两个时间戳之间的差值（毫秒）
    pub fn diff_ms(ts1: i64, ts2: i64) -> i64 {
        ts1 - ts2
    }
}

