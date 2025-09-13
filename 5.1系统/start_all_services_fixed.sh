#!/bin/bash
set -e

echo "🚀 启动5.1套利系统387个API接口服务 (修正版)"
echo "================================================="

cd "$(dirname "$0")"

# 创建日志目录
mkdir -p logs

# 停止可能存在的服务
echo "🛑 停止现有服务..."
pkill -f "logging-service" 2>/dev/null || true
pkill -f "cleaning-service" 2>/dev/null || true
pkill -f "strategy-service" 2>/dev/null || true
pkill -f "performance-service" 2>/dev/null || true
pkill -f "trading-service" 2>/dev/null || true
pkill -f "ai-model-service" 2>/dev/null || true
pkill -f "config-service" 2>/dev/null || true
sleep 2

# 检查编译状态
echo "🔍 检查微服务编译状态..."
missing_services=()

services=(
    "logging-service:4001:45:日志监控"
    "cleaning-service:4002:52:清洗配置" 
    "strategy-service:4003:38:策略监控"
    "performance-service:4004:67:性能调优"
    "trading-service:4005:41:交易监控"
    "ai-model-service:4006:48:AI模型"
    "config-service:4007:96:配置管理"
)

for service_info in "${services[@]}"; do
    IFS=':' read -r service port apis desc <<< "$service_info"
    
    if [ ! -f "$service/target/release/$service" ]; then
        missing_services+=("$service")
        echo "❌ $service 可执行文件不存在"
    fi
done

if [ ${#missing_services[@]} -gt 0 ]; then
    echo ""
    echo "⚠️  发现 ${#missing_services[@]} 个微服务未编译，正在编译..."
    
    for service in "${missing_services[@]}"; do
        echo "   编译 $service..."
        cd "$service"
        cargo build --release --quiet
        if [ $? -eq 0 ]; then
            echo "   ✅ $service 编译成功"
        else
            echo "   ❌ $service 编译失败"
            exit 1
        fi
        cd ..
    done
fi

# 启动所有微服务
echo ""
echo "🚀 启动微服务..."

pids=()

for service_info in "${services[@]}"; do
    IFS=':' read -r service port apis desc <<< "$service_info"
    
    echo "   启动 $desc (端口$port) - $apis个API..."
    
    cd "$service"
    
    # 设置环境变量
    export RUST_LOG=info
    export SERVER_PORT=$port
    
    # 启动服务
    nohup ./target/release/$service > "../logs/$service.log" 2>&1 &
    service_pid=$!
    pids+=($service_pid)
    
    echo "      PID: $service_pid"
    
    cd ..
    sleep 1
done

echo ""
echo "⏳ 等待服务启动..."
sleep 5

# 健康检查
echo ""
echo "🏥 执行健康检查..."

healthy_count=0
total_services=${#services[@]}

check_service_health() {
    local service=$1
    local port=$2
    local desc=$3
    
    if curl -sf --connect-timeout 3 "http://localhost:$port/health" > /dev/null 2>&1; then
        echo "✅ $desc (端口$port) - 健康运行"
        return 0
    else
        echo "❌ $desc (端口$port) - 服务异常"
        return 1
    fi
}

for service_info in "${services[@]}"; do
    IFS=':' read -r service port apis desc <<< "$service_info"
    
    if check_service_health "$service" "$port" "$desc"; then
        ((healthy_count++))
    else
        # 检查日志
        echo "      查看错误日志: tail -20 logs/$service.log"
    fi
done

echo ""
echo "📊 服务状态总览:"
echo "==============================================================================="
echo "| 服务名称           | 端口  | API数量 | 状态   | 访问地址                      |"
echo "==============================================================================="

total_apis=0
for service_info in "${services[@]}"; do
    IFS=':' read -r service port apis desc <<< "$service_info"
    
    # 检查服务状态
    if curl -sf --connect-timeout 2 "http://localhost:$port/health" > /dev/null 2>&1; then
        status="✅ 运行"
        total_apis=$((total_apis + apis))
    else
        status="❌ 异常"
    fi
    
    printf "| %-17s | %-4s | %-6s | %-6s | http://localhost:%-4s/health |\n" \
           "$desc" "$port" "${apis}个" "$status" "$port"
done

echo "==============================================================================="
echo "| 总计               | -     | ${total_apis}/387 | ${healthy_count}/${total_services}运行 | 微服务架构已部署              |"
echo "==============================================================================="

# 结果评估
echo ""
if [ $healthy_count -eq $total_services ]; then
    echo "🎉 所有微服务启动成功！"
    echo "📈 API接口可用性: ${total_apis}/387 ($(echo "scale=1; $total_apis*100/387" | bc)%)"
    echo ""
    echo "🌐 访问入口:"
    echo "   前端界面: http://localhost:3000"
    echo "   API网关:  http://localhost:3000/api"
    echo ""
    echo "📋 服务进程 (PID):"
    for i in "${!pids[@]}"; do
        service_info=${services[$i]}
        IFS=':' read -r service port apis desc <<< "$service_info"
        echo "   $desc: ${pids[$i]}"
    done
    
elif [ $healthy_count -gt 0 ]; then
    echo "⚠️  部分微服务启动成功 ($healthy_count/$total_services)"
    echo "📈 API接口可用性: ${total_apis}/387 ($(echo "scale=1; $total_apis*100/387" | bc)%)"
    echo ""
    echo "🔧 排查异常服务:"
    for service_info in "${services[@]}"; do
        IFS=':' read -r service port apis desc <<< "$service_info"
        if ! curl -sf --connect-timeout 2 "http://localhost:$port/health" > /dev/null 2>&1; then
            echo "   查看 $desc 日志: tail -20 logs/$service.log"
        fi
    done
    
else
    echo "❌ 所有微服务启动失败"
    echo ""
    echo "🔧 排查步骤:"
    echo "1. 检查端口占用: netstat -tlnp | grep -E '(4001|4002|4003|4004|4005|4006|4007)'"
    echo "2. 查看服务日志: ls -la logs/"
    echo "3. 检查编译状态: ls -la */target/release/"
    exit 1
fi

# 创建停止脚本
cat > stop_all_services.sh << 'EOF'
#!/bin/bash
echo "🛑 停止所有微服务..."

services=(
    "logging-service"
    "cleaning-service" 
    "strategy-service"
    "performance-service"
    "trading-service"
    "ai-model-service"
    "config-service"
)

for service in "${services[@]}"; do
    if pgrep -f "$service" > /dev/null; then
        echo "   停止 $service..."
        pkill -f "$service"
    fi
done

sleep 2
echo "✅ 所有服务已停止"
EOF

chmod +x stop_all_services.sh

echo ""
echo "💡 管理命令:"
echo "   停止所有服务: ./stop_all_services.sh"
echo "   查看服务状态: ps aux | grep -E 'service' | grep -v grep"
echo "   查看端口占用: ss -tlnp | grep -E '(4001|4002|4003|4004|4005|4006|4007)'" 