#!/bin/bash

# 5.1套利系统 - 综合管理仪表板
# Comprehensive Management Dashboard for 5.1 Arbitrage System

set -euo pipefail

# 颜色配置
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'

# 配置
SYSTEM_DIR="/home/ubuntu/5.1xitong"
TOOLS_DIR="$SYSTEM_DIR"
FRONTEND_URL="http://localhost:3003"
DASHBOARD_URL="$FRONTEND_URL/dashboard"

# 清屏并显示标题
clear_and_header() {
    clear
    echo -e "${WHITE}╔══════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${WHITE}║                     5.1套利系统 - 综合管理仪表板                                 ║${NC}"
    echo -e "${WHITE}║                   5.1 Arbitrage System - Management Dashboard                ║${NC}"
    echo -e "${WHITE}╚══════════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# 显示系统状态概览
show_system_overview() {
    echo -e "${CYAN}🖥️  系统概览 (System Overview)${NC}"
    echo -e "${CYAN}═══════════════════════════════════${NC}"
    
    # 系统信息
    echo -e "${WHITE}系统时间:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
    echo -e "${WHITE}运行时间:${NC} $(uptime -p)"
    
    # 资源使用
    local cpu_usage=$(top -bn1 | grep "load average" | awk '{print $10,$11,$12}')
    local memory_info=$(free -h | grep Mem | awk '{printf "%.1fG/%.1fG (%.1f%%)", $3/1024, $2/1024, $3/$2*100}')
    local disk_info=$(df -h / | tail -1 | awk '{printf "%s/%s (%s)", $3, $2, $5}')
    
    echo -e "${WHITE}CPU负载:${NC} $cpu_usage"
    echo -e "${WHITE}内存使用:${NC} $memory_info"
    echo -e "${WHITE}磁盘使用:${NC} $disk_info"
    echo ""
}

# 显示服务状态
show_service_status() {
    echo -e "${CYAN}🔧 微服务状态 (Microservice Status)${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════${NC}"
    
    # 使用服务管理器检查状态
    if [ -f "$TOOLS_DIR/auto-service-manager.sh" ]; then
        "$TOOLS_DIR/auto-service-manager.sh" status 2>/dev/null | grep -E "SERVICE|RUNNING|STOPPED|HEALTHY|UNHEALTHY" | head -15
    else
        echo -e "${RED}❌ Service manager not found${NC}"
    fi
    echo ""
}

# 显示前端状态
show_frontend_status() {
    echo -e "${CYAN}🌐 前端服务状态 (Frontend Status)${NC}"
    echo -e "${CYAN}═══════════════════════════════════════${NC}"
    
    # 检查前端端口
    if ss -tlnp | grep -q ":3003"; then
        echo -e "${GREEN}✅ 前端服务运行中 (Frontend Running)${NC}"
        echo -e "${WHITE}   本地访问:${NC} http://localhost:3003"
        echo -e "${WHITE}   外网访问:${NC} http://$(curl -s ifconfig.me 2>/dev/null || echo "YOUR_SERVER_IP"):3003"
        
        # 检查Dashboard页面
        if curl -sf "$DASHBOARD_URL" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ Dashboard页面可访问${NC}"
        else
            echo -e "${YELLOW}⚠️  Dashboard页面检查失败${NC}"
        fi
    else
        echo -e "${RED}❌ 前端服务未运行${NC}"
    fi
    
    # 检查API服务器
    if ss -tlnp | grep -q ":3001"; then
        echo -e "${GREEN}✅ API服务器运行中 (API Server Running)${NC}"
    else
        echo -e "${RED}❌ API服务器未运行${NC}"
    fi
    echo ""
}

# 显示可用工具
show_available_tools() {
    echo -e "${CYAN}🛠️  可用工具 (Available Tools)${NC}"
    echo -e "${CYAN}═════════════════════════════════════${NC}"
    
    echo -e "${WHITE}1.${NC} 微服务诊断工具     - ${YELLOW}node $TOOLS_DIR/microservice-diagnostic-tool.js${NC}"
    echo -e "${WHITE}2.${NC} 自动服务管理器     - ${YELLOW}$TOOLS_DIR/auto-service-manager.sh${NC}"
    echo -e "${WHITE}3.${NC} 自愈式监控系统     - ${YELLOW}python3 $TOOLS_DIR/self-healing-monitor.py${NC}"
    echo -e "${WHITE}4.${NC} 系统综合仪表板     - ${YELLOW}$TOOLS_DIR/system-dashboard.sh${NC}"
    echo ""
}

# 显示快速操作菜单
show_quick_actions() {
    echo -e "${CYAN}⚡ 快速操作 (Quick Actions)${NC}"
    echo -e "${CYAN}════════════════════════════════${NC}"
    
    echo -e "${WHITE}[1]${NC} 检查所有服务状态"
    echo -e "${WHITE}[2]${NC} 启动所有服务"
    echo -e "${WHITE}[3]${NC} 重启所有服务" 
    echo -e "${WHITE}[4]${NC} 自动修复故障服务"
    echo -e "${WHITE}[5]${NC} 启动持续监控"
    echo -e "${WHITE}[6]${NC} 打开前端Dashboard"
    echo -e "${WHITE}[7]${NC} 查看详细系统报告"
    echo -e "${WHITE}[8]${NC} 清理日志文件"
    echo -e "${WHITE}[0]${NC} 退出"
    echo ""
}

# 执行快速操作
execute_action() {
    local action=$1
    
    case $action in
        1)
            echo -e "${CYAN}🔍 检查所有服务状态...${NC}"
            echo ""
            if [ -f "$TOOLS_DIR/microservice-diagnostic-tool.js" ]; then
                node "$TOOLS_DIR/microservice-diagnostic-tool.js" status
            else
                "$TOOLS_DIR/auto-service-manager.sh" status
            fi
            ;;
        2)
            echo -e "${CYAN}🚀 启动所有服务...${NC}"
            echo ""
            "$TOOLS_DIR/auto-service-manager.sh" start
            ;;
        3)
            echo -e "${CYAN}🔄 重启所有服务...${NC}"
            echo ""
            "$TOOLS_DIR/auto-service-manager.sh" restart
            ;;
        4)
            echo -e "${CYAN}🔧 自动修复故障服务...${NC}"
            echo ""
            if [ -f "$TOOLS_DIR/self-healing-monitor.py" ]; then
                python3 "$TOOLS_DIR/self-healing-monitor.py" repair
            else
                "$TOOLS_DIR/auto-service-manager.sh" repair
            fi
            ;;
        5)
            echo -e "${CYAN}👁️  启动持续监控 (按Ctrl+C停止)...${NC}"
            echo ""
            if [ -f "$TOOLS_DIR/microservice-diagnostic-tool.js" ]; then
                node "$TOOLS_DIR/microservice-diagnostic-tool.js" monitor 30000
            else
                echo -e "${YELLOW}持续监控工具不可用，使用定时检查模式${NC}"
                while true; do
                    clear_and_header
                    show_system_overview
                    show_service_status
                    echo -e "${CYAN}下次检查: 30秒后... (按Ctrl+C停止)${NC}"
                    sleep 30
                done
            fi
            ;;
        6)
            echo -e "${CYAN}🌐 打开前端Dashboard...${NC}"
            echo ""
            echo -e "${GREEN}Dashboard访问地址:${NC}"
            echo -e "${WHITE}   本地: ${YELLOW}$DASHBOARD_URL${NC}"
            echo -e "${WHITE}   外网: ${YELLOW}http://$(curl -s ifconfig.me 2>/dev/null || echo "YOUR_SERVER_IP"):3003/dashboard${NC}"
            echo ""
            if command -v xdg-open >/dev/null 2>&1; then
                xdg-open "$DASHBOARD_URL" 2>/dev/null &
            elif command -v open >/dev/null 2>&1; then
                open "$DASHBOARD_URL" 2>/dev/null &
            else
                echo -e "${WHITE}请在浏览器中打开上述链接${NC}"
            fi
            ;;
        7)
            echo -e "${CYAN}📊 生成详细系统报告...${NC}"
            echo ""
            
            # 生成综合报告
            local report_file="/home/ubuntu/5.1xitong/logs/system_report_$(date +%Y%m%d_%H%M%S).txt"
            
            {
                echo "5.1套利系统综合报告 - $(date)"
                echo "========================================"
                echo ""
                
                echo "1. 系统信息"
                echo "----------"
                uname -a
                uptime
                free -h
                df -h
                echo ""
                
                echo "2. 微服务状态"
                echo "------------"
                "$TOOLS_DIR/auto-service-manager.sh" status 2>/dev/null || echo "Service manager unavailable"
                echo ""
                
                echo "3. 网络连接"
                echo "----------"
                ss -tlnp | grep -E ":(3000|3001|3003|400[0-9])" | head -10
                echo ""
                
                echo "4. 运行进程"
                echo "----------"
                ps aux | grep -E "(vite|node|cargo|unified-gateway|service)" | grep -v grep | head -10
                echo ""
                
                echo "5. 日志文件"
                echo "----------"
                ls -la /home/ubuntu/5.1xitong/logs/ 2>/dev/null | head -10 || echo "Logs directory not found"
                
            } > "$report_file"
            
            echo -e "${GREEN}✅ 系统报告已生成: $report_file${NC}"
            echo ""
            head -20 "$report_file"
            echo "..."
            ;;
        8)
            echo -e "${CYAN}🧹 清理日志文件...${NC}"
            echo ""
            
            local cleaned_count=0
            
            # 清理30天以上的日志
            if [ -d "/home/ubuntu/5.1xitong/logs" ]; then
                cleaned_count=$(find /home/ubuntu/5.1xitong/logs -name "*.log" -mtime +30 -type f | wc -l)
                find /home/ubuntu/5.1xitong/logs -name "*.log" -mtime +30 -type f -delete
            fi
            
            # 清理大型日志文件
            if [ -d "/home/ubuntu/5.1xitong/logs" ]; then
                find /home/ubuntu/5.1xitong/logs -name "*.log" -size +100M -exec truncate -s 50M {} \;
            fi
            
            echo -e "${GREEN}✅ 清理完成: 删除了 $cleaned_count 个旧日志文件${NC}"
            echo -e "${GREEN}✅ 大型日志文件已截断${NC}"
            ;;
        0)
            echo -e "${GREEN}👋 感谢使用5.1套利系统管理仪表板！${NC}"
            exit 0
            ;;
        *)
            echo -e "${RED}❌ 无效的选择${NC}"
            ;;
    esac
}

# 交互模式
interactive_mode() {
    while true; do
        clear_and_header
        show_system_overview
        show_service_status
        show_frontend_status
        show_available_tools
        show_quick_actions
        
        echo -ne "${WHITE}请选择操作 (Enter choice): ${NC}"
        read -r choice
        
        if [[ $choice =~ ^[0-8]$ ]]; then
            echo ""
            execute_action "$choice"
            
            if [ "$choice" != "0" ] && [ "$choice" != "5" ]; then
                echo ""
                echo -ne "${WHITE}按回车键继续... ${NC}"
                read -r
            fi
        else
            echo -e "${RED}❌ 请输入有效数字 (0-8)${NC}"
            sleep 2
        fi
    done
}

# 非交互模式
non_interactive_mode() {
    local command=$1
    
    case $command in
        "status")
            show_system_overview
            show_service_status
            show_frontend_status
            ;;
        "start")
            execute_action 2
            ;;
        "restart")
            execute_action 3
            ;;
        "repair")
            execute_action 4
            ;;
        "monitor")
            execute_action 5
            ;;
        "report")
            execute_action 7
            ;;
        "cleanup")
            execute_action 8
            ;;
        *)
            echo "使用方法: $0 [status|start|restart|repair|monitor|report|cleanup]"
            echo "或者不带参数运行进入交互模式"
            exit 1
            ;;
    esac
}

# 主函数
main() {
    # 检查必要的工具
    if [ ! -f "$TOOLS_DIR/auto-service-manager.sh" ]; then
        echo -e "${RED}❌ 错误: 找不到自动服务管理器${NC}"
        echo -e "${WHITE}请确保文件存在: $TOOLS_DIR/auto-service-manager.sh${NC}"
        exit 1
    fi
    
    # 创建日志目录
    mkdir -p "/home/ubuntu/5.1xitong/logs"
    
    if [ $# -eq 0 ]; then
        # 交互模式
        interactive_mode
    else
        # 非交互模式
        non_interactive_mode "$1"
    fi
}

# 信号处理
trap 'echo -e "\n${GREEN}👋 Dashboard已停止${NC}"; exit 0' INT TERM

# 执行主函数
main "$@"