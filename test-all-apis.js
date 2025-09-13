#!/usr/bin/env node

/**
 * 5.1套利系统API完整测试脚本
 * 测试所有387个API接口的可用性
 */

const axios = require('axios');
const chalk = require('chalk').default || require('chalk');

// API网关基础URL
const API_BASE_URL = 'http://localhost:3000/api';

// 测试结果统计
let totalTests = 0;
let passedTests = 0;
let failedTests = 0;
const failedAPIs = [];

// 创建axios实例
const apiClient = axios.create({
  baseURL: API_BASE_URL,
  timeout: 5000,
  headers: {
    'Content-Type': 'application/json',
  }
});

// 测试单个API
async function testAPI(method, endpoint, description, data = null) {
  totalTests++;
  try {
    const config = {
      method,
      url: endpoint,
    };
    
    if (data) {
      config.data = data;
    }
    
    const response = await apiClient.request(config);
    
    if (response.status >= 200 && response.status < 300) {
      passedTests++;
      console.log(chalk.green(`✅ [${method}] ${endpoint} - ${description}`));
      return true;
    } else {
      failedTests++;
      failedAPIs.push({ method, endpoint, description, status: response.status });
      console.log(chalk.red(`❌ [${method}] ${endpoint} - ${description} (Status: ${response.status})`));
      return false;
    }
  } catch (error) {
    failedTests++;
    const status = error.response?.status || 'Network Error';
    failedAPIs.push({ method, endpoint, description, status });
    console.log(chalk.red(`❌ [${method}] ${endpoint} - ${description} (Error: ${status})`));
    return false;
  }
}

// 主测试函数
async function runAllTests() {
  console.log(chalk.cyan('\n🚀 开始测试5.1套利系统所有API接口...\n'));
  
  // ==================== 策略服务API测试 (38个) ====================
  console.log(chalk.yellow('\n📦 策略服务API测试 (38个接口)\n'));
  
  // 策略生命周期管理 (12个)
  await testAPI('GET', '/strategies/list', '获取策略列表');
  await testAPI('GET', '/strategies/strategy_001', '获取策略详情');
  await testAPI('POST', '/strategies/strategy_001/start', '启动策略');
  await testAPI('POST', '/strategies/strategy_001/stop', '停止策略');
  await testAPI('POST', '/strategies/strategy_001/restart', '重启策略');
  await testAPI('POST', '/strategies/strategy_001/pause', '暂停策略');
  await testAPI('POST', '/strategies/strategy_001/resume', '恢复策略');
  await testAPI('GET', '/strategies/strategy_001/status', '获取策略状态');
  await testAPI('GET', '/strategies/strategy_001/config', '获取策略配置');
  await testAPI('POST', '/strategies/strategy_001/config', '更新策略配置', { max_position: 1000 });
  await testAPI('GET', '/strategies/strategy_001/logs', '获取策略日志');
  await testAPI('GET', '/strategies/strategy_001/metrics', '获取策略指标');
  
  // 实时监控 (8个)
  await testAPI('GET', '/monitoring/realtime', '获取实时状态');
  await testAPI('GET', '/monitoring/health', '获取系统健康状态');
  await testAPI('GET', '/monitoring/performance', '获取性能概览');
  await testAPI('GET', '/monitoring/alerts', '获取活跃告警');
  await testAPI('GET', '/monitoring/metrics/cpu', '获取CPU指标');
  await testAPI('GET', '/monitoring/metrics/memory', '获取内存指标');
  await testAPI('GET', '/monitoring/metrics/network', '获取网络指标');
  await testAPI('GET', '/monitoring/metrics/history', '获取历史指标');
  
  // 调试工具 (9个)
  await testAPI('GET', '/debug/sessions', '列出调试会话');
  await testAPI('POST', '/debug/sessions', '创建调试会话', { strategy_id: 'strategy_001' });
  await testAPI('GET', '/debug/sessions/debug_001', '获取调试会话');
  await testAPI('DELETE', '/debug/sessions/debug_001', '删除调试会话');
  await testAPI('GET', '/debug/breakpoints/strategy_001', '列出断点');
  await testAPI('POST', '/debug/breakpoints/strategy_001', '添加断点', { line: 100 });
  await testAPI('DELETE', '/debug/breakpoints/strategy_001/bp_001', '删除断点');
  await testAPI('GET', '/debug/variables/strategy_001', '获取变量');
  await testAPI('GET', '/debug/stack/strategy_001', '获取调用栈');
  
  // 热重载 (9个)
  await testAPI('GET', '/hotreload/status', '获取重载状态');
  await testAPI('GET', '/hotreload/strategy_001/status', '获取策略重载状态');
  await testAPI('POST', '/hotreload/strategy_001/reload', '重载策略');
  await testAPI('POST', '/hotreload/strategy_001/enable', '启用热重载');
  await testAPI('POST', '/hotreload/strategy_001/disable', '禁用热重载');
  await testAPI('POST', '/hotreload/strategy_001/validate', '验证变更', { code: 'console.log("test")' });
  await testAPI('POST', '/hotreload/strategy_001/rollback', '回滚变更');
  await testAPI('GET', '/hotreload/history', '获取重载历史');
  await testAPI('GET', '/hotreload/config', '获取重载配置');
  
  // ==================== 配置服务API测试 (96个) ====================
  console.log(chalk.yellow('\n📦 配置服务API测试 (96个接口)\n'));
  
  // 基础配置管理 (24个)
  await testAPI('GET', '/config/list', '获取配置列表');
  await testAPI('GET', '/config/system.name', '获取配置项');
  await testAPI('PUT', '/config/system.name', '设置配置项', { value: 'Arbitrage System 5.1' });
  await testAPI('DELETE', '/config/temp.config', '删除配置项');
  await testAPI('GET', '/config/system.name/metadata', '获取配置元数据');
  await testAPI('GET', '/config/system.name/history', '获取配置历史');
  await testAPI('POST', '/config/batch/get', '批量获取配置', { keys: ['system.name', 'system.version'] });
  await testAPI('POST', '/config/batch/set', '批量设置配置', { configs: { 'test1': 'value1', 'test2': 'value2' } });
  await testAPI('POST', '/config/batch/delete', '批量删除配置', { keys: ['test1', 'test2'] });
  await testAPI('POST', '/config/search', '搜索配置', { query: 'system' });
  await testAPI('GET', '/config/tree', '获取配置树');
  await testAPI('GET', '/config/tree/system', '获取配置树路径');
  await testAPI('POST', '/config/export', '导出配置', { format: 'json' });
  await testAPI('POST', '/config/import', '导入配置', { configs: {} });
  await testAPI('POST', '/config/validate', '验证配置', { config: { test: 'value' } });
  await testAPI('GET', '/config/schema', '获取配置架构');
  await testAPI('POST', '/config/schema/validate', '验证配置架构', { schema: {}, config: {} });
  await testAPI('GET', '/config/defaults', '获取默认配置');
  await testAPI('POST', '/config/reset', '重置配置');
  await testAPI('GET', '/config/backup', '获取配置备份');
  await testAPI('POST', '/config/backup', '创建配置备份');
  await testAPI('POST', '/config/restore', '恢复配置', { backup_id: 'backup_001' });
  await testAPI('GET', '/config/diff', '获取配置差异');
  await testAPI('POST', '/config/merge', '合并配置', { source: {}, target: {} });
  
  // 版本控制 (24个)
  await testAPI('GET', '/config/versions', '获取版本列表');
  await testAPI('POST', '/config/versions', '创建新版本', { name: 'v1.0.0' });
  await testAPI('GET', '/config/versions/v1.0.0', '获取版本详情');
  await testAPI('DELETE', '/config/versions/v1.0.0', '删除版本');
  await testAPI('POST', '/config/versions/v1.0.0/deploy', '部署版本');
  await testAPI('POST', '/config/versions/v1.0.0/rollback', '回滚版本');
  await testAPI('GET', '/config/versions/v1.0.0/compare/v2.0.0', '比较版本');
  await testAPI('GET', '/config/versions/current', '获取当前版本');
  await testAPI('GET', '/config/versions/latest', '获取最新版本');
  await testAPI('POST', '/config/versions/v1.0.0/tag', '标记版本', { tag: 'stable' });
  await testAPI('GET', '/config/versions/tags', '获取版本标签');
  await testAPI('GET', '/config/versions/v1.0.0/changelog', '获取版本更新日志');
  await testAPI('POST', '/config/versions/v1.0.0/lock', '锁定版本');
  await testAPI('POST', '/config/versions/v1.0.0/unlock', '解锁版本');
  await testAPI('GET', '/config/versions/v1.0.0/dependencies', '获取版本依赖');
  await testAPI('POST', '/config/versions/v1.0.0/validate', '验证版本');
  await testAPI('GET', '/config/versions/v1.0.0/export', '导出版本');
  await testAPI('POST', '/config/versions/import', '导入版本', { version_data: {} });
  await testAPI('POST', '/config/versions/v1.0.0/clone', '克隆版本', { new_name: 'v1.0.1' });
  await testAPI('GET', '/config/versions/v1.0.0/status', '获取版本状态');
  await testAPI('POST', '/config/versions/v1.0.0/approve', '批准版本');
  await testAPI('POST', '/config/versions/v1.0.0/reject', '拒绝版本');
  await testAPI('GET', '/config/versions/v1.0.0/audit', '获取版本审计日志');
  await testAPI('POST', '/config/versions/v1.0.0/comment', '添加版本注释', { comment: 'Test comment' });
  
  // 热重载 (18个)
  await testAPI('GET', '/config/hot-reload/status', '获取热重载状态');
  await testAPI('POST', '/config/hot-reload/enable', '启用热重载');
  await testAPI('POST', '/config/hot-reload/disable', '禁用热重载');
  await testAPI('POST', '/config/hot-reload/trigger', '触发热重载');
  await testAPI('POST', '/config/hot-reload/validate', '验证热重载', { config: {} });
  await testAPI('POST', '/config/hot-reload/preview', '预览热重载', { config: {} });
  await testAPI('POST', '/config/hot-reload/rollback', '回滚热重载');
  await testAPI('GET', '/config/hot-reload/history', '获取热重载历史');
  await testAPI('GET', '/config/hot-reload/queue', '获取热重载队列');
  await testAPI('POST', '/config/hot-reload/queue/clear', '清空热重载队列');
  await testAPI('GET', '/config/hot-reload/locks', '获取热重载锁');
  await testAPI('POST', '/config/hot-reload/lock', '锁定热重载');
  await testAPI('POST', '/config/hot-reload/unlock', '解锁热重载');
  await testAPI('GET', '/config/hot-reload/metrics', '获取热重载指标');
  await testAPI('POST', '/config/hot-reload/test', '测试热重载', { config: {} });
  await testAPI('GET', '/config/hot-reload/dependencies', '获取热重载依赖');
  await testAPI('POST', '/config/hot-reload/schedule', '计划热重载', { schedule: '*/5 * * * *' });
  await testAPI('GET', '/config/hot-reload/schedules', '获取热重载计划');
  
  // 环境管理 (30个)
  await testAPI('GET', '/config/environments', '获取环境列表');
  await testAPI('POST', '/config/environments', '创建环境', { name: 'development' });
  await testAPI('GET', '/config/environments/development', '获取环境详情');
  await testAPI('PUT', '/config/environments/development', '更新环境', { description: 'Dev environment' });
  await testAPI('DELETE', '/config/environments/development', '删除环境');
  await testAPI('GET', '/config/environments/development/config', '获取环境配置');
  await testAPI('PUT', '/config/environments/development/config', '设置环境配置', { config: {} });
  await testAPI('POST', '/config/environments/development/activate', '激活环境');
  await testAPI('GET', '/config/environments/current', '获取当前环境');
  await testAPI('POST', '/config/environments/development/clone', '克隆环境', { new_name: 'staging' });
  await testAPI('GET', '/config/environments/development/variables', '获取环境变量');
  await testAPI('PUT', '/config/environments/development/variables', '设置环境变量', { variables: {} });
  await testAPI('GET', '/config/environments/development/secrets', '获取环境密钥');
  await testAPI('PUT', '/config/environments/development/secrets', '设置环境密钥', { secrets: {} });
  await testAPI('POST', '/config/environments/development/validate', '验证环境');
  await testAPI('GET', '/config/environments/development/status', '获取环境状态');
  await testAPI('POST', '/config/environments/development/lock', '锁定环境');
  await testAPI('POST', '/config/environments/development/unlock', '解锁环境');
  await testAPI('GET', '/config/environments/development/history', '获取环境历史');
  await testAPI('POST', '/config/environments/development/rollback', '回滚环境', { version: 'v1.0.0' });
  await testAPI('GET', '/config/environments/development/compare/production', '比较环境');
  await testAPI('POST', '/config/environments/sync', '同步环境', { from: 'development', to: 'staging' });
  await testAPI('GET', '/config/environments/development/export', '导出环境');
  await testAPI('POST', '/config/environments/import', '导入环境', { environment_data: {} });
  await testAPI('GET', '/config/environments/development/permissions', '获取环境权限');
  await testAPI('PUT', '/config/environments/development/permissions', '设置环境权限', { permissions: {} });
  await testAPI('GET', '/config/environments/development/audit', '获取环境审计日志');
  await testAPI('POST', '/config/environments/development/approve', '批准环境变更');
  await testAPI('POST', '/config/environments/development/reject', '拒绝环境变更');
  await testAPI('GET', '/config/environments/templates', '获取环境模板');
  
  // ==================== 交易服务API测试 (68个) ====================
  console.log(chalk.yellow('\n📦 交易服务API测试 (68个接口)\n'));
  
  // 订单管理 (32个)
  await testAPI('GET', '/orders/list', '获取订单列表');
  await testAPI('GET', '/orders/active', '获取活跃订单');
  await testAPI('GET', '/orders/history', '获取历史订单');
  await testAPI('GET', '/orders/order_001', '获取订单详情');
  await testAPI('POST', '/orders/create', '创建订单', { symbol: 'BTC/USDT', side: 'buy', amount: 0.001 });
  await testAPI('POST', '/orders/order_001/cancel', '取消订单');
  await testAPI('POST', '/orders/order_001/modify', '修改订单', { price: 50000 });
  await testAPI('GET', '/orders/order_001/status', '获取订单状态');
  await testAPI('GET', '/orders/order_001/fills', '获取订单成交');
  await testAPI('POST', '/orders/batch/create', '批量创建订单', { orders: [] });
  await testAPI('POST', '/orders/batch/cancel', '批量取消订单', { order_ids: [] });
  await testAPI('GET', '/orders/statistics', '获取订单统计');
  await testAPI('GET', '/orders/pending', '获取待处理订单');
  await testAPI('GET', '/orders/filled', '获取已成交订单');
  await testAPI('GET', '/orders/cancelled', '获取已取消订单');
  await testAPI('GET', '/orders/rejected', '获取已拒绝订单');
  await testAPI('POST', '/orders/order_001/retry', '重试订单');
  await testAPI('GET', '/orders/order_001/timeline', '获取订单时间线');
  await testAPI('GET', '/orders/order_001/logs', '获取订单日志');
  await testAPI('POST', '/orders/order_001/archive', '归档订单');
  await testAPI('GET', '/orders/archived', '获取归档订单');
  await testAPI('POST', '/orders/order_001/restore', '恢复订单');
  await testAPI('GET', '/orders/search', '搜索订单');
  await testAPI('GET', '/orders/export', '导出订单');
  await testAPI('POST', '/orders/import', '导入订单', { orders: [] });
  await testAPI('GET', '/orders/order_001/audit', '获取订单审计');
  await testAPI('POST', '/orders/order_001/approve', '批准订单');
  await testAPI('POST', '/orders/order_001/reject', '拒绝订单');
  await testAPI('GET', '/orders/limits', '获取订单限制');
  await testAPI('PUT', '/orders/limits', '设置订单限制', { limits: {} });
  await testAPI('GET', '/orders/fees', '获取订单费用');
  await testAPI('GET', '/orders/order_001/pnl', '获取订单盈亏');
  
  // 仓位管理 (18个)
  await testAPI('GET', '/positions/list', '获取仓位列表');
  await testAPI('GET', '/positions/current', '获取当前仓位');
  await testAPI('GET', '/positions/BTC_USDT', '获取仓位详情');
  await testAPI('POST', '/positions/BTC_USDT/close', '平仓');
  await testAPI('POST', '/positions/BTC_USDT/reduce', '减仓', { amount: 0.001 });
  await testAPI('POST', '/positions/BTC_USDT/increase', '加仓', { amount: 0.001 });
  await testAPI('GET', '/positions/BTC_USDT/pnl', '获取仓位盈亏');
  await testAPI('GET', '/positions/BTC_USDT/risk', '获取仓位风险');
  await testAPI('GET', '/positions/summary', '获取仓位汇总');
  await testAPI('GET', '/positions/exposure', '获取仓位暴露');
  await testAPI('POST', '/positions/hedge', '对冲仓位', { positions: [] });
  await testAPI('GET', '/positions/history', '获取仓位历史');
  await testAPI('GET', '/positions/BTC_USDT/history', '获取单个仓位历史');
  await testAPI('POST', '/positions/BTC_USDT/stop-loss', '设置止损', { price: 45000 });
  await testAPI('POST', '/positions/BTC_USDT/take-profit', '设置止盈', { price: 55000 });
  await testAPI('GET', '/positions/alerts', '获取仓位告警');
  await testAPI('POST', '/positions/alerts', '设置仓位告警', { alerts: [] });
  await testAPI('GET', '/positions/limits', '获取仓位限制');
  
  // 资金管理 (18个)
  await testAPI('GET', '/balance/total', '获取总余额');
  await testAPI('GET', '/balance/available', '获取可用余额');
  await testAPI('GET', '/balance/frozen', '获取冻结余额');
  await testAPI('GET', '/balance/details', '获取余额详情');
  await testAPI('GET', '/balance/history', '获取余额历史');
  await testAPI('POST', '/balance/transfer', '资金划转', { from: 'spot', to: 'futures', amount: 100 });
  await testAPI('GET', '/balance/transfers', '获取划转记录');
  await testAPI('GET', '/balance/deposits', '获取充值记录');
  await testAPI('GET', '/balance/withdrawals', '获取提现记录');
  await testAPI('POST', '/balance/withdraw', '申请提现', { currency: 'USDT', amount: 100, address: '0x123' });
  await testAPI('GET', '/balance/fees', '获取手续费记录');
  await testAPI('GET', '/balance/pnl', '获取盈亏统计');
  await testAPI('GET', '/balance/pnl/daily', '获取每日盈亏');
  await testAPI('GET', '/balance/pnl/monthly', '获取每月盈亏');
  await testAPI('GET', '/balance/roi', '获取投资回报率');
  await testAPI('GET', '/balance/risk', '获取资金风险');
  await testAPI('POST', '/balance/risk/limits', '设置风险限制', { limits: {} });
  await testAPI('GET', '/balance/audit', '获取资金审计');
  
  // ==================== 性能服务API测试 (48个) ====================
  console.log(chalk.yellow('\n📦 性能服务API测试 (48个接口)\n'));
  
  // CPU性能 (12个)
  await testAPI('GET', '/performance/cpu/usage', '获取CPU使用率');
  await testAPI('GET', '/performance/cpu/cores', '获取CPU核心信息');
  await testAPI('GET', '/performance/cpu/frequency', '获取CPU频率');
  await testAPI('GET', '/performance/cpu/temperature', '获取CPU温度');
  await testAPI('GET', '/performance/cpu/load', '获取CPU负载');
  await testAPI('GET', '/performance/cpu/processes', '获取CPU进程');
  await testAPI('GET', '/performance/cpu/history', '获取CPU历史');
  await testAPI('GET', '/performance/cpu/alerts', '获取CPU告警');
  await testAPI('POST', '/performance/cpu/alerts', '设置CPU告警', { threshold: 80 });
  await testAPI('GET', '/performance/cpu/optimization', '获取CPU优化建议');
  await testAPI('POST', '/performance/cpu/optimize', '执行CPU优化');
  await testAPI('GET', '/performance/cpu/profile', '获取CPU性能分析');
  
  // 内存管理 (12个)
  await testAPI('GET', '/performance/memory/usage', '获取内存使用率');
  await testAPI('GET', '/performance/memory/available', '获取可用内存');
  await testAPI('GET', '/performance/memory/cached', '获取缓存内存');
  await testAPI('GET', '/performance/memory/swap', '获取交换内存');
  await testAPI('GET', '/performance/memory/processes', '获取内存进程');
  await testAPI('GET', '/performance/memory/leaks', '检测内存泄漏');
  await testAPI('POST', '/performance/memory/gc', '触发垃圾回收');
  await testAPI('GET', '/performance/memory/history', '获取内存历史');
  await testAPI('GET', '/performance/memory/alerts', '获取内存告警');
  await testAPI('POST', '/performance/memory/alerts', '设置内存告警', { threshold: 90 });
  await testAPI('GET', '/performance/memory/optimization', '获取内存优化建议');
  await testAPI('POST', '/performance/memory/optimize', '执行内存优化');
  
  // 网络监控 (12个)
  await testAPI('GET', '/performance/network/bandwidth', '获取网络带宽');
  await testAPI('GET', '/performance/network/connections', '获取网络连接');
  await testAPI('GET', '/performance/network/latency', '获取网络延迟');
  await testAPI('GET', '/performance/network/packet-loss', '获取丢包率');
  await testAPI('GET', '/performance/network/throughput', '获取吞吐量');
  await testAPI('GET', '/performance/network/protocols', '获取协议统计');
  await testAPI('GET', '/performance/network/interfaces', '获取网络接口');
  await testAPI('GET', '/performance/network/routes', '获取路由信息');
  await testAPI('GET', '/performance/network/dns', '获取DNS信息');
  await testAPI('GET', '/performance/network/history', '获取网络历史');
  await testAPI('GET', '/performance/network/alerts', '获取网络告警');
  await testAPI('POST', '/performance/network/alerts', '设置网络告警', { threshold: 100 });
  
  // 磁盘I/O (12个)
  await testAPI('GET', '/performance/disk/usage', '获取磁盘使用率');
  await testAPI('GET', '/performance/disk/io', '获取磁盘I/O');
  await testAPI('GET', '/performance/disk/read', '获取磁盘读取');
  await testAPI('GET', '/performance/disk/write', '获取磁盘写入');
  await testAPI('GET', '/performance/disk/latency', '获取磁盘延迟');
  await testAPI('GET', '/performance/disk/queue', '获取磁盘队列');
  await testAPI('GET', '/performance/disk/partitions', '获取磁盘分区');
  await testAPI('GET', '/performance/disk/smart', '获取SMART信息');
  await testAPI('GET', '/performance/disk/history', '获取磁盘历史');
  await testAPI('GET', '/performance/disk/alerts', '获取磁盘告警');
  await testAPI('POST', '/performance/disk/alerts', '设置磁盘告警', { threshold: 85 });
  await testAPI('POST', '/performance/disk/cleanup', '执行磁盘清理');
  
  // ==================== AI模型服务API测试 (51个) ====================
  console.log(chalk.yellow('\n📦 AI模型服务API测试 (51个接口)\n'));
  
  // 模型管理 (21个)
  await testAPI('GET', '/ml/models', '获取模型列表');
  await testAPI('GET', '/ml/models/model_001', '获取模型详情');
  await testAPI('POST', '/ml/models', '创建模型', { name: 'Test Model', type: 'neural_network' });
  await testAPI('PUT', '/ml/models/model_001', '更新模型', { description: 'Updated model' });
  await testAPI('DELETE', '/ml/models/model_001', '删除模型');
  await testAPI('POST', '/ml/models/model_001/deploy', '部署模型');
  await testAPI('POST', '/ml/models/model_001/undeploy', '下线模型');
  await testAPI('GET', '/ml/models/model_001/status', '获取模型状态');
  await testAPI('GET', '/ml/models/model_001/version', '获取模型版本');
  await testAPI('POST', '/ml/models/model_001/version', '创建模型版本', { version: '2.0.0' });
  await testAPI('GET', '/ml/models/model_001/metrics', '获取模型指标');
  await testAPI('GET', '/ml/models/model_001/performance', '获取模型性能');
  await testAPI('POST', '/ml/models/model_001/evaluate', '评估模型', { test_data: [] });
  await testAPI('POST', '/ml/models/model_001/validate', '验证模型');
  await testAPI('GET', '/ml/models/model_001/export', '导出模型');
  await testAPI('POST', '/ml/models/import', '导入模型', { model_data: {} });
  await testAPI('POST', '/ml/models/model_001/clone', '克隆模型', { new_name: 'Cloned Model' });
  await testAPI('GET', '/ml/models/model_001/config', '获取模型配置');
  await testAPI('PUT', '/ml/models/model_001/config', '更新模型配置', { config: {} });
  await testAPI('GET', '/ml/models/model_001/logs', '获取模型日志');
  await testAPI('GET', '/ml/models/model_001/audit', '获取模型审计');
  
  // 训练任务 (15个)
  await testAPI('GET', '/ml/training/jobs', '获取训练任务');
  await testAPI('POST', '/ml/training/jobs', '创建训练任务', { model_id: 'model_001', dataset_id: 'dataset_001' });
  await testAPI('GET', '/ml/training/jobs/job_001', '获取训练任务详情');
  await testAPI('POST', '/ml/training/jobs/job_001/start', '启动训练');
  await testAPI('POST', '/ml/training/jobs/job_001/stop', '停止训练');
  await testAPI('POST', '/ml/training/jobs/job_001/pause', '暂停训练');
  await testAPI('POST', '/ml/training/jobs/job_001/resume', '恢复训练');
  await testAPI('GET', '/ml/training/jobs/job_001/status', '获取训练状态');
  await testAPI('GET', '/ml/training/jobs/job_001/progress', '获取训练进度');
  await testAPI('GET', '/ml/training/jobs/job_001/metrics', '获取训练指标');
  await testAPI('GET', '/ml/training/jobs/job_001/logs', '获取训练日志');
  await testAPI('GET', '/ml/training/jobs/job_001/checkpoints', '获取检查点');
  await testAPI('POST', '/ml/training/jobs/job_001/checkpoint', '保存检查点');
  await testAPI('POST', '/ml/training/jobs/job_001/rollback', '回滚到检查点', { checkpoint_id: 'checkpoint_001' });
  await testAPI('DELETE', '/ml/training/jobs/job_001', '删除训练任务');
  
  // 推理服务 (15个)
  await testAPI('POST', '/ml/inference/predict', '执行预测', { model_id: 'model_001', data: {} });
  await testAPI('POST', '/ml/inference/batch', '批量预测', { model_id: 'model_001', data: [] });
  await testAPI('GET', '/ml/inference/sessions', '获取推理会话');
  await testAPI('POST', '/ml/inference/sessions', '创建推理会话', { model_id: 'model_001' });
  await testAPI('GET', '/ml/inference/sessions/session_001', '获取会话详情');
  await testAPI('DELETE', '/ml/inference/sessions/session_001', '结束推理会话');
  await testAPI('GET', '/ml/inference/sessions/session_001/history', '获取会话历史');
  await testAPI('GET', '/ml/inference/latency', '获取推理延迟');
  await testAPI('GET', '/ml/inference/throughput', '获取推理吞吐量');
  await testAPI('GET', '/ml/inference/cache', '获取推理缓存');
  await testAPI('POST', '/ml/inference/cache/clear', '清除推理缓存');
  await testAPI('GET', '/ml/inference/optimization', '获取推理优化建议');
  await testAPI('POST', '/ml/inference/optimize', '执行推理优化');
  await testAPI('GET', '/ml/inference/monitoring', '获取推理监控');
  await testAPI('GET', '/ml/inference/alerts', '获取推理告警');
  
  // ==================== 日志服务API测试 (48个) ====================
  console.log(chalk.yellow('\n📦 日志服务API测试 (48个接口)\n'));
  
  // 日志流 (12个)
  await testAPI('GET', '/logs/stream/realtime', '获取实时日志流');
  await testAPI('GET', '/logs/stream/historical', '获取历史日志流');
  await testAPI('POST', '/logs/stream/subscribe', '订阅日志流', { topics: ['system', 'trading'] });
  await testAPI('POST', '/logs/stream/unsubscribe', '取消订阅日志流', { topics: ['system'] });
  await testAPI('GET', '/logs/stream/topics', '获取日志主题');
  await testAPI('GET', '/logs/stream/filters', '获取日志过滤器');
  await testAPI('POST', '/logs/stream/filters', '设置日志过滤器', { filters: {} });
  await testAPI('GET', '/logs/stream/buffer', '获取日志缓冲区');
  await testAPI('POST', '/logs/stream/buffer/clear', '清空日志缓冲区');
  await testAPI('GET', '/logs/stream/statistics', '获取日志流统计');
  await testAPI('GET', '/logs/stream/health', '获取日志流健康状态');
  await testAPI('POST', '/logs/stream/pause', '暂停日志流');
  
  // 日志聚合 (12个)
  await testAPI('GET', '/logs/aggregate/summary', '获取日志汇总');
  await testAPI('GET', '/logs/aggregate/by-level', '按级别聚合日志');
  await testAPI('GET', '/logs/aggregate/by-source', '按来源聚合日志');
  await testAPI('GET', '/logs/aggregate/by-time', '按时间聚合日志');
  await testAPI('GET', '/logs/aggregate/errors', '获取错误日志汇总');
  await testAPI('GET', '/logs/aggregate/warnings', '获取警告日志汇总');
  await testAPI('GET', '/logs/aggregate/patterns', '获取日志模式');
  await testAPI('GET', '/logs/aggregate/trends', '获取日志趋势');
  await testAPI('GET', '/logs/aggregate/anomalies', '获取日志异常');
  await testAPI('GET', '/logs/aggregate/top-errors', '获取高频错误');
  await testAPI('GET', '/logs/aggregate/performance', '获取性能日志汇总');
  await testAPI('GET', '/logs/aggregate/security', '获取安全日志汇总');
  
  // 日志配置 (12个)
  await testAPI('GET', '/logs/config', '获取日志配置');
  await testAPI('PUT', '/logs/config', '更新日志配置', { config: {} });
  await testAPI('GET', '/logs/config/levels', '获取日志级别');
  await testAPI('PUT', '/logs/config/levels', '设置日志级别', { level: 'INFO' });
  await testAPI('GET', '/logs/config/rotation', '获取日志轮转配置');
  await testAPI('PUT', '/logs/config/rotation', '设置日志轮转', { max_size: '100MB', max_age: '7d' });
  await testAPI('GET', '/logs/config/retention', '获取日志保留策略');
  await testAPI('PUT', '/logs/config/retention', '设置日志保留策略', { days: 30 });
  await testAPI('GET', '/logs/config/format', '获取日志格式');
  await testAPI('PUT', '/logs/config/format', '设置日志格式', { format: 'json' });
  await testAPI('GET', '/logs/config/outputs', '获取日志输出');
  await testAPI('PUT', '/logs/config/outputs', '设置日志输出', { outputs: ['file', 'console'] });
  
  // 日志分析 (12个)
  await testAPI('POST', '/logs/analyze/search', '搜索日志', { query: 'error', timeRange: '1h' });
  await testAPI('POST', '/logs/analyze/filter', '过滤日志', { filters: { level: 'ERROR' } });
  await testAPI('POST', '/logs/analyze/parse', '解析日志', { log_text: '2024-01-01 ERROR: Test' });
  await testAPI('GET', '/logs/analyze/statistics', '获取日志统计');
  await testAPI('GET', '/logs/analyze/correlation', '获取日志关联');
  await testAPI('POST', '/logs/analyze/export', '导出日志', { format: 'csv', timeRange: '1d' });
  await testAPI('POST', '/logs/analyze/archive', '归档日志', { before: '2024-01-01' });
  await testAPI('GET', '/logs/analyze/archived', '获取归档日志');
  await testAPI('POST', '/logs/analyze/restore', '恢复归档日志', { archive_id: 'archive_001' });
  await testAPI('POST', '/logs/analyze/clean', '清理日志', { before: '2024-01-01' });
  await testAPI('GET', '/logs/analyze/size', '获取日志大小');
  await testAPI('GET', '/logs/analyze/growth', '获取日志增长率');
  
  // ==================== 清洗服务API测试 (42个) ====================
  console.log(chalk.yellow('\n📦 清洗服务API测试 (42个接口)\n'));
  
  // 清洗规则 (18个)
  await testAPI('GET', '/cleaning/rules/list', '获取清洗规则列表');
  await testAPI('POST', '/cleaning/rules', '创建清洗规则', { name: 'Test Rule', pattern: '.*' });
  await testAPI('GET', '/cleaning/rules/rule_001', '获取清洗规则详情');
  await testAPI('PUT', '/cleaning/rules/rule_001', '更新清洗规则', { pattern: '.+' });
  await testAPI('DELETE', '/cleaning/rules/rule_001', '删除清洗规则');
  await testAPI('POST', '/cleaning/rules/rule_001/enable', '启用清洗规则');
  await testAPI('POST', '/cleaning/rules/rule_001/disable', '禁用清洗规则');
  await testAPI('POST', '/cleaning/rules/rule_001/test', '测试清洗规则', { sample_data: {} });
  await testAPI('GET', '/cleaning/rules/rule_001/statistics', '获取规则统计');
  await testAPI('GET', '/cleaning/rules/rule_001/history', '获取规则历史');
  await testAPI('POST', '/cleaning/rules/rule_001/validate', '验证清洗规则');
  await testAPI('GET', '/cleaning/rules/templates', '获取规则模板');
  await testAPI('POST', '/cleaning/rules/import', '导入清洗规则', { rules: [] });
  await testAPI('GET', '/cleaning/rules/export', '导出清洗规则');
  await testAPI('POST', '/cleaning/rules/batch/enable', '批量启用规则', { rule_ids: [] });
  await testAPI('POST', '/cleaning/rules/batch/disable', '批量禁用规则', { rule_ids: [] });
  await testAPI('POST', '/cleaning/rules/batch/delete', '批量删除规则', { rule_ids: [] });
  await testAPI('GET', '/cleaning/rules/priorities', '获取规则优先级');
  
  // 交易所管理 (12个)
  await testAPI('GET', '/cleaning/exchanges/list', '获取交易所列表');
  await testAPI('GET', '/cleaning/exchanges/binance', '获取交易所详情');
  await testAPI('PUT', '/cleaning/exchanges/binance/config', '更新交易所配置', { config: {} });
  await testAPI('GET', '/cleaning/exchanges/binance/status', '获取交易所状态');
  await testAPI('POST', '/cleaning/exchanges/binance/enable', '启用交易所');
  await testAPI('POST', '/cleaning/exchanges/binance/disable', '禁用交易所');
  await testAPI('GET', '/cleaning/exchanges/binance/markets', '获取交易所市场');
  await testAPI('GET', '/cleaning/exchanges/binance/symbols', '获取交易所交易对');
  await testAPI('GET', '/cleaning/exchanges/binance/fees', '获取交易所费率');
  await testAPI('GET', '/cleaning/exchanges/binance/limits', '获取交易所限制');
  await testAPI('POST', '/cleaning/exchanges/binance/test', '测试交易所连接');
  await testAPI('GET', '/cleaning/exchanges/binance/statistics', '获取交易所统计');
  
  // 数据质量 (12个)
  await testAPI('GET', '/cleaning/quality/overview', '获取数据质量概览');
  await testAPI('GET', '/cleaning/quality/score', '获取数据质量评分');
  await testAPI('GET', '/cleaning/quality/issues', '获取数据质量问题');
  await testAPI('POST', '/cleaning/quality/validate', '验证数据质量', { data: {} });
  await testAPI('GET', '/cleaning/quality/metrics', '获取质量指标');
  await testAPI('GET', '/cleaning/quality/trends', '获取质量趋势');
  await testAPI('GET', '/cleaning/quality/reports', '获取质量报告');
  await testAPI('POST', '/cleaning/quality/reports', '生成质量报告');
  await testAPI('GET', '/cleaning/quality/alerts', '获取质量告警');
  await testAPI('POST', '/cleaning/quality/alerts', '设置质量告警', { thresholds: {} });
  await testAPI('POST', '/cleaning/quality/improve', '改进数据质量');
  await testAPI('GET', '/cleaning/quality/recommendations', '获取改进建议');
  
  // ==================== 测试结果统计 ====================
  console.log(chalk.cyan('\n📊 测试结果统计\n'));
  console.log(chalk.white(`总测试数: ${totalTests}`));
  console.log(chalk.green(`通过: ${passedTests}`));
  console.log(chalk.red(`失败: ${failedTests}`));
  console.log(chalk.yellow(`通过率: ${((passedTests / totalTests) * 100).toFixed(2)}%`));
  
  if (failedAPIs.length > 0) {
    console.log(chalk.red('\n❌ 失败的API列表:'));
    failedAPIs.forEach(api => {
      console.log(chalk.red(`  - [${api.method}] ${api.endpoint} - ${api.description} (${api.status})`));
    });
  }
  
  // 生成测试报告
  const report = {
    timestamp: new Date().toISOString(),
    summary: {
      total: totalTests,
      passed: passedTests,
      failed: failedTests,
      passRate: ((passedTests / totalTests) * 100).toFixed(2) + '%'
    },
    failedAPIs: failedAPIs
  };
  
  require('fs').writeFileSync(
    '/home/ubuntu/5.1xitong/api-test-report.json',
    JSON.stringify(report, null, 2)
  );
  
  console.log(chalk.cyan('\n✅ 测试报告已保存到: /home/ubuntu/5.1xitong/api-test-report.json'));
}

// 运行测试
runAllTests().catch(console.error);