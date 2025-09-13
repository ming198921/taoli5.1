#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
简化版AVX-512性能优化验证测试
直接测试编译后的二进制文件性能
"""

import asyncio
import time
import json
import logging
import subprocess
import psutil
import numpy as np
from pathlib import Path
import nats
import random

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class OptimizationTest:
    def __init__(self):
        self.workspace = Path.cwd()
        self.binary_path = self.workspace / 'target' / 'x86_64-unknown-linux-gnu' / 'release' / 'celue-arbitrage-monitor'
        
    async def run_performance_test(self):
        """运行性能优化测试"""
        logger.info("🚀 开始AVX-512性能优化测试...")
        
        # 1. 编译优化版本
        logger.info("📦 编译高性能版本...")
        result = subprocess.run([
            'cargo', 'build', '--release', '--target=x86_64-unknown-linux-gnu'
        ], cwd=self.workspace, capture_output=True, text=True)
        
        if result.returncode != 0:
            logger.error(f"编译失败: {result.stderr}")
            return
        
        # 2. 验证CPU支持
        self.check_cpu_features()
        
        # 3. 启动高频数据测试
        await self.start_high_frequency_test()
        
    def check_cpu_features(self):
        """检查CPU特性"""
        try:
            result = subprocess.run(['lscpu'], capture_output=True, text=True)
            cpu_info = result.stdout
            
            if 'avx512f' in cpu_info.lower():
                logger.info("✅ CPU支持AVX-512F")
            else:
                logger.warning("⚠️ CPU不支持AVX-512F")
                
            with open('/proc/cpuinfo', 'r') as f:
                cpuinfo = f.read()
                
            supported_features = []
            if 'avx512f' in cpuinfo: supported_features.append('AVX-512F')
            if 'avx512dq' in cpuinfo: supported_features.append('AVX-512DQ') 
            if 'avx512bw' in cpuinfo: supported_features.append('AVX-512BW')
            if 'avx2' in cpuinfo: supported_features.append('AVX2')
            
            logger.info(f"支持的SIMD特性: {', '.join(supported_features)}")
            
        except Exception as e:
            logger.error(f"无法检测CPU特性: {e}")
    
    async def start_high_frequency_test(self):
        """启动高频测试"""
        logger.info("⚡ 启动高频数据测试...")
        
        # 连接NATS
        try:
            nc = await nats.connect("nats://localhost:4222")
            logger.info("✅ NATS连接成功")
        except Exception as e:
            logger.error(f"NATS连接失败: {e}")
            return
        
        # 生成高频测试数据
        test_duration = 300  # 5分钟
        target_rate = 100_000  # 100k/秒
        total_messages = test_duration * target_rate
        
        logger.info(f"📊 开始发送 {total_messages:,} 条消息，目标速率: {target_rate:,}/秒")
        
        start_time = time.time()
        sent_count = 0
        
        # 批量发送优化
        batch_size = 2048  # 使用优化的批处理大小
        
        try:
            for batch_idx in range(0, total_messages, batch_size):
                batch_start = time.time()
                
                # 生成一批数据
                batch_data = []
                for i in range(batch_size):
                    if sent_count >= total_messages:
                        break
                        
                    data = self.generate_market_data(sent_count)
                    batch_data.append(data)
                    sent_count += 1
                
                # 批量发送
                for data in batch_data:
                    await nc.publish("qx.v5.md.clean.perf.test.ob50", json.dumps(data).encode())
                
                # 性能监控
                if batch_idx % (batch_size * 10) == 0:  # 每10个批次报告一次
                    elapsed = time.time() - start_time
                    current_rate = sent_count / elapsed if elapsed > 0 else 0
                    remaining_time = (total_messages - sent_count) / current_rate if current_rate > 0 else 0
                    
                    logger.info(f"📊 已发送: {sent_count:,} 条, 速率: {current_rate:.0f}/秒, 剩余: {remaining_time:.0f}秒")
                
                # 速率控制
                batch_time = time.time() - batch_start
                target_batch_time = batch_size / target_rate
                if batch_time < target_batch_time:
                    await asyncio.sleep(target_batch_time - batch_time)
                    
        except KeyboardInterrupt:
            logger.info("⏹️ 测试被中断")
        except Exception as e:
            logger.error(f"测试出错: {e}")
        
        # 测试完成统计
        total_time = time.time() - start_time
        actual_rate = sent_count / total_time
        
        logger.info(f"📈 测试完成:")
        logger.info(f"  总发送: {sent_count:,} 条消息")
        logger.info(f"  总时长: {total_time:.2f} 秒")
        logger.info(f"  实际速率: {actual_rate:.0f} 消息/秒")
        logger.info(f"  目标达成率: {(actual_rate/target_rate)*100:.1f}%")
        
        await nc.close()
        
    def generate_market_data(self, sequence):
        """生成市场数据"""
        exchanges = ['binance', 'okx', 'huobi', 'bybit', 'gateio']
        symbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'XRPUSDT', 'ADAUSDT', 'SOLUSDT']
        
        exchange = random.choice(exchanges)
        symbol = random.choice(symbols)
        
        base_price = random.uniform(100, 50000)
        spread = base_price * random.uniform(0.0001, 0.01)
        
        bid_price = base_price - spread/2
        ask_price = base_price + spread/2
        
        return {
            "exchange": exchange,
            "symbol": symbol,
            "timestamp": int(time.time() * 1000000000),
            "bids": [[bid_price, random.uniform(1.0, 100.0)]],
            "asks": [[ask_price, random.uniform(1.0, 100.0)]],
            "sequence": sequence,
            "mid_price": int((bid_price + ask_price) / 2 * 100_000_000),
            "spread": int((ask_price - bid_price) * 100_000_000),
        }

async def main():
    test = OptimizationTest()
    await test.run_performance_test()

if __name__ == "__main__":
    asyncio.run(main()) 
# -*- coding: utf-8 -*-
"""
简化版AVX-512性能优化验证测试
直接测试编译后的二进制文件性能
"""

import asyncio
import time
import json
import logging
import subprocess
import psutil
import numpy as np
from pathlib import Path
import nats
import random

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class OptimizationTest:
    def __init__(self):
        self.workspace = Path.cwd()
        self.binary_path = self.workspace / 'target' / 'x86_64-unknown-linux-gnu' / 'release' / 'celue-arbitrage-monitor'
        
    async def run_performance_test(self):
        """运行性能优化测试"""
        logger.info("🚀 开始AVX-512性能优化测试...")
        
        # 1. 编译优化版本
        logger.info("📦 编译高性能版本...")
        result = subprocess.run([
            'cargo', 'build', '--release', '--target=x86_64-unknown-linux-gnu'
        ], cwd=self.workspace, capture_output=True, text=True)
        
        if result.returncode != 0:
            logger.error(f"编译失败: {result.stderr}")
            return
        
        # 2. 验证CPU支持
        self.check_cpu_features()
        
        # 3. 启动高频数据测试
        await self.start_high_frequency_test()
        
    def check_cpu_features(self):
        """检查CPU特性"""
        try:
            result = subprocess.run(['lscpu'], capture_output=True, text=True)
            cpu_info = result.stdout
            
            if 'avx512f' in cpu_info.lower():
                logger.info("✅ CPU支持AVX-512F")
            else:
                logger.warning("⚠️ CPU不支持AVX-512F")
                
            with open('/proc/cpuinfo', 'r') as f:
                cpuinfo = f.read()
                
            supported_features = []
            if 'avx512f' in cpuinfo: supported_features.append('AVX-512F')
            if 'avx512dq' in cpuinfo: supported_features.append('AVX-512DQ') 
            if 'avx512bw' in cpuinfo: supported_features.append('AVX-512BW')
            if 'avx2' in cpuinfo: supported_features.append('AVX2')
            
            logger.info(f"支持的SIMD特性: {', '.join(supported_features)}")
            
        except Exception as e:
            logger.error(f"无法检测CPU特性: {e}")
    
    async def start_high_frequency_test(self):
        """启动高频测试"""
        logger.info("⚡ 启动高频数据测试...")
        
        # 连接NATS
        try:
            nc = await nats.connect("nats://localhost:4222")
            logger.info("✅ NATS连接成功")
        except Exception as e:
            logger.error(f"NATS连接失败: {e}")
            return
        
        # 生成高频测试数据
        test_duration = 300  # 5分钟
        target_rate = 100_000  # 100k/秒
        total_messages = test_duration * target_rate
        
        logger.info(f"📊 开始发送 {total_messages:,} 条消息，目标速率: {target_rate:,}/秒")
        
        start_time = time.time()
        sent_count = 0
        
        # 批量发送优化
        batch_size = 2048  # 使用优化的批处理大小
        
        try:
            for batch_idx in range(0, total_messages, batch_size):
                batch_start = time.time()
                
                # 生成一批数据
                batch_data = []
                for i in range(batch_size):
                    if sent_count >= total_messages:
                        break
                        
                    data = self.generate_market_data(sent_count)
                    batch_data.append(data)
                    sent_count += 1
                
                # 批量发送
                for data in batch_data:
                    await nc.publish("qx.v5.md.clean.perf.test.ob50", json.dumps(data).encode())
                
                # 性能监控
                if batch_idx % (batch_size * 10) == 0:  # 每10个批次报告一次
                    elapsed = time.time() - start_time
                    current_rate = sent_count / elapsed if elapsed > 0 else 0
                    remaining_time = (total_messages - sent_count) / current_rate if current_rate > 0 else 0
                    
                    logger.info(f"📊 已发送: {sent_count:,} 条, 速率: {current_rate:.0f}/秒, 剩余: {remaining_time:.0f}秒")
                
                # 速率控制
                batch_time = time.time() - batch_start
                target_batch_time = batch_size / target_rate
                if batch_time < target_batch_time:
                    await asyncio.sleep(target_batch_time - batch_time)
                    
        except KeyboardInterrupt:
            logger.info("⏹️ 测试被中断")
        except Exception as e:
            logger.error(f"测试出错: {e}")
        
        # 测试完成统计
        total_time = time.time() - start_time
        actual_rate = sent_count / total_time
        
        logger.info(f"📈 测试完成:")
        logger.info(f"  总发送: {sent_count:,} 条消息")
        logger.info(f"  总时长: {total_time:.2f} 秒")
        logger.info(f"  实际速率: {actual_rate:.0f} 消息/秒")
        logger.info(f"  目标达成率: {(actual_rate/target_rate)*100:.1f}%")
        
        await nc.close()
        
    def generate_market_data(self, sequence):
        """生成市场数据"""
        exchanges = ['binance', 'okx', 'huobi', 'bybit', 'gateio']
        symbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'XRPUSDT', 'ADAUSDT', 'SOLUSDT']
        
        exchange = random.choice(exchanges)
        symbol = random.choice(symbols)
        
        base_price = random.uniform(100, 50000)
        spread = base_price * random.uniform(0.0001, 0.01)
        
        bid_price = base_price - spread/2
        ask_price = base_price + spread/2
        
        return {
            "exchange": exchange,
            "symbol": symbol,
            "timestamp": int(time.time() * 1000000000),
            "bids": [[bid_price, random.uniform(1.0, 100.0)]],
            "asks": [[ask_price, random.uniform(1.0, 100.0)]],
            "sequence": sequence,
            "mid_price": int((bid_price + ask_price) / 2 * 100_000_000),
            "spread": int((ask_price - bid_price) * 100_000_000),
        }

async def main():
    test = OptimizationTest()
    await test.run_performance_test()

if __name__ == "__main__":
    asyncio.run(main()) 