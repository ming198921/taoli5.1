#![allow(dead_code)]
//! # qingxi 市场数据服务 - 生产级主程序入口
//!
//! 基于权威类型系统的高性能多源加密货币市场数据采集、清洗与一致性验证系统主入口。

use market_data_module::{
    // 适配器实现导入
    adapters::{binance::BinanceAdapter, huobi::HuobiAdapter, okx::OkxAdapter},
    // 核心服务模块
    api_server,
    http_api,
    build_info,
    observability,
    settings::{self, Settings},
    // 权威API导入
    CentralManager,
    CentralManagerApi,
    MarketDataError,
    SYSTEM_NAME,
    VERSION,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

fn main() -> anyhow::Result<()> {
    // 权威系统启动信息
    println!("🚀 Starting {SYSTEM_NAME} v{VERSION}");
    println!("📊 {}", build_info());
    println!("📂 Current directory: {:?}", std::env::current_dir()?);

    // 早期加载配置以获取线程配置
    let settings = Settings::load().unwrap_or_else(|e| {
        eprintln!("⚠️ Failed to load settings, using defaults: {}", e);
        Settings {
            general: settings::GeneralSettings {
                log_level: "info".to_string(),
                metrics_enabled: true,
            },
            api_server: settings::ApiServerSettings {
                host: "0.0.0.0".to_string(),
                port: 50051,
                metrics_port_offset: 1,
                health_port_offset: 2,
                http_port_offset: 10,
                orderbook_depth_limit: 20,
                symbols_list_limit: 100,
            },
            central_manager: settings::CentralManagerSettings { event_buffer_size: 1000 },
            sources: vec![],
            consistency_thresholds: market_data_module::types::ConsistencyThresholds {
                price_diff_percentage: 0.5,
                timestamp_diff_ms: 5000,
                sequence_gap_threshold: 10,
                spread_threshold_percentage: 1.0,
                critical_spread_threshold_percentage: 2.0,
                max_time_diff_ms: 10000.0,
                volume_consistency_threshold: 0.5,
            },
            reasoner: settings::ReasonerSettings {
                api_endpoint: "http://127.0.0.1:8081".to_string(),
            },
            anomaly_detection: settings::AnomalyDetectionSettings {
                spread_threshold: 2.0,
                volume_threshold: 100.0,
                price_change_threshold: 5.0,
                spread_threshold_percentage: 1.0,
            },
            performance: settings::PerformanceSettings::default(),
            threading: settings::ThreadingSettings::default(),
            quality_thresholds: settings::QualityThresholds::default(),
        }
    });

    // 检查是否在容器环境中或禁用 CPU 亲和性
    let disable_cpu_affinity = std::env::var("QINGXI_DISABLE_CPU_AFFINITY")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if disable_cpu_affinity {
        println!("⚠️ CPU affinity disabled via environment variable");
    }

    // 使用配置中的网络运行时设置
    let threading_config = settings.threading.clone();
    let network_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threading_config.network_worker_threads)
        .thread_name("qingxi-network")
        .on_thread_start(move || {
            if !disable_cpu_affinity {
                let core_ids = threading_config.network_cpu_cores.clone();
                let current_id = std::thread::current().id();
                let thread_index = format!("{current_id:?}")
                    .chars()
                    .filter(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<usize>()
                    .unwrap_or(0)
                    % core_ids.len();

                let core_id = core_affinity::CoreId {
                    id: core_ids[thread_index],
                };
                if !core_affinity::set_for_current(core_id) {
                    eprintln!(
                        "⚠️ Failed to set network thread affinity to core {}",
                        core_ids[thread_index]
                    );
                } else {
                    println!(
                        "✅ Network thread bound to CPU core {}",
                        core_ids[thread_index]
                    );
                }
            } else {
                println!("✅ Network thread started (CPU affinity disabled)");
            }
        })
        .enable_all()
        .build()?;

    // 使用配置中的处理运行时设置
    let processing_core = settings.threading.processing_cpu_core;
    let processing_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(settings.threading.processing_worker_threads)
        .thread_name("qingxi-processing")
        .on_thread_start(move || {
            if !disable_cpu_affinity {
                let core_id = core_affinity::CoreId { id: processing_core };
                if !core_affinity::set_for_current(core_id) {
                    eprintln!("⚠️ Failed to set processing thread affinity to core {}", processing_core);
                } else {
                    println!("✅ Processing thread bound to CPU core {}", processing_core);
                }
            } else {
                println!("✅ Processing thread started (CPU affinity disabled)");
            }
        })
        .enable_all()
        .build()?;

    // 主运行时处理配置和协调
    let main_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(settings.threading.main_worker_threads)
        .thread_name("qingxi-main")
        .enable_all()
        .build()?;

    main_runtime.block_on(async { run_main_logic(network_runtime, processing_runtime, settings).await })
}

async fn run_main_logic(
    network_runtime: tokio::runtime::Runtime,
    processing_runtime: tokio::runtime::Runtime,
    settings: Settings,
) -> anyhow::Result<()> {
    // 配置加载与验证
    println!("🔧 Loading configuration...");

    let config_result = config::Config::builder()
        .add_source(config::File::with_name("configs/qingxi").required(true))
        .add_source(config::Environment::with_prefix("QINGXI").separator("__"))
        .build();

    match config_result {
        Ok(config) => {
            println!("✅ Raw config loaded successfully");

            match config.try_deserialize::<Settings>() {
                Ok(settings) => {
                    println!("✅ Settings deserialized successfully");
                    println!(
                        "📊 Found {} market sources configured",
                        settings.sources.len()
                    );
                }
                Err(e) => {
                    eprintln!("❌ Failed to deserialize settings: {e}");
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to load raw config: {e}");
            return Err(e.into());
        }
    }

    observability::init_subscriber(&settings.general.log_level, "qingxi-market-data");
    let metrics_addr = settings.get_metrics_address();
    if settings.general.metrics_enabled {
        // 初始化指标注册表
        let _registry = observability::init_metrics();
        info!("Metrics registry initialized at {}", metrics_addr);
    }

    // 健康检查和关闭信号系统初始化
    let (readiness_tx, readiness_rx) = tokio::sync::watch::channel(false);
    let health_probe_addr = settings.get_health_address().parse()?;
    observability::start_health_probe_server(health_probe_addr, Arc::new(readiness_rx.clone()));

    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

    // 创建中央管理器
    let (manager, manager_handle) = CentralManager::new(&settings);

    // 注册交易所适配器 - 配置驱动方式
    let enabled_exchanges: Vec<String> = settings
        .sources
        .iter()
        .filter(|s| s.enabled)
        .map(|s| s.exchange_id.clone())
        .collect();

    info!("📋 Enabled exchanges from configuration: {:?}", enabled_exchanges);

    for exchange_id in &enabled_exchanges {
        match exchange_id.as_str() {
            "binance" => {
                if let Some(config) = settings.sources.iter().find(|s| s.exchange_id == "binance") {
                    manager.register_adapter(Arc::new(BinanceAdapter::new_with_config(config)));
                }
            },
            "okx" => {
                if let Some(config) = settings.sources.iter().find(|s| s.exchange_id == "okx") {
                    manager.register_adapter(Arc::new(OkxAdapter::new_with_config(config)));
                }
            },
            "huobi" => {
                if let Some(config) = settings.sources.iter().find(|s| s.exchange_id == "huobi") {
                    manager.register_adapter(Arc::new(HuobiAdapter::new_with_config(config)));
                }
            },
            _ => {
                warn!("Unknown exchange adapter: {}", exchange_id);
            }
        }
    }

    info!("✅ Registered exchange adapters: {:?}", enabled_exchanges);

    // Clone the handle for API server
    let api_manager_handle = manager_handle.clone();

    let mut tasks = tokio::task::JoinSet::new();

    // 启动API服务器（gRPC API）
    let api_addr = settings.get_api_address().parse()?;
    tasks.spawn(async move {
        if let Err(e) = api_server::serve_grpc_api(api_addr, api_manager_handle).await {
            error!("gRPC API server failed: {}", e);
        }
    });

    // 启动HTTP API服务器 
    let http_addr = settings.get_http_address().parse()?;
    let http_manager_handle = manager_handle.clone();
    let health_monitor = manager.health_monitor();
    tasks.spawn(async move {
        if let Err(e) = http_api::serve_http_api(http_addr, http_manager_handle, health_monitor).await {
            error!("HTTP API server failed: {}", e);
        }
    });

    // 启动中央管理器
    let manager_shutdown_rx = shutdown_tx.subscribe();
    let manager_readiness_tx = readiness_tx.clone();
    tasks.spawn(async move {
        if let Err(e) = manager.run(manager_readiness_tx, manager_shutdown_rx).await {
            error!("Central manager failed: {}", e);
        }
    });

    // 系统性能监控任务 - 使用配置中的间隔
    let perf_manager_handle = manager_handle.clone();
    tasks.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(settings.performance.performance_stats_interval_sec));
        loop {
            interval.tick().await;
            
            // 获取系统统计信息
            match perf_manager_handle.get_performance_stats().await {
                Ok(stats) => {
                    info!("🚀 PERFORMANCE OPTIMIZATION STATUS:");
                    info!("   📊 Active orderbooks: {}", stats.orderbook_count);
                    info!("   ⚡ Batch processed items: {}", stats.batch_processed_count);
                    info!("   💾 Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
                    info!("   🔓 Lock-free buffer usage: {:.1}%", stats.lockfree_buffer_usage);
                    info!("   🧮 SIMD operations: {}", stats.simd_operations_count);
                    info!("   📦 Compression ratio: {:.2}x", stats.compression_ratio);
                    
                    // 获取订单簿分布
                    match perf_manager_handle.get_all_orderbooks().await {
                        Ok(orderbooks) => {
                            let mut exchange_counts = std::collections::HashMap::new();
                            for (symbol, _) in &orderbooks {
                                let symbol_pair = symbol.as_pair();
                                let parts: Vec<&str> = symbol_pair.split('-').collect();
                                let exchange = if parts.len() > 1 { parts[1].to_string() } else { "unknown".to_string() };
                                *exchange_counts.entry(exchange).or_insert(0) += 1;
                            }
                            
                            info!("   📈 Exchange distribution:");
                            for (exchange, count) in exchange_counts {
                                info!("     🏢 {}: {} symbols", exchange, count);
                            }
                        }
                        Err(e) => {
                            error!("Failed to get orderbook distribution: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get performance statistics: {}", e);
                }
            }
        }
    });

    info!("Waiting for system to become ready...");
    let mut readiness_check_rx = readiness_rx.clone();
    if tokio::time::timeout(
        Duration::from_secs(settings.performance.system_readiness_timeout_sec),
        readiness_check_rx.wait_for(|ready| *ready),
    )
    .await
    .is_err()
    {
        error!("System did not become ready within {} seconds. Shutting down.", settings.performance.system_readiness_timeout_sec);
        let _ = shutdown_tx.send(());
    } else {
        info!("System is ready. Starting API servers...");

        // 等待任务完成或关闭信号
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received");
                let _ = shutdown_tx.send(());
            }
            Some(res) = tasks.join_next() => {
                match res {
                    Ok(_) => info!("Task completed successfully"),
                    Err(e) => error!("Task failed: {}", e),
                }
            }
        }
    }

    info!("Shutting down gracefully...");
    Ok(())
}
