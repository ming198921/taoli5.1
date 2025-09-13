//! API兼容性测试工具
//! 使用统一的common_types定义，确保与前端完全兼容

use serde_json;
use anyhow::Result;
use common_types::{ArbitrageOpportunity, OpportunityStatus, ApiResponse};

fn main() -> Result<()> {
    println!("🧪 开始API兼容性测试...");
    
    test_arbitrage_opportunity_serialization()?;
    test_api_response_serialization()?;
    
    println!("✅ 所有API兼容性测试通过!");
    Ok(())
}

fn test_arbitrage_opportunity_serialization() -> Result<()> {
    println!("📊 测试 ArbitrageOpportunity 序列化...");
    
    let opportunity = ArbitrageOpportunity::new(
        "opp_123".to_string(),
        "BTC/USDT".to_string(),
        "binance".to_string(),
        "okx".to_string(),
        50000.0,
        50100.0,
        1000.0,
        60000, // 60秒TTL
    );
    
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
    
    // 验证状态枚举序列化
    assert_eq!(json_obj["status"], "active");
    
    println!("✅ ArbitrageOpportunity 兼容性测试通过");
    Ok(())
}

fn test_api_response_serialization() -> Result<()> {
    println!("📡 测试 ApiResponse 序列化...");
    
    // 成功响应测试
    let success_response = ApiResponse::success("test data".to_string());
    
    let json = serde_json::to_string_pretty(&success_response)?;
    println!("成功响应: {}", json);
    let _: ApiResponse<String> = serde_json::from_str(&json)?;
    
    // 错误响应测试
    let error_response: ApiResponse<String> = ApiResponse::error("Test error".to_string());
    
    let json = serde_json::to_string_pretty(&error_response)?;
    println!("错误响应: {}", json);
    let _: ApiResponse<String> = serde_json::from_str(&json)?;
    
    println!("✅ ApiResponse 兼容性测试通过");
    Ok(())
}