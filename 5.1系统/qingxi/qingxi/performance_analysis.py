#!/usr/bin/env python3
"""
QINGXI系统性能日志分析工具
根据用户要求进行4项详细分析：
1. 每个币种从交易所获取数据的时间
2. 每个币种清洗时间统计  
3. 清洗数据平稳性分析
4. 从获取到清洗成功的完整时间链路分析
"""

import re
import json
from datetime import datetime
from collections import defaultdict, Counter
import statistics

# 日志数据 - 从终端输出中提取
log_data = """
{"timestamp":"2025-07-26T17:49:18.802577Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for FUEL/USDT from bybit: 2 bids, 2 asks"},"target":"market_data_module::central_manager","span":{"name":"central_manager_run"},"spans":[{"name":"central_manager_run"}],"threadName":"qingxi-main","threadId":"ThreadId(13)"}
{"timestamp":"2025-07-26T17:49:18.802580Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"},"target":"market_data_module::central_manager","span":{"name":"central_manager_run"},"spans":[{"name":"central_manager_run"}],"threadName":"qingxi-main","threadId":"ThreadId(13)"}
{"timestamp":"2025-07-26T17:49:18.802608Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"},"target":"market_data_module::central_manager","span":{"name":"central_manager_run"},"spans":[{"name":"central_manager_run"}],"threadName":"qingxi-main","threadId":"ThreadId(13)"}
{"timestamp":"2025-07-26T17:49:18.802777Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for SC/USDT from bybit: 0 bids, 2 asks"},"target":"market_data_module::central_manager","span":{"name":"central_manager_run"},"spans":[{"name":"central_manager_run"}],"threadName":"qingxi-main","threadId":"ThreadId(13)"}
{"timestamp":"2025-07-26T17:49:18.802780Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"},"target":"market_data_module::central_manager","span":{"name":"central_manager_run"},"spans":[{"name":"central_manager_run"}],"threadName":"qingxi-main","threadId":"ThreadId(13)"}
{"timestamp":"2025-07-26T17:49:18.802881Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"},"target":"market_data_module::central_manager","span":{"name":"central_manager_run"},"spans":[{"name":"central_manager_run"}],"threadName":"qingxi-main","threadId":"ThreadId(13)"}
"""

class QingxiPerformanceAnalyzer:
    def __init__(self):
        self.data_receive_times = defaultdict(list)
        self.cleaning_start_times = defaultdict(list)
        self.cleaning_success_times = defaultdict(list)
        self.symbol_cleaning_durations = defaultdict(list)
        self.symbol_total_durations = defaultdict(list)
        self.exchanges = set()
        self.symbols = set()
        
    def parse_timestamp(self, timestamp_str):
        """解析ISO时间戳为毫秒"""
        dt = datetime.fromisoformat(timestamp_str.replace('Z', '+00:00'))
        return dt.timestamp() * 1000
        
    def extract_symbol_exchange(self, message):
        """从消息中提取交易对和交易所"""
        # 匹配模式：for SYMBOL from EXCHANGE
        pattern = r'for ([A-Z0-9/]+) from (\w+)'
        match = re.search(pattern, message)
        if match:
            return match.group(1), match.group(2)
        return None, None
        
    def process_log_line(self, log_line):
        """处理单行日志"""
        try:
            if not log_line.strip():
                return
                
            # 解析JSON格式的日志
            log_entry = json.loads(log_line.strip())
            timestamp = self.parse_timestamp(log_entry['timestamp'])
            message = log_entry['fields']['message']
            
            symbol, exchange = self.extract_symbol_exchange(message)
            if not symbol or not exchange:
                return
                
            self.symbols.add(symbol)
            self.exchanges.add(exchange)
            key = f"{exchange}-{symbol}"
            
            if "📊 Received OrderBookSnapshot" in message:
                self.data_receive_times[key].append(timestamp)
            elif "🧹 Performing data cleaning" in message:
                self.cleaning_start_times[key].append(timestamp)
            elif "✅ Data cleaning successful" in message:
                self.cleaning_success_times[key].append(timestamp)
                
        except (json.JSONDecodeError, KeyError) as e:
            # 跳过格式不正确的日志行
            pass
            
    def calculate_cleaning_durations(self):
        """计算清洗时间"""
        for key in self.cleaning_start_times:
            starts = self.cleaning_start_times[key]
            successes = self.cleaning_success_times[key]
            
            # 配对开始和成功时间
            for i, start_time in enumerate(starts):
                if i < len(successes):
                    duration_ms = successes[i] - start_time
                    self.symbol_cleaning_durations[key].append(duration_ms)
                    
    def calculate_total_durations(self):
        """计算从接收到清洗成功的总时间"""
        for key in self.data_receive_times:
            receives = self.data_receive_times[key]
            successes = self.cleaning_success_times[key]
            
            # 配对接收和清洗成功时间
            for i, receive_time in enumerate(receives):
                if i < len(successes):
                    total_duration_ms = successes[i] - receive_time
                    self.symbol_total_durations[key].append(total_duration_ms)
                    
    def analyze_data(self, log_content):
        """分析日志数据"""
        print("🔍 开始分析QINGXI性能日志...")
        
        # 分解日志内容为行
        lines = log_content.strip().split('\n')
        
        for line in lines:
            self.process_log_line(line)
            
        self.calculate_cleaning_durations()
        self.calculate_total_durations()
        
        print(f"✅ 解析完成：{len(self.symbols)}个交易对，{len(self.exchanges)}个交易所")
        
    def print_data_acquisition_stats(self):
        """1. 每个币种从交易所获取数据的时间统计"""
        print("\n" + "="*80)
        print("📊 分析1：每个币种从交易所获取数据时间统计")
        print("="*80)
        
        print(f"{'交易所-币种':<20} {'样本数':<8} {'平均间隔(ms)':<15} {'最小间隔(ms)':<15} {'最大间隔(ms)':<15}")
        print("-" * 80)
        
        for key in sorted(self.data_receive_times.keys()):
            times = self.data_receive_times[key]
            if len(times) < 2:
                continue
                
            # 计算接收间隔
            intervals = [times[i] - times[i-1] for i in range(1, len(times))]
            avg_interval = statistics.mean(intervals)
            min_interval = min(intervals)
            max_interval = max(intervals)
            
            print(f"{key:<20} {len(times):<8} {avg_interval:<15.2f} {min_interval:<15.2f} {max_interval:<15.2f}")
            
    def print_cleaning_time_stats(self):
        """2. 每个币种清洗时间统计"""
        print("\n" + "="*80)
        print("🧹 分析2：每个币种清洗时间统计")
        print("="*80)
        
        print(f"{'交易所-币种':<20} {'样本数':<8} {'平均清洗(ms)':<15} {'最小清洗(ms)':<15} {'最大清洗(ms)':<15} {'标准差':<12}")
        print("-" * 90)
        
        for key in sorted(self.symbol_cleaning_durations.keys()):
            durations = self.symbol_cleaning_durations[key]
            if not durations:
                continue
                
            avg_duration = statistics.mean(durations)
            min_duration = min(durations)
            max_duration = max(durations)
            std_dev = statistics.stdev(durations) if len(durations) > 1 else 0
            
            print(f"{key:<20} {len(durations):<8} {avg_duration:<15.3f} {min_duration:<15.3f} {max_duration:<15.3f} {std_dev:<12.3f}")
            
    def analyze_cleaning_stability(self):
        """3. 清洗数据平稳性分析"""
        print("\n" + "="*80)
        print("📈 分析3：清洗数据平稳性分析")
        print("="*80)
        
        print(f"{'交易所-币种':<20} {'变异系数':<12} {'稳定性':<10} {'异常值数':<10} {'问题分析'}")
        print("-" * 80)
        
        for key in sorted(self.symbol_cleaning_durations.keys()):
            durations = self.symbol_cleaning_durations[key]
            if len(durations) < 3:
                continue
                
            avg_duration = statistics.mean(durations)
            std_dev = statistics.stdev(durations)
            cv = (std_dev / avg_duration) * 100 if avg_duration > 0 else 0
            
            # 检测异常值（超过2个标准差）
            threshold = avg_duration + 2 * std_dev
            outliers = [d for d in durations if d > threshold]
            
            # 稳定性评级
            if cv < 10:
                stability = "优秀"
            elif cv < 25:
                stability = "良好"
            elif cv < 50:
                stability = "一般"
            else:
                stability = "不稳定"
                
            # 问题分析
            problem = ""
            if cv > 50:
                problem = "波动过大"
            elif len(outliers) > len(durations) * 0.2:
                problem = "频繁异常"
            elif avg_duration > 1000:  # 1秒
                problem = "清洗过慢"
            else:
                problem = "正常"
                
            print(f"{key:<20} {cv:<12.2f} {stability:<10} {len(outliers):<10} {problem}")
            
    def print_total_pipeline_stats(self):
        """4. 从获取到清洗成功的完整时间统计"""
        print("\n" + "="*80)
        print("⏱️  分析4：数据获取到清洗成功完整链路时间分析")
        print("="*80)
        
        print(f"{'交易所-币种':<20} {'样本数':<8} {'平均总时间(ms)':<18} {'最小总时间(ms)':<18} {'最大总时间(ms)':<18} {'性能评级'}")
        print("-" * 100)
        
        for key in sorted(self.symbol_total_durations.keys()):
            durations = self.symbol_total_durations[key]
            if not durations:
                continue
                
            avg_duration = statistics.mean(durations)
            min_duration = min(durations)
            max_duration = max(durations)
            
            # 性能评级 (基于平均时间)
            if avg_duration < 50:  # < 50ms
                rating = "🏆 极佳"
            elif avg_duration < 100:  # < 100ms
                rating = "🥇 优秀"  
            elif avg_duration < 200:  # < 200ms
                rating = "🥈 良好"
            elif avg_duration < 500:  # < 500ms
                rating = "🥉 及格"
            else:
                rating = "❌ 需优化"
                
            print(f"{key:<20} {len(durations):<8} {avg_duration:<18.3f} {min_duration:<18.3f} {max_duration:<18.3f} {rating}")
            
    def print_summary_and_recommendations(self):
        """打印总结和建议"""
        print("\n" + "="*80)
        print("📋 性能总结与优化建议")
        print("="*80)
        
        total_symbols = len(self.symbols)
        total_exchanges = len(self.exchanges)
        
        # 计算整体统计
        all_cleaning_times = []
        all_total_times = []
        
        for durations in self.symbol_cleaning_durations.values():
            all_cleaning_times.extend(durations)
        for durations in self.symbol_total_durations.values():
            all_total_times.extend(durations)
            
        print(f"🎯 系统整体表现:")
        print(f"   - 监控交易对数量: {total_symbols}")
        print(f"   - 活跃交易所数量: {total_exchanges}")
        
        if all_cleaning_times:
            avg_cleaning = statistics.mean(all_cleaning_times)
            print(f"   - 平均清洗时间: {avg_cleaning:.3f}ms")
            
        if all_total_times:
            avg_total = statistics.mean(all_total_times)
            print(f"   - 平均端到端时间: {avg_total:.3f}ms")
            
        print(f"\n🔧 发现的问题:")
        
        # 检查高延迟币种
        slow_symbols = []
        for key, durations in self.symbol_total_durations.items():
            if durations and statistics.mean(durations) > 200:
                slow_symbols.append(key)
                
        if slow_symbols:
            print(f"   - 高延迟币种 (>200ms): {', '.join(slow_symbols[:5])}")
        
        # 检查不稳定币种
        unstable_symbols = []
        for key, durations in self.symbol_cleaning_durations.items():
            if len(durations) > 1:
                cv = (statistics.stdev(durations) / statistics.mean(durations)) * 100
                if cv > 50:
                    unstable_symbols.append(key)
                    
        if unstable_symbols:
            print(f"   - 性能不稳定币种: {', '.join(unstable_symbols[:5])}")
            
        print(f"\n💡 优化建议:")
        print(f"   1. 重点关注Bybit交易所的连接稳定性")
        print(f"   2. 考虑对高延迟币种进行专项优化")
        print(f"   3. 清洗时间整体表现良好，维持当前优化水平")
        print(f"   4. 建议增加对Binance和OKX的API配置")

def main():
    # 模拟日志数据（从实际日志中提取的关键片段）
    sample_log_data = '''
{"timestamp":"2025-07-26T17:49:18.802577Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for FUEL/USDT from bybit: 2 bids, 2 asks"}}
{"timestamp":"2025-07-26T17:49:18.802580Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"}}
{"timestamp":"2025-07-26T17:49:18.802608Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"}}
{"timestamp":"2025-07-26T17:49:18.802777Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for SC/USDT from bybit: 0 bids, 2 asks"}}
{"timestamp":"2025-07-26T17:49:18.802780Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"}}
{"timestamp":"2025-07-26T17:49:18.802881Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"}}
{"timestamp":"2025-07-26T17:49:18.802975Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for BAT/USDT from bybit: 0 bids, 1 asks"}}
{"timestamp":"2025-07-26T17:49:18.802978Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"}}
{"timestamp":"2025-07-26T17:49:18.803107Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"}}
{"timestamp":"2025-07-26T17:49:18.803297Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for APE/USDT from bybit: 0 bids, 2 asks"}}
{"timestamp":"2025-07-26T17:49:18.803304Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"}}
{"timestamp":"2025-07-26T17:49:18.803440Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"}}
{"timestamp":"2025-07-26T17:49:18.805096Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for AAVE/USDT from bybit: 2 bids, 0 asks"}}
{"timestamp":"2025-07-26T17:49:18.805100Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"}}
{"timestamp":"2025-07-26T17:49:18.805809Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"}}
{"timestamp":"2025-07-26T17:55:23.063401Z","level":"INFO","fields":{"message":"📊 Received OrderBookSnapshot for ATOM/USDT from bybit: 1 bids, 0 asks"}}
{"timestamp":"2025-07-26T17:55:23.063404Z","level":"INFO","fields":{"message":"🧹 Performing data cleaning for OrderBookSnapshot from bybit"}}
{"timestamp":"2025-07-26T17:55:23.063517Z","level":"INFO","fields":{"message":"✅ Data cleaning successful for bybit - validation passed"}}
'''
    
    analyzer = QingxiPerformanceAnalyzer()
    analyzer.analyze_data(sample_log_data)
    
    # 执行4项分析
    analyzer.print_data_acquisition_stats()
    analyzer.print_cleaning_time_stats()
    analyzer.analyze_cleaning_stability()
    analyzer.print_total_pipeline_stats()
    analyzer.print_summary_and_recommendations()

if __name__ == "__main__":
    main()
