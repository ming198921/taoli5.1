#!/usr/bin/env python3
"""
极致性能测试脚本
Ultra Performance Test Script
"""

import time
import json
import requests
import logging
from datetime import datetime
from typing import Dict, List, Tuple

class UltraPerformanceTest:
    def __init__(self):
        self.api_base = "http://localhost:3000"
        self.results = []
        
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - %(message)s'
        )
        self.logger = logging.getLogger(__name__)

    def test_ultra_order_execution(self) -> Dict:
        """测试极致订单执行性能"""
        self.logger.info("🚀 测试极致订单执行性能 (目标: <30ms)")
        
        start_time = time.time()
        
        try:
            order_data = {
                "symbol": "BTCUSDT",
                "side": "SELL",
                "order_type": "MARKET",
                "quantity": 0.0001
            }
            
            response = requests.post(
                f"{self.api_base}/api/exchange-api/binance/order",
                json=order_data,
                headers={"Content-Type": "application/json"},
                timeout=5
            )
            
            execution_time = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                result = response.json()
                success = result.get("success", False)
                
                self.logger.info(f"✅ 订单执行时间: {execution_time:.2f}ms")
                if execution_time < 30:
                    self.logger.info(f"🎯 性能目标达成: {execution_time:.2f}ms < 30ms")
                else:
                    self.logger.warning(f"⚠️ 性能未达标: {execution_time:.2f}ms >= 30ms")
                
                return {
                    "test": "ultra_order_execution",
                    "execution_time_ms": execution_time,
                    "success": success,
                    "target_achieved": execution_time < 30,
                    "response": result
                }
            else:
                self.logger.error(f"❌ HTTP错误: {response.status_code}")
                return {
                    "test": "ultra_order_execution", 
                    "execution_time_ms": execution_time,
                    "success": False,
                    "error": f"HTTP {response.status_code}"
                }
                
        except Exception as e:
            execution_time = (time.time() - start_time) * 1000
            self.logger.error(f"❌ 测试异常: {str(e)}")
            return {
                "test": "ultra_order_execution",
                "execution_time_ms": execution_time,
                "success": False,
                "error": str(e)
            }

    def test_concurrent_data_fetch(self) -> Dict:
        """测试并发数据获取性能"""
        self.logger.info("🚀 测试并发数据获取性能 (目标: <0.5ms/币种)")
        
        start_time = time.time()
        
        try:
            response = requests.get(
                f"{self.api_base}/api/exchange-api/ultra-fast-prices",
                timeout=10
            )
            
            execution_time = (time.time() - start_time) * 1000
            
            if response.status_code == 200:
                result = response.json()
                
                if result.get("success"):
                    data = result.get("data", {})
                    symbols_fetched = data.get("symbols_fetched", 0)
                    avg_per_symbol = data.get("avg_per_symbol_ms", 0)
                    
                    self.logger.info(f"✅ 并发获取: {symbols_fetched}个币种, {execution_time:.2f}ms")
                    self.logger.info(f"📊 平均每币种: {avg_per_symbol:.3f}ms")
                    
                    if avg_per_symbol < 0.5:
                        self.logger.info(f"🎯 性能目标达成: {avg_per_symbol:.3f}ms < 0.5ms")
                    else:
                        self.logger.warning(f"⚠️ 性能未达标: {avg_per_symbol:.3f}ms >= 0.5ms")
                    
                    return {
                        "test": "concurrent_data_fetch",
                        "execution_time_ms": execution_time,
                        "symbols_fetched": symbols_fetched,
                        "avg_per_symbol_ms": avg_per_symbol,
                        "target_achieved": avg_per_symbol < 0.5,
                        "success": True
                    }
                else:
                    return {
                        "test": "concurrent_data_fetch",
                        "execution_time_ms": execution_time,
                        "success": False,
                        "error": "API返回失败"
                    }
            else:
                self.logger.error(f"❌ HTTP错误: {response.status_code}")
                return {
                    "test": "concurrent_data_fetch",
                    "execution_time_ms": execution_time,
                    "success": False,
                    "error": f"HTTP {response.status_code}"
                }
                
        except Exception as e:
            execution_time = (time.time() - start_time) * 1000
            self.logger.error(f"❌ 测试异常: {str(e)}")
            return {
                "test": "concurrent_data_fetch",
                "execution_time_ms": execution_time,
                "success": False,
                "error": str(e)
            }

    def run_performance_tests(self) -> Dict:
        """运行完整性能测试套件"""
        print("=" * 80)
        print("🚀 套利系统5.1 - 极致性能测试")
        print("Ultra Performance Test Suite")
        print("目标1: 订单执行 < 30ms | 目标2: 数据获取 < 0.5ms/币种")
        print("=" * 80)
        
        # 测试1: 极致订单执行
        order_test = self.test_ultra_order_execution()
        self.results.append(order_test)
        
        time.sleep(2)  # 短暂间隔
        
        # 测试2: 并发数据获取
        data_test = self.test_concurrent_data_fetch()
        self.results.append(data_test)
        
        # 生成最终报告
        report = self.generate_final_report()
        
        # 保存结果
        timestamp = int(time.time())
        with open(f"/home/ubuntu/5.1xitong/ultra_performance_report_{timestamp}.json", "w") as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        
        print(f"\n{'='*80}")
        print("🎯 极致性能测试完成!")
        print(f"📊 详细报告已保存到: ultra_performance_report_{timestamp}.json")
        print("=" * 80)
        
        return report

    def generate_final_report(self) -> Dict:
        """生成最终性能报告"""
        order_tests = [r for r in self.results if r["test"] == "ultra_order_execution"]
        data_tests = [r for r in self.results if r["test"] == "concurrent_data_fetch"]
        
        order_success = any(t.get("target_achieved", False) for t in order_tests)
        data_success = any(t.get("target_achieved", False) for t in data_tests)
        
        report = {
            "测试摘要": {
                "测试时间": datetime.now().isoformat(),
                "订单执行目标(30ms)": "✅ 达成" if order_success else "❌ 未达成",
                "数据获取目标(0.5ms)": "✅ 达成" if data_success else "❌ 未达成",
                "总体评价": "🎯 极致性能" if order_success and data_success else "⚠️ 需要优化"
            },
            "详细测试结果": self.results,
            "性能分析": {
                "订单执行最佳时间": min([t.get("execution_time_ms", 999) for t in order_tests]) if order_tests else 0,
                "数据获取最佳效率": min([t.get("avg_per_symbol_ms", 999) for t in data_tests]) if data_tests else 0
            }
        }
        
        return report

def main():
    """主执行函数"""
    tester = UltraPerformanceTest()
    result = tester.run_performance_tests()
    return result

if __name__ == "__main__":
    main()