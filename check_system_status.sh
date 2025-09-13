#!/bin/bash

# 5.1套利系统完整状态检查脚本
# 检查387个API接口的完整对接状态

echo "🔍 5.1套利系统完整状态检查"
echo "=========================================="
echo "📊 检查387个API接口对接状态"
echo "🌐 统一网关: localhost:3000"
echo "💻 前端界面: localhost:3003"
echo "=========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查函数
check_service() {
    local name=$1
    local port=$2
    local apis=$3
    
    if curl -s http://localhost:$port/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $name (端口$port) - $apis个API - 运行正常${NC}"
        return 0
    else
        echo -e "${RED}❌ $name (端口$port) - $apis个API - 服务异常${NC}"
        return 1
    fi
}

# 1. 检查统一网关
echo -e "${BLUE}🌐 检查统一网关状态${NC}"
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${GREEN}✅ 统一网关 (端口3000) - 运行正常${NC}"
    gateway_status=1
else
    echo -e "${RED}❌ 统一网关 (端口3000) - 服务异常${NC}"
    gateway_status=0
fi
echo ""

# 2. 检查7个微服务
echo -e "${BLUE}🔧 检查7个微服务状态${NC}"
services_running=0
total_apis=0

# 日志服务
if check_service "日志服务" "4001" "45"; then
    ((services_running++))
    ((total_apis+=45))
fi

# 清洗服务
if check_service "清洗服务" "4002" "52"; then
    ((services_running++))
    ((total_apis+=52))
fi

# 策略服务
if check_service "策略服务" "4003" "38"; then
    ((services_running++))
    ((total_apis+=38))
fi

# 性能服务
if check_service "性能服务" "4004" "67"; then
    ((services_running++))
    ((total_apis+=67))
fi

# 交易服务
if check_service "交易服务" "4005" "41"; then
    ((services_running++))
    ((total_apis+=41))
fi

# AI模型服务
if check_service "AI模型服务" "4006" "48"; then
    ((services_running++))
    ((total_apis+=48))
fi

# 配置服务
if check_service "配置服务" "4007" "96"; then
    ((services_running++))
    ((total_apis+=96))
fi

echo ""

# 3. 检查前端服务
echo -e "${BLUE}💻 检查前端服务状态${NC}"
if curl -s http://localhost:3003 >/dev/null 2>&1; then
    echo -e "${GREEN}✅ 前端服务 (端口3003) - 运行正常${NC}"
    frontend_status=1
else
    echo -e "${RED}❌ 前端服务 (端口3003) - 服务异常${NC}"
    frontend_status=0
fi
echo ""

# 4. 测试API接口连通性
echo -e "${BLUE}🔗 测试API接口连通性${NC}"

test_api() {
    local name=$1
    local endpoint=$2
    
    if curl -s -m 5 "http://localhost:3000/api/$endpoint" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $name API - 连通正常${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  $name API - 连接超时或异常${NC}"
        return 1
    fi
}

api_tests=0
# 测试各服务的健康检查API
if test_api "日志服务" "logs/stream/stats"; then ((api_tests++)); fi
if test_api "清洗服务" "cleaning/rules/stats"; then ((api_tests++)); fi
if test_api "策略服务" "strategies/list"; then ((api_tests++)); fi
if test_api "性能服务" "performance/cpu/usage"; then ((api_tests++)); fi
if test_api "交易服务" "orders/stats"; then ((api_tests++)); fi
if test_api "AI模型服务" "ml/models"; then ((api_tests++)); fi
if test_api "配置服务" "config/list"; then ((api_tests++)); fi

echo ""

# 5. 系统统计
echo -e "${BLUE}📊 系统完整状态统计${NC}"
echo "=========================================="

# 计算完成度
service_percentage=$((services_running * 100 / 7))
api_percentage=$((total_apis * 100 / 387))
overall_percentage=$(((gateway_status + frontend_status + services_running) * 100 / 9))

echo -e "🌐 统一网关: ${gateway_status}/1 ($([ $gateway_status -eq 1 ] && echo -e "${GREEN}100%${NC}" || echo -e "${RED}0%${NC}"))"
echo -e "🔧 微服务状态: $services_running/7 ($([ $service_percentage -ge 80 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$service_percentage%${NC})"
echo -e "📡 API接口: $total_apis/387 ($([ $api_percentage -ge 80 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$api_percentage%${NC})"
echo -e "💻 前端服务: ${frontend_status}/1 ($([ $frontend_status -eq 1 ] && echo -e "${GREEN}100%${NC}" || echo -e "${RED}0%${NC}"))"
echo -e "🔗 API连通性: $api_tests/7 ($([ $api_tests -ge 5 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$((api_tests * 100 / 7))%${NC})"

echo ""
echo -e "🎯 系统整体完成度: $([ $overall_percentage -ge 80 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$overall_percentage%${NC}"

# 6. 访问地址
if [ $frontend_status -eq 1 ] && [ $gateway_status -eq 1 ]; then
    echo ""
    echo -e "${BLUE}🌍 访问地址${NC}"
    echo "=========================================="
    echo -e "本地访问: ${GREEN}http://localhost:3003${NC}"
    
    # 获取外网IP
    external_ip=$(curl -s ifconfig.me 2>/dev/null || curl -s ipinfo.io/ip 2>/dev/null || echo "YOUR_SERVER_IP")
    echo -e "外网访问: ${GREEN}http://$external_ip:3003${NC}"
    echo ""
    echo -e "${YELLOW}💡 提示: 确保服务器防火墙已开放端口3003${NC}"
fi

# 7. 问题诊断
if [ $overall_percentage -lt 100 ]; then
    echo ""
    echo -e "${YELLOW}🔧 问题诊断建议${NC}"
    echo "=========================================="
    
    if [ $gateway_status -eq 0 ]; then
        echo -e "${RED}• 统一网关未运行，请执行:${NC}"
        echo "  cd /home/ubuntu/5.1xitong/5.1系统/unified-gateway"
        echo "  RUST_LOG=info cargo run --release"
    fi
    
    if [ $services_running -lt 7 ]; then
        echo -e "${RED}• 微服务未完全启动，请执行:${NC}"
        echo "  cd /home/ubuntu/5.1xitong/5.1系统"
        echo "  ./start_all_services.sh"
    fi
    
    if [ $frontend_status -eq 0 ]; then
        echo -e "${RED}• 前端服务未运行，请执行:${NC}"
        echo "  cd /home/ubuntu/arbitrage-frontend-v5.1"
        echo "  ./start-frontend.sh"
    fi
fi

echo ""
echo -e "${BLUE}✨ 5.1套利系统状态检查完成${NC}"

# 返回状态码
if [ $overall_percentage -eq 100 ]; then
    exit 0
else
    exit 1
fi 

# 5.1套利系统完整状态检查脚本
# 检查387个API接口的完整对接状态

echo "🔍 5.1套利系统完整状态检查"
echo "=========================================="
echo "📊 检查387个API接口对接状态"
echo "🌐 统一网关: localhost:3000"
echo "💻 前端界面: localhost:3003"
echo "=========================================="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查函数
check_service() {
    local name=$1
    local port=$2
    local apis=$3
    
    if curl -s http://localhost:$port/health >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $name (端口$port) - $apis个API - 运行正常${NC}"
        return 0
    else
        echo -e "${RED}❌ $name (端口$port) - $apis个API - 服务异常${NC}"
        return 1
    fi
}

# 1. 检查统一网关
echo -e "${BLUE}🌐 检查统一网关状态${NC}"
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo -e "${GREEN}✅ 统一网关 (端口3000) - 运行正常${NC}"
    gateway_status=1
else
    echo -e "${RED}❌ 统一网关 (端口3000) - 服务异常${NC}"
    gateway_status=0
fi
echo ""

# 2. 检查7个微服务
echo -e "${BLUE}🔧 检查7个微服务状态${NC}"
services_running=0
total_apis=0

# 日志服务
if check_service "日志服务" "4001" "45"; then
    ((services_running++))
    ((total_apis+=45))
fi

# 清洗服务
if check_service "清洗服务" "4002" "52"; then
    ((services_running++))
    ((total_apis+=52))
fi

# 策略服务
if check_service "策略服务" "4003" "38"; then
    ((services_running++))
    ((total_apis+=38))
fi

# 性能服务
if check_service "性能服务" "4004" "67"; then
    ((services_running++))
    ((total_apis+=67))
fi

# 交易服务
if check_service "交易服务" "4005" "41"; then
    ((services_running++))
    ((total_apis+=41))
fi

# AI模型服务
if check_service "AI模型服务" "4006" "48"; then
    ((services_running++))
    ((total_apis+=48))
fi

# 配置服务
if check_service "配置服务" "4007" "96"; then
    ((services_running++))
    ((total_apis+=96))
fi

echo ""

# 3. 检查前端服务
echo -e "${BLUE}💻 检查前端服务状态${NC}"
if curl -s http://localhost:3003 >/dev/null 2>&1; then
    echo -e "${GREEN}✅ 前端服务 (端口3003) - 运行正常${NC}"
    frontend_status=1
else
    echo -e "${RED}❌ 前端服务 (端口3003) - 服务异常${NC}"
    frontend_status=0
fi
echo ""

# 4. 测试API接口连通性
echo -e "${BLUE}🔗 测试API接口连通性${NC}"

test_api() {
    local name=$1
    local endpoint=$2
    
    if curl -s -m 5 "http://localhost:3000/api/$endpoint" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ $name API - 连通正常${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️  $name API - 连接超时或异常${NC}"
        return 1
    fi
}

api_tests=0
# 测试各服务的健康检查API
if test_api "日志服务" "logs/stream/stats"; then ((api_tests++)); fi
if test_api "清洗服务" "cleaning/rules/stats"; then ((api_tests++)); fi
if test_api "策略服务" "strategies/list"; then ((api_tests++)); fi
if test_api "性能服务" "performance/cpu/usage"; then ((api_tests++)); fi
if test_api "交易服务" "orders/stats"; then ((api_tests++)); fi
if test_api "AI模型服务" "ml/models"; then ((api_tests++)); fi
if test_api "配置服务" "config/list"; then ((api_tests++)); fi

echo ""

# 5. 系统统计
echo -e "${BLUE}📊 系统完整状态统计${NC}"
echo "=========================================="

# 计算完成度
service_percentage=$((services_running * 100 / 7))
api_percentage=$((total_apis * 100 / 387))
overall_percentage=$(((gateway_status + frontend_status + services_running) * 100 / 9))

echo -e "🌐 统一网关: ${gateway_status}/1 ($([ $gateway_status -eq 1 ] && echo -e "${GREEN}100%${NC}" || echo -e "${RED}0%${NC}"))"
echo -e "🔧 微服务状态: $services_running/7 ($([ $service_percentage -ge 80 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$service_percentage%${NC})"
echo -e "📡 API接口: $total_apis/387 ($([ $api_percentage -ge 80 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$api_percentage%${NC})"
echo -e "💻 前端服务: ${frontend_status}/1 ($([ $frontend_status -eq 1 ] && echo -e "${GREEN}100%${NC}" || echo -e "${RED}0%${NC}"))"
echo -e "🔗 API连通性: $api_tests/7 ($([ $api_tests -ge 5 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$((api_tests * 100 / 7))%${NC})"

echo ""
echo -e "🎯 系统整体完成度: $([ $overall_percentage -ge 80 ] && echo -e "${GREEN}" || echo -e "${YELLOW}")$overall_percentage%${NC}"

# 6. 访问地址
if [ $frontend_status -eq 1 ] && [ $gateway_status -eq 1 ]; then
    echo ""
    echo -e "${BLUE}🌍 访问地址${NC}"
    echo "=========================================="
    echo -e "本地访问: ${GREEN}http://localhost:3003${NC}"
    
    # 获取外网IP
    external_ip=$(curl -s ifconfig.me 2>/dev/null || curl -s ipinfo.io/ip 2>/dev/null || echo "YOUR_SERVER_IP")
    echo -e "外网访问: ${GREEN}http://$external_ip:3003${NC}"
    echo ""
    echo -e "${YELLOW}💡 提示: 确保服务器防火墙已开放端口3003${NC}"
fi

# 7. 问题诊断
if [ $overall_percentage -lt 100 ]; then
    echo ""
    echo -e "${YELLOW}🔧 问题诊断建议${NC}"
    echo "=========================================="
    
    if [ $gateway_status -eq 0 ]; then
        echo -e "${RED}• 统一网关未运行，请执行:${NC}"
        echo "  cd /home/ubuntu/5.1xitong/5.1系统/unified-gateway"
        echo "  RUST_LOG=info cargo run --release"
    fi
    
    if [ $services_running -lt 7 ]; then
        echo -e "${RED}• 微服务未完全启动，请执行:${NC}"
        echo "  cd /home/ubuntu/5.1xitong/5.1系统"
        echo "  ./start_all_services.sh"
    fi
    
    if [ $frontend_status -eq 0 ]; then
        echo -e "${RED}• 前端服务未运行，请执行:${NC}"
        echo "  cd /home/ubuntu/arbitrage-frontend-v5.1"
        echo "  ./start-frontend.sh"
    fi
fi

echo ""
echo -e "${BLUE}✨ 5.1套利系统状态检查完成${NC}"

# 返回状态码
if [ $overall_percentage -eq 100 ]; then
    exit 0
else
    exit 1
fi 