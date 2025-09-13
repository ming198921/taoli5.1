use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, RwLock as AsyncRwLock};
use tokio::time::{interval, Instant};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use prometheus::{Counter, Gauge, Histogram, Registry, TextEncoder};

/// Enhanced monitoring system with real-time metrics and alerting
pub struct EnhancedMonitoringSystem {
    metrics_store: Arc<AsyncRwLock<MetricsStore>>,
    alert_manager: Arc<AlertManager>,
    health_checker: Arc<HealthChecker>,
    event_stream: broadcast::Sender<MonitoringEvent>,
    registry: Registry,
    config: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_retention_days: u32,
    pub alert_cooldown_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub metrics_collection_interval_seconds: u64,
    pub enable_prometheus: bool,
    pub prometheus_port: u16,
    pub alert_webhook_url: Option<String>,
    pub alert_email_recipients: Vec<String>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_retention_days: 30,
            alert_cooldown_seconds: 300,
            health_check_interval_seconds: 30,
            metrics_collection_interval_seconds: 10,
            enable_prometheus: true,
            prometheus_port: 9090,
            alert_webhook_url: None,
            alert_email_recipients: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MetricsStore {
    metrics: HashMap<String, Vec<MetricPoint>>,
    last_cleanup: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringEvent {
    MetricUpdated { 
        name: String, 
        value: f64, 
        timestamp: u64 
    },
    Alert { 
        level: AlertLevel, 
        message: String, 
        component: String 
    },
    HealthCheckFailed { 
        component: String, 
        error: String 
    },
    SystemStatus { 
        status: SystemHealth 
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}

/// Alert manager with deduplication and routing
pub struct AlertManager {
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    cooldown_tracker: Arc<RwLock<HashMap<String, SystemTime>>>,
    alert_rules: Vec<AlertRule>,
    config: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub component: String,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub acknowledged: bool,
    pub resolved: bool,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub level: AlertLevel,
    pub message_template: String,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    RateIncreasing,
    RateDecreasing,
}

/// Health checker for system components
pub struct HealthChecker {
    components: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    checks: Vec<HealthCheck>,
}

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub component: String,
    pub check_fn: Arc<dyn Fn() -> Result<HealthStatus> + Send + Sync>,
    pub interval: Duration,
    pub timeout: Duration,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub last_check: SystemTime,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Error,
    Unknown,
}

impl EnhancedMonitoringSystem {
    pub fn new(config: MonitoringConfig) -> Result<Self> {
        let (event_sender, _) = broadcast::channel(10000);
        let registry = Registry::new();
        
        let alert_manager = Arc::new(AlertManager::new(config.clone()));
        let health_checker = Arc::new(HealthChecker::new());
        
        Ok(Self {
            metrics_store: Arc::new(AsyncRwLock::new(MetricsStore {
                metrics: HashMap::new(),
                last_cleanup: SystemTime::now(),
            })),
            alert_manager,
            health_checker,
            event_stream: event_sender,
            registry,
            config,
        })
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<()> {
        info!("Starting enhanced monitoring system");
        
        self.start_metrics_collector().await?;
        self.start_health_checker().await?;
        self.start_alert_processor().await?;
        
        if self.config.enable_prometheus {
            self.start_prometheus_server().await?;
        }
        
        self.start_cleanup_task().await?;
        
        info!("Enhanced monitoring system started successfully");
        Ok(())
    }

    /// Record a metric
    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: Option<HashMap<String, String>>,
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let metric_point = MetricPoint {
            timestamp,
            value,
            labels: labels.unwrap_or_default(),
        };

        {
            let mut store = self.metrics_store.write().await;
            store.metrics
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(metric_point);
        }

        let _ = self.event_stream.send(MonitoringEvent::MetricUpdated {
            name: name.to_string(),
            value,
            timestamp,
        });

        Ok(())
    }

    /// Get metric history
    pub async fn get_metric_history(
        &self,
        name: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<Vec<MetricPoint>> {
        let store = self.metrics_store.read().await;
        
        if let Some(points) = store.metrics.get(name) {
            let filtered = points
                .iter()
                .filter(|p| {
                    if let Some(from) = from {
                        p.timestamp >= from
                    } else {
                        true
                    }
                })
                .filter(|p| {
                    if let Some(to) = to {
                        p.timestamp <= to
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();
            Ok(filtered)
        } else {
            Ok(Vec::new())
        }
    }

    /// Create an alert
    pub async fn create_alert(
        &self,
        level: AlertLevel,
        title: &str,
        message: &str,
        component: &str,
        labels: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let alert_id = uuid::Uuid::new_v4().to_string();
        
        let alert = Alert {
            id: alert_id.clone(),
            level: level.clone(),
            title: title.to_string(),
            message: message.to_string(),
            component: component.to_string(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            acknowledged: false,
            resolved: false,
            labels: labels.unwrap_or_default(),
        };

        self.alert_manager.add_alert(alert).await?;

        let _ = self.event_stream.send(MonitoringEvent::Alert {
            level,
            message: message.to_string(),
            component: component.to_string(),
        });

        Ok(alert_id)
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        self.alert_manager.get_active_alerts().await
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        self.alert_manager.acknowledge_alert(alert_id).await
    }

    /// Get system health status
    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        self.health_checker.get_overall_health().await
    }

    /// Subscribe to monitoring events
    pub fn subscribe(&self) -> broadcast::Receiver<MonitoringEvent> {
        self.event_stream.subscribe()
    }

    /// Start metrics collection task
    async fn start_metrics_collector(&self) -> Result<()> {
        let store = Arc::clone(&self.metrics_store);
        let interval_secs = self.config.metrics_collection_interval_seconds;
        let event_sender = self.event_stream.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                // Collect system metrics
                if let Ok(cpu_usage) = Self::get_cpu_usage().await {
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    
                    let mut store_guard = store.write().await;
                    store_guard.metrics
                        .entry("system_cpu_usage".to_string())
                        .or_insert_with(Vec::new)
                        .push(MetricPoint {
                            timestamp,
                            value: cpu_usage,
                            labels: HashMap::new(),
                        });
                    drop(store_guard);

                    let _ = event_sender.send(MonitoringEvent::MetricUpdated {
                        name: "system_cpu_usage".to_string(),
                        value: cpu_usage,
                        timestamp,
                    });
                }

                if let Ok(memory_usage) = Self::get_memory_usage().await {
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    
                    let mut store_guard = store.write().await;
                    store_guard.metrics
                        .entry("system_memory_usage".to_string())
                        .or_insert_with(Vec::new)
                        .push(MetricPoint {
                            timestamp,
                            value: memory_usage,
                            labels: HashMap::new(),
                        });
                    drop(store_guard);

                    let _ = event_sender.send(MonitoringEvent::MetricUpdated {
                        name: "system_memory_usage".to_string(),
                        value: memory_usage,
                        timestamp,
                    });
                }
            }
        });

        Ok(())
    }

    /// Start health checking task
    async fn start_health_checker(&self) -> Result<()> {
        let health_checker = Arc::clone(&self.health_checker);
        let event_sender = self.event_stream.clone();
        let interval_secs = self.config.health_check_interval_seconds;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                if let Ok(results) = health_checker.run_all_checks().await {
                    for result in results {
                        if result.status != HealthStatus::Healthy {
                            let _ = event_sender.send(MonitoringEvent::HealthCheckFailed {
                                component: result.component.clone(),
                                error: result.message.clone(),
                            });
                        }
                    }
                }
                
                if let Ok(overall_health) = health_checker.get_overall_health().await {
                    let _ = event_sender.send(MonitoringEvent::SystemStatus {
                        status: overall_health,
                    });
                }
            }
        });

        Ok(())
    }

    /// Start alert processing task
    async fn start_alert_processor(&self) -> Result<()> {
        let alert_manager = Arc::clone(&self.alert_manager);
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                if let Err(e) = alert_manager.process_alert_rules().await {
                    error!("Error processing alert rules: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Start Prometheus metrics server
    async fn start_prometheus_server(&self) -> Result<()> {
        let registry = self.registry.clone();
        let port = self.config.prometheus_port;
        
        // Register default metrics
        self.register_prometheus_metrics()?;
        
        tokio::spawn(async move {
            let addr = ([0, 0, 0, 0], port).into();
            
            let make_svc = hyper::service::make_service_fn(move |_conn| {
                let registry = registry.clone();
                async move {
                    Ok::<_, hyper::Error>(hyper::service::service_fn(move |req| {
                        let registry = registry.clone();
                        async move {
                            if req.uri().path() == "/metrics" {
                                let encoder = TextEncoder::new();
                                let metric_families = registry.gather();
                                let mut buffer = Vec::new();
                                if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
                                    error!("Failed to encode metrics: {}", e);
                                    return Ok::<_, hyper::Error>(hyper::Response::builder()
                                        .status(500)
                                        .body(hyper::Body::from("Internal Server Error"))?);
                                }
                                
                                Ok::<_, hyper::Error>(hyper::Response::builder()
                                    .header("Content-Type", encoder.format_type())
                                    .body(hyper::Body::from(buffer))?)
                            } else {
                                Ok::<_, hyper::Error>(hyper::Response::builder()
                                    .status(404)
                                    .body(hyper::Body::from("Not Found"))?)
                            }
                        }
                    }))
                }
            });
            
            let server = hyper::Server::bind(&addr).serve(make_svc);
            info!("Prometheus metrics server listening on port {}", port);
            
            if let Err(e) = server.await {
                error!("Prometheus server error: {}", e);
            }
        });

        Ok(())
    }

    /// Start cleanup task for old metrics
    async fn start_cleanup_task(&self) -> Result<()> {
        let store = Arc::clone(&self.metrics_store);
        let retention_days = self.config.metrics_retention_days;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(86400)); // Daily cleanup
            
            loop {
                interval.tick().await;
                
                let cutoff = SystemTime::now()
                    .checked_sub(Duration::from_secs(retention_days as u64 * 86400))
                    .unwrap_or(UNIX_EPOCH);
                
                let cutoff_timestamp = cutoff
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_secs();

                let mut store_guard = store.write().await;
                for (_, points) in store_guard.metrics.iter_mut() {
                    points.retain(|p| p.timestamp >= cutoff_timestamp);
                }
                store_guard.last_cleanup = SystemTime::now();
                drop(store_guard);
                
                info!("Completed metrics cleanup, removed data older than {} days", retention_days);
            }
        });

        Ok(())
    }

    fn register_prometheus_metrics(&self) -> Result<()> {
        // Register system metrics
        let cpu_gauge = Gauge::new("system_cpu_usage_percent", "System CPU usage percentage")?;
        let memory_gauge = Gauge::new("system_memory_usage_percent", "System memory usage percentage")?;
        let alerts_counter = Counter::new("alerts_total", "Total number of alerts")?;
        
        self.registry.register(Box::new(cpu_gauge))?;
        self.registry.register(Box::new(memory_gauge))?;
        self.registry.register(Box::new(alerts_counter))?;
        
        Ok(())
    }

    async fn get_cpu_usage() -> Result<f64> {
        // Simplified CPU usage calculation
        // In production, use a proper system metrics library
        use std::fs;
        
        let stat1 = fs::read_to_string("/proc/stat")?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        let stat2 = fs::read_to_string("/proc/stat")?;
        
        let parse_cpu_line = |line: &str| -> Option<(u64, u64)> {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 4 && parts[0] == "cpu" {
                let idle: u64 = parts[4].parse().ok()?;
                let total: u64 = parts[1..].iter()
                    .filter_map(|s| s.parse::<u64>().ok())
                    .sum();
                Some((total, idle))
            } else {
                None
            }
        };
        
        if let (Some((total1, idle1)), Some((total2, idle2))) = (
            stat1.lines().next().and_then(parse_cpu_line),
            stat2.lines().next().and_then(parse_cpu_line)
        ) {
            let total_diff = total2.saturating_sub(total1);
            let idle_diff = idle2.saturating_sub(idle1);
            
            if total_diff > 0 {
                let usage = 100.0 * (1.0 - (idle_diff as f64 / total_diff as f64));
                return Ok(usage);
            }
        }
        
        Ok(0.0)
    }

    async fn get_memory_usage() -> Result<f64> {
        use std::fs;
        
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut total = 0u64;
        let mut available = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        if total > 0 {
            Ok(100.0 * (1.0 - (available as f64 / total as f64)))
        } else {
            Ok(0.0)
        }
    }
}

impl AlertManager {
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            cooldown_tracker: Arc::new(RwLock::new(HashMap::new())),
            alert_rules: Self::create_default_alert_rules(),
            config,
        }
    }

    pub async fn add_alert(&self, alert: Alert) -> Result<()> {
        let alert_key = format!("{}:{}", alert.component, alert.title);
        
        // Check cooldown
        {
            let cooldowns = self.cooldown_tracker.read().unwrap();
            if let Some(&last_alert) = cooldowns.get(&alert_key) {
                let cooldown_duration = Duration::from_secs(self.config.alert_cooldown_seconds);
                if last_alert.elapsed().unwrap_or(Duration::MAX) < cooldown_duration {
                    return Ok(()); // Skip due to cooldown
                }
            }
        }

        // Add alert
        {
            let mut alerts = self.active_alerts.write().unwrap();
            alerts.insert(alert.id.clone(), alert);
        }

        // Update cooldown
        {
            let mut cooldowns = self.cooldown_tracker.write().unwrap();
            cooldowns.insert(alert_key, SystemTime::now());
        }

        Ok(())
    }

    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let alerts = self.active_alerts.read().unwrap();
        Ok(alerts.values()
            .filter(|alert| !alert.resolved)
            .cloned()
            .collect())
    }

    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        let mut alerts = self.active_alerts.write().unwrap();
        if let Some(alert) = alerts.get_mut(alert_id) {
            alert.acknowledged = true;
            alert.updated_at = SystemTime::now();
        }
        Ok(())
    }

    pub async fn process_alert_rules(&self) -> Result<()> {
        // Process alert rules against metrics
        // This is a simplified implementation
        info!("Processing {} alert rules", self.alert_rules.len());
        Ok(())
    }

    fn create_default_alert_rules() -> Vec<AlertRule> {
        vec![
            AlertRule {
                name: "high_cpu_usage".to_string(),
                metric_name: "system_cpu_usage".to_string(),
                condition: AlertCondition::GreaterThan,
                threshold: 80.0,
                duration_seconds: 300,
                level: AlertLevel::Warning,
                message_template: "CPU usage is above 80%".to_string(),
            },
            AlertRule {
                name: "high_memory_usage".to_string(),
                metric_name: "system_memory_usage".to_string(),
                condition: AlertCondition::GreaterThan,
                threshold: 90.0,
                duration_seconds: 300,
                level: AlertLevel::Error,
                message_template: "Memory usage is above 90%".to_string(),
            },
        ]
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            components: Arc::new(RwLock::new(HashMap::new())),
            checks: Vec::new(),
        }
    }

    pub async fn run_all_checks(&self) -> Result<Vec<HealthCheckResult>> {
        let mut results = Vec::new();
        
        for check in &self.checks {
            let start_time = Instant::now();
            let status = match tokio::time::timeout(check.timeout, async {
                (check.check_fn)()
            }).await {
                Ok(Ok(status)) => status,
                Ok(Err(_)) => HealthStatus::Error,
                Err(_) => HealthStatus::Error, // Timeout
            };
            
            let response_time = start_time.elapsed().as_millis() as u64;
            
            let result = HealthCheckResult {
                component: check.component.clone(),
                status,
                message: match status {
                    HealthStatus::Healthy => "OK".to_string(),
                    HealthStatus::Warning => "Warning detected".to_string(),
                    HealthStatus::Error => "Error detected".to_string(),
                    HealthStatus::Unknown => "Unknown status".to_string(),
                },
                last_check: SystemTime::now(),
                response_time_ms: response_time,
            };
            
            results.push(result.clone());
            
            // Update component status
            {
                let mut components = self.components.write().unwrap();
                components.insert(check.component.clone(), result);
            }
        }
        
        Ok(results)
    }

    pub async fn get_overall_health(&self) -> Result<SystemHealth> {
        let components = self.components.read().unwrap();
        
        if components.is_empty() {
            return Ok(SystemHealth::Unknown);
        }

        let mut critical_count = 0;
        let mut error_count = 0;
        let mut warning_count = 0;
        
        for result in components.values() {
            match result.status {
                HealthStatus::Error => error_count += 1,
                HealthStatus::Warning => warning_count += 1,
                HealthStatus::Unknown => error_count += 1,
                HealthStatus::Healthy => {},
            }
        }

        let total = components.len();
        
        if error_count > total / 2 {
            Ok(SystemHealth::Critical)
        } else if error_count > 0 || warning_count > total / 2 {
            Ok(SystemHealth::Unhealthy)
        } else if warning_count > 0 {
            Ok(SystemHealth::Degraded)
        } else {
            Ok(SystemHealth::Healthy)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_recording() {
        let config = MonitoringConfig::default();
        let monitor = EnhancedMonitoringSystem::new(config).unwrap();
        
        monitor.record_metric("test_metric", 42.0, None).await.unwrap();
        
        let history = monitor.get_metric_history("test_metric", None, None).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].value, 42.0);
    }

    #[tokio::test]
    async fn test_alert_creation() {
        let config = MonitoringConfig::default();
        let monitor = EnhancedMonitoringSystem::new(config).unwrap();
        
        let alert_id = monitor.create_alert(
            AlertLevel::Warning,
            "Test Alert",
            "This is a test alert",
            "test_component",
            None,
        ).await.unwrap();
        
        let alerts = monitor.get_active_alerts().await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].id, alert_id);
    }

    #[test]
    fn test_alert_manager() {
        let config = MonitoringConfig::default();
        let alert_manager = AlertManager::new(config);
        
        assert!(!alert_manager.alert_rules.is_empty());
    }
}