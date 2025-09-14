#!/usr/bin/env python3
import json
from collections import defaultdict
from datetime import datetime
import statistics

def parse_timestamp(timestamp_str):
    return datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))

def analyze_cleaning_stability():
    print("🔍 分析每个币种清洗数据的平稳性...")
    
    # Group cleaning times by symbol and exchange
    symbol_cleaning_times = defaultdict(list)
    symbol_timestamps = defaultdict(list)
    
    print("📖 读取日志文件...")
    with open('qingxi_system_20250726_050553.log', 'r') as f:
        for line_num, line in enumerate(f, 1):
            if line_num % 100000 == 0:
                print(f"  处理了 {line_num} 行...")
            
            try:
                data = json.loads(line.strip())
                message = data['fields']['message']
                
                if "🧹 Performing data cleaning" in message:
                    # Get the timestamp for cleaning start
                    timestamp = parse_timestamp(data['timestamp'])
                    
                    # Try to find the exchange and symbol information from context
                    # Look for recent "Received OrderBook" messages
                    continue
                    
                elif "✅ Data cleaning successful for" in message:
                    timestamp = parse_timestamp(data['timestamp'])
                    
                    # Extract exchange
                    if "successful for bybit" in message:
                        exchange = "bybit"
                    elif "successful for okx" in message:
                        exchange = "okx"
                    elif "successful for huobi" in message:
                        exchange = "huobi"
                    else:
                        continue
                    
                    # For now, we'll analyze by exchange since we don't have symbol info in cleaning messages
                    symbol_cleaning_times[exchange].append(timestamp)
                    
            except (json.JSONDecodeError, KeyError) as e:
                continue
    
    print(f"\n找到清洗事件数据按交易所分组:")
    for exchange, times in symbol_cleaning_times.items():
        print(f"  {exchange}: {len(times)} 条记录")
    
    # Analyze time intervals between cleaning operations
    print("\n📊 清洗数据稳定性分析结果:")
    print("=" * 100)
    print(f"{'交易所':<10} {'总记录数':<10} {'平均间隔(ms)':<15} {'标准差(ms)':<15} {'变异系数':<12} {'稳定性评估':<12}")
    print("-" * 100)
    
    stability_results = []
    
    for exchange in sorted(symbol_cleaning_times.keys()):
        timestamps = sorted(symbol_cleaning_times[exchange])
        
        if len(timestamps) < 2:
            continue
            
        # Calculate intervals between consecutive cleaning operations
        intervals = []
        for i in range(1, len(timestamps)):
            interval = (timestamps[i] - timestamps[i-1]).total_seconds() * 1000  # ms
            intervals.append(interval)
        
        if not intervals:
            continue
            
        # Statistical analysis
        avg_interval = statistics.mean(intervals)
        std_dev = statistics.stdev(intervals) if len(intervals) > 1 else 0
        cv = std_dev / avg_interval if avg_interval > 0 else 0  # Coefficient of variation
        
        # Stability assessment
        if cv < 0.1:
            stability = "非常稳定"
        elif cv < 0.3:
            stability = "稳定"
        elif cv < 0.6:
            stability = "中等波动"
        elif cv < 1.0:
            stability = "波动较大"
        else:
            stability = "极不稳定"
        
        print(f"{exchange:<10} {len(timestamps):<10} {avg_interval:<15.2f} {std_dev:<15.2f} {cv:<12.4f} {stability:<12}")
        
        stability_results.append({
            'exchange': exchange,
            'count': len(timestamps),
            'avg_interval': avg_interval,
            'std_dev': std_dev,
            'cv': cv,
            'stability': stability,
            'intervals': intervals
        })
    
    # Find exchanges with high volatility
    high_volatility = [r for r in stability_results if r['cv'] > 0.5]
    if high_volatility:
        print(f"\n⚠️  波动较大的交易所详细分析:")
        print("=" * 80)
        
        for result in high_volatility:
            print(f"\n🔍 {result['exchange']} 交易所详细分析:")
            print(f"  📊 总记录数: {result['count']}")
            print(f"  ⏱️  平均清洗间隔: {result['avg_interval']:.2f}ms")
            print(f"  📈 标准差: {result['std_dev']:.2f}ms")
            print(f"  📊 变异系数: {result['cv']:.4f}")
            print(f"  🎯 稳定性评估: {result['stability']}")
            
            intervals = result['intervals']
            min_interval = min(intervals)
            max_interval = max(intervals)
            median_interval = statistics.median(intervals)
            
            print(f"  📏 最小间隔: {min_interval:.2f}ms")
            print(f"  📏 最大间隔: {max_interval:.2f}ms")
            print(f"  📏 中位数间隔: {median_interval:.2f}ms")
            print(f"  📏 间隔范围: {max_interval - min_interval:.2f}ms")
            
            # Find outliers (intervals > 3 standard deviations from mean)
            outliers = [i for i in intervals if abs(i - result['avg_interval']) > 3 * result['std_dev']]
            if outliers:
                print(f"  ⚠️  异常间隔数量: {len(outliers)}")
                print(f"  🚨 异常间隔范围: {min(outliers):.2f}ms - {max(outliers):.2f}ms")
    
    # Time series analysis for the most volatile exchange
    if stability_results:
        most_volatile = max(stability_results, key=lambda x: x['cv'])
        print(f"\n📈 最不稳定交易所 ({most_volatile['exchange']}) 时间序列分析:")
        print("=" * 80)
        
        timestamps = sorted(symbol_cleaning_times[most_volatile['exchange']])
        intervals = most_volatile['intervals']
        
        # Show first 20 intervals
        print("前20个清洗间隔时间 (ms):")
        for i, interval in enumerate(intervals[:20]):
            print(f"  {i+1:2d}. {interval:8.2f}ms")
        
        # Group by time periods to see if there are patterns
        print(f"\n📊 按时间段分析 (每1分钟):")
        minute_intervals = defaultdict(list)
        
        for i, timestamp in enumerate(timestamps[1:], 1):
            minute_key = timestamp.replace(second=0, microsecond=0)
            minute_intervals[minute_key].append(intervals[i-1])
        
        print(f"{'时间段':<20} {'记录数':<8} {'平均间隔(ms)':<15} {'标准差(ms)':<15}")
        print("-" * 65)
        
        for minute in sorted(minute_intervals.keys())[:10]:  # Show first 10 minutes
            minute_data = minute_intervals[minute]
            avg = statistics.mean(minute_data)
            std = statistics.stdev(minute_data) if len(minute_data) > 1 else 0
            
            time_str = minute.strftime('%H:%M:%S')
            print(f"{time_str:<20} {len(minute_data):<8} {avg:<15.2f} {std:<15.2f}")

if __name__ == "__main__":
    analyze_cleaning_stability()
