#!/bin/bash

# 策略模块和风控子模块完整集成测试启动脚本

set -e  # 遇到错误立即退出

echo "🎯 策略模块和风控子模块完整集成测试"
echo "=================================="

# 检查Python环境
echo "🔍 检查Python环境..."
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3未安装"
    exit 1
fi

# 安装Python依赖
echo "📦 安装Python依赖..."
pip3 install --quiet py-cpuinfo asyncio-nats-client aiohttp psutil pyyaml || {
    echo "⚠️  使用备用安装方法..."
    python3 -m pip install --user py-cpuinfo asyncio-nats-client aiohttp psutil pyyaml
}

# 检查Rust环境
echo "🔍 检查Rust环境..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo未安装"
    exit 1
fi

# 检查NATS服务器
echo "🔍 检查NATS服务器..."
if ! pgrep -f "nats-server" > /dev/null; then
    echo "⚠️  NATS服务器未运行，尝试启动..."
    if command -v nats-server &> /dev/null; then
        nats-server --port 4222 --jetstream &
        sleep 3
        echo "✅ NATS服务器已启动"
    else
        echo "❌ NATS服务器未安装，请先安装nats-server"
        exit 1
    fi
else
    echo "✅ NATS服务器正在运行"
fi

# 编译项目
echo "🔧 编译策略模块..."
cargo build --release --bin arbitrage_monitor_simple

echo "🔧 编译orchestrator..."
cd orchestrator
cargo build --release
cd ..

# 设置CPU亲和性
echo "🔧 设置CPU亲和性..."
export CELUE_CPU_AFFINITY="0,1,2,3"

# 运行集成测试
echo "🚀 开始集成测试..."
echo "测试参数："
echo "  - 测试时长: 60秒"
echo "  - 数据速率: 每秒10条消息"
echo "  - 预期消息总数: 600条"
echo "  - CPU亲和性: 核心0-3"
echo ""

# 创建日志目录
mkdir -p test_logs

# 运行测试并记录日志
python3 test_strategy_integration.py 2>&1 | tee test_logs/integration_test_$(date +%Y%m%d_%H%M%S).log

# 检查测试结果
if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo ""
    echo "✅ 集成测试完成！"
    echo "📋 测试结果已保存到 test_logs/ 目录"
    echo ""
    echo "📊 测试覆盖的功能："
    echo "  ✅ 策略模块启动和运行状态"
    echo "  ✅ 风控模块发现和处理问题"
    echo "  ✅ SIMD和CPU亲和性完整触发"
    echo "  ✅ 1秒10条数据标准测试"
    echo ""
else
    echo ""
    echo "❌ 集成测试失败！"
    echo "📋 请检查 test_logs/ 目录中的日志文件"
    echo ""
    exit 1
fi 

# 策略模块和风控子模块完整集成测试启动脚本

set -e  # 遇到错误立即退出

echo "🎯 策略模块和风控子模块完整集成测试"
echo "=================================="

# 检查Python环境
echo "🔍 检查Python环境..."
if ! command -v python3 &> /dev/null; then
    echo "❌ Python3未安装"
    exit 1
fi

# 安装Python依赖
echo "📦 安装Python依赖..."
pip3 install --quiet py-cpuinfo asyncio-nats-client aiohttp psutil pyyaml || {
    echo "⚠️  使用备用安装方法..."
    python3 -m pip install --user py-cpuinfo asyncio-nats-client aiohttp psutil pyyaml
}

# 检查Rust环境
echo "🔍 检查Rust环境..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo未安装"
    exit 1
fi

# 检查NATS服务器
echo "🔍 检查NATS服务器..."
if ! pgrep -f "nats-server" > /dev/null; then
    echo "⚠️  NATS服务器未运行，尝试启动..."
    if command -v nats-server &> /dev/null; then
        nats-server --port 4222 --jetstream &
        sleep 3
        echo "✅ NATS服务器已启动"
    else
        echo "❌ NATS服务器未安装，请先安装nats-server"
        exit 1
    fi
else
    echo "✅ NATS服务器正在运行"
fi

# 编译项目
echo "🔧 编译策略模块..."
cargo build --release --bin arbitrage_monitor_simple

echo "🔧 编译orchestrator..."
cd orchestrator
cargo build --release
cd ..

# 设置CPU亲和性
echo "🔧 设置CPU亲和性..."
export CELUE_CPU_AFFINITY="0,1,2,3"

# 运行集成测试
echo "🚀 开始集成测试..."
echo "测试参数："
echo "  - 测试时长: 60秒"
echo "  - 数据速率: 每秒10条消息"
echo "  - 预期消息总数: 600条"
echo "  - CPU亲和性: 核心0-3"
echo ""

# 创建日志目录
mkdir -p test_logs

# 运行测试并记录日志
python3 test_strategy_integration.py 2>&1 | tee test_logs/integration_test_$(date +%Y%m%d_%H%M%S).log

# 检查测试结果
if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo ""
    echo "✅ 集成测试完成！"
    echo "📋 测试结果已保存到 test_logs/ 目录"
    echo ""
    echo "📊 测试覆盖的功能："
    echo "  ✅ 策略模块启动和运行状态"
    echo "  ✅ 风控模块发现和处理问题"
    echo "  ✅ SIMD和CPU亲和性完整触发"
    echo "  ✅ 1秒10条数据标准测试"
    echo ""
else
    echo ""
    echo "❌ 集成测试失败！"
    echo "📋 请检查 test_logs/ 目录中的日志文件"
    echo ""
    exit 1
fi 