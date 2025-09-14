#!/usr/bin/env python3
import json
import re
from datetime import datetime
from collections import defaultdict, Counter
import statistics

def parse_timestamp(timestamp_str):
    """解析时间戳"""
    return datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))

def extract_symbol_and_exchange(message):
    """从消息中提取币种和交易所"""
    # 提取币种，例如 "SOL/USDT from bybit" -> ("SOL/USDT", "bybit")
    pattern = r'for ([A-Z]+/[A-Z]+) from (\w+)'
    match = re.search(pattern, message)
    if match:
        return match.group(1), match.group(2)
    return None, None

def main():
    log_file = "qingxi_system_20250726_050553.log"
    
    # 存储数据
    receive_data = []  # (timestamp, symbol, exchange)
    cleaning_start = []  # (timestamp, symbol, exchange)
    cleaning_success = []  # (timestamp, symbol, exchange)
    
    print("开始分析日志文件...")
    
    with open(log_file, 'r', encoding='utf-8') as f:
        line_count = 0
        for line in f:
            line_count += 1
            if line_count % 100000 == 0:
                print(f"已处理 {line_count} 行...")
            
            # 跳过非JSON行
            if not line.strip().startswith('{'):
                continue
                
            try:
                log_entry = json.loads(line.strip())
                timestamp = parse_timestamp(log_entry['timestamp'])
                message = log_entry['fields']['message']
                
                # 数据接收记录
                if "📊 Received" in message:
                    symbol, exchange = extract_symbol_and_exchange(message)
                    if symbol and exchange:
                        receive_data.append((timestamp, symbol, exchange))
                
                # 数据清洗开始记录
                elif "🧹 Performing data cleaning" in message:
                    # 从前一条记录推断symbol和exchange
                    if "from bybit" in message:
                        exchange = "bybit"
                    elif "from huobi" in message:
                        exchange = "huobi"
                    else:
                        exchange = "unknown"
                    cleaning_start.append((timestamp, None, exchange))
                
                # 数据清洗成功记录
                elif "Data cleaning successful" in message:
                    if "for bybit" in message:
                        exchange = "bybit"
                    elif "for huobi" in message:
                        exchange = "huobi"
                    else:
                        exchange = "unknown"
                    cleaning_success.append((timestamp, None, exchange))
                    
            except (json.JSONDecodeError, KeyError):
                continue
    
    print(f"总共处理了 {line_count} 行")
    print(f"找到 {len(receive_data)} 条数据接收记录")
    print(f"找到 {len(cleaning_start)} 条清洗开始记录")
    print(f"找到 {len(cleaning_success)} 条清洗成功记录")
    
    # 分析1：每个币种从交易所获取数据的时间间隔
    print("\n=== 分析1：每个币种数据获取时间间隔 ===")
    symbol_intervals = defaultdict(list)
    exchange_symbols = defaultdict(list)
    
    # 按币种和交易所分组
    for timestamp, symbol, exchange in receive_data:
        key = f"{symbol}@{exchange}"
        exchange_symbols[key].append(timestamp)
    
    # 计算时间间隔
    for key, timestamps in exchange_symbols.items():
        if len(timestamps) > 1:
            timestamps.sort()
            intervals = []
            for i in range(1, len(timestamps)):
                interval = (timestamps[i] - timestamps[i-1]).total_seconds()
                intervals.append(interval)
            symbol_intervals[key] = intervals
    
    # 输出表格1
    print("币种@交易所\t\t平均间隔(秒)\t最小间隔(秒)\t最大间隔(秒)\t标准差\t\t记录数")
    print("-" * 80)
    for key, intervals in sorted(symbol_intervals.items()):
        if intervals:
            avg_interval = statistics.mean(intervals)
            min_interval = min(intervals)
            max_interval = max(intervals)
            std_dev = statistics.stdev(intervals) if len(intervals) > 1 else 0
            count = len(intervals) + 1  # +1 because intervals is one less than data points
            print(f"{key:<20}\t{avg_interval:.3f}\t\t{min_interval:.3f}\t\t{max_interval:.3f}\t\t{std_dev:.3f}\t\t{count}")
    
    # 分析2：清洗时间分析
    print("\n=== 分析2：数据清洗时间分析 ===")
    cleaning_times = []
    
    # 匹配清洗开始和成功的记录
    start_idx = 0
    for success_time, _, success_exchange in cleaning_success:
        # 找到最近的清洗开始记录
        best_match = None
        best_diff = float('inf')
        
        for i in range(start_idx, len(cleaning_start)):
            start_time, _, start_exchange = cleaning_start[i]
            if start_exchange == success_exchange and start_time <= success_time:
                diff = (success_time - start_time).total_seconds()
                if diff < best_diff and diff >= 0:
                    best_match = (start_time, success_time, success_exchange, diff)
                    best_diff = diff
        
        if best_match:
            cleaning_times.append(best_match)
    
    # 按交易所分组清洗时间
    exchange_cleaning_times = defaultdict(list)
    for start_time, end_time, exchange, duration in cleaning_times:
        exchange_cleaning_times[exchange].append(duration)
    
    print("交易所\t\t平均清洗时间(秒)\t最小时间(秒)\t最大时间(秒)\t标准差\t\t记录数")
    print("-" * 75)
    for exchange, times in sorted(exchange_cleaning_times.items()):
        if times:
            avg_time = statistics.mean(times)
            min_time = min(times)
            max_time = max(times)
            std_dev = statistics.stdev(times) if len(times) > 1 else 0
            count = len(times)
            print(f"{exchange:<15}\t{avg_time:.6f}\t\t{min_time:.6f}\t\t{max_time:.6f}\t\t{std_dev:.6f}\t\t{count}")
    
    # 分析3：清洗时间波动分析
    print("\n=== 分析3：清洗时间波动分析 ===")
    for exchange, times in sorted(exchange_cleaning_times.items()):
        if len(times) > 10:  # 至少要有10个样本
            avg_time = statistics.mean(times)
            std_dev = statistics.stdev(times)
            cv = std_dev / avg_time if avg_time > 0 else 0  # 变异系数
            
            # 找出异常值（超过2个标准差）
            outliers = [t for t in times if abs(t - avg_time) > 2 * std_dev]
            
            print(f"\n{exchange} 交易所清洗时间波动分析:")
            print(f"  变异系数: {cv:.4f} {'(波动较大)' if cv > 0.3 else '(波动正常)'}")
            print(f"  异常值数量: {len(outliers)}")
            if outliers:
                print(f"  异常值范围: {min(outliers):.6f}s - {max(outliers):.6f}s")
    
    # 分析4：端到端时间分析
    print("\n=== 分析4：端到端时间分析 ===")
    end_to_end_times = []
    
    # 匹配接收数据到清洗成功的完整流程
    for receive_time, symbol, receive_exchange in receive_data[:1000]:  # 限制分析前1000条
        # 找到对应的清洗成功记录
        for success_time, _, success_exchange in cleaning_success:
            if (receive_exchange == success_exchange and 
                success_time > receive_time and 
                (success_time - receive_time).total_seconds() < 1.0):  # 1秒内完成
                
                duration = (success_time - receive_time).total_seconds()
                end_to_end_times.append((receive_time, success_time, symbol, receive_exchange, duration))
                break
    
    # 按币种分组端到端时间
    symbol_e2e_times = defaultdict(list)
    for _, _, symbol, exchange, duration in end_to_end_times:
        key = f"{symbol}@{exchange}"
        symbol_e2e_times[key].append(duration)
    
    print("币种@交易所\t\t平均端到端时间(秒)\t最小时间(秒)\t最大时间(秒)\t记录数")
    print("-" * 75)
    for key, times in sorted(symbol_e2e_times.items()):
        if times:
            avg_time = statistics.mean(times)
            min_time = min(times)
            max_time = max(times)
            count = len(times)
            print(f"{key:<20}\t{avg_time:.6f}\t\t{min_time:.6f}\t\t{max_time:.6f}\t\t{count}")

if __name__ == "__main__":
    main()
