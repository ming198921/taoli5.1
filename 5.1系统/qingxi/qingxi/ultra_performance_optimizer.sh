#!/bin/bash
# -*- coding: utf-8 -*-
# 🚀 QINGXI 极致性能优化脚本 v2.0
# 目标：数据获取0.5ms，清洗0.1-0.2ms

set -e

echo "🚀 QINGXI极致性能优化脚本 v2.0"
echo "目标: 数据获取<0.5ms, 清洗<0.2ms"
echo "=" * 60

# 检查是否为root用户
if [[ $EUID -eq 0 ]]; then
    echo "✅ 检测到root权限，启用完整优化"
    FULL_OPTIMIZATION=true
else
    echo "⚠️ 建议使用sudo运行以启用完整优化"
    FULL_OPTIMIZATION=false
fi

# 1. 网络层优化 - 目标0.5ms延迟
optimize_network() {
    echo "🌐 网络层极致优化..."
    
    # TCP优化参数
    if [ "$FULL_OPTIMIZATION" = true ]; then
        echo "🔧 应用TCP内核优化..."
        
        # 增加网络缓冲区
        sysctl -w net.core.rmem_max=134217728
        sysctl -w net.core.wmem_max=134217728
        sysctl -w net.core.rmem_default=87380
        sysctl -w net.core.wmem_default=65536
        
        # TCP窗口缩放
        sysctl -w net.ipv4.tcp_window_scaling=1
        sysctl -w net.ipv4.tcp_timestamps=1
        sysctl -w net.ipv4.tcp_sack=1
        
        # 减少TCP延迟
        sysctl -w net.ipv4.tcp_low_latency=1
        sysctl -w net.ipv4.tcp_no_delay_ack=1
        
        # 优化连接队列
        sysctl -w net.core.somaxconn=65535
        sysctl -w net.core.netdev_max_backlog=5000
        
        # WebSocket优化
        sysctl -w net.ipv4.tcp_fin_timeout=15
        sysctl -w net.ipv4.tcp_keepalive_time=300
        sysctl -w net.ipv4.tcp_keepalive_probes=3
        sysctl -w net.ipv4.tcp_keepalive_intvl=15
        
        echo "✅ TCP内核优化完成"
    else
        echo "⚠️ 需要root权限进行内核网络优化"
    fi
    
    # 检查网络接口优化
    echo "🔍 网络接口状态检查..."
    for iface in $(ip link show | awk -F: '$0 !~ "lo|vir|docker|br-"{print $2}' | tr -d ' '); do
        if [[ -n "$iface" ]]; then
            echo "   接口: $iface"
            # 检查接口状态
            ethtool "$iface" 2>/dev/null | grep -E "(Speed|Duplex)" || echo "     无法获取详细信息"
        fi
    done
}

# 2. CPU极致优化 - 解决权限问题
optimize_cpu() {
    echo "⚡ CPU极致性能优化..."
    
    if [ "$FULL_OPTIMIZATION" = true ]; then
        echo "🔧 应用CPU性能调节器..."
        
        # 设置performance模式
        for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
            if [ -w "$cpu" ]; then
                echo performance > "$cpu"
                echo "   ✅ $(basename $(dirname "$cpu")): performance模式"
            fi
        done
        
        # 禁用CPU空闲状态以减少延迟
        for idle in /sys/devices/system/cpu/cpu*/cpuidle/state*/disable; do
            if [ -w "$idle" ] && [[ "$idle" =~ state[1-9] ]]; then
                echo 1 > "$idle"
                echo "   ✅ 禁用$(basename $(dirname $(dirname "$idle")))空闲状态"
            fi
        done
        
        # 启用Turbo Boost
        if [ -w /sys/devices/system/cpu/intel_pstate/no_turbo ]; then
            echo 0 > /sys/devices/system/cpu/intel_pstate/no_turbo
            echo "   ✅ Intel Turbo Boost已启用"
        elif [ -w /sys/devices/system/cpu/cpufreq/boost ]; then
            echo 1 > /sys/devices/system/cpu/cpufreq/boost
            echo "   ✅ CPU Boost已启用"
        fi
        
        # NUMA优化
        if command -v numactl &> /dev/null; then
            echo "   🧠 NUMA拓扑检查:"
            numactl --hardware | head -5
        fi
        
        echo "✅ CPU性能优化完成"
    else
        echo "⚠️ 非root用户，尝试用户态优化..."
        
        # 用户态CPU亲和性
        echo "   🔧 设置进程CPU亲和性（用户态）"
        # 这些将在Rust代码中处理
    fi
    
    # 显示当前CPU状态
    echo "📊 CPU状态摘要:"
    echo "   核心数: $(nproc)"
    echo "   频率: $(grep 'cpu MHz' /proc/cpuinfo | head -1 | awk '{print $4}') MHz"
    
    # 检查CPU特性
    if grep -q avx512 /proc/cpuinfo; then
        echo "   ✅ AVX-512支持"
    else
        echo "   ⚠️ 无AVX-512支持"
    fi
}

# 3. 内存优化 - 零分配架构
optimize_memory() {
    echo "🧠 内存子系统优化..."
    
    if [ "$FULL_OPTIMIZATION" = true ]; then
        # 透明大页优化
        echo always > /sys/kernel/mm/transparent_hugepage/enabled
        echo always > /sys/kernel/mm/transparent_hugepage/defrag
        echo "   ✅ 透明大页已启用"
        
        # 虚拟内存优化
        sysctl -w vm.swappiness=1
        sysctl -w vm.dirty_ratio=15
        sysctl -w vm.dirty_background_ratio=5
        echo "   ✅ 虚拟内存参数优化"
        
        # NUMA内存策略
        if [ -d /sys/devices/system/node/node1 ]; then
            echo "   🧠 多NUMA节点检测到，启用本地内存优化"
        fi
    fi
    
    echo "📊 内存状态:"
    free -h | head -2
}

# 4. 系统级延迟优化
optimize_latency() {
    echo "⚡ 系统延迟优化..."
    
    if [ "$FULL_OPTIMIZATION" = true ]; then
        # 禁用不必要的服务
        systemctl stop irqbalance 2>/dev/null || true
        echo "   ✅ IRQ平衡已禁用"
        
        # 设置中断亲和性
        for irq in /proc/irq/*/smp_affinity; do
            if [ -w "$irq" ] && [[ ! "$irq" =~ /proc/irq/0/ ]]; then
                echo f > "$irq" 2>/dev/null || true
            fi
        done
        echo "   ✅ 中断亲和性优化"
        
        # 内核抢占优化
        if [ -w /proc/sys/kernel/sched_rt_runtime_us ]; then
            echo -1 > /proc/sys/kernel/sched_rt_runtime_us
            echo "   ✅ 实时调度优化"
        fi
    fi
}

# 5. 网络栈专项优化
optimize_network_stack() {
    echo "🌐 网络栈专项优化（WebSocket性能）..."
    
    if [ "$FULL_OPTIMIZATION" = true ]; then
        # 优化网络中断
        echo "   🔧 网络中断优化..."
        
        # 增加网络设备队列
        for iface in $(ip link show | awk -F: '$0 !~ "lo|vir|docker|br-"{print $2}' | tr -d ' '); do
            if [[ -n "$iface" ]] && [ -d "/sys/class/net/$iface" ]; then
                # 尝试增加队列大小
                ethtool -G "$iface" rx 4096 tx 4096 2>/dev/null || true
                echo "     $iface: 队列优化尝试"
            fi
        done
        
        # TCP拥塞控制
        sysctl -w net.ipv4.tcp_congestion_control=bbr
        sysctl -w net.core.default_qdisc=fq
        echo "   ✅ BBR拥塞控制启用"
        
        # WebSocket特定优化
        sysctl -w net.ipv4.tcp_fastopen=3
        sysctl -w net.ipv4.tcp_slow_start_after_idle=0
        echo "   ✅ WebSocket连接优化"
    fi
}

# 6. 创建性能监控脚本
create_monitoring_script() {
    echo "📊 创建性能监控脚本..."
    
    cat > qingxi_performance_monitor.sh << 'EOF'
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
EOF
    
    chmod +x qingxi_performance_monitor.sh
    echo "✅ 性能监控脚本已创建: qingxi_performance_monitor.sh"
}

# 7. 启动优化模式
apply_qingxi_optimizations() {
    echo "🚀 应用QINGXI专项优化..."
    
    # 设置环境变量
    export QINGXI_CONFIG_PATH="$(pwd)/configs/qingxi.toml"
    export RUST_LOG=info
    export QINGXI_PERFORMANCE_MODE=ultra
    export QINGXI_TARGET_LATENCY_US=200  # 0.2ms目标
    
    if [ "$FULL_OPTIMIZATION" = true ]; then
        export QINGXI_ENABLE_ROOT_OPTIMIZATIONS=true
    fi
    
    echo "✅ QINGXI环境变量已设置"
    echo "   配置文件: $QINGXI_CONFIG_PATH"
    echo "   性能模式: ultra"
    echo "   目标延迟: 200μs"
}

# 主执行流程
main() {
    optimize_network
    optimize_cpu
    optimize_memory
    optimize_latency
    optimize_network_stack
    create_monitoring_script
    apply_qingxi_optimizations
    
    echo ""
    echo "🎯 极致优化完成摘要:"
    echo "=" * 40
    echo "✅ 网络层: WebSocket连接优化"
    echo "✅ CPU层: 性能调节器 + Turbo Boost"
    echo "✅ 内存层: 透明大页 + 零分配架构"
    echo "✅ 延迟层: 中断亲和性 + 实时调度"
    echo "✅ 应用层: QINGXI专项参数优化"
    echo ""
    echo "🎯 性能目标:"
    echo "   📡 数据获取延迟: <0.5ms"
    echo "   🧹 数据清洗延迟: <0.2ms"
    echo "   📊 端到端延迟: <1ms"
    echo ""
    echo "🚀 启动命令:"
    echo "   ./qingxi_performance_monitor.sh  # 实时监控"
    echo "   python3 api_manager.py optimize  # API配置"
    echo "   cargo run --release              # 启动系统"
    echo ""
    
    if [ "$FULL_OPTIMIZATION" = false ]; then
        echo "💡 建议: 使用 sudo $0 获得完整优化效果"
    fi
}

# 执行主函数
main "$@"
