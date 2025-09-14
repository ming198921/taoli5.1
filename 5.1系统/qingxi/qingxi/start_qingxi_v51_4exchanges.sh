#!/bin/bash

# QingXi V5.1 系统启动脚本 - 5交易所50币种配置
# 🚀 V3+O1 数据清洗系统

echo "🚀 Starting QingXi V5.1 System with 5 Exchanges (50 symbols each)"
echo "📊 Exchanges: Binance, OKX, Bybit, Gate.io, Huobi"
echo "💱 Total symbols: 250 (50 per exchange)"
echo "🧹 Data cleaning: V3+O1 optimized cleaning engine"

# 设置环境变量
export RUST_LOG=info
export QINGXI_CONFIG_PATH="configs/four_exchanges_50_symbols_optimized.toml"

# 创建必要的目录
echo "📁 Creating necessary directories..."
mkdir -p cache/l2_cleaned_data
mkdir -p cache/l3_processed_data
mkdir -p logs/system
mkdir -p logs/audit
mkdir -p logs/performance

echo "✅ Directories created:"
echo "   - cache/l2_cleaned_data (V3+O1 清洗后数据)"
echo "   - cache/l3_processed_data (最终处理数据)"
echo "   - logs/system (系统日志)"
echo "   - logs/audit (审计日志)"
echo "   - logs/performance (性能日志)"

# 检查配置文件
if [ ! -f "$QINGXI_CONFIG_PATH" ]; then
    echo "❌ Configuration file not found: $QINGXI_CONFIG_PATH"
    exit 1
fi

echo "✅ Configuration file found: $QINGXI_CONFIG_PATH"

# 检查并启动 NATS 服务器
echo "🔍 Checking NATS server..."
if ! pgrep -x "nats-server" > /dev/null; then
    echo "🚀 Starting NATS server..."
                    if command -v nats-server &> /dev/null; then
                    nats-server --port 4222 &
                    echo "✅ NATS server started on port 4222"
                    sleep 3
                else
                    echo "❌ NATS server not found. Installing..."
                    # 下载并安装 NATS
                    wget https://github.com/nats-io/nats-server/releases/download/v2.10.4/nats-server-v2.10.4-linux-amd64.tar.gz
                    tar -xzf nats-server-v2.10.4-linux-amd64.tar.gz
                    sudo mv nats-server-v2.10.4-linux-amd64/nats-server /usr/local/bin/
                    rm -rf nats-server-v2.10.4-linux-amd64*
                    echo "✅ NATS server installed"
                    nats-server --port 4222 &
                    echo "✅ NATS server started on port 4222"
                    sleep 3
                fi
else
    echo "✅ NATS server already running"
fi

# 验证 NATS 连接
echo "🔍 Verifying NATS connection..."
timeout 5s bash -c 'until nc -z localhost 4222; do sleep 1; done' && echo "✅ NATS server is accessible" || echo "❌ NATS server connection failed"

# 启动系统
echo "🚀 Starting QingXi V5.1 system..."
echo "📡 WebSocket connections will be established for 250 symbols across 5 exchanges"
echo "🧹 V3+O1 cleaning engine will process all incoming data"
echo "💾 Cleaned data will be stored in: cache/l2_cleaned_data/"
echo "📈 Final processed data will be stored in: cache/l3_processed_data/"

# 创建日志目录并启动QingXi，日志保存到指定文件
mkdir -p logs
echo "📋 QingXi日志将保存到: logs/qingxi_runtime.log"
cargo run --release --bin market_data_module > logs/qingxi_runtime.log 2>&1

echo "🛑 QingXi V5.1 system stopped" 