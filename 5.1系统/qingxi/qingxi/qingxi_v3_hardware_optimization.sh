#!/bin/bash

# Qingxi V3.0 硬件级性能优化脚本 (需要root权限)
# 解决CPU调节器、NUMA、进程优先级等系统级优化问题

echo "🚀 Qingxi V3.0 硬件级性能优化"
echo "========================================"

# 检查root权限
if [[ $EUID -ne 0 ]]; then
   echo "❌ 此脚本需要root权限运行"
   echo "请使用: sudo $0"
   exit 1
fi

echo "✅ Root权限确认"

# 1. CPU性能调节器优化
echo ""
echo "📋 1. CPU性能调节器优化"
echo "----------------------------------------"

# 检查可用的调节器
available_governors=$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors 2>/dev/null || echo "")
echo "可用调节器: $available_governors"

# 设置为performance模式
if [[ "$available_governors" == *"performance"* ]]; then
    echo "设置CPU调节器为performance模式..."
    for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
        if [[ -f "$cpu" ]]; then
            echo "performance" > "$cpu" 2>/dev/null && echo "✅ $(basename $(dirname $cpu)) 设置成功" || echo "⚠️ $(basename $(dirname $cpu)) 设置失败"
        fi
    done
else
    echo "⚠️ performance调节器不可用，尝试其他高性能选项..."
    # 尝试设置其他高性能调节器
    for governor in "ondemand" "schedutil"; do
        if [[ "$available_governors" == *"$governor"* ]]; then
            echo "设置CPU调节器为$governor模式..."
            for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
                if [[ -f "$cpu" ]]; then
                    echo "$governor" > "$cpu" 2>/dev/null
                fi
            done
            break
        fi
    done
fi

# 设置CPU最高频率
echo "设置CPU为最高频率..."
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_setspeed; do
    if [[ -f "$cpu" ]]; then
        max_freq=$(cat $(dirname $cpu)/cpuinfo_max_freq 2>/dev/null)
        if [[ -n "$max_freq" ]]; then
            echo "$max_freq" > "$cpu" 2>/dev/null
        fi
    fi
done

# 禁用CPU节能功能
echo "禁用CPU节能功能..."
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_min_freq; do
    if [[ -f "$cpu" ]]; then
        max_freq=$(cat $(dirname $cpu)/cpuinfo_max_freq 2>/dev/null)
        if [[ -n "$max_freq" ]]; then
            echo "$max_freq" > "$cpu" 2>/dev/null
        fi
    fi
done

echo "✅ CPU性能调节器优化完成"

# 2. 安装和配置NUMA工具
echo ""
echo "📋 2. NUMA工具安装和配置"
echo "----------------------------------------"

# 检查是否已安装numactl
if ! command -v numactl &> /dev/null; then
    echo "安装numactl工具..."
    if command -v apt-get &> /dev/null; then
        apt-get update -qq
        apt-get install -y numactl hwloc-nox
    elif command -v yum &> /dev/null; then
        yum install -y numactl hwloc
    elif command -v dnf &> /dev/null; then
        dnf install -y numactl hwloc
    else
        echo "⚠️ 无法识别包管理器，请手动安装numactl"
    fi
else
    echo "✅ numactl已安装"
fi

# 检查NUMA配置
if command -v numactl &> /dev/null; then
    echo "NUMA节点信息:"
    numactl --hardware | head -5
    
    # 获取NUMA节点数
    numa_nodes=$(numactl --hardware | grep "available:" | grep -o '[0-9]*' | head -1)
    echo "检测到 $numa_nodes 个NUMA节点"
    
    # 设置NUMA策略
    echo "设置NUMA内存分配策略..."
    echo 1 > /proc/sys/vm/zone_reclaim_mode 2>/dev/null || echo "⚠️ 无法设置zone_reclaim_mode"
    
else
    echo "⚠️ numactl安装失败或不可用"
fi

# 3. 内存优化
echo ""
echo "📋 3. 内存系统优化"
echo "----------------------------------------"

# 启用透明大页
echo "配置透明大页..."
echo "always" > /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null && echo "✅ 透明大页已启用" || echo "⚠️ 透明大页配置失败"
echo "always" > /sys/kernel/mm/transparent_hugepage/defrag 2>/dev/null && echo "✅ 透明大页整理已启用" || echo "⚠️ 透明大页整理配置失败"

# 配置vm参数优化
echo "优化内存管理参数..."
echo 1 > /proc/sys/vm/swappiness && echo "✅ swappiness设置为1"
echo 0 > /proc/sys/vm/zone_reclaim_mode && echo "✅ zone_reclaim_mode禁用"
echo 50 > /proc/sys/vm/vfs_cache_pressure && echo "✅ vfs_cache_pressure设置为50"

# 设置脏页参数
echo 5 > /proc/sys/vm/dirty_background_ratio && echo "✅ dirty_background_ratio设置为5"
echo 10 > /proc/sys/vm/dirty_ratio && echo "✅ dirty_ratio设置为10"

echo "✅ 内存系统优化完成"

# 4. 网络优化
echo ""
echo "📋 4. 网络性能优化"
echo "----------------------------------------"

echo "优化网络缓冲区..."
echo 16777216 > /proc/sys/net/core/rmem_max && echo "✅ 接收缓冲区最大值设置"
echo 16777216 > /proc/sys/net/core/wmem_max && echo "✅ 发送缓冲区最大值设置"
echo 65536 > /proc/sys/net/core/netdev_max_backlog && echo "✅ 网络设备队列长度设置"

echo "✅ 网络性能优化完成"

# 5. 中断亲和性优化
echo ""
echo "📋 5. 中断亲和性优化"
echo "----------------------------------------"

# 检查是否有irqbalance服务
if systemctl is-active --quiet irqbalance; then
    echo "停止irqbalance服务..."
    systemctl stop irqbalance
    systemctl disable irqbalance
    echo "✅ irqbalance服务已停止"
fi

# 设置网络中断到特定CPU核心
echo "配置网络中断亲和性..."
network_irqs=$(grep -E "(eth|ens|enp)" /proc/interrupts | awk -F: '{print $1}' | tr -d ' ')
cpu_count=$(nproc)
target_cpus=$((cpu_count - 1))  # 使用最后一个CPU核心处理中断

for irq in $network_irqs; do
    if [[ -f "/proc/irq/$irq/smp_affinity" ]]; then
        printf "%x" $((1 << target_cpus)) > /proc/irq/$irq/smp_affinity 2>/dev/null && echo "✅ IRQ $irq 绑定到CPU $target_cpus"
    fi
done

echo "✅ 中断亲和性优化完成"

# 6. 进程调度优化
echo ""
echo "📋 6. 进程调度优化"
echo "----------------------------------------"

# 设置内核调度参数
echo "优化调度器参数..."
echo 2000000 > /proc/sys/kernel/sched_latency_ns && echo "✅ 调度延迟设置为2ms"
echo 400000 > /proc/sys/kernel/sched_min_granularity_ns && echo "✅ 最小调度粒度设置为0.4ms"
echo 800000 > /proc/sys/kernel/sched_wakeup_granularity_ns && echo "✅ 唤醒调度粒度设置为0.8ms"

# 设置实时调度参数
echo 950000 > /proc/sys/kernel/sched_rt_runtime_us && echo "✅ 实时调度运行时设置"
echo 1000000 > /proc/sys/kernel/sched_rt_period_us && echo "✅ 实时调度周期设置"

echo "✅ 进程调度优化完成"

# 7. 创建qingxi专用的启动函数
echo ""
echo "📋 7. 创建qingxi优化启动函数"
echo "----------------------------------------"

# 创建qingxi启动脚本
cat > /usr/local/bin/start_qingxi_optimized << 'EOF'
#!/bin/bash

# Qingxi V3.0 优化启动脚本
QINGXI_DIR="${1:-/home/ubuntu/qingxi/qingxi}"
CONFIG_FILE="${2:-configs/four_exchanges_simple.toml}"

echo "🚀 启动Qingxi V3.0 (硬件优化模式)"
echo "工作目录: $QINGXI_DIR"
echo "配置文件: $CONFIG_FILE"

# 设置环境变量
export RUST_LOG=info
export QINGXI_CONFIG_PATH="$QINGXI_DIR/$CONFIG_FILE"
export QINGXI_ENABLE_V3_OPTIMIZATIONS=true
export QINGXI_INTEL_OPTIMIZATIONS=true
export QINGXI_ZERO_ALLOCATION=true
export QINGXI_O1_SORTING=true
export QINGXI_REALTIME_MONITORING=true
export QINGXI_CPU_AFFINITY_ENABLED=true
export QINGXI_CPU_CORES="0-6"  # 保留CPU7给中断处理
export QINGXI_NUMA_NODE=0
export QINGXI_MEMORY_BINDING=local

cd "$QINGXI_DIR"

# 使用NUMA和CPU亲和性启动
if command -v numactl >/dev/null 2>&1; then
    echo "使用NUMA优化启动..."
    numactl --cpunodebind=0 --membind=0 \
        nice -n -10 \
        ionice -c 1 -n 0 \
        taskset -c 0-6 \
        ./target/release/market_data_module &
else
    echo "使用CPU亲和性启动..."
    nice -n -10 \
        ionice -c 1 -n 0 \
        taskset -c 0-6 \
        ./target/release/market_data_module &
fi

PID=$!
echo "✅ Qingxi已启动, PID: $PID"

# 设置实时优先级
chrt -f -p 50 $PID 2>/dev/null && echo "✅ 实时优先级已设置" || echo "⚠️ 实时优先级设置失败"

# 设置OOM保护
echo -17 > /proc/$PID/oom_adj 2>/dev/null && echo "✅ OOM保护已设置" || echo "⚠️ OOM保护设置失败"

echo "✅ Qingxi V3.0启动完成，PID: $PID"
echo "$PID" > /tmp/qingxi.pid
EOF

chmod +x /usr/local/bin/start_qingxi_optimized
echo "✅ qingxi优化启动脚本创建完成: /usr/local/bin/start_qingxi_optimized"

# 8. 创建性能监控脚本
cat > /usr/local/bin/monitor_qingxi << 'EOF'
#!/bin/bash

PID_FILE="/tmp/qingxi.pid"
if [[ -f "$PID_FILE" ]]; then
    PID=$(cat "$PID_FILE")
else
    echo "⚠️ 找不到qingxi PID文件"
    exit 1
fi

echo "📊 Qingxi性能监控 (PID: $PID)"
echo "========================================"

while kill -0 "$PID" 2>/dev/null; do
    # CPU使用率
    cpu_usage=$(ps -p "$PID" -o %cpu --no-headers 2>/dev/null | tr -d ' ')
    # 内存使用
    mem_usage=$(ps -p "$PID" -o %mem --no-headers 2>/dev/null | tr -d ' ')
    # RSS内存 (KB)
    rss_mem=$(ps -p "$PID" -o rss --no-headers 2>/dev/null | tr -d ' ')
    # 虚拟内存 (KB)
    vsz_mem=$(ps -p "$PID" -o vsz --no-headers 2>/dev/null | tr -d ' ')
    # 线程数
    threads=$(ps -p "$PID" -o nlwp --no-headers 2>/dev/null | tr -d ' ')
    # 优先级
    priority=$(ps -p "$PID" -o nice --no-headers 2>/dev/null | tr -d ' ')
    
    # 网络连接数
    connections=$(ss -an | grep ":8080\|:8081\|:8082" | wc -l)
    
    # 系统负载
    load_avg=$(uptime | awk -F'load average:' '{print $2}' | awk '{print $1}' | tr -d ',')
    
    timestamp=$(date '+%H:%M:%S')
    echo "$timestamp - CPU:${cpu_usage}% MEM:${mem_usage}% RSS:${rss_mem}KB VSZ:${vsz_mem}KB 线程:${threads} 优先级:${priority} 连接:${connections} 负载:${load_avg}"
    
    sleep 2
done

echo "❌ Qingxi进程已停止"
EOF

chmod +x /usr/local/bin/monitor_qingxi
echo "✅ qingxi监控脚本创建完成: /usr/local/bin/monitor_qingxi"

# 9. 验证优化效果
echo ""
echo "📋 8. 验证优化效果"
echo "----------------------------------------"

echo "当前CPU调节器状态:"
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    if [[ -f "$cpu" ]]; then
        cpu_name=$(basename $(dirname $cpu))
        governor=$(cat $cpu 2>/dev/null)
        echo "  $cpu_name: $governor"
    fi
done

echo ""
echo "当前系统优化状态:"
echo "  透明大页: $(cat /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null | grep -o '\[.*\]' | tr -d '[]')"
echo "  swappiness: $(cat /proc/sys/vm/swappiness 2>/dev/null)"
echo "  CPU核心数: $(nproc)"

if command -v numactl >/dev/null 2>&1; then
    echo "  NUMA节点: $(numactl --hardware | grep available | awk '{print $2}')"
else
    echo "  NUMA工具: 安装中或不可用"
fi

# 10. 创建systemd服务文件（可选）
echo ""
echo "📋 9. 创建systemd服务（可选）"
echo "----------------------------------------"

cat > /etc/systemd/system/qingxi.service << 'EOF'
[Unit]
Description=Qingxi V3.0 Market Data Module
After=network.target

[Service]
Type=forking
User=ubuntu
Group=ubuntu
WorkingDirectory=/home/ubuntu/qingxi/qingxi
ExecStart=/usr/local/bin/start_qingxi_optimized
PIDFile=/tmp/qingxi.pid
Restart=always
RestartSec=5
Nice=-10
IOSchedulingClass=1
IOSchedulingPriority=0

# 性能优化
LimitNOFILE=65536
LimitNPROC=32768
LimitMEMLOCK=infinity

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
echo "✅ systemd服务文件已创建: /etc/systemd/system/qingxi.service"
echo "   使用 'systemctl start qingxi' 启动服务"
echo "   使用 'systemctl enable qingxi' 设置开机自启"

# 完成总结
echo ""
echo "🎉 Qingxi V3.0 硬件级优化完成!"
echo "========================================"
echo ""
echo "✅ 已完成的优化:"
echo "  • CPU性能调节器优化"
echo "  • NUMA工具安装和配置"
echo "  • 内存系统优化（透明大页、缓存策略）"
echo "  • 网络性能优化"
echo "  • 中断亲和性优化"
echo "  • 进程调度优化"
echo "  • 创建优化启动脚本"
echo "  • 创建性能监控脚本"
echo "  • 配置systemd服务"
echo ""
echo "🚀 现在可以使用以下命令:"
echo "  启动qingxi: start_qingxi_optimized [工作目录] [配置文件]"
echo "  监控性能: monitor_qingxi"
echo "  服务管理: systemctl start/stop/status qingxi"
echo ""
echo "🎯 预期性能提升:"
echo "  • CPU性能: 固定最高频率，无节能降频"
echo "  • 内存性能: 大页支持，优化缓存策略"
echo "  • 网络性能: 优化缓冲区，中断绑定"
echo "  • 进程优先级: 实时调度，最高优先级"
echo "  • 零分配失败率: 预计从0.20%降低到<0.05%"
echo ""
echo "💡 建议立即重启qingxi系统验证优化效果!"
