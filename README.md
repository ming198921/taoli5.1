# 套利系统5.1 延迟测试框架

## 概述

本测试框架用于测量套利系统5.1向各大交易所（币安、火币、OKEx）发送订单的网络延迟。框架不修改套利系统代码，通过模拟交易所服务器和网络拦截来测量真实延迟。

## 功能特性

- **多交易所支持**：支持币安(Binance)、火币(Huobi)、OKEx三大交易所
- **精确延迟测量**：毫秒级延迟测量精度
- **多种测试模式**：
  - 模拟器模式：使用模拟交易所服务器测试
  - 拦截器模式：拦截真实网络请求测试
  - 混合模式：同时使用两种模式
- **详细报告生成**：
  - 文本报告：包含统计摘要和建议
  - JSON报告：原始数据和统计信息
  - 图表可视化：延迟分布图、时间序列图、对比箱线图

## 安装

```bash
# 安装依赖
pip install -r requirements.txt
```

## 使用方法

### 1. 快速测试（模拟器模式）

```bash
# 运行60秒测试
python run_latency_test.py --mode simulator --duration 60
```

### 2. 启动模拟交易所服务器

```bash
# 在单独终端运行
python exchange_simulator.py
```

服务器将在以下端口启动：
- Binance: http://127.0.0.1:8881
- Huobi: http://127.0.0.1:8882
- OKEx: http://127.0.0.1:8883

### 3. 配置套利系统5.1

修改套利系统的配置文件，将交易所API地址改为模拟服务器地址：

```json
{
  "exchanges": {
    "binance": {
      "api_url": "http://127.0.0.1:8881"
    },
    "huobi": {
      "api_url": "http://127.0.0.1:8882"
    },
    "okex": {
      "api_url": "http://127.0.0.1:8883"
    }
  }
}
```

### 4. 运行测试

```bash
# 基础测试
python test_framework.py

# 完整测试流程
python run_latency_test.py
```

## 文件说明

- `test_framework.py`: 主测试框架，负责发送测试请求和收集延迟数据
- `exchange_simulator.py`: 模拟交易所服务器，模拟真实交易所的响应
- `network_interceptor.py`: 网络请求拦截器，用于监控真实请求
- `report_generator.py`: 报告生成器，生成文本、JSON报告和图表
- `run_latency_test.py`: 主执行脚本，协调整个测试流程

## 测试报告

测试完成后会生成以下报告：

1. **文本报告** (`latency_report_YYYYMMDD_HHMMSS.txt`)
   - 延迟汇总表
   - 详细统计信息（P25、P50、P75、P90、P95、P99）
   - 性能建议

2. **JSON报告** (`latency_report_YYYYMMDD_HHMMSS.json`)
   - 原始测试数据
   - 统计摘要

3. **图表** (`reports_YYYYMMDD_HHMMSS/`)
   - `latency_distribution.png`: 延迟分布直方图
   - `latency_timeline.png`: 延迟时间序列图
   - `latency_boxplot.png`: 交易所延迟对比箱线图

## 延迟评估标准

- **优秀**: P95 < 10ms
- **良好**: P95 < 20ms  
- **一般**: P95 < 50ms
- **需优化**: P95 >= 50ms

## 注意事项

1. 测试期间套利系统应配置为测试模式，避免真实交易
2. 确保网络连接稳定，避免其他程序占用大量带宽
3. 建议在不同时间段多次测试，获取更全面的延迟数据
4. 如需测试真实交易所延迟，请使用拦截器模式

## 交易所API配置

### 添加真实交易所API密钥

要配置5.1系统使用真实的交易所API进行交易，可以通过以下方式添加API密钥：

#### 方法1：通过统一网关配置
```bash
curl -X POST "http://localhost:4001/api/config/exchange" \
-H "Content-Type: application/json" \
-d '{
  "name": "binance",
  "api_key": "YOUR_API_KEY",
  "api_secret": "YOUR_API_SECRET",
  "sandbox_mode": false,
  "enabled": true
}'
```

#### 方法2：通过交易服务配置
```bash
curl -X POST "http://localhost:4008/api/config/binance" \
-H "Content-Type: application/json" \
-d '{
  "api_key": "YOUR_API_KEY",
  "api_secret": "YOUR_API_SECRET",
  "live_trading": true
}'
```

#### 方法3：通过配置服务设置
```bash
curl -X POST "http://localhost:4000/api/config/set" \
-H "Content-Type: application/json" \
-d '{
  "key": "binance.api_key",
  "value": "YOUR_API_KEY"
}'

curl -X POST "http://localhost:4000/api/config/set" \
-H "Content-Type: application/json" \
-d '{
  "key": "binance.api_secret",
  "value": "YOUR_API_SECRET"
}'
```

### 验证配置
```bash
# 检查配置是否成功
curl -s "http://localhost:4008/api/config/binance" | jq .
```

### 服务端口映射
- 统一网关: http://localhost:4001
- 配置服务: http://localhost:4000
- 策略服务: http://localhost:4003
- 交易服务: http://localhost:4008

## 高级配置

### 自定义延迟模拟

编辑 `exchange_simulator.py` 中的延迟配置：

```python
self.latency_profiles = {
    'binance': {'base': 5, 'variance': 2},   # 5±2ms
    'huobi': {'base': 8, 'variance': 3},      # 8±3ms
    'okex': {'base': 6, 'variance': 2.5}      # 6±2.5ms
}
```

### 调整测试参数

编辑 `test_framework.py` 中的测试配置：

```python
order_interval = 0.1  # 订单发送间隔（秒）
test_duration = 60    # 测试持续时间（秒）
```

## 故障排除

1. **端口占用错误**
   ```bash
   # 检查端口占用
   netstat -an | grep 888[1-3]
   # 杀死占用进程
   kill -9 <PID>
   ```

2. **连接超时**
   - 检查防火墙设置
   - 确认套利系统配置正确
   - 验证模拟服务器正在运行

3. **matplotlib图表生成失败**
   ```bash
   # 安装GUI后端
   pip install PyQt5
   # 或使用无头模式
   export MPLBACKEND=Agg
   ```