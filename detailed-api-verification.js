#!/usr/bin/env node

/**
 * 5.1套利系统387个API详细验证脚本
 * 基于技术文档逐一验证每个API的实现情况
 */

const axios = require('axios');
const chalk = require('chalk').default || require('chalk');

const API_BASE_URL = 'http://localhost:3000/api';
const DIRECT_BASE_URLS = {
  'logging': 'http://localhost:4001/api',
  'cleaning': 'http://localhost:4002/api', 
  'strategy': 'http://localhost:4003/api',
  'performance': 'http://localhost:4004/api',
  'trading': 'http://localhost:4005/api',
  'ai-model': 'http://localhost:4006/api',
  'config': 'http://localhost:4007/api'
};

// 测试结果统计
let results = {
  'logging-service': { total: 45, tested: 0, passed: 0, failed: 0, details: [] },
  'cleaning-service': { total: 52, tested: 0, passed: 0, failed: 0, details: [] },
  'strategy-service': { total: 38, tested: 0, passed: 0, failed: 0, details: [] },
  'performance-service': { total: 67, tested: 0, passed: 0, failed: 0, details: [] },
  'trading-service': { total: 41, tested: 0, passed: 0, failed: 0, details: [] },
  'ai-model-service': { total: 48, tested: 0, passed: 0, failed: 0, details: [] },
  'config-service': { total: 96, tested: 0, passed: 0, failed: 0, details: [] }
};

// 创建axios实例
const apiClient = axios.create({
  timeout: 5000,
  headers: { 'Content-Type': 'application/json' }
});

// 测试单个API
async function testAPI(service, method, endpoint, description, data = null) {
  const fullEndpoint = endpoint.startsWith('/api/') ? endpoint : `/api/${endpoint}`;
  results[service].tested++;
  
  try {
    const config = {
      method: method.toLowerCase(),
      url: `${API_BASE_URL}${fullEndpoint.replace('/api/', '/')}`,
    };
    
    if (data && ['post', 'put'].includes(method.toLowerCase())) {
      config.data = data;
    }
    
    const response = await apiClient.request(config);
    
    if (response.status >= 200 && response.status < 400) {
      results[service].passed++;
      results[service].details.push({
        method, endpoint, description, status: 'PASS', code: response.status
      });
      console.log(chalk.green(`✅ [${method}] ${endpoint} - ${description}`));
      return true;
    } else {
      results[service].failed++;
      results[service].details.push({
        method, endpoint, description, status: 'FAIL', code: response.status
      });
      console.log(chalk.red(`❌ [${method}] ${endpoint} - ${description} (${response.status})`));
      return false;
    }
  } catch (error) {
    results[service].failed++;
    const statusCode = error.response?.status || 'ERROR';
    results[service].details.push({
      method, endpoint, description, status: 'FAIL', code: statusCode
    });
    console.log(chalk.red(`❌ [${method}] ${endpoint} - ${description} (${statusCode})`));
    return false;
  }
}

// 主测试函数
async function runDetailedVerification() {
  console.log(chalk.cyan('\n🔍 5.1套利系统387个API详细验证开始...\n'));
  
  // ==================== 1. 日志服务 (45个API) ====================
  console.log(chalk.yellow('\n📦 日志服务 (logging-service) - 声称45个API\n'));
  
  // 实时日志流API (15个)
  await testAPI('logging-service', 'GET', '/logs/stream/realtime', '获取实时日志流');
  await testAPI('logging-service', 'GET', '/logs/stream/by-service/strategy-service', '按服务过滤日志');
  await testAPI('logging-service', 'GET', '/logs/stream/by-level/error', '按级别过滤日志');
  await testAPI('logging-service', 'GET', '/logs/stream/by-module/trading', '按模块过滤日志');
  await testAPI('logging-service', 'POST', '/logs/stream/search', '搜索日志内容', {query: 'error'});
  await testAPI('logging-service', 'GET', '/logs/stream/tail', '尾随日志输出');
  await testAPI('logging-service', 'GET', '/logs/stream/follow', '跟踪日志变化');
  await testAPI('logging-service', 'GET', '/logs/stream/buffer', '获取缓冲区日志');
  await testAPI('logging-service', 'GET', '/logs/stream/history', '获取历史日志');
  await testAPI('logging-service', 'POST', '/logs/stream/export', '导出日志数据', {format: 'json'});
  await testAPI('logging-service', 'GET', '/logs/stream/stats', '流处理统计');
  await testAPI('logging-service', 'POST', '/logs/stream/pause', '暂停日志流');
  await testAPI('logging-service', 'POST', '/logs/stream/resume', '恢复日志流');
  
  // 日志配置API (18个)
  await testAPI('logging-service', 'GET', '/logs/config/levels', '获取日志级别配置');
  await testAPI('logging-service', 'PUT', '/logs/config/levels', '设置日志级别配置', {level: 'debug'});
  await testAPI('logging-service', 'GET', '/logs/config/levels/trading-service', '获取服务日志级别');
  await testAPI('logging-service', 'PUT', '/logs/config/levels/trading-service', '设置服务日志级别', {level: 'info'});
  await testAPI('logging-service', 'GET', '/logs/config/filters', '获取日志过滤器');
  await testAPI('logging-service', 'POST', '/logs/config/filters', '添加日志过滤器', {pattern: '*.error'});
  await testAPI('logging-service', 'DELETE', '/logs/config/filters/123', '删除日志过滤器');
  await testAPI('logging-service', 'GET', '/logs/config/retention', '获取保留策略');
  await testAPI('logging-service', 'PUT', '/logs/config/retention', '设置保留策略', {days: 30});
  await testAPI('logging-service', 'GET', '/logs/config/rotation', '获取轮转配置');
  await testAPI('logging-service', 'PUT', '/logs/config/rotation', '设置轮转配置', {size: '100MB'});
  await testAPI('logging-service', 'GET', '/logs/config/storage', '获取存储配置');
  await testAPI('logging-service', 'PUT', '/logs/config/storage', '设置存储配置', {path: '/logs'});
  await testAPI('logging-service', 'GET', '/logs/config/format', '获取日志格式');
  await testAPI('logging-service', 'PUT', '/logs/config/format', '设置日志格式', {format: 'json'});
  await testAPI('logging-service', 'GET', '/logs/config/sampling', '获取采样配置');
  await testAPI('logging-service', 'PUT', '/logs/config/sampling', '设置采样配置', {rate: 0.1});
  await testAPI('logging-service', 'POST', '/logs/config/export', '导出配置');
  
  // 日志分析API (12个)
  await testAPI('logging-service', 'GET', '/logs/analysis/stats', '获取日志统计');
  await testAPI('logging-service', 'GET', '/logs/analysis/trends', '获取日志趋势');
  await testAPI('logging-service', 'POST', '/logs/analysis/anomaly', '异常检测', {threshold: 0.95});
  await testAPI('logging-service', 'POST', '/logs/analysis/patterns', '模式查找', {regex: 'ERROR.*'});
  await testAPI('logging-service', 'GET', '/logs/analysis/errors', '错误分析');
  await testAPI('logging-service', 'GET', '/logs/analysis/performance', '性能分析');
  await testAPI('logging-service', 'GET', '/logs/analysis/frequency', '频率分析');
  await testAPI('logging-service', 'GET', '/logs/analysis/correlations', '关联分析');
  await testAPI('logging-service', 'POST', '/logs/analysis/custom', '自定义分析', {query: 'SELECT * FROM logs'});
  await testAPI('logging-service', 'GET', '/logs/analysis/reports', '分析报告');
  await testAPI('logging-service', 'POST', '/logs/analysis/reports', '创建分析报告', {name: 'daily-report'});
  await testAPI('logging-service', 'GET', '/logs/analysis/insights', '获取洞察');
  
  // ==================== 2. 清洗服务 (52个API) ====================
  console.log(chalk.yellow('\n📦 清洗服务 (cleaning-service) - 声称52个API\n'));
  
  // 清洗规则管理API (20个)
  await testAPI('cleaning-service', 'GET', '/cleaning/rules/list', '列出所有清洗规则');
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/create', '创建新的清洗规则', {name: 'price-validation'});
  await testAPI('cleaning-service', 'GET', '/cleaning/rules/123', '获取特定规则详情');
  await testAPI('cleaning-service', 'PUT', '/cleaning/rules/123', '更新清洗规则', {enabled: true});
  await testAPI('cleaning-service', 'DELETE', '/cleaning/rules/123', '删除清洗规则');
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/123/enable', '启用清洗规则');
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/123/disable', '禁用清洗规则');
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/test', '测试清洗规则', {rule: {}, data: {}});
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/validate', '验证清洗规则', {rule: {}});
  await testAPI('cleaning-service', 'GET', '/cleaning/rules/export', '导出清洗规则');
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/import', '导入清洗规则', {rules: []});
  await testAPI('cleaning-service', 'GET', '/cleaning/rules/templates', '获取规则模板');
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/templates/basic-validation', '从模板创建规则');
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/search', '搜索清洗规则', {query: 'price'});
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/batch/enable', '批量启用规则', {ids: [1,2,3]});
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/batch/disable', '批量禁用规则', {ids: [1,2,3]});
  await testAPI('cleaning-service', 'POST', '/cleaning/rules/batch/delete', '批量删除规则', {ids: [1,2,3]});
  await testAPI('cleaning-service', 'GET', '/cleaning/rules/history/123', '获取规则历史');
  await testAPI('cleaning-service', 'GET', '/cleaning/rules/stats', '获取规则统计');
  await testAPI('cleaning-service', 'GET', '/cleaning/rules/dependencies/123', '获取规则依赖');
  
  // 交易所配置API (16个)
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges', '列出所有交易所');
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges/binance/config', '获取交易所配置');
  await testAPI('cleaning-service', 'PUT', '/cleaning/exchanges/binance/config', '更新交易所配置', {});
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges/binance/status', '获取交易所状态');
  await testAPI('cleaning-service', 'POST', '/cleaning/exchanges/binance/enable', '启用交易所');
  await testAPI('cleaning-service', 'POST', '/cleaning/exchanges/binance/disable', '禁用交易所');
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges/binance/symbols', '获取交易对列表');
  await testAPI('cleaning-service', 'PUT', '/cleaning/exchanges/binance/symbols', '更新交易对配置', {});
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges/binance/rules', '获取交易所规则');
  await testAPI('cleaning-service', 'PUT', '/cleaning/exchanges/binance/rules', '更新交易所规则', {});
  await testAPI('cleaning-service', 'POST', '/cleaning/exchanges/binance/test', '测试交易所连接');
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges/binance/metrics', '获取交易所指标');
  await testAPI('cleaning-service', 'POST', '/cleaning/exchanges/binance/reset', '重置交易所配置');
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges/binance/history', '获取配置历史');
  await testAPI('cleaning-service', 'POST', '/cleaning/exchanges/batch/update', '批量更新配置', {});
  await testAPI('cleaning-service', 'GET', '/cleaning/exchanges/templates', '获取配置模板');
  
  // 数据质量API (16个)
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/score', '获取数据质量分数');
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/metrics', '获取质量指标');
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/issues', '获取质量问题');
  await testAPI('cleaning-service', 'POST', '/cleaning/quality/issues/123/resolve', '解决质量问题');
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/trends', '获取质量趋势');
  await testAPI('cleaning-service', 'POST', '/cleaning/quality/analyze', '分析数据质量', {data: {}});
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/reports', '获取质量报告');
  await testAPI('cleaning-service', 'POST', '/cleaning/quality/reports', '生成质量报告', {period: '24h'});
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/benchmarks', '获取质量基准');
  await testAPI('cleaning-service', 'PUT', '/cleaning/quality/benchmarks', '设置质量基准', {score: 0.95});
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/alerts', '获取质量告警');
  await testAPI('cleaning-service', 'POST', '/cleaning/quality/alerts', '创建质量告警', {threshold: 0.8});
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/validation', '数据验证结果');
  await testAPI('cleaning-service', 'POST', '/cleaning/quality/validation', '执行数据验证', {rules: []});
  await testAPI('cleaning-service', 'GET', '/cleaning/quality/statistics', '获取质量统计');
  await testAPI('cleaning-service', 'POST', '/cleaning/quality/optimize', '优化数据质量');
  
  // ==================== 3. 策略服务 (38个API) ====================
  console.log(chalk.yellow('\n📦 策略服务 (strategy-service) - 声称38个API\n'));
  
  // 策略生命周期管理API (12个)
  await testAPI('strategy-service', 'GET', '/strategies/list', '列出所有策略');
  await testAPI('strategy-service', 'GET', '/strategies/triangular-v4', '获取策略详情');
  await testAPI('strategy-service', 'POST', '/strategies/triangular-v4/start', '启动策略');
  await testAPI('strategy-service', 'POST', '/strategies/triangular-v4/stop', '停止策略');
  await testAPI('strategy-service', 'POST', '/strategies/triangular-v4/restart', '重启策略');
  await testAPI('strategy-service', 'POST', '/strategies/triangular-v4/pause', '暂停策略');
  await testAPI('strategy-service', 'POST', '/strategies/triangular-v4/resume', '恢复策略');
  await testAPI('strategy-service', 'GET', '/strategies/triangular-v4/status', '获取策略状态');
  await testAPI('strategy-service', 'GET', '/strategies/triangular-v4/config', '获取策略配置');
  await testAPI('strategy-service', 'POST', '/strategies/triangular-v4/config', '更新策略配置', {});
  await testAPI('strategy-service', 'GET', '/strategies/triangular-v4/logs', '获取策略日志');
  await testAPI('strategy-service', 'GET', '/strategies/triangular-v4/metrics', '获取策略指标');
  
  // 实时监控API (8个)
  await testAPI('strategy-service', 'GET', '/monitoring/realtime', '获取实时状态');
  await testAPI('strategy-service', 'GET', '/monitoring/health', '获取系统健康状态');
  await testAPI('strategy-service', 'GET', '/monitoring/performance', '获取性能概览');
  await testAPI('strategy-service', 'GET', '/monitoring/alerts', '获取活跃告警');
  await testAPI('strategy-service', 'GET', '/monitoring/metrics/cpu', '获取CPU指标');
  await testAPI('strategy-service', 'GET', '/monitoring/metrics/memory', '获取内存指标');
  await testAPI('strategy-service', 'GET', '/monitoring/metrics/network', '获取网络指标');
  await testAPI('strategy-service', 'GET', '/monitoring/metrics/history', '获取历史指标');
  
  // 调试工具API (9个)
  await testAPI('strategy-service', 'GET', '/debug/sessions', '列出调试会话');
  await testAPI('strategy-service', 'POST', '/debug/sessions', '创建调试会话', {strategy_id: 'test'});
  await testAPI('strategy-service', 'GET', '/debug/sessions/123', '获取调试会话');
  await testAPI('strategy-service', 'DELETE', '/debug/sessions/123', '删除调试会话');
  await testAPI('strategy-service', 'GET', '/debug/breakpoints/triangular-v4', '列出断点');
  await testAPI('strategy-service', 'POST', '/debug/breakpoints/triangular-v4', '添加断点', {line: 42});
  await testAPI('strategy-service', 'DELETE', '/debug/breakpoints/triangular-v4/1', '删除断点');
  await testAPI('strategy-service', 'GET', '/debug/variables/triangular-v4', '获取变量');
  await testAPI('strategy-service', 'GET', '/debug/stack/triangular-v4', '获取调用栈');
  
  // 热重载API (9个)
  await testAPI('strategy-service', 'GET', '/hotreload/status', '获取重载状态');
  await testAPI('strategy-service', 'GET', '/hotreload/triangular-v4/status', '获取策略重载状态');
  await testAPI('strategy-service', 'POST', '/hotreload/triangular-v4/reload', '重载策略');
  await testAPI('strategy-service', 'POST', '/hotreload/triangular-v4/enable', '启用热重载');
  await testAPI('strategy-service', 'POST', '/hotreload/triangular-v4/disable', '禁用热重载');
  await testAPI('strategy-service', 'POST', '/hotreload/triangular-v4/validate', '验证变更', {code: ''});
  await testAPI('strategy-service', 'POST', '/hotreload/triangular-v4/rollback', '回滚变更');
  await testAPI('strategy-service', 'GET', '/hotreload/history', '获取重载历史');
  await testAPI('strategy-service', 'GET', '/hotreload/config', '获取重载配置');
  
  // ==================== 4. 性能服务 (67个API) ====================
  console.log(chalk.yellow('\n📦 性能服务 (performance-service) - 声称67个API\n'));
  
  // CPU优化API (18个)
  await testAPI('performance-service', 'GET', '/performance/cpu/usage', '获取CPU使用率');
  await testAPI('performance-service', 'GET', '/performance/cpu/cores', '获取CPU核心信息');
  await testAPI('performance-service', 'GET', '/performance/cpu/frequency', '获取CPU频率');
  await testAPI('performance-service', 'PUT', '/performance/cpu/frequency', '设置CPU频率', {freq: '3.5GHz'});
  await testAPI('performance-service', 'GET', '/performance/cpu/governor', '获取CPU调度器');
  await testAPI('performance-service', 'PUT', '/performance/cpu/governor', '设置CPU调度器', {governor: 'performance'});
  await testAPI('performance-service', 'GET', '/performance/cpu/affinity/trading-service', '获取进程CPU亲和性');
  await testAPI('performance-service', 'PUT', '/performance/cpu/affinity/trading-service', '设置进程CPU亲和性', {cores: [0,1]});
  await testAPI('performance-service', 'GET', '/performance/cpu/cache', '获取CPU缓存统计');
  await testAPI('performance-service', 'POST', '/performance/cpu/cache/flush', '刷新CPU缓存');
  await testAPI('performance-service', 'GET', '/performance/cpu/temperature', '获取CPU温度');
  await testAPI('performance-service', 'GET', '/performance/cpu/throttling', '获取CPU节流状态');
  await testAPI('performance-service', 'GET', '/performance/cpu/topology', '获取CPU拓扑');
  await testAPI('performance-service', 'GET', '/performance/cpu/processes', '获取进程CPU使用');
  await testAPI('performance-service', 'POST', '/performance/cpu/optimize', '优化CPU性能');
  await testAPI('performance-service', 'POST', '/performance/cpu/benchmark', '运行CPU基准测试');
  await testAPI('performance-service', 'GET', '/performance/cpu/scheduler', '获取调度器信息');
  await testAPI('performance-service', 'PUT', '/performance/cpu/scheduler', '调优调度器', {policy: 'CFS'});
  
  // 内存优化API (16个)
  await testAPI('performance-service', 'GET', '/performance/memory/usage', '获取内存使用情况');
  await testAPI('performance-service', 'GET', '/performance/memory/swap', '获取交换空间使用');
  await testAPI('performance-service', 'PUT', '/performance/memory/swap', '配置交换空间', {size: '8GB'});
  await testAPI('performance-service', 'GET', '/performance/memory/cache', '获取内存缓存');
  await testAPI('performance-service', 'POST', '/performance/memory/cache/clear', '清理内存缓存');
  await testAPI('performance-service', 'GET', '/performance/memory/fragmentation', '获取内存碎片');
  await testAPI('performance-service', 'POST', '/performance/memory/compaction', '内存压缩');
  await testAPI('performance-service', 'GET', '/performance/memory/huge-pages', '获取大页配置');
  await testAPI('performance-service', 'PUT', '/performance/memory/huge-pages', '配置大页', {enabled: true});
  await testAPI('performance-service', 'GET', '/performance/memory/numa', '获取NUMA信息');
  await testAPI('performance-service', 'PUT', '/performance/memory/numa', '优化NUMA', {policy: 'local'});
  await testAPI('performance-service', 'GET', '/performance/memory/pressure', '获取内存压力');
  await testAPI('performance-service', 'GET', '/performance/memory/leaks', '检测内存泄漏');
  await testAPI('performance-service', 'GET', '/performance/memory/gc', '获取GC统计');
  await testAPI('performance-service', 'POST', '/performance/memory/gc', '触发垃圾回收');
  await testAPI('performance-service', 'POST', '/performance/memory/optimize', '优化内存');
  
  // 网络优化API (15个)
  await testAPI('performance-service', 'GET', '/performance/network/interfaces', '获取网络接口');
  await testAPI('performance-service', 'GET', '/performance/network/stats', '获取网络统计');
  await testAPI('performance-service', 'GET', '/performance/network/bandwidth', '获取带宽信息');
  await testAPI('performance-service', 'GET', '/performance/network/latency', '测量网络延迟');
  await testAPI('performance-service', 'GET', '/performance/network/connections', '获取网络连接');
  await testAPI('performance-service', 'GET', '/performance/network/tcp-tuning', '获取TCP调优参数');
  await testAPI('performance-service', 'PUT', '/performance/network/tcp-tuning', '设置TCP调优参数', {});
  await testAPI('performance-service', 'GET', '/performance/network/buffer-sizes', '获取缓冲区大小');
  await testAPI('performance-service', 'PUT', '/performance/network/buffer-sizes', '设置缓冲区大小', {});
  await testAPI('performance-service', 'GET', '/performance/network/congestion', '获取拥塞控制算法');
  await testAPI('performance-service', 'PUT', '/performance/network/congestion', '设置拥塞控制算法', {algo: 'bbr'});
  await testAPI('performance-service', 'GET', '/performance/network/queue', '获取队列规则');
  await testAPI('performance-service', 'PUT', '/performance/network/queue', '设置队列规则', {});
  await testAPI('performance-service', 'POST', '/performance/network/optimize', '优化网络性能');
  await testAPI('performance-service', 'POST', '/performance/network/test', '运行网络测试');
  
  // 磁盘I/O优化API (18个)
  await testAPI('performance-service', 'GET', '/performance/disk/usage', '获取磁盘使用情况');
  await testAPI('performance-service', 'GET', '/performance/disk/io-stats', '获取I/O统计');
  await testAPI('performance-service', 'GET', '/performance/disk/iops', '测量IOPS');
  await testAPI('performance-service', 'GET', '/performance/disk/latency', '测量磁盘延迟');
  await testAPI('performance-service', 'GET', '/performance/disk/scheduler', '获取I/O调度器');
  await testAPI('performance-service', 'PUT', '/performance/disk/scheduler', '设置I/O调度器', {scheduler: 'mq-deadline'});
  await testAPI('performance-service', 'GET', '/performance/disk/queue-depth', '获取队列深度');
  await testAPI('performance-service', 'PUT', '/performance/disk/queue-depth', '设置队列深度', {depth: 32});
  await testAPI('performance-service', 'GET', '/performance/disk/read-ahead', '获取预读设置');
  await testAPI('performance-service', 'PUT', '/performance/disk/read-ahead', '设置预读', {size: '256KB'});
  await testAPI('performance-service', 'GET', '/performance/disk/cache', '获取磁盘缓存');
  await testAPI('performance-service', 'PUT', '/performance/disk/cache', '配置磁盘缓存', {enabled: true});
  await testAPI('performance-service', 'GET', '/performance/disk/mount-options', '获取挂载选项');
  await testAPI('performance-service', 'PUT', '/performance/disk/mount-options', '设置挂载选项', {});
  await testAPI('performance-service', 'POST', '/performance/disk/defrag', '磁盘碎片整理');
  await testAPI('performance-service', 'POST', '/performance/disk/trim', 'SSD TRIM');
  await testAPI('performance-service', 'POST', '/performance/disk/benchmark', '运行磁盘基准测试');
  await testAPI('performance-service', 'POST', '/performance/disk/optimize', '优化磁盘性能');
  
  // ==================== 继续其他服务的测试...
  
  // 生成详细报告
  console.log(chalk.cyan('\n📊 生成详细验证报告...\n'));
  generateDetailedReport();
}

function generateDetailedReport() {
  console.log(chalk.cyan('\n' + '='.repeat(80)));
  console.log(chalk.cyan('📋 5.1套利系统387个API详细验证报告'));
  console.log(chalk.cyan('='.repeat(80) + '\n'));
  
  let totalTested = 0, totalPassed = 0, totalFailed = 0;
  
  Object.entries(results).forEach(([service, data]) => {
    const passRate = data.tested > 0 ? ((data.passed / data.tested) * 100).toFixed(2) : '0.00';
    const implRate = data.total > 0 ? ((data.passed / data.total) * 100).toFixed(2) : '0.00';
    
    console.log(chalk.white(`📦 ${service}`));
    console.log(`   声称API: ${data.total}个`);
    console.log(`   实际测试: ${data.tested}个`);
    console.log(`   通过: ${chalk.green(data.passed)}个`);
    console.log(`   失败: ${chalk.red(data.failed)}个`);
    console.log(`   测试通过率: ${passRate}%`);
    console.log(`   实现完整率: ${implRate}%`);
    console.log('');
    
    totalTested += data.tested;
    totalPassed += data.passed;
    totalFailed += data.failed;
  });
  
  const totalDeclared = Object.values(results).reduce((sum, data) => sum + data.total, 0);
  const overallPassRate = totalTested > 0 ? ((totalPassed / totalTested) * 100).toFixed(2) : '0.00';
  const overallImplRate = totalDeclared > 0 ? ((totalPassed / totalDeclared) * 100).toFixed(2) : '0.00';
  
  console.log(chalk.cyan('📊 总体统计:'));
  console.log(`   声称总API数: ${totalDeclared}个`);
  console.log(`   实际测试数: ${totalTested}个`);
  console.log(`   实际可用数: ${chalk.green(totalPassed)}个`);
  console.log(`   实际失败数: ${chalk.red(totalFailed)}个`);
  console.log(`   测试通过率: ${overallPassRate}%`);
  console.log(`   整体实现率: ${chalk.yellow(overallImplRate + '%')}`);
  
  // 保存详细结果
  require('fs').writeFileSync(
    '/home/ubuntu/5.1xitong/detailed-api-verification-report.json',
    JSON.stringify({
      timestamp: new Date().toISOString(),
      summary: {
        declared: totalDeclared,
        tested: totalTested,
        passed: totalPassed,
        failed: totalFailed,
        testPassRate: overallPassRate,
        implementationRate: overallImplRate
      },
      services: results
    }, null, 2)
  );
  
  console.log(chalk.cyan('\n✅ 详细报告已保存到: detailed-api-verification-report.json'));
}

// 运行详细验证
runDetailedVerification().catch(console.error);