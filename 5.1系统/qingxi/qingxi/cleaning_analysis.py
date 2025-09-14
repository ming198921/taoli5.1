#!/usr/bin/env python3
import json
import re
from datetime import datetime
from collections import defaultdict
import statistics

def parse_timestamp(timestamp_str):
    """解析时间戳"""
    return datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))

def main():
    log_file = "qingxi_system_20250726_050553.log"
    
    # 存储数据 - 简化版本
    cleaning_times = []  # (start_time, end_time, exchange)
    
    print("开始快速分析清洗时间...")
    
    current_cleaning_start = {}  # exchange -> timestamp
    
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
                
                # 数据清洗开始记录
                if "🧹 Performing data cleaning" in message:
                    if "from bybit" in message:
                        exchange = "bybit"
                    elif "from huobi" in message:
                        exchange = "huobi"
                    else:
                        exchange = "unknown"
                    current_cleaning_start[exchange] = timestamp
                
                # 数据清洗成功记录
                elif "Data cleaning successful" in message:
                    if "for bybit" in message:
                        exchange = "bybit"
                    elif "for huobi" in message:
                        exchange = "huobi"
                    else:
                        exchange = "unknown"
                    
                    if exchange in current_cleaning_start:
                        start_time = current_cleaning_start[exchange]
                        duration = (timestamp - start_time).total_seconds()
                        if 0 <= duration <= 1.0:  # 合理的清洗时间范围
                            cleaning_times.append((start_time, timestamp, exchange, duration))
                        del current_cleaning_start[exchange]
                    
            except (json.JSONDecodeError, KeyError):
                continue
    
    print(f"总共处理了 {line_count} 行")
    print(f"找到 {len(cleaning_times)} 条有效清洗时间记录")
    
    # 分析清洗时间
    print("\n=== 数据清洗时间分析 ===")
    exchange_cleaning_times = defaultdict(list)
    for start_time, end_time, exchange, duration in cleaning_times:
        exchange_cleaning_times[exchange].append(duration)
    
    print("交易所\t\t平均清洗时间(毫秒)\t最小时间(毫秒)\t最大时间(毫秒)\t标准差(毫秒)\t记录数")
    print("-" * 85)
    for exchange, times in sorted(exchange_cleaning_times.items()):
        if times:
            # 转换为毫秒
            times_ms = [t * 1000 for t in times]
            avg_time = statistics.mean(times_ms)
            min_time = min(times_ms)
            max_time = max(times_ms)
            std_dev = statistics.stdev(times_ms) if len(times_ms) > 1 else 0
            count = len(times_ms)
            print(f"{exchange:<15}\t{avg_time:.2f}\t\t\t{min_time:.2f}\t\t\t{max_time:.2f}\t\t\t{std_dev:.2f}\t\t{count}")
    
    # 分析清洗时间波动
    print("\n=== 清洗时间波动分析 ===")
    for exchange, times in sorted(exchange_cleaning_times.items()):
        if len(times) > 10:
            times_ms = [t * 1000 for t in times]
            avg_time = statistics.mean(times_ms)
            std_dev = statistics.stdev(times_ms)
            cv = std_dev / avg_time if avg_time > 0 else 0
            
            # 找出异常值（超过2个标准差）
            outliers = [t for t in times_ms if abs(t - avg_time) > 2 * std_dev]
            
            print(f"\n{exchange} 交易所清洗时间分析:")
            print(f"  平均时间: {avg_time:.2f} 毫秒")
            print(f"  标准差: {std_dev:.2f} 毫秒")
            print(f"  变异系数: {cv:.4f} {'(波动较大)' if cv > 0.5 else '(波动正常)'}")
            print(f"  异常值数量: {len(outliers)} ({len(outliers)/len(times_ms)*100:.1f}%)")
            if outliers:
                print(f"  异常值范围: {min(outliers):.2f} - {max(outliers):.2f} 毫秒")
            
            # 分析时间分布
            sorted_times = sorted(times_ms)
            p50 = sorted_times[len(sorted_times)//2]
            p90 = sorted_times[int(len(sorted_times)*0.9)]
            p99 = sorted_times[int(len(sorted_times)*0.99)]
            print(f"  P50: {p50:.2f} 毫秒, P90: {p90:.2f} 毫秒, P99: {p99:.2f} 毫秒")

if __name__ == "__main__":
    main()
