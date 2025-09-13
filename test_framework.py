#!/usr/bin/env python3
import asyncio
import aiohttp
import time
import json
import logging
from datetime import datetime
from typing import Dict, List, Tuple
import threading
from aiohttp import web
import socket
import struct
from collections import defaultdict

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class LatencyTestFramework:
    def __init__(self):
        self.exchanges = {
            'binance': {
                'spot_endpoints': ['api.binance.com', 'api1.binance.com', 'api2.binance.com', 'api3.binance.com'],
                'futures_endpoints': ['fapi.binance.com', 'fapi1.binance.com'],
                'port': 8881
            },
            'huobi': {
                'spot_endpoints': ['api.huobi.pro', 'api-aws.huobi.pro'],
                'futures_endpoints': ['api.hbdm.com'],
                'port': 8882
            },
            'okex': {
                'spot_endpoints': ['www.okx.com', 'aws.okx.com'],
                'futures_endpoints': ['www.okx.com', 'aws.okx.com'],
                'port': 8883
            }
        }
        
        self.latency_records = defaultdict(list)
        self.order_count = defaultdict(int)
        self.mock_servers = {}
        
    async def start_mock_exchange(self, exchange_name: str, port: int):
        """启动模拟交易所服务器"""
        app = web.Application()
        
        async def handle_order(request):
            receive_time = time.time() * 1000
            
            try:
                data = await request.json()
            except:
                data = {}
            
            # 从请求头获取发送时间戳
            send_time = float(request.headers.get('X-Send-Time', receive_time))
            latency = receive_time - send_time
            
            # 记录延迟
            self.latency_records[exchange_name].append({
                'timestamp': datetime.now().isoformat(),
                'latency_ms': latency,
                'endpoint': request.path,
                'method': request.method
            })
            
            self.order_count[exchange_name] += 1
            
            logger.info(f"{exchange_name} received order - Latency: {latency:.2f}ms")
            
            # 模拟交易所响应
            response = {
                'orderId': f'{exchange_name}_{int(time.time()*1000)}',
                'status': 'NEW',
                'receive_time': receive_time,
                'latency_ms': latency
            }
            
            return web.json_response(response)
        
        # 注册所有可能的订单路径
        app.router.add_post('/api/v3/order', handle_order)  # Binance spot
        app.router.add_post('/api/v3/order/test', handle_order)  # Binance test
        app.router.add_post('/fapi/v1/order', handle_order)  # Binance futures
        app.router.add_post('/v1/order/orders/place', handle_order)  # Huobi spot
        app.router.add_post('/v1/contract_order', handle_order)  # Huobi futures
        app.router.add_post('/api/v5/trade/order', handle_order)  # OKEx unified
        app.router.add_post('/{path:.*}', handle_order)  # Catch all
        
        runner = web.AppRunner(app)
        await runner.setup()
        site = web.TCPSite(runner, '127.0.0.1', port)
        await site.start()
        
        logger.info(f"Mock {exchange_name} server started on port {port}")
        return runner

    async def intercept_and_forward(self, local_port: int, remote_host: str, remote_port: int, exchange_name: str):
        """TCP代理拦截器，测量真实延迟"""
        server = await asyncio.start_server(
            lambda r, w: self.handle_client(r, w, remote_host, remote_port, exchange_name),
            '127.0.0.1', local_port
        )
        
        async with server:
            logger.info(f"Interceptor for {exchange_name} started on port {local_port}")
            await server.serve_forever()
    
    async def handle_client(self, reader, writer, remote_host: str, remote_port: int, exchange_name: str):
        """处理客户端连接并转发到真实交易所"""
        try:
            # 连接到真实交易所
            remote_reader, remote_writer = await asyncio.open_connection(remote_host, remote_port)
            
            # 双向转发
            await asyncio.gather(
                self.forward_data(reader, remote_writer, f"{exchange_name}_request"),
                self.forward_data(remote_reader, writer, f"{exchange_name}_response")
            )
        except Exception as e:
            logger.error(f"Error handling client for {exchange_name}: {e}")
        finally:
            writer.close()
            await writer.wait_closed()
    
    async def forward_data(self, reader, writer, direction: str):
        """转发数据并记录时间戳"""
        try:
            while True:
                data = await reader.read(4096)
                if not data:
                    break
                
                if 'request' in direction:
                    send_time = time.time() * 1000
                    # 尝试注入时间戳到HTTP头
                    if b'POST' in data or b'GET' in data:
                        timestamp_header = f'X-Send-Time: {send_time}\r\n'.encode()
                        data = data.replace(b'\r\n\r\n', b'\r\n' + timestamp_header + b'\r\n')
                
                writer.write(data)
                await writer.drain()
        except Exception as e:
            logger.debug(f"Forward ended for {direction}: {e}")

    async def test_exchange_latency(self, exchange_name: str, test_duration: int = 60):
        """测试特定交易所的延迟"""
        logger.info(f"Starting latency test for {exchange_name} for {test_duration} seconds")
        
        exchange_config = self.exchanges[exchange_name]
        port = exchange_config['port']
        
        # 启动模拟服务器
        mock_server = await self.start_mock_exchange(exchange_name, port)
        self.mock_servers[exchange_name] = mock_server
        
        # 模拟发送订单
        start_time = time.time()
        order_interval = 0.1  # 每100ms发送一个订单
        
        async with aiohttp.ClientSession() as session:
            while time.time() - start_time < test_duration:
                send_time = time.time() * 1000
                
                # 构造订单数据
                order_data = {
                    'symbol': 'BTCUSDT',
                    'side': 'BUY',
                    'type': 'LIMIT',
                    'quantity': 0.001,
                    'price': 50000,
                    'timestamp': int(send_time)
                }
                
                try:
                    headers = {'X-Send-Time': str(send_time)}
                    async with session.post(
                        f'http://127.0.0.1:{port}/api/v3/order',
                        json=order_data,
                        headers=headers,
                        timeout=aiohttp.ClientTimeout(total=5)
                    ) as response:
                        result = await response.json()
                        logger.debug(f"Order sent to {exchange_name}: {result}")
                except Exception as e:
                    logger.error(f"Error sending order to {exchange_name}: {e}")
                
                await asyncio.sleep(order_interval)
        
        logger.info(f"Completed latency test for {exchange_name}")

    def generate_report(self):
        """生成测试报告"""
        report = {
            'test_time': datetime.now().isoformat(),
            'summary': {},
            'details': {}
        }
        
        for exchange, records in self.latency_records.items():
            if records:
                latencies = [r['latency_ms'] for r in records]
                report['summary'][exchange] = {
                    'total_orders': len(records),
                    'avg_latency_ms': sum(latencies) / len(latencies),
                    'min_latency_ms': min(latencies),
                    'max_latency_ms': max(latencies),
                    'p50_latency_ms': self.percentile(latencies, 50),
                    'p95_latency_ms': self.percentile(latencies, 95),
                    'p99_latency_ms': self.percentile(latencies, 99)
                }
                report['details'][exchange] = records[:10]  # 保存前10条记录作为样本
        
        return report
    
    def percentile(self, data: List[float], p: int) -> float:
        """计算百分位数"""
        if not data:
            return 0
        sorted_data = sorted(data)
        index = int(len(sorted_data) * p / 100)
        return sorted_data[min(index, len(sorted_data) - 1)]
    
    async def run_full_test(self, test_duration: int = 60):
        """运行完整的测试"""
        logger.info("Starting full latency test framework")
        
        # 并行测试所有交易所
        tasks = []
        for exchange in self.exchanges.keys():
            tasks.append(self.test_exchange_latency(exchange, test_duration))
        
        await asyncio.gather(*tasks)
        
        # 生成报告
        report = self.generate_report()
        
        # 保存报告
        report_file = f'latency_report_{datetime.now().strftime("%Y%m%d_%H%M%S")}.json'
        with open(report_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        # 打印摘要
        print("\n" + "="*80)
        print("LATENCY TEST REPORT")
        print("="*80)
        
        for exchange, stats in report['summary'].items():
            print(f"\n{exchange.upper()}:")
            print(f"  Total Orders: {stats['total_orders']}")
            print(f"  Average Latency: {stats['avg_latency_ms']:.2f} ms")
            print(f"  Min Latency: {stats['min_latency_ms']:.2f} ms")
            print(f"  Max Latency: {stats['max_latency_ms']:.2f} ms")
            print(f"  P50 Latency: {stats['p50_latency_ms']:.2f} ms")
            print(f"  P95 Latency: {stats['p95_latency_ms']:.2f} ms")
            print(f"  P99 Latency: {stats['p99_latency_ms']:.2f} ms")
        
        print(f"\nDetailed report saved to: {report_file}")
        
        # 清理
        for server in self.mock_servers.values():
            await server.cleanup()

async def main():
    framework = LatencyTestFramework()
    await framework.run_full_test(test_duration=30)  # 运行30秒测试

if __name__ == "__main__":
    asyncio.run(main())