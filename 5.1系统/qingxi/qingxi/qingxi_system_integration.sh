#!/bin/bash

# Qingxi V3.0 系统级完全集成脚本
# 将所有优化permanently集成到系统，无需依赖启动脚本

echo "🚀 Qingxi V3.0 系统级完全集成开始"
echo "========================================"

# 自动获取root权限
if [[ $EUID -ne 0 ]]; then
    echo "🔐 自动获取root权限..."
    exec sudo -E "$0" "$@"
fi

echo "✅ Root权限已获取"

# 1. 永久性内核参数配置
echo ""
echo "📋 1. 永久性内核参数配置"
echo "----------------------------------------"

# 备份原始配置
cp /etc/sysctl.conf /etc/sysctl.conf.backup.$(date +%Y%m%d_%H%M%S)

# 创建qingxi专用内核参数配置
cat >> /etc/sysctl.conf << 'EOF'

# Qingxi V3.0 高性能优化参数
# =====================================

# 内存管理优化
vm.swappiness = 1
vm.zone_reclaim_mode = 0
vm.vfs_cache_pressure = 50
vm.dirty_background_ratio = 5
vm.dirty_ratio = 10
vm.min_free_kbytes = 131072
vm.overcommit_memory = 1

# 网络性能优化
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.core.netdev_max_backlog = 65536
net.core.somaxconn = 65536
net.ipv4.tcp_rmem = 4096 65536 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216
net.ipv4.tcp_congestion_control = bbr

# 调度器优化
kernel.sched_latency_ns = 2000000
kernel.sched_min_granularity_ns = 400000
kernel.sched_wakeup_granularity_ns = 800000
kernel.sched_rt_runtime_us = 950000
kernel.sched_rt_period_us = 1000000

# CPU性能优化
kernel.sched_autogroup_enabled = 0
kernel.sched_tunable_scaling = 0
kernel.timer_migration = 0

# 文件系统优化
fs.file-max = 1048576
fs.nr_open = 1048576

# 进程优化
kernel.pid_max = 4194304
EOF

echo "✅ 内核参数已永久配置"

# 2. 永久性CPU性能模式配置
echo ""
echo "📋 2. 永久性CPU性能模式配置"
echo "----------------------------------------"

# 检测CPU频率管理类型
CPU_FREQ_TYPE="unknown"
if [[ -d "/sys/devices/system/cpu/intel_pstate" ]]; then
    CPU_FREQ_TYPE="intel_pstate"
elif [[ -f "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor" ]]; then
    CPU_FREQ_TYPE="cpufreq"
elif dmesg 2>/dev/null | grep -q "P-states controlled by the platform"; then
    CPU_FREQ_TYPE="platform_controlled"
fi

echo "检测到CPU频率管理类型: $CPU_FREQ_TYPE"

# 创建智能CPU性能模式服务
cat > /etc/systemd/system/qingxi-cpu-performance.service << 'EOF'
[Unit]
Description=Qingxi CPU Performance Mode
After=multi-user.target

[Service]
Type=oneshot
RemainAfterExit=yes
# Intel P-State 驱动优化
ExecStart=/bin/bash -c 'if [[ -f "/sys/devices/system/cpu/intel_pstate/no_turbo" ]]; then echo "0" > /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null || true; fi'
ExecStart=/bin/bash -c 'if [[ -f "/sys/devices/system/cpu/intel_pstate/min_perf_pct" ]]; then echo "100" > /sys/devices/system/cpu/intel_pstate/min_perf_pct 2>/dev/null || true; fi'
ExecStart=/bin/bash -c 'if [[ -f "/sys/devices/system/cpu/intel_pstate/max_perf_pct" ]]; then echo "100" > /sys/devices/system/cpu/intel_pstate/max_perf_pct 2>/dev/null || true; fi'
# 传统 cpufreq 驱动优化
ExecStart=/bin/bash -c 'for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do [ -f "$cpu" ] && echo "performance" > "$cpu" 2>/dev/null || true; done'
ExecStart=/bin/bash -c 'for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_min_freq; do [ -f "$cpu" ] && cat $(dirname $cpu)/cpuinfo_max_freq > "$cpu" 2>/dev/null || true; done'
# Turbo Boost 启用（多种方法）
ExecStart=/bin/bash -c 'echo "0" > /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null || true'
ExecStart=/bin/bash -c 'for cpu_boost in /sys/devices/system/cpu/cpufreq/boost /sys/devices/system/cpu/cpufreq/policy*/boost; do [ -f "$cpu_boost" ] && echo "1" > "$cpu_boost" 2>/dev/null || true; done'
ExecStart=/bin/bash -c 'modprobe msr 2>/dev/null || true'
ExecStart=/bin/bash -c 'if command -v wrmsr >/dev/null 2>&1; then wrmsr -a 0x1a0 0x850089 2>/dev/null || true; fi'
# 禁用CPU节能功能
ExecStart=/bin/bash -c 'for cpu in /sys/devices/system/cpu/cpu*/power/energy_perf_bias; do [ -f "$cpu" ] && echo "0" > "$cpu" 2>/dev/null || true; done'
ExecStart=/bin/bash -c 'for cpu in /sys/devices/system/cpu/cpu*/power/energy_performance_preference; do [ -f "$cpu" ] && echo "performance" > "$cpu" 2>/dev/null || true; done'

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable qingxi-cpu-performance.service
systemctl start qingxi-cpu-performance.service
echo "✅ CPU性能模式已永久启用（支持多种CPU管理方式）"

# 3. 永久性透明大页配置
echo ""
echo "📋 3. 永久性透明大页配置"
echo "----------------------------------------"

# 创建透明大页配置
cat > /etc/systemd/system/qingxi-hugepages.service << 'EOF'
[Unit]
Description=Qingxi Transparent Huge Pages
After=multi-user.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/bin/bash -c 'echo "always" > /sys/kernel/mm/transparent_hugepage/enabled'
ExecStart=/bin/bash -c 'echo "always" > /sys/kernel/mm/transparent_hugepage/defrag'

[Install]
WantedBy=multi-user.target
EOF

systemctl enable qingxi-hugepages.service
systemctl start qingxi-hugepages.service
echo "✅ 透明大页已永久启用"

# 3.5. 永久性Turbo Boost优化服务
echo ""
echo "📋 3.5. 永久性Turbo Boost优化配置"
echo "----------------------------------------"

# 创建Turbo Boost优化服务
cat > /etc/systemd/system/qingxi-turbo-boost.service << 'EOF'
[Unit]
Description=Qingxi Turbo Boost Optimization
After=qingxi-cpu-performance.service
Requires=qingxi-cpu-performance.service

[Service]
Type=oneshot
RemainAfterExit=yes
# 确保MSR模块加载
ExecStartPre=/bin/bash -c 'modprobe msr 2>/dev/null || true'
# Intel P-State Turbo Boost
ExecStart=/bin/bash -c 'echo "0" > /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null || echo "Intel P-State no_turbo not available"'
# 传统cpufreq Turbo Boost
ExecStart=/bin/bash -c 'for boost in /sys/devices/system/cpu/cpufreq/boost /sys/devices/system/cpu/cpufreq/policy*/boost; do [ -f "$boost" ] && echo "1" > "$boost" 2>/dev/null || true; done'
# MSR寄存器直接控制 (0x1a0 = IA32_MISC_ENABLE)
ExecStart=/bin/bash -c 'if command -v wrmsr >/dev/null 2>&1; then for cpu in $(seq 0 $(($(nproc)-1))); do wrmsr -p $cpu 0x1a0 0x850089 2>/dev/null || true; done; fi'
# BIOS级别控制（如果可用）
ExecStart=/bin/bash -c 'for turbo in /sys/devices/system/cpu/cpu*/cpufreq/boost; do [ -f "$turbo" ] && echo "1" > "$turbo" 2>/dev/null || true; done'
# 验证状态
ExecStart=/bin/bash -c 'echo "Turbo Boost 状态检查:" && if [[ -f "/sys/devices/system/cpu/intel_pstate/no_turbo" ]]; then echo "Intel P-State no_turbo: $(cat /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null || echo N/A)"; fi'

[Install]
WantedBy=multi-user.target
EOF

systemctl enable qingxi-turbo-boost.service
systemctl start qingxi-turbo-boost.service
echo "✅ Turbo Boost优化已永久启用"

# 4. 自动安装必需工具
echo ""
echo "📋 4. 自动安装必需工具"
echo "----------------------------------------"

# 检测包管理器并安装
if command -v apt-get &> /dev/null; then
    apt-get update -qq
    apt-get install -y numactl hwloc-nox build-essential linux-tools-common linux-tools-generic cpufrequtils msr-tools
    # 安装额外的性能工具
    apt-get install -y turbostat powertop i7z cpuid
elif command -v yum &> /dev/null; then
    yum install -y numactl hwloc perf cpupowerutils msr-tools
    yum install -y turbostat powertop
elif command -v dnf &> /dev/null; then
    dnf install -y numactl hwloc perf cpupowerutils msr-tools
    dnf install -y turbostat powertop
fi

# 加载MSR模块（用于直接操作CPU寄存器）
modprobe msr 2>/dev/null || true
echo "msr" >> /etc/modules-load.d/qingxi.conf 2>/dev/null || true

echo "✅ 必需工具已安装（包括MSR工具和性能监控工具）"

# 5. 永久性进程限制配置
echo ""
echo "📋 5. 永久性进程限制配置"
echo "----------------------------------------"

# 备份原始limits配置
cp /etc/security/limits.conf /etc/security/limits.conf.backup.$(date +%Y%m%d_%H%M%S)

# 添加qingxi用户的特殊限制
cat >> /etc/security/limits.conf << 'EOF'

# Qingxi V3.0 性能优化限制
ubuntu          soft    nofile          1048576
ubuntu          hard    nofile          1048576
ubuntu          soft    nproc           unlimited
ubuntu          hard    nproc           unlimited
ubuntu          soft    memlock         unlimited
ubuntu          hard    memlock         unlimited
ubuntu          soft    nice            -20
ubuntu          hard    nice            -20
ubuntu          soft    rtprio          99
ubuntu          hard    rtprio          99
EOF

echo "✅ 进程限制已永久配置"

# 6. 禁用irqbalance并设置中断亲和性
echo ""
echo "📋 6. 中断优化配置"
echo "----------------------------------------"

# 禁用irqbalance
systemctl stop irqbalance 2>/dev/null || true
systemctl disable irqbalance 2>/dev/null || true

# 创建中断亲和性服务
cat > /etc/systemd/system/qingxi-irq-affinity.service << 'EOF'
[Unit]
Description=Qingxi IRQ Affinity Optimization
After=network.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=/bin/bash -c 'for irq in $(awk -F: "/eth|ens|enp/ {print \$1}" /proc/interrupts | tr -d " "); do echo 2 > /proc/irq/\$irq/smp_affinity 2>/dev/null || true; done'

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable qingxi-irq-affinity.service
systemctl start qingxi-irq-affinity.service
echo "✅ 中断亲和性优化已配置"

# 7. 应用所有配置
echo ""
echo "📋 7. GRUB启动参数优化"
echo "----------------------------------------"

# 备份GRUB配置
if [[ -f "/etc/default/grub" ]]; then
    cp /etc/default/grub /etc/default/grub.backup.$(date +%Y%m%d_%H%M%S)
    
    # 添加CPU性能启动参数
    if ! grep -q "intel_pstate=force" /etc/default/grub; then
        sed -i 's/GRUB_CMDLINE_LINUX_DEFAULT="/GRUB_CMDLINE_LINUX_DEFAULT="intel_pstate=force processor.max_cstate=1 intel_idle.max_cstate=0 /' /etc/default/grub
        echo "✅ 已添加CPU性能启动参数"
        
        # 更新GRUB配置
        if command -v update-grub >/dev/null 2>&1; then
            update-grub
        elif command -v grub2-mkconfig >/dev/null 2>&1; then
            grub2-mkconfig -o /boot/grub2/grub.cfg
        fi
        echo "✅ GRUB配置已更新"
    else
        echo "✅ CPU性能启动参数已存在"
    fi
else
    echo "⚠️ GRUB配置文件不存在，跳过启动参数配置"
fi

echo ""
echo "📋 8. 应用系统配置"
echo "----------------------------------------"

# 应用sysctl配置
sysctl -p
echo "✅ 内核参数已应用"

# 验证配置
echo ""
echo "📋 9. 验证系统集成状态"
echo "----------------------------------------"

echo "CPU调节器状态:"
for cpu in /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor; do
    if [[ -f "$cpu" ]]; then
        cpu_name=$(basename $(dirname $cpu))
        governor=$(cat $cpu 2>/dev/null || echo "N/A")
        echo "  $cpu_name: $governor"
    fi
done

if [[ ! -f "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor" ]]; then
    echo "  传统cpufreq不可用 - 检查Intel P-State:"
    if [[ -f "/sys/devices/system/cpu/intel_pstate/no_turbo" ]]; then
        echo "  Intel P-State no_turbo: $(cat /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null)"
        echo "  Intel P-State min_perf_pct: $(cat /sys/devices/system/cpu/intel_pstate/min_perf_pct 2>/dev/null || echo 'N/A')"
        echo "  Intel P-State max_perf_pct: $(cat /sys/devices/system/cpu/intel_pstate/max_perf_pct 2>/dev/null || echo 'N/A')"
    else
        echo "  P-states由平台控制（虚拟化环境）"
    fi
fi

echo "Turbo Boost状态:"
if [[ -f "/sys/devices/system/cpu/intel_pstate/no_turbo" ]]; then
    no_turbo=$(cat /sys/devices/system/cpu/intel_pstate/no_turbo 2>/dev/null)
    if [[ "$no_turbo" == "0" ]]; then
        echo "  Intel Turbo Boost: ✅ 启用"
    else
        echo "  Intel Turbo Boost: ❌ 禁用"
    fi
else
    echo "  Intel P-State Turbo控制不可用"
fi

# 检查cpufreq boost
boost_found=false
for boost in /sys/devices/system/cpu/cpufreq/boost /sys/devices/system/cpu/cpufreq/policy*/boost; do
    if [[ -f "$boost" ]]; then
        boost_status=$(cat $boost 2>/dev/null)
        echo "  cpufreq boost: $boost_status"
        boost_found=true
    fi
done
if [[ "$boost_found" == "false" ]]; then
    echo "  cpufreq boost控制不可用"
fi

echo "透明大页状态: $(cat /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null | grep -o '\[.*\]' | tr -d '[]' || echo 'N/A')"
echo "swappiness: $(cat /proc/sys/vm/swappiness 2>/dev/null || echo 'N/A')"
echo "CPU核心数: $(nproc)"

if command -v numactl >/dev/null 2>&1; then
    echo "NUMA节点: $(numactl --hardware | grep available | awk '{print $2}' || echo 'N/A')"
else
    echo "NUMA工具: 安装失败"
fi

# 检查MSR模块
if lsmod | grep -q msr; then
    echo "MSR模块: ✅ 已加载"
else
    echo "MSR模块: ❌ 未加载"
fi

# 检查qingxi相关服务状态
echo ""
echo "Qingxi服务状态:"
for service in qingxi-cpu-performance qingxi-turbo-boost qingxi-hugepages qingxi-irq-affinity; do
    status=$(systemctl is-enabled $service 2>/dev/null || echo "不存在")
    active=$(systemctl is-active $service 2>/dev/null || echo "不活跃")
    echo "  $service: $status ($active)"
done

echo ""
echo "🎉 Qingxi V3.0 系统级集成完成!"
echo "========================================"
echo "✅ 所有优化已permanently集成到系统"
echo "✅ 重启后所有优化自动生效"
echo "✅ 无需依赖任何启动脚本"
echo ""
echo "💡 建议现在重启系统以确保所有配置生效："
echo "   sudo reboot"
