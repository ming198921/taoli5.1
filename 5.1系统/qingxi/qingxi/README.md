# Qingxi Market Data System

## 概述

Qingxi 是一个生产级高性能市场数据收集和一致性验证系统，支持实时多交易所数据聚合、异常检测和智能分析。系统提供 gRPC 和 HTTP REST API 双协议支持，满足不同客户端需求。

## 主要特性

### 🚀 核心功能
- **多交易所数据采集**: 支持 Binance、OKX、Huobi 等主流交易所
- **实时数据处理**: 高性能 WebSocket 连接，毫秒级延迟
- **一致性验证**: 跨交易所价格、时间戳、订单量一致性检查
- **异常检测**: 智能异常模式识别和告警
- **数据质量监控**: 实时健康状态监控和数据质量评估

### 🌐 API 接口
- **gRPC API**: 高性能 Protocol Buffers 协议 (端口 50051)
- **HTTP REST API**: 标准 RESTful 接口 (端口 50061)
- **实时流数据**: WebSocket 和 gRPC 流支持
- **完整的 API 文档**: OpenAPI/Swagger 兼容

### ⚡ 性能优化
- **无锁数据结构**: 高并发场景下的无锁环形缓冲区、栈和队列
- **批处理优化**: SIMD 指令集优化的批量数据处理
- **多级缓存**: L1(内存)/L2(磁盘)/L3(网络) 缓存系统
- **CPU 亲和性**: 智能线程调度和 CPU 绑定

## 安装与部署

### 本地开发环境

```bash
# 克隆项目
git clone https://github.com/your-username/qingxi.git
cd qingxi

# 构建项目
cargo build --release

# 运行主服务
cargo run --bin market_data_module

# 运行 HTTP API 演示
cargo run --bin http_api_demo
```

### Docker 部署

```bash
# 构建 Docker 镜像
docker build -t qingxi-market-data .

# 运行容器
docker run -p 50051:50051 -p 50061:50061 qingxi-market-data
```

### 生产环境配置

```toml
# configs/qingxi.toml
[general]
log_level = "info"
metrics_enabled = true

[api_server]
host = "0.0.0.0"
port = 50051

[central_manager]
event_buffer_size = 10000

[consistency_thresholds]
price_diff_percentage = 0.5
timestamp_diff_ms = 5000
sequence_gap_threshold = 10
spread_threshold_percentage = 1.0
critical_spread_threshold_percentage = 2.0
max_time_diff_ms = 10000.0
volume_consistency_threshold = 0.5

[[sources]]
exchange_id = "binance"
symbols = [
    { base = "BTC", quote = "USDT" },
    { base = "ETH", quote = "USDT" }
]
ws_endpoint = "wss://stream.binance.com:9443/ws"
rest_endpoint = "https://api.binance.com/api/v3"
channel = "orderbook"
```

## API 文档

### HTTP REST API 端点

#### 健康检查
```http
GET /api/v1/health
GET /api/v1/health/summary
```

#### 市场数据
```http
GET /api/v1/orderbook/{exchange}/{symbol}
GET /api/v1/exchanges
GET /api/v1/symbols
GET /api/v1/stats
```

#### API 文档
```http
GET /
```

### 示例响应

```json
{
  "name": "Qingxi Market Data API",
  "version": "1.0.0",
  "description": "Production-grade market data collection and analysis",
  "endpoints": {
    "health": "/api/v1/health",
    "orderbook": "/api/v1/orderbook/{exchange}/{symbol}",
    "exchanges": "/api/v1/exchanges",
    "symbols": "/api/v1/symbols",
    "stats": "/api/v1/stats"
  }
}
```

### gRPC API

```protobuf
service MarketDataService {
  rpc GetOrderBook(OrderBookRequest) returns (OrderBookResponse);
  rpc StreamOrderBook(OrderBookRequest) returns (stream OrderBookUpdate);
  rpc GetHealthStatus(HealthRequest) returns (HealthResponse);
}
```

## 架构设计

### 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP REST     │    │      gRPC       │    │   WebSocket     │
│      API        │    │       API       │    │    Clients      │
│   (Port 50061)  │    │   (Port 50051)  │    │                 │
└─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘
          │                      │                      │
          └──────────────────────┼──────────────────────┘
                                 │
                    ┌─────────────┴─────────────┐
                    │    Central Manager        │
                    │   - Event Processing      │
                    │   - Data Orchestration    │
                    │   - Health Monitoring     │
                    └─────────────┬─────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        │                         │                         │
┌───────▼───────┐       ┌─────────▼─────────┐       ┌───────▼───────┐
│   Consistency │       │   Market Data     │       │   Anomaly     │
│    Checker    │       │    Collector      │       │   Detection   │
└───────────────┘       └─────────┬─────────┘       └───────────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    │             │             │
              ┌─────▼─────┐ ┌─────▼─────┐ ┌─────▼─────┐
              │  Binance  │ │    OKX    │ │   Huobi   │
              │  Adapter  │ │  Adapter  │ │  Adapter  │
              └───────────┘ └───────────┘ └───────────┘
```

### 核心模块

- **Central Manager**: 事件协调和数据分发
- **Market Collectors**: 多交易所数据采集适配器
- **Consistency Engine**: 跨交易所一致性验证
- **Anomaly Detection**: 智能异常检测和告警
- **API Servers**: HTTP REST 和 gRPC 双协议支持
- **Health Monitor**: 系统健康状态监控

## 性能指标

### 基准测试结果

- **延迟**: < 1ms 平均处理延迟
- **吞吐量**: > 100,000 消息/秒处理能力
- **内存使用**: < 512MB 常驻内存
- **CPU 使用**: < 20% 单核心使用率
- **连接数**: 支持 1000+ 并发 WebSocket 连接

### 监控指标

```bash
# Prometheus 指标端点
curl http://localhost:50052/metrics

# 健康检查端点
curl http://localhost:50053/health
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行基准测试
cargo bench

# 运行集成测试
cargo test --test integration_basic_usage

# 运行演示程序
cargo run --bin http_api_demo
cargo run --bin market_collector_demo
cargo run --bin market_data_feed_demo
```

## 部署选项

### Kubernetes 部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: qingxi-market-data
spec:
  replicas: 3
  selector:
    matchLabels:
      app: qingxi-market-data
  template:
    metadata:
      labels:
        app: qingxi-market-data
    spec:
      containers:
      - name: qingxi
        image: qingxi-market-data:latest
        ports:
        - containerPort: 50051
        - containerPort: 50061
        env:
        - name: RUST_LOG
          value: "info"
        - name: QINGXI_CONFIG_PATH
          value: "/app/configs/qingxi.toml"
```

### Docker Compose

```yaml
version: '3.8'
services:
  qingxi:
    build: .
    ports:
      - "50051:50051"
      - "50061:50061"
      - "50052:50052"  # Metrics
      - "50053:50053"  # Health
    environment:
      - RUST_LOG=info
      - QINGXI_CONFIG_PATH=/app/configs/qingxi.toml
    volumes:
      - ./configs:/app/configs
```

## 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 联系方式

- **项目维护者**: Qingxi Performance Team
- **邮箱**: dev@qingxi.tech
- **文档**: [https://docs.qingxi.tech](https://docs.qingxi.tech)
- **问题反馈**: [GitHub Issues](https://github.com/your-username/qingxi/issues)

## 更新日志

### v1.0.1 (最新)

- ✅ 新增 HTTP REST API 支持
- ✅ 实现一致性检查系统
- ✅ 添加无锁数据结构优化
- ✅ 实现多级缓存系统
- ✅ 添加批处理优化
- ✅ 完善监控和健康检查
- ✅ 优化 Docker 部署配置

### v1.0.0

- ✅ 基础 gRPC API 实现
- ✅ 多交易所数据采集
- ✅ 异常检测系统
- ✅ 基础配置管理
