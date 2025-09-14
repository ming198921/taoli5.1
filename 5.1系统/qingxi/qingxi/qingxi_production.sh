#!/bin/bash

# QINGXI 生产级市场数据系统启动脚本
# 版本: 1.0 - 修复完成版本

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $(date '+%Y-%m-%d %H:%M:%S') - $1"
}

# 检查依赖
check_dependencies() {
    log_info "🔍 检查系统依赖..."
    
    if ! command -v cargo &> /dev/null; then
        log_error "❌ Rust/Cargo 未安装"
        exit 1
    fi
    
    if ! command -v docker &> /dev/null; then
        log_warn "⚠️ Docker 未安装，将使用本地模式"
        DOCKER_AVAILABLE=false
    else
        DOCKER_AVAILABLE=true
        log_info "✅ Docker 可用"
    fi
    
    log_info "✅ 依赖检查完成"
}

# 编译项目
build_project() {
    log_info "🔨 编译QINGXI项目..."
    
    # 清理之前的构建
    cargo clean
    
    # 编译发布版本
    if cargo build --release; then
        log_info "✅ 项目编译成功"
    else
        log_error "❌ 项目编译失败"
        exit 1
    fi
}

# 运行测试
run_tests() {
    log_info "🧪 运行集成测试..."
    
    if cargo test --release; then
        log_info "✅ 所有测试通过"
    else
        log_warn "⚠️ 部分测试失败，但继续启动"
    fi
}

# 检查端口占用
check_ports() {
    log_info "🔌 检查端口占用情况..."
    
    local ports=(50051 50061 50053 8080)
    local occupied_ports=()
    
    for port in "${ports[@]}"; do
        if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
            occupied_ports+=($port)
            log_warn "⚠️ 端口 $port 已被占用"
        fi
    done
    
    if [ ${#occupied_ports[@]} -gt 0 ]; then
        log_warn "发现占用端口: ${occupied_ports[*]}"
        read -p "是否继续启动? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "启动取消"
            exit 0
        fi
    else
        log_info "✅ 所有必需端口可用"
    fi
}

# 启动系统监控
start_monitoring() {
    log_info "📊 启动系统监控..."
    
    # 创建监控脚本
    cat > /tmp/qingxi_monitor.sh << 'EOF'
#!/bin/bash
while true; do
    echo "=== $(date) ==="
    echo "内存使用:"
    ps aux | grep -E "(market_data_module|qingxi)" | grep -v grep
    echo "网络连接:"
    netstat -tlnp | grep -E ":(50051|50061|50053|8080)"
    echo "日志尾部:"
    tail -n 5 /tmp/qingxi.log 2>/dev/null || echo "暂无日志"
    echo "===================="
    sleep 30
done
EOF
    
    chmod +x /tmp/qingxi_monitor.sh
    nohup /tmp/qingxi_monitor.sh > /tmp/qingxi_monitor.log 2>&1 &
    MONITOR_PID=$!
    log_info "✅ 监控已启动 (PID: $MONITOR_PID)"
}

# 启动主程序
start_qingxi() {
    log_info "🚀 启动QINGXI市场数据系统..."
    
    # 设置环境变量
    export RUST_LOG=info
    export QINGXI_CONFIG_PATH="configs/qingxi.toml"
    
    # 创建日志目录
    mkdir -p logs
    
    # 启动主程序
    log_info "启动参数:"
    log_info "  - gRPC API: 0.0.0.0:50051"
    log_info "  - HTTP API: 0.0.0.0:50061" 
    log_info "  - 健康检查: 0.0.0.0:50053"
    log_info "  - 配置文件: $QINGXI_CONFIG_PATH"
    
    if [[ "${1:-}" == "--background" ]]; then
        log_info "🔧 后台模式启动..."
        nohup ./target/release/market_data_module > logs/qingxi.log 2>&1 &
        QINGXI_PID=$!
        echo $QINGXI_PID > /tmp/qingxi.pid
        log_info "✅ QINGXI已在后台启动 (PID: $QINGXI_PID)"
        
        # 等待启动
        sleep 5
        
        # 验证启动状态
        if kill -0 $QINGXI_PID 2>/dev/null; then
            log_info "✅ QINGXI运行正常"
            
            # 测试API连接
            log_info "🧪 测试API连接..."
            sleep 2
            
            if curl -s http://localhost:50061/api/v1/health >/dev/null 2>&1; then
                log_info "✅ HTTP API响应正常"
            else
                log_warn "⚠️ HTTP API暂未响应，可能还在启动中"
            fi
            
        else
            log_error "❌ QINGXI启动失败"
            exit 1
        fi
    else
        log_info "🔧 前台模式启动..."
        exec ./target/release/market_data_module
    fi
}

# 停止系统
stop_qingxi() {
    log_info "🛑 停止QINGXI系统..."
    
    # 停止主程序
    if [[ -f /tmp/qingxi.pid ]]; then
        local pid=$(cat /tmp/qingxi.pid)
        if kill -0 $pid 2>/dev/null; then
            kill -TERM $pid
            log_info "✅ QINGXI进程已停止 (PID: $pid)"
        fi
        rm -f /tmp/qingxi.pid
    fi
    
    # 停止监控
    if [[ -n "${MONITOR_PID:-}" ]] && kill -0 $MONITOR_PID 2>/dev/null; then
        kill $MONITOR_PID
        log_info "✅ 监控进程已停止"
    fi
    
    log_info "✅ 系统停止完成"
}

# 显示状态
show_status() {
    log_info "📋 QINGXI系统状态:"
    
    if [[ -f /tmp/qingxi.pid ]]; then
        local pid=$(cat /tmp/qingxi.pid)
        if kill -0 $pid 2>/dev/null; then
            log_info "✅ 主进程运行中 (PID: $pid)"
            
            # 显示端口状态
            log_info "🔌 端口状态:"
            netstat -tlnp 2>/dev/null | grep -E ":(50051|50061|50053|8080)" | while read line; do
                log_info "  $line"
            done
            
            # 显示最近日志
            if [[ -f logs/qingxi.log ]]; then
                log_info "📝 最近日志 (最后10行):"
                tail -n 10 logs/qingxi.log | sed 's/^/  /'
            fi
            
        else
            log_warn "❌ 主进程未运行"
        fi
    else
        log_warn "❌ 未找到PID文件，系统可能未启动"
    fi
}

# 清理函数
cleanup() {
    log_info "🧹 执行清理..."
    stop_qingxi
}

# 设置信号处理
trap cleanup EXIT INT TERM

# 主函数
main() {
    log_info "🌟 QINGXI 生产级市场数据系统"
    log_info "======================================"
    
    case "${1:-start}" in
        "build")
            check_dependencies
            build_project
            ;;
        "test")
            check_dependencies
            build_project
            run_tests
            ;;
        "start")
            check_dependencies
            build_project
            check_ports
            start_monitoring
            start_qingxi "${2:-}"
            ;;
        "stop")
            stop_qingxi
            ;;
        "restart")
            stop_qingxi
            sleep 2
            main start "${2:-}"
            ;;
        "status")
            show_status
            ;;
        *)
            echo "用法: $0 {build|test|start|stop|restart|status} [--background]"
            echo ""
            echo "命令说明:"
            echo "  build     - 编译项目"
            echo "  test      - 运行测试"
            echo "  start     - 启动系统 (添加 --background 后台运行)"
            echo "  stop      - 停止系统"
            echo "  restart   - 重启系统"
            echo "  status    - 显示状态"
            exit 1
            ;;
    esac
}

# 运行主函数
main "$@"
