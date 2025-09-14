import { chromium } from 'playwright';

async function checkTradingPage() {
  const browser = await chromium.launch({ 
    headless: true, // headless模式
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔍 检查交易页面状态...');

    // 监听所有控制台输出和错误
    page.on('console', msg => {
      console.log(`Console [${msg.type()}]: ${msg.text()}`);
    });

    page.on('pageerror', error => {
      console.error(`Page Error: ${error.message}`);
    });

    // 监听网络请求
    page.on('response', response => {
      if (response.status() >= 400) {
        console.log(`Network Error: ${response.url()} - ${response.status()}`);
      }
    });

    await page.goto('http://57.183.21.242:3003/trading', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    await page.waitForTimeout(5000);

    const pageAnalysis = await page.evaluate(() => {
      return {
        title: document.title,
        hasContent: document.body.textContent.trim().length > 100,
        mainHeading: document.querySelector('h1, h2, .ant-typography-title')?.textContent,
        tabCount: document.querySelectorAll('.ant-tabs-tab').length,
        cardCount: document.querySelectorAll('.ant-card').length,
        tableCount: document.querySelectorAll('.ant-table').length,
        buttonCount: document.querySelectorAll('button').length,
        hasTrading: document.body.textContent.includes('交易') || document.body.textContent.includes('trading'),
        hasOrders: document.body.textContent.includes('订单') || document.body.textContent.includes('orders'),
        hasPositions: document.body.textContent.includes('仓位') || document.body.textContent.includes('positions'),
        hasFunds: document.body.textContent.includes('资金') || document.body.textContent.includes('funds'),
        elementCount: document.querySelectorAll('*').length,
        bodyPreview: document.body.textContent.slice(0, 500),
        errorElements: document.querySelectorAll('.ant-result-error, .ant-empty, [class*="error"]').length
      };
    });

    console.log('\n📊 页面分析结果:');
    console.log(`标题: ${pageAnalysis.title}`);
    console.log(`主标题: ${pageAnalysis.mainHeading}`);
    console.log(`有内容: ${pageAnalysis.hasContent ? '是' : '否'}`);
    console.log(`Tab数量: ${pageAnalysis.tabCount}`);
    console.log(`卡片数量: ${pageAnalysis.cardCount}`);
    console.log(`表格数量: ${pageAnalysis.tableCount}`);
    console.log(`按钮数量: ${pageAnalysis.buttonCount}`);
    console.log(`总元素数: ${pageAnalysis.elementCount}`);
    console.log(`错误元素数: ${pageAnalysis.errorElements}`);

    console.log('\n🔍 功能检查:');
    console.log(`交易管理: ${pageAnalysis.hasTrading ? '✅ 存在' : '❌ 缺失'}`);
    console.log(`订单监控: ${pageAnalysis.hasOrders ? '✅ 存在' : '❌ 缺失'}`);
    console.log(`仓位管理: ${pageAnalysis.hasPositions ? '✅ 存在' : '❌ 缺失'}`);
    console.log(`资金管理: ${pageAnalysis.hasFunds ? '✅ 存在' : '❌ 缺失'}`);

    console.log('\n📄 页面内容预览:');
    console.log(`"${pageAnalysis.bodyPreview}"`);

    // 检查是否有JavaScript错误
    const jsErrors = await page.evaluate(() => {
      return window.jsErrors || [];
    });

    if (jsErrors.length > 0) {
      console.log('\n🚨 JavaScript错误:');
      jsErrors.forEach(error => console.log(`  ${error}`));
    }

    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/current-trading-page.png',
      fullPage: true 
    });

    console.log('\n🎯 结论:');
    if (pageAnalysis.elementCount < 50 || !pageAnalysis.hasTrading) {
      console.log('❌ 交易页面功能不完整或存在白屏问题');
      console.log('🔧 建议检查：');
      console.log('   1. 组件导入是否正确');
      console.log('   2. API调用是否存在错误');
      console.log('   3. React渲染是否出现异常');
      console.log('   4. 控制台是否有错误信息');
    } else {
      console.log('✅ 交易页面基本结构正常');
    }

    // headless模式，不需要保持打开
    console.log('\n✅ 页面检查完成');

  } catch (error) {
    console.error('❌ 检查过程出错:', error.message);
    console.log('🔧 可能的原因：');
    console.log('   1. 页面加载超时');
    console.log('   2. React应用崩溃');
    console.log('   3. 网络连接问题');
    console.log('   4. 组件编译错误');
  } finally {
    await browser.close();
  }
}

checkTradingPage().catch(console.error);