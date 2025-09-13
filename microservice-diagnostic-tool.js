#!/usr/bin/env node

/**
 * 5.1套利系统 - 微服务自动化诊断与修复工具
 * Comprehensive Microservice Diagnostic & Auto-Repair System
 * 
 * 功能:
 * 1. 服务发现与状态检查
 * 2. 自动修复机制
 * 3. 实时监控系统
 * 4. 智能故障恢复
 * 5. 性能监控与优化
 */

const { exec, spawn } = require('child_process');
const fs = require('fs').promises;
const path = require('path');
const http = require('http');
const util = require('util');

const execAsync = util.promisify(exec);

// 微服务配置
const MICROSERVICES = {
  'unified-gateway': { port: 3000, healthPath: '/health', critical: true },
  'logging-service': { port: 4001, healthPath: '/health', critical: true },
  'cleaning-service': { port: 4002, healthPath: '/health', critical: false },
  'strategy-service': { port: 4003, healthPath: '/health', critical: true },
  'performance-service': { port: 4004, healthPath: '/health', critical: false },
  'trading-service': { port: 4005, healthPath: '/health', critical: true },
  'ai-model-service': { port: 4006, healthPath: '/health', critical: false },
  'config-service': { port: 4007, healthPath: '/health', critical: true }
};

const FRONTEND_CONFIG = {
  port: 3003,
  healthPath: '/',
  name: 'arbitrage-frontend-v5.1'
};

const API_SERVER_CONFIG = {
  port: 3001,
  healthPath: '/health',
  name: 'real-api-server'
};

class MicroserviceDiagnostic {
  constructor() {
    this.healthStatus = new Map();
    this.lastHealthCheck = new Date();
    this.isMonitoring = false;
    this.alertThreshold = 3; // 连续失败次数阈值
    this.retryAttempts = new Map();
    this.logFile = '/home/ubuntu/5.1xitong/diagnostic.log';
  }

  // 日志记录
  async log(level, message, data = null) {
    const timestamp = new Date().toISOString();
    const logEntry = {
      timestamp,
      level,
      message,
      data,
      pid: process.pid
    };
    
    const logLine = `[${timestamp}] [${level}] ${message}${data ? ' | Data: ' + JSON.stringify(data) : ''}\n`;
    
    try {
      await fs.appendFile(this.logFile, logLine);
    } catch (err) {
      console.error('Failed to write log:', err.message);
    }
    
    // 同时输出到控制台
    const colors = {
      'INFO': '\x1b[36m',    // 青色
      'WARN': '\x1b[33m',    // 黄色
      'ERROR': '\x1b[31m',   // 红色
      'SUCCESS': '\x1b[32m', // 绿色
      'DEBUG': '\x1b[90m'    // 灰色
    };
    
    console.log(`${colors[level] || '\x1b[0m'}${logLine.trim()}\x1b[0m`);
  }

  // 检查端口是否在监听
  async checkPortListening(port) {
    try {
      const { stdout } = await execAsync(`ss -tlnp | grep ":${port}"`);
      return stdout.trim().length > 0;
    } catch (error) {
      return false;
    }
  }

  // HTTP健康检查
  async httpHealthCheck(host, port, path, timeout = 5000) {
    return new Promise((resolve) => {
      const req = http.request({
        host,
        port,
        path,
        method: 'GET',
        timeout
      }, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
          const isHealthy = res.statusCode >= 200 && res.statusCode < 300;
          resolve({
            healthy: isHealthy,
            status: res.statusCode,
            response: data,
            responseTime: Date.now() - startTime
          });
        });
      });

      const startTime = Date.now();
      
      req.on('error', (err) => {
        resolve({
          healthy: false,
          error: err.message,
          responseTime: Date.now() - startTime
        });
      });

      req.on('timeout', () => {
        req.destroy();
        resolve({
          healthy: false,
          error: 'Request timeout',
          responseTime: timeout
        });
      });

      req.end();
    });
  }

  // 单个服务健康检查
  async checkServiceHealth(serviceName, config) {
    const startTime = Date.now();
    
    try {
      // 检查端口是否监听
      const portListening = await this.checkPortListening(config.port);
      
      if (!portListening) {
        return {
          service: serviceName,
          healthy: false,
          error: 'Port not listening',
          port: config.port,
          responseTime: Date.now() - startTime
        };
      }

      // HTTP健康检查
      const healthResult = await this.httpHealthCheck('localhost', config.port, config.healthPath);
      
      return {
        service: serviceName,
        healthy: healthResult.healthy,
        port: config.port,
        status: healthResult.status,
        response: healthResult.response,
        responseTime: healthResult.responseTime,
        error: healthResult.error
      };
    } catch (error) {
      return {
        service: serviceName,
        healthy: false,
        error: error.message,
        port: config.port,
        responseTime: Date.now() - startTime
      };
    }
  }

  // 检查所有微服务状态
  async checkAllServices() {
    await this.log('INFO', 'Starting comprehensive health check for all services');
    
    const results = {
      timestamp: new Date().toISOString(),
      microservices: {},
      frontend: null,
      apiServer: null,
      summary: {
        total: 0,
        healthy: 0,
        unhealthy: 0,
        critical_failures: 0
      }
    };

    // 检查微服务
    for (const [serviceName, config] of Object.entries(MICROSERVICES)) {
      const health = await this.checkServiceHealth(serviceName, config);
      results.microservices[serviceName] = health;
      
      results.summary.total++;
      if (health.healthy) {
        results.summary.healthy++;
        await this.log('SUCCESS', `Service ${serviceName} is healthy`, health);
      } else {
        results.summary.unhealthy++;
        if (config.critical) {
          results.summary.critical_failures++;
        }
        await this.log('ERROR', `Service ${serviceName} is unhealthy`, health);
      }
    }

    // 检查前端
    results.frontend = await this.checkServiceHealth('frontend', {
      port: FRONTEND_CONFIG.port,
      healthPath: FRONTEND_CONFIG.healthPath
    });

    // 检查API服务器
    results.apiServer = await this.checkServiceHealth('api-server', {
      port: API_SERVER_CONFIG.port,
      healthPath: API_SERVER_CONFIG.healthPath
    });

    this.lastHealthCheck = new Date();
    this.healthStatus = results;

    await this.log('INFO', 'Health check completed', results.summary);
    
    return results;
  }

  // 尝试启动服务
  async startService(serviceName) {
    await this.log('INFO', `Attempting to start service: ${serviceName}`);
    
    const serviceDir = `/home/ubuntu/5.1xitong/5.1系统/${serviceName}`;
    const systemDir = '/home/ubuntu/5.1xitong/5.1系统';
    
    try {
      // 检查服务目录是否存在
      try {
        await fs.access(serviceDir);
      } catch {
        await this.log('ERROR', `Service directory not found: ${serviceDir}`);
        return false;
      }

      // 使用统一启动脚本
      const startScript = path.join(systemDir, 'start_all_services.sh');
      try {
        await fs.access(startScript);
        await this.log('INFO', `Using unified start script: ${startScript}`);
        
        const { stdout, stderr } = await execAsync(`cd ${systemDir} && ./start_all_services.sh`, { 
          timeout: 30000 
        });
        
        await this.log('SUCCESS', `Start script executed for ${serviceName}`, { stdout, stderr });
        
        // 等待服务启动
        await this.sleep(5000);
        
        // 验证服务是否启动成功
        const health = await this.checkServiceHealth(serviceName, MICROSERVICES[serviceName]);
        if (health.healthy) {
          await this.log('SUCCESS', `Service ${serviceName} started successfully`);
          return true;
        } else {
          await this.log('WARN', `Service ${serviceName} start script executed but health check failed`);
          return false;
        }
        
      } catch (startError) {
        await this.log('ERROR', `Failed to execute start script for ${serviceName}`, startError.message);
        
        // 尝试直接启动单个服务
        return await this.startSingleService(serviceName, serviceDir);
      }
      
    } catch (error) {
      await this.log('ERROR', `Failed to start service ${serviceName}`, error.message);
      return false;
    }
  }

  // 启动单个服务
  async startSingleService(serviceName, serviceDir) {
    try {
      await this.log('INFO', `Attempting direct start for ${serviceName} in ${serviceDir}`);
      
      // 尝试使用 cargo run
      const cargoCommand = `cd ${serviceDir} && RUST_LOG=info nohup cargo run --release > ../${serviceName}.log 2>&1 &`;
      
      await execAsync(cargoCommand, { timeout: 10000 });
      await this.log('INFO', `Executed cargo run for ${serviceName}`);
      
      // 等待启动
      await this.sleep(8000);
      
      return true;
    } catch (error) {
      await this.log('ERROR', `Failed to start ${serviceName} directly`, error.message);
      return false;
    }
  }

  // 自动修复不健康的服务
  async autoRepair() {
    await this.log('INFO', 'Starting automatic repair process');
    
    const healthResults = await this.checkAllServices();
    const repairResults = {
      attempted: [],
      succeeded: [],
      failed: [],
      skipped: []
    };

    for (const [serviceName, health] of Object.entries(healthResults.microservices)) {
      if (!health.healthy) {
        const config = MICROSERVICES[serviceName];
        
        // 检查是否已经尝试过多次修复
        const retryCount = this.retryAttempts.get(serviceName) || 0;
        
        if (retryCount >= this.alertThreshold) {
          await this.log('WARN', `Service ${serviceName} exceeded retry threshold (${retryCount}), skipping auto-repair`);
          repairResults.skipped.push(serviceName);
          continue;
        }

        repairResults.attempted.push(serviceName);
        this.retryAttempts.set(serviceName, retryCount + 1);
        
        await this.log('WARN', `Attempting to repair unhealthy service: ${serviceName} (attempt ${retryCount + 1})`);
        
        const repairSuccess = await this.startService(serviceName);
        
        if (repairSuccess) {
          repairResults.succeeded.push(serviceName);
          this.retryAttempts.set(serviceName, 0); // 重置重试计数
          await this.log('SUCCESS', `Successfully repaired service: ${serviceName}`);
        } else {
          repairResults.failed.push(serviceName);
          await this.log('ERROR', `Failed to repair service: ${serviceName}`);
        }
        
        // 在修复尝试之间添加延迟
        await this.sleep(2000);
      }
    }

    await this.log('INFO', 'Auto-repair process completed', repairResults);
    return repairResults;
  }

  // 生成状态报告
  generateStatusReport() {
    if (!this.healthStatus || !this.healthStatus.microservices) {
      return 'No health data available. Run health check first.';
    }

    let report = '\n' + '='.repeat(80) + '\n';
    report += '           5.1套利系统 - 微服务状态报告\n';
    report += '           Microservice Health Status Report\n';
    report += '='.repeat(80) + '\n';
    
    report += `检查时间 (Timestamp): ${this.healthStatus.timestamp}\n`;
    report += `总服务数 (Total Services): ${this.healthStatus.summary.total}\n`;
    report += `健康服务 (Healthy): ${this.healthStatus.summary.healthy}\n`;
    report += `异常服务 (Unhealthy): ${this.healthStatus.summary.unhealthy}\n`;
    report += `关键服务异常 (Critical Failures): ${this.healthStatus.summary.critical_failures}\n\n`;

    // 微服务详情
    report += '微服务详情 (Microservices Details):\n';
    report += '-'.repeat(50) + '\n';
    
    for (const [serviceName, health] of Object.entries(this.healthStatus.microservices)) {
      const config = MICROSERVICES[serviceName];
      const status = health.healthy ? '✅ HEALTHY' : '❌ UNHEALTHY';
      const critical = config.critical ? ' [CRITICAL]' : ' [NON-CRITICAL]';
      
      report += `${serviceName}${critical}:\n`;
      report += `  状态: ${status}\n`;
      report += `  端口: ${health.port}\n`;
      report += `  响应时间: ${health.responseTime}ms\n`;
      
      if (health.healthy && health.response) {
        try {
          const responseData = JSON.parse(health.response);
          report += `  API数量: ${responseData.data?.apis_count || responseData.data?.apis || 'N/A'}\n`;
        } catch (e) {
          // Ignore parsing errors
        }
      }
      
      if (!health.healthy) {
        report += `  错误: ${health.error || 'Unknown error'}\n`;
      }
      
      report += '\n';
    }

    // 前端和API服务器状态
    report += '其他服务 (Other Services):\n';
    report += '-'.repeat(30) + '\n';
    
    if (this.healthStatus.frontend) {
      const status = this.healthStatus.frontend.healthy ? '✅ HEALTHY' : '❌ UNHEALTHY';
      report += `前端服务 (Frontend): ${status} (Port: ${this.healthStatus.frontend.port})\n`;
    }
    
    if (this.healthStatus.apiServer) {
      const status = this.healthStatus.apiServer.healthy ? '✅ HEALTHY' : '❌ UNHEALTHY';
      report += `API服务器 (API Server): ${status} (Port: ${this.healthStatus.apiServer.port})\n`;
    }

    report += '\n' + '='.repeat(80) + '\n';
    
    return report;
  }

  // 启动持续监控
  async startContinuousMonitoring(intervalMs = 30000) {
    if (this.isMonitoring) {
      await this.log('WARN', 'Monitoring is already running');
      return;
    }

    this.isMonitoring = true;
    await this.log('INFO', `Starting continuous monitoring with ${intervalMs}ms interval`);

    const monitor = async () => {
      try {
        if (!this.isMonitoring) return;

        await this.log('DEBUG', 'Running scheduled health check');
        const healthResults = await this.checkAllServices();
        
        // 检查是否有不健康的关键服务
        const criticalFailures = Object.entries(healthResults.microservices)
          .filter(([name, health]) => !health.healthy && MICROSERVICES[name].critical)
          .map(([name]) => name);

        if (criticalFailures.length > 0) {
          await this.log('ERROR', 'Critical service failures detected, starting auto-repair', criticalFailures);
          await this.autoRepair();
        }

        // 打印状态报告（仅在有变化时）
        if (healthResults.summary.unhealthy > 0) {
          console.log(this.generateStatusReport());
        }

      } catch (error) {
        await this.log('ERROR', 'Error during monitoring cycle', error.message);
      }

      // 调度下一次检查
      if (this.isMonitoring) {
        setTimeout(monitor, intervalMs);
      }
    };

    // 启动监控循环
    setTimeout(monitor, 0);
  }

  // 停止持续监控
  async stopMonitoring() {
    this.isMonitoring = false;
    await this.log('INFO', 'Continuous monitoring stopped');
  }

  // 辅助函数：休眠
  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  // 获取系统资源使用情况
  async getSystemResources() {
    try {
      const { stdout: memOutput } = await execAsync('free -m');
      const { stdout: diskOutput } = await execAsync('df -h /');
      const { stdout: cpuOutput } = await execAsync('top -bn1 | grep "load average"');
      
      return {
        memory: memOutput,
        disk: diskOutput,
        cpu: cpuOutput,
        timestamp: new Date().toISOString()
      };
    } catch (error) {
      await this.log('ERROR', 'Failed to get system resources', error.message);
      return null;
    }
  }

  // 清理日志文件
  async cleanupLogs(maxAgeDays = 7) {
    try {
      const { stdout } = await execAsync(`find /home/ubuntu/5.1xitong -name "*.log" -mtime +${maxAgeDays} -type f`);
      const oldLogs = stdout.trim().split('\n').filter(line => line.length > 0);
      
      for (const logFile of oldLogs) {
        await fs.unlink(logFile);
        await this.log('INFO', `Cleaned up old log file: ${logFile}`);
      }
      
      return oldLogs.length;
    } catch (error) {
      await this.log('ERROR', 'Failed to cleanup logs', error.message);
      return 0;
    }
  }
}

// CLI 接口
async function main() {
  const diagnostic = new MicroserviceDiagnostic();
  const command = process.argv[2] || 'status';

  try {
    switch (command) {
      case 'status':
      case 'check':
        await diagnostic.log('INFO', 'Running health check command');
        await diagnostic.checkAllServices();
        console.log(diagnostic.generateStatusReport());
        break;

      case 'repair':
        await diagnostic.log('INFO', 'Running repair command');
        const repairResults = await diagnostic.autoRepair();
        console.log('\n修复结果 (Repair Results):');
        console.log('尝试修复:', repairResults.attempted);
        console.log('修复成功:', repairResults.succeeded);
        console.log('修复失败:', repairResults.failed);
        console.log('跳过修复:', repairResults.skipped);
        break;

      case 'monitor':
        const interval = parseInt(process.argv[3]) || 30000;
        await diagnostic.log('INFO', `Starting monitoring mode with ${interval}ms interval`);
        console.log(`🔍 Starting continuous monitoring (interval: ${interval}ms)`);
        console.log('Press Ctrl+C to stop...');
        
        // 处理优雅退出
        process.on('SIGINT', async () => {
          await diagnostic.stopMonitoring();
          console.log('\n👋 Monitoring stopped');
          process.exit(0);
        });
        
        await diagnostic.startContinuousMonitoring(interval);
        break;

      case 'resources':
        await diagnostic.log('INFO', 'Checking system resources');
        const resources = await diagnostic.getSystemResources();
        if (resources) {
          console.log('\n系统资源使用情况 (System Resources):');
          console.log('Memory:', resources.memory);
          console.log('Disk:', resources.disk);
          console.log('CPU Load:', resources.cpu);
        }
        break;

      case 'cleanup':
        await diagnostic.log('INFO', 'Running log cleanup');
        const cleanedCount = await diagnostic.cleanupLogs();
        console.log(`清理了 ${cleanedCount} 个旧日志文件`);
        break;

      case 'help':
        console.log(`
5.1套利系统微服务诊断工具 (Microservice Diagnostic Tool)

使用方法 (Usage):
  node microservice-diagnostic-tool.js [command] [options]

命令 (Commands):
  status|check         检查所有服务状态 (Check all service status)
  repair               自动修复不健康的服务 (Auto-repair unhealthy services)  
  monitor [interval]   启动持续监控 (Start continuous monitoring)
  resources            显示系统资源使用 (Show system resource usage)
  cleanup              清理旧日志文件 (Cleanup old log files)
  help                 显示帮助信息 (Show help)

示例 (Examples):
  node microservice-diagnostic-tool.js status
  node microservice-diagnostic-tool.js repair
  node microservice-diagnostic-tool.js monitor 60000
  node microservice-diagnostic-tool.js resources
        `);
        break;

      default:
        console.error(`Unknown command: ${command}`);
        console.log('Use "help" for usage information');
        process.exit(1);
    }
  } catch (error) {
    await diagnostic.log('ERROR', 'Command execution failed', error.message);
    console.error('Error:', error.message);
    process.exit(1);
  }
}

// 如果直接运行此脚本
if (require.main === module) {
  main().catch(console.error);
}

module.exports = MicroserviceDiagnostic;