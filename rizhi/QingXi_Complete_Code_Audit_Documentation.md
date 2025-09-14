---

## 📊 项目概览

### 📁 项目结构
```
qingxi/
├── Cargo.toml                          # 主项目配置
├── config.toml                         # 统一系统配置
├── build.rs                           # 构建脚本
├── README.md                          # 项目说明
├── src/                               # 源代码目录
│   ├── main.rs                        # 主程序入口
│   ├── lib.rs                         # 库入口
│   ├── types.rs                       # 核心类型定义
│   ├── settings.rs                    # 配置管理
│   ├── central_manager.rs             # 中央管理器
│   ├── http_api.rs                    # HTTP API服务
│   ├── adapters/                      # 交易所适配器
│   │   ├── mod.rs                     # 适配器模块
│   │   ├── binance.rs                 # 币安适配器
│   │   ├── bybit.rs                   # Bybit适配器
│   │   ├── huobi.rs                   # 火币适配器
│   │   ├── okx.rs                     # OKX适配器
│   │   └── gateio.rs                  # Gate.io适配器
│   ├── memory/                        # 内存管理模块
│   │   ├── mod.rs
│   │   ├── advanced_allocator.rs      # 高级内存分配器
│   │   └── zero_allocation_engine.rs  # 零分配引擎
│   ├── cleaner/                       # 数据清洗模块
│   │   ├── mod.rs
│   │   ├── optimized_cleaner.rs       # 优化清洗器
│   │   └── simd_orderbook.rs          # SIMD订单簿处理
│   ├── collector/                     # 数据采集模块
│   │   ├── mod.rs
│   │   ├── market_collector_system.rs # 市场数据采集系统
│   │   └── websocket_collector.rs     # WebSocket采集器
│   └── bin/                           # 二进制工具
│       ├── config_validator.rs        # 配置验证工具
│       ├── http_api_test.rs           # API测试工具
│       └── v3_ultra_benchmark.rs      # 性能基准测试
├── configs/                           # 配置文件目录
│   ├── production.toml                # 生产环境配置
│   ├── qingxi.toml                    # 标准配置
│   └── four_exchanges_simple.toml     # 简化配置
├── tests/                             # 测试代码
│   ├── integration_test.rs            # 集成测试
│   └── config_parsing_test.rs         # 配置解析测试
└── examples/                          # 示例代码
    └── health_and_anomaly_demo.rs     # 健康监控示例
```

---

## 🚀 核心源代码

### 1. 主项目配置 (Cargo.toml)

```toml
[package]
name = "market_data_module"
version = "1.0.1"
edition = "2021"
authors = ["Qingxi Performance Team <dev@qingxi.tech>"]
description = "Production-Grade High-Performance Market Data Collection and Consistency Module"

# 🚀 阶段2优化：启用AVX-512和内联汇编支持
[features]
default = ["avx512"]
avx512 = []
nightly = []

[lib]
name = "market_data_module"
path = "src/lib.rs"

# 二进制目标配置 - 生成可执行文件
[[bin]]
name = "market_data_module"
path = "src/main.rs"

[[bin]]
name = "config_validator"
path = "src/bin/config_validator.rs"

[dependencies]
tokio = { version = "1.38", features = ["full"] }
tokio-tungstenite = { version = "0.23", features = ["native-tls"] }
futures-util = "0.3"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tonic = "0.12"
prost = "0.13"
tokio-stream = "0.1"
hyper = { version = "0.14", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
simd-json = "0.13"
ordered-float = { version = "4.2", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.24"
opentelemetry = { version = "0.23" }
opentelemetry-otlp = { version = "0.16", features = ["tonic"] }
metrics = "0.23"
metrics-exporter-prometheus = "0.15"
# 基础依赖
dashmap = "5.5"
bytes = "1.4"
chrono = { version = "0.4", features = ["serde"] }
config = { version = "0.13", features = ["toml"] }
flate2 = "1.0"
flume = "0.11"
toml = "0.8"
log = "0.4"
# 🚀 阶段1性能优化依赖
pdqsort = "1.0"
rayon = "1.7"
crossbeam = "0.8"
# 🚀 未来阶段优化依赖
smallvec = "1.11"
memmap2 = "0.9"
tempfile = "3.8"
rand = "0.8"
num_cpus = "1.16"
core_affinity = "0.8"
fast-float = "0.2"
nom = "7.1"
warp = "0.3"
prometheus = "0.13"
once_cell = "1.19"
url = "2.5"
crossbeam-epoch = "0.9"
crossbeam-utils = "0.8"
bincode = "1.3"
libc = "0.2"
# 🚀 V3.0高级内存管理依赖
lazy_static = "1.4"
parking_lot = "0.12"
regex = "1.11.1"

[build-dependencies]
tonic-build = "0.12"
prost-build = "0.13"
```

---

### 2. 统一系统配置 (configs/qingxi.toml)

```toml
[general]
log_level = "info"
metrics_enabled = true

[api_server]
host = "127.0.0.1"  # 生产环境建议使用具体IP或内网IP
port = 50051
metrics_port_offset = 1
health_port_offset = 2
http_port_offset = 10
orderbook_depth_limit = 20
symbols_list_limit = 100

[central_manager]
event_buffer_size = 1000

[[sources]]
id = "binance_spot"
adapter_type = "binance"
enabled = true
exchange_id = "binance"
symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "SOLUSDT", "XRPUSDT", "DOTUSDT", "MATICUSDT", "AVAXUSDT", "LTCUSDT", "UNIUSDT", "LINKUSDT", "ATOMUSDT", "XLMUSDT", "VETUSDT", "FILUSDT", "TRXUSDT", "EOSUSDT", "XMRUSDT", "NEOUSDT", "DASHUSDT", "IOTAUSDT", "ALGOUSDT", "ZECUSDT", "COMPUSDT", "YFIUSDT", "MKRUSDT", "AAVEUSDT", "SUSHIUSDT", "SNXUSDT", "CRVUSDT", "1INCHUSDT", "BALUSDT", "RENUSDT", "KNCUSDT", "ZRXUSDT", "UMAUSDT", "BANDUSDT", "ALPHAUSDT", "REEFUSDT", "OCEANUSDT", "INJUSDT", "AUDIOUSDT", "CTSIUSDT", "AKROUSDT", "RAYUSDT", "SRMUSDT", "FIDAUSDT", "OOKIUSDT", "SPELLUSDT", "GALAUSDT", "MANAUSDT", "SANDUSDT", "APEUSDT", "LRCUSDT", "ENJUSDT", "CHZUSDT", "BATUSDT", "ZILUSDT", "HOTUSDT", "ICXUSDT", "QTUMUSDT", "LSKUSDT", "SCUSDT", "ZENUSDT", "WAVESUSDT", "KMDUSDT", "ARKUSDT", "STRATUSDT", "BNTUSDT", "STORJUSDT", "ANTUSDT", "OMGUSDT", "GASUSDT", "POWRUSDT", "SUBUSDT", "ENGUSDT", "SALTUSDT", "FUNUSDT", "REQUSDT", "VIBUSDT", "POEUSDT", "FUELUSDT", "MTLUSDT", "DNTUSDT", "ASTUSDT", "TNBUSDT", "DLTUSDT", "AMBUSDT", "BCPTUSDT", "ARNUSDT", "GVTUSDT", "CDTUSDT", "GXSUSDT", "POAUSDT", "QSPUSDT"]
ws_endpoint = "wss://stream.binance.com:9443/ws/"
rest_url = "https://api.binance.com"
channel = "depth20@100ms"

[[sources]]
id = "huobi_spot"
adapter_type = "huobi"
enabled = true
exchange_id = "huobi"
symbols = ["BTCUSDT", "ETHUSDT", "ADAUSDT", "SOLUSDT", "XRPUSDT", "DOTUSDT", "MATICUSDT", "AVAXUSDT", "LTCUSDT", "UNIUSDT", "LINKUSDT", "ATOMUSDT", "XLMUSDT", "VETUSDT", "FILUSDT", "TRXUSDT", "EOSUSDT", "XMRUSDT", "NEOUSDT", "DASHUSDT", "IOTAUSDT", "ALGOUSDT", "ZECUSDT", "COMPUSDT", "YFIUSDT", "MKRUSDT", "AAVEUSDT", "SUSHIUSDT", "SNXUSDT", "CRVUSDT", "1INCHUSDT", "BALUSDT", "RENUSDT", "KNCUSDT", "ZRXUSDT", "UMAUSDT", "BANDUSDT", "ALPHAUSDT", "REEFUSDT", "OCEANUSDT", "INJUSDT", "AUDIOUSDT", "CTSIUSDT", "AKROUSDT", "RAYUSDT", "SRMUSDT", "FIDAUSDT", "OOKIUSDT", "SPELLUSDT", "GALAUSDT", "MANAUSDT", "SANDUSDT", "APEUSDT", "LRCUSDT", "ENJUSDT", "CHZUSDT", "BATUSDT", "ZILUSDT", "HOTUSDT", "ICXUSDT", "QTUMUSDT", "LSKUSDT", "SCUSDT", "ZENUSDT", "WAVESUSDT", "KMDUSDT", "ARKUSDT", "STRATUSDT", "BNTUSDT", "STORJUSDT", "ANTUSDT", "OMGUSDT", "GASUSDT", "POWRUSDT", "SUBUSDT", "ENGUSDT", "SALTUSDT", "FUNUSDT", "REQUSDT", "VIBUSDT", "POEUSDT", "FUELUSDT", "MTLUSDT", "DNTUSDT", "ASTUSDT", "TNBUSDT", "DLTUSDT", "AMBUSDT", "BCPTUSDT", "ARNUSDT", "GVTUSDT", "CDTUSDT", "GXSUSDT", "POAUSDT", "QSPUSDT"]
ws_endpoint = "wss://api.huobi.pro/ws"
rest_url = "https://api.huobi.pro"
channel = "market.depth.step0"

[[sources]]
id = "okx_spot"
adapter_type = "okx"
enabled = true
exchange_id = "okx"
symbols = ["BTC-USDT", "ETH-USDT", "ADA-USDT", "SOL-USDT", "XRP-USDT", "DOT-USDT", "MATIC-USDT", "AVAX-USDT", "LTC-USDT", "UNI-USDT", "LINK-USDT", "ATOM-USDT", "XLM-USDT", "VET-USDT", "FIL-USDT", "TRX-USDT", "EOS-USDT", "XMR-USDT", "NEO-USDT", "DASH-USDT", "IOTA-USDT", "ALGO-USDT", "ZEC-USDT", "COMP-USDT", "YFI-USDT", "MKR-USDT", "AAVE-USDT", "SUSHI-USDT", "SNX-USDT", "CRV-USDT", "1INCH-USDT", "BAL-USDT", "REN-USDT", "KNC-USDT", "ZRX-USDT", "UMA-USDT", "BAND-USDT", "ALPHA-USDT", "REEF-USDT", "OCEAN-USDT", "INJ-USDT", "AUDIO-USDT", "CTSI-USDT", "AKRO-USDT", "RAY-USDT", "SRM-USDT", "FIDA-USDT", "OOKI-USDT", "SPELL-USDT", "GALA-USDT", "MANA-USDT", "SAND-USDT", "APE-USDT", "LRC-USDT", "ENJ-USDT", "CHZ-USDT", "BAT-USDT", "ZIL-USDT", "HOT-USDT", "ICX-USDT", "QTUM-USDT", "LSK-USDT", "SC-USDT", "ZEN-USDT", "WAVES-USDT", "KMD-USDT", "ARK-USDT", "STRAT-USDT", "BNT-USDT", "STORJ-USDT", "ANT-USDT", "OMG-USDT", "GAS-USDT", "POWR-USDT", "SUB-USDT", "ENG-USDT", "SALT-USDT", "FUN-USDT", "REQ-USDT", "VIB-USDT", "POE-USDT", "FUEL-USDT", "MTL-USDT", "DNT-USDT", "AST-USDT", "TNB-USDT", "DLT-USDT", "AMB-USDT", "BCPT-USDT", "ARN-USDT", "GVT-USDT", "CDT-USDT", "GXS-USDT", "POA-USDT", "QSP-USDT"]
ws_endpoint = "wss://ws.okx.com:8443/ws/v5/public"
rest_url = "https://www.okx.com"
channel = "books5"

[[sources]]
id = "kucoin_spot"
adapter_type = "kucoin"
enabled = true
exchange_id = "kucoin"
symbols = ["BTC-USDT", "ETH-USDT", "ADA-USDT", "SOL-USDT", "XRP-USDT", "DOT-USDT", "MATIC-USDT", "AVAX-USDT", "LTC-USDT", "UNI-USDT", "LINK-USDT", "ATOM-USDT", "XLM-USDT", "VET-USDT", "FIL-USDT", "TRX-USDT", "EOS-USDT", "XMR-USDT", "NEO-USDT", "DASH-USDT", "IOTA-USDT", "ALGO-USDT", "ZEC-USDT", "COMP-USDT", "YFI-USDT", "MKR-USDT", "AAVE-USDT", "SUSHI-USDT", "SNX-USDT", "CRV-USDT", "1INCH-USDT", "BAL-USDT", "REN-USDT", "KNC-USDT", "ZRX-USDT", "UMA-USDT", "BAND-USDT", "ALPHA-USDT", "REEF-USDT", "OCEAN-USDT", "INJ-USDT", "AUDIO-USDT", "CTSI-USDT", "AKRO-USDT", "RAY-USDT", "SRM-USDT", "FIDA-USDT", "OOKI-USDT", "SPELL-USDT", "GALA-USDT"]
ws_endpoint = "wss://ws-api.kucoin.com/endpoint"
rest_url = "https://api.kucoin.com"
channel = "level2"

[[sources]]
id = "coinbase_spot"
adapter_type = "coinbase"
enabled = true
exchange_id = "coinbase"
symbols = ["BTC-USD", "ETH-USD", "ADA-USD", "SOL-USD", "XRP-USD", "DOT-USD", "MATIC-USD", "AVAX-USD", "LTC-USD", "UNI-USD", "LINK-USD", "ATOM-USD", "XLM-USD", "VET-USD", "FIL-USD", "TRX-USD", "EOS-USD", "XMR-USD", "NEO-USD", "DASH-USD", "IOTA-USD", "ALGO-USD", "ZEC-USD", "COMP-USD", "YFI-USD", "MKR-USD", "AAVE-USD", "SUSHI-USD", "SNX-USD", "CRV-USD", "1INCH-USD", "BAL-USD", "REN-USD", "KNC-USD", "ZRX-USD", "UMA-USD", "BAND-USD", "ALPHA-USD", "REEF-USD", "OCEAN-USD", "INJ-USD", "AUDIO-USD", "CTSI-USD", "AKRO-USD", "RAY-USD", "SRM-USD", "FIDA-USD", "OOKI-USD", "SPELL-USD", "GALA-USD"]
ws_endpoint = "wss://ws-feed.exchange.coinbase.com"
rest_url = "https://api.exchange.coinbase.com"
channel = "level2"

[[sources]]
id = "bybit_spot"
adapter_type = "bybit"
enabled = true
exchange_id = "bybit"
symbols = ["BTCUSDT", "ETHUSDT", "ADAUSDT", "SOLUSDT", "XRPUSDT", "DOTUSDT", "MATICUSDT", "AVAXUSDT", "LTCUSDT", "UNIUSDT", "LINKUSDT", "ATOMUSDT", "XLMUSDT", "VETUSDT", "FILUSDT", "TRXUSDT", "EOSUSDT", "XMRUSDT", "NEOUSDT", "DASHUSDT", "IOTAUSDT", "ALGOUSDT", "ZECUSDT", "COMPUSDT", "YFIUSDT", "MKRUSDT", "AAVEUSDT", "SUSHIUSDT", "SNXUSDT", "CRVUSDT", "1INCHUSDT", "BALUSDT", "RENUSDT", "KNCUSDT", "ZRXUSDT", "UMAUSDT", "BANDUSDT", "ALPHAUSDT", "REEFUSDT", "OCEANUSDT", "INJUSDT", "AUDIOUSDT", "CTSIUSDT", "AKROUSDT", "RAYUSDT", "SRMUSDT", "FIDAUSDT", "OOKIUSDT", "SPELLUSDT", "GALAUSDT", "MANAUSDT", "SANDUSDT", "APEUSDT", "LRCUSDT", "ENJUSDT", "CHZUSDT", "BATUSDT", "ZILUSDT", "HOTUSDT", "ICXUSDT", "QTUMUSDT", "LSKUSDT", "SCUSDT", "ZENUSDT", "WAVESUSDT", "KMDUSDT", "ARKUSDT", "STRATUSDT", "BNTUSDT", "STORJUSDT", "ANTUSDT", "OMGUSDT", "GASUSDT", "POWRUSDT", "SUBUSDT", "ENGUSDT", "SALTUSDT", "FUNUSDT", "REQUSDT", "VIBUSDT", "POEUSDT", "FUELUSDT", "MTLUSDT", "DNTUSDT", "ASTUSDT", "TNBUSDT", "DLTUSDT", "AMBUSDT", "BCPTUSDT", "ARNUSDT", "GVTUSDT", "CDTUSDT", "GXSUSDT", "POAUSDT", "QSPUSDT"]
ws_endpoint = "wss://stream.bybit.com/v5/public/spot"
rest_url = "https://api.bybit.com"
channel = "orderbook.200"

[consistency_thresholds]
price_diff_percentage = 0.5
timestamp_diff_ms = 5000
sequence_gap_threshold = 10
spread_threshold_percentage = 1.0
critical_spread_threshold_percentage = 2.0
max_time_diff_ms = 10000.0
volume_consistency_threshold = 0.5

[reasoner]
api_endpoint = "http://reasoner-service:8081"

[anomaly_detection]
spread_threshold = 2.0
volume_threshold = 100.0
price_change_threshold = 5.0
spread_threshold_percentage = 1.0

[performance]
enable_batch_processing = true
batch_size = 10000
batch_timeout_ms = 50
enable_simd = true
enable_parallel_processing = true
max_concurrent_tasks = 16
memory_pool_size = 2097152
enable_zero_copy = true
performance_stats_interval_sec = 10
system_readiness_timeout_sec = 30

[threading]
num_worker_threads = 8
enable_cpu_affinity = true
preferred_cores = [0, 1, 2, 3, 4, 5, 6, 7]
enable_numa_awareness = true
network_worker_threads = 6
processing_worker_threads = 4
main_worker_threads = 2

[quality_thresholds]
minimum_data_freshness_ms = 1000
maximum_latency_ms = 100
minimum_completeness_ratio = 0.95
maximum_error_rate = 0.01
cache_hit_rate_threshold = 0.8
buffer_usage_threshold = 0.8
compression_ratio_threshold = 2.0

# 新增缓存配置段
[cache]
l2_directory = "cache/l2"
l3_directory = "cache/l3" 
log_directory = "logs"
auto_create_dirs = true

# 新增配置段 - 解决硬编码问题
[exchanges]
bybit_orderbook_depth = 60
binance_orderbook_depth = 100
event_buffer_size = 5000

[memory_pools]
orderbook_entry_pool_size = 1000
trade_update_pool_size = 5000
snapshot_pool_size = 500
f64_buffer_pool_size = 1000
usize_buffer_pool_size = 1000
default_vec_capacity = 100
cleaner_buffer_size = 1000

[algorithm_scoring]
depth_score_max = 30.0
liquidity_score_baseline = 1000.0
liquidity_score_max = 25.0
```

---

### 3. 主程序入口 (src/main.rs)

```rust
#![allow(dead_code)]
//! # qingxi 市场数据服务 - 生产级主程序入口  
//!
//! 基于权威类型系统的高性能多源加密货币市场数据采集、清洗与一致性验证系统主入口。

use market_data_module::{
    // 适配器实现导入
    adapters::{binance::BinanceAdapter, huobi::HuobiAdapter, okx::OkxAdapter, bybit::BybitAdapter, gateio::GateioAdapter},
    // 核心服务模块
    http_api,
    build_info,
    observability,
    settings::Settings,
    // 权威API导入
    CentralManager,
    CentralManagerApi,
    SYSTEM_NAME,
    VERSION,
    // V3.0优化组件
    intel_cpu_optimizer::IntelCpuOptimizer,
    zero_allocation_arch,
    // V3.0 高级内存管理模块
    memory::{init_zero_allocation_system, ZERO_ALLOCATION_ENGINE, benchmark_memory_performance},
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

    // 🚀 V3.0优化系统提前初始化 - 确保硬件优化在系统启动时就激活
    println!("🚀 Initializing V3.0 optimization components...");
    initialize_v3_optimizations_sync();

    // 早期加载配置以获取线程配置
    let settings = Settings::load().unwrap_or_else(|e| {
        eprintln!("⚠️ Failed to load settings, using defaults: {}", e);
        Settings::default()
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

    // 将运行时存储在局部变量中，避免在异步上下文中丢弃
    let network_rt = Arc::new(network_runtime);
    let processing_rt = Arc::new(processing_runtime);
    
    let result = main_runtime.block_on(async { 
        run_main_logic(network_rt.clone(), processing_rt.clone(), settings).await 
    });
    
    // 确保运行时在同步上下文中被丢弃
    main_runtime.shutdown_background();
    
    result
}

async fn run_main_logic(
    _network_runtime: Arc<tokio::runtime::Runtime>,
    _processing_runtime: Arc<tokio::runtime::Runtime>,
    settings: Settings,
) -> anyhow::Result<()> {
    // 🚀 V3.0优化组件初始化 - 在系统启动早期执行
    println!("🚀 Initializing V3.0 optimization components...");
    initialize_v3_optimizations().await?;
    
    // 配置加载与验证
    println!("🔧 Loading configuration...");

    // 支持通过环境变量指定配置文件路径
    let config_path = std::env::var("QINGXI_CONFIG_PATH")
        .unwrap_or_else(|_| "qingxi/configs/qingxi".to_string());

    let config_result = config::Config::builder()
        .add_source(config::File::with_name(&config_path).required(true))
        .add_source(config::Environment::with_prefix("QINGXI").separator("__"))
        .build();

    match config_result {
        Ok(config) => {
            println!("✅ Raw config loaded successfully");

            match config.try_deserialize::<Settings>() {
                Ok(settings_validated) => {
                    println!("✅ Settings deserialized successfully");
                    println!(
                        "📊 Found {} market sources configured",
                        settings_validated.sources.len()
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

    let (shutdown_tx, _shutdown_rx) = broadcast::channel::<()>(1);

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

    // 检查API密钥配置并发出警告
    for config in &settings.sources {
        if config.enabled {
            if !config.has_complete_api_credentials() {
                match config.exchange_id.as_str() {
                    "okx" => {
                        if !config.has_valid_api_key() {
                            warn!("⚠️ WARNING: API Key for OKX is missing or invalid. Some features might be disabled.");
                        }
                        if !config.has_valid_api_secret() {
                            warn!("⚠️ WARNING: API Secret for OKX is missing or invalid. Some features might be disabled.");
                        }
                        if !config.has_valid_api_passphrase() {
                            warn!("⚠️ WARNING: API Passphrase for OKX is missing or invalid. Some features might be disabled.");
                        }
                    },
                    "binance" => {
                        if !config.has_valid_api_key() {
                            warn!("⚠️ WARNING: API Key for Binance is missing or invalid. Some features might be disabled.");
                        }
                        if !config.has_valid_api_secret() {
                            warn!("⚠️ WARNING: API Secret for Binance is missing or invalid. Some features might be disabled.");
                        }
                    },
                    "huobi" => {
                        if !config.has_valid_api_key() {
                            warn!("⚠️ WARNING: API Key for Huobi is missing or invalid. Some features might be disabled.");
                        }
                        if !config.has_valid_api_secret() {
                            warn!("⚠️ WARNING: API Secret for Huobi is missing or invalid. Some features might be disabled.");
                        }
                    },
                    _ => {}
                }
                warn!("⚠️ Exchange {} will operate with limited functionality due to missing API credentials.", config.exchange_id);
            } else {
                info!("✅ Complete API credentials found for {}", config.exchange_id);
            }
        }
    }

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
            "bybit" => {
                if let Some(config) = settings.sources.iter().find(|s| s.exchange_id == "bybit") {
                    manager.register_adapter(Arc::new(BybitAdapter::new_with_config(config)));
                }
            },
            "gateio" => {
                if let Some(config) = settings.sources.iter().find(|s| s.exchange_id == "gateio") {
                    manager.register_adapter(Arc::new(GateioAdapter::new_with_config(config)));
                }
            },
            _ => {
                warn!("Unknown exchange adapter: {}", exchange_id);
            }
        }
    }

    info!("✅ Registered exchange adapters: {:?}", enabled_exchanges);

    let mut tasks = tokio::task::JoinSet::new();

    // 启动HTTP API服务器 
    let http_addr = settings.get_http_address().parse()?;
    let http_manager_handle = manager_handle.clone();
    let health_monitor = manager.health_monitor();
    let api_config = settings.api_server.clone();
    tasks.spawn(async move {
        if let Err(e) = http_api::serve_http_api(http_addr, http_manager_handle, health_monitor, api_config).await {
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

    // 等待中央管理器启动完成
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 🚀 启动市场数据采集器 - 在中央管理器启动后进行配置
    info!("🚀 Starting market data collectors...");
    manager_handle.reconfigure(settings.sources.clone()).await?;
    info!("✅ Market data collectors configuration completed");

    // 系统性能监控任务 - 使用配置中的间隔
    let perf_manager_handle = manager_handle.clone();
    let performance_settings = settings.performance.clone();
    tasks.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(performance_settings.performance_stats_interval_sec));
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

/// 🚀 V3.0优化组件同步初始化 - 在main()函数中提前执行
fn initialize_v3_optimizations_sync() {
    use std::sync::Once;
    static V3_MAIN_INIT: Once = Once::new();
    
    V3_MAIN_INIT.call_once(|| {
        println!("🚀 开始V3.0优化组件系统级初始化");
        
        // 1. 初始化高级内存管理系统
        println!("🧠 初始化高级内存管理系统...");
        init_zero_allocation_system();
        
        // 运行内存基准测试
        benchmark_memory_performance();
        
        // 获取初始内存统计
        let memory_stats = ZERO_ALLOCATION_ENGINE.get_detailed_stats();
        println!("📊 内存系统初始状态:");
        println!("   活跃交易对: {}/{}", memory_stats.active_symbols, memory_stats.total_symbols);
        println!("   内存分配: {:.2} MB", memory_stats.memory_allocated_mb);
        println!("   零分配成功率: {:.2}%", memory_stats.success_rate);
        
        // 2. Intel CPU优化器初始化
        match IntelCpuOptimizer::new() {
            Ok(optimizer) => {
                match optimizer.initialize() {
                    Ok(_) => {
                        println!("✅ Intel CPU优化器初始化成功");
                        
                        // 尝试应用系统级优化 (可能因权限失败)
                        let cpu_count = num_cpus::get();
                        println!("🔧 检测到{}个CPU核心", cpu_count);
                        
                        // 检查系统级CPU优化状态
                        let mut cpu_optimized = false;
                        let mut turbo_optimized = false;
                        
                        // 检查CPU性能模式
                        if std::path::Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").exists() {
                            if let Ok(governor) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
                                if governor.trim() == "performance" {
                                    cpu_optimized = true;
                                }
                            }
                        } else if std::path::Path::new("/sys/devices/system/cpu/intel_pstate/min_perf_pct").exists() {
                            if let Ok(min_perf) = std::fs::read_to_string("/sys/devices/system/cpu/intel_pstate/min_perf_pct") {
                                if min_perf.trim().parse::<i32>().unwrap_or(0) >= 100 {
                                    cpu_optimized = true;
                                }
                            }
                        } else {
                            // 平台控制的情况，检查服务状态
                            if std::process::Command::new("systemctl")
                                .args(&["is-active", "qingxi-cpu-performance.service"])
                                .output()
                                .map(|output| output.status.success())
                                .unwrap_or(false) {
                                cpu_optimized = true;
                            }
                        }
                        
                        // 检查Turbo Boost状态
                        if std::path::Path::new("/sys/devices/system/cpu/intel_pstate/no_turbo").exists() {
                            if let Ok(no_turbo) = std::fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
                                if no_turbo.trim() == "0" {
                                    turbo_optimized = true;
                                }
                            }
                        } else {
                            // 检查turbo boost服务状态
                            if std::process::Command::new("systemctl")
                                .args(&["is-active", "qingxi-turbo-boost.service"])
                                .output()
                                .map(|output| output.status.success())
                                .unwrap_or(false) {
                                turbo_optimized = true;
                            }
                        }
                        
                        // 显示优化状态
                        if cpu_optimized {
                            println!("✅ 系统级CPU性能优化已启用");
                        } else {
                            println!("⚠️ 系统级CPU优化需要特殊权限，跳过");
                        }
                        
                        if turbo_optimized {
                            println!("✅ Turbo Boost已启用");
                        } else {
                            println!("⚠️ Turbo Boost优化需要root权限，跳过");
                        }
                    },
                    Err(e) => {
                        println!("⚠️ Intel CPU优化器初始化失败(将使用通用模式): {}", e);
                    }
                }
            },
            Err(e) => {
                println!("⚠️ 无法创建Intel CPU优化器: {}", e);
            }
        }
        
        // 零分配内存池初始化
        let pool = zero_allocation_arch::get_global_memory_pool();
        match pool.warmup() {
            Ok(_) => {
                println!("✅ 零分配内存池预热完成");
            },
            Err(e) => {
                println!("⚠️ 零分配内存池预热失败: {}", e);
            }
        }
    });
}

/// 🚀 V3.0优化组件异步初始化 - 在系统启动早期执行
async fn initialize_v3_optimizations() -> anyhow::Result<()> {
    // 1. 初始化高级内存管理系统
    println!("🧠 初始化高级内存管理系统...");
    init_zero_allocation_system();
    
    // 运行内存基准测试
    benchmark_memory_performance();
    
    // 获取初始内存统计
    let memory_stats = ZERO_ALLOCATION_ENGINE.get_detailed_stats();
    println!("📊 内存系统初始状态:");
    println!("   活跃交易对: {}/{}", memory_stats.active_symbols, memory_stats.total_symbols);
    println!("   内存分配: {:.2} MB", memory_stats.memory_allocated_mb);
    println!("   零分配成功率: {:.2}%", memory_stats.success_rate);
    
    // 2. Intel CPU优化器初始化
    match IntelCpuOptimizer::new() {
        Ok(optimizer) => {
            match optimizer.initialize() {
                Ok(_) => {
                    println!("✅ Intel CPU优化器初始化成功");
                    
                    // 尝试应用系统级优化 (可能因权限失败)
                    let cpu_count = num_cpus::get();
                    println!("🔧 检测到{}个CPU核心", cpu_count);
                    
                    // 检查系统级CPU优化状态
                    let mut cpu_optimized = false;
                    let mut turbo_optimized = false;
                    
                    // 检查CPU性能模式
                    if std::path::Path::new("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor").exists() {
                        if let Ok(governor) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor") {
                            if governor.trim() == "performance" {
                                cpu_optimized = true;
                            }
                        }
                    } else if std::path::Path::new("/sys/devices/system/cpu/intel_pstate/min_perf_pct").exists() {
                        if let Ok(min_perf) = std::fs::read_to_string("/sys/devices/system/cpu/intel_pstate/min_perf_pct") {
                            if min_perf.trim().parse::<i32>().unwrap_or(0) >= 100 {
                                cpu_optimized = true;
                            }
                        }
                    } else {
                        // 平台控制的情况，检查服务状态
                        if std::process::Command::new("systemctl")
                            .args(&["is-active", "qingxi-cpu-performance.service"])
                            .output()
                            .map(|output| output.status.success())
                            .unwrap_or(false) {
                            cpu_optimized = true;
                        }
                    }
                    
                    // 检查Turbo Boost状态
                    if std::path::Path::new("/sys/devices/system/cpu/intel_pstate/no_turbo").exists() {
                        if let Ok(no_turbo) = std::fs::read_to_string("/sys/devices/system/cpu/intel_pstate/no_turbo") {
                            if no_turbo.trim() == "0" {
                                turbo_optimized = true;
                            }
                        }
                    } else {
                        // 检查turbo boost服务状态
                        if std::process::Command::new("systemctl")
                            .args(&["is-active", "qingxi-turbo-boost.service"])
                            .output()
                            .map(|output| output.status.success())
                            .unwrap_or(false) {
                            turbo_optimized = true;
                        }
                    }
                    
                    // 显示优化状态
                    if cpu_optimized {
                        println!("✅ 系统级CPU性能优化已启用");
                    } else {
                        println!("⚠️ 系统级CPU优化需要特殊权限，跳过");
                    }
                    
                    if turbo_optimized {
                        println!("✅ Turbo Boost已启用");
                    } else {
                        println!("⚠️ Turbo Boost优化需要root权限，跳过");
                    }
                },
                Err(e) => {
                    println!("⚠️ Intel CPU优化器初始化失败(将使用通用模式): {}", e);
                }
            }
        },
        Err(e) => {
            println!("⚠️ 无法创建Intel CPU优化器: {}", e);
        }
    }
    
    // 零分配内存池初始化
    let pool = zero_allocation_arch::get_global_memory_pool();
    match pool.warmup() {
        Ok(_) => {
            println!("✅ 零分配内存池预热完成");
        },
        Err(e) => {
            println!("⚠️ 零分配内存池预热失败: {}", e);
        }
    }
}
```

---

## 5. 核心源代码模块详细实现

### 5.1 核心类型定义 (src/types.rs)

核心类型定义模块提供了系统的权威数据结构，包括Symbol、OrderBook、TradeUpdate等核心类型。所有模块都依赖这些类型定义来确保数据一致性。

```rust
#![allow(dead_code)]
//! # 核心类型定义 - 系统唯一权威来源
//!
//! 本模块定义了整个qingxi系统中使用的所有核心数据类型。
//! 这是系统类型系统的权威定义，所有其他模块必须导入并使用这些类型。

use ordered_float::OrderedFloat;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// 类型别名，用于更好的语义表达
pub type Price = OrderedFloat<f64>;
pub type Quantity = OrderedFloat<f64>;

/// 权威交易对定义
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub base: String,
    pub quote: String,
}

impl Symbol {
    pub fn new(base: &str, quote: &str) -> Self {
        Self {
            base: base.to_uppercase(),
            quote: quote.to_uppercase(),
        }
    }

    /// V3.0 零分配优化：空符号常量
    pub const EMPTY: Self = Self {
        base: String::new(),
        quote: String::new(),
    };

    pub fn as_pair(&self) -> String {
        format!("{}/{}", self.base, self.quote)
    }

    pub fn as_combined(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }

    pub fn from_pair(pair: &str) -> Option<Self> {
        if let Some(separator_pos) = pair.find('/') {
            let base = pair[..separator_pos].to_uppercase();
            let quote = pair[separator_pos + 1..].to_uppercase();
            Some(Symbol { base, quote })
        } else {
            None
        }
    }

    pub fn from_string(s: &str) -> Self {
        // 尝试解析为 BASE/QUOTE 格式
        if let Some(symbol) = Self::from_pair(s) {
            symbol
        } else {
            // 如果没有分隔符，假设前3个字符是base，剩余是quote
            if s.len() >= 6 {
                let base = &s[..3];
                let quote = &s[3..];
                Symbol::new(base, quote)
            } else {
                // 默认处理
                Symbol::new(s, "USDT")
            }
        }
    }

    pub fn to_string(&self) -> String {
        self.as_combined()
    }
}
```

---

### 5.2 配置管理模块 (src/settings.rs)

配置管理模块负责加载和管理整个系统的配置，支持环境变量覆盖和多种配置源。

```rust
#![allow(dead_code)]
// src/settings.rs
//! # Configuration Management Module
use crate::types::{ConsistencyThresholds, MarketSourceConfig};
use serde::Deserialize;

fn default_metrics_port_offset() -> u16 { 1 }
fn default_health_port_offset() -> u16 { 2 }
fn default_http_port_offset() -> u16 { 10 }
fn default_orderbook_depth_limit() -> usize { 10 }
fn default_symbols_list_limit() -> usize { 50 }
fn default_performance_stats_interval() -> u64 { 30 }
fn default_system_readiness_timeout() -> u64 { 60 }
fn default_command_channel_size() -> usize { 128 }
fn default_internal_channel_size() -> usize { 1000 }
fn default_cleaner_buffer_size() -> usize { 1000 }
fn default_network_worker_threads() -> usize { 3 }
fn default_network_cpu_cores() -> Vec<usize> { vec![2, 3, 4] }
fn default_processing_worker_threads() -> usize { 1 }
fn default_processing_cpu_core() -> usize { 5 }
fn default_main_worker_threads() -> usize { 2 }
fn default_cache_hit_rate_threshold() -> f64 { 0.8 }
fn default_buffer_usage_threshold() -> f64 { 0.8 }
fn default_compression_ratio_threshold() -> f64 { 2.0 }
fn default_data_freshness_warning_ms() -> u64 { 1000 }
fn default_data_freshness_critical_ms() -> u64 { 5000 }
fn default_max_orderbook_count() -> usize { 10000 }
fn default_max_batch_size() -> usize { 1000 }
fn default_max_memory_usage_mb() -> usize { 1024 }
fn default_auto_create_dirs() -> bool { true }
fn default_snapshot_pool_size() -> usize { 500 }
fn default_default_vec_capacity() -> usize { 100 }

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub general: GeneralSettings,
    pub api_server: ApiServerSettings,
    pub central_manager: CentralManagerSettings,
    #[serde(default)]
    pub sources: Vec<MarketSourceConfig>,
    pub consistency_thresholds: ConsistencyThresholds,
    pub reasoner: ReasonerSettings,
    pub anomaly_detection: AnomalyDetectionSettings,
    #[serde(default)]
    pub performance: PerformanceSettings,
    #[serde(default)]
    pub threading: ThreadingSettings,
    #[serde(default)]
    pub quality_thresholds: QualityThresholds,
    #[serde(default)]
    pub cache: CacheSettings,
    #[serde(default)]
    pub exchanges: ExchangeSettings,
    #[serde(default)]
    pub api_credentials: ApiCredentialsSettings,
    #[serde(default)]
    pub websocket_network: WebSocketNetworkSettings,
    #[serde(default)]
    pub memory_pools: MemoryPoolSettings,
    #[serde(default)]
    pub algorithm_scoring: AlgorithmScoringSettings,
    #[serde(default)]
    pub memory_allocator: MemoryAllocatorSettings,
    #[serde(default)]
    pub cleaner: CleanerSettings,
    #[serde(default)]
    pub batch: BatchSettings,
    #[serde(default)]
    pub benchmark: BenchmarkSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralSettings {
    pub log_level: String,
    pub metrics_enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ApiServerSettings {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_metrics_port_offset")]
    pub metrics_port_offset: u16,
    #[serde(default = "default_health_port_offset")]
    pub health_port_offset: u16,
    #[serde(default = "default_http_port_offset")]
    pub http_port_offset: u16,
    #[serde(default = "default_orderbook_depth_limit")]
    pub orderbook_depth_limit: usize,
    #[serde(default = "default_symbols_list_limit")]
    pub symbols_list_limit: usize,
}

impl ApiServerSettings {
    /// 获取API服务器绑定地址，支持环境变量覆盖
    pub fn get_host(&self) -> String {
        std::env::var("QINGXI_API_SERVER__HOST")
            .unwrap_or_else(|_| self.host.clone())
    }
    
    /// 获取API服务器端口，支持环境变量覆盖
    pub fn get_port(&self) -> u16 {
        std::env::var("QINGXI_API_SERVER__PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(self.port)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CentralManagerSettings {
    pub event_buffer_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReasonerSettings {
    pub api_endpoint: String,
}

impl ReasonerSettings {
    pub fn get_api_endpoint(&self) -> String {
        std::env::var("QINGXI_REASONER_ENDPOINT")
            .unwrap_or_else(|_| self.api_endpoint.clone())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnomalyDetectionSettings {
    pub spread_threshold: f64,
    pub volume_threshold: f64,
    pub price_change_threshold: f64,
    pub spread_threshold_percentage: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PerformanceSettings {
    #[serde(default = "default_performance_stats_interval")]
    pub performance_stats_interval_sec: u64,
    #[serde(default = "default_system_readiness_timeout")]
    pub system_readiness_timeout_sec: u64,
    #[serde(default = "default_command_channel_size")]
    pub command_channel_size: usize,
    #[serde(default = "default_internal_channel_size")]
    pub internal_channel_size: usize,
    #[serde(default = "default_cleaner_buffer_size")]
    pub cleaner_input_buffer_size: usize,
    #[serde(default = "default_cleaner_buffer_size")]
    pub cleaner_output_buffer_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThreadingSettings {
    #[serde(default = "default_network_worker_threads")]
    pub network_worker_threads: usize,
    #[serde(default = "default_network_cpu_cores")]
    pub network_cpu_cores: Vec<usize>,
    #[serde(default = "default_processing_worker_threads")]
    pub processing_worker_threads: usize,
    #[serde(default = "default_processing_cpu_core")]
    pub processing_cpu_core: usize,
    #[serde(default = "default_main_worker_threads")]
    pub main_worker_threads: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QualityThresholds {
    #[serde(default = "default_cache_hit_rate_threshold")]
    pub cache_hit_rate_threshold: f64,
    #[serde(default = "default_buffer_usage_threshold")]
    pub buffer_usage_threshold: f64,
    #[serde(default = "default_compression_ratio_threshold")]
    pub compression_ratio_threshold: f64,
    #[serde(default = "default_data_freshness_warning_ms")]
    pub data_freshness_warning_ms: u64,
    #[serde(default = "default_data_freshness_critical_ms")]
    pub data_freshness_critical_ms: u64,
    #[serde(default = "default_max_orderbook_count")]
    pub max_orderbook_count: usize,
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    #[serde(default = "default_max_memory_usage_mb")]
    pub max_memory_usage_mb: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CacheSettings {
    pub l2_directory: String,
    pub l3_directory: String,
    pub log_directory: String,
    #[serde(default = "default_auto_create_dirs")]
    pub auto_create_dirs: bool,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config_path =
            std::env::var("QINGXI_CONFIG_PATH").unwrap_or_else(|_| "configs/qingxi".to_string());

        config::Config::builder()
            .add_source(config::File::with_name(&config_path).required(true))
            .add_source(config::Environment::with_prefix("QINGXI").separator("__"))
            .build()?
            .try_deserialize()
    }

    pub fn get_api_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port())
    }

    pub fn get_metrics_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port() + self.api_server.metrics_port_offset)
    }

    pub fn get_health_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port() + self.api_server.health_port_offset)
    }

    pub fn get_http_address(&self) -> String {
        format!("{}:{}", self.api_server.get_host(), self.api_server.get_port() + self.api_server.http_port_offset)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            general: GeneralSettings {
                log_level: "info".to_string(),
                metrics_enabled: true,
            },
            api_server: ApiServerSettings {
                host: std::env::var("QINGXI_API_SERVER__HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: std::env::var("QINGXI_API_SERVER__PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(50051),
                metrics_port_offset: 1,
                health_port_offset: 2,
                http_port_offset: 10,
                orderbook_depth_limit: 20,
                symbols_list_limit: 100,
            },
            api_credentials: ApiCredentialsSettings::default(),
            websocket_network: WebSocketNetworkSettings::default(),
            central_manager: CentralManagerSettings { 
                event_buffer_size: 1000 
            },
            sources: vec![],
            consistency_thresholds: crate::types::ConsistencyThresholds {
                price_diff_percentage: 0.5,
                timestamp_diff_ms: 5000,
                sequence_gap_threshold: 10,
                spread_threshold_percentage: 1.0,
                critical_spread_threshold_percentage: 2.0,
                max_time_diff_ms: 10000.0,
                volume_consistency_threshold: 0.5,
            },
            reasoner: ReasonerSettings {
                api_endpoint: std::env::var("QINGXI_REASONER__API_ENDPOINT")
                    .unwrap_or_else(|_| "http://reasoner-service:8081".to_string()),
            },
            anomaly_detection: AnomalyDetectionSettings {
                spread_threshold: 2.0,
                volume_threshold: 100.0,
                price_change_threshold: 5.0,
                spread_threshold_percentage: 1.0,
            },
            performance: PerformanceSettings::default(),
            threading: ThreadingSettings::default(),
            quality_thresholds: QualityThresholds::default(),
            cache: CacheSettings::default(),
            exchanges: ExchangeSettings::default(),
            memory_pools: MemoryPoolSettings::default(),
            algorithm_scoring: AlgorithmScoringSettings::default(),
            memory_allocator: MemoryAllocatorSettings::default(),
            cleaner: CleanerSettings::default(),
            batch: BatchSettings::default(),
            benchmark: BenchmarkSettings::default(),
        }
    }
}

// ... 其余配置结构体和实现代码 ...
```

---

### 5.3 中央管理器 (src/central_manager.rs)

中央管理器是整个系统的核心协调器，负责管理所有交易所适配器、数据处理管道和性能优化组件。

```rust
#![allow(dead_code)]
// src/central_manager.rs - Final Refactored Version with Performance Optimizations
use super::{
    adapters::ExchangeAdapter, collector::market_collector_system::MarketCollectorSystem,
    errors::*, health::ApiHealthMonitor, pipeline::DataPipeline, reasoner_client::ReasonerClient,
    settings::Settings, types::*, MarketDataMessage,
};
use crate::batch::{BatchConfig, MarketDataBatchProcessor, SIMDBatchProcessor};
use crate::cache::{CacheLevel, MultiLevelCache};
use crate::lockfree::{MarketDataLockFreeBuffer};
use crate::cleaner::{OptimizedDataCleaner, DataCleaner};
use crate::event_bus::EventBus;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, watch};
use tracing::{debug, error, info, warn, instrument};

// 1. 定义与外部世界交互的所有命令 ---
#[derive(Debug)]
pub enum ApiCommand {
    GetLatestOrderbook {
        exchange_id: String,
        symbol: Symbol,
        responder: oneshot::Sender<Result<OrderBook, MarketDataApiError>>,
    },
    GetLatestSnapshot {
        symbol: String,
        responder: oneshot::Sender<Result<MarketDataSnapshot, MarketDataApiError>>,
    },
    GetLatestAnomaly {
        symbol: String,
        responder: oneshot::Sender<Result<AnomalyDetectionResult, MarketDataApiError>>,
    },
    GetAllOrderbooks {
        responder: oneshot::Sender<Result<Vec<(Symbol, OrderBook)>, MarketDataApiError>>,
    },
    GetPerformanceStats {
        responder: oneshot::Sender<Result<PerformanceStats, MarketDataApiError>>,
    },
    StartCollectors {
        responder: oneshot::Sender<Result<(), MarketDataError>>,
    },
    Reconfigure {
        sources: Vec<MarketSourceConfig>,
        responder: oneshot::Sender<Result<(), MarketDataError>>,
    },
}

// 2. 创建轻量级的"句柄"或"遥控器" ---
#[async_trait::async_trait]
pub trait CentralManagerApi: Send + Sync {
    async fn reconfigure(&self, sources: Vec<MarketSourceConfig>) -> Result<(), MarketDataError>;
    async fn get_latest_orderbook(
        &self,
        exchange_id: &str,
        symbol: &Symbol,
    ) -> Result<OrderBook, MarketDataApiError>;
    async fn get_latest_snapshot(&self, symbol: &str) -> Result<MarketDataSnapshot, MarketDataApiError>;
    async fn get_latest_anomaly(&self, symbol: &str) -> Result<AnomalyDetectionResult, MarketDataApiError>;
    async fn get_all_orderbooks(&self) -> Result<Vec<(Symbol, OrderBook)>, MarketDataApiError>;
    async fn get_performance_stats(&self) -> Result<PerformanceStats, MarketDataApiError>;
    async fn start_collectors(&self) -> Result<(), MarketDataError>;
}

#[derive(Clone)]
pub struct CentralManagerHandle {
    command_sender: flume::Sender<ApiCommand>,
    config_sender: tokio::sync::mpsc::UnboundedSender<Vec<MarketSourceConfig>>,
}

#[async_trait::async_trait]
impl CentralManagerApi for CentralManagerHandle {
    async fn reconfigure(&self, sources: Vec<MarketSourceConfig>) -> Result<(), MarketDataError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::Reconfigure {
            sources,
            responder: tx,
        };
        self.command_sender
            .send_async(command)
            .await
            .map_err(|e| MarketDataError::InternalError(e.to_string()))?;
        rx.await
            .map_err(|e| MarketDataError::InternalError(e.to_string()))?
    }

    async fn get_latest_orderbook(
        &self,
        exchange_id: &str,
        symbol: &Symbol,
    ) -> Result<OrderBook, MarketDataApiError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::GetLatestOrderbook {
            exchange_id: exchange_id.to_string(),
            symbol: symbol.clone(),
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataApiError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataApiError::InternalError(e.to_string()))?
    }

    // ... 其余API实现方法 ...

    async fn get_performance_stats(&self) -> Result<PerformanceStats, MarketDataApiError> {
        let (tx, rx) = oneshot::channel();
        let command = ApiCommand::GetPerformanceStats {
            responder: tx,
        };
        if self.command_sender.send_async(command).await.is_err() {
            return Err(MarketDataApiError::InternalError(
                "Command channel closed".to_string(),
            ));
        }
        rx.await
            .map_err(|e| MarketDataApiError::InternalError(e.to_string()))?
    }
}

// 3. 核心状态机 - 增强性能优化组件 ---
pub struct CentralManager {
    command_receiver: flume::Receiver<ApiCommand>,
    data_receiver: flume::Receiver<AdapterEvent>,
    config_receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<MarketSourceConfig>>,
    collector_system: Arc<MarketCollectorSystem>,
    pipeline: DataPipeline,
    reasoner_client: ReasonerClient,
    latest_books: Arc<DashMap<(String, Symbol), OrderBook>>,
    
    // 性能优化组件
    batch_processor: Arc<MarketDataBatchProcessor>,
    simd_processor: Arc<SIMDBatchProcessor>,
    cache_manager: Arc<MultiLevelCache>,
    lockfree_buffer: Arc<MarketDataLockFreeBuffer>,
    
    // 数据清洗组件 - 使用优化清洗器
    data_cleaner: Option<Arc<tokio::sync::Mutex<OptimizedDataCleaner>>>,
    
    snapshot_pool: Arc<crate::object_pool::ObjectPool<MarketDataSnapshot>>,
    orderbook_pool: Arc<crate::object_pool::ObjectPool<OrderBook>>,
    health_monitor: Arc<ApiHealthMonitor>,
    
    // 事件总线系统
    event_bus: EventBus,
}

impl CentralManager {
    pub fn new(settings: &Settings) -> (Self, CentralManagerHandle) {
        let (command_tx, command_rx) = flume::bounded(settings.performance.command_channel_size);
        let (data_tx, data_rx) = flume::bounded(settings.central_manager.event_buffer_size);
        let (config_tx, config_rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = CentralManagerHandle {
            command_sender: command_tx,
            config_sender: config_tx,
        };

        // 创建性能优化组件
        let batch_config = BatchConfig {
            max_batch_size: settings.quality_thresholds.max_batch_size,
            max_wait_time: std::time::Duration::from_millis(10),
            concurrency: 4,
            enable_compression: true,
        };
        
        let batch_processor = Arc::new(MarketDataBatchProcessor::new(batch_config.clone()));
        let simd_processor = Arc::new(SIMDBatchProcessor::new(batch_config));
        
        // 初始化多级缓存系统
        let cache_manager = Arc::new(MultiLevelCache::new_detailed(
            settings.quality_thresholds.max_orderbook_count,
            std::time::Duration::from_secs(3600),
            std::path::PathBuf::from(&settings.cache.l2_directory),
            1024,
            std::time::Duration::from_secs(7200),
        ).expect("Failed to create multi-level cache"));

        // 初始化无锁缓冲区
        let lockfree_buffer = Arc::new(MarketDataLockFreeBuffer::new(
            settings.quality_thresholds.max_orderbook_count
        ));

        // 创建数据清洗器
        let (_cleaner_input_tx, cleaner_input_rx) = flume::bounded(
            settings.performance.cleaner_input_buffer_size
        );
        let (cleaner_output_tx, _cleaner_output_rx) = flume::bounded(
            settings.performance.cleaner_output_buffer_size
        );
        let data_cleaner = Some(Arc::new(tokio::sync::Mutex::new(
            OptimizedDataCleaner::new(cleaner_input_rx, cleaner_output_tx)
        )));

        let health_monitor = Arc::new(ApiHealthMonitor::new(30000));
        let event_bus = EventBus::new(settings.central_manager.event_buffer_size);

        let manager = Self {
            command_receiver: command_rx,
            data_receiver: data_rx,
            config_receiver: config_rx,
            collector_system: Arc::new(MarketCollectorSystem::new(
                data_tx, 
                health_monitor.clone(),
                settings.websocket_network.clone()
            )),
            pipeline: DataPipeline::new(settings),
            reasoner_client: ReasonerClient::new(settings),
            latest_books: Arc::new(DashMap::new()),
            
            batch_processor,
            simd_processor,
            cache_manager,
            lockfree_buffer,
            data_cleaner,
            
            snapshot_pool: Arc::new(crate::object_pool::ObjectPool::new(
                || MarketDataSnapshot {
                    orderbook: None,
                    trades: Vec::new(),
                    timestamp: crate::high_precision_time::Nanos::now(),
                    source: String::new(),
                },
                100,
            )),
            orderbook_pool: Arc::new(crate::object_pool::ObjectPool::new(
                || OrderBook::new(Symbol::new("", ""), String::new()),
                50,
            )),
            health_monitor,
            event_bus,
        };
        (manager, handle)
    }

    pub fn register_adapter(&self, adapter: Arc<dyn ExchangeAdapter>) {
        self.collector_system.register_adapter(adapter);
    }

    pub fn health_monitor(&self) -> Arc<ApiHealthMonitor> {
        self.health_monitor.clone()
    }

    #[instrument(name = "central_manager_run", skip_all)]
    pub async fn run(
        mut self,
        readiness_tx: watch::Sender<bool>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<(), MarketDataError> {
        info!("Central Manager started. Waiting for events.");
        let mut initial_data_received = false;

        loop {
            tokio::select! {
                biased;
                _ = shutdown_rx.recv() => {
                    info!("Manager received shutdown signal. Stopping all systems.");
                    self.collector_system.stop_all().await;
                    break;
                },
                Ok(command) = self.command_receiver.recv_async() => {
                    self.handle_api_command(command).await;
                },
                Some(new_configs) = self.config_receiver.recv() => {
                    info!("🔄 Received configuration update");
                    match self.collector_system.reconfigure(new_configs).await {
                        Ok(()) => info!("✅ Configuration hot reload completed"),
                        Err(e) => error!("❌ Configuration hot reload failed: {}", e),
                    }
                },
                Ok(message) = self.data_receiver.recv_async() => {
                    if !initial_data_received {
                        info!("🚀 First data message received. System READY.");
                        readiness_tx.send(true).ok();
                        initial_data_received = true;
                    }
                    self.process_adapter_event(message).await;
                },
                else => {
                    info!("All channels closed. Shutting down Central Manager.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn process_adapter_event(&mut self, event: AdapterEvent) {
        match event {
            AdapterEvent::MarketData(market_msg) => {
                // 处理市场数据，使用高性能优化组件
                match &market_msg {
                    MarketDataMessage::OrderBookSnapshot(ob) => {
                        info!("📊 Processing OrderBook: {} from {}", 
                              ob.symbol.as_pair(), ob.source);
                        
                        // 数据清洗处理
                        if let Some(ref cleaner_arc) = self.data_cleaner {
                            let cleaner = cleaner_arc.lock().await;
                            let test_snapshot = MarketDataSnapshot {
                                orderbook: Some(ob.clone()),
                                trades: vec![],
                                timestamp: crate::high_precision_time::Nanos::now(),
                                source: ob.source.clone(),
                            };
                            
                            match cleaner.clean(test_snapshot).await {
                                Ok(_) => info!("✅ Data cleaning successful"),
                                Err(e) => error!("❌ Data cleaning failed: {}", e),
                            }
                        }

                        // 使用无锁缓冲区和多级缓存
                        let _ = self.lockfree_buffer.push_orderbook(ob.clone());
                        let cache_key = format!("{}:{}", ob.source, ob.symbol.as_pair());
                        let _ = self.cache_manager.put(
                            cache_key, ob.clone(), CacheLevel::L1Memory
                        ).await;

                        // 更新订单簿缓存
                        let key = (ob.source.clone(), ob.symbol.clone());
                        self.latest_books.insert(key, ob.clone());
                    },
                    MarketDataMessage::Trade(trade) => {
                        info!("💰 Processing Trade: {} @ ${}", 
                              trade.symbol.as_pair(), trade.price.0);
                        
                        // 批处理和无锁缓冲区处理
                        let _ = self.lockfree_buffer.push_trade(trade.clone());
                        let _ = self.batch_processor.process_trade(trade.clone()).await;
                    },
                    MarketDataMessage::OrderBookUpdate(update) => {
                        info!("📈 Processing Update: {}", update.symbol.as_pair());
                        
                        // SIMD加速批处理
                        let updates = vec![update.clone()];
                        let _ = self.simd_processor.process_orderbook_updates(updates).await;
                    },
                    _ => {
                        debug!("Processing other market data message");
                    }
                }

                // 数据管道处理
                if let Some(local_message) = self.convert_to_local_message(&market_msg) {
                    let result = self.pipeline.process(local_message).await;
                    if let Some(anomaly) = result.anomaly {
                        info!("🚨 Anomaly detected: {:?}", anomaly.anomaly_type);
                    }
                }
            },
            AdapterEvent::Error(err) => error!("❌ Adapter error: {}", err),
            AdapterEvent::Connected => info!("✅ Adapter connected"),
            AdapterEvent::Disconnected => info!("❌ Adapter disconnected"),
            _ => debug!("Other adapter event processed"),
        }
    }

    // ... 其余实现方法 ...
}

/// 性能统计结构体
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub orderbook_count: usize,
    pub batch_processed_count: u64,
    pub cache_hit_rate: f64,
    pub lockfree_buffer_usage: f64,
    pub simd_operations_count: u64,
    pub compression_ratio: f64,
}
```

---

### 5.4 HTTP API服务 (src/http_api.rs)

HTTP API服务模块提供RESTful API端点，为外部系统提供访问市场数据的统一接口。

```rust
#![allow(dead_code)]
//! # HTTP REST API 模块
//!
//! 提供RESTful HTTP API端点，补充现有的gRPC API

use crate::{
    central_manager::{CentralManagerHandle, CentralManagerApi},
    health::ApiHealthMonitor,
    types::Symbol,
    settings::ApiServerSettings,
};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use serde_json::json;
use tracing::{info, error, warn};

/// HTTP API服务器结构
pub struct HttpApiServer {
    manager: CentralManagerHandle,
    health_monitor: Arc<ApiHealthMonitor>,
    config: ApiServerSettings,
}

impl HttpApiServer {
    pub fn new(
        manager: CentralManagerHandle, 
        health_monitor: Arc<ApiHealthMonitor>,
        config: ApiServerSettings,
    ) -> Self {
        Self {
            manager,
            health_monitor,
            config,
        }
    }

    /// 处理HTTP请求
    pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let method = req.method();
        let path = req.uri().path();
        
        match (method, path) {
            (&Method::GET, "/api/v1/health") => self.handle_health_check().await,
            (&Method::GET, "/api/v1/health/summary") => self.handle_health_summary().await,
            (&Method::GET, path) if path.starts_with("/api/v1/orderbook/") => {
                self.handle_orderbook_request(path).await
            },
            (&Method::GET, "/api/v1/exchanges") => self.handle_exchanges_list().await,
            (&Method::GET, "/api/v1/symbols") => self.handle_symbols_list().await,
            (&Method::GET, "/api/v1/stats") => self.handle_stats().await,
            (&Method::GET, "/api/v1/v3/performance") => self.handle_v3_performance().await,
            (&Method::GET, "/api/v1/v3/optimization-status") => self.handle_v3_optimization_status().await,
            (&Method::POST, "/api/v1/v3/reset-stats") => self.handle_v3_reset_stats().await,
            (&Method::POST, "/api/v1/v3/enable-optimization") => self.handle_v3_enable_optimization(req).await,
            (&Method::POST, "/api/v1/reconfigure") => self.handle_reconfigure_request(req).await,
            (&Method::GET, "/") => self.handle_root().await,
            _ => Ok(self.not_found()),
        }
    }

    /// 健康检查端点
    async fn handle_health_check(&self) -> Result<Response<Body>, Infallible> {
        let health_summary = self.health_monitor.get_health_summary();
        let is_healthy = health_summary.unhealthy_sources == 0;
        
        let response = json!({
            "status": if is_healthy { "healthy" } else { "unhealthy" },
            "healthy_sources": health_summary.healthy_sources,
            "unhealthy_sources": health_summary.unhealthy_sources,
            "total_sources": health_summary.total_sources,
            "timestamp": health_summary.timestamp.as_millis()
        });

        let status = if is_healthy {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        };

        Ok(Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(Body::from(response.to_string()))
            .expect("Operation failed"))
    }

    /// 获取订单簿数据
    async fn handle_orderbook_request(&self, path: &str) -> Result<Response<Body>, Infallible> {
        // 解析路径: /api/v1/orderbook/{exchange}/{symbol}
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() != 6 {
            return Ok(self.bad_request("Invalid orderbook path format"));
        }

        let exchange_id = parts[4];
        let symbol_pair = parts[5];
        
        // 解析交易对
        let symbol = match Symbol::from_pair(symbol_pair) {
            Some(s) => s,
            None => return Ok(self.bad_request("Invalid symbol format")),
        };

        // 获取订单簿
        match self.manager.get_latest_orderbook(exchange_id, &symbol).await {
            Ok(orderbook) => {
                let response = json!({
                    "symbol": orderbook.symbol.as_pair(),
                    "exchange": orderbook.source,
                    "timestamp": orderbook.timestamp.as_millis(),
                    "bids": orderbook.bids.iter().take(self.config.orderbook_depth_limit).map(|entry| [
                        entry.price.into_inner(),
                        entry.quantity.into_inner()
                    ]).collect::<Vec<_>>(),
                    "asks": orderbook.asks.iter().take(self.config.orderbook_depth_limit).map(|entry| [
                        entry.price.into_inner(),
                        entry.quantity.into_inner()
                    ]).collect::<Vec<_>>(),
                    "best_bid": orderbook.best_bid().map(|entry| entry.price.into_inner()),
                    "best_ask": orderbook.best_ask().map(|entry| entry.price.into_inner()),
                    "spread": orderbook.best_ask().zip(orderbook.best_bid())
                        .map(|(ask, bid)| ask.price.into_inner() - bid.price.into_inner()),
                    "sequence_id": orderbook.sequence_id
                });

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(response.to_string()))
                    .expect("Operation failed"))
            },
            Err(e) => {
                warn!("Failed to get orderbook for {}-{}: {}", exchange_id, symbol_pair, e);
                Ok(self.not_found())
            }
        }
    }

    /// 支持的交易所列表
    async fn handle_exchanges_list(&self) -> Result<Response<Body>, Infallible> {
        let exchanges = vec!["binance", "okx", "huobi", "gateio", "bybit"];
        
        let response = json!({
            "exchanges": exchanges,
            "status": "active",
            "total_supported": exchanges.len(),
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(response.to_string()))
            .expect("Operation failed"))
    }

    /// V3.0 性能监控端点
    async fn handle_v3_performance(&self) -> Result<Response<Body>, Infallible> {
        let response = json!({
            "v3_performance": {
                "target_latency_ns": 100_000,
                "current_avg_latency_ns": "pending_measurement",
                "o1_sort_engine": {
                    "bucket_count": 65536,
                    "status": "active",
                    "operations_per_second": "pending_measurement"
                },
                "intel_optimizations": {
                    "cpu_affinity": "enabled",
                    "avx512_support": "detected",
                    "performance_governor": "checking"
                },
                "zero_allocation_arch": {
                    "buffer_count": 65536,
                    "hit_rate": "pending_measurement",
                    "memory_saved_mb": "calculating"
                },
                "realtime_monitoring": {
                    "status": "active",
                    "metrics_collected": "live_data"
                }
            },
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(response.to_string()))
            .expect("Operation failed"))
    }

    /// 根路径 - API文档
    async fn handle_root(&self) -> Result<Response<Body>, Infallible> {
        let api_docs = json!({
            "name": "Qingxi Market Data API",
            "version": "3.0.0",
            "description": "High-performance cryptocurrency market data API with V3.0 optimizations",
            "endpoints": {
                "health": "/api/v1/health",
                "health_summary": "/api/v1/health/summary",
                "orderbook": "/api/v1/orderbook/{exchange}/{symbol}",
                "exchanges": "/api/v1/exchanges",
                "symbols": "/api/v1/symbols",
                "stats": "/api/v1/stats",
                "v3_performance": "/api/v1/v3/performance",
                "v3_optimization_status": "/api/v1/v3/optimization-status",
                "reconfigure": "/api/v1/reconfigure (POST)"
            },
            "v3_features": {
                "o1_sorting": "65536 bucket O(1) sorting engine",
                "intel_optimizations": "CPU affinity, AVX512, Performance Governor",
                "zero_allocation": "Zero allocation memory pool architecture",
                "realtime_monitoring": "Sub-millisecond performance tracking"
            },
            "examples": {
                "get_orderbook": "/api/v1/orderbook/binance/BTC/USDT",
                "health_check": "/api/v1/health",
                "v3_performance": "/api/v1/v3/performance"
            },
            "protocols": ["HTTP/REST", "gRPC"],
            "status": "operational"
        });

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(api_docs.to_string()))
            .expect("Operation failed"))
    }

    /// 动态配置重载端点
    async fn handle_reconfigure_request(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
            Ok(bytes) => bytes,
            Err(_) => return Ok(self.bad_request("Failed to read request body")),
        };

        let body_str = match std::str::from_utf8(&body_bytes) {
            Ok(s) => s,
            Err(_) => return Ok(self.bad_request("Invalid UTF-8 in request body")),
        };

        #[derive(serde::Deserialize)]
        struct ReconfigureRequest {
            reload_from_file: Option<bool>,
            config_path: Option<String>,
        }

        let request: ReconfigureRequest = match serde_json::from_str(body_str) {
            Ok(req) => req,
            Err(_) => return Ok(self.bad_request("Invalid JSON format")),
        };

        if let Some(config_path) = request.config_path {
            std::env::set_var("QINGXI_CONFIG_PATH", &config_path);
            info!("🔄 Updated QINGXI_CONFIG_PATH to: {}", config_path);
        }

        let new_settings = match crate::settings::Settings::load() {
            Ok(settings) => settings,
            Err(e) => {
                error!("Failed to reload configuration: {}", e);
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({
                        "status": "error",
                        "message": "Failed to reload configuration",
                        "error": e.to_string()
                    }).to_string()))
                    .expect("Operation failed"));
            }
        };

        let sources_count = new_settings.sources.len();
        match self.manager.reconfigure(new_settings.sources).await {
            Ok(_) => {
                info!("✅ Configuration successfully reloaded");
                let response = json!({
                    "status": "success",
                    "message": "Configuration reloaded successfully",
                    "timestamp": chrono::Utc::now().timestamp_millis(),
                    "sources_count": sources_count
                });

                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(response.to_string()))
                    .expect("Operation failed"))
            }
            Err(e) => {
                error!("Failed to apply new configuration: {}", e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .header("content-type", "application/json")
                    .body(Body::from(json!({
                        "status": "error",
                        "message": "Failed to apply new configuration",
                        "error": e.to_string()
                    }).to_string()))
                    .expect("Operation failed"))
            }
        }
    }

    /// 404 Not Found
    fn not_found(&self) -> Response<Body> {
        let error = json!({
            "error": "Not Found",
            "message": "The requested resource was not found",
            "code": 404
        });

        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "application/json")
            .body(Body::from(error.to_string()))
            .expect("Operation failed")
    }

    /// 400 Bad Request
    fn bad_request(&self, message: &str) -> Response<Body> {
        let error = json!({
            "error": "Bad Request",
            "message": message,
            "code": 400
        });

        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("content-type", "application/json")
            .body(Body::from(error.to_string()))
            .expect("Operation failed")
    }

    // ... 其余处理方法 ...
}

/// 启动HTTP API服务器
pub async fn serve_http_api(
    addr: SocketAddr,
    manager: CentralManagerHandle,
    health_monitor: Arc<ApiHealthMonitor>,
    config: ApiServerSettings,
) -> Result<(), Box<dyn std::error::Error>> {
    let api_server = Arc::new(HttpApiServer::new(manager, health_monitor, config));

    let make_svc = make_service_fn(move |_conn| {
        let api_server = api_server.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let api_server = api_server.clone();
                async move {
                    api_server.handle_request(req).await
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);
    info!("🌐 HTTP REST API server listening on {}", addr);

    if let Err(e) = server.await {
        error!("HTTP API server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
```

---

## 📋 代码审计总结

### 已包含的核心模块

✅ **项目配置文件** (Cargo.toml)  
✅ **系统配置文件** (configs/qingxi.toml)  
✅ **主程序入口** (src/main.rs)  
✅ **核心类型定义** (src/types.rs) - 部分  
✅ **配置管理模块** (src/settings.rs)  
✅ **中央管理器** (src/central_manager.rs)  
✅ **HTTP API服务** (src/http_api.rs)  

### 核心特性

🚀 **V3.0 性能优化**
- Intel CPU优化器集成
- 零分配内存架构  
- SIMD加速批处理
- 多级缓存系统
- 无锁缓冲区

🔧 **高性能架构**
- 异步并发处理
- 多线程CPU亲和性绑定
- 实时数据清洗
- 动态配置热重载

🌐 **完整API支持**
- RESTful HTTP API
- 健康监控端点
- 实时订单簿数据
- 性能统计接口

---

*注：本文档包含了qingxi项目的核心源代码，用于站外安全审核。所有敏感信息已被配置化处理，支持环境变量覆盖。*
```

/// 权威订单簿条目定义 - 结构体形式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct OrderBookEntry {
    pub price: Price,
    pub quantity: Quantity,
}

impl OrderBookEntry {
    pub fn new(price: f64, quantity: f64) -> Self {
        Self {
            price: OrderedFloat(price),
            quantity: OrderedFloat(quantity),
        }
    }
    
    /// V3.0 零分配优化：空订单簿条目常量
    pub const EMPTY: Self = Self {
        price: OrderedFloat(0.0),
        quantity: OrderedFloat(0.0),
    };
}

/// 权威订单簿定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBook {
    pub symbol: Symbol,
    pub source: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub timestamp: crate::high_precision_time::Nanos,
    pub sequence_id: Option<u64>,
    pub checksum: Option<String>,
}

impl OrderBook {
    pub fn new(symbol: Symbol, source: String) -> Self {
        Self {
            symbol,
            source,
            bids: Vec::new(),
            asks: Vec::new(),
            timestamp: crate::high_precision_time::Nanos::now(),
            sequence_id: None,
            checksum: None,
        }
    }

    /// 获取最佳买价
    pub fn best_bid(&self) -> Option<&OrderBookEntry> {
        self.bids.first()
    }

    /// 获取最佳卖价
    pub fn best_ask(&self) -> Option<&OrderBookEntry> {
        self.asks.first()
    }
}

/// 权威交易方向定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

/// 权威交易更新定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeUpdate {
    pub symbol: Symbol,
    pub price: OrderedFloat<f64>,
    pub quantity: OrderedFloat<f64>,
    pub side: TradeSide,
    pub timestamp: crate::high_precision_time::Nanos,
    pub source: String,
    pub trade_id: Option<String>,
}

/// 权威市场数据快照定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketDataSnapshot {
    pub orderbook: Option<OrderBook>,
    pub trades: Vec<TradeUpdate>,
    pub timestamp: crate::high_precision_time::Nanos,
    pub source: String,
}

/// 权威订阅详情定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionDetail {
    pub symbol: Symbol,
    pub channel: String,
}

impl SubscriptionDetail {
    pub fn new(symbol: Symbol, channel: &str) -> Self {
        Self {
            symbol,
            channel: channel.to_string(),
        }
    }
}

/// 权威心跳配置定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeartbeatConfig {
    pub interval_sec: u64,
    pub message: Option<String>,
}

fn default_true() -> bool {
    true
}

/// 权威市场数据源配置定义
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketSourceConfig {
    pub exchange_id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub websocket_url: Option<String>,
    pub rest_api_url: Option<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub api_passphrase: Option<String>, // For OKX
    #[serde(deserialize_with = "deserialize_symbols")]
    pub symbols: Vec<Symbol>,
    pub ws_endpoint: String,
    pub rest_endpoint: Option<String>,
    pub channel: Option<String>,
    pub heartbeat: Option<HeartbeatConfig>,
    pub reconnect_interval_sec: Option<u64>,
    pub max_reconnect_attempts: Option<u32>,
}

impl MarketSourceConfig {
    pub fn new(exchange: &str, ws_endpoint: &str) -> Self {
        Self {
            exchange_id: exchange.to_string(),
            enabled: true,
            websocket_url: None,
            rest_api_url: None,
            api_key: None,
            api_secret: None,
            api_passphrase: None,
            symbols: Vec::new(),
            ws_endpoint: ws_endpoint.to_string(),
            rest_endpoint: None,
            channel: None,
            heartbeat: None,
            reconnect_interval_sec: Some(5),
            max_reconnect_attempts: Some(10),
        }
    }

    pub fn with_symbols(mut self, symbols: Vec<Symbol>) -> Self {
        self.symbols = symbols;
        self
    }

    pub fn with_rest_endpoint(mut self, rest_endpoint: &str) -> Self {