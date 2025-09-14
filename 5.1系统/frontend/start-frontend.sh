#!/bin/bash

# 5.1套利系统前端启动脚本
# 端口: 3003 (外网可访问)
# API网关: localhost:3000

echo "🚀 启动5.1套利系统前端..."
echo "=========================================="
echo "📊 前端端口: 3003"
echo "🌐 API网关: localhost:3000"
echo "🔗 387个API接口统一管理"
echo "=========================================="

# 检查Node.js版本
echo "🔍 检查Node.js版本..."
node_version=$(node -v 2>/dev/null)
if [ $? -ne 0 ]; then
    echo "❌ Node.js未安装，请先安装Node.js 18+"
    exit 1
fi

echo "✅ Node.js版本: $node_version"

# 检查依赖
if [ ! -d "node_modules" ]; then
    echo "📦 安装依赖包..."
    npm install
fi

# 检查统一网关是否运行
echo "🔍 检查统一网关状态..."
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo "✅ 统一网关 (localhost:3000) 运行正常"
else
    echo "⚠️  统一网关 (localhost:3000) 未运行，请先启动后端服务"
    echo "   提示: 前端可以启动，但API调用可能失败"
fi

# 检查端口是否被占用
if lsof -Pi :3003 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  端口3003已被占用，正在清理..."
    pkill -f "vite.*3003" 2>/dev/null || true
    sleep 2
fi

# 设置环境变量
export VITE_API_BASE_URL=http://localhost:3000
export VITE_WS_BASE_URL=ws://localhost:3000

echo "🌍 启动前端服务 (外网可访问)..."
echo "   本地访问: http://localhost:3003"
echo "   外网访问: http://YOUR_SERVER_IP:3003"
echo ""

# 启动开发服务器
npm run dev

echo "👋 前端服务已停止" 

# 5.1套利系统前端启动脚本
# 端口: 3003 (外网可访问)
# API网关: localhost:3000

echo "🚀 启动5.1套利系统前端..."
echo "=========================================="
echo "📊 前端端口: 3003"
echo "🌐 API网关: localhost:3000"
echo "🔗 387个API接口统一管理"
echo "=========================================="

# 检查Node.js版本
echo "🔍 检查Node.js版本..."
node_version=$(node -v 2>/dev/null)
if [ $? -ne 0 ]; then
    echo "❌ Node.js未安装，请先安装Node.js 18+"
    exit 1
fi

echo "✅ Node.js版本: $node_version"

# 检查依赖
if [ ! -d "node_modules" ]; then
    echo "📦 安装依赖包..."
    npm install
fi

# 检查统一网关是否运行
echo "🔍 检查统一网关状态..."
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo "✅ 统一网关 (localhost:3000) 运行正常"
else
    echo "⚠️  统一网关 (localhost:3000) 未运行，请先启动后端服务"
    echo "   提示: 前端可以启动，但API调用可能失败"
fi

# 检查端口是否被占用
if lsof -Pi :3003 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "⚠️  端口3003已被占用，正在清理..."
    pkill -f "vite.*3003" 2>/dev/null || true
    sleep 2
fi

# 设置环境变量
export VITE_API_BASE_URL=http://localhost:3000
export VITE_WS_BASE_URL=ws://localhost:3000

echo "🌍 启动前端服务 (外网可访问)..."
echo "   本地访问: http://localhost:3003"
echo "   外网访问: http://YOUR_SERVER_IP:3003"
echo ""

# 启动开发服务器
npm run dev

echo "👋 前端服务已停止" 