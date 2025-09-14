#!/usr/bin/env python3
import json
import re
from datetime import datetime
from collections import defaultdict
import statistics

def parse_timestamp(timestamp_str):
    """解析时间戳"""
    return datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))

def extract_symbol_and_exchange(message):
    """从消息中提取币种和交易所"""
    pattern = r'for ([A-Z]+/[A-Z]+) from (\w+)'
    match = re.search(pattern, message)
    if match:
        return match.group(1), match.group(2)
    return None, None

def main():
    log_file = "qingxi_system_20250726_050553.log"
    
    print("开始分析端到端处理时间...")
    
    # 存储最近的数据接收记录
    recent_receives = defaultdict(list)  # symbol@exchange -> [(timestamp, symbol, exchange)]
    e2e_times = []  # (symbol, exchange, duration)
    
    with open(log_file, 'r', encoding='utf-8') as f:
        line_count = 0
        for line in f:
            line_count += 1
            if line_count % 200000 == 0:
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
                        key = f"{symbol}@{exchange}"
                        recent_receives[key].append((timestamp, symbol, exchange))
                        # 只保留最近50条记录以节省内存
                        if len(recent_receives[key]) > 50:
                            recent_receives[key] = recent_receives[key][-50:]
                
                # 查找"Initializing local book from snapshot"记录（表示处理完成）
                elif "Initializing local book from snapshot" in message:
                    # 提取source和symbol
                    source_match = re.search(r'"source":"(\w+)"', message)
                    symbol_match = re.search(r'"symbol":"([A-Z]+/[A-Z]+)"', message)
                    
                    if source_match and symbol_match:
                        exchange = source_match.group(1)
                        symbol = symbol_match.group(1)
                        key = f"{symbol}@{exchange}"
                        
                        # 查找最近的接收记录
                        if key in recent_receives:
                            for receive_time, recv_symbol, recv_exchange in reversed(recent_receives[key]):
                                if (timestamp > receive_time and 
                                    (timestamp - receive_time).total_seconds() < 10.0):  # 10秒内完成
                                    duration = (timestamp - receive_time).total_seconds()
                                    e2e_times.append((symbol, exchange, duration))
                                    break
                    
            except (json.JSONDecodeError, KeyError):
                continue
    
    print(f"总共处理了 {line_count} 行")
    print(f"找到 {len(e2e_times)} 条端到端时间记录")
    
    # 分析端到端时间
    print("\n=== 端到端处理时间分析 ===")
    symbol_e2e_times = defaultdict(list)
    exchange_e2e_times = defaultdict(list)
    
    for symbol, exchange, duration in e2e_times:
        key = f"{symbol}@{exchange}"
        symbol_e2e_times[key].append(duration)
        exchange_e2e_times[exchange].append(duration)
    
    # 按币种统计
    print("币种@交易所\t\t平均处理时间(毫秒)\t最小时间(毫秒)\t最大时间(毫秒)\t标准差(毫秒)\t记录数")
    print("-" * 95)
    for key, times in sorted(symbol_e2e_times.items()):
        if times and len(times) >= 5:  # 至少5个样本
            times_ms = [t * 1000 for t in times]
            avg_time = statistics.mean(times_ms)
            min_time = min(times_ms)
            max_time = max(times_ms)
            std_dev = statistics.stdev(times_ms) if len(times_ms) > 1 else 0
            count = len(times_ms)
            print(f"{key:<20}\t{avg_time:.2f}\t\t\t{min_time:.2f}\t\t\t{max_time:.2f}\t\t\t{std_dev:.2f}\t\t{count}")
    
    # 按交易所统计
    print("\n=== 按交易所统计端到端时间 ===")
    print("交易所\t\t平均处理时间(毫秒)\t最小时间(毫秒)\t最大时间(毫秒)\t标准差(毫秒)\t记录数")
    print("-" * 85)
    for exchange, times in sorted(exchange_e2e_times.items()):
        if times:
            times_ms = [t * 1000 for t in times]
            avg_time = statistics.mean(times_ms)
            min_time = min(times_ms)
            max_time = max(times_ms)
            std_dev = statistics.stdev(times_ms) if len(times_ms) > 1 else 0
            count = len(times_ms)
            print(f"{exchange:<15}\t{avg_time:.2f}\t\t\t{min_time:.2f}\t\t\t{max_time:.2f}\t\t\t{std_dev:.2f}\t\t{count}")
    
    # 性能问题分析
    print("\n=== 性能问题分析 ===")
    for exchange, times in sorted(exchange_e2e_times.items()):
        if len(times) > 10:
            times_ms = [t * 1000 for t in times]
            avg_time = statistics.mean(times_ms)
            
            # 找出慢请求（超过平均时间2倍）
            slow_requests = [t for t in times_ms if t > avg_time * 2]
            very_slow_requests = [t for t in times_ms if t > 1000]  # 超过1秒的请求
            
            print(f"\n{exchange} 交易所性能分析:")
            print(f"  平均处理时间: {avg_time:.2f} 毫秒")
            print(f"  慢请求数量: {len(slow_requests)} ({len(slow_requests)/len(times_ms)*100:.1f}%)")
            if slow_requests:
                print(f"  慢请求时间范围: {min(slow_requests):.2f} - {max(slow_requests):.2f} 毫秒")
            print(f"  超慢请求(>1秒): {len(very_slow_requests)} ({len(very_slow_requests)/len(times_ms)*100:.1f}%)")
            
            # 计算百分位数
            sorted_times = sorted(times_ms)
            p95 = sorted_times[int(len(sorted_times)*0.95)]
            p99 = sorted_times[int(len(sorted_times)*0.99)]
            print(f"  P95: {p95:.2f} 毫秒, P99: {p99:.2f} 毫秒")

if __name__ == "__main__":
    main()
