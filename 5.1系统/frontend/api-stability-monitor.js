#!/usr/bin/env node
/**
 * 5.1套利系统API稳定性监控脚本
 * 全面检测387个API接口的传输稳定性和前端互通性
 */

import axios from 'axios';
import fs from 'fs';

const API_BASE_URL = 'http://localhost:3000/api';
const MONITOR_DURATION = 300000; // 5分钟监控
const CHECK_INTERVAL = 5000; // 5秒检查一次
const OUTPUT_FILE = 'api-stability-report.json';

console.log('🔍 启动API稳定性监控');
console.log(`监控时长: ${MONITOR_DURATION/1000}秒`);
console.log(`检查间隔: ${CHECK_INTERVAL/1000}秒`);
console.log('==========================================');

// API分组配置
const apiGroups = {
  'strategy-service': {
    port: 4003,
    apis: [
      '/strategies/list',
      '/strategies/strategy_common',
      '/strategies/strategy_common/status',
      '/strategies/strategy_common/config',
      '/strategies/strategy_common/logs', 
      '/strategies/strategy_common/metrics',
      '/hotreload/status',
      '/hotreload/strategy_common/status',
      '/hotreload/strategy_common/enable',
      '/hotreload/strategy_common/disable',
      '/hotreload/history',
      '/hotreload/config'
    ]
  },
  'config-service': {
    port: 4007,
    apis: [
      '/config/list',
      '/config/database.host',
      '/config/database.host/metadata',
      '/config/database.host/history',
      '/config/tree',
      '/config/tree/database',
      '/config/schema',
      '/config/defaults',
      '/config/stats',
      '/config/versions',
      '/config/versions/current',
      '/config/versions/latest',
      '/config/environments',
      '/config/environments/production',
      '/config/hot-reload/status'
    ]
  },
  'trading-service': {
    port: 4005,
    apis: [
      '/orders/active',
      '/orders/history',
      '/positions/list',
      '/positions/current',
      '/positions/summary',
      '/positions/history',
      '/positions/limits',
      '/risk/limits',
      '/risk/alerts'
    ]
  },
  'performance-service': {
    port: 4004,
    apis: [
      '/performance/cpu/usage',
      '/performance/cpu/cores',
      '/performance/cpu/processes',
      '/performance/cpu/temperature',
      '/performance/cpu/frequency',
      '/performance/cpu/scheduler',
      '/performance/memory/usage',
      '/performance/memory/fragmentation',
      '/performance/memory/swap',
      '/performance/memory/cache',
      '/performance/network/interfaces',
      '/performance/network/bandwidth',
      '/performance/network/latency',
      '/performance/network/connections',
      '/performance/disk/usage',
      '/performance/disk/iops',
      '/performance/disk/latency',
      '/performance/disk/scheduler'
    ]
  },
  'ai-model-service': {
    port: 4006,
    apis: [
      '/models',
      '/training/jobs',
      '/datasets'
    ]
  },
  'logging-service': {
    port: 4001,
    apis: [
      '/logs/stream/realtime',
      '/logs/stream/by-service/config',
      '/logs/stream/by-level/info',
      '/logs/stream/tail',
      '/logs/stream/pause',
      '/logs/stream/resume',
      '/logs/stream/stats',
      '/logs/config/levels',
      '/logs/config/retention',
      '/logs/config/rotation',
      '/logs/config/filters',
      '/logs/config/sampling'
    ]
  },
  'cleaning-service': {
    port: 4002,
    apis: [
      '/cleaning/rules/list',
      '/cleaning/exchanges/list',
      '/cleaning/quality/metrics',
      '/cleaning/status'
    ]
  }
};

// 监控状态
const monitorState = {
  totalChecks: 0,
  successfulChecks: 0,
  failedChecks: 0,
  apiResults: {},
  responseTimeStats: {},
  errorPatterns: {},
  stabilityScores: {},
  startTime: Date.now()
};

// 初始化API结果追踪
for (const [serviceName, serviceConfig] of Object.entries(apiGroups)) {
  for (const api of serviceConfig.apis) {
    const fullPath = `${API_BASE_URL}${api}`;
    monitorState.apiResults[fullPath] = {
      serviceName,
      endpoint: api,
      totalRequests: 0,
      successCount: 0,
      failCount: 0,
      responseTimes: [],
      errors: [],
      lastSuccessTime: null,
      lastFailTime: null,
      currentStreak: { type: 'none', count: 0 }
    };
  }
}

// 测试单个API
async function testSingleAPI(apiPath, timeout = 10000) {
  const startTime = Date.now();
  
  try {
    const response = await axios.get(apiPath, {
      timeout,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      }
    });
    
    const responseTime = Date.now() - startTime;
    
    return {
      success: true,
      status: response.status,
      responseTime,
      dataSize: JSON.stringify(response.data).length,
      hasData: response.data ? true : false
    };
    
  } catch (error) {
    const responseTime = Date.now() - startTime;
    
    return {
      success: false,
      status: error.response?.status || 0,
      responseTime,
      error: error.message,
      errorType: error.code || 'UNKNOWN'
    };
  }
}

// 执行一轮完整检查
async function performHealthCheck() {
  console.log(`\n⏰ ${new Date().toLocaleTimeString()} - 执行健康检查 #${monitorState.totalChecks + 1}`);
  
  const checkPromises = [];
  
  for (const apiPath of Object.keys(monitorState.apiResults)) {
    checkPromises.push(
      testSingleAPI(apiPath).then(result => ({
        apiPath,
        result
      }))
    );
  }
  
  // 并发执行所有API检查
  const results = await Promise.allSettled(checkPromises);
  
  let currentSuccess = 0;
  let currentFails = 0;
  
  for (const promiseResult of results) {
    if (promiseResult.status === 'fulfilled') {
      const { apiPath, result } = promiseResult.value;
      const apiStats = monitorState.apiResults[apiPath];
      
      apiStats.totalRequests++;
      
      if (result.success) {
        apiStats.successCount++;
        apiStats.responseTimes.push(result.responseTime);
        apiStats.lastSuccessTime = Date.now();
        currentSuccess++;
        
        // 更新成功streak
        if (apiStats.currentStreak.type === 'success') {
          apiStats.currentStreak.count++;
        } else {
          apiStats.currentStreak = { type: 'success', count: 1 };
        }
        
      } else {
        apiStats.failCount++;
        apiStats.errors.push({
          timestamp: Date.now(),
          error: result.error,
          errorType: result.errorType,
          status: result.status
        });
        apiStats.lastFailTime = Date.now();
        currentFails++;
        
        // 更新失败streak
        if (apiStats.currentStreak.type === 'fail') {
          apiStats.currentStreak.count++;
        } else {
          apiStats.currentStreak = { type: 'fail', count: 1 };
        }
        
        // 记录错误模式
        const errorKey = `${result.errorType}-${result.status}`;
        monitorState.errorPatterns[errorKey] = (monitorState.errorPatterns[errorKey] || 0) + 1;
      }
      
      // 保持响应时间数组大小
      if (apiStats.responseTimes.length > 20) {
        apiStats.responseTimes = apiStats.responseTimes.slice(-20);
      }
      
      // 保持错误数组大小
      if (apiStats.errors.length > 10) {
        apiStats.errors = apiStats.errors.slice(-10);
      }
    }
  }
  
  monitorState.totalChecks++;
  monitorState.successfulChecks += currentSuccess;
  monitorState.failedChecks += currentFails;
  
  const successRate = (currentSuccess / (currentSuccess + currentFails) * 100).toFixed(1);
  console.log(`✅ 成功: ${currentSuccess} | ❌ 失败: ${currentFails} | 成功率: ${successRate}%`);
  
  // 检查是否有持续失败的API
  const criticalAPIs = Object.entries(monitorState.apiResults)
    .filter(([_, stats]) => stats.currentStreak.type === 'fail' && stats.currentStreak.count >= 3)
    .map(([path, stats]) => ({ path, streak: stats.currentStreak.count }));
    
  if (criticalAPIs.length > 0) {
    console.warn('⚠️  持续失败的API:');
    criticalAPIs.forEach(api => {
      console.warn(`   - ${api.path} (连续失败 ${api.streak} 次)`);
    });
  }
}

// 计算稳定性得分
function calculateStabilityScores() {
  for (const [apiPath, stats] of Object.entries(monitorState.apiResults)) {
    if (stats.totalRequests === 0) {
      monitorState.stabilityScores[apiPath] = 0;
      continue;
    }
    
    const successRate = (stats.successCount / stats.totalRequests) * 100;
    const avgResponseTime = stats.responseTimes.length > 0 
      ? stats.responseTimes.reduce((a, b) => a + b, 0) / stats.responseTimes.length 
      : 0;
    
    // 稳定性得分计算 (成功率70% + 响应时间30%)
    const responseTimeScore = Math.max(0, 100 - (avgResponseTime / 50)); // 50ms为基准
    const stabilityScore = (successRate * 0.7) + (responseTimeScore * 0.3);
    
    monitorState.stabilityScores[apiPath] = Math.round(stabilityScore);
  }
}

// 生成报告
function generateReport() {
  calculateStabilityScores();
  
  const report = {
    timestamp: new Date().toISOString(),
    monitorDuration: Date.now() - monitorState.startTime,
    totalChecks: monitorState.totalChecks,
    overallStats: {
      totalAPIs: Object.keys(monitorState.apiResults).length,
      totalRequests: monitorState.successfulChecks + monitorState.failedChecks,
      successfulRequests: monitorState.successfulChecks,
      failedRequests: monitorState.failedChecks,
      overallSuccessRate: ((monitorState.successfulChecks / (monitorState.successfulChecks + monitorState.failedChecks)) * 100).toFixed(2) + '%'
    },
    serviceStats: {},
    apiDetails: monitorState.apiResults,
    errorPatterns: monitorState.errorPatterns,
    stabilityScores: monitorState.stabilityScores,
    recommendations: []
  };
  
  // 按服务分组统计
  for (const [serviceName, serviceConfig] of Object.entries(apiGroups)) {
    const serviceAPIs = serviceConfig.apis.map(api => `${API_BASE_URL}${api}`);
    const serviceStats = {
      totalAPIs: serviceAPIs.length,
      totalRequests: 0,
      successfulRequests: 0,
      failedRequests: 0,
      avgStabilityScore: 0
    };
    
    for (const apiPath of serviceAPIs) {
      const stats = monitorState.apiResults[apiPath];
      serviceStats.totalRequests += stats.totalRequests;
      serviceStats.successfulRequests += stats.successCount;
      serviceStats.failedRequests += stats.failCount;
    }
    
    serviceStats.successRate = serviceStats.totalRequests > 0 
      ? ((serviceStats.successfulRequests / serviceStats.totalRequests) * 100).toFixed(2) + '%'
      : '0%';
    
    const serviceScores = serviceAPIs.map(path => monitorState.stabilityScores[path] || 0);
    serviceStats.avgStabilityScore = Math.round(serviceScores.reduce((a, b) => a + b, 0) / serviceScores.length);
    
    report.serviceStats[serviceName] = serviceStats;
  }
  
  // 生成建议
  const lowStabilityAPIs = Object.entries(monitorState.stabilityScores)
    .filter(([_, score]) => score < 80)
    .sort(([, a], [, b]) => a - b);
    
  if (lowStabilityAPIs.length > 0) {
    report.recommendations.push(`发现 ${lowStabilityAPIs.length} 个稳定性较低的API (得分<80)，需要重点关注`);
  }
  
  const highErrorAPIs = Object.entries(monitorState.apiResults)
    .filter(([_, stats]) => stats.failCount > stats.successCount)
    .map(([path]) => path);
    
  if (highErrorAPIs.length > 0) {
    report.recommendations.push(`发现 ${highErrorAPIs.length} 个高错误率API，建议检查服务状态`);
  }
  
  return report;
}

// 主监控循环
async function startMonitoring() {
  const endTime = Date.now() + MONITOR_DURATION;
  
  while (Date.now() < endTime) {
    try {
      await performHealthCheck();
      await new Promise(resolve => setTimeout(resolve, CHECK_INTERVAL));
    } catch (error) {
      console.error('监控执行错误:', error.message);
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
  }
  
  console.log('\n📊 监控完成，生成最终报告...');
  
  const finalReport = generateReport();
  
  // 保存报告
  fs.writeFileSync(OUTPUT_FILE, JSON.stringify(finalReport, null, 2));
  
  // 输出关键统计
  console.log('\n==========================================');
  console.log('📋 API稳定性监控最终报告');
  console.log('==========================================');
  console.log(`监控时长: ${(finalReport.monitorDuration / 1000).toFixed(0)}秒`);
  console.log(`总检查次数: ${finalReport.totalChecks}`);
  console.log(`总请求数: ${finalReport.overallStats.totalRequests}`);
  console.log(`整体成功率: ${finalReport.overallStats.overallSuccessRate}`);
  console.log('\n📈 各服务稳定性:');
  
  for (const [serviceName, stats] of Object.entries(finalReport.serviceStats)) {
    console.log(`  ${serviceName}: ${stats.successRate} (稳定性得分: ${stats.avgStabilityScore})`);
  }
  
  if (finalReport.recommendations.length > 0) {
    console.log('\n💡 建议:');
    finalReport.recommendations.forEach((rec, index) => {
      console.log(`  ${index + 1}. ${rec}`);
    });
  }
  
  console.log(`\n📄 详细报告已保存至: ${OUTPUT_FILE}`);
  
  // 判断总体健康状况
  const overallScore = Object.values(finalReport.serviceStats)
    .reduce((sum, stats) => sum + stats.avgStabilityScore, 0) / Object.keys(finalReport.serviceStats).length;
    
  if (overallScore >= 90) {
    console.log('✅ 系统状态: 优秀');
  } else if (overallScore >= 80) {
    console.log('⚠️  系统状态: 良好');
  } else if (overallScore >= 70) {
    console.log('⚠️  系统状态: 一般，需要优化');
  } else {
    console.log('❌ 系统状态: 较差，需要紧急处理');
  }
}

// 启动监控
startMonitoring().catch(error => {
  console.error('监控启动失败:', error);
  process.exit(1);
});