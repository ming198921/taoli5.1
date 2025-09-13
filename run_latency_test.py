#!/usr/bin/env python3
"""
套利系统5.1延迟测试主程序

使用方法:
1. 启动模拟交易所服务器
2. 配置套利系统5.1连接到模拟服务器
3. 运行测试并生成报告
"""

import asyncio
import sys
import os
import time
import json
import subprocess
import signal
from datetime import datetime
import argparse
from typing import Optional

# 导入测试组件
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from test_framework import LatencyTestFramework
from exchange_simulator import MockExchangeServer, run_mock_exchanges
from report_generator import ReportGenerator
from network_interceptor import NetworkInterceptor, DirectInterceptor

class ArbitrageLatencyTester:
    """套利系统延迟测试器"""
    
    def __init__(self, test_mode: str = 'simulator'):
        """
        初始化测试器
        
        Args:
            test_mode: 测试模式
                - 'simulator': 使用模拟交易所测试
                - 'interceptor': 拦截真实请求测试
                - 'hybrid': 混合模式
        """
        self.test_mode = test_mode
        self.processes = []
        self.test_results = {}
        
    async def setup_mock_exchanges(self):
        """设置模拟交易所"""
        print("正在启动模拟交易所服务器...")
        
        # 启动交易所模拟器进程
        exchange_process = subprocess.Popen(
            [sys.executable, 'exchange_simulator.py'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        self.processes.append(exchange_process)
        
        # 等待服务器启动
        await asyncio.sleep(2)
        
        print("模拟交易所服务器已启动:")
        print("  - Binance: http://127.0.0.1:8881")
        print("  - Huobi: http://127.0.0.1:8882")
        print("  - OKEx: http://127.0.0.1:8883")
        
        return True
    
    def configure_arbitrage_system(self):
        """配置套利系统5.1使用模拟服务器"""
        config_instructions = """
请按以下步骤配置套利系统5.1:

1. 修改配置文件，将交易所API地址改为:
   - Binance: http://127.0.0.1:8881
   - Huobi: http://127.0.0.1:8882
   - OKEx: http://127.0.0.1:8883

2. 确保套利系统使用测试模式（不进行真实交易）

3. 启动套利系统5.1

配置完成后按Enter继续...
        """
        print(config_instructions)
        input()
    
    async def run_simulator_test(self, duration: int = 60):
        """运行模拟器测试"""
        print(f"\n开始模拟器测试 (持续 {duration} 秒)...")
        
        framework = LatencyTestFramework()
        await framework.run_full_test(test_duration=duration)
        
        return framework.latency_records
    
    async def run_interceptor_test(self, duration: int = 60):
        """运行拦截器测试"""
        print(f"\n开始拦截器测试 (持续 {duration} 秒)...")
        
        interceptor = DirectInterceptor()
        
        # 启动TCP拦截器
        tasks = []
        exchanges = [
            ('binance', 'api.binance.com', 443, 9881),
            ('huobi', 'api.huobi.pro', 443, 9882),
            ('okex', 'www.okx.com', 443, 9883)
        ]
        
        for exchange, host, port, local_port in exchanges:
            task = asyncio.create_task(
                interceptor.intercept_requests(host, port, local_port, exchange)
            )
            tasks.append(task)
        
        print("TCP拦截器已启动:")
        for exchange, _, _, local_port in exchanges:
            print(f"  - {exchange}: 127.0.0.1:{local_port}")
        
        # 运行指定时间
        await asyncio.sleep(duration)
        
        # 停止拦截器
        for task in tasks:
            task.cancel()
        
        return interceptor.latency_records
    
    async def run_test(self, duration: int = 60):
        """运行完整测试"""
        try:
            # 设置模拟交易所
            if self.test_mode in ['simulator', 'hybrid']:
                await self.setup_mock_exchanges()
                self.configure_arbitrage_system()
            
            # 运行测试
            if self.test_mode == 'simulator':
                results = await self.run_simulator_test(duration)
            elif self.test_mode == 'interceptor':
                results = await self.run_interceptor_test(duration)
            else:  # hybrid
                sim_results = await self.run_simulator_test(duration // 2)
                int_results = await self.run_interceptor_test(duration // 2)
                results = {**sim_results, **int_results}
            
            self.test_results = results
            
            # 生成报告
            self.generate_report()
            
        finally:
            # 清理进程
            self.cleanup()
    
    def generate_report(self):
        """生成测试报告"""
        print("\n正在生成测试报告...")
        
        generator = ReportGenerator()
        
        # 添加测试结果
        for exchange, records in self.test_results.items():
            generator.add_test_results(exchange, records)
        
        # 生成文本报告
        text_report = generator.generate_text_report()
        print("\n" + text_report)
        
        # 保存报告文件
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        
        # 保存文本报告
        text_file = f'latency_report_{timestamp}.txt'
        with open(text_file, 'w') as f:
            f.write(text_report)
        
        # 保存JSON报告
        json_file = generator.save_json_report(f'latency_report_{timestamp}.json')
        
        # 生成图表
        try:
            generator.generate_charts(f'reports_{timestamp}')
            print(f"\n图表已保存到 reports_{timestamp}/ 目录")
        except Exception as e:
            print(f"生成图表时出错: {e}")
        
        print(f"\n报告文件:")
        print(f"  - 文本报告: {text_file}")
        print(f"  - JSON报告: {json_file}")
    
    def cleanup(self):
        """清理资源"""
        print("\n正在清理...")
        for process in self.processes:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()

async def main():
    parser = argparse.ArgumentParser(description='套利系统5.1延迟测试')
    parser.add_argument(
        '--mode', 
        choices=['simulator', 'interceptor', 'hybrid'],
        default='simulator',
        help='测试模式'
    )
    parser.add_argument(
        '--duration',
        type=int,
        default=60,
        help='测试持续时间（秒）'
    )
    
    args = parser.parse_args()
    
    print("="*80)
    print("套利系统5.1 延迟测试框架")
    print("="*80)
    print(f"测试模式: {args.mode}")
    print(f"测试时长: {args.duration} 秒")
    print("="*80)
    
    tester = ArbitrageLatencyTester(test_mode=args.mode)
    
    try:
        await tester.run_test(duration=args.duration)
    except KeyboardInterrupt:
        print("\n测试被用户中断")
    except Exception as e:
        print(f"\n测试出错: {e}")
        import traceback
        traceback.print_exc()
    finally:
        tester.cleanup()

if __name__ == "__main__":
    asyncio.run(main())