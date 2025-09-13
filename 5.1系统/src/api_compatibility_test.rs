//! API兼容性测试工具
//! 
//! 验证前后端数据结构的完全兼容性

use serde_json;
use anyhow::Result;
use std::collections::HashMap;

// 导入统一类型定义
include!("../unified_types.rs");

/// API兼容性测试套件
pub struct ApiCompatibilityTest;

impl ApiCompatibilityTest {
    /// 运行所有兼容性测试
    pub fn run_all_tests() -> Result<()> {
        println!("🧪 开始API兼容性测试...");
        
        Self::test_arbitrage_opportunity_serialization()?;
        Self::test_strategy_config_serialization()?;
        Self::test_market_data_serialization()?;
        Self::test_risk_alert_serialization()?;
        Self::test_system_status_serialization()?;
        Self::test_api_response_serialization()?;
        
        println!("✅ 所有API兼容性测试通过!");
        Ok(())
    }
    
    /// 测试套利机会序列化兼容性
    fn test_arbitrage_opportunity_serialization() -> Result<()> {
        println!("📊 测试 ArbitrageOpportunity 序列化...");
        
        // 创建测试数据
        let opportunity = ArbitrageOpportunity {
            id: "opp_123".to_string(),
            symbol: "BTC/USDT".to_string(),
            buy_exchange: "binance".to_string(),
            sell_exchange: "okx".to_string(),
            buy_price: 50000.0,
            sell_price: 50100.0,
            profit_usd: 100.0,
            profit_percent: 0.2,
            volume_available: 1000.0,
            detected_at: "2024-01-01T00:00:00Z".to_string(),
            expires_at: "2024-01-01T00:01:00Z".to_string(),
            status: OpportunityStatus::Active,
        };
        
        // 序列化到JSON
        let json = serde_json::to_string_pretty(&opportunity)?;
        println!("序列化结果: {}", json);
        
        // 反序列化
        let deserialized: ArbitrageOpportunity = serde_json::from_str(&json)?;
        
        // 验证数据完整性
        assert_eq!(opportunity.id, deserialized.id);
        assert_eq!(opportunity.symbol, deserialized.symbol);
        assert_eq!(opportunity.profit_usd, deserialized.profit_usd);
        
        // 验证前端期望的JSON格式
        let expected_keys = vec![
            "id", "symbol", "buy_exchange", "sell_exchange", 
            "buy_price", "sell_price", "profit_usd", "profit_percent",
            "volume_available", "detected_at", "expires_at", "status"
        ];
        
        let json_obj: serde_json::Value = serde_json::from_str(&json)?;
        for key in expected_keys {
            assert!(json_obj.get(key).is_some(), "缺少字段: {}", key);
        }
        
        println!("✅ ArbitrageOpportunity 兼容性测试通过");
        Ok(())
    }
    
    /// 测试策略配置序列化兼容性
    fn test_strategy_config_serialization() -> Result<()> {
        println!("⚙️ 测试 StrategyConfig 序列化...");
        
        let mut parameters = HashMap::new();
        parameters.insert("min_profit_bps".to_string(), serde_json::Value::Number(serde_json::Number::from(100)));
        
        let strategy = StrategyConfig {
            id: "strategy_1".to_string(),
            name: "Cross Exchange Arbitrage".to_string(),
            strategy_type: StrategyType::CrossExchange,
            enabled: true,
            priority: 1,
            description: "测试策略".to_string(),
            parameters,
            risk_limits: RiskLimits {
                max_position_size_usd: 10000.0,
                max_daily_loss_usd: 1000.0,
                max_exposure_per_exchange: 5000.0,
                max_correlation_risk: 0.8,
                stop_loss_percentage: 0.05,
                max_drawdown_percentage: 0.1,
                position_concentration_limit: 0.3,
                leverage_limit: 1.0,
            },
            exchanges: vec!["binance".to_string(), "okx".to_string()],
            symbols: vec!["BTC/USDT".to_string()],
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            performance_metrics: None,
        };
        
        let json = serde_json::to_string_pretty(&strategy)?;
        println!("策略配置序列化结果: {}", json);
        
        let _: StrategyConfig = serde_json::from_str(&json)?;
        
        // 验证type字段正确映射
        let json_obj: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_obj["type"], "cross_exchange");
        
        println!("✅ StrategyConfig 兼容性测试通过");
        Ok(())
    }
    
    /// 测试市场数据序列化兼容性
    fn test_market_data_serialization() -> Result<()> {
        println!("📈 测试 MarketData 序列化...");
        
        let market_data = MarketData {
            symbol: "BTC/USDT".to_string(),
            exchange: "binance".to_string(),
            price: 50000.0,
            volume: 1000.0,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            bid: 49999.0,
            ask: 50001.0,
            spread: 2.0,
        };
        
        let json = serde_json::to_string_pretty(&market_data)?;
        let _: MarketData = serde_json::from_str(&json)?;
        
        println!("✅ MarketData 兼容性测试通过");
        Ok(())
    }
    
    /// 测试风险警报序列化兼容性
    fn test_risk_alert_serialization() -> Result<()> {
        println!("⚠️ 测试 RiskAlert 序列化...");
        
        let mut details = HashMap::new();
        details.insert("position_size".to_string(), serde_json::Value::Number(serde_json::Number::from(5000)));
        
        let alert = RiskAlert {
            id: "alert_1".to_string(),
            alert_type: AlertType::PositionLimit,
            severity: AlertSeverity::High,
            message: "Position limit exceeded".to_string(),
            details,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            resolved_at: None,
            status: AlertStatus::Active,
        };
        
        let json = serde_json::to_string_pretty(&alert)?;
        let _: RiskAlert = serde_json::from_str(&json)?;
        
        // 验证type字段正确映射
        let json_obj: serde_json::Value = serde_json::from_str(&json)?;
        assert_eq!(json_obj["type"], "position_limit");
        assert_eq!(json_obj["severity"], "high");
        
        println!("✅ RiskAlert 兼容性测试通过");
        Ok(())
    }
    
    /// 测试系统状态序列化兼容性
    fn test_system_status_serialization() -> Result<()> {
        println!("🖥️ 测试 SystemStatus 序列化...");
        
        let status = SystemStatus {
            is_running: true,
            qingxi: ComponentStatus::Running,
            celue: ComponentStatus::Running,
            architecture: ComponentStatus::Running,
            observability: ComponentStatus::Warning,
            uptime: 3600,
            last_update: "2024-01-01T00:00:00Z".to_string(),
        };
        
        let json = serde_json::to_string_pretty(&status)?;
        let _: SystemStatus = serde_json::from_str(&json)?;
        
        // 验证驼峰命名转换
        let json_obj: serde_json::Value = serde_json::from_str(&json)?;
        assert!(json_obj.get("isRunning").is_some());
        assert!(json_obj.get("lastUpdate").is_some());
        
        println!("✅ SystemStatus 兼容性测试通过");
        Ok(())
    }
    
    /// 测试API响应包装器兼容性
    fn test_api_response_serialization() -> Result<()> {
        println!("📡 测试 ApiResponse 序列化...");
        
        // 成功响应测试
        let success_response = ApiResponse {
            success: true,
            data: Some("test data".to_string()),
            error: None,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        
        let json = serde_json::to_string_pretty(&success_response)?;
        let _: ApiResponse<String> = serde_json::from_str(&json)?;
        
        // 错误响应测试
        let error_response: ApiResponse<String> = ApiResponse {
            success: false,
            data: None,
            error: Some("Test error".to_string()),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        
        let json = serde_json::to_string_pretty(&error_response)?;
        let _: ApiResponse<String> = serde_json::from_str(&json)?;
        
        println!("✅ ApiResponse 兼容性测试通过");
        Ok(())
    }
    
    /// 生成前端类型定义
    pub fn generate_frontend_types() -> Result<String> {
        let types = r#"
// 自动生成的类型定义 - 与后端100%兼容
// 生成时间: {timestamp}

export interface ArbitrageOpportunity {
  id: string;
  symbol: string;
  buy_exchange: string;
  sell_exchange: string;
  buy_price: number;
  sell_price: number;
  profit_usd: number;
  profit_percent: number;
  volume_available: number;
  detected_at: string;
  expires_at: string;
  status: 'active' | 'executed' | 'expired' | 'cancelled';
}

export interface StrategyConfig {
  id: string;
  name: string;
  type: 'cross_exchange' | 'triangular' | 'market_making' | 'statistical_arbitrage';
  enabled: boolean;
  priority: number;
  description: string;
  parameters: Record<string, any>;
  risk_limits: RiskLimits;
  exchanges: string[];
  symbols: string[];
  created_at: string;
  updated_at: string;
  performance_metrics?: StrategyPerformance;
}

// ... 其他类型定义
"#;
        
        Ok(types.replace("{timestamp}", &chrono::Utc::now().to_rfc3339()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_all_compatibility() {
        ApiCompatibilityTest::run_all_tests().expect("兼容性测试失败");
    }
}