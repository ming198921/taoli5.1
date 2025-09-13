#!/bin/bash

echo "📊 5.1套利系统387个API微服务监控面板"
echo "============================================="

services=(
    "logging-service:4001:45:日志监控"
    "cleaning-service:4002:52:清洗配置" 
    "strategy-service:4003:38:策略监控"
    "performance-service:4004:67:性能调优"
    "trading-service:4005:41:交易监控"
    "ai-model-service:4006:48:AI模型"
    "config-service:4007:96:配置管理"
)

while true; do
    clear
    echo "📊 5.1套利系统387个API微服务监控面板"
    echo "============================================="
    echo "刷新时间: $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""
    
    total_apis=0
    running_services=0
    
    printf "%-15s %-6s %-8s %-8s %-12s %-10s\n" "服务" "端口" "API数" "状态" "CPU%" "内存MB"
    echo "------------------------------------------------------------------------"
    
    for service_info in "${services[@]}"; do
        IFS=':' read -r service port apis desc <<< "$service_info"
        
        # 检查服务状态
        if curl -sf --connect-timeout 2 "http://localhost:$port/health" > /dev/null 2>&1; then
            status="✅运行"
            total_apis=$((total_apis + apis))
            running_services=$((running_services + 1))
            
            # 获取进程信息
            pid=$(pgrep -f "$service" | head -1)
            if [ -n "$pid" ]; then
                cpu=$(ps -p $pid -o %cpu --no-headers 2>/dev/null | tr -d ' ' || echo "0")
                mem=$(ps -p $pid -o rss --no-headers 2>/dev/null | awk '{print int($1/1024)}' || echo "0")
            else
                cpu="0"
                mem="0"
            fi
        else
            status="❌停止"
            cpu="0"
            mem="0"
        fi
        
        printf "%-15s %-6s %-8s %-8s %-12s %-10s\n" \
               "$desc" "$port" "${apis}个" "$status" "$cpu" "$mem"
    done
    
    echo "------------------------------------------------------------------------"
    echo "总览: $running_services/7 服务运行中, $total_apis/387 API可用 ($(echo "scale=1; $total_apis*100/387" | bc)%)"
    
    # 系统资源
    echo ""
    echo "🖥️  系统资源:"
    echo "CPU使用率: $(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)%"
    echo "内存使用: $(free -m | awk 'NR==2{printf "%.1f%%\n", $3*100/$2 }')"
    echo "磁盘使用: $(df -h / | awk 'NR==2{print $5}')"
    
    # 网络连接
    echo ""
    echo "🌐 API端口监听状态:"
    for service_info in "${services[@]}"; do
        IFS=':' read -r service port apis desc <<< "$service_info"
        if ss -tlnp | grep ":$port " > /dev/null 2>&1; then
            echo "   端口$port: ✅监听中"
        else
            echo "   端口$port: ❌未监听"
        fi
    done
    
    echo ""
    echo "按 Ctrl+C 退出监控"
    sleep 5
done 