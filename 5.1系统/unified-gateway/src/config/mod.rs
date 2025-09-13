use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub rate_limit: RateLimitConfig,
    pub load_balancer: LoadBalancerConfig,
    pub services: Vec<ServiceConfig>,
    pub monitoring: MonitoringConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub token_expiry: u64,
    pub refresh_expiry: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoadBalancerConfig {
    pub algorithm: String, // round_robin, least_connections, weighted
    pub health_check_interval: u64,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub weight: u32,
    pub health_check_path: String,
    pub protocol: String, // http, grpc, websocket
    pub timeout: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    pub redis_url: String,
    pub default_ttl: u64,
    pub enabled: bool,
}

impl AppConfig {
    pub async fn load() -> Result<Self> {
        // 加载环境变量
        dotenv::dotenv().ok();

        // 默认配置
        let default_config = Self::default();
        
        // 尝试从文件加载配置
        if let Ok(config_str) = std::fs::read_to_string("config/gateway.toml") {
            let file_config: AppConfig = toml::from_str(&config_str)?;
            Ok(Self::merge(default_config, file_config))
        } else {
            Ok(default_config)
        }
    }

    fn merge(default: Self, file: Self) -> Self {
        // 合并配置，文件配置优先
        file
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                workers: 4,
                max_connections: 10000,
            },
            auth: AuthConfig {
                jwt_secret: std::env::var("JWT_SECRET")
                    .unwrap_or_else(|_| "default-secret-change-me".to_string()),
                token_expiry: 3600,
                refresh_expiry: 86400,
                enabled: true,
            },
            rate_limit: RateLimitConfig {
                requests_per_minute: 1000,
                burst_size: 100,
                enabled: true,
            },
            load_balancer: LoadBalancerConfig {
                algorithm: "round_robin".to_string(),
                health_check_interval: 30,
                failure_threshold: 3,
                recovery_threshold: 2,
            },
            services: vec![
                // 现有服务器
                ServiceConfig {
                    name: "main-api".to_string(),
                    host: "localhost".to_string(),
                    port: 8080,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "qingxi-data".to_string(),
                    host: "localhost".to_string(),
                    port: 50061,
                    weight: 100,
                    health_check_path: "/api/v1/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "grpc-api".to_string(),
                    host: "localhost".to_string(),
                    port: 50051,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "grpc".to_string(),
                    timeout: 30,
                },
                // 新增服务器
                ServiceConfig {
                    name: "logging-service".to_string(),
                    host: "localhost".to_string(),
                    port: 4001,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "cleaning-service".to_string(),
                    host: "localhost".to_string(),
                    port: 4002,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "strategy-service".to_string(),
                    host: "localhost".to_string(),
                    port: 4003,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "performance-service".to_string(),
                    host: "localhost".to_string(),
                    port: 4004,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "trading-service".to_string(),
                    host: "localhost".to_string(),
                    port: 4005,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "ai-model-service".to_string(),
                    host: "localhost".to_string(),
                    port: 4006,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
                ServiceConfig {
                    name: "config-service".to_string(),
                    host: "localhost".to_string(),
                    port: 4007,
                    weight: 100,
                    health_check_path: "/health".to_string(),
                    protocol: "http".to_string(),
                    timeout: 30,
                },
            ],
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                tracing_enabled: true,
                log_level: "info".to_string(),
            },
            cache: CacheConfig {
                redis_url: "redis://localhost:6379".to_string(),
                default_ttl: 300,
                enabled: true,
            },
        }
    }
}