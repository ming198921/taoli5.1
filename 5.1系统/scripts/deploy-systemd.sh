#!/bin/bash

# 5.1套利系统 Systemd 部署脚本
# 用于将系统部署为systemd服务

set -e

# 配置变量
PROJECT_ROOT="/home/ubuntu/5.1xitong/5.1系统"
SERVICE_NAME="arbitrage-system"
SERVICE_FILE="${PROJECT_ROOT}/systemd/${SERVICE_NAME}.service"
SYSTEMD_DIR="/etc/systemd/system"
BINARY_PATH="${PROJECT_ROOT}/target/release/${SERVICE_NAME}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查是否为root用户或有sudo权限
check_permissions() {
    log_info "检查权限..."
    if [ "$EUID" -eq 0 ]; then
        log_warning "正在以root用户运行"
    elif ! sudo -n true 2>/dev/null; then
        log_error "需要sudo权限来操作systemd服务"
        exit 1
    fi
    log_success "权限检查通过"
}

# 检查必要文件是否存在
check_files() {
    log_info "检查必要文件..."
    
    if [ ! -f "$SERVICE_FILE" ]; then
        log_error "服务文件不存在: $SERVICE_FILE"
        exit 1
    fi
    
    if [ ! -f "$BINARY_PATH" ]; then
        log_warning "二进制文件不存在: $BINARY_PATH"
        log_info "尝试编译项目..."
        cd "$PROJECT_ROOT"
        if ! cargo build --release --bin "$SERVICE_NAME"; then
            log_error "编译失败"
            exit 1
        fi
        log_success "编译完成"
    fi
    
    log_success "文件检查通过"
}

# 创建必要的目录
create_directories() {
    log_info "创建必要目录..."
    
    mkdir -p "${PROJECT_ROOT}/logs"
    mkdir -p "${PROJECT_ROOT}/data"
    mkdir -p "/tmp/qingxi_cache"
    
    # 设置权限
    chown -R ubuntu:ubuntu "${PROJECT_ROOT}/logs"
    chown -R ubuntu:ubuntu "${PROJECT_ROOT}/data"
    chown -R ubuntu:ubuntu "/tmp/qingxi_cache"
    
    log_success "目录创建完成"
}

# 停止现有服务
stop_existing_service() {
    log_info "检查现有服务..."
    
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        log_info "停止现有服务..."
        sudo systemctl stop "$SERVICE_NAME"
        log_success "服务已停止"
    else
        log_info "服务未在运行"
    fi
}

# 安装服务文件
install_service() {
    log_info "安装systemd服务文件..."
    
    # 复制服务文件
    sudo cp "$SERVICE_FILE" "$SYSTEMD_DIR/"
    
    # 设置权限
    sudo chmod 644 "${SYSTEMD_DIR}/${SERVICE_NAME}.service"
    
    # 重新加载systemd配置
    sudo systemctl daemon-reload
    
    log_success "服务文件安装完成"
}

# 启用并启动服务
enable_and_start_service() {
    log_info "启用并启动服务..."
    
    # 启用服务（开机自启动）
    sudo systemctl enable "$SERVICE_NAME"
    log_success "服务已启用（开机自启动）"
    
    # 启动服务
    sudo systemctl start "$SERVICE_NAME"
    log_success "服务已启动"
    
    # 等待一下让服务启动
    sleep 3
}

# 检查服务状态
check_service_status() {
    log_info "检查服务状态..."
    
    if systemctl is-active --quiet "$SERVICE_NAME"; then
        log_success "✅ 服务运行正常"
        
        # 显示状态信息
        echo
        log_info "服务详细状态:"
        sudo systemctl status "$SERVICE_NAME" --no-pager -l
        
        echo
        log_info "最新日志:"
        sudo journalctl -u "$SERVICE_NAME" -n 10 --no-pager
        
        echo
        log_info "服务管理命令:"
        echo "  查看状态: sudo systemctl status $SERVICE_NAME"
        echo "  停止服务: sudo systemctl stop $SERVICE_NAME"
        echo "  启动服务: sudo systemctl start $SERVICE_NAME"
        echo "  重启服务: sudo systemctl restart $SERVICE_NAME"
        echo "  查看日志: sudo journalctl -u $SERVICE_NAME -f"
        echo "  禁用服务: sudo systemctl disable $SERVICE_NAME"
        
    else
        log_error "❌ 服务启动失败"
        
        echo
        log_error "错误日志:"
        sudo journalctl -u "$SERVICE_NAME" -n 20 --no-pager
        exit 1
    fi
}

# 测试API连接
test_api() {
    log_info "测试API连接..."
    
    # 等待服务完全启动
    sleep 5
    
    if curl -f -s http://localhost:8080/health > /dev/null; then
        log_success "✅ API服务响应正常"
        
        # 显示API信息
        echo
        log_info "API服务信息:"
        echo "  健康检查: http://localhost:8080/health"
        echo "  系统状态: http://localhost:8080/api/system/status" 
        echo "  WebSocket: ws://localhost:8080"
        
    else
        log_warning "⚠️  API服务可能还未完全启动，请稍后手动检查"
        echo "  可以运行: curl http://localhost:8080/health"
    fi
}

# 主函数
main() {
    echo "================================================"
    echo "      5.1套利系统 Systemd 部署脚本"
    echo "================================================"
    echo
    
    check_permissions
    check_files
    create_directories
    stop_existing_service
    install_service
    enable_and_start_service
    check_service_status
    test_api
    
    echo
    echo "================================================"
    log_success "🎉 部署完成！"
    echo "================================================"
    
    # 显示下一步操作
    echo
    log_info "下一步操作:"
    echo "1. 前端现在可以通过环境变量 REACT_APP_DEPLOYMENT_TYPE=systemd 来使用systemd控制"
    echo "2. 前端API将通过 /api/control/systemd/* 路径控制服务"
    echo "3. 系统日志可通过 journalctl -u $SERVICE_NAME -f 查看"
}

# 清理函数（Ctrl+C时调用）
cleanup() {
    echo
    log_warning "部署被中断"
    exit 1
}

# 捕获信号
trap cleanup SIGINT SIGTERM

# 运行主函数
main "$@"