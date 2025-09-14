# 🎯 实时套利监控系统

这是一个专为Celue策略模块设计的实时套利监控系统，能够检测跨交易所套利和三角套利机会。

## 📋 功能特性

### 🔄 跨交易所套利检测
- 实时监控多个交易所的价格差异
- 自动计算买入/卖出价差
- 显示利润百分比和绝对利润
- 只显示有意义的套利机会（>0.1%）

### 🔺 三角套利检测
- 在单个交易所内检测三角套利路径
- 支持的路径：
  - BTC/USDT -> ETH/BTC -> ETH/USDT
  - BTC/USDT -> BNB/BTC -> BNB/USDT
- 计算完整循环的利润率

### 📊 实时统计
- 总套利机会数量
- 跨交易所套利次数
- 三角套利次数
- 最大利润率记录
- 最近5条套利机会历史

## 🚀 快速启动

### 方法一：使用启动脚本（推荐）
```bash
cd /home/ubuntu/celue
./monitor_arbitrage.sh
```

### 方法二：直接运行
```bash
cd /home/ubuntu/celue
cargo run --bin arbitrage_monitor
```

## 📖 系统要求

### 必需服务
1. **NATS服务器** - 端口4222
2. **QingXi 5.1系统** - 市场数据采集和清洗
3. **QingXi桥接器** - 数据转换和发布

### 检查服务状态
```bash
# 检查NATS服务器
pgrep -f "nats-server"

# 检查QingXi系统
pgrep -f "market_data_module"

# 检查桥接器
pgrep -f "qingxi_bridge"
```

### 启动服务
```bash
# 启动QingXi系统
cd /home/ubuntu/qingxi && ./start_qingxi_v51_4exchanges.sh

# 启动桥接器
cd /home/ubuntu/celue && cargo run --bin qingxi_bridge
```

## 📊 监控界面说明

### 套利机会显示格式

#### 跨交易所套利
```
🚨 跨交易所套利 | BTC/USDT | 买入: binance@45000.00 | 卖出: okx@45100.00 | 利润: 0.22% ($100.00)
```

#### 三角套利
```
🔺 三角套利 | BTCUSDT->ETHBTC->ETHUSDT | 交易所: binance | 利润: 0.15%
```

### 统计信息显示
```
📊 统计信息:
   总套利机会: 127
   跨交易所套利: 89
   三角套利: 38
   最大利润率: 2.34%
   最后更新: 14:32:15
```

## ⚙️ 配置参数

### 利润阈值
- **最小利润率**: 0.1% (可在代码中修改)
- **三角套利阈值**: 0.1% (可在代码中修改)

### 更新频率
- **统计显示**: 每30秒刷新一次
- **数据处理**: 实时处理（微秒级延迟）

### 历史记录
- **最大记录数**: 1000条
- **显示数量**: 最近5条

## 🔧 故障排除

### 常见问题

#### 1. 连接NATS失败
```
Error: 连接被拒绝 (os error 111)
```
**解决方案**: 检查NATS服务器是否运行
```bash
nats-server --port 4222 --jetstream &
```

#### 2. 没有接收到数据
**可能原因**:
- QingXi系统未运行
- 桥接器未启动
- NATS主题不匹配

**检查方法**:
```bash
# 检查NATS主题
nats sub "market.data.normalized.*.*"
```

#### 3. 编译错误
**解决方案**: 确保依赖正确安装
```bash
cargo build --bin arbitrage_monitor
```

### 日志级别
系统默认显示：
- ✅ 系统状态信息
- 🚨 跨交易所套利机会
- 🔺 三角套利机会
- 📊 定期统计更新

## 📈 性能特性

- **延迟**: 微秒级数据处理
- **吞吐量**: 支持高频实时数据流
- **内存使用**: 优化的缓存机制
- **CPU使用**: 高效的套利检测算法

## 🛠️ 自定义扩展

### 添加新的套利路径
在`detect_triangular_arbitrage`方法中添加:
```rust
self.check_triangular_path(
    exchange,
    exchange_data,
    "PAIR1",
    "PAIR2", 
    "PAIR3"
).await;
```

### 修改利润阈值
在`check_arbitrage_pair`方法中修改:
```rust
if profit_percentage > 0.5 { // 改为0.5%
```

### 添加新的交易所
系统自动支持新的交易所，只需确保数据源包含该交易所即可。

## 📞 支持

如有问题，请检查：
1. 所有必需服务是否运行
2. 网络连接是否正常
3. 配置文件是否正确

系统设计为**7x24小时不间断运行**，能够捕获所有市场套利机会。 

这是一个专为Celue策略模块设计的实时套利监控系统，能够检测跨交易所套利和三角套利机会。

## 📋 功能特性

### 🔄 跨交易所套利检测
- 实时监控多个交易所的价格差异
- 自动计算买入/卖出价差
- 显示利润百分比和绝对利润
- 只显示有意义的套利机会（>0.1%）

### 🔺 三角套利检测
- 在单个交易所内检测三角套利路径
- 支持的路径：
  - BTC/USDT -> ETH/BTC -> ETH/USDT
  - BTC/USDT -> BNB/BTC -> BNB/USDT
- 计算完整循环的利润率

### 📊 实时统计
- 总套利机会数量
- 跨交易所套利次数
- 三角套利次数
- 最大利润率记录
- 最近5条套利机会历史

## 🚀 快速启动

### 方法一：使用启动脚本（推荐）
```bash
cd /home/ubuntu/celue
./monitor_arbitrage.sh
```

### 方法二：直接运行
```bash
cd /home/ubuntu/celue
cargo run --bin arbitrage_monitor
```

## 📖 系统要求

### 必需服务
1. **NATS服务器** - 端口4222
2. **QingXi 5.1系统** - 市场数据采集和清洗
3. **QingXi桥接器** - 数据转换和发布

### 检查服务状态
```bash
# 检查NATS服务器
pgrep -f "nats-server"

# 检查QingXi系统
pgrep -f "market_data_module"

# 检查桥接器
pgrep -f "qingxi_bridge"
```

### 启动服务
```bash
# 启动QingXi系统
cd /home/ubuntu/qingxi && ./start_qingxi_v51_4exchanges.sh

# 启动桥接器
cd /home/ubuntu/celue && cargo run --bin qingxi_bridge
```

## 📊 监控界面说明

### 套利机会显示格式

#### 跨交易所套利
```
🚨 跨交易所套利 | BTC/USDT | 买入: binance@45000.00 | 卖出: okx@45100.00 | 利润: 0.22% ($100.00)
```

#### 三角套利
```
🔺 三角套利 | BTCUSDT->ETHBTC->ETHUSDT | 交易所: binance | 利润: 0.15%
```

### 统计信息显示
```
📊 统计信息:
   总套利机会: 127
   跨交易所套利: 89
   三角套利: 38
   最大利润率: 2.34%
   最后更新: 14:32:15
```

## ⚙️ 配置参数

### 利润阈值
- **最小利润率**: 0.1% (可在代码中修改)
- **三角套利阈值**: 0.1% (可在代码中修改)

### 更新频率
- **统计显示**: 每30秒刷新一次
- **数据处理**: 实时处理（微秒级延迟）

### 历史记录
- **最大记录数**: 1000条
- **显示数量**: 最近5条

## 🔧 故障排除

### 常见问题

#### 1. 连接NATS失败
```
Error: 连接被拒绝 (os error 111)
```
**解决方案**: 检查NATS服务器是否运行
```bash
nats-server --port 4222 --jetstream &
```

#### 2. 没有接收到数据
**可能原因**:
- QingXi系统未运行
- 桥接器未启动
- NATS主题不匹配

**检查方法**:
```bash
# 检查NATS主题
nats sub "market.data.normalized.*.*"
```

#### 3. 编译错误
**解决方案**: 确保依赖正确安装
```bash
cargo build --bin arbitrage_monitor
```

### 日志级别
系统默认显示：
- ✅ 系统状态信息
- 🚨 跨交易所套利机会
- 🔺 三角套利机会
- 📊 定期统计更新

## 📈 性能特性

- **延迟**: 微秒级数据处理
- **吞吐量**: 支持高频实时数据流
- **内存使用**: 优化的缓存机制
- **CPU使用**: 高效的套利检测算法

## 🛠️ 自定义扩展

### 添加新的套利路径
在`detect_triangular_arbitrage`方法中添加:
```rust
self.check_triangular_path(
    exchange,
    exchange_data,
    "PAIR1",
    "PAIR2", 
    "PAIR3"
).await;
```

### 修改利润阈值
在`check_arbitrage_pair`方法中修改:
```rust
if profit_percentage > 0.5 { // 改为0.5%
```

### 添加新的交易所
系统自动支持新的交易所，只需确保数据源包含该交易所即可。

## 📞 支持

如有问题，请检查：
1. 所有必需服务是否运行
2. 网络连接是否正常
3. 配置文件是否正确

系统设计为**7x24小时不间断运行**，能够捕获所有市场套利机会。 

这是一个专为Celue策略模块设计的实时套利监控系统，能够检测跨交易所套利和三角套利机会。

## 📋 功能特性

### 🔄 跨交易所套利检测
- 实时监控多个交易所的价格差异
- 自动计算买入/卖出价差
- 显示利润百分比和绝对利润
- 只显示有意义的套利机会（>0.1%）

### 🔺 三角套利检测
- 在单个交易所内检测三角套利路径
- 支持的路径：
  - BTC/USDT -> ETH/BTC -> ETH/USDT
  - BTC/USDT -> BNB/BTC -> BNB/USDT
- 计算完整循环的利润率

### 📊 实时统计
- 总套利机会数量
- 跨交易所套利次数
- 三角套利次数
- 最大利润率记录
- 最近5条套利机会历史

## 🚀 快速启动

### 方法一：使用启动脚本（推荐）
```bash
cd /home/ubuntu/celue
./monitor_arbitrage.sh
```

### 方法二：直接运行
```bash
cd /home/ubuntu/celue
cargo run --bin arbitrage_monitor
```

## 📖 系统要求

### 必需服务
1. **NATS服务器** - 端口4222
2. **QingXi 5.1系统** - 市场数据采集和清洗
3. **QingXi桥接器** - 数据转换和发布

### 检查服务状态
```bash
# 检查NATS服务器
pgrep -f "nats-server"

# 检查QingXi系统
pgrep -f "market_data_module"

# 检查桥接器
pgrep -f "qingxi_bridge"
```

### 启动服务
```bash
# 启动QingXi系统
cd /home/ubuntu/qingxi && ./start_qingxi_v51_4exchanges.sh

# 启动桥接器
cd /home/ubuntu/celue && cargo run --bin qingxi_bridge
```

## 📊 监控界面说明

### 套利机会显示格式

#### 跨交易所套利
```
🚨 跨交易所套利 | BTC/USDT | 买入: binance@45000.00 | 卖出: okx@45100.00 | 利润: 0.22% ($100.00)
```

#### 三角套利
```
🔺 三角套利 | BTCUSDT->ETHBTC->ETHUSDT | 交易所: binance | 利润: 0.15%
```

### 统计信息显示
```
📊 统计信息:
   总套利机会: 127
   跨交易所套利: 89
   三角套利: 38
   最大利润率: 2.34%
   最后更新: 14:32:15
```

## ⚙️ 配置参数

### 利润阈值
- **最小利润率**: 0.1% (可在代码中修改)
- **三角套利阈值**: 0.1% (可在代码中修改)

### 更新频率
- **统计显示**: 每30秒刷新一次
- **数据处理**: 实时处理（微秒级延迟）

### 历史记录
- **最大记录数**: 1000条
- **显示数量**: 最近5条

## 🔧 故障排除

### 常见问题

#### 1. 连接NATS失败
```
Error: 连接被拒绝 (os error 111)
```
**解决方案**: 检查NATS服务器是否运行
```bash
nats-server --port 4222 --jetstream &
```

#### 2. 没有接收到数据
**可能原因**:
- QingXi系统未运行
- 桥接器未启动
- NATS主题不匹配

**检查方法**:
```bash
# 检查NATS主题
nats sub "market.data.normalized.*.*"
```

#### 3. 编译错误
**解决方案**: 确保依赖正确安装
```bash
cargo build --bin arbitrage_monitor
```

### 日志级别
系统默认显示：
- ✅ 系统状态信息
- 🚨 跨交易所套利机会
- 🔺 三角套利机会
- 📊 定期统计更新

## 📈 性能特性

- **延迟**: 微秒级数据处理
- **吞吐量**: 支持高频实时数据流
- **内存使用**: 优化的缓存机制
- **CPU使用**: 高效的套利检测算法

## 🛠️ 自定义扩展

### 添加新的套利路径
在`detect_triangular_arbitrage`方法中添加:
```rust
self.check_triangular_path(
    exchange,
    exchange_data,
    "PAIR1",
    "PAIR2", 
    "PAIR3"
).await;
```

### 修改利润阈值
在`check_arbitrage_pair`方法中修改:
```rust
if profit_percentage > 0.5 { // 改为0.5%
```

### 添加新的交易所
系统自动支持新的交易所，只需确保数据源包含该交易所即可。

## 📞 支持

如有问题，请检查：
1. 所有必需服务是否运行
2. 网络连接是否正常
3. 配置文件是否正确

系统设计为**7x24小时不间断运行**，能够捕获所有市场套利机会。 
 
 
 