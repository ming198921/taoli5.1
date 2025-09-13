#!/usr/bin/env python3
import json
import matplotlib.pyplot as plt
import pandas as pd
from datetime import datetime
import numpy as np
from typing import Dict, List
import os

class ReportGenerator:
    """生成详细的延迟测试报告"""
    
    def __init__(self, data_file: str = None):
        self.data = {}
        if data_file and os.path.exists(data_file):
            with open(data_file, 'r') as f:
                self.data = json.load(f)
    
    def add_test_results(self, exchange: str, latency_records: List[Dict]):
        """添加测试结果"""
        if exchange not in self.data:
            self.data[exchange] = []
        self.data[exchange].extend(latency_records)
    
    def calculate_statistics(self, latencies: List[float]) -> Dict:
        """计算统计指标"""
        if not latencies:
            return {}
        
        latencies = sorted(latencies)
        return {
            'count': len(latencies),
            'mean': np.mean(latencies),
            'median': np.median(latencies),
            'std': np.std(latencies),
            'min': np.min(latencies),
            'max': np.max(latencies),
            'p25': np.percentile(latencies, 25),
            'p50': np.percentile(latencies, 50),
            'p75': np.percentile(latencies, 75),
            'p90': np.percentile(latencies, 90),
            'p95': np.percentile(latencies, 95),
            'p99': np.percentile(latencies, 99),
            'p999': np.percentile(latencies, 99.9)
        }
    
    def generate_text_report(self) -> str:
        """生成文本格式的报告"""
        report_lines = []
        report_lines.append("="*80)
        report_lines.append("套利系统5.1 延迟测试报告")
        report_lines.append(f"测试时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        report_lines.append("="*80)
        report_lines.append("")
        
        # 汇总表格
        report_lines.append("延迟汇总（单位：毫秒）")
        report_lines.append("-"*80)
        report_lines.append(f"{'交易所':<10} {'样本数':<8} {'平均值':<10} {'中位数':<10} {'P95':<10} {'P99':<10} {'最小值':<10} {'最大值':<10}")
        report_lines.append("-"*80)
        
        for exchange in ['binance', 'huobi', 'okex']:
            if exchange in self.data:
                latencies = [r.get('latency_ms', 0) for r in self.data[exchange]]
                if latencies:
                    stats = self.calculate_statistics(latencies)
                    report_lines.append(
                        f"{exchange.upper():<10} "
                        f"{stats['count']:<8} "
                        f"{stats['mean']:<10.2f} "
                        f"{stats['median']:<10.2f} "
                        f"{stats['p95']:<10.2f} "
                        f"{stats['p99']:<10.2f} "
                        f"{stats['min']:<10.2f} "
                        f"{stats['max']:<10.2f}"
                    )
        
        report_lines.append("-"*80)
        report_lines.append("")
        
        # 详细统计
        report_lines.append("详细延迟分布")
        report_lines.append("-"*80)
        
        for exchange in ['binance', 'huobi', 'okex']:
            if exchange in self.data:
                latencies = [r.get('latency_ms', 0) for r in self.data[exchange]]
                if latencies:
                    stats = self.calculate_statistics(latencies)
                    report_lines.append(f"\n{exchange.upper()}:")
                    report_lines.append(f"  样本数量: {stats['count']}")
                    report_lines.append(f"  平均延迟: {stats['mean']:.2f} ms")
                    report_lines.append(f"  标准差: {stats['std']:.2f} ms")
                    report_lines.append(f"  延迟分布:")
                    report_lines.append(f"    P25 (25%): {stats['p25']:.2f} ms")
                    report_lines.append(f"    P50 (中位数): {stats['p50']:.2f} ms")
                    report_lines.append(f"    P75 (75%): {stats['p75']:.2f} ms")
                    report_lines.append(f"    P90 (90%): {stats['p90']:.2f} ms")
                    report_lines.append(f"    P95 (95%): {stats['p95']:.2f} ms")
                    report_lines.append(f"    P99 (99%): {stats['p99']:.2f} ms")
                    report_lines.append(f"    P99.9 (99.9%): {stats['p999']:.2f} ms")
                    report_lines.append(f"  最小延迟: {stats['min']:.2f} ms")
                    report_lines.append(f"  最大延迟: {stats['max']:.2f} ms")
        
        report_lines.append("")
        report_lines.append("="*80)
        report_lines.append("建议:")
        report_lines.append("-"*80)
        
        # 根据测试结果提供建议
        for exchange in ['binance', 'huobi', 'okex']:
            if exchange in self.data:
                latencies = [r.get('latency_ms', 0) for r in self.data[exchange]]
                if latencies:
                    stats = self.calculate_statistics(latencies)
                    if stats['p95'] < 10:
                        report_lines.append(f"✓ {exchange.upper()}: 延迟表现优秀 (P95 < 10ms)")
                    elif stats['p95'] < 20:
                        report_lines.append(f"○ {exchange.upper()}: 延迟表现良好 (P95 < 20ms)")
                    elif stats['p95'] < 50:
                        report_lines.append(f"△ {exchange.upper()}: 延迟表现一般 (P95 < 50ms)")
                    else:
                        report_lines.append(f"✗ {exchange.upper()}: 延迟较高，建议优化网络连接 (P95 >= 50ms)")
        
        report_lines.append("="*80)
        
        return "\n".join(report_lines)
    
    def generate_charts(self, output_dir: str = "."):
        """生成图表"""
        os.makedirs(output_dir, exist_ok=True)
        
        # 1. 延迟分布直方图
        fig, axes = plt.subplots(1, 3, figsize=(15, 5))
        exchanges = ['binance', 'huobi', 'okex']
        
        for idx, exchange in enumerate(exchanges):
            if exchange in self.data:
                latencies = [r.get('latency_ms', 0) for r in self.data[exchange]]
                if latencies:
                    axes[idx].hist(latencies, bins=50, alpha=0.7, color=f'C{idx}', edgecolor='black')
                    axes[idx].set_title(f'{exchange.upper()} Latency Distribution')
                    axes[idx].set_xlabel('Latency (ms)')
                    axes[idx].set_ylabel('Frequency')
                    axes[idx].grid(True, alpha=0.3)
                    
                    # 添加统计线
                    stats = self.calculate_statistics(latencies)
                    axes[idx].axvline(stats['mean'], color='red', linestyle='--', label=f'Mean: {stats["mean"]:.1f}ms')
                    axes[idx].axvline(stats['p95'], color='orange', linestyle='--', label=f'P95: {stats["p95"]:.1f}ms')
                    axes[idx].legend()
        
        plt.tight_layout()
        plt.savefig(f'{output_dir}/latency_distribution.png', dpi=100)
        plt.close()
        
        # 2. 延迟时间序列图
        fig, ax = plt.subplots(figsize=(12, 6))
        
        for idx, exchange in enumerate(exchanges):
            if exchange in self.data:
                records = self.data[exchange]
                if records:
                    # 提取时间戳和延迟
                    timestamps = []
                    latencies = []
                    for r in records:
                        if 'timestamp' in r:
                            try:
                                timestamps.append(datetime.fromisoformat(r['timestamp']))
                                latencies.append(r.get('latency_ms', 0))
                            except:
                                pass
                    
                    if timestamps and latencies:
                        ax.plot(timestamps, latencies, label=exchange.upper(), alpha=0.7, marker='.', markersize=2)
        
        ax.set_title('Latency Over Time')
        ax.set_xlabel('Time')
        ax.set_ylabel('Latency (ms)')
        ax.legend()
        ax.grid(True, alpha=0.3)
        plt.xticks(rotation=45)
        plt.tight_layout()
        plt.savefig(f'{output_dir}/latency_timeline.png', dpi=100)
        plt.close()
        
        # 3. 延迟对比箱线图
        fig, ax = plt.subplots(figsize=(10, 6))
        
        data_for_boxplot = []
        labels = []
        
        for exchange in exchanges:
            if exchange in self.data:
                latencies = [r.get('latency_ms', 0) for r in self.data[exchange]]
                if latencies:
                    data_for_boxplot.append(latencies)
                    labels.append(exchange.upper())
        
        if data_for_boxplot:
            bp = ax.boxplot(data_for_boxplot, labels=labels, patch_artist=True)
            
            # 设置颜色
            for patch, color in zip(bp['boxes'], ['C0', 'C1', 'C2']):
                patch.set_facecolor(color)
                patch.set_alpha(0.7)
            
            ax.set_title('Latency Comparison')
            ax.set_ylabel('Latency (ms)')
            ax.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plt.savefig(f'{output_dir}/latency_boxplot.png', dpi=100)
        plt.close()
        
        print(f"Charts saved to {output_dir}/")
    
    def save_json_report(self, filename: str = None):
        """保存JSON格式的报告"""
        if filename is None:
            filename = f'report_{datetime.now().strftime("%Y%m%d_%H%M%S")}.json'
        
        report = {
            'test_time': datetime.now().isoformat(),
            'summary': {},
            'raw_data': self.data
        }
        
        for exchange in ['binance', 'huobi', 'okex']:
            if exchange in self.data:
                latencies = [r.get('latency_ms', 0) for r in self.data[exchange]]
                if latencies:
                    report['summary'][exchange] = self.calculate_statistics(latencies)
        
        with open(filename, 'w') as f:
            json.dump(report, f, indent=2, default=str)
        
        print(f"JSON report saved to {filename}")
        return filename
    
    def save_csv_report(self, filename: str = None):
        """保存CSV格式的报告"""
        if filename is None:
            filename = f'report_{datetime.now().strftime("%Y%m%d_%H%M%S")}.csv'
        
        all_records = []
        for exchange, records in self.data.items():
            for record in records:
                record_copy = record.copy()
                record_copy['exchange'] = exchange
                all_records.append(record_copy)
        
        if all_records:
            df = pd.DataFrame(all_records)
            df.to_csv(filename, index=False)
            print(f"CSV report saved to {filename}")
        
        return filename

if __name__ == "__main__":
    # 测试报告生成器
    generator = ReportGenerator()
    
    # 添加模拟数据
    import random
    for exchange in ['binance', 'huobi', 'okex']:
        records = []
        for i in range(100):
            base_latency = {'binance': 5, 'huobi': 8, 'okex': 6}[exchange]
            latency = base_latency + random.gauss(0, 2)
            records.append({
                'timestamp': datetime.now().isoformat(),
                'latency_ms': max(0.1, latency),
                'path': '/api/v3/order',
                'method': 'POST'
            })
        generator.add_test_results(exchange, records)
    
    # 生成报告
    print(generator.generate_text_report())
    generator.generate_charts()
    generator.save_json_report()
    generator.save_csv_report()