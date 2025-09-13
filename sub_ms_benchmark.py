#!/usr/bin/env python3
"""
<1ms 延迟基准测试工具
测试不同优化策略的延迟表现
"""
import asyncio
import aiohttp
import time
import numpy as np
import matplotlib.pyplot as plt
from dataclasses import dataclass
from typing import List, Dict, Tuple
from concurrent.futures import ThreadPoolExecutor
import uvloop
import struct
import socket
import threading

@dataclass
class LatencyResult:
    strategy: str
    latencies: List[float]
    success_rate: float
    under_1ms_rate: float

class SubMillisecondBenchmark:
    def __init__(self):
        self.results: List[LatencyResult] = []
        
    async def test_baseline(self, iterations: int = 1000) -> LatencyResult:
        """基准测试：当前实现"""
        latencies = []
        successful = 0
        
        async with aiohttp.ClientSession() as session:
            for i in range(iterations):
                start = time.perf_counter()
                
                try:
                    async with session.post(
                        'http://127.0.0.1:8881/api/v3/order',
                        json={
                            'symbol': 'BTCUSDT',
                            'side': 'BUY',
                            'type': 'LIMIT',
                            'quantity': 0.001,
                            'price': 50000 + i,
                            'timestamp': int(time.time() * 1000)
                        }
                    ) as response:
                        await response.json()
                        successful += 1
                except:
                    pass
                
                latency = (time.perf_counter() - start) * 1000  # ms
                latencies.append(latency)
                
                if i % 100 == 0:
                    print(f"Baseline: {i}/{iterations}")
                
                await asyncio.sleep(0.001)  # 1ms间隔
        
        under_1ms = sum(1 for l in latencies if l < 1.0)
        
        return LatencyResult(
            strategy="Baseline (HTTP/JSON)",
            latencies=latencies,
            success_rate=successful / iterations,
            under_1ms_rate=under_1ms / len(latencies)
        )
    
    async def test_binary_protocol(self, iterations: int = 1000) -> LatencyResult:
        """二进制协议测试"""
        latencies = []
        successful = 0
        
        # 连接到模拟交易所
        reader, writer = await asyncio.open_connection('127.0.0.1', 8881)
        
        # 预编译的订单结构
        order_struct = struct.Struct('<12s B B Q Q Q I I')
        
        for i in range(iterations):
            start = time.perf_counter()
            
            try:
                # 二进制订单数据
                symbol = b'BTCUSDT\x00\x00\x00\x00\x00'
                order_data = order_struct.pack(
                    symbol,          # symbol[12]
                    0,               # side (BUY)
                    0,               # type (LIMIT)
                    100000,          # quantity (0.001 * 1e8)
                    (50000 + i) * 100000000,  # price
                    int(time.time() * 1e9),   # timestamp_ns
                    i,               # nonce
                    0                # checksum
                )
                
                writer.write(order_data)
                await writer.drain()
                
                # 等待响应（简化处理）
                response = await reader.read(1024)
                if response:
                    successful += 1
                    
            except:
                pass
            
            latency = (time.perf_counter() - start) * 1000
            latencies.append(latency)
            
            if i % 100 == 0:
                print(f"Binary: {i}/{iterations}")
        
        writer.close()
        await writer.wait_closed()
        
        under_1ms = sum(1 for l in latencies if l < 1.0)
        
        return LatencyResult(
            strategy="Binary Protocol",
            latencies=latencies,
            success_rate=successful / iterations,
            under_1ms_rate=under_1ms / len(latencies)
        )
    
    def test_raw_socket(self, iterations: int = 1000) -> LatencyResult:
        """原始套接字测试（同步）"""
        latencies = []
        successful = 0
        
        # 创建原始套接字
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        
        try:
            sock.connect(('127.0.0.1', 8881))
            
            order_data = b'BINARY_ORDER_DATA_32_BYTES_FIXED'
            
            for i in range(iterations):
                start = time.perf_counter()
                
                try:
                    # 发送订单
                    sock.send(order_data)
                    
                    # 接收响应
                    response = sock.recv(1024)
                    if response:
                        successful += 1
                        
                except:
                    pass
                
                latency = (time.perf_counter() - start) * 1000
                latencies.append(latency)
                
                if i % 100 == 0:
                    print(f"Raw Socket: {i}/{iterations}")
                
                time.sleep(0.001)  # 1ms间隔
                
        finally:
            sock.close()
        
        under_1ms = sum(1 for l in latencies if l < 1.0)
        
        return LatencyResult(
            strategy="Raw Socket",
            latencies=latencies,
            success_rate=successful / iterations,
            under_1ms_rate=under_1ms / len(latencies)
        )
    
    async def test_connection_pool(self, iterations: int = 1000) -> LatencyResult:
        """连接池优化测试"""
        latencies = []
        successful = 0
        
        # 预建立多个连接
        pool_size = 8
        connections = []
        
        for _ in range(pool_size):
            try:
                reader, writer = await asyncio.open_connection('127.0.0.1', 8881)
                connections.append((reader, writer))
            except:
                pass
        
        if not connections:
            return LatencyResult("Connection Pool", [], 0, 0)
        
        connection_idx = 0
        
        for i in range(iterations):
            start = time.perf_counter()
            
            try:
                # 轮询使用连接
                reader, writer = connections[connection_idx % len(connections)]
                connection_idx += 1
                
                # 发送订单
                order_json = f'{{"symbol":"BTCUSDT","side":"BUY","price":{50000+i}}}\n'
                writer.write(order_json.encode())
                await writer.drain()
                
                successful += 1
                
            except:
                pass
            
            latency = (time.perf_counter() - start) * 1000
            latencies.append(latency)
            
            if i % 100 == 0:
                print(f"Pool: {i}/{iterations}")
        
        # 清理连接
        for reader, writer in connections:
            writer.close()
            await writer.wait_closed()
        
        under_1ms = sum(1 for l in latencies if l < 1.0)
        
        return LatencyResult(
            strategy="Connection Pool",
            latencies=latencies,
            success_rate=successful / iterations,
            under_1ms_rate=under_1ms / len(latencies)
        )
    
    def test_threading_optimization(self, iterations: int = 1000) -> LatencyResult:
        """多线程优化测试"""
        latencies = []
        successful = 0
        
        def send_order_sync(order_id):
            try:
                sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
                sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
                
                start = time.perf_counter()
                
                sock.connect(('127.0.0.1', 8881))
                
                order_data = f'ORDER_{order_id}'.encode()
                sock.send(order_data)
                
                response = sock.recv(1024)
                
                latency = (time.perf_counter() - start) * 1000
                sock.close()
                
                return latency, len(response) > 0
                
            except:
                return 999.0, False
        
        with ThreadPoolExecutor(max_workers=4) as executor:
            futures = [executor.submit(send_order_sync, i) for i in range(iterations)]
            
            for i, future in enumerate(futures):
                latency, success = future.result()
                latencies.append(latency)
                if success:
                    successful += 1
                
                if i % 100 == 0:
                    print(f"Threading: {i}/{iterations}")
        
        under_1ms = sum(1 for l in latencies if l < 1.0)
        
        return LatencyResult(
            strategy="Multi-threading",
            latencies=latencies,
            success_rate=successful / iterations,
            under_1ms_rate=under_1ms / len(latencies)
        )
    
    async def run_all_tests(self, iterations: int = 500):
        """运行所有测试"""
        print("=" * 60)
        print("Sub-Millisecond Latency Benchmark")
        print("=" * 60)
        
        # 运行各种测试策略
        tests = [
            ("Baseline", self.test_baseline(iterations)),
            ("Binary Protocol", self.test_binary_protocol(iterations)),
            ("Connection Pool", self.test_connection_pool(iterations))
        ]
        
        for name, test_coro in tests:
            print(f"\nRunning {name} test...")
            result = await test_coro
            self.results.append(result)
        
        # 同步测试
        print(f"\nRunning Raw Socket test...")
        raw_result = self.test_raw_socket(iterations)
        self.results.append(raw_result)
        
        print(f"\nRunning Threading test...")
        thread_result = self.test_threading_optimization(iterations)
        self.results.append(thread_result)
        
        self.generate_report()
        self.plot_results()
    
    def generate_report(self):
        """生成测试报告"""
        print("\n" + "=" * 80)
        print("SUB-MILLISECOND LATENCY OPTIMIZATION REPORT")
        print("=" * 80)
        
        print(f"{'Strategy':<20} {'Samples':<8} {'Min':<8} {'Avg':<8} {'P95':<8} {'P99':<8} {'<1ms%':<8} {'Success%':<10}")
        print("-" * 80)
        
        for result in self.results:
            if result.latencies:
                latencies = np.array(result.latencies)
                stats = {
                    'min': np.min(latencies),
                    'avg': np.mean(latencies),
                    'p95': np.percentile(latencies, 95),
                    'p99': np.percentile(latencies, 99)
                }
                
                print(f"{result.strategy:<20} "
                      f"{len(latencies):<8} "
                      f"{stats['min']:<8.3f} "
                      f"{stats['avg']:<8.3f} "
                      f"{stats['p95']:<8.3f} "
                      f"{stats['p99']:<8.3f} "
                      f"{result.under_1ms_rate*100:<8.1f} "
                      f"{result.success_rate*100:<10.1f}")
        
        print("-" * 80)
        
        # 找出最佳策略
        best_strategy = max(self.results, key=lambda r: r.under_1ms_rate)
        
        print(f"\n🏆 Best Strategy: {best_strategy.strategy}")
        print(f"   <1ms Rate: {best_strategy.under_1ms_rate*100:.1f}%")
        
        if best_strategy.under_1ms_rate > 0.5:
            print("✅ SUCCESS: >50% orders under 1ms!")
        elif best_strategy.under_1ms_rate > 0.1:
            print("🔶 PARTIAL: >10% orders under 1ms")
        else:
            print("❌ Target not achieved: <10% orders under 1ms")
    
    def plot_results(self):
        """绘制结果图表"""
        if not self.results:
            return
        
        fig, ((ax1, ax2), (ax3, ax4)) = plt.subplots(2, 2, figsize=(15, 10))
        
        # 延迟分布直方图
        for result in self.results:
            if result.latencies:
                # 过滤极值
                filtered = [l for l in result.latencies if l < 50]
                ax1.hist(filtered, bins=50, alpha=0.7, label=result.strategy, density=True)
        
        ax1.axvline(x=1.0, color='red', linestyle='--', label='1ms Target')
        ax1.set_xlabel('Latency (ms)')
        ax1.set_ylabel('Density')
        ax1.set_title('Latency Distribution')
        ax1.legend()
        ax1.grid(True, alpha=0.3)
        
        # <1ms 成功率对比
        strategies = [r.strategy for r in self.results]
        under_1ms_rates = [r.under_1ms_rate * 100 for r in self.results]
        
        bars = ax2.bar(strategies, under_1ms_rates)
        ax2.axhline(y=50, color='red', linestyle='--', label='50% Target')
        ax2.set_ylabel('<1ms Rate (%)')
        ax2.set_title('Sub-1ms Success Rate')
        ax2.tick_params(axis='x', rotation=45)
        ax2.legend()
        
        # 为每个柱子添加数值标签
        for bar, rate in zip(bars, under_1ms_rates):
            ax2.text(bar.get_x() + bar.get_width()/2, bar.get_height() + 1,
                    f'{rate:.1f}%', ha='center', va='bottom')
        
        # P95延迟对比
        p95_latencies = []
        for result in self.results:
            if result.latencies:
                p95_latencies.append(np.percentile(result.latencies, 95))
            else:
                p95_latencies.append(0)
        
        ax3.bar(strategies, p95_latencies)
        ax3.axhline(y=1.0, color='red', linestyle='--', label='1ms Target')
        ax3.set_ylabel('P95 Latency (ms)')
        ax3.set_title('P95 Latency Comparison')
        ax3.tick_params(axis='x', rotation=45)
        ax3.legend()
        
        # 时间序列图（取第一个结果）
        if self.results and self.results[0].latencies:
            sample_data = self.results[0].latencies[:200]  # 前200个样本
            ax4.plot(sample_data, alpha=0.7)
            ax4.axhline(y=1.0, color='red', linestyle='--', label='1ms Target')
            ax4.set_xlabel('Order Sequence')
            ax4.set_ylabel('Latency (ms)')
            ax4.set_title(f'{self.results[0].strategy} - Time Series')
            ax4.legend()
            ax4.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig('sub_ms_benchmark_results.png', dpi=150, bbox_inches='tight')
        plt.show()
        
        print(f"\n📊 Charts saved to: sub_ms_benchmark_results.png")

async def main():
    # 使用 uvloop 优化异步性能
    asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())
    
    benchmark = SubMillisecondBenchmark()
    await benchmark.run_all_tests(iterations=200)  # 减少迭代次数用于演示

if __name__ == "__main__":
    asyncio.run(main())