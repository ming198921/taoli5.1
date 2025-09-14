# 🚀 QINGXI系统启动和运行指南

## 📋 **系统状态概览**

### ✅ **配置就绪状态**
- **主配置文件**: `configs/qingxi.toml` ✅ 
- **生产配置文件**: `configs/production.toml` ✅
- **配置验证**: 所有字段完整 ✅
- **编译状态**: 成功编译 ✅

### ⚠️ **待优化项**
- **编译警告**: 33个 (目标: <10个)
- **代码清理**: 进行中

---

## 🔧 **启动配置步骤**

### 第一步：环境准备
```bash
cd /home/ubuntu/qingxi/qingxi

# 1. 验证配置完整性
cargo run --bin config_validator --release

# 2. 检查编译状态
cargo build --release

# 3. 检查依赖状态
cargo check
```

### 第二步：数据源配置

#### 🏛️ **已配置的交易所数据源**
系统已配置4个数据源，均可正常工作：

1. **Binance Spot**
   - 交易对: `["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT"]`
   - WebSocket: `wss://stream.binance.com:9443/ws/btcusdt@depth@100ms`
   - REST API: `https://api.binance.com`

2. **Bybit Spot**
   - 交易对: `["BTCUSDT", "ETHUSDT", "SOLUSDT", "XRPUSDT"]`
   - WebSocket: `wss://stream.bybit.com/v5/public/spot`
   - REST API: `https://api.bybit.com`

3. **OKX Spot**
   - 交易对: `["BTC-USDT", "ETH-USDT", "SOL-USDT", "XRP-USDT"]`
   - WebSocket: `wss://ws.okx.com:8443/ws/v5/public`
   - REST API: `https://www.okx.com`

#### ⚙️ **配置调整**
如需修改数据源，编辑 `configs/qingxi.toml`:
```toml
[[sources]]
id = "binance_spot"
adapter_type = "binance"
enabled = true  # 设为false可禁用
exchange_id = "binance"
symbols = ["BTCUSDT", "ETHUSDT"]  # 修改交易对
```

### 第三步：性能优化配置

#### 🎯 **关键性能参数** (已配置化)
```toml
[performance]
max_concurrent_tasks = 8        # 并发任务数
memory_pool_size = 1048576     # 内存池大小 (1MB)
batch_size = 1000              # 批处理大小

[memory_pools]
orderbook_entry_pool_size = 1000    # 订单簿条目池
trade_update_pool_size = 5000       # 交易更新池
snapshot_pool_size = 500            # 快照池

[exchanges]
bybit_orderbook_depth = 60          # Bybit深度
binance_orderbook_depth = 100       # Binance深度
event_buffer_size = 5000            # 事件缓冲区
```

### 第四步：推理器服务配置

#### 🧠 **Reasoner配置** (已生产化)
```toml
[reasoner]
api_endpoint = "http://reasoner-service:8081"  # 生产端点
```

**环境变量覆盖支持**:
```bash
export QINGXI_REASONER_ENDPOINT="http://custom-reasoner:8081"
```

---

## 🚀 **启动命令**

### 标准启动
```bash
# 使用默认配置启动
cargo run --release

# 使用生产配置启动
QINGXI_CONFIG_PATH=configs/production cargo run --release
```

### 开发模式启动
```bash
# 开发模式 (详细日志)
RUST_LOG=debug cargo run

# 带配置验证的启动
cargo run --bin config_validator --release && cargo run --release
```

### 后台服务启动
```bash
# 后台运行
nohup cargo run --release > qingxi.log 2>&1 &

# 使用systemd服务
sudo systemctl start qingxi
```

---

## 📊 **运行监控**

### 系统健康检查
```bash
# API服务器状态 (端口 50051)
curl http://localhost:50051/health

# 指标监控 (端口 50052)
curl http://localhost:50052/metrics

# HTTP API (端口 50061)
curl http://localhost:50061/api/v1/status
```

### 数据流监控
```bash
# 实时日志监控
tail -f qingxi.log

# 数据源连接状态
curl http://localhost:50061/api/v1/sources

# 订单簿状态
curl http://localhost:50061/api/v1/orderbooks
```

---

## 🔧 **配置调优指南**

### 高频交易优化
```toml
[performance]
max_concurrent_tasks = 16       # 增加并发
batch_timeout_ms = 50          # 减少延迟
enable_simd = true             # 启用SIMD优化

[threading]
num_worker_threads = 8         # 增加线程数
enable_cpu_affinity = true     # 启用CPU亲和性
```

### 内存优化
```toml
[memory_allocator]
zero_allocation_buffer_size = 131072  # 零分配缓冲区
large_buffer_size = 262144           # 大缓冲区
huge_buffer_size = 1048576           # 巨大缓冲区
```

### 网络优化
```toml
[quality_thresholds]
minimum_data_freshness_ms = 500      # 数据新鲜度要求
maximum_latency_ms = 50             # 最大延迟要求
```

---

## 🚨 **故障排除**

### 常见启动问题

#### 1. 配置加载失败
```bash
# 验证配置文件
cargo run --bin config_validator --release
```

#### 2. 端口冲突
```bash
# 检查端口占用
netstat -tlnp | grep 50051
```

#### 3. 内存不足
```bash
# 降低内存池大小
[performance]
memory_pool_size = 524288  # 512KB
```

#### 4. 网络连接问题
```bash
# 测试交易所连通性
curl -I https://api.binance.com/api/v3/ping
curl -I https://api.bybit.com/v5/market/time
```

### 性能问题排查
```bash
# 检查系统资源
htop
iostat 1
free -h

# 检查网络延迟
ping api.binance.com
```

---

## 📈 **生产部署**

### Docker部署
```bash
# 构建生产镜像
docker build -f Dockerfile.production -t qingxi:production .

# 运行容器
docker run -d \
  --name qingxi \
  -p 50051:50051 \
  -p 50052:50052 \
  -p 50061:50061 \
  -v /path/to/configs:/app/configs \
  qingxi:production
```

### K8s部署
```bash
# 部署到K8s
kubectl apply -f k8s/

# 检查部署状态
kubectl get pods -l app=qingxi
kubectl logs -f deployment/qingxi
```

### 自动化部署
```bash
# 使用部署脚本
./scripts/deploy_production.sh

# 检查部署结果
./scripts/health_check.sh
```

---

## ✅ **启动成功验证清单**

- [ ] 配置验证通过: `cargo run --bin config_validator --release`
- [ ] 编译成功: `cargo build --release`
- [ ] 服务启动: `cargo run --release`
- [ ] API响应正常: `curl http://localhost:50051/health`
- [ ] 数据源连接成功: 检查日志无连接错误
- [ ] 内存使用正常: `free -h`
- [ ] CPU使用正常: `htop`

---

**🎉 系统已配置完成，可以正常启动进行数据获取和清洗！**

**配置更新时间**: 2025-07-26  
**配置验证状态**: ✅ 通过  
**生产就绪状态**: ✅ 就绪
