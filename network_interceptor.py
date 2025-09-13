#!/usr/bin/env python3
import asyncio
import time
import json
import logging
from mitmproxy import http, options
from mitmproxy.tools.dump import DumpMaster
import threading
from collections import defaultdict
from datetime import datetime

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class NetworkInterceptor:
    """网络请求拦截器，监控套利系统5.1的真实请求"""
    
    def __init__(self):
        self.latency_data = defaultdict(list)
        self.request_timestamps = {}
        self.exchange_endpoints = {
            'binance': ['api.binance.com', 'fapi.binance.com', 'api1.binance.com'],
            'huobi': ['api.huobi.pro', 'api-aws.huobi.pro', 'api.hbdm.com'],
            'okex': ['www.okx.com', 'aws.okx.com']
        }
        
    def identify_exchange(self, host: str) -> str:
        """识别请求的交易所"""
        for exchange, endpoints in self.exchange_endpoints.items():
            for endpoint in endpoints:
                if endpoint in host:
                    return exchange
        return 'unknown'
    
    def request(self, flow: http.HTTPFlow):
        """拦截请求并记录发送时间"""
        host = flow.request.pretty_host
        exchange = self.identify_exchange(host)
        
        if exchange != 'unknown':
            # 记录发送时间
            send_time = time.time() * 1000
            flow_id = id(flow)
            self.request_timestamps[flow_id] = {
                'send_time': send_time,
                'exchange': exchange,
                'path': flow.request.path,
                'method': flow.request.method
            }
            
            # 注入时间戳头
            flow.request.headers['X-Send-Time'] = str(send_time)
            
            logger.info(f"Intercepted {exchange} request: {flow.request.method} {flow.request.path}")
    
    def response(self, flow: http.HTTPFlow):
        """拦截响应并计算延迟"""
        flow_id = id(flow)
        
        if flow_id in self.request_timestamps:
            receive_time = time.time() * 1000
            request_info = self.request_timestamps[flow_id]
            
            # 计算延迟
            latency = receive_time - request_info['send_time']
            
            # 记录延迟数据
            self.latency_data[request_info['exchange']].append({
                'timestamp': datetime.now().isoformat(),
                'latency_ms': latency,
                'path': request_info['path'],
                'method': request_info['method'],
                'status_code': flow.response.status_code
            })
            
            logger.info(f"{request_info['exchange']} response - Latency: {latency:.2f}ms")
            
            # 清理
            del self.request_timestamps[flow_id]
    
    def get_statistics(self):
        """获取延迟统计信息"""
        stats = {}
        for exchange, records in self.latency_data.items():
            if records:
                latencies = [r['latency_ms'] for r in records]
                stats[exchange] = {
                    'total_requests': len(records),
                    'avg_latency_ms': sum(latencies) / len(latencies),
                    'min_latency_ms': min(latencies),
                    'max_latency_ms': max(latencies),
                    'p50_latency_ms': self.percentile(latencies, 50),
                    'p95_latency_ms': self.percentile(latencies, 95),
                    'p99_latency_ms': self.percentile(latencies, 99)
                }
        return stats
    
    def percentile(self, data: list, p: int) -> float:
        """计算百分位数"""
        if not data:
            return 0
        sorted_data = sorted(data)
        index = int(len(sorted_data) * p / 100)
        return sorted_data[min(index, len(sorted_data) - 1)]
    
    def save_report(self, filename: str = None):
        """保存测试报告"""
        if filename is None:
            filename = f'interceptor_report_{datetime.now().strftime("%Y%m%d_%H%M%S")}.json'
        
        report = {
            'test_time': datetime.now().isoformat(),
            'statistics': self.get_statistics(),
            'raw_data': dict(self.latency_data)
        }
        
        with open(filename, 'w') as f:
            json.dump(report, f, indent=2)
        
        logger.info(f"Report saved to {filename}")
        return filename

class ProxyRunner:
    """运行mitmproxy作为透明代理"""
    
    def __init__(self, interceptor: NetworkInterceptor):
        self.interceptor = interceptor
        self.master = None
        
    async def start(self, port: int = 8080):
        """启动代理服务器"""
        opts = options.Options(
            listen_port=port,
            mode=['transparent'],  # 透明代理模式
            ssl_insecure=True
        )
        
        self.master = DumpMaster(opts)
        self.master.addons.add(self.interceptor)
        
        logger.info(f"Starting proxy on port {port}")
        await self.master.run()
    
    def stop(self):
        """停止代理服务器"""
        if self.master:
            self.master.shutdown()

class DirectInterceptor:
    """直接拦截方式（不使用mitmproxy）"""
    
    def __init__(self):
        self.latency_records = defaultdict(list)
        
    async def intercept_requests(self, target_host: str, target_port: int, 
                                 local_port: int, exchange_name: str):
        """TCP层面的请求拦截"""
        
        async def handle_client(reader, writer):
            try:
                # 记录请求开始时间
                send_time = time.time() * 1000
                
                # 连接到目标服务器
                target_reader, target_writer = await asyncio.open_connection(
                    target_host, target_port
                )
                
                # 转发请求
                request_data = await reader.read(4096)
                target_writer.write(request_data)
                await target_writer.drain()
                
                # 等待响应
                response_data = await target_reader.read(65536)
                
                # 记录响应时间
                receive_time = time.time() * 1000
                latency = receive_time - send_time
                
                # 保存延迟记录
                self.latency_records[exchange_name].append({
                    'timestamp': datetime.now().isoformat(),
                    'latency_ms': latency,
                    'host': target_host,
                    'port': target_port
                })
                
                logger.info(f"{exchange_name} TCP latency: {latency:.2f}ms")
                
                # 返回响应给客户端
                writer.write(response_data)
                await writer.drain()
                
                # 继续转发剩余数据
                await asyncio.gather(
                    self.forward_data(reader, target_writer),
                    self.forward_data(target_reader, writer)
                )
                
            except Exception as e:
                logger.error(f"Error in handle_client: {e}")
            finally:
                writer.close()
                await writer.wait_closed()
                target_writer.close()
                await target_writer.wait_closed()
        
        server = await asyncio.start_server(handle_client, '127.0.0.1', local_port)
        
        async with server:
            logger.info(f"TCP interceptor for {exchange_name} started on port {local_port}")
            await server.serve_forever()
    
    async def forward_data(self, reader, writer):
        """转发数据"""
        try:
            while True:
                data = await reader.read(4096)
                if not data:
                    break
                writer.write(data)
                await writer.drain()
        except:
            pass

if __name__ == "__main__":
    # 测试拦截器
    interceptor = NetworkInterceptor()
    
    # 这里可以配置为透明代理或HTTP代理
    print("Network Interceptor initialized")
    print("Configure your arbitrage system to use proxy at 127.0.0.1:8080")
    print("Or use iptables rules for transparent proxy")