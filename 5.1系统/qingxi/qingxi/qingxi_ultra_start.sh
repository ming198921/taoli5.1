#!/bin/bash
# 🚀 QINGXI 一键启动脚本 - 极致性能模式
# 解决所有5个问题的综合方案

set -e

echo "🚀 QINGXI 极致性能一键启动"
echo "目标: 获取<0.5ms, 清洗<0.2ms, API管理"
echo "=================================================="

# 检查依赖
check_dependencies() {
    echo "🔍 检查系统依赖..."
    
    # 检查必要工具
    for tool in jq curl cargo; do
        if ! command -v $tool &> /dev/null; then
            echo "❌ 缺少工具: $tool"
            echo "安装建议: sudo apt-get install $tool"
            exit 1
        fi
    done
    
    echo "✅ 系统依赖检查完成"
}

# 应用极致优化
apply_optimizations() {
    echo "⚡ 应用极致性能优化..."
    
    # 运行优化脚本
    if [ -f "./ultra_performance_optimizer.sh" ]; then
        if [[ $EUID -eq 0 ]]; then
            echo "🔧 运行完整root优化..."
            ./ultra_performance_optimizer.sh
        else
            echo "🔧 运行用户态优化..."
            sudo ./ultra_performance_optimizer.sh 2>/dev/null || ./ultra_performance_optimizer.sh
        fi
    else
        echo "⚠️ 优化脚本未找到，跳过系统优化"
    fi
}

# 编译系统
build_system() {
    echo "🔨 编译QINGXI系统..."
    
    # 设置编译环境
    export RUSTFLAGS="-C target-cpu=native -C target-features=+avx2,+avx512f"
    export CARGO_BUILD_JOBS=8
    
    # 编译发布版本
    cargo build --release --bin market_data_module
    
    if [ $? -eq 0 ]; then
        echo "✅ 系统编译成功"
    else
        echo "❌ 系统编译失败"
        exit 1
    fi
}

# 启动HTTP API服务器（后台）
start_api_server() {
    echo "🌐 启动HTTP API服务器..."
    
    # 设置环境变量
    export QINGXI_CONFIG_PATH="$(pwd)/configs/qingxi.toml"
    export RUST_LOG=info
    export QINGXI_PERFORMANCE_MODE=ultra
    
    # 启动主系统（后台）
    nohup ./target/release/market_data_module > qingxi_system.log 2>&1 &
    QINGXI_PID=$!
    echo $QINGXI_PID > qingxi.pid
    
    echo "🚀 QINGXI系统已启动 (PID: $QINGXI_PID)"
    echo "📊 日志文件: qingxi_system.log"
    
    # 等待系统就绪
    echo "⏳ 等待API服务器就绪..."
    for i in {1..30}; do
        if curl -s http://localhost:50061/api/v1/health > /dev/null 2>&1; then
            echo "✅ API服务器已就绪"
            return 0
        fi
        echo -n "."
        sleep 2
    done
    
    echo "❌ API服务器启动超时"
    return 1
}

# 应用优化配置
apply_optimized_config() {
    echo "⚡ 应用优化配置..."
    
    # 使用Python API管理器应用配置
    if python3 api_manager.py optimize; then
        echo "✅ 优化配置已应用"
    else
        echo "⚠️ 配置应用失败，使用默认配置"
    fi
}

# 性能测试
run_performance_test() {
    echo "🧪 运行性能基准测试..."
    
    # 等待系统稳定
    sleep 5
    
    # 获取系统状态
    echo "📊 系统状态检查:"
    python3 api_manager.py status
    
    echo ""
    echo "📈 性能指标检查:"
    python3 api_manager.py performance
    
    echo ""
    echo "🔍 实时数据测试:"
    # 测试几个主要交易对的订单簿获取
    for symbol in BTCUSDT ETHUSDT BNBUSDT; do
        response_time=$(curl -w "%{time_total}" -s -o /dev/null http://localhost:50061/api/v1/orderbook/bybit/$symbol)
        echo "   $symbol: ${response_time}s"
    done
}

# 显示管理面板
show_management_panel() {
    echo ""
    echo "🎛️ QINGXI 管理面板"
    echo "=================="
    echo "系统状态: ✅ 运行中 (PID: $(cat qingxi.pid 2>/dev/null || echo 'N/A'))"
    echo "API地址: http://localhost:50061"
    echo "健康检查: http://localhost:50061/api/v1/health"
    echo ""
    echo "📋 可用命令:"
    echo "   python3 api_manager.py status      # 查看系统状态"
    echo "   python3 api_manager.py performance # 查看性能数据"
    echo "   python3 api_manager.py monitor     # 实时监控"
    echo "   python3 api_manager.py optimize    # 重新优化"
    echo ""
    echo "🔧 系统管理:"
    echo "   ./ultra_performance_optimizer.sh   # 重新优化系统"
    echo "   ./qingxi_performance_monitor.sh    # 性能监控"
    echo "   tail -f qingxi_system.log          # 查看系统日志"
    echo ""
    echo "⛔ 停止系统:"
    echo "   kill \$(cat qingxi.pid) && rm qingxi.pid"
    echo ""
}

# 主函数
main() {
    echo "开始时间: $(date)"
    
    # 检查是否已在运行
    if [ -f qingxi.pid ] && kill -0 $(cat qingxi.pid) 2>/dev/null; then
        echo "⚠️ 系统已在运行 (PID: $(cat qingxi.pid))"
        echo "如需重启，请先停止: kill $(cat qingxi.pid) && rm qingxi.pid"
        exit 1
    fi
    
    # 执行启动流程
    check_dependencies
    apply_optimizations
    build_system
    start_api_server
    
    if [ $? -eq 0 ]; then
        apply_optimized_config
        run_performance_test
        show_management_panel
        
        echo ""
        echo "🎉 QINGXI极致性能模式启动完成！"
        echo "🎯 目标性能:"
        echo "   📡 数据获取: <0.5ms"
        echo "   🧹 数据清洗: <0.2ms"
        echo "   🌐 HTTP API: 已启用"
        echo "   📊 监控面板: 已就绪"
        echo ""
        echo "💡 提示: 运行 'python3 api_manager.py monitor' 查看实时性能"
    else
        echo "❌ 启动失败，请检查日志"
        exit 1
    fi
}

# 捕获退出信号
trap 'echo "正在停止..."; [ -f qingxi.pid ] && kill $(cat qingxi.pid) 2>/dev/null; exit 0' SIGINT SIGTERM

# 执行主函数
main "$@"
