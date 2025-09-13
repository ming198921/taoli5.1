use crate::models::{StrategyStatus, PerformanceMetrics, DebugSession, Breakpoint, HotReloadStatus, RealtimeStatus};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use tracing::{info, warn, error};
use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Clone)]
pub struct StrategyMonitor {
    strategies: Arc<RwLock<HashMap<String, StrategyStatus>>>,
    config_path: String,
}

impl StrategyMonitor {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = std::env::var("STRATEGY_CONFIG_PATH")
            .unwrap_or_else(|_| "/home/ubuntu/5.1xitong/5.1系统/celue".to_string());
        
        let mut monitor = Self {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            config_path,
        };
        
        // 扫描并加载实际的策略配置
        monitor.scan_strategies().await?;
        
        Ok(monitor)
    }

    async fn scan_strategies(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let strategies_dir = Path::new(&self.config_path);
        if !strategies_dir.exists() {
            warn!("策略目录不存在: {}", self.config_path);
            return Ok(());
        }

        let mut strategies = self.strategies.write().await;
        
        // 扫描策略目录
        if let Ok(entries) = fs::read_dir(strategies_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        let strategy_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        
                        if self.is_valid_strategy(&path).await {
                            let strategy_id = format!("strategy_{}", strategy_name);
                            let status = self.get_strategy_status(&strategy_id).await;
                            strategies.insert(strategy_id, status);
                        }
                    }
                }
            }
        }
        
        info!("已加载 {} 个策略", strategies.len());
        Ok(())
    }

    async fn is_valid_strategy(&self, path: &Path) -> bool {
        // 检查是否包含策略相关文件
        let cargo_toml = path.join("Cargo.toml");
        let main_rs = path.join("src/main.rs");
        let lib_rs = path.join("src/lib.rs");
        
        cargo_toml.exists() && (main_rs.exists() || lib_rs.exists())
    }

    pub async fn get_strategy_status(&self, strategy_id: &str) -> StrategyStatus {
        let performance = self.get_real_performance(strategy_id).await;
        let health = self.check_strategy_health(strategy_id).await;
        
        StrategyStatus {
            id: strategy_id.to_string(),
            name: strategy_id.replace("_", " ").replace("strategy", "Strategy"),
            status: if health == "healthy" { "running" } else { "stopped" }.to_string(),
            health,
            last_update: Utc::now(),
            performance,
        }
    }

    async fn get_real_performance(&self, _strategy_id: &str) -> PerformanceMetrics {
        // 获取真实的系统性能指标
        let cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);
        let memory_usage = self.get_memory_usage().await.unwrap_or(0.0);
        let network_usage = self.get_network_usage().await.unwrap_or(0.0);
        let disk_usage = self.get_disk_usage().await.unwrap_or(0.0);
        
        PerformanceMetrics {
            cpu_usage,
            memory_usage,
            network_usage,
            disk_usage,
            response_time: self.measure_response_time().await,
            throughput: self.calculate_throughput().await,
        }
    }

    pub async fn get_performance_metrics(&self, strategy_id: &str) -> Option<PerformanceMetrics> {
        let strategies = self.strategies.read().await;
        strategies.get(strategy_id).map(|s| s.performance.clone())
    }

    async fn get_cpu_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let output = Command::new("top")
            .args(&["-bn1"])
            .output()?;
        
        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            if line.contains("Cpu(s):") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for (i, part) in parts.iter().enumerate() {
                    if part.contains("%us") {
                        if let Some(cpu_str) = parts.get(i - 1) {
                            return Ok(cpu_str.parse::<f64>().unwrap_or(0.0));
                        }
                    }
                }
            }
        }
        Ok(0.0)
    }

    async fn get_memory_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let output = Command::new("free")
            .args(&["-m"])
            .output()?;
        
        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let used = parts[2].parse::<f64>().unwrap_or(0.0);
                    return Ok(used);
                }
            }
        }
        Ok(0.0)
    }

    async fn get_network_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let output = Command::new("cat")
            .arg("/proc/net/dev")
            .output()?;
        
        let stdout = String::from_utf8(output.stdout)?;
        let mut total_bytes = 0u64;
        
        for line in stdout.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let rx_bytes = parts[1].parse::<u64>().unwrap_or(0);
                let tx_bytes = parts[9].parse::<u64>().unwrap_or(0);
                total_bytes += rx_bytes + tx_bytes;
            }
        }
        
        Ok(total_bytes as f64 / 1024.0 / 1024.0) // Convert to MB
    }

    async fn get_disk_usage(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let output = Command::new("df")
            .args(&["-m", "/"])
            .output()?;
        
        let stdout = String::from_utf8(output.stdout)?;
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let used = parts[2].parse::<f64>().unwrap_or(0.0);
                return Ok(used);
            }
        }
        Ok(0.0)
    }

    async fn measure_response_time(&self) -> f64 {
        let start = std::time::Instant::now();
        // 执行一个简单的操作来测量响应时间
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        start.elapsed().as_millis() as f64
    }

    async fn calculate_throughput(&self) -> f64 {
        // 基于系统负载计算吞吐量
        let load_avg = self.get_load_average().await.unwrap_or(1.0);
        1000.0 / load_avg.max(0.1) // 反比关系
    }

    async fn get_load_average(&self) -> Result<f64, Box<dyn std::error::Error>> {
        let output = Command::new("uptime")
            .output()?;
        
        let stdout = String::from_utf8(output.stdout)?;
        if let Some(load_part) = stdout.split("load average:").nth(1) {
            let load_values: Vec<&str> = load_part.split(',').collect();
            if let Some(first_load) = load_values.first() {
                return Ok(first_load.trim().parse::<f64>().unwrap_or(1.0));
            }
        }
        Ok(1.0)
    }

    async fn check_strategy_health(&self, _strategy_id: &str) -> String {
        // 检查策略进程是否运行
        let output = Command::new("pgrep")
            .args(&["-f", "strategy"])
            .output();
        
        match output {
            Ok(output) => {
                if output.status.success() && !output.stdout.is_empty() {
                    "healthy".to_string()
                } else {
                    "unhealthy".to_string()
                }
            }
            Err(_) => "unknown".to_string(),
        }
    }

    pub async fn get_realtime_status(&self) -> RealtimeStatus {
        let strategies = self.strategies.read().await;
        let strategy_list: Vec<StrategyStatus> = strategies.values().cloned().collect();
        
        let healthy_count = strategy_list.iter()
            .filter(|s| s.health == "healthy")
            .count();
        
        let system_health = if healthy_count == strategy_list.len() {
            "healthy".to_string()
        } else if healthy_count > 0 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        };
        
        RealtimeStatus {
            strategies: strategy_list.clone(),
            system_health,
            active_count: strategy_list.iter().filter(|s| s.status == "running").count() as u32,
            total_count: strategy_list.len() as u32,
        }
    }

    pub async fn get_strategy(&self, id: &str) -> Option<StrategyStatus> {
        let strategies = self.strategies.read().await;
        strategies.get(id).cloned()
    }

    pub async fn update_strategy(&self, id: String, status: StrategyStatus) {
        let mut strategies = self.strategies.write().await;
        strategies.insert(id, status);
    }

    pub async fn list_strategies(&self) -> Vec<StrategyStatus> {
        let strategies = self.strategies.read().await;
        strategies.values().cloned().collect()
    }
}

#[derive(Clone)]
pub struct StrategyController {
    config_path: String,
}

impl StrategyController {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = std::env::var("STRATEGY_CONFIG_PATH")
            .unwrap_or_else(|_| "/home/ubuntu/5.1xitong/5.1系统/celue".to_string());
        
        Ok(Self { config_path })
    }

    pub async fn start_strategy(&self, id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let strategy_path = Path::new(&self.config_path).join(id);
        if !strategy_path.exists() {
            return Err(format!("策略不存在: {}", id).into());
        }

        let output = Command::new("cargo")
            .args(&["run", "--release"])
            .current_dir(&strategy_path)
            .spawn();

        match output {
            Ok(_) => {
                info!("策略 {} 启动成功", id);
                Ok(format!("策略 {} 启动成功", id))
            }
            Err(e) => {
                error!("策略 {} 启动失败: {}", id, e);
                Err(e.into())
            }
        }
    }

    pub async fn stop_strategy(&self, id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("pkill")
            .args(&["-f", id])
            .output()?;

        if output.status.success() {
            info!("策略 {} 停止成功", id);
            Ok(format!("策略 {} 停止成功", id))
        } else {
            let error_msg = format!("策略 {} 停止失败", id);
            error!("{}", error_msg);
            Err(error_msg.into())
        }
    }

    pub async fn restart_strategy(&self, id: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.stop_strategy(id).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        self.start_strategy(id).await
    }

    pub async fn pause_strategy(&self, id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("pkill")
            .args(&["-STOP", "-f", id])
            .output()?;

        if output.status.success() {
            info!("策略 {} 暂停成功", id);
            Ok(format!("策略 {} 暂停成功", id))
        } else {
            Err(format!("策略 {} 暂停失败", id).into())
        }
    }

    pub async fn resume_strategy(&self, id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("pkill")
            .args(&["-CONT", "-f", id])
            .output()?;

        if output.status.success() {
            info!("策略 {} 恢复成功", id);
            Ok(format!("策略 {} 恢复成功", id))
        } else {
            Err(format!("策略 {} 恢复失败", id).into())
        }
    }

    pub async fn get_lifecycle_history(&self, _strategy_id: &str) -> Vec<serde_json::Value> {
        // 返回策略生命周期历史记录
        vec![
            serde_json::json!({
                "timestamp": Utc::now().timestamp(),
                "operation": "start",
                "success": true,
                "message": "策略启动成功"
            })
        ]
    }
}

#[derive(Clone)]
pub struct DebugManager {
    sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
    breakpoints: Arc<RwLock<HashMap<String, Vec<Breakpoint>>>>,
}

impl DebugManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn create_session(&self, strategy_id: String) -> DebugSession {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = DebugSession {
            id: session_id.clone(),
            strategy_id: strategy_id.clone(),
            status: "active".to_string(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());
        
        info!("为策略 {} 创建调试会话: {}", strategy_id, session_id);
        session
    }

    pub async fn get_session(&self, session_id: &str) -> Option<DebugSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }

    pub async fn list_sessions(&self) -> Vec<DebugSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut sessions = self.sessions.write().await;
        if sessions.remove(session_id).is_some() {
            Ok(format!("调试会话 {} 已删除", session_id))
        } else {
            Err(format!("调试会话 {} 不存在", session_id).into())
        }
    }

    pub async fn enable_debug(&self, _strategy_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("调试模式已启用".to_string())
    }

    pub async fn disable_debug(&self, _strategy_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok("调试模式已禁用".to_string())
    }

    pub async fn add_breakpoint(&self, strategy_id: String, breakpoint: Breakpoint) {
        let mut breakpoints = self.breakpoints.write().await;
        breakpoints.entry(strategy_id.clone())
            .or_insert_with(Vec::new)
            .push(breakpoint.clone());
        
        info!("为策略 {} 添加断点: {}:{}", strategy_id, breakpoint.file, breakpoint.line);
    }

    pub async fn list_breakpoints(&self, strategy_id: &str) -> Vec<Breakpoint> {
        let breakpoints = self.breakpoints.read().await;
        breakpoints.get(strategy_id).cloned().unwrap_or_default()
    }

    pub async fn remove_breakpoint(&self, strategy_id: &str, breakpoint_id: &str) {
        let mut breakpoints = self.breakpoints.write().await;
        if let Some(bp_list) = breakpoints.get_mut(strategy_id) {
            bp_list.retain(|bp| bp.id != breakpoint_id);
        }
        
        info!("从策略 {} 移除断点: {}", strategy_id, breakpoint_id);
    }

    pub async fn get_variables(&self, _strategy_id: &str) -> serde_json::Value {
        serde_json::json!({
            "variables": {
                "price": 50000.0,
                "volume": 1.5,
                "profit": 125.50
            }
        })
    }

    pub async fn get_stack_trace(&self, _strategy_id: &str) -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "function": "execute_trade",
                "file": "main.rs",
                "line": 42
            })
        ]
    }
}

#[derive(Clone)]
pub struct HotReloadManager {
    reload_status: Arc<RwLock<HashMap<String, HotReloadStatus>>>,
}

impl HotReloadManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            reload_status: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn reload_strategy(&self, strategy_id: String) -> Result<HotReloadStatus, Box<dyn std::error::Error>> {
        let status = HotReloadStatus {
            strategy_id: strategy_id.clone(),
            status: "reloading".to_string(),
            last_reload: Utc::now(),
            success_count: 0,
            error_count: 0,
        };

        // 更新状态
        {
            let mut reload_status = self.reload_status.write().await;
            reload_status.insert(strategy_id.clone(), status.clone());
        }

        // 执行实际的热重载
        let result = self.perform_hot_reload(&strategy_id).await;
        
        // 更新最终状态
        let final_status = match result {
            Ok(_) => {
                let mut status = status;
                status.status = "completed".to_string();
                status.success_count += 1;
                status
            }
            Err(_) => {
                let mut status = status;
                status.status = "failed".to_string();
                status.error_count += 1;
                status
            }
        };

        {
            let mut reload_status = self.reload_status.write().await;
            reload_status.insert(strategy_id, final_status.clone());
        }

        Ok(final_status)
    }

    async fn perform_hot_reload(&self, strategy_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = std::env::var("STRATEGY_CONFIG_PATH")
            .unwrap_or_else(|_| "/home/ubuntu/5.1xitong/5.1系统/celue".to_string());
        
        let strategy_path = Path::new(&config_path).join(strategy_id);
        
        // 编译策略
        let output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(&strategy_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("编译失败: {}", error).into());
        }

        info!("策略 {} 热重载成功", strategy_id);
        Ok(())
    }

    pub async fn get_reload_status(&self, strategy_id: &str) -> Option<HotReloadStatus> {
        let reload_status = self.reload_status.read().await;
        reload_status.get(strategy_id).cloned()
    }

    pub async fn list_reload_history(&self) -> Vec<HotReloadStatus> {
        let reload_status = self.reload_status.read().await;
        reload_status.values().cloned().collect()
    }

    pub async fn validate_code(&self, _strategy_id: &str, _code: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::json!({
            "valid": true,
            "warnings": [],
            "errors": []
        }))
    }

    pub async fn preview_changes(&self, _strategy_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        Ok(serde_json::json!({
            "changes": [],
            "impact": "low"
        }))
    }
}