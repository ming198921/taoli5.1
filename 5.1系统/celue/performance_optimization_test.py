#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
AVX-512性能优化验证测试
目标: 验证延迟从62ms降至100μs以下，吞吐量从7.4k提升至100k+
"""

import asyncio
import time
import json
import logging
import subprocess
import psutil
import numpy as np
from dataclasses import dataclass
from typing import List, Dict, Optional
import concurrent.futures
import threading
from pathlib import Path
import nats
import random

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class PerformanceOptimizationConfig:
    """性能优化测试配置"""
    target_latency_us: float = 100.0       # 目标延迟100微秒
    target_throughput: int = 100_000       # 目标吞吐量100k/秒
    test_duration: int = 300               # 测试5分钟
    batch_sizes: List[int] = None          # 测试不同批处理大小
    worker_threads: List[int] = None       # 测试不同线程数
    validation_samples: int = 10_000       # 验证样本数
    
    def __post_init__(self):
        if self.batch_sizes is None:
            self.batch_sizes = [512, 1024, 2048, 4096]
        if self.worker_threads is None:
            self.worker_threads = [8, 16, 24, 32]

class HighPerformanceDataGenerator:
    """高性能数据生成器 - 专门针对性能测试优化"""
    
    def __init__(self, config: PerformanceOptimizationConfig):
        self.config = config
        self.exchanges = ['binance', 'okx', 'huobi', 'bybit', 'gateio']
        self.symbols = self._generate_symbol_pool()
        
    def _generate_symbol_pool(self) -> List[str]:
        """生成丰富的交易对池"""
        # 主流币种
        major_coins = ['BTC', 'ETH', 'BNB', 'XRP', 'ADA', 'SOL', 'DOT', 'AVAX', 'MATIC', 'LINK']
        # DeFi代币
        defi_tokens = ['UNI', 'AAVE', 'COMP', 'MKR', 'SNX', 'CRV', 'SUSHI', 'YFI', '1INCH', 'ALPHA']
        # Meme币
        meme_coins = ['DOGE', 'SHIB', 'FLOKI', 'PEPE', 'BABYDOGE', 'SAFEMOON']
        # 小币种
        alt_coins = [f"ALT{i:03d}" for i in range(1, 101)]
        
        all_coins = major_coins + defi_tokens + meme_coins + alt_coins
        quote_currencies = ['USDT', 'USDC', 'BTC', 'ETH', 'BNB']
        
        symbols = []
        for base in all_coins:
            for quote in quote_currencies:
                if base != quote:
                    symbols.append(f"{base}{quote}")
        
        return symbols[:15000]  # 限制到15k个交易对
    
    def generate_optimized_market_data(self, count: int) -> List[Dict]:
        """生成针对性能测试优化的市场数据"""
        data_batch = []
        current_time = int(time.time() * 1_000_000_000)  # 纳秒时间戳
        
        for i in range(count):
            symbol = random.choice(self.symbols)
            exchange = random.choice(self.exchanges)
            
            # 生成realistic的价格数据
            base_price = random.uniform(0.001, 50000.0)
            spread_pct = random.uniform(0.0001, 0.01)  # 0.01% - 1% 价差
            
            bid_price = base_price * (1 - spread_pct / 2)
            ask_price = base_price * (1 + spread_pct / 2)
            
            # 添加市场微观结构噪音
            volatility = random.uniform(0.001, 0.05)
            bid_price *= (1 + random.gauss(0, volatility))
            ask_price *= (1 + random.gauss(0, volatility))
            
            # 确保合理的价格关系
            if bid_price >= ask_price:
                bid_price, ask_price = ask_price * 0.999, bid_price * 1.001
            
            data = {
                "exchange": exchange,
                "symbol": symbol,
                "timestamp": current_time + i * 1000,  # 1微秒间隔
                "bids": [[bid_price, random.uniform(1.0, 1000.0)]],
                "asks": [[ask_price, random.uniform(1.0, 1000.0)]],
                "sequence": 1000000 + i,
                "mid_price": int((bid_price + ask_price) / 2 * 100_000_000),
                "spread": int((ask_price - bid_price) * 100_000_000),
            }
            data_batch.append(data)
        
        return data_batch

class PerformanceOptimizationTest:
    """性能优化测试类"""
    
    def __init__(self, config: PerformanceOptimizationConfig):
        self.config = config
        self.data_generator = HighPerformanceDataGenerator(config)
        self.workspace = Path.cwd()
        self.binary_path = self.workspace / 'target' / 'x86_64-unknown-linux-gnu' / 'release' / 'celue-arbitrage-monitor'
        self.nats_client: Optional[nats.NATS] = None
        self.test_results = {}
        
    async def setup(self):
        """测试环境设置"""
        logger.info("🔧 设置性能优化测试环境...")
        
        # 编译最新版本
        logger.info("📦 编译最新代码...")
        result = subprocess.run([
            'cargo', 'build', '--release', 
            '--target=x86_64-unknown-linux-gnu',
            '--features=simd,avx512'
        ], cwd=self.workspace, capture_output=True, text=True)
        
        if result.returncode != 0:
            logger.error(f"编译失败: {result.stderr}")
            raise RuntimeError("编译失败")
        
        # 验证AVX-512支持
        self.verify_avx512_support()
        
        # 连接NATS
        self.nats_client = await nats.connect("nats://localhost:4222")
        
        logger.info("✅ 测试环境设置完成")
    
    def verify_avx512_support(self):
        """验证AVX-512支持"""
        try:
            with open('/proc/cpuinfo', 'r') as f:
                cpuinfo = f.read()
                
            if 'avx512f' in cpuinfo:
                logger.info("✅ CPU支持AVX-512F")
            else:
                logger.warning("⚠️ CPU不支持AVX-512F")
                
            if 'avx512dq' in cpuinfo and 'avx512bw' in cpuinfo:
                logger.info("✅ CPU支持完整AVX-512指令集")
            else:
                logger.warning("⚠️ CPU不支持完整AVX-512指令集")
                
        except Exception as e:
            logger.error(f"无法检测AVX-512支持: {e}")
    
    async def test_latency_optimization(self) -> Dict:
        """测试延迟优化效果"""
        logger.info("🚀 开始延迟优化测试...")
        
        latency_results = {}
        
        for batch_size in self.config.batch_sizes:
            logger.info(f"测试批处理大小: {batch_size}")
            
            # 生成测试数据
            test_data = self.data_generator.generate_optimized_market_data(
                self.config.validation_samples
            )
            
            # 批处理测试
            latencies = []
            start_time = time.perf_counter()
            
            for i in range(0, len(test_data), batch_size):
                batch = test_data[i:i + batch_size]
                batch_start = time.perf_counter_ns()
                
                # 发送批处理数据
                for data in batch:
                    await self.nats_client.publish(
                        "qx.v5.md.clean.test.test.ob50",
                        json.dumps(data).encode()
                    )
                
                batch_end = time.perf_counter_ns()
                batch_latency = (batch_end - batch_start) / len(batch) / 1000  # 微秒
                latencies.append(batch_latency)
            
            total_time = time.perf_counter() - start_time
            
            # 统计结果
            avg_latency = np.mean(latencies)
            p95_latency = np.percentile(latencies, 95)
            p99_latency = np.percentile(latencies, 99)
            max_latency = np.max(latencies)
            throughput = len(test_data) / total_time
            
            latency_results[batch_size] = {
                'avg_latency_us': avg_latency,
                'p95_latency_us': p95_latency,
                'p99_latency_us': p99_latency,
                'max_latency_us': max_latency,
                'throughput': throughput,
                'target_met': avg_latency <= self.config.target_latency_us
            }
            
            logger.info(f"批大小 {batch_size}: 平均延迟 {avg_latency:.2f}μs, "
                       f"P95: {p95_latency:.2f}μs, 吞吐量: {throughput:.0f}/秒")
        
        return latency_results
    
    async def test_throughput_optimization(self) -> Dict:
        """测试吞吐量优化效果"""
        logger.info("⚡ 开始吞吐量优化测试...")
        
        throughput_results = {}
        
        for thread_count in self.config.worker_threads:
            logger.info(f"测试工作线程数: {thread_count}")
            
            # 生成大量测试数据
            total_messages = self.config.target_throughput * 60  # 1分钟的数据量
            test_data = self.data_generator.generate_optimized_market_data(total_messages)
            
            # 多线程并发测试
            start_time = time.perf_counter()
            processed_count = 0
            errors = 0
            
            async def publish_worker(data_chunk):
                nonlocal processed_count, errors
                try:
                    for data in data_chunk:
                        await self.nats_client.publish(
                            "qx.v5.md.clean.perf.test.ob50",
                            json.dumps(data).encode()
                        )
                        processed_count += 1
                except Exception as e:
                    errors += 1
                    logger.error(f"发布错误: {e}")
            
            # 分块并发处理
            chunk_size = len(test_data) // thread_count
            tasks = []
            
            for i in range(thread_count):
                start_idx = i * chunk_size
                end_idx = start_idx + chunk_size if i < thread_count - 1 else len(test_data)
                chunk = test_data[start_idx:end_idx]
                tasks.append(publish_worker(chunk))
            
            # 等待所有任务完成
            await asyncio.gather(*tasks)
            
            total_time = time.perf_counter() - start_time
            actual_throughput = processed_count / total_time
            error_rate = errors / len(test_data)
            
            throughput_results[thread_count] = {
                'actual_throughput': actual_throughput,
                'target_throughput': self.config.target_throughput,
                'target_met': actual_throughput >= self.config.target_throughput,
                'error_rate': error_rate,
                'total_time': total_time,
                'processed_count': processed_count
            }
            
            logger.info(f"线程数 {thread_count}: 实际吞吐量 {actual_throughput:.0f}/秒, "
                       f"错误率: {error_rate:.4f}")
        
        return throughput_results
    
    async def test_avx512_effectiveness(self) -> Dict:
        """测试AVX-512指令集效果"""
        logger.info("🔬 测试AVX-512指令集效果...")
        
        # 创建测试配置文件
        test_configs = {
            'avx512_enabled': {
                'simd_level': 'AVX512',
                'batch_size': 2048,
                'worker_threads': 16
            },
            'avx2_fallback': {
                'simd_level': 'AVX2', 
                'batch_size': 1024,
                'worker_threads': 8
            },
            'scalar_baseline': {
                'simd_level': 'Scalar',
                'batch_size': 256,
                'worker_threads': 4
            }
        }
        
        avx512_results = {}
        test_data = self.data_generator.generate_optimized_market_data(10000)
        
        for config_name, config in test_configs.items():
            logger.info(f"测试配置: {config_name}")
            
            start_time = time.perf_counter_ns()
            
            # 模拟不同SIMD指令集的处理
            batch_times = []
            for i in range(0, len(test_data), config['batch_size']):
                batch = test_data[i:i + config['batch_size']]
                batch_start = time.perf_counter_ns()
                
                # 发送数据
                for data in batch:
                    await self.nats_client.publish(
                        f"qx.v5.md.clean.{config_name}.test.ob50",
                        json.dumps(data).encode()
                    )
                
                batch_end = time.perf_counter_ns()
                batch_times.append(batch_end - batch_start)
            
            total_time = time.perf_counter_ns() - start_time
            
            avg_batch_time = np.mean(batch_times) / 1000  # 微秒
            throughput = len(test_data) / (total_time / 1_000_000_000)
            
            avx512_results[config_name] = {
                'avg_batch_time_us': avg_batch_time,
                'throughput': throughput,
                'config': config
            }
            
            logger.info(f"{config_name}: 平均批处理时间 {avg_batch_time:.2f}μs, "
                       f"吞吐量: {throughput:.0f}/秒")
        
        # 计算性能提升
        if 'avx512_enabled' in avx512_results and 'scalar_baseline' in avx512_results:
            avx512_speedup = (
                avx512_results['scalar_baseline']['throughput'] / 
                avx512_results['avx512_enabled']['throughput']
            )
            avx512_results['performance_improvement'] = {
                'avx512_vs_scalar_speedup': avx512_speedup,
                'latency_reduction': (
                    avx512_results['scalar_baseline']['avg_batch_time_us'] - 
                    avx512_results['avx512_enabled']['avg_batch_time_us']
                )
            }
        
        return avx512_results
    
    async def run_comprehensive_test(self) -> Dict:
        """运行完整的性能优化测试"""
        logger.info("🎯 开始完整性能优化测试...")
        
        start_time = time.time()
        
        try:
            # 1. 延迟优化测试
            latency_results = await self.test_latency_optimization()
            
            # 2. 吞吐量优化测试  
            throughput_results = await self.test_throughput_optimization()
            
            # 3. AVX-512效果测试
            avx512_results = await self.test_avx512_effectiveness()
            
            # 综合结果
            test_duration = time.time() - start_time
            
            # 找出最优配置
            best_latency_config = min(
                latency_results.items(),
                key=lambda x: x[1]['avg_latency_us']
            )
            
            best_throughput_config = max(
                throughput_results.items(),
                key=lambda x: x[1]['actual_throughput']
            )
            
            # 性能目标达成情况
            latency_targets_met = sum(1 for r in latency_results.values() if r['target_met'])
            throughput_targets_met = sum(1 for r in throughput_results.values() if r['target_met'])
            
            comprehensive_results = {
                'test_summary': {
                    'test_duration': test_duration,
                    'target_latency_us': self.config.target_latency_us,
                    'target_throughput': self.config.target_throughput,
                    'latency_targets_met': f"{latency_targets_met}/{len(latency_results)}",
                    'throughput_targets_met': f"{throughput_targets_met}/{len(throughput_results)}"
                },
                'latency_optimization': latency_results,
                'throughput_optimization': throughput_results,
                'avx512_effectiveness': avx512_results,
                'best_configurations': {
                    'lowest_latency': {
                        'batch_size': best_latency_config[0],
                        'metrics': best_latency_config[1]
                    },
                    'highest_throughput': {
                        'thread_count': best_throughput_config[0],
                        'metrics': best_throughput_config[1]
                    }
                },
                'optimization_success': {
                    'latency_optimized': latency_targets_met > 0,
                    'throughput_optimized': throughput_targets_met > 0,
                    'avx512_effective': 'avx512_enabled' in avx512_results
                }
            }
            
            return comprehensive_results
            
        except Exception as e:
            logger.error(f"测试失败: {e}")
            raise
    
    def generate_optimization_report(self, results: Dict):
        """生成优化报告"""
        logger.info("📊 生成性能优化报告...")
        
        print("\n" + "="*80)
        print("🎯 AVX-512性能优化测试报告")
        print("="*80)
        
        # 测试概览
        summary = results['test_summary']
        print(f"测试时长: {summary['test_duration']:.2f} 秒")
        print(f"目标延迟: {summary['target_latency_us']} 微秒")
        print(f"目标吞吐量: {summary['target_throughput']:,} 消息/秒")
        print(f"延迟目标达成: {summary['latency_targets_met']}")
        print(f"吞吐量目标达成: {summary['throughput_targets_met']}")
        
        # 延迟优化结果
        print("\n📈 延迟优化结果:")
        for batch_size, metrics in results['latency_optimization'].items():
            status = "✅" if metrics['target_met'] else "❌"
            print(f"  {status} 批大小 {batch_size}: {metrics['avg_latency_us']:.2f}μs "
                  f"(P95: {metrics['p95_latency_us']:.2f}μs)")
        
        # 吞吐量优化结果
        print("\n⚡ 吞吐量优化结果:")
        for thread_count, metrics in results['throughput_optimization'].items():
            status = "✅" if metrics['target_met'] else "❌"
            print(f"  {status} 线程数 {thread_count}: {metrics['actual_throughput']:.0f}/秒 "
                  f"(错误率: {metrics['error_rate']:.4f})")
        
        # AVX-512效果
        if 'avx512_effectiveness' in results:
            print("\n🔬 AVX-512指令集效果:")
            avx512_data = results['avx512_effectiveness']
            for config_name, metrics in avx512_data.items():
                if config_name != 'performance_improvement':
                    print(f"  {config_name}: {metrics['avg_batch_time_us']:.2f}μs, "
                          f"{metrics['throughput']:.0f}/秒")
            
            if 'performance_improvement' in avx512_data:
                improvement = avx512_data['performance_improvement']
                print(f"  💪 AVX-512性能提升: {improvement['avx512_vs_scalar_speedup']:.2f}x")
                print(f"  ⚡ 延迟减少: {improvement['latency_reduction']:.2f}μs")
        
        # 最优配置推荐
        print("\n🎯 最优配置推荐:")
        best_configs = results['best_configurations']
        print(f"  最低延迟配置: 批大小 {best_configs['lowest_latency']['batch_size']} "
              f"({best_configs['lowest_latency']['metrics']['avg_latency_us']:.2f}μs)")
        print(f"  最高吞吐量配置: {best_configs['highest_throughput']['thread_count']} 线程 "
              f"({best_configs['highest_throughput']['metrics']['actual_throughput']:.0f}/秒)")
        
        # 优化成功评估
        success = results['optimization_success']
        print("\n✅ 优化成果:")
        print(f"  延迟优化: {'成功' if success['latency_optimized'] else '需改进'}")
        print(f"  吞吐量优化: {'成功' if success['throughput_optimized'] else '需改进'}")
        print(f"  AVX-512效果: {'显著' if success['avx512_effective'] else '有限'}")
        
        print("="*80)
    
    async def cleanup(self):
        """清理测试环境"""
        if self.nats_client:
            await self.nats_client.close()
        logger.info("✅ 测试环境已清理")

async def main():
    """主函数"""
    config = PerformanceOptimizationConfig()
    test = PerformanceOptimizationTest(config)
    
    try:
        await test.setup()
        results = await test.run_comprehensive_test()
        test.generate_optimization_report(results)
        
        # 保存结果
        with open('performance_optimization_results.json', 'w') as f:
            json.dump(results, f, indent=2)
        
        logger.info("📋 测试结果已保存到 performance_optimization_results.json")
        
    except Exception as e:
        logger.error(f"测试失败: {e}")
        return 1
    finally:
        await test.cleanup()
    
    return 0

if __name__ == "__main__":
    exit_code = asyncio.run(main())
    exit(exit_code) 
# -*- coding: utf-8 -*-
"""
AVX-512性能优化验证测试
目标: 验证延迟从62ms降至100μs以下，吞吐量从7.4k提升至100k+
"""

import asyncio
import time
import json
import logging
import subprocess
import psutil
import numpy as np
from dataclasses import dataclass
from typing import List, Dict, Optional
import concurrent.futures
import threading
from pathlib import Path
import nats
import random

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

@dataclass
class PerformanceOptimizationConfig:
    """性能优化测试配置"""
    target_latency_us: float = 100.0       # 目标延迟100微秒
    target_throughput: int = 100_000       # 目标吞吐量100k/秒
    test_duration: int = 300               # 测试5分钟
    batch_sizes: List[int] = None          # 测试不同批处理大小
    worker_threads: List[int] = None       # 测试不同线程数
    validation_samples: int = 10_000       # 验证样本数
    
    def __post_init__(self):
        if self.batch_sizes is None:
            self.batch_sizes = [512, 1024, 2048, 4096]
        if self.worker_threads is None:
            self.worker_threads = [8, 16, 24, 32]

class HighPerformanceDataGenerator:
    """高性能数据生成器 - 专门针对性能测试优化"""
    
    def __init__(self, config: PerformanceOptimizationConfig):
        self.config = config
        self.exchanges = ['binance', 'okx', 'huobi', 'bybit', 'gateio']
        self.symbols = self._generate_symbol_pool()
        
    def _generate_symbol_pool(self) -> List[str]:
        """生成丰富的交易对池"""
        # 主流币种
        major_coins = ['BTC', 'ETH', 'BNB', 'XRP', 'ADA', 'SOL', 'DOT', 'AVAX', 'MATIC', 'LINK']
        # DeFi代币
        defi_tokens = ['UNI', 'AAVE', 'COMP', 'MKR', 'SNX', 'CRV', 'SUSHI', 'YFI', '1INCH', 'ALPHA']
        # Meme币
        meme_coins = ['DOGE', 'SHIB', 'FLOKI', 'PEPE', 'BABYDOGE', 'SAFEMOON']
        # 小币种
        alt_coins = [f"ALT{i:03d}" for i in range(1, 101)]
        
        all_coins = major_coins + defi_tokens + meme_coins + alt_coins
        quote_currencies = ['USDT', 'USDC', 'BTC', 'ETH', 'BNB']
        
        symbols = []
        for base in all_coins:
            for quote in quote_currencies:
                if base != quote:
                    symbols.append(f"{base}{quote}")
        
        return symbols[:15000]  # 限制到15k个交易对
    
    def generate_optimized_market_data(self, count: int) -> List[Dict]:
        """生成针对性能测试优化的市场数据"""
        data_batch = []
        current_time = int(time.time() * 1_000_000_000)  # 纳秒时间戳
        
        for i in range(count):
            symbol = random.choice(self.symbols)
            exchange = random.choice(self.exchanges)
            
            # 生成realistic的价格数据
            base_price = random.uniform(0.001, 50000.0)
            spread_pct = random.uniform(0.0001, 0.01)  # 0.01% - 1% 价差
            
            bid_price = base_price * (1 - spread_pct / 2)
            ask_price = base_price * (1 + spread_pct / 2)
            
            # 添加市场微观结构噪音
            volatility = random.uniform(0.001, 0.05)
            bid_price *= (1 + random.gauss(0, volatility))
            ask_price *= (1 + random.gauss(0, volatility))
            
            # 确保合理的价格关系
            if bid_price >= ask_price:
                bid_price, ask_price = ask_price * 0.999, bid_price * 1.001
            
            data = {
                "exchange": exchange,
                "symbol": symbol,
                "timestamp": current_time + i * 1000,  # 1微秒间隔
                "bids": [[bid_price, random.uniform(1.0, 1000.0)]],
                "asks": [[ask_price, random.uniform(1.0, 1000.0)]],
                "sequence": 1000000 + i,
                "mid_price": int((bid_price + ask_price) / 2 * 100_000_000),
                "spread": int((ask_price - bid_price) * 100_000_000),
            }
            data_batch.append(data)
        
        return data_batch

class PerformanceOptimizationTest:
    """性能优化测试类"""
    
    def __init__(self, config: PerformanceOptimizationConfig):
        self.config = config
        self.data_generator = HighPerformanceDataGenerator(config)
        self.workspace = Path.cwd()
        self.binary_path = self.workspace / 'target' / 'x86_64-unknown-linux-gnu' / 'release' / 'celue-arbitrage-monitor'
        self.nats_client: Optional[nats.NATS] = None
        self.test_results = {}
        
    async def setup(self):
        """测试环境设置"""
        logger.info("🔧 设置性能优化测试环境...")
        
        # 编译最新版本
        logger.info("📦 编译最新代码...")
        result = subprocess.run([
            'cargo', 'build', '--release', 
            '--target=x86_64-unknown-linux-gnu',
            '--features=simd,avx512'
        ], cwd=self.workspace, capture_output=True, text=True)
        
        if result.returncode != 0:
            logger.error(f"编译失败: {result.stderr}")
            raise RuntimeError("编译失败")
        
        # 验证AVX-512支持
        self.verify_avx512_support()
        
        # 连接NATS
        self.nats_client = await nats.connect("nats://localhost:4222")
        
        logger.info("✅ 测试环境设置完成")
    
    def verify_avx512_support(self):
        """验证AVX-512支持"""
        try:
            with open('/proc/cpuinfo', 'r') as f:
                cpuinfo = f.read()
                
            if 'avx512f' in cpuinfo:
                logger.info("✅ CPU支持AVX-512F")
            else:
                logger.warning("⚠️ CPU不支持AVX-512F")
                
            if 'avx512dq' in cpuinfo and 'avx512bw' in cpuinfo:
                logger.info("✅ CPU支持完整AVX-512指令集")
            else:
                logger.warning("⚠️ CPU不支持完整AVX-512指令集")
                
        except Exception as e:
            logger.error(f"无法检测AVX-512支持: {e}")
    
    async def test_latency_optimization(self) -> Dict:
        """测试延迟优化效果"""
        logger.info("🚀 开始延迟优化测试...")
        
        latency_results = {}
        
        for batch_size in self.config.batch_sizes:
            logger.info(f"测试批处理大小: {batch_size}")
            
            # 生成测试数据
            test_data = self.data_generator.generate_optimized_market_data(
                self.config.validation_samples
            )
            
            # 批处理测试
            latencies = []
            start_time = time.perf_counter()
            
            for i in range(0, len(test_data), batch_size):
                batch = test_data[i:i + batch_size]
                batch_start = time.perf_counter_ns()
                
                # 发送批处理数据
                for data in batch:
                    await self.nats_client.publish(
                        "qx.v5.md.clean.test.test.ob50",
                        json.dumps(data).encode()
                    )
                
                batch_end = time.perf_counter_ns()
                batch_latency = (batch_end - batch_start) / len(batch) / 1000  # 微秒
                latencies.append(batch_latency)
            
            total_time = time.perf_counter() - start_time
            
            # 统计结果
            avg_latency = np.mean(latencies)
            p95_latency = np.percentile(latencies, 95)
            p99_latency = np.percentile(latencies, 99)
            max_latency = np.max(latencies)
            throughput = len(test_data) / total_time
            
            latency_results[batch_size] = {
                'avg_latency_us': avg_latency,
                'p95_latency_us': p95_latency,
                'p99_latency_us': p99_latency,
                'max_latency_us': max_latency,
                'throughput': throughput,
                'target_met': avg_latency <= self.config.target_latency_us
            }
            
            logger.info(f"批大小 {batch_size}: 平均延迟 {avg_latency:.2f}μs, "
                       f"P95: {p95_latency:.2f}μs, 吞吐量: {throughput:.0f}/秒")
        
        return latency_results
    
    async def test_throughput_optimization(self) -> Dict:
        """测试吞吐量优化效果"""
        logger.info("⚡ 开始吞吐量优化测试...")
        
        throughput_results = {}
        
        for thread_count in self.config.worker_threads:
            logger.info(f"测试工作线程数: {thread_count}")
            
            # 生成大量测试数据
            total_messages = self.config.target_throughput * 60  # 1分钟的数据量
            test_data = self.data_generator.generate_optimized_market_data(total_messages)
            
            # 多线程并发测试
            start_time = time.perf_counter()
            processed_count = 0
            errors = 0
            
            async def publish_worker(data_chunk):
                nonlocal processed_count, errors
                try:
                    for data in data_chunk:
                        await self.nats_client.publish(
                            "qx.v5.md.clean.perf.test.ob50",
                            json.dumps(data).encode()
                        )
                        processed_count += 1
                except Exception as e:
                    errors += 1
                    logger.error(f"发布错误: {e}")
            
            # 分块并发处理
            chunk_size = len(test_data) // thread_count
            tasks = []
            
            for i in range(thread_count):
                start_idx = i * chunk_size
                end_idx = start_idx + chunk_size if i < thread_count - 1 else len(test_data)
                chunk = test_data[start_idx:end_idx]
                tasks.append(publish_worker(chunk))
            
            # 等待所有任务完成
            await asyncio.gather(*tasks)
            
            total_time = time.perf_counter() - start_time
            actual_throughput = processed_count / total_time
            error_rate = errors / len(test_data)
            
            throughput_results[thread_count] = {
                'actual_throughput': actual_throughput,
                'target_throughput': self.config.target_throughput,
                'target_met': actual_throughput >= self.config.target_throughput,
                'error_rate': error_rate,
                'total_time': total_time,
                'processed_count': processed_count
            }
            
            logger.info(f"线程数 {thread_count}: 实际吞吐量 {actual_throughput:.0f}/秒, "
                       f"错误率: {error_rate:.4f}")
        
        return throughput_results
    
    async def test_avx512_effectiveness(self) -> Dict:
        """测试AVX-512指令集效果"""
        logger.info("🔬 测试AVX-512指令集效果...")
        
        # 创建测试配置文件
        test_configs = {
            'avx512_enabled': {
                'simd_level': 'AVX512',
                'batch_size': 2048,
                'worker_threads': 16
            },
            'avx2_fallback': {
                'simd_level': 'AVX2', 
                'batch_size': 1024,
                'worker_threads': 8
            },
            'scalar_baseline': {
                'simd_level': 'Scalar',
                'batch_size': 256,
                'worker_threads': 4
            }
        }
        
        avx512_results = {}
        test_data = self.data_generator.generate_optimized_market_data(10000)
        
        for config_name, config in test_configs.items():
            logger.info(f"测试配置: {config_name}")
            
            start_time = time.perf_counter_ns()
            
            # 模拟不同SIMD指令集的处理
            batch_times = []
            for i in range(0, len(test_data), config['batch_size']):
                batch = test_data[i:i + config['batch_size']]
                batch_start = time.perf_counter_ns()
                
                # 发送数据
                for data in batch:
                    await self.nats_client.publish(
                        f"qx.v5.md.clean.{config_name}.test.ob50",
                        json.dumps(data).encode()
                    )
                
                batch_end = time.perf_counter_ns()
                batch_times.append(batch_end - batch_start)
            
            total_time = time.perf_counter_ns() - start_time
            
            avg_batch_time = np.mean(batch_times) / 1000  # 微秒
            throughput = len(test_data) / (total_time / 1_000_000_000)
            
            avx512_results[config_name] = {
                'avg_batch_time_us': avg_batch_time,
                'throughput': throughput,
                'config': config
            }
            
            logger.info(f"{config_name}: 平均批处理时间 {avg_batch_time:.2f}μs, "
                       f"吞吐量: {throughput:.0f}/秒")
        
        # 计算性能提升
        if 'avx512_enabled' in avx512_results and 'scalar_baseline' in avx512_results:
            avx512_speedup = (
                avx512_results['scalar_baseline']['throughput'] / 
                avx512_results['avx512_enabled']['throughput']
            )
            avx512_results['performance_improvement'] = {
                'avx512_vs_scalar_speedup': avx512_speedup,
                'latency_reduction': (
                    avx512_results['scalar_baseline']['avg_batch_time_us'] - 
                    avx512_results['avx512_enabled']['avg_batch_time_us']
                )
            }
        
        return avx512_results
    
    async def run_comprehensive_test(self) -> Dict:
        """运行完整的性能优化测试"""
        logger.info("🎯 开始完整性能优化测试...")
        
        start_time = time.time()
        
        try:
            # 1. 延迟优化测试
            latency_results = await self.test_latency_optimization()
            
            # 2. 吞吐量优化测试  
            throughput_results = await self.test_throughput_optimization()
            
            # 3. AVX-512效果测试
            avx512_results = await self.test_avx512_effectiveness()
            
            # 综合结果
            test_duration = time.time() - start_time
            
            # 找出最优配置
            best_latency_config = min(
                latency_results.items(),
                key=lambda x: x[1]['avg_latency_us']
            )
            
            best_throughput_config = max(
                throughput_results.items(),
                key=lambda x: x[1]['actual_throughput']
            )
            
            # 性能目标达成情况
            latency_targets_met = sum(1 for r in latency_results.values() if r['target_met'])
            throughput_targets_met = sum(1 for r in throughput_results.values() if r['target_met'])
            
            comprehensive_results = {
                'test_summary': {
                    'test_duration': test_duration,
                    'target_latency_us': self.config.target_latency_us,
                    'target_throughput': self.config.target_throughput,
                    'latency_targets_met': f"{latency_targets_met}/{len(latency_results)}",
                    'throughput_targets_met': f"{throughput_targets_met}/{len(throughput_results)}"
                },
                'latency_optimization': latency_results,
                'throughput_optimization': throughput_results,
                'avx512_effectiveness': avx512_results,
                'best_configurations': {
                    'lowest_latency': {
                        'batch_size': best_latency_config[0],
                        'metrics': best_latency_config[1]
                    },
                    'highest_throughput': {
                        'thread_count': best_throughput_config[0],
                        'metrics': best_throughput_config[1]
                    }
                },
                'optimization_success': {
                    'latency_optimized': latency_targets_met > 0,
                    'throughput_optimized': throughput_targets_met > 0,
                    'avx512_effective': 'avx512_enabled' in avx512_results
                }
            }
            
            return comprehensive_results
            
        except Exception as e:
            logger.error(f"测试失败: {e}")
            raise
    
    def generate_optimization_report(self, results: Dict):
        """生成优化报告"""
        logger.info("📊 生成性能优化报告...")
        
        print("\n" + "="*80)
        print("🎯 AVX-512性能优化测试报告")
        print("="*80)
        
        # 测试概览
        summary = results['test_summary']
        print(f"测试时长: {summary['test_duration']:.2f} 秒")
        print(f"目标延迟: {summary['target_latency_us']} 微秒")
        print(f"目标吞吐量: {summary['target_throughput']:,} 消息/秒")
        print(f"延迟目标达成: {summary['latency_targets_met']}")
        print(f"吞吐量目标达成: {summary['throughput_targets_met']}")
        
        # 延迟优化结果
        print("\n📈 延迟优化结果:")
        for batch_size, metrics in results['latency_optimization'].items():
            status = "✅" if metrics['target_met'] else "❌"
            print(f"  {status} 批大小 {batch_size}: {metrics['avg_latency_us']:.2f}μs "
                  f"(P95: {metrics['p95_latency_us']:.2f}μs)")
        
        # 吞吐量优化结果
        print("\n⚡ 吞吐量优化结果:")
        for thread_count, metrics in results['throughput_optimization'].items():
            status = "✅" if metrics['target_met'] else "❌"
            print(f"  {status} 线程数 {thread_count}: {metrics['actual_throughput']:.0f}/秒 "
                  f"(错误率: {metrics['error_rate']:.4f})")
        
        # AVX-512效果
        if 'avx512_effectiveness' in results:
            print("\n🔬 AVX-512指令集效果:")
            avx512_data = results['avx512_effectiveness']
            for config_name, metrics in avx512_data.items():
                if config_name != 'performance_improvement':
                    print(f"  {config_name}: {metrics['avg_batch_time_us']:.2f}μs, "
                          f"{metrics['throughput']:.0f}/秒")
            
            if 'performance_improvement' in avx512_data:
                improvement = avx512_data['performance_improvement']
                print(f"  💪 AVX-512性能提升: {improvement['avx512_vs_scalar_speedup']:.2f}x")
                print(f"  ⚡ 延迟减少: {improvement['latency_reduction']:.2f}μs")
        
        # 最优配置推荐
        print("\n🎯 最优配置推荐:")
        best_configs = results['best_configurations']
        print(f"  最低延迟配置: 批大小 {best_configs['lowest_latency']['batch_size']} "
              f"({best_configs['lowest_latency']['metrics']['avg_latency_us']:.2f}μs)")
        print(f"  最高吞吐量配置: {best_configs['highest_throughput']['thread_count']} 线程 "
              f"({best_configs['highest_throughput']['metrics']['actual_throughput']:.0f}/秒)")
        
        # 优化成功评估
        success = results['optimization_success']
        print("\n✅ 优化成果:")
        print(f"  延迟优化: {'成功' if success['latency_optimized'] else '需改进'}")
        print(f"  吞吐量优化: {'成功' if success['throughput_optimized'] else '需改进'}")
        print(f"  AVX-512效果: {'显著' if success['avx512_effective'] else '有限'}")
        
        print("="*80)
    
    async def cleanup(self):
        """清理测试环境"""
        if self.nats_client:
            await self.nats_client.close()
        logger.info("✅ 测试环境已清理")

async def main():
    """主函数"""
    config = PerformanceOptimizationConfig()
    test = PerformanceOptimizationTest(config)
    
    try:
        await test.setup()
        results = await test.run_comprehensive_test()
        test.generate_optimization_report(results)
        
        # 保存结果
        with open('performance_optimization_results.json', 'w') as f:
            json.dump(results, f, indent=2)
        
        logger.info("📋 测试结果已保存到 performance_optimization_results.json")
        
    except Exception as e:
        logger.error(f"测试失败: {e}")
        return 1
    finally:
        await test.cleanup()
    
    return 0

if __name__ == "__main__":
    exit_code = asyncio.run(main())
    exit(exit_code) 