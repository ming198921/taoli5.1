#!/usr/bin/env python3
"""
AWS云服务器优化版本的超低延迟订单发送客户端
目标: 在AWS环境下实现<1ms延迟
"""

import asyncio
import aiohttp
import time
import orjson
import psutil
import os
import gc
import socket
import dns.resolver
from typing import Dict, List, Optional
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
import uvloop
import weakref

# 设置uvloop作为默认事件循环
asyncio.set_event_loop_policy(uvloop.EventLoopPolicy())

@dataclass
class OrderResult:
    latency_ms: float
    success: bool
    error: Optional[str] = None

class AWSProcessOptimizer:
    """AWS环境下的进程优化器"""
    
    def __init__(self):
        self.setup_process_optimization()
        self.setup_memory_optimization()
        
    def setup_process_optimization(self):
        """进程优化设置"""
        try:
            # 设置最高优先级（在AWS允许范围内）
            os.nice(-10)  # AWS上通常最多只能到-10
            process = psutil.Process()
            process.nice(-10)
            
            # 尝试设置CPU亲和性（绑定到特定核心）
            cpu_count = psutil.cpu_count()
            if cpu_count >= 2:
                # 绑定到第二个CPU核心，避开系统进程
                process.cpu_affinity([1])
                print(f"✅ 进程绑定到CPU核心1，总核心数: {cpu_count}")
            
            print("✅ 进程优化完成")
            
        except PermissionError:
            print("⚠️ 权限不足，使用默认优先级")
        except Exception as e:
            print(f"⚠️ 进程优化失败: {e}")
    
    def setup_memory_optimization(self):
        """内存优化"""
        # 在性能关键时期禁用垃圾回收
        gc.set_threshold(0)  # 禁用自动垃圾回收
        print("✅ 垃圾回收优化完成")

class DNSOptimizer:
    """DNS解析优化器"""
    
    def __init__(self):
        self.dns_cache: Dict[str, str] = {}
        self.setup_dns_cache()
    
    def setup_dns_cache(self):
        """预解析并缓存交易所IP"""
        exchanges = {
            'api.binance.com': 'api.binance.com',
            'api.huobi.pro': 'api.huobi.pro', 
            'www.okx.com': 'www.okx.com'
        }
        
        print("正在预解析DNS...")
        for name, domain in exchanges.items():
            try:
                ip = socket.gethostbyname(domain)
                self.dns_cache[domain] = ip
                print(f"✅ {domain} -> {ip}")
            except Exception as e:
                print(f"❌ DNS解析失败 {domain}: {e}")
    
    def get_optimized_url(self, domain: str, path: str) -> str:
        """获取优化后的URL（直接使用IP）"""
        if domain in self.dns_cache:
            ip = self.dns_cache[domain]
            return f"http://{ip}{path}"
        return f"http://{domain}{path}"

class FastJSONProcessor:
    """极速JSON处理器"""
    
    def __init__(self):
        # 预分配订单对象池，避免动态创建
        self.order_pool = []
        for _ in range(1000):
            self.order_pool.append({
                's': '',      # symbol
                'S': '',      # side  
                'q': 0,       # quantity
                'p': 0,       # price
                't': 0        # timestamp
            })
        self.pool_index = 0
    
    def get_reusable_order(self) -> Dict:
        """获取可复用的订单对象"""
        order = self.order_pool[self.pool_index]
        self.pool_index = (self.pool_index + 1) % len(self.order_pool)
        return order
    
    def serialize_order_fast(self, symbol: str, side: str, 
                           quantity: float, price: float) -> bytes:
        """超快序列化（使用orjson）"""
        order = self.get_reusable_order()
        order['s'] = symbol
        order['S'] = side[0]  # 只取首字母: B/S
        order['q'] = quantity
        order['p'] = price
        order['t'] = int(time.time() * 1000)
        
        # orjson比标准json快2-3倍
        return orjson.dumps(order)
    
    def serialize_minimal_string(self, symbol: str, side: str,
                                quantity: float, price: float) -> str:
        """极简字符串拼接（最快方法）"""
        t = int(time.time() * 1000)
        s = side[0]  # B/S
        # 手工拼接，避免模板和格式化开销
        return f'{{"s":"{symbol}","S":"{s}","q":{quantity},"p":{price},"t":{t}}}'

class OptimizedHTTPClient:
    """AWS优化的HTTP客户端"""
    
    def __init__(self):
        self.dns_optimizer = DNSOptimizer()
        self.json_processor = FastJSONProcessor()
        
        # 极限优化的连接器
        self.connector = aiohttp.TCPConnector(
            limit=200,                    # 大幅增加连接池
            limit_per_host=64,           # 每个主机更多连接
            ttl_dns_cache=3600,          # DNS缓存1小时
            use_dns_cache=True,
            keepalive_timeout=600,       # 保持连接10分钟
            enable_cleanup_closed=True,
            # tcp_nodelay=True,          # 某些版本不支持
            # tcp_keepalive=True,        # 某些版本不支持
        )
        
        # 激进的超时设置
        self.timeout = aiohttp.ClientTimeout(
            total=0.3,        # 总超时300ms
            connect=0.05,     # 连接超时50ms
            sock_read=0.15,   # 读取超时150ms  
        )
        
        # 会话配置
        self.session = aiohttp.ClientSession(
            connector=self.connector,
            timeout=self.timeout,
            skip_auto_headers=[        # 减少HTTP头开销
                'User-Agent',
                'Accept-Encoding'
            ],
            headers={
                'Connection': 'keep-alive',
                'Content-Type': 'application/json',
                'Accept': 'application/json'
            },
            json_serialize=orjson.dumps  # 使用orjson序列化
        )
        
        print("✅ 优化HTTP客户端初始化完成")
    
    async def send_order_http(self, exchange_url: str, symbol: str, 
                             side: str, quantity: float, price: float) -> OrderResult:
        """发送HTTP订单（极限优化版）"""
        start_time = time.perf_counter()
        
        try:
            # 使用预序列化的JSON字符串
            json_str = self.json_processor.serialize_minimal_string(
                symbol, side, quantity, price
            )
            
            async with self.session.post(
                exchange_url,
                data=json_str.encode(),  # 直接发送字节
                compress=False,          # 禁用压缩节省CPU
            ) as response:
                # 快速读取响应
                await response.json()
                
            latency = (time.perf_counter() - start_time) * 1000
            return OrderResult(latency_ms=latency, success=True)
            
        except asyncio.TimeoutError:
            latency = (time.perf_counter() - start_time) * 1000
            return OrderResult(latency_ms=latency, success=False, error="Timeout")
        except Exception as e:
            latency = (time.perf_counter() - start_time) * 1000
            return OrderResult(latency_ms=latency, success=False, error=str(e))
    
    async def close(self):
        """关闭客户端"""
        await self.session.close()

class WebSocketOptimizedClient:
    """WebSocket长连接优化客户端"""
    
    def __init__(self):
        self.connections: Dict[str, List] = {}
        self.json_processor = FastJSONProcessor()
        
    async def establish_connections(self):
        """建立优化的WebSocket连接池"""
        # 模拟交易所WebSocket端点
        exchanges = {
            'binance': 'ws://127.0.0.1:8881/ws/btcusdt@depth',
            'huobi': 'ws://127.0.0.1:8882/ws',
            'okex': 'ws://127.0.0.1:8883/ws/v5/public'
        }
        
        for exchange, url in exchanges.items():
            connections = []
            try:
                # 为每个交易所建立多个WebSocket连接
                for i in range(4):
                    ws = await websockets.connect(
                        url,
                        ping_interval=30,
                        ping_timeout=10,
                        close_timeout=5,
                        max_size=2**15,      # 32KB缓冲区
                        compression=None,    # 禁用压缩
                        max_queue=32        # 限制队列大小
                    )
                    connections.append(ws)
                
                self.connections[exchange] = connections
                print(f"✅ {exchange}: {len(connections)} WebSocket连接建立")
                
            except Exception as e:
                print(f"❌ {exchange} WebSocket连接失败: {e}")
    
    async def send_order_ws(self, exchange: str, symbol: str,
                           side: str, quantity: float, price: float) -> OrderResult:
        """通过WebSocket发送订单"""
        if exchange not in self.connections or not self.connections[exchange]:
            return OrderResult(latency_ms=999, success=False, error="No connection")
        
        start_time = time.perf_counter()
        
        try:
            # 轮询选择连接
            import random
            conn = random.choice(self.connections[exchange])
            
            # 构造WebSocket消息
            message = {
                "id": int(time.time() * 1000000),  # 微秒级ID
                "method": "order.place",
                "params": {
                    "symbol": symbol,
                    "side": side[0],
                    "quantity": quantity,
                    "price": price,
                    "timestamp": int(time.time() * 1000)
                }
            }
            
            # 快速序列化并发送
            json_str = orjson.dumps(message).decode()
            await conn.send(json_str)
            
            # 接收响应
            response = await asyncio.wait_for(conn.recv(), timeout=0.2)
            
            latency = (time.perf_counter() - start_time) * 1000
            return OrderResult(latency_ms=latency, success=True)
            
        except Exception as e:
            latency = (time.perf_counter() - start_time) * 1000
            return OrderResult(latency_ms=latency, success=False, error=str(e))

class AWSUltraLowLatencySystem:
    """AWS云上的超低延迟交易系统"""
    
    def __init__(self):
        # 初始化组件
        self.process_optimizer = AWSProcessOptimizer()
        self.http_client = OptimizedHTTPClient()
        self.ws_client = WebSocketOptimizedClient()
        
        # 线程池用于并行处理
        self.thread_pool = ThreadPoolExecutor(max_workers=4)
        
        # 统计数据
        self.results: List[OrderResult] = []
        
        print("✅ AWS超低延迟系统初始化完成")
    
    async def setup(self):
        """系统设置"""
        print("正在设置WebSocket连接...")
        await self.ws_client.establish_connections()
        print("✅ 系统设置完成")
    
    async def benchmark_http(self, iterations: int = 1000) -> List[OrderResult]:
        """HTTP基准测试"""
        print(f"开始HTTP基准测试 ({iterations} 次)...")
        results = []
        
        exchange_url = 'http://127.0.0.1:8881/api/v3/order'
        
        for i in range(iterations):
            result = await self.http_client.send_order_http(
                exchange_url, 
                'BTCUSDT', 
                'BUY',
                0.001,
                50000 + i
            )
            results.append(result)
            
            if i % 200 == 0:
                avg_latency = sum(r.latency_ms for r in results[-200:]) / min(200, len(results))
                print(f"  进度: {i}/{iterations}, 近期平均延迟: {avg_latency:.3f}ms")
            
            # 短暂休息避免过载
            await asyncio.sleep(0.002)
        
        return results
    
    async def benchmark_parallel(self, iterations: int = 500) -> List[OrderResult]:
        """并行基准测试"""
        print(f"开始并行基准测试 ({iterations} 次)...")
        
        # 创建并行任务
        tasks = []
        for i in range(iterations):
            task = asyncio.create_task(
                self.http_client.send_order_http(
                    'http://127.0.0.1:8881/api/v3/order',
                    'BTCUSDT',
                    'BUY' if i % 2 == 0 else 'SELL',
                    0.001,
                    50000 + i
                )
            )
            tasks.append(task)
        
        # 等待所有任务完成
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # 过滤有效结果
        valid_results = [r for r in results if isinstance(r, OrderResult)]
        return valid_results
    
    def analyze_results(self, results: List[OrderResult]) -> Dict:
        """分析测试结果"""
        if not results:
            return {}
        
        successful = [r for r in results if r.success]
        latencies = [r.latency_ms for r in successful]
        
        if not latencies:
            return {'success_rate': 0}
        
        import numpy as np
        
        # 过滤异常值
        filtered = [l for l in latencies if l < 50]  # 过滤>50ms异常值
        
        under_1ms = sum(1 for l in filtered if l < 1.0)
        under_2ms = sum(1 for l in filtered if l < 2.0)
        under_5ms = sum(1 for l in filtered if l < 5.0)
        
        return {
            'total_samples': len(results),
            'successful': len(successful),
            'success_rate': len(successful) / len(results) * 100,
            'min_ms': min(filtered),
            'max_ms': max(filtered),
            'mean_ms': np.mean(filtered),
            'median_ms': np.median(filtered),
            'p95_ms': np.percentile(filtered, 95),
            'p99_ms': np.percentile(filtered, 99),
            'std_ms': np.std(filtered),
            'under_1ms_count': under_1ms,
            'under_1ms_rate': under_1ms / len(filtered) * 100,
            'under_2ms_rate': under_2ms / len(filtered) * 100,
            'under_5ms_rate': under_5ms / len(filtered) * 100,
        }
    
    def print_optimization_results(self, stats: Dict):
        """打印优化结果"""
        print("\n" + "="*80)
        print("AWS云服务器超低延迟优化结果")
        print("="*80)
        
        if not stats:
            print("❌ 测试失败，无有效数据")
            return
        
        print(f"总样本数: {stats['total_samples']}")
        print(f"成功率: {stats['success_rate']:.1f}%")
        print(f"延迟范围: {stats['min_ms']:.3f}ms - {stats['max_ms']:.3f}ms")
        print(f"平均延迟: {stats['mean_ms']:.3f}ms")
        print(f"中位数延迟: {stats['median_ms']:.3f}ms")
        print(f"P95延迟: {stats['p95_ms']:.3f}ms")
        print(f"P99延迟: {stats['p99_ms']:.3f}ms")
        print()
        
        print("延迟分布:")
        print(f"  < 1ms:  {stats['under_1ms_count']} ({stats['under_1ms_rate']:.1f}%)")
        print(f"  < 2ms:  {stats['under_2ms_rate']:.1f}%")
        print(f"  < 5ms:  {stats['under_5ms_rate']:.1f}%")
        print()
        
        # 优化评估
        if stats['under_1ms_rate'] > 50:
            level = "🏆 突破性成功"
            desc = "超过50%订单达到<1ms，AWS优化极其成功！"
        elif stats['under_1ms_rate'] > 20:
            level = "🥇 优化成功"  
            desc = "超过20%订单达到<1ms，优化效果显著"
        elif stats['under_1ms_rate'] > 5:
            level = "🥈 部分成功"
            desc = "部分订单达到<1ms，继续优化空间"
        else:
            level = "📈 基础改善"
            desc = "延迟有改善，但需要进一步优化"
        
        print(f"优化等级: {level}")
        print(f"评估: {desc}")
        
        if stats['mean_ms'] < 1.0:
            print("🎯 目标达成: 平均延迟已突破1ms界限！")
        else:
            improvement_needed = (stats['mean_ms'] - 1.0) / stats['mean_ms'] * 100
            print(f"🎯 距离目标: 还需改善 {improvement_needed:.1f}% 达到1ms平均延迟")
        
        print("="*80)
    
    async def run_comprehensive_test(self):
        """运行综合测试"""
        print("启动AWS云服务器综合延迟优化测试...")
        
        # 运行HTTP基准测试
        http_results = await self.benchmark_http(800)
        http_stats = self.analyze_results(http_results)
        
        print("\nHTTP优化结果:")
        self.print_optimization_results(http_stats)
        
        # 运行并行测试
        parallel_results = await self.benchmark_parallel(400)
        parallel_stats = self.analyze_results(parallel_results)
        
        print("\n并行优化结果:")
        self.print_optimization_results(parallel_stats)
        
        return http_stats, parallel_stats
    
    async def cleanup(self):
        """清理资源"""
        await self.http_client.close()
        self.thread_pool.shutdown(wait=True)
        print("✅ 资源清理完成")

async def main():
    """主函数"""
    print("AWS云服务器超低延迟优化测试")
    print("目标: 在AWS环境下实现<1ms延迟")
    print("="*60)
    
    try:
        # 创建并设置系统
        system = AWSUltraLowLatencySystem()
        await system.setup()
        
        # 运行综合测试
        await system.run_comprehensive_test()
        
        # 清理
        await system.cleanup()
        
    except KeyboardInterrupt:
        print("\n测试被用户中断")
    except Exception as e:
        print(f"测试出错: {e}")
        import traceback
        traceback.print_exc()

if __name__ == "__main__":
    # 启用所有优化
    try:
        import websockets  # 导入websockets库
    except ImportError:
        print("警告: websockets库未安装，WebSocket功能将不可用")
        print("安装: pip install websockets")
    
    asyncio.run(main())