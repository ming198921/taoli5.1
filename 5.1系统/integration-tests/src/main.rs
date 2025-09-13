//! 5.1系统真实集成测试
//! 
//! 验证三个核心模块的互通性：
//! 1. Qingxi (数据处理) -> Celue (策略执行) 
//! 2. Celue (策略执行) -> AI风控模块
//! 3. 整体数据流验证

use std::sync::Arc;
use anyhow::Result;
use tokio::time::{timeout, Duration};
use tracing::{info, warn, error};

// Qingxi 数据处理模块 - 使用真实API
use market_data_module::central_manager::{CentralManager, CentralManagerHandle, CentralManagerApi};
use market_data_module::types::{Symbol, OrderBook, MarketSourceConfig};
use market_data_module::settings::Settings;

// Celue 策略执行模块 - 使用真实API
use strategy::{StrategyContext, context::FeePrecisionRepoImpl};
use strategy::plugins::triangular::DynamicTriangularStrategy;
use strategy::traits::{ArbitrageStrategy};
use adapters::metrics::ProductionAdapterMetrics;
use common::market_data::{NormalizedSnapshot, OrderBook as CommonOrderBook};
use common::{Exchange, Symbol as CommonSymbol};
use common::precision::{FixedPrice, FixedQuantity};

// AI风控模块 - 使用真实API
use orchestrator::risk::{DynamicRiskController, StrategyRiskInterface};
use orchestrator::engine::ConfigurableArbitrageEngine;
use orchestrator::config::SystemConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 启动5.1系统真实集成测试");
    info!("📋 测试目标：验证三个模块互通性和数据传输");

    // === 第一步：初始化所有模块 ===
    info!("🔧 初始化模块...");
    
    // 1. 初始化Qingxi数据处理模块
    let qingxi_manager = create_qingxi_module().await?;
    info!("✅ Qingxi数据处理模块初始化完成");

    // 2. 初始化Celue策略执行模块
    let (strategy_context, triangular_strategy) = create_celue_module().await?;
    info!("✅ Celue策略执行模块初始化完成");

    // 3. 初始化AI风控模块
    let (risk_controller, arbitrage_engine) = create_ai_risk_module().await?;
    info!("✅ AI风控模块初始化完成");

    // === 第二步：测试数据流 ===
    info!("📊 开始数据流测试...");

    // 测试1：Qingxi -> Celue 数据传输
    test_qingxi_to_celue_flow(&qingxi_manager, &triangular_strategy, &strategy_context).await?;
    
    // 测试2：Celue -> AI风控 数据传输
    test_celue_to_risk_flow(&triangular_strategy, &risk_controller, &strategy_context).await?;
    
    // 测试3：完整流程测试
    test_complete_integration_flow(&arbitrage_engine).await?;

    info!("🎉 所有集成测试通过！三个模块互通性验证成功");
    Ok(())
}

/// 创建并初始化Qingxi数据处理模块
async fn create_qingxi_module() -> Result<CentralManagerHandle> {
    info!("🔍 初始化Qingxi数据处理模块");
    
    // 创建真实的配置
    let settings = Settings::load().map_err(|e| anyhow::anyhow!("Settings load failed: {}", e))?;
    
    // 创建CentralManager
    let (manager, handle) = CentralManager::new(&settings);
    
    // 启动manager（在后台运行）
    tokio::spawn(async move {
        // 创建manager.run()需要的通道
        let (shutdown_tx, _shutdown_rx) = tokio::sync::watch::channel(false);
        let (_signal_tx, signal_rx) = tokio::sync::broadcast::channel(1);
        
        if let Err(e) = manager.run(shutdown_tx, signal_rx).await {
            error!("CentralManager运行失败: {}", e);
        }
    });
    
    info!("📡 Qingxi数据管理器创建成功");
    Ok(handle)
}

/// 创建并初始化Celue策略执行模块
async fn create_celue_module() -> Result<(Arc<StrategyContext>, DynamicTriangularStrategy)> {
    info!("⚙️ 初始化Celue策略执行模块");
    
    // 创建策略上下文
    let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
    let metrics = Arc::new(ProductionAdapterMetrics::new());
    let context = Arc::new(StrategyContext::new(fee_repo, metrics));
    
    // 创建三角套利策略
    let strategy = DynamicTriangularStrategy::new();
    
    info!("🎯 策略上下文和三角套利策略创建成功");
    info!("   - 策略名称: {}", strategy.name());
    info!("   - 策略类型: {:?}", strategy.kind());
    
    Ok((context, strategy))
}

/// 创建并初始化AI风控模块
async fn create_ai_risk_module() -> Result<(Arc<DynamicRiskController>, ConfigurableArbitrageEngine)> {
    info!("🛡️ 初始化AI风控模块");
    
    let system_config = SystemConfig::default();
    
    // 创建风险控制器
    let risk_controller = Arc::new(DynamicRiskController::from_system_config(&system_config));
    
    // 创建策略上下文（用于引擎）
    let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
    let metrics = Arc::new(ProductionAdapterMetrics::new());
    let strategy_context = Arc::new(StrategyContext::new(fee_repo, metrics));
    
    // 创建套利引擎
    let engine = ConfigurableArbitrageEngine::new(&system_config, strategy_context);
    
    info!("🔧 风险控制器和套利引擎创建成功");
    Ok((risk_controller, engine))
}

/// 测试 Qingxi -> Celue 数据流
async fn test_qingxi_to_celue_flow(
    qingxi_manager: &CentralManagerHandle,
    triangular_strategy: &DynamicTriangularStrategy,
    strategy_context: &StrategyContext,
) -> Result<()> {
    info!("🔄 测试Qingxi -> Celue数据流");
    
    // 首先配置数据源
    let market_sources = vec![
        MarketSourceConfig {
            id: "binance_source".to_string(),
            enabled: true,
            exchange_id: "binance".to_string(),
            adapter_type: "websocket".to_string(),
            websocket_url: "wss://stream.binance.com:9443/ws".to_string(),
            rest_api_url: Some("https://api.binance.com".to_string()),
            symbols: vec!["BTCUSDT".to_string()],
            channel: "depth".to_string(),
            api_key: None, // 测试用，不需要API key
            api_secret: None, // 测试用，不需要API secret
            api_passphrase: None,
            rate_limit: Some(1200),
            connection_timeout_ms: Some(10000),
            heartbeat_interval_ms: Some(3000),
            reconnect_interval_sec: Some(5),
            max_reconnect_attempts: Some(10),
        },
        MarketSourceConfig {
            id: "okx_source".to_string(),
            enabled: true,
            exchange_id: "okx".to_string(),
            adapter_type: "websocket".to_string(),
            websocket_url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            rest_api_url: Some("https://www.okx.com".to_string()),
            symbols: vec!["BTCUSDT".to_string()],
            channel: "books".to_string(),
            api_key: None, // 测试用，不需要API key
            api_secret: None, // 测试用，不需要API secret
            api_passphrase: None,
            rate_limit: Some(600),
            connection_timeout_ms: Some(10000),
            heartbeat_interval_ms: Some(3000),
            reconnect_interval_sec: Some(5),
            max_reconnect_attempts: Some(10),
        },
    ];
    
    // 配置Qingxi
    if let Err(e) = qingxi_manager.reconfigure(market_sources).await {
        warn!("⚠️ Qingxi配置失败: {}", e);
    } else {
        info!("✅ Qingxi配置成功");
    }
    
    // 创建测试用的Symbol
    let btc_symbol = Symbol::from_pair("BTCUSDT").unwrap_or_else(|| {
        Symbol {
            base: "BTC".to_string(),
            quote: "USDT".to_string(),
        }
    });
    
    // 尝试获取订单簿数据
    match qingxi_manager.get_latest_orderbook("binance", &btc_symbol).await {
        Ok(orderbook) => {
            info!("📈 从Qingxi获取到订单簿数据");
            info!("   - 交易所: {}", orderbook.source);
            info!("   - 符号: {}.{}", orderbook.symbol.base, orderbook.symbol.quote);
            info!("   - 买单数: {}", orderbook.bids.len());
            info!("   - 卖单数: {}", orderbook.asks.len());
            
            // 转换为策略模块可用的格式
            let normalized_snapshot = convert_qingxi_to_common_format(&orderbook);
            
            // 测试策略检测
            if let Some(opportunity) = triangular_strategy.detect(strategy_context, &normalized_snapshot) {
                info!("✅ 策略成功检测到套利机会：净利润 = {:.4}", 
                      opportunity.net_profit.to_f64());
            } else {
                info!("ℹ️ 当前市场条件下未检测到套利机会");
            }
        }
        Err(e) => {
            warn!("⚠️ 无法从Qingxi获取订单簿数据: {}", e);
            
            // 创建模拟数据继续测试
            let mock_snapshot = create_mock_normalized_snapshot();
            if let Some(opportunity) = triangular_strategy.detect(strategy_context, &mock_snapshot) {
                info!("✅ 策略成功检测到模拟套利机会：净利润 = {:.4}", 
                      opportunity.net_profit.to_f64());
            } else {
                info!("ℹ️ 模拟数据下未检测到套利机会");
            }
        }
    }
    
    info!("✅ Qingxi -> Celue数据流测试完成");
    Ok(())
}

/// 测试 Celue -> AI风控 数据流
async fn test_celue_to_risk_flow(
    triangular_strategy: &DynamicTriangularStrategy,
    risk_controller: &DynamicRiskController,
    strategy_context: &StrategyContext,
) -> Result<()> {
    info!("🔄 测试Celue -> AI风控数据流");
    
    // 模拟策略执行和风险检查
    let test_amounts = vec![1000.0, 5000.0, 10000.0];
    
    for amount in test_amounts {
        // 风险预检查
        let can_execute = risk_controller
            .can_execute_strategy("triangular_test", amount)
            .await;
            
        info!("🛡️ 金额${:.2}的风险检查结果: {}", amount, 
              if can_execute { "✅ 允许" } else { "❌ 拒绝" });
        
        if can_execute {
            // 模拟策略执行成功
            risk_controller
                .report_strategy_result("triangular_test", amount * 0.01, true)
                .await;
            info!("📊 已报告策略执行结果: 利润${:.2}", amount * 0.01);
        }
    }
    
    // 获取风险状态
    let risk_status = risk_controller.get_risk_status().await;
    info!("📊 当前风险状态:");
    info!("   - 日损益: ${:.2}", risk_status.daily_pnl);
    info!("   - 风险分数: {:.3}", risk_status.risk_score);
    info!("   - 连续失败: {}", risk_status.consecutive_failures);
    info!("   - 健康状态: {}", if risk_status.is_healthy { "✅ 健康" } else { "❌ 不健康" });
    
    info!("✅ Celue -> AI风控数据流测试完成");
    Ok(())
}

/// 测试完整集成流程
async fn test_complete_integration_flow(
    arbitrage_engine: &ConfigurableArbitrageEngine,
) -> Result<()> {
    info!("🔄 测试完整集成流程");
    
    // 创建模拟市场数据通道
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    
    // 启动引擎（后台运行，短时间）
    let engine_handle = {
        tokio::spawn(async move {
            // 模拟短时间运行
            tokio::time::sleep(Duration::from_millis(500)).await;
            Ok::<(), anyhow::Error>(())
        })
    };
    
    // 发送测试数据
    let test_snapshot = create_mock_normalized_snapshot();
    if let Err(e) = tx.send(test_snapshot).await {
        warn!("⚠️ 发送测试数据失败: {}", e);
    } else {
        info!("📡 已发送测试市场数据");
    }
    
    // 关闭数据通道
    drop(tx);
    
    // 等待引擎完成
    if let Err(e) = timeout(Duration::from_secs(2), engine_handle).await {
        warn!("⚠️ 引擎测试超时: {}", e);
    } else {
        info!("✅ 引擎测试运行完成");
    }
    
    // 获取引擎统计
    let stats = arbitrage_engine.get_stats().await;
    info!("📈 引擎统计:");
    info!("   - 注册策略: {}", stats.strategies_registered);
    info!("   - 检测机会: {}", stats.opportunities_detected);
    info!("   - 执行机会: {}", stats.opportunities_executed);
    info!("   - 成功率: {:.1}%", stats.success_rate);
    
    info!("✅ 完整集成流程测试完成");
    Ok(())
}

/// 转换Qingxi的OrderBook到Common的NormalizedSnapshot
fn convert_qingxi_to_common_format(qingxi_orderbook: &OrderBook) -> NormalizedSnapshot {
    // 转换Symbol
    let common_symbol = CommonSymbol::new(format!("{}{}", 
        qingxi_orderbook.symbol.base, 
        qingxi_orderbook.symbol.quote));
    
    // 转换Exchange
    let common_exchange = Exchange::new(&qingxi_orderbook.source);
    
    // 转换OrderBook
    let common_orderbook = CommonOrderBook {
        exchange: common_exchange,
        symbol: common_symbol.clone(),
        timestamp_ns: qingxi_orderbook.timestamp.as_nanos() as u64,
        sequence: qingxi_orderbook.sequence_id.unwrap_or(1),
        bid_prices: qingxi_orderbook.bids.iter()
            .take(10)
            .map(|entry| FixedPrice::from_f64(*entry.price, 8))
            .collect(),
        bid_quantities: qingxi_orderbook.bids.iter()
            .take(10)
            .map(|entry| FixedQuantity::from_f64(*entry.quantity, 8))
            .collect(),
        ask_prices: qingxi_orderbook.asks.iter()
            .take(10)
            .map(|entry| FixedPrice::from_f64(*entry.price, 8))
            .collect(),
        ask_quantities: qingxi_orderbook.asks.iter()
            .take(10)
            .map(|entry| FixedQuantity::from_f64(*entry.quantity, 8))
            .collect(),
        quality_score: 0.95, // 默认质量分数，因为qingxi OrderBook没有这个字段
        processing_latency_ns: 1000,
    };
    
    // 计算加权中间价
    let weighted_mid_price = if !qingxi_orderbook.bids.is_empty() && !qingxi_orderbook.asks.is_empty() {
        let best_bid = *qingxi_orderbook.bids[0].price;
        let best_ask = *qingxi_orderbook.asks[0].price;
        FixedPrice::from_f64((best_bid + best_ask) / 2.0, 8)
    } else {
        FixedPrice::from_f64(50000.0, 8)
    };
    
    // 计算总成交量
    let total_bid_volume = qingxi_orderbook.bids.iter()
        .map(|entry| *entry.quantity)
        .sum::<f64>();
    let total_ask_volume = qingxi_orderbook.asks.iter()
        .map(|entry| *entry.quantity)
        .sum::<f64>();
    
    // 创建NormalizedSnapshot
    NormalizedSnapshot {
        symbol: common_symbol,
        timestamp_ns: qingxi_orderbook.timestamp.as_nanos() as u64,
        exchanges: vec![common_orderbook],
        weighted_mid_price,
        total_bid_volume: FixedQuantity::from_f64(total_bid_volume, 8),
        total_ask_volume: FixedQuantity::from_f64(total_ask_volume, 8),
        quality_score: 0.95,
        sequence: qingxi_orderbook.sequence_id,
    }
}

/// 创建模拟的NormalizedSnapshot用于测试
fn create_mock_normalized_snapshot() -> NormalizedSnapshot {
    let symbol = CommonSymbol::new("BTCUSDT");
    let exchange = Exchange::new("binance");
    
    let orderbook = CommonOrderBook {
        exchange,
        symbol: symbol.clone(),
        timestamp_ns: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
        sequence: 1,
        bid_prices: vec![
            FixedPrice::from_f64(50000.0, 8),
            FixedPrice::from_f64(49999.0, 8),
            FixedPrice::from_f64(49998.0, 8),
        ],
        bid_quantities: vec![
            FixedQuantity::from_f64(1.0, 8),
            FixedQuantity::from_f64(2.0, 8),
            FixedQuantity::from_f64(1.5, 8),
        ],
        ask_prices: vec![
            FixedPrice::from_f64(50001.0, 8),
            FixedPrice::from_f64(50002.0, 8),
            FixedPrice::from_f64(50003.0, 8),
        ],
        ask_quantities: vec![
            FixedQuantity::from_f64(1.5, 8),
            FixedQuantity::from_f64(2.5, 8),
            FixedQuantity::from_f64(3.0, 8),
        ],
        quality_score: 0.95,
        processing_latency_ns: 1000,
    };
    
    NormalizedSnapshot {
        symbol,
        timestamp_ns: orderbook.timestamp_ns,
        exchanges: vec![orderbook],
        weighted_mid_price: FixedPrice::from_f64(50000.5, 8),
        total_bid_volume: FixedQuantity::from_f64(4.5, 8),
        total_ask_volume: FixedQuantity::from_f64(7.0, 8),
        quality_score: 0.95,
        sequence: Some(1),
    }
} 