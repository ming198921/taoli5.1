#!/bin/bash

# 实时套利监控启动脚本
echo "🎯 启动实时套利监控系统..."
echo "=================================================="
echo "监控功能："
echo "  🔄 跨交易所套利机会检测"
echo "  🔺 同交易所三角套利检测"
echo "  📊 实时价差分析"
echo "  💰 盈利机会统计"
echo "=================================================="
echo ""

# 检查NATS服务器是否运行
echo "🔍 检查NATS服务器状态..."
if ! pgrep -f "nats-server" > /dev/null; then
    echo "⚠️  NATS服务器未运行，正在启动..."
    nats-server --port 4222 --jetstream &
    sleep 3
else
    echo "✅ NATS服务器正在运行"
fi

# 检查QingXi系统是否运行
echo "🔍 检查QingXi系统状态..."
if ! pgrep -f "market_data_module" > /dev/null; then
    echo "⚠️  QingXi系统未运行，请先启动QingXi系统"
    echo "   运行命令: cd /home/ubuntu/qingxi && ./start_qingxi_v51_4exchanges.sh"
    exit 1
else
    echo "✅ QingXi系统正在运行"
fi

# 启动套利监控
echo "🚀 启动套利监控程序..."
cd /home/ubuntu/celue

# 先编译（如果需要）
echo "📦 编译套利监控程序..."
cargo build --bin arbitrage_monitor

# 启动监控
echo "▶️  运行套利监控..."
cargo run --bin arbitrage_monitor 

# 实时套利监控启动脚本
echo "🎯 启动实时套利监控系统..."
echo "=================================================="
echo "监控功能："
echo "  🔄 跨交易所套利机会检测"
echo "  🔺 同交易所三角套利检测"
echo "  📊 实时价差分析"
echo "  💰 盈利机会统计"
echo "=================================================="
echo ""

# 检查NATS服务器是否运行
echo "🔍 检查NATS服务器状态..."
if ! pgrep -f "nats-server" > /dev/null; then
    echo "⚠️  NATS服务器未运行，正在启动..."
    nats-server --port 4222 --jetstream &
    sleep 3
else
    echo "✅ NATS服务器正在运行"
fi

# 检查QingXi系统是否运行
echo "🔍 检查QingXi系统状态..."
if ! pgrep -f "market_data_module" > /dev/null; then
    echo "⚠️  QingXi系统未运行，请先启动QingXi系统"
    echo "   运行命令: cd /home/ubuntu/qingxi && ./start_qingxi_v51_4exchanges.sh"
    exit 1
else
    echo "✅ QingXi系统正在运行"
fi

# 启动套利监控
echo "🚀 启动套利监控程序..."
cd /home/ubuntu/celue

# 先编译（如果需要）
echo "📦 编译套利监控程序..."
cargo build --bin arbitrage_monitor

# 启动监控
echo "▶️  运行套利监控..."
cargo run --bin arbitrage_monitor 

# 实时套利监控启动脚本
echo "🎯 启动实时套利监控系统..."
echo "=================================================="
echo "监控功能："
echo "  🔄 跨交易所套利机会检测"
echo "  🔺 同交易所三角套利检测"
echo "  📊 实时价差分析"
echo "  💰 盈利机会统计"
echo "=================================================="
echo ""

# 检查NATS服务器是否运行
echo "🔍 检查NATS服务器状态..."
if ! pgrep -f "nats-server" > /dev/null; then
    echo "⚠️  NATS服务器未运行，正在启动..."
    nats-server --port 4222 --jetstream &
    sleep 3
else
    echo "✅ NATS服务器正在运行"
fi

# 检查QingXi系统是否运行
echo "🔍 检查QingXi系统状态..."
if ! pgrep -f "market_data_module" > /dev/null; then
    echo "⚠️  QingXi系统未运行，请先启动QingXi系统"
    echo "   运行命令: cd /home/ubuntu/qingxi && ./start_qingxi_v51_4exchanges.sh"
    exit 1
else
    echo "✅ QingXi系统正在运行"
fi

# 启动套利监控
echo "🚀 启动套利监控程序..."
cd /home/ubuntu/celue

# 先编译（如果需要）
echo "📦 编译套利监控程序..."
cargo build --bin arbitrage_monitor

# 启动监控
echo "▶️  运行套利监控..."
cargo run --bin arbitrage_monitor 
 
 
 