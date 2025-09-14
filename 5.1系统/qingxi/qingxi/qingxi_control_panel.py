#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
QingXi V5.1 系统控制面板 - 前端后端交互测试工具
🚀 V3+O1 数据清洗系统控制面板

功能:
- 系统启动/停止/重启控制
- 实时数据监控
- 性能指标查看
- 配置动态更新
- WebSocket连接状态监控
"""

import requests
import json
import time
import asyncio
import websockets
from datetime import datetime
from typing import Dict, List, Optional
import argparse

class QingXiControlPanel:
    def __init__(self, base_url: str = "http://localhost:50061"):
        self.base_url = base_url
        self.api_v1 = f"{base_url}/api/v1"
        
    def print_status(self, message: str, status: str = "INFO"):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        status_emoji = {
            "INFO": "ℹ️",
            "SUCCESS": "✅", 
            "ERROR": "❌",
            "WARNING": "⚠️",
            "START": "🚀",
            "STOP": "🛑",
            "RESTART": "🔄"
        }
        print(f"[{timestamp}] {status_emoji.get(status, 'ℹ️')} {message}")

    def system_start(self) -> bool:
        """启动QingXi系统"""
        try:
            self.print_status("发送系统启动命令...", "START")
            response = requests.post(f"{self.api_v1}/system/start", timeout=30)
            
            if response.status_code == 200:
                result = response.json()
                if result.get("status") == "success":
                    self.print_status(f"系统启动成功: {result.get('message')}", "SUCCESS")
                    return True
                else:
                    self.print_status(f"系统启动失败: {result.get('message')}", "ERROR")
                    return False
            else:
                self.print_status(f"HTTP错误: {response.status_code}", "ERROR")
                return False
                
        except Exception as e:
            self.print_status(f"启动命令异常: {str(e)}", "ERROR")
            return False

    def system_stop(self) -> bool:
        """停止QingXi系统"""
        try:
            self.print_status("发送系统停止命令...", "STOP")
            response = requests.post(f"{self.api_v1}/system/stop", timeout=30)
            
            if response.status_code == 200:
                result = response.json()
                if result.get("status") == "success":
                    self.print_status(f"系统停止成功: {result.get('message')}", "SUCCESS")
                    return True
                else:
                    self.print_status(f"系统停止失败: {result.get('message')}", "ERROR")
                    return False
            else:
                self.print_status(f"HTTP错误: {response.status_code}", "ERROR")
                return False
                
        except Exception as e:
            self.print_status(f"停止命令异常: {str(e)}", "ERROR")
            return False

    def system_restart(self) -> bool:
        """重启QingXi系统"""
        try:
            self.print_status("发送系统重启命令...", "RESTART")
            response = requests.post(f"{self.api_v1}/system/restart", timeout=60)
            
            if response.status_code == 200:
                result = response.json()
                if result.get("status") == "success":
                    self.print_status(f"系统重启成功: {result.get('message')}", "SUCCESS")
                    return True
                else:
                    self.print_status(f"系统重启失败: {result.get('message')}", "ERROR")
                    return False
            else:
                self.print_status(f"HTTP错误: {response.status_code}", "ERROR")
                return False
                
        except Exception as e:
            self.print_status(f"重启命令异常: {str(e)}", "ERROR")
            return False

    def get_system_status(self) -> Optional[Dict]:
        """获取系统状态"""
        try:
            response = requests.get(f"{self.api_v1}/system/status", timeout=10)
            
            if response.status_code == 200:
                return response.json()
            else:
                self.print_status(f"获取状态失败: HTTP {response.status_code}", "ERROR")
                return None
                
        except Exception as e:
            self.print_status(f"获取状态异常: {str(e)}", "ERROR")
            return None

    def get_health_status(self) -> Optional[Dict]:
        """获取健康状态"""
        try:
            response = requests.get(f"{self.api_v1}/health", timeout=10)
            
            if response.status_code == 200:
                return response.json()
            else:
                self.print_status(f"获取健康状态失败: HTTP {response.status_code}", "ERROR")
                return None
                
        except Exception as e:
            self.print_status(f"获取健康状态异常: {str(e)}", "ERROR")
            return None

    def get_exchanges(self) -> Optional[List[str]]:
        """获取活跃交易所列表"""
        try:
            response = requests.get(f"{self.api_v1}/exchanges", timeout=10)
            
            if response.status_code == 200:
                data = response.json()
                return data.get("exchanges", [])
            else:
                self.print_status(f"获取交易所列表失败: HTTP {response.status_code}", "ERROR")
                return None
                
        except Exception as e:
            self.print_status(f"获取交易所列表异常: {str(e)}", "ERROR")
            return None

    def get_performance_stats(self) -> Optional[Dict]:
        """获取V3性能统计"""
        try:
            response = requests.get(f"{self.api_v1}/v3/performance", timeout=10)
            
            if response.status_code == 200:
                return response.json()
            else:
                self.print_status(f"获取性能统计失败: HTTP {response.status_code}", "ERROR")
                return None
                
        except Exception as e:
            self.print_status(f"获取性能统计异常: {str(e)}", "ERROR")
            return None

    def update_config(self, config_path: str) -> bool:
        """动态更新配置"""
        try:
            payload = {
                "reload_from_file": True,
                "config_path": config_path
            }
            
            response = requests.post(
                f"{self.api_v1}/reconfigure", 
                json=payload,
                timeout=30
            )
            
            if response.status_code == 200:
                result = response.json()
                if result.get("status") == "success":
                    self.print_status(f"配置更新成功: {result.get('message')}", "SUCCESS")
                    return True
                else:
                    self.print_status(f"配置更新失败: {result.get('message')}", "ERROR")
                    return False
            else:
                self.print_status(f"配置更新HTTP错误: {response.status_code}", "ERROR")
                return False
                
        except Exception as e:
            self.print_status(f"配置更新异常: {str(e)}", "ERROR")
            return False

    def display_status_dashboard(self):
        """显示状态仪表板"""
        print("\n" + "="*80)
        print("🚀 QingXi V5.1 系统状态仪表板")
        print("="*80)
        
        # 系统状态
        status = self.get_system_status()
        if status:
            print(f"📊 系统状态: {status.get('status', 'Unknown')}")
            if 'health' in status:
                health = status['health']
                print(f"🏥 健康状态:")
                print(f"   - 总数据源: {health.get('total_sources', 0)}")
                print(f"   - 健康数据源: {health.get('healthy_sources', 0)}")
                print(f"   - 不健康数据源: {health.get('unhealthy_sources', 0)}")
                print(f"   - 平均延迟: {health.get('average_latency_us', 0)} μs")
        
        # 交易所列表
        exchanges = self.get_exchanges()
        if exchanges:
            print(f"🏦 活跃交易所 ({len(exchanges)}): {', '.join(exchanges)}")
        
        # 性能统计
        perf_stats = self.get_performance_stats()
        if perf_stats:
            print(f"⚡ V3性能统计:")
            for key, value in perf_stats.items():
                if isinstance(value, (int, float)):
                    print(f"   - {key}: {value}")
        
        print("="*80)

    def monitor_mode(self, interval: int = 30):
        """监控模式 - 持续显示系统状态"""
        self.print_status(f"开始监控模式，刷新间隔: {interval}秒", "INFO")
        
        try:
            while True:
                self.display_status_dashboard()
                print(f"\n⏰ 下次刷新时间: {interval}秒后 (Ctrl+C 退出)")
                time.sleep(interval)
                
        except KeyboardInterrupt:
            self.print_status("监控模式已停止", "INFO")

def main():
    parser = argparse.ArgumentParser(description="QingXi V5.1 系统控制面板")
    parser.add_argument("--url", default="http://localhost:50061", help="QingXi API基础URL")
    parser.add_argument("--action", choices=[
        "start", "stop", "restart", "status", "monitor", "config"
    ], required=True, help="要执行的操作")
    parser.add_argument("--config-path", help="配置文件路径 (用于config操作)")
    parser.add_argument("--interval", type=int, default=30, help="监控模式刷新间隔(秒)")
    
    args = parser.parse_args()
    
    control_panel = QingXiControlPanel(args.url)
    
    if args.action == "start":
        success = control_panel.system_start()
        exit(0 if success else 1)
        
    elif args.action == "stop":
        success = control_panel.system_stop()
        exit(0 if success else 1)
        
    elif args.action == "restart":
        success = control_panel.system_restart()
        exit(0 if success else 1)
        
    elif args.action == "status":
        control_panel.display_status_dashboard()
        
    elif args.action == "monitor":
        control_panel.monitor_mode(args.interval)
        
    elif args.action == "config":
        if not args.config_path:
            print("❌ 配置更新需要指定 --config-path 参数")
            exit(1)
        success = control_panel.update_config(args.config_path)
        exit(0 if success else 1)

if __name__ == "__main__":
    main() 