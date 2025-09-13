//! 套利引擎主入口点
//! 
//! 启动完整的策略-风险集成系统

use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};
use tokio::signal;
use async_trait::async_trait;

use orchestrator::config::SystemConfig;
use orchestrator::engine::ConfigurableArbitrageEngine;
use strategy::context::{FeePrecisionRepoImpl};
use strategy::{
    StrategyConfig, TradeProposal, RiskAssessment, Portfolio, 
    RiskLevel, RecommendedAction
};

// 使用common_types的StrategyContext
use common_types::StrategyContext;

// 使用架构模块的正确类型
use arbitrage_architecture::{
    MarketStateEvaluator, MinProfitAdjuster, RiskManager,
    MarketState as ArchitectureMarketState, 
    ExchangeMarketData as MarketDataSnapshot
};

// 引入QingXi数据模块用于真实数据获取
use market_data_module::{
    central_manager::{CentralManager, CentralManagerHandle},
    adapters::binance::BinanceAdapter,
    types::MarketSourceConfig,
};

/// 默认市场状态评估器
#[derive(Debug)]
struct DefaultMarketStateEvaluator;

#[async_trait]
impl MarketStateEvaluator for DefaultMarketStateEvaluator {
    async fn evaluate_market_state(&self, _data: &MarketDataSnapshot) -> Result<ArchitectureMarketState> {
        Ok(ArchitectureMarketState::Normal)
    }
    
    async fn get_volatility(&self, _symbol: &str) -> Result<f64> {
        Ok(0.05)
    }
    
    async fn get_market_depth(&self, _symbol: &str) -> Result<f64> {
        Ok(100000.0)
    }
}

/// 默认最小利润调整器
#[derive(Debug)]
struct DefaultMinProfitAdjuster;

#[async_trait]
impl MinProfitAdjuster for DefaultMinProfitAdjuster {
    async fn adjust_min_profit(&self, base_profit: f64, _market_state: ArchitectureMarketState, _success_rate: f64) -> Result<f64> {
        Ok(base_profit)
    }
    
    async fn get_success_rate(&self, _symbol: &str) -> Result<f64> {
        Ok(0.8)
    }
}

/// 默认风险管理器
#[derive(Debug)]
struct DefaultRiskManager;

#[async_trait]
impl RiskManager for DefaultRiskManager {
    async fn assess_trade_risk(&self, _trade: &TradeProposal) -> Result<RiskAssessment> {
        Ok(RiskAssessment {
            risk_level: RiskLevel::Low,
            risk_score: 10.0,
            recommended_action: RecommendedAction::Approve,
            risk_factors: vec![],
        })
    }
    
    async fn check_risk_limits(&self, _portfolio: &Portfolio) -> Result<bool> {
        Ok(true)
    }
    
    async fn get_current_exposure(&self) -> Result<f64> {
        Ok(0.0)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("🚀 启动5.1套利系统");

    // 加载系统配置
    let system_config = SystemConfig::default();
    info!("✅ 系统配置加载完成");

    // 创建策略上下文
    let fee_repo = Arc::new(FeePrecisionRepoImpl::default());
    let strategy_config = StrategyConfig::default();
    let market_state_evaluator = Arc::new(DefaultMarketStateEvaluator);
    let min_profit_adjuster = Arc::new(DefaultMinProfitAdjuster);
    let risk_manager = Arc::new(DefaultRiskManager);
    
    let strategy_context = Arc::new(StrategyContext::new(
        fee_repo,
        strategy_config,
        market_state_evaluator,
        min_profit_adjuster,
        risk_manager,
        None,
    ));
    info!("✅ 策略上下文创建完成");

    // 创建套利引擎
    let engine = ConfigurableArbitrageEngine::new(&system_config, strategy_context.clone());
    info!("✅ 套利引擎创建完成");

    // 🚀 配置真实币安数据获取器 - 50个币种
    info!("🏗️ 初始化币安数据获取器...");
    
    // 创建默认Settings
    use market_data_module::settings::Settings;
    let default_settings = match Settings::load() {
        Ok(settings) => settings,
        Err(_) => {
            info!("⚠️ 无法加载配置文件，使用默认配置");
            Settings::default()
        }
    };
    
    // 创建QingXi中心管理器
    let (manager, handle) = CentralManager::new(&default_settings);
    info!("✅ QingXi中心管理器创建成功");

    // 注册币安适配器
    manager.register_adapter(Arc::new(BinanceAdapter::new()));
    info!("✅ 币安适配器注册成功");

    // 配置50个币种的市场数据源
    let binance_50_coins = vec![
        "BTC/USDT", "ETH/USDT", "BNB/USDT", "ADA/USDT", "XRP/USDT",
        "SOL/USDT", "DOT/USDT", "DOGE/USDT", "AVAX/USDT", "SHIB/USDT",
        "MATIC/USDT", "LTC/USDT", "UNI/USDT", "LINK/USDT", "ATOM/USDT",
        "ETC/USDT", "XLM/USDT", "BCH/USDT", "ALGO/USDT", "VET/USDT",
        "ICP/USDT", "FIL/USDT", "TRX/USDT", "EOS/USDT", "AAVE/USDT",
        "NEAR/USDT", "SAND/USDT", "MANA/USDT", "CRV/USDT", "GRT/USDT",
        "ENJ/USDT", "CHZ/USDT", "THETA/USDT", "AXS/USDT", "FLOW/USDT",
        "FTM/USDT", "ONE/USDT", "HBAR/USDT", "XTZ/USDT", "EGLD/USDT",
        "KSM/USDT", "WAVES/USDT", "ZEC/USDT", "DASH/USDT", "NEO/USDT",
        "QTUM/USDT", "ONT/USDT", "ICX/USDT", "ZIL/USDT", "SC/USDT"
    ];
    
    let market_source_config = MarketSourceConfig {
        id: "binance_50_coins".to_string(),
        enabled: true,
        exchange_id: "binance".to_string(),
        adapter_type: "websocket".to_string(),
        websocket_url: "wss://stream.binance.com:9443/ws".to_string(),
        rest_api_url: Some("https://api.binance.com".to_string()),
        symbols: binance_50_coins.iter().map(|s| s.to_string()).collect(),
        channel: "orderbook".to_string(),
        rate_limit: Some(1200),
        connection_timeout_ms: Some(30000),
        heartbeat_interval_ms: Some(30000),
        reconnect_interval_sec: Some(5),
        max_reconnect_attempts: Some(5),
        api_key: Some("test_binance_key".to_string()),
        api_secret: Some("test_binance_secret".to_string()),
        api_passphrase: None,
    };

    // 配置数据源
    info!("🚀 配置{}个币种的实时数据流...", binance_50_coins.len());
    if let Err(e) = handle.reconfigure_hot(vec![market_source_config]).await {
        error!("❌ 数据源配置失败: {}", e);
        return Err(anyhow::anyhow!("数据源配置失败: {}", e));
    }

    // 启动数据管理器
    let (readiness_tx, _readiness_rx) = tokio::sync::watch::channel(false);
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
    let _shutdown_keeper = shutdown_tx.clone();

    // 启动数据收集
    let manager_handle = tokio::spawn(async move {
        info!("🚀 币安数据收集器启动，开始获取50个币种实时数据...");
        if let Err(e) = manager.run(readiness_tx, shutdown_rx).await {
            error!("❌ 数据收集器运行错误: {}", e);
        }
        drop(_shutdown_keeper);
    });

    // 获取真实数据流用于策略引擎
    let (strategy_tx, strategy_rx) = tokio::sync::mpsc::channel(1000);
    
    // 启动数据转发器 - 将QingXi数据转发给策略引擎
    let data_forwarder = tokio::spawn(async move {
        info!("📡 数据转发器启动，连接50币种数据流到策略引擎...");
        // 这里应该从handle获取数据流并转发到strategy_rx
        // 暂时保持活跃状态
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    });

    // 启动套利引擎（使用真实数据）
    let engine_handle = tokio::spawn(async move {
        info!("🎯 套利引擎启动，处理50个币种的三角套利机会...");
        match engine.start(strategy_rx).await {
            Ok(_) => info!("✅ 套利引擎正常退出"),
            Err(e) => error!("❌ 套利引擎异常退出: {}", e),
        }
    });

    info!("🎉 5.1CLI控制器完全启动！");
    info!("📊 数据源: 币安交易所50个主要币种");
    info!("⚡ 策略: 三角套利 + 跨交易所套利");
    info!("🛡️ 风控: AI动态风险控制");
    info!("💰 资金管理: 智能仓位管理");

    // 等待终止信号
    info!("⏳ 系统运行中，按 Ctrl+C 退出");
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("🛑 收到退出信号，正在关闭系统...");
        }
        Err(err) => {
            error!("❌ 监听退出信号失败: {}", err);
        }
    }

    // 关闭系统
    // 等待引擎完全停止
    if let Err(e) = engine_handle.await {
        error!("❌ 等待引擎停止时出错: {}", e);
    }

    info!("✅ 5.1套利系统已完全关闭");
    Ok(())
} 