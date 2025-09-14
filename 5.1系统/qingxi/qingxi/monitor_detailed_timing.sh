#!/bin/bash

# QingXi 5.1 详细时间分析监控脚本
# 专门分析250个交易对从获取到清洗完毕的精确时间

set -euo pipefail

# 配置参数
MONITOR_INTERVAL=${1:-600}  # 默认10分钟
ANALYSIS_DURATION=300       # 分析时长5分钟
LOG_FILE="/tmp/qingxi_timing_analysis.log"
REPORT_DIR="./timing_reports"
TEMP_DIR="/tmp/qingxi_timing"

# 创建必要目录
mkdir -p "$REPORT_DIR" "$TEMP_DIR"

# 日志函数
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

# 获取QingXi进程PID
get_qingxi_pid() {
    ps aux | grep market_data_module | grep -v grep | awk '{print $2}' | head -1
}

# 捕获实时JSON日志
capture_json_logs() {
    local pid="$1"
    local output_file="$2"
    local duration="$3"
    
    log "🔍 开始捕获PID $pid 的JSON日志，持续 ${duration}秒..."
    
    # 清空输出文件
    > "$output_file"
    
    # 方法1: 直接从进程的标准输出读取
    if [[ -r "/proc/$pid/fd/1" ]]; then
        timeout "$duration" tail -f "/proc/$pid/fd/1" 2>/dev/null | \
        grep -E '(timestamp|Received|cleaning|successful|Published)' | \
        head -5000 >> "$output_file" 2>/dev/null &
    fi
    
    # 方法2: 监控系统日志
    timeout "$duration" journalctl -f --since "1 minute ago" -u "*qingxi*" 2>/dev/null | \
    grep -E '(timestamp|Received|cleaning|successful)' | \
    head -5000 >> "$output_file" 2>/dev/null &
    
    # 方法3: 如果有权限，使用strace监控写入
    if command -v strace >/dev/null && [[ $(id -u) -eq 0 ]]; then
        timeout "$duration" strace -e trace=write -p "$pid" 2>&1 | \
        grep -E '(timestamp|Received|cleaning|successful)' | \
        head -3000 >> "$output_file" 2>/dev/null &
    fi
    
    sleep "$duration"
    
    # 停止所有后台作业
    jobs -p | xargs -r kill 2>/dev/null || true
    
    local line_count=$(wc -l < "$output_file" 2>/dev/null || echo 0)
    log "✅ JSON日志捕获完成, 收集了 $line_count 行数据"
}

# 解析时间戳为纳秒
parse_timestamp() {
    local timestamp="$1"
    # 解析ISO 8601格式: 2025-08-17T06:27:19.832474Z
    # 转换为纳秒时间戳
    date -d "$timestamp" +%s%N 2>/dev/null || echo "0"
}

# 分析单个交易对的时间
analyze_trading_pair_timing() {
    local log_file="$1"
    local exchange="$2"
    local symbol="$3"
    local temp_file="$TEMP_DIR/${exchange}_${symbol//\//_}.tmp"
    
    # 提取该交易对的相关日志
    grep -E "\"source\":\"$exchange\".*\"symbol\":\"$symbol\"|\"$exchange.*$symbol\"|$exchange@$symbol" "$log_file" > "$temp_file" 2>/dev/null || true
    
    if [[ ! -s "$temp_file" ]]; then
        return 1
    fi
    
    local received_time=""
    local cleaning_time=""
    local successful_time=""
    local published_time=""
    
    # 解析时间戳
    while IFS= read -r line; do
        if [[ "$line" =~ \"timestamp\":\"([^\"]+)\" ]]; then
            local ts="${BASH_REMATCH[1]}"
            
            if [[ "$line" =~ (Received.*OrderBook|Starting.*collector) ]]; then
                received_time="$ts"
            elif [[ "$line" =~ (Performing.*cleaning|cleaning.*for) ]]; then
                cleaning_time="$ts"
            elif [[ "$line" =~ (cleaning.*successful|validation.*passed) ]]; then
                successful_time="$ts"
            elif [[ "$line" =~ (Published.*NATS|sent.*strategy) ]]; then
                published_time="$ts"
            fi
        fi
    done < "$temp_file"
    
    # 计算时间差(微秒)
    if [[ -n "$received_time" && -n "$successful_time" ]]; then
        local start_ns=$(parse_timestamp "$received_time")
        local end_ns=$(parse_timestamp "$successful_time")
        
        if [[ "$start_ns" -gt 0 && "$end_ns" -gt 0 && "$end_ns" -gt "$start_ns" ]]; then
            local diff_us=$(( (end_ns - start_ns) / 1000 ))
            echo "$exchange,$symbol,$diff_us,$received_time,$successful_time,SUCCESS"
            return 0
        fi
    fi
    
    echo "$exchange,$symbol,0,,$,FAILED"
    return 1
}

# 分析所有交易对
analyze_all_trading_pairs() {
    local log_file="$1"
    local output_file="$2"
    
    log "📊 开始分析所有交易对的时间数据..."
    
    # CSV头部
    echo "Exchange,Symbol,TimingUs,ReceivedTime,CompletedTime,Status" > "$output_file"
    
    # 预定义的交易对列表 (基于配置文件)
    local exchanges=("binance" "bybit" "huobi" "okx" "gateio")
    local symbols=("BTC/USDT" "ETH/USDT" "BNB/USDT" "XRP/USDT" "ADA/USDT" "DOGE/USDT" "MATIC/USDT" "SOL/USDT" "DOT/USDT" "LTC/USDT" 
                   "AVAX/USDT" "SHIB/USDT" "TRX/USDT" "UNI/USDT" "ATOM/USDT" "ETC/USDT" "XLM/USDT" "LINK/USDT" "BCH/USDT" "NEAR/USDT"
                   "APE/USDT" "MANA/USDT" "SAND/USDT" "CRV/USDT" "GRT/USDT" "ENJ/USDT" "CHZ/USDT" "SUSHI/USDT" "BAT/USDT" "ZEC/USDT"
                   "COMP/USDT" "MKR/USDT" "YFI/USDT" "AAVE/USDT" "SNX/USDT" "REN/USDT" "KNC/USDT" "ZRX/USDT" "REP/USDT" "BAL/USDT"
                   "STORJ/USDT" "ANT/USDT" "BNT/USDT" "MLN/USDT" "NMR/USDT" "RLC/USDT" "LRC/USDT" "FIL/USDT" "ICP/USDT" "RUNE/USDT")
    
    local total_pairs=0
    local successful_pairs=0
    
    for exchange in "${exchanges[@]}"; do
        for symbol in "${symbols[@]}"; do
            if analyze_trading_pair_timing "$log_file" "$exchange" "$symbol" >> "$output_file"; then
                ((successful_pairs++))
            fi
            ((total_pairs++))
        done
    done
    
    log "✅ 分析完成: $successful_pairs/$total_pairs 个交易对有时间数据"
}

# 生成详细报告
generate_detailed_report() {
    local analysis_file="$1"
    local report_file="$REPORT_DIR/qingxi_timing_report_$(date +%Y%m%d_%H%M%S).md"
    
    log "📝 生成详细时间报告: $report_file"
    
    if [[ ! -s "$analysis_file" ]] || [[ $(wc -l < "$analysis_file") -le 1 ]]; then
        log "❌ 分析文件为空或无数据"
        return 1
    fi
    
    cat > "$report_file" << EOF
# 🚀 QingXi 5.1 数据清洗时间分析报告

## 📊 报告概览

| 项目 | 值 |
|------|-----|
| 报告时间 | $(date '+%Y-%m-%d %H:%M:%S') |
| 系统版本 | QingXi 5.1 Enhanced |
| 分析模式 | 精确时间戳分析 |
| 目标交易对 | 250个 (5交易所×50币种) |

## 🖥️ 系统运行状态

| 指标 | 状态 | 数值 |
|------|------|------|
| 进程状态 | $(ps aux | grep market_data_module | grep -v grep >/dev/null && echo "✅ 运行中" || echo "❌ 未运行") | PID: $(get_qingxi_pid || echo "N/A") |
| CPU使用率 | $(top -bn1 | grep "market_data_module" | awk '{print $9}' | head -1 || echo "0")% | |
| 内存使用率 | $(free | awk '/^Mem:/{printf "%.1f%%", $3/$2 * 100.0}') | $(free -h | awk '/^Mem:/{print $3"/"$2}') |
| 运行时长 | $(ps -o etime= -p $(get_qingxi_pid 2>/dev/null || echo 1) 2>/dev/null | tr -d ' ' || echo "N/A") | |
| 系统负载 | $(uptime | awk -F'load average:' '{print $2}' | tr -d ' ') | |
| 缓存磁盘 | $(df -h cache/ 2>/dev/null | tail -1 | awk '{print $5}' || echo "N/A") used | |

## ⏱️ 数据清洗时间分析

EOF

    # 分析时间数据
    local temp_data="$TEMP_DIR/analysis_summary.tmp"
    
    # 跳过头部，处理数据
    tail -n +2 "$analysis_file" > "$temp_data"
    
    if [[ -s "$temp_data" ]]; then
        local total_pairs=$(wc -l < "$temp_data")
        local successful_pairs=$(grep -c "SUCCESS" "$temp_data" || echo 0)
        local failed_pairs=$(grep -c "FAILED" "$temp_data" || echo 0)
        
        # 计算时间统计 (只针对成功的)
        local timing_stats=""
        if [[ $successful_pairs -gt 0 ]]; then
            timing_stats=$(grep "SUCCESS" "$temp_data" | cut -d',' -f3 | sort -n | awk '
            BEGIN { sum=0; count=0; }
            { 
                if($1 > 0) {
                    times[count++] = $1;
                    sum += $1;
                }
            }
            END {
                if(count > 0) {
                    avg = sum/count;
                    min = times[0];
                    max = times[count-1];
                    p99_idx = int(count * 0.99);
                    if(p99_idx >= count) p99_idx = count-1;
                    p99 = times[p99_idx];
                    printf "%.1f,%.1f,%.1f,%.1f", avg, min, max, p99;
                } else {
                    print "0,0,0,0";
                }
            }')
            
            IFS=',' read -r avg_time min_time max_time p99_time <<< "$timing_stats"
        else
            avg_time=0; min_time=0; max_time=0; p99_time=0
        fi
        
        cat >> "$report_file" << EOF
### 📈 整体性能汇总

| 性能指标 | 实际值 | 目标值 | 状态 |
|----------|--------|--------|------|
| 总交易对数 | $total_pairs | 250 | $([ $total_pairs -ge 200 ] && echo "✅ 良好" || echo "⚠️ 偏少") |
| 成功处理数 | $successful_pairs | >240 | $([ $successful_pairs -ge 240 ] && echo "✅ 优秀" || [ $successful_pairs -ge 200 ] && echo "👍 良好" || echo "❌ 需改进") |
| 处理成功率 | $(( successful_pairs * 100 / total_pairs ))% | >95% | $([ $(( successful_pairs * 100 / total_pairs )) -ge 95 ] && echo "✅ 达标" || echo "❌ 未达标") |
| 平均清洗时间 | ${avg_time}μs | <300μs | $(awk "BEGIN{print ($avg_time < 300) ? \"✅ 优秀\" : ($avg_time < 500) ? \"👍 良好\" : \"❌ 超时\"}") |
| 最快清洗时间 | ${min_time}μs | <100μs | $(awk "BEGIN{print ($min_time < 100) ? \"🚀 极快\" : \"👍 良好\"}") |
| 最慢清洗时间 | ${max_time}μs | <1000μs | $(awk "BEGIN{print ($max_time < 1000) ? \"✅ 正常\" : \"⚠️ 偏慢\"}") |
| P99延迟 | ${p99_time}μs | <600μs | $(awk "BEGIN{print ($p99_time < 600) ? \"✅ 达标\" : \"❌ 超标\"}") |

### 📊 各交易所性能详情

EOF

        # 各交易所统计
        for exchange in "binance" "bybit" "huobi" "okx" "gateio"; do
            local exchange_data="$TEMP_DIR/${exchange}_stats.tmp"
            grep "^$exchange," "$temp_data" > "$exchange_data" 2>/dev/null || true
            
            if [[ -s "$exchange_data" ]]; then
                local ex_total=$(wc -l < "$exchange_data")
                local ex_success=$(grep -c "SUCCESS" "$exchange_data" || echo 0)
                local ex_timing=""
                
                if [[ $ex_success -gt 0 ]]; then
                    ex_timing=$(grep "SUCCESS" "$exchange_data" | cut -d',' -f3 | sort -n | awk '
                    BEGIN { sum=0; count=0; }
                    { if($1 > 0) { sum += $1; count++; } }
                    END { if(count > 0) printf "%.1f", sum/count; else print "0"; }')
                else
                    ex_timing="0"
                fi
                
                cat >> "$report_file" << EOF
#### 🏪 $(echo $exchange | tr '[:lower:]' '[:upper:]') 交易所

| 指标 | 数值 | 状态 |
|------|------|------|
| 总交易对 | $ex_total | $([ $ex_total -ge 45 ] && echo "✅" || echo "⚠️") |
| 成功处理 | $ex_success | $([ $ex_success -ge 40 ] && echo "✅" || echo "⚠️") |
| 成功率 | $(( ex_success * 100 / ex_total ))% | $([ $(( ex_success * 100 / ex_total )) -ge 90 ] && echo "✅" || echo "❌") |
| 平均时间 | ${ex_timing}μs | $(awk "BEGIN{print ($ex_timing < 300) ? \"✅\" : \"⚠️\"}") |

EOF
            else
                cat >> "$report_file" << EOF
#### 🏪 $(echo $exchange | tr '[:lower:]' '[:upper:]') 交易所

| 指标 | 数值 | 状态 |
|------|------|------|
| 状态 | 无数据 | ❌ |

EOF
            fi
        done
        
        # 最慢的10个交易对
        cat >> "$report_file" << EOF
### 🐌 最慢的10个交易对

| 排名 | 交易所 | 交易对 | 清洗时间 | 状态 |
|------|--------|--------|----------|------|
EOF
        
        grep "SUCCESS" "$temp_data" | sort -t',' -k3 -nr | head -10 | awk -F',' '
        {
            status = ($3 > 1000) ? "❌ 超慢" : ($3 > 500) ? "⚠️ 偏慢" : "✅ 正常";
            printf "| %d | %s | %s | %sμs | %s |\n", NR, $1, $2, $3, status;
        }' >> "$report_file"
        
        # 最快的10个交易对
        cat >> "$report_file" << EOF

### 🚀 最快的10个交易对

| 排名 | 交易所 | 交易对 | 清洗时间 | 状态 |
|------|--------|--------|----------|------|
EOF
        
        grep "SUCCESS" "$temp_data" | sort -t',' -k3 -n | head -10 | awk -F',' '
        {
            status = ($3 < 100) ? "🚀 极快" : ($3 < 200) ? "✅ 很快" : "👍 正常";
            printf "| %d | %s | %s | %sμs | %s |\n", NR, $1, $2, $3, status;
        }' >> "$report_file"
        
    else
        cat >> "$report_file" << EOF
### ❌ 数据分析失败

无法获取到有效的时间数据，可能原因：
- 系统日志格式变更
- 进程输出重定向
- 权限不足
- 系统未正常运行

EOF
    fi
    
    cat >> "$report_file" << EOF

## 💡 系统建议

EOF

    # 基于分析结果给出建议
    if [[ $successful_pairs -lt 200 ]]; then
        echo "### ⚠️ 数据获取问题" >> "$report_file"
        echo "- 成功处理的交易对数量不足，建议检查网络连接和交易所API状态" >> "$report_file"
        echo "- 检查配置文件中的交易对设置是否正确" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    if [[ $(awk "BEGIN{print ($avg_time > 300)}") -eq 1 ]]; then
        echo "### 🐌 性能优化建议" >> "$report_file"
        echo "- 平均清洗时间超过300μs，建议优化内存配置" >> "$report_file"
        echo "- 检查CPU亲和性设置和NUMA配置" >> "$report_file"
        echo "- 考虑增加清洗线程数量" >> "$report_file"
        echo "" >> "$report_file"
    fi
    
    cat >> "$report_file" << EOF

---

**生成时间**: $(date)
**NATS状态**: $(ps aux | grep nats-server | grep -v grep >/dev/null && echo "✅ 运行中" || echo "❌ 未运行")
**下次报告**: $(date -d "+$MONITOR_INTERVAL seconds")
**监控版本**: QingXi 5.1 精确时间分析 v1.0
EOF

    log "✅ 详细时间报告生成完成: $report_file"
    echo "$report_file"
}

# 主监控循环
main_timing_monitor() {
    log "🚀 启动QingXi 5.1 详细时间分析监控..."
    log "⏰ 监控间隔: ${MONITOR_INTERVAL}秒"
    log "📊 分析时长: ${ANALYSIS_DURATION}秒"
    
    local cycle=1
    
    while true; do
        log "📊 开始第 $cycle 轮时间分析..."
        
        local qingxi_pid=$(get_qingxi_pid)
        if [[ -z "$qingxi_pid" ]]; then
            log "❌ QingXi系统未运行，等待启动..."
            sleep 60
            continue
        fi
        
        log "✅ QingXi系统运行中 (PID: $qingxi_pid)"
        
        # 捕获日志
        local raw_log="$TEMP_DIR/raw_logs_$(date +%Y%m%d_%H%M%S).log"
        capture_json_logs "$qingxi_pid" "$raw_log" "$ANALYSIS_DURATION"
        
        # 分析时间数据
        local analysis_file="$TEMP_DIR/timing_analysis_$(date +%Y%m%d_%H%M%S).csv"
        analyze_all_trading_pairs "$raw_log" "$analysis_file"
        
        # 生成报告
        local report_file=$(generate_detailed_report "$analysis_file")
        
        if [[ -n "$report_file" ]]; then
            echo ""
            echo "=== QingXi 5.1 时间分析报告 ==="
            echo "报告时间: $(date)"
            echo "详细报告: $report_file"
            echo ""
        fi
        
        log "⏰ 下次分析时间: $(date -d "+$MONITOR_INTERVAL seconds" '+%H:%M:%S')"
        ((cycle++))
        sleep "$MONITOR_INTERVAL"
    done
}

# 信号处理
cleanup() {
    log "🛑 收到停止信号，清理资源..."
    jobs -p | xargs -r kill 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

# 启动监控
main_timing_monitor 