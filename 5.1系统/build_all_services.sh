#!/bin/bash
set -e

echo "🔨 编译5.1套利系统387个API微服务"
echo "======================================="

cd "$(dirname "$0")"

# 检查Rust环境
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo未安装，请先安装Rust"
    exit 1
fi

# 微服务列表 (按优先级排序)
services=(
    "logging-service:45:日志监控"
    "cleaning-service:52:清洗配置" 
    "strategy-service:38:策略监控"
    "performance-service:67:性能调优"
    "trading-service:41:交易监控"
    "ai-model-service:48:AI模型"
    "config-service:96:配置管理"
)

echo "📊 开始编译7个微服务 (387个API接口)..."

# 编译每个微服务
for service_info in "${services[@]}"; do
    IFS=':' read -r service apis desc <<< "$service_info"
    
    echo ""
    echo "🔧 编译 $desc ($service) - $apis个API..."
    
    if [ -d "$service" ]; then
        cd "$service"
        
        # 清理之前的构建
        cargo clean > /dev/null 2>&1 || true
        
        # 编译release版本
        echo "   正在编译..."
        if cargo build --release; then
            echo "   ✅ $service 编译成功"
            
            # 检查可执行文件
            if [ -f "target/release/$service" ]; then
                echo "   ✅ 可执行文件已生成: target/release/$service"
            else
                echo "   ⚠️  可执行文件未找到，检查项目配置"
            fi
        else
            echo "   ❌ $service 编译失败"
            exit 1
        fi
        
        cd ..
    else
        echo "   ❌ 目录 $service 不存在"
        exit 1
    fi
done

echo ""
echo "🎉 所有微服务编译完成！"
echo ""

# 验证编译结果
echo "📋 编译结果验证:"
echo "============================================"
total_apis=0
compiled_services=0

for service_info in "${services[@]}"; do
    IFS=':' read -r service apis desc <<< "$service_info"
    
    if [ -f "$service/target/release/$service" ]; then
        size=$(ls -lh "$service/target/release/$service" | awk '{print $5}')
        echo "✅ $desc: $apis个API - 可执行文件大小: $size"
        total_apis=$((total_apis + apis))
        compiled_services=$((compiled_services + 1))
    else
        echo "❌ $desc: 编译失败"
    fi
done

echo "============================================"
echo "📊 编译统计:"
echo "   成功编译服务: $compiled_services/7"
echo "   可用API接口: $total_apis/387"
echo ""

if [ $compiled_services -eq 7 ] && [ $total_apis -eq 387 ]; then
    echo "🎯 所有微服务编译成功，准备启动！"
    echo ""
    echo "下一步: 运行 ./start_all_services.sh 启动所有服务"
else
    echo "❌ 编译不完整，请检查错误信息"
    exit 1
fi 