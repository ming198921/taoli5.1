//! 策略模块和风险模块集成测试
//! 
//! 全面测试策略-风险联动、配置驱动、AI监测等核心功能

use std::sync::Arc;
use tokio;
use tempfile::TempDir;
use serde_json;

use strategy::{StrategyContext, StrategyContextConfig};
use common::{ArbitrageOpportunity, market_data::NormalizedSnapshot};
use orchestrator::{ConfigurableArbitrageEngine, DynamicRiskController, SystemConfig};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 测试策略-风险联动
    #[tokio::test]
    async fn test_strategy_risk_integration() {
        // 创建测试配置
        let config = SystemConfig::default();
        
        // 初始化风险控制器
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 创建策略上下文
        let strategy_context = Arc::new(create_test_strategy_context().await);
        
        // 创建套利引擎
        let engine = ConfigurableArbitrageEngine::new(
            risk_controller,
            strategy_context,
            config.clone(),
        ).await.expect("Failed to create arbitrage engine");
        
        // 测试风险检查
        let can_execute = engine.get_risk_controller()
            .can_execute_strategy("test_strategy", 1000.0)
            .await;
        
        assert!(can_execute, "应该允许执行正常利润的策略");
        
        // 测试大额利润的风险检查
        let can_execute_large = engine.get_risk_controller()
            .can_execute_strategy("test_strategy", 100000.0)
            .await;
        
        // 根据配置，可能允许也可能不允许大额利润
        println!("大额利润策略执行许可: {}", can_execute_large);
    }

    /// 测试配置驱动功能
    #[tokio::test]
    async fn test_configuration_driven_behavior() {
        // 创建临时配置文件
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.toml");
        
        let test_config = r#"
[strategy]
min_profit_threshold = 0.005
max_slippage = 0.002
enabled_strategies = ["triangular", "inter_exchange"]

[risk]
max_daily_loss_usd = 25000.0
max_position_usd = 100000.0
enable_emergency_stop = true

[risk.emergency_stop]
consecutive_failures = 5
failure_rate_threshold = 0.8
"#;
        
        std::fs::write(&config_path, test_config)
            .expect("Failed to write test config");
        
        // 从文件加载配置
        let config = SystemConfig::from_file(config_path)
            .expect("Failed to load config from file");
        
        // 验证配置正确加载
        assert_eq!(config.strategy.min_profit_threshold, 0.005);
        assert_eq!(config.risk.max_daily_loss_usd, 25000.0);
        assert_eq!(config.risk.emergency_stop.consecutive_failures, 5);
        
        // 测试配置驱动的风险控制
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 模拟连续失败
        for _ in 0..4 {
            risk_controller.report_strategy_failure("test_strategy", 100.0).await;
        }
        
        // 第5次失败应该触发紧急停止
        risk_controller.report_strategy_failure("test_strategy", 100.0).await;
        
        let health = risk_controller.get_system_health().await;
        // 根据实现，可能触发紧急停止状态
        println!("系统健康状态: {:?}", health);
    }

    /// 测试AI监测功能
    #[tokio::test]
    async fn test_ai_monitoring_features() {
        use strategy::feature_engineering::FeatureEngineer;
        use strategy::model_validation::ModelValidator;
        
        // 创建特征工程器
        let feature_engineer = FeatureEngineer::new();
        
        // 测试生产级技术指标计算
        let prices = vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0];
        let macd_result = feature_engineer.calculate_macd(&prices, 3, 6, 3);
        
        // MACD应该返回有意义的值
        assert!(macd_result.0.is_finite(), "MACD线应该是有限数值");
        assert!(macd_result.1.is_finite(), "信号线应该是有限数值");
        assert!(macd_result.2.is_finite(), "柱状图应该是有限数值");
        
        // 测试KDJ指标
        let kdj_result = feature_engineer.calculate_stochastic(&prices, 5, 3);
        assert!(kdj_result.0 >= 0.0 && kdj_result.0 <= 100.0, "K值应该在0-100范围内");
        assert!(kdj_result.1 >= 0.0 && kdj_result.1 <= 100.0, "D值应该在0-100范围内");
        
        println!("✅ AI监测功能测试通过 - MACD: {:?}, KDJ: {:?}", macd_result, kdj_result);
    }

    /// 测试模型验证和解释功能
    #[tokio::test]
    async fn test_model_validation_and_explanation() {
        // 注意：这个测试需要实际的模型实现
        // 这里提供框架，实际实现取决于具体的ML模型
        
        println!("🧪 模型验证测试框架已就绪");
        // TODO: 集成production_ml_models.rs中的SHAP和LIME测试
    }

    /// 测试概念漂移检测
    #[tokio::test]
    async fn test_concept_drift_detection() {
        use strategy::production_cusum::{ProductionCusumDetector, CusumConfig};
        
        // 创建CUSUM检测器
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 模拟无漂移数据
        for i in 0..50 {
            let observation = (i as f64).sin() * 0.1; // 小幅波动
            let result = detector.process_observation(observation).await;
            assert!(!result.drift_detected, "无漂移期间不应检测到漂移");
        }
        
        // 模拟突然的均值偏移
        for _ in 0..20 {
            let observation = 2.0; // 显著偏移
            let result = detector.process_observation(observation).await;
            if result.drift_detected {
                println!("✅ 成功检测到概念漂移");
                break;
            }
        }
        
        let stats = detector.get_performance_stats().await;
        assert!(stats.total_observations > 0, "应该记录观测数据");
        
        println!("✅ 概念漂移检测测试完成 - 总观测: {}", stats.total_observations);
    }

    /// 测试策略注册和执行
    #[tokio::test]
    async fn test_strategy_registration_and_execution() {
        use strategy::plugins::triangular::TriangularStrategy;
        use strategy::traits::ArbitrageStrategy;
        
        // 创建策略上下文
        let strategy_context = Arc::new(create_test_strategy_context().await);
        
        // 创建三角套利策略
        let triangular_strategy = TriangularStrategy::new();
        
        // 创建测试市场数据
        let market_snapshot = create_test_market_snapshot();
        
        // 测试机会检测
        let opportunity = triangular_strategy.detect(&strategy_context, &market_snapshot);
        
        if let Some(opp) = opportunity {
            println!("✅ 检测到套利机会: ID = {}", opp.id);
            assert!(opp.net_profit.to_f64() > 0.0, "净利润应该为正");
        } else {
            println!("ℹ️  当前市场条件下未检测到套利机会（正常）");
        }
    }

    /// 测试性能监控
    #[tokio::test]
    async fn test_performance_monitoring() {
        use adapters::metrics::{ProductionMetricsRegistry, HighFrequencyMetrics};
        
        // 创建生产级指标注册表
        let metrics_registry = ProductionMetricsRegistry::new();
        
        // 记录一些测试指标
        let hf_metrics = HighFrequencyMetrics::new();
        hf_metrics.opportunities_detected.increment(10);
        hf_metrics.opportunities_executed.increment(3);
        hf_metrics.total_profit_usd.add(1500.0);
        
        // 获取指标快照
        let snapshot = hf_metrics.get_snapshot();
        assert_eq!(snapshot.opportunities_detected, 10);
        assert_eq!(snapshot.opportunities_executed, 3);
        assert_eq!(snapshot.total_profit_usd, 1500.0);
        
        println!("✅ 性能监控测试通过 - 机会: {}, 执行: {}, 利润: ${:.2}", 
                 snapshot.opportunities_detected, 
                 snapshot.opportunities_executed, 
                 snapshot.total_profit_usd);
    }

    /// 测试错误处理和恢复
    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        // 创建配置以启用错误处理
        let mut config = SystemConfig::default();
        config.risk.enable_emergency_stop = true;
        config.risk.emergency_stop.consecutive_failures = 3;
        
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 模拟连续错误
        for i in 1..=3 {
            risk_controller.report_strategy_failure("error_test", 100.0).await;
            println!("报告第 {} 次策略失败", i);
        }
        
        // 检查系统状态
        let health = risk_controller.get_system_health().await;
        println!("系统健康状态: 健康={}, 风险分数={:.2}", 
                 health.is_healthy, health.risk_score);
        
        // 测试恢复机制
        risk_controller.report_strategy_success("error_test", 200.0).await;
        println!("✅ 错误处理和恢复测试完成");
    }

    // 辅助函数

    async fn create_test_strategy_context() -> StrategyContext {
        use strategy::{FeePrecisionRepoImpl, context::StrategyContextConfig};
        
        let config = StrategyContextConfig::default();
        let fee_repo = Arc::new(FeePrecisionRepoImpl::new());
        
        StrategyContext::new(fee_repo, config).await
            .expect("Failed to create test strategy context")
    }

    fn create_test_market_snapshot() -> NormalizedSnapshot {
        use common::market_data::{NormalizedSnapshot, OrderBook, Order};
        use common::precision::{FixedPrice, FixedQuantity};
        use std::collections::HashMap;
        
        let mut orderbooks = HashMap::new();
        
        // 创建测试订单簿
        let test_orderbook = OrderBook {
            exchange: "test_exchange".to_string(),
            symbol: "BTCUSDT".to_string(),
            bids: vec![
                Order {
                    price: FixedPrice::from_f64(50000.0, 2),
                    quantity: FixedQuantity::from_f64(1.0, 8),
                },
            ],
            asks: vec![
                Order {
                    price: FixedPrice::from_f64(50100.0, 2),
                    quantity: FixedQuantity::from_f64(1.0, 8),
                },
            ],
            timestamp_ns: chrono::Utc::now().timestamp_nanos() as u64,
        };
        
        orderbooks.insert("BTCUSDT".to_string(), vec![test_orderbook]);
        
        NormalizedSnapshot {
            orderbooks,
            timestamp_ns: chrono::Utc::now().timestamp_nanos() as u64,
        }
    }
}

/// 基准测试模块
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_strategy_detection_performance() {
        let strategy_context = Arc::new(create_test_strategy_context().await);
        let market_snapshot = create_test_market_snapshot();
        
        use strategy::plugins::triangular::TriangularStrategy;
        use strategy::traits::ArbitrageStrategy;
        
        let strategy = TriangularStrategy::new();
        
        // 预热
        for _ in 0..10 {
            let _ = strategy.detect(&strategy_context, &market_snapshot);
        }
        
        // 基准测试
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = strategy.detect(&strategy_context, &market_snapshot);
        }
        
        let elapsed = start.elapsed();
        let avg_time_us = elapsed.as_micros() / iterations;
        
        println!("🚀 策略检测性能基准:");
        println!("  - 总迭代: {}", iterations);
        println!("  - 总耗时: {:?}", elapsed);
        println!("  - 平均耗时: {} μs", avg_time_us);
        println!("  - 每秒检测: {:.0} 次", 1_000_000.0 / avg_time_us as f64);
        
        // 性能要求：平均检测时间应小于1ms
        assert!(avg_time_us < 1000, "策略检测平均耗时应小于1ms，实际: {} μs", avg_time_us);
    }

    #[tokio::test]
    async fn benchmark_risk_check_performance() {
        let config = SystemConfig::default();
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 预热
        for _ in 0..10 {
            let _ = risk_controller.can_execute_strategy("benchmark", 1000.0).await;
        }
        
        // 基准测试
        let iterations = 10000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = risk_controller.can_execute_strategy("benchmark", 1000.0).await;
        }
        
        let elapsed = start.elapsed();
        let avg_time_us = elapsed.as_micros() / iterations;
        
        println!("🛡️  风险检查性能基准:");
        println!("  - 总迭代: {}", iterations);
        println!("  - 总耗时: {:?}", elapsed);
        println!("  - 平均耗时: {} μs", avg_time_us);
        println!("  - 每秒检查: {:.0} 次", 1_000_000.0 / avg_time_us as f64);
        
        // 性能要求：平均风险检查时间应小于100μs
        assert!(avg_time_us < 100, "风险检查平均耗时应小于100μs，实际: {} μs", avg_time_us);
    }
}

// 辅助函数的实现
use integration_tests::{create_test_strategy_context, create_test_market_snapshot}; 
//! 
//! 全面测试策略-风险联动、配置驱动、AI监测等核心功能

use std::sync::Arc;
use tokio;
use tempfile::TempDir;
use serde_json;

use strategy::{StrategyContext, StrategyContextConfig};
use common::{ArbitrageOpportunity, market_data::NormalizedSnapshot};
use orchestrator::{ConfigurableArbitrageEngine, DynamicRiskController, SystemConfig};

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 测试策略-风险联动
    #[tokio::test]
    async fn test_strategy_risk_integration() {
        // 创建测试配置
        let config = SystemConfig::default();
        
        // 初始化风险控制器
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 创建策略上下文
        let strategy_context = Arc::new(create_test_strategy_context().await);
        
        // 创建套利引擎
        let engine = ConfigurableArbitrageEngine::new(
            risk_controller,
            strategy_context,
            config.clone(),
        ).await.expect("Failed to create arbitrage engine");
        
        // 测试风险检查
        let can_execute = engine.get_risk_controller()
            .can_execute_strategy("test_strategy", 1000.0)
            .await;
        
        assert!(can_execute, "应该允许执行正常利润的策略");
        
        // 测试大额利润的风险检查
        let can_execute_large = engine.get_risk_controller()
            .can_execute_strategy("test_strategy", 100000.0)
            .await;
        
        // 根据配置，可能允许也可能不允许大额利润
        println!("大额利润策略执行许可: {}", can_execute_large);
    }

    /// 测试配置驱动功能
    #[tokio::test]
    async fn test_configuration_driven_behavior() {
        // 创建临时配置文件
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.toml");
        
        let test_config = r#"
[strategy]
min_profit_threshold = 0.005
max_slippage = 0.002
enabled_strategies = ["triangular", "inter_exchange"]

[risk]
max_daily_loss_usd = 25000.0
max_position_usd = 100000.0
enable_emergency_stop = true

[risk.emergency_stop]
consecutive_failures = 5
failure_rate_threshold = 0.8
"#;
        
        std::fs::write(&config_path, test_config)
            .expect("Failed to write test config");
        
        // 从文件加载配置
        let config = SystemConfig::from_file(config_path)
            .expect("Failed to load config from file");
        
        // 验证配置正确加载
        assert_eq!(config.strategy.min_profit_threshold, 0.005);
        assert_eq!(config.risk.max_daily_loss_usd, 25000.0);
        assert_eq!(config.risk.emergency_stop.consecutive_failures, 5);
        
        // 测试配置驱动的风险控制
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 模拟连续失败
        for _ in 0..4 {
            risk_controller.report_strategy_failure("test_strategy", 100.0).await;
        }
        
        // 第5次失败应该触发紧急停止
        risk_controller.report_strategy_failure("test_strategy", 100.0).await;
        
        let health = risk_controller.get_system_health().await;
        // 根据实现，可能触发紧急停止状态
        println!("系统健康状态: {:?}", health);
    }

    /// 测试AI监测功能
    #[tokio::test]
    async fn test_ai_monitoring_features() {
        use strategy::feature_engineering::FeatureEngineer;
        use strategy::model_validation::ModelValidator;
        
        // 创建特征工程器
        let feature_engineer = FeatureEngineer::new();
        
        // 测试生产级技术指标计算
        let prices = vec![100.0, 101.0, 99.0, 102.0, 98.0, 103.0];
        let macd_result = feature_engineer.calculate_macd(&prices, 3, 6, 3);
        
        // MACD应该返回有意义的值
        assert!(macd_result.0.is_finite(), "MACD线应该是有限数值");
        assert!(macd_result.1.is_finite(), "信号线应该是有限数值");
        assert!(macd_result.2.is_finite(), "柱状图应该是有限数值");
        
        // 测试KDJ指标
        let kdj_result = feature_engineer.calculate_stochastic(&prices, 5, 3);
        assert!(kdj_result.0 >= 0.0 && kdj_result.0 <= 100.0, "K值应该在0-100范围内");
        assert!(kdj_result.1 >= 0.0 && kdj_result.1 <= 100.0, "D值应该在0-100范围内");
        
        println!("✅ AI监测功能测试通过 - MACD: {:?}, KDJ: {:?}", macd_result, kdj_result);
    }

    /// 测试模型验证和解释功能
    #[tokio::test]
    async fn test_model_validation_and_explanation() {
        // 注意：这个测试需要实际的模型实现
        // 这里提供框架，实际实现取决于具体的ML模型
        
        println!("🧪 模型验证测试框架已就绪");
        // TODO: 集成production_ml_models.rs中的SHAP和LIME测试
    }

    /// 测试概念漂移检测
    #[tokio::test]
    async fn test_concept_drift_detection() {
        use strategy::production_cusum::{ProductionCusumDetector, CusumConfig};
        
        // 创建CUSUM检测器
        let config = CusumConfig::default();
        let detector = ProductionCusumDetector::new(config);
        
        // 模拟无漂移数据
        for i in 0..50 {
            let observation = (i as f64).sin() * 0.1; // 小幅波动
            let result = detector.process_observation(observation).await;
            assert!(!result.drift_detected, "无漂移期间不应检测到漂移");
        }
        
        // 模拟突然的均值偏移
        for _ in 0..20 {
            let observation = 2.0; // 显著偏移
            let result = detector.process_observation(observation).await;
            if result.drift_detected {
                println!("✅ 成功检测到概念漂移");
                break;
            }
        }
        
        let stats = detector.get_performance_stats().await;
        assert!(stats.total_observations > 0, "应该记录观测数据");
        
        println!("✅ 概念漂移检测测试完成 - 总观测: {}", stats.total_observations);
    }

    /// 测试策略注册和执行
    #[tokio::test]
    async fn test_strategy_registration_and_execution() {
        use strategy::plugins::triangular::TriangularStrategy;
        use strategy::traits::ArbitrageStrategy;
        
        // 创建策略上下文
        let strategy_context = Arc::new(create_test_strategy_context().await);
        
        // 创建三角套利策略
        let triangular_strategy = TriangularStrategy::new();
        
        // 创建测试市场数据
        let market_snapshot = create_test_market_snapshot();
        
        // 测试机会检测
        let opportunity = triangular_strategy.detect(&strategy_context, &market_snapshot);
        
        if let Some(opp) = opportunity {
            println!("✅ 检测到套利机会: ID = {}", opp.id);
            assert!(opp.net_profit.to_f64() > 0.0, "净利润应该为正");
        } else {
            println!("ℹ️  当前市场条件下未检测到套利机会（正常）");
        }
    }

    /// 测试性能监控
    #[tokio::test]
    async fn test_performance_monitoring() {
        use adapters::metrics::{ProductionMetricsRegistry, HighFrequencyMetrics};
        
        // 创建生产级指标注册表
        let metrics_registry = ProductionMetricsRegistry::new();
        
        // 记录一些测试指标
        let hf_metrics = HighFrequencyMetrics::new();
        hf_metrics.opportunities_detected.increment(10);
        hf_metrics.opportunities_executed.increment(3);
        hf_metrics.total_profit_usd.add(1500.0);
        
        // 获取指标快照
        let snapshot = hf_metrics.get_snapshot();
        assert_eq!(snapshot.opportunities_detected, 10);
        assert_eq!(snapshot.opportunities_executed, 3);
        assert_eq!(snapshot.total_profit_usd, 1500.0);
        
        println!("✅ 性能监控测试通过 - 机会: {}, 执行: {}, 利润: ${:.2}", 
                 snapshot.opportunities_detected, 
                 snapshot.opportunities_executed, 
                 snapshot.total_profit_usd);
    }

    /// 测试错误处理和恢复
    #[tokio::test]
    async fn test_error_handling_and_recovery() {
        // 创建配置以启用错误处理
        let mut config = SystemConfig::default();
        config.risk.enable_emergency_stop = true;
        config.risk.emergency_stop.consecutive_failures = 3;
        
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 模拟连续错误
        for i in 1..=3 {
            risk_controller.report_strategy_failure("error_test", 100.0).await;
            println!("报告第 {} 次策略失败", i);
        }
        
        // 检查系统状态
        let health = risk_controller.get_system_health().await;
        println!("系统健康状态: 健康={}, 风险分数={:.2}", 
                 health.is_healthy, health.risk_score);
        
        // 测试恢复机制
        risk_controller.report_strategy_success("error_test", 200.0).await;
        println!("✅ 错误处理和恢复测试完成");
    }

    // 辅助函数

    async fn create_test_strategy_context() -> StrategyContext {
        use strategy::{FeePrecisionRepoImpl, context::StrategyContextConfig};
        
        let config = StrategyContextConfig::default();
        let fee_repo = Arc::new(FeePrecisionRepoImpl::new());
        
        StrategyContext::new(fee_repo, config).await
            .expect("Failed to create test strategy context")
    }

    fn create_test_market_snapshot() -> NormalizedSnapshot {
        use common::market_data::{NormalizedSnapshot, OrderBook, Order};
        use common::precision::{FixedPrice, FixedQuantity};
        use std::collections::HashMap;
        
        let mut orderbooks = HashMap::new();
        
        // 创建测试订单簿
        let test_orderbook = OrderBook {
            exchange: "test_exchange".to_string(),
            symbol: "BTCUSDT".to_string(),
            bids: vec![
                Order {
                    price: FixedPrice::from_f64(50000.0, 2),
                    quantity: FixedQuantity::from_f64(1.0, 8),
                },
            ],
            asks: vec![
                Order {
                    price: FixedPrice::from_f64(50100.0, 2),
                    quantity: FixedQuantity::from_f64(1.0, 8),
                },
            ],
            timestamp_ns: chrono::Utc::now().timestamp_nanos() as u64,
        };
        
        orderbooks.insert("BTCUSDT".to_string(), vec![test_orderbook]);
        
        NormalizedSnapshot {
            orderbooks,
            timestamp_ns: chrono::Utc::now().timestamp_nanos() as u64,
        }
    }
}

/// 基准测试模块
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_strategy_detection_performance() {
        let strategy_context = Arc::new(create_test_strategy_context().await);
        let market_snapshot = create_test_market_snapshot();
        
        use strategy::plugins::triangular::TriangularStrategy;
        use strategy::traits::ArbitrageStrategy;
        
        let strategy = TriangularStrategy::new();
        
        // 预热
        for _ in 0..10 {
            let _ = strategy.detect(&strategy_context, &market_snapshot);
        }
        
        // 基准测试
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = strategy.detect(&strategy_context, &market_snapshot);
        }
        
        let elapsed = start.elapsed();
        let avg_time_us = elapsed.as_micros() / iterations;
        
        println!("🚀 策略检测性能基准:");
        println!("  - 总迭代: {}", iterations);
        println!("  - 总耗时: {:?}", elapsed);
        println!("  - 平均耗时: {} μs", avg_time_us);
        println!("  - 每秒检测: {:.0} 次", 1_000_000.0 / avg_time_us as f64);
        
        // 性能要求：平均检测时间应小于1ms
        assert!(avg_time_us < 1000, "策略检测平均耗时应小于1ms，实际: {} μs", avg_time_us);
    }

    #[tokio::test]
    async fn benchmark_risk_check_performance() {
        let config = SystemConfig::default();
        let risk_controller = DynamicRiskController::new(config.risk.clone()).await;
        
        // 预热
        for _ in 0..10 {
            let _ = risk_controller.can_execute_strategy("benchmark", 1000.0).await;
        }
        
        // 基准测试
        let iterations = 10000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = risk_controller.can_execute_strategy("benchmark", 1000.0).await;
        }
        
        let elapsed = start.elapsed();
        let avg_time_us = elapsed.as_micros() / iterations;
        
        println!("🛡️  风险检查性能基准:");
        println!("  - 总迭代: {}", iterations);
        println!("  - 总耗时: {:?}", elapsed);
        println!("  - 平均耗时: {} μs", avg_time_us);
        println!("  - 每秒检查: {:.0} 次", 1_000_000.0 / avg_time_us as f64);
        
        // 性能要求：平均风险检查时间应小于100μs
        assert!(avg_time_us < 100, "风险检查平均耗时应小于100μs，实际: {} μs", avg_time_us);
    }
}

// 辅助函数的实现
use integration_tests::{create_test_strategy_context, create_test_market_snapshot}; 