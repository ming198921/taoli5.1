#!/bin/bash

# 5.1套利系统统一构建脚本
# 支持多种构建模式，确保跨平台兼容性

set -e

echo "🚀 5.1套利系统统一构建脚本"
echo "=================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 构建模式
BUILD_MODE=${1:-"default"}

echo -e "${YELLOW}构建模式: $BUILD_MODE${NC}"

case $BUILD_MODE in
    "full"|"production")
        echo -e "${GREEN}使用完整优化模式${NC}"
        FEATURES="--features full_optimization"
        PROFILE="--release"
        ;;
    "minimal"|"compatibility")
        echo -e "${YELLOW}使用最小兼容模式${NC}"
        FEATURES="--no-default-features --features minimal"
        PROFILE="--release"
        ;;
    "dev"|"development")
        echo -e "${GREEN}使用开发模式${NC}"
        FEATURES=""
        PROFILE=""
        ;;
    *)
        echo -e "${GREEN}使用默认模式${NC}"
        FEATURES=""
        PROFILE="--release"
        ;;
esac

echo "=================================="

# 1. 检查环境
echo -e "${YELLOW}1. 检查构建环境...${NC}"

# 检查Rust
if ! command -v rustc &> /dev/null; then
    echo -e "${RED}❌ Rust未安装，请先安装Rust${NC}"
    exit 1
fi

# 检查Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python3未安装，请先安装Python3${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 环境检查通过${NC}"

# 2. 清理旧构建
echo -e "${YELLOW}2. 清理旧构建文件...${NC}"
cargo clean
rm -rf target/
echo -e "${GREEN}✅ 清理完成${NC}"

# 3. 构建主系统
echo -e "${YELLOW}3. 构建主系统...${NC}"

# 首先尝试默认构建
if cargo build $PROFILE $FEATURES --workspace; then
    echo -e "${GREEN}✅ 主系统构建成功${NC}"
else
    echo -e "${RED}❌ 主系统构建失败，尝试兼容模式${NC}"

    # 如果失败，尝试兼容模式
    echo -e "${YELLOW}🔄 切换到兼容模式构建...${NC}"
    if cargo build $PROFILE --no-default-features --features minimal --workspace; then
        echo -e "${GREEN}✅ 兼容模式构建成功${NC}"
    else
        echo -e "${RED}❌ 兼容模式构建也失败${NC}"
        echo -e "${RED}请检查依赖和代码问题${NC}"
        exit 1
    fi
fi

# 4. 构建超低延迟系统
echo -e "${YELLOW}4. 构建超低延迟订单系统...${NC}"

if cargo build $PROFILE $FEATURES --bin ultra_latency_test; then
    echo -e "${GREEN}✅ 超低延迟系统构建成功${NC}"
else
    echo -e "${YELLOW}⚠️  超低延迟系统构建失败，尝试兼容模式${NC}"
    if cargo build $PROFILE --no-default-features --bin ultra_latency_test; then
        echo -e "${GREEN}✅ 超低延迟系统兼容模式构建成功${NC}"
    else
        echo -e "${RED}❌ 超低延迟系统构建失败${NC}"
        exit 1
    fi
fi

# 5. 安装Python依赖
echo -e "${YELLOW}5. 安装Python依赖...${NC}"
if [ -f requirements.txt ]; then
    pip3 install -r requirements.txt
    echo -e "${GREEN}✅ Python依赖安装完成${NC}"
else
    echo -e "${YELLOW}⚠️  requirements.txt不存在，跳过Python依赖安装${NC}"
fi

# 6. 验证构建
echo -e "${YELLOW}6. 验证构建结果...${NC}"

# 检查关键二进制文件
if [ -f target/release/arbitrage-system ] || [ -f target/debug/arbitrage-system ]; then
    echo -e "${GREEN}✅ 主系统二进制文件存在${NC}"
else
    echo -e "${RED}❌ 主系统二进制文件不存在${NC}"
fi

# 7. 设置权限
echo -e "${YELLOW}7. 设置执行权限...${NC}"
chmod +x *.sh
chmod +x *.py
echo -e "${GREEN}✅ 权限设置完成${NC}"

echo "=================================="
echo -e "${GREEN}🎉 构建完成！${NC}"

echo -e "${YELLOW}快速启动命令:${NC}"
echo "  开发模式: ./start-gateway.sh"
echo "  检查状态: ./check_system_status.sh"
echo "  运行测试: python3 test-all-apis.js"

echo -e "${YELLOW}故障排除:${NC}"
echo "  如果遇到socket错误: $0 minimal"
echo "  如果遇到依赖错误: cargo build --no-default-features"

echo "=================================="