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
    # Examples:
    # "📊 Received OrderBookSnapshot for SOL/USDT from bybit: 200 bids, 200 asks"
    # "✅ Data cleaning successful for bybit - validation passed"
    
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
    
    # Store data reception events by exchange and timestamp
    reception_events = []
    cleaning_events = []
    
    with open('qingxi_system_20250726_050553.log', 'r') as f:
        for line_num, line in enumerate(f, 1):
            try:
                data = json.loads(line.strip())
                timestamp = parse_timestamp(data['timestamp'])
                message = data['fields']['message']
                
                # Collect reception events
                if "📊 Received OrderBook" in message and " from " in message:
                    symbol, exchange = extract_symbol_exchange(message)
                    if symbol and exchange:
                        reception_events.append({
                            'timestamp': timestamp,
                            'symbol': symbol,
                            'exchange': exchange,
                            'line_num': line_num
                        })
                
                # Collect cleaning events
                elif "✅ Data cleaning successful for" in message:
                    symbol, exchange = extract_symbol_exchange(message)
                    if exchange:
                        cleaning_events.append({
                            'timestamp': timestamp,
                            'exchange': exchange,
                            'line_num': line_num
                        })
                
            except (json.JSONDecodeError, KeyError) as e:
                continue
    
    print(f"📥 找到 {len(reception_events)} 个数据接收事件")
    print(f"🧹 找到 {len(cleaning_events)} 个清洗完成事件")
    
    # Match reception with immediate cleaning events
    e2e_times = []
    
    for reception in reception_events:
        # Find the nearest cleaning event after this reception for the same exchange
        best_match = None
        min_time_diff = float('inf')
        
        for cleaning in cleaning_events:
            if (cleaning['exchange'] == reception['exchange'] and 
                cleaning['timestamp'] >= reception['timestamp']):
                
                time_diff = (cleaning['timestamp'] - reception['timestamp']).total_seconds()
                if time_diff < min_time_diff:
                    min_time_diff = time_diff
                    best_match = cleaning
        
        if best_match and min_time_diff <= 1.0:  # Within 1 second
            e2e_times.append({
                'symbol': reception['symbol'],
                'exchange': reception['exchange'],
                'reception_time': reception['timestamp'],
                'cleaning_time': best_match['timestamp'],
                'processing_time_ms': min_time_diff * 1000,
                'reception_line': reception['line_num'],
                'cleaning_line': best_match['line_num']
            })
    
    print(f"\n✅ 成功匹配 {len(e2e_times)} 个端到端处理记录")
    
    if not e2e_times:
        print("⚠️ 没有找到匹配的端到端记录")
        print("\n样本数据接收事件:")
        for i, event in enumerate(reception_events[:5]):
            print(f"  {i+1}. {event['timestamp']} - {event['symbol']}@{event['exchange']} (行 {event['line_num']})")
        
        print("\n样本清洗事件:")
        for i, event in enumerate(cleaning_events[:5]):
            print(f"  {i+1}. {event['timestamp']} - {event['exchange']} (行 {event['line_num']})")
        return
    
    # Group by symbol and exchange
    symbol_exchange_stats = defaultdict(list)
    for record in e2e_times:
        key = f"{record['symbol']}@{record['exchange']}"
        symbol_exchange_stats[key].append(record['processing_time_ms'])
    
    print("\n📊 每个币种每一条数据从交易所获取到清洗成功的时间统计表:")
    print("=" * 80)
    print(f"{'币种@交易所':<20} {'记录数':<8} {'平均时间(ms)':<12} {'最小值(ms)':<12} {'最大值(ms)':<12} {'标准差(ms)':<12}")
    print("-" * 80)
    
    for key in sorted(symbol_exchange_stats.keys()):
        times = symbol_exchange_stats[key]
        avg_time = sum(times) / len(times)
        min_time = min(times)
        max_time = max(times)
        
        # Calculate standard deviation
        variance = sum((t - avg_time) ** 2 for t in times) / len(times)
        std_dev = variance ** 0.5
        
        print(f"{key:<20} {len(times):<8} {avg_time:<12.2f} {min_time:<12.2f} {max_time:<12.2f} {std_dev:<12.2f}")
    
    # Show some sample records
    print(f"\n📝 样本端到端处理记录 (前10条):")
    print("=" * 100)
    print(f"{'币种@交易所':<20} {'接收时间':<26} {'清洗时间':<26} {'处理时间(ms)':<12}")
    print("-" * 100)
    
    for i, record in enumerate(e2e_times[:10]):
        key = f"{record['symbol']}@{record['exchange']}"
        reception_str = record['reception_time'].strftime('%H:%M:%S.%f')[:-3]
        cleaning_str = record['cleaning_time'].strftime('%H:%M:%S.%f')[:-3]
        
        print(f"{key:<20} {reception_str:<26} {cleaning_str:<26} {record['processing_time_ms']:<12.2f}")
    
    # Analysis of high processing times
    high_processing = [r for r in e2e_times if r['processing_time_ms'] > 10]
    if high_processing:
        print(f"\n⚠️  处理时间超过10ms的记录 ({len(high_processing)}条):")
        print("-" * 80)
        for record in high_processing[:20]:  # Show first 20
            key = f"{record['symbol']}@{record['exchange']}"
            print(f"  {key}: {record['processing_time_ms']:.2f}ms")
    
    # Save detailed results
    output_file = 'e2e_processing_times.json'
    with open(output_file, 'w') as f:
        # Convert datetime to string for JSON serialization
        serializable_data = []
        for record in e2e_times:
            serializable_record = record.copy()
            serializable_record['reception_time'] = record['reception_time'].isoformat()
            serializable_record['cleaning_time'] = record['cleaning_time'].isoformat()
            serializable_data.append(serializable_record)
        
        json.dump(serializable_data, f, indent=2)
    
    print(f"\n💾 详细结果已保存到 {output_file}")
    print(f"📈 总计分析了 {len(e2e_times)} 个端到端处理记录")

if __name__ == "__main__":
    main()
