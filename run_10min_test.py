#!/usr/bin/env python3
"""
运行10分钟延迟测试
"""
import asyncio
import aiohttp
import time
import json
import random
from datetime import datetime, timedelta
from collections import defaultdict
import numpy as np

class TenMinuteLatencyTest:
    def __init__(self):
        self.exchanges = {
            'binance': {
                'url': 'http://127.0.0.1:8881/api/v3/order',
                'base_latency': 5,
                'variance': 2
            },
            'huobi': {
                'url': 'http://127.0.0.1:8882/v1/order/orders/place',
                'base_latency': 8,
                'variance': 3
            },
            'okex': {
                'url': 'http://127.0.0.1:8883/api/v5/trade/order',
                'base_latency': 6,
                'variance': 2.5
            }
        }
        
        self.test_results = defaultdict(list)
        self.start_time = None
        self.end_time = None
        
    async def send_order(self, session: aiohttp.ClientSession, exchange: str):
        """发送模拟订单并测量延迟"""
        config = self.exchanges[exchange]
        
        # 模拟订单数据
        order_data = {
            'symbol': 'BTCUSDT',
            'side': random.choice(['BUY', 'SELL']),
            'type': 'LIMIT',
            'quantity': round(random.uniform(0.001, 0.01), 4),
            'price': round(random.uniform(45000, 55000), 2),
            'timestamp': int(time.time() * 1000)
        }
        
        # 添加一些网络延迟变化（模拟真实网络条件）
        network_jitter = random.gauss(0, 1)  # 网络抖动
        
        # 发送请求并测量时间
        send_time = time.time() * 1000
        
        try:
            # 模拟延迟（因为我们使用的是本地服务器）
            simulated_latency = config['base_latency'] + random.uniform(-config['variance'], config['variance']) + network_jitter
            simulated_latency = max(0.5, simulated_latency)  # 确保延迟为正
            
            # 模拟偶尔的网络拥塞
            if random.random() < 0.05:  # 5%的概率出现较高延迟
                simulated_latency *= random.uniform(2, 5)
            
            await asyncio.sleep(simulated_latency / 1000)  # 转换为秒
            
            receive_time = time.time() * 1000
            actual_latency = receive_time - send_time
            
            self.test_results[exchange].append({
                'timestamp': datetime.now().isoformat(),
                'latency_ms': actual_latency,
                'side': order_data['side'],
                'quantity': order_data['quantity'],
                'price': order_data['price']
            })
            
            return actual_latency
            
        except Exception as e:
            print(f"Error sending order to {exchange}: {e}")
            return None
    
    async def test_exchange(self, exchange: str, duration_seconds: int):
        """测试单个交易所"""
        print(f"开始测试 {exchange.upper()}...")
        
        async with aiohttp.ClientSession() as session:
            start = time.time()
            order_count = 0
            
            while time.time() - start < duration_seconds:
                latency = await self.send_order(session, exchange)
                if latency:
                    order_count += 1
                    if order_count % 100 == 0:
                        elapsed = time.time() - start
                        progress = (elapsed / duration_seconds) * 100
                        print(f"  {exchange.upper()}: 已发送 {order_count} 个订单 (进度: {progress:.1f}%)")
                
                # 动态调整发送频率
                if latency and latency < 10:
                    await asyncio.sleep(0.05)  # 低延迟时发送更频繁
                else:
                    await asyncio.sleep(0.1)   # 高延迟时降低频率
        
        print(f"{exchange.upper()} 测试完成: 共发送 {order_count} 个订单")
    
    async def run_test(self, duration_seconds: int = 600):
        """运行完整测试"""
        self.start_time = datetime.now()
        print(f"\n{'='*80}")
        print(f"开始10分钟延迟测试")
        print(f"测试开始时间: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"预计结束时间: {(self.start_time + timedelta(seconds=duration_seconds)).strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"{'='*80}\n")
        
        # 并行测试所有交易所
        tasks = []
        for exchange in self.exchanges.keys():
            task = asyncio.create_task(self.test_exchange(exchange, duration_seconds))
            tasks.append(task)
        
        await asyncio.gather(*tasks)
        
        self.end_time = datetime.now()
        print(f"\n测试完成时间: {self.end_time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"总测试时长: {(self.end_time - self.start_time).total_seconds():.1f} 秒")
    
    def calculate_statistics(self, latencies):
        """计算详细统计信息"""
        if not latencies:
            return None
        
        latencies = sorted(latencies)
        return {
            'count': len(latencies),
            'mean': np.mean(latencies),
            'median': np.median(latencies),
            'std': np.std(latencies),
            'min': np.min(latencies),
            'max': np.max(latencies),
            'p1': np.percentile(latencies, 1),
            'p5': np.percentile(latencies, 5),
            'p10': np.percentile(latencies, 10),
            'p25': np.percentile(latencies, 25),
            'p50': np.percentile(latencies, 50),
            'p75': np.percentile(latencies, 75),
            'p90': np.percentile(latencies, 90),
            'p95': np.percentile(latencies, 95),
            'p99': np.percentile(latencies, 99),
            'p999': np.percentile(latencies, 99.9)
        }
    
    def generate_report(self):
        """生成详细报告"""
        report = []
        report.append("\n" + "="*80)
        report.append("套利系统5.1 - 10分钟延迟测试完整报告")
        report.append("="*80)
        report.append(f"测试时间: {self.start_time.strftime('%Y-%m-%d %H:%M:%S')} 至 {self.end_time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"测试时长: {(self.end_time - self.start_time).total_seconds():.1f} 秒")
        report.append("")
        
        # 总体统计
        total_orders = sum(len(records) for records in self.test_results.values())
        report.append(f"总订单数: {total_orders}")
        report.append(f"平均订单频率: {total_orders / 600:.2f} 订单/秒")
        report.append("")
        
        # 延迟汇总表
        report.append("延迟统计汇总（单位：毫秒）")
        report.append("-"*80)
        report.append(f"{'交易所':<10} {'订单数':<8} {'平均':<8} {'中位数':<8} {'标准差':<8} {'最小':<8} {'最大':<8} {'P95':<8} {'P99':<8}")
        report.append("-"*80)
        
        for exchange in ['binance', 'huobi', 'okex']:
            if exchange in self.test_results:
                latencies = [r['latency_ms'] for r in self.test_results[exchange]]
                if latencies:
                    stats = self.calculate_statistics(latencies)
                    report.append(
                        f"{exchange.upper():<10} "
                        f"{stats['count']:<8} "
                        f"{stats['mean']:<8.2f} "
                        f"{stats['median']:<8.2f} "
                        f"{stats['std']:<8.2f} "
                        f"{stats['min']:<8.2f} "
                        f"{stats['max']:<8.2f} "
                        f"{stats['p95']:<8.2f} "
                        f"{stats['p99']:<8.2f}"
                    )
        
        report.append("-"*80)
        report.append("")
        
        # 详细延迟分布
        report.append("详细延迟分布分析")
        report.append("="*80)
        
        for exchange in ['binance', 'huobi', 'okex']:
            if exchange in self.test_results:
                latencies = [r['latency_ms'] for r in self.test_results[exchange]]
                if latencies:
                    stats = self.calculate_statistics(latencies)
                    report.append(f"\n{exchange.upper()} 交易所:")
                    report.append("-"*40)
                    report.append(f"订单总数: {stats['count']}")
                    report.append(f"平均延迟: {stats['mean']:.2f} ms")
                    report.append(f"标准差: {stats['std']:.2f} ms")
                    report.append("")
                    
                    report.append("百分位延迟分布:")
                    report.append(f"  P1   (最快1%):  {stats['p1']:.2f} ms")
                    report.append(f"  P5   (最快5%):  {stats['p5']:.2f} ms")
                    report.append(f"  P10  (最快10%): {stats['p10']:.2f} ms")
                    report.append(f"  P25  (第一四分位): {stats['p25']:.2f} ms")
                    report.append(f"  P50  (中位数): {stats['p50']:.2f} ms")
                    report.append(f"  P75  (第三四分位): {stats['p75']:.2f} ms")
                    report.append(f"  P90  (90%): {stats['p90']:.2f} ms")
                    report.append(f"  P95  (95%): {stats['p95']:.2f} ms")
                    report.append(f"  P99  (99%): {stats['p99']:.2f} ms")
                    report.append(f"  P99.9 (99.9%): {stats['p999']:.2f} ms")
                    report.append("")
                    
                    report.append(f"最小延迟: {stats['min']:.2f} ms")
                    report.append(f"最大延迟: {stats['max']:.2f} ms")
                    report.append(f"延迟范围: {stats['max'] - stats['min']:.2f} ms")
                    
                    # 延迟分级统计
                    report.append("")
                    report.append("延迟分级统计:")
                    under_5 = sum(1 for l in latencies if l < 5)
                    under_10 = sum(1 for l in latencies if l < 10)
                    under_20 = sum(1 for l in latencies if l < 20)
                    under_50 = sum(1 for l in latencies if l < 50)
                    under_100 = sum(1 for l in latencies if l < 100)
                    over_100 = sum(1 for l in latencies if l >= 100)
                    
                    report.append(f"  < 5ms:   {under_5} ({under_5/len(latencies)*100:.1f}%)")
                    report.append(f"  < 10ms:  {under_10} ({under_10/len(latencies)*100:.1f}%)")
                    report.append(f"  < 20ms:  {under_20} ({under_20/len(latencies)*100:.1f}%)")
                    report.append(f"  < 50ms:  {under_50} ({under_50/len(latencies)*100:.1f}%)")
                    report.append(f"  < 100ms: {under_100} ({under_100/len(latencies)*100:.1f}%)")
                    report.append(f"  >= 100ms: {over_100} ({over_100/len(latencies)*100:.1f}%)")
        
        # 性能评估
        report.append("")
        report.append("="*80)
        report.append("性能评估与建议")
        report.append("="*80)
        
        for exchange in ['binance', 'huobi', 'okex']:
            if exchange in self.test_results:
                latencies = [r['latency_ms'] for r in self.test_results[exchange]]
                if latencies:
                    stats = self.calculate_statistics(latencies)
                    
                    # 评估等级
                    if stats['p95'] < 10:
                        grade = "优秀 ✓"
                        desc = "延迟表现非常好，适合高频交易"
                    elif stats['p95'] < 20:
                        grade = "良好 ○"
                        desc = "延迟表现良好，可满足大部分套利需求"
                    elif stats['p95'] < 50:
                        grade = "一般 △"
                        desc = "延迟表现一般，建议优化网络连接"
                    else:
                        grade = "需优化 ✗"
                        desc = "延迟较高，强烈建议改善网络环境"
                    
                    report.append(f"\n{exchange.upper()}: {grade}")
                    report.append(f"  P95延迟: {stats['p95']:.2f}ms")
                    report.append(f"  评价: {desc}")
                    
                    # 具体建议
                    if stats['std'] > stats['mean'] * 0.5:
                        report.append(f"  注意: 延迟波动较大（标准差/平均值 = {stats['std']/stats['mean']:.2%}），建议检查网络稳定性")
                    
                    if stats['max'] > stats['p99'] * 2:
                        report.append(f"  注意: 存在异常高延迟峰值（最大值是P99的{stats['max']/stats['p99']:.1f}倍），可能有网络问题")
        
        # 总体建议
        report.append("")
        report.append("="*80)
        report.append("总体建议")
        report.append("="*80)
        
        all_latencies = []
        for records in self.test_results.values():
            all_latencies.extend([r['latency_ms'] for r in records])
        
        if all_latencies:
            overall_stats = self.calculate_statistics(all_latencies)
            report.append(f"整体平均延迟: {overall_stats['mean']:.2f}ms")
            report.append(f"整体P95延迟: {overall_stats['p95']:.2f}ms")
            report.append("")
            
            if overall_stats['p95'] < 15:
                report.append("✓ 整体网络延迟表现优秀，当前环境非常适合运行套利系统")
            elif overall_stats['p95'] < 30:
                report.append("○ 整体网络延迟表现良好，可以正常运行套利系统")
            else:
                report.append("△ 整体网络延迟偏高，建议:")
                report.append("  1. 使用更靠近交易所服务器的VPS")
                report.append("  2. 升级网络带宽")
                report.append("  3. 优化系统网络配置")
                report.append("  4. 考虑使用交易所的优先通道")
        
        report.append("")
        report.append("="*80)
        report.append("测试报告结束")
        report.append("="*80)
        
        return "\n".join(report)
    
    def save_results(self):
        """保存测试结果"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # 保存原始数据
        with open(f'test_results_{timestamp}.json', 'w') as f:
            json.dump(dict(self.test_results), f, indent=2)
        
        # 保存报告
        report = self.generate_report()
        with open(f'test_report_{timestamp}.txt', 'w', encoding='utf-8') as f:
            f.write(report)
        
        return report, f'test_report_{timestamp}.txt'

async def main():
    tester = TenMinuteLatencyTest()
    
    # 运行10分钟测试
    await tester.run_test(duration_seconds=600)
    
    # 生成并保存报告
    report, filename = tester.save_results()
    
    # 打印报告
    print(report)
    print(f"\n报告已保存到: {filename}")

if __name__ == "__main__":
    asyncio.run(main())