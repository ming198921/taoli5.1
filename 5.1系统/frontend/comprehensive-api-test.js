#!/usr/bin/env node

/**
 * 5.1套利系统前端API对接完整测试
 * 测试所有387个API接口的可用性和响应
 */

import axios from 'axios';
import fs from 'fs';

const API_BASE_URL = 'http://localhost:3000/api';
const OUTPUT_FILE = 'frontend-api-test-report.json';

// 定义所有API端点测试用例
const apiTests = {
  // 策略服务 - 38个API
  strategy: [
    // 策略生命周期管理 (12个)
    { method: 'GET', path: '/strategies/list', description: '获取策略列表' },
    { method: 'GET', path: '/strategies/strategy_common', description: '获取策略详情' },
    { method: 'POST', path: '/strategies/strategy_common/start', description: '启动策略' },
    { method: 'POST', path: '/strategies/strategy_common/stop', description: '停止策略' },
    { method: 'POST', path: '/strategies/strategy_common/restart', description: '重启策略' },
    { method: 'POST', path: '/strategies/strategy_common/pause', description: '暂停策略' },
    { method: 'POST', path: '/strategies/strategy_common/resume', description: '恢复策略' },
    { method: 'GET', path: '/strategies/strategy_common/status', description: '获取策略状态' },
    { method: 'GET', path: '/strategies/strategy_common/config', description: '获取策略配置' },
    { method: 'POST', path: '/strategies/strategy_common/config', description: '更新策略配置', body: { type: 'test' } },
    { method: 'GET', path: '/strategies/strategy_common/logs', description: '获取策略日志' },
    { method: 'GET', path: '/strategies/strategy_common/metrics', description: '获取策略指标' },
    
    // 实时监控 (8个)
    { method: 'GET', path: '/monitoring/realtime', description: '获取实时状态' },
    { method: 'GET', path: '/monitoring/health', description: '获取系统健康状态' },
    { method: 'GET', path: '/monitoring/performance', description: '获取性能概览' },
    { method: 'GET', path: '/monitoring/alerts', description: '获取活跃告警' },
    { method: 'GET', path: '/monitoring/metrics/cpu', description: '获取CPU指标' },
    { method: 'GET', path: '/monitoring/metrics/memory', description: '获取内存指标' },
    { method: 'GET', path: '/monitoring/metrics/network', description: '获取网络指标' },
    { method: 'GET', path: '/monitoring/metrics/history', description: '获取历史指标' },
    
    // 调试工具 (9个)
    { method: 'GET', path: '/debug/sessions', description: '列出调试会话' },
    { method: 'POST', path: '/debug/sessions', description: '创建调试会话', body: { strategy_id: 'test' } },
    { method: 'GET', path: '/debug/sessions/test', description: '获取调试会话' },
    { method: 'DELETE', path: '/debug/sessions/test', description: '删除调试会话' },
    { method: 'GET', path: '/debug/breakpoints/test', description: '列出断点' },
    { method: 'POST', path: '/debug/breakpoints/test', description: '添加断点', body: { line: 10 } },
    { method: 'DELETE', path: '/debug/breakpoints/test/1', description: '删除断点' },
    { method: 'GET', path: '/debug/variables/test', description: '获取变量' },
    { method: 'GET', path: '/debug/stack/test', description: '获取调用栈' },
    
    // 热重载 (9个)
    { method: 'GET', path: '/hotreload/status', description: '获取重载状态' },
    { method: 'GET', path: '/hotreload/strategy_common/status', description: '获取策略重载状态' },
    { method: 'POST', path: '/hotreload/strategy_common/reload', description: '重载策略' },
    { method: 'POST', path: '/hotreload/strategy_common/enable', description: '启用热重载' },
    { method: 'POST', path: '/hotreload/strategy_common/disable', description: '禁用热重载' },
    { method: 'POST', path: '/hotreload/strategy_common/validate', description: '验证变更', body: { code: 'test' } },
    { method: 'POST', path: '/hotreload/strategy_common/rollback', description: '回滚变更' },
    { method: 'GET', path: '/hotreload/history', description: '获取重载历史' },
    { method: 'GET', path: '/hotreload/config', description: '获取重载配置' }
  ],

  // 配置服务 - 96个API
  config: [
    // 基础配置管理 (24个)
    { method: 'GET', path: '/config/list', description: '列出所有配置' },
    { method: 'GET', path: '/config/database.host', description: '获取配置值' },
    { method: 'PUT', path: '/config/database.host', description: '设置配置值', body: { value: 'localhost' } },
    { method: 'DELETE', path: '/config/test.key', description: '删除配置项' },
    { method: 'GET', path: '/config/database.host/metadata', description: '获取配置元数据' },
    { method: 'GET', path: '/config/database.host/history', description: '获取配置历史' },
    { method: 'POST', path: '/config/batch/get', description: '批量获取配置', body: { keys: ['database.host'] } },
    { method: 'POST', path: '/config/batch/set', description: '批量设置配置', body: { configs: { 'test.key': 'test' } } },
    { method: 'POST', path: '/config/batch/delete', description: '批量删除配置', body: { keys: ['test.key'] } },
    { method: 'POST', path: '/config/search', description: '搜索配置', body: { query: 'database' } },
    { method: 'GET', path: '/config/tree', description: '获取配置树' },
    { method: 'GET', path: '/config/tree/database', description: '获取配置子树' },
    { method: 'POST', path: '/config/export', description: '导出配置', body: { format: 'json' } },
    { method: 'POST', path: '/config/import', description: '导入配置', body: { configs: {} } },
    { method: 'POST', path: '/config/validate', description: '验证配置', body: { config: {} } },
    { method: 'GET', path: '/config/schema', description: '获取配置模式' },
    { method: 'POST', path: '/config/schema', description: '更新配置模式', body: { schema: {} } },
    { method: 'GET', path: '/config/defaults', description: '获取默认配置' },
    { method: 'POST', path: '/config/defaults', description: '设置默认配置', body: { defaults: {} } },
    { method: 'POST', path: '/config/diff', description: '配置差异比较', body: { config1: {}, config2: {} } },
    { method: 'POST', path: '/config/merge', description: '合并配置', body: { configs: [] } },
    { method: 'POST', path: '/config/backup', description: '备份配置' },
    { method: 'POST', path: '/config/restore', description: '恢复配置', body: { backup_id: 'test' } },
    { method: 'GET', path: '/config/stats', description: '获取配置统计' },
    
    // 版本控制 (24个)
    { method: 'GET', path: '/config/versions', description: '列出所有版本' },
    { method: 'POST', path: '/config/versions', description: '创建新版本', body: { name: 'test' } },
    { method: 'GET', path: '/config/versions/current', description: '获取当前版本' },
    { method: 'GET', path: '/config/versions/latest', description: '获取最新版本' },
    { method: 'GET', path: '/config/versions/v1.0', description: '获取版本详情' },
    { method: 'DELETE', path: '/config/versions/v1.0', description: '删除版本' },
    { method: 'POST', path: '/config/versions/v1.0/deploy', description: '部署版本' },
    { method: 'POST', path: '/config/versions/v1.0/rollback', description: '回滚版本' },
    { method: 'GET', path: '/config/versions/v1.0/compare/v2.0', description: '比较版本' },
    { method: 'GET', path: '/config/versions/v1.0/changes', description: '获取版本变更' },
    { method: 'POST', path: '/config/versions/v1.0/validate', description: '验证版本' },
    { method: 'GET', path: '/config/versions/v1.0/conflicts', description: '检查冲突' },
    { method: 'POST', path: '/config/versions/branch', description: '创建分支', body: { name: 'test' } },
    { method: 'POST', path: '/config/versions/merge', description: '合并版本', body: { from: 'v1.0', to: 'v2.0' } },
    { method: 'POST', path: '/config/versions/tag', description: '标记版本', body: { version: 'v1.0', tag: 'stable' } },
    { method: 'GET', path: '/config/versions/tags', description: '列出标签' },
    { method: 'GET', path: '/config/versions/tags/stable', description: '获取标签版本' },
    { method: 'POST', path: '/config/versions/v1.0/lock', description: '锁定版本' },
    { method: 'POST', path: '/config/versions/v1.0/unlock', description: '解锁版本' },
    { method: 'POST', path: '/config/versions/v1.0/clone', description: '克隆版本' },
    { method: 'POST', path: '/config/versions/gc', description: '垃圾回收版本' },
    { method: 'GET', path: '/config/versions/audit', description: '获取版本审计' },
    { method: 'GET', path: '/config/versions/permissions', description: '获取版本权限' },
    { method: 'PUT', path: '/config/versions/permissions', description: '设置版本权限', body: {} },
    
    // 热重载 (18个)
    { method: 'GET', path: '/config/hot-reload/status', description: '获取重载状态' },
    { method: 'POST', path: '/config/hot-reload/enable', description: '启用热重载' },
    { method: 'POST', path: '/config/hot-reload/disable', description: '禁用热重载' },
    { method: 'POST', path: '/config/hot-reload/trigger', description: '触发重载' },
    { method: 'POST', path: '/config/hot-reload/validate', description: '验证重载', body: { config: {} } },
    { method: 'POST', path: '/config/hot-reload/preview', description: '预览重载', body: { config: {} } },
    { method: 'POST', path: '/config/hot-reload/rollback', description: '回滚重载' },
    { method: 'GET', path: '/config/hot-reload/history', description: '获取重载历史' },
    { method: 'GET', path: '/config/hot-reload/services', description: '列出重载服务' },
    { method: 'GET', path: '/config/hot-reload/services/config', description: '获取服务重载状态' },
    { method: 'POST', path: '/config/hot-reload/services/config/trigger', description: '触发服务重载' },
    { method: 'POST', path: '/config/hot-reload/batch', description: '批量重载', body: { services: ['config'] } },
    { method: 'POST', path: '/config/hot-reload/schedule', description: '计划重载', body: { time: '2025-01-01T00:00:00Z' } },
    { method: 'GET', path: '/config/hot-reload/schedule/1', description: '获取计划重载' },
    { method: 'DELETE', path: '/config/hot-reload/schedule/1', description: '取消计划重载' },
    { method: 'GET', path: '/config/hot-reload/hooks', description: '列出重载钩子' },
    { method: 'POST', path: '/config/hot-reload/hooks', description: '添加重载钩子', body: { hook: {} } },
    { method: 'DELETE', path: '/config/hot-reload/hooks/1', description: '删除重载钩子' },
    
    // 环境管理 (30个)
    { method: 'GET', path: '/config/environments', description: '列出所有环境' },
    { method: 'POST', path: '/config/environments', description: '创建新环境', body: { name: 'test' } },
    { method: 'GET', path: '/config/environments/production', description: '获取环境详情' },
    { method: 'PUT', path: '/config/environments/production', description: '更新环境', body: {} },
    { method: 'DELETE', path: '/config/environments/test', description: '删除环境' },
    { method: 'GET', path: '/config/environments/production/config', description: '获取环境配置' },
    { method: 'PUT', path: '/config/environments/production/config', description: '设置环境配置', body: {} },
    { method: 'POST', path: '/config/environments/production/activate', description: '激活环境' },
    { method: 'POST', path: '/config/environments/production/deactivate', description: '停用环境' },
    { method: 'GET', path: '/config/environments/production/status', description: '获取环境状态' },
    { method: 'POST', path: '/config/environments/production/clone', description: '克隆环境', body: { name: 'test' } },
    { method: 'POST', path: '/config/environments/production/sync', description: '同步环境', body: { target: 'test' } },
    { method: 'GET', path: '/config/environments/production/diff/test', description: '比较环境' },
    { method: 'POST', path: '/config/environments/production/promote', description: '提升环境', body: { target: 'test' } },
    { method: 'GET', path: '/config/environments/production/variables', description: '获取环境变量' },
    { method: 'PUT', path: '/config/environments/production/variables', description: '设置环境变量', body: {} },
    { method: 'GET', path: '/config/environments/production/secrets', description: '获取环境密钥' },
    { method: 'PUT', path: '/config/environments/production/secrets', description: '设置环境密钥', body: {} },
    { method: 'GET', path: '/config/environments/production/permissions', description: '获取环境权限' },
    { method: 'PUT', path: '/config/environments/production/permissions', description: '设置环境权限', body: {} },
    { method: 'GET', path: '/config/environments/production/audit', description: '获取环境审计' },
    { method: 'POST', path: '/config/environments/production/backup', description: '备份环境' },
    { method: 'POST', path: '/config/environments/production/restore', description: '恢复环境', body: { backup_id: 'test' } },
    { method: 'GET', path: '/config/environments/production/health', description: '环境健康检查' },
    { method: 'POST', path: '/config/environments/production/validate', description: '验证环境' },
    { method: 'GET', path: '/config/environments/production/metrics', description: '获取环境指标' },
    { method: 'POST', path: '/config/environments/production/reset', description: '重置环境' },
    { method: 'GET', path: '/config/environments/production/templates', description: '获取环境模板' },
    { method: 'POST', path: '/config/environments/production/apply-template', description: '应用环境模板', body: { template: 'default' } },
    { method: 'GET', path: '/config/environments/current', description: '获取当前环境' }
  ],

  // 交易服务 - 41个API
  trading: [
    // 订单管理 (15个)
    { method: 'GET', path: '/orders/active', description: '获取活跃订单' },
    { method: 'GET', path: '/orders/history', description: '获取历史订单' },
    { method: 'GET', path: '/orders/1', description: '获取订单详情' },
    { method: 'POST', path: '/orders', description: '创建订单', body: { symbol: 'BTC/USDT', type: 'limit', side: 'buy', amount: 0.001, price: 50000 } },
    { method: 'POST', path: '/orders/1/cancel', description: '取消订单' },
    { method: 'POST', path: '/orders/1/modify', description: '修改订单', body: { price: 51000 } },
    { method: 'POST', path: '/orders/batch', description: '批量下单', body: { orders: [] } },
    { method: 'POST', path: '/orders/batch/cancel', description: '批量取消', body: { order_ids: [] } },
    { method: 'GET', path: '/orders/status/1', description: '获取订单状态' },
    { method: 'GET', path: '/orders/fills/1', description: '获取订单成交' },
    { method: 'POST', path: '/orders/conditional', description: '条件订单', body: { condition: 'price > 50000' } },
    { method: 'GET', path: '/orders/conditional', description: '获取条件订单' },
    { method: 'POST', path: '/orders/stop-loss', description: '止损订单', body: { stop_price: 45000 } },
    { method: 'POST', path: '/orders/take-profit', description: '止盈订单', body: { take_price: 55000 } },
    { method: 'GET', path: '/orders/statistics', description: '订单统计' },
    
    // 仓位管理 (10个)
    { method: 'GET', path: '/positions/list', description: '获取仓位列表' },
    { method: 'GET', path: '/positions/current', description: '获取当前仓位' },
    { method: 'GET', path: '/positions/BTC-USDT', description: '获取指定仓位' },
    { method: 'GET', path: '/positions/BTC-USDT/pnl', description: '获取仓位盈亏' },
    { method: 'POST', path: '/positions/BTC-USDT/close', description: '平仓', body: { amount: 0.001 } },
    { method: 'POST', path: '/positions/BTC-USDT/hedge', description: '对冲', body: { amount: 0.001 } },
    { method: 'GET', path: '/positions/summary', description: '仓位汇总' },
    { method: 'GET', path: '/positions/history', description: '历史仓位' },
    { method: 'GET', path: '/positions/limits', description: '仓位限制' },
    { method: 'POST', path: '/positions/limits', description: '设置限制', body: { symbol: 'BTC/USDT', max_size: 1.0 } },
    
    // 账户管理 (8个)
    { method: 'GET', path: '/account/balance', description: '获取账户余额' },
    { method: 'GET', path: '/account/info', description: '获取账户信息' },
    { method: 'GET', path: '/account/assets', description: '获取资产列表' },
    { method: 'GET', path: '/account/assets/BTC', description: '获取指定资产' },
    { method: 'POST', path: '/account/transfer', description: '账户转账', body: { from: 'spot', to: 'futures', asset: 'USDT', amount: 100 } },
    { method: 'GET', path: '/account/transfer/history', description: '转账历史' },
    { method: 'GET', path: '/account/fees', description: '手续费信息' },
    { method: 'POST', path: '/account/settings', description: '账户设置', body: {} },
    
    // 风险管理 (8个)
    { method: 'GET', path: '/risk/profile', description: '风险概况' },
    { method: 'GET', path: '/risk/limits', description: '风险限制' },
    { method: 'POST', path: '/risk/limits', description: '设置风险限制', body: { max_loss: 1000 } },
    { method: 'GET', path: '/risk/exposure', description: '风险敞口' },
    { method: 'GET', path: '/risk/var', description: 'VaR计算' },
    { method: 'POST', path: '/risk/alert', description: '风险告警', body: { type: 'loss_limit', threshold: 500 } },
    { method: 'GET', path: '/risk/alerts', description: '风险告警列表' },
    { method: 'POST', path: '/risk/emergency-stop', description: '紧急停止' }
  ],

  // 性能服务 - 67个API
  performance: [
    // CPU优化 (18个)
    { method: 'GET', path: '/performance/cpu/usage', description: '获取CPU使用率' },
    { method: 'GET', path: '/performance/cpu/cores', description: '获取CPU核心信息' },
    { method: 'GET', path: '/performance/cpu/load', description: '获取系统负载' },
    { method: 'GET', path: '/performance/cpu/processes', description: '获取进程信息' },
    { method: 'GET', path: '/performance/cpu/threads', description: '获取线程信息' },
    { method: 'POST', path: '/performance/cpu/optimize', description: 'CPU优化', body: { strategy: 'balance' } },
    { method: 'GET', path: '/performance/cpu/temperature', description: '获取CPU温度' },
    { method: 'GET', path: '/performance/cpu/frequency', description: '获取CPU频率' },
    { method: 'POST', path: '/performance/cpu/frequency/set', description: '设置CPU频率', body: { frequency: 2400 } },
    { method: 'GET', path: '/performance/cpu/affinity', description: '获取CPU亲和性' },
    { method: 'POST', path: '/performance/cpu/affinity/set', description: '设置CPU亲和性', body: { process: 'trading', cores: [0, 1] } },
    { method: 'GET', path: '/performance/cpu/cache', description: '获取缓存信息' },
    { method: 'POST', path: '/performance/cpu/cache/clear', description: '清理缓存' },
    { method: 'GET', path: '/performance/cpu/scheduler', description: '获取调度信息' },
    { method: 'POST', path: '/performance/cpu/scheduler/tune', description: '调度优化', body: { policy: 'fifo' } },
    { method: 'GET', path: '/performance/cpu/bottlenecks', description: '检测瓶颈' },
    { method: 'GET', path: '/performance/cpu/predictions', description: '负载预测' },
    { method: 'GET', path: '/performance/cpu/history', description: '历史数据' },
    
    // 内存优化 (16个)
    { method: 'GET', path: '/performance/memory/usage', description: '获取内存使用率' },
    { method: 'GET', path: '/performance/memory/allocation', description: '内存分配' },
    { method: 'GET', path: '/performance/memory/fragmentation', description: '内存碎片' },
    { method: 'POST', path: '/performance/memory/defrag', description: '内存整理' },
    { method: 'GET', path: '/performance/memory/swap', description: '交换分区' },
    { method: 'POST', path: '/performance/memory/swap/manage', description: '交换管理', body: { action: 'enable' } },
    { method: 'GET', path: '/performance/memory/cache', description: '内存缓存' },
    { method: 'POST', path: '/performance/memory/cache/clear', description: '清理缓存' },
    { method: 'GET', path: '/performance/memory/leaks', description: '内存泄漏检测' },
    { method: 'POST', path: '/performance/memory/gc', description: '垃圾回收' },
    { method: 'GET', path: '/performance/memory/pools', description: '内存池' },
    { method: 'POST', path: '/performance/memory/pools/create', description: '创建内存池', body: { size: 1024 } },
    { method: 'GET', path: '/performance/memory/numa', description: 'NUMA信息' },
    { method: 'POST', path: '/performance/memory/numa/optimize', description: 'NUMA优化' },
    { method: 'GET', path: '/performance/memory/compression', description: '内存压缩' },
    { method: 'GET', path: '/performance/memory/history', description: '历史数据' },
    
    // 网络优化 (15个)
    { method: 'GET', path: '/performance/network/interfaces', description: '网络接口' },
    { method: 'GET', path: '/performance/network/bandwidth', description: '带宽使用' },
    { method: 'GET', path: '/performance/network/latency', description: '网络延迟' },
    { method: 'GET', path: '/performance/network/throughput', description: '吞吐量' },
    { method: 'GET', path: '/performance/network/connections', description: '连接统计' },
    { method: 'POST', path: '/performance/network/optimize', description: '网络优化', body: { target: 'latency' } },
    { method: 'GET', path: '/performance/network/buffers', description: '缓冲区状态' },
    { method: 'POST', path: '/performance/network/buffers/tune', description: '缓冲区调优', body: { size: 65536 } },
    { method: 'GET', path: '/performance/network/queues', description: '队列状态' },
    { method: 'POST', path: '/performance/network/queues/optimize', description: '队列优化' },
    { method: 'GET', path: '/performance/network/tcp', description: 'TCP参数' },
    { method: 'POST', path: '/performance/network/tcp/tune', description: 'TCP调优', body: { window_size: 131072 } },
    { method: 'GET', path: '/performance/network/firewall', description: '防火墙状态' },
    { method: 'GET', path: '/performance/network/routes', description: '路由表' },
    { method: 'GET', path: '/performance/network/history', description: '历史数据' },
    
    // 磁盘I/O优化 (18个)
    { method: 'GET', path: '/performance/disk/usage', description: '磁盘使用率' },
    { method: 'GET', path: '/performance/disk/iops', description: 'IOPS统计' },
    { method: 'GET', path: '/performance/disk/throughput', description: '磁盘吞吐量' },
    { method: 'GET', path: '/performance/disk/latency', description: '磁盘延迟' },
    { method: 'GET', path: '/performance/disk/queue', description: 'I/O队列' },
    { method: 'POST', path: '/performance/disk/optimize', description: '磁盘优化', body: { strategy: 'performance' } },
    { method: 'GET', path: '/performance/disk/cache', description: '磁盘缓存' },
    { method: 'POST', path: '/performance/disk/cache/tune', description: '缓存调优', body: { read_ahead: 128 } },
    { method: 'GET', path: '/performance/disk/scheduler', description: 'I/O调度器' },
    { method: 'POST', path: '/performance/disk/scheduler/set', description: '设置调度器', body: { scheduler: 'deadline' } },
    { method: 'GET', path: '/performance/disk/fragmentation', description: '磁盘碎片' },
    { method: 'POST', path: '/performance/disk/defrag', description: '磁盘整理' },
    { method: 'GET', path: '/performance/disk/health', description: '磁盘健康' },
    { method: 'GET', path: '/performance/disk/smart', description: 'SMART信息' },
    { method: 'GET', path: '/performance/disk/raids', description: 'RAID状态' },
    { method: 'POST', path: '/performance/disk/raids/optimize', description: 'RAID优化' },
    { method: 'GET', path: '/performance/disk/filesystem', description: '文件系统' },
    { method: 'GET', path: '/performance/disk/history', description: '历史数据' }
  ],

  // AI模型服务 - 48个API  
  aiModel: [
    // 模型管理 (15个)
    { method: 'GET', path: '/ml/models', description: '获取模型列表' },
    { method: 'GET', path: '/ml/models/risk-v1', description: '获取模型详情' },
    { method: 'POST', path: '/ml/models', description: '创建模型', body: { name: 'test-model', type: 'classification' } },
    { method: 'PUT', path: '/ml/models/risk-v1', description: '更新模型', body: { description: 'updated' } },
    { method: 'DELETE', path: '/ml/models/test-model', description: '删除模型' },
    { method: 'POST', path: '/ml/models/risk-v1/deploy', description: '部署模型' },
    { method: 'POST', path: '/ml/models/risk-v1/undeploy', description: '撤销部署' },
    { method: 'GET', path: '/ml/models/risk-v1/status', description: '获取模型状态' },
    { method: 'GET', path: '/ml/models/risk-v1/metrics', description: '获取模型指标' },
    { method: 'GET', path: '/ml/models/risk-v1/versions', description: '获取模型版本' },
    { method: 'POST', path: '/ml/models/risk-v1/versions', description: '创建模型版本', body: { version: 'v1.1' } },
    { method: 'POST', path: '/ml/models/risk-v1/backup', description: '备份模型' },
    { method: 'POST', path: '/ml/models/risk-v1/restore', description: '恢复模型', body: { backup_id: 'backup-1' } },
    { method: 'GET', path: '/ml/models/templates', description: '获取模型模板' },
    { method: 'POST', path: '/ml/models/from-template', description: '从模板创建', body: { template: 'regression' } },
    
    // 训练管理 (12个)
    { method: 'GET', path: '/ml/training/jobs', description: '获取训练任务' },
    { method: 'POST', path: '/ml/training/jobs', description: '创建训练任务', body: { model: 'risk-v1', dataset: 'training-data' } },
    { method: 'GET', path: '/ml/training/jobs/job-1', description: '获取训练详情' },
    { method: 'POST', path: '/ml/training/jobs/job-1/start', description: '开始训练' },
    { method: 'POST', path: '/ml/training/jobs/job-1/stop', description: '停止训练' },
    { method: 'POST', path: '/ml/training/jobs/job-1/pause', description: '暂停训练' },
    { method: 'POST', path: '/ml/training/jobs/job-1/resume', description: '恢复训练' },
    { method: 'GET', path: '/ml/training/jobs/job-1/logs', description: '获取训练日志' },
    { method: 'GET', path: '/ml/training/jobs/job-1/progress', description: '获取训练进度' },
    { method: 'GET', path: '/ml/training/jobs/job-1/checkpoints', description: '获取检查点' },
    { method: 'POST', path: '/ml/training/jobs/job-1/checkpoints/restore', description: '恢复检查点', body: { checkpoint: 'cp-1' } },
    { method: 'GET', path: '/ml/training/history', description: '训练历史' },
    
    // 推理服务 (10个)
    { method: 'GET', path: '/ml/inference/services', description: '推理服务列表' },
    { method: 'POST', path: '/ml/inference/services', description: '创建推理服务', body: { model: 'risk-v1' } },
    { method: 'GET', path: '/ml/inference/services/service-1', description: '获取推理服务' },
    { method: 'POST', path: '/ml/inference/services/service-1/start', description: '启动推理服务' },
    { method: 'POST', path: '/ml/inference/services/service-1/stop', description: '停止推理服务' },
    { method: 'POST', path: '/ml/inference/predict', description: '执行推理', body: { model: 'risk-v1', data: [] } },
    { method: 'POST', path: '/ml/inference/batch', description: '批量推理', body: { model: 'risk-v1', batch: [] } },
    { method: 'GET', path: '/ml/inference/services/service-1/metrics', description: '推理服务指标' },
    { method: 'GET', path: '/ml/inference/services/service-1/health', description: '推理服务健康' },
    { method: 'GET', path: '/ml/inference/history', description: '推理历史' },
    
    // 特征工程 (11个)
    { method: 'GET', path: '/ml/features/pipelines', description: '特征管道列表' },
    { method: 'POST', path: '/ml/features/pipelines', description: '创建特征管道', body: { name: 'price-features' } },
    { method: 'GET', path: '/ml/features/pipelines/price-features', description: '获取特征管道' },
    { method: 'PUT', path: '/ml/features/pipelines/price-features', description: '更新特征管道', body: {} },
    { method: 'DELETE', path: '/ml/features/pipelines/test-features', description: '删除特征管道' },
    { method: 'POST', path: '/ml/features/pipelines/price-features/execute', description: '执行特征提取', body: { data: [] } },
    { method: 'GET', path: '/ml/features/pipelines/price-features/status', description: '特征管道状态' },
    { method: 'GET', path: '/ml/features/store', description: '特征存储' },
    { method: 'POST', path: '/ml/features/store', description: '存储特征', body: { features: {} } },
    { method: 'GET', path: '/ml/features/transformers', description: '特征转换器' },
    { method: 'POST', path: '/ml/features/validate', description: '特征验证', body: { features: {} } }
  ],

  // 日志服务 - 45个API
  logging: [
    // 实时日志流 (15个)
    { method: 'GET', path: '/logs/stream/realtime', description: '获取实时日志流' },
    { method: 'GET', path: '/logs/stream/by-service/config', description: '按服务过滤日志' },
    { method: 'GET', path: '/logs/stream/by-level/info', description: '按级别过滤日志' },
    { method: 'POST', path: '/logs/stream/search', description: '搜索日志内容', body: { query: 'error' } },
    { method: 'GET', path: '/logs/stream/by-component/trading', description: '按组件过滤' },
    { method: 'GET', path: '/logs/stream/by-user/system', description: '按用户过滤' },
    { method: 'POST', path: '/logs/stream/filter', description: '自定义过滤', body: { filters: {} } },
    { method: 'GET', path: '/logs/stream/tail', description: '日志尾部流' },
    { method: 'POST', path: '/logs/stream/subscribe', description: '订阅日志', body: { topics: ['error'] } },
    { method: 'POST', path: '/logs/stream/unsubscribe', description: '取消订阅', body: { topics: ['error'] } },
    { method: 'GET', path: '/logs/stream/subscribers', description: '订阅者列表' },
    { method: 'POST', path: '/logs/stream/pause', description: '暂停日志流' },
    { method: 'POST', path: '/logs/stream/resume', description: '恢复日志流' },
    { method: 'GET', path: '/logs/stream/stats', description: '流统计信息' },
    { method: 'POST', path: '/logs/stream/export', description: '导出日志流', body: { format: 'json' } },
    
    // 日志配置 (18个)
    { method: 'GET', path: '/logs/config/levels', description: '获取日志级别配置' },
    { method: 'PUT', path: '/logs/config/levels', description: '设置日志级别配置', body: { level: 'info' } },
    { method: 'GET', path: '/logs/config/retention', description: '获取保留策略' },
    { method: 'PUT', path: '/logs/config/retention', description: '设置保留策略', body: { days: 30 } },
    { method: 'GET', path: '/logs/config/rotation', description: '获取轮转配置' },
    { method: 'PUT', path: '/logs/config/rotation', description: '设置轮转配置', body: { size: '100MB' } },
    { method: 'GET', path: '/logs/config/formats', description: '获取格式配置' },
    { method: 'PUT', path: '/logs/config/formats', description: '设置格式配置', body: { format: 'json' } },
    { method: 'GET', path: '/logs/config/destinations', description: '获取目标配置' },
    { method: 'PUT', path: '/logs/config/destinations', description: '设置目标配置', body: { destinations: ['file'] } },
    { method: 'GET', path: '/logs/config/filters', description: '获取过滤器配置' },
    { method: 'PUT', path: '/logs/config/filters', description: '设置过滤器配置', body: { filters: {} } },
    { method: 'GET', path: '/logs/config/sampling', description: '获取采样配置' },
    { method: 'PUT', path: '/logs/config/sampling', description: '设置采样配置', body: { rate: 0.1 } },
    { method: 'GET', path: '/logs/config/buffering', description: '获取缓冲配置' },
    { method: 'PUT', path: '/logs/config/buffering', description: '设置缓冲配置', body: { size: 1000 } },
    { method: 'POST', path: '/logs/config/reload', description: '重载配置' },
    { method: 'GET', path: '/logs/config/validate', description: '验证配置' },
    
    // 日志分析 (12个)
    { method: 'GET', path: '/logs/analysis/stats', description: '获取日志统计' },
    { method: 'POST', path: '/logs/analysis/anomaly', description: '异常检测', body: { threshold: 0.95 } },
    { method: 'POST', path: '/logs/analysis/patterns', description: '模式分析', body: { timeframe: '1h' } },
    { method: 'POST', path: '/logs/analysis/trends', description: '趋势分析', body: { period: '24h' } },
    { method: 'POST', path: '/logs/analysis/correlation', description: '关联分析', body: { events: [] } },
    { method: 'POST', path: '/logs/analysis/clustering', description: '聚类分析', body: { algorithm: 'kmeans' } },
    { method: 'POST', path: '/logs/analysis/timeline', description: '时间线分析', body: { start: '2025-01-01', end: '2025-01-02' } },
    { method: 'POST', path: '/logs/analysis/frequency', description: '频率分析', body: { field: 'level' } },
    { method: 'POST', path: '/logs/analysis/sentiment', description: '情感分析', body: { messages: [] } },
    { method: 'GET', path: '/logs/analysis/reports', description: '分析报告列表' },
    { method: 'GET', path: '/logs/analysis/reports/report-1', description: '获取分析报告' },
    { method: 'POST', path: '/logs/analysis/export', description: '导出分析结果', body: { format: 'csv' } }
  ],

  // 清洗服务 - 52个API
  cleaning: [
    // 清洗规则管理 (20个)
    { method: 'GET', path: '/cleaning/rules/list', description: '列出所有清洗规则' },
    { method: 'POST', path: '/cleaning/rules/create', description: '创建新的清洗规则', body: { name: 'test-rule' } },
    { method: 'GET', path: '/cleaning/rules/rule_001', description: '获取清洗规则详情' },
    { method: 'PUT', path: '/cleaning/rules/rule_001', description: '更新清洗规则', body: { enabled: true } },
    { method: 'DELETE', path: '/cleaning/rules/test-rule', description: '删除清洗规则' },
    { method: 'POST', path: '/cleaning/rules/rule_001/enable', description: '启用清洗规则' },
    { method: 'POST', path: '/cleaning/rules/rule_001/disable', description: '禁用清洗规则' },
    { method: 'POST', path: '/cleaning/rules/rule_001/test', description: '测试清洗规则', body: { data: [] } },
    { method: 'GET', path: '/cleaning/rules/rule_001/stats', description: '获取规则统计' },
    { method: 'GET', path: '/cleaning/rules/rule_001/history', description: '获取规则历史' },
    { method: 'POST', path: '/cleaning/rules/rule_001/clone', description: '克隆清洗规则', body: { name: 'cloned-rule' } },
    { method: 'POST', path: '/cleaning/rules/import', description: '导入清洗规则', body: { rules: [] } },
    { method: 'GET', path: '/cleaning/rules/export', description: '导出清洗规则' },
    { method: 'POST', path: '/cleaning/rules/validate', description: '验证清洗规则', body: { rule: {} } },
    { method: 'GET', path: '/cleaning/rules/templates', description: '获取规则模板' },
    { method: 'POST', path: '/cleaning/rules/from-template', description: '从模板创建', body: { template: 'dedup' } },
    { method: 'GET', path: '/cleaning/rules/categories', description: '获取规则分类' },
    { method: 'POST', path: '/cleaning/rules/batch/enable', description: '批量启用', body: { rules: [] } },
    { method: 'POST', path: '/cleaning/rules/batch/disable', description: '批量禁用', body: { rules: [] } },
    { method: 'GET', path: '/cleaning/rules/conflicts', description: '检查规则冲突' },
    
    // 交易所配置 (16个)
    { method: 'GET', path: '/cleaning/exchanges', description: '列出所有交易所' },
    { method: 'GET', path: '/cleaning/exchanges/binance/config', description: '获取交易所配置' },
    { method: 'PUT', path: '/cleaning/exchanges/binance/config', description: '更新交易所配置', body: {} },
    { method: 'GET', path: '/cleaning/exchanges/binance/symbols', description: '获取交易对列表' },
    { method: 'POST', path: '/cleaning/exchanges/binance/symbols', description: '添加交易对', body: { symbol: 'ETH/USDT' } },
    { method: 'DELETE', path: '/cleaning/exchanges/binance/symbols/ETHUSDT', description: '删除交易对' },
    { method: 'GET', path: '/cleaning/exchanges/binance/status', description: '获取交易所状态' },
    { method: 'POST', path: '/cleaning/exchanges/binance/connect', description: '连接交易所' },
    { method: 'POST', path: '/cleaning/exchanges/binance/disconnect', description: '断开连接' },
    { method: 'GET', path: '/cleaning/exchanges/binance/limits', description: '获取API限制' },
    { method: 'PUT', path: '/cleaning/exchanges/binance/limits', description: '设置API限制', body: { rate: 1000 } },
    { method: 'GET', path: '/cleaning/exchanges/binance/credentials', description: '获取认证信息' },
    { method: 'PUT', path: '/cleaning/exchanges/binance/credentials', description: '更新认证信息', body: { api_key: 'test' } },
    { method: 'GET', path: '/cleaning/exchanges/binance/health', description: '健康检查' },
    { method: 'GET', path: '/cleaning/exchanges/binance/metrics', description: '获取指标' },
    { method: 'POST', path: '/cleaning/exchanges/binance/test', description: '测试连接' },
    
    // 数据质量 (16个)
    { method: 'GET', path: '/cleaning/quality/score', description: '获取数据质量分数' },
    { method: 'GET', path: '/cleaning/quality/metrics', description: '获取质量指标' },
    { method: 'POST', path: '/cleaning/quality/check', description: '质量检查', body: { data: [] } },
    { method: 'GET', path: '/cleaning/quality/issues', description: '获取质量问题' },
    { method: 'POST', path: '/cleaning/quality/fix', description: '修复质量问题', body: { issue_id: 'issue-1' } },
    { method: 'GET', path: '/cleaning/quality/rules', description: '获取质量规则' },
    { method: 'POST', path: '/cleaning/quality/rules', description: '创建质量规则', body: { name: 'completeness' } },
    { method: 'PUT', path: '/cleaning/quality/rules/rule-1', description: '更新质量规则', body: {} },
    { method: 'DELETE', path: '/cleaning/quality/rules/rule-1', description: '删除质量规则' },
    { method: 'GET', path: '/cleaning/quality/history', description: '质量历史' },
    { method: 'GET', path: '/cleaning/quality/trends', description: '质量趋势' },
    { method: 'POST', path: '/cleaning/quality/baseline', description: '设置基线', body: { threshold: 0.95 } },
    { method: 'GET', path: '/cleaning/quality/baseline', description: '获取基线' },
    { method: 'POST', path: '/cleaning/quality/alert', description: '设置质量告警', body: { threshold: 0.8 } },
    { method: 'GET', path: '/cleaning/quality/alerts', description: '获取质量告警' },
    { method: 'POST', path: '/cleaning/quality/report', description: '生成质量报告', body: { period: '1d' } }
  ]
};

// 测试统计
let totalTests = 0;
let passedTests = 0;
let failedTests = 0;
const results = {};

// 计算总测试数
Object.values(apiTests).forEach(serviceTests => {
  totalTests += serviceTests.length;
});

console.log(`🚀 开始前端API对接测试 - 总计 ${totalTests} 个API接口`);
console.log('==================================================\n');

// 执行API测试
async function runApiTest(serviceName, test) {
  const url = `${API_BASE_URL}${test.path}`;
  const config = {
    method: test.method,
    url: url,
    timeout: 10000,
    headers: {
      'Content-Type': 'application/json'
    }
  };

  if (test.body && (test.method === 'POST' || test.method === 'PUT')) {
    config.data = test.body;
  }

  try {
    const response = await axios(config);
    passedTests++;
    return {
      success: true,
      status: response.status,
      data: response.data ? (typeof response.data === 'string' ? response.data.substring(0, 200) : JSON.stringify(response.data).substring(0, 200)) : null,
      responseTime: response.headers['x-response-time'] || 'N/A'
    };
  } catch (error) {
    failedTests++;
    return {
      success: false,
      status: error.response?.status || 0,
      error: error.response?.data?.message || error.message,
      responseTime: 'N/A'
    };
  }
}

// 主测试函数
async function runAllTests() {
  for (const [serviceName, tests] of Object.entries(apiTests)) {
    console.log(`\n📊 测试 ${serviceName.toUpperCase()} 服务 (${tests.length}个API):`);
    console.log('-'.repeat(60));
    
    results[serviceName] = {
      total: tests.length,
      passed: 0,
      failed: 0,
      tests: {}
    };

    for (const test of tests) {
      process.stdout.write(`  ${test.method.padEnd(6)} ${test.path.padEnd(50)}`);
      
      const result = await runApiTest(serviceName, test);
      
      results[serviceName].tests[`${test.method} ${test.path}`] = {
        description: test.description,
        ...result
      };

      if (result.success) {
        results[serviceName].passed++;
        console.log(`✅ ${result.status} (${result.responseTime})`);
      } else {
        results[serviceName].failed++;
        console.log(`❌ ${result.status} - ${result.error}`);
      }
    }
    
    const passRate = ((results[serviceName].passed / results[serviceName].total) * 100).toFixed(1);
    console.log(`\n🔸 ${serviceName} 通过率: ${results[serviceName].passed}/${results[serviceName].total} (${passRate}%)`);
  }
}

// 生成总结报告
function generateSummaryReport() {
  const summary = {
    timestamp: new Date().toISOString(),
    total_apis: totalTests,
    passed: passedTests,
    failed: failedTests,
    success_rate: ((passedTests / totalTests) * 100).toFixed(2) + '%',
    services: {}
  };

  Object.entries(results).forEach(([serviceName, serviceResult]) => {
    summary.services[serviceName] = {
      total: serviceResult.total,
      passed: serviceResult.passed,
      failed: serviceResult.failed,
      success_rate: ((serviceResult.passed / serviceResult.total) * 100).toFixed(2) + '%'
    };
  });

  return summary;
}

// 运行测试并生成报告
async function main() {
  try {
    await runAllTests();
    
    console.log('\n' + '='.repeat(70));
    console.log('🎯 前端API对接测试完成总结');
    console.log('='.repeat(70));
    
    const summary = generateSummaryReport();
    
    console.log(`\n📊 总体测试结果:`);
    console.log(`   总API数量: ${summary.total_apis}`);
    console.log(`   通过数量: ${summary.passed}`);
    console.log(`   失败数量: ${summary.failed}`);
    console.log(`   成功率: ${summary.success_rate}`);
    
    console.log(`\n📋 各服务测试结果:`);
    Object.entries(summary.services).forEach(([service, stats]) => {
      const statusIcon = parseFloat(stats.success_rate) > 50 ? '🟢' : parseFloat(stats.success_rate) > 0 ? '🟡' : '🔴';
      console.log(`   ${statusIcon} ${service.padEnd(15)}: ${stats.passed}/${stats.total} (${stats.success_rate})`);
    });

    // 保存详细报告
    const fullReport = {
      summary,
      detailed_results: results,
      test_info: {
        test_time: new Date().toISOString(),
        api_base_url: API_BASE_URL,
        frontend_url: 'http://57.183.21.242:3003',
        total_services: Object.keys(apiTests).length
      }
    };

    fs.writeFileSync(OUTPUT_FILE, JSON.stringify(fullReport, null, 2));
    console.log(`\n📄 详细测试报告已保存到: ${OUTPUT_FILE}`);
    
    // 根据成功率给出建议
    const successRate = parseFloat(summary.success_rate);
    console.log(`\n💡 建议:`);
    if (successRate >= 90) {
      console.log('   🎉 API对接状态优秀！前端可以正常使用所有功能。');
    } else if (successRate >= 70) {
      console.log('   ✅ API对接状态良好，部分功能可能需要进一步调试。');
    } else if (successRate >= 50) {
      console.log('   ⚠️  API对接状态一般，建议优先修复核心功能API。');
    } else {
      console.log('   🚨 API对接存在严重问题，需要全面检查后端服务状态。');
    }
    
  } catch (error) {
    console.error('❌ 测试执行失败:', error);
    process.exit(1);
  }
}

// 开始执行测试
main().catch(console.error);