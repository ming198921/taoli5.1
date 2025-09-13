#!/bin/bash

echo "🛑 停止5.1套利系统所有服务"
echo "================================"

# 停止所有服务
services=(
    "gateway:统一网关"
    "logging:日志服务"
    "cleaning:清洗服务"
    "strategy:策略服务"
    "performance:性能服务"
    "trading:交易服务"
    "ai-model:AI模型服务"
    "config:配置服务"
)

for service_info in "${services[@]}"; do
    IFS=':' read -r service_name display_name <<< "$service_info"
    pid_file=".$service_name.pid"
    
    if [ -f "$pid_file" ]; then
        pid=$(cat "$pid_file")
        if ps -p $pid > /dev/null 2>&1; then
            echo "   停止 $display_name (PID: $pid)..."
            kill $pid
            sleep 1
            if ps -p $pid > /dev/null 2>&1; then
                echo "   强制停止 $display_name..."
                kill -9 $pid
            fi
        else
            echo "   $display_name 已经停止"
        fi
        rm -f "$pid_file"
    else
        echo "   未找到 $display_name 的PID文件"
    fi
done

# 强制杀死所有相关进程
echo ""
echo "🧹 清理残余进程..."
pkill -f "unified-gateway" 2>/dev/null || true
pkill -f "logging-service" 2>/dev/null || true
pkill -f "cleaning-service" 2>/dev/null || true
pkill -f "strategy-service" 2>/dev/null || true
pkill -f "performance-service" 2>/dev/null || true
pkill -f "trading-service" 2>/dev/null || true
pkill -f "ai-model-service" 2>/dev/null || true
pkill -f "config-service" 2>/dev/null || true

echo ""
echo "✅ 所有服务已停止"
echo ""
echo "🔍 端口状态检查:"
for port in 3000 4001 4002 4003 4004 4005 4006 4007; do
    if lsof -i :$port > /dev/null 2>&1; then
        echo "   端口 $port: ❌ 仍被占用"
    else
        echo "   端口 $port: ✅ 已释放"
    fi
done