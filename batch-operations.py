#!/usr/bin/env python3
"""
套利系统5.1批量操作工具
=======================

这个工具提供批量操作功能，可以同时执行多个操作，适用于：
- 批量配置管理
- 批量策略操作
- 批量数据处理
- 自动化测试场景

使用方法:
    python batch-operations.py [操作文件.yaml]
    python batch-operations.py --interactive
"""

import sys
import yaml
import json
import time
import asyncio
import logging
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any
import subprocess

# 导入主控制器
from pathlib import Path
sys.path.append(str(Path(__file__).parent))

# 配置日志
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class BatchOperationManager:
    """批量操作管理器"""
    
    def __init__(self):
        self.controller_path = Path(__file__).parent / "arbitrage-cli-controller.py"
        self.results = []
        
    def load_batch_config(self, config_file: str) -> Dict:
        """加载批量操作配置"""
        try:
            with open(config_file, 'r', encoding='utf-8') as f:
                if config_file.endswith('.yaml') or config_file.endswith('.yml'):
                    return yaml.safe_load(f)
                else:
                    return json.load(f)
        except Exception as e:
            logger.error(f"加载配置文件失败: {e}")
            return {}
    
    def execute_command(self, command: List[str]) -> Dict:
        """执行单个命令"""
        cmd = ["python3", str(self.controller_path)] + command
        
        try:
            start_time = time.time()
            result = subprocess.run(
                cmd, 
                capture_output=True, 
                text=True, 
                timeout=300
            )
            
            execution_time = time.time() - start_time
            
            return {
                "command": " ".join(command),
                "success": result.returncode == 0,
                "returncode": result.returncode,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "execution_time": execution_time,
                "timestamp": datetime.now().isoformat()
            }
        except subprocess.TimeoutExpired:
            return {
                "command": " ".join(command),
                "success": False,
                "error": "命令执行超时",
                "execution_time": 300,
                "timestamp": datetime.now().isoformat()
            }
        except Exception as e:
            return {
                "command": " ".join(command),
                "success": False,
                "error": str(e),
                "execution_time": 0,
                "timestamp": datetime.now().isoformat()
            }
    
    def execute_batch(self, config: Dict) -> List[Dict]:
        """执行批量操作"""
        operations = config.get("operations", [])
        results = []
        
        print(f"🚀 开始执行批量操作，共 {len(operations)} 个操作")
        print("=" * 80)
        
        for i, operation in enumerate(operations, 1):
            name = operation.get("name", f"操作 {i}")
            command = operation.get("command", [])
            delay = operation.get("delay", 0)
            retry = operation.get("retry", 1)
            ignore_error = operation.get("ignore_error", False)
            
            print(f"\n[{i}/{len(operations)}] {name}")
            print(f"命令: {' '.join(command)}")
            
            # 重试逻辑
            success = False
            for attempt in range(retry):
                if retry > 1:
                    print(f"  尝试 {attempt + 1}/{retry}")
                
                result = self.execute_command(command)
                result["operation_name"] = name
                result["attempt"] = attempt + 1
                
                if result["success"]:
                    print(f"  ✅ 成功 (耗时: {result['execution_time']:.1f}s)")
                    success = True
                    results.append(result)
                    break
                else:
                    error_msg = result.get("stderr") or result.get("error", "Unknown error")
                    print(f"  ❌ 失败: {error_msg}")
                    if attempt < retry - 1:
                        print(f"  ⏳ 等待重试...")
                        time.sleep(2)
                    else:
                        results.append(result)
            
            if not success and not ignore_error:
                print(f"\n🛑 操作失败，停止批量执行")
                break
            
            # 延迟
            if delay > 0 and i < len(operations):
                print(f"  ⏳ 等待 {delay} 秒...")
                time.sleep(delay)
        
        return results
    
    def generate_report(self, results: List[Dict], output_file: str = None):
        """生成执行报告"""
        if not output_file:
            output_file = f"batch-report-{datetime.now().strftime('%Y%m%d-%H%M%S')}.json"
        
        # 统计信息
        total_operations = len(results)
        successful_operations = sum(1 for r in results if r["success"])
        total_time = sum(r["execution_time"] for r in results)
        
        report = {
            "summary": {
                "total_operations": total_operations,
                "successful_operations": successful_operations,
                "failed_operations": total_operations - successful_operations,
                "success_rate": (successful_operations / total_operations * 100) if total_operations > 0 else 0,
                "total_execution_time": total_time,
                "execution_date": datetime.now().isoformat()
            },
            "operations": results
        }
        
        # 保存报告
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        
        # 打印摘要
        print(f"\n📊 执行摘要:")
        print("=" * 60)
        print(f"总操作数: {total_operations}")
        print(f"成功操作: {successful_operations}")
        print(f"失败操作: {total_operations - successful_operations}")
        print(f"成功率: {successful_operations / total_operations * 100:.1f}%")
        print(f"总耗时: {total_time:.1f} 秒")
        print(f"报告已保存: {output_file}")
        
        return report
    
    def interactive_builder(self):
        """交互式批量操作构建器"""
        print("🛠️ 批量操作构建器")
        print("=" * 60)
        
        operations = []
        
        while True:
            print(f"\n当前操作数: {len(operations)}")
            print("1) 添加系统控制操作")
            print("2) 添加数据处理操作")
            print("3) 添加策略管理操作")
            print("4) 添加AI操作")
            print("5) 添加自定义操作")
            print("6) 查看当前操作")
            print("7) 执行批量操作")
            print("8) 保存操作配置")
            print("0) 退出")
            
            choice = input("\n请选择 [0-8]: ").strip()
            
            if choice == "0":
                break
            elif choice == "1":
                operations.append(self._build_system_operation())
            elif choice == "2":
                operations.append(self._build_data_operation())
            elif choice == "3":
                operations.append(self._build_strategy_operation())
            elif choice == "4":
                operations.append(self._build_ai_operation())
            elif choice == "5":
                operations.append(self._build_custom_operation())
            elif choice == "6":
                self._display_operations(operations)
            elif choice == "7":
                if operations:
                    config = {"operations": operations}
                    results = self.execute_batch(config)
                    self.generate_report(results)
                else:
                    print("❌ 没有操作需要执行")
            elif choice == "8":
                if operations:
                    self._save_operations(operations)
                else:
                    print("❌ 没有操作需要保存")
    
    def _build_system_operation(self) -> Dict:
        """构建系统控制操作"""
        print("\n系统控制操作:")
        print("1) 查看状态  2) 启动系统  3) 停止系统  4) 重启系统")
        choice = input("选择 [1-4]: ").strip()
        
        operations = {
            "1": ("系统状态检查", ["system", "status"]),
            "2": ("启动系统", ["system", "start"]),
            "3": ("停止系统", ["system", "stop"]),
            "4": ("重启系统", ["system", "restart"])
        }
        
        if choice in operations:
            name, command = operations[choice]
            return {
                "name": name,
                "command": command,
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": int(input("重试次数 (默认1): ") or "1"),
                "ignore_error": input("忽略错误? (y/N): ").lower().startswith('y')
            }
        else:
            return self._build_system_operation()
    
    def _build_data_operation(self) -> Dict:
        """构建数据处理操作"""
        print("\n数据处理操作:")
        print("1) 启动数据采集  2) 停止数据采集  3) 查看状态  4) 数据清洗")
        choice = input("选择 [1-4]: ").strip()
        
        operations = {
            "1": ("启动数据采集", ["data", "start-all"]),
            "2": ("停止数据采集", ["data", "stop-all"]),
            "3": ("数据状态检查", ["data", "status"]),
            "4": ("数据清洗", ["data", "clean"])
        }
        
        if choice in operations:
            name, command = operations[choice]
            
            # 数据清洗可以指定交易所
            if choice == "4":
                exchange = input("指定交易所 (留空为全部): ").strip()
                if exchange:
                    command.append(exchange)
                    name = f"数据清洗 - {exchange}"
            
            return {
                "name": name,
                "command": command,
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": int(input("重试次数 (默认1): ") or "1"),
                "ignore_error": input("忽略错误? (y/N): ").lower().startswith('y')
            }
        else:
            return self._build_data_operation()
    
    def _build_strategy_operation(self) -> Dict:
        """构建策略管理操作"""
        print("\n策略管理操作:")
        print("1) 列出策略  2) 启动策略  3) 停止策略  4) 查看状态")
        choice = input("选择 [1-4]: ").strip()
        
        if choice == "1":
            return {
                "name": "列出所有策略",
                "command": ["strategy", "list"],
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": 1,
                "ignore_error": False
            }
        elif choice in ["2", "3"]:
            action = "start" if choice == "2" else "stop"
            strategy_name = input("策略名称: ").strip()
            
            return {
                "name": f"{'启动' if choice == '2' else '停止'}策略 - {strategy_name}",
                "command": ["strategy", action, strategy_name],
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": int(input("重试次数 (默认1): ") or "1"),
                "ignore_error": input("忽略错误? (y/N): ").lower().startswith('y')
            }
        elif choice == "4":
            strategy_name = input("策略名称 (留空为全部): ").strip()
            command = ["strategy", "status"]
            if strategy_name:
                command.append(strategy_name)
            
            return {
                "name": f"策略状态 - {strategy_name or '全部'}",
                "command": command,
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": 1,
                "ignore_error": False
            }
        else:
            return self._build_strategy_operation()
    
    def _build_ai_operation(self) -> Dict:
        """构建AI操作"""
        print("\nAI操作:")
        print("1) 列出AI模型  2) 训练模型  3) 部署模型  4) 风控状态")
        choice = input("选择 [1-4]: ").strip()
        
        if choice == "1":
            return {
                "name": "列出AI模型",
                "command": ["ai", "models"],
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": 1,
                "ignore_error": False
            }
        elif choice == "2":
            model_name = input("模型名称: ").strip()
            days = input("训练天数 (默认30): ").strip() or "30"
            
            return {
                "name": f"训练模型 - {model_name}",
                "command": ["ai", "train", model_name, days],
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": 1,
                "ignore_error": input("忽略错误? (y/N): ").lower().startswith('y')
            }
        elif choice == "3":
            model_name = input("模型名称: ").strip()
            version = input("版本 (默认latest): ").strip() or "latest"
            
            return {
                "name": f"部署模型 - {model_name}",
                "command": ["ai", "deploy", model_name, version],
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": 1,
                "ignore_error": input("忽略错误? (y/N): ").lower().startswith('y')
            }
        elif choice == "4":
            return {
                "name": "风控状态检查",
                "command": ["risk", "status"],
                "delay": int(input("延迟秒数 (默认0): ") or "0"),
                "retry": 1,
                "ignore_error": False
            }
        else:
            return self._build_ai_operation()
    
    def _build_custom_operation(self) -> Dict:
        """构建自定义操作"""
        print("\n自定义操作:")
        name = input("操作名称: ").strip()
        command_str = input("命令 (空格分隔): ").strip()
        command = command_str.split()
        
        return {
            "name": name,
            "command": command,
            "delay": int(input("延迟秒数 (默认0): ") or "0"),
            "retry": int(input("重试次数 (默认1): ") or "1"),
            "ignore_error": input("忽略错误? (y/N): ").lower().startswith('y')
        }
    
    def _display_operations(self, operations: List[Dict]):
        """显示当前操作"""
        print(f"\n📋 当前操作列表 ({len(operations)} 个):")
        print("=" * 60)
        
        for i, op in enumerate(operations, 1):
            print(f"{i:2}. {op['name']}")
            print(f"    命令: {' '.join(op['command'])}")
            print(f"    延迟: {op.get('delay', 0)}s | 重试: {op.get('retry', 1)} | 忽略错误: {op.get('ignore_error', False)}")
    
    def _save_operations(self, operations: List[Dict]):
        """保存操作配置"""
        filename = input("配置文件名 (默认batch-config.yaml): ").strip() or "batch-config.yaml"
        
        config = {
            "description": "批量操作配置",
            "created": datetime.now().isoformat(),
            "operations": operations
        }
        
        if filename.endswith('.yaml') or filename.endswith('.yml'):
            with open(filename, 'w', encoding='utf-8') as f:
                yaml.dump(config, f, default_flow_style=False, allow_unicode=True)
        else:
            with open(filename, 'w', encoding='utf-8') as f:
                json.dump(config, f, indent=2, ensure_ascii=False)
        
        print(f"✅ 配置已保存: {filename}")

def create_sample_configs():
    """创建示例配置文件"""
    
    # 完整系统启动配置
    startup_config = {
        "description": "完整系统启动流程",
        "operations": [
            {
                "name": "系统状态检查",
                "command": ["system", "status"],
                "delay": 0,
                "retry": 1,
                "ignore_error": True
            },
            {
                "name": "启动核心系统",
                "command": ["system", "start"],
                "delay": 10,
                "retry": 2,
                "ignore_error": False
            },
            {
                "name": "启动数据采集",
                "command": ["data", "start-all"],
                "delay": 5,
                "retry": 2,
                "ignore_error": False
            },
            {
                "name": "启动跨交易所策略",
                "command": ["strategy", "start", "inter_exchange_production"],
                "delay": 3,
                "retry": 1,
                "ignore_error": True
            },
            {
                "name": "启动三角套利策略",
                "command": ["strategy", "start", "triangular_production"],
                "delay": 3,
                "retry": 1,
                "ignore_error": True
            },
            {
                "name": "最终状态检查",
                "command": ["system", "status"],
                "delay": 0,
                "retry": 1,
                "ignore_error": False
            }
        ]
    }
    
    # 每日维护配置
    maintenance_config = {
        "description": "每日系统维护",
        "operations": [
            {
                "name": "数据清洗 - 全部",
                "command": ["data", "clean"],
                "delay": 2,
                "retry": 1,
                "ignore_error": True
            },
            {
                "name": "AI模型状态检查",
                "command": ["ai", "models"],
                "delay": 1,
                "retry": 1,
                "ignore_error": True
            },
            {
                "name": "风控状态检查",
                "command": ["risk", "status"],
                "delay": 1,
                "retry": 1,
                "ignore_error": True
            },
            {
                "name": "策略状态检查",
                "command": ["strategy", "status"],
                "delay": 1,
                "retry": 1,
                "ignore_error": True
            }
        ]
    }
    
    # 保存配置文件
    with open("batch-startup.yaml", 'w', encoding='utf-8') as f:
        yaml.dump(startup_config, f, default_flow_style=False, allow_unicode=True)
    
    with open("batch-maintenance.yaml", 'w', encoding='utf-8') as f:
        yaml.dump(maintenance_config, f, default_flow_style=False, allow_unicode=True)
    
    print("✅ 示例配置文件已创建:")
    print("  - batch-startup.yaml: 完整系统启动流程")
    print("  - batch-maintenance.yaml: 每日系统维护")

def main():
    """主函数"""
    manager = BatchOperationManager()
    
    if len(sys.argv) == 1:
        print("🛠️ 套利系统5.1批量操作工具")
        print("=" * 60)
        print("使用方法:")
        print("  python batch-operations.py [配置文件]")
        print("  python batch-operations.py --interactive")
        print("  python batch-operations.py --create-samples")
        print("")
        choice = input("选择操作 (1:交互式构建 2:创建示例 0:退出): ").strip()
        
        if choice == "1":
            manager.interactive_builder()
        elif choice == "2":
            create_sample_configs()
        else:
            return
    
    elif sys.argv[1] == "--interactive":
        manager.interactive_builder()
    
    elif sys.argv[1] == "--create-samples":
        create_sample_configs()
    
    else:
        config_file = sys.argv[1]
        if not Path(config_file).exists():
            print(f"❌ 配置文件不存在: {config_file}")
            return
        
        config = manager.load_batch_config(config_file)
        if not config:
            print("❌ 加载配置文件失败")
            return
        
        results = manager.execute_batch(config)
        manager.generate_report(results)

if __name__ == "__main__":
    main()