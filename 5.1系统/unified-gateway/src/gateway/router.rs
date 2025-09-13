use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    pub pattern: String,
    pub service: String,
    pub methods: Vec<String>,
    pub websocket: bool,
}

pub struct ServiceRouter {
    routes: Vec<RouteRule>,
}

impl ServiceRouter {
    pub fn new() -> Self {
        Self {
            routes: Self::default_routes(),
        }
    }

    fn default_routes() -> Vec<RouteRule> {
        vec![
            // 主API路由
            RouteRule {
                pattern: "/api/auth/*".to_string(),
                service: "main-api".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            RouteRule {
                pattern: "/api/system/*".to_string(),
                service: "main-api".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            // QingXi数据路由
            RouteRule {
                pattern: "/api/qingxi/*".to_string(),
                service: "qingxi-data".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            // 日志服务路由
            RouteRule {
                pattern: "/api/logs/*".to_string(),
                service: "logging-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            RouteRule {
                pattern: "/ws/logs/*".to_string(),
                service: "logging-service".to_string(),
                methods: vec!["GET".to_string()],
                websocket: true,
            },
            // 清洗服务路由
            RouteRule {
                pattern: "/api/cleaning/*".to_string(),
                service: "cleaning-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            // 策略服务路由
            RouteRule {
                pattern: "/api/strategies/*".to_string(),
                service: "strategy-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            // 性能服务路由
            RouteRule {
                pattern: "/api/performance/*".to_string(),
                service: "performance-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            // 交易服务路由
            RouteRule {
                pattern: "/api/trading/*".to_string(),
                service: "trading-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            RouteRule {
                pattern: "/api/orders/*".to_string(),
                service: "trading-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            // AI模型服务路由
            RouteRule {
                pattern: "/api/ml/*".to_string(),
                service: "ai-model-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
            // 配置服务路由
            RouteRule {
                pattern: "/api/config/*".to_string(),
                service: "config-service".to_string(),
                methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                websocket: false,
            },
        ]
    }

    pub fn match_route(&self, path: &str, method: &str) -> Option<String> {
        for route in &self.routes {
            if self.matches_pattern(&route.pattern, path) && 
               route.methods.contains(&method.to_string()) {
                return Some(route.service.clone());
            }
        }
        None
    }

    fn matches_pattern(&self, pattern: &str, path: &str) -> bool {
        if pattern.ends_with("*") {
            let prefix = &pattern[..pattern.len() - 1];
            path.starts_with(prefix)
        } else {
            pattern == path
        }
    }
}