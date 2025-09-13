#!/bin/bash

# 套利监控系统状态检查脚本
echo "🎯 套利监控系统状态检查"
echo "=================================================="

# 检查NATS服务器
echo -n "🔗 NATS服务器: "
if pgrep -f "nats-server" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查QingXi系统
echo -n "📊 QingXi系统: "
if pgrep -f "market_data_module" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查桥接器
echo -n "🌉 QingXi桥接器: "
if pgrep -f "qingxi_bridge" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查套利监控器
echo -n "🎯 套利监控器: "
if pgrep -f "arbitrage_monitor" > /dev/null; then
    PID=$(pgrep -f "arbitrage_monitor")
    echo "✅ 运行中 (PID: $PID)"
else
    echo "❌ 未运行"
fi

echo "=================================================="

# 检查NATS主题数据流
echo "📡 检查NATS数据流 (5秒采样)..."
echo "如果看到数据流，说明系统工作正常："
timeout 5 nats sub "market.data.normalized.*.*" 2>/dev/null | head -5

echo ""
echo "💡 提示："
echo "  - 如果所有服务都显示 ✅，系统运行正常"
echo "  - 如果看到数据流，说明有套利数据在传输"
echo "  - 查看监控输出: 连接到运行监控的终端" 

# 套利监控系统状态检查脚本
echo "🎯 套利监控系统状态检查"
echo "=================================================="

# 检查NATS服务器
echo -n "🔗 NATS服务器: "
if pgrep -f "nats-server" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查QingXi系统
echo -n "📊 QingXi系统: "
if pgrep -f "market_data_module" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查桥接器
echo -n "🌉 QingXi桥接器: "
if pgrep -f "qingxi_bridge" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查套利监控器
echo -n "🎯 套利监控器: "
if pgrep -f "arbitrage_monitor" > /dev/null; then
    PID=$(pgrep -f "arbitrage_monitor")
    echo "✅ 运行中 (PID: $PID)"
else
    echo "❌ 未运行"
fi

echo "=================================================="

# 检查NATS主题数据流
echo "📡 检查NATS数据流 (5秒采样)..."
echo "如果看到数据流，说明系统工作正常："
timeout 5 nats sub "market.data.normalized.*.*" 2>/dev/null | head -5

echo ""
echo "💡 提示："
echo "  - 如果所有服务都显示 ✅，系统运行正常"
echo "  - 如果看到数据流，说明有套利数据在传输"
echo "  - 查看监控输出: 连接到运行监控的终端" 

# 套利监控系统状态检查脚本
echo "🎯 套利监控系统状态检查"
echo "=================================================="

# 检查NATS服务器
echo -n "🔗 NATS服务器: "
if pgrep -f "nats-server" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查QingXi系统
echo -n "📊 QingXi系统: "
if pgrep -f "market_data_module" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查桥接器
echo -n "🌉 QingXi桥接器: "
if pgrep -f "qingxi_bridge" > /dev/null; then
    echo "✅ 运行中"
else
    echo "❌ 未运行"
fi

# 检查套利监控器
echo -n "🎯 套利监控器: "
if pgrep -f "arbitrage_monitor" > /dev/null; then
    PID=$(pgrep -f "arbitrage_monitor")
    echo "✅ 运行中 (PID: $PID)"
else
    echo "❌ 未运行"
fi

echo "=================================================="

# 检查NATS主题数据流
echo "📡 检查NATS数据流 (5秒采样)..."
echo "如果看到数据流，说明系统工作正常："
timeout 5 nats sub "market.data.normalized.*.*" 2>/dev/null | head -5

echo ""
echo "💡 提示："
echo "  - 如果所有服务都显示 ✅，系统运行正常"
echo "  - 如果看到数据流，说明有套利数据在传输"
echo "  - 查看监控输出: 连接到运行监控的终端" 
 
 
 