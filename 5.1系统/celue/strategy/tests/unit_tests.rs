//! 策略模块单元测试
//! 
//! 测试核心策略功能、特征工程、模型验证等

#[cfg(test)]
mod strategy_tests {
    use strategy::feature_engineering::FeatureEngineer;
    use strategy::model_validation::ModelValidator;
    use strategy::config_loader::ConfigLoader;
    
    #[test]
    fn test_feature_engineering_macd() {
        let engineer = FeatureEngineer::new();
        let prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 102.0, 104.0];
        
        let (macd, signal, histogram) = engineer.calculate_macd(&prices, 3, 6, 3);
        
        assert!(macd.is_finite(), "MACD值应该是有限数值");
        assert!(signal.is_finite(), "信号线应该是有限数值");
        assert!(histogram.is_finite(), "柱状图应该是有限数值");
        
        println!("✅ MACD测试通过: MACD={:.4}, Signal={:.4}, Histogram={:.4}", 
                 macd, signal, histogram);
    }
    
    #[test]
    fn test_feature_engineering_kdj() {
        let engineer = FeatureEngineer::new();
        let prices = vec![100.0, 99.0, 101.0, 98.0, 102.0, 97.0, 103.0];
        
        let (k, d) = engineer.calculate_stochastic(&prices, 5, 3);
        
        assert!(k >= 0.0 && k <= 100.0, "K值应该在0-100范围内，实际: {}", k);
        assert!(d >= 0.0 && d <= 100.0, "D值应该在0-100范围内，实际: {}", d);
        
        println!("✅ KDJ测试通过: K={:.2}, D={:.2}", k, d);
    }
    
    #[test]
    fn test_config_loader() {
        let loader = ConfigLoader::new();
        
        // 测试默认配置加载
        let config = loader.load_default_config();
        assert!(config.min_profit_threshold > 0.0, "最小利润阈值应该大于0");
        assert!(config.max_slippage > 0.0, "最大滑点应该大于0");
        assert!(!config.enabled_strategies.is_empty(), "应该有启用的策略");
        
        println!("✅ 配置加载测试通过 - 利润阈值: {:.4}, 滑点: {:.4}", 
                 config.min_profit_threshold, config.max_slippage);
    }
}

#[cfg(test)]
mod risk_tests {
    use orchestrator::{DynamicRiskController, SystemConfig};
    
    #[tokio::test]
    async fn test_risk_controller_basic_checks() {
        let config = SystemConfig::default();
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 测试正常利润策略
        let can_execute = risk_controller
            .can_execute_strategy("test_strategy", 1000.0)
            .await;
        assert!(can_execute, "正常利润策略应该被允许执行");
        
        // 测试系统健康状态
        let health = risk_controller.get_system_health().await;
        assert!(health.risk_score >= 0.0 && health.risk_score <= 1.0, 
                "风险分数应该在0-1范围内");
        
        println!("✅ 风险控制基础测试通过 - 风险分数: {:.2}", health.risk_score);
    }
    
    #[tokio::test]
    async fn test_risk_controller_failure_handling() {
        let config = SystemConfig::default();
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 报告策略失败
        risk_controller.report_strategy_failure("test_strategy", 500.0).await;
        
        // 报告策略成功
        risk_controller.report_strategy_success("test_strategy", 1000.0).await;
        
        let health = risk_controller.get_system_health().await;
        println!("✅ 风险控制失败处理测试完成 - 最终健康状态: {}", health.is_healthy);
    }
}

#[cfg(test)]
mod ml_tests {
    use strategy::production_cusum::{ProductionCusumDetector, CusumConfig};
    
    #[tokio::test]
    async fn test_cusum_detector() {
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 测试正常数据
        for i in 0..20 {
            let observation = (i as f64 * 0.1).sin();
            let result = detector.process_observation(observation).await;
            assert!(!result.drift_detected, "正常数据不应触发漂移检测");
        }
        
        // 测试显著偏移
        let result = detector.process_observation(5.0).await;
        // 可能检测到漂移，取决于配置
        
        let stats = detector.get_performance_stats().await;
        assert!(stats.total_observations == 21, "应该记录21个观测");
        
        println!("✅ CUSUM检测器测试通过 - 观测数: {}", stats.total_observations);
    }
}

#[cfg(test)]
mod performance_tests {
    use std::time::Instant;
    use strategy::feature_engineering::FeatureEngineer;
    
    #[test]
    fn benchmark_macd_calculation() {
        let engineer = FeatureEngineer::new();
        let prices: Vec<f64> = (0..1000).map(|i| 100.0 + (i as f64 * 0.01).sin()).collect();
        
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = engineer.calculate_macd(&prices, 12, 26, 9);
        }
        
        let elapsed = start.elapsed();
        let avg_time_us = elapsed.as_micros() / iterations;
        
        println!("📊 MACD计算性能:");
        println!("  - 数据点: {}", prices.len());
        println!("  - 迭代: {}", iterations);
        println!("  - 平均耗时: {} μs", avg_time_us);
        
        // 性能要求：每次MACD计算应该在100μs内完成
        assert!(avg_time_us < 100, "MACD计算耗时过长: {} μs", avg_time_us);
    }
} 
//! 
//! 测试核心策略功能、特征工程、模型验证等

#[cfg(test)]
mod strategy_tests {
    use strategy::feature_engineering::FeatureEngineer;
    use strategy::model_validation::ModelValidator;
    use strategy::config_loader::ConfigLoader;
    
    #[test]
    fn test_feature_engineering_macd() {
        let engineer = FeatureEngineer::new();
        let prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 102.0, 104.0];
        
        let (macd, signal, histogram) = engineer.calculate_macd(&prices, 3, 6, 3);
        
        assert!(macd.is_finite(), "MACD值应该是有限数值");
        assert!(signal.is_finite(), "信号线应该是有限数值");
        assert!(histogram.is_finite(), "柱状图应该是有限数值");
        
        println!("✅ MACD测试通过: MACD={:.4}, Signal={:.4}, Histogram={:.4}", 
                 macd, signal, histogram);
    }
    
    #[test]
    fn test_feature_engineering_kdj() {
        let engineer = FeatureEngineer::new();
        let prices = vec![100.0, 99.0, 101.0, 98.0, 102.0, 97.0, 103.0];
        
        let (k, d) = engineer.calculate_stochastic(&prices, 5, 3);
        
        assert!(k >= 0.0 && k <= 100.0, "K值应该在0-100范围内，实际: {}", k);
        assert!(d >= 0.0 && d <= 100.0, "D值应该在0-100范围内，实际: {}", d);
        
        println!("✅ KDJ测试通过: K={:.2}, D={:.2}", k, d);
    }
    
    #[test]
    fn test_config_loader() {
        let loader = ConfigLoader::new();
        
        // 测试默认配置加载
        let config = loader.load_default_config();
        assert!(config.min_profit_threshold > 0.0, "最小利润阈值应该大于0");
        assert!(config.max_slippage > 0.0, "最大滑点应该大于0");
        assert!(!config.enabled_strategies.is_empty(), "应该有启用的策略");
        
        println!("✅ 配置加载测试通过 - 利润阈值: {:.4}, 滑点: {:.4}", 
                 config.min_profit_threshold, config.max_slippage);
    }
}

#[cfg(test)]
mod risk_tests {
    use orchestrator::{DynamicRiskController, SystemConfig};
    
    #[tokio::test]
    async fn test_risk_controller_basic_checks() {
        let config = SystemConfig::default();
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 测试正常利润策略
        let can_execute = risk_controller
            .can_execute_strategy("test_strategy", 1000.0)
            .await;
        assert!(can_execute, "正常利润策略应该被允许执行");
        
        // 测试系统健康状态
        let health = risk_controller.get_system_health().await;
        assert!(health.risk_score >= 0.0 && health.risk_score <= 1.0, 
                "风险分数应该在0-1范围内");
        
        println!("✅ 风险控制基础测试通过 - 风险分数: {:.2}", health.risk_score);
    }
    
    #[tokio::test]
    async fn test_risk_controller_failure_handling() {
        let config = SystemConfig::default();
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 报告策略失败
        risk_controller.report_strategy_failure("test_strategy", 500.0).await;
        
        // 报告策略成功
        risk_controller.report_strategy_success("test_strategy", 1000.0).await;
        
        let health = risk_controller.get_system_health().await;
        println!("✅ 风险控制失败处理测试完成 - 最终健康状态: {}", health.is_healthy);
    }
}

#[cfg(test)]
mod ml_tests {
    use strategy::production_cusum::{ProductionCusumDetector, CusumConfig};
    
    #[tokio::test]
    async fn test_cusum_detector() {
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 测试正常数据
        for i in 0..20 {
            let observation = (i as f64 * 0.1).sin();
            let result = detector.process_observation(observation).await;
            assert!(!result.drift_detected, "正常数据不应触发漂移检测");
        }
        
        // 测试显著偏移
        let result = detector.process_observation(5.0).await;
        // 可能检测到漂移，取决于配置
        
        let stats = detector.get_performance_stats().await;
        assert!(stats.total_observations == 21, "应该记录21个观测");
        
        println!("✅ CUSUM检测器测试通过 - 观测数: {}", stats.total_observations);
    }
}

#[cfg(test)]
mod performance_tests {
    use std::time::Instant;
    use strategy::feature_engineering::FeatureEngineer;
    
    #[test]
    fn benchmark_macd_calculation() {
        let engineer = FeatureEngineer::new();
        let prices: Vec<f64> = (0..1000).map(|i| 100.0 + (i as f64 * 0.01).sin()).collect();
        
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = engineer.calculate_macd(&prices, 12, 26, 9);
        }
        
        let elapsed = start.elapsed();
        let avg_time_us = elapsed.as_micros() / iterations;
        
        println!("📊 MACD计算性能:");
        println!("  - 数据点: {}", prices.len());
        println!("  - 迭代: {}", iterations);
        println!("  - 平均耗时: {} μs", avg_time_us);
        
        // 性能要求：每次MACD计算应该在100μs内完成
        assert!(avg_time_us < 100, "MACD计算耗时过长: {} μs", avg_time_us);
    }
} 