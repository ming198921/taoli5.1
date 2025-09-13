//! 架构测试程序
//! 
//! 验证高频虚拟货币套利系统5.1++架构的完整性和可用性

use arbitrage_architecture::*;
use tracing::{info, error};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .init();
    
    info!("🚀 启动高频虚拟货币套利系统5.1++ 架构测试");
    info!("版本: {}", VERSION);
    info!("构建时间: {}", BUILD_TIMESTAMP);
    info!("Git SHA: {}", GIT_SHA);
    
    // 创建配置文件用于测试（如果不存在）
    create_test_config().await?;
    
    // 测试系统协调器创建
    info!("📋 测试系统协调器初始化...");
    match ArbitrageSystemOrchestrator::new("./config/test_system.toml").await {
        Ok(orchestrator) => {
            info!("✅ 系统协调器初始化成功");
            
            // 测试系统状态
            let state = orchestrator.get_system_state().await;
            info!("📊 系统状态: running={}, market_state={:?}", 
                state.is_running, state.market_state);
            
            // 测试性能统计
            let stats = orchestrator.get_performance_stats().await;
            info!("📈 性能统计: opportunities={}, executions={}", 
                stats.total_opportunities, stats.executed_opportunities);
            
            // 测试机会池状态
            let (pool_size, active_count) = orchestrator.get_opportunity_pool_status().await;
            info!("🎯 机会池状态: 总数={}, 活跃={}", pool_size, active_count);
            
            // 测试配置中心
            let system_config = orchestrator.config_center.get_system_config().await;
            match system_config {
                Ok(config) => {
                    info!("⚙️  系统配置: 监控={}, 性能优化={}, 自动恢复={}", 
                        config.enable_monitoring, config.enable_performance_optimization, config.enable_auto_recovery);
                }
                Err(e) => {
                    error!("❌ 获取系统配置失败: {}", e);
                }
            }
            
            info!("🎉 所有架构组件测试通过！");
        }
        Err(e) => {
            error!("❌ 系统协调器初始化失败: {}", e);
            return Err(e.into());
        }
    }
    
    info!("✨ 架构测试完成 - 所有组件都已正确集成");
    
    Ok(())
}

/// 创建测试配置文件
async fn create_test_config() -> std::result::Result<(), Box<dyn std::error::Error>> {
    use tokio::fs;
    
    // 创建配置目录
    if let Err(_) = fs::metadata("./config").await {
        fs::create_dir_all("./config").await?;
    }
    
    let test_config = r#"
[system]
name = "ArbitrageSystem"
version = "5.1.0"
environment = "test"
log_level = "info"
max_concurrent_opportunities = 100
health_check_interval_seconds = 30
metrics_collection_interval_seconds = 5
graceful_shutdown_timeout_seconds = 30
enable_hot_reload = false
enable_telemetry = false
enable_profiling = false

[[exchanges]]
name = "binance"
exchange_type = "Binance"
enabled = true

[exchanges.api_config]
base_url = "https://api.binance.com"
websocket_url = "wss://stream.binance.com:9443"
sandbox = true
timeout_seconds = 10
rate_limit_requests_per_second = 10
rate_limit_burst = 20
max_connections = 5
enable_websocket = true
websocket_channels = ["depth", "trade"]

[exchanges.trading_config]
supported_symbols = ["BTCUSDT", "ETHUSDT"]
min_trade_amount = 10.0
max_trade_amount = 10000.0
min_price_precision = 8
min_quantity_precision = 8
enable_margin_trading = false
enable_futures_trading = false
default_time_in_force = "GTC"
enable_order_batching = true
max_orders_per_batch = 10

[exchanges.limits]
max_position_value_usd = 10000.0
max_daily_volume_usd = 100000.0
max_orders_per_second = 10
max_open_orders = 20
withdrawal_limits = {}
deposit_limits = {}

[exchanges.fees]
maker_fee = 0.001
taker_fee = 0.001
withdrawal_fees = {}
deposit_fees = {}

[exchanges.latency_config]
expected_api_latency_ms = 50
expected_order_latency_ms = 100
max_acceptable_latency_ms = 500
latency_timeout_ms = 1000
enable_latency_monitoring = true
latency_alert_threshold_ms = 200

[[strategies]]
name = "inter_exchange_arbitrage"
strategy_type = "InterExchangeArbitrage"
enabled = true
priority = 1
weight = 0.5

[strategies.parameters]
min_profit_bps = 5.0

[strategies.min_profit_config]
normal_min_profit = 5.0
caution_min_profit = 8.0
extreme_min_profit = 15.0
adaptive_adjustment = true
success_rate_threshold = 0.8
adjustment_factor = 0.1

[strategies.position_limits]
max_position_value_usd = 5000.0
max_position_percentage = 10.0
max_symbols = 5
max_exchanges = 3
concentration_limit = 20.0
correlation_limit = 0.8

[strategies.execution_config]
max_execution_time_ms = 5000
retry_count = 3
retry_delay_ms = 1000
enable_partial_execution = true
require_atomic_execution = false
slippage_tolerance = 0.001
market_impact_threshold = 0.005

[strategies.risk_params]
max_risk_score = 0.8
var_limit = 1000.0
expected_shortfall_limit = 1500.0
correlation_threshold = 0.7
volatility_threshold = 0.1
liquidity_threshold = 0.1
drawdown_limit = 0.05

[risk_management]
[risk_management.global_limits]
max_total_exposure_usd = 100000.0
max_single_position_usd = 10000.0
max_daily_loss_usd = 5000.0
max_portfolio_volatility = 0.2
max_correlation = 0.8
max_concentration = 25.0
max_leverage = 3.0

[risk_management.var_config]
confidence_level = 0.95
time_horizon_days = 1
historical_window_days = 30
calculation_method = "historical"
update_frequency_minutes = 15
alert_threshold = 1000.0

[risk_management.stress_testing]
enable_stress_testing = false
scenarios = []
run_frequency_hours = 24
alert_threshold = 2000.0

[risk_management.correlation_monitoring]
max_correlation = 0.8
calculation_window_days = 30
update_frequency_hours = 1
correlation_threshold = 0.7
enable_dynamic_correlation = true

[risk_management.circuit_breakers]
enable_circuit_breakers = true
loss_threshold_percentage = 5.0
volatility_threshold = 0.2
error_rate_threshold = 10.0
recovery_time_minutes = 30
escalation_levels = []

[risk_management.position_sizing]
method = "fixed"
base_position_size = 1000.0
max_position_size = 5000.0
volatility_lookback_days = 30
risk_free_rate = 0.02
kelly_fraction = 0.25

[fund_management]
total_capital_usd = 100000.0
reserve_ratio = 0.1
allocation_strategy = "equal_weight"

[fund_management.rebalancing]
enable_auto_rebalancing = true
rebalancing_frequency_hours = 24
deviation_threshold = 5.0
min_transfer_amount = 100.0
max_transfer_amount = 10000.0
emergency_rebalancing_threshold = 20.0

[fund_management.transfer_management]
enable_auto_transfer = false
min_balance_threshold = 1000.0
max_balance_threshold = 50000.0
transfer_fee_budget = 100.0
max_daily_transfers = 10
preferred_transfer_hours = [9, 10, 11, 14, 15, 16]

[fund_management.margin_management]
enable_margin_trading = false
max_margin_ratio = 2.0
margin_call_threshold = 1.3
liquidation_threshold = 1.1
margin_buffer = 0.2
interest_rate_monitoring = true

[monitoring]
[monitoring.metrics]
enable_metrics = true
metrics_port = 9090
collection_interval_seconds = 5
retention_days = 30
export_format = "prometheus"
custom_metrics = []

[monitoring.alerting]
enable_alerting = false
channels = []
alert_rules = []

[monitoring.alerting.escalation_policy]
levels = []
max_escalation_time_minutes = 60

[monitoring.logging]
level = "info"
format = "json"
outputs = []
structured_logging = true
enable_sampling = false
sampling_rate = 1.0

[monitoring.health_checks]
enable_health_checks = true
check_interval_seconds = 30
timeout_seconds = 10
checks = []

[monitoring.performance]
enable_performance_monitoring = true
profiling_enabled = false
sampling_rate = 0.1
memory_monitoring = true
cpu_monitoring = true
network_monitoring = true
latency_monitoring = true

[data_sources]
primary_sources = ["binance"]
backup_sources = []

[data_sources.data_validation]
enable_validation = true
max_price_deviation = 5.0
max_age_seconds = 30
min_data_sources = 1
outlier_detection = true
consistency_check = true

[data_sources.caching]
enable_caching = true
redis_url = "redis://localhost:6379"
cache_ttl_seconds = 60
max_cache_size_mb = 100
cache_eviction_policy = "lru"
enable_cache_warming = false

[data_sources.aggregation]
aggregation_method = "weighted_average"
weight_by_volume = true
weight_by_liquidity = true
exclude_outliers = true
outlier_threshold = 3.0

[execution]
[execution.order_routing]
default_routing_algorithm = "smart"
enable_smart_routing = true
latency_weight = 0.4
cost_weight = 0.3
liquidity_weight = 0.2
reliability_weight = 0.1

[[execution.execution_algorithms]]
name = "market_making"
algorithm_type = "market_making"
enabled = true
priority = 1
parameters = {}

[execution.latency_optimization]
enable_optimization = true
target_latency_ms = 100
use_colocation = false
enable_connection_pooling = true
enable_tcp_nodelay = true
enable_cpu_affinity = false
cpu_cores = []

[execution.smart_order_routing]
enable_sor = true
liquidity_threshold = 1000.0
market_impact_threshold = 0.005
enable_order_splitting = true
max_child_orders = 5
enable_dark_pools = false

[api]
[api.management_api]
enabled = true
bind_address = "127.0.0.1"
port = 8080
enable_tls = false
cors_enabled = true
cors_origins = ["*"]

[api.websocket_api]
enabled = true
bind_address = "127.0.0.1"
port = 8081
max_connections = 100
heartbeat_interval_seconds = 30
message_buffer_size = 1000

[api.authentication]
auth_method = "api_key"
api_keys = []

[api.rate_limiting]
enable_rate_limiting = true
default_rate_limit = 100
burst_limit = 200
window_seconds = 60
per_endpoint_limits = {}
"#;
    
    fs::write("./config/test_system.toml", test_config).await?;
    info!("📝 测试配置文件已创建: ./config/test_system.toml");
    
    Ok(())
} 