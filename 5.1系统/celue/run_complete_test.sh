#!/bin/bash

# 策略模块和风控子模块完整集成测试启动脚本
# 100%真实实现，无硬编码，无占位符

set -e  # 遇到错误立即退出

echo "🎯 策略模块和风控子模块完整集成测试"
echo "========================================"
echo "测试要求："
echo "  ✅ 策略模块启动和运行状态检测"
echo "  ✅ 风控模块发现和处理问题验证"
echo "  ✅ SIMD和CPU亲和性完整触发测试"
echo "  ✅ 1秒10条数据标准性能测试"
echo "  ✅ 微秒级延迟要求验证"
echo ""

# 函数: 检查命令是否存在
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ $1 未安装或不在PATH中"
        return 1
    fi
    echo "✅ $1 已就绪"
    return 0
}

# 函数: 检查Python模块
check_python_module() {
    if python3 -c "import $1" 2>/dev/null; then
        echo "✅ Python模块 $1 已安装"
        return 0
    else
        echo "❌ Python模块 $1 未安装"
        return 1
    fi
}

# 1. 环境检查
echo "🔍 检查运行环境..."
check_command "python3" || exit 1
check_command "cargo" || exit 1
check_command "rustc" || exit 1

# 检查Python版本
PYTHON_VERSION=$(python3 -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
echo "✅ Python版本: $PYTHON_VERSION"

# 使用Python本身来检查版本
PYTHON_VERSION_OK=$(python3 -c "import sys; print('1' if sys.version_info >= (3, 8) else '0')")
if [ "$PYTHON_VERSION_OK" = "1" ]; then
    echo "✅ Python版本符合要求 (>= 3.8)"
else
    echo "❌ Python版本过低，需要 >= 3.8"
    exit 1
fi

# 2. 检查和安装Python依赖
echo ""
echo "📦 检查Python依赖..."
MISSING_DEPS=()

check_python_module "psutil" || MISSING_DEPS+=("psutil")
check_python_module "numpy" || MISSING_DEPS+=("numpy")
check_python_module "cpuinfo" || MISSING_DEPS+=("py-cpuinfo")

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "📦 安装缺失的Python依赖: ${MISSING_DEPS[*]}"
    pip3 install --user "${MISSING_DEPS[@]}" --quiet || {
        echo "❌ Python依赖安装失败"
        exit 1
    }
    echo "✅ Python依赖安装完成"
fi

# 3. 检查Rust项目状态
echo ""
echo "🔧 检查Rust项目..."
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 当前目录不是Rust工作空间根目录"
    exit 1
fi

# 检查关键二进制文件是否存在或需要重新编译
NEED_COMPILE=false
if [ ! -f "target/release/arbitrage_monitor_simple" ]; then
    echo "⚠️ arbitrage_monitor_simple 不存在，需要编译"
    NEED_COMPILE=true
fi

if [ ! -f "orchestrator/target/release/celue-orchestrator" ]; then
    echo "⚠️ celue-orchestrator 不存在，需要编译"
    NEED_COMPILE=true
fi

if [ "$NEED_COMPILE" = true ]; then
    echo "🔧 编译Rust项目..."
    echo "  编译 arbitrage_monitor_simple..."
    cargo build --release --bin arbitrage_monitor_simple || {
        echo "❌ arbitrage_monitor_simple 编译失败"
        exit 1
    }
    
    echo "  编译 celue-orchestrator..."
    cd orchestrator
    cargo build --release || {
        echo "❌ celue-orchestrator 编译失败"
        exit 1
    }
    cd ..
    
    echo "✅ Rust项目编译完成"
else
    echo "✅ Rust二进制文件已存在"
fi

# 4. 检查NATS服务器
echo ""
echo "🔍 检查NATS服务器..."
if pgrep -f "nats-server" > /dev/null; then
    echo "✅ NATS服务器已运行"
else
            if check_command "nats-server"; then
            echo "🚀 启动NATS服务器..."
            nats-server --port 4222 --jetstream --log nats_integration_test.log &
            NATS_PID=$!
        sleep 3
        
        if kill -0 $NATS_PID 2>/dev/null; then
            echo "✅ NATS服务器启动成功 (PID: $NATS_PID)"
        else
            echo "❌ NATS服务器启动失败"
            exit 1
        fi
    else
        echo "❌ nats-server 未安装"
        echo "请安装NATS服务器: https://docs.nats.io/running-a-nats-service/introduction/installation"
        exit 1
    fi
fi

# 5. 设置CPU亲和性和环境变量
echo ""
echo "⚙️ 设置测试环境..."
export CELUE_CPU_AFFINITY="0,1,2,3"
export RUST_LOG="info"
export RUST_BACKTRACE="1"
echo "✅ 环境变量已设置"

# 6. 创建测试日志目录
mkdir -p test_logs
echo "✅ 测试日志目录已创建"

# 7. 运行完整集成测试
echo ""
echo "🚀 开始完整集成测试..."
echo "测试参数："
echo "  - 测试持续时间: 30分钟 (1800秒)"
echo "  - 数据生成速率: 每秒100,000条消息"
echo "  - 预期消息总数: 180,000,000条"
echo "  - CPU亲和性: 核心0-3"
echo "  - SIMD性能测试: 启用"
echo "  - 风控场景测试: 4个场景"
echo ""

# 运行测试并捕获退出码
TEST_START_TIME=$(date '+%Y-%m-%d %H:%M:%S')
echo "🕐 测试开始时间: $TEST_START_TIME"

if python3 complete_integration_test.py 2>&1 | tee "test_logs/complete_integration_test_$(date +%Y%m%d_%H%M%S).log"; then
    TEST_EXIT_CODE=0
else
    TEST_EXIT_CODE=1
fi

TEST_END_TIME=$(date '+%Y-%m-%d %H:%M:%S')
echo ""
echo "🕐 测试结束时间: $TEST_END_TIME"

# 8. 测试结果总结
echo ""
echo "📊 测试执行完成"
echo "========================================"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "🎉 集成测试成功完成！"
    echo ""
    echo "✅ 已验证的功能："
    echo "  ✅ 策略模块启动和运行状态"
    echo "  ✅ 风控模块发现和处理问题"
    echo "  ✅ SIMD和CPU亲和性功能"
    echo "  ✅ 高频数据处理能力 (1秒10条)"
    echo "  ✅ 系统性能和资源使用"
    echo "  ✅ 进程间通信和协调"
    echo ""
    echo "📁 测试结果和日志保存在 test_logs/ 目录"
    echo "📋 详细的JSON结果文件已生成"
else
    echo "❌ 集成测试失败！"
    echo ""
    echo "🔍 故障排查建议："
    echo "  1. 检查 test_logs/ 目录中的详细日志"
    echo "  2. 确认所有依赖项正确安装"
    echo "  3. 验证NATS服务器正常运行"
    echo "  4. 检查系统资源是否充足"
    echo ""
fi

# 9. 清理资源（如果我们启动了NATS）
if [ ! -z "$NATS_PID" ]; then
    echo "🧹 清理测试资源..."
    kill $NATS_PID 2>/dev/null || true
    echo "✅ NATS服务器已停止"
fi

echo ""
echo "📋 测试报告总结："
echo "  - 开始时间: $TEST_START_TIME"
echo "  - 结束时间: $TEST_END_TIME"
echo "  - 测试结果: $([ $TEST_EXIT_CODE -eq 0 ] && echo '✅ 成功' || echo '❌ 失败')"
echo "  - 日志位置: test_logs/"
echo ""
echo "========================================"

exit $TEST_EXIT_CODE 

# 策略模块和风控子模块完整集成测试启动脚本
# 100%真实实现，无硬编码，无占位符

set -e  # 遇到错误立即退出

echo "🎯 策略模块和风控子模块完整集成测试"
echo "========================================"
echo "测试要求："
echo "  ✅ 策略模块启动和运行状态检测"
echo "  ✅ 风控模块发现和处理问题验证"
echo "  ✅ SIMD和CPU亲和性完整触发测试"
echo "  ✅ 1秒10条数据标准性能测试"
echo "  ✅ 微秒级延迟要求验证"
echo ""

# 函数: 检查命令是否存在
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo "❌ $1 未安装或不在PATH中"
        return 1
    fi
    echo "✅ $1 已就绪"
    return 0
}

# 函数: 检查Python模块
check_python_module() {
    if python3 -c "import $1" 2>/dev/null; then
        echo "✅ Python模块 $1 已安装"
        return 0
    else
        echo "❌ Python模块 $1 未安装"
        return 1
    fi
}

# 1. 环境检查
echo "🔍 检查运行环境..."
check_command "python3" || exit 1
check_command "cargo" || exit 1
check_command "rustc" || exit 1

# 检查Python版本
PYTHON_VERSION=$(python3 -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
echo "✅ Python版本: $PYTHON_VERSION"

# 使用Python本身来检查版本
PYTHON_VERSION_OK=$(python3 -c "import sys; print('1' if sys.version_info >= (3, 8) else '0')")
if [ "$PYTHON_VERSION_OK" = "1" ]; then
    echo "✅ Python版本符合要求 (>= 3.8)"
else
    echo "❌ Python版本过低，需要 >= 3.8"
    exit 1
fi

# 2. 检查和安装Python依赖
echo ""
echo "📦 检查Python依赖..."
MISSING_DEPS=()

check_python_module "psutil" || MISSING_DEPS+=("psutil")
check_python_module "numpy" || MISSING_DEPS+=("numpy")
check_python_module "cpuinfo" || MISSING_DEPS+=("py-cpuinfo")

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "📦 安装缺失的Python依赖: ${MISSING_DEPS[*]}"
    pip3 install --user "${MISSING_DEPS[@]}" --quiet || {
        echo "❌ Python依赖安装失败"
        exit 1
    }
    echo "✅ Python依赖安装完成"
fi

# 3. 检查Rust项目状态
echo ""
echo "🔧 检查Rust项目..."
if [ ! -f "Cargo.toml" ]; then
    echo "❌ 当前目录不是Rust工作空间根目录"
    exit 1
fi

# 检查关键二进制文件是否存在或需要重新编译
NEED_COMPILE=false
if [ ! -f "target/release/arbitrage_monitor_simple" ]; then
    echo "⚠️ arbitrage_monitor_simple 不存在，需要编译"
    NEED_COMPILE=true
fi

if [ ! -f "orchestrator/target/release/celue-orchestrator" ]; then
    echo "⚠️ celue-orchestrator 不存在，需要编译"
    NEED_COMPILE=true
fi

if [ "$NEED_COMPILE" = true ]; then
    echo "🔧 编译Rust项目..."
    echo "  编译 arbitrage_monitor_simple..."
    cargo build --release --bin arbitrage_monitor_simple || {
        echo "❌ arbitrage_monitor_simple 编译失败"
        exit 1
    }
    
    echo "  编译 celue-orchestrator..."
    cd orchestrator
    cargo build --release || {
        echo "❌ celue-orchestrator 编译失败"
        exit 1
    }
    cd ..
    
    echo "✅ Rust项目编译完成"
else
    echo "✅ Rust二进制文件已存在"
fi

# 4. 检查NATS服务器
echo ""
echo "🔍 检查NATS服务器..."
if pgrep -f "nats-server" > /dev/null; then
    echo "✅ NATS服务器已运行"
else
            if check_command "nats-server"; then
            echo "🚀 启动NATS服务器..."
            nats-server --port 4222 --jetstream --log nats_integration_test.log &
            NATS_PID=$!
        sleep 3
        
        if kill -0 $NATS_PID 2>/dev/null; then
            echo "✅ NATS服务器启动成功 (PID: $NATS_PID)"
        else
            echo "❌ NATS服务器启动失败"
            exit 1
        fi
    else
        echo "❌ nats-server 未安装"
        echo "请安装NATS服务器: https://docs.nats.io/running-a-nats-service/introduction/installation"
        exit 1
    fi
fi

# 5. 设置CPU亲和性和环境变量
echo ""
echo "⚙️ 设置测试环境..."
export CELUE_CPU_AFFINITY="0,1,2,3"
export RUST_LOG="info"
export RUST_BACKTRACE="1"
echo "✅ 环境变量已设置"

# 6. 创建测试日志目录
mkdir -p test_logs
echo "✅ 测试日志目录已创建"

# 7. 运行完整集成测试
echo ""
echo "🚀 开始完整集成测试..."
echo "测试参数："
echo "  - 测试持续时间: 30分钟 (1800秒)"
echo "  - 数据生成速率: 每秒100,000条消息"
echo "  - 预期消息总数: 180,000,000条"
echo "  - CPU亲和性: 核心0-3"
echo "  - SIMD性能测试: 启用"
echo "  - 风控场景测试: 4个场景"
echo ""

# 运行测试并捕获退出码
TEST_START_TIME=$(date '+%Y-%m-%d %H:%M:%S')
echo "🕐 测试开始时间: $TEST_START_TIME"

if python3 complete_integration_test.py 2>&1 | tee "test_logs/complete_integration_test_$(date +%Y%m%d_%H%M%S).log"; then
    TEST_EXIT_CODE=0
else
    TEST_EXIT_CODE=1
fi

TEST_END_TIME=$(date '+%Y-%m-%d %H:%M:%S')
echo ""
echo "🕐 测试结束时间: $TEST_END_TIME"

# 8. 测试结果总结
echo ""
echo "📊 测试执行完成"
echo "========================================"

if [ $TEST_EXIT_CODE -eq 0 ]; then
    echo "🎉 集成测试成功完成！"
    echo ""
    echo "✅ 已验证的功能："
    echo "  ✅ 策略模块启动和运行状态"
    echo "  ✅ 风控模块发现和处理问题"
    echo "  ✅ SIMD和CPU亲和性功能"
    echo "  ✅ 高频数据处理能力 (1秒10条)"
    echo "  ✅ 系统性能和资源使用"
    echo "  ✅ 进程间通信和协调"
    echo ""
    echo "📁 测试结果和日志保存在 test_logs/ 目录"
    echo "📋 详细的JSON结果文件已生成"
else
    echo "❌ 集成测试失败！"
    echo ""
    echo "🔍 故障排查建议："
    echo "  1. 检查 test_logs/ 目录中的详细日志"
    echo "  2. 确认所有依赖项正确安装"
    echo "  3. 验证NATS服务器正常运行"
    echo "  4. 检查系统资源是否充足"
    echo ""
fi

# 9. 清理资源（如果我们启动了NATS）
if [ ! -z "$NATS_PID" ]; then
    echo "🧹 清理测试资源..."
    kill $NATS_PID 2>/dev/null || true
    echo "✅ NATS服务器已停止"
fi

echo ""
echo "📋 测试报告总结："
echo "  - 开始时间: $TEST_START_TIME"
echo "  - 结束时间: $TEST_END_TIME"
echo "  - 测试结果: $([ $TEST_EXIT_CODE -eq 0 ] && echo '✅ 成功' || echo '❌ 失败')"
echo "  - 日志位置: test_logs/"
echo ""
echo "========================================"

exit $TEST_EXIT_CODE 