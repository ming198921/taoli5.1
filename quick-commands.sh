#!/bin/bash
# 套利系统5.1快速命令脚本
# 这个脚本提供了一些常用的快捷命令别名

# 设置颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 脚本路径
CLI_CONTROLLER="/home/ubuntu/5.1xitong/arbitrage-cli-controller.py"

# 打印标题
print_title() {
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}                   套利系统5.1 快速命令工具                              ${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════════════${NC}"
}

# 显示帮助信息
show_help() {
    print_title
    echo -e "${GREEN}使用方法: ./quick-commands.sh [命令]${NC}"
    echo ""
    echo -e "${YELLOW}系统控制:${NC}"
    echo "  status          - 查看系统状态"
    echo "  start           - 启动整个系统"
    echo "  stop            - 停止整个系统"  
    echo "  restart         - 重启整个系统"
    echo ""
    echo -e "${YELLOW}数据处理:${NC}"
    echo "  data-start      - 启动所有数据采集"
    echo "  data-stop       - 停止所有数据采集"
    echo "  data-status     - 查看数据采集状态"
    echo "  data-clean      - 执行数据清洗"
    echo ""
    echo -e "${YELLOW}策略管理:${NC}"
    echo "  strategies      - 列出所有策略"
    echo "  start-inter     - 启动跨交易所策略"
    echo "  start-tri       - 启动三角套利策略"
    echo "  stop-all-strat  - 停止所有策略"
    echo "  strategy-status - 查看策略状态"
    echo ""
    echo -e "${YELLOW}AI风控:${NC}"
    echo "  risk-status     - 查看风控状态"
    echo "  emergency       - 紧急停止"
    echo "  set-max-exp     - 设置最大敞口限制"
    echo ""
    echo -e "${YELLOW}AI模型:${NC}"
    echo "  ai-models       - 列出所有AI模型"
    echo "  train-risk      - 训练风险模型"
    echo "  train-price     - 训练价格预测模型"
    echo ""
    echo -e "${YELLOW}监控工具:${NC}"
    echo "  logs            - 查看实时日志"
    echo "  monitor         - 性能监控"
    echo "  tail-strategy   - 查看策略日志"
    echo ""
    echo -e "${YELLOW}快速诊断:${NC}"
    echo "  health-check    - 完整健康检查"
    echo "  quick-test      - 快速功能测试"
    echo "  benchmark       - 性能基准测试"
}

# 检查Python环境
check_python() {
    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}❌ Python3未安装${NC}"
        exit 1
    fi
}

# 执行CLI控制器命令
run_cli() {
    check_python
    python3 "$CLI_CONTROLLER" "$@"
}

# 主命令处理
case "$1" in
    # 系统控制
    "status")
        echo -e "${BLUE}📊 检查系统状态...${NC}"
        run_cli system status
        ;;
    "start")
        echo -e "${GREEN}🚀 启动套利系统...${NC}"
        
        # 先检查是否已经在运行
        if curl -s http://localhost:3000/health >/dev/null 2>&1; then
            echo -e "${YELLOW}⚠️ 系统已在运行中！${NC}"
            run_cli system status
        else
            run_cli system start
        fi
        ;;
    "stop")
        echo -e "${YELLOW}🛑 停止套利系统...${NC}"
        run_cli system stop
        ;;
    "restart")
        echo -e "${PURPLE}🔄 重启套利系统...${NC}"
        run_cli system restart
        ;;
    
    # 数据处理
    "data-start")
        echo -e "${CYAN}📊 启动数据采集...${NC}"
        run_cli data start-all
        ;;
    "data-stop")
        echo -e "${YELLOW}⏹️ 停止数据采集...${NC}"
        run_cli data stop-all
        ;;
    "data-status")
        echo -e "${BLUE}📈 检查数据状态...${NC}"
        run_cli data status
        ;;
    "data-clean")
        echo -e "${GREEN}🧹 执行数据清洗...${NC}"
        run_cli data clean
        ;;
    
    # 策略管理
    "strategies")
        echo -e "${PURPLE}📋 列出所有策略...${NC}"
        run_cli strategy list
        ;;
    "start-inter")
        echo -e "${GREEN}🚀 启动跨交易所策略...${NC}"
        run_cli strategy start inter_exchange_production
        ;;
    "start-tri")
        echo -e "${GREEN}🚀 启动三角套利策略...${NC}"
        run_cli strategy start triangular_production
        ;;
    "stop-all-strat")
        echo -e "${YELLOW}⏹️ 停止所有策略...${NC}"
        run_cli strategy stop inter_exchange_production
        run_cli strategy stop triangular_production
        ;;
    "strategy-status")
        echo -e "${BLUE}📊 查看策略状态...${NC}"
        run_cli strategy status
        ;;
    
    # AI风控
    "risk-status")
        echo -e "${BLUE}🛡️ 查看风控状态...${NC}"
        run_cli risk status
        ;;
    "emergency")
        echo -e "${RED}🚨 执行紧急停止...${NC}"
        read -p "确定要执行紧急停止吗? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            run_cli risk emergency-stop
        else
            echo "操作已取消"
        fi
        ;;
    "set-max-exp")
        read -p "请输入最大敞口限制 (USDT): " max_exposure
        if [[ $max_exposure =~ ^[0-9]+\.?[0-9]*$ ]]; then
            echo -e "${GREEN}⚙️ 设置最大敞口: $max_exposure USDT${NC}"
            run_cli risk set-limit max_exposure "$max_exposure"
        else
            echo -e "${RED}❌ 输入的金额格式不正确${NC}"
        fi
        ;;
    
    # AI模型
    "ai-models")
        echo -e "${PURPLE}🤖 列出AI模型...${NC}"
        run_cli ai models
        ;;
    "train-risk")
        echo -e "${YELLOW}🎓 训练风险模型...${NC}"
        run_cli ai train risk_model 30
        ;;
    "train-price")
        echo -e "${YELLOW}🎓 训练价格预测模型...${NC}"
        run_cli ai train price_prediction 7
        ;;
    
    # 监控工具
    "logs")
        echo -e "${CYAN}📋 查看实时日志...${NC}"
        run_cli logs tail all 100
        ;;
    "monitor")
        echo -e "${GREEN}📈 启动性能监控...${NC}"
        run_cli monitor performance --duration 300
        ;;
    "tail-strategy")
        echo -e "${CYAN}📋 查看策略日志...${NC}"
        run_cli logs tail strategy 50
        ;;
    
    # 快速诊断
    "health-check")
        echo -e "${BLUE}🔍 执行完整健康检查...${NC}"
        echo ""
        echo "1. 检查系统状态..."
        run_cli system status
        echo ""
        echo "2. 检查数据采集..."
        run_cli data status
        echo ""
        echo "3. 检查策略状态..."
        run_cli strategy status
        echo ""
        echo "4. 检查风控状态..."
        run_cli risk status
        echo ""
        echo -e "${GREEN}✅ 健康检查完成${NC}"
        ;;
    
    "quick-test")
        echo -e "${YELLOW}🧪 执行快速功能测试...${NC}"
        
        # 测试API连通性
        echo "测试统一网关连通性..."
        if curl -s -f "http://localhost:3000/health" >/dev/null 2>&1; then
            echo "✅ 统一网关: 正常"
        else
            echo "❌ 统一网关: 异常"
        fi
        
        # 测试各微服务
        services=(4001 4002 4003 4004 4005 4006 4007)
        service_names=("日志服务" "清洗服务" "策略服务" "性能服务" "交易服务" "AI模型服务" "配置服务")
        
        for i in "${!services[@]}"; do
            port=${services[$i]}
            name=${service_names[$i]}
            if curl -s -f "http://localhost:$port/health" >/dev/null 2>&1; then
                echo "✅ $name: 正常"
            else
                echo "❌ $name: 异常"
            fi
        done
        
        echo -e "${GREEN}🎉 快速测试完成${NC}"
        ;;
    
    "benchmark")
        echo -e "${PURPLE}🏃 执行性能基准测试...${NC}"
        cd /home/ubuntu/5.1xitong
        if [[ -f "simple_benchmark.py" ]]; then
            python3 simple_benchmark.py
        else
            echo -e "${YELLOW}⚠️ 基准测试脚本不存在${NC}"
        fi
        ;;
    
    # 交互式菜单
    "menu"|"")
        while true; do
            clear
            print_title
            echo -e "${GREEN}请选择操作:${NC}"
            echo ""
            echo "  1) 系统状态检查"
            echo "  2) 启动整个系统"
            echo "  3) 停止整个系统"
            echo "  4) 数据采集管理"
            echo "  5) 策略管理"
            echo "  6) AI风控管理"
            echo "  7) 查看日志"
            echo "  8) 性能监控"
            echo "  9) 健康检查"
            echo "  0) 退出"
            echo ""
            read -p "请输入选择 [0-9]: " choice
            
            case $choice in
                1) run_cli system status; read -p "按任意键继续..." ;;
                2) run_cli system start; read -p "按任意键继续..." ;;
                3) run_cli system stop; read -p "按任意键继续..." ;;
                4) 
                    echo "数据采集管理:"
                    echo "1) 启动数据采集  2) 停止数据采集  3) 查看状态  4) 数据清洗"
                    read -p "选择 [1-4]: " data_choice
                    case $data_choice in
                        1) run_cli data start-all ;;
                        2) run_cli data stop-all ;;
                        3) run_cli data status ;;
                        4) run_cli data clean ;;
                    esac
                    read -p "按任意键继续..."
                    ;;
                5)
                    echo "策略管理:"
                    echo "1) 列出策略  2) 启动跨交易所  3) 启动三角套利  4) 查看状态"
                    read -p "选择 [1-4]: " strat_choice
                    case $strat_choice in
                        1) run_cli strategy list ;;
                        2) run_cli strategy start inter_exchange_production ;;
                        3) run_cli strategy start triangular_production ;;
                        4) run_cli strategy status ;;
                    esac
                    read -p "按任意键继续..."
                    ;;
                6) 
                    echo "AI风控管理:"
                    echo "1) 查看状态  2) 紧急停止"
                    read -p "选择 [1-2]: " risk_choice
                    case $risk_choice in
                        1) run_cli risk status ;;
                        2) run_cli risk emergency-stop ;;
                    esac
                    read -p "按任意键继续..."
                    ;;
                7) run_cli logs tail all 50; read -p "按任意键继续..." ;;
                8) run_cli monitor performance --duration 60 ;;
                9) 
                    ./quick-commands.sh health-check
                    read -p "按任意键继续..."
                    ;;
                0) echo "再见！"; exit 0 ;;
                *) echo "无效选择"; sleep 1 ;;
            esac
        done
        ;;
    
    *)
        show_help
        ;;
esac