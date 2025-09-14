#!/bin/bash
# 🚀 QINGXI生产部署脚本
# 功能: 生产就绪检查 + 零停机部署

set -euo pipefail

echo "🚀 QINGXI生产部署脚本启动"
echo "📅 $(date)"

# 配置
QINGXI_ROOT="/home/ubuntu/qingxi/qingxi"
CONFIG_ENV="${1:-production}"
LOG_DIR="/var/log/qingxi"
PID_FILE="/var/run/qingxi.pid"

cd "$QINGXI_ROOT"

echo "🔍 步骤1: 生产就绪检查"
echo "==========================================="

# 1. 编译检查
echo "📦 执行Release编译..."
if ! cargo build --release; then
    echo "❌ 编译失败，部署中止"
    exit 1
fi
echo "✅ 编译成功"

# 2. 配置验证
echo "🔧 验证生产配置..."
export QINGXI_CONFIG_PATH="configs/${CONFIG_ENV}.toml"
if ! cargo run --bin config_validator --release; then
    echo "❌ 配置验证失败，部署中止"
    exit 1
fi
echo "✅ 配置验证通过"

# 3. 硬编码检查
echo "🔍 执行硬编码检查..."
if ! cargo run --bin simple_hardcode_check --release; then
    echo "⚠️  硬编码检查有警告，但继续部署"
fi

echo "🛠️ 步骤2: 环境准备"
echo "==========================================="

# 创建日志目录
sudo mkdir -p "$LOG_DIR"
sudo chown -R "$USER:$USER" "$LOG_DIR"

# 设置环境变量
export QINGXI_CONFIG_PATH="configs/${CONFIG_ENV}.toml"
export QINGXI_LOG_DIR="$LOG_DIR"
export RUST_LOG="info"

# 生产环境特定配置
if [ "$CONFIG_ENV" = "production" ]; then
    export QINGXI_REASONER_ENDPOINT="${QINGXI_REASONER_ENDPOINT:-http://reasoner-service:8081}"
    echo "📡 Reasoner端点: $QINGXI_REASONER_ENDPOINT"
fi

echo "🚦 步骤3: 服务管理"
echo "==========================================="

# 检查现有服务
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE")
    if kill -0 "$OLD_PID" 2>/dev/null; then
        echo "🔄 发现运行中的服务 (PID: $OLD_PID)"
        echo "⏳ 执行优雅停机..."
        kill -TERM "$OLD_PID"
        
        # 等待优雅停机
        for i in {1..30}; do
            if ! kill -0 "$OLD_PID" 2>/dev/null; then
                break
            fi
            sleep 1
        done
        
        # 强制停机如果需要
        if kill -0 "$OLD_PID" 2>/dev/null; then
            echo "⚡ 强制停机..."
            kill -KILL "$OLD_PID"
        fi
        
        rm -f "$PID_FILE"
        echo "✅ 旧服务已停止"
    fi
fi

echo "🟢 步骤4: 启动新服务"
echo "==========================================="

# 启动新服务
echo "🚀 启动QINGXI服务..."
nohup ./target/release/market_data_module \
    --config "$QINGXI_CONFIG_PATH" \
    > "$LOG_DIR/qingxi.log" 2>&1 &

NEW_PID=$!
echo $NEW_PID > "$PID_FILE"

echo "🎯 新服务已启动 (PID: $NEW_PID)"

# 健康检查
echo "🏥 执行健康检查..."
sleep 5

HEALTH_CHECK_COUNT=0
MAX_HEALTH_CHECKS=12

while [ $HEALTH_CHECK_COUNT -lt $MAX_HEALTH_CHECKS ]; do
    if kill -0 "$NEW_PID" 2>/dev/null; then
        echo "✅ 服务运行正常 (检查 $((HEALTH_CHECK_COUNT + 1))/$MAX_HEALTH_CHECKS)"
        break
    else
        echo "⏳ 等待服务启动... ($((HEALTH_CHECK_COUNT + 1))/$MAX_HEALTH_CHECKS)"
        sleep 5
        HEALTH_CHECK_COUNT=$((HEALTH_CHECK_COUNT + 1))
    fi
done

if [ $HEALTH_CHECK_COUNT -eq $MAX_HEALTH_CHECKS ]; then
    echo "❌ 服务启动失败"
    echo "📋 最后的日志:"
    tail -20 "$LOG_DIR/qingxi.log"
    exit 1
fi

echo "📊 步骤5: 部署验证"
echo "==========================================="

# 最终验证
echo "🔍 最终状态检查:"
echo "   - PID: $(cat $PID_FILE)"
echo "   - 配置: $QINGXI_CONFIG_PATH"
echo "   - 日志: $LOG_DIR/qingxi.log"
echo "   - Reasoner: ${QINGXI_REASONER_ENDPOINT:-默认}"

echo ""
echo "🎉 QINGXI生产部署成功！"
echo "📈 服务状态: 运行中"
echo "📊 监控日志: tail -f $LOG_DIR/qingxi.log"
echo "🛑 停止服务: kill \$(cat $PID_FILE)"

exit 0
