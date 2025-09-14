#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
QINGXI HTTP API 管理工具
用于通过API调用管理系统配置，避免复杂的命令行操作
"""

import requests
import json
import time
import sys
from datetime import datetime
from typing import Dict, List, Any, Optional

class QingxiApiManager:
    """QINGXI API 管理器"""
    
    def __init__(self, base_url: str = "http://localhost:50061"):
        self.base_url = base_url.rstrip('/')
        self.session = requests.Session()
        self.session.headers.update({
            'Content-Type': 'application/json',
            'User-Agent': 'QINGXI-API-Manager/1.0'
        })
    
    def _make_request(self, method: str, endpoint: str, **kwargs) -> Dict[str, Any]:
        """发起API请求"""
        url = f"{self.base_url}/{endpoint.lstrip('/')}"
        try:
            response = self.session.request(method, url, **kwargs)
            response.raise_for_status()
            return response.json() if response.content else {}
        except requests.exceptions.RequestException as e:
            print(f"❌ API请求失败 {method} {url}: {e}")
            return {"error": str(e)}
    
    def get_system_status(self) -> Dict[str, Any]:
        """获取系统状态"""
        return self._make_request('GET', '/api/v1/health/summary')
    
    def get_v3_performance(self) -> Dict[str, Any]:
        """获取V3.0性能状态"""
        return self._make_request('GET', '/api/v1/v3/performance')
    
    def get_optimization_status(self) -> Dict[str, Any]:
        """获取优化组件状态"""
        return self._make_request('GET', '/api/v1/v3/optimization-status')
    
    def get_exchanges(self) -> Dict[str, Any]:
        """获取交易所列表"""
        return self._make_request('GET', '/api/v1/exchanges')
    
    def get_symbols(self) -> Dict[str, Any]:
        """获取交易对列表"""
        return self._make_request('GET', '/api/v1/symbols')
    
    def reconfigure_system(self, new_config: Dict[str, Any]) -> Dict[str, Any]:
        """重新配置系统"""
        return self._make_request('POST', '/api/v1/reconfigure', json=new_config)
    
    def reset_stats(self) -> Dict[str, Any]:
        """重置统计信息"""
        return self._make_request('POST', '/api/v1/v3/reset-stats')
    
    def enable_optimization(self, optimization_config: Dict[str, Any]) -> Dict[str, Any]:
        """启用优化功能"""
        return self._make_request('POST', '/api/v1/v3/enable-optimization', json=optimization_config)
    
    def get_orderbook(self, exchange: str, symbol: str) -> Dict[str, Any]:
        """获取订单簿数据"""
        return self._make_request('GET', f'/api/v1/orderbook/{exchange}/{symbol}')
    
    def wait_for_system_ready(self, timeout: int = 60) -> bool:
        """等待系统就绪"""
        print(f"⏳ 等待系统启动就绪 (超时: {timeout}秒)...")
        start_time = time.time()
        
        while time.time() - start_time < timeout:
            try:
                status = self.get_system_status()
                if not status.get('error') and status.get('summary', {}).get('healthy_sources', 0) > 0:
                    print("✅ 系统已就绪")
                    return True
            except:
                pass
            
            print(".", end="", flush=True)
            time.sleep(2)
        
        print(f"\n❌ 系统启动超时 ({timeout}秒)")
        return False

def create_optimized_config() -> Dict[str, Any]:
    """创建优化后的配置"""
    return {
        "general": {
            "log_level": "info",
            "metrics_enabled": True
        },
        "api_server": {
            "host": "127.0.0.1",
            "port": 50051,
            "metrics_port_offset": 1,
            "health_port_offset": 2,
            "http_port_offset": 10,
            "orderbook_depth_limit": 150,  # 增加到150档
            "symbols_list_limit": 100
        },
        "sources": [
            {
                "id": "binance_spot",
                "adapter_type": "binance",
                "enabled": True,
                "exchange_id": "binance",
                "symbols": generate_top_symbols(100),
                "ws_endpoint": "wss://stream.binance.com:9443/ws/",
                "rest_url": "https://api.binance.com",
                "channel": "depth20@100ms"  # 修复channel配置
            },
            {
                "id": "huobi_spot", 
                "adapter_type": "huobi",
                "enabled": True,
                "exchange_id": "huobi",
                "symbols": generate_top_symbols(100),
                "ws_endpoint": "wss://api.huobi.pro/ws",
                "rest_url": "https://api.huobi.pro",
                "channel": "market.depth.step0"  # 修复channel配置
            },
            {
                "id": "okx_spot",
                "adapter_type": "okx", 
                "enabled": True,
                "exchange_id": "okx",
                "symbols": generate_top_symbols(100, okx_format=True),
                "ws_endpoint": "wss://ws.okx.com:8443/ws/v5/public",
                "rest_url": "https://www.okx.com",
                "channel": "books5"  # 修复channel配置
            },
            {
                "id": "bybit_spot",
                "adapter_type": "bybit",
                "enabled": True, 
                "exchange_id": "bybit",
                "symbols": generate_top_symbols(100),
                "ws_endpoint": "wss://stream.bybit.com/v5/public/spot",
                "rest_url": "https://api.bybit.com",
                "channel": "orderbook.200"  # 优化为200档深度
            }
        ],
        "performance": {
            "enable_batch_processing": True,
            "batch_size": 10000,  # 增大批处理
            "batch_timeout_ms": 50,  # 减少超时
            "enable_simd": True,
            "enable_parallel_processing": True,
            "max_concurrent_tasks": 16,  # 增加并发
            "memory_pool_size": 2097152,  # 增大内存池
            "enable_zero_copy": True,
            "performance_stats_interval_sec": 10,  # 更频繁的统计
            "system_readiness_timeout_sec": 30
        },
        "threading": {
            "num_worker_threads": 8,  # 增加线程数
            "enable_cpu_affinity": True,
            "preferred_cores": [0, 1, 2, 3, 4, 5, 6, 7],
            "enable_numa_awareness": True,
            "network_worker_threads": 6,  # 增加网络线程
            "processing_worker_threads": 4,  # 增加处理线程
            "main_worker_threads": 2
        },
        "cleaner": {
            "memory_pool_size": 131072,  # 增大清洗内存池
            "batch_size": 50000,  # 增大批处理
            "orderbook_capacity": 5000,  # 增大容量
            "zero_alloc_buffer_count": 131072,
            "thread_count": 8,  # 增加清洗线程
            "target_latency_ns": 50000,  # 目标50μs (0.05ms)
            "orderbook_bid_capacity": 5000,
            "orderbook_ask_capacity": 5000,
            "volume_top_count": 20
        }
    }

def generate_top_symbols(count: int, okx_format: bool = False) -> List[str]:
    """生成顶级交易对列表"""
    base_symbols = [
        "BTC", "ETH", "BNB", "ADA", "SOL", "XRP", "DOT", "MATIC", "AVAX", "LTC",
        "UNI", "LINK", "ATOM", "XLM", "VET", "FIL", "TRX", "EOS", "XMR", "NEO",
        "DASH", "IOTA", "ALGO", "ZEC", "COMP", "YFI", "MKR", "AAVE", "SUSHI", "SNX",
        "CRV", "1INCH", "BAL", "REN", "KNC", "ZRX", "UMA", "BAND", "ALPHA", "REEF",
        "OCEAN", "INJ", "AUDIO", "CTSI", "AKRO", "RAY", "SRM", "FIDA", "OOKI", "SPELL",
        "GALA", "MANA", "SAND", "APE", "LRC", "ENJ", "CHZ", "BAT", "ZIL", "HOT",
        "ICX", "QTUM", "LSK", "SC", "ZEN", "WAVES", "KMD", "ARK", "STRAT", "BNT",
        "GNT", "STORJ", "ANT", "OMG", "GAS", "POWR", "SUB", "ENG", "SALT", "FUN",
        "REQ", "VIB", "TRX", "POE", "FUEL", "MTL", "DNT", "LOOPRING", "AST", "MANA",
        "TNB", "DLT", "AMB", "BCPT", "ARN", "GVT", "CDT", "GXS", "POA", "QSP"
    ]
    
    if okx_format:
        return [f"{symbol}-USDT" for symbol in base_symbols[:count]]
    else:
        return [f"{symbol}USDT" for symbol in base_symbols[:count]]

def main():
    """主函数"""
    print("🚀 QINGXI HTTP API 管理工具")
    print("=" * 50)
    
    # 初始化API管理器
    api = QingxiApiManager()
    
    if len(sys.argv) < 2:
        print("用法:")
        print("  python3 api_manager.py status           # 查看系统状态")
        print("  python3 api_manager.py optimize         # 启动优化配置")
        print("  python3 api_manager.py performance      # 查看性能数据")
        print("  python3 api_manager.py reconfigure      # 重新配置系统")
        print("  python3 api_manager.py test             # 测试API连通性")
        print("  python3 api_manager.py monitor          # 实时监控")
        return
    
    command = sys.argv[1].lower()
    
    if command == "status":
        print("📊 获取系统状态...")
        status = api.get_system_status()
        if status.get('error'):
            print(f"❌ 获取状态失败: {status['error']}")
        else:
            print_status(status)
    
    elif command == "optimize":
        print("⚡ 启动优化配置...")
        config = create_optimized_config()
        result = api.reconfigure_system(config)
        if result.get('error'):
            print(f"❌ 配置失败: {result['error']}")
        else:
            print("✅ 优化配置已应用")
            print(json.dumps(result, indent=2))
    
    elif command == "performance":
        print("📈 获取性能数据...")
        perf = api.get_v3_performance()
        opt_status = api.get_optimization_status()
        print_performance(perf, opt_status)
    
    elif command == "reconfigure":
        print("🔧 重新配置系统...")
        config = create_optimized_config()
        result = api.reconfigure_system(config)
        print("配置结果:", json.dumps(result, indent=2))
    
    elif command == "test":
        print("🧪 测试API连通性...")
        test_api_connectivity(api)
    
    elif command == "monitor":
        print("👁️ 启动实时监控...")
        monitor_system(api)
    
    else:
        print(f"❌ 未知命令: {command}")

def print_status(status: Dict[str, Any]):
    """打印系统状态"""
    summary = status.get('summary', {})
    sources = status.get('sources', [])
    
    print(f"📊 系统状态摘要:")
    print(f"   总数据源: {summary.get('total_sources', 0)}")
    print(f"   健康数据源: {summary.get('healthy_sources', 0)}")
    print(f"   异常数据源: {summary.get('unhealthy_sources', 0)}")
    print(f"   平均延迟: {summary.get('average_latency_us', 0):.2f}μs")
    print(f"   总消息数: {summary.get('total_messages', 0)}")
    
    print(f"\n📋 数据源详情:")
    for source in sources:
        status_icon = "✅" if source.get('is_connected') else "❌"
        print(f"   {status_icon} {source.get('source_id')}: "
              f"{source.get('latency_us', 0):.2f}μs, "
              f"{source.get('message_count', 0)} msgs")

def print_performance(perf: Dict[str, Any], opt_status: Dict[str, Any]):
    """打印性能数据"""
    print("📈 V3.0性能数据:")
    if perf.get('error'):
        print(f"❌ {perf['error']}")
    else:
        print(json.dumps(perf, indent=2))
    
    print("\n⚡ 优化状态:")
    if opt_status.get('error'):
        print(f"❌ {opt_status['error']}")
    else:
        print(json.dumps(opt_status, indent=2))

def test_api_connectivity(api: QingxiApiManager):
    """测试API连通性"""
    tests = [
        ("健康检查", lambda: api.get_system_status()),
        ("交易所列表", lambda: api.get_exchanges()),
        ("交易对列表", lambda: api.get_symbols()),
        ("性能数据", lambda: api.get_v3_performance()),
        ("优化状态", lambda: api.get_optimization_status()),
    ]
    
    for test_name, test_func in tests:
        try:
            result = test_func()
            if result.get('error'):
                print(f"❌ {test_name}: {result['error']}")
            else:
                print(f"✅ {test_name}: OK")
        except Exception as e:
            print(f"❌ {test_name}: {e}")

def monitor_system(api: QingxiApiManager):
    """实时监控系统"""
    print("开始实时监控... (Ctrl+C 退出)")
    try:
        while True:
            status = api.get_system_status()
            if not status.get('error'):
                summary = status.get('summary', {})
                healthy = summary.get('healthy_sources', 0)
                total = summary.get('total_sources', 0)
                latency = summary.get('average_latency_us', 0)
                
                print(f"[{datetime.now().strftime('%H:%M:%S')}] "
                      f"健康: {healthy}/{total}, "
                      f"延迟: {latency:.2f}μs")
            
            time.sleep(5)
    except KeyboardInterrupt:
        print("\n监控已停止")

if __name__ == "__main__":
    main()
