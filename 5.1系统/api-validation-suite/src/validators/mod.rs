use serde_json::Value;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// 验证器模块 - 用于验证API响应的完整性和正确性

/// API响应验证器
pub struct ResponseValidator;

impl ResponseValidator {
    /// 验证JSON响应格式
    pub fn validate_json_format(response_text: &str) -> ValidationResult {
        match serde_json::from_str::<Value>(response_text) {
            Ok(json) => ValidationResult::success("JSON格式有效"),
            Err(e) => ValidationResult::failure(&format!("JSON格式无效: {}", e)),
        }
    }

    /// 验证标准响应结构
    pub fn validate_standard_response_structure(json: &Value) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 0.0;

        // 检查必需字段
        if json.get("success").is_some() {
            score += 30.0;
        } else {
            issues.push("缺少success字段".to_string());
        }

        if json.get("data").is_some() || json.get("error").is_some() {
            score += 25.0;
        } else {
            issues.push("缺少data或error字段".to_string());
        }

        if let Some(metadata) = json.get("metadata") {
            score += 20.0;
            
            // 检查metadata子字段
            if metadata.get("timestamp").is_some() {
                score += 10.0;
            } else {
                issues.push("metadata缺少timestamp字段".to_string());
            }

            if metadata.get("request_id").is_some() {
                score += 10.0;
            }

            if metadata.get("execution_time_ms").is_some() {
                score += 5.0;
            }
        } else {
            issues.push("缺少metadata字段".to_string());
        }

        if score >= 70.0 {
            ValidationResult::success_with_score("响应结构验证通过", score)
        } else {
            ValidationResult::failure_with_issues("响应结构验证失败", issues, score)
        }
    }

    /// 验证数据类型一致性
    pub fn validate_data_types(json: &Value, expected_types: &HashMap<String, DataType>) -> ValidationResult {
        let mut issues = Vec::new();
        let mut valid_count = 0;
        let total_count = expected_types.len();

        for (field_name, expected_type) in expected_types {
            if let Some(value) = json.get(field_name) {
                if Self::check_data_type(value, expected_type) {
                    valid_count += 1;
                } else {
                    issues.push(format!("字段{}类型不匹配，期望{:?}", field_name, expected_type));
                }
            } else {
                issues.push(format!("缺少字段{}", field_name));
            }
        }

        let score = if total_count > 0 {
            (valid_count as f64 / total_count as f64) * 100.0
        } else {
            100.0
        };

        if issues.is_empty() {
            ValidationResult::success_with_score("数据类型验证通过", score)
        } else {
            ValidationResult::failure_with_issues("数据类型验证失败", issues, score)
        }
    }

    /// 检查单个值的数据类型
    fn check_data_type(value: &Value, expected_type: &DataType) -> bool {
        match expected_type {
            DataType::String => value.is_string(),
            DataType::Number => value.is_number(),
            DataType::Boolean => value.is_boolean(),
            DataType::Array => value.is_array(),
            DataType::Object => value.is_object(),
            DataType::Null => value.is_null(),
        }
    }

    /// 验证时间戳格式
    pub fn validate_timestamp_format(timestamp_str: &str) -> ValidationResult {
        // 尝试解析ISO 8601格式
        if DateTime::parse_from_rfc3339(timestamp_str).is_ok() {
            ValidationResult::success("时间戳格式有效 (RFC3339)")
        } else if chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S").is_ok() {
            ValidationResult::success("时间戳格式有效 (SQL格式)")
        } else if timestamp_str.parse::<i64>().is_ok() {
            ValidationResult::success("时间戳格式有效 (Unix时间戳)")
        } else {
            ValidationResult::failure("时间戳格式无效")
        }
    }

    /// 验证数值范围
    pub fn validate_number_range(value: f64, min: Option<f64>, max: Option<f64>) -> ValidationResult {
        let mut issues = Vec::new();

        if let Some(min_val) = min {
            if value < min_val {
                issues.push(format!("数值{}小于最小值{}", value, min_val));
            }
        }

        if let Some(max_val) = max {
            if value > max_val {
                issues.push(format!("数值{}大于最大值{}", value, max_val));
            }
        }

        if issues.is_empty() {
            ValidationResult::success("数值范围验证通过")
        } else {
            ValidationResult::failure_with_issues("数值范围验证失败", issues, 0.0)
        }
    }
}

/// 数据类型枚举
#[derive(Debug, Clone)]
pub enum DataType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub message: String,
    pub score: f64,
    pub issues: Vec<String>,
}

impl ValidationResult {
    /// 创建成功的验证结果
    pub fn success(message: &str) -> Self {
        Self {
            is_valid: true,
            message: message.to_string(),
            score: 100.0,
            issues: Vec::new(),
        }
    }

    /// 创建带分数的成功验证结果
    pub fn success_with_score(message: &str, score: f64) -> Self {
        Self {
            is_valid: score >= 70.0,
            message: message.to_string(),
            score,
            issues: Vec::new(),
        }
    }

    /// 创建失败的验证结果
    pub fn failure(message: &str) -> Self {
        Self {
            is_valid: false,
            message: message.to_string(),
            score: 0.0,
            issues: Vec::new(),
        }
    }

    /// 创建带问题列表的失败验证结果
    pub fn failure_with_issues(message: &str, issues: Vec<String>, score: f64) -> Self {
        Self {
            is_valid: false,
            message: message.to_string(),
            score,
            issues,
        }
    }
}

/// 性能验证器
pub struct PerformanceValidator;

impl PerformanceValidator {
    /// 验证响应时间是否在可接受范围内
    pub fn validate_response_time(response_time_ms: u64, threshold_ms: u64) -> ValidationResult {
        if response_time_ms <= threshold_ms {
            ValidationResult::success(&format!("响应时间{}ms符合要求", response_time_ms))
        } else {
            ValidationResult::failure(&format!("响应时间{}ms超过阈值{}ms", response_time_ms, threshold_ms))
        }
    }

    /// 验证吞吐量
    pub fn validate_throughput(requests_per_second: f64, min_threshold: f64) -> ValidationResult {
        if requests_per_second >= min_threshold {
            ValidationResult::success(&format!("吞吐量{:.1}req/s达标", requests_per_second))
        } else {
            ValidationResult::failure(&format!("吞吐量{:.1}req/s低于要求{:.1}req/s", requests_per_second, min_threshold))
        }
    }

    /// 验证错误率
    pub fn validate_error_rate(error_rate: f64, max_threshold: f64) -> ValidationResult {
        if error_rate <= max_threshold {
            ValidationResult::success(&format!("错误率{:.2}%在可接受范围内", error_rate))
        } else {
            ValidationResult::failure(&format!("错误率{:.2}%超过阈值{:.2}%", error_rate, max_threshold))
        }
    }
}

/// 业务逻辑验证器
pub struct BusinessValidator;

impl BusinessValidator {
    /// 验证套利机会数据
    pub fn validate_arbitrage_opportunity(opportunity: &Value) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 0.0;

        // 检查必需字段
        if opportunity.get("profit_percent").and_then(|v| v.as_f64()).is_some() {
            score += 25.0;
        } else {
            issues.push("缺少profit_percent字段".to_string());
        }

        if opportunity.get("volume").and_then(|v| v.as_f64()).is_some() {
            score += 25.0;
        } else {
            issues.push("缺少volume字段".to_string());
        }

        if opportunity.get("exchanges").and_then(|v| v.as_array()).is_some() {
            score += 25.0;
        } else {
            issues.push("缺少exchanges字段".to_string());
        }

        if opportunity.get("symbol").and_then(|v| v.as_str()).is_some() {
            score += 25.0;
        } else {
            issues.push("缺少symbol字段".to_string());
        }

        // 验证利润率合理性
        if let Some(profit) = opportunity.get("profit_percent").and_then(|v| v.as_f64()) {
            if profit > 0.0 && profit < 100.0 {
                // 利润率合理
            } else {
                issues.push("profit_percent数值不合理".to_string());
                score -= 10.0;
            }
        }

        if score >= 70.0 && issues.is_empty() {
            ValidationResult::success_with_score("套利机会数据验证通过", score)
        } else {
            ValidationResult::failure_with_issues("套利机会数据验证失败", issues, score)
        }
    }

    /// 验证订单数据
    pub fn validate_order_data(order: &Value) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 0.0;

        // 验证必需字段
        let required_fields = vec!["symbol", "side", "amount", "price", "order_type"];
        
        for field in &required_fields {
            if order.get(field).is_some() {
                score += 20.0;
            } else {
                issues.push(format!("缺少{}字段", field));
            }
        }

        // 验证side字段值
        if let Some(side) = order.get("side").and_then(|v| v.as_str()) {
            if side == "buy" || side == "sell" {
                // 有效值
            } else {
                issues.push("side字段值无效，应为buy或sell".to_string());
                score -= 10.0;
            }
        }

        // 验证数量和价格为正数
        if let Some(amount) = order.get("amount").and_then(|v| v.as_f64()) {
            if amount <= 0.0 {
                issues.push("amount应为正数".to_string());
                score -= 15.0;
            }
        }

        if let Some(price) = order.get("price").and_then(|v| v.as_f64()) {
            if price <= 0.0 {
                issues.push("price应为正数".to_string());
                score -= 15.0;
            }
        }

        if score >= 70.0 && issues.len() <= 1 {
            ValidationResult::success_with_score("订单数据验证通过", score)
        } else {
            ValidationResult::failure_with_issues("订单数据验证失败", issues, score)
        }
    }

    /// 验证策略配置
    pub fn validate_strategy_config(config: &Value) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 0.0;

        // 检查基本字段
        if config.get("name").and_then(|v| v.as_str()).is_some() {
            score += 20.0;
        } else {
            issues.push("缺少name字段".to_string());
        }

        if config.get("type").and_then(|v| v.as_str()).is_some() {
            score += 20.0;
        } else {
            issues.push("缺少type字段".to_string());
        }

        // 检查参数配置
        if let Some(params) = config.get("parameters") {
            if params.is_object() {
                score += 30.0;
                
                // 检查关键参数
                if params.get("min_profit_threshold").is_some() {
                    score += 10.0;
                }
                if params.get("max_position_size").is_some() {
                    score += 10.0;
                }
                if params.get("timeout_seconds").is_some() {
                    score += 10.0;
                }
            } else {
                issues.push("parameters应为对象类型".to_string());
            }
        } else {
            issues.push("缺少parameters字段".to_string());
        }

        if score >= 70.0 {
            ValidationResult::success_with_score("策略配置验证通过", score)
        } else {
            ValidationResult::failure_with_issues("策略配置验证失败", issues, score)
        }
    }
}

/// 安全验证器
pub struct SecurityValidator;

impl SecurityValidator {
    /// 验证API响应中是否包含敏感信息
    pub fn validate_no_sensitive_data(response_text: &str) -> ValidationResult {
        let sensitive_patterns = vec![
            "password",
            "secret",
            "token",
            "key",
            "private",
            "credential",
        ];

        let mut found_sensitive = Vec::new();
        let response_lower = response_text.to_lowercase();

        for pattern in &sensitive_patterns {
            if response_lower.contains(pattern) {
                found_sensitive.push(pattern.to_string());
            }
        }

        if found_sensitive.is_empty() {
            ValidationResult::success("未发现敏感信息")
        } else {
            ValidationResult::failure(&format!("发现可能的敏感信息: {}", found_sensitive.join(", ")))
        }
    }

    /// 验证输入数据的安全性
    pub fn validate_input_security(input: &str) -> ValidationResult {
        let dangerous_patterns = vec![
            "<script",
            "javascript:",
            "onclick=",
            "onerror=",
            "'; DROP TABLE",
            "UNION SELECT",
            "../",
            "..\\",
        ];

        let mut security_issues = Vec::new();
        let input_lower = input.to_lowercase();

        for pattern in &dangerous_patterns {
            if input_lower.contains(&pattern.to_lowercase()) {
                security_issues.push(format!("发现潜在恶意模式: {}", pattern));
            }
        }

        if security_issues.is_empty() {
            ValidationResult::success("输入安全验证通过")
        } else {
            ValidationResult::failure_with_issues("输入存在安全风险", security_issues, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_json_format() {
        let valid_json = r#"{"success": true, "data": []}"#;
        let invalid_json = r#"{"success": true, "data": "#;

        assert!(ResponseValidator::validate_json_format(valid_json).is_valid);
        assert!(!ResponseValidator::validate_json_format(invalid_json).is_valid);
    }

    #[test]
    fn test_validate_standard_response_structure() {
        let valid_response = json!({
            "success": true,
            "data": {},
            "metadata": {
                "timestamp": "2023-01-01T00:00:00Z",
                "request_id": "123",
                "execution_time_ms": 50
            }
        });

        let result = ResponseValidator::validate_standard_response_structure(&valid_response);
        assert!(result.is_valid);
        assert!(result.score >= 70.0);
    }

    #[test]
    fn test_validate_arbitrage_opportunity() {
        let valid_opportunity = json!({
            "profit_percent": 1.5,
            "volume": 1000.0,
            "exchanges": ["binance", "okx"],
            "symbol": "BTC/USDT"
        });

        let result = BusinessValidator::validate_arbitrage_opportunity(&valid_opportunity);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_order_data() {
        let valid_order = json!({
            "symbol": "BTC/USDT",
            "side": "buy",
            "amount": 0.01,
            "price": 45000.0,
            "order_type": "limit"
        });

        let result = BusinessValidator::validate_order_data(&valid_order);
        assert!(result.is_valid);
    }

    #[test]
    fn test_validate_response_time() {
        assert!(PerformanceValidator::validate_response_time(500, 1000).is_valid);
        assert!(!PerformanceValidator::validate_response_time(1500, 1000).is_valid);
    }

    #[test]
    fn test_validate_no_sensitive_data() {
        let safe_response = r#"{"data": {"price": 45000, "volume": 100}}"#;
        let unsafe_response = r#"{"data": {"password": "secret123", "token": "abc"}}"#;

        assert!(SecurityValidator::validate_no_sensitive_data(safe_response).is_valid);
        assert!(!SecurityValidator::validate_no_sensitive_data(unsafe_response).is_valid);
    }
}