use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::config::{AppConfig, ServiceConfig};

pub mod router;
pub mod load_balancer;
pub mod health_check;

pub use router::ServiceRouter;
pub use load_balancer::LoadBalancer;
pub use health_check::HealthChecker;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub weight: u32,
    pub healthy: bool,
    pub protocol: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone)]
pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    routes: Arc<RwLock<HashMap<String, String>>>,
    health_checker: Arc<HealthChecker>,
}

impl ServiceRegistry {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        let mut services = HashMap::new();
        let mut routes = HashMap::new();

        // 初始化服务实例
        for service_config in &config.services {
            let instance = ServiceInstance {
                name: service_config.name.clone(),
                host: service_config.host.clone(),
                port: service_config.port,
                weight: service_config.weight,
                healthy: true,
                protocol: service_config.protocol.clone(),
                last_check: chrono::Utc::now(),
            };

            services.entry(service_config.name.clone())
                .or_insert_with(Vec::new)
                .push(instance);

            // 配置路由规则
            Self::configure_routes(&service_config.name, &mut routes);
        }

        let registry = Self {
            services: Arc::new(RwLock::new(services)),
            routes: Arc::new(RwLock::new(routes)),
            health_checker: Arc::new(HealthChecker::new(config.load_balancer.clone())),
        };

        // 启动健康检查
        registry.start_health_checks().await;

        Ok(registry)
    }

    fn configure_routes(service_name: &str, routes: &mut HashMap<String, String>) {
        match service_name {
            "main-api" => {
                routes.insert("/api/auth".to_string(), service_name.to_string());
                routes.insert("/api/system".to_string(), service_name.to_string());
                routes.insert("/api/celue".to_string(), service_name.to_string());
                routes.insert("/api/risk".to_string(), service_name.to_string());
            },
            "qingxi-data" => {
                routes.insert("/api/qingxi".to_string(), service_name.to_string());
                routes.insert("/api/data".to_string(), service_name.to_string());
            },
            "logging-service" => {
                routes.insert("/api/logs".to_string(), service_name.to_string());
                routes.insert("/ws/logs".to_string(), service_name.to_string());
            },
            "cleaning-service" => {
                routes.insert("/api/cleaning".to_string(), service_name.to_string());
            },
            "strategy-service" => {
                routes.insert("/api/strategies".to_string(), service_name.to_string());
            },
            "performance-service" => {
                routes.insert("/api/performance".to_string(), service_name.to_string());
            },
            "trading-service" => {
                routes.insert("/api/trading".to_string(), service_name.to_string());
                routes.insert("/api/orders".to_string(), service_name.to_string());
                routes.insert("/api/positions".to_string(), service_name.to_string());
                routes.insert("/api/funds".to_string(), service_name.to_string());
            },
            "ai-model-service" => {
                routes.insert("/api/ml".to_string(), service_name.to_string());
                routes.insert("/api/models".to_string(), service_name.to_string());
            },
            "config-service" => {
                routes.insert("/api/config".to_string(), service_name.to_string());
            },
            _ => {}
        }
    }

    pub fn resolve_service(&self, path: &str) -> Option<String> {
        let routes = self.routes.blocking_read();
        
        // 查找最长匹配的路由
        let mut best_match = None;
        let mut best_match_len = 0;

        for (route_prefix, service_name) in routes.iter() {
            if path.starts_with(route_prefix) && route_prefix.len() > best_match_len {
                best_match = Some(service_name.clone());
                best_match_len = route_prefix.len();
            }
        }

        best_match
    }

    pub async fn get_healthy_instances(&self, service_name: &str) -> Vec<ServiceInstance> {
        let services = self.services.read().await;
        
        services
            .get(service_name)
            .map(|instances| {
                instances
                    .iter()
                    .filter(|i| i.healthy)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn mark_instance_unhealthy(&self, service_name: &str, host: &str, port: u16) {
        let mut services = self.services.write().await;
        
        if let Some(instances) = services.get_mut(service_name) {
            for instance in instances {
                if instance.host == host && instance.port == port {
                    instance.healthy = false;
                    warn!("Marked instance {}:{}:{} as unhealthy", 
                          service_name, host, port);
                }
            }
        }
    }

    pub async fn mark_instance_healthy(&self, service_name: &str, host: &str, port: u16) {
        let mut services = self.services.write().await;
        
        if let Some(instances) = services.get_mut(service_name) {
            for instance in instances {
                if instance.host == host && instance.port == port {
                    instance.healthy = true;
                    info!("Marked instance {}:{}:{} as healthy", 
                          service_name, host, port);
                }
            }
        }
    }

    pub async fn check_all_health(&self) -> HashMap<String, bool> {
        let services = self.services.read().await;
        let mut health_status = HashMap::new();

        for (name, instances) in services.iter() {
            let healthy = instances.iter().any(|i| i.healthy);
            health_status.insert(name.clone(), healthy);
        }

        health_status
    }

    pub async fn list_all_services(&self) -> Vec<ServiceInstance> {
        let services = self.services.read().await;
        let mut all_instances = Vec::new();

        for instances in services.values() {
            all_instances.extend(instances.clone());
        }

        all_instances
    }

    pub fn service_count(&self) -> usize {
        self.services.blocking_read().len()
    }

    async fn start_health_checks(&self) {
        let registry = self.clone();
        let health_checker = self.health_checker.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                
                let services = registry.services.read().await;
                for (service_name, instances) in services.iter() {
                    for instance in instances {
                        let is_healthy = health_checker
                            .check_health(instance)
                            .await
                            .unwrap_or(false);
                        
                        if is_healthy != instance.healthy {
                            drop(services);
                            if is_healthy {
                                registry.mark_instance_healthy(
                                    service_name, 
                                    &instance.host, 
                                    instance.port
                                ).await;
                            } else {
                                registry.mark_instance_unhealthy(
                                    service_name, 
                                    &instance.host, 
                                    instance.port
                                ).await;
                            }
                            break;
                        }
                    }
                }
            }
        });
    }
}