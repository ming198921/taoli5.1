#![allow(dead_code)]
// src/reasoner_client/mod.rs
//! # Reasoner Client Module
//!
//! 提供与推理服务通信的客户端

use crate::settings::Settings;
use crate::types::{AnomalyDetectionResult, MarketDataSnapshot};
use reqwest::Client;
use tracing::{error, info};

pub struct ReasonerClient {
    client: Client,
    endpoint: String,
}

impl ReasonerClient {
    pub fn new(settings: &Settings) -> Self {
        Self {
            client: Client::new(),
            endpoint: settings.reasoner.api_endpoint.clone(),
        }
    }

    /// 分析异常检测结果
    pub async fn analyze_anomaly(
        &self,
        anomaly: &AnomalyDetectionResult,
    ) -> Result<AnomalyDetectionResult, String> {
        info!(
            "🔍 Analyzing anomaly: {:?} for {}",
            anomaly.anomaly_type,
            anomaly.symbol.as_pair()
        );

        // 检查endpoint是否配置
        if self.endpoint.is_empty() {
            info!("📝 Reasoner endpoint not configured, skipping analysis");
            return Ok(anomaly.clone());
        }

        // 将AnomalyDetectionResult直接序列化为JSON
        let json_payload = match serde_json::to_string(anomaly) {
            Ok(json) => json,
            Err(e) => {
                let error_msg = format!("Failed to serialize anomaly to JSON: {e}");
                error!("❌ {}", error_msg);
                return Err(error_msg);
            }
        };

        // 构建URL
        let url = if self.endpoint.ends_with('/') {
            format!("{}analyze", self.endpoint)
        } else {
            format!("{}/analyze", self.endpoint)
        };

        // 发送异步HTTP POST请求
        match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_payload)
            .timeout(std::time::Duration::from_secs(5)) // 5秒超时
            .send()
            .await
        {
            Ok(resp) => {
                if resp.status().is_success() {
                    // 尝试解析响应
                    match resp.text().await {
                        Ok(response_text) => {
                            info!(
                                "✅ Reasoner analysis completed successfully: {}",
                                response_text
                            );
                            // 简化实现：返回增强的异常结果
                            let mut enhanced_anomaly = anomaly.clone();
                            enhanced_anomaly.details = format!(
                                "{} | Reasoner analysis: {}",
                                enhanced_anomaly.details, response_text
                            );
                            Ok(enhanced_anomaly)
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to read reasoner response: {e}");
                            error!("❌ {}", error_msg);
                            Err(error_msg)
                        }
                    }
                } else {
                    let error_msg = format!("Reasoner service error: HTTP {}", resp.status());
                    error!("❌ {}", error_msg);
                    Err(error_msg)
                }
            }
            Err(e) => {
                let error_msg = if e.is_timeout() {
                    "Reasoner service request timeout (5s)".to_string()
                } else if e.is_connect() {
                    format!("Failed to connect to reasoner service: {e}")
                } else {
                    format!("Reasoner service request failed: {e}")
                };
                error!("❌ {}", error_msg);
                Err(error_msg)
            }
        }
    }

    /// 向推理服务批量发送快照，返回推理结果
    pub async fn infer(
        &self,
        snapshots: &[MarketDataSnapshot],
    ) -> Result<Vec<AnomalyDetectionResult>, String> {
        let url = format!("{}/infer", self.endpoint);
        let resp = self
            .client
            .post(&url)
            .json(&snapshots)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            let results = resp
                .json::<Vec<AnomalyDetectionResult>>()
                .await
                .map_err(|e| e.to_string())?;
            Ok(results)
        } else {
            Err(format!("Reasoner error: {}", resp.status()))
        }
    }
}
