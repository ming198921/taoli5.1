#!/usr/bin/env python3
"""
配置管理脚本
用于管理 arbitrage_monitor 的所有配置文件
"""

import os
import sys
import json
import toml
import argparse
import shutil
from pathlib import Path
from typing import Dict, Any, List
from datetime import datetime

class ConfigManager:
    """配置文件管理器"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.config_dir = self.project_root / "config"
        self.backup_dir = self.project_root / "config_backups"
        
        # 配置文件映射
        self.config_files = {
            "main": "arbitrage_monitor_config.toml",
            "runtime": "runtime_config.toml", 
            "performance": "performance_config.toml",
            "test": "test_config.toml",
            "deployment": "deployment_config.toml"
        }
        
    def validate_config(self, config_type: str = "all") -> bool:
        """验证配置文件的有效性"""
        print(f"🔍 验证配置文件: {config_type}")
        
        configs_to_validate = []
        if config_type == "all":
            configs_to_validate = list(self.config_files.keys())
        else:
            configs_to_validate = [config_type]
            
        validation_results = {}
        
        for config_name in configs_to_validate:
            config_file = self.config_dir / self.config_files[config_name]
            try:
                with open(config_file, 'r', encoding='utf-8') as f:
                    config_data = toml.load(f)
                    
                # 基本验证
                validation_results[config_name] = self._validate_config_content(
                    config_name, config_data
                )
                
                if validation_results[config_name]:
                    print(f"  ✅ {config_name}: 验证通过")
                else:
                    print(f"  ❌ {config_name}: 验证失败")
                    
            except Exception as e:
                print(f"  ❌ {config_name}: 读取失败 - {e}")
                validation_results[config_name] = False
                
        return all(validation_results.values())
    
    def _validate_config_content(self, config_name: str, config_data: Dict[str, Any]) -> bool:
        """验证配置内容的有效性"""
        
        if config_name == "main":
            return self._validate_main_config(config_data)
        elif config_name == "performance":
            return self._validate_performance_config(config_data)
        elif config_name == "test":
            return self._validate_test_config(config_data)
        elif config_name == "runtime":
            return self._validate_runtime_config(config_data)
        elif config_name == "deployment":
            return self._validate_deployment_config(config_data)
        
        return True
    
    def _validate_main_config(self, config: Dict[str, Any]) -> bool:
        """验证主配置文件"""
        required_sections = [
            "system", "performance", "networking", "monitoring", 
            "arbitrage", "risk_management", "exchanges"
        ]
        
        for section in required_sections:
            if section not in config:
                print(f"    ❌ 缺少必需的配置段: {section}")
                return False
                
        # 验证关键参数
        if config.get("performance", {}).get("optimal_batch_size", 0) < 512:
            print(f"    ❌ optimal_batch_size 过小")
            return False
            
        if not config.get("performance", {}).get("enable_avx512"):
            print(f"    ⚠️  AVX-512 未启用")
            
        return True
    
    def _validate_performance_config(self, config: Dict[str, Any]) -> bool:
        """验证性能配置文件"""
        if not config.get("simd", {}).get("force_avx512"):
            print(f"    ❌ 未强制启用 AVX-512")
            return False
            
        target_latency = config.get("optimization_targets", {}).get("latency_target_microseconds", 0)
        if target_latency > 1000:
            print(f"    ⚠️  延迟目标可能过高: {target_latency}μs")
            
        return True
    
    def _validate_test_config(self, config: Dict[str, Any]) -> bool:
        """验证测试配置文件"""
        if not config.get("simd_testing", {}).get("avx512_mandatory"):
            print(f"    ❌ 测试配置中 AVX-512 非强制性")
            return False
            
        return True
    
    def _validate_runtime_config(self, config: Dict[str, Any]) -> bool:
        """验证运行时配置文件"""
        if not config.get("features", {}).get("avx512_enabled"):
            print(f"    ❌ 运行时 AVX-512 未启用")
            return False
            
        return True
    
    def _validate_deployment_config(self, config: Dict[str, Any]) -> bool:
        """验证部署配置文件"""
        required_features = ["avx512f", "avx512dq", "avx512bw"]
        actual_features = config.get("system_requirements", {}).get("required_cpu_features", [])
        
        for feature in required_features:
            if feature not in actual_features:
                print(f"    ❌ 缺少必需的CPU特性: {feature}")
                return False
                
        return True
    
    def backup_configs(self) -> str:
        """备份所有配置文件"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_path = self.backup_dir / f"backup_{timestamp}"
        backup_path.mkdir(parents=True, exist_ok=True)
        
        print(f"📁 备份配置文件到: {backup_path}")
        
        for config_name, config_file in self.config_files.items():
            source = self.config_dir / config_file
            if source.exists():
                destination = backup_path / config_file
                shutil.copy2(source, destination)
                print(f"  ✅ 已备份: {config_file}")
            else:
                print(f"  ⚠️  文件不存在: {config_file}")
                
        return str(backup_path)
    
    def restore_configs(self, backup_path: str) -> bool:
        """从备份恢复配置文件"""
        backup_dir = Path(backup_path)
        if not backup_dir.exists():
            print(f"❌ 备份目录不存在: {backup_path}")
            return False
            
        print(f"🔄 从备份恢复配置文件: {backup_path}")
        
        for config_name, config_file in self.config_files.items():
            source = backup_dir / config_file
            if source.exists():
                destination = self.config_dir / config_file
                shutil.copy2(source, destination)
                print(f"  ✅ 已恢复: {config_file}")
            else:
                print(f"  ⚠️  备份中无此文件: {config_file}")
                
        return True
    
    def list_backups(self) -> List[str]:
        """列出所有备份"""
        if not self.backup_dir.exists():
            return []
            
        backups = []
        for item in self.backup_dir.iterdir():
            if item.is_dir() and item.name.startswith("backup_"):
                backups.append(item.name)
                
        return sorted(backups, reverse=True)
    
    def optimize_for_environment(self, environment: str) -> bool:
        """针对特定环境优化配置"""
        print(f"⚙️  针对 {environment} 环境优化配置")
        
        if environment == "production":
            return self._optimize_for_production()
        elif environment == "test":
            return self._optimize_for_test()
        elif environment == "development":
            return self._optimize_for_development()
        else:
            print(f"❌ 未知环境: {environment}")
            return False
    
    def _optimize_for_production(self) -> bool:
        """生产环境优化"""
        # 读取主配置
        main_config_path = self.config_dir / self.config_files["main"]
        with open(main_config_path, 'r', encoding='utf-8') as f:
            config = toml.load(f)
        
        # 生产环境优化
        config["system"]["environment"] = "production"
        config["system"]["log_level"] = "info"
        config["performance"]["optimal_batch_size"] = 2048
        config["performance"]["enable_avx512"] = True
        config["risk_management"]["enable_dynamic_risk"] = True
        
        # 写回配置
        with open(main_config_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
            
        print("  ✅ 生产环境优化完成")
        return True
    
    def _optimize_for_test(self) -> bool:
        """测试环境优化"""
        # 读取测试配置
        test_config_path = self.config_dir / self.config_files["test"]
        with open(test_config_path, 'r', encoding='utf-8') as f:
            config = toml.load(f)
        
        # 测试环境优化
        config["test_environment"]["fast_execution"] = True
        config["test_environment"]["detailed_logging"] = True
        config["simd_testing"]["avx512_mandatory"] = True
        config["performance_targets"]["max_latency_microseconds"] = 100
        
        # 写回配置
        with open(test_config_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
            
        print("  ✅ 测试环境优化完成")
        return True
    
    def _optimize_for_development(self) -> bool:
        """开发环境优化"""
        # 读取主配置
        main_config_path = self.config_dir / self.config_files["main"]
        with open(main_config_path, 'r', encoding='utf-8') as f:
            config = toml.load(f)
        
        # 开发环境优化
        config["system"]["environment"] = "development"
        config["system"]["log_level"] = "debug"
        config["performance"]["optimal_batch_size"] = 1024  # 较小批次便于调试
        config["debugging"]["enable_debug_mode"] = True
        config["debugging"]["verbose_simd_operations"] = True
        
        # 写回配置
        with open(main_config_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
            
        print("  ✅ 开发环境优化完成")
        return True
    
    def generate_summary(self) -> Dict[str, Any]:
        """生成配置摘要"""
        summary = {
            "timestamp": datetime.now().isoformat(),
            "configs": {},
            "validation_status": "unknown",
            "key_settings": {}
        }
        
        # 收集关键设置
        try:
            main_config_path = self.config_dir / self.config_files["main"]
            with open(main_config_path, 'r', encoding='utf-8') as f:
                main_config = toml.load(f)
                
            summary["key_settings"] = {
                "environment": main_config.get("system", {}).get("environment"),
                "batch_size": main_config.get("performance", {}).get("optimal_batch_size"),
                "avx512_enabled": main_config.get("performance", {}).get("enable_avx512"),
                "target_throughput": main_config.get("performance", {}).get("target_throughput_msg_per_sec"),
                "log_level": main_config.get("system", {}).get("log_level")
            }
        except Exception as e:
            summary["error"] = str(e)
        
        # 验证状态
        summary["validation_status"] = "valid" if self.validate_config("all") else "invalid"
        
        return summary

def main():
    parser = argparse.ArgumentParser(description="配置文件管理工具")
    parser.add_argument("--project-root", default=".", help="项目根目录")
    
    subparsers = parser.add_subparsers(dest="command", help="可用命令")
    
    # 验证命令
    validate_parser = subparsers.add_parser("validate", help="验证配置文件")
    validate_parser.add_argument("--type", default="all", 
                                choices=["all", "main", "runtime", "performance", "test", "deployment"],
                                help="要验证的配置类型")
    
    # 备份命令
    backup_parser = subparsers.add_parser("backup", help="备份配置文件")
    
    # 恢复命令  
    restore_parser = subparsers.add_parser("restore", help="恢复配置文件")
    restore_parser.add_argument("backup_path", help="备份路径")
    
    # 列出备份命令
    list_parser = subparsers.add_parser("list-backups", help="列出所有备份")
    
    # 优化命令
    optimize_parser = subparsers.add_parser("optimize", help="针对环境优化配置")
    optimize_parser.add_argument("environment", choices=["production", "test", "development"],
                                help="目标环境")
    
    # 摘要命令
    summary_parser = subparsers.add_parser("summary", help="生成配置摘要")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    config_manager = ConfigManager(args.project_root)
    
    if args.command == "validate":
        success = config_manager.validate_config(args.type)
        sys.exit(0 if success else 1)
        
    elif args.command == "backup":
        backup_path = config_manager.backup_configs()
        print(f"✅ 备份完成: {backup_path}")
        
    elif args.command == "restore":
        success = config_manager.restore_configs(args.backup_path)
        sys.exit(0 if success else 1)
        
    elif args.command == "list-backups":
        backups = config_manager.list_backups()
        if backups:
            print("📁 可用备份:")
            for backup in backups:
                print(f"  - {backup}")
        else:
            print("📁 无可用备份")
            
    elif args.command == "optimize":
        success = config_manager.optimize_for_environment(args.environment)
        sys.exit(0 if success else 1)
        
    elif args.command == "summary":
        summary = config_manager.generate_summary()
        print("📊 配置摘要:")
        print(json.dumps(summary, indent=2, ensure_ascii=False))

if __name__ == "__main__":
    main() 
"""
配置管理脚本
用于管理 arbitrage_monitor 的所有配置文件
"""

import os
import sys
import json
import toml
import argparse
import shutil
from pathlib import Path
from typing import Dict, Any, List
from datetime import datetime

class ConfigManager:
    """配置文件管理器"""
    
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.config_dir = self.project_root / "config"
        self.backup_dir = self.project_root / "config_backups"
        
        # 配置文件映射
        self.config_files = {
            "main": "arbitrage_monitor_config.toml",
            "runtime": "runtime_config.toml", 
            "performance": "performance_config.toml",
            "test": "test_config.toml",
            "deployment": "deployment_config.toml"
        }
        
    def validate_config(self, config_type: str = "all") -> bool:
        """验证配置文件的有效性"""
        print(f"🔍 验证配置文件: {config_type}")
        
        configs_to_validate = []
        if config_type == "all":
            configs_to_validate = list(self.config_files.keys())
        else:
            configs_to_validate = [config_type]
            
        validation_results = {}
        
        for config_name in configs_to_validate:
            config_file = self.config_dir / self.config_files[config_name]
            try:
                with open(config_file, 'r', encoding='utf-8') as f:
                    config_data = toml.load(f)
                    
                # 基本验证
                validation_results[config_name] = self._validate_config_content(
                    config_name, config_data
                )
                
                if validation_results[config_name]:
                    print(f"  ✅ {config_name}: 验证通过")
                else:
                    print(f"  ❌ {config_name}: 验证失败")
                    
            except Exception as e:
                print(f"  ❌ {config_name}: 读取失败 - {e}")
                validation_results[config_name] = False
                
        return all(validation_results.values())
    
    def _validate_config_content(self, config_name: str, config_data: Dict[str, Any]) -> bool:
        """验证配置内容的有效性"""
        
        if config_name == "main":
            return self._validate_main_config(config_data)
        elif config_name == "performance":
            return self._validate_performance_config(config_data)
        elif config_name == "test":
            return self._validate_test_config(config_data)
        elif config_name == "runtime":
            return self._validate_runtime_config(config_data)
        elif config_name == "deployment":
            return self._validate_deployment_config(config_data)
        
        return True
    
    def _validate_main_config(self, config: Dict[str, Any]) -> bool:
        """验证主配置文件"""
        required_sections = [
            "system", "performance", "networking", "monitoring", 
            "arbitrage", "risk_management", "exchanges"
        ]
        
        for section in required_sections:
            if section not in config:
                print(f"    ❌ 缺少必需的配置段: {section}")
                return False
                
        # 验证关键参数
        if config.get("performance", {}).get("optimal_batch_size", 0) < 512:
            print(f"    ❌ optimal_batch_size 过小")
            return False
            
        if not config.get("performance", {}).get("enable_avx512"):
            print(f"    ⚠️  AVX-512 未启用")
            
        return True
    
    def _validate_performance_config(self, config: Dict[str, Any]) -> bool:
        """验证性能配置文件"""
        if not config.get("simd", {}).get("force_avx512"):
            print(f"    ❌ 未强制启用 AVX-512")
            return False
            
        target_latency = config.get("optimization_targets", {}).get("latency_target_microseconds", 0)
        if target_latency > 1000:
            print(f"    ⚠️  延迟目标可能过高: {target_latency}μs")
            
        return True
    
    def _validate_test_config(self, config: Dict[str, Any]) -> bool:
        """验证测试配置文件"""
        if not config.get("simd_testing", {}).get("avx512_mandatory"):
            print(f"    ❌ 测试配置中 AVX-512 非强制性")
            return False
            
        return True
    
    def _validate_runtime_config(self, config: Dict[str, Any]) -> bool:
        """验证运行时配置文件"""
        if not config.get("features", {}).get("avx512_enabled"):
            print(f"    ❌ 运行时 AVX-512 未启用")
            return False
            
        return True
    
    def _validate_deployment_config(self, config: Dict[str, Any]) -> bool:
        """验证部署配置文件"""
        required_features = ["avx512f", "avx512dq", "avx512bw"]
        actual_features = config.get("system_requirements", {}).get("required_cpu_features", [])
        
        for feature in required_features:
            if feature not in actual_features:
                print(f"    ❌ 缺少必需的CPU特性: {feature}")
                return False
                
        return True
    
    def backup_configs(self) -> str:
        """备份所有配置文件"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        backup_path = self.backup_dir / f"backup_{timestamp}"
        backup_path.mkdir(parents=True, exist_ok=True)
        
        print(f"📁 备份配置文件到: {backup_path}")
        
        for config_name, config_file in self.config_files.items():
            source = self.config_dir / config_file
            if source.exists():
                destination = backup_path / config_file
                shutil.copy2(source, destination)
                print(f"  ✅ 已备份: {config_file}")
            else:
                print(f"  ⚠️  文件不存在: {config_file}")
                
        return str(backup_path)
    
    def restore_configs(self, backup_path: str) -> bool:
        """从备份恢复配置文件"""
        backup_dir = Path(backup_path)
        if not backup_dir.exists():
            print(f"❌ 备份目录不存在: {backup_path}")
            return False
            
        print(f"🔄 从备份恢复配置文件: {backup_path}")
        
        for config_name, config_file in self.config_files.items():
            source = backup_dir / config_file
            if source.exists():
                destination = self.config_dir / config_file
                shutil.copy2(source, destination)
                print(f"  ✅ 已恢复: {config_file}")
            else:
                print(f"  ⚠️  备份中无此文件: {config_file}")
                
        return True
    
    def list_backups(self) -> List[str]:
        """列出所有备份"""
        if not self.backup_dir.exists():
            return []
            
        backups = []
        for item in self.backup_dir.iterdir():
            if item.is_dir() and item.name.startswith("backup_"):
                backups.append(item.name)
                
        return sorted(backups, reverse=True)
    
    def optimize_for_environment(self, environment: str) -> bool:
        """针对特定环境优化配置"""
        print(f"⚙️  针对 {environment} 环境优化配置")
        
        if environment == "production":
            return self._optimize_for_production()
        elif environment == "test":
            return self._optimize_for_test()
        elif environment == "development":
            return self._optimize_for_development()
        else:
            print(f"❌ 未知环境: {environment}")
            return False
    
    def _optimize_for_production(self) -> bool:
        """生产环境优化"""
        # 读取主配置
        main_config_path = self.config_dir / self.config_files["main"]
        with open(main_config_path, 'r', encoding='utf-8') as f:
            config = toml.load(f)
        
        # 生产环境优化
        config["system"]["environment"] = "production"
        config["system"]["log_level"] = "info"
        config["performance"]["optimal_batch_size"] = 2048
        config["performance"]["enable_avx512"] = True
        config["risk_management"]["enable_dynamic_risk"] = True
        
        # 写回配置
        with open(main_config_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
            
        print("  ✅ 生产环境优化完成")
        return True
    
    def _optimize_for_test(self) -> bool:
        """测试环境优化"""
        # 读取测试配置
        test_config_path = self.config_dir / self.config_files["test"]
        with open(test_config_path, 'r', encoding='utf-8') as f:
            config = toml.load(f)
        
        # 测试环境优化
        config["test_environment"]["fast_execution"] = True
        config["test_environment"]["detailed_logging"] = True
        config["simd_testing"]["avx512_mandatory"] = True
        config["performance_targets"]["max_latency_microseconds"] = 100
        
        # 写回配置
        with open(test_config_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
            
        print("  ✅ 测试环境优化完成")
        return True
    
    def _optimize_for_development(self) -> bool:
        """开发环境优化"""
        # 读取主配置
        main_config_path = self.config_dir / self.config_files["main"]
        with open(main_config_path, 'r', encoding='utf-8') as f:
            config = toml.load(f)
        
        # 开发环境优化
        config["system"]["environment"] = "development"
        config["system"]["log_level"] = "debug"
        config["performance"]["optimal_batch_size"] = 1024  # 较小批次便于调试
        config["debugging"]["enable_debug_mode"] = True
        config["debugging"]["verbose_simd_operations"] = True
        
        # 写回配置
        with open(main_config_path, 'w', encoding='utf-8') as f:
            toml.dump(config, f)
            
        print("  ✅ 开发环境优化完成")
        return True
    
    def generate_summary(self) -> Dict[str, Any]:
        """生成配置摘要"""
        summary = {
            "timestamp": datetime.now().isoformat(),
            "configs": {},
            "validation_status": "unknown",
            "key_settings": {}
        }
        
        # 收集关键设置
        try:
            main_config_path = self.config_dir / self.config_files["main"]
            with open(main_config_path, 'r', encoding='utf-8') as f:
                main_config = toml.load(f)
                
            summary["key_settings"] = {
                "environment": main_config.get("system", {}).get("environment"),
                "batch_size": main_config.get("performance", {}).get("optimal_batch_size"),
                "avx512_enabled": main_config.get("performance", {}).get("enable_avx512"),
                "target_throughput": main_config.get("performance", {}).get("target_throughput_msg_per_sec"),
                "log_level": main_config.get("system", {}).get("log_level")
            }
        except Exception as e:
            summary["error"] = str(e)
        
        # 验证状态
        summary["validation_status"] = "valid" if self.validate_config("all") else "invalid"
        
        return summary

def main():
    parser = argparse.ArgumentParser(description="配置文件管理工具")
    parser.add_argument("--project-root", default=".", help="项目根目录")
    
    subparsers = parser.add_subparsers(dest="command", help="可用命令")
    
    # 验证命令
    validate_parser = subparsers.add_parser("validate", help="验证配置文件")
    validate_parser.add_argument("--type", default="all", 
                                choices=["all", "main", "runtime", "performance", "test", "deployment"],
                                help="要验证的配置类型")
    
    # 备份命令
    backup_parser = subparsers.add_parser("backup", help="备份配置文件")
    
    # 恢复命令  
    restore_parser = subparsers.add_parser("restore", help="恢复配置文件")
    restore_parser.add_argument("backup_path", help="备份路径")
    
    # 列出备份命令
    list_parser = subparsers.add_parser("list-backups", help="列出所有备份")
    
    # 优化命令
    optimize_parser = subparsers.add_parser("optimize", help="针对环境优化配置")
    optimize_parser.add_argument("environment", choices=["production", "test", "development"],
                                help="目标环境")
    
    # 摘要命令
    summary_parser = subparsers.add_parser("summary", help="生成配置摘要")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    config_manager = ConfigManager(args.project_root)
    
    if args.command == "validate":
        success = config_manager.validate_config(args.type)
        sys.exit(0 if success else 1)
        
    elif args.command == "backup":
        backup_path = config_manager.backup_configs()
        print(f"✅ 备份完成: {backup_path}")
        
    elif args.command == "restore":
        success = config_manager.restore_configs(args.backup_path)
        sys.exit(0 if success else 1)
        
    elif args.command == "list-backups":
        backups = config_manager.list_backups()
        if backups:
            print("📁 可用备份:")
            for backup in backups:
                print(f"  - {backup}")
        else:
            print("📁 无可用备份")
            
    elif args.command == "optimize":
        success = config_manager.optimize_for_environment(args.environment)
        sys.exit(0 if success else 1)
        
    elif args.command == "summary":
        summary = config_manager.generate_summary()
        print("📊 配置摘要:")
        print(json.dumps(summary, indent=2, ensure_ascii=False))

if __name__ == "__main__":
    main() 