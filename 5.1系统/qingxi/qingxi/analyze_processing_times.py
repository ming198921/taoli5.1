#!/usr/bin/env python3
"""
QingXi 5.1 数据处理时间分析脚本
分析250个交易对从获取到清洗完毕的精确时间
"""

import json
import re
from datetime import datetime
from collections import defaultdict
import statistics

def parse_timestamp(ts_str):
    """解析ISO时间戳为微秒"""
    dt = datetime.fromisoformat(ts_str.replace('Z', '+00:00'))
    return int(dt.timestamp() * 1_000_000)

def extract_symbol_exchange(message):
    """从消息中提取交易对和交易所"""
    # 匹配 "Received OrderBook for SOL/USDT from huobi" 格式
    received_match = re.search(r'Received.*?for\s+(\S+)\s+from\s+(\w+)', message)
    if received_match:
        return received_match.group(1), received_match.group(2)
    
    # 匹配 "Data cleaning successful for huobi" 格式
    success_match = re.search(r'cleaning successful for\s+(\w+)', message)
    if success_match:
        return None, success_match.group(1)
    
    return None, None

def analyze_log_file(log_file_path):
    """分析日志文件，提取处理时间数据"""
    processing_times = {}  # {(exchange, symbol): {'received': timestamp, 'cleaned': timestamp}}
    pending_receives = {}  # {exchange: {'timestamp': ts, 'symbol': symbol}}
    
    print("📊 开始分析日志文件...")
    
    with open(log_file_path, 'r', encoding='utf-8') as f:
        line_count = 0
        processed_pairs = set()
        
        for line in f:
            line_count += 1
            if line_count % 100000 == 0:
                print(f"已处理 {line_count} 行...")
            
            try:
                log_entry = json.loads(line.strip())
                timestamp = parse_timestamp(log_entry['timestamp'])
                message = log_entry['fields']['message']
                
                symbol, exchange = extract_symbol_exchange(message)
                
                if 'Received OrderBook' in message and symbol and exchange:
                    # 记录数据接收时间
                    pending_receives[exchange] = {
                        'timestamp': timestamp,
                        'symbol': symbol
                    }
                
                elif 'cleaning successful' in message and exchange:
                    # 查找对应的接收记录
                    if exchange in pending_receives:
                        receive_data = pending_receives[exchange]
                        key = (exchange, receive_data['symbol'])
                        
                        if key not in processing_times:
                            processing_times[key] = {}
                        
                        processing_times[key]['received'] = receive_data['timestamp']
                        processing_times[key]['cleaned'] = timestamp
                        
                        # 计算处理时间
                        process_time = timestamp - receive_data['timestamp']
                        processing_times[key]['duration_us'] = process_time
                        
                        processed_pairs.add(key)
                        del pending_receives[exchange]
                        
            except (json.JSONDecodeError, KeyError):
                continue
    
    print(f"✅ 分析完成！处理了 {line_count} 行日志")
    print(f"🎯 找到 {len(processed_pairs)} 个完整的处理记录")
    
    return processing_times

def generate_report(processing_times):
    """生成详细的处理时间报告"""
    if not processing_times:
        print("❌ 没有找到完整的处理时间数据")
        return
    
    # 按交易所分组
    exchange_data = defaultdict(list)
    all_times = []
    
    for (exchange, symbol), data in processing_times.items():
        if 'duration_us' in data:
            duration_us = data['duration_us']
            exchange_data[exchange].append({
                'symbol': symbol,
                'duration_us': duration_us,
                'duration_ms': duration_us / 1000
            })
            all_times.append(duration_us)
    
    print("\n" + "="*80)
    print("🎯 QingXi 5.1 数据处理时间分析报告")
    print("="*80)
    
    # 总体统计
    if all_times:
        total_count = len(all_times)
        avg_us = statistics.mean(all_times)
        median_us = statistics.median(all_times)
        min_us = min(all_times)
        max_us = max(all_times)
        
        print(f"\n📊 总体统计 (共 {total_count} 个交易对):")
        print(f"   平均时间: {avg_us:.1f} 微秒 ({avg_us/1000:.3f} 毫秒)")
        print(f"   中位数: {median_us:.1f} 微秒 ({median_us/1000:.3f} 毫秒)")
        print(f"   最快时间: {min_us:.1f} 微秒 ({min_us/1000:.3f} 毫秒)")
        print(f"   最慢时间: {max_us:.1f} 微秒 ({max_us/1000:.3f} 毫秒)")
    
    # 按交易所统计
    print(f"\n📈 各交易所详细统计:")
    print("-"*80)
    
    for exchange in sorted(exchange_data.keys()):
        data_list = exchange_data[exchange]
        if not data_list:
            continue
            
        times = [d['duration_us'] for d in data_list]
        count = len(times)
        avg_us = statistics.mean(times)
        min_us = min(times)
        max_us = max(times)
        
        print(f"\n🏪 {exchange.upper()}:")
        print(f"   交易对数量: {count}/50")
        print(f"   平均时间: {avg_us:.1f} 微秒 ({avg_us/1000:.3f} 毫秒)")
        print(f"   最快时间: {min_us:.1f} 微秒 ({min_us/1000:.3f} 毫秒)")
        print(f"   最慢时间: {max_us:.1f} 微秒 ({max_us/1000:.3f} 毫秒)")
        
        # 显示前10个交易对的详细时间
        print(f"   前10个交易对详情:")
        for i, item in enumerate(sorted(data_list, key=lambda x: x['duration_us'])[:10]):
            print(f"     {i+1:2d}. {item['symbol']:12s} {item['duration_us']:6.1f} 微秒")
    
    # 生成CSV格式的详细表格
    print(f"\n📋 CSV格式数据 (可导入Excel):")
    print("-"*80)
    print("交易所,交易对,处理时间(微秒),处理时间(毫秒)")
    
    for exchange in sorted(exchange_data.keys()):
        for item in sorted(exchange_data[exchange], key=lambda x: x['duration_us']):
            print(f"{exchange},{item['symbol']},{item['duration_us']:.1f},{item['duration_ms']:.3f}")

if __name__ == "__main__":
    log_file = "/home/ubuntu/qingxi/qingxi/logs/qingxi_runtime.log"
    
    try:
        processing_times = analyze_log_file(log_file)
        generate_report(processing_times)
        
        print(f"\n✅ 分析完成！")
        print(f"📁 日志文件: {log_file}")
        print(f"🕒 分析时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        
    except FileNotFoundError:
        print(f"❌ 日志文件不存在: {log_file}")
    except Exception as e:
        print(f"❌ 分析出错: {e}") 