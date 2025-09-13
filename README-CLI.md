# 套利系统5.1命令行控制器使用指南

## 🎯 概述

这是一套完整的套利系统5.1命令行控制工具，让你可以通过简单的命令完全控制整个系统的所有功能，包括：

- **QingXi数据处理模块** - 多交易所数据采集和清洗
- **CeLue策略执行模块** - 跨交易所和三角套利策略
- **AI风控系统** - 智能风险控制和监控  
- **AI模型训练** - 机器学习模型训练和部署
- **费用管理系统** - 交易所费率查询、比较和套利成本分析
- **系统监控** - 实时性能监控和日志查看
- **配置管理** - 系统配置的查看和修改

## 📦 工具组成

### 1. 主控制器 (`arbitrage-cli-controller.py`)
**最完整的控制工具**，支持所有系统功能的细粒度控制。

### 2. 快速命令工具 (`quick-commands.sh`) 
**最便捷的控制工具**，提供常用操作的快捷命令和交互式菜单。

### 3. 批量操作工具 (`batch-operations.py`)
**最强大的自动化工具**，支持批量执行操作和自动化流程。

## 🚀 快速开始

### 方法1: 使用快速命令工具（推荐新手）

```bash
# 进入系统目录
cd /home/ubuntu/5.1xitong

# 查看系统状态
./quick-commands.sh status

# 启动整个系统  
./quick-commands.sh start

# 使用交互式菜单
./quick-commands.sh menu
```

### 方法2: 使用主控制器（推荐高级用户）

```bash
# 查看帮助
python3 arbitrage-cli-controller.py --help

# 系统控制
python3 arbitrage-cli-controller.py system status
python3 arbitrage-cli-controller.py system start

# 数据处理
python3 arbitrage-cli-controller.py data start-all
python3 arbitrage-cli-controller.py data status

# 策略管理
python3 arbitrage-cli-controller.py strategy list
python3 arbitrage-cli-controller.py strategy start inter_exchange_production
```

### 方法3: 使用批量操作（推荐自动化）

```bash
# 交互式构建批量操作
python3 batch-operations.py --interactive

# 执行预定义的批量操作
python3 batch-operations.py batch-startup.yaml
```

## 📋 完整功能列表

### 🏗️ 系统控制

| 命令 | 快速命令 | 主控制器 | 功能描述 |
|------|----------|----------|----------|
| 查看状态 | `./quick-commands.sh status` | `python3 arbitrage-cli-controller.py system status` | 检查所有服务状态 |
| 启动系统 | `./quick-commands.sh start` | `python3 arbitrage-cli-controller.py system start` | 启动整个套利系统 |
| 停止系统 | `./quick-commands.sh stop` | `python3 arbitrage-cli-controller.py system stop` | 停止整个套利系统 |
| 重启系统 | `./quick-commands.sh restart` | `python3 arbitrage-cli-controller.py system restart` | 重启整个套利系统 |

### 🔧 手动服务启动命令

| 服务 | 启动命令 | 端口 | 功能描述 |
|------|----------|------|----------|
| 统一网关 | `cd /home/ubuntu/5.1xitong/5.1系统 && ./target/release/unified-gateway &` | 3000 | API统一入口网关 |
| 交易服务 | `cd /home/ubuntu/5.1xitong/5.1系统/trading-service && TRADING_SERVICE_PORT=4005 ./target/release/trading-service &` | 4005 | 真实交易执行服务 |
| 策略服务 | `cd /home/ubuntu/5.1xitong/5.1系统/strategy-service && ./target/release/strategy-service &` | 4003 | 策略执行和管理 |
| 配置服务 | `cd /home/ubuntu/5.1xitong/5.1系统/config-service && ./target/release/config-service &` | 4000 | 系统配置管理 |
| 清洗服务 | `cd /home/ubuntu/5.1xitong/5.1系统/cleaning-service && ./target/release/cleaning-service &` | 4002 | 数据清洗处理 |

### 📊 数据处理模块

| 功能 | 快速命令 | 主控制器 | 功能描述 |
|------|----------|----------|----------|
| 启动数据采集 | `./quick-commands.sh data-start` | `python3 arbitrage-cli-controller.py data start-all` | 启动所有交易所数据采集 |
| 停止数据采集 | `./quick-commands.sh data-stop` | `python3 arbitrage-cli-controller.py data stop-all` | 停止所有数据采集 |
| 数据状态 | `./quick-commands.sh data-status` | `python3 arbitrage-cli-controller.py data status` | 查看数据采集状态 |
| 数据清洗 | `./quick-commands.sh data-clean` | `python3 arbitrage-cli-controller.py data clean [exchange]` | 执行数据清洗 |

### ⚙️ 策略管理

| 功能 | 快速命令 | 主控制器 | 功能描述 |
|------|----------|----------|----------|
| 列出策略 | `./quick-commands.sh strategies` | `python3 arbitrage-cli-controller.py strategy list` | 显示所有可用策略 |
| 启动跨交易所策略 | `./quick-commands.sh start-inter` | `python3 arbitrage-cli-controller.py strategy start inter_exchange_production` | 启动跨交易所套利 |
| 启动三角套利 | `./quick-commands.sh start-tri` | `python3 arbitrage-cli-controller.py strategy start triangular_production` | 启动三角套利 |
| 策略状态 | `./quick-commands.sh strategy-status` | `python3 arbitrage-cli-controller.py strategy status [name]` | 查看策略运行状态 |
| 停止策略 | - | `python3 arbitrage-cli-controller.py strategy stop [name]` | 停止指定策略 |

### 🛡️ AI风控系统

| 功能 | 快速命令 | 主控制器 | 功能描述 |
|------|----------|----------|----------|
| 风控状态 | `./quick-commands.sh risk-status` | `python3 arbitrage-cli-controller.py risk status` | 查看风控系统状态 |
| 设置最大敞口 | `./quick-commands.sh set-max-exp` | `python3 arbitrage-cli-controller.py risk set-limit max_exposure 10000` | 设置风控限制 |
| 紧急停止 | `./quick-commands.sh emergency` | `python3 arbitrage-cli-controller.py risk emergency-stop` | 紧急停止所有交易 |

### 🤖 AI模型管理

| 功能 | 快速命令 | 主控制器 | 功能描述 |
|------|----------|----------|----------|
| 列出模型 | `./quick-commands.sh ai-models` | `python3 arbitrage-cli-controller.py ai models` | 显示所有AI模型 |
| 训练风险模型 | `./quick-commands.sh train-risk` | `python3 arbitrage-cli-controller.py ai train risk_model 30` | 训练风险预测模型 |
| 训练价格模型 | `./quick-commands.sh train-price` | `python3 arbitrage-cli-controller.py ai train price_prediction 7` | 训练价格预测模型 |
| 部署模型 | - | `python3 arbitrage-cli-controller.py ai deploy model_name latest` | 部署AI模型 |

### 📋 监控和日志

| 功能 | 快速命令 | 主控制器 | 功能描述 |
|------|----------|----------|----------|
| 查看日志 | `./quick-commands.sh logs` | `python3 arbitrage-cli-controller.py logs tail all 100` | 查看实时日志 |
| 性能监控 | `./quick-commands.sh monitor` | `python3 arbitrage-cli-controller.py monitor performance --duration 300` | 实时性能监控 |
| 策略日志 | `./quick-commands.sh tail-strategy` | `python3 arbitrage-cli-controller.py logs tail strategy 50` | 查看策略日志 |

### 💰 费用管理

| 功能 | 主控制器 | 功能描述 |
|------|----------|----------|
| 查看所有费率 | `python3 arbitrage-cli-controller.py fees list` | 显示所有交易所费率概览 |
| 查看单个交易所费率 | `python3 arbitrage-cli-controller.py fees list binance` | 显示特定交易所详细费率信息 |
| 费率比较 | `python3 arbitrage-cli-controller.py fees compare BTCUSDT` | 比较各交易所费率并排名 |
| 计算交易费用 | `python3 arbitrage-cli-controller.py fees calculate 10000 binance BTCUSDT` | 计算指定金额的交易费用 |
| 套利费用分析 | `python3 arbitrage-cli-controller.py fees arbitrage-analysis BTCUSDT 5000` | 分析套利交易的费用成本 |
| 刷新费率数据 | `python3 arbitrage-cli-controller.py fees refresh` | 从交易所API更新最新费率 |

### ⚙️ 配置管理

| 功能 | 主控制器 | 功能描述 |
|------|----------|----------|
| 查看配置 | `python3 arbitrage-cli-controller.py config show system` | 显示系统配置 |
| 设置配置 | `python3 arbitrage-cli-controller.py config set trading.min_profit 0.001` | 修改配置项 |

### 🔑 交易所API配置

| 功能 | CLI命令 | 功能描述 |
|------|---------|----------|
| 添加Binance API | `curl -X POST "http://localhost:4005/api/config/binance" -H "Content-Type: application/json" -d '{"api_key":"YOUR_KEY","api_secret":"YOUR_SECRET","live_trading":true}'` | 配置币安真实交易API |
| 通过统一网关配置 | `curl -X POST "http://localhost:4001/api/config/exchange" -H "Content-Type: application/json" -d '{"name":"binance","api_key":"YOUR_KEY","api_secret":"YOUR_SECRET","sandbox_mode":false,"enabled":true}'` | 通过统一网关配置交易所API |
| 分别设置密钥 | `curl -X POST "http://localhost:4000/api/config/set" -H "Content-Type: application/json" -d '{"key":"binance.api_key","value":"YOUR_API_KEY"}'` | 分别配置API密钥和Secret |
| 验证API配置 | `curl -s "http://localhost:4005/api/config/binance" \| jq .` | 检查API配置是否成功 |

## 🔄 批量操作示例

### 1. 完整系统启动流程

```yaml
# batch-startup.yaml
description: "完整系统启动流程"
operations:
  - name: "系统状态检查"
    command: ["system", "status"]
    delay: 0
    retry: 1
    ignore_error: true
    
  - name: "启动核心系统"  
    command: ["system", "start"]
    delay: 10
    retry: 2
    ignore_error: false
    
  - name: "启动数据采集"
    command: ["data", "start-all"]  
    delay: 5
    retry: 2
    ignore_error: false
    
  - name: "启动跨交易所策略"
    command: ["strategy", "start", "inter_exchange_production"]
    delay: 3
    retry: 1
    ignore_error: true
```

执行：`python3 batch-operations.py batch-startup.yaml`

### 2. 每日维护流程

```yaml
# batch-maintenance.yaml  
description: "每日系统维护"
operations:
  - name: "数据清洗"
    command: ["data", "clean"]
    delay: 2
    
  - name: "AI模型状态检查"  
    command: ["ai", "models"]
    delay: 1
    
  - name: "风控状态检查"
    command: ["risk", "status"] 
    delay: 1
```

## 🔑 交易所API配置详细演示

### 1. 配置币安真实交易API

```bash
# 方法1: 通过交易服务直接配置（推荐）
curl -X POST "http://localhost:4005/api/config/binance" \
-H "Content-Type: application/json" \
-d '{
  "api_key": "aJS2cL8LyIHw5PfUeKvYkdfM1pf0ewaVKI7m0GkwsXs3qYhrVQgHz8mGjkCZ6xL0",
  "api_secret": "rGrBCqmSxT0khRWFuh72eG6irYw0z82BSvT7cxcIRR1yrxAdJ4jiODnjUkXPLGzk",
  "live_trading": true
}'

# 成功响应示例:
# {
#   "success": true,
#   "message": "Binance API配置成功",
#   "data": {
#     "exchange": "binance",
#     "api_configured": true,
#     "live_trading": true,
#     "timestamp": "2025-09-11T08:30:00Z"
#   }
# }
```

### 2. 通过统一网关配置（全功能方式）

```bash
# 方法2: 通过统一网关配置多个交易所
curl -X POST "http://localhost:4001/api/config/exchange" \
-H "Content-Type: application/json" \
-d '{
  "name": "binance",
  "api_key": "YOUR_API_KEY",
  "api_secret": "YOUR_API_SECRET", 
  "sandbox_mode": false,
  "enabled": true,
  "rate_limit": 1200,
  "retry_attempts": 3
}'

# 配置多个交易所示例:
# Binance
curl -X POST "http://localhost:4001/api/config/exchange" -H "Content-Type: application/json" -d '{"name":"binance","api_key":"KEY1","api_secret":"SECRET1","enabled":true}'

# OKX  
curl -X POST "http://localhost:4001/api/config/exchange" -H "Content-Type: application/json" -d '{"name":"okx","api_key":"KEY2","api_secret":"SECRET2","enabled":true}'

# Huobi
curl -X POST "http://localhost:4001/api/config/exchange" -H "Content-Type: application/json" -d '{"name":"huobi","api_key":"KEY3","api_secret":"SECRET3","enabled":true}'
```

### 3. 分别配置API密钥（高级配置）

```bash
# 方法3: 通过配置服务分别设置各项参数
# 设置API Key
curl -X POST "http://localhost:4000/api/config/set" \
-H "Content-Type: application/json" \
-d '{
  "key": "binance.api_key",
  "value": "aJS2cL8LyIHw5PfUeKvYkdfM1pf0ewaVKI7m0GkwsXs3qYhrVQgHz8mGjkCZ6xL0"
}'

# 设置API Secret
curl -X POST "http://localhost:4000/api/config/set" \
-H "Content-Type: application/json" \
-d '{
  "key": "binance.api_secret",
  "value": "rGrBCqmSxT0khRWFuh72eG6irYw0z82BSvT7cxcIRR1yrxAdJ4jiODnjUkXPLGzk"
}'

# 启用真实交易
curl -X POST "http://localhost:4000/api/config/set" \
-H "Content-Type: application/json" \
-d '{
  "key": "binance.live_trading",
  "value": true
}'
```

### 4. 验证API配置

```bash
# 检查API配置状态
curl -s "http://localhost:4005/api/config/binance" | jq .

# 输出示例:
# {
#   "success": true,
#   "data": {
#     "exchange": "binance",
#     "api_key_configured": true,
#     "api_secret_configured": true,
#     "live_trading": true,
#     "connection_status": "connected",
#     "last_test": "2025-09-11T08:30:00Z"
#   }
# }

# 测试API连接
curl -s "http://localhost:4005/api/exchanges/binance/test-connection" | jq .

# 查看账户余额（验证API有效性）
curl -s "http://localhost:4005/api/exchanges/binance/account" | jq .
```

### 5. API安全配置建议

```bash
# 🔒 安全配置建议:

# 1. 设置IP白名单（在币安网站设置）
# 2. 限制API权限（只启用现货交易，禁用提现）
# 3. 使用环境变量（生产环境推荐）

# 环境变量配置方式:
export BINANCE_API_KEY="your_api_key_here"
export BINANCE_API_SECRET="your_api_secret_here"

# 然后配置系统使用环境变量
curl -X POST "http://localhost:4000/api/config/set" \
-H "Content-Type: application/json" \
-d '{
  "key": "binance.use_env_vars",
  "value": true
}'
```

### 6. 批量API配置

```yaml
# api-config-batch.yaml
description: "批量配置交易所API"
operations:
  - name: "配置Binance API"
    command: ["curl", "-X", "POST", "http://localhost:4005/api/config/binance", 
              "-H", "Content-Type: application/json",
              "-d", '{"api_key":"KEY1","api_secret":"SECRET1","live_trading":true}']
    delay: 2
    
  - name: "配置OKX API"  
    command: ["curl", "-X", "POST", "http://localhost:4005/api/config/okx",
              "-H", "Content-Type: application/json", 
              "-d", '{"api_key":"KEY2","api_secret":"SECRET2","live_trading":true}']
    delay: 2
    
  - name: "验证所有API配置"
    command: ["curl", "-s", "http://localhost:4001/api/exchanges/status"]
    delay: 1
```

执行批量配置：`python3 batch-operations.py api-config-batch.yaml`

## 💰 费用管理功能演示

### 1. 查看所有交易所费率对比

```bash
# 查看所有交易所费率概览
python3 arbitrage-cli-controller.py fees list

# 输出示例:
# 📊 交易所费率概览 (4 个交易所):
#   BINANCE  | Maker: 0.100% | Taker: 0.100% | 平均: 0.100% 🟢 低
#   OKX      | Maker: 0.080% | Taker: 0.100% | 平均: 0.090% 🟢 低  
#   HUOBI    | Maker: 0.200% | Taker: 0.200% | 平均: 0.200% 🔴 高
#   BYBIT    | Maker: 0.100% | Taker: 0.100% | 平均: 0.100% 🟢 低
```

### 2. 查看单个交易所详细费率

```bash
# 查看Binance详细费率信息，包括VIP等级
python3 arbitrage-cli-controller.py fees list binance

# 输出示例:
# 交易所: BINANCE
# 基础Maker费率: 0.100%
# 基础Taker费率: 0.100%
# 
# 🎖️ VIP等级费率:
#   等级  0: Maker 0.100% | Taker 0.100% | 要求: < $50,000 30-day volume
#   等级  1: Maker 0.090% | Taker 0.100% | 要求: > $50,000 30-day volume
#   等级  2: Maker 0.080% | Taker 0.100% | 要求: > $500,000 30-day volume
```

### 3. 交易费用计算

```bash
# 计算10000美元在Binance交易BTCUSDT的费用
python3 arbitrage-cli-controller.py fees calculate 10000 binance BTCUSDT

# 输出示例:
# 🧮 交易费用计算:
# 交易金额: $10,000.00 | 交易所: BINANCE | 交易对: BTCUSDT
# ============================================================
# 💳 费用明细:
#   Maker订单费用: $10.0000
#   Taker订单费用: $10.0000
# 
# 📈 盈利分析 (假设1%价差):
#   Maker净利润: $90.0000
#   Taker净利润: $90.0000
# 
# ⚖️ 盈亏平衡点: 0.100% 价差
# ✅ 建议: 使用Maker订单可获得正收益
```

### 4. 套利费用分析

```bash
# 分析BTCUSDT套利机会的费用成本
python3 arbitrage-cli-controller.py fees arbitrage-analysis BTCUSDT 5000

# 输出示例:
# 🔄 BTCUSDT 套利费用分析:
# 分析金额: $5,000.00
# ================================================================================
# 💡 套利机会分析 (6 个交易对组合):
#  1. ✅ binance <-> okx       | 费用: $10.0000 (0.200%) | 盈亏平衡: 0.200% | 推荐
#  2. ✅ binance <-> bybit     | 费用: $10.0000 (0.200%) | 盈亏平衡: 0.200% | 推荐
#  3. ⚠️ binance <-> huobi     | 费用: $15.0000 (0.300%) | 盈亏平衡: 0.300% | 谨慎
#  4. ✅ okx <-> bybit         | 费用: $10.0000 (0.200%) | 盈亏平衡: 0.200% | 推荐
# 
# 🎯 交易建议:
#   最优组合: binance <-> okx
#   最低费用: $10.0000 (0.200%)
#   盈亏平衡: 0.200% 价差
```

### 5. 费率比较和排名

```bash
# 比较各交易所的BTCUSDT费率
python3 arbitrage-cli-controller.py fees compare BTCUSDT

# 输出示例:
# 📊 BTCUSDT 费率比较:
# ================================================================================
# 📈 费率排行 (从低到高):
#  1. 🥇 OKX      | Maker: 0.080% | Taker: 0.100% | 平均: 0.090% | 得分: 95.0
#  2. 🥈 BINANCE  | Maker: 0.100% | Taker: 0.100% | 平均: 0.100% | 得分: 90.0
#  3. 🥉 BYBIT    | Maker: 0.100% | Taker: 0.100% | 平均: 0.100% | 得分: 90.0
#  4. 🏅 HUOBI    | Maker: 0.200% | Taker: 0.200% | 平均: 0.200% | 得分: 80.0
# 
# 🎯 推荐选择:
#   最低费率: OKX (平均 0.090%)
#   最高费率: HUOBI (平均 0.200%)
#   💰 费率差异: 0.110% (选择最优可节省费用)
```

## 🛠️ 高级用法

### 1. 自定义批量操作

```bash
# 进入交互式批量操作构建器
python3 batch-operations.py --interactive

# 创建示例配置文件
python3 batch-operations.py --create-samples
```

### 2. 实时监控脚本

```bash
# 持续监控系统状态（每30秒检查一次）
while true; do 
    python3 arbitrage-cli-controller.py system status
    sleep 30
done

# 持续性能监控
python3 arbitrage-cli-controller.py monitor performance --duration 3600
```

### 3. 日志分析

```bash
# 查看错误日志
python3 arbitrage-cli-controller.py logs tail all 1000 | grep -i error

# 查看特定服务日志
python3 arbitrage-cli-controller.py logs tail strategy 200
```

## 🔍 故障排除

### 1. 服务启动问题

```bash
# 检查所有服务端口状态
ss -ln | grep -E ":(3000|400[0-8])"

# 如果统一网关(3000)未运行，启动它：
cd /home/ubuntu/5.1xitong/5.1系统 && ./target/release/unified-gateway &

# 如果交易服务(4005)未运行，启动它：
cd /home/ubuntu/5.1xitong/5.1系统/trading-service && TRADING_SERVICE_PORT=4005 ./target/release/trading-service &

# 检查服务启动日志
tail -f /home/ubuntu/5.1xitong/logs/*.log

# 一键启动所有缺失服务
./quick-commands.sh start-missing
```

### 2. 端口冲突检查

```bash
# 检查端口占用情况
ss -tlnp | grep -E ":(3000|4008)" 

# 如果端口被占用，查找进程ID
lsof -i :3000
lsof -i :4008

# 强制杀死占用进程
sudo kill -9 <PID>
```

### 3. 服务连接问题

```bash
# 检查服务端口状态
ss -tlnp | grep -E ':(3000|400[0-8])'

# 测试API连通性
curl -s http://localhost:3000/health
curl -s http://localhost:4008/health

# 验证服务响应
curl -s http://localhost:4001/health | jq .
curl -s http://localhost:4003/health | jq .
```

### 2. 权限问题

```bash
# 确保脚本有执行权限
chmod +x *.sh *.py

# 检查日志目录权限
ls -la /home/ubuntu/5.1xitong/logs/
```

### 3. Python依赖问题

```bash
# 安装必需的Python包
pip3 install pyyaml requests

# 检查Python版本
python3 --version
```

## 📊 系统架构图

```
┌─────────────────────────────────────────────────────────┐
│                命令行控制器                               │
├─────────────────┬─────────────────┬─────────────────────┤
│  主控制器        │  快速命令        │  批量操作            │
│  (Python)       │  (Shell)        │  (Python)           │
└─────────────────┼─────────────────┼─────────────────────┘
                  │                 │
        ┌─────────▼─────────────────▼─────────┐
        │         统一API网关                 │
        │         (端口: 3000)               │
        └─────────┬───────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼───┐ ┌───────▼───────┐ ┌───▼───┐
│日志服务│ │  策略服务      │ │清洗服务│
│ 4001  │ │    4003       │ │ 4002  │
└───────┘ └───────────────┘ └───────┘
┌─────────┐ ┌─────────┐ ┌─────────┐ 
│性能服务 │ │交易服务 │ │AI模型   │
│ 4004   │ │  4005  │ │ 4006   │
└─────────┘ └─────────┘ └─────────┘
          ┌─────────┐
          │配置服务 │
          │ 4000   │
          └─────────┘
```

### 📋 端口映射表

| 服务名称 | 端口 | 状态检查 | 功能描述 |
|----------|------|----------|----------|
| 统一网关 | 3000 | `curl -s http://localhost:3000/health` | API统一入口网关 |
| 配置服务 | 4000 | `curl -s http://localhost:4000/health` | 系统配置管理 |
| 日志服务 | 4001 | `curl -s http://localhost:4001/health` | 日志收集和查询 |
| 清洗服务 | 4002 | `curl -s http://localhost:4002/health` | 数据清洗处理 |
| 策略服务 | 4003 | `curl -s http://localhost:4003/health` | 策略执行和管理 |
| 性能服务 | 4004 | `curl -s http://localhost:4004/health` | 性能监控分析 |
| AI模型服务 | 4006 | `curl -s http://localhost:4006/health` | AI模型训练推理 |
| 交易服务 | 4005 | `curl -s http://localhost:4005/health` | 真实交易执行 |

## 📞 技术支持

如果在使用过程中遇到问题：

1. **查看日志**: `python3 arbitrage-cli-controller.py logs tail all 100`
2. **健康检查**: `./quick-commands.sh health-check`  
3. **系统状态**: `python3 arbitrage-cli-controller.py system status`

---

**🎉 现在你可以通过简单的命令行完全控制套利系统5.1的所有功能！**