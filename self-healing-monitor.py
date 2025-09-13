#!/usr/bin/env python3
"""
5.1套利系统 - 自愈式监控系统
Self-Healing Monitoring System for 5.1 Arbitrage System

功能 (Features):
1. 持续健康监控 (Continuous Health Monitoring)
2. 智能故障检测 (Intelligent Failure Detection) 
3. 自动故障恢复 (Automatic Failure Recovery)
4. 性能指标收集 (Performance Metrics Collection)
5. 告警与通知 (Alerting & Notifications)
6. 负载均衡建议 (Load Balancing Recommendations)
7. 预测性维护 (Predictive Maintenance)
"""

import asyncio
import aiohttp
import json
import time
import logging
import psutil
import subprocess
import statistics
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, asdict
from collections import deque, defaultdict
import signal
import sys
import os

# 配置日志
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s',
    handlers=[
        logging.FileHandler('/home/ubuntu/5.1xitong/logs/self-healing-monitor.log'),
        logging.StreamHandler()
    ]
)
logger = logging.getLogger(__name__)

@dataclass
class ServiceConfig:
    name: str
    port: int
    health_path: str
    critical: bool
    max_response_time_ms: int = 5000
    failure_threshold: int = 3
    recovery_threshold: int = 2

@dataclass
class HealthCheck:
    timestamp: datetime
    service_name: str
    healthy: bool
    response_time_ms: int
    status_code: Optional[int] = None
    error_message: Optional[str] = None
    cpu_usage: float = 0.0
    memory_usage: float = 0.0

@dataclass
class ServiceMetrics:
    service_name: str
    avg_response_time: float
    success_rate: float
    uptime_percentage: float
    total_requests: int
    failed_requests: int
    last_failure: Optional[datetime] = None
    failure_streak: int = 0
    recovery_streak: int = 0

class SelfHealingMonitor:
    def __init__(self):
        self.services = {
            'unified-gateway': ServiceConfig('unified-gateway', 3000, '/health', True, 3000),
            'logging-service': ServiceConfig('logging-service', 4001, '/health', True),
            'cleaning-service': ServiceConfig('cleaning-service', 4002, '/health', False),
            'strategy-service': ServiceConfig('strategy-service', 4003, '/health', True),
            'performance-service': ServiceConfig('performance-service', 4004, '/health', False),
            'trading-service': ServiceConfig('trading-service', 4005, '/health', True),
            'ai-model-service': ServiceConfig('ai-model-service', 4006, '/health', False),
            'config-service': ServiceConfig('config-service', 4007, '/health', True),
        }
        
        # 监控配置
        self.check_interval = 10  # 检查间隔（秒）
        self.metrics_window = 300  # 指标时间窗口（秒）
        self.auto_repair_enabled = True
        self.alert_cooldown = 300  # 告警冷却时间（秒）
        
        # 数据存储
        self.health_history: Dict[str, deque] = defaultdict(lambda: deque(maxlen=100))
        self.metrics: Dict[str, ServiceMetrics] = {}
        self.last_alerts: Dict[str, datetime] = {}
        self.system_load_history = deque(maxlen=60)
        
        # 状态跟踪
        self.running = False
        self.session: Optional[aiohttp.ClientSession] = None
        
        logger.info("Self-healing monitor initialized with %d services", len(self.services))

    async def __aenter__(self):
        connector = aiohttp.TCPConnector(limit=30, limit_per_host=10)
        timeout = aiohttp.ClientTimeout(total=10, connect=3)
        self.session = aiohttp.ClientSession(connector=connector, timeout=timeout)
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()

    async def _check_service_health(self, service_config: ServiceConfig) -> HealthCheck:
        """检查单个服务健康状态"""
        start_time = time.time()
        
        try:
            url = f"http://localhost:{service_config.port}{service_config.health_path}"
            
            async with self.session.get(url) as response:
                response_time_ms = int((time.time() - start_time) * 1000)
                
                # 获取系统资源使用情况
                cpu_usage, memory_usage = self._get_service_resources(service_config.name)
                
                if response.status == 200:
                    return HealthCheck(
                        timestamp=datetime.now(),
                        service_name=service_config.name,
                        healthy=True,
                        response_time_ms=response_time_ms,
                        status_code=response.status,
                        cpu_usage=cpu_usage,
                        memory_usage=memory_usage
                    )
                else:
                    return HealthCheck(
                        timestamp=datetime.now(),
                        service_name=service_config.name,
                        healthy=False,
                        response_time_ms=response_time_ms,
                        status_code=response.status,
                        error_message=f"HTTP {response.status}",
                        cpu_usage=cpu_usage,
                        memory_usage=memory_usage
                    )
                    
        except asyncio.TimeoutError:
            return HealthCheck(
                timestamp=datetime.now(),
                service_name=service_config.name,
                healthy=False,
                response_time_ms=int((time.time() - start_time) * 1000),
                error_message="Timeout"
            )
        except Exception as e:
            return HealthCheck(
                timestamp=datetime.now(),
                service_name=service_config.name,
                healthy=False,
                response_time_ms=int((time.time() - start_time) * 1000),
                error_message=str(e)
            )

    def _get_service_resources(self, service_name: str) -> Tuple[float, float]:
        """获取服务的CPU和内存使用率"""
        try:
            # 查找服务进程
            for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
                try:
                    cmdline = ' '.join(proc.info['cmdline']) if proc.info['cmdline'] else ''
                    if service_name in cmdline or service_name in proc.info['name']:
                        # 获取CPU和内存使用率
                        cpu_percent = proc.cpu_percent(interval=None)
                        memory_percent = proc.memory_percent()
                        return cpu_percent, memory_percent
                except (psutil.NoSuchProcess, psutil.AccessDenied):
                    continue
        except Exception as e:
            logger.debug(f"Failed to get resource usage for {service_name}: {e}")
        
        return 0.0, 0.0

    async def _restart_service(self, service_name: str) -> bool:
        """重启服务"""
        try:
            logger.info(f"🔄 Restarting service: {service_name}")
            
            # 使用自动服务管理器重启
            cmd = ["/home/ubuntu/5.1xitong/auto-service-manager.sh", "restart", service_name]
            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            stdout, stderr = await process.communicate()
            
            if process.returncode == 0:
                logger.info(f"✅ Service {service_name} restarted successfully")
                return True
            else:
                logger.error(f"❌ Failed to restart {service_name}: {stderr.decode()}")
                return False
                
        except Exception as e:
            logger.error(f"❌ Error restarting {service_name}: {e}")
            return False

    async def get_status_report(self) -> Dict:
        """获取状态报告"""
        # 检查所有服务健康状态
        tasks = []
        for service_config in self.services.values():
            task = asyncio.create_task(self._check_service_health(service_config))
            tasks.append(task)
        
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        health_checks = {}
        for i, result in enumerate(results):
            service_name = list(self.services.keys())[i]
            if isinstance(result, Exception):
                health_checks[service_name] = HealthCheck(
                    timestamp=datetime.now(),
                    service_name=service_name,
                    healthy=False,
                    response_time_ms=0,
                    error_message=str(result)
                )
            else:
                health_checks[service_name] = result
        
        # 生成报告
        healthy_count = sum(1 for hc in health_checks.values() if hc.healthy)
        unhealthy_count = len(health_checks) - healthy_count
        
        service_reports = {}
        for service_name, health_check in health_checks.items():
            service_config = self.services[service_name]
            service_reports[service_name] = {
                "healthy": health_check.healthy,
                "port": service_config.port,
                "critical": service_config.critical,
                "response_time_ms": health_check.response_time_ms,
                "status_code": health_check.status_code,
                "error_message": health_check.error_message,
                "cpu_usage": health_check.cpu_usage,
                "memory_usage": health_check.memory_usage,
                "last_check": health_check.timestamp.isoformat()
            }
        
        return {
            "timestamp": datetime.now().isoformat(),
            "summary": {
                "total_services": len(self.services),
                "healthy_services": healthy_count,
                "unhealthy_services": unhealthy_count,
                "overall_health_score": (healthy_count / len(self.services)) * 100
            },
            "services": service_reports
        }


async def main():
    """主函数"""
    import argparse
    
    parser = argparse.ArgumentParser(description='5.1套利系统自愈式监控器')
    parser.add_argument('command', choices=['status', 'repair'], 
                       help='Command to execute')
    
    args = parser.parse_args()
    
    async with SelfHealingMonitor() as monitor:
        if args.command == 'status':
            report = await monitor.get_status_report()
            print(json.dumps(report, indent=2, ensure_ascii=False))
            
        elif args.command == 'repair':
            # 执行一次修复
            report = await monitor.get_status_report()
            
            repair_count = 0
            for service_name, service_info in report['services'].items():
                if not service_info['healthy']:
                    logger.info(f"🔧 Repairing {service_name}...")
                    success = await monitor._restart_service(service_name)
                    if success:
                        repair_count += 1
            
            logger.info(f"✅ Repair completed: {repair_count} services repaired")


if __name__ == "__main__":
    asyncio.run(main())