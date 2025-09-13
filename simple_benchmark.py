#!/usr/bin/env python3
"""
简化版 <1ms 延迟基准测试
"""
import asyncio
import aiohttp
import time
import numpy as np
from typing import List

async def test_baseline_latency(iterations: int = 1000) -> List[float]:
    """基准延迟测试"""
    latencies = []
    
    connector = aiohttp.TCPConnector(
        limit=100,
        ttl_dns_cache=300,
        use_dns_cache=True,
        keepalive_timeout=60,
        enable_cleanup_closed=True
    )
    
    timeout = aiohttp.ClientTimeout(total=1.0)
    
    async with aiohttp.ClientSession(
        connector=connector, 
        timeout=timeout,
        skip_auto_headers=['User-Agent']
    ) as session:
        
        # 预热连接
        for _ in range(10):
            try:
                async with session.post(
                    'http://127.0.0.1:8881/api/v3/order',
                    json={'symbol': 'WARMUP'}
                ) as response:
                    await response.json()
            except:
                pass
        
        print("开始基准测试...")
        
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
                    },
                    headers={'Connection': 'keep-alive'}
                ) as response:
                    await response.json()
                    
            except Exception as e:
                print(f"请求失败: {e}")
                continue
            
            latency_ms = (time.perf_counter() - start) * 1000
            latencies.append(latency_ms)
            
            if i % 200 == 0:
                print(f"进度: {i}/{iterations}, 当前延迟: {latency_ms:.3f}ms")
            
            # 避免过载
            await asyncio.sleep(0.005)
    
    return latencies

def analyze_results(latencies: List[float]) -> dict:
    """分析结果"""
    if not latencies:
        return {}
    
    latencies_np = np.array(latencies)
    
    # 过滤异常值
    filtered = latencies_np[latencies_np < 100]  # 过滤>100ms的异常值
    
    under_1ms = np.sum(filtered < 1.0)
    under_2ms = np.sum(filtered < 2.0)
    under_5ms = np.sum(filtered < 5.0)
    under_10ms = np.sum(filtered < 10.0)
    
    return {
        'total_samples': len(filtered),
        'min_ms': np.min(filtered),
        'max_ms': np.max(filtered),
        'mean_ms': np.mean(filtered),
        'median_ms': np.median(filtered),
        'p95_ms': np.percentile(filtered, 95),
        'p99_ms': np.percentile(filtered, 99),
        'p999_ms': np.percentile(filtered, 99.9),
        'std_ms': np.std(filtered),
        'under_1ms_count': under_1ms,
        'under_1ms_rate': under_1ms / len(filtered) * 100,
        'under_2ms_rate': under_2ms / len(filtered) * 100,
        'under_5ms_rate': under_5ms / len(filtered) * 100,
        'under_10ms_rate': under_10ms / len(filtered) * 100,
    }

def print_optimization_analysis(stats: dict):
    """打印优化分析"""
    print("\n" + "=" * 80)
    print("5.1套利系统 延迟优化分析报告")
    print("=" * 80)
    
    print(f"样本数量: {stats['total_samples']}")
    print(f"延迟范围: {stats['min_ms']:.3f}ms - {stats['max_ms']:.3f}ms")
    print(f"平均延迟: {stats['mean_ms']:.3f}ms")
    print(f"中位数延迟: {stats['median_ms']:.3f}ms")
    print(f"标准差: {stats['std_ms']:.3f}ms")
    print()
    
    print("延迟分布:")
    print(f"  P50:  {stats['median_ms']:.3f}ms")
    print(f"  P95:  {stats['p95_ms']:.3f}ms")
    print(f"  P99:  {stats['p99_ms']:.3f}ms")
    print(f"  P99.9: {stats['p999_ms']:.3f}ms")
    print()
    
    print("延迟分级统计:")
    print(f"  < 1ms:  {stats['under_1ms_count']} ({stats['under_1ms_rate']:.1f}%)")
    print(f"  < 2ms:  {stats['under_2ms_rate']:.1f}%")
    print(f"  < 5ms:  {stats['under_5ms_rate']:.1f}%")
    print(f"  < 10ms: {stats['under_10ms_rate']:.1f}%")
    print()
    
    print("=" * 80)
    print("革命性优化潜力分析")
    print("=" * 80)
    
    # 当前最佳性能
    min_latency = stats['min_ms']
    avg_latency = stats['mean_ms']
    under_1ms_rate = stats['under_1ms_rate']
    
    print(f"✅ 当前最佳延迟: {min_latency:.3f}ms")
    
    if min_latency < 1.0:
        print(f"🎯 已实现亚毫秒级延迟！最快: {min_latency:.3f}ms")
    else:
        print(f"🔸 最快延迟: {min_latency:.3f}ms (距离1ms目标: {min_latency-1:.3f}ms)")
    
    print()
    print("优化建议等级:")
    
    if under_1ms_rate > 50:
        print("🏆 Level 5: 超越期望 - >50% 订单已达到 <1ms")
        print("   建议: 维持当前优化，考虑微秒级优化")
    elif under_1ms_rate > 20:
        print("🥇 Level 4: 接近目标 - >20% 订单达到 <1ms")
        print("   建议: 连接池优化、二进制协议")
    elif under_1ms_rate > 10:
        print("🥈 Level 3: 部分成功 - >10% 订单达到 <1ms")
        print("   建议: 网络优化、专线连接")
    elif under_1ms_rate > 1:
        print("🥉 Level 2: 偶有突破 - >1% 订单达到 <1ms")
        print("   建议: 物理位置优化、Co-location")
    else:
        print("📈 Level 1: 基础优化 - <1% 订单达到 <1ms")
        print("   建议: 全面系统重构、硬件升级")
    
    print()
    print("革命性优化路径:")
    
    # 计算理论最优延迟
    theoretical_min = 0.1  # 100微秒理论最小值（co-location + 硬件优化）
    
    potential_improvement = (avg_latency - theoretical_min) / avg_latency * 100
    
    print(f"1. 📡 网络层优化:")
    print(f"   当前平均: {avg_latency:.3f}ms")
    print(f"   Co-location目标: 0.5ms (改善 {(avg_latency-0.5)/avg_latency*100:.1f}%)")
    print(f"   理论极限: 0.1ms (改善 {potential_improvement:.1f}%)")
    
    print(f"2. 🚀 协议层优化:")
    if stats['p95_ms'] > 5:
        print(f"   二进制协议可减少 2-3ms (当前P95: {stats['p95_ms']:.3f}ms)")
    print(f"   UDP协议可减少 1-2ms")
    
    print(f"3. 💻 系统层优化:")
    print(f"   内核旁路 (DPDK): 减少 0.5-1ms")
    print(f"   CPU绑定: 减少 0.1-0.3ms")
    print(f"   内存预分配: 减少 0.1-0.2ms")
    
    print(f"4. 🏗️ 硬件层优化:")
    print(f"   FPGA加速: 减少到微秒级 (<0.1ms)")
    print(f"   专用网卡: 减少 0.2-0.5ms")
    print(f"   高频CPU: 减少 0.1-0.2ms")
    
    print("\n" + "=" * 80)
    
    # 可行性评估
    if under_1ms_rate > 10:
        feasibility = "高"
        color = "🟢"
    elif under_1ms_rate > 1:
        feasibility = "中等"
        color = "🟡"
    else:
        feasibility = "具有挑战"
        color = "🔴"
    
    print(f"{color} <1ms 延迟可行性: {feasibility}")
    print(f"   当前基础: {under_1ms_rate:.1f}% 已达到目标")
    print(f"   预期通过优化可达到: {min(under_1ms_rate * 5, 80):.1f}% <1ms")
    
    print("=" * 80)

async def main():
    print("启动 <1ms 延迟分析测试...")
    print("目标: 评估5.1套利系统革命性优化潜力")
    
    # 运行基准测试
    latencies = await test_baseline_latency(1000)
    
    if not latencies:
        print("❌ 测试失败: 无法连接到交易所模拟器")
        print("请确保运行: python3 exchange_simulator.py")
        return
    
    # 分析结果
    stats = analyze_results(latencies)
    
    # 打印详细分析
    print_optimization_analysis(stats)

if __name__ == "__main__":
    asyncio.run(main())