#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
QingXi V5.1 数据查看器
🧹 V3+O1 清洗后数据查看工具

功能:
- 查看L2清洗后数据 (cache/l2_cleaned_data/)
- 查看L3最终处理数据 (cache/l3_processed_data/)
- 数据统计分析
- 实时数据监控
"""

import os
import json
import pickle
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any
import argparse

class QingXiDataViewer:
    def __init__(self, base_path: str = "."):
        self.base_path = Path(base_path)
        self.l2_cache_dir = self.base_path / "cache" / "l2_cleaned_data"
        self.l3_cache_dir = self.base_path / "cache" / "l3_processed_data"
        self.log_dir = self.base_path / "logs" / "cache"
        
    def print_status(self, message: str, status: str = "INFO"):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        status_emoji = {
            "INFO": "ℹ️",
            "SUCCESS": "✅", 
            "ERROR": "❌",
            "WARNING": "⚠️",
            "DATA": "📊"
        }
        print(f"[{timestamp}] {status_emoji.get(status, 'ℹ️')} {message}")

    def scan_cache_directories(self):
        """扫描缓存目录"""
        print("\n" + "="*80)
        print("🧹 QingXi V5.1 数据存储位置报告")
        print("="*80)
        
        # L2 清洗后数据目录
        if self.l2_cache_dir.exists():
            l2_files = list(self.l2_cache_dir.glob("*.cache"))
            self.print_status(f"L2清洗后数据目录: {self.l2_cache_dir}", "SUCCESS")
            self.print_status(f"L2缓存文件数量: {len(l2_files)}", "DATA")
            
            if l2_files:
                # 显示最新的几个文件
                l2_files_sorted = sorted(l2_files, key=lambda x: x.stat().st_mtime, reverse=True)
                self.print_status("最新的L2清洗数据文件:", "DATA")
                for i, file in enumerate(l2_files_sorted[:5]):
                    mtime = datetime.fromtimestamp(file.stat().st_mtime)
                    size = file.stat().st_size
                    print(f"   {i+1}. {file.name} ({size} bytes, {mtime})")
        else:
            self.print_status(f"L2清洗后数据目录不存在: {self.l2_cache_dir}", "WARNING")
            
        # L3 最终处理数据目录
        if self.l3_cache_dir.exists():
            l3_files = list(self.l3_cache_dir.glob("*.cache"))
            self.print_status(f"L3最终处理数据目录: {self.l3_cache_dir}", "SUCCESS")
            self.print_status(f"L3缓存文件数量: {len(l3_files)}", "DATA")
            
            if l3_files:
                # 显示最新的几个文件
                l3_files_sorted = sorted(l3_files, key=lambda x: x.stat().st_mtime, reverse=True)
                self.print_status("最新的L3处理数据文件:", "DATA")
                for i, file in enumerate(l3_files_sorted[:5]):
                    mtime = datetime.fromtimestamp(file.stat().st_mtime)
                    size = file.stat().st_size
                    print(f"   {i+1}. {file.name} ({size} bytes, {mtime})")
        else:
            self.print_status(f"L3最终处理数据目录不存在: {self.l3_cache_dir}", "WARNING")
            
        # 日志目录
        if self.log_dir.exists():
            log_files = list(self.log_dir.glob("*.log"))
            self.print_status(f"缓存日志目录: {self.log_dir}", "SUCCESS")
            self.print_status(f"日志文件数量: {len(log_files)}", "DATA")
        else:
            self.print_status(f"缓存日志目录不存在: {self.log_dir}", "WARNING")
            
        print("="*80)

    def analyze_cache_file(self, file_path: Path) -> Dict[str, Any]:
        """分析单个缓存文件"""
        try:
            with open(file_path, 'rb') as f:
                data = f.read()
                
            # 尝试不同的反序列化方法
            try:
                # 先尝试 bincode 格式 (Rust序列化)
                # 这里我们只能分析文件大小和基本信息
                analysis = {
                    "file_name": file_path.name,
                    "file_size": len(data),
                    "creation_time": datetime.fromtimestamp(file_path.stat().st_ctime),
                    "modification_time": datetime.fromtimestamp(file_path.stat().st_mtime),
                    "data_format": "bincode (Rust)",
                    "raw_data_preview": data[:100].hex() if len(data) > 0 else "empty"
                }
                
                # 尝试检测数据中的交易所和交易对信息
                data_str = data.decode('utf-8', errors='ignore')
                exchanges = []
                symbols = []
                
                # 检测常见的交易所名称
                for exchange in ['binance', 'okx', 'bybit', 'gateio']:
                    if exchange.lower() in data_str.lower():
                        exchanges.append(exchange)
                
                # 检测常见的交易对
                for symbol in ['BTC', 'ETH', 'USDT', 'ADA', 'XRP', 'SOL']:
                    if symbol in data_str:
                        symbols.append(symbol)
                
                analysis["detected_exchanges"] = exchanges
                analysis["detected_symbols"] = symbols
                
                return analysis
                
            except Exception as e:
                return {
                    "file_name": file_path.name,
                    "file_size": len(data),
                    "error": f"Failed to analyze: {str(e)}",
                    "data_format": "unknown"
                }
                
        except Exception as e:
            return {
                "file_name": file_path.name,
                "error": f"Failed to read file: {str(e)}"
            }

    def view_cache_data(self, cache_type: str = "l2"):
        """查看缓存数据详情"""
        cache_dir = self.l2_cache_dir if cache_type == "l2" else self.l3_cache_dir
        
        if not cache_dir.exists():
            self.print_status(f"{cache_type.upper()}缓存目录不存在", "ERROR")
            return
            
        cache_files = list(cache_dir.glob("*.cache"))
        if not cache_files:
            self.print_status(f"{cache_type.upper()}缓存目录为空", "WARNING")
            return
            
        print(f"\n🔍 {cache_type.upper()}缓存数据详细分析")
        print("="*60)
        
        # 按修改时间排序
        cache_files_sorted = sorted(cache_files, key=lambda x: x.stat().st_mtime, reverse=True)
        
        total_size = 0
        exchange_stats = {}
        symbol_stats = {}
        
        for i, file_path in enumerate(cache_files_sorted[:10]):  # 只分析最新的10个文件
            analysis = self.analyze_cache_file(file_path)
            total_size += analysis.get("file_size", 0)
            
            print(f"\n📄 文件 {i+1}: {analysis['file_name']}")
            print(f"   大小: {analysis.get('file_size', 0)} bytes")
            print(f"   修改时间: {analysis.get('modification_time', 'Unknown')}")
            print(f"   数据格式: {analysis.get('data_format', 'Unknown')}")
            
            if 'detected_exchanges' in analysis:
                exchanges = analysis['detected_exchanges']
                symbols = analysis['detected_symbols']
                
                if exchanges:
                    print(f"   检测到的交易所: {', '.join(exchanges)}")
                    for exchange in exchanges:
                        exchange_stats[exchange] = exchange_stats.get(exchange, 0) + 1
                        
                if symbols:
                    print(f"   检测到的交易对: {', '.join(symbols)}")
                    for symbol in symbols:
                        symbol_stats[symbol] = symbol_stats.get(symbol, 0) + 1
            
            if 'error' in analysis:
                print(f"   ❌ 错误: {analysis['error']}")
        
        # 统计摘要
        print(f"\n📊 {cache_type.upper()}缓存统计摘要:")
        print(f"   总文件数: {len(cache_files)}")
        print(f"   总大小: {total_size / 1024:.2f} KB")
        
        if exchange_stats:
            print(f"   交易所分布: {dict(exchange_stats)}")
        if symbol_stats:
            print(f"   交易对分布: {dict(symbol_stats)}")

    def monitor_data_flow(self, interval: int = 10):
        """监控数据流"""
        self.print_status(f"开始监控数据流，检查间隔: {interval}秒", "INFO")
        
        prev_l2_count = 0
        prev_l3_count = 0
        prev_l2_size = 0
        prev_l3_size = 0
        
        try:
            while True:
                # 统计L2数据
                l2_files = list(self.l2_cache_dir.glob("*.cache")) if self.l2_cache_dir.exists() else []
                l2_count = len(l2_files)
                l2_size = sum(f.stat().st_size for f in l2_files)
                
                # 统计L3数据
                l3_files = list(self.l3_cache_dir.glob("*.cache")) if self.l3_cache_dir.exists() else []
                l3_count = len(l3_files)
                l3_size = sum(f.stat().st_size for f in l3_files)
                
                # 计算变化
                l2_count_delta = l2_count - prev_l2_count
                l3_count_delta = l3_count - prev_l3_count
                l2_size_delta = l2_size - prev_l2_size
                l3_size_delta = l3_size - prev_l3_size
                
                # 显示状态
                print(f"\n⏰ {datetime.now().strftime('%H:%M:%S')} 数据流监控:")
                print(f"   L2清洗数据: {l2_count}个文件 ({l2_size/1024:.1f}KB) [+{l2_count_delta}文件, +{l2_size_delta/1024:.1f}KB]")
                print(f"   L3处理数据: {l3_count}个文件 ({l3_size/1024:.1f}KB) [+{l3_count_delta}文件, +{l3_size_delta/1024:.1f}KB]")
                
                # 更新前值
                prev_l2_count = l2_count
                prev_l3_count = l3_count
                prev_l2_size = l2_size
                prev_l3_size = l3_size
                
                time.sleep(interval)
                
        except KeyboardInterrupt:
            self.print_status("数据流监控已停止", "INFO")

def main():
    parser = argparse.ArgumentParser(description="QingXi V5.1 数据查看器")
    parser.add_argument("--path", default=".", help="QingXi项目根目录路径")
    parser.add_argument("--action", choices=[
        "scan", "view-l2", "view-l3", "monitor"
    ], required=True, help="要执行的操作")
    parser.add_argument("--interval", type=int, default=10, help="监控模式检查间隔(秒)")
    
    args = parser.parse_args()
    
    viewer = QingXiDataViewer(args.path)
    
    if args.action == "scan":
        viewer.scan_cache_directories()
        
    elif args.action == "view-l2":
        viewer.view_cache_data("l2")
        
    elif args.action == "view-l3":
        viewer.view_cache_data("l3")
        
    elif args.action == "monitor":
        viewer.monitor_data_flow(args.interval)

if __name__ == "__main__":
    main() 