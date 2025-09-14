#!/bin/bash

# qingxi实时日志监控脚本
# 获取运行中的qingxi系统日志，包括数据清洗时间和交易所数据获取时间

echo "🚀 qingxi实时日志监控器启动"
echo "======================================"
echo "监控内容:"
echo "  - 数据清洗处理时间"
echo "  - 交易所数据获取时间"
echo "  - 性能优化状态"
echo "  - 系统运行状态"
echo "======================================"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 获取qingxi进程PID
get_qingxi_pid() {
    ps aux | grep "market_data_module" | grep -v grep | awk '{print $2}' | head -1
}

# 实时监控日志
monitor_logs() {
    local pid=$(get_qingxi_pid)
    
    if [ -z "$pid" ]; then
        echo "❌ 未找到运行中的qingxi进程"
        echo "请先启动qingxi系统："
        echo "cd /home/devbox/project/qingxi_clean_8bd559a/qingxi"
        echo "RUST_LOG=info cargo run --release --bin market_data_module -- --config configs/four_exchanges_simple.toml"
        exit 1
    fi
    
    echo "✅ 找到qingxi进程 PID: $pid"
    echo ""
    
    # 使用journalctl监控系统日志，同时监控cargo输出
    {
        # 监控cargo进程的输出
        if command -v strace >/dev/null 2>&1; then
            timeout 1 strace -p $pid -e write 2>/dev/null | grep -o "write.*" || true
        fi
        
        # 监控系统日志中的qingxi相关信息
        journalctl -f --no-pager 2>/dev/null | grep -i qingxi || true
    } &
    
    # 主要监控标准输出/错误输出
    echo "🔍 开始监控qingxi实时日志..."
    echo "按 Ctrl+C 停止监控"
    echo ""
    
    # 创建临时文件来存储日志
    local temp_log="/tmp/qingxi_monitor_$$.log"
    
    # 使用ps监控进程输出（如果可能的话）
    while kill -0 $pid 2>/dev/null; do
        # 检查是否有新的日志输出
        if [ -f "/proc/$pid/fd/1" ]; then
            timeout 1 tail -f /proc/$pid/fd/1 2>/dev/null | while read line; do
                parse_and_display_log "$line"
            done || true
        fi
        
        # 检查stderr
        if [ -f "/proc/$pid/fd/2" ]; then
            timeout 1 tail -f /proc/$pid/fd/2 2>/dev/null | while read line; do
                parse_and_display_log "$line"
            done || true
        fi
        
        sleep 0.1
    done
}

# 解析并显示日志
parse_and_display_log() {
    local line="$1"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    # 数据清洗相关日志
    if echo "$line" | grep -q "清洗\|clean\|Clean\|CLEAN"; then
        if echo "$line" | grep -q "耗时\|elapsed\|took\|duration"; then
            echo -e "${GREEN}[$timestamp] 🧹 数据清洗: ${NC}$line"
        else
            echo -e "${CYAN}[$timestamp] 🔧 清洗处理: ${NC}$line"
        fi
    
    # 交易所数据获取相关日志
    elif echo "$line" | grep -qE "(huobi|binance|okx|bybit|exchange).*received\|Received.*Trade\|Received.*OrderBook"; then
        echo -e "${BLUE}[$timestamp] 📈 交易所数据: ${NC}$line"
    
    # 性能优化相关日志
    elif echo "$line" | grep -qE "SIMD\|cache\|batch\|lock.*free\|memory.*pool\|performance\|Performance\|PERFORMANCE"; then
        echo -e "${PURPLE}[$timestamp] 🚀 性能优化: ${NC}$line"
    
    # WebSocket连接相关
    elif echo "$line" | grep -qE "WebSocket\|Connected\|Disconnected\|websocket"; then
        if echo "$line" | grep -q "Connected\|connected"; then
            echo -e "${GREEN}[$timestamp] 🔗 连接状态: ${NC}$line"
        else
            echo -e "${YELLOW}[$timestamp] 🔗 连接状态: ${NC}$line"
        fi
    
    # 错误和警告
    elif echo "$line" | grep -qE "ERROR\|error\|Error\|WARN\|warn\|Warn\|Failed\|failed"; then
        echo -e "${RED}[$timestamp] ⚠️  错误/警告: ${NC}$line"
    
    # 系统状态
    elif echo "$line" | grep -qE "ready\|Ready\|READY\|started\|Started\|STARTED"; then
        echo -e "${GREEN}[$timestamp] ✅ 系统状态: ${NC}$line"
    
    # 其他重要信息
    elif echo "$line" | grep -qE "INFO\|info"; then
        echo -e "${NC}[$timestamp] ℹ️  信息: $line"
    fi
}

# 显示系统状态
show_system_status() {
    echo "📊 qingxi系统状态检查"
    echo "======================"
    
    local pid=$(get_qingxi_pid)
    if [ -n "$pid" ]; then
        echo "✅ qingxi进程运行中 (PID: $pid)"
        echo "🔍 进程信息:"
        ps -p $pid -o pid,ppid,cmd,pmem,pcpu,etime
        echo ""
        
        echo "🌐 网络连接状态:"
        netstat -tnp 2>/dev/null | grep $pid | head -5
        echo ""
        
        echo "💾 内存使用:"
        ps -p $pid -o pid,vsz,rss,pmem
        echo ""
    else
        echo "❌ qingxi进程未运行"
    fi
    
    echo "🔧 系统端口检查:"
    echo "gRPC API (50051):" $(netstat -tln | grep :50051 > /dev/null && echo "✅ 监听中" || echo "❌ 未监听")
    echo "HTTP API (50061):" $(netstat -tln | grep :50061 > /dev/null && echo "✅ 监听中" || echo "❌ 未监听")
    echo "健康检查 (50053):" $(netstat -tln | grep :50053 > /dev/null && echo "✅ 监听中" || echo "❌ 未监听")
    echo ""
}

# 主函数
main() {
    case "${1:-monitor}" in
        "status")
            show_system_status
            ;;
        "monitor")
            show_system_status
            echo "开始实时监控..."
            echo ""
            monitor_logs
            ;;
        "help")
            echo "用法: $0 [monitor|status|help]"
            echo "  monitor  - 实时监控日志 (默认)"
            echo "  status   - 显示系统状态"
            echo "  help     - 显示此帮助信息"
            ;;
        *)
            echo "未知命令: $1"
            echo "使用 '$0 help' 查看帮助"
            exit 1
            ;;
    esac
}

# 信号处理
trap 'echo ""; echo "👋 监控已停止"; exit 0' INT TERM

# 运行主函数
main "$@"
