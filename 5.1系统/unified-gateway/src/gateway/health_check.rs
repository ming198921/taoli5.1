use std::time::Duration;
use anyhow::Result;
use tracing::{debug, warn};

use super::ServiceInstance;
use crate::config::LoadBalancerConfig;

pub struct HealthChecker {
    config: LoadBalancerConfig,
    http_client: reqwest::Client,
}

impl HealthChecker {
    pub fn new(config: LoadBalancerConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();

        Self {
            config,
            http_client,
        }
    }

    pub async fn check_health(&self, instance: &ServiceInstance) -> Result<bool> {
        match instance.protocol.as_str() {
            "http" => self.check_http_health(instance).await,
            "grpc" => self.check_grpc_health(instance).await,
            _ => Ok(true),
        }
    }

    async fn check_http_health(&self, instance: &ServiceInstance) -> Result<bool> {
        let url = format!("http://{}:{}/health", instance.host, instance.port);
        
        debug!("Checking health for {}", url);
        
        match self.http_client.get(&url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if !is_healthy {
                    warn!("Health check failed for {} with status {}", 
                          url, response.status());
                }
                Ok(is_healthy)
            },
            Err(e) => {
                warn!("Health check failed for {}: {}", url, e);
                Ok(false)
            }
        }
    }

    async fn check_grpc_health(&self, instance: &ServiceInstance) -> Result<bool> {
        // 简化的gRPC健康检查
        // 实际实现需要使用tonic进行gRPC健康检查
        debug!("Checking gRPC health for {}:{}", instance.host, instance.port);
        Ok(true)
    }
}