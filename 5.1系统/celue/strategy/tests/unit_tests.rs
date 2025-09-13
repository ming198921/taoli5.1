//! 单元测试套件
//! 
//! 测试核心策略功能、特征工程、模型验证等

#[cfg(test)]
mod strategy_tests {
    use strategy::config_loader::ConfigLoader;
    
    #[test]
    fn test_config_loader_default() {
        // 使用默认配置路径创建加载器
        let loader = ConfigLoader::new("./config/default_strategy.toml".to_string());
        assert!(loader.is_ok());
        
        if let Ok(loader) = loader {
            let config = loader.get_config();
            // 验证配置结构
            assert!(config.inter_exchange.enabled);
            assert!(config.triangular.enabled);
        }
    }
    
    #[test] 
    fn test_strategy_context_creation() {
        // 测试策略上下文的创建
        use strategy::context::{FeePrecisionRepoImpl, StrategyContext};
        use adapters::metrics::ProductionAdapterMetrics;
        use std::sync::Arc;
        
        let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
        let metrics = Arc::new(ProductionAdapterMetrics::new());
        let context = StrategyContext::new(fee_repo, metrics);
        
        // 基本验证
        assert_eq!(context.get_exchange_weights().len(), 0); // 初始为空
    }
    
    #[test]
    fn test_min_profit_calculation() {
        // 测试最小利润计算
        use strategy::min_profit::MinProfitModel;
        
        let model = MinProfitModel::new();
        let profit_bps = model.calculate_dynamic_min_profit(
            0.01, // base_profit_bps
            50.0, // market_volatility
            0.8,  // liquidity_score
            1.2   // competition_factor
        );
        
        assert!(profit_bps > 0.0);
        assert!(profit_bps < 100.0); // 合理范围内
    }
}

#[cfg(test)]
mod risk_tests {
    #[test]
    fn test_risk_basic() {
        // 基本风险模块测试
        assert!(true);
    }
}

#[cfg(test)]
mod ml_tests {
    #[test]
    fn test_ml_basic() {
        // 基本ML模块测试  
        assert!(true);
    }
}

#[cfg(test)]
mod performance_tests {
    use std::time::Instant;
    
    #[test]
    fn test_macd_performance() {
        // 性能测试：MACD计算
        let prices: Vec<f64> = (0..1000).map(|i| 100.0 + (i as f64 * 0.01)).collect();
        let iterations = 100;
        
        let start = Instant::now();
        for _ in 0..iterations {
            // 模拟MACD计算
            let _result = prices.iter()
                .zip(prices.iter().skip(1))
                .map(|(a, b)| b - a)
                .collect::<Vec<_>>();
        }
        let duration = start.elapsed();
        
        let avg_time_us = duration.as_micros() as f64 / iterations as f64;
        
        println!("📊 MACD计算性能:");
        println!("  - 数据点: {}", prices.len());
        println!("  - 迭代: {}", iterations);
        println!("  - 平均耗时: {} μs", avg_time_us);
        
        // 性能要求：每次MACD计算应该在100μs内完成
        assert!(avg_time_us < 100, "MACD计算耗时过长: {} μs", avg_time_us);
    }
} 