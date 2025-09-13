/**
 * 高级使用示例
 * 演示SDK的高级功能，包括系统控制、报告生成、监控管理等
 */

import { ArbitrageSystemSDK, UserRole } from '../src';

async function advancedUsageExample() {
  console.log('=== 5.1套利系统前端SDK - 高级使用示例 ===\n');

  const sdk = new ArbitrageSystemSDK({
    baseUrl: 'http://localhost:8080',
    wsUrl: 'ws://localhost:8080/ws',
    enableLogging: true,
    timeout: 60000, // 高级操作可能需要更长时间
  });

  try {
    // 初始化并登录
    await sdk.initialize();
    const user = await sdk.login({
      username: 'admin',
      password: 'admin123',
    });
    console.log(`👤 以管理员身份登录: ${user.username}\n`);

    // 1. 系统控制演示
    console.log('1. 系统控制演示...');
    
    // 获取当前系统状态
    let systemStatus = await sdk.system.getSystemStatus();
    console.log('当前系统状态:', systemStatus.status);
    
    if (systemStatus.status === 'running') {
      console.log('停止系统...');
      await sdk.system.stopSystem();
      console.log('✅ 系统已停止');
      
      // 等待一下再重启
      await ArbitrageSystemSDK.wait(2000);
      
      console.log('重新启动系统...');
      await sdk.system.startSystem();
      console.log('✅ 系统已重启');
    }
    console.log();

    // 2. 高级监控管理
    console.log('2. 高级监控管理...');
    
    // 创建自定义警报规则
    console.log('创建自定义警报规则...');
    const alertRule = await sdk.monitoring.createAlertRule({
      name: 'API响应时间过长',
      description: 'API平均响应时间超过500ms时触发警报',
      metric: 'response_time_ms',
      operator: 'gt',
      threshold: 500,
      severity: 'warning',
      cooldown_minutes: 10,
    });
    console.log('✅ 创建警报规则:', alertRule.name);

    // 获取系统指标统计
    console.log('获取系统指标统计...');
    const cpuStats = await sdk.monitoring.getMetricsStats('cpu_usage_percent', {
      startTime: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
      endTime: new Date().toISOString(),
    });
    console.log('✅ CPU使用率统计:');
    console.log(`- 平均值: ${cpuStats.average.toFixed(2)}%`);
    console.log(`- 最大值: ${cpuStats.maximum.toFixed(2)}%`);
    console.log(`- P95: ${cpuStats.p95.toFixed(2)}%`);
    console.log(`- 趋势: ${cpuStats.trend}`);

    // 设置通知配置
    console.log('配置监控通知...');
    await sdk.monitoring.setNotificationConfig({
      email: {
        enabled: true,
        recipients: ['admin@example.com', 'alerts@example.com'],
        severityThreshold: 'warning',
      },
      webhook: {
        enabled: true,
        url: 'https://hooks.slack.com/services/xxx/yyy/zzz',
        severityThreshold: 'error',
      },
    });
    console.log('✅ 通知配置已更新');
    console.log();

    // 3. 用户和权限管理
    console.log('3. 用户和权限管理...');
    
    // 创建新用户
    console.log('创建新用户...');
    const newUser = await sdk.auth.createUser({
      username: 'trader_demo',
      email: 'trader@example.com',
      password: 'password123',
      role: UserRole.Trader,
    });
    console.log('✅ 创建用户:', newUser.username, `(${newUser.role})`);

    // 获取用户列表
    const users = await sdk.auth.getUsers({ page: 1, limit: 10 });
    console.log(`✅ 系统共有 ${users.total} 个用户`);

    // 更新用户信息
    console.log('更新用户权限...');
    const updatedUser = await sdk.auth.updateUser(newUser.id, {
      role: UserRole.Analyst,
    });
    console.log('✅ 用户权限已更新:', updatedUser.role);

    // 获取角色权限映射
    const rolePermissions = await sdk.auth.getRolePermissions();
    console.log('✅ 系统角色权限:');
    Object.entries(rolePermissions).forEach(([role, permissions]) => {
      console.log(`- ${role}: ${permissions.length} 个权限`);
    });
    console.log();

    // 4. 高级仪表板功能
    console.log('4. 高级仪表板功能...');
    
    // 获取交易对分析
    console.log('获取BTC/USDT分析...');
    const symbolAnalysis = await sdk.dashboard.getSymbolAnalysis('BTC/USDT', {
      startTime: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
      endTime: new Date().toISOString(),
    });
    console.log('✅ BTC/USDT 7天分析:');
    console.log(`- 总利润: $${symbolAnalysis.totalProfit.toFixed(2)}`);
    console.log(`- 交易次数: ${symbolAnalysis.tradeCount}`);
    console.log(`- 胜率: ${(symbolAnalysis.winRate * 100).toFixed(1)}%`);
    console.log(`- 平均利润: $${symbolAnalysis.averageProfit.toFixed(2)}`);

    // 获取风险分析
    console.log('获取风险分析...');
    const riskAnalysis = await sdk.dashboard.getRiskAnalysis();
    console.log('✅ 系统风险分析:');
    console.log(`- 总体风险评分: ${riskAnalysis.totalRiskScore.toFixed(1)}`);
    console.log(`- 风险因素数量: ${riskAnalysis.riskFactors.length}`);
    console.log(`- VaR (95%): ${riskAnalysis.portfolioRisk.var95.toFixed(2)}%`);
    console.log(`- 最大回撤: ${riskAnalysis.portfolioRisk.maxDrawdown.toFixed(2)}%`);
    console.log(`- 夏普比率: ${riskAnalysis.portfolioRisk.sharpeRatio.toFixed(2)}`);

    // 生成自定义报告
    console.log('生成月度性能报告...');
    const reportRequest = await sdk.dashboard.generateCustomReport({
      timeRange: {
        startTime: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
        endTime: new Date().toISOString(),
      },
      reportType: 'monthly',
      includeSections: ['summary', 'profit', 'risk', 'performance', 'recommendations'],
      filters: {
        minProfit: 10,
      },
      format: 'json',
    });
    console.log('✅ 报告生成请求已提交:', reportRequest.reportId);
    
    // 等待报告生成完成
    console.log('等待报告生成...');
    let reportStatus = await sdk.dashboard.getReportStatus(reportRequest.reportId);
    while (reportStatus.status === 'generating') {
      console.log(`报告生成中... ${reportStatus.progress}%`);
      await ArbitrageSystemSDK.wait(2000);
      reportStatus = await sdk.dashboard.getReportStatus(reportRequest.reportId);
    }
    
    if (reportStatus.status === 'completed') {
      console.log('✅ 报告生成完成:', reportStatus.downloadUrl);
    } else {
      console.log('❌ 报告生成失败:', reportStatus.error);
    }
    console.log();

    // 5. 数据收集器管理
    console.log('5. 数据收集器管理...');
    
    // 获取所有收集器状态
    const collectors = await sdk.qingxi.getCollectorStatus();
    console.log(`✅ 系统共有 ${collectors.length} 个数据收集器`);
    
    collectors.forEach(collector => {
      const status = collector.status === 'running' ? '🟢' : 
                    collector.status === 'error' ? '🔴' : '🟡';
      console.log(`${status} ${collector.name} (${collector.exchange}): ${collector.status}`);
      console.log(`   - 活跃连接: ${collector.active_connections}`);
      console.log(`   - 运行时间: ${(collector.uptime_seconds / 3600).toFixed(1)} 小时`);
    });

    // 配置收集器
    if (collectors.length > 0) {
      const firstCollector = collectors[0];
      console.log(`配置收集器: ${firstCollector.name}...`);
      await sdk.qingxi.configureCollector(firstCollector.id, {
        symbols: ['BTC/USDT', 'ETH/USDT', 'BNB/USDT'],
        updateInterval: 5000,
        enableOrderbook: true,
        orderbookDepth: 20,
      });
      console.log('✅ 收集器配置已更新');
    }

    // 获取数据质量报告
    console.log('获取数据质量报告...');
    const qualityReport = await sdk.qingxi.getDataQualityReport({
      start: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
      end: new Date().toISOString(),
    });
    console.log('✅ 24小时数据质量报告:');
    console.log(`- 数据质量评分: ${qualityReport.dataQualityScore.toFixed(1)}/100`);
    console.log(`- 总数据点: ${qualityReport.totalDataPoints.toLocaleString()}`);
    console.log(`- 缺失数据: ${qualityReport.missingData.toLocaleString()}`);
    console.log(`- 建议: ${qualityReport.recommendations.join(', ')}`);
    console.log();

    // 6. 高级WebSocket功能
    console.log('6. 高级WebSocket功能...');
    await sdk.connectWebSocket();
    
    // 设置多个订阅
    const subscriptions: Array<{ name: string; sub: any }> = [];
    
    // 订阅系统状态变更
    subscriptions.push({
      name: '系统状态',
      sub: sdk.subscribeSystemStatus((status) => {
        console.log('📊 系统状态变更:', status.status);
      })
    });

    // 订阅高价值套利机会
    let highValueCount = 0;
    subscriptions.push({
      name: '高价值套利机会',
      sub: sdk.subscribeArbitrageOpportunities((opportunity) => {
        if (opportunity.profit_percentage > 1.0) { // 超过1%利润
          highValueCount++;
          console.log(`💎 高价值套利机会 #${highValueCount}:`, 
            `${opportunity.symbol} - ${opportunity.profit_percentage.toFixed(2)}% 利润`
          );
        }
      })
    });

    // 订阅关键警报
    subscriptions.push({
      name: '关键警报',
      sub: sdk.subscribeAlerts((alert) => {
        if (['error', 'critical'].includes(alert.type)) {
          console.log(`🚨 ${alert.type.toUpperCase()} 警报:`, alert.title);
          console.log(`   ${alert.message}`);
          
          // 自动确认非关键警报
          if (alert.type === 'error') {
            sdk.monitoring.acknowledgeAlert(alert.id, 'SDK自动确认');
          }
        }
      })
    });

    console.log('✅ WebSocket订阅已设置，等待实时数据 (15秒)...');
    await ArbitrageSystemSDK.wait(15000);

    // 清理所有订阅
    subscriptions.forEach(({ name, sub }) => {
      sub.unsubscribe();
      console.log(`📴 已取消订阅: ${name}`);
    });
    console.log();

    // 7. 系统维护功能
    console.log('7. 系统维护功能...');
    
    // 执行系统维护
    console.log('执行系统维护任务...');
    const maintenance = await sdk.system.performMaintenance({
      clearCache: true,
      cleanupLogs: true,
      optimizeDatabase: false, // 生产环境中谨慎使用
    });
    console.log('✅ 维护任务已启动:', maintenance.taskId);

    // 监控维护进度
    let maintenanceStatus = maintenance;
    while (maintenanceStatus.status === 'running') {
      console.log(`维护进行中... ${maintenanceStatus.progress}%`);
      await ArbitrageSystemSDK.wait(2000);
      maintenanceStatus = await sdk.system.getMaintenanceStatus(maintenance.taskId);
    }

    console.log('✅ 维护任务完成:', maintenanceStatus.status);
    console.log('维护结果:', JSON.stringify(maintenanceStatus.results, null, 2));
    console.log();

    // 8. 数据导出功能
    console.log('8. 数据导出功能...');
    
    // 导出交易流数据
    console.log('导出最近7天的交易流数据...');
    const exportRequest = await sdk.dashboard.exportData({
      dataType: 'flows',
      timeRange: {
        startTime: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
        endTime: new Date().toISOString(),
      },
      format: 'csv',
      filters: {
        status: 'completed',
        minProfit: 5,
      },
    });
    console.log('✅ 数据导出完成:', exportRequest.exportId);
    console.log('下载链接:', exportRequest.downloadUrl);
    console.log('链接过期时间:', exportRequest.expiresAt);
    console.log();

    // 9. 性能分析
    console.log('9. 系统性能分析...');
    
    // 获取性能报告
    const performanceReport = await sdk.monitoring.getPerformanceReport({
      startTime: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString(),
      endTime: new Date().toISOString(),
    });
    
    console.log('✅ 24小时性能报告:');
    console.log(`- 平均响应时间: ${performanceReport.summary.averageResponseTime}ms`);
    console.log(`- 最大响应时间: ${performanceReport.summary.maxResponseTime}ms`);
    console.log(`- 错误率: ${(performanceReport.summary.errorRate * 100).toFixed(2)}%`);
    console.log(`- 吞吐量: ${performanceReport.summary.throughput.toFixed(1)} req/s`);
    console.log(`- 可用性: ${(performanceReport.summary.availability * 100).toFixed(2)}%`);
    
    if (performanceReport.bottlenecks.length > 0) {
      console.log('🔍 发现的性能瓶颈:');
      performanceReport.bottlenecks.forEach((bottleneck, index) => {
        console.log(`${index + 1}. ${bottleneck.component} - ${bottleneck.metric}`);
        console.log(`   影响等级: ${bottleneck.impact}`);
        console.log(`   建议: ${bottleneck.recommendation}`);
      });
    }

    if (performanceReport.recommendations.length > 0) {
      console.log('💡 性能优化建议:');
      performanceReport.recommendations.forEach((rec, index) => {
        console.log(`${index + 1}. ${rec}`);
      });
    }
    console.log();

    // 10. 最终状态检查
    console.log('10. 最终状态检查...');
    const finalStatus = sdk.getStatus();
    const finalHealth = await sdk.healthCheck();
    const finalSystemStatus = await sdk.system.getSystemStatus();
    
    console.log('✅ SDK最终状态:');
    console.log('- SDK状态:', finalStatus);
    console.log('- 健康状态:', finalHealth);
    console.log('- 系统状态:', finalSystemStatus.status);
    console.log('- 系统运行时间:', finalSystemStatus.uptime, '秒');

    // 清理演示用户
    console.log('\n清理演示数据...');
    try {
      await sdk.auth.deleteUser(newUser.id);
      console.log('✅ 演示用户已删除');
    } catch (error) {
      console.log('⚠️ 清理演示用户失败:', error);
    }

    try {
      await sdk.monitoring.deleteAlertRule(alertRule.id);
      console.log('✅ 演示警报规则已删除');
    } catch (error) {
      console.log('⚠️ 清理警报规则失败:', error);
    }

    console.log('\n=== 高级使用示例完成 ===');
    console.log('🎉 所有高级功能演示成功！');

  } catch (error) {
    console.error('❌ 高级示例执行失败:', error);
    const formattedError = ArbitrageSystemSDK.formatError(error);
    console.error('详细错误信息:', formattedError);
  } finally {
    console.log('\n清理资源...');
    sdk.disconnectWebSocket();
    console.log('👋 高级示例结束');
  }
}

// 运行示例
if (require.main === module) {
  advancedUsageExample().catch(console.error);
}

export { advancedUsageExample };