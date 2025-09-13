/**
 * 基础使用示例
 * 演示SDK的基本功能和用法
 */

import { ArbitrageSystemSDK } from '../src';

async function basicUsageExample() {
  console.log('=== 5.1套利系统前端SDK - 基础使用示例 ===\n');

  // 1. 创建SDK实例
  console.log('1. 创建SDK实例...');
  const sdk = new ArbitrageSystemSDK({
    baseUrl: 'http://localhost:8080',
    wsUrl: 'ws://localhost:8080/ws',
    enableLogging: true,
    timeout: 30000,
    retryAttempts: 3,
  });

  try {
    // 2. 初始化SDK
    console.log('2. 初始化SDK...');
    await sdk.initialize();
    console.log('✅ SDK初始化完成\n');

    // 3. 用户登录
    console.log('3. 用户登录...');
    const user = await sdk.login({
      username: 'admin',
      password: 'admin123',
      remember: true,
    });
    console.log('✅ 登录成功:', user.username, `(${user.role})`);
    console.log('权限:', user.permissions.slice(0, 3).join(', '), '...\n');

    // 4. 系统状态检查
    console.log('4. 检查系统状态...');
    const systemStatus = await sdk.system.getSystemStatus();
    console.log('✅ 系统状态:', systemStatus.status);
    console.log('运行时间:', systemStatus.uptime, '秒');
    console.log('活跃组件:', Object.entries(systemStatus.components)
      .filter(([_, status]) => status.status === 'running')
      .map(([name]) => name)
      .join(', ')
    );
    console.log();

    // 5. 获取市场数据
    console.log('5. 获取市场数据...');
    const marketData = await sdk.qingxi.getMarketData();
    console.log(`✅ 获取到 ${marketData.length} 个交易对的数据`);
    if (marketData.length > 0) {
      const sample = marketData[0];
      console.log('示例数据:', `${sample.symbol}@${sample.exchange} - $${sample.price}`);
    }
    console.log();

    // 6. 获取套利机会
    console.log('6. 获取套利机会...');
    const opportunities = await sdk.qingxi.getArbitrageOpportunities({
      page: 1,
      limit: 5,
      minProfitPercent: 0.1,
    });
    console.log(`✅ 找到 ${opportunities.total} 个套利机会`);
    if (opportunities.data.length > 0) {
      const best = opportunities.data[0];
      console.log('最佳机会:', 
        `${best.symbol} - ${best.profit_percentage.toFixed(2)}% 利润`
      );
    }
    console.log();

    // 7. 获取仪表板统计
    console.log('7. 获取仪表板统计...');
    const dashboardStats = await sdk.dashboard.getDashboardStats();
    console.log('✅ 24小时统计:');
    console.log(`- 总利润: $${dashboardStats.total_profit_24h.toFixed(2)}`);
    console.log(`- 交易次数: ${dashboardStats.total_trades_24h}`);
    console.log(`- 成功率: ${(dashboardStats.success_rate * 100).toFixed(1)}%`);
    console.log();

    // 8. 监控系统健康
    console.log('8. 检查系统健康...');
    const healthChecks = await sdk.monitoring.getHealthChecks();
    const healthyComponents = healthChecks.filter(h => h.status === 'healthy');
    console.log(`✅ ${healthyComponents.length}/${healthChecks.length} 个组件健康`);
    
    healthChecks.forEach(check => {
      const status = check.status === 'healthy' ? '✅' : 
                    check.status === 'degraded' ? '⚠️' : '❌';
      console.log(`${status} ${check.component}: ${check.status}`);
    });
    console.log();

    // 9. WebSocket连接和实时数据
    console.log('9. 连接WebSocket获取实时数据...');
    await sdk.connectWebSocket();
    console.log('✅ WebSocket连接成功');

    // 订阅市场数据更新
    let marketUpdateCount = 0;
    const marketDataSub = sdk.subscribeMarketData((data) => {
      marketUpdateCount++;
      if (marketUpdateCount <= 3) {
        console.log(`📈 市场数据更新 #${marketUpdateCount}:`, 
          `${data.symbol}@${data.exchange} - $${data.price}`
        );
      }
    });

    // 订阅套利机会
    let opportunityCount = 0;
    const opportunitySub = sdk.subscribeArbitrageOpportunities((opportunity) => {
      opportunityCount++;
      console.log(`💰 新套利机会 #${opportunityCount}:`, 
        `${opportunity.symbol} - ${opportunity.profit_percentage.toFixed(2)}%`
      );
    });

    // 等待一些实时数据
    console.log('等待实时数据 (10秒)...');
    await ArbitrageSystemSDK.wait(10000);

    // 清理订阅
    marketDataSub.unsubscribe();
    opportunitySub.unsubscribe();
    console.log('📴 取消WebSocket订阅\n');

    // 10. 权限检查示例
    console.log('10. 检查用户权限...');
    const isAdmin = await sdk.isAdmin();
    const hasTrading = await sdk.hasPermission('trading.execute');
    const hasConfig = await sdk.hasPermission('system.config');
    
    console.log(`✅ 权限检查结果:`);
    console.log(`- 管理员: ${isAdmin ? '是' : '否'}`);
    console.log(`- 交易权限: ${hasTrading ? '有' : '无'}`);
    console.log(`- 配置权限: ${hasConfig ? '有' : '无'}`);
    console.log();

    // 11. 批量操作示例
    console.log('11. 执行批量操作...');
    const batch = sdk.batch();
    
    batch.add(() => sdk.system.getSystemStats());
    batch.add(() => sdk.monitoring.getCurrentMetrics());
    batch.add(() => sdk.dashboard.getRealTimeFlows());

    const batchResults = await batch.executeParallel();
    console.log(`✅ 批量操作完成:`);
    console.log(`- 成功: ${batchResults.successful.length} 个操作`);
    console.log(`- 失败: ${batchResults.failed.length} 个操作`);
    console.log();

    // 12. SDK状态检查
    console.log('12. SDK状态总结...');
    const sdkStatus = sdk.getStatus();
    const healthStatus = await sdk.healthCheck();
    
    console.log('✅ SDK状态:', {
      初始化: sdkStatus.initialized,
      已登录: sdkStatus.loggedIn,
      用户: sdkStatus.user?.username,
      API连接: healthStatus.api,
      WebSocket: healthStatus.websocket,
    });

    console.log('\n=== 基础使用示例完成 ===');

  } catch (error) {
    console.error('❌ 示例执行失败:', error);
    
    // 使用SDK错误格式化工具
    const formattedError = ArbitrageSystemSDK.formatError(error);
    console.error('格式化错误信息:', formattedError);
  } finally {
    // 清理资源
    console.log('\n清理资源...');
    sdk.disconnectWebSocket();
    console.log('👋 示例结束');
  }
}

// 运行示例
if (require.main === module) {
  basicUsageExample().catch(console.error);
}

export { basicUsageExample };