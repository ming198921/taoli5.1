#!/usr/bin/env python3
import asyncio
import json
import time
import random
from aiohttp import web
import logging

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ExchangeSimulator:
    """模拟真实交易所的响应行为"""
    
    def __init__(self, exchange_name: str):
        self.exchange_name = exchange_name
        self.orders = {}
        self.order_counter = 0
        
        # 模拟不同交易所的延迟特性
        self.latency_profiles = {
            'binance': {'base': 5, 'variance': 2},   # 5±2ms
            'huobi': {'base': 8, 'variance': 3},      # 8±3ms
            'okex': {'base': 6, 'variance': 2.5}      # 6±2.5ms
        }
    
    async def simulate_processing_delay(self):
        """模拟交易所内部处理延迟"""
        profile = self.latency_profiles.get(self.exchange_name, {'base': 5, 'variance': 2})
        delay = profile['base'] + random.uniform(-profile['variance'], profile['variance'])
        await asyncio.sleep(delay / 1000)  # 转换为秒
    
    def create_binance_response(self, order_data: dict) -> dict:
        """创建币安格式的响应"""
        self.order_counter += 1
        return {
            'symbol': order_data.get('symbol', 'BTCUSDT'),
            'orderId': self.order_counter,
            'orderListId': -1,
            'clientOrderId': order_data.get('newClientOrderId', f'auto_{self.order_counter}'),
            'transactTime': int(time.time() * 1000),
            'price': order_data.get('price', '0.00000000'),
            'origQty': order_data.get('quantity', '0.00000000'),
            'executedQty': '0.00000000',
            'cummulativeQuoteQty': '0.00000000',
            'status': 'NEW',
            'timeInForce': order_data.get('timeInForce', 'GTC'),
            'type': order_data.get('type', 'LIMIT'),
            'side': order_data.get('side', 'BUY')
        }
    
    def create_huobi_response(self, order_data: dict) -> dict:
        """创建火币格式的响应"""
        self.order_counter += 1
        return {
            'status': 'ok',
            'data': str(self.order_counter),
            'err-code': None,
            'err-msg': None
        }
    
    def create_okex_response(self, order_data: dict) -> dict:
        """创建OKEx格式的响应"""
        self.order_counter += 1
        return {
            'code': '0',
            'msg': '',
            'data': [{
                'clOrdId': order_data.get('clOrdId', ''),
                'ordId': str(self.order_counter),
                'tag': '',
                'sCode': '0',
                'sMsg': ''
            }]
        }

class MockExchangeServer:
    """模拟交易所服务器"""
    
    def __init__(self):
        self.simulators = {
            'binance': ExchangeSimulator('binance'),
            'huobi': ExchangeSimulator('huobi'),
            'okex': ExchangeSimulator('okex')
        }
        self.latency_stats = []
    
    async def handle_binance_order(self, request):
        """处理币安订单请求"""
        receive_time = time.time() * 1000
        simulator = self.simulators['binance']
        
        try:
            data = await request.json()
        except:
            data = {}
        
        # 获取发送时间
        send_time = float(request.headers.get('X-Send-Time', receive_time))
        
        # 模拟处理延迟
        await simulator.simulate_processing_delay()
        
        # 创建响应
        response = simulator.create_binance_response(data)
        
        # 计算总延迟
        total_latency = time.time() * 1000 - send_time
        
        # 记录统计
        self.latency_stats.append({
            'exchange': 'binance',
            'timestamp': time.time(),
            'latency_ms': total_latency,
            'endpoint': request.path
        })
        
        logger.info(f"Binance order processed - Latency: {total_latency:.2f}ms")
        
        return web.json_response(response)
    
    async def handle_huobi_order(self, request):
        """处理火币订单请求"""
        receive_time = time.time() * 1000
        simulator = self.simulators['huobi']
        
        try:
            data = await request.json()
        except:
            data = {}
        
        send_time = float(request.headers.get('X-Send-Time', receive_time))
        
        await simulator.simulate_processing_delay()
        
        response = simulator.create_huobi_response(data)
        
        total_latency = time.time() * 1000 - send_time
        
        self.latency_stats.append({
            'exchange': 'huobi',
            'timestamp': time.time(),
            'latency_ms': total_latency,
            'endpoint': request.path
        })
        
        logger.info(f"Huobi order processed - Latency: {total_latency:.2f}ms")
        
        return web.json_response(response)
    
    async def handle_okex_order(self, request):
        """处理OKEx订单请求"""
        receive_time = time.time() * 1000
        simulator = self.simulators['okex']
        
        try:
            data = await request.json()
        except:
            data = {}
        
        send_time = float(request.headers.get('X-Send-Time', receive_time))
        
        await simulator.simulate_processing_delay()
        
        response = simulator.create_okex_response(data)
        
        total_latency = time.time() * 1000 - send_time
        
        self.latency_stats.append({
            'exchange': 'okex',
            'timestamp': time.time(),
            'latency_ms': total_latency,
            'endpoint': request.path
        })
        
        logger.info(f"OKEx order processed - Latency: {total_latency:.2f}ms")
        
        return web.json_response(response)
    
    async def handle_stats(self, request):
        """返回延迟统计信息"""
        stats = {}
        for exchange in ['binance', 'huobi', 'okex']:
            exchange_stats = [s for s in self.latency_stats if s['exchange'] == exchange]
            if exchange_stats:
                latencies = [s['latency_ms'] for s in exchange_stats]
                stats[exchange] = {
                    'count': len(latencies),
                    'avg_ms': sum(latencies) / len(latencies),
                    'min_ms': min(latencies),
                    'max_ms': max(latencies)
                }
        
        return web.json_response(stats)

async def run_mock_exchanges():
    """运行模拟交易所服务器"""
    server = MockExchangeServer()
    
    # 币安服务器
    binance_app = web.Application()
    binance_app.router.add_post('/api/v3/order', server.handle_binance_order)
    binance_app.router.add_post('/api/v3/order/test', server.handle_binance_order)
    binance_app.router.add_post('/fapi/v1/order', server.handle_binance_order)
    binance_app.router.add_get('/stats', server.handle_stats)
    
    # 火币服务器
    huobi_app = web.Application()
    huobi_app.router.add_post('/v1/order/orders/place', server.handle_huobi_order)
    huobi_app.router.add_post('/v1/contract_order', server.handle_huobi_order)
    huobi_app.router.add_get('/stats', server.handle_stats)
    
    # OKEx服务器
    okex_app = web.Application()
    okex_app.router.add_post('/api/v5/trade/order', server.handle_okex_order)
    okex_app.router.add_get('/stats', server.handle_stats)
    
    # 启动所有服务器
    runners = []
    sites = []
    
    for app, port, name in [
        (binance_app, 8881, 'Binance'),
        (huobi_app, 8882, 'Huobi'),
        (okex_app, 8883, 'OKEx')
    ]:
        runner = web.AppRunner(app)
        await runner.setup()
        site = web.TCPSite(runner, '127.0.0.1', port)
        await site.start()
        runners.append(runner)
        sites.append(site)
        logger.info(f"Mock {name} server started on port {port}")
    
    try:
        await asyncio.Event().wait()
    except KeyboardInterrupt:
        logger.info("Shutting down mock exchange servers...")
        for runner in runners:
            await runner.cleanup()

if __name__ == "__main__":
    asyncio.run(run_mock_exchanges())