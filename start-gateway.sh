#!/bin/bash
# 统一网关启动脚本

set -e

echo "🌐 启动统一API网关..."

# 检查是否已经在运行
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo "✅ 统一网关已在运行 (端口 3000)"
    exit 0
fi

# 进入网关目录
cd /home/ubuntu/5.1xitong/5.1系统/unified-gateway

# 检查可执行文件
if [ ! -f "target/release/unified-gateway" ]; then
    echo "🔨 编译统一网关..."
    cargo build --release --quiet
    
    if [ $? -eq 0 ]; then
        echo "✅ 统一网关编译成功"
    else
        echo "❌ 统一网关编译失败"
        exit 1
    fi
fi

# 启动网关
echo "🚀 启动统一网关 (端口 3000)..."
export RUST_LOG=info
export GATEWAY_PORT=3000

nohup ./target/release/unified-gateway > ../logs/unified-gateway.log 2>&1 &
gateway_pid=$!

echo "📋 网关进程 PID: $gateway_pid"

# 等待启动
echo "⏳ 等待网关启动..."
sleep 5

# 健康检查
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo "✅ 统一网关启动成功"
    echo "🌐 访问地址: http://localhost:3000"
    echo "📋 API文档: http://localhost:3000/api"
    echo ""
    echo "💡 管理命令:"
    echo "   查看状态: curl http://localhost:3000/health"
    echo "   查看日志: tail -f ../logs/unified-gateway.log"
    echo "   停止网关: pkill -f unified-gateway"
else
    echo "❌ 统一网关启动失败"
    echo "🔍 查看日志: tail -20 ../logs/unified-gateway.log"
    exit 1
fi