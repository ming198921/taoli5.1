#!/bin/bash
# 币安三角套利配置启动脚本
# 作者: 套利系统5.1
# 日期: 2024-01-15

set -e

echo "🚀 启动币安三角套利配置..."

# 配置路径
CONFIG_PATH="/home/ubuntu/5.1xitong/5.1系统/config/binance_triangular_config.toml"
GATEWAY_URL="http://localhost:8080"
CLI_CONTROLLER="/home/ubuntu/5.1xitong/arbitrage-cli-controller.py"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# 检查系统状态
check_system_status() {
    log_step "检查系统状态..."
    
    if python3 "$CLI_CONTROLLER" system status > /dev/null 2>&1; then
        log_info "✅ 系统运行正常"
    else
        log_warn "⚠️  系统可能未启动，尝试启动..."
        python3 "$CLI_CONTROLLER" system start
        sleep 5
    fi
}

# 停止现有策略
stop_existing_strategies() {
    log_step "停止现有策略..."
    
    python3 "$CLI_CONTROLLER" strategy stop-all || {
        log_warn "没有运行中的策略需要停止"
    }
}

# 配置币安交易所
configure_binance_exchange() {
    log_step "配置币安交易所..."
    
    # 通过统一网关API配置交易所
    curl -X POST "$GATEWAY_URL/api/config/exchanges" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "binance",
            "enabled": true,
            "api_key": "aJS2cL8LyIHw5PfUeKvYkdfM1pf0ewaVKI7m0GkwsXs3qYhrVQgHz8mGjkCZ6xL0",
            "api_secret": "rGrBCqmSxT0khRWFuh72eG6irYw0z82BSvT7cxcIRR1yrxAdJ4jiODnjUkXPLGzk",
            "sandbox_mode": true,
            "base_url": "https://testnet.binance.vision",
            "ws_url": "wss://testnet.binance.vision/ws/"
        }' || {
        log_error "配置交易所失败"
        exit 1
    }
    
    log_info "✅ 币安交易所配置完成"
}

# 配置币种列表
configure_symbols() {
    log_step "配置50个币种..."
    
    # 币种列表
    SYMBOLS=(
        "BTCUSDT" "ETHUSDT" "BNBUSDT" "XRPUSDT" "ADAUSDT"
        "DOTUSDT" "LINKUSDT" "LTCUSDT" "BCHUSDT" "XLMUSDT"
        "EOSUSDT" "TRXUSDT" "ETCUSDT" "UNIUSDT" "AAVEUSDT"
        "SUSHIUSDT" "COMPUSDT" "MKRUSDT" "SOLUSDT" "AVAXUSDT"
        "MATICUSDT" "ATOMUSDT" "FILUSDT" "VETUSDT" "ICPUSDT"
        "THETAUSDT" "ALGOUSDT" "NEARUSDT" "FTMUSDT" "SANDUSDT"
        "MANAUSDT" "AXSUSDT" "CHZUSDT" "ARBUSDT" "OPUSDT"
        "LRCUSDT" "ZKSUSDT" "STRKUSDT" "BUSDUSDT" "USDCUSDT"
        "TUSDUSDT" "BTCETH" "BTCBNB" "ETHBNB" "BNBBTC"
        "BNBETH" "ADABTC" "XRPBTC" "DOTBTC" "LINKBTC"
    )
    
    # 通过API配置币种
    for symbol in "${SYMBOLS[@]}"; do
        curl -X POST "$GATEWAY_URL/api/config/symbols" \
            -H "Content-Type: application/json" \
            -d "{\"symbol\": \"$symbol\", \"exchange\": \"binance\", \"enabled\": true}" \
            -s > /dev/null || {
            log_warn "配置币种 $symbol 失败，继续..."
        }
    done
    
    log_info "✅ 50个币种配置完成"
}

# 配置三角套利策略
configure_triangular_strategy() {
    log_step "配置三角套利策略（利润0.15%）..."
    
    curl -X POST "$GATEWAY_URL/api/strategies" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "binance_triangular",
            "type": "triangular",
            "exchange": "binance",
            "enabled": true,
            "config": {
                "min_profit_pct": 0.15,
                "max_position_size_usd": 1000.0,
                "max_slippage_pct": 0.05,
                "execution_timeout_ms": 5000,
                "concurrent_opportunities": 3,
                "retry_attempts": 3
            }
        }' || {
        log_error "配置三角套利策略失败"
        exit 1
    }
    
    log_info "✅ 三角套利策略配置完成（最小利润: 0.15%）"
}

# 配置风控参数
configure_risk_management() {
    log_step "配置风控参数..."
    
    curl -X POST "$GATEWAY_URL/api/config/risk" \
        -H "Content-Type: application/json" \
        -d '{
            "max_daily_loss_usd": 100.0,
            "max_positions": 3,
            "stop_loss_pct": 2.0,
            "max_drawdown_pct": 5.0,
            "enabled": true
        }' || {
        log_error "配置风控参数失败"
        exit 1
    }
    
    log_info "✅ 风控参数配置完成（最大日损失: $100）"
}

# 启动数据采集
start_data_collection() {
    log_step "启动数据采集..."
    
    python3 "$CLI_CONTROLLER" data start binance || {
        log_error "启动数据采集失败"
        exit 1
    }
    
    log_info "✅ 币安数据采集已启动"
}

# 启动策略
start_strategy() {
    log_step "启动三角套利策略..."
    
    python3 "$CLI_CONTROLLER" strategy start triangular binance || {
        log_error "启动策略失败"
        exit 1
    }
    
    log_info "✅ 三角套利策略已启动"
}

# 验证配置
verify_configuration() {
    log_step "验证配置..."
    
    # 检查系统状态
    echo "📊 系统状态:"
    python3 "$CLI_CONTROLLER" system status
    
    # 检查策略状态
    echo -e "\n📈 策略状态:"
    python3 "$CLI_CONTROLLER" strategy status
    
    # 检查数据采集状态
    echo -e "\n📡 数据采集状态:"
    python3 "$CLI_CONTROLLER" data status
    
    log_info "✅ 配置验证完成"
}

# 测试下单功能
test_order_placement() {
    log_step "测试下单功能（模拟）..."
    
    # 获取当前价格
    echo "🔍 获取BTCUSDT当前价格..."
    curl -s "$GATEWAY_URL/api/market/price/binance/BTCUSDT" | jq '.'
    
    # 模拟测试订单
    echo -e "\n🧪 执行模拟测试订单..."
    curl -X POST "$GATEWAY_URL/api/orders/test" \
        -H "Content-Type: application/json" \
        -d '{
            "exchange": "binance",
            "symbol": "BTCUSDT",
            "side": "BUY",
            "type": "MARKET",
            "quantity": 0.001,
            "test_only": true
        }' | jq '.'
    
    log_info "✅ 测试下单完成"
}

# 主函数
main() {
    echo "==============================================="
    echo "🏛️  币安三角套利系统5.1配置脚本"
    echo "==============================================="
    
    check_system_status
    stop_existing_strategies
    configure_binance_exchange
    configure_symbols
    configure_triangular_strategy
    configure_risk_management
    start_data_collection
    start_strategy
    verify_configuration
    test_order_placement
    
    echo "==============================================="
    log_info "🎉 币安三角套利配置完成！"
    echo "==============================================="
    
    echo "📋 配置摘要:"
    echo "   • 交易所: 币安 (测试环境)"
    echo "   • 币种数量: 50个"
    echo "   • 策略类型: 三角套利"
    echo "   • 最小利润: 0.15%"
    echo "   • 最大持仓: $1000"
    echo "   • 风控: 最大日损失$100"
    echo ""
    echo "🎯 监控命令:"
    echo "   python3 $CLI_CONTROLLER system status"
    echo "   python3 $CLI_CONTROLLER strategy status"
    echo "   python3 $CLI_CONTROLLER monitor profits"
    echo ""
    echo "⚠️  注意: 当前使用测试API，请确保在生产环境前切换到实盘API"
}

# 执行主函数
main "$@"