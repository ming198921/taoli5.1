#!/bin/bash

# 5.1套利系统 - 自动服务管理器
# Automated Service Manager for 5.1 Arbitrage System
# 
# 功能:
# 1. 智能服务启动与停止
# 2. 端口冲突检测与解决  
# 3. 依赖关系管理
# 4. 故障转移与恢复
# 5. 性能监控与优化

set -euo pipefail

# 颜色配置
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 配置
SYSTEM_DIR="/home/ubuntu/5.1xitong/5.1系统"
LOG_DIR="/home/ubuntu/5.1xitong/logs"
LOG_FILE="${LOG_DIR}/auto-service-manager.log"
PID_DIR="/tmp/5.1-system-pids"

# 确保目录存在
mkdir -p "$LOG_DIR" "$PID_DIR"

# 服务配置
declare -A SERVICES=(
    ["unified-gateway"]="3000:critical:$SYSTEM_DIR/unified-gateway"
    ["logging-service"]="4001:critical:$SYSTEM_DIR/logging-service"
    ["cleaning-service"]="4002:normal:$SYSTEM_DIR/cleaning-service"
    ["strategy-service"]="4003:critical:$SYSTEM_DIR/strategy-service"
    ["performance-service"]="4004:normal:$SYSTEM_DIR/performance-service"
    ["trading-service"]="4005:critical:$SYSTEM_DIR/trading-service"
    ["ai-model-service"]="4006:normal:$SYSTEM_DIR/ai-model-service"
    ["config-service"]="4007:critical:$SYSTEM_DIR/config-service"
)

# 服务启动顺序 (依赖关系)
STARTUP_ORDER=(
    "config-service"      # 配置服务最先启动
    "logging-service"     # 日志服务
    "unified-gateway"     # 统一网关
    "trading-service"     # 交易服务
    "strategy-service"    # 策略服务
    "cleaning-service"    # 清算服务
    "performance-service" # 性能服务
    "ai-model-service"   # AI模型服务
)

# 日志函数
log() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${CYAN}[INFO]${NC} $1"
    log "INFO" "$1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    log "SUCCESS" "$1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    log "WARN" "$1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    log "ERROR" "$1"
}

# 检查端口是否被占用
check_port() {
    local port=$1
    if ss -tlnp | grep -q ":$port "; then
        return 0  # 端口被占用
    else
        return 1  # 端口未被占用
    fi
}

# 获取占用端口的进程
get_port_process() {
    local port=$1
    ss -tlnp | grep ":$port " | awk -F',' '{print $2}' | awk -F'=' '{print $2}' | head -1
}

# 检查服务是否健康
check_service_health() {
    local service_name=$1
    local port=$2
    local max_retries=3
    local retry=0
    
    while [ $retry -lt $max_retries ]; do
        if curl -sf "http://localhost:$port/health" >/dev/null 2>&1; then
            return 0  # 健康
        fi
        retry=$((retry + 1))
        sleep 2
    done
    
    return 1  # 不健康
}

# 启动单个服务
start_service() {
    local service_name=$1
    local config=${SERVICES[$service_name]}
    
    if [ -z "$config" ]; then
        log_error "Unknown service: $service_name"
        return 1
    fi
    
    IFS=':' read -r port priority service_dir <<< "$config"
    
    log_info "Starting service: $service_name (port: $port, priority: $priority)"
    
    # 检查服务目录
    if [ ! -d "$service_dir" ]; then
        log_error "Service directory not found: $service_dir"
        return 1
    fi
    
    # 检查端口冲突
    if check_port "$port"; then
        local existing_process
        existing_process=$(get_port_process "$port")
        
        if [ -n "$existing_process" ]; then
            log_warn "Port $port is already in use by process: $existing_process"
            
            # 检查是否是同一个服务
            if pgrep -f "$service_name" >/dev/null; then
                if check_service_health "$service_name" "$port"; then
                    log_success "Service $service_name is already running and healthy"
                    return 0
                else
                    log_warn "Service $service_name is running but unhealthy, restarting..."
                    stop_service "$service_name"
                    sleep 3
                fi
            else
                log_error "Port $port is occupied by different process: $existing_process"
                return 1
            fi
        fi
    fi
    
    # 启动服务
    cd "$service_dir"
    
    # 设置环境变量
    export RUST_LOG=info
    export RUST_BACKTRACE=1
    
    # 启动服务并获取PID
    local log_file="${LOG_DIR}/${service_name}.log"
    local pid_file="${PID_DIR}/${service_name}.pid"
    
    log_info "Executing: cargo run --release in $service_dir"
    
    # 使用nohup启动服务
    nohup cargo run --release > "$log_file" 2>&1 &
    local service_pid=$!
    
    echo "$service_pid" > "$pid_file"
    
    log_info "Service $service_name started with PID: $service_pid"
    
    # 等待服务启动
    local wait_time=15
    local elapsed=0
    
    while [ $elapsed -lt $wait_time ]; do
        if check_service_health "$service_name" "$port"; then
            log_success "Service $service_name is healthy (took ${elapsed}s)"
            return 0
        fi
        
        sleep 2
        elapsed=$((elapsed + 2))
        echo -n "."
    done
    
    echo ""
    log_error "Service $service_name failed to start or become healthy within ${wait_time}s"
    
    # 显示启动日志
    if [ -f "$log_file" ]; then
        log_warn "Last 10 lines of $service_name log:"
        tail -10 "$log_file"
    fi
    
    return 1
}

# 停止单个服务
stop_service() {
    local service_name=$1
    local pid_file="${PID_DIR}/${service_name}.pid"
    
    log_info "Stopping service: $service_name"
    
    # 尝试从PID文件停止
    if [ -f "$pid_file" ]; then
        local pid
        pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping $service_name with PID: $pid"
            kill -TERM "$pid"
            
            # 等待进程结束
            local wait_count=0
            while kill -0 "$pid" 2>/dev/null && [ $wait_count -lt 10 ]; do
                sleep 1
                wait_count=$((wait_count + 1))
            done
            
            # 如果仍然运行，强制杀死
            if kill -0 "$pid" 2>/dev/null; then
                log_warn "Force killing $service_name with PID: $pid"
                kill -KILL "$pid"
            fi
        fi
        rm -f "$pid_file"
    fi
    
    # 确保杀死所有相关进程
    pkill -f "$service_name" 2>/dev/null || true
    
    log_success "Service $service_name stopped"
}

# 启动所有服务
start_all_services() {
    log_info "=========================================="
    log_info "Starting 5.1 Arbitrage System Services"
    log_info "=========================================="
    
    local failed_services=()
    local success_count=0
    
    for service_name in "${STARTUP_ORDER[@]}"; do
        echo ""
        log_info "[$((success_count + 1))/${#STARTUP_ORDER[@]}] Starting $service_name..."
        
        if start_service "$service_name"; then
            success_count=$((success_count + 1))
        else
            failed_services+=("$service_name")
            
            # 对于关键服务，如果启动失败则停止整个流程
            local config=${SERVICES[$service_name]}
            IFS=':' read -r port priority service_dir <<< "$config"
            
            if [ "$priority" = "critical" ]; then
                log_error "Critical service $service_name failed to start. Stopping startup process."
                return 1
            fi
        fi
        
        # 服务间启动延迟
        sleep 2
    done
    
    echo ""
    log_info "=========================================="
    log_success "Startup completed: $success_count/${#STARTUP_ORDER[@]} services started"
    
    if [ ${#failed_services[@]} -gt 0 ]; then
        log_warn "Failed services: ${failed_services[*]}"
    fi
    
    log_info "=========================================="
    
    return 0
}

# 停止所有服务
stop_all_services() {
    log_info "=========================================="
    log_info "Stopping 5.1 Arbitrage System Services"
    log_info "=========================================="
    
    # 按照相反顺序停止服务
    local reverse_order=()
    for ((i=${#STARTUP_ORDER[@]}-1; i>=0; i--)); do
        reverse_order+=("${STARTUP_ORDER[i]}")
    done
    
    for service_name in "${reverse_order[@]}"; do
        stop_service "$service_name"
        sleep 1
    done
    
    log_success "All services stopped"
    log_info "=========================================="
}

# 重启所有服务
restart_all_services() {
    log_info "Restarting all services..."
    stop_all_services
    sleep 5
    start_all_services
}

# 检查所有服务状态
check_all_services() {
    log_info "=========================================="
    log_info "5.1 Arbitrage System Service Status"
    log_info "=========================================="
    
    local healthy_count=0
    local total_count=${#SERVICES[@]}
    
    printf "%-20s %-8s %-10s %-15s %s\n" "SERVICE" "PORT" "STATUS" "PRIORITY" "HEALTH"
    printf "%s\n" "--------------------------------------------------------------------------------"
    
    for service_name in "${STARTUP_ORDER[@]}"; do
        local config=${SERVICES[$service_name]}
        IFS=':' read -r port priority service_dir <<< "$config"
        
        local status="STOPPED"
        local health_status="UNKNOWN"
        
        if check_port "$port"; then
            status="RUNNING"
            if check_service_health "$service_name" "$port"; then
                health_status="HEALTHY"
                healthy_count=$((healthy_count + 1))
            else
                health_status="UNHEALTHY"
            fi
        fi
        
        local status_color=""
        local health_color=""
        
        case $status in
            "RUNNING") status_color="${GREEN}" ;;
            *) status_color="${RED}" ;;
        esac
        
        case $health_status in
            "HEALTHY") health_color="${GREEN}" ;;
            "UNHEALTHY") health_color="${RED}" ;;
            *) health_color="${YELLOW}" ;;
        esac
        
        printf "%-20s %-8s ${status_color}%-10s${NC} %-15s ${health_color}%s${NC}\n" \
            "$service_name" "$port" "$status" "$priority" "$health_status"
    done
    
    printf "%s\n" "--------------------------------------------------------------------------------"
    log_info "Overall Status: $healthy_count/$total_count services healthy"
    log_info "=========================================="
    
    return 0
}

# 自动修复不健康的服务
auto_repair() {
    log_info "Starting automatic repair process..."
    
    local repaired_services=()
    local failed_repairs=()
    
    for service_name in "${STARTUP_ORDER[@]}"; do
        local config=${SERVICES[$service_name]}
        IFS=':' read -r port priority service_dir <<< "$config"
        
        if check_port "$port"; then
            if ! check_service_health "$service_name" "$port"; then
                log_warn "Service $service_name is unhealthy, attempting repair..."
                
                if stop_service "$service_name" && sleep 3 && start_service "$service_name"; then
                    repaired_services+=("$service_name")
                    log_success "Successfully repaired $service_name"
                else
                    failed_repairs+=("$service_name")
                    log_error "Failed to repair $service_name"
                fi
            fi
        else
            log_warn "Service $service_name is not running, attempting to start..."
            
            if start_service "$service_name"; then
                repaired_services+=("$service_name")
                log_success "Successfully started $service_name"
            else
                failed_repairs+=("$service_name")
                log_error "Failed to start $service_name"
            fi
        fi
    done
    
    echo ""
    log_info "Repair Summary:"
    if [ ${#repaired_services[@]} -gt 0 ]; then
        log_success "Repaired services: ${repaired_services[*]}"
    fi
    
    if [ ${#failed_repairs[@]} -gt 0 ]; then
        log_error "Failed repairs: ${failed_repairs[*]}"
        return 1
    fi
    
    log_success "Auto-repair completed successfully"
    return 0
}

# 显示帮助信息
show_help() {
    cat << EOF
5.1套利系统自动服务管理器 (Auto Service Manager)

用法 (Usage):
    $0 [COMMAND] [OPTIONS]

命令 (Commands):
    start [service]     启动服务 (Start service(s))
    stop [service]      停止服务 (Stop service(s))
    restart [service]   重启服务 (Restart service(s))
    status              检查服务状态 (Check service status)
    repair              自动修复不健康服务 (Auto-repair unhealthy services)
    help                显示帮助信息 (Show help)

示例 (Examples):
    $0 start                    # 启动所有服务
    $0 start trading-service    # 启动交易服务
    $0 stop                     # 停止所有服务
    $0 restart                  # 重启所有服务
    $0 status                   # 检查服务状态
    $0 repair                   # 自动修复服务

支持的服务 (Supported Services):
EOF

    for service_name in "${STARTUP_ORDER[@]}"; do
        local config=${SERVICES[$service_name]}
        IFS=':' read -r port priority service_dir <<< "$config"
        printf "    %-20s (port: %s, priority: %s)\n" "$service_name" "$port" "$priority"
    done
}

# 主函数
main() {
    local command=${1:-"help"}
    local service_name=${2:-""}
    
    case $command in
        "start")
            if [ -n "$service_name" ]; then
                if [[ ${SERVICES[$service_name]+_} ]]; then
                    start_service "$service_name"
                else
                    log_error "Unknown service: $service_name"
                    exit 1
                fi
            else
                start_all_services
            fi
            ;;
        "stop")
            if [ -n "$service_name" ]; then
                if [[ ${SERVICES[$service_name]+_} ]]; then
                    stop_service "$service_name"
                else
                    log_error "Unknown service: $service_name"
                    exit 1
                fi
            else
                stop_all_services
            fi
            ;;
        "restart")
            if [ -n "$service_name" ]; then
                if [[ ${SERVICES[$service_name]+_} ]]; then
                    stop_service "$service_name"
                    sleep 3
                    start_service "$service_name"
                else
                    log_error "Unknown service: $service_name"
                    exit 1
                fi
            else
                restart_all_services
            fi
            ;;
        "status")
            check_all_services
            ;;
        "repair")
            auto_repair
            ;;
        "help"|"--help"|"-h")
            show_help
            ;;
        *)
            log_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

# 执行主函数
main "$@"