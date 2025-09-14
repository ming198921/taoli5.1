#!/usr/bin/env python3
import json
import re
from collections import defaultdict
from datetime import datetime

def parse_timestamp(timestamp_str):
    # Parse timestamp: 2025-07-26T05:06:03.835445Z
    return datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))

def extract_symbol_exchange(message):
    # Extract symbol and exchange from message
    if "Received OrderBook" in message and " from " in message:
        # Extract symbol and exchange from received message
        match = re.search(r'for ([A-Z]+/[A-Z]+) from (\w+)', message)
        if match:
            return match.group(1), match.group(2)  # symbol, exchange
    
    if "Data cleaning successful for" in message:
        # Extract exchange from cleaning message
        match = re.search(r'successful for (\w+)', message)
        if match:
            return None, match.group(1)  # None for symbol, exchange
    
    return None, None

def main():
    print("🔍 开始端到端处理时间分析...")
    
    # Group events by exchange for faster matching
    reception_by_exchange = defaultdict(list)
    cleaning_by_exchange = defaultdict(list)
    
    print("📖 读取日志文件...")
    with open('qingxi_system_20250726_050553.log', 'r') as f:
        for line_num, line in enumerate(f, 1):
            if line_num % 100000 == 0:
                print(f"  处理了 {line_num} 行...")
            
            try:
                data = json.loads(line.strip())
                timestamp = parse_timestamp(data['timestamp'])
                message = data['fields']['message']
                
                # Collect reception events
                if "📊 Received OrderBook" in message and " from " in message:
                    symbol, exchange = extract_symbol_exchange(message)
                    if symbol and exchange:
                        reception_by_exchange[exchange].append({
                            'timestamp': timestamp,
                            'symbol': symbol,
                            'line_num': line_num
                        })
                
                # Collect cleaning events
                elif "✅ Data cleaning successful for" in message:
                    symbol, exchange = extract_symbol_exchange(message)
                    if exchange:
                        cleaning_by_exchange[exchange].append({
                            'timestamp': timestamp,
                            'line_num': line_num
                        })
                
            except (json.JSONDecodeError, KeyError) as e:
                continue
    
    total_receptions = sum(len(events) for events in reception_by_exchange.values())
    total_cleanings = sum(len(events) for events in cleaning_by_exchange.values())
    
    print(f"📥 找到 {total_receptions} 个数据接收事件")
    print(f"🧹 找到 {total_cleanings} 个清洗完成事件")
    
    # Match reception with cleaning events efficiently
    e2e_times = []
    
    for exchange in reception_by_exchange:
        if exchange not in cleaning_by_exchange:
            continue
            
        receptions = reception_by_exchange[exchange]
        cleanings = cleaning_by_exchange[exchange]
        
        print(f"🔄 处理交易所 {exchange}: {len(receptions)} 接收, {len(cleanings)} 清洗")
        
        # Sort both lists by timestamp for efficient matching
        receptions.sort(key=lambda x: x['timestamp'])
        cleanings.sort(key=lambda x: x['timestamp'])
        
        cleaning_idx = 0
        
        for reception in receptions:
            # Find the first cleaning event after this reception
            while (cleaning_idx < len(cleanings) and 
                   cleanings[cleaning_idx]['timestamp'] < reception['timestamp']):
                cleaning_idx += 1
            
            if cleaning_idx < len(cleanings):
                cleaning = cleanings[cleaning_idx]
                time_diff = (cleaning['timestamp'] - reception['timestamp']).total_seconds()
                
                if time_diff <= 1.0:  # Within 1 second
                    e2e_times.append({
                        'symbol': reception['symbol'],
                        'exchange': exchange,
                        'reception_time': reception['timestamp'],
                        'cleaning_time': cleaning['timestamp'],
                        'processing_time_ms': time_diff * 1000,
                        'reception_line': reception['line_num'],
                        'cleaning_line': cleaning['line_num']
                    })
    
    print(f"\n✅ 成功匹配 {len(e2e_times)} 个端到端处理记录")
    
    if not e2e_times:
        print("⚠️ 没有找到匹配的端到端记录")
        return
    
    # Group by symbol and exchange
    symbol_exchange_stats = defaultdict(list)
    for record in e2e_times:
        key = f"{record['symbol']}@{record['exchange']}"
        symbol_exchange_stats[key].append(record['processing_time_ms'])
    
    print("\n📊 每个币种每一条数据从交易所获取到清洗成功的时间统计表:")
    print("=" * 90)
    print(f"{'币种@交易所':<20} {'记录数':<8} {'平均时间(ms)':<12} {'最小值(ms)':<12} {'最大值(ms)':<12} {'标准差(ms)':<12}")
    print("-" * 90)
    
    for key in sorted(symbol_exchange_stats.keys()):
        times = symbol_exchange_stats[key]
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        
        # Calculate standard deviation
        variance = sum((t - avg_time) ** 2 for t in times) / len(times)
        std_dev = variance ** 0.5
        
        print(f"{key:<20} {len(times):<8} {avg_time:<12.3f} {min_time:<12.3f} {max_time:<12.3f} {std_dev:<12.3f}")
    
    # Show some sample records
    print(f"\n📝 样本端到端处理记录 (前15条):")
    print("=" * 100)
    print(f"{'币种@交易所':<20} {'接收时间':<26} {'清洗时间':<26} {'处理时间(ms)':<12}")
    print("-" * 100)
    
    for i, record in enumerate(e2e_times[:15]):
        key = f"{record['symbol']}@{record['exchange']}"
        reception_str = record['reception_time'].strftime('%H:%M:%S.%f')[:-3]
        cleaning_str = record['cleaning_time'].strftime('%H:%M:%S.%f')[:-3]
        
        print(f"{key:<20} {reception_str:<26} {cleaning_str:<26} {record['processing_time_ms']:<12.3f}")
    
    # Analysis by exchange
    exchange_stats = defaultdict(list)
    for record in e2e_times:
        exchange_stats[record['exchange']].append(record['processing_time_ms'])
    
    print(f"\n📈 各交易所端到端处理性能对比:")
    print("=" * 70)
    print(f"{'交易所':<10} {'记录数':<8} {'平均时间(ms)':<12} {'最小值(ms)':<12} {'最大值(ms)':<12}")
    print("-" * 70)
    
    for exchange in sorted(exchange_stats.keys()):
        times = exchange_stats[exchange]
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        
        print(f"{exchange:<10} {len(times):<8} {avg_time:<12.3f} {min_time:<12.3f} {max_time:<12.3f}")
    
    # Analysis of high processing times
    high_processing = [r for r in e2e_times if r['processing_time_ms'] > 5]
    if high_processing:
        print(f"\n⚠️  处理时间超过5ms的记录 ({len(high_processing)}条):")
        print("-" * 80)
        for record in high_processing[:15]:  # Show first 15
            key = f"{record['symbol']}@{record['exchange']}"
            print(f"  {key}: {record['processing_time_ms']:.3f}ms")
    
    print(f"\n📈 总计分析了 {len(e2e_times)} 个端到端处理记录")
    print(f"📊 覆盖了 {len(symbol_exchange_stats)} 个币种-交易所组合")

if __name__ == "__main__":
    main()
