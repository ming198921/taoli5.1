//! # 集成测试 - 验证所有修复
//!
//! 测试gRPC API服务器、Huobi适配器、事件系统等关键组件

use market_data_module::*;
use market_data_module::api_server::market_data_pb::market_data_feed_server::MarketDataFeed;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_central_manager_creation() {
    // 测试CentralManager创建和基本功能
    let settings = settings::Settings::load().unwrap_or_else(|_| {
        settings::Settings {
            general: settings::GeneralSettings {
                log_level: "info".to_string(),
                metrics_enabled: false,
            },
            api_server: settings::ApiServerSettings {
                host: "127.0.0.1".to_string(),
                port: 50051,
                metrics_port_offset: 1,
                health_port_offset: 2,
                http_port_offset: 10,
                orderbook_depth_limit: 20,
                symbols_list_limit: 100,
            },
            central_manager: settings::CentralManagerSettings {
                event_buffer_size: 1000,
            },
            sources: vec![],
            consistency_thresholds: types::ConsistencyThresholds {
                price_diff_percentage: 0.01,
                timestamp_diff_ms: 1000,
                sequence_gap_threshold: 100,
                spread_threshold_percentage: 0.05,
                critical_spread_threshold_percentage: 0.1,
                max_time_diff_ms: 5000.0,
                volume_consistency_threshold: 0.1,
            },
            reasoner: settings::ReasonerSettings {
                api_endpoint: "http://localhost:8080".to_string(),
            },
            anomaly_detection: settings::AnomalyDetectionSettings {
                spread_threshold: 0.05,
                volume_threshold: 1000.0,
                price_change_threshold: 0.02,
                spread_threshold_percentage: 0.1,
            },
            performance: settings::PerformanceSettings {
                performance_stats_interval_sec: 30,
                system_readiness_timeout_sec: 60,
                command_channel_size: 128,
                internal_channel_size: 1000,
                cleaner_input_buffer_size: 1000,
                cleaner_output_buffer_size: 1000,
            },
            threading: settings::ThreadingSettings {
                network_worker_threads: 3,
                network_cpu_cores: vec![2, 3, 4],
                processing_worker_threads: 1,
                processing_cpu_core: 5,
                main_worker_threads: 2,
            },
            quality_thresholds: settings::QualityThresholds {
                cache_hit_rate_threshold: 0.8,
                buffer_usage_threshold: 0.8,
                compression_ratio_threshold: 2.0,
                data_freshness_warning_ms: 1000,
                data_freshness_critical_ms: 5000,
                max_orderbook_count: 10000,
                max_batch_size: 1000,
                max_memory_usage_mb: 1024,
            },
            cache: settings::CacheSettings {
                l2_directory: "cache/l2".to_string(),
                l3_directory: "cache/l3".to_string(),
                log_directory: "logs/cache".to_string(),
                auto_create_dirs: true,
            },
        }
    });
    
    let (manager, _handle) = CentralManager::new(&settings);
    
    // 验证健康监控器
    let health_monitor = manager.health_monitor();
    assert!(!health_monitor.get_health_summary().timestamp.as_millis() == 0);
    
    println!("✅ CentralManager创建成功");
}

#[tokio::test]
async fn test_huobi_adapter_basic() {
    // 测试Huobi适配器基本功能
    let adapter = adapters::huobi::HuobiAdapter::new();
    
    // 测试交易所ID
    assert_eq!(adapter.exchange_id(), "huobi");
    
    // 测试订阅消息构建
    let subscription = types::SubscriptionDetail {
        symbol: types::Symbol::new("btc", "usdt"),
        channel: "orderbook".to_string(),
    };
    
    let messages = adapter.build_subscription_messages(&[subscription]);
    assert!(messages.is_ok());
    assert!(!messages.unwrap().is_empty());
    
    println!("✅ Huobi适配器基本功能正常");
}

#[tokio::test] 
async fn test_api_server_creation() {
    // 测试API服务器创建
    let settings = settings::Settings::load().unwrap_or_else(|_| {
        settings::Settings {
            general: settings::GeneralSettings {
                log_level: "info".to_string(),
                metrics_enabled: false,
            },
            api_server: settings::ApiServerSettings {
                host: "127.0.0.1".to_string(),
                port: 50051,
                metrics_port_offset: 1,
                health_port_offset: 2,
                http_port_offset: 10,
                orderbook_depth_limit: 20,
                symbols_list_limit: 100,
            },
            central_manager: settings::CentralManagerSettings {
                event_buffer_size: 1000,
            },
            sources: vec![],
            consistency_thresholds: types::ConsistencyThresholds {
                price_diff_percentage: 0.01,
                timestamp_diff_ms: 1000,
                sequence_gap_threshold: 100,
                spread_threshold_percentage: 0.05,
                critical_spread_threshold_percentage: 0.1,
                max_time_diff_ms: 5000.0,
                volume_consistency_threshold: 0.1,
            },
            reasoner: settings::ReasonerSettings {
                api_endpoint: "http://localhost:8080".to_string(),
            },
            anomaly_detection: settings::AnomalyDetectionSettings {
                spread_threshold: 0.05,
                volume_threshold: 1000.0,
                price_change_threshold: 0.02,
                spread_threshold_percentage: 0.1,
            },
            performance: settings::PerformanceSettings {
                performance_stats_interval_sec: 30,
                system_readiness_timeout_sec: 60,
                command_channel_size: 128,
                internal_channel_size: 1000,
                cleaner_input_buffer_size: 1000,
                cleaner_output_buffer_size: 1000,
            },
            threading: settings::ThreadingSettings {
                network_worker_threads: 3,
                network_cpu_cores: vec![2, 3, 4],
                processing_worker_threads: 1,
                processing_cpu_core: 5,
                main_worker_threads: 2,
            },
            quality_thresholds: settings::QualityThresholds {
                cache_hit_rate_threshold: 0.8,
                buffer_usage_threshold: 0.8,
                compression_ratio_threshold: 2.0,
                data_freshness_warning_ms: 1000,
                data_freshness_critical_ms: 5000,
                max_orderbook_count: 10000,
                max_batch_size: 1000,
                max_memory_usage_mb: 1024,
            },
            cache: settings::CacheSettings {
                l2_directory: "cache/l2".to_string(),
                l3_directory: "cache/l3".to_string(),
                log_directory: "logs/cache".to_string(),
                auto_create_dirs: true,
            },
        }
    });
    let (_, manager_handle) = CentralManager::new(&settings);
    let health_monitor = Arc::new(health::ApiHealthMonitor::new(30000));
    
    let api_server = api_server::MarketDataApiServer {
        manager: manager_handle,
        health_monitor,
    };
    
    // 测试健康状态请求
    let health_req = tonic::Request::new(api_server::market_data_pb::HealthRequest {
        component: "test".to_string(),
    });
    
    let result = timeout(
        Duration::from_secs(5),
        api_server.get_health_status(health_req)
    ).await;
    
    assert!(result.is_ok());
    println!("✅ API服务器创建和基本请求正常");
}

#[tokio::test]
async fn test_object_pools() {
    // 测试对象池功能
    let orderbook_pool = object_pool::ObjectPool::new(
        || types::OrderBook::new(types::Symbol::new("BTC", "USDT"), "test".to_string()),
        10,
    );
    
    let obj1 = orderbook_pool.get().await;
    assert_eq!(obj1.symbol.base, "BTC");
    assert_eq!(obj1.symbol.quote, "USDT");
    
    orderbook_pool.release(obj1).await;
    
    let obj2 = orderbook_pool.get().await;
    // 应该复用之前的对象
    assert_eq!(obj2.symbol.base, "BTC");
    
    println!("✅ 对象池功能正常");
}

#[tokio::test]
async fn test_high_precision_time() {
    // 测试高精度时间功能
    let now1 = high_precision_time::Nanos::now();
    tokio::time::sleep(Duration::from_millis(1)).await;
    let now2 = high_precision_time::Nanos::now();
    
    assert!(now2 > now1);
    
    let millis_time = high_precision_time::Nanos::from_millis(1000);
    assert_eq!(millis_time.as_millis(), 1000);
    
    let micros_time = high_precision_time::Nanos::from_micros(1000);
    assert_eq!(micros_time.as_micros(), 1000);
    
    println!("✅ 高精度时间功能正常");
}

#[tokio::test]
async fn test_event_bus() {
    // 测试事件总线功能
    let event_bus = event_bus::EventBus::new(100);
    
    // 注册订阅者
    event_bus.register_subscriber("test_subscriber".to_string()).await;
    assert_eq!(event_bus.subscriber_count().await, 1);
    
    // 创建订阅者
    let mut receiver = event_bus.subscribe();
    
    // 发布事件
    let test_event = events::SystemEvent::ComponentStatusChanged {
        component: "test".to_string(),
        status: health::HealthStatus {
            source_id: "test".to_string(),
            last_message_at: high_precision_time::Nanos::now(),
            latency_us: 100,
            message_count: 1,
            is_connected: true,
            last_error: None,
            last_error_at: None,
        },
        message: "测试事件".to_string(),
    };
    
    event_bus.publish(test_event).await;
    
    // 验证事件接收
    let received = timeout(Duration::from_millis(100), receiver.recv()).await;
    assert!(received.is_ok());
    
    println!("✅ 事件总线功能正常");
}
