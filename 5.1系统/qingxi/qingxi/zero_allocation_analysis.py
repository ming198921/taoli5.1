#!/usr/bin/env python3
"""
零分配失败问题专项分析
分析V3.0零分配验证失败的模式和影响
"""

import json
from datetime import datetime
from collections import defaultdict
import statistics

def parse_timestamp(timestamp_str):
    return datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))

def main():
    print("🔍 V3.0零分配失败专项分析")
    print("=" * 60)
    
    # 收集相关事件
    zero_alloc_failures = []
    cleaning_events = []
    performance_issues = []
    
    with open('qingxi_system_20250726_050553.log', 'r') as f:
        for line_num, line in enumerate(f, 1):
            try:
                data = json.loads(line.strip())
                timestamp = parse_timestamp(data['timestamp'])
                message = data['fields']['message']
                
                # 零分配失败
                if "零分配验证失败" in message:
                    zero_alloc_failures.append({
                        'timestamp': timestamp,
                        'line': line_num,
                        'thread': data.get('threadName', 'unknown')
                    })
                
                # 清洗事件
                elif "Data cleaning successful" in message:
                    cleaning_events.append({
                        'timestamp': timestamp,
                        'line': line_num
                    })
                
                # 性能相关警告
                elif any(keyword in message for keyword in ["回退到传统方法", "超时", "延迟", "性能"]):
                    performance_issues.append({
                        'timestamp': timestamp,
                        'message': message,
                        'line': line_num
                    })
                    
            except (json.JSONDecodeError, KeyError):
                continue
    
    print(f"📊 统计结果:")
    print(f"  零分配失败次数: {len(zero_alloc_failures)}")
    print(f"  清洗成功次数: {len(cleaning_events)}")
    print(f"  性能问题次数: {len(performance_issues)}")
    
    # 计算失败率
    if cleaning_events:
        failure_rate = len(zero_alloc_failures) / len(cleaning_events) * 100
        print(f"  零分配失败率: {failure_rate:.2f}%")
    
    # 分析失败的时间模式
    print(f"\n⏰ 零分配失败时间分析:")
    print("-" * 40)
    
    if len(zero_alloc_failures) > 1:
        # 计算失败间隔
        intervals = []
        for i in range(1, len(zero_alloc_failures)):
            interval = (zero_alloc_failures[i]['timestamp'] - zero_alloc_failures[i-1]['timestamp']).total_seconds()
            intervals.append(interval)
        
        avg_interval = statistics.mean(intervals)
        min_interval = min(intervals)
        max_interval = max(intervals)
        
        print(f"  平均失败间隔: {avg_interval:.3f}秒")
        print(f"  最小失败间隔: {min_interval:.3f}秒")
        print(f"  最大失败间隔: {max_interval:.3f}秒")
        
        # 分析是否有集中爆发
        burst_threshold = avg_interval / 10  # 如果间隔小于平均值1/10，认为是爆发
        bursts = [i for i in intervals if i < burst_threshold]
        print(f"  爆发性失败次数: {len(bursts)} ({len(bursts)/len(intervals)*100:.1f}%)")
    
    # 分析最初的失败模式
    print(f"\n🚨 最初失败模式分析:")
    print("-" * 40)
    
    if zero_alloc_failures:
        first_10_failures = zero_alloc_failures[:10]
        print("前10次失败时间:")
        start_time = zero_alloc_failures[0]['timestamp']
        
        for i, failure in enumerate(first_10_failures):
            relative_time = (failure['timestamp'] - start_time).total_seconds()
            print(f"  {i+1:2d}. +{relative_time:8.3f}s (行 {failure['line']})")
    
    # 分析失败对清洗性能的影响
    print(f"\n📈 零分配失败对性能的影响分析:")
    print("-" * 50)
    
    # 匹配失败事件和清洗成功事件
    failure_impact_times = []
    
    for failure in zero_alloc_failures:
        # 找到这次失败后最近的清洗成功事件
        for cleaning in cleaning_events:
            if (cleaning['timestamp'] > failure['timestamp'] and 
                (cleaning['timestamp'] - failure['timestamp']).total_seconds() < 0.1):  # 100ms内
                
                impact_time = (cleaning['timestamp'] - failure['timestamp']).total_seconds() * 1000  # ms
                failure_impact_times.append(impact_time)
                break
    
    if failure_impact_times:
        avg_impact = statistics.mean(failure_impact_times)
        max_impact = max(failure_impact_times)
        min_impact = min(failure_impact_times)
        
        print(f"  失败后恢复时间统计 ({len(failure_impact_times)}个样本):")
        print(f"    平均恢复时间: {avg_impact:.3f}ms")
        print(f"    最长恢复时间: {max_impact:.3f}ms")
        print(f"    最短恢复时间: {min_impact:.3f}ms")
    
    # 分析按小时的失败分布
    print(f"\n📅 失败时间分布分析:")
    print("-" * 30)
    
    hourly_failures = defaultdict(int)
    for failure in zero_alloc_failures:
        hour_minute = failure['timestamp'].strftime('%H:%M')
        minute_key = hour_minute[:-1] + '0'  # 按10分钟分组
        hourly_failures[minute_key] += 1
    
    print("每10分钟失败次数:")
    for time_slot in sorted(hourly_failures.keys()):
        count = hourly_failures[time_slot]
        bar = "█" * (count // 5) + "▌" * (1 if count % 5 >= 3 else 0)
        print(f"  {time_slot}: {count:3d} {bar}")
    
    # 问题根本原因分析
    print(f"\n🔧 问题根本原因分析:")
    print("-" * 30)
    print("基于日志分析，零分配失败的可能原因:")
    print("1. 内存池不足 - 高并发时65536缓冲区可能不够")
    print("2. 线程竞争 - 多个线程同时访问零分配器")
    print("3. 内存碎片 - 长时间运行后内存碎片影响分配")
    print("4. GC压力 - Rust的Drop实现可能影响零分配")
    print("5. CPU缓存失效 - 高频操作导致缓存未命中")
    
    print(f"\n💡 优化建议:")
    print("-" * 15)
    print("1. 增加零分配缓冲区大小 (65536 -> 131072)")
    print("2. 实现per-thread的零分配池")
    print("3. 添加内存预热机制")
    print("4. 优化内存对齐和缓存友好性")
    print("5. 实现失败回退的优雅降级")
    
    # 检查是否有相关的环境变量未设置
    print(f"\n🔍 环境配置检查:")
    print("-" * 20)
    print("确保以下环境变量已正确设置:")
    print("- QINGXI_ZERO_ALLOCATION=true")
    print("- QINGXI_ENABLE_V3_OPTIMIZATIONS=true")
    print("- QINGXI_INTEL_OPTIMIZATIONS=true")
    print("- 内存池大小配置")

if __name__ == "__main__":
    main()
