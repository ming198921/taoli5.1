#!/bin/bash

# Qingxi V3.0 用户级性能优化脚本 (无需root权限)
# 立即优化当前进程的性能设置

echo "🚀 Qingxi V3.0 用户级性能优化"
echo "========================================"

# 1. 检查当前CPU状态
echo ""
echo "📋 1. 当前系统状态检查"
echo "----------------------------------------"

echo "CPU核心数: $(nproc)"
echo "当前CPU调节器: $(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo '无法检测')"

if command -v numactl >/dev/null 2>&1; then
    echo "NUMA节点: $(numactl --hardware | grep available)"
else
    echo "NUMA工具: 未安装"
fi

# 2. 设置环境变量优化
echo ""
echo "📋 2. 设置V3.0性能环境变量"
echo "----------------------------------------"

export RUST_LOG=info
export QINGXI_ENABLE_V3_OPTIMIZATIONS=true
export QINGXI_INTEL_OPTIMIZATIONS=true
export QINGXI_ZERO_ALLOCATION=true
export QINGXI_O1_SORTING=true
export QINGXI_REALTIME_MONITORING=true
export QINGXI_CPU_AFFINITY_ENABLED=true

# 设置CPU亲和性到前16个核心 (高性能核心)
export QINGXI_CPU_CORES="0-15"

# NUMA优化
export QINGXI_NUMA_NODE=0
export QINGXI_MEMORY_BINDING=local

echo "✅ V3.0优化环境变量已设置"

# 3. 进程优化设置
echo ""
echo "📋 3. 当前进程优化设置"
echo "----------------------------------------"

# 设置进程优先级 (用户权限范围内)
renice -10 $$ 2>/dev/null && echo "✅ 进程优先级已提升" || echo "⚠️ 进程优先级提升失败(权限不足)"

# 设置OMP线程数
export OMP_NUM_THREADS=$(nproc)
export OMP_PROC_BIND=true
export OMP_PLACES=cores

echo "✅ OpenMP线程优化已设置"

# 4. 创建优化启动函数
echo ""
echo "📋 4. 创建V3.0优化启动函数"
echo "----------------------------------------"

# 启动Qingxi的优化函数
start_qingxi_optimized() {
    local config_file="${1:-configs/dynamic_4exchange_test.toml}"
    
    echo "🚀 启动Qingxi V3.0 (优化模式)"
    echo "配置文件: $config_file"
    echo "优化状态: V3.0全启用"
    
    # 如果有numactl，使用NUMA优化启动
    if command -v numactl >/dev/null 2>&1; then
        echo "使用NUMA优化启动..."
        numactl --cpunodebind=0 --membind=0 \
            ./target/debug/market_data_module &
    else
        echo "使用标准模式启动..."
        ./target/debug/market_data_module &
    fi
    
    local pid=$!
    echo "✅ Qingxi已启动, PID: $pid"
    
    # 尝试设置CPU亲和性
    if command -v taskset >/dev/null 2>&1; then
        taskset -cp 0-15 $pid 2>/dev/null && echo "✅ CPU亲和性已设置到核心0-15" || echo "⚠️ CPU亲和性设置失败"
    fi
    
    return $pid
}

# 5. 性能监控函数
monitor_performance() {
    local pid=$1
    echo ""
    echo "📊 性能监控 (PID: $pid)"
    echo "----------------------------------------"
    
    while kill -0 $pid 2>/dev/null; do
        # CPU使用率
        cpu_usage=$(ps -p $pid -o %cpu --no-headers 2>/dev/null || echo "0")
        # 内存使用
        mem_usage=$(ps -p $pid -o %mem --no-headers 2>/dev/null || echo "0")
        # RSS内存
        rss_mem=$(ps -p $pid -o rss --no-headers 2>/dev/null || echo "0")
        
        echo "$(date '+%H:%M:%S') - CPU: ${cpu_usage}%, MEM: ${mem_usage}%, RSS: ${rss_mem}KB"
        sleep 5
    done
}

# 6. 输出优化建议
echo ""
echo "🎯 V3.0性能优化建议"
echo "========================================"
echo ""
echo "立即可用的优化:"
echo "1. 使用函数启动: start_qingxi_optimized"
echo "2. 监控性能: monitor_performance <PID>"
echo ""
echo "系统级优化 (需要root权限):"
echo "1. 运行: sudo ./qingxi_v3_hardware_optimization.sh"
echo "2. 设置CPU调节器为performance模式"
echo "3. 配置NUMA优化和IRQ绑定"
echo ""
echo "V3.0配置验证:"
echo "- 确保O(1)排序引擎使用65536桶"
echo "- 确保零分配内存池65536缓冲区"
echo "- 确保Intel CPU优化器已启用"
echo "- 确保实时性能监控已激活"
echo ""
echo "目标性能指标:"
echo "- 清洗延迟: <0.1ms (当前可能2-6ms)"
echo "- 系统启动: <1秒"
echo "- CPU利用率: 最大化多核使用"
echo "- 内存分配: 零运行时分配"

# 导出函数供shell使用
export -f start_qingxi_optimized
export -f monitor_performance

echo ""
echo "✅ 用户级优化配置完成!"
echo "现在可以使用: start_qingxi_optimized 启动优化版本"
