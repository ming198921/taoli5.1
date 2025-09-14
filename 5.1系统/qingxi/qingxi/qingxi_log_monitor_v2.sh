#!/bin/bash

# qingxi系统实时日志监控脚本 v2.0
# 专门监控数据清洗时间和交易所数据获取时间

LOG_FILE="qingxi_monitor_$(date +%Y%m%d_%H%M%S).log"
SCRIPT_DIR=$(dirname "$0")

echo "🚀 qingxi系统实时日志监控启动 v2.0" | tee -a "$LOG_FILE"
echo "监控开始时间: $(date)" | tee -a "$LOG_FILE"
echo "工作目录: $(pwd)" | tee -a "$LOG_FILE"
echo "================================================" | tee -a "$LOG_FILE"

# 检查qingxi进程状态
check_qingxi_process() {
    local cargo_pid=$(pgrep -f "cargo run.*qingxi" | head -1)
    local qingxi_pid=$(pgrep -f "qingxi.*config" | head -1)
    
    if [ -n "$cargo_pid" ]; then
        echo "✅ 找到cargo进程 PID: $cargo_pid" | tee -a "$LOG_FILE"
        return 0
    elif [ -n "$qingxi_pid" ]; then
        echo "✅ 找到qingxi进程 PID: $qingxi_pid" | tee -a "$LOG_FILE"
        return 0
    else
        echo "⚠️  未找到运行中的qingxi进程" | tee -a "$LOG_FILE"
        return 1
    fi
}

# 监控特定关键词的日志
monitor_key_logs() {
    echo "📊 开始监控关键日志信息..." | tee -a "$LOG_FILE"
    echo "关注指标: 数据清洗时间、交易所数据获取时间、处理延迟" | tee -a "$LOG_FILE"
    echo "------------------------------------------------" | tee -a "$LOG_FILE"
    
    # 监控当前目录下的所有日志文件
    find . -name "*.log" -type f | while read -r logfile; do
        if [ -f "$logfile" ]; then
            echo "📋 监控日志文件: $logfile" | tee -a "$LOG_FILE"
            tail -f "$logfile" 2>/dev/null | while IFS= read -r line; do
                local timestamp=$(date +'%H:%M:%S.%3N')
                
                # 过滤关键信息
                if [[ "$line" =~ (清洗|cleaning|处理时间|processing.*time|耗时|duration|数据获取|received|订单簿|orderbook|trade|延迟|latency|ms|μs|快照|snapshot|批处理|batch|SIMD|缓存|cache|性能|performance) ]]; then
                    echo "[$timestamp] 🔍 KEY: $line" | tee -a "$LOG_FILE"
                elif [[ "$line" =~ (ERROR|WARN|error|warn|failed|失败|错误|异常) ]]; then
                    echo "[$timestamp] ⚠️  ERR: $line" | tee -a "$LOG_FILE"
                elif [[ "$line" =~ (INFO|info|成功|完成|启动|连接) ]]; then
                    echo "[$timestamp] ℹ️  INFO: $line" | tee -a "$LOG_FILE"
                fi
            done &
        fi
    done
    
    # 等待所有后台进程
    wait
}

# 实时性能统计
show_performance_stats() {
    while true; do
        sleep 60  # 每分钟输出一次统计
        local timestamp=$(date +'%H:%M:%S')
        
        # 统计最近1分钟的关键事件
        local recent_logs=$(tail -100 "$LOG_FILE" | grep "$(date +'%H:%M')" | wc -l)
        local error_count=$(tail -100 "$LOG_FILE" | grep -i "error\|failed\|错误" | wc -l)
        local processing_count=$(tail -100 "$LOG_FILE" | grep -i "processing\|处理\|清洗" | wc -l)
        
        echo "[$timestamp] 📈 统计报告 - 日志条数:$recent_logs, 处理事件:$processing_count, 错误:$error_count" | tee -a "$LOG_FILE"
    done &
}

# 主监控逻辑
main() {
    # 检查进程状态
    if check_qingxi_process; then
        echo "🎯 qingxi系统正在运行，开始监控..." | tee -a "$LOG_FILE"
    else
        echo "🔍 qingxi系统可能正在启动，继续监控日志..." | tee -a "$LOG_FILE"
    fi
    
    # 启动性能统计
    show_performance_stats &
    STATS_PID=$!
    
    # 设置信号处理
    trap "echo '🛑 停止监控...'; kill $STATS_PID 2>/dev/null; exit" INT TERM
    
    # 开始监控日志
    monitor_key_logs
}

# 执行主函数
main
