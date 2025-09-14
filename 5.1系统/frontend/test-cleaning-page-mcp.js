const { chromium } = require('playwright');

async function testCleaningPageWithMCP() {
  console.log('🚀 启动清洗页面MCP测试分析...');
  
  const browser = await chromium.launch({
    headless: false, // 显示浏览器界面以便观察
    slowMo: 1000    // 放慢操作速度以便观察
  });
  
  try {
    const context = await browser.newContext({
      viewport: { width: 1920, height: 1080 }
    });
    
    const page = await context.newPage();
    
    // 导航到清洗页面
    console.log('📍 导航到清洗页面...');
    await page.goto('http://57.183.21.242:3003/cleaning');
    
    // 等待页面加载
    await page.waitForSelector('#root', { timeout: 10000 });
    await page.waitForTimeout(3000);
    
    // 截取初始页面状态
    await page.screenshot({
      path: 'cleaning-page-initial.png',
      fullPage: true
    });
    console.log('📸 已保存初始页面截图');
    
    // 分析页面元素
    console.log('🔍 分析页面功能和元素...');
    
    // 检查页面标题
    const pageTitle = await page.textContent('h1');
    console.log(`页面标题: ${pageTitle}`);
    
    // 检查数据质量概览卡片
    const qualityCards = await page.locator('.ant-col .ant-card').count();
    console.log(`数据质量概览卡片数量: ${qualityCards}`);
    
    // 检查标签页
    const tabs = await page.locator('.ant-tabs-tab').allTextContents();
    console.log('可用标签页:', tabs);
    
    // 测试清洗规则标签页
    console.log('🧪 测试清洗规则功能...');
    if (tabs.some(tab => tab.includes('清洗规则'))) {
      await page.click('.ant-tabs-tab:has-text("清洗规则")');
      await page.waitForTimeout(2000);
      
      // 检查表格是否加载
      const hasTable = await page.locator('.ant-table-tbody').isVisible();
      console.log(`清洗规则表格是否显示: ${hasTable}`);
      
      // 测试搜索功能
      await page.fill('.ant-input[placeholder*="搜索清洗规则"]', 'test');
      await page.waitForTimeout(1000);
      
      // 测试刷新按钮
      await page.click('button:has-text("刷新")');
      await page.waitForTimeout(2000);
      
      // 截取清洗规则页面
      await page.screenshot({
        path: 'cleaning-rules-tab.png',
        fullPage: true
      });
    }
    
    // 测试交易所配置标签页
    console.log('🏦 测试交易所配置功能...');
    if (tabs.some(tab => tab.includes('交易所配置'))) {
      await page.click('.ant-tabs-tab:has-text("交易所配置")');
      await page.waitForTimeout(2000);
      
      const exchangeTable = await page.locator('.ant-table-tbody').isVisible();
      console.log(`交易所配置表格是否显示: ${exchangeTable}`);
      
      // 截取交易所配置页面
      await page.screenshot({
        path: 'cleaning-exchanges-tab.png',
        fullPage: true
      });
    }
    
    // 测试质量监控标签页
    console.log('📊 测试质量监控功能...');
    if (tabs.some(tab => tab.includes('质量监控'))) {
      await page.click('.ant-tabs-tab:has-text("质量监控")');
      await page.waitForTimeout(2000);
      
      // 检查质量指标卡片
      const qualityMetrics = await page.locator('.ant-card').count();
      console.log(`质量指标卡片数量: ${qualityMetrics}`);
      
      // 截取质量监控页面
      await page.screenshot({
        path: 'cleaning-quality-tab.png',
        fullPage: true
      });
    }
    
    // 测试SIMD优化标签页
    console.log('⚡ 测试SIMD优化功能...');
    if (tabs.some(tab => tab.includes('SIMD优化'))) {
      await page.click('.ant-tabs-tab:has-text("SIMD优化")');
      await page.waitForTimeout(2000);
      
      // 测试开关功能
      const switches = await page.locator('.ant-switch').count();
      console.log(`SIMD优化开关数量: ${switches}`);
      
      // 测试第一个开关
      if (switches > 0) {
        await page.click('.ant-switch').first();
        await page.waitForTimeout(1000);
      }
      
      // 截取SIMD优化页面
      await page.screenshot({
        path: 'cleaning-simd-tab.png',
        fullPage: true
      });
    }
    
    // 开发者工具检查
    console.log('🔧 打开开发者工具进行深度分析...');
    await page.keyboard.press('F12');
    await page.waitForTimeout(2000);
    
    // 检查控制台错误
    const consoleLogs = [];
    page.on('console', msg => {
      consoleLogs.push(`${msg.type()}: ${msg.text()}`);
    });
    
    // 检查网络请求
    const networkRequests = [];
    page.on('request', request => {
      if (request.url().includes('/cleaning/')) {
        networkRequests.push(request.url());
      }
    });
    
    // 刷新页面查看网络请求
    await page.reload();
    await page.waitForTimeout(5000);
    
    // 截取最终状态
    await page.screenshot({
      path: 'cleaning-page-with-devtools.png',
      fullPage: true
    });
    
    console.log('📝 生成功能分析报告...');
    const report = {
      timestamp: new Date().toISOString(),
      pageTitle: pageTitle,
      availableTabs: tabs,
      qualityCardsCount: qualityCards,
      consoleLogs: consoleLogs,
      networkRequests: networkRequests,
      screenshots: [
        'cleaning-page-initial.png',
        'cleaning-rules-tab.png', 
        'cleaning-exchanges-tab.png',
        'cleaning-quality-tab.png',
        'cleaning-simd-tab.png',
        'cleaning-page-with-devtools.png'
      ],
      functionalityStatus: {
        cleaningRules: tabs.some(tab => tab.includes('清洗规则')),
        exchangeConfig: tabs.some(tab => tab.includes('交易所配置')),
        qualityMonitoring: tabs.some(tab => tab.includes('质量监控')),
        simdOptimization: tabs.some(tab => tab.includes('SIMD优化'))
      },
      recommendedEnhancements: [
        '添加实时数据流显示',
        '增强数据可视化图表',
        '添加批量操作功能',
        '实现规则模板系统',
        '添加性能监控仪表板',
        '集成告警通知系统'
      ]
    };
    
    // 保存报告
    const fs = require('fs');
    fs.writeFileSync('cleaning-page-mcp-analysis-report.json', 
      JSON.stringify(report, null, 2));
    
    console.log('✅ 清洗页面MCP分析完成！');
    console.log('📄 报告已保存到: cleaning-page-mcp-analysis-report.json');
    console.log('📸 截图文件已保存');
    
    // 保持浏览器打开30秒以便观察
    console.log('🔍 保持浏览器打开30秒以便手动检查...');
    await page.waitForTimeout(30000);
    
  } catch (error) {
    console.error('❌ 测试过程中发生错误:', error);
  } finally {
    await browser.close();
    console.log('🏁 浏览器已关闭，测试完成');
  }
}

// 运行测试
testCleaningPageWithMCP().catch(console.error);