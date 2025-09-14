#!/bin/bash

# 🚀 5.1套利系统启动脚本
# 按照依赖顺序启动各个模块

set -e

echo "🚀 Starting 5.1 Arbitrage System..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查Rust环境
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

# 检查Node.js环境
if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm not found. Please install Node.js.${NC}"
    exit 1
fi

# 启动函数
start_module() {
    local module=$1
    local path=$2
    local command=$3

    echo -e "${BLUE}🔄 Starting $module...${NC}"
    cd "$path"

    if [ "$command" == "cargo" ]; then
        cargo run --release &
    elif [ "$command" == "npm" ]; then
        npm run dev &
    else
        $command &
    fi

    local pid=$!
    echo -e "${GREEN}✅ $module started (PID: $pid)${NC}"
    sleep 2
    cd - > /dev/null
}

# 1. 启动QingXi数据处理模块
echo -e "${YELLOW}📊 Phase 1: Starting Data Processing Layer${NC}"
start_module "QingXi Data Processor" "./qingxi" "cargo"

# 2. 启动CeLue策略模块
echo -e "${YELLOW}🎯 Phase 2: Starting Strategy Execution Layer${NC}"
start_module "CeLue Strategy Engine" "./celue" "cargo"

# 3. 启动系统架构模块
echo -e "${YELLOW}🏛️ Phase 3: Starting System Architecture Layer${NC}"
start_module "System Architecture" "./jiagou" "cargo"

# 4. 启动超低延迟模块
echo -e "${YELLOW}⚡ Phase 4: Starting Ultra-Low Latency Layer${NC}"
start_module "Ultra-Latency Engine" "./ultra-latency" "cargo"

# 5. 启动前端界面
echo -e "${YELLOW}🖥️ Phase 5: Starting Frontend Interface${NC}"
start_module "Frontend Dashboard" "./frontend" "npm"

# 等待所有服务启动
echo -e "${BLUE}⏳ Waiting for all services to initialize...${NC}"
sleep 10

# 检查服务状态
echo -e "${YELLOW}📋 System Status Check:${NC}"
echo "🔹 QingXi Data Processor: $(pgrep -f "qingxi" > /dev/null && echo "✅ Running" || echo "❌ Stopped")"
echo "🔹 CeLue Strategy Engine: $(pgrep -f "celue" > /dev/null && echo "✅ Running" || echo "❌ Stopped")"
echo "🔹 System Architecture: $(pgrep -f "jiagou" > /dev/null && echo "✅ Running" || echo "❌ Stopped")"
echo "🔹 Ultra-Latency Engine: $(pgrep -f "ultra-latency" > /dev/null && echo "✅ Running" || echo "❌ Stopped")"
echo "🔹 Frontend Dashboard: $(pgrep -f "vite\|webpack\|npm" > /dev/null && echo "✅ Running" || echo "❌ Stopped")"

echo ""
echo -e "${GREEN}🎉 5.1 Arbitrage System is now running!${NC}"
echo ""
echo "📱 Access Points:"
echo "🔗 Frontend Dashboard: http://localhost:5173"
echo "🔗 QingXi API: http://localhost:8081"
echo "🔗 CeLue API: http://localhost:8082"
echo "🔗 Architecture API: http://localhost:8083"
echo "🔗 Ultra-Latency Monitor: http://localhost:8084"
echo ""
echo "🛑 To stop the system: ./tools/stop_system.sh"

# 保持脚本运行
wait