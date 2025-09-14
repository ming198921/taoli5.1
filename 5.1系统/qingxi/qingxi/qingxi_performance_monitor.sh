#!/bin/bash
# QINGXI性能实时监控

echo "🔍 QINGXI实时性能监控"
echo "======================"

while true; do
    echo -e "\n[$(date '+%H:%M:%S')] 系统状态:"
    
    # CPU使用率
    cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | sed 's/%us,//')
    echo "   CPU使用: ${cpu_usage}%"
    
    # 内存使用率
    mem_usage=$(free | grep Mem | awk '{printf("%.1f", $3/$2 * 100.0)}')
    echo "   内存使用: ${mem_usage}%"
    
    # 网络连接数
    tcp_conns=$(ss -tun | wc -l)
    echo "   TCP连接: $tcp_conns"
    
    # 如果系统在运行，显示API状态
    if curl -s http://localhost:50061/api/v1/health > /dev/null 2>&1; then
        echo "   ✅ QINGXI API: 运行中"
        
        # 获取系统状态
        api_status=$(curl -s http://localhost:50061/api/v1/health/summary 2>/dev/null)
        if [[ $? -eq 0 ]] && [[ -n "$api_status" ]]; then
            healthy=$(echo "$api_status" | jq -r '.summary.healthy_sources // 0' 2>/dev/null)
            total=$(echo "$api_status" | jq -r '.summary.total_sources // 0' 2>/dev/null)
            latency=$(echo "$api_status" | jq -r '.summary.average_latency_us // 0' 2>/dev/null)
            echo "   数据源: $healthy/$total 健康"
            echo "   平均延迟: ${latency}μs"
        fi
    else
        echo "   ⚠️ QINGXI API: 未运行"
    fi
    
    sleep 5
done
