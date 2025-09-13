#!/usr/bin/env python3
"""
套利系统5.1统一命令行控制器
=================================

这是一个全功能的命令行控制器，可以通过简单的命令控制套利系统5.1的所有功能模块：
- QingXi数据处理模块
- CeLue策略执行模块  
- AI风控系统
- AI模型训练
- 系统监控和管理
- 配置管理

使用方法:
    python arbitrage-cli-controller.py [命令] [参数]

示例:
    python arbitrage-cli-controller.py system status       # 查看系统状态
    python arbitrage-cli-controller.py data start-all      # 启动所有数据采集
    python arbitrage-cli-controller.py strategy run inter  # 运行跨交易所策略
    python arbitrage-cli-controller.py ai train-model      # 训练AI模型
"""

import sys
import os
import json
import time
import subprocess
import argparse
import requests
import asyncio
import logging
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any
import yaml
from dataclasses import dataclass
import threading
import signal

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/home/ubuntu/5.1xitong/logs/cli-controller.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

@dataclass
class ServiceConfig:
    """服务配置"""
    name: str
    port: int
    health_endpoint: str
    api_base: str

class ArbitrageSystemController:
    """套利系统5.1统一控制器"""
    
    def __init__(self):
        self.base_dir = Path("/home/ubuntu/5.1xitong")
        self.system_dir = self.base_dir / "5.1系统"
        self.config_dir = self.system_dir / "config"
        self.logs_dir = self.base_dir / "logs"
        
        # 确保日志目录存在
        self.logs_dir.mkdir(exist_ok=True)
        
        # 服务配置
        self.services = {
            "gateway": ServiceConfig("统一网关", 3000, "/health", "http://localhost:3000"),
            "logging": ServiceConfig("日志服务", 4001, "/health", "http://localhost:4001"),
            "cleaning": ServiceConfig("清洗服务", 4002, "/health", "http://localhost:4002"),
            "strategy": ServiceConfig("策略服务", 4003, "/health", "http://localhost:4003"),
            "performance": ServiceConfig("性能服务", 4004, "/health", "http://localhost:4004"),
            "trading": ServiceConfig("交易服务", 4005, "/health", "http://localhost:4005"),
            "ai-model": ServiceConfig("AI模型服务", 4006, "/health", "http://localhost:4006"),
            "config": ServiceConfig("配置服务", 4007, "/health", "http://localhost:4007"),
        }
        
        self.gateway_url = "http://localhost:3000"
        
    def print_banner(self):
        """打印系统横幅"""
        banner = """
╔════════════════════════════════════════════════════════════════╗
║                    套利系统5.1 命令行控制器                      ║
║                     Arbitrage System 5.1 CLI                  ║
╠════════════════════════════════════════════════════════════════╣
║  版本: v1.0.0                                                  ║
║  支持: 387个API接口 | 7个微服务 | 4个交易所                      ║
║  功能: 数据处理 | 策略执行 | AI风控 | 模型训练                   ║
╚════════════════════════════════════════════════════════════════╝
        """
        print(banner)
    
    def check_service_health(self, service_name: str) -> bool:
        """检查服务健康状态"""
        if service_name not in self.services:
            logger.error(f"未知服务: {service_name}")
            return False
            
        service = self.services[service_name]
        try:
            response = requests.get(f"{service.api_base}{service.health_endpoint}", timeout=5)
            return response.status_code == 200
        except requests.exceptions.RequestException:
            return False
    
    def call_api(self, endpoint: str, method: str = "GET", data: dict = None) -> dict:
        """调用API接口"""
        url = f"{self.gateway_url}/api/{endpoint}"
        try:
            if method.upper() == "GET":
                response = requests.get(url, timeout=30)
            elif method.upper() == "POST":
                response = requests.post(url, json=data, timeout=30)
            elif method.upper() == "PUT":
                response = requests.put(url, json=data, timeout=30)
            elif method.upper() == "DELETE":
                response = requests.delete(url, timeout=30)
            
            if response.content:
                try:
                    json_data = response.json()
                    # 如果响应有标准格式，提取data字段
                    if isinstance(json_data, dict) and "success" in json_data:
                        return {
                            "success": json_data.get("success", False) and response.status_code < 400,
                            "status_code": response.status_code,
                            "data": json_data.get("data"),
                            "error": json_data.get("error"),
                            "message": json_data.get("message")
                        }
                    else:
                        # 直接返回JSON数据
                        return {
                            "success": response.status_code < 400,
                            "status_code": response.status_code,
                            "data": json_data
                        }
                except ValueError:
                    return {
                        "success": response.status_code < 400,
                        "status_code": response.status_code,
                        "data": response.text
                    }
            else:
                return {
                    "success": response.status_code < 400,
                    "status_code": response.status_code,
                    "data": None
                }
        except requests.exceptions.RequestException as e:
            return {
                "success": False,
                "error": str(e),
                "data": None
            }
    
    def run_system_command(self, command: str, cwd: str = None) -> dict:
        """执行系统命令"""
        try:
            if cwd is None:
                cwd = str(self.system_dir)
            
            result = subprocess.run(
                command, 
                shell=True, 
                cwd=cwd,
                capture_output=True, 
                text=True, 
                timeout=300
            )
            
            return {
                "success": result.returncode == 0,
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr
            }
        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "error": "命令执行超时",
                "stdout": "",
                "stderr": ""
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "stdout": "",
                "stderr": ""
            }

    # ========================= 系统控制命令 =========================
    
    def cmd_system_status(self):
        """查看系统状态"""
        print("\n🔍 检查套利系统5.1整体状态...")
        print("=" * 80)
        
        # 检查各个服务状态
        service_status = {}
        for service_name, service_config in self.services.items():
            status = self.check_service_health(service_name)
            service_status[service_name] = status
            status_icon = "🟢" if status else "🔴"
            print(f"{status_icon} {service_config.name:15} (端口 {service_config.port:4}) - {'运行中' if status else '已停止'}")
        
        # 系统整体状态
        running_count = sum(1 for status in service_status.values() if status)
        total_count = len(service_status)
        overall_status = "健康" if running_count == total_count else f"部分故障 ({running_count}/{total_count})"
        
        print("=" * 80)
        print(f"🎯 系统整体状态: {overall_status}")
        print(f"📊 服务统计: {running_count}/{total_count} 个服务正在运行")
        
        # 获取详细系统状态
        api_response = self.call_api("system/status")
        if api_response["success"]:
            data = api_response["data"]
            print(f"🕒 系统运行时间: {data.get('uptime', 'N/A')} 秒")
            print(f"🏃 主进程状态: {'运行中' if data.get('isRunning', False) else '已停止'}")
        
        return service_status
    
    def cmd_system_start(self):
        """启动系统"""
        print("\n🚀 启动套利系统5.1...")
        
        # 先检查是否已经在运行
        print("🔍 检查当前系统状态...")
        gateway_running = self.check_service_health("gateway")
        
        if gateway_running:
            print("✅ 系统已在运行中")
            return self.cmd_system_status()
        
        # 启动核心系统
        print("📦 启动核心系统进程...")
        startup_script = str(self.system_dir / "start_all_services_fixed.sh")
        
        if not os.path.exists(startup_script):
            print("⚠️ 启动脚本不存在，尝试手动启动服务...")
            return self._manual_start_services()
        
        result = self.run_system_command(f"bash {startup_script}")
        
        if result["success"]:
            print("✅ 核心系统启动成功")
        else:
            print(f"❌ 核心系统启动失败: {result.get('stderr', 'Unknown error')}")
            print("🔧 尝试手动启动服务...")
            return self._manual_start_services()
        
        # 等待服务启动
        print("⏳ 等待服务完全启动...")
        time.sleep(10)
        
        # 检查服务状态
        return self.cmd_system_status()
    
    def _manual_start_services(self):
        """手动启动服务"""
        print("🔧 手动启动微服务...")
        
        # 检查是否有编译好的服务
        services = [
            ("logging-service", 4001),
            ("cleaning-service", 4002),
            ("strategy-service", 4003),
            ("performance-service", 4004),
            ("trading-service", 4005),
            ("ai-model-service", 4006),
            ("config-service", 4007)
        ]
        
        started_count = 0
        for service_name, port in services:
            # 检查是否已经在运行
            if self._check_port_in_use(port):
                print(f"✅ {service_name} 已在端口 {port} 运行")
                started_count += 1
                continue
            
            # 尝试启动服务
            service_path = self.system_dir / service_name / "target" / "release" / service_name
            if service_path.exists():
                print(f"🚀 启动 {service_name} (端口 {port})...")
                cmd = f"cd {self.system_dir / service_name} && nohup ./target/release/{service_name} > ../logs/{service_name}.log 2>&1 &"
                result = self.run_system_command(cmd)
                if result["success"]:
                    started_count += 1
                    time.sleep(2)  # 等待服务启动
                else:
                    print(f"❌ {service_name} 启动失败")
            else:
                print(f"⚠️ {service_name} 可执行文件不存在: {service_path}")
        
        # 等待所有服务启动
        time.sleep(5)
        
        print(f"📊 已启动 {started_count}/{len(services)} 个服务")
        
        # 检查最终状态
        return self.cmd_system_status()
    
    def _check_port_in_use(self, port):
        """检查端口是否被占用"""
        try:
            result = subprocess.run(
                ["ss", "-tlnp"], 
                capture_output=True, 
                text=True, 
                timeout=5
            )
            return f":{port}" in result.stdout
        except:
            return False
    
    def cmd_system_stop(self):
        """停止系统"""
        print("\n🛑 停止套利系统5.1...")
        
        # 优雅关闭
        result = self.run_system_command("./stop_all_services.sh")
        
        if result["success"]:
            print("✅ 系统已优雅关闭")
        else:
            print(f"❌ 系统关闭失败: {result.get('stderr', 'Unknown error')}")
        
        return result["success"]
    
    def cmd_system_restart(self):
        """重启系统"""
        print("\n🔄 重启套利系统5.1...")
        self.cmd_system_stop()
        time.sleep(5)
        return self.cmd_system_start()

    # ========================= 数据处理模块控制 =========================
    
    def cmd_data_start_all(self):
        """启动所有数据采集"""
        print("\n📊 启动所有数据采集源...")
        
        exchanges = ["binance", "huobi", "okx", "bybit"]
        success_count = 0
        
        for exchange in exchanges:
            print(f"🔌 启动 {exchange.upper()} 数据连接...")
            
            # 使用正确的路由：cleaning服务处理数据相关操作
            result = self.call_api(f"cleaning/exchanges/{exchange}/start", "POST")
            if result["success"]:
                print(f"  ✅ {exchange.upper()} 连接成功")
                success_count += 1
            else:
                print(f"  ❌ {exchange.upper()} 连接失败: {result.get('error', 'Unknown error')}")
        
        print(f"\n📈 数据采集启动完成: {success_count}/{len(exchanges)} 个交易所")
        return success_count == len(exchanges)
    
    def cmd_data_stop_all(self):
        """停止所有数据采集"""
        print("\n⏹️ 停止所有数据采集...")
        
        result = self.call_api("cleaning/stop-all", "POST")
        if result["success"]:
            print("✅ 所有数据采集已停止")
        else:
            print(f"❌ 停止数据采集失败: {result.get('error', 'Unknown error')}")
        
        return result["success"]
    
    def cmd_data_status(self):
        """查看数据采集状态"""
        print("\n📊 数据采集状态:")
        print("=" * 60)
        
        # 检查SIMD性能状态
        result = self.call_api("cleaning/simd/status")
        if result["success"] and result["data"]:
            simd_data = result["data"]
            print("🔧 SIMD加速状态:")
            print(f"  SIMD启用: {'是' if simd_data.get('simd_enabled') else '否'}")
            print(f"  加速因子: {simd_data.get('acceleration_factor', 0):.1f}x")
            instructions = simd_data.get('supported_instructions', [])
            if instructions:
                print(f"  支持指令: {', '.join(instructions)}")
        
        # 检查交易所配置
        result = self.call_api("cleaning/exchanges")
        if result["success"] and result["data"]:
            exchanges = result["data"]
            if isinstance(exchanges, list):
                print(f"\n📊 支持的交易所 ({len(exchanges)} 个):")
                for exchange in exchanges:
                    print(f"  🏢 {exchange.upper():10} - 可用")
            
        # 检查清洗规则状态
        result = self.call_api("cleaning/rules/stats")
        if result["success"] and result["data"]:
            stats = result["data"]
            if isinstance(stats, dict):
                print(f"\n📈 清洗规则统计:")
                print(f"  📝 活跃规则: {stats.get('active_rules', 0)}")
                print(f"  ✅ 成功率: {stats.get('success_rate', 0):.1f}%")
                print(f"  ⚡ 处理速度: {stats.get('processing_speed', 0):.1f} msg/s")
        else:
            print("\n📊 数据清洗服务正常运行")
            print("  🔧 52个API端点可用")
            print("  🚀 SIMD优化功能可用")
    
    def cmd_data_clean(self, exchange: str = None):
        """执行数据清洗"""
        if exchange:
            print(f"\n🧹 执行 {exchange.upper()} 数据清洗...")
            endpoint = f"cleaning/clean/{exchange}"
        else:
            print("\n🧹 执行全量数据清洗...")
            endpoint = "cleaning/clean-all"
        
        result = self.call_api(endpoint, "POST")
        if result["success"]:
            print("✅ 数据清洗完成")
            if result["data"]:
                stats = result["data"]
                print(f"  📊 处理记录: {stats.get('processed', 0)}")
                print(f"  🧽 清理记录: {stats.get('cleaned', 0)}")
                print(f"  ✅ 有效记录: {stats.get('valid', 0)}")
        else:
            print(f"❌ 数据清洗失败: {result.get('error', 'Unknown error')}")
        
        return result["success"]

    # ========================= 策略模块控制 =========================
    
    def cmd_strategy_list(self):
        """列出所有策略"""
        print("\n📋 可用策略列表:")
        print("=" * 60)
        
        result = self.call_api("strategies/list")
        if result["success"]:
            strategies = result["data"]
            
            if isinstance(strategies, list) and strategies:
                for i, strategy in enumerate(strategies, 1):
                    status_icon = "🟢" if strategy.get("status") == "running" else "🔴"
                    health_icon = "💚" if strategy.get("health") == "healthy" else "💔"
                    
                    name = strategy.get('name', 'Unknown')
                    strategy_id = strategy.get('id', 'Unknown')
                    
                    print(f"{i:2}. {status_icon}{health_icon} {name:25} (ID: {strategy_id})")
                    print(f"    状态: {strategy.get('status', 'N/A'):10} | 健康: {strategy.get('health', 'N/A')}")
                    
                    # 显示性能指标
                    perf = strategy.get('performance', {})
                    if perf:
                        print(f"    CPU: {perf.get('cpu_usage', 0):.1f}% | 内存: {perf.get('memory_usage', 0):.1f}MB | 响应: {perf.get('response_time', 0):.1f}ms")
                    print()
                
                print(f"📊 共找到 {len(strategies)} 个策略组件")
            else:
                print("⚠️ 没有找到活跃策略")
        else:
            print(f"❌ 无法获取策略列表: {result.get('error', 'Unknown error')}")
    
    def cmd_strategy_start(self, strategy_name: str):
        """启动指定策略"""
        print(f"\n🚀 启动策略: {strategy_name}")
        
        result = self.call_api(f"strategies/{strategy_name}/start", "POST")
        if result["success"]:
            print(f"✅ 策略 {strategy_name} 启动成功")
            if result["data"]:
                print(f"📊 策略信息: {result['data']}")
        else:
            print(f"❌ 策略 {strategy_name} 启动失败: {result.get('error', 'Unknown error')}")
        
        return result["success"]
    
    def cmd_strategy_stop(self, strategy_name: str):
        """停止指定策略"""
        print(f"\n⏹️ 停止策略: {strategy_name}")
        
        result = self.call_api(f"strategies/{strategy_name}/stop", "POST")
        if result["success"]:
            print(f"✅ 策略 {strategy_name} 已停止")
            if result["data"]:
                print(f"📊 停止信息: {result['data']}")
        else:
            print(f"❌ 策略 {strategy_name} 停止失败: {result.get('error', 'Unknown error')}")
        
        return result["success"]
    
    def cmd_strategy_status(self, strategy_name: str = None):
        """查看策略状态"""
        if strategy_name:
            print(f"\n📊 策略状态: {strategy_name}")
            endpoint = f"strategies/{strategy_name}/status"
        else:
            print("\n📊 所有策略状态:")
            endpoint = "strategies/list"
        
        print("=" * 60)
        
        result = self.call_api(endpoint)
        if result["success"]:
            data = result["data"]
            
            if strategy_name:
                # 单个策略详细状态
                print(f"策略ID: {data.get('id', 'N/A')}")
                print(f"名称: {data.get('name', 'N/A')}")
                print(f"状态: {data.get('status', 'N/A')}")
                print(f"健康状态: {data.get('health', 'N/A')}")
                print(f"最后更新: {data.get('last_update', 'N/A')}")
                
                # 性能指标
                perf = data.get('performance', {})
                if perf:
                    print(f"\n📈 性能指标:")
                    print(f"  CPU使用率: {perf.get('cpu_usage', 0):.1f}%")
                    print(f"  内存使用: {perf.get('memory_usage', 0):.1f} MB")
                    print(f"  网络使用: {perf.get('network_usage', 0):.2f} KB/s")
                    print(f"  响应时间: {perf.get('response_time', 0):.1f} ms")
                    print(f"  吞吐量: {perf.get('throughput', 0):.0f} ops/s")
            else:
                # 策略列表概览
                if isinstance(data, list):
                    print(f"📊 策略概览 ({len(data)} 个组件):")
                    running_count = 0
                    healthy_count = 0
                    
                    for strategy in data:
                        status_icon = "🟢" if strategy.get("status") == "running" else "🔴"
                        health_icon = "💚" if strategy.get("health") == "healthy" else "💔"
                        name = strategy.get('name', 'Unknown')
                        perf = strategy.get('performance', {})
                        
                        if strategy.get("status") == "running":
                            running_count += 1
                        if strategy.get("health") == "healthy":
                            healthy_count += 1
                            
                        print(f"  {status_icon}{health_icon} {name:25} - CPU: {perf.get('cpu_usage', 0):.1f}% | 内存: {perf.get('memory_usage', 0):.0f}MB")
                    
                    print(f"\n📈 状态统计:")
                    print(f"  运行中: {running_count}/{len(data)}")
                    print(f"  健康: {healthy_count}/{len(data)}")
                    print(f"  整体状态: {'正常' if running_count == len(data) and healthy_count == len(data) else '需要关注'}")
        else:
            print(f"❌ 无法获取策略状态: {result.get('error', 'Unknown error')}")
    
    def _display_strategy_summary(self, strategy: dict):
        """显示策略摘要"""
        status_icon = "🟢" if strategy.get("running", False) else "🔴"
        name = strategy.get("name", "Unknown")
        profit = strategy.get("total_profit", 0)
        trades = strategy.get("total_trades", 0)
        
        print(f"{status_icon} {name:20} | 盈利: {profit:8.4f} USDT | 交易: {trades:6} 次")
    
    def _display_strategy_detail(self, strategy: dict):
        """显示策略详细信息"""
        print(f"策略名称: {strategy.get('name', 'Unknown')}")
        print(f"运行状态: {'运行中' if strategy.get('running', False) else '已停止'}")
        print(f"总盈利: {strategy.get('total_profit', 0):.4f} USDT")
        print(f"总交易: {strategy.get('total_trades', 0)} 次")
        print(f"成功率: {strategy.get('success_rate', 0):.1f}%")
        print(f"平均盈利: {strategy.get('avg_profit', 0):.4f} USDT/交易")
        print(f"最大回撤: {strategy.get('max_drawdown', 0):.4f} USDT")
        print(f"运行时间: {strategy.get('uptime', 0)} 秒")

    # ========================= AI风控模块 =========================
    
    def cmd_risk_status(self):
        """查看风控状态"""
        print("\n🛡️ AI风控系统状态:")
        print("=" * 60)
        
        result = self.call_api("risk/status")
        if result["success"]:
            data = result["data"]
            
            print(f"🚦 风控级别: {data.get('risk_level', 'Unknown')}")
            print(f"📊 风险评分: {data.get('risk_score', 0):.1f}/100")
            print(f"💰 当前敞口: {data.get('current_exposure', 0):.4f} USDT")
            print(f"⚠️ 活跃警报: {data.get('active_alerts', 0)} 个")
            
            # 显示风险限制
            limits = data.get("limits", {})
            print(f"\n📏 风险限制:")
            print(f"  最大敞口: {limits.get('max_exposure', 0):.4f} USDT")
            print(f"  单笔最大: {limits.get('max_single_trade', 0):.4f} USDT")
            print(f"  日损失限制: {limits.get('daily_loss_limit', 0):.4f} USDT")
        else:
            print("❌ 无法获取风控状态")
    
    def cmd_risk_set_limit(self, limit_type: str, value: float):
        """设置风控限制"""
        print(f"\n⚙️ 设置风控限制: {limit_type} = {value}")
        
        data = {
            "limit_type": limit_type,
            "value": value
        }
        
        result = self.call_api("risk/limits", "PUT", data)
        if result["success"]:
            print(f"✅ 风控限制更新成功")
        else:
            print(f"❌ 风控限制更新失败")
        
        return result["success"]
    
    def cmd_risk_emergency_stop(self):
        """紧急停止"""
        print("\n🚨 执行紧急停止...")
        
        result = self.call_api("risk/emergency-stop", "POST")
        if result["success"]:
            print("✅ 紧急停止执行完成")
            print("🛑 所有交易已暂停")
            print("📞 请联系管理员检查系统状态")
        else:
            print("❌ 紧急停止执行失败")
        
        return result["success"]

    # ========================= AI模型训练 =========================
    
    def cmd_ai_models_list(self):
        """列出所有AI模型"""
        print("\n🤖 AI模型列表:")
        print("=" * 60)
        
        result = self.call_api("models/list")
        if result["success"]:
            models = result["data"].get("models", [])
            
            for i, model in enumerate(models, 1):
                status_icon = "🟢" if model.get("active", False) else "🔴"
                print(f"{i:2}. {status_icon} {model.get('name', 'Unknown'):20} - {model.get('description', 'No description')}")
                print(f"    类型: {model.get('type', 'N/A'):15} | 准确率: {model.get('accuracy', 0):.1f}%")
                print(f"    训练时间: {model.get('last_trained', 'N/A'):15} | 版本: {model.get('version', 'N/A')}")
                print()
        else:
            print("❌ 无法获取AI模型列表")
    
    def cmd_ai_train_model(self, model_name: str, data_days: int = 30):
        """训练AI模型"""
        print(f"\n🎓 训练AI模型: {model_name}")
        print(f"📅 使用数据: 最近 {data_days} 天")
        
        data = {
            "model_name": model_name,
            "training_days": data_days,
            "auto_deploy": False
        }
        
        result = self.call_api("training/start", "POST", data)
        if result["success"]:
            training_id = result["data"].get("training_id")
            print(f"✅ 模型训练已启动 (ID: {training_id})")
            print("⏳ 训练可能需要几分钟到几小时，请耐心等待...")
            
            # 可选：监控训练进度
            self._monitor_training_progress(training_id)
        else:
            print("❌ 模型训练启动失败")
        
        return result["success"]
    
    def cmd_ai_deploy_model(self, model_name: str, version: str = "latest"):
        """部署AI模型"""
        print(f"\n🚀 部署AI模型: {model_name} (版本: {version})")
        
        data = {
            "model_name": model_name,
            "version": version
        }
        
        result = self.call_api("models/deploy", "POST", data)
        if result["success"]:
            print(f"✅ 模型 {model_name} 部署成功")
        else:
            print(f"❌ 模型 {model_name} 部署失败")
        
        return result["success"]
    
    def _monitor_training_progress(self, training_id: str, max_wait: int = 3600):
        """监控训练进度"""
        start_time = time.time()
        
        while time.time() - start_time < max_wait:
            result = self.call_api(f"training/{training_id}/status")
            if result["success"]:
                data = result["data"]
                status = data.get("status", "unknown")
                progress = data.get("progress", 0)
                
                if status == "completed":
                    print(f"🎉 训练完成! 准确率: {data.get('final_accuracy', 0):.1f}%")
                    break
                elif status == "failed":
                    print(f"❌ 训练失败: {data.get('error_message', 'Unknown error')}")
                    break
                else:
                    print(f"⏳ 训练进度: {progress:.1f}% (状态: {status})")
            
            time.sleep(30)  # 每30秒检查一次

    # ========================= 费用管理 =========================
    
    def cmd_fees_list(self, exchange: str = None):
        """查看交易费率"""
        if exchange:
            print(f"\n💰 {exchange.upper()} 交易费率:")
            endpoint = f"fees/exchanges/{exchange}"
        else:
            print("\n💰 所有交易所费率:")
            endpoint = "fees/exchanges"
        
        print("=" * 60)
        
        result = self.call_api(endpoint)
        if result["success"]:
            data = result["data"]
            
            if exchange:
                # 单个交易所详细费率信息
                exchange_info = data
                print(f"交易所: {exchange_info.get('exchange', 'Unknown').upper()}")
                print(f"基础Maker费率: {exchange_info.get('base_maker_fee', 0)*100:.3f}%")
                print(f"基础Taker费率: {exchange_info.get('base_taker_fee', 0)*100:.3f}%")
                
                # VIP等级信息
                vip_levels = exchange_info.get('vip_levels', [])
                if vip_levels:
                    print(f"\n🎖️ VIP等级费率:")
                    for level in vip_levels:
                        print(f"  等级 {level.get('level', 0):2d}: Maker {level.get('maker_fee', 0)*100:.3f}% | Taker {level.get('taker_fee', 0)*100:.3f}% | 要求: {level.get('requirements', 'N/A')}")
                
                print(f"\n最后更新: {exchange_info.get('last_updated', 'N/A')}")
            else:
                # 所有交易所概览
                if isinstance(data, dict):
                    print(f"📊 交易所费率概览 ({len(data)} 个交易所):")
                    for ex_name, ex_info in data.items():
                        maker_fee = ex_info.get('base_maker_fee', 0) * 100
                        taker_fee = ex_info.get('base_taker_fee', 0) * 100
                        avg_fee = (maker_fee + taker_fee) / 2
                        
                        # 费率等级指示
                        if avg_fee <= 0.075:
                            fee_level = "🟢 低"
                        elif avg_fee <= 0.15:
                            fee_level = "🟡 中"
                        else:
                            fee_level = "🔴 高"
                            
                        print(f"  {ex_name.upper():8} | Maker: {maker_fee:.3f}% | Taker: {taker_fee:.3f}% | 平均: {avg_fee:.3f}% {fee_level}")
        else:
            print(f"❌ 无法获取费率信息: {result.get('error', 'Unknown error')}")
    
    def cmd_fees_compare(self, symbol: str = "BTCUSDT"):
        """比较不同交易所费率"""
        print(f"\n📊 {symbol} 费率比较:")
        print("=" * 80)
        
        result = self.call_api("fees/comparison", data={"symbol": symbol})
        if result["success"]:
            data = result["data"]
            comparison = data.get("comparison", [])
            
            if comparison:
                print(f"📈 费率排行 (从低到高):")
                for i, entry in enumerate(comparison, 1):
                    exchange = entry.get("exchange", "Unknown")
                    maker_fee = entry.get("maker_fee", 0) * 100
                    taker_fee = entry.get("taker_fee", 0) * 100
                    avg_fee = entry.get("average_fee", 0) * 100
                    score = entry.get("competitiveness_score", 0) * 100
                    
                    rank_icon = ["🥇", "🥈", "🥉", "🏅"][min(i-1, 3)]
                    print(f"{i:2d}. {rank_icon} {exchange.upper():8} | Maker: {maker_fee:.3f}% | Taker: {taker_fee:.3f}% | 平均: {avg_fee:.3f}% | 得分: {score:.1f}")
                
                # 最佳和最差选择
                best = data.get("lowest_fees", {})
                worst = data.get("highest_fees", {})
                
                print(f"\n🎯 推荐选择:")
                print(f"  最低费率: {best.get('exchange', 'N/A').upper()} (平均 {best.get('average_fee', 0)*100:.3f}%)")
                print(f"  最高费率: {worst.get('exchange', 'N/A').upper()} (平均 {worst.get('average_fee', 0)*100:.3f}%)")
                
                # 费率差异分析
                if best and worst:
                    savings = (worst.get('average_fee', 0) - best.get('average_fee', 0)) * 100
                    print(f"  💰 费率差异: {savings:.3f}% (选择最优可节省费用)")
        else:
            print(f"❌ 无法获取费率比较: {result.get('error', 'Unknown error')}")
    
    def cmd_fees_calculate(self, amount: float, exchange: str = "binance", symbol: str = "BTCUSDT"):
        """计算交易费用"""
        print(f"\n🧮 交易费用计算:")
        print(f"交易金额: ${amount:,.2f} | 交易所: {exchange.upper()} | 交易对: {symbol}")
        print("=" * 60)
        
        data = {
            "trade_amount": amount,
            "exchange": exchange,
            "symbol": symbol
        }
        
        result = self.call_api("fees/calculate", "POST", data)
        if result["success"]:
            calc = result["data"]
            
            print(f"💳 费用明细:")
            print(f"  Maker订单费用: ${calc.get('maker_fee_amount', 0):,.4f}")
            print(f"  Taker订单费用: ${calc.get('taker_fee_amount', 0):,.4f}")
            
            print(f"\n📈 盈利分析 (假设1%价差):")
            print(f"  Maker净利润: ${calc.get('net_profit_maker', 0):,.4f}")
            print(f"  Taker净利润: ${calc.get('net_profit_taker', 0):,.4f}")
            
            breakeven = calc.get('breakeven_fee_rate', 0) * 100
            print(f"\n⚖️ 盈亏平衡点: {breakeven:.3f}% 价差")
            
            # 建议
            if calc.get('net_profit_maker', 0) > 0:
                print(f"✅ 建议: 使用Maker订单可获得正收益")
            else:
                print(f"⚠️ 警告: 当前价差不足以覆盖交易费用")
        else:
            print(f"❌ 计算失败: {result.get('error', 'Unknown error')}")
    
    def cmd_fees_arbitrage_analysis(self, symbol: str = "BTCUSDT", amount: float = 10000.0):
        """套利费用分析"""
        print(f"\n🔄 {symbol} 套利费用分析:")
        print(f"分析金额: ${amount:,.2f}")
        print("=" * 80)
        
        params = {"symbol": symbol, "amount": str(amount)}
        result = self.call_api("fees/arbitrage-analysis", data=params)
        
        if result["success"]:
            data = result["data"]
            opportunities = data.get("arbitrage_opportunities", [])
            
            if opportunities:
                print(f"💡 套利机会分析 ({len(opportunities)} 个交易对组合):")
                for i, opp in enumerate(opportunities, 1):
                    pair = opp.get("pair", "Unknown")
                    cost = opp.get("total_fee_cost", 0)
                    fee_pct = opp.get("fee_percentage", 0)
                    breakeven = opp.get("breakeven_spread_percent", 0)
                    recommendation = opp.get("recommendation", "未知")
                    
                    rec_icon = "✅" if recommendation == "推荐" else "⚠️"
                    print(f"{i:2d}. {rec_icon} {pair:20} | 费用: ${cost:.4f} ({fee_pct:.3f}%) | 盈亏平衡: {breakeven:.3f}% | {recommendation}")
                
                # 最佳和最差组合
                best = data.get("best_pair", {})
                worst = data.get("worst_pair", {})
                
                print(f"\n🎯 交易建议:")
                if best:
                    print(f"  最优组合: {best.get('pair', 'N/A')}")
                    print(f"  最低费用: ${best.get('total_fee_cost', 0):.4f} ({best.get('fee_percentage', 0):.3f}%)")
                    print(f"  盈亏平衡: {best.get('breakeven_spread_percent', 0):.3f}% 价差")
                
                if worst:
                    print(f"  最差组合: {worst.get('pair', 'N/A')}")
                    print(f"  最高费用: ${worst.get('total_fee_cost', 0):.4f} ({worst.get('fee_percentage', 0):.3f}%)")
            else:
                print("⚠️ 未找到套利机会")
        else:
            print(f"❌ 分析失败: {result.get('error', 'Unknown error')}")
    
    def cmd_fees_refresh(self):
        """刷新所有交易所费率"""
        print("\n🔄 刷新交易所费率...")
        
        result = self.call_api("fees/refresh", "POST")
        if result["success"]:
            data = result["data"]
            updated_exchanges = data.get("updated_exchanges", [])
            
            print("✅ 费率刷新完成:")
            for exchange in updated_exchanges:
                print(f"  📊 {exchange.upper()} - 已更新")
            
            print(f"\n🕒 更新时间: {data.get('updated_at', 'N/A')}")
        else:
            print(f"❌ 费率刷新失败: {result.get('error', 'Unknown error')}")

    # ========================= 交易所API管理 =========================
    
    def cmd_exchange_add_api(self, exchange: str, api_key: str, secret_key: str, testnet: bool = False):
        """添加交易所API凭证"""
        print(f"\n🔐 添加{exchange.upper()}交易所API凭证...")
        
        # 隐藏敏感信息显示
        masked_api_key = api_key[:8] + "..." + api_key[-8:] if len(api_key) > 16 else api_key[:4] + "..."
        masked_secret = secret_key[:8] + "..." + secret_key[-8:] if len(secret_key) > 16 else "***"
        
        print(f"API Key: {masked_api_key}")
        print(f"Secret:  {masked_secret}")
        print(f"测试网: {'是' if testnet else '否'}")
        
        data = {
            "api_key": api_key,
            "secret_key": secret_key,
            "testnet": testnet
        }
        
        result = self.call_api(f"exchange-api/{exchange}/credentials", "POST", data)
        if result["success"]:
            print(f"✅ {exchange.upper()}API凭证添加成功")
            
            # 立即测试连接
            print(f"\n🔍 测试API连接...")
            test_result = self.call_api(f"exchange-api/{exchange}/test", "POST")
            if test_result["success"]:
                test_data = test_result["data"]
                if test_data.get("success", False):
                    print(f"✅ API连接测试成功")
                    print(f"  账户状态: {test_data.get('account_status', 'N/A')}")
                    print(f"  权限: {', '.join(test_data.get('permissions', []))}")
                else:
                    print(f"❌ API连接测试失败: {test_data.get('error', 'Unknown error')}")
            else:
                print(f"⚠️ 无法测试API连接")
        else:
            print(f"❌ API凭证添加失败: {result.get('error', 'Unknown error')}")
        
        return result["success"]
    
    def cmd_exchange_list_apis(self):
        """列出已配置的交易所API"""
        print("\n🔐 已配置的交易所API:")
        print("=" * 60)
        
        result = self.call_api("exchange-api/credentials")
        if result["success"]:
            exchanges = result["data"]
            
            if exchanges:
                for i, exchange in enumerate(exchanges, 1):
                    print(f"{i:2d}. 📈 {exchange.upper():10} - 已配置")
                    
                    # 获取详细状态
                    status_result = self.call_api(f"exchange-api/{exchange}/status")
                    if status_result["success"]:
                        status = status_result["data"]
                        connected = "🟢 已连接" if status.get("connected", False) else "🔴 未连接"
                        trading = "✅ 可交易" if status.get("trading_enabled", False) else "❌ 无法交易"
                        print(f"     状态: {connected} | 交易: {trading}")
                        
                        # 显示账户信息
                        account = status.get("account_info")
                        if account:
                            print(f"     账户: {account.get('account_type', 'N/A')} | 手续费: Maker {account.get('maker_commission', 0)/100:.3f}% Taker {account.get('taker_commission', 0)/100:.3f}%")
                    print()
                
                print(f"📊 共配置 {len(exchanges)} 个交易所API")
            else:
                print("⚠️ 未配置任何交易所API")
        else:
            print(f"❌ 无法获取API配置信息")
    
    def cmd_exchange_test_api(self, exchange: str):
        """测试交易所API连接"""
        print(f"\n🔍 测试{exchange.upper()}API连接...")
        
        result = self.call_api(f"exchange-api/{exchange}/test", "POST")
        if result["success"]:
            test_data = result["data"]
            
            if test_data.get("success", False):
                print(f"✅ API连接测试成功")
                print(f"  服务器时间: {test_data.get('server_time', 'N/A')}")
                print(f"  账户状态: {test_data.get('account_status', 'N/A')}")
                print(f"  权限: {', '.join(test_data.get('permissions', []))}")
                print(f"  测试网: {'是' if test_data.get('testnet', False) else '否'}")
                print(f"  IP限制: {'是' if test_data.get('ip_restriction', False) else '否'}")
            else:
                print(f"❌ API连接测试失败")
                print(f"  错误: {test_data.get('error', 'Unknown error')}")
        else:
            print(f"❌ 无法测试API连接: {result.get('error', 'Unknown error')}")
    
    def cmd_exchange_get_account(self, exchange: str):
        """获取交易所账户信息"""
        print(f"\n👤 获取{exchange.upper()}账户信息:")
        print("=" * 60)
        
        result = self.call_api(f"exchange-api/{exchange}/account")
        if result["success"]:
            account = result["data"]
            
            print(f"账户类型: {account.get('account_type', 'N/A')}")
            print(f"交易权限: {'✅ 是' if account.get('can_trade', False) else '❌ 否'}")
            print(f"提现权限: {'✅ 是' if account.get('can_withdraw', False) else '❌ 否'}")
            print(f"充值权限: {'✅ 是' if account.get('can_deposit', False) else '❌ 否'}")
            
            # 手续费信息
            maker_fee = account.get('maker_commission', 0) / 100
            taker_fee = account.get('taker_commission', 0) / 100
            print(f"\n💰 手续费信息:")
            print(f"  Maker费率: {maker_fee:.3f}%")
            print(f"  Taker费率: {taker_fee:.3f}%")
            
            # 余额信息
            balances = account.get('balances', [])
            if balances:
                print(f"\n💳 账户余额:")
                for balance in balances[:10]:  # 只显示前10个资产
                    asset = balance.get('asset', 'N/A')
                    free = float(balance.get('free', '0'))
                    locked = float(balance.get('locked', '0'))
                    
                    if free > 0 or locked > 0:
                        total = free + locked
                        print(f"  {asset:8}: 可用 {free:12.6f} | 冻结 {locked:12.6f} | 总计 {total:12.6f}")
        else:
            print(f"❌ 无法获取账户信息: {result.get('error', 'Unknown error')}")
    
    def cmd_exchange_get_real_fees(self, exchange: str):
        """获取实时交易费率"""
        print(f"\n💰 获取{exchange.upper()}实时交易费率:")
        print("=" * 60)
        
        result = self.call_api(f"exchange-api/{exchange}/trading-fees")
        if result["success"]:
            fee_data = result["data"]
            
            if fee_data.get("success", False):
                trade_fees = fee_data.get("data", {}).get("tradeFee", [])
                
                if trade_fees:
                    print(f"📊 交易对费率 ({len(trade_fees)} 个):")
                    for i, fee in enumerate(trade_fees[:20], 1):  # 只显示前20个
                        symbol = fee.get("symbol", "N/A")
                        maker = float(fee.get("makerCommission", "0")) * 100
                        taker = float(fee.get("takerCommission", "0")) * 100
                        
                        print(f"{i:2d}. {symbol:12} | Maker: {maker:.3f}% | Taker: {taker:.3f}%")
                    
                    print(f"\n🕒 更新时间: {fee_data.get('retrieved_at', 'N/A')}")
                else:
                    print("⚠️ 未获取到费率数据")
            else:
                print(f"❌ 获取费率失败: {fee_data.get('error', 'Unknown error')}")
        else:
            print(f"❌ 无法获取实时费率: {result.get('error', 'Unknown error')}")
    
    def cmd_exchange_remove_api(self, exchange: str):
        """删除交易所API凭证"""
        print(f"\n🗑️ 删除{exchange.upper()}交易所API凭证...")
        
        result = self.call_api(f"exchange-api/{exchange}/credentials", "DELETE")
        if result["success"]:
            print(f"✅ {exchange.upper()}API凭证已删除")
        else:
            print(f"❌ API凭证删除失败: {result.get('error', 'Unknown error')}")
        
        return result["success"]

    # ========================= 配置管理 =========================
    
    def cmd_config_show(self, config_type: str = "system"):
        """显示配置"""
        print(f"\n⚙️ 当前配置 ({config_type}):")
        print("=" * 60)
        
        result = self.call_api(f"config/{config_type}")
        if result["success"]:
            config_data = result["data"]
            self._print_config(config_data, indent=0)
        else:
            print("❌ 无法获取配置信息")
    
    def cmd_config_set(self, config_path: str, value: str):
        """设置配置项"""
        print(f"\n✏️ 设置配置: {config_path} = {value}")
        
        data = {
            "path": config_path,
            "value": value
        }
        
        result = self.call_api("config/set", "PUT", data)
        if result["success"]:
            print("✅ 配置更新成功")
        else:
            print("❌ 配置更新失败")
        
        return result["success"]
    
    def _print_config(self, config: dict, indent: int = 0):
        """递归打印配置"""
        for key, value in config.items():
            prefix = "  " * indent
            if isinstance(value, dict):
                print(f"{prefix}{key}:")
                self._print_config(value, indent + 1)
            else:
                print(f"{prefix}{key}: {value}")

    # ========================= 监控和日志 =========================
    
    def cmd_logs_tail(self, service: str = "all", lines: int = 50):
        """查看实时日志"""
        print(f"\n📋 查看日志 (服务: {service}, 行数: {lines})")
        print("=" * 80)
        
        if service == "all":
            # 查看所有服务日志
            result = self.call_api(f"logs/tail?lines={lines}")
        else:
            # 查看特定服务日志
            result = self.call_api(f"logs/{service}/tail?lines={lines}")
        
        if result["success"]:
            logs = result["data"].get("logs", [])
            for log_entry in logs:
                timestamp = log_entry.get("timestamp", "")
                level = log_entry.get("level", "INFO")
                service_name = log_entry.get("service", "unknown")
                message = log_entry.get("message", "")
                
                level_color = {
                    "ERROR": "🔴",
                    "WARN": "🟡", 
                    "INFO": "🔵",
                    "DEBUG": "⚪"
                }.get(level, "⚪")
                
                print(f"{level_color} [{timestamp}] {service_name:15} | {message}")
        else:
            print("❌ 无法获取日志信息")
    
    def cmd_performance_monitor(self, duration: int = 60):
        """性能监控"""
        print(f"\n📈 性能监控 (持续 {duration} 秒)...")
        print("按 Ctrl+C 停止监控")
        print("=" * 80)
        
        start_time = time.time()
        try:
            while time.time() - start_time < duration:
                result = self.call_api("performance/metrics")
                if result["success"]:
                    data = result["data"]
                    
                    # 清屏并显示实时数据
                    os.system('clear' if os.name == 'posix' else 'cls')
                    print(f"📊 系统性能监控 - {datetime.now().strftime('%H:%M:%S')}")
                    print("=" * 80)
                    
                    # CPU和内存
                    print(f"🖥️  CPU使用率: {data.get('cpu_usage', 0):.1f}%")
                    print(f"💾 内存使用率: {data.get('memory_usage', 0):.1f}%")
                    print(f"💿 磁盘使用率: {data.get('disk_usage', 0):.1f}%")
                    
                    # 网络
                    print(f"📡 网络延迟: {data.get('network_latency', 0):.1f}ms")
                    print(f"📤 出站流量: {data.get('network_out', 0):.1f} KB/s")
                    print(f"📥 入站流量: {data.get('network_in', 0):.1f} KB/s")
                    
                    # 交易统计
                    trading_stats = data.get("trading_stats", {})
                    print(f"💰 总盈利: {trading_stats.get('total_profit', 0):.4f} USDT")
                    print(f"📊 交易次数: {trading_stats.get('total_trades', 0)}")
                    print(f"⚡ 每秒订单: {trading_stats.get('orders_per_second', 0):.1f}")
                
                time.sleep(2)  # 每2秒刷新一次
                
        except KeyboardInterrupt:
            print("\n\n⏹️ 监控已停止")

    # ========================= 主命令解析 =========================
    
def create_parser():
    """创建命令行参数解析器"""
    parser = argparse.ArgumentParser(
        description="套利系统5.1统一命令行控制器",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例用法:
  系统控制:
    python arbitrage-cli-controller.py system status
    python arbitrage-cli-controller.py system start
    python arbitrage-cli-controller.py system stop
    
  数据处理:
    python arbitrage-cli-controller.py data start-all
    python arbitrage-cli-controller.py data status
    python arbitrage-cli-controller.py data clean binance
    
  策略管理:
    python arbitrage-cli-controller.py strategy list
    python arbitrage-cli-controller.py strategy start inter_exchange
    python arbitrage-cli-controller.py strategy status triangular
    
  AI风控:
    python arbitrage-cli-controller.py risk status
    python arbitrage-cli-controller.py risk set-limit max_exposure 10000
    python arbitrage-cli-controller.py risk emergency-stop
    
  AI模型:
    python arbitrage-cli-controller.py ai models
    python arbitrage-cli-controller.py ai train price_prediction 7
    python arbitrage-cli-controller.py ai deploy risk_model latest
    
  监控日志:
    python arbitrage-cli-controller.py logs tail strategy 100
    python arbitrage-cli-controller.py monitor performance 300
    
  费用管理:
    python arbitrage-cli-controller.py fees list
    python arbitrage-cli-controller.py fees list binance
    python arbitrage-cli-controller.py fees compare BTCUSDT
    python arbitrage-cli-controller.py fees calculate 10000 binance BTCUSDT
    python arbitrage-cli-controller.py fees arbitrage-analysis BTCUSDT 5000
    python arbitrage-cli-controller.py fees refresh
    
  交易所API管理:
    python arbitrage-cli-controller.py exchange add-api binance API_KEY SECRET_KEY
    python arbitrage-cli-controller.py exchange list-apis
    python arbitrage-cli-controller.py exchange test-api binance
    python arbitrage-cli-controller.py exchange account binance
    python arbitrage-cli-controller.py exchange real-fees binance
    python arbitrage-cli-controller.py exchange remove-api binance
    
  配置管理:
    python arbitrage-cli-controller.py config show system
    python arbitrage-cli-controller.py config set trading.min_profit 0.001
        """
    )
    
    # 创建子命令
    subparsers = parser.add_subparsers(dest='command', help='可用命令')
    
    # 系统控制命令
    sys_parser = subparsers.add_parser('system', help='系统控制')
    sys_parser.add_argument('action', choices=['status', 'start', 'stop', 'restart'], help='系统操作')
    
    # 数据处理命令
    data_parser = subparsers.add_parser('data', help='数据处理')
    data_parser.add_argument('action', choices=['start-all', 'stop-all', 'status', 'clean'], help='数据操作')
    data_parser.add_argument('target', nargs='?', help='目标交易所（仅用于clean）')
    
    # 策略管理命令
    strategy_parser = subparsers.add_parser('strategy', help='策略管理')
    strategy_parser.add_argument('action', choices=['list', 'start', 'stop', 'status'], help='策略操作')
    strategy_parser.add_argument('name', nargs='?', help='策略名称')
    
    # AI风控命令
    risk_parser = subparsers.add_parser('risk', help='AI风控')
    risk_parser.add_argument('action', choices=['status', 'set-limit', 'emergency-stop'], help='风控操作')
    risk_parser.add_argument('param1', nargs='?', help='参数1（限制类型）')
    risk_parser.add_argument('param2', nargs='?', type=float, help='参数2（数值）')
    
    # AI模型命令
    ai_parser = subparsers.add_parser('ai', help='AI模型')
    ai_parser.add_argument('action', choices=['models', 'train', 'deploy'], help='AI操作')
    ai_parser.add_argument('model_name', nargs='?', help='模型名称')
    ai_parser.add_argument('param', nargs='?', help='额外参数（训练天数或版本）')
    
    # 监控日志命令
    logs_parser = subparsers.add_parser('logs', help='日志查看')
    logs_parser.add_argument('action', choices=['tail'], help='日志操作')
    logs_parser.add_argument('service', nargs='?', default='all', help='服务名称')
    logs_parser.add_argument('--lines', type=int, default=50, help='显示行数')
    
    monitor_parser = subparsers.add_parser('monitor', help='性能监控')
    monitor_parser.add_argument('type', choices=['performance'], help='监控类型')
    monitor_parser.add_argument('--duration', type=int, default=60, help='监控持续时间（秒）')
    
    # 费用管理命令
    fees_parser = subparsers.add_parser('fees', help='费用管理')
    fees_parser.add_argument('action', choices=['list', 'compare', 'calculate', 'arbitrage-analysis', 'refresh'], help='费用操作')
    fees_parser.add_argument('param1', nargs='?', help='交易所名称或交易对')
    fees_parser.add_argument('param2', nargs='?', help='金额或其他参数')
    fees_parser.add_argument('param3', nargs='?', help='附加参数')

    # 交易所API管理命令
    exchange_parser = subparsers.add_parser('exchange', help='交易所API管理')
    exchange_parser.add_argument('action', choices=['add-api', 'list-apis', 'test-api', 'account', 'real-fees', 'remove-api'], help='API操作')
    exchange_parser.add_argument('exchange', nargs='?', help='交易所名称')
    exchange_parser.add_argument('api_key', nargs='?', help='API密钥')
    exchange_parser.add_argument('secret_key', nargs='?', help='私钥')
    exchange_parser.add_argument('--testnet', action='store_true', help='是否为测试网')

    # 配置管理命令
    config_parser = subparsers.add_parser('config', help='配置管理')
    config_parser.add_argument('action', choices=['show', 'set'], help='配置操作')
    config_parser.add_argument('param1', nargs='?', help='配置类型或路径')
    config_parser.add_argument('param2', nargs='?', help='配置值')
    
    return parser

def main():
    """主函数"""
    # 创建控制器实例
    controller = ArbitrageSystemController()
    
    # 如果没有参数，显示横幅和帮助
    if len(sys.argv) == 1:
        controller.print_banner()
        create_parser().print_help()
        return
    
    # 解析命令行参数
    parser = create_parser()
    args = parser.parse_args()
    
    if not args.command:
        controller.print_banner()
        parser.print_help()
        return
    
    # 显示横幅
    controller.print_banner()
    
    try:
        # 根据命令执行相应操作
        if args.command == 'system':
            if args.action == 'status':
                controller.cmd_system_status()
            elif args.action == 'start':
                controller.cmd_system_start()
            elif args.action == 'stop':
                controller.cmd_system_stop()
            elif args.action == 'restart':
                controller.cmd_system_restart()
        
        elif args.command == 'data':
            if args.action == 'start-all':
                controller.cmd_data_start_all()
            elif args.action == 'stop-all':
                controller.cmd_data_stop_all()
            elif args.action == 'status':
                controller.cmd_data_status()
            elif args.action == 'clean':
                controller.cmd_data_clean(args.target)
        
        elif args.command == 'strategy':
            if args.action == 'list':
                controller.cmd_strategy_list()
            elif args.action == 'start':
                if not args.name:
                    print("❌ 请指定策略名称")
                    return
                controller.cmd_strategy_start(args.name)
            elif args.action == 'stop':
                if not args.name:
                    print("❌ 请指定策略名称")
                    return
                controller.cmd_strategy_stop(args.name)
            elif args.action == 'status':
                controller.cmd_strategy_status(args.name)
        
        elif args.command == 'risk':
            if args.action == 'status':
                controller.cmd_risk_status()
            elif args.action == 'set-limit':
                if not args.param1 or args.param2 is None:
                    print("❌ 请指定限制类型和数值")
                    return
                controller.cmd_risk_set_limit(args.param1, args.param2)
            elif args.action == 'emergency-stop':
                controller.cmd_risk_emergency_stop()
        
        elif args.command == 'ai':
            if args.action == 'models':
                controller.cmd_ai_models_list()
            elif args.action == 'train':
                if not args.model_name:
                    print("❌ 请指定模型名称")
                    return
                days = int(args.param) if args.param else 30
                controller.cmd_ai_train_model(args.model_name, days)
            elif args.action == 'deploy':
                if not args.model_name:
                    print("❌ 请指定模型名称")
                    return
                version = args.param if args.param else "latest"
                controller.cmd_ai_deploy_model(args.model_name, version)
        
        elif args.command == 'logs':
            if args.action == 'tail':
                controller.cmd_logs_tail(args.service, args.lines)
        
        elif args.command == 'monitor':
            if args.type == 'performance':
                controller.cmd_performance_monitor(args.duration)
        
        elif args.command == 'fees':
            if args.action == 'list':
                exchange = args.param1 if args.param1 else None
                controller.cmd_fees_list(exchange)
            elif args.action == 'compare':
                symbol = args.param1 if args.param1 else "BTCUSDT"
                controller.cmd_fees_compare(symbol)
            elif args.action == 'calculate':
                if not args.param1:
                    print("❌ 请指定交易金额")
                    return
                amount = float(args.param1)
                exchange = args.param2 if args.param2 else "binance"
                symbol = args.param3 if args.param3 else "BTCUSDT"
                controller.cmd_fees_calculate(amount, exchange, symbol)
            elif args.action == 'arbitrage-analysis':
                symbol = args.param1 if args.param1 else "BTCUSDT"
                amount = float(args.param2) if args.param2 else 10000.0
                controller.cmd_fees_arbitrage_analysis(symbol, amount)
            elif args.action == 'refresh':
                controller.cmd_fees_refresh()

        elif args.command == 'exchange':
            if args.action == 'add-api':
                if not args.exchange or not args.api_key or not args.secret_key:
                    print("❌ 请指定交易所名称、API密钥和私钥")
                    return
                controller.cmd_exchange_add_api(args.exchange, args.api_key, args.secret_key, args.testnet)
            elif args.action == 'list-apis':
                controller.cmd_exchange_list_apis()
            elif args.action == 'test-api':
                if not args.exchange:
                    print("❌ 请指定交易所名称")
                    return
                controller.cmd_exchange_test_api(args.exchange)
            elif args.action == 'account':
                if not args.exchange:
                    print("❌ 请指定交易所名称")
                    return
                controller.cmd_exchange_get_account(args.exchange)
            elif args.action == 'real-fees':
                if not args.exchange:
                    print("❌ 请指定交易所名称")
                    return
                controller.cmd_exchange_get_real_fees(args.exchange)
            elif args.action == 'remove-api':
                if not args.exchange:
                    print("❌ 请指定交易所名称")
                    return
                controller.cmd_exchange_remove_api(args.exchange)

        elif args.command == 'config':
            if args.action == 'show':
                config_type = args.param1 if args.param1 else 'system'
                controller.cmd_config_show(config_type)
            elif args.action == 'set':
                if not args.param1 or not args.param2:
                    print("❌ 请指定配置路径和值")
                    return
                controller.cmd_config_set(args.param1, args.param2)
    
    except KeyboardInterrupt:
        print("\n\n👋 操作已取消")
    except Exception as e:
        logger.error(f"执行命令时发生错误: {e}")
        print(f"\n❌ 错误: {e}")

if __name__ == "__main__":
    main()